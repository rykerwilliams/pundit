//! Bus end to end: the export sheet's Scoreboard picker (spec M), the copied
//! whole match and the `.srt` beside it.
//!
//! The sources here are H.264 + AAC in MP4 — the shape a copy can join —
//! rather than the WebM fixtures the rest of the harness uses, because what
//! these tests are about is the renderer the picker chooses.
//!
//! Layout per test: `<tmp>/config` holds the state file, `<tmp>/project` the
//! project (and, once a run starts, its `exports/`), `<tmp>/media` the
//! fixture game videos.

use std::path::{Path, PathBuf};

use pundit_app::bus::{Command, Event, ExportChoices, ExportRun, TargetState};
use pundit_core::cues::{cues_to_srt, scoreboard_cues};
use pundit_core::export::compilation_schedule;
use pundit_core::plan::{compilation_plan, ExportTarget, ScoreboardMode};
use pundit_core::project::{Project, Quality, Resolution, SourceRef};
use pundit_core::scoreboard::{MatchEventKind, ScoreboardConfig, ScoreboardContext, TeamConfig};
use pundit_core::store::{self, EXPORTS_DIRNAME};
use pundit_core::stroke::Rgba;
use pundit_harness::{add_clips, Harness};
use pundit_media::fixtures::{counter_video, ffprobe, CounterKind};
use tempfile::TempDir;
use uuid::Uuid;

/// Every fixture's frame rate, which is also the output's.
const FPS: u32 = 30;

/// The tags most of this file's projects carry: a kick-off early in the first
/// source and a home goal a second in, so the cue list has a score turning
/// over and a clock running.
const DEFAULT_TAGS: [(MatchEventKind, f64); 2] = [
    (MatchEventKind::StartStop, 0.2),
    (MatchEventKind::HomeGoal, 1.0),
];

/// This file's standard export choices, with the sheet's defaults for the
/// three switches: the sound carried and both outputs written.
///
/// **Written out rather than `..Default::default()`**, because
/// [`ExportChoices`] deliberately has no `Default`: both switches' correct
/// value is `true` and a derived one would be `false`, which would make every
/// test here assert the wrong baseline.
fn choices() -> ExportChoices {
    ExportChoices {
        resolution: Resolution::R720,
        quality: Quality::Low,
        scoreboard: None,
        mute_source: false,
        chapters: true,
        cues: true,
    }
}

/// A project of H.264 + AAC MP4 sources, with a scoreboard, a kick-off and a
/// home goal tagged, opened on a fresh bus.
struct Match {
    h: Harness,
    folder: PathBuf,
    /// The project as the tags left it.
    project: Project,
    /// Each source's video packet count, for the copy's own proof.
    packets: Vec<i64>,
    _tmp: TempDir,
}

impl Match {
    /// `sources` as `(name, width, height, frames)`. The kick-off is tagged
    /// early in the first source and a home goal half-way through it, so the
    /// cue list has a score turning over and a clock running.
    fn open(sources: &[(&str, u32, u32, u32)]) -> Self {
        Self::open_with(sources, &[])
    }

    /// [`Match::open`], with a clip of `clip_seconds[i]` on source 0 for each
    /// entry — a target of its own beside the whole match.
    fn open_with(sources: &[(&str, u32, u32, u32)], clip_seconds: &[f64]) -> Self {
        Self::open_tagged(sources, clip_seconds, &DEFAULT_TAGS)
    }

    /// [`Match::open_with`], with `tags` as the match events, each
    /// `(kind, source_seconds)` on source 0.
    ///
    /// The tags are a parameter for one reason: the chapters are the match's
    /// own moments, and a `.chapters.txt` needs three of them
    /// [`MIN_GAP_SECONDS`](pundit_core::chapters::MIN_GAP_SECONDS) apart,
    /// which [`DEFAULT_TAGS`] on a two-second fixture cannot give.
    fn open_tagged(
        sources: &[(&str, u32, u32, u32)],
        clip_seconds: &[f64],
        tags: &[(MatchEventKind, f64)],
    ) -> Self {
        gstreamer::init().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let folder = tmp.path().join("project");
        let media = tmp.path().join("media");
        for dir in [&folder, &media] {
            std::fs::create_dir(dir).unwrap();
        }

        let mut project = Project::new("Game");
        let mut packets = Vec::with_capacity(sources.len());
        for &(name, w, h, frames) in sources {
            let file = format!("{name}.mp4");
            let path = counter_video(
                &media.join(&file),
                w,
                h,
                FPS,
                frames,
                CounterKind::H264AacMp4,
            );
            packets.push(video_packets(&path));
            project.source_videos.push(SourceRef {
                relative_path: format!("../media/{file}"),
                display_name: name.into(),
                duration_seconds: f64::from(frames) / f64::from(FPS),
                display_aspect: f64::from(w) / f64::from(h),
            });
        }
        add_clips(&folder, &mut project, &vec![0; clip_seconds.len()]);
        for (clip, &secs) in project.clips.iter_mut().zip(clip_seconds) {
            clip.recording_duration = secs;
            clip.name = "Lesson".into();
        }
        store::write(&folder, &mut project).unwrap();

        let mut h = Harness::new(&tmp.path().join("config"));
        h.send(Command::OpenProject(folder.clone()));
        h.wait_opened();

        h.send(Command::SetScoreboard(ScoreboardConfig {
            home: TeamConfig::new("Rovers", Rgba::RED, Rgba::RED),
            away: TeamConfig::new("United", Rgba::RED, Rgba::RED),
            format: Default::default(),
            auto_back_anchor_p1: false,
        }));
        h.wait_changed();
        for &(kind, source_seconds) in tags {
            h.send(Command::TagMatchEvent {
                kind,
                source_index: 0,
                source_seconds,
            });
        }
        let project = tags
            .iter()
            .map(|_| h.wait_changed())
            .last()
            .unwrap()
            .project;

        Match {
            h,
            folder,
            project: (*project).clone(),
            packets,
            _tmp: tmp,
        }
    }

    fn export(&self, targets: Vec<ExportTarget>, scoreboard: Option<ScoreboardMode>) {
        self.export_with(
            targets,
            ExportChoices {
                scoreboard,
                ..choices()
            },
        );
    }

    /// As [`Match::export`], with every one of the sheet's controls as given.
    fn export_with(&self, targets: Vec<ExportTarget>, choices: ExportChoices) {
        self.h.send(Command::Export { targets, choices });
    }

    /// The run's last word: the event with nothing left running.
    fn outcome(&mut self) -> ExportRun {
        self.h.wait_map("the run's outcome", |e| match e {
            Event::Export(run) if !run.is_running() => Some(run.clone()),
            _ => None,
        })
    }

    fn exports(&self) -> PathBuf {
        self.folder.join(EXPORTS_DIRNAME)
    }

    fn clip(&self) -> Uuid {
        self.project.clips[0].id
    }

    /// The scoreboard cue list of `target`, as core builds it: what the
    /// sidecar must hold, byte for byte.
    fn srt(&self, target: &ExportTarget) -> String {
        let compilation = compilation_schedule(&self.project, target);
        let context = ScoreboardContext::for_project(&self.project).expect("a scoreboard is set");
        cues_to_srt(&scoreboard_cues(&compilation, &context))
    }
}

/// The files in `exports/`, sorted, `.part` files included. Empty when no run
/// has created the folder.
fn outputs(exports: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(exports)
        .into_iter()
        .flatten()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

/// `path`'s video packet count, as `ffprobe` reads it: the packets a decoder
/// hides, which is what says the copy is packet for packet.
fn video_packets(path: &Path) -> i64 {
    let probe = ffprobe(
        path,
        &["-select_streams", "v:0", "-show_streams", "-count_packets"],
    );
    let packets = &probe["streams"][0]["nb_read_packets"];
    packets
        .as_i64()
        .or_else(|| packets.as_str()?.parse().ok())
        .unwrap_or_else(|| panic!("no packet count in {probe}"))
}

/// How many streams of kind `select` (`"a"`, `"s"`) `path` has.
fn streams(path: &Path, select: &str) -> usize {
    ffprobe(path, &["-select_streams", select, "-show_streams"])["streams"]
        .as_array()
        .expect("a streams array")
        .len()
}

/// `path`'s video size, which is the copy/encode discriminator: the sources
/// here are 640x360 and the sheet sends `R720`, so a copy reads back the
/// source's own size and an encode 1280x720.
///
/// **The packet count cannot do this job.** An encode of a whole match runs at
/// `OUTPUT_FPS` over 30 fps sources, so it writes the *same* number of video
/// packets as the copy —
/// [`the_whole_match_is_copied_with_a_sidecar`]'s assertion proves the packets
/// came through whole, not that the renderer was the copy.
fn video_size(path: &Path) -> (i64, i64) {
    let probe = ffprobe(path, &["-select_streams", "v:0", "-show_streams"]);
    let stream = &probe["streams"][0];
    let side = |key| {
        stream[key]
            .as_i64()
            .or_else(|| stream[key].as_str()?.parse().ok())
            .unwrap_or_else(|| panic!("no {key} in {probe}"))
    };
    (side("width"), side("height"))
}

/// `path`'s chapters as `ffprobe` reads them: `(start in seconds, title)`.
///
/// The independent reader, because GStreamer's `qtdemux` doesn't read `chpl`
/// and no released Rust MP4 crate parses it — the same helper `reel.rs` uses
/// for the encoded path's chapters.
fn ffprobe_chapters(path: &Path) -> Vec<(f64, String)> {
    ffprobe(path, &["-show_chapters"])["chapters"]
        .as_array()
        .expect("a chapters array")
        .iter()
        .map(|c| {
            (
                c["start_time"].as_str().unwrap().parse().unwrap(),
                c["tags"]["title"].as_str().unwrap_or_default().to_owned(),
            )
        })
        .collect()
}

/// The whole match asked for on a separate track is copied, not re-encoded,
/// and the scoreboard lands beside it as core formats it.
#[test]
fn the_whole_match_is_copied_with_a_sidecar() {
    let mut m = Match::open(&[("first half", 640, 360, 60), ("second half", 640, 360, 45)]);
    let target = ExportTarget::WholeMatch;
    let plan = compilation_plan(&m.project, &target);
    let srt = m.srt(&target);

    m.export(vec![target], Some(ScoreboardMode::Track));
    let run = m.outcome();
    assert_eq!(run.targets[0].label, "Whole match");
    assert_eq!(run.targets[0].frames, plan.total_frames());
    let TargetState::Done(path) = &run.targets[0].state else {
        panic!("{:?}", run.targets[0]);
    };

    assert_eq!(
        outputs(&m.exports()),
        ["Whole match - Game.mp4", "Whole match - Game.srt"]
    );
    // Every source packet is in the output: a copy, not a re-encode, which
    // would have written 1280x720 frames of its own.
    assert_eq!(video_packets(path), m.packets.iter().sum::<i64>());
    // The sources' sound, carried — which is what the muted run below drops.
    assert_eq!(streams(path, "a"), 1);

    let written = std::fs::read_to_string(path.with_extension("srt")).unwrap();
    assert_eq!(written, srt);
    let first = written.lines().nth(2).expect("a first cue");
    assert!(
        first.starts_with("Rovers 0 - 0 United · "),
        "the first cue reads {first:?}"
    );
    m.h.shutdown();
}

/// "Mute source audio" on a copied whole match: still a copy, packet for
/// packet, with the audio track simply not carried (spec M2).
///
/// **The copy is the half of the mute the bus could get wrong**, because the
/// encode path expresses it as an empty region list in core and this one as a
/// pad the muxer never requests. The sidecar is unaffected — the scoreboard
/// is text, and muting is about sound.
#[test]
fn a_muted_whole_match_is_copied_without_its_sound() {
    let mut m = Match::open(&[("first half", 640, 360, 60), ("second half", 640, 360, 45)]);
    let target = ExportTarget::WholeMatch;

    m.export_with(
        vec![target],
        ExportChoices {
            scoreboard: Some(ScoreboardMode::Track),
            mute_source: true,
            ..choices()
        },
    );
    let run = m.outcome();
    let TargetState::Done(path) = &run.targets[0].state else {
        panic!("{:?}", run.targets[0]);
    };

    // A copy, not a re-encode: every source packet is still there.
    assert_eq!(video_packets(path), m.packets.iter().sum::<i64>());
    assert_eq!(
        streams(path, "a"),
        0,
        "the muted copy carried the sound it was told to drop"
    );
    // **Only** the sound: the board still rides inside the file on its own
    // `tx3g` track, which is the other pad track mode requests.
    assert_eq!(
        streams(path, "s"),
        1,
        "the mute took the scoreboard with it"
    );
    assert_eq!(
        outputs(&m.exports()),
        ["Whole match - Game.mp4", "Whole match - Game.srt"]
    );
    m.h.shutdown();
}

/// With the picker on Default the mode is the target's: the whole match is
/// copied and gets its sidecar, the clip is re-encoded and gets none.
#[test]
fn the_default_mode_copies_the_whole_match_and_burns_a_clip() {
    let mut m = Match::open_with(&[("first half", 640, 360, 30)], &[0.5]);
    m.export(
        vec![ExportTarget::WholeMatch, ExportTarget::Clip(m.clip())],
        None,
    );
    let run = m.outcome();
    for target in &run.targets {
        assert!(matches!(target.state, TargetState::Done(_)), "{target:#?}");
    }
    assert_eq!(
        outputs(&m.exports()),
        [
            "Lesson - Game.mp4",
            "Whole match - Game.mp4",
            "Whole match - Game.srt",
        ]
    );
    m.h.shutdown();
}

/// Only the whole match carries a cue list (spec T1), so a clip asked for on
/// a separate track **burns the board in instead**: it re-encodes either way,
/// for its drawings, its inset and its zoom, and the one thing the picker
/// must never do is lose the board altogether. Nothing lands beside it — a
/// clip already says what it is in its own text bar — and a `.srt` a coach
/// put there themselves is left alone.
#[test]
fn a_clip_on_a_separate_track_burns_the_board_in() {
    let mut m = Match::open_with(&[("first half", 640, 360, 30)], &[0.5]);
    // A subtitle file the coach wrote themselves, at the name this clip
    // exports to. No export of ours put it there, so none of ours removes it.
    std::fs::create_dir_all(m.exports()).unwrap();
    let theirs = m.exports().join("Lesson - Game.srt");
    std::fs::write(&theirs, "1\n00:00:00,000 --> 00:00:01,000\nmine\n\n").unwrap();
    m.export(
        vec![ExportTarget::Clip(m.clip())],
        Some(ScoreboardMode::Track),
    );
    let run = m.outcome();
    assert!(
        matches!(run.targets[0].state, TargetState::Done(_)),
        "{:#?}",
        run.targets[0]
    );
    assert_eq!(
        outputs(&m.exports()),
        ["Lesson - Game.mp4", "Lesson - Game.srt"]
    );
    assert!(
        std::fs::read_to_string(&theirs).unwrap().contains("mine"),
        "the export took the coach's own subtitle file"
    );
    m.h.shutdown();
}

/// Switching the picker back burns the board into the picture, and with the
/// subtitles **off** the sidecar the last run left goes with it — or the old
/// score plays over the new film.
///
/// **It is driven with the switch off, which is a rewrite rather than an
/// extension.** The removal is `write_sidecar`'s `Some(empty)` branch, and
/// with `last_export_cues` defaulting on it is no longer where a burned run
/// lands: a burned whole match now writes an `.srt` of its own
/// ([`a_burned_whole_match_writes_the_srt_beside_it`], the accepted cost of
/// independent switches). The switch is the one thing that still asks for
/// nothing beside the file, so it is what this test sends.
#[test]
fn a_burned_whole_match_removes_a_stale_sidecar() {
    let mut m = Match::open(&[("first half", 640, 360, 30)]);
    m.export(vec![ExportTarget::WholeMatch], Some(ScoreboardMode::Track));
    m.outcome();
    let sidecar = m.exports().join("Whole match - Game.srt");
    assert!(sidecar.exists(), "{:?}", outputs(&m.exports()));

    m.export_with(
        vec![ExportTarget::WholeMatch],
        ExportChoices {
            scoreboard: Some(ScoreboardMode::Burned),
            cues: false,
            ..choices()
        },
    );
    let run = m.outcome();
    assert!(
        matches!(run.targets[0].state, TargetState::Done(_)),
        "{:#?}",
        run.targets[0]
    );
    assert_eq!(outputs(&m.exports()), ["Whole match - Game.mp4"]);
    m.h.shutdown();
}

/// The board burned into the picture **and** an `.srt` beside the file: the
/// combination independent switches exist for, unreachable until now (spec
/// S4), and the cheapest proof that the subtitles switch is not the Scoreboard
/// picker in disguise.
///
/// Only the `.srt`: a burned export re-encodes, and the `tx3g` track rides the
/// copy alone, so there is no embedded track to disagree with the picture.
#[test]
fn a_burned_whole_match_writes_the_srt_beside_it() {
    let mut m = Match::open(&[("first half", 640, 360, 30)]);
    let target = ExportTarget::WholeMatch;
    let srt = m.srt(&target);

    m.export(vec![target], Some(ScoreboardMode::Burned));
    let run = m.outcome();
    let TargetState::Done(path) = &run.targets[0].state else {
        panic!("{:?}", run.targets[0]);
    };
    assert_eq!(
        outputs(&m.exports()),
        ["Whole match - Game.mp4", "Whole match - Game.srt"]
    );
    assert_eq!(
        std::fs::read_to_string(path.with_extension("srt")).unwrap(),
        srt
    );
    assert_eq!(
        streams(path, "s"),
        0,
        "an encode carried a subtitle track, which only the copy requests"
    );
    m.h.shutdown();
}

/// "Scoreboard subtitles" off on a copied whole match: no `.srt`, the stale
/// one removed, and **no subtitle track inside the file** — the half that
/// lives in `copy.rs`, and the one an implementation can get right beside the
/// file and wrong inside it (spec S1).
///
/// Still a copy: *Separate track* chosen by hand is the coach asking for one,
/// and the switch takes the board away rather than the renderer. That is §S4's
/// "no board anywhere", which the sheet says out loud before the run.
#[test]
fn the_subtitles_switch_off_leaves_a_copy_with_no_board() {
    let mut m = Match::open(&[("first half", 640, 360, 30)]);
    // On first, so there is a stale sidecar for the second run to remove.
    m.export(vec![ExportTarget::WholeMatch], Some(ScoreboardMode::Track));
    m.outcome();
    assert!(m.exports().join("Whole match - Game.srt").exists());

    m.export_with(
        vec![ExportTarget::WholeMatch],
        ExportChoices {
            scoreboard: Some(ScoreboardMode::Track),
            cues: false,
            ..choices()
        },
    );
    let run = m.outcome();
    let TargetState::Done(path) = &run.targets[0].state else {
        panic!("{:?}", run.targets[0]);
    };
    assert_eq!(outputs(&m.exports()), ["Whole match - Game.mp4"]);
    assert_eq!(
        streams(path, "s"),
        0,
        "the switch took the .srt but left the track inside the file"
    );
    // The renderer is untouched: the switch is about the board, not the copy.
    assert_eq!(video_size(path), (640, 360));
    m.h.shutdown();
}

/// **"Default" never trades the board away** (spec S4). With the subtitles off
/// a copy would carry no board anywhere, so Default burns it in instead —
/// which is the one behavioural inference in the spec, so it is pinned.
///
/// The discriminator is the **frame size**: the sources are 640x360 and the
/// sheet sends `R720`, so a copy reads back 640x360 and an encode 1280x720.
#[test]
fn default_with_the_subtitles_off_burns_the_board_in() {
    let mut m = Match::open(&[("first half", 640, 360, 30)]);
    m.export_with(
        vec![ExportTarget::WholeMatch],
        ExportChoices {
            scoreboard: None,
            cues: false,
            ..choices()
        },
    );
    let run = m.outcome();
    let TargetState::Done(path) = &run.targets[0].state else {
        panic!("{:?}", run.targets[0]);
    };
    assert_eq!(
        video_size(path),
        (1280, 720),
        "Default copied a film with no board on it"
    );
    assert_eq!(outputs(&m.exports()), ["Whole match - Game.mp4"]);
    m.h.shutdown();
}

/// The Chapters switch governs **both** forms, in one test, so the two halves
/// cannot drift: the `chpl` box inside the file and the `.chapters.txt` beside
/// it (spec S1).
///
/// **Thirty seconds of source and three tagged moments**, because no other rig
/// in the tree can produce a `.chapters.txt` at all: `chapter_list` needs
/// `MIN_CHAPTERS` (3) survivors `MIN_GAP_SECONDS` (10) apart. On the copy path
/// that is a header read and a packet copy; on the encoded path it would be
/// 900+ output frames through llvmpipe.
///
/// Run on and then off, because the pair is what proves the switch rather than
/// an absence — and the off run inherits the first's file, so it also pins the
/// stale `.chapters.txt` being removed.
#[test]
fn the_chapters_switch_governs_both_forms() {
    let mut m = Match::open_tagged(
        &[("first half", 640, 360, 900)],
        &[],
        &[
            (MatchEventKind::StartStop, 0.2),
            (MatchEventKind::HomeGoal, 12.0),
            (MatchEventKind::StartStop, 25.0),
        ],
    );

    m.export(vec![ExportTarget::WholeMatch], Some(ScoreboardMode::Track));
    let run = m.outcome();
    let TargetState::Done(path) = &run.targets[0].state else {
        panic!("{:?}", run.targets[0]);
    };
    assert_eq!(
        outputs(&m.exports()),
        [
            "Whole match - Game.chapters.txt",
            "Whole match - Game.mp4",
            "Whole match - Game.srt",
        ]
    );
    assert_eq!(
        std::fs::read_to_string(path.with_extension("chapters.txt")).unwrap(),
        "0:00 Kick-off\n0:12 Rovers goal 1-0\n0:25 Half time\n"
    );
    let titles: Vec<String> = ffprobe_chapters(path)
        .into_iter()
        .map(|(_, title)| title)
        .collect();
    assert_eq!(titles, ["Kick-off", "Rovers goal 1-0", "Half time"]);

    m.export_with(
        vec![ExportTarget::WholeMatch],
        ExportChoices {
            scoreboard: Some(ScoreboardMode::Track),
            chapters: false,
            ..choices()
        },
    );
    let run = m.outcome();
    let TargetState::Done(path) = &run.targets[0].state else {
        panic!("{:?}", run.targets[0]);
    };
    assert_eq!(
        outputs(&m.exports()),
        ["Whole match - Game.mp4", "Whole match - Game.srt"],
        "the chapter list the last run wrote is still there"
    );
    let inside = ffprobe_chapters(path);
    assert!(
        inside.is_empty(),
        "the chapters are still inside the file: {inside:?}"
    );
    m.h.shutdown();
}

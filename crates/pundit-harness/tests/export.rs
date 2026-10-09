//! Bus end to end: export runs (Phase 5 spec X4, Phase 8 specs E1, E5 and
//! E6) — the bus's own behavior. The files' contents are the media crate's
//! tests.
//!
//! Runs here are short (1 s, 30 frames a target) or cancelled early, and
//! exported at 720p: on CI they run on llvmpipe, at a fraction of a second
//! per frame.
//!
//! Layout per test: `<tmp>/config` holds the state file, `<tmp>/project` the
//! project (and, once a run starts, its `exports/`), `<tmp>/media` the
//! fixture game video.

use std::path::{Path, PathBuf};

use pundit_app::bus::{
    AppFiles, Command, Event, ExportChoices, ExportRun, QueueRow, RecordingStatus, TargetState,
    UserError,
};
use pundit_core::layout::scoreboard_rects;
use pundit_core::plan::ExportTarget;
use pundit_core::project::{Quality, Resolution};
use pundit_core::scoreboard::{MatchEventKind, MatchFormat, ScoreboardConfig, TeamConfig};
use pundit_core::store::{self, EXPORTS_DIRNAME, RECORDINGS_DIRNAME};
use pundit_core::stroke::Rgba;
use pundit_core::zoom::Zoom;
use pundit_harness::{add_clips, write_project, Harness, FRAME};
use pundit_media::fixtures;
use tempfile::TempDir;
use uuid::Uuid;

/// A team colour as `0xRRGGBB`.
fn rgb(c: u32) -> Rgba {
    Rgba {
        r: f64::from((c >> 16) as u8) / 255.0,
        g: f64::from((c >> 8) as u8) / 255.0,
        b: f64::from(c as u8) / 255.0,
        a: 1.0,
    }
}

/// A project called `Game` with a 2-second fixture video and one clip per
/// entry of `secs`, opened on a fresh bus.
///
/// Clip `i` is called `clip i` and carries the single tag `ti`, so every clip
/// is a target of its own beside All clips.
struct Rig {
    h: Harness,
    clips: Vec<Uuid>,
    /// `<project>/exports`, which no run has created yet.
    exports: PathBuf,
    tmp: TempDir,
}

impl Rig {
    /// Clip `i` lasts `secs[i]`: past the video's end it freezes on its last
    /// frame.
    fn open(secs: &[f64]) -> Self {
        Self::open_with(secs, |_, _| {})
    }

    /// [`Rig::open`], with `before_open` run on the project and media folders
    /// first.
    fn open_with(secs: &[f64], before_open: impl FnOnce(&Path, &Path)) -> Self {
        Self::open_full(&[("a.webm", 2)], 0, secs, before_open)
    }

    /// [`Rig::open`] over several source videos, with every clip on
    /// `source_index` — so a test can reach a video the player never loads.
    fn open_with_sources(videos: &[(&str, u32)], source_index: usize, secs: &[f64]) -> Self {
        Self::open_full(videos, source_index, secs, |_, _| {})
    }

    fn open_full(
        videos: &[(&str, u32)],
        source_index: usize,
        secs: &[f64],
        before_open: impl FnOnce(&Path, &Path),
    ) -> Self {
        gstreamer::init().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let folder = tmp.path().join("project");
        let media = tmp.path().join("media");
        for dir in [&folder, &media] {
            std::fs::create_dir(dir).unwrap();
        }
        let mut project = write_project(&folder, &media, videos);
        let clips = add_clips(&folder, &mut project, &vec![source_index; secs.len()])
            .iter()
            .map(|c| c.id)
            .collect();
        for (i, clip) in project.clips.iter_mut().enumerate() {
            clip.recording_duration = secs[i];
            clip.name = format!("clip {i}");
            clip.tags = vec![format!("t{i}")];
        }
        store::write(&folder, &mut project).unwrap();
        before_open(&folder, &media);

        let mut h = Harness::new(&tmp.path().join("config"));
        h.send(Command::OpenProject(folder.clone()));
        h.wait_opened();
        Rig {
            h,
            clips,
            exports: folder.join(EXPORTS_DIRNAME),
            tmp,
        }
    }

    /// Runs `targets`, smallest and cheapest: these render on llvmpipe.
    fn export(&self, targets: Vec<ExportTarget>) {
        self.export_with(targets, choices());
    }

    /// As [`Rig::export`], with the sheet's controls as given. A test that
    /// cares about one of them writes `ExportChoices { mute_source: true,
    /// ..choices() }` and names that one.
    fn export_with(&self, targets: Vec<ExportTarget>, choices: ExportChoices) {
        self.h.send(Command::Export { targets, choices });
    }

    fn tag(&self, n: usize) -> ExportTarget {
        ExportTarget::Tag(format!("t{n}"))
    }

    /// **A second project under the same `Harness`**, for the export queue
    /// (BACKLOG #77): one clip, its own `media2/` and its own `exports/`.
    /// Returns the folder and the clip's id.
    ///
    /// `Rig` is otherwise single-project — one folder, one `exports`, one
    /// `clips` — and the queue's whole point is spanning that, so this is the
    /// one piece of test scaffolding the feature needs. It uses
    /// `pundit_harness`'s own `write_project` / `add_clips`, so nothing in the
    /// library changes.
    ///
    /// **It does not open the project**; the test does, when it wants the
    /// switch to be the thing under test.
    fn second_project(&self, secs: f64) -> (PathBuf, Uuid) {
        let folder = self.tmp.path().join("project2");
        let media = self.tmp.path().join("media2");
        for dir in [&folder, &media] {
            std::fs::create_dir(dir).unwrap();
        }
        let mut project = write_project(&folder, &media, &[("b.webm", 2)]);
        project.name = "Away Game".into();
        let clips = add_clips(&folder, &mut project, &[0]);
        project.clips[0].recording_duration = secs;
        project.clips[0].name = "clip 0".into();
        project.clips[0].tags = vec!["t0".into()];
        store::write(&folder, &mut project).unwrap();
        (folder, clips[0].id)
    }
}

/// This file's standard export choices: the smallest and cheapest render —
/// these go through llvmpipe — the target's own scoreboard mode, the sound
/// carried, and both output switches on, which is what the sheet's defaults
/// send.
///
/// **Written out rather than `..Default::default()`**, because
/// [`ExportChoices`] deliberately has no `Default`: both switches' correct
/// value is `true` and a derived one would be `false`, so every test here
/// would silently be exporting with both outputs off.
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

/// The run's last word: the event with nothing left running.
fn outcome(h: &mut Harness) -> ExportRun {
    h.wait_map("the run's outcome", |e| match e {
        Event::Export(run) if !run.is_running() => Some(run.clone()),
        _ => None,
    })
}

fn no_export_events(rest: &[Event]) {
    assert!(
        !rest.iter().any(|e| matches!(e, Event::Export(_))),
        "{rest:#?}"
    );
}

/// Two targets, one after the other: the frames left only fall, every target
/// ends written, and the files are named as spec E6 says.
#[test]
fn a_run_renders_every_target_and_its_frames_only_fall() {
    let mut rig = Rig::open(&[1.0, 1.0]);
    rig.export(vec![rig.tag(0), ExportTarget::Clip(rig.clips[1])]);

    let first = rig.h.wait_export();
    let labels: Vec<&str> = first.targets.iter().map(|t| t.label.as_str()).collect();
    assert_eq!(labels, ["t0", "clip 1"]);
    assert_eq!(first.targets.iter().map(|t| t.frames).sum::<usize>(), 60);
    assert_eq!(first.remaining_frames(), 60);
    assert!(first.is_running());

    let mut left = first.remaining_frames();
    let done = loop {
        let run = rig.h.wait_export();
        let now = run.remaining_frames();
        assert!(now <= left, "{now} frames left after {left}");
        left = now;
        if !run.is_running() {
            break run;
        }
    };
    assert_eq!(done.remaining_frames(), 0);
    for target in &done.targets {
        assert!(matches!(target.state, TargetState::Done(_)), "{target:?}");
    }
    assert_eq!(
        outputs(&rig.exports),
        ["clip 1 - Game.mp4", "t0 - Game.mp4"]
    );
    rig.h.shutdown();
}

/// Two targets that would be called the same thing — a clip named after a tag
/// — get one file each: the label names the file (spec E6), so without the
/// suffix the second target would overwrite the first's output part-way
/// through the run.
#[test]
fn two_targets_with_the_same_name_write_two_files() {
    let mut rig = Rig::open_with(&[1.0, 1.0], |folder, _| {
        let mut project = store::read(folder).unwrap();
        project.clips[1].name = "t0".into();
        store::write(folder, &mut project).unwrap();
    });
    rig.export(vec![rig.tag(0), ExportTarget::Clip(rig.clips[1])]);

    let done = outcome(&mut rig.h);
    let labels: Vec<&str> = done.targets.iter().map(|t| t.label.as_str()).collect();
    assert_eq!(labels, ["t0", "t0 (2)"]);
    assert_eq!(
        outputs(&rig.exports),
        ["t0 (2) - Game.mp4", "t0 - Game.mp4"]
    );
    rig.h.shutdown();
}

/// A target that can't be written reports it and the run carries on: one
/// broken output must not cost the rest of an evening's exports.
#[test]
fn a_failed_target_does_not_stop_the_run() {
    let mut rig = Rig::open_with(&[1.0, 1.0], |folder, _| {
        // A directory where the first target's file goes. It renders, and the
        // move into place then fails -- a target failing, rather than the
        // run being refused before it starts.
        std::fs::create_dir_all(folder.join(EXPORTS_DIRNAME).join("t0 - Game.mp4")).unwrap();
    });
    rig.export(vec![rig.tag(0), rig.tag(1)]);

    let done = outcome(&mut rig.h);
    assert!(
        matches!(done.targets[0].state, TargetState::Failed(_)),
        "{:?}",
        done.targets[0]
    );
    assert!(
        matches!(done.targets[1].state, TargetState::Done(_)),
        "{:?}",
        done.targets[1]
    );
    assert!(rig.exports.join("t1 - Game.mp4").is_file());
    assert!(!rig.exports.join("t0 - Game.mp4.part").exists());
    rig.h.shutdown();
}

/// Cancel stops the run where it stands: the target still rendering loses its
/// file, the ones after it never start, and the one already written stays
/// (spec E5).
#[test]
fn cancel_leaves_the_targets_already_written_alone() {
    // The second target is 300 frames, so the cancel lands well inside it.
    let mut rig = Rig::open(&[1.0, 10.0]);
    rig.export(vec![rig.tag(0), rig.tag(1), ExportTarget::AllClips]);

    rig.h.wait_map("the first target's file", |e| match e {
        Event::Export(run) => matches!(run.targets[0].state, TargetState::Done(_)).then_some(()),
        _ => None,
    });
    rig.h.send(Command::CancelExport);

    let done = outcome(&mut rig.h);
    assert!(
        matches!(done.targets[0].state, TargetState::Done(_)),
        "{:?}",
        done.targets[0]
    );
    assert_eq!(done.targets[1].state, TargetState::Cancelled);
    assert_eq!(done.targets[2].state, TargetState::Cancelled);
    assert_eq!(outputs(&rig.exports), ["t0 - Game.mp4"]);
    rig.h.shutdown();
}

/// **A project opens while a run is going, and the run finishes** (#77 spec
/// §Q7). The coach's own answer, asked as that spec's one open question
/// (2026-10-07): *"yes you should be able to keep working"*.
///
/// **This replaces `a_project_open_is_refused_while_a_run_is_going` rather
/// than amending it**: that test asserted the refusal itself, down to its
/// `UserError::CantExport("an export is running")` text. The three things it
/// was really guarding — the refused open published nothing, pushed no recent
/// and wrote no `project.json` — now have to hold **of an open that
/// succeeds**, which is the opposite assertion over the same three facts.
///
/// What makes it safe is not that a run is cheap to interrupt: it is that a run
/// holds nothing of the open project. §Q7 audited every one of `commit`'s
/// eleven steps against a job that is rendering, and the one that carries it is
/// `entry_media`'s deliberate `stat` over `Bus::missing` — a job reads its own
/// files off the disk and consults no bus state. The old guard's stated reason
/// (*"`commit` empties that project's trash underneath it"*) was false: a job
/// never names a path under `.trash`, and what costs it its recording is the
/// clip **delete**, which is answered where the delete is.
/// **One second of clip, not ten, and the reason is CI.** This is the only test
/// in this file that waits for a whole run to *finish* with a project open in
/// the middle of it — the `&[10.0]` tests around it either cancel or only check
/// a refusal. Ten seconds is 300 frames, and on CI's four-core llvmpipe that
/// plus `commit`'s own `ensure_loaded` (the decode contention §Q7's table
/// calls a performance matter) overran the harness's 15 s. Thirty frames is
/// still far longer than the microseconds the bus needs to handle the
/// `OpenProject` queued behind `begin`'s event.
///
/// **The "mid-run" half is not vacuous and does not need its own assertion.**
/// `wait_map` scans from its cursor, which `wait_opened` left just past
/// `ProjectOpened` — so `outcome` can only match a terminal `Event::Export`
/// that came *after* the open. A run that finished first would time out here
/// rather than pass.
#[test]
fn a_project_opens_while_a_run_is_going() {
    let mut rig = Rig::open(&[1.0]);
    let elsewhere = rig.tmp.path().join("elsewhere");
    std::fs::create_dir(&elsewhere).unwrap();
    // `Rig::open_full` opens `<tmp>/project`; `commit` stored it canonical.
    let opened = rig.tmp.path().join("project").canonicalize().unwrap();

    rig.export(vec![ExportTarget::AllClips]);
    assert!(rig.h.wait_export().is_running());

    rig.h.send(Command::OpenProject(elsewhere.clone()));
    let snapshot = rig.h.wait_opened();
    assert_eq!(
        snapshot.folder.canonicalize().unwrap(),
        elsewhere.canonicalize().unwrap(),
        "the open landed somewhere else"
    );

    // And the run the coach walked away from still finishes, into the folder
    // of the project he left rather than the one he is now in.
    let done = outcome(&mut rig.h);
    assert_eq!(done.targets.len(), 1);
    assert!(
        matches!(done.targets[0].state, TargetState::Done(_)),
        "the run did not finish: {:?}",
        done.targets[0].state
    );
    assert_eq!(outputs(&rig.exports), ["All clips - Game.mp4"]);

    let config = rig.tmp.path().join("config");
    rig.h.shutdown();
    // The two halves of the old test, inverted: the open is a real open, so it
    // heads the recents list and leaves a project behind it.
    let recents = AppFiles::in_config_dir(&config).recent_projects();
    assert_eq!(
        recents,
        [elsewhere.canonicalize().unwrap(), opened],
        "the opened folder is not at the head of the recents"
    );
    assert!(
        elsewhere.join("project.json").exists(),
        "the open created no project"
    );
}

/// **A preview still refuses it**, which is the clause that remains (#77 spec
/// §Q7) and the one this file had no test for while `refuse_if_busy` answered
/// both.
///
/// Phase 7 spec P5's exclusivity is the reason, and the message is its own:
/// `UserError::CantOpen` rather than the `CantExport` a project open used to
/// borrow from an export's refusal. Tested **apart** from the run above on
/// purpose — a change that failed both would have broken the command rather
/// than proven either rule.
#[test]
fn a_project_open_is_refused_while_a_preview_is_open() {
    let mut rig = Rig::open(&[10.0]);
    let elsewhere = rig.tmp.path().join("elsewhere");
    std::fs::create_dir(&elsewhere).unwrap();

    rig.h.send(Command::OpenPreview(rig.clips[0]));
    assert_eq!(rig.h.wait_preview(), Some(rig.clips[0]));

    rig.h.send(Command::OpenProject(elsewhere.clone()));
    assert_eq!(rig.h.wait_for_error(), UserError::CantOpen);

    let config = rig.tmp.path().join("config");
    let rest = rig.h.shutdown();
    assert!(
        !rest.iter().any(|e| matches!(e, Event::ProjectOpened(_))),
        "the refused open published something: {rest:#?}"
    );
    let recents = AppFiles::in_config_dir(&config).recent_projects();
    assert_eq!(
        recents,
        [rig.tmp.path().join("project").canonicalize().unwrap()],
        "the refused folder was pushed"
    );
    assert!(
        !elsewhere.join("project.json").exists(),
        "the refused open wrote a project"
    );
}

/// One run at a time, and never alongside a recording.
#[test]
fn while_a_run_is_going_a_second_run_and_recording_are_refused() {
    let mut rig = Rig::open(&[10.0]);
    rig.export(vec![ExportTarget::AllClips]);
    rig.export(vec![ExportTarget::AllClips]);
    rig.h.send(Command::ToggleRecording {
        zoom: Zoom::IDENTITY,
        slate: None,
    });

    assert!(rig.h.wait_export().is_running());
    assert_eq!(
        rig.h.wait_for_error(),
        UserError::CantExport("an export is running".into())
    );
    assert_eq!(
        rig.h.wait_for_error(),
        UserError::CantRecord("an export is running")
    );
    rig.h.send(Command::CancelExport);
    assert_eq!(outcome(&mut rig.h).targets[0].state, TargetState::Cancelled);
    let rest = rig.h.shutdown();
    assert!(
        !rest.iter().any(|e| matches!(e, Event::Recording(_))),
        "{rest:#?}"
    );
    assert!(
        outputs(&rig.exports).is_empty(),
        "{:?}",
        outputs(&rig.exports)
    );
}

/// A missing game video is refused before anything renders, naming the clip:
/// media only warns about one and would export an hour of black.
#[test]
fn a_missing_game_video_is_refused_up_front_naming_the_clip() {
    let mut rig = Rig::open_with(&[1.0], |_, media| {
        std::fs::remove_file(media.join("a.webm")).unwrap();
    });
    rig.export(vec![ExportTarget::AllClips]);
    assert_eq!(
        rig.h.wait_for_error(),
        UserError::CantExport("clip 0's game video is missing; relink it first".into())
    );
    let rest = rig.h.shutdown();
    no_export_events(&rest);
    assert!(
        !rig.exports.exists(),
        "a refused run made {:?}",
        rig.exports
    );
}

/// And so is a missing commentary recording, which media would degrade to
/// silence and a black inset.
#[test]
fn a_missing_recording_is_refused_up_front_naming_the_clip() {
    let mut rig = Rig::open_with(&[1.0], |folder, _| {
        for entry in std::fs::read_dir(folder.join(RECORDINGS_DIRNAME)).unwrap() {
            std::fs::remove_file(entry.unwrap().path()).unwrap();
        }
    });
    rig.export(vec![ExportTarget::AllClips]);
    assert_eq!(
        rig.h.wait_for_error(),
        UserError::CantExport("clip 0's commentary recording is missing".into())
    );
    let rest = rig.h.shutdown();
    no_export_events(&rest);
    assert!(
        !rig.exports.exists(),
        "a refused run made {:?}",
        rig.exports
    );
}

/// A target with no clips, or no frames, or a clip that has been deleted:
/// each is refused by name rather than producing an empty file.
#[test]
fn a_target_with_nothing_in_it_is_refused() {
    let mut rig = Rig::open(&[0.0]);
    rig.export(vec![ExportTarget::AllClips]);
    assert_eq!(
        rig.h.wait_for_error(),
        UserError::CantExport("All clips has nothing to export".into())
    );

    rig.export(vec![ExportTarget::Tag("nobody".into())]);
    assert_eq!(
        rig.h.wait_for_error(),
        UserError::CantExport("nobody has nothing to export".into())
    );

    rig.export(vec![ExportTarget::Clip(Uuid::new_v4())]);
    assert_eq!(
        rig.h.wait_for_error(),
        UserError::CantExport("the clip is gone".into())
    );

    let rest = rig.h.shutdown();
    no_export_events(&rest);
    assert!(
        !rig.exports.exists(),
        "a refused run made {:?}",
        rig.exports
    );
}

/// The pickers' values and the three switches become the project's, so the
/// next run uses them without being told (spec E4, S1).
///
/// **The mute is run second, and unmuted, on purpose.** `Rig::export` sends
/// 720p / Low, which already differ from a fresh project's 1080p / Medium, so
/// a run that *adds* the mute writes the project back whatever `Pickers::of`
/// read — and an assertion on it would pass even if `of` never looked at the
/// field. The second run below changes **nothing but the mute**, in the
/// direction the stored value has to be read to notice: a muted project asked
/// for an unmuted run. That is what pins `Pickers::of` for the mute; the two
/// output switches are pinned in `bus/export.rs`'s own unit test, where it
/// costs an `assert_eq!` rather than three more renders.
#[test]
fn a_run_persists_the_resolution_quality_and_switches() {
    let mut rig = Rig::open(&[1.0]);
    rig.export_with(
        vec![ExportTarget::AllClips],
        ExportChoices {
            mute_source: true,
            chapters: false,
            cues: false,
            ..choices()
        },
    );
    let snapshot = rig.h.wait_changed();
    let prefs = &snapshot.project.preferences;
    assert_eq!(prefs.last_export_resolution, Resolution::R720);
    assert_eq!(prefs.last_export_quality, Quality::Low);
    assert_eq!(prefs.export_source_volume, 0.0);
    assert!(!prefs.last_export_chapters);
    assert!(!prefs.last_export_cues);

    outcome(&mut rig.h);
    let folder = rig.tmp.path().join("project");
    let saved = store::read(&folder).unwrap();
    assert_eq!(saved.preferences.last_export_resolution, Resolution::R720);
    assert_eq!(saved.preferences.export_source_volume, 0.0);
    assert!(!saved.preferences.last_export_chapters);
    assert!(!saved.preferences.last_export_cues);

    // Everything the sheet sends now matches what is stored but the mute.
    rig.export_with(
        vec![ExportTarget::AllClips],
        ExportChoices {
            chapters: false,
            cues: false,
            ..choices()
        },
    );
    let snapshot = rig.h.wait_changed();
    assert_eq!(snapshot.project.preferences.export_source_volume, 1.0);
    outcome(&mut rig.h);
    assert_eq!(
        store::read(&folder)
            .unwrap()
            .preferences
            .export_source_volume,
        1.0,
        "unmuting a muted project left it muted"
    );
    rig.h.shutdown();
}

/// The recording guard drops it: the UI greys the menu item out, so it's
/// only reached through a UI bug.
#[test]
fn an_export_while_recording_is_dropped() {
    let mut rig = Rig::open(&[1.0]);
    rig.h.poll_until("settled at the start", |h| {
        let settled = h.log().iter().rev().find_map(|e| match e {
            Event::Position { target_abs, .. } => Some(target_abs.is_none()),
            _ => None,
        });
        settled == Some(true) && h.position_secs().is_some_and(|p| p.abs() < FRAME)
    });
    rig.h.send(Command::ToggleRecording {
        zoom: Zoom::IDENTITY,
        slate: None,
    });
    assert_eq!(rig.h.wait_recording(), RecordingStatus::Starting);

    rig.export(vec![ExportTarget::AllClips]);
    rig.h.send(Command::StopRecording);
    let rest = rig.h.shutdown();
    no_export_events(&rest);
    assert!(
        !rest.iter().any(|e| matches!(e, Event::Error(_))),
        "{rest:#?}"
    );
    assert!(
        !rig.exports.exists(),
        "a dropped run made {:?}",
        rig.exports
    );
}

/// The `ScoreboardContext` the bus builds from the open project reaches the
/// export driver, so the board is burned into the file (Phase 9 spec S2).
/// What it draws, and the clock it reads, are the media crate's tests and
/// core's; this pins the wiring, which was `None` until the bus filled it in.
#[test]
fn an_export_carries_the_projects_scoreboard() {
    let mut rig = Rig::open_with(&[0.3], |folder, _| {
        let mut project = store::read(folder).unwrap();
        project.scoreboard = Some(ScoreboardConfig {
            home: TeamConfig::new("HOME", rgb(0x00ff00), rgb(0xffffff)),
            away: TeamConfig::new("AWAY", rgb(0x0000ff), rgb(0xffffff)),
            format: MatchFormat::default(),
            auto_back_anchor_p1: false,
        });
        // Kick-off at the top of the game video, so every exported frame is
        // inside the first half and the board has a clock to show.
        project.append_match_event(MatchEventKind::StartStop, 0, 0.0);
        store::write(folder, &mut project).unwrap();
    });
    rig.export(vec![rig.tag(0)]);

    let done = outcome(&mut rig.h);
    assert!(
        matches!(done.targets[0].state, TargetState::Done(_)),
        "{:?}",
        done.targets[0]
    );
    rig.h.shutdown();

    // `Rig::export` renders at 720p.
    let rects = scoreboard_rects(1280.0, 720.0);
    let frames = fixtures::decode_rgb(&rig.exports.join("t0 - Game.mp4"));
    let frame = frames.last().expect("frames out");
    for (what, cell, expected) in [
        ("the home cell", &rects.home, [0x00u8, 0xff, 0x00]),
        ("the away cell", &rects.away, [0x00u8, 0x00, 0xff]),
    ] {
        // A corner of the cell, clear of its centred name.
        let (x, y) = ((cell.x + 4.0) as usize, (cell.y + cell.h - 4.0) as usize);
        let actual = frame.at(x, y);
        let off = (0..3).any(|c| (i32::from(actual[c]) - i32::from(expected[c])).abs() > 40);
        assert!(!off, "{what} at ({x}, {y}): got {actual:?}");
    }
}

/// A game video deleted since the project opened is refused too: the check is
/// a `stat` at Start, not the `missing` flags cached at open — which say
/// nothing about a file that vanished while the project sat there, and nothing
/// at all about the closed projects a basket reaches into (basket spec V2).
///
/// The clip is on the **second** source, which the player never loads, so
/// nothing refreshes the flags behind the test's back.
#[test]
fn a_game_video_deleted_since_the_open_is_still_refused() {
    let mut rig = Rig::open_with_sources(&[("a.webm", 2), ("b.webm", 2)], 1, &[1.0]);
    std::fs::remove_file(rig.tmp.path().join("media").join("b.webm")).unwrap();
    rig.export(vec![ExportTarget::AllClips]);
    assert_eq!(
        rig.h.wait_for_error(),
        UserError::CantExport("clip 0's game video is missing; relink it first".into())
    );
    let rest = rig.h.shutdown();
    no_export_events(&rest);
    assert!(
        !rig.exports.exists(),
        "a refused run made {:?}",
        rig.exports
    );
}

/// The pickers become the project's only once a run has actually begun (basket
/// spec C3): a refused Start leaves `Preferences` as they were rather than
/// dirtying the project and saving it.
#[test]
fn a_refused_run_leaves_the_pickers_alone() {
    let mut rig = Rig::open_with(&[1.0], |_, media| {
        std::fs::remove_file(media.join("a.webm")).unwrap();
    });
    // The sheet's pickers are 720p / Low, the mute is on and both output
    // switches are off; the project has never exported, so its own are the
    // defaults. **Every one of the three is sent away from its stored value**,
    // or asserting that the stored volume is still `1.0` and both switches
    // still `true` says nothing.
    rig.export_with(
        vec![ExportTarget::AllClips],
        ExportChoices {
            mute_source: true,
            chapters: false,
            cues: false,
            ..choices()
        },
    );
    rig.h.wait_for_error();
    let rest = rig.h.shutdown();
    assert!(
        !rest.iter().any(|e| matches!(e, Event::ProjectChanged(_))),
        "{rest:#?}"
    );
    let saved = store::read(&rig.tmp.path().join("project")).unwrap();
    assert_eq!(
        saved.preferences.last_export_resolution,
        Resolution::default()
    );
    assert_eq!(saved.preferences.last_export_quality, Quality::default());
    assert_eq!(saved.preferences.export_source_volume, 1.0);
    assert!(saved.preferences.last_export_chapters);
    assert!(saved.preferences.last_export_cues);
}

/// What the queue holds, resolved: the next `Event::Queue`.
///
/// Local, like [`outcome`], rather than a `Harness` method — the queue is one
/// feature's event and `pundit_harness`'s lib needs nothing for it.
fn queue(h: &mut Harness) -> Vec<QueueRow> {
    h.wait_map("the queue", |e| match e {
        Event::Queue(rows) => Some(rows.clone()),
        _ => None,
    })
}

/// **The whole feature: two projects' exports, one run** (BACKLOG #77). Enqueue
/// in project 1, open project 2, enqueue there, press Start once and walk away.
///
/// It also carries the two things the plan folded in here rather than paying
/// for a render each:
/// - **an enqueue during a run is allowed**, which is spec §Q8's one documented
///   divergence from the basket and §Q11's stated behaviour, and was otherwise
///   untested — `Start queue` is what waits, not the enqueue;
/// - **a project opens while the queue renders**, the cross-project half of
///   `a_project_opens_while_a_run_is_going`.
///
/// One second of clip per target, for that test's own reason: this waits for a
/// whole run to finish and CI renders on llvmpipe.
#[test]
fn a_queue_runs_several_projects_jobs_as_one_run() {
    let mut rig = Rig::open(&[1.0, 1.0]);
    let first = rig.tmp.path().join("project").canonicalize().unwrap();
    let (second, _) = rig.second_project(1.0);

    // Two from the project that is open.
    rig.h.send(Command::EnqueueExport {
        targets: vec![rig.tag(0), rig.tag(1)],
        choices: choices(),
    });
    let rows = queue(&mut rig.h);
    assert_eq!(
        rows.iter().map(|r| r.label.as_str()).collect::<Vec<_>>(),
        ["t0", "t1"],
        "the rows are not the two targets, in order"
    );
    assert!(
        rows.iter().all(|r| r.match_label == "Game"),
        "a row does not name its match: {rows:#?}"
    );

    // Then one from another project, which is the point.
    rig.h.send(Command::OpenProject(second.clone()));
    rig.h.wait_opened();
    rig.h.send(Command::EnqueueExport {
        targets: vec![ExportTarget::AllClips],
        choices: choices(),
    });
    let rows = queue(&mut rig.h);
    assert_eq!(rows.len(), 3, "the first project's rows did not survive");
    assert_eq!(
        rows[2].match_label, "Away Game",
        "the third row names the wrong match"
    );

    rig.h.send(Command::StartQueue);
    assert!(rig.h.wait_export().is_running());
    assert!(
        queue(&mut rig.h).is_empty(),
        "Start left the rows in the queue"
    );

    // **An enqueue during a run is allowed**: it renders nothing, so it lands
    // and waits for the next Start rather than being refused.
    rig.h.send(Command::EnqueueExport {
        targets: vec![ExportTarget::AllClips],
        choices: choices(),
    });
    assert_eq!(
        queue(&mut rig.h).len(),
        1,
        "an enqueue during a run was refused or lost"
    );

    // **And a project opens while it renders**, for #77 task 1's reason.
    rig.h.send(Command::OpenProject(first.clone()));
    rig.h.wait_opened();

    let done = outcome(&mut rig.h);
    assert_eq!(done.targets.len(), 3, "the run is not the three jobs");
    assert!(
        done.targets
            .iter()
            .all(|t| matches!(t.state, TargetState::Done(_))),
        "a job did not finish: {:?}",
        done.targets
    );
    // Each row says which match it came from, so three "All clips" from three
    // projects would still read apart (spec §Q3).
    assert_eq!(
        done.targets
            .iter()
            .map(|t| t.label.as_str())
            .collect::<Vec<_>>(),
        ["t0 — Game", "t1 — Game", "All clips — Away Game"]
    );

    // **Three files, in two `exports/` folders** — which is the feature.
    assert_eq!(outputs(&rig.exports), ["t0 - Game.mp4", "t1 - Game.mp4"]);
    assert_eq!(
        outputs(&second.join(EXPORTS_DIRNAME)),
        ["All clips - Away Game.mp4"]
    );
    rig.h.shutdown();
}

/// **An enqueue makes every refusal a run makes from the open project, and
/// starts nothing** (spec §Q6) — that is the whole reason the refusals moved to
/// enqueue: the coach is standing in that project and can fix it, where
/// refusing at Start would name a clip in a project he left hours ago.
///
/// **And the duplicate refusal is keyed on the target, not on the output
/// path.** Two *different* targets of one project can share a name — a clip
/// named after a tag, which `two_targets_with_the_same_name_write_two_files`
/// exists for — and across two clicks they reach the same path. A path key
/// would refuse the second as "already in the queue" when it is not in the
/// queue at all, with no way out. Both enqueue here, and `jobs`' seeded
/// `de_duplicate` is what keeps their files apart.
#[test]
fn enqueue_refuses_what_a_run_refuses_and_starts_nothing() {
    let mut rig = Rig::open_with(&[1.0], |folder, _| {
        // A clip named after the tag the other target is, so the two labels
        // collide exactly as the suffix test's do.
        let mut project = store::read(folder).unwrap();
        project.clips[0].name = "t0".into();
        store::write(folder, &mut project).unwrap();
    });

    // Nothing ticked.
    rig.h.send(Command::EnqueueExport {
        targets: vec![],
        choices: choices(),
    });
    assert_eq!(
        rig.h.wait_for_error(),
        UserError::CantExport("nothing is ticked".into())
    );

    // A tag no clip carries.
    rig.h.send(Command::EnqueueExport {
        targets: vec![ExportTarget::Tag("nobody".into())],
        choices: choices(),
    });
    assert_eq!(
        rig.h.wait_for_error(),
        UserError::CantExport("nobody has nothing to export".into())
    );

    // The same target twice: refused by identity, as a notice.
    rig.h.send(Command::EnqueueExport {
        targets: vec![rig.tag(0)],
        choices: choices(),
    });
    assert_eq!(queue(&mut rig.h).len(), 1);
    rig.h.send(Command::EnqueueExport {
        targets: vec![rig.tag(0)],
        choices: choices(),
    });
    let refusal = rig.h.wait_for_error();
    assert!(
        matches!(&refusal, UserError::Queue(why) if why.contains("already in the queue")),
        "{refusal:?}"
    );
    assert!(refusal.is_notice(), "a duplicate should not be a modal");

    // **A different target with the same name goes in**, and gets its own file
    // name rather than the first one's.
    rig.h.send(Command::EnqueueExport {
        targets: vec![ExportTarget::Clip(rig.clips[0])],
        choices: choices(),
    });
    let rows = queue(&mut rig.h);
    assert_eq!(
        rows.iter().map(|r| r.label.as_str()).collect::<Vec<_>>(),
        ["t0", "t0 (2)"],
        "the second target was refused, or would overwrite the first's file"
    );

    let rest = rig.h.shutdown();
    no_export_events(&rest);
}

/// **A queued job whose game video went fails its own row, and the jobs behind
/// it still write** (spec §Q6, §Q4): the one check that moves from a refusal to
/// a failure, because nothing true at Start is still true when a job's turn
/// comes.
#[test]
fn a_queued_job_whose_video_went_fails_its_own_row() {
    let mut rig = Rig::open(&[1.0, 1.0]);
    let (second, _) = rig.second_project(1.0);

    rig.h.send(Command::EnqueueExport {
        targets: vec![rig.tag(0)],
        choices: choices(),
    });
    queue(&mut rig.h);
    rig.h.send(Command::OpenProject(second.clone()));
    rig.h.wait_opened();
    rig.h.send(Command::EnqueueExport {
        targets: vec![ExportTarget::AllClips],
        choices: choices(),
    });
    assert_eq!(queue(&mut rig.h).len(), 2);

    // Gone **after** the enqueue, which is why no refusal can catch it.
    std::fs::remove_file(rig.tmp.path().join("media").join("a.webm")).unwrap();

    rig.h.send(Command::StartQueue);
    let done = outcome(&mut rig.h);
    assert!(
        matches!(done.targets[0].state, TargetState::Failed(_)),
        "the job with no video did not fail: {:?}",
        done.targets[0].state
    );
    assert!(
        matches!(done.targets[1].state, TargetState::Done(_)),
        "the job behind it did not write: {:?}",
        done.targets[1].state
    );
    assert_eq!(
        outputs(&second.join(EXPORTS_DIRNAME)),
        ["All clips - Away Game.mp4"]
    );
    rig.h.shutdown();
}

/// **Deleting a clip drops the queued jobs that needed it** (spec §Q6), and
/// leaves the ones that did not.
///
/// A job holds the clip's `recordings/<file>`, which the delete renames into
/// `.trash` — so from that moment the inset cannot be read and the entry falls
/// back to the GL filler. The job would still write a film, with the coach's
/// commentary silently gone and its text bar and chapter still in place, and a
/// silent quality loss is worse than a failure.
///
/// **Driven through real jobs rather than a hand-built `Render`**, which is a
/// deviation from the plan and a deliberate one: what can actually break is
/// `job()` ceasing to populate `ClipMedia`, and a fixture assembled in the test
/// would pin the fixture (`fit_window.rs`'s header is this repo's record of
/// that trap). The whole-match row is the "no clip" half — its plan entries
/// carry none — and `Render::Copy`'s own arm needs no test, because a copy
/// holds no `Encode` at all and so is not a holder of the file by construction.
#[test]
fn deleting_a_clip_drops_the_queued_jobs_that_needed_it() {
    let mut rig = Rig::open(&[1.0, 1.0]);

    rig.h.send(Command::EnqueueExport {
        targets: vec![
            ExportTarget::Clip(rig.clips[0]),
            ExportTarget::Clip(rig.clips[1]),
            ExportTarget::WholeMatch,
        ],
        choices: choices(),
    });
    assert_eq!(queue(&mut rig.h).len(), 3);

    rig.h.send(Command::DeleteClip(rig.clips[0]));
    // **The queue is published before the notice**, because the drop is what
    // the notice is about. Read in that order: `wait_for_error` advances the
    // cursor past everything before the error, so taking the notice first
    // would leave the `Event::Queue` behind it.
    let rows = queue(&mut rig.h);
    assert_eq!(
        rows.iter().map(|r| r.label.as_str()).collect::<Vec<_>>(),
        ["clip 1", "Whole match"],
        "the wrong jobs were dropped"
    );
    let notice = rig.h.wait_for_error();
    assert!(
        matches!(&notice, UserError::Queue(why) if why.contains("dropped 1 queued export")),
        "{notice:?}"
    );
    assert!(
        notice.is_notice(),
        "it must not be a modal over his project"
    );

    // **A delete that does not happen drops nothing**, which is why the drop
    // sits below `remove_clip` rather than beside the transcription and preview
    // calls above it. `DeleteClip` is a public command with no gate — the same
    // reason `restore_last_project` is guarded — so a clip id belonging to
    // *another* project reaches here and `remove_clip` answers `None`. Above
    // the delete that would destroy a queued job for a clip nobody deleted,
    // and the queue has no undo where a cancelled transcription and a closed
    // preview are both recoverable.
    let (second, elsewhere_clip) = rig.second_project(1.0);
    rig.h.send(Command::OpenProject(second.clone()));
    rig.h.wait_opened();
    rig.h.send(Command::EnqueueExport {
        targets: vec![ExportTarget::Clip(elsewhere_clip)],
        choices: choices(),
    });
    assert_eq!(queue(&mut rig.h).len(), 3, "the enqueue did not land");

    rig.h.send(Command::OpenProject(
        rig.tmp.path().join("project").canonicalize().unwrap(),
    ));
    rig.h.wait_opened();
    rig.h.send(Command::DeleteClip(elsewhere_clip));
    // Nothing to wait *for*, so the proof is the next queue event: a
    // `publish_queue` from the bogus delete would be found here with two rows
    // where the clear's is empty.
    rig.h.send(Command::ClearQueue);
    assert!(
        queue(&mut rig.h).is_empty(),
        "the bogus delete published a queue of its own"
    );
    rig.h.shutdown();
}

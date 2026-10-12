//! Bus end to end: the tactics whiteboard (BACKLOG #138) — arming a diagram,
//! recording over it, the keys a whiteboard take refuses, and a whiteboard
//! clip previewing and exporting with its game video gone.
//!
//! Layout per test: `<tmp>/config` holds the state file, `<tmp>/project` the
//! project with its `recordings/` and `backdrops/`, `<tmp>/media` the fixture
//! game video and the diagram the coach picks.

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use pundit_app::bus::{
    AppFiles, CaptureKind, Command, Event, ExportChoices, RecordingStatus, Take, TargetState,
    UserError,
};
use pundit_core::event::EventKind;
use pundit_core::highlight::NormRect;
use pundit_core::plan::ExportTarget;
use pundit_core::project::{Clip, Quality, Resolution};
use pundit_core::scoreboard::MatchEventKind;
use pundit_core::store::{self, BACKDROPS_DIRNAME, RECORDINGS_DIRNAME};
use pundit_core::stroke::{Rgba, Stroke, StrokeEnd, StrokePoint};
use pundit_core::zoom::Zoom;
use pundit_harness::{write_project, Harness, FRAME};
use pundit_media::fixtures::{self, StillFormat};
use pundit_media::now_ns;
use tempfile::TempDir;
use uuid::Uuid;

/// A camera slow enough to warm up that a test can cancel before its first
/// frame — the abort path.
const SLOW_CAMERA: CaptureKind = CaptureKind::Test {
    video_delay: Duration::from_secs(2),
};

/// A project of one 4-second fixture video, opened on a fresh bus that has
/// settled, with a 4:3 diagram on disk and — after [`Rig::arm`] — armed.
struct Rig {
    h: Harness,
    folder: PathBuf,
    media: PathBuf,
    config: PathBuf,
    /// The coach's own file, outside the project.
    diagram: PathBuf,
    /// Held so the scratch directory outlives the paths above.
    _tmp: TempDir,
}

impl Rig {
    fn open() -> Self {
        Self::open_with(CaptureKind::Test {
            video_delay: Duration::ZERO,
        })
    }

    fn open_with(capture: CaptureKind) -> Self {
        gstreamer::init().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let folder = tmp.path().join("project");
        let media = tmp.path().join("media");
        std::fs::create_dir(&folder).unwrap();
        std::fs::create_dir(&media).unwrap();
        write_project(&folder, &media, &[("a.webm", 4)]);
        let diagram = fixtures::still_image(&media, "pitch.PNG", 320, 240, StillFormat::Png);

        let config = tmp.path().join("config");
        let mut h = Harness::with_capture(&config, capture);
        h.send(Command::OpenProject(folder.clone()));
        h.wait_opened();
        h.wait_settled();
        Rig {
            h,
            folder,
            media,
            config,
            diagram,
            _tmp: tmp,
        }
    }

    /// Arms the rig's diagram and waits for the bus to say so.
    fn arm(&mut self) {
        self.h.send(Command::ArmBackdrop(self.diagram.clone()));
        assert_eq!(wait_backdrop(&mut self.h), Some(self.diagram.clone()));
    }

    fn whiteboard(&self) {
        self.h.send(Command::ToggleRecording {
            zoom: Zoom::IDENTITY,
            take: Take::Whiteboard,
        });
    }

    /// Starts a whiteboard take and waits for its first buffer.
    fn record(&mut self) {
        self.whiteboard();
        assert_eq!(self.h.wait_recording(), RecordingStatus::Starting);
        assert!(matches!(
            self.h.wait_recording(),
            RecordingStatus::Recording { .. }
        ));
    }

    /// Stops the take and returns the clip it made.
    fn stop(&mut self) -> Clip {
        self.h.send(Command::StopRecording);
        let changed = self.h.wait_changed();
        assert_eq!(self.h.wait_recording(), RecordingStatus::Idle);
        changed
            .project
            .clips
            .last()
            .expect("the take made a clip")
            .clone()
    }

    fn backdrops(&self) -> PathBuf {
        self.folder.join(BACKDROPS_DIRNAME)
    }
}

fn wait_backdrop(h: &mut Harness) -> Option<PathBuf> {
    h.wait_map("the armed diagram", |e| match e {
        Event::Backdrop(path) => Some(path.clone()),
        _ => None,
    })
}

/// The files in `dir`, sorted; none if it doesn't exist.
fn files(dir: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = match std::fs::read_dir(dir) {
        Ok(entries) => entries.map(|e| e.unwrap().path()).collect(),
        Err(_) => Vec::new(),
    };
    found.sort();
    found
}

fn the_notice(h: &mut Harness) -> String {
    match h.wait_for_error() {
        e @ UserError::Whiteboard(_) => {
            // A modal here could land over a live take and swallow the
            // transport keys: only `is_notice` stops that, and it is a
            // hand-written list a new variant can be left out of.
            assert!(e.is_notice(), "{e:?} is shown as a modal");
            e.to_string()
        }
        e => panic!("expected a whiteboard notice, got {e:?}"),
    }
}

/// The armed path is machine state: it reaches `state.json`, and disarming
/// takes it out again.
#[test]
fn arming_is_remembered_machine_wide_and_disarming_forgets() {
    let mut rig = Rig::open();
    rig.arm();
    assert_eq!(
        AppFiles::in_config_dir(&rig.config).backdrop(),
        Some(rig.diagram.clone())
    );
    rig.h.send(Command::DisarmBackdrop);
    assert_eq!(wait_backdrop(&mut rig.h), None);
    assert_eq!(AppFiles::in_config_dir(&rig.config).backdrop(), None);
    rig.h.shutdown();
}

/// Decoded at the pick: a file that isn't an image is refused, as a notice,
/// and the diagram armed before it stays armed.
#[test]
fn a_file_that_will_not_decode_is_refused_and_the_arm_is_kept() {
    let mut rig = Rig::open();
    rig.arm();
    let junk = rig.media.join("notes.png");
    std::fs::write(&junk, b"not a picture").unwrap();
    rig.h.send(Command::ArmBackdrop(junk));
    the_notice(&mut rig.h);
    assert_eq!(
        AppFiles::in_config_dir(&rig.config).backdrop(),
        Some(rig.diagram.clone())
    );
    let rest = rig.h.shutdown();
    assert!(
        !rest.iter().any(|e| matches!(e, Event::Backdrop(_))),
        "{rest:#?}"
    );
}

/// The whole of the take: a clip naming its own copy of the diagram, under
/// `backdrops/` by the clip's id with the extension kept, anchored where the
/// player was — and jumped to like any other clip (spec W10 item 12).
#[test]
fn a_whiteboard_take_makes_a_clip_over_its_own_copy_of_the_diagram() {
    let mut rig = Rig::open();
    rig.h.send(Command::ScrubRelease { abs: 1.5 });
    rig.h.poll_until("settled at 1.5", |h| {
        h.position_secs().is_some_and(|p| (p - 1.5).abs() < FRAME)
    });
    rig.arm();
    rig.record();
    std::thread::sleep(Duration::from_millis(500));
    let clip = rig.stop();

    let name = format!("{}.png", clip.id);
    assert_eq!(clip.backdrop.as_deref(), Some(name.as_str()));
    let copy = rig.backdrops().join(&name);
    assert_eq!(
        files(&rig.backdrops()),
        std::slice::from_ref(&copy),
        "no temp file left"
    );
    assert_eq!(
        std::fs::read(&copy).unwrap(),
        std::fs::read(&rig.diagram).unwrap()
    );
    assert_eq!(clip.source_index, 0);
    assert!(
        (clip.start_source_seconds - 1.5).abs() < FRAME,
        "{}",
        clip.start_source_seconds
    );
    // And it was saved.
    assert_eq!(
        store::read(&rig.folder).unwrap().clips,
        std::slice::from_ref(&clip)
    );
    // The coach's own file is theirs.
    assert!(rig.diagram.exists());

    rig.h.send(Command::ScrubRelease { abs: 0.5 });
    rig.h.poll_until("settled at 0.5", |h| {
        h.position_secs().is_some_and(|p| (p - 0.5).abs() < FRAME)
    });
    rig.h.send(Command::JumpToClip(clip.id));
    rig.h.poll_until("parked on the anchor", |h| {
        h.position_secs().is_some_and(|p| (p - 1.5).abs() < FRAME)
    });
    rig.h.shutdown();
}

/// W with nothing armed says so, as a notice, and records nothing.
#[test]
fn nothing_armed_is_a_notice_and_records_nothing() {
    let mut rig = Rig::open();
    rig.whiteboard();
    assert_eq!(
        the_notice(&mut rig.h),
        "no diagram is armed; pick one first"
    );
    let rest = rig.h.shutdown();
    assert!(
        !rest.iter().any(|e| matches!(e, Event::Recording(_))),
        "{rest:#?}"
    );
    assert!(files(&rig.folder.join(RECORDINGS_DIRNAME)).is_empty());
    assert!(
        !rig.folder.join(BACKDROPS_DIRNAME).exists(),
        "made for nothing"
    );
}

/// A take cancelled before its first buffer leaves neither a recording nor a
/// diagram: nothing would ever name it.
#[test]
fn an_aborted_take_leaves_no_diagram() {
    let mut rig = Rig::open_with(SLOW_CAMERA);
    rig.arm();
    rig.whiteboard();
    assert_eq!(rig.h.wait_recording(), RecordingStatus::Starting);
    assert_eq!(files(&rig.backdrops()).len(), 1, "copied at the start");
    rig.whiteboard();
    assert_eq!(rig.h.wait_recording(), RecordingStatus::Idle);
    let backdrops = rig.backdrops();
    rig.h.shutdown();
    assert!(files(&backdrops).is_empty());
    assert!(files(&rig.folder.join(RECORDINGS_DIRNAME)).is_empty());
    assert!(store::read(&rig.folder).unwrap().clips.is_empty());
}

/// The recorder failing to start returns without reaching `abort_recording`,
/// so the diagram copied above it has to be taken back on that path too.
/// `recordings/` made read-only is what makes `filesink` refuse.
#[test]
fn a_recorder_that_fails_to_start_leaves_no_diagram() {
    let mut rig = Rig::open();
    rig.arm();
    let recordings = rig.folder.join(RECORDINGS_DIRNAME);
    std::fs::set_permissions(&recordings, std::fs::Permissions::from_mode(0o555)).unwrap();
    rig.whiteboard();
    let refused = rig.h.wait_for_error();
    std::fs::set_permissions(&recordings, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(
        matches!(refused, UserError::RecordingFailed(_)),
        "{refused:?}"
    );
    let backdrops = rig.backdrops();
    rig.h.shutdown();
    assert!(files(&backdrops).is_empty());
    assert!(store::read(&rig.folder).unwrap().clips.is_empty());
}

/// A diagram deleted since the pick fails the copy, and the take is refused
/// **before anything changes** — the player still playing where it was. That
/// is the half a copy placed beside `create_dir_all`, below the pause, would
/// get wrong while still writing no clip.
#[test]
fn a_copy_that_fails_refuses_the_take_and_leaves_the_player_alone() {
    let mut rig = Rig::open();
    rig.arm();
    std::fs::remove_file(&rig.diagram).unwrap();
    rig.h.toggle_play();
    assert!(rig.h.wait_playing());
    rig.whiteboard();
    assert_eq!(
        the_notice(&mut rig.h),
        "the armed diagram is gone; pick it again"
    );
    let before = rig.h.position_secs().unwrap();
    rig.h.poll_until("still playing", |h| {
        h.position_secs().is_some_and(|p| p > before + 0.3)
    });
    let backdrops = rig.backdrops();
    let rest = rig.h.shutdown();
    assert!(
        !rest
            .iter()
            .any(|e| matches!(e, Event::Recording(_) | Event::Playing(false))),
        "{rest:#?}"
    );
    assert!(files(&backdrops).is_empty());
}

/// The six commands a whiteboard take refuses, each with its own notice:
/// play and skip (nothing on screen for them to move, and they would log into
/// a clip whose picture never moves), and the four records about the footage —
/// a match tag, both slate marks and a highlight — placeable whenever the
/// footage is on screen, which here it is not. **The highlight is the one that
/// matters most**: keyed to footage nobody saw, it would ring a player over the
/// game video somewhere else entirely.
#[test]
fn a_whiteboard_take_refuses_what_belongs_to_the_footage() {
    let mut rig = Rig::open();
    rig.arm();
    rig.record();
    rig.h.toggle_play();
    the_notice(&mut rig.h);
    rig.h.skip(1.0);
    the_notice(&mut rig.h);
    rig.h.send(Command::TagMatchEvent {
        kind: MatchEventKind::HomeGoal,
        source_index: 0,
        source_seconds: 0.5,
    });
    the_notice(&mut rig.h);
    rig.h.send(Command::MarkSlateIn {
        source_index: 0,
        source_seconds: 0.5,
    });
    the_notice(&mut rig.h);
    rig.h.send(Command::MarkSlateOut {
        source_index: 0,
        source_seconds: 1.0,
    });
    the_notice(&mut rig.h);
    rig.h.send(Command::SetHighlightKey {
        id: Uuid::new_v4(),
        source_index: 0,
        source_seconds: 0.0,
        rect: NormRect {
            x: 0.4,
            y: 0.4,
            w: 0.1,
            h: 0.2,
        },
        color: Rgba::RED,
    });
    the_notice(&mut rig.h);
    let clip = rig.stop();
    rig.h.shutdown();

    assert!(
        !clip
            .events
            .iter()
            .any(|e| matches!(e.kind, EventKind::Play { .. } | EventKind::Skip { .. })),
        "{:?}",
        clip.events
    );
    let saved = store::read(&rig.folder).unwrap();
    assert!(saved.match_events.is_empty(), "{:?}", saved.match_events);
    assert!(saved.slates.is_empty(), "{:?}", saved.slates);
    assert!(
        saved.player_highlights.is_empty(),
        "{:?}",
        saved.player_highlights
    );
}

/// What a whiteboard take still allows is the plain take's list: a stroke is
/// the point of the feature, and a zoom drives the picture pad whatever it
/// carries.
#[test]
fn a_whiteboard_take_still_logs_strokes_and_zooms() {
    let mut rig = Rig::open();
    rig.arm();
    rig.record();
    let zoom = Zoom {
        scale: 2.0,
        ..Zoom::IDENTITY
    };
    rig.h.send(Command::Zoom {
        host_ns: now_ns(),
        zoom,
    });
    let stroke = Stroke {
        id: Uuid::new_v4(),
        color: Rgba::RED,
        line_width: 0.005,
        points: vec![
            StrokePoint {
                x: 0.25,
                y: 0.5,
                t: 0.0,
            },
            StrokePoint {
                x: 0.75,
                y: 0.5,
                t: 0.1,
            },
        ],
        auto_clear_after_seconds: None,
        end: StrokeEnd::Arrow,
    };
    rig.h.send(Command::Stroke {
        host_ns: now_ns(),
        stroke: stroke.clone(),
    });
    let clip = rig.stop();
    let kinds: Vec<EventKind> = clip.events.iter().map(|e| e.kind.clone()).collect();
    assert!(kinds.contains(&EventKind::Zoom(zoom)), "{kinds:?}");
    assert!(kinds.contains(&EventKind::Stroke(stroke)), "{kinds:?}");
    rig.h.shutdown();
}

/// Records a whiteboard clip, then takes its game video away and reopens the
/// project, so the bus's own missing flags say so. Returns the clip.
fn a_whiteboard_clip_whose_game_video_is_gone(rig: &mut Rig) -> Clip {
    rig.arm();
    rig.record();
    std::thread::sleep(Duration::from_millis(500));
    let clip = rig.stop();
    std::fs::remove_file(rig.media.join("a.webm")).unwrap();
    rig.h.send(Command::OpenProject(rig.folder.clone()));
    assert_eq!(*rig.h.wait_opened().missing, [true]);
    clip
}

/// A whiteboard draws not a pixel of the footage, so a missing game video is
/// no reason to refuse its preview — that would be a refusal whose only
/// remedy is relinking footage the coach isn't looking at.
#[test]
fn a_whiteboard_clip_previews_while_its_game_video_is_missing() {
    let mut rig = Rig::open();
    let clip = a_whiteboard_clip_whose_game_video_is_gone(&mut rig);
    rig.h.send(Command::OpenPreview(clip.id));
    assert_eq!(rig.h.wait_preview(), Some(clip.id));
    rig.h.send(Command::ClosePreview);
    assert_eq!(rig.h.wait_preview(), None);
    rig.h.shutdown();
}

/// And it is refused when the file it **does** draw is gone.
#[test]
fn a_whiteboard_clip_whose_diagram_is_missing_is_refused() {
    let mut rig = Rig::open();
    let clip = a_whiteboard_clip_whose_game_video_is_gone(&mut rig);
    for file in files(&rig.backdrops()) {
        std::fs::remove_file(file).unwrap();
    }
    rig.h.send(Command::OpenPreview(clip.id));
    assert_eq!(
        rig.h.wait_for_error(),
        UserError::CantPreview("the clip's diagram is missing".into())
    );
    rig.h.send(Command::Export {
        targets: vec![ExportTarget::Clip(clip.id)],
        choices: choices(),
    });
    assert_eq!(
        rig.h.wait_for_error(),
        UserError::CantExport(format!("{}'s diagram is missing", clip.name))
    );
    rig.h.shutdown();
}

/// The export's resolver makes the same call as the preview's.
#[test]
fn a_whiteboard_clip_exports_while_its_game_video_is_missing() {
    let mut rig = Rig::open();
    let clip = a_whiteboard_clip_whose_game_video_is_gone(&mut rig);
    rig.h.send(Command::Export {
        targets: vec![ExportTarget::Clip(clip.id)],
        choices: choices(),
    });
    let run = loop {
        let run = rig.h.wait_export();
        if !run.is_running() {
            break run;
        }
    };
    assert!(
        matches!(run.targets[0].state, TargetState::Done(_)),
        "{run:?}"
    );
    rig.h.shutdown();
}

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

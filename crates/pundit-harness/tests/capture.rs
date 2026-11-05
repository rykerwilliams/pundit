//! Bus end to end: `Command::CaptureSource` (in-gym capture spec §C, §V1) —
//! the camera recorded as a raw source, with test capture sources (no camera,
//! microphone or display). A capture plays over nothing: it makes a `SourceRef`
//! and never a `Clip`, and runs no aspect gate, reaching into a session that
//! `can_record` would refuse.
//!
//! Layout: `<tmp>/config` holds the state file, `<tmp>/media` the fixture
//! videos, `<tmp>/existing` a project of its own so `last_project` lands the
//! session's folder inside `<tmp>` (as `new_session.rs` arranges).
//!
//! **Two tiers, because of a `playbin3` crash, not the capture logic.** A
//! capture records the camera as H.264/Matroska, and `finish_capture` then
//! loads it into the scan player (`playbin3`) when nothing else is loaded, so
//! the coach can review it. On this GStreamer (1.24.2, Ubuntu 24.04's and CI's)
//! that load crashes `playbin3`'s `uridecodebin3` in `gst_object_replace`
//! intermittently (~40% per cycle; coredump backtrace: `gst_object_replace` ←
//! `libgstplayback` ← `gst_element_change_state`, signals 11 and 6) — the same
//! `playbin3` instability BACKLOG #72 documents, on the software path too
//! (bwrap `--tmpfs /dev/dri`, so not the hardware codec). The capture *logic*
//! is unaffected: every project-data assertion lands before the load.
//!
//! So the gating tests capture into a session that **already has a loaded
//! source**, where `ensure_loaded` is a no-op (`self.loaded()` is already true)
//! and the captured MKV is never opened — these are stable. The scenarios that
//! *must* open the capture in `playbin3` (an empty session's first capture, and
//! a review take over the capture) are `#[ignore]`d, run explicitly, and
//! tracked in BACKLOG as #72's crash.

use std::path::{Path, PathBuf};
use std::time::Duration;

use pundit_app::bus::{CaptureKind, Command, Event, RecordingStatus, Take, UserError};
use pundit_core::project::{Project, ProjectKind, SourceRef};
use pundit_core::store::{self, PROJECT_FILENAME, SOURCES_DIRNAME};
use pundit_core::zoom::Zoom;
use pundit_harness::{write_project, Harness};
use pundit_media::probe;
use tempfile::TempDir;

struct Rig {
    h: Harness,
    tmp: TempDir,
    folder: PathBuf,
    /// Whether a source was loaded before the capture, so `capture` may assert
    /// the capture issued no seek (an empty session's capture *does* load, via
    /// `finish_capture`'s `ensure_loaded`, so the check only applies here).
    preloaded: bool,
}

impl Rig {
    /// Opens an empty session (via `new_session.rs`'s arrangement) and returns
    /// a rig whose `folder` is that session.
    fn empty_session() -> Self {
        gstreamer::init().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(tmp.path().join("media")).unwrap();
        let mut h = Harness::with_capture(
            &tmp.path().join("config"),
            CaptureKind::Test {
                video_delay: Duration::ZERO,
            },
        );
        // A project of its own, so `last_project` points inside `<tmp>` and the
        // session lands beside it (not the real home).
        let existing = tmp.path().join("existing");
        std::fs::create_dir(&existing).unwrap();
        write_project(&existing, &tmp.path().join("media"), &[("a.webm", 2)]);
        h.send(Command::OpenProject(existing));
        h.wait_opened();

        h.send(Command::NewSession {
            name: "drills".into(),
        });
        let opened = h.wait_opened();
        assert_eq!(opened.project.kind, ProjectKind::Session);
        let folder = session_folder(tmp.path());
        Rig {
            h,
            tmp,
            folder,
            preloaded: false,
        }
    }

    /// An empty session with one **imported** webm added and loaded, so a
    /// subsequent capture leaves `playbin3` alone (`ensure_loaded` no-ops). The
    /// webm is `<tmp>/media/a.webm`, outside the session folder, so its stored
    /// path climbs out with `..` and is not under `sources/`.
    fn session_with_a_loaded_source() -> Self {
        let mut rig = Self::empty_session();
        let imported = rig.tmp.path().join("media").join("a.webm");
        assert!(imported.is_file());
        rig.h.send(Command::AddSource(imported));
        rig.h.wait_changed();
        rig.h.wait_settled();
        rig.preloaded = true;
        rig
    }

    /// Captures the camera once, from start to a finished source. Returns the
    /// project after the capture landed. A second press stops the capture in
    /// flight (§C4).
    ///
    /// **Asserts the capture issued no seek** (§C1: it "touches the player not
    /// at all"): with a source already loaded, a `heading`+`load` during capture
    /// start would move that source and publish a `Position` with a target. From
    /// the mark the capture began, none may appear — the capture's own
    /// `finish_capture` load (`ensure_loaded`) only fires when nothing is loaded,
    /// which `session_with_a_loaded_source` rules out.
    fn capture(&mut self) -> Project {
        let before = self.h.log().len();
        self.h.send(Command::CaptureSource);
        assert_eq!(
            self.h.wait_recording(),
            RecordingStatus::Starting { whiteboard: false }
        );
        match self.h.wait_recording() {
            RecordingStatus::Recording { .. } => {}
            status => panic!("expected Recording, got {status:?}"),
        }
        self.h.send(Command::CaptureSource);
        let changed = self.h.wait_changed();
        assert_eq!(self.h.wait_recording(), RecordingStatus::Idle);
        if self.preloaded {
            let seeked = self.h.log()[before..].iter().any(|e| {
                matches!(
                    e,
                    Event::Position {
                        target_abs: Some(_),
                        ..
                    }
                )
            });
            assert!(
                !seeked,
                "a capture must not seek the player: {:#?}",
                &self.h.log()[before..]
            );
        }
        (*changed.project).clone()
    }

    /// Records an ordinary take over the loaded source, returning the clip.
    fn record_a_take(&mut self) -> Project {
        self.h.send(Command::ToggleRecording {
            zoom: Zoom::IDENTITY,
            take: Take::Footage,
        });
        assert_eq!(
            self.h.wait_recording(),
            RecordingStatus::Starting { whiteboard: false }
        );
        match self.h.wait_recording() {
            RecordingStatus::Recording { .. } => {}
            status => panic!("expected Recording, got {status:?}"),
        }
        self.h.send(Command::StopRecording);
        let changed = self.h.wait_changed();
        assert_eq!(self.h.wait_recording(), RecordingStatus::Idle);
        (*changed.project).clone()
    }

    /// The captured source's file on disk.
    fn source_file(&self, source: &SourceRef) -> PathBuf {
        self.folder.join(&source.relative_path)
    }
}

/// The one session folder under `projects` (holds a `project.json` for a
/// `Session`).
fn session_folder(projects: &Path) -> PathBuf {
    let mut found: Vec<PathBuf> = std::fs::read_dir(projects)
        .into_iter()
        .flatten()
        .map(|e| e.unwrap().path())
        .filter(|p| p.join(PROJECT_FILENAME).is_file())
        .filter(|p| {
            store::read(p)
                .map(|pr| pr.kind == ProjectKind::Session)
                .unwrap_or(false)
        })
        .collect();
    found.sort();
    assert_eq!(found.len(), 1, "exactly one session folder: {found:?}");
    found.pop().unwrap()
}

fn is_under_sources(source: &SourceRef) -> bool {
    source
        .relative_path
        .starts_with(&format!("{SOURCES_DIRNAME}/"))
}

/// §C, §V1: a capture adds exactly one source, under `sources/`, with no clip;
/// the file exists and probes as video, and is persisted. Stable tier: a webm
/// is already loaded, so the capture never opens its own file in `playbin3`.
#[test]
fn capture_source_adds_a_source() {
    let mut rig = Rig::session_with_a_loaded_source();
    let project = rig.capture();

    // The webm (imported) plus the capture (under sources/).
    assert_eq!(
        project.source_videos.len(),
        2,
        "{:?}",
        project.source_videos
    );
    let captured = &project.source_videos[1];
    assert!(
        is_under_sources(captured),
        "a captured source lives under sources/: {:?}",
        captured.relative_path
    );
    assert!(
        !is_under_sources(&project.source_videos[0]),
        "the imported webm is not under sources/"
    );
    assert!(
        project.clips.is_empty(),
        "a capture makes a source, not a clip: {:?}",
        project.clips
    );
    let file = rig.source_file(captured);
    assert!(
        file.is_file(),
        "the captured file exists: {}",
        file.display()
    );
    assert!(
        probe(&file).is_ok(),
        "the captured file probes as video: {}",
        file.display()
    );
    // Persisted, so a reopen would find both sources.
    assert_eq!(
        store::read(&rig.folder).unwrap().source_videos.len(),
        2,
        "the source was saved"
    );

    rig.h.shutdown();
}

/// Step 9: removing an **unreferenced** captured source deletes its file —
/// source deletion is irreversible, and a project owns what it captured.
#[test]
fn removing_an_unreferenced_captured_source_deletes_its_file() {
    let mut rig = Rig::session_with_a_loaded_source();
    let project = rig.capture();
    let file = rig.source_file(&project.source_videos[1]);
    assert!(file.is_file());

    rig.h.send(Command::RemoveSource(1));
    let changed = rig.h.wait_changed();
    assert_eq!(
        changed.project.source_videos.len(),
        1,
        "only the capture was removed"
    );
    assert!(
        !file.exists(),
        "a captured source's file is deleted with it: {}",
        file.display()
    );

    rig.h.shutdown();
}

/// Step 9: removing an **imported** source — the coach's own file, added from
/// outside the project folder — leaves the file on disk. Only `sources/` is the
/// project's to delete. (No capture here, so no `playbin3` crash to dodge.)
#[test]
fn removing_an_imported_source_leaves_its_file() {
    let mut rig = Rig::session_with_a_loaded_source();
    let imported = rig.tmp.path().join("media").join("a.webm");
    assert!(imported.is_file());

    rig.h.send(Command::RemoveSource(0));
    let changed = rig.h.wait_changed();
    assert!(changed.project.source_videos.is_empty());
    assert!(
        imported.is_file(),
        "an imported source's file is the coach's and is untouched: {}",
        imported.display()
    );

    rig.h.shutdown();
}

/// Step 9 / sabotage 4: a **referenced** captured source keeps its file. A
/// slate marked on it makes `source_is_referenced` true, so core refuses the
/// removal and nothing is deleted — the delete must sit on core's `Ok` branch,
/// never before it. The reference is a slate, not a take, so no `playbin3`
/// load is needed and this gates stably.
#[test]
fn removing_a_referenced_captured_source_keeps_its_file() {
    let mut rig = Rig::session_with_a_loaded_source();
    let project = rig.capture();
    let captured = &project.source_videos[1];
    let file = rig.source_file(captured);
    assert!(is_under_sources(captured));

    // Mark a slate on the captured source (index 1): it now references it,
    // without ever opening it in the player.
    rig.h.send(Command::MarkSlateIn {
        source_index: 1,
        source_seconds: 0.0,
    });
    rig.h.wait_changed();

    rig.h.send(Command::RemoveSource(1));
    let err = rig.h.wait_for_error();
    assert!(
        matches!(err, UserError::SourceReferenced { .. }),
        "a referenced source is refused: {err:?}"
    );
    assert!(
        file.is_file(),
        "a refused removal keeps the file: {}",
        file.display()
    );

    rig.h.shutdown();
}

// --- #[ignore]d: scenarios that must open the capture in `playbin3` (BACKLOG
// #72's intermittent crash). They specify correct behaviour and pass whenever
// the `playbin3` race does not fire; run them explicitly:
//   cargo test -p pundit-harness --test capture -- --ignored
// --------------------------------------------------------------------------

/// §C, §V1: a capture into a **truly empty** session adds exactly one source,
/// under `sources/`, with no clip — the session's first move, where
/// `ensure_loaded` opens the capture in `playbin3` (hence `#[ignore]`).
#[test]
#[ignore = "BACKLOG #141 (playbin3 load crash, #72 family): loading the capture into playbin3 crashes intermittently"]
fn capture_source_into_an_empty_session_adds_a_source() {
    let mut rig = Rig::empty_session();
    let project = rig.capture();

    assert_eq!(
        project.source_videos.len(),
        1,
        "{:?}",
        project.source_videos
    );
    assert!(project.clips.is_empty(), "{:?}", project.clips);
    assert!(is_under_sources(&project.source_videos[0]));
    let file = rig.source_file(&project.source_videos[0]);
    assert!(file.is_file());
    assert!(probe(&file).is_ok());

    rig.h.shutdown();
}

/// §C1: a capture succeeds into an empty session where `can_record` refuses —
/// it bypasses the empty-project guard ("add a game video to record over
/// first"), the whole point of the own-guards path. `#[ignore]` for #141: the
/// capture opens in `playbin3` after the guard-bypass assertion has passed.
#[test]
#[ignore = "BACKLOG #141 (playbin3 load crash, #72 family): loading the capture into playbin3 crashes intermittently"]
fn capture_source_bypasses_the_empty_project_guard() {
    let mut rig = Rig::empty_session();
    let project = rig.capture();
    assert_eq!(project.source_videos.len(), 1);

    let rest = rig.h.shutdown();
    assert!(
        !rest.iter().any(|e| matches!(e, Event::Error(_))),
        "a capture into an empty session raises no error: {rest:#?}"
    );
}

/// §C3: once captured, a source takes an ordinary record take, which makes a
/// `Clip` through the unchanged recording path — capture-as-source costs the
/// review flow no new code. `#[ignore]` for #141: the review take must load the
/// captured source in `playbin3`.
#[test]
#[ignore = "BACKLOG #141 (playbin3 load crash, #72 family): loading the capture into playbin3 crashes intermittently"]
fn a_review_take_over_a_captured_source_is_an_ordinary_clip() {
    let mut rig = Rig::empty_session();
    rig.capture();
    // The captured source is now the session's footage; settle on it.
    rig.h.wait_settled();

    let project = rig.record_a_take();
    assert_eq!(
        project.source_videos.len(),
        1,
        "the review take recorded over the one captured source"
    );
    assert_eq!(
        project.clips.len(),
        1,
        "a review take makes an ordinary clip: {:?}",
        project.clips
    );

    rig.h.shutdown();
}

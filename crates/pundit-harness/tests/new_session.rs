//! Bus end to end: `Command::NewSession` (in-gym capture spec §N1–N4) — one
//! command creates an **empty** session and opens it, the inverse of a match.
//!
//! Layout: `<tmp>/config` holds the state file, `<tmp>/media` the fixture
//! video, `<tmp>/existing` a project of its own. The session is captured into,
//! so it has no footage; opening the existing project first puts `last_project`
//! in `<tmp>/existing`, so `projects_dir_for`'s `BesideLastProject` tier lands
//! the session's folder in `<tmp>` rather than the real home.

use std::path::{Path, PathBuf};
use std::time::Duration;

use pundit_app::bus::{CaptureKind, Command, Event, RecordingStatus, Take};
use pundit_core::project::{Project, ProjectKind};
use pundit_core::store::{self, PROJECT_FILENAME, RECORDINGS_DIRNAME};
use pundit_core::zoom::Zoom;
use pundit_harness::{write_project, Harness};
use tempfile::TempDir;

struct Rig {
    h: Harness,
    tmp: TempDir,
}

impl Rig {
    fn new() -> Self {
        Self::build(None)
    }

    fn with_capture() -> Self {
        Self::build(Some(CaptureKind::Test {
            video_delay: Duration::ZERO,
        }))
    }

    fn build(capture: Option<CaptureKind>) -> Self {
        gstreamer::init().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(tmp.path().join("media")).unwrap();
        let config = tmp.path().join("config");
        let h = match capture {
            Some(capture) => Harness::with_capture(&config, capture),
            None => Harness::new(&config),
        };
        Rig { h, tmp }
    }

    fn media(&self) -> PathBuf {
        self.tmp.path().join("media")
    }

    /// Opens a project of its own in `<tmp>/existing`, so `last_project` points
    /// inside `<tmp>` and the session lands beside it. Returns its folder.
    fn open_existing(&mut self) -> PathBuf {
        let folder = self.tmp.path().join("existing");
        std::fs::create_dir(&folder).unwrap();
        write_project(&folder, &self.media(), &[("a.webm", 2)]);
        self.h.send(Command::OpenProject(folder.clone()));
        self.h.wait_opened();
        folder
    }

    /// The projects folder the session folder is created in — `<tmp>`, the
    /// parent of the last-opened project.
    fn projects(&self) -> PathBuf {
        self.tmp.path().to_owned()
    }
}

/// The names in `dir` that are session folders (hold a `project.json` and a
/// `recordings/`), sorted.
fn session_folders(projects: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = std::fs::read_dir(projects)
        .into_iter()
        .flatten()
        .map(|e| e.unwrap().path())
        .filter(|p| p.join(PROJECT_FILENAME).is_file())
        .filter(|p| {
            // The "existing" match fixture has a project.json too — keep only
            // the sessions.
            store::read(p)
                .map(|pr| pr.kind == ProjectKind::Session)
                .unwrap_or(false)
        })
        .collect();
    found.sort();
    found
}

/// §N1: a session is created empty — no videos, no scoreboard — marked
/// `Session`, dated, written to disk and opened.
#[test]
fn new_session_creates_an_empty_session_and_opens_it() {
    let mut rig = Rig::new();
    rig.open_existing();

    rig.h.send(Command::NewSession {
        name: "Monday drills".into(),
    });
    let opened = rig.h.wait_opened();
    let p = &opened.project;

    assert_eq!(p.name, "Monday drills");
    assert_eq!(p.kind, ProjectKind::Session);
    assert!(p.created.is_some(), "a session is stamped with its date");
    assert!(p.source_videos.is_empty(), "a session starts empty");
    assert!(p.scoreboard.is_none());
    assert!(p.clips.is_empty());

    let sessions = session_folders(&rig.projects());
    assert_eq!(
        sessions.len(),
        1,
        "exactly one session folder: {sessions:?}"
    );
    let folder = &sessions[0];
    assert!(folder.join(PROJECT_FILENAME).is_file());
    assert!(folder.join(RECORDINGS_DIRNAME).is_dir());
    assert_eq!(&store::read(folder).unwrap(), &**p);

    // One `ProjectOpened` for the session, and none left over.
    let rest = rig.h.shutdown();
    assert!(
        !rest.iter().any(|e| matches!(e, Event::ProjectOpened(_))),
        "{rest:#?}"
    );
}

/// The recording allow-list is deny-by-default (`bus/mod.rs`), and `NewSession`
/// is deliberately left off it, as `NewMatch` is: a UI bug reaches a silent
/// refusal rather than a half-built project mid-take.
#[test]
fn new_session_is_refused_while_recording() {
    let mut rig = Rig::with_capture();
    rig.open_existing();
    rig.h.wait_settled();

    rig.h.send(Command::ToggleRecording {
        zoom: Zoom::IDENTITY,
        take: Take::Footage,
    });
    assert_eq!(
        rig.h.wait_recording(),
        RecordingStatus::Starting { whiteboard: false }
    );
    let live = rig.h.wait_recording();
    assert!(
        matches!(live, RecordingStatus::Recording { .. }),
        "{live:?}"
    );

    rig.h.send(Command::NewSession {
        name: "mid take".into(),
    });
    rig.h.send(Command::StopRecording);
    rig.h.wait_changed();
    assert_eq!(rig.h.wait_recording(), RecordingStatus::Idle);

    let projects = rig.projects();
    let rest = rig.h.shutdown();
    assert!(
        session_folders(&projects).is_empty(),
        "a refused session must create nothing"
    );
    assert!(
        !rest.iter().any(|e| matches!(e, Event::ProjectOpened(_))),
        "{rest:#?}"
    );
    assert!(
        !rest.iter().any(|e| matches!(e, Event::Error(_))),
        "a dropped command must be silent: {rest:#?}"
    );
}

/// §N2: a blank name defaults to `Session <date>`, created and opened.
#[test]
fn new_session_defaults_the_name_when_blank() {
    let mut rig = Rig::new();
    rig.open_existing();

    rig.h.send(Command::NewSession { name: "   ".into() });
    let opened = rig.h.wait_opened();
    let p: &Project = &opened.project;

    assert!(
        p.name.starts_with("Session "),
        "a blank name becomes a dated one: {:?}",
        p.name
    );
    // `Session YYYY-MM-DD`: the date the project also stamps itself with.
    let date = p.name.strip_prefix("Session ").unwrap();
    assert_eq!(Some(date), p.created.as_deref());
    assert_eq!(p.kind, ProjectKind::Session);

    let sessions = session_folders(&rig.projects());
    assert_eq!(
        sessions.len(),
        1,
        "exactly one session folder: {sessions:?}"
    );
    assert!(sessions[0].join(PROJECT_FILENAME).is_file());

    rig.h.shutdown();
}

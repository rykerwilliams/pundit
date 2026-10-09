//! Bus end to end: `Command::NewMatch` (new match spec C1–C5) — one command
//! makes the folder, writes the project and opens it, and every refusal leaves
//! nothing behind.
//!
//! Layout per test: `<tmp>/config` holds the state file, `<tmp>/media` the
//! fixture videos, `<tmp>/projects` is the projects folder a match folder is
//! created in — so stored source paths climb out of the project folder twice,
//! as real ones do — and `<tmp>/existing` is a project of its own, for the
//! tests that prove a refusal left the open one alone.
//!
//! **"Nothing was created" is asserted by listing the projects folder**, never
//! by testing one path: a typo in the path under test would make that
//! assertion pass whatever the command did.

use std::path::{Path, PathBuf};
use std::time::Duration;

use pundit_app::bus::{
    AppFiles, CaptureKind, Command, Event, ExportChoices, RecordingStatus, TargetState, UserError,
};
use pundit_core::plan::ExportTarget;
use pundit_core::project::{Project, Quality, Resolution};
use pundit_core::scoreboard::{MatchFormat, ScoreboardConfig, TeamConfig};
use pundit_core::store::{self, EXPORTS_DIRNAME, PROJECT_FILENAME, RECORDINGS_DIRNAME};
use pundit_core::stroke::Rgba;
use pundit_core::zoom::Zoom;
use pundit_harness::{add_clips, write_project, Harness, ReadOnly};
use pundit_media::fixtures;
use tempfile::TempDir;

/// The two teams, named, with everything else at its default: what the sheet
/// sends when the coach has typed both names and touched nothing else.
fn config(home: &str, away: &str) -> ScoreboardConfig {
    let kit = |c: u32| Rgba {
        r: f64::from((c >> 16) as u8) / 255.0,
        g: f64::from((c >> 8) as u8) / 255.0,
        b: f64::from(c as u8) / 255.0,
        a: 1.0,
    };
    ScoreboardConfig {
        home: TeamConfig::new(home, kit(0x00ff00), kit(0xffffff)),
        away: TeamConfig::new(away, kit(0x0000ff), kit(0xffffff)),
        format: MatchFormat::default(),
        auto_back_anchor_p1: false,
    }
}

struct Rig {
    h: Harness,
    tmp: TempDir,
}

impl Rig {
    /// A bus with nothing open; `media/` and `projects/` both exist.
    fn new() -> Self {
        Self::build(true, None)
    }

    /// [`Rig::new`] with `projects/` **absent**, so a test can prove the
    /// command creates its leaf.
    fn without_projects_dir() -> Self {
        Self::build(false, None)
    }

    /// [`Rig::new`] with test capture sources, for the recording guard.
    fn with_capture() -> Self {
        Self::build(
            true,
            Some(CaptureKind::Test {
                video_delay: Duration::ZERO,
            }),
        )
    }

    fn build(make_projects: bool, capture: Option<CaptureKind>) -> Self {
        gstreamer::init().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(tmp.path().join("media")).unwrap();
        if make_projects {
            std::fs::create_dir(tmp.path().join("projects")).unwrap();
        }
        let config = tmp.path().join("config");
        let h = match capture {
            Some(capture) => Harness::with_capture(&config, capture),
            None => Harness::new(&config),
        };
        Rig { h, tmp }
    }

    fn config(&self) -> PathBuf {
        self.tmp.path().join("config")
    }

    fn media(&self) -> PathBuf {
        self.tmp.path().join("media")
    }

    /// The projects folder — `<tmp>/projects`, whether or not it exists.
    fn projects(&self) -> PathBuf {
        self.tmp.path().join("projects")
    }

    /// The folder a match called `name` would be created in.
    fn match_dir(&self, name: &str) -> PathBuf {
        self.projects().join(name)
    }

    /// A 2-second 30 fps WebM in `media/`, written **once**: `fixtures::webm`
    /// re-runs its pipeline on every call, and rewriting a file the player
    /// already has open leaves it mid-write — "could not determine type of
    /// stream", which arrives as a `Playback` error the next `wait_for_error`
    /// would pick up instead of the refusal under test.
    fn video(&self, name: &str, w: u32, h: u32) -> PathBuf {
        let path = self.media().join(name);
        match path.is_file() {
            true => path,
            false => fixtures::webm(&self.media(), name, 2, w, h, 30, 15),
        }
    }

    /// Two 16:9 halves, in the order a game is played in.
    fn halves(&self) -> Vec<PathBuf> {
        vec![
            self.video("first half.webm", 320, 180),
            self.video("second half.webm", 320, 180),
        ]
    }

    /// Sends the command the sheet's Create button sends. **There is no name
    /// argument**: the project is called `<Home> v <Away>`, which the bus builds
    /// off the scoreboard it has to validate anyway (spec N4).
    fn create(&self, project_dir: &Path, teams: (&str, &str), videos: Vec<PathBuf>) {
        self.h.send(Command::NewMatch {
            project_dir: project_dir.to_owned(),
            scoreboard: config(teams.0, teams.1),
            videos,
        });
    }

    /// Opens a project of its own in `<tmp>/existing`, so a test can prove a
    /// refusal left it alone. Returns its folder.
    fn open_existing(&mut self, videos: &[(&str, u32)]) -> (PathBuf, Project) {
        let folder = self.tmp.path().join("existing");
        std::fs::create_dir(&folder).unwrap();
        let project = write_project(&folder, &self.media(), videos);
        self.h.send(Command::OpenProject(folder.clone()));
        self.h.wait_opened();
        (folder, project)
    }
}

/// The names in `dir`, sorted; empty when it doesn't exist.
fn entries(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

fn assert_nothing_created(projects: &Path) {
    assert_eq!(
        entries(projects),
        Vec::<String>::new(),
        "{} should be empty",
        projects.display()
    );
}

fn names(p: &Project) -> Vec<&str> {
    p.source_videos
        .iter()
        .map(|s| s.display_name.as_str())
        .collect()
}

fn stored_paths(p: &Project) -> Vec<&str> {
    p.source_videos
        .iter()
        .map(|s| s.relative_path.as_str())
        .collect()
}

fn teams(p: &Project) -> Option<(&str, &str)> {
    let s = p.scoreboard.as_ref()?;
    Some((s.home.name.as_str(), s.away.name.as_str()))
}

fn read_bytes(path: &Path) -> Vec<u8> {
    std::fs::read(path).unwrap()
}

/// Spec C2 whole: the folder, `recordings/`, the sources in the order given as
/// relative paths, the scoreboard, the name, and exactly one `ProjectOpened`.
#[test]
fn a_new_match_makes_the_folder_writes_the_project_and_opens_it() {
    let mut rig = Rig::new();
    let folder = rig.match_dir("2026-09-27-city-v-rovers");
    rig.create(&folder, ("City", "Rovers"), rig.halves());

    let opened = rig.h.wait_opened();
    let p = &opened.project;
    assert_eq!(p.name, "City v Rovers");
    assert_eq!(names(p), ["first half.webm", "second half.webm"]);
    assert_eq!(
        stored_paths(p),
        [
            "../../media/first half.webm",
            "../../media/second half.webm"
        ]
    );
    assert_eq!(teams(p), Some(("City", "Rovers")));
    assert!(!opened.missing.contains(&true), "{:?}", opened.missing);
    assert!(p.clips.is_empty());

    assert!(folder.join(PROJECT_FILENAME).is_file());
    assert!(folder.join(RECORDINGS_DIRNAME).is_dir());
    assert_eq!(store::read(&folder).unwrap(), **p);
    // A created project enters the recents list like any other open, at the
    // head — the flow stores nothing of its own.
    assert_eq!(
        AppFiles::in_config_dir(&rig.config()).recent_projects(),
        [folder.canonicalize().unwrap()]
    );

    // One `ProjectOpened`, not one per source: the whole point of the command.
    let rest = rig.h.shutdown();
    assert!(
        !rest.iter().any(|e| matches!(e, Event::ProjectOpened(_))),
        "{rest:#?}"
    );
}

/// The projects folder is created when it is missing — **its leaf only** (spec
/// W3). `create_dir_all` would turn a typo in that hand-editable path into a
/// tree, which is why the parent has to be there already.
#[test]
fn a_missing_projects_folder_is_created_leaf_only() {
    let mut rig = Rig::without_projects_dir();
    assert!(!rig.projects().exists());
    let folder = rig.match_dir("saturday");
    rig.create(&folder, ("City", "Rovers"), rig.halves());

    assert_eq!(rig.h.wait_opened().project.name, "City v Rovers");
    assert!(rig.projects().is_dir());
    assert_eq!(entries(&rig.projects()), ["saturday"]);
    rig.h.shutdown();
}

/// Trap 1, one half: an **empty** folder is adopted, so the coach who made one
/// by hand can point the flow at it. Keyed on `create_dir`'s `AlreadyExists`
/// instead, this would be refused — which is the bug the flow exists to fix.
#[test]
fn an_empty_folder_is_adopted() {
    let mut rig = Rig::new();
    let folder = rig.match_dir("stranded");
    std::fs::create_dir(&folder).unwrap();

    rig.create(&folder, ("City", "Rovers"), rig.halves());

    let opened = rig.h.wait_opened();
    assert_eq!(opened.project.name, "City v Rovers");
    assert_eq!(
        names(&opened.project),
        ["first half.webm", "second half.webm"]
    );
    assert_eq!(store::read(&folder).unwrap(), *opened.project);
    rig.h.shutdown();
}

/// Trap 1, the other half: the refusal is keyed on `project.json`, and the
/// project already there is not touched. Fails **apart** from
/// [`an_empty_folder_is_adopted`] by design — a change that fails both has
/// broken the command rather than proven the rule.
#[test]
fn a_folder_holding_a_project_is_refused_and_its_file_untouched() {
    let mut rig = Rig::new();
    let folder = rig.match_dir("taken");
    rig.create(&folder, ("City", "Rovers"), rig.halves());
    rig.h.wait_opened();
    let before = read_bytes(&folder.join(PROJECT_FILENAME));

    rig.create(&folder, ("Town", "United"), rig.halves());
    assert_eq!(
        rig.h.wait_for_error(),
        UserError::Io(format!(
            "{} already holds a project — open that instead",
            folder.display()
        ))
    );
    assert_eq!(
        read_bytes(&folder.join(PROJECT_FILENAME)),
        before,
        "the refused folder's project must not be touched"
    );

    // The first project is still the open one.
    rig.h.send(Command::RenameProject("Renamed".into()));
    assert_eq!(rig.h.wait_changed().project.name, "Renamed");
    rig.h.shutdown();
}

/// A regular file where the folder should go: `create_dir` reports it as
/// `AlreadyExists` too, and without the `is_dir` guard the `project.json`
/// probe finds nothing and `store::write` fails later with a confusing
/// `ENOTDIR`.
#[test]
fn a_file_where_the_folder_should_go_is_refused() {
    let mut rig = Rig::new();
    let folder = rig.match_dir("not-a-folder");
    std::fs::write(&folder, b"").unwrap();

    rig.create(&folder, ("City", "Rovers"), rig.halves());
    assert_eq!(
        rig.h.wait_for_error(),
        UserError::Io(format!("{} is a file, not a folder", folder.display()))
    );
    assert_eq!(entries(&rig.projects()), ["not-a-folder"]);
    rig.h.shutdown();
}

/// The guard `set_scoreboard` has and a project created with a scoreboard
/// already on it would otherwise have walked around (spec C2's `storable`).
/// Modal here, where `set_scoreboard` makes it a notice: a notice draws in the
/// status line, behind the sheet's scrim.
#[test]
fn a_blank_team_name_is_refused_and_nothing_is_created() {
    let mut rig = Rig::new();
    let folder = rig.match_dir("blank");
    rig.create(&folder, ("City", "   "), rig.halves());

    assert_eq!(
        rig.h.wait_for_error(),
        UserError::Io("both teams need a name".into())
    );
    assert_nothing_created(&rig.projects());
    rig.h.shutdown();
}

/// The cheap argument checks, before anything is probed or created.
#[test]
fn an_empty_video_list_and_a_relative_path_are_refused() {
    let mut rig = Rig::new();
    let folder = rig.match_dir("args");

    rig.create(&folder, ("City", "Rovers"), Vec::new());
    assert_eq!(
        rig.h.wait_for_error(),
        UserError::Io("pick the game's video files first".into())
    );

    rig.create(
        Path::new("projects/relative"),
        ("City", "Rovers"),
        rig.halves(),
    );
    assert_eq!(
        rig.h.wait_for_error(),
        UserError::Io("not a full path: projects/relative".into())
    );

    assert_nothing_created(&rig.projects());
    rig.h.shutdown();
}

/// A video that can't be probed: an error naming the file, and **nothing
/// created** — the probes run before the first `create_dir`.
#[test]
fn a_video_that_cant_be_probed_creates_nothing() {
    let mut rig = Rig::new();
    let notes = rig.media().join("notes.txt");
    std::fs::write(&notes, b"not a video").unwrap();
    let folder = rig.match_dir("unprobeable");

    rig.create(
        &folder,
        ("City", "Rovers"),
        vec![rig.video("first half.webm", 320, 180), notes.clone()],
    );

    let prefix = format!("{}: ", notes.display());
    match rig.h.wait_for_error() {
        UserError::Io(msg) => assert!(msg.starts_with(&prefix), "{msg:?}"),
        e => panic!("expected a modal Io naming the file, got {e:?}"),
    }
    assert_nothing_created(&rig.projects());
    rig.h.shutdown();
}

/// The decisive refusal (spec C1): the second half's shape is gated **before**
/// any directory is made, so there is never a named folder holding one of a
/// game's two halves.
#[test]
fn a_second_video_of_a_different_shape_creates_no_folder_holding_the_first() {
    let mut rig = Rig::new();
    let wide = rig.video("first half.webm", 320, 180);
    let tall = rig.video("second half.webm", 320, 240);
    let folder = rig.match_dir("mismatched");

    rig.create(
        &folder,
        ("City", "Rovers"),
        vec![wide.clone(), tall.clone()],
    );

    assert_eq!(
        rig.h.wait_for_error(),
        UserError::Io(format!(
            "{} is 1.778:1 but {} is 1.333:1 — a match's videos all have to be the same shape",
            wide.display(),
            tall.display()
        ))
    );
    assert_nothing_created(&rig.projects());
    rig.h.shutdown();
}

/// Spec W3's floor: only the projects folder's leaf is ever created, so its own
/// parent has to exist. Refused before anything is probed, and **no tree**.
#[test]
fn a_projects_folder_whose_parent_is_missing_creates_no_tree() {
    let mut rig = Rig::new();
    let missing = rig.tmp.path().join("no-such-drive");
    let projects = missing.join("pundit");
    let folder = projects.join("saturday");

    rig.create(&folder, ("City", "Rovers"), rig.halves());
    assert_eq!(
        rig.h.wait_for_error(),
        UserError::Io(format!(
            "{} can't be created: the folder it would go in doesn't exist",
            projects.display()
        ))
    );
    assert!(!missing.exists(), "{} was created", missing.display());
    rig.h.shutdown();
}

/// A read-only projects folder: `create_dir` fails, the error names the path,
/// and nothing is created.
#[test]
fn a_read_only_projects_folder_is_refused() {
    let mut rig = Rig::new();
    let read_only = ReadOnly::new(&rig.projects());
    if !read_only.enforced() {
        eprintln!("skipped: the read-only mode isn't enforced (running as root?)");
        return;
    }
    let folder = rig.match_dir("saturday");
    rig.create(&folder, ("City", "Rovers"), rig.halves());

    let prefix = format!("{}: ", folder.display());
    match rig.h.wait_for_error() {
        UserError::Io(msg) => assert!(msg.starts_with(&prefix), "{msg:?}"),
        e => panic!("expected a modal Io naming the folder, got {e:?}"),
    }
    assert_nothing_created(&rig.projects());
    drop(read_only);
    rig.h.shutdown();
}

/// The one documented failure that **does** leave something behind (spec C3): a
/// step after the two `create_dir`s fails, nothing is rolled back, and the folder
/// is then an empty one — which the next Create adopts. A read-only *match*
/// folder is what reaches it: `store::write` cannot write into it.
///
/// Every other refusal test asserts the projects folder is empty, so without this
/// one the state the doc comment describes has no test at all.
#[test]
fn a_write_that_fails_after_the_folder_is_made_leaves_one_the_next_create_adopts() {
    let mut rig = Rig::new();
    let folder = rig.match_dir("saturday");
    std::fs::create_dir(&folder).unwrap();
    let read_only = ReadOnly::new(&folder);
    if !read_only.enforced() {
        eprintln!("skipped: the read-only mode isn't enforced (running as root?)");
        return;
    }
    let videos = rig.halves();

    rig.create(&folder, ("City", "Rovers"), videos.clone());
    let prefix = format!("{}: ", folder.display());
    match rig.h.wait_for_error() {
        UserError::Io(msg) => assert!(msg.starts_with(&prefix), "{msg:?}"),
        e => panic!("expected a modal Io naming the folder, got {e:?}"),
    }
    // The folder is there and empty: nothing was rolled back, and nothing was
    // written into it.
    assert!(folder.is_dir());
    assert!(!folder.join(PROJECT_FILENAME).exists());

    // And the next Create adopts it, which is what makes the no-rollback choice
    // safe rather than merely cheap.
    drop(read_only);
    rig.create(&folder, ("City", "Rovers"), videos);
    let opened = rig.h.wait_opened();
    assert_eq!(opened.project.name, "City v Rovers");
    assert_eq!(
        names(&opened.project),
        ["first half.webm", "second half.webm"]
    );
    rig.h.shutdown();
}

/// Spec C3's last row: a Create that fails at any step leaves the coach in the
/// project he was in, and its file untouched.
#[test]
fn the_open_project_survives_every_refusal() {
    let mut rig = Rig::new();
    let tall = rig.video("tall.webm", 320, 240);
    let wide = rig.video("wide.webm", 320, 180);

    // One project in the projects folder, so the "already holds a project"
    // refusal has something to refuse.
    let taken = rig.match_dir("taken");
    rig.create(&taken, ("City", "Rovers"), vec![wide.clone()]);
    rig.h.wait_opened();

    let (existing, project) = rig.open_existing(&[("a.webm", 2)]);
    let mut before = read_bytes(&existing.join(PROJECT_FILENAME));

    for (why, dir, both, videos) in [
        (
            "a blank team name",
            rig.match_dir("blank"),
            ("City", ""),
            vec![wide.clone()],
        ),
        (
            "a mismatched shape",
            rig.match_dir("mismatched"),
            ("City", "Rovers"),
            vec![wide.clone(), tall.clone()],
        ),
        (
            "a folder that holds a project",
            taken.clone(),
            ("City", "Rovers"),
            vec![wide.clone()],
        ),
    ] {
        rig.create(&dir, both, videos);
        let err = rig.h.wait_for_error();
        assert!(matches!(err, UserError::Io(_)), "{why}: {err:?}");
        assert_eq!(
            read_bytes(&existing.join(PROJECT_FILENAME)),
            before,
            "{why}: the open project's file was written"
        );

        // A mutation still lands on the project that was open, in its folder.
        rig.h.send(Command::RenameProject(format!("after {why}")));
        let changed = rig.h.wait_changed();
        assert_eq!(changed.project.name, format!("after {why}"), "{why}");
        assert_eq!(
            names(&changed.project),
            names(&project),
            "{why}: the open project's sources"
        );
        assert_eq!(entries(&rig.projects()), ["taken"], "{why}");
        before = read_bytes(&existing.join(PROJECT_FILENAME));
    }

    let rest = rig.h.shutdown();
    assert!(
        !rest.iter().any(|e| matches!(e, Event::ProjectOpened(_))),
        "{rest:#?}"
    );
}

/// **New match during an export goes through, and the run finishes** (#77 spec
/// §Q7, the coach 2026-10-07: *"yes you should be able to keep working"*).
///
/// **This inverts rather than being amended**, as its sibling in `export.rs`
/// did: it used to assert `refuse_if_busy`'s *"an export is running"* here, on
/// the stated grounds that *"an export must not have the project swapped
/// underneath it"* — and the swap reaches no job. `Active` reads `self.open`
/// nowhere, export composites on its own surfaceless display, and
/// `entry_media` `stat`s its files rather than consulting `Bus::missing`
/// (basket spec V2), so a project the bus has closed is still safe to render
/// from. §Q7 audited all eleven of `commit`'s steps against a rendering job.
///
/// Two of the three things it was guarding are kept and **inverted** — the
/// folder *is* created and the open project *does* change — and the third
/// stands unchanged and is the point: **the run is not disturbed**. `New
/// match…` was already clickable mid-export because the sheet is meant to be
/// closed while a run continues; now the project it creates opens too.
#[test]
fn new_match_during_an_export_goes_through_and_the_run_finishes() {
    let mut rig = Rig::new();
    let folder = rig.tmp.path().join("existing");
    std::fs::create_dir(&folder).unwrap();
    let mut project = write_project(&folder, &rig.media(), &[("a.webm", 2)]);
    add_clips(&folder, &mut project, &[0]);
    project.clips[0].recording_duration = 1.0;
    project.clips[0].name = "clip 0".into();
    store::write(&folder, &mut project).unwrap();
    rig.h.send(Command::OpenProject(folder.clone()));
    rig.h.wait_opened();

    // Written before the run: a `fixtures::webm` pipeline alongside an export
    // is noise this test does not need.
    let videos = rig.halves();
    let new = rig.match_dir("mid-export");

    rig.h.send(Command::Export {
        targets: vec![ExportTarget::AllClips],
        choices: ExportChoices {
            resolution: Resolution::R720,
            quality: Quality::Low,
            scoreboard: None,
            mute_source: false,
            chapters: true,
            cues: true,
        },
    });
    assert!(rig.h.wait_export().is_running());

    rig.create(&new, ("City", "Rovers"), videos);
    let opened = rig.h.wait_opened();
    assert_eq!(
        opened.project.name, "City v Rovers",
        "the match the coach asked for is not the one that opened"
    );
    // **The folder is the path the command carried; the name is the
    // scoreboard's** — `Command::NewMatch` has no name field and the bus
    // builds one from the two teams it has to validate anyway, so these are
    // deliberately two different strings and both are asserted.
    assert_eq!(
        entries(&rig.projects()),
        ["mid-export"],
        "one folder, the one asked for, and nothing else created"
    );

    // **The one assertion that did not invert, and it is the point**: the run
    // the coach walked away from finishes, into the folder of the project he
    // left rather than the one he is now in.
    let done = rig.h.wait_map("the run's outcome", |e| match e {
        Event::Export(run) if !run.is_running() => Some(run.clone()),
        _ => None,
    });
    assert!(
        matches!(done.targets[0].state, TargetState::Done(_)),
        "{:?}",
        done.targets[0]
    );
    assert!(
        folder
            .join(EXPORTS_DIRNAME)
            .join("All clips - Game.mp4")
            .is_file(),
        "the run wrote into the project it was started in"
    );
    // The project it was started in is untouched by the Create, which went to a
    // folder of its own.
    assert_eq!(store::read(&folder).unwrap().name, project.name);
    rig.h.shutdown();
}

/// The recording allow-list is deny-by-default (`bus/mod.rs`), and `NewMatch`
/// is deliberately left off it: a UI bug reaches a silent refusal rather than a
/// half-built project mid-take.
#[test]
fn new_match_while_recording_is_dropped() {
    let mut rig = Rig::with_capture();
    let folder = rig.tmp.path().join("existing");
    std::fs::create_dir(&folder).unwrap();
    write_project(&folder, &rig.media(), &[("a.webm", 2)]);
    rig.h.send(Command::OpenProject(folder.clone()));
    rig.h.wait_opened();
    rig.h.wait_settled();

    // The fixtures the command would be given, written before the take: a
    // `fixtures::webm` pipeline alongside a live recording is noise this test
    // does not need.
    let videos = rig.halves();
    let new = rig.match_dir("mid-take");

    rig.h.send(Command::ToggleRecording {
        zoom: Zoom::IDENTITY,
        slate: None,
    });
    assert_eq!(rig.h.wait_recording(), RecordingStatus::Starting);
    let live = rig.h.wait_recording();
    assert!(
        matches!(live, RecordingStatus::Recording { .. }),
        "{live:?}"
    );

    rig.create(&new, ("City", "Rovers"), videos);
    rig.h.send(Command::StopRecording);
    rig.h.wait_changed();
    assert_eq!(rig.h.wait_recording(), RecordingStatus::Idle);

    let projects = rig.projects();
    let rest = rig.h.shutdown();
    assert_nothing_created(&projects);
    assert!(
        !rest.iter().any(|e| matches!(e, Event::ProjectOpened(_))),
        "{rest:#?}"
    );
    assert!(
        !rest.iter().any(|e| matches!(e, Event::Error(_))),
        "a dropped command must be silent: {rest:#?}"
    );
    assert_eq!(store::read(&folder).unwrap().name, "Game");
}

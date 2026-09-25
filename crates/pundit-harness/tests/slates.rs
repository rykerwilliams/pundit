//! Bus end to end: slates — marking a range while watching, the refusal that
//! is a notice, the one command pair that is live during a take, and the undo
//! step each edit is. What a slate *is* and what the mutators refuse are
//! core's tests.
//!
//! Layout per test: `<tmp>/config` holds the state file, `<tmp>/project` the
//! project, `<tmp>/media` the fixture game videos.

use std::path::PathBuf;

use pundit_app::bus::{Command, RecordingStatus, UserError};
use pundit_core::project::{Project, SlateEdit};
use pundit_core::store;
use pundit_core::zoom::Zoom;
use pundit_harness::{write_project, Harness};
use tempfile::TempDir;

/// A project of 2-second fixture videos, as written.
struct Proj {
    folder: PathBuf,
    _tmp: TempDir,
}

impl Proj {
    fn open(videos: &[&str]) -> (Harness, Self) {
        gstreamer::init().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let folder = tmp.path().join("project");
        let media = tmp.path().join("media");
        std::fs::create_dir(&folder).unwrap();
        std::fs::create_dir(&media).unwrap();
        let videos: Vec<(&str, u32)> = videos.iter().map(|&name| (name, 2)).collect();
        write_project(&folder, &media, &videos);

        let mut h = Harness::new(&tmp.path().join("config"));
        h.send(Command::OpenProject(folder.clone()));
        h.wait_opened();
        (h, Proj { folder, _tmp: tmp })
    }

    fn saved(&self) -> Project {
        store::read(&self.folder).unwrap()
    }
}

fn mark_in(source_index: usize, source_seconds: f64) -> Command {
    Command::MarkSlateIn {
        source_index,
        source_seconds,
    }
}

fn mark_out(source_index: usize, source_seconds: f64) -> Command {
    Command::MarkSlateOut {
        source_index,
        source_seconds,
    }
}

/// The pair, and what reaches the file: one range, on the video it was marked
/// on, at the times the caller captured.
#[test]
fn marking_in_then_out_saves_one_range() {
    let (mut h, p) = Proj::open(&["a.webm", "b.webm"]);
    h.wait_settled();

    h.send(mark_in(1, 0.5));
    let open = h.wait_changed();
    assert_eq!(open.project.slates.len(), 1);
    assert_eq!(open.project.slates[0].out_seconds, None);

    h.send(mark_out(1, 1.5));
    h.wait_changed();
    h.shutdown();

    let saved = p.saved();
    assert_eq!(saved.slates.len(), 1);
    assert_eq!(saved.slates[0].source_index, 1);
    assert_eq!(saved.slates[0].in_seconds, 0.5);
    assert_eq!(saved.slates[0].out_seconds, Some(1.5));
    assert_eq!(saved.slates[0].name, "");
}

/// `o` with nothing open says so **out loud and as a notice**: marking is live
/// during a take, so a modal here could land over a recording and swallow the
/// transport keys. Nothing is stored either way.
#[test]
fn closing_nothing_is_a_spoken_refusal_that_stores_nothing() {
    let (mut h, p) = Proj::open(&["a.webm"]);
    h.wait_settled();

    h.send(mark_out(0, 1.0));
    let err = h.wait_for_error();
    assert!(matches!(err, UserError::Slate(_)), "{err:?}");
    assert!(err.is_notice(), "a modal could land over a live take");
    assert!(
        err.to_string().contains("press i"),
        "it says which key to press: {err}"
    );

    h.shutdown();
    assert!(p.saved().slates.is_empty());
}

/// The coach spots the next moment while talking over this one, so both marks
/// are on the recording allow-list — the rule that already puts a match tag
/// and a highlight key there. Editing and deleting wait, as every other edit
/// does.
#[test]
fn marking_works_while_recording_and_the_other_edits_are_refused() {
    let (mut h, p) = Proj::open(&["a.webm"]);
    h.wait_settled();
    h.send(mark_in(0, 0.2));
    let id = h.wait_changed().project.slates[0].id;

    h.send(Command::ToggleRecording {
        zoom: Zoom::IDENTITY,
    });
    h.wait_recording();
    h.wait_recording();

    // Live: the range closes mid-take.
    h.send(mark_out(0, 1.2));
    assert_eq!(
        h.wait_changed().project.slates[0].out_seconds,
        Some(1.2),
        "marking is live during a take"
    );

    // Not live: these are silently refused by the allow-list.
    h.send(Command::EditSlate {
        id,
        edit: SlateEdit::Name("corner routine".into()),
    });
    h.send(Command::DeleteSlate(id));
    // A refusal while recording is silent, so the stop behind them is what
    // proves they never landed.
    h.send(Command::StopRecording);
    h.wait_changed();
    h.wait_recording();

    h.shutdown();
    let saved = p.saved();
    assert_eq!(saved.slates.len(), 1, "the delete was refused");
    assert_eq!(saved.slates[0].name, "", "the rename was refused");
}

/// Every edit is one undo step, and a command naming a slate that is gone
/// costs neither a step nor a save.
#[test]
fn each_edit_is_one_undo_step() {
    let (mut h, p) = Proj::open(&["a.webm"]);
    h.wait_settled();

    h.send(mark_in(0, 0.3));
    let id = h.wait_changed().project.slates[0].id;
    h.send(mark_out(0, 1.3));
    h.wait_changed();
    h.send(Command::EditSlate {
        id,
        edit: SlateEdit::Tags(vec!["corners".into()]),
    });
    assert_eq!(h.wait_changed().project.slates[0].tags, ["corners"]);

    // Back over the tags, the out point, and the slate itself.
    h.send(Command::Undo);
    assert!(h.wait_changed().project.slates[0].tags.is_empty());
    h.send(Command::Undo);
    assert_eq!(h.wait_changed().project.slates[0].out_seconds, None);
    h.send(Command::Undo);
    assert!(h.wait_changed().project.slates.is_empty());

    // And forward again.
    h.send(Command::Redo);
    assert_eq!(h.wait_changed().project.slates.len(), 1);

    h.shutdown();
    assert_eq!(p.saved().slates.len(), 1);
}

/// A slate holds its video open, and the refusal names slates among the kinds
/// rather than listing only clips, match events and highlights.
#[test]
fn a_source_a_slate_sits_on_cannot_be_removed() {
    let (mut h, p) = Proj::open(&["a.webm", "b.webm"]);
    h.wait_settled();
    h.send(mark_in(1, 0.4));
    h.wait_changed();

    h.send(Command::RemoveSource(1));
    let err = h.wait_for_error();
    assert!(
        err.to_string().contains("slate"),
        "the refusal should name slates: {err}"
    );

    h.shutdown();
    let saved = p.saved();
    assert_eq!(saved.source_videos.len(), 2, "nothing was removed");
    assert_eq!(saved.slates.len(), 1);
}

// ------------------------------------------------------------ shooting

/// The whole point of the feature: Record on a row starts the take at the
/// range's in point, and the clip it produces carries the slate's name, tags
/// and id — the tagging done live is not done twice.
#[test]
fn shooting_a_slate_starts_at_its_in_point_and_the_clip_inherits_it() {
    let (mut h, p) = Proj::open(&["a.webm", "b.webm"]);
    h.wait_settled();

    h.send(mark_in(1, 1.2));
    let id = h.wait_changed().project.slates[0].id;
    h.send(mark_out(1, 1.8));
    h.wait_changed();
    h.send(Command::EditSlate {
        id,
        edit: SlateEdit::Name("corner routine".into()),
    });
    h.send(Command::EditSlate {
        id,
        edit: SlateEdit::Tags(vec!["corners".into()]),
    });
    h.wait_changed();
    h.wait_changed();

    h.send(Command::ShootSlate {
        id,
        zoom: Zoom::IDENTITY,
    });
    h.wait_recording();
    h.wait_recording();
    h.send(Command::StopRecording);
    h.wait_changed();
    h.wait_recording();
    h.shutdown();

    let saved = p.saved();
    assert_eq!(saved.clips.len(), 1);
    let clip = &saved.clips[0];
    assert_eq!(clip.source_index, 1, "the slate's video");
    assert!(
        (clip.start_source_seconds - 1.2).abs() < 0.05,
        "the take starts at the in point, not where the player was: {}",
        clip.start_source_seconds
    );
    assert_eq!(clip.name, "corner routine");
    assert_eq!(clip.tags, ["corners"]);
    assert_eq!(
        clip.slate_id,
        Some(id),
        "the clip names the slate it came from"
    );
    assert_eq!(saved.slates.len(), 1, "and the slate survives its take");
}

/// A skip burst still in the air must not carry the take with it.
/// `start_recording` reads the skip coordinator's pending target in preference
/// to the player's, so the shoot resets it before seeking — otherwise the clip
/// is stamped where the arrows were heading.
#[test]
fn a_pending_skip_does_not_drag_the_take_off_the_in_point() {
    let (mut h, p) = Proj::open(&["a.webm"]);
    h.wait_settled();
    h.send(mark_in(0, 0.3));
    let id = h.wait_changed().project.slates[0].id;

    // Two taps of the arrow: a burst is pending when Record is pressed.
    h.send(Command::Skip {
        delta: 3.0,
        host_ns: pundit_media::now_ns(),
    });
    h.send(Command::Skip {
        delta: 3.0,
        host_ns: pundit_media::now_ns(),
    });
    h.send(Command::ShootSlate {
        id,
        zoom: Zoom::IDENTITY,
    });
    h.wait_recording();
    h.wait_recording();
    h.send(Command::StopRecording);
    h.wait_changed();
    h.wait_recording();
    h.shutdown();

    let clip = &p.saved().clips[0];
    assert!(
        (clip.start_source_seconds - 0.3).abs() < 0.05,
        "the burst should not move the take: {}",
        clip.start_source_seconds
    );
}

/// An unnamed slate leaves the clip's generated name alone — "2-00:00:01"
/// says more than an empty string, and the tagging is still inherited.
#[test]
fn an_unnamed_slate_leaves_the_generated_clip_name() {
    let (mut h, p) = Proj::open(&["a.webm"]);
    h.wait_settled();
    h.send(mark_in(0, 0.4));
    let id = h.wait_changed().project.slates[0].id;

    h.send(Command::ShootSlate {
        id,
        zoom: Zoom::IDENTITY,
    });
    h.wait_recording();
    h.wait_recording();
    h.send(Command::StopRecording);
    h.wait_changed();
    h.wait_recording();
    h.shutdown();

    let clip = &p.saved().clips[0];
    assert!(!clip.name.is_empty(), "the generated name stands");
    assert_eq!(clip.slate_id, Some(id));
}

/// Shooting a slate while a take is already running is refused, and the
/// running take is left alone — starting a second would lose the first.
#[test]
fn shooting_while_recording_is_refused() {
    let (mut h, p) = Proj::open(&["a.webm"]);
    h.wait_settled();
    h.send(mark_in(0, 0.5));
    let id = h.wait_changed().project.slates[0].id;

    h.send(Command::ToggleRecording {
        zoom: Zoom::IDENTITY,
    });
    h.wait_recording();
    assert!(matches!(
        h.wait_recording(),
        RecordingStatus::Recording { .. }
    ));

    h.send(Command::ShootSlate {
        id,
        zoom: Zoom::IDENTITY,
    });
    h.send(Command::StopRecording);
    h.wait_changed();
    h.wait_recording();
    h.shutdown();

    let saved = p.saved();
    assert_eq!(saved.clips.len(), 1, "one take, not two");
    assert_eq!(saved.clips[0].slate_id, None, "and it was not the slate's");
}

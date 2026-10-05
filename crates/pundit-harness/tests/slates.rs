//! Bus end to end: slates — marking a range while watching, the refusal that
//! is a notice, the one command pair that is live during a take, the undo
//! step each edit is, the out-point stop (BACKLOG #114), the preview that
//! shares it (BACKLOG #104, spec P), and `R` carrying the selected range
//! (BACKLOG #120). What a slate *is* and what the mutators refuse are core's
//! tests; where the themed pass goes *next* is a unit test, because the queue
//! is the window's own row model and the harness has no window.
//!
//! Layout per test: `<tmp>/config` holds the state file, `<tmp>/project` the
//! project, `<tmp>/media` the fixture game videos.

use std::path::PathBuf;
use std::time::Duration;

use pundit_app::bus::{Command, Event, RecordingStatus, ScanStep, UserError};
use pundit_core::event::{CommentaryEvent, EventKind};
use pundit_core::project::{Clip, Project, SlateEdit};
use pundit_core::store;
use pundit_core::zoom::Zoom;
use pundit_harness::{write_project, Harness, FRAME};
use tempfile::TempDir;
use uuid::Uuid;

/// A project of fixture videos, as written.
struct Proj {
    folder: PathBuf,
    _tmp: TempDir,
}

impl Proj {
    /// Two-second videos, which is all a test about marking needs.
    fn open(videos: &[&str]) -> (Harness, Self) {
        let videos: Vec<(&str, u32)> = videos.iter().map(|&name| (name, 2)).collect();
        Self::open_lengths(&videos)
    }

    /// [`Proj::open`] with each video's length in seconds: the out-point tests
    /// need footage to play through.
    fn open_lengths(videos: &[(&str, u32)]) -> (Harness, Self) {
        gstreamer::init().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let folder = tmp.path().join("project");
        let media = tmp.path().join("media");
        std::fs::create_dir(&folder).unwrap();
        std::fs::create_dir(&media).unwrap();
        write_project(&folder, &media, videos);

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

/// **Moving a mark, and the refusal reaching the coach** (BACKLOG #119).
///
/// This is the assertion the spec's review asked for: `Project::edit_slate` is
/// infallible and `Bus::edit_slates` returns **silently** when nothing changed,
/// so a core-side refusal would be indistinguishable from a button that did
/// nothing. The check is in the bus, and this proves the notice comes out.
#[test]
fn moving_a_mark_works_and_an_inverting_move_is_a_spoken_refusal() {
    let (mut h, p) = Proj::open(&["a.webm"]);
    h.wait_settled();

    h.send(mark_in(0, 1.0));
    h.wait_changed();
    h.send(mark_out(0, 2.0));
    h.wait_changed();
    let id = p.saved().slates[0].id;

    // Both marks move.
    h.send(Command::EditSlate {
        id,
        edit: SlateEdit::Out(3.0),
    });
    h.wait_changed();
    h.send(Command::EditSlate {
        id,
        edit: SlateEdit::In(1.5),
    });
    h.wait_changed();
    let moved = p.saved().slates[0].clone();
    assert_eq!((moved.in_seconds, moved.out_seconds), (1.5, Some(3.0)));

    // And an inverting move is refused out loud, storing nothing — in both
    // directions, which is the half an earlier draft of the spec missed.
    for edit in [SlateEdit::Out(1.5), SlateEdit::In(3.0)] {
        h.send(Command::EditSlate { id, edit });
        let err = h.wait_for_error();
        assert!(matches!(err, UserError::Slate(_)), "{err:?}");
        assert!(err.is_notice(), "a modal could land over a live take");
        let after = p.saved().slates[0].clone();
        assert_eq!(
            (after.in_seconds, after.out_seconds),
            (1.5, Some(3.0)),
            "a refused move stores nothing"
        );
    }

    h.shutdown();
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
        slate: None,
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
        slate: None,
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

/// **`R` with a range selected is that range's take** (BACKLOG #120): the
/// window hands `ToggleRecording` whatever is selected and the bus turns it
/// into the shoot, so the clip starts at the in point and carries the
/// `slate_id` that marks the range shot — which is what the themed pass
/// advances on. A plain take here would record the right frames under a clip
/// that left the range looking unshot for ever.
#[test]
fn toggling_a_recording_with_a_range_selected_shoots_it() {
    let (mut h, p) = Proj::open(&["a.webm"]);
    h.wait_settled();
    h.send(mark_in(0, 1.1));
    let id = h.wait_changed().project.slates[0].id;

    h.send(Command::ToggleRecording {
        zoom: Zoom::IDENTITY,
        slate: Some(id),
    });
    h.wait_recording();
    h.wait_recording();
    h.send(Command::StopRecording);
    h.wait_changed();
    h.wait_recording();
    h.shutdown();

    let clip = &p.saved().clips[0];
    assert_eq!(clip.slate_id, Some(id), "the take is the range's");
    assert!(
        (clip.start_source_seconds - 1.1).abs() < 0.05,
        "and it starts at the in point: {}",
        clip.start_source_seconds
    );
}

/// **And a second `R` still stops it**, which is why the range travels on
/// `ToggleRecording` rather than the window choosing `ShootSlate` itself. The
/// window's phase lags the bus's, so that choice would be made on a stale
/// reading — and `shooting_while_recording_is_refused` above is what the
/// mistake costs: the key would be swallowed and the take would run on. The
/// range is selected throughout its own take, so this is the ordinary case and
/// not an edge one.
#[test]
fn a_second_toggle_stops_the_range_take_rather_than_starting_another() {
    let (mut h, p) = Proj::open(&["a.webm"]);
    h.wait_settled();
    h.send(mark_in(0, 0.5));
    let id = h.wait_changed().project.slates[0].id;

    h.send(Command::ToggleRecording {
        zoom: Zoom::IDENTITY,
        slate: Some(id),
    });
    assert_eq!(h.wait_recording(), RecordingStatus::Starting);
    assert!(matches!(
        h.wait_recording(),
        RecordingStatus::Recording { .. }
    ));

    h.send(Command::ToggleRecording {
        zoom: Zoom::IDENTITY,
        slate: Some(id),
    });
    h.wait_changed();
    assert_eq!(
        h.wait_recording(),
        RecordingStatus::Idle,
        "the second R stops the take"
    );
    h.shutdown();

    let saved = p.saved();
    assert_eq!(saved.clips.len(), 1, "one take, not two");
    assert_eq!(saved.clips[0].slate_id, Some(id));
}

// -------------------------------------------- stopping at the out point

/// Shoots slate `id` and waits for the take to be recording.
fn shoot(h: &mut Harness, id: Uuid) {
    h.send(Command::ShootSlate {
        id,
        zoom: Zoom::IDENTITY,
    });
    assert_eq!(h.wait_recording(), RecordingStatus::Starting);
    assert!(matches!(
        h.wait_recording(),
        RecordingStatus::Recording { .. }
    ));
}

/// Stops the take and returns the clip it made.
fn stop(h: &mut Harness) -> Clip {
    h.send(Command::StopRecording);
    let changed = h.wait_changed();
    assert_eq!(h.wait_recording(), RecordingStatus::Idle);
    changed
        .project
        .clips
        .last()
        .expect("the take made a clip")
        .clone()
}

/// The latest `Position` the bus published: source index and target.
fn latest_position(h: &Harness) -> Option<(usize, Option<f64>)> {
    h.log().iter().rev().find_map(|e| match e {
        Event::Position {
            source_index,
            target_abs,
        } => Some((*source_index, *target_abs)),
        _ => None,
    })
}

/// Every pause in `events` but the one `RecordingLog::new` writes at record
/// time 0, which every take opens with — "every clip starts on a still frame".
fn pauses_after_the_first(events: &[CommentaryEvent]) -> Vec<&CommentaryEvent> {
    let mut pauses: Vec<&CommentaryEvent> = events
        .iter()
        .filter(|e| matches!(e.kind, EventKind::Pause { .. }))
        .collect();
    assert!(
        pauses.first().is_some_and(|e| e.record_time == 0.0),
        "every log opens with a pause at record time 0: {events:?}"
    );
    pauses.remove(0);
    pauses
}

/// **The feature** (BACKLOG #114, spec S1): a take shot from a timed slate
/// plays to the range's out point and the *footage* pauses there while the
/// recording runs on, so the picture holds on the range's last frame while the
/// coach finishes the sentence.
///
/// The anchor is the stored `out`, exactly — not a reading of the playhead,
/// which is what makes replay freeze on the frame the coach was looking at.
#[test]
fn a_take_from_a_timed_slate_pauses_at_its_out_point() {
    let (mut h, p) = Proj::open_lengths(&[("a.webm", 4)]);
    h.wait_settled();
    h.send(mark_in(0, 0.2));
    let id = h.wait_changed().project.slates[0].id;
    h.send(mark_out(0, 1.2));
    h.wait_changed();

    shoot(&mut h, id);
    h.toggle_play();
    assert!(h.wait_playing(), "the take plays from the in point");

    // Nobody sends a command for this: the bus pauses the footage itself.
    assert!(!h.wait_playing(), "the footage pauses at the out point");
    let at_pause = h.position_secs().expect("a position");
    assert!(
        (1.2..2.0).contains(&at_pause),
        "paused at the end of the range, not well past it: {at_pause}"
    );
    std::thread::sleep(Duration::from_millis(300));
    let later = h.position_secs().expect("a position");
    assert!(
        (later - at_pause).abs() < FRAME,
        "the footage really stopped: played on from {at_pause} to {later}"
    );

    // The recording was never stopped, and its log ends with that pause.
    let clip = stop(&mut h);
    assert_eq!(
        clip.events.last().map(|e| e.kind.clone()),
        Some(EventKind::Pause { source_time: 1.2 }),
        "anchored at the stored out point, not at a reading: {:?}",
        clip.events
    );
    assert!(
        clip.events.last().is_some_and(|e| e.record_time > 0.0),
        "and it happened during the take: {:?}",
        clip.events
    );
    h.shutdown();
    assert_eq!(p.saved().slates.len(), 1, "the slate survives its take");
}

/// **A deliberate move past the out point disarms and never pauses** (#114's
/// own rule, and the plan's item 6).
///
/// Measured: a forward skip makes `query_position` read the skip's *target*
/// within 5 ms, so a check that trusted the position alone would fire here and
/// log a pause anchored at `out` while the picture is seconds past it — replay
/// would then freeze behind the footage the coach is talking over. The
/// distinguisher is therefore the **event**: the landing spends the arm in
/// silence, and the poll only ever sees a position playback reached.
#[test]
fn a_forward_skip_past_the_out_point_disarms_without_pausing() {
    let (mut h, _p) = Proj::open_lengths(&[("a.webm", 8)]);
    h.wait_settled();
    h.send(mark_in(0, 0.2));
    let id = h.wait_changed().project.slates[0].id;
    h.send(mark_out(0, 2.0));
    h.wait_changed();

    shoot(&mut h, id);
    h.toggle_play();
    assert!(h.wait_playing());
    // Well short of the out point, so nothing has fired when the skip is sent.
    h.poll_until("playback past 0.4 s", |h| {
        h.position_secs().is_some_and(|p| p > 0.4)
    });
    h.skip(4.0);
    // Its landing is past the out point: this is where a naive check fires.
    h.poll_until("the skip landed past the out point", |h| {
        h.position_secs().is_some_and(|p| p > 3.0)
    });
    // Several polls' worth of wall time, so "it hasn't fired yet" can't pass
    // for "it never fires".
    std::thread::sleep(Duration::from_millis(400));
    let clip = stop(&mut h);

    assert!(
        pauses_after_the_first(&clip.events).is_empty(),
        "the skip spends the arm without logging a pause: {:?}",
        clip.events
    );
    assert!(
        !h.log().iter().any(|e| matches!(e, Event::Playing(false))),
        "and nothing paused the footage: {:?}",
        h.log()
    );
    h.shutdown();
}

/// A half-marked range has no end, so its take behaves as one started with
/// `R`: the footage plays on.
#[test]
fn a_half_marked_slates_take_never_pauses() {
    let (mut h, _p) = Proj::open_lengths(&[("a.webm", 4)]);
    h.wait_settled();
    h.send(mark_in(0, 0.2));
    let id = h.wait_changed().project.slates[0].id;

    shoot(&mut h, id);
    h.toggle_play();
    assert!(h.wait_playing());
    h.poll_until("playback past 1.5 s", |h| {
        h.position_secs().is_some_and(|p| p > 1.5)
    });
    let clip = stop(&mut h);

    assert!(
        pauses_after_the_first(&clip.events).is_empty(),
        "nothing to stop at: {:?}",
        clip.events
    );
    h.shutdown();
}

/// A plain `R` take is unchanged by a timed range sitting elsewhere in the
/// project: the arm belongs to a take shot *from* a slate, and nothing else
/// arms it.
///
/// What this cannot reach yet is the arm *surviving* into a plain take — the
/// arm is only ever set where a take starts, so a refused shoot leaves nothing
/// behind. The inherited-arm case arrives with the preview path (task C), which
/// is why `start_recording` assigns the arm unconditionally rather than only
/// for a shoot.
#[test]
fn a_plain_take_is_not_stopped_by_a_timed_slate_on_another_video() {
    let (mut h, p) = Proj::open_lengths(&[("a.webm", 2), ("b.webm", 6)]);
    h.wait_settled();
    h.send(mark_in(0, 0.2));
    h.wait_changed();
    h.send(mark_out(0, 0.5));
    h.wait_changed();

    // A plain take on the second video, played well past the first range's
    // out point in its own source time.
    let second = p.saved().source_videos[0].duration_seconds + 0.1;
    h.send(Command::ScrubRelease { abs: second });
    h.poll_until("the second video, settled", |h| {
        latest_position(h) == Some((1, None))
    });
    h.send(Command::ToggleRecording {
        zoom: Zoom::IDENTITY,
        slate: None,
    });
    assert_eq!(h.wait_recording(), RecordingStatus::Starting);
    assert!(matches!(
        h.wait_recording(),
        RecordingStatus::Recording { .. }
    ));
    h.toggle_play();
    assert!(h.wait_playing());
    h.poll_until("playback past 1.5 s into the second video", |h| {
        h.position_secs().is_some_and(|p| p > 1.5)
    });
    let clip = stop(&mut h);

    assert_eq!(clip.source_index, 1);
    assert!(
        pauses_after_the_first(&clip.events).is_empty(),
        "another video's range is not this take's business: {:?}",
        clip.events
    );
    h.shutdown();
}

/// **The arm is an id, resolved at every check — never the `(source_index,
/// out)` pair it stands for.** `MarkSlateOut` is on the recording allow-list,
/// so a take shot from a *half-marked* range can gain its out point mid-take:
/// `o` closes the range and the footage stops where the coach said it ends.
/// A pair cached when the take began could not do this — there was no out
/// point to cache.
#[test]
fn closing_the_range_mid_take_stops_the_footage_there() {
    let (mut h, _p) = Proj::open_lengths(&[("a.webm", 4)]);
    h.wait_settled();
    h.send(mark_in(0, 0.2));
    let id = h.wait_changed().project.slates[0].id;

    shoot(&mut h, id);
    h.toggle_play();
    assert!(h.wait_playing());
    h.poll_until("playback past 1.0 s", |h| {
        h.position_secs().is_some_and(|p| p > 1.0)
    });
    // `o`, with the position the UI read at the key press.
    let at_mark = h.position_secs().expect("a position");
    h.send(mark_out(0, at_mark));
    assert_eq!(
        h.wait_changed().project.slates[0].out_seconds,
        Some(at_mark)
    );
    assert!(!h.wait_playing(), "the range just acquired its end");
    let clip = stop(&mut h);

    assert_eq!(
        clip.events.last().map(|e| e.kind.clone()),
        Some(EventKind::Pause {
            source_time: at_mark
        }),
        "anchored at the out point just marked: {:?}",
        clip.events
    );
    h.shutdown();
}

/// **The arm goes with the take.** A take stopped before its out point leaves
/// nothing armed, so scanning on afterwards runs through the range — the stop
/// is the take's, not the footage's, and the commentary is already recorded.
#[test]
fn the_arm_does_not_outlive_the_take() {
    let (mut h, _p) = Proj::open_lengths(&[("a.webm", 4)]);
    h.wait_settled();
    h.send(mark_in(0, 0.2));
    let id = h.wait_changed().project.slates[0].id;
    h.send(mark_out(0, 1.2));
    h.wait_changed();

    // Shot and stopped without ever playing: the out point is still ahead.
    shoot(&mut h, id);
    stop(&mut h);

    h.toggle_play();
    assert!(h.wait_playing());
    // A stale arm would pause the footage at 1.2 and this would never land.
    h.poll_until("playback through the range and past it", |h| {
        h.position_secs().is_some_and(|p| p > 1.8)
    });
    assert!(
        !h.log()
            .iter()
            .rev()
            .take_while(|e| !matches!(e, Event::Playing(true)))
            .any(|e| matches!(e, Event::Playing(false))),
        "the footage is still playing: {:?}",
        h.log()
    );
    h.shutdown();
}

// --------------------------------------------- previewing a marked range

/// Waits for a preview to be playing: **two `Playing` events, not one.** The
/// park pauses before it seeks — that is what makes the landing still, and it
/// is also what returns the rate to 1x — so the play of the range itself is
/// the second to arrive.
fn wait_previewing(h: &mut Harness) {
    assert!(!h.wait_playing(), "the park pauses before it seeks");
    assert!(h.wait_playing(), "and then the range plays");
}

/// Every `Playing(false)` since the footage last started, which is the only
/// honest way to ask "did it stop?" of a path whose own park pauses first.
fn stopped_since_it_started(h: &Harness) -> bool {
    h.log()
        .iter()
        .rev()
        .take_while(|e| !matches!(e, Event::Playing(true)))
        .any(|e| matches!(e, Event::Playing(false)))
}

/// **#104's jump**: the row's park, on a slate on *another* video, so it is
/// the switch as well as the seek. Nothing plays — a jump is for finding the
/// range, and the footage running on from it would be the thing the themed
/// pass has to undo.
///
/// The slate here is **half-marked** on purpose: it still has an in point, so
/// it jumps like any other.
#[test]
fn jumping_to_a_slate_parks_on_its_in_point_without_playing() {
    let (mut h, _p) = Proj::open_lengths(&[("a.webm", 2), ("b.webm", 4)]);
    h.wait_settled();
    h.send(mark_in(1, 1.5));
    let id = h.wait_changed().project.slates[0].id;

    h.send(Command::JumpToSlate(id));
    h.poll_until("parked on the slate's own video, settled", |h| {
        latest_position(h) == Some((1, None))
    });
    let at = h.position_secs().expect("a position");
    assert!((at - 1.5).abs() < FRAME, "parked on the in point: {at}");
    assert!(
        !h.log().iter().any(|e| matches!(e, Event::Playing(true))),
        "a jump never plays: {:?}",
        h.log()
    );
    h.shutdown();
}

/// **The preview** (spec P2): the jump, then play, then the armed stop ends
/// it at the out point. No recording is involved — the arm is not a take's
/// alone.
///
/// **It starts from past the end of the range on purpose**, which is what
/// makes this the regression guard for `slate_out_reached`'s `is_idle`: the
/// park seeks back to the in point and plays in the same bus iteration, so a
/// check that read the position before the seek was acted on saw the 4.5 the
/// coach was watching, fired at once, and stopped the preview on its first
/// frame. Measured — it is how this test first failed.
#[test]
fn previewing_a_slate_plays_the_range_and_stops_at_its_end() {
    let (mut h, _p) = Proj::open_lengths(&[("a.webm", 6)]);
    h.wait_settled();
    h.send(mark_in(0, 1.0));
    let id = h.wait_changed().project.slates[0].id;
    h.send(mark_out(0, 2.5));
    h.wait_changed();

    // Watching past the end of the range first, so the jump back is what the
    // "inside the range" poll below can only be explained by.
    h.send(Command::ScrubRelease { abs: 4.5 });
    h.poll_until("settled past the range", |h| {
        latest_position(h) == Some((0, None))
    });

    h.send(Command::PreviewSlate(id));
    wait_previewing(&mut h);
    h.poll_until("playing inside the range", |h| {
        h.position_secs().is_some_and(|p| (1.0..2.5).contains(&p))
    });

    // Nobody sends a command for this: the bus pauses the footage itself.
    assert!(!h.wait_playing(), "and it stops where the range ends");
    let at_stop = h.position_secs().expect("a position");
    assert!(
        (2.5..3.3).contains(&at_stop),
        "stopped at the end of the range, not well past it: {at_stop}"
    );
    std::thread::sleep(Duration::from_millis(300));
    let later = h.position_secs().expect("a position");
    assert!(
        (later - at_stop).abs() < FRAME,
        "the footage really stopped: played on from {at_stop} to {later}"
    );
    h.shutdown();
}

/// **A plain `R` take does not inherit a live preview's arm** — the case the
/// out-point task could not reach, and the reason `start_recording` assigns
/// the arm *unconditionally* rather than only for a shoot. Without that, this
/// take would pause the footage at a range it has nothing to do with, and
/// write a pause into a log whose clip never went near it.
#[test]
fn a_plain_take_does_not_inherit_a_previews_arm() {
    let (mut h, _p) = Proj::open_lengths(&[("a.webm", 6)]);
    h.wait_settled();
    h.send(mark_in(0, 0.2));
    let id = h.wait_changed().project.slates[0].id;
    h.send(mark_out(0, 3.0));
    h.wait_changed();

    h.send(Command::PreviewSlate(id));
    wait_previewing(&mut h);

    // `R` over the live preview. `start_recording` pauses the footage, so the
    // take is parked well short of the out point, which is still ahead of it.
    h.send(Command::ToggleRecording {
        zoom: Zoom::IDENTITY,
        slate: None,
    });
    assert_eq!(h.wait_recording(), RecordingStatus::Starting);
    assert!(matches!(
        h.wait_recording(),
        RecordingStatus::Recording { .. }
    ));
    h.toggle_play();
    assert!(h.wait_playing());
    h.poll_until("playback through the range's out point and past it", |h| {
        h.position_secs().is_some_and(|p| p > 3.4)
    });
    let clip = stop(&mut h);

    assert_eq!(clip.slate_id, None, "it is nobody's slate's take");
    assert!(
        pauses_after_the_first(&clip.events).is_empty(),
        "the preview's arm went with the preview: {:?}",
        clip.events
    );
    h.shutdown();
}

/// **The arm is checked against the slate's own video**, which the out-point
/// task could not reach either: a take's source never changes, but a preview
/// plays on and the footage can be taken to the next video with the range
/// still armed. A check on the time alone would stop the footage wherever
/// *that* video's time passed the out point.
#[test]
fn a_previews_arm_does_not_stop_the_next_video_at_the_same_time() {
    let (mut h, p) = Proj::open_lengths(&[("a.webm", 6), ("b.webm", 6)]);
    h.wait_settled();
    h.send(mark_in(0, 0.2));
    let id = h.wait_changed().project.slates[0].id;
    h.send(mark_out(0, 2.0));
    h.wait_changed();

    h.send(Command::PreviewSlate(id));
    wait_previewing(&mut h);
    // Into the second video, short of 2.0 in *its* source time: the landing
    // is not past the armed slate's out point on the slate's own video, so it
    // does not spend the arm.
    let second = p.saved().source_videos[0].duration_seconds + 0.1;
    h.send(Command::ScrubRelease { abs: second });
    h.poll_until("past 2 s of the second video, still playing", |h| {
        latest_position(h).map(|(i, _)| i) == Some(1)
            && h.position_secs().is_some_and(|pos| pos > 2.4)
    });

    assert!(
        !stopped_since_it_started(&h),
        "another video's time is not this range's end: {:?}",
        h.log()
    );
    h.shutdown();
}

/// **`J` and `L` are refused while a range is armed** (spec P3). The control
/// half is the same key on plain playback, so what this pins is the arm and
/// not a harness that can't press it.
///
/// Why it matters is measured: at 32x the poll overshoots the out point by
/// 0.64 s, and the pause's own seek lands on the frame on screen, which at
/// that speed trails the position by up to 0.6 s — so the footage would come
/// to rest *before* the mark the stop is named after.
#[test]
fn the_scan_keys_are_refused_while_a_range_is_armed() {
    let (mut h, _p) = Proj::open_lengths(&[("a.webm", 6)]);
    h.wait_settled();
    h.send(mark_in(0, 0.2));
    let id = h.wait_changed().project.slates[0].id;
    h.send(mark_out(0, 4.0));
    h.wait_changed();

    // The control: nothing armed, and the same key takes the speed up.
    h.toggle_play();
    assert!(h.wait_playing());
    h.send(Command::ScanSpeed(ScanStep::Faster));
    h.wait_speed(2.0);

    // The preview's park pauses, and every pause of the game video returns it
    // to 1x -- which is spec P3's "it forces 1x", with no rule of its own.
    h.send(Command::PreviewSlate(id));
    h.wait_speed(1.0);
    assert!(h.wait_playing(), "and it plays the range at 1x");

    h.send(Command::ScanSpeed(ScanStep::Faster));
    // A fence: the bus is one thread handling in order, so a mark that has
    // come back proves the scan press was already dealt with.
    h.send(mark_in(0, 5.0));
    h.wait_changed();
    assert!(
        !h.log()
            .iter()
            .rev()
            .take_while(|e| !matches!(e, Event::Playing(true)))
            .any(|e| matches!(e, Event::ScanSpeed(s) if *s != 1.0)),
        "the speed is the armed range's, not the key's: {:?}",
        h.log()
    );
    h.shutdown();
}

//! Marking ranges: the mutators behind `i` and `o`, and what they refuse.
//!
//! All tests here are new — the macOS app had no such record.

use pundit_core::project::{Project, SlateEdit, SlateError, SourceRef};
use pundit_core::recording::PendingClip;
use uuid::Uuid;

fn project(sources: usize) -> Project {
    let mut p = Project::new("Game");
    for i in 0..sources {
        p.source_videos.push(SourceRef {
            relative_path: format!("../media/{i}.mp4"),
            display_name: format!("video {i}"),
            duration_seconds: 1800.0,
            display_aspect: 16.0 / 9.0,
        });
    }
    p
}

/// The pair, and the shape a marked range has on disk: open on the first
/// press, closed on the second.
#[test]
fn marking_in_then_out_leaves_one_closed_range() {
    let mut p = project(1);
    let id = p.mark_slate_in(0, 845.0);
    assert_eq!(p.slates.len(), 1);
    assert_eq!(p.slates[0].out_seconds, None, "open until `o`");

    assert_eq!(p.mark_slate_out(0, 880.0), Ok(id));
    assert_eq!(p.slates[0].in_seconds, 845.0);
    assert_eq!(p.slates[0].out_seconds, Some(880.0));
}

/// `o` with nothing open is the whole of the "out before in" refusal the first
/// draft needed a rule for: there is simply nothing to close.
#[test]
fn marking_out_with_nothing_open_is_refused_and_stores_nothing() {
    let mut p = project(1);
    assert_eq!(p.mark_slate_out(0, 12.0), Err(SlateError::NothingOpen));
    assert!(p.slates.is_empty());

    // And a range already closed is not re-closed.
    p.mark_slate_in(0, 5.0);
    p.mark_slate_out(0, 9.0).unwrap();
    assert_eq!(p.mark_slate_out(0, 20.0), Err(SlateError::NothingOpen));
    assert_eq!(p.slates[0].out_seconds, Some(9.0));
}

/// A range open on another video is not this video's to close: `o` on video 0
/// leaves video 1's alone, which is what makes "one source per slate" true
/// without a cross-source check anywhere.
#[test]
fn marking_out_only_closes_a_range_on_the_same_video() {
    let mut p = project(2);
    p.mark_slate_in(1, 30.0);

    assert_eq!(p.mark_slate_out(0, 40.0), Err(SlateError::NothingOpen));
    assert_eq!(p.slates[0].out_seconds, None);

    assert!(p.mark_slate_out(1, 40.0).is_ok());
    assert_eq!(p.slates[0].out_seconds, Some(40.0));
}

/// Scrubbing backwards and pressing `o` would otherwise store a range that
/// runs backwards, which every reader of it would have to defend against.
#[test]
fn an_out_point_at_or_before_the_in_point_is_refused() {
    let mut p = project(1);
    p.mark_slate_in(0, 100.0);

    for out in [99.0, 100.0] {
        assert_eq!(
            p.mark_slate_out(0, out),
            Err(SlateError::OutBeforeIn { in_seconds: 100.0 })
        );
        assert_eq!(p.slates[0].out_seconds, None, "and nothing is stored");
    }
    assert!(p.mark_slate_out(0, 100.5).is_ok());
}

/// Two ranges open at once close newest-first, which is why the stored order
/// is the marked order and `slates_sorted` is for reading only.
#[test]
fn the_newest_open_range_is_the_one_that_closes() {
    let mut p = project(1);
    let first = p.mark_slate_in(0, 10.0);
    let second = p.mark_slate_in(0, 20.0);

    assert_eq!(p.mark_slate_out(0, 25.0), Ok(second));
    assert_eq!(p.mark_slate_out(0, 30.0), Ok(first));
    assert_eq!(p.slates[0].out_seconds, Some(30.0));
    assert_eq!(p.slates[1].out_seconds, Some(25.0));
}

/// A name is trimmed like a clip's, and tags arrive already normalized —
/// `normalize_tags` is the caller's, as it is for a clip.
#[test]
fn editing_a_slate_sets_the_field_it_names() {
    let mut p = project(1);
    let id = p.mark_slate_in(0, 1.0);

    p.edit_slate(id, SlateEdit::Name("  corner routine  ".into()));
    p.edit_slate(id, SlateEdit::Tags(vec!["corners".into()]));
    assert_eq!(p.slates[0].name, "corner routine");
    assert_eq!(p.slates[0].tags, ["corners"]);

    // A command naming a slate that is gone changes nothing.
    p.edit_slate(Uuid::new_v4(), SlateEdit::Name("ghost".into()));
    assert_eq!(p.slates[0].name, "corner routine");
}

/// Deleting a slate leaves the clip shot from it alone, `slate_id` and all:
/// "has this been shot?" is asked of the clips, so a link naming nothing is
/// harmless — which is the whole reason the link points this way.
#[test]
fn deleting_a_slate_leaves_the_clip_it_was_shot_into() {
    let mut p = project(1);
    let id = p.mark_slate_in(0, 1.0);
    let pending = PendingClip {
        id: Uuid::new_v4(),
        source_index: 0,
        start_source_seconds: 1.0,
    };
    p.add_recorded_clip(pending, 12.0, Vec::new(), "2026-09-25T00:00:00Z".into());
    p.clips[0].slate_id = Some(id);

    p.delete_slate(id);

    assert!(p.slates.is_empty());
    assert_eq!(p.clips.len(), 1, "the take is not deleted with its slate");
    assert_eq!(p.clips[0].slate_id, Some(id), "and keeps its dangling link");
}

//! Where the selected slate's marked range falls on the scrubber (spec S6).
//!
//! The rules live here rather than in `main.rs` because `main.rs` is wiring and
//! has no `#[cfg(test)]` module at all — the same reason [`crate::recents`] and
//! [`crate::fit`] are their own modules. What is worth pinning is the mapping
//! into **concat** time: a slate belongs to one source, and the spec's own §R
//! records a draft that dropped `source_index` and so compared times across a
//! multi-video timeline.
//!
//! **Nothing here reaches the bus, and the bus publishes nothing for it.** The
//! selection is UI state, so the window already holds both halves the span
//! needs; the armed slate is always the selected one at every entry point (the
//! Shoot button sends the selection, a preview comes from a row, the themed
//! pass selects the row it parks on), so there is no second range to serve and
//! no new `Event`.

use pundit_core::project::Project;
use uuid::Uuid;

/// The selected slate's range in concat time, `(from, to)`, or `None` when
/// there is nothing to draw.
///
/// `None` covers all three of: no selection, a selection whose slate has left
/// the project, and a **half-marked** slate — `i` stores a range with
/// `out_seconds: None`, and a range with no end is not a span. The caller turns
/// `None` into `to == from`, which is the scrubber's own "invisible".
///
/// **Two numbers, not a point:** the scrubber's `Mark` is drawn centred on its
/// value, so a span built as a wide `Mark` would start half its own length
/// before the in point.
pub fn slate_span(project: &Project, selected: Option<Uuid>) -> Option<(f64, f64)> {
    let selected = selected?;
    let slate = project.slates.iter().find(|s| s.id == selected)?;
    let out = slate.out_seconds?;
    // `abs_seconds` clamps the *index*, not the time: an index past the end
    // returns the whole timeline's duration plus the slate's own time, which
    // draws the span off the end of the bar rather than failing. Only a remap
    // bug can get here, so this is an assert and not a fallback — there is no
    // honest value to substitute.
    debug_assert!(
        slate.source_index < project.source_videos.len(),
        "slate {} is on source {}, of {}",
        slate.id,
        slate.source_index,
        project.source_videos.len()
    );
    Some((
        project.abs_seconds(slate.source_index, slate.in_seconds),
        project.abs_seconds(slate.source_index, out),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pundit_core::project::SourceRef;

    /// Two 600 s sources, so the second one's offset is visible.
    fn project() -> Project {
        let mut p = Project::new("p");
        for i in 0..2 {
            p.source_videos.push(SourceRef {
                relative_path: format!("{i}.mp4"),
                display_name: format!("video {}", i + 1),
                duration_seconds: 600.0,
                display_aspect: 16.0 / 9.0,
            });
        }
        p
    }

    fn mark(project: &mut Project, source_index: usize, in_seconds: f64, out: Option<f64>) -> Uuid {
        let id = project.mark_slate_in(source_index, in_seconds);
        if let Some(out) = out {
            project.mark_slate_out(source_index, out).unwrap();
        }
        id
    }

    #[test]
    fn a_timed_slate_spans_its_marks() {
        let mut p = project();
        let id = mark(&mut p, 0, 30.0, Some(45.0));
        assert_eq!(slate_span(&p, Some(id)), Some((30.0, 45.0)));
    }

    #[test]
    fn a_slate_on_the_second_video_is_offset_by_the_first() {
        let mut p = project();
        let id = mark(&mut p, 1, 30.0, Some(45.0));
        assert_eq!(slate_span(&p, Some(id)), Some((630.0, 645.0)));
    }

    #[test]
    fn a_half_marked_slate_has_no_span() {
        let mut p = project();
        let id = mark(&mut p, 0, 30.0, None);
        assert_eq!(slate_span(&p, Some(id)), None);
    }

    #[test]
    fn nothing_selected_and_a_slate_that_has_gone_both_draw_nothing() {
        let mut p = project();
        let id = mark(&mut p, 0, 30.0, Some(45.0));
        assert_eq!(slate_span(&p, None), None);
        p.slates.clear();
        assert_eq!(slate_span(&p, Some(id)), None);
    }
}

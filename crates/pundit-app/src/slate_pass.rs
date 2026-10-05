//! The themed pass: which range to park on when a take ends (BACKLOG #120).
//!
//! The coach (2026-10-02): *"i want to be able to filter slates on tags too,
//! then record all the slates with that tag"*, and, clarifying the same day:
//! *"it should only show the timed slates, and not 'free record' like the
//! current slate recording action."* The filter shipped with #120's first half; this is the
//! advance, and it is one function because the whole design is chosen so that
//! the bug the spec's own review found (T3) **cannot be written**.
//!
//! **The queue is the displayed list, and nothing is snapshotted.** The draft
//! held the ordered ids the pass started with, because stopping a take adds a
//! clip carrying `slate_id` — so with a queue of *unshot* slates, `[A,B,C]`,
//! shoot A, the queue becomes `[B,C]` and index + 1 is **C**: B is never
//! offered. A shot slate **stays** in the Slates list (a coach re-records), so
//! the list is a stable frame of reference and "the next candidate at or after
//! the current row" needs no snapshot, no pass mode to start and end, and no
//! second definition of what is in the pass.
//!
//! **It reads the row model** — `SlateRow`'s `timed` and `shot`, which T1
//! shipped for this — rather than rescanning the project, so the list and the
//! pass cannot disagree about what is in it, and the *filtered* list is what
//! the coach sees: a filter changed between two takes is honoured with no
//! state of its own (T7's filter is the Slates panel's own property, never the
//! window's `tag-filter`, which the bus writes).
//!
//! **A range marked *during* the pass is not a special case in either
//! direction**, and that is worth saying rather than leaving as an accident
//! (the plan's item 6). `i` and `o` are on the recording allow-list — a coach
//! spots the next moment while talking over this one — so it is reachable. With
//! a tag filter set the new range is **not** in the pass: a fresh mark carries
//! no tags, so it is filtered out of the list, and the list is the queue. With
//! no filter it is simply a row, picked up at the next advance if it falls at
//! or after the cursor.
//!
//! The rule lives here and not in `main.rs` because `main.rs` is wiring and
//! has no `#[cfg(test)]` module at all — [`crate::slate_span`]'s reason, and
//! [`crate::recents`]'s before it. It cannot live in `pundit-core` either, and
//! not only because the row model is the UI's: a `core` predicate over
//! `(&Slate, &[Clip])` would be exactly the second definition of "timed and
//! unshot" that `SlateRow`'s own comment forbids.

/// One row of the Slates list, as the pass reads it: the id to park on and the
/// two states T1 put on the row model for this.
///
/// A projection of `SlateRow` rather than a second row model — the window's
/// generated struct lives in the binary, which this library cannot see.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PassRow<'a> {
    pub id: &'a str,
    /// Both marks are set. A half-marked range has no end for the footage to
    /// stop at, so it is **shown and skipped** (spec T2): the row says which
    /// it is, and the coach can finish it or delete it.
    pub timed: bool,
    /// A clip was recorded from it (`Clip::slate_id`), which is what makes it
    /// done and is right across a delete, an undo and a re-record.
    pub shot: bool,
}

/// The row the pass parks on once the take on `current` has ended: the first
/// **timed and unshot** row at or **after** `current`'s place in the list.
///
/// **"At or after", not "after", and that is the whole trick.** After a
/// successful take the current row is `shot`, so the search passes over it by
/// itself and lands on the next range. After an **aborted** take — nothing
/// reached the muxer, so there is no clip — it is still a candidate and the
/// search returns it, so the pass re-parks on the range it failed to record
/// rather than skipping it. Both behaviours fall out of one rule; neither is
/// a case to remember.
///
/// `None` ends the pass quietly, and covers every reason there is nothing to
/// park on: no row is selected, the selected row has been filtered out of the
/// list (which `show_project` deliberately does *not* clear), or there is no
/// candidate left at or after it — the last range of a themed set, which is
/// where the pass is meant to stop.
pub fn next_slate<'a>(rows: &[PassRow<'a>], current: &str) -> Option<&'a str> {
    let at = rows.iter().position(|row| row.id == current)?;
    rows[at..]
        .iter()
        .find(|row| row.timed && !row.shot)
        .map(|row| row.id)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `a` shot, `b` and `c` waiting — the list as it stands the moment the
    /// first take of a three-range pass ends.
    fn rows() -> Vec<PassRow<'static>> {
        vec![
            PassRow {
                id: "a",
                timed: true,
                shot: true,
            },
            PassRow {
                id: "b",
                timed: true,
                shot: false,
            },
            PassRow {
                id: "c",
                timed: true,
                shot: false,
            },
        ]
    }

    /// The plan's floor, and the regression guard for T3: shoot the first of
    /// three and the **second** is next. An id snapshot answered `c` here.
    #[test]
    fn the_take_that_just_landed_hands_on_to_the_next_range() {
        assert_eq!(next_slate(&rows(), "a"), Some("b"));
    }

    /// Nothing reached the muxer, so no clip exists and the range is still
    /// waiting: the pass parks on it again rather than skipping it.
    #[test]
    fn an_aborted_take_leaves_the_pass_on_the_same_range() {
        let mut rows = rows();
        rows[0].shot = false;
        assert_eq!(next_slate(&rows, "a"), Some("a"));
    }

    /// A range with no out point has no end to stop at, so the pass passes
    /// over it — and the row says so, which is why it may stay in the list.
    #[test]
    fn a_half_marked_range_is_skipped() {
        let mut rows = rows();
        rows[1].timed = false;
        assert_eq!(next_slate(&rows, "a"), Some("c"));
    }

    /// The pass works forwards only. A range earlier in the match that was
    /// never shot is not reached by finishing a later one — the coach went
    /// past it on purpose, and the list is still there to go back to.
    #[test]
    fn a_waiting_range_before_the_current_one_is_not_gone_back_for() {
        let mut rows = rows();
        rows[0] = PassRow {
            id: "a0",
            timed: true,
            shot: false,
        };
        rows[1].shot = true;
        // Standing on `b`, which has just been shot: `a0` is behind it.
        assert_eq!(next_slate(&rows, "b"), Some("c"));
    }

    /// The last range of a themed set: the pass ends, and the row stays
    /// selected because there is nothing better to select.
    #[test]
    fn the_last_range_ends_the_pass() {
        let mut rows = rows();
        for row in &mut rows {
            row.shot = true;
        }
        assert_eq!(next_slate(&rows, "c"), None);
    }

    /// The list *is* the queue, so a filter typed between two takes changes
    /// what comes next with no state of its own. Here `b` left the list.
    #[test]
    fn a_filter_change_between_takes_is_honoured() {
        let rows = rows();
        let filtered = [rows[0], rows[2]];
        assert_eq!(next_slate(&filtered, "a"), Some("c"));
    }

    /// No selection, and a selection the filter has hidden: there is no cursor
    /// in this list, so there is nothing to advance from. A plain take — one
    /// started with no range selected — ends here.
    #[test]
    fn a_cursor_that_is_not_in_the_list_advances_nothing() {
        assert_eq!(next_slate(&rows(), ""), None);
        assert_eq!(next_slate(&rows(), "elsewhere"), None);
        assert_eq!(next_slate(&[], "a"), None);
    }
}

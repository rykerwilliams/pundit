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

/// The row a pass **starts** on (BACKLOG #128, `g` by default): the first
/// **timed and unshot** row of the list as it is displayed — filtered, when
/// the coach has typed a tag.
///
/// **It is the one rule [`next_slate`] is**, read from the top of the list
/// rather than from the cursor, and `next_slate` is written as exactly that, so
/// "what is a candidate" has one definition. That makes it the way back to a
/// range the pass went past, too: the advance only works forwards, and this
/// starts again from the first one still waiting wherever the selection is.
///
/// `None` is a list with nothing left to shoot in it — empty, every range
/// half-marked, or every one already shot, which [`all_shot`] tells apart.
pub fn first_slate<'a>(rows: &[PassRow<'a>]) -> Option<&'a str> {
    rows.iter()
        .find(|row| row.timed && !row.shot)
        .map(|row| row.id)
}

/// The row the pass parks on once the take on `current` has ended: the first
/// **timed and unshot** row at or **after** `current`'s place in the list —
/// [`first_slate`] of the list from the cursor on.
///
/// **"At or after", not "after", and that is the whole trick.** After a
/// successful take the current row is `shot`, so the search passes over it by
/// itself and lands on the next range. After an **aborted** take — nothing
/// reached the muxer, so there is no clip — it is still a candidate and the
/// search returns it, so the pass re-parks on the range it failed to record
/// rather than skipping it. Both behaviours fall out of one rule; neither is
/// a case to remember.
///
/// `None` ends the pass, and covers every reason there is nothing to park on:
/// no row is selected, the selected row has been filtered out of the list
/// (which `show_project` deliberately does *not* clear), or there is no
/// candidate left at or after it — the last range of a themed set, which is
/// where the pass is meant to stop. Only that last one is *said*, and
/// [`ran_out`] is what tells it from the other two.
pub fn next_slate<'a>(rows: &[PassRow<'a>], current: &str) -> Option<&'a str> {
    let at = rows.iter().position(|row| row.id == current)?;
    first_slate(&rows[at..])
}

/// How many ranges the list holds, when **every one of them is shot** — the
/// count the end-of-pass notice names. `None` while any timed row is still
/// waiting, and for a list with no timed row at all, where "all 0 shot" would
/// be a claim about nothing.
///
/// **Every timed row, not every one after the cursor.** The advance works
/// forwards, so it can run out with a range *behind* the cursor still unshot —
/// one the coach went past. "All 8 shot" would then be false, so the pass ends
/// as it always did, quietly, and `g` ([`first_slate`]) is the way back to it.
/// A half-marked row is in neither count: the pass never offers one.
pub fn all_shot(rows: &[PassRow]) -> Option<usize> {
    if rows.iter().any(|row| row.timed && !row.shot) {
        return None;
    }
    Some(rows.iter().filter(|row| row.timed).count()).filter(|&n| n > 0)
}

/// Whether the take on `current` has just **finished the pass** — the one
/// case of [`next_slate`]'s `None` that is said aloud (BACKLOG #128) — and
/// over how many ranges.
///
/// **The cursor has to be in the list**, which is what keeps the other two
/// `None`s quiet: a plain take (nothing selected) and a take on a range the
/// filter has since hidden are not the end of anything, even when the list
/// they are not in happens to be all shot. An **aborted** take can never land
/// here: its range is still unshot, so `all_shot` is `None` — and `next_slate`
/// re-parks on it, which is the same fact read the other way.
pub fn ran_out(rows: &[PassRow], current: &str) -> Option<usize> {
    if !rows.iter().any(|row| row.id == current) {
        return None;
    }
    all_shot(rows)
}

/// The status-bar line for a finished pass: **"All 8 'corner' ranges shot"**,
/// or **"All 8 ranges shot"** with no tag filter — the coach's own words
/// (2026-10-13), counting the timed rows of the list he is looking at.
///
/// **One range is "The one 'corner' range is shot"**, because "All 1" reads as
/// a slip. The tag is quoted exactly as typed into the filter, which is also
/// exactly what the filter matched on (`show_slates` compares it untrimmed).
pub fn finished_notice(ranges: usize, tag: &str) -> String {
    let tag = if tag.is_empty() {
        String::new()
    } else {
        format!("'{tag}' ")
    };
    if ranges == 1 {
        format!("The one {tag}range is shot")
    } else {
        format!("All {ranges} {tag}ranges shot")
    }
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

    /// The pass starts on the first range still waiting, from the top of the
    /// list — passing over the shot and the half-marked alike.
    #[test]
    fn the_first_range_waiting_starts_the_pass() {
        let mut rows = rows();
        rows[1].timed = false;
        assert_eq!(first_slate(&rows), Some("c"));
    }

    /// Starting is not advancing: a range *behind* the selection is where the
    /// pass starts, which is what makes `g` the way back to one the pass went
    /// past.
    #[test]
    fn starting_goes_back_for_a_range_the_pass_went_past() {
        let mut rows = rows();
        rows[0].shot = false;
        rows[1].shot = true;
        // From `b` the advance looks forwards; the start looks from the top.
        assert_eq!(next_slate(&rows, "b"), Some("c"));
        assert_eq!(first_slate(&rows), Some("a"));
    }

    /// Nothing left to shoot, nothing to start on.
    #[test]
    fn a_list_with_nothing_waiting_starts_nothing() {
        let mut rows = rows();
        rows[1].shot = true;
        rows[2].timed = false;
        assert_eq!(first_slate(&rows), None);
        assert_eq!(first_slate(&[]), None);
    }

    /// The last take of a set finishes the pass, and the count is the list's
    /// timed rows — the half-marked one is in the list and not in the count.
    #[test]
    fn the_last_take_of_a_set_finishes_the_pass() {
        let mut rows = rows();
        rows[1].shot = true;
        rows[2].shot = true;
        rows.push(PassRow {
            id: "open",
            timed: false,
            shot: false,
        });
        assert_eq!(ran_out(&rows, "c"), Some(3));
    }

    /// An aborted last take leaves its range unshot, so the pass re-parks on
    /// it and says nothing: the notice can never follow a failed take.
    #[test]
    fn an_aborted_last_take_does_not_finish_the_pass() {
        let mut rows = rows();
        rows[1].shot = true;
        assert_eq!(next_slate(&rows, "c"), Some("c"));
        assert_eq!(ran_out(&rows, "c"), None);
    }

    /// The advance ran out with a range behind the cursor still waiting:
    /// "all shot" would be false, so the pass ends quietly, as it always did.
    #[test]
    fn a_range_left_behind_keeps_the_end_quiet() {
        let mut rows = rows();
        rows[0].shot = false;
        rows[1].shot = true;
        rows[2].shot = true;
        assert_eq!(next_slate(&rows, "c"), None);
        assert_eq!(ran_out(&rows, "c"), None);
    }

    /// The two quiet `None`s of `next_slate` stay quiet even over a list that
    /// is all shot: a plain take, or a take on a range the filter has hidden,
    /// is not the end of a pass.
    #[test]
    fn a_take_from_outside_the_list_finishes_nothing() {
        let mut rows = rows();
        for row in &mut rows {
            row.shot = true;
        }
        assert_eq!(ran_out(&rows, ""), None);
        assert_eq!(ran_out(&rows, "elsewhere"), None);
    }

    /// A list of half-marked ranges is not "all 0 shot".
    #[test]
    fn a_list_with_no_timed_range_is_not_all_shot() {
        let rows = [PassRow {
            id: "a",
            timed: false,
            shot: true,
        }];
        assert_eq!(all_shot(&rows), None);
        assert_eq!(all_shot(&[]), None);
    }

    /// The coach's wording, with and without the tag filter.
    #[test]
    fn the_notice_names_the_count_and_the_tag() {
        assert_eq!(finished_notice(8, "corner"), "All 8 'corner' ranges shot");
        assert_eq!(finished_notice(8, ""), "All 8 ranges shot");
    }

    /// One range is not "All 1".
    #[test]
    fn one_range_is_not_all_one() {
        assert_eq!(
            finished_notice(1, "corner"),
            "The one 'corner' range is shot"
        );
        assert_eq!(finished_notice(1, ""), "The one range is shot");
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

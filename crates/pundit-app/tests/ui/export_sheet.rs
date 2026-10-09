//! The export sheet's three switches and the two lines under its Scoreboard
//! picker, driven through the **real** sheet (#78).
//!
//! **What this closes.** The sheet's controls are wired to the window with
//! `<=>`, and a wire that is missing leaves a control that moves on screen and
//! changes nothing — which until now no test in this crate could see. The
//! window root can only read its own `export-chapters`; the leaf the wire feeds
//! is a `CheckBox.checked` inside `ExportSheet`, which has no id the root can
//! name. So the four controls that shipped before these two (#78's plan says so
//! plainly) had no test of their wires or their seeding either.
//!
//! **What reaches it is the accessibility tree**, which `build.rs` now compiles
//! the debug info for. Every style's `CheckBox` binds
//! `accessible-checked <=> root.checked`, `accessible-label: root.text` and
//! `accessible-enabled: root.enabled`, and a `Text` element binds
//! `accessible-label: text` by default — so the words on the row find the
//! control, and what the control reads back is the property the wire actually
//! arrived at.
//!
//! **No synthesized clicks.** A tick is
//! `invoke_accessible_default_action`, which is the `CheckBox`'s own
//! `accessible-action-default` calling the touch area's `clicked`. A pointer
//! click would have to land on a computed position inside a sheet behind a
//! modal `Scrim` that covers the window — a layout measurement this test does
//! not want to be a test of.
//!
//! **What is still not covered**, and the plan says so too: `main.rs`'s
//! `open_export_sheet`, which is where the six controls are *seeded* from the
//! project's `Preferences`. It reads the `UI` thread-local, so it is not
//! reachable from here. The window properties this file drives are exactly the
//! ones that function sets, which is as close as a UI test gets; a
//! `set_export_cues` missing from it would still pass everything below.

use crate::AppWindow;
use i_slint_backend_testing::{ElementHandle, ElementQuery};
use slint::ComponentHandle;
use std::cell::RefCell;
use std::rc::Rc;

/// The words on each switch's row — which are also how it is found, so these
/// strings are the test's half of a contract with `app.slint`.
const CHAPTERS: &str = "Chapters, in the file and as a list beside it";
const CUES: &str = "Scoreboard subtitles — an .srt beside the file, and a track inside a copy";
const MUTE: &str = "Mute source audio";

/// The explanatory line that was already there, and the one spec S4 added.
const COPY_LINE: &str = "The whole match is copied, not re-encoded: player highlights and pen drawings can't ride a copy.";
const NO_BOARD_LINE: &str =
    "This export carries no scoreboard at all: not in the picture, not beside the file, not inside it.";

/// One switch: the words on it, and the window property its `<=>` is supposed
/// to reach. Paired here so a crossed wire is a failure rather than a reading.
struct Switch {
    label: &'static str,
    get: fn(&AppWindow) -> bool,
    set: fn(&AppWindow, bool),
}

/// All three, in the order the sheet shows them.
fn switches() -> [Switch; 3] {
    [
        Switch {
            label: CHAPTERS,
            get: AppWindow::get_export_chapters,
            set: AppWindow::set_export_chapters,
        },
        Switch {
            label: CUES,
            get: AppWindow::get_export_cues,
            set: AppWindow::set_export_cues,
        },
        Switch {
            label: MUTE,
            get: AppWindow::get_export_mute_source,
            set: AppWindow::set_export_mute_source,
        },
    ]
}

/// The window with the export sheet up. **`export-sheet-open` is the only door
/// the sheet has that a test can use** — `main.rs`'s `open_export_sheet` is the
/// app's, and it seeds the controls on the way through; here they are set
/// directly, because what is under test is the sheet's wiring and not that
/// function's.
fn sheet() -> AppWindow {
    i_slint_backend_testing::init_no_event_loop();
    let w = AppWindow::new().unwrap();
    w.set_export_sheet_open(true);
    w.show().unwrap();
    w
}

/// The one control reading `label`. **Uniqueness is asserted, not assumed:**
/// the basket sheet has a "Mute source audio" of its own, and this finding
/// exactly one is what says that sheet is not instantiated — a second match
/// would make every assertion below a test of whichever came first.
fn switch(w: &AppWindow, label: &str) -> ElementHandle {
    let mut found = ElementHandle::find_by_accessible_label(w, label);
    let one = found
        .next()
        .unwrap_or_else(|| panic!("no control reading {label:?} in the open export sheet"));
    assert!(
        found.next().is_none(),
        "{label:?} matches more than one element in this window"
    );
    one
}

/// What the row on screen shows, as the accessibility tree reports it — the far
/// end of the `<=>` chain the window property starts.
fn ticked(w: &AppWindow, label: &str) -> bool {
    switch(w, label)
        .accessible_checked()
        .unwrap_or_else(|| panic!("{label:?} is not a checkbox"))
}

/// Whether a line with these words is on screen at all. The lines are inside
/// `if` bodies, so an absent one is an element that does not exist.
fn shows(w: &AppWindow, words: &str) -> bool {
    ElementHandle::find_by_accessible_label(w, words)
        .next()
        .is_some()
}

/// Turns every switch on, then `off` alone.
fn only_off(w: &AppWindow, off: &str) {
    for s in switches() {
        (s.set)(w, true);
    }
    for s in switches() {
        if s.label == off {
            (s.set)(w, false);
        }
    }
}

/// **The downward half of each wire, and that it ends at the right property.**
/// Each switch is turned off alone with the other two left on: a dead wire
/// leaves its row still ticked, and a wire crossed with another switch's
/// unticks the wrong row. Both are invisible to every other test in this
/// crate.
#[test]
fn every_switch_shows_its_own_window_property() {
    let w = sheet();

    for turned_off in switches() {
        only_off(&w, turned_off.label);

        for s in switches() {
            assert_eq!(
                ticked(&w, s.label),
                s.label != turned_off.label,
                "with only {:?} turned off, the row {:?} reads wrong",
                turned_off.label,
                s.label
            );
        }
    }
}

/// **The upward half**, which is the half a coach notices: a tick that reaches
/// nothing leaves the sheet exporting what he just turned off. Each switch is
/// ticked from a window where all three are on, and the other two must not
/// move with it.
#[test]
fn ticking_a_switch_reaches_its_own_window_property() {
    let w = sheet();

    for clicked in switches() {
        for s in switches() {
            (s.set)(&w, true);
        }

        switch(&w, clicked.label).invoke_accessible_default_action();

        for s in switches() {
            assert_eq!(
                (s.get)(&w),
                s.label != clicked.label,
                "after ticking {:?}, the window's value behind {:?} reads wrong",
                clicked.label,
                s.label
            );
        }
    }
}

/// **A run leaves all three live, and that inverts what this test used to
/// assert** (BACKLOG #77 task 3).
///
/// It read *"a run settles all three, as it settles the pickers: the choices a
/// job was built from must not change under it"*, and the export queue makes
/// that false. Each **queued** job froze its own six controls at enqueue, so
/// what the live controls describe is the **next** enqueue and nothing that is
/// rendering — and `Add to queue` is live during a run, so settling them would
/// mean a coach could queue nothing but the choices already on the sheet.
///
/// `Export` and `Start queue` are the only things a run disables now.
///
/// **The three checkboxes and not the pickers.** `CLAUDE.md`'s accessibility
/// guarantee — `accessible-label` / `-checked` / `-enabled` on every style's
/// control — is stated for `CheckBox`; the three pickers are `ComboBox`es, and
/// `a_combo_box_reports_whether_it_is_enabled` below is what says whether this
/// can honestly cover them too. They came off the same seven-line edit either
/// way.
#[test]
fn a_run_leaves_every_switch_live() {
    let w = sheet();
    w.set_exporting(true);

    for s in switches() {
        assert_eq!(
            switch(&w, s.label).accessible_enabled(),
            Some(true),
            "{:?} is settled during a run, so a coach can't queue anything else",
            s.label
        );
    }
}

/// **Whether a `ComboBox` answers `accessible_enabled` at all**, asked rather
/// than assumed — the review of #77's plan flagged that the repo's stated
/// guarantee covers `CheckBox` and the pickers are not one.
///
/// It is written as a question with either answer recorded, because what it
/// settles is whether a future test *may* assert on a picker's enabled state.
/// If this starts failing, a `ComboBox` has gained or lost that reporting and
/// the note above is what needs re-reading.
#[test]
fn a_combo_box_reports_whether_it_is_enabled() {
    let w = sheet();
    let found = ElementQuery::from_root(&w)
        .match_type_name("ComboBox")
        .find_all();
    assert!(
        !found.is_empty(),
        "the sheet's three pickers are not ComboBoxes any more"
    );
    // The pickers are live, and a run no longer settles them (#77 task 3).
    w.set_exporting(true);
    assert!(
        found
            .iter()
            .all(|picker| picker.accessible_enabled() != Some(false)),
        "a picker reports itself disabled during a run"
    );
}

/// **Spec S4's table, which is the whole of `would-copy` and the new line.**
///
/// The rows that earn their place: *Default* with the subtitles **off** shows
/// **neither** line, because a copy would then carry no board anywhere and
/// Default therefore burns it in instead — that is the one case the old
/// condition (`scoreboard != 1`) got wrong, and with the subtitles on the new
/// condition is the old one exactly. *Separate track* with them off shows
/// **both**: it does copy, and it does carry no board, and both facts are
/// true at once.
#[test]
fn the_lines_under_the_picker_follow_the_effective_mode() {
    let w = sheet();
    w.set_export_whole_match_ticked(true);

    // (the picker, the subtitles) -> (the copy line, the no-board line)
    let table = [
        (0, true, true, false),
        (0, false, false, false),
        (1, true, false, false),
        (1, false, false, false),
        (2, true, true, false),
        (2, false, true, true),
    ];
    for (scoreboard, cues, copy_line, no_board_line) in table {
        w.set_export_scoreboard(scoreboard);
        w.set_export_cues(cues);

        assert_eq!(
            shows(&w, COPY_LINE),
            copy_line,
            "the copy line is wrong for picker {scoreboard}, subtitles {cues}"
        );
        assert_eq!(
            shows(&w, NO_BOARD_LINE),
            no_board_line,
            "the no-board line is wrong for picker {scoreboard}, subtitles {cues}"
        );
    }
}

/// **Neither line is about anything but the whole match**, which is the only
/// target that is ever copied. The combination that shows both is used, so a
/// condition that had dropped the tick would show both here.
#[test]
fn neither_line_shows_without_the_whole_match() {
    let w = sheet();
    w.set_export_whole_match_ticked(false);
    w.set_export_scoreboard(2);
    w.set_export_cues(false);

    assert!(
        !shows(&w, COPY_LINE),
        "the copy line shows with nothing to copy"
    );
    assert!(
        !shows(&w, NO_BOARD_LINE),
        "the no-board line shows with nothing to export"
    );
}

/// **The sheet an export is set up in needs no scrolling**, at the smallest
/// window the app allows — which is the one thing two more rows could have cost
/// and the plan asked to be a measurement rather than the spec's estimate.
///
/// **This no longer reads the card's height, because that number stopped
/// answering the question.** `Sheet` is capped at the window and scrolls its
/// body (BACKLOG #134), so the card is 660px whether its content is 660px or
/// 823px, and `height <= min_h` is now true by construction. What fitting
/// *means* here is that nothing had to be scrolled to reach the buttons, so
/// that is what is asked: is the button row on screen with the body untouched?
/// The general rule — no sheet, of the six, is ever taller than its window —
/// is `tests/ui/sheet_scroll.rs`, and this stays as the narrower claim about
/// the one sheet two of #78's rows went into.
///
/// **Found by the word "Close"**, the one word in that row nothing else in this
/// sheet says: "Export" is also its heading, and a test satisfied by the title
/// would pass with no buttons at all. The count is asserted, so a second
/// "Close" appearing somewhere would be a failure rather than a reading of
/// whichever came first.
///
/// **Loaded with the worst case the switches are actually set in:** the target
/// list past its own 210px cap and **both** lines under the picker at once,
/// which is *Separate track* with the subtitles off. There is deliberately no
/// run list, and that is the honest boundary of this test rather than a softer
/// case chosen to pass — **a sheet reopened over a running export is taller
/// than a 700px window and does scroll**, which is #134's own test. Measured
/// at the minimum with twelve targets, 2026-10-08: **589px** of content with
/// one explanatory line and no run, **623px** with both, **798px** with a run
/// list at its 140px cap and **823px** with the finish line as well, against
/// the 700px window less the scrim's 20px either side. (The 614/648/733 series
/// this doc carried before was right about every delta and 25px high at the
/// base; #134 records the correction.)
///
/// **It reads no number, and #77's plan review had to point that out** — the
/// plan called this "the fit test's number" and asked for a re-measure. The
/// figures above are documentation; the one assertion is that exactly one
/// `"Close"` is findable at the floor. **And this case carries no queue rows
/// either**, for the run list's own stated reason: that section is absent at
/// zero, so what is measured stays the sheet a coach *configures* an export
/// in. #77 added a fourth button to the action row and the card stayed 480px,
/// which is what this still passing says.
#[test]
fn the_sheet_an_export_is_set_up_in_needs_no_scrolling() {
    let w = sheet();
    // Past the list's cap, so the height is the cap and not the row count.
    w.set_export_targets(slint::ModelRc::new(slint::VecModel::from(
        (0..12)
            .map(|i| crate::TargetRow {
                label: format!("Tag {i}").into(),
                detail: "4 clips · 3:12".into(),
                ticked: true,
            })
            .collect::<Vec<_>>(),
    )));
    w.set_export_whole_match_ticked(true);
    w.set_export_scoreboard(2);
    w.set_export_cues(false);
    // The floor the window itself declares, which is what the card has to fit.
    let (min_w, min_h) = (w.get_min_window_width(), w.get_min_window_height());
    w.window().set_size(slint::LogicalSize::new(min_w, min_h));

    assert!(
        shows(&w, COPY_LINE) && shows(&w, NO_BOARD_LINE),
        "the case this measures is supposed to show both lines"
    );

    let on_screen = ElementHandle::find_by_accessible_label(&w, "Close").count();
    assert_eq!(
        on_screen, 1,
        "the export sheet's button row is not on screen in a {min_w}x{min_h} \
         window without scrolling the body to it"
    );
}

/// A queue row, as Rust hands it over.
fn queue_line(label: &str, match_label: &str) -> crate::QueueLine {
    crate::QueueLine {
        match_label: match_label.into(),
        label: label.into(),
        length: "1:00".into(),
    }
}

fn with_queue(w: &AppWindow, rows: &[crate::QueueLine]) {
    w.set_export_queue(slint::ModelRc::new(slint::VecModel::from(rows.to_vec())));
}

/// **`Add to queue` exists, and is live while a run is going** (#77 spec Q6).
///
/// The one assertion that would have caught the gap the plan shipped with: it
/// specified the Queue section and `Start queue` and **no way to enqueue**, and
/// it could not have been recovered by analogy later — the section is hidden at
/// zero rows, so the control cannot live in it or there would be nothing to
/// click with an empty queue.
///
/// Live during a run because an enqueue renders nothing. `Export`, beside it,
/// is absent during one.
#[test]
fn add_to_queue_is_there_and_stays_live_during_a_run() {
    let w = sheet();
    w.set_export_any_ticked(true);

    // **Clicked, not merely found.** A button wired to nothing is exactly the
    // gap that let this control be left out of the plan in the first place.
    let fired: Rc<RefCell<usize>> = Rc::default();
    w.on_add_to_queue({
        let fired = Rc::clone(&fired);
        move || *fired.borrow_mut() += 1
    });
    let button = switch(&w, "Add to queue");
    assert_eq!(button.accessible_enabled(), Some(true));
    button.invoke_accessible_default_action();
    assert_eq!(*fired.borrow(), 1, "Add to queue is wired to nothing");

    w.set_exporting(true);
    assert_eq!(
        switch(&w, "Add to queue").accessible_enabled(),
        Some(true),
        "an enqueue renders nothing, so a run must not settle it"
    );
    // **Scoped to Buttons, because the sheet's own title reads "Export" too**
    // — a `Text` carries an `accessible-label` by default, so the unscoped
    // query finds the heading and this assertion would never fail.
    assert_eq!(
        ElementQuery::from_root(&w)
            .match_type_name("Button")
            .match_predicate(|e| e.accessible_label().as_deref() == Some("Export"))
            .find_all()
            .len(),
        0,
        "Export is supposed to be absent during a run, not merely greyed"
    );

    // Nothing ticked is the one thing that does settle it, as it settles
    // Export.
    w.set_export_any_ticked(false);
    assert_eq!(
        switch(&w, "Add to queue").accessible_enabled(),
        Some(false),
        "there is nothing to freeze"
    );
}

/// **The Queue section is absent at zero rows and present at one**, as the Run
/// section is — which is the whole reason `Add to queue` lives in the action
/// row instead.
///
/// Found by the heading, which carries the count: a coach who has queued three
/// should not have to count rows.
#[test]
fn the_queue_section_appears_with_its_first_row() {
    let w = sheet();
    assert_eq!(
        ElementHandle::find_by_accessible_label(&w, "Start queue").count(),
        0,
        "an empty queue shows a Start for nothing"
    );

    with_queue(
        &w,
        &[
            queue_line("All clips", "Rovers v Athletic"),
            queue_line("Corners", "Rovers v Athletic"),
        ],
    );
    assert_eq!(
        switch(&w, "Queue (2)").accessible_label().as_deref(),
        Some("Queue (2)"),
        "the heading does not count the rows"
    );
    assert_eq!(
        switch(&w, "Start queue").accessible_enabled(),
        Some(true),
        "Start queue is greyed with rows waiting"
    );

    // A row reads as its target and its match, which is what makes three
    // projects' "All clips" tell apart (spec Q3).
    assert_eq!(
        ElementHandle::find_by_accessible_label(&w, "All clips — Rovers v Athletic").count(),
        1,
        "the row does not name its match beside its target"
    );
}

/// **`Start queue` is the other thing a run settles**, and the only one besides
/// `Export`.
#[test]
fn a_run_settles_start_queue_and_nothing_else_in_the_section() {
    let w = sheet();
    with_queue(&w, &[queue_line("All clips", "Rovers v Athletic")]);
    w.set_exporting(true);

    assert_eq!(
        switch(&w, "Start queue").accessible_enabled(),
        Some(false),
        "a second run would be refused by the bus anyway; the button says so"
    );
    // The row's own ✕ stays live: what it takes out is *waiting*, which a run
    // does not touch.
    let crosses = ElementQuery::from_root(&w)
        .match_type_name("Button")
        .match_predicate(|e| e.accessible_label().as_deref() == Some("✕"))
        .find_all();
    assert_eq!(crosses.len(), 1, "one ✕ per queued row");
    assert_eq!(
        crosses[0].accessible_enabled(),
        Some(true),
        "a run settled a row it is not rendering"
    );
}

/// **A row's ✕ carries that row's index.** The assertion a single-row test
/// cannot make: with one row, index 0 is also the only answer a broken wire
/// could give.
#[test]
fn the_cross_removes_the_row_it_is_on() {
    let w = sheet();
    with_queue(
        &w,
        &[
            queue_line("All clips", "A"),
            queue_line("Corners", "A"),
            queue_line("Whole match", "B"),
        ],
    );
    let removed: Rc<RefCell<Vec<i32>>> = Rc::default();
    w.on_remove_from_queue({
        let removed = Rc::clone(&removed);
        move |index| removed.borrow_mut().push(index)
    });

    let crosses = ElementQuery::from_root(&w)
        .match_type_name("Button")
        .match_predicate(|e| e.accessible_label().as_deref() == Some("✕"))
        .find_all();
    assert_eq!(crosses.len(), 3);
    crosses[1].invoke_accessible_default_action();
    assert_eq!(
        *removed.borrow(),
        [1],
        "the middle row's ✕ removed something else"
    );
}

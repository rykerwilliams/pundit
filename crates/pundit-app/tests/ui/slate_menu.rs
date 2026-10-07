//! The slate row's right-click menu, driven through the real menu (BACKLOG
//! #127): right-click the row, find the item by the words on it, click it.
//!
//! **Why it is driven and not simulated.** A `MenuItem`'s `activated` body is
//! reachable from nowhere but the menu — it is not a callback the window
//! exposes — so a test that called a stand-in would be testing its own copy of
//! the rule. `build.rs` compiles the UI with Slint's debug info for this, which
//! is what `i_slint_backend_testing`'s `ElementHandle` needs; its own comment
//! has the measurement.
//!
//! **The rule under test is what a context menu may change.** Two of the four
//! items — *Record* and *Preview slate* — **arm** a range, and the span drawn
//! on the scrubber is the **selected** slate's (spec S6 of
//! `2026-10-02-slate-workflow-design.md`). A menu acting on a row that is not
//! selected would leave the arm and the span naming different ranges: the
//! footage would stop at a mark nothing on screen shows. The selection is also
//! the themed pass's cursor (#120), so each of those items moves it, and the
//! other two must not.

use crate::{AppWindow, SlateRow};
use i_slint_backend_testing::ElementHandle;
use slint::platform::PointerEventButton;
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::cell::RefCell;
use std::rc::Rc;

fn slate(id: &str) -> SlateRow {
    SlateRow {
        id: id.into(),
        range: "0:05–0:12".into(),
        video: SharedString::new(),
        name: "corner".into(),
        tags: SharedString::new(),
        shot: false,
        timed: true,
    }
}

/// Two ranges with the **first** selected, and every command the menu can send
/// recorded. The second row is therefore the one that is not selected, which
/// is the whole point of the fixture.
fn window() -> (AppWindow, Rc<RefCell<Vec<String>>>) {
    i_slint_backend_testing::init_no_event_loop();
    let w = AppWindow::new().unwrap();
    let sent: Rc<RefCell<Vec<String>>> = Rc::default();
    let record = |sent: &Rc<RefCell<Vec<String>>>, what: &'static str| {
        let sent = Rc::clone(sent);
        move |id: SharedString| sent.borrow_mut().push(format!("{what} {id}"))
    };
    w.on_shoot_slate(record(&sent, "shoot"));
    w.on_preview_slate(record(&sent, "preview"));
    w.on_jump_to_slate(record(&sent, "jump"));
    w.on_delete_slate(record(&sent, "delete"));
    w.set_can_play(true);
    w.set_slates(ModelRc::new(VecModel::from(vec![slate("s1"), slate("s2")])));
    w.set_selected_slate("s1".into());
    w.show().unwrap();
    (w, sent)
}

/// Opens the menu of the slate row at `index` and clicks the item that reads
/// `title`, or says why it could not.
///
/// The rows are found by the element id `app.slint` gives the row's own
/// `ContextMenuArea`: every `LineEdit` carries one too (Undo, Redo, Cut…), so
/// the type name alone finds six in this window. `find_by_element_id` walks the
/// item tree, so the two come back in the model's order — asserted here rather
/// than assumed, because an index silently off by one would make every
/// assertion below a test of the wrong row.
///
/// The item is then found under that same `ContextMenuArea`, which is what the
/// menu's popup hangs off, and clicked where a coach would click it: a press
/// and a release on the words. `MenuItemBase` activates on the release and only
/// after a press of its own, so nothing shorter than a real click will do.
fn pick(w: &AppWindow, index: usize, title: &str) {
    let rows: Vec<ElementHandle> =
        ElementHandle::find_by_element_id(w, "AppWindow::slate-menu").collect();
    assert_eq!(rows.len(), 2, "one menu per slate row");
    assert!(
        rows[0].absolute_position().y < rows[1].absolute_position().y,
        "the rows should come back top to bottom, i.e. in the model's order"
    );
    let row = &rows[index];
    row.mock_single_click(PointerEventButton::Right);
    let item = row
        .query_descendants()
        .match_predicate({
            let title = title.to_string();
            move |e| e.accessible_label().is_some_and(|l| l == title.as_str())
        })
        .find_first()
        .unwrap_or_else(|| panic!("no {title:?} in the menu"));
    item.mock_single_click(PointerEventButton::Left);
}

/// **Record from a row that is not selected selects it first.** Without that
/// the take is shot from the second range while the scrubber draws the first,
/// so the footage stops at an out point nothing on screen marks — and the
/// themed pass, whose cursor is the selection, carries on from a range the
/// coach never shot.
#[test]
fn recording_from_the_menu_selects_the_row_it_shoots() {
    let (w, sent) = window();

    pick(&w, 1, "Record");

    assert_eq!(*sent.borrow(), ["shoot s2"]);
    assert_eq!(w.get_selected_slate(), "s2");
}

/// **And so does Preview**, for the half of the reason that is the span: a
/// preview is a range played to its out point, where the footage stops, and
/// the span is how the coach sees where that is. It is the same thing the
/// row's double-click does, which gets there by selecting on the click Slint
/// delivers first.
#[test]
fn previewing_from_the_menu_selects_the_row_it_arms() {
    let (w, sent) = window();

    pick(&w, 1, "Preview slate");

    assert_eq!(*sent.borrow(), ["preview s2"]);
    assert_eq!(w.get_selected_slate(), "s2");
}

/// **The other two leave the selection where it was**, and that is the rule's
/// other half rather than an omission. Neither arms anything, so neither can
/// disagree with the span; and the selection is the themed pass's cursor, so
/// looking at where a range starts — or deleting one — must not move the pass
/// to it. "Jump to clip start" on the clip row selects nothing either.
///
/// One window for both, because neither is supposed to change anything a
/// second pick could read differently — `init_no_event_loop` is once per
/// thread, and a test's thread is its own (`main.rs`'s header).
#[test]
fn jumping_and_deleting_leave_the_selection_alone() {
    let (w, sent) = window();

    pick(&w, 1, "Jump to slate start");
    pick(&w, 1, "Delete slate");

    assert_eq!(*sent.borrow(), ["jump s2", "delete s2"]);
    assert_eq!(w.get_selected_slate(), "s1");
}

/// **Both arming items are greyed while a clip preview is open**, which is the
/// editor's own Record button's gate: the bus refuses a take with a preview
/// open (`can_record`), and `slate_out_reached` refuses to fire while one is,
/// so an arm set here would never be spent and `J`/`L` would stay refused with
/// it. A disabled `MenuItem` swallows the click, so neither the command nor the
/// selection moves.
#[test]
fn a_clip_preview_greys_out_both_arming_items() {
    let (w, sent) = window();
    w.set_previewing_clip("c1".into());

    pick(&w, 1, "Record");
    pick(&w, 1, "Preview slate");

    assert!(
        sent.borrow().is_empty(),
        "an arming item fired over an open preview: {:?}",
        sent.borrow()
    );
    assert_eq!(w.get_selected_slate(), "s1");
}

//! The line editor sheet (BACKLOG #97): one sheet, two lists. What does not
//! know which list is open — the field, the box, the Esc branch, the card's
//! scrolling — is tested **once** (spec Testing); `text_editing.rs` and
//! `sheet_scroll.rs` hold the field and the scrolling.

use crate::{AppWindow, LineEditorRow};
use i_slint_backend_testing::ElementHandle;
use slint::platform::{Key, PointerEventButton, WindowEvent};
use slint::{ComponentHandle, ModelRc, PhysicalSize, SharedString, VecModel};
use std::cell::RefCell;
use std::rc::Rc;

fn press(w: &AppWindow, key: impl Into<SharedString> + Clone) {
    w.window().dispatch_event(WindowEvent::KeyPressed {
        text: key.clone().into(),
    });
    w.window()
        .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
}

/// The one element with this id, or a message naming it.
fn one(w: &AppWindow, id: &str) -> ElementHandle {
    let mut found = ElementHandle::find_by_element_id(w, id);
    let it = found
        .next()
        .unwrap_or_else(|| panic!("no element {id:?} on screen"));
    assert!(
        found.next().is_none(),
        "{id:?} matches more than one element"
    );
    it
}

/// The sheet open on one selected row, and every callback a key could reach
/// recorded.
fn editor() -> (AppWindow, Rc<RefCell<Vec<&'static str>>>) {
    i_slint_backend_testing::init_no_event_loop();
    let w = AppWindow::new().unwrap();
    let fired: Rc<RefCell<Vec<&'static str>>> = Rc::default();
    w.on_reseed_match_line({
        let fired = Rc::clone(&fired);
        move || fired.borrow_mut().push("reseed")
    });
    w.set_line_editor_rows(ModelRc::new(VecModel::from(vec![LineEditorRow {
        id: "e1".into(),
        where_text: "1 · 14:05.0".into(),
        label: "Home goal".into(),
        role_less: false,
    }])));
    w.set_line_editor_selected("e1".into());
    w.set_line_editor_line("1 14:05.0 home goal".into());
    w.set_line_editor_open(true);
    w.show().unwrap();
    w.window().set_size(PhysicalSize::new(1600, 1200));
    (w, fired)
}

/// **Esc in a row's field cancels** — the line goes back to what the row was
/// seeded with and the sheet stays — and the next Esc, with no field focused,
/// closes it. Without `handle-key`'s reseed arm the first Esc would go to the
/// field, the half-typed line would stay, and leaving the field would commit
/// it.
#[test]
fn esc_in_the_row_field_reseeds_and_the_second_closes() {
    let (w, fired) = editor();
    one(&w, "LineEditorSheet::line-edit").mock_single_click(PointerEventButton::Left);
    press(&w, "x");
    assert!(
        w.get_line_editor_line_focused(),
        "the row's field has focus"
    );

    press(&w, Key::Escape);
    assert_eq!(*fired.borrow(), ["reseed"], "Esc puts the seeded line back");
    assert!(w.get_line_editor_open(), "and leaves the sheet open");

    press(&w, Key::Escape);
    assert!(!w.get_line_editor_open(), "the second Esc closes the sheet");
}

//! The line editor sheet (BACKLOG #97): one sheet, two lists. What does not
//! know which list is open — the field, the box, the Esc branch, the card's
//! scrolling — is tested **once** (spec Testing); `text_editing.rs` and
//! `sheet_scroll.rs` hold the field and the scrolling.

use crate::{AppWindow, LineEditorKind, LineEditorRow};
use i_slint_backend_testing::{ElementHandle, ElementQuery};
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

// ------------------------------------------------------- the slates kind

/// The element whose accessible label is `label`, if one is on screen.
fn labelled(w: &AppWindow, label: &str) -> Option<ElementHandle> {
    ElementHandle::find_by_accessible_label(w, label).next()
}

/// Whether any element's label starts with `prefix`.
fn any_label_starts(w: &AppWindow, prefix: &str) -> bool {
    let prefix = prefix.to_string();
    ElementQuery::from_root(w)
        .match_predicate(move |e| {
            e.accessible_label()
                .is_some_and(|l| l.starts_with(prefix.as_str()))
        })
        .find_first()
        .is_some()
}

/// A bare window whose two seeding callbacks are recorded.
fn bare() -> (AppWindow, Rc<RefCell<Vec<&'static str>>>) {
    i_slint_backend_testing::init_no_event_loop();
    let w = AppWindow::new().unwrap();
    let fired: Rc<RefCell<Vec<&'static str>>> = Rc::default();
    let record = |what: &'static str| {
        let fired = Rc::clone(&fired);
        move || fired.borrow_mut().push(what)
    };
    w.on_open_match_editor(record("seed events"));
    w.on_open_slate_editor(record("seed slates"));
    w.show().unwrap();
    w.window().set_size(PhysicalSize::new(1600, 1600));
    (w, fired)
}

/// **Type… is there before the first slate is**, which is when the first paste
/// comes (spec S8): the Slates header shows once the project has a video, and
/// the "No slates tagged" line does not, since no filter is set. Pressing it
/// seeds the slates and opens the sheet on them. With no video there is no
/// Slates section at all.
#[test]
fn type_is_there_with_no_slates() {
    let (w, fired) = bare();
    w.set_can_play(true);
    assert!(labelled(&w, "Type…").is_none(), "no video, no section");

    crate::one_source(&w);
    let button = labelled(&w, "Type…").expect("Type… with no slates");
    assert!(
        !any_label_starts(&w, "No slates tagged"),
        "with no filter, there is nothing to explain"
    );

    button.invoke_accessible_default_action();
    assert_eq!(*fired.borrow(), ["seed slates"]);
    assert!(w.get_line_editor_open());
    assert_eq!(w.get_line_editor_kind(), LineEditorKind::Slates);
}

/// Each kind reads its own words: the title, the paste heading and the hint.
#[test]
fn each_kind_reads_its_own_words() {
    let (w, _fired) = bare();

    w.invoke_open_line_editor(LineEditorKind::Events);
    assert!(labelled(&w, "Match events").is_some(), "the events' title");
    assert!(labelled(&w, "Paste events").is_some());
    assert!(any_label_starts(&w, "One event per line"));
    assert!(labelled(&w, "Slates").is_none());
    w.set_line_editor_open(false);

    w.invoke_open_line_editor(LineEditorKind::Slates);
    assert!(labelled(&w, "Slates").is_some(), "the slates' title");
    assert!(labelled(&w, "Paste slates").is_some());
    assert!(any_label_starts(&w, "One slate per line"));
    assert!(labelled(&w, "Match events").is_none());
}

/// **A slate's Go selects it, then jumps** (spec S17): the selection is what
/// moves the scrubber's span to the range and what `R` shoots, so it has to be
/// in place when the jump goes out. The picture is behind the sheet, so it
/// closes.
#[test]
fn go_on_a_slate_selects_it_before_it_jumps() {
    let (w, _fired) = bare();
    let jumped: Rc<RefCell<Vec<(String, String)>>> = Rc::default();
    w.on_jump_to_slate({
        let (weak, jumped) = (w.as_weak(), Rc::clone(&jumped));
        move |id| {
            let selected = weak.upgrade().unwrap().get_selected_slate();
            jumped.borrow_mut().push((id.into(), selected.into()));
        }
    });
    w.invoke_open_line_editor(LineEditorKind::Slates);
    w.set_line_editor_rows(ModelRc::new(VecModel::from(vec![LineEditorRow {
        id: "s1".into(),
        where_text: "1 · 0:05.0–0:12.0".into(),
        label: "corner".into(),
        role_less: false,
    }])));

    labelled(&w, "Seek")
        .expect("the row's Go")
        .invoke_accessible_default_action();
    assert_eq!(
        *jumped.borrow(),
        [("s1".to_string(), "s1".to_string())],
        "selected at the moment of the jump"
    );
    assert!(!w.get_line_editor_open(), "Go closes the sheet");
}

/// Each box keeps its own block for the session (spec S7): what was pasted
/// for the events is not in the slates' box, and is back when the events
/// open again.
#[test]
fn each_box_keeps_its_own_block() {
    let (w, _fired) = bare();
    let done = |w: &AppWindow| {
        labelled(w, "Done")
            .expect("Done")
            .invoke_accessible_default_action();
        assert!(!w.get_line_editor_open());
    };

    w.invoke_open_line_editor(LineEditorKind::Events);
    w.set_line_editor_paste("2 14:05 home goal".into());
    done(&w);

    w.invoke_open_line_editor(LineEditorKind::Slates);
    assert_eq!(w.get_line_editor_paste(), "", "the slates' box is its own");
    w.set_line_editor_paste("2 14:05-14:40 corner".into());
    done(&w);

    w.invoke_open_line_editor(LineEditorKind::Events);
    assert_eq!(w.get_line_editor_paste(), "2 14:05 home goal");
    done(&w);
    w.invoke_open_line_editor(LineEditorKind::Slates);
    assert_eq!(w.get_line_editor_paste(), "2 14:05-14:40 corner");
}

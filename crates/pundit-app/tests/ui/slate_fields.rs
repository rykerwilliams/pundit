//! The slate panel against the **real** `AppWindow`, on Slint's headless
//! backend: the two failures that made the first version of this section a
//! trap, neither of which any bus-level test can see, and what `R` means when
//! a range is selected (BACKLOG #120).
//!
//! The first two come from where a field *lives* rather than what it does,
//! which is why they are pinned here and not in the harness. `R`'s is here for
//! a related reason: which command the key sends is decided in `app.slint`, so
//! the harness — which has no window — can only see whichever one arrives.

use i_slint_backend_testing::ElementHandle;
use slint::platform::{Key, PointerEventButton, WindowEvent};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::cell::RefCell;
use std::rc::Rc;

use crate::key_action;
// The window itself, as `main.rs` builds it: these two failures are about
// where a field lives in the real tree, so a stand-in would not have them. The
// generated UI is compiled once for this whole test binary, in its `main.rs`
// (BACKLOG #121).
use crate::{AppWindow, SlateField, SlateRow};
use pundit_app::keymap::Keymap;

fn slate(id: &str, name: &str) -> SlateRow {
    SlateRow {
        id: id.into(),
        range: "0:05–0:12".into(),
        video: SharedString::new(),
        name: name.into(),
        tags: SharedString::new(),
        shot: false,
        timed: true,
        title: SharedString::new(),
    }
}

/// A window with one slate, selected, and the marks recorded so a shortcut
/// that fires can be seen.
fn window() -> (AppWindow, Rc<RefCell<Vec<&'static str>>>) {
    i_slint_backend_testing::init_no_event_loop();
    let w = AppWindow::new().unwrap();
    // **The keys come from the table now** (BACKLOG #96): with `action-for`
    // unwired every shortcut reads as `KeyAction.none` and this fixture would
    // pass on nothing. `wire_keys` is the binary's own, shared as a module.
    key_action::wire_keys(&w, &crate::scratch_state(), Keymap::defaults());
    let fired: Rc<RefCell<Vec<&'static str>>> = Rc::default();
    w.on_mark_in({
        let fired = Rc::clone(&fired);
        move || fired.borrow_mut().push("mark-in")
    });
    w.on_mark_out({
        let fired = Rc::clone(&fired);
        move || fired.borrow_mut().push("mark-out")
    });
    w.set_can_play(true);
    w.set_slates(ModelRc::new(VecModel::from(vec![slate("s1", "corner")])));
    w.set_selected_slate("s1".into());
    w.show().unwrap();
    (w, fired)
}

/// **Typing in a slate field must not fire the letter shortcuts.** The window
/// folds the fields' focus into `text-editing`, and `handle-key` rejects on it
/// before the letter branches — so `i` and `o` reach the field, not the bus.
///
/// The first version of this section wrote one shared flag from both fields'
/// `changed has-focus`. Slint raises the new item's focus *before* lowering the
/// old one's, so moving between the two fields left the flag `false` while a
/// field still had focus, and typing a tag marked slates, started recordings
/// and tagged goals. The flag is derived from both fields now, never assigned.
#[test]
fn typing_in_a_slate_field_does_not_fire_the_shortcuts() {
    let (w, fired) = window();

    // Into the name field, then **Tab** to the tags field: moving between them
    // with the keyboard is the path that left the old shared flag false while
    // a field still had focus.
    w.invoke_focus_slate_name();
    w.window().dispatch_event(WindowEvent::KeyPressed {
        text: slint::platform::Key::Tab.into(),
    });
    w.window().dispatch_event(WindowEvent::KeyReleased {
        text: slint::platform::Key::Tab.into(),
    });

    for letter in ['i', 'o'] {
        w.window().dispatch_event(WindowEvent::KeyPressed {
            text: letter.into(),
        });
        w.window().dispatch_event(WindowEvent::KeyReleased {
            text: letter.into(),
        });
    }
    assert!(
        fired.borrow().is_empty(),
        "the letters went to the shortcuts instead of the field: {:?}",
        fired.borrow()
    );
}

/// **A project change under a focused field must not kill the keyboard.** The
/// slate list is rebuilt on every project change — including the save a
/// commit itself triggers — and Slint fires no focus-lost callback for an item
/// it destroys. With the fields inside the list, one Enter in the name field
/// left `text-editing` stuck true, every shortcut dead, and no way back but
/// reselecting the row.
///
/// The fields live outside the list now, so a rebuild cannot touch them.
#[test]
fn rebuilding_the_list_leaves_the_keyboard_alive() {
    let (w, fired) = window();
    w.invoke_focus_slate_name();

    // What every `ProjectChanged` does — including the save a commit in this
    // very field triggers.
    w.set_slates(ModelRc::new(VecModel::from(vec![slate("s1", "corner")])));

    // Off the field, as a click on the picture would put it.
    w.set_selected_slate(SharedString::new());

    w.window()
        .dispatch_event(WindowEvent::KeyPressed { text: 'i'.into() });
    w.window()
        .dispatch_event(WindowEvent::KeyReleased { text: 'i'.into() });
    assert_eq!(
        *fired.borrow(),
        ["mark-in"],
        "the keyboard should work again once the field is gone"
    );
}

/// A window with two ranges and nothing selected, recording what `R` sent
/// (BACKLOG #120).
fn record_window() -> (AppWindow, Rc<RefCell<Vec<String>>>) {
    i_slint_backend_testing::init_no_event_loop();
    let w = AppWindow::new().unwrap();
    // **The keys come from the table now** (BACKLOG #96): with `action-for`
    // unwired every shortcut reads as `KeyAction.none` and this fixture would
    // pass on nothing. `wire_keys` is the binary's own, shared as a module.
    key_action::wire_keys(&w, &crate::scratch_state(), Keymap::defaults());
    let sent: Rc<RefCell<Vec<String>>> = Rc::default();
    w.on_toggle_recording({
        let sent = Rc::clone(&sent);
        move |slate| sent.borrow_mut().push(format!("toggle {slate:?}"))
    });
    w.set_can_play(true);
    w.set_slates(ModelRc::new(VecModel::from(vec![
        slate("s1", "corner"),
        slate("s2", "corner"),
    ])));
    w.show().unwrap();
    (w, sent)
}

fn press_r(w: &AppWindow) {
    w.window()
        .dispatch_event(WindowEvent::KeyPressed { text: 'r'.into() });
    w.window()
        .dispatch_event(WindowEvent::KeyReleased { text: 'r'.into() });
}

/// **`R` carries the selected range**, which is the whole of what the window
/// decides about it — the bus turns it into that range's take, or into a stop.
///
/// It is what makes the themed pass work from the keyboard: a plain take would
/// record the right frames under a clip with no `slate_id`, so the range would
/// never read as shot, the footage would never stop at its out point, and the
/// pass would park on the same row for ever.
#[test]
fn r_carries_the_selected_range() {
    let (w, sent) = record_window();
    w.set_selected_slate("s2".into());

    press_r(&w);

    assert_eq!(*sent.borrow(), [r#"toggle "s2""#]);
}

/// With nothing selected it is the plain take it always was. The selection is
/// the whole of the mode, and Esc is how a coach leaves it.
#[test]
fn r_with_nothing_selected_carries_no_range() {
    let (w, sent) = record_window();

    press_r(&w);

    assert_eq!(*sent.borrow(), [r#"toggle """#]);
}

fn press(w: &AppWindow, key: impl Into<SharedString> + Clone) {
    w.window().dispatch_event(WindowEvent::KeyPressed {
        text: key.clone().into(),
    });
    w.window()
        .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
}

/// **A title commits to the slate the field opened on** (v18, #133), not to
/// whatever is selected when focus leaves — `editing-slate-id`'s rule.
///
/// The first test of that rule for **any** slate field: the name field's #132
/// guard lives in `main.rs`, which no test reaches, and nothing pinned the
/// Slint half — the field's focus-in writing the id the commit then reads.
#[test]
fn the_title_field_commits_to_the_slate_it_opened_on() {
    i_slint_backend_testing::init_no_event_loop();
    let w = AppWindow::new().unwrap();
    key_action::wire_keys(&w, &crate::scratch_state(), Keymap::defaults());
    // `(field, text, editing-slate-id)`, the id read **inside** the callback,
    // where `main.rs`'s `on_edit_slate_field` reads it.
    let commits: Rc<RefCell<Vec<(SlateField, String, String)>>> = Rc::default();
    w.on_edit_slate_field({
        let (commits, weak) = (Rc::clone(&commits), w.as_weak());
        move |field, text| {
            let editing = weak.unwrap().get_editing_slate_id().to_string();
            commits
                .borrow_mut()
                .push((field, text.to_string(), editing));
        }
    });
    w.set_slates(ModelRc::new(VecModel::from(vec![
        slate("s1", "corner"),
        slate("s2", "free kick"),
    ])));
    w.set_selected_slate("s1".into());
    w.show().unwrap();

    // Clicked, as a coach reaches it — `text_editing.rs`'s way in.
    let mut found = ElementHandle::find_by_element_id(&w, "AppWindow::slate-title-edit");
    let field = found.next().expect("the title field is on screen");
    assert!(found.next().is_none(), "one title field");
    field.mock_single_click(PointerEventButton::Left);
    for letter in ["C", "o", "r", "n", "e", "r"] {
        press(&w, letter);
    }

    // The selection moves on under the focused field, then focus leaves it.
    // **Esc, not a click**: Slint raises the new item's focus before lowering
    // the old one's, so a click into another slate field would be a test of
    // that ordering rather than of this field's commit.
    w.set_selected_slate("s2".into());
    press(&w, Key::Escape);

    assert_eq!(
        *commits.borrow(),
        [(SlateField::Title, "Corner".to_string(), "s1".to_string())],
        "one commit, of the title, to the slate the field opened on"
    );
}

/// **The row says when a slate has a title, and only then** (v18, #133): a
/// `T` beside the shot marker, found by the words on it. An untitled row shows
/// nothing, so a project that never uses titles sees no change in the list.
#[test]
fn a_titled_slate_shows_its_marker_and_an_untitled_one_none() {
    i_slint_backend_testing::init_no_event_loop();
    let w = AppWindow::new().unwrap();
    w.set_slates(ModelRc::new(VecModel::from(vec![
        SlateRow {
            title: "Corner routine".into(),
            ..slate("s1", "corner")
        },
        slate("s2", "free kick"),
    ])));
    w.show().unwrap();

    assert_eq!(
        ElementHandle::find_by_accessible_label(&w, "T").count(),
        1,
        "one marker, on the titled row"
    );
}

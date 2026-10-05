//! The slate panel against the **real** `AppWindow`, on Slint's headless
//! backend: the two failures that made the first version of this section a
//! trap, neither of which any bus-level test can see, and what `R` means when
//! a range is selected (BACKLOG #120).
//!
//! The first two come from where a field *lives* rather than what it does,
//! which is why they are pinned here and not in the harness. `R`'s is here for
//! a related reason: which command the key sends is decided in `app.slint`, so
//! the harness — which has no window — can only see whichever one arrives.

use slint::platform::WindowEvent;
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::cell::RefCell;
use std::rc::Rc;

// The window itself, as `main.rs` builds it: these two failures are about
// where a field lives in the real tree, so a stand-in would not have them. The
// generated UI is compiled once for this whole test binary, in its `main.rs`
// (BACKLOG #121).
use crate::{AppWindow, SlateRow};

fn slate(id: &str, name: &str) -> SlateRow {
    SlateRow {
        id: id.into(),
        range: "0:05–0:12".into(),
        video: SharedString::new(),
        name: name.into(),
        tags: SharedString::new(),
        shot: false,
        timed: true,
    }
}

/// A window with one slate, selected, and the marks recorded so a shortcut
/// that fires can be seen.
fn window() -> (AppWindow, Rc<RefCell<Vec<&'static str>>>) {
    i_slint_backend_testing::init_no_event_loop();
    let w = AppWindow::new().unwrap();
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

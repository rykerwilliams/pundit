//! The shared `TagField` against the **real** `AppWindow`, on Slint's
//! headless backend: Tab takes a suggestion and Esc closes the list before it
//! leaves the field, at **both** of the component's two sites (BACKLOG #105).
//!
//! Two reasons these are here and not in the harness. The rules live in
//! `app.slint` -- which key the field accepts and which it lets bubble -- so
//! nothing without a window can see them. And Esc's cascade is a property of
//! where the field *lives*: the window's `handle-key` rejects it down to the
//! focused field only because that field's focus is folded into
//! `text-editing`, and the field hands it back only when no list is open.
//! Lifting the field out of the inspector into a component is exactly the
//! change that could have broken either half silently.
//!
//! `suggest-tags` and `take-suggestion` are wired to the same `core::tag`
//! functions `main.rs` wires them to, so what is pinned is the picker end to
//! end rather than a stand-in for it.

use pundit_core::tag::{tag_suggestions, take_suggestion};
use slint::platform::WindowEvent;
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::cell::RefCell;
use std::rc::Rc;

use crate::key_action;
use crate::{AppWindow, SlateRow};
use pundit_app::keymap::Keymap;

/// The tags already in use, as a project of a few clips would have them.
const VOCABULARY: [&str; 3] = ["corner", "counter", "set piece"];

/// A window with a clip selected (so the inspector is shown) and one slate
/// selected (so the slate fields are), the two suggestion callbacks wired to
/// `core::tag`, and `i` / `o` recorded: those two are the probe for "did this
/// key reach the shortcuts or the field?".
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
    w.on_suggest_tags(|text| {
        let vocabulary: Vec<String> = VOCABULARY.iter().map(|t| (*t).to_owned()).collect();
        let tags: Vec<SharedString> = tag_suggestions(&vocabulary, &text)
            .into_iter()
            .map(SharedString::from)
            .collect();
        ModelRc::new(VecModel::from(tags))
    });
    w.on_take_suggestion(|text, tag| take_suggestion(&text, &tag).into());

    w.set_can_play(true);
    w.set_selected_clip("c1".into());
    w.set_slates(ModelRc::new(VecModel::from(vec![SlateRow {
        id: "s1".into(),
        range: "0:05–0:12".into(),
        video: SharedString::new(),
        name: "corner".into(),
        tags: SharedString::new(),
        shot: false,
        timed: true,
    }])));
    w.set_selected_slate("s1".into());
    w.show().unwrap();
    (w, fired)
}

fn press(w: &AppWindow, key: impl Into<SharedString> + Clone) {
    w.window().dispatch_event(WindowEvent::KeyPressed {
        text: key.clone().into(),
    });
    w.window()
        .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
}

fn press_esc(w: &AppWindow) {
    press(w, slint::platform::Key::Escape);
}

fn press_tab(w: &AppWindow) {
    press(w, slint::platform::Key::Tab);
}

/// **The clip inspector's picker is unchanged by the extraction**: two letters
/// open the list, and Tab splices the top suggestion into the field with the
/// `", "` that lets the next one be typed straight away.
#[test]
fn tab_completes_a_clip_tag() {
    let (w, _) = window();
    w.invoke_focus_clip_tags();

    press(&w, 'c');
    press(&w, 'o');
    press_tab(&w);

    // `corner` sorts before `counter`, and the field keeps focus.
    assert_eq!(w.get_clip_tags(), "corner, ");
}

/// The same field in the slate editor, which is the whole of BACKLOG #105: the
/// vocabulary is `tag_vocabulary`'s clips ∪ slates either way, so a tag first
/// typed on a clip completes on a slate.
#[test]
fn tab_completes_a_slate_tag() {
    let (w, _) = window();
    w.invoke_focus_slate_tags();

    press(&w, 'c');
    press(&w, 'o');
    press_tab(&w);

    assert_eq!(w.get_slate_tags(), "corner, ");
}

/// **Esc closes the list first and leaves the field second**, in the clip
/// inspector. The first press dismisses the suggestions and the field keeps
/// focus -- so the next letter is still text, not a shortcut; the second
/// bubbles to the window, which drops focus, and then `o` marks a slate out
/// point again.
#[test]
fn esc_dismisses_the_list_before_it_leaves_the_clip_field() {
    let (w, fired) = window();
    w.invoke_focus_clip_tags();

    press(&w, 'c');
    press(&w, 'o');
    press_esc(&w);

    // Still in the field: the letter is typed, and no shortcut fires.
    press(&w, 'o');
    assert_eq!(w.get_clip_tags(), "coo");
    assert!(
        fired.borrow().is_empty(),
        "Esc left the field on the first press: {:?}",
        fired.borrow()
    );

    // With no list open, Esc is the window's and leaves the field.
    press_esc(&w);
    press(&w, 'o');
    assert_eq!(*fired.borrow(), ["mark-out"]);
}

/// And in the slate editor, where the cascade is the one the slates spec's §S6
/// settled: the fields' focus is folded into the window's `text-editing`, so
/// Esc reaches the field at all. Without the fold the first press would leave
/// the field and throw away a half-typed tag.
#[test]
fn esc_dismisses_the_list_before_it_leaves_the_slate_field() {
    let (w, fired) = window();
    w.invoke_focus_slate_tags();

    press(&w, 'c');
    press(&w, 'o');
    press_esc(&w);

    press(&w, 'o');
    assert_eq!(w.get_slate_tags(), "coo");
    assert!(
        fired.borrow().is_empty(),
        "Esc left the field on the first press: {:?}",
        fired.borrow()
    );

    press_esc(&w);
    press(&w, 'o');
    assert_eq!(*fired.borrow(), ["mark-out"]);
}

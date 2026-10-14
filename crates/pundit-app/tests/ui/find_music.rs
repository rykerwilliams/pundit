//! Find music (BACKLOG #84 phase 2): the Music row's door to it, the lines
//! Openverse's terms and the licence caveat put on it, its Add buttons, and
//! its place in `handle-key`'s sheet chain — driven through the real window.
//!
//! **What is not reached**, and the plan says so: `main.rs`'s
//! `wire_find_music` — the search and Add threads, the generation counter, the
//! default folder and the genre written on a successful Add. Those are
//! manual checks 1–5; what they map over is `music_search`'s own tests.

use crate::key_action;
use crate::{AppWindow, MusicRow};
use i_slint_backend_testing::{AccessibleRole, ElementHandle};
use pundit_app::keymap::Keymap;
use slint::platform::{Key, PointerEventButton, WindowEvent};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::cell::RefCell;
use std::rc::Rc;

const OPENVERSE_LINE: &str =
    "Search by Openverse. pundit is not endorsed or certified by Openverse.";
const LICENCE_LINE: &str =
    "Licences are as the catalogue reports them — check the track's page before you publish.";

/// The export sheet up with a reel ticked — where Find music is opened from —
/// and the keys wired, so `handle-key` acts on them as the app's does.
fn window() -> AppWindow {
    i_slint_backend_testing::init_no_event_loop();
    let w = AppWindow::new().unwrap();
    key_action::wire_keys(&w, &crate::scratch_state(), Keymap::defaults());
    w.set_export_reel_ticked(true);
    w.set_export_sheet_open(true);
    w.show().unwrap();
    w
}

fn shows(w: &AppWindow, words: &str) -> bool {
    ElementHandle::find_by_accessible_label(w, words)
        .next()
        .is_some()
}

/// Every button reading `label`, in the order the tree holds them.
fn buttons(w: &AppWindow, label: &str) -> Vec<ElementHandle> {
    ElementHandle::find_by_accessible_label(w, label)
        .filter(|e| e.accessible_role() == Some(AccessibleRole::Button))
        .collect()
}

fn row(id: &str, title: &str) -> MusicRow {
    MusicRow {
        id: id.into(),
        title: title.into(),
        detail: "Artist · 3:24 · CC BY 4.0".into(),
        state: SharedString::new(),
    }
}

fn press(w: &AppWindow, key: impl Into<SharedString> + Clone) {
    w.window().dispatch_event(WindowEvent::KeyPressed {
        text: key.clone().into(),
    });
    w.window()
        .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
}

/// The Music row's Find music… reaches the window's `find-music`, and not the
/// Folder… button's callback beside it.
#[test]
fn find_music_is_reached_from_the_music_row() {
    let w = window();
    let fired: Rc<RefCell<Vec<&str>>> = Rc::default();
    w.on_find_music({
        let fired = Rc::clone(&fired);
        move || fired.borrow_mut().push("find-music")
    });
    w.on_choose_music_folder({
        let fired = Rc::clone(&fired);
        move || fired.borrow_mut().push("choose-music-folder")
    });
    let [find] = buttons(&w, "Find music…")
        .try_into()
        .unwrap_or_else(|found: Vec<_>| {
            panic!("{} Find music… buttons on the Music row", found.len())
        });
    find.invoke_accessible_default_action();
    assert_eq!(*fired.borrow(), ["find-music"]);
}

/// Openverse's terms ask that an app using it says so (spec §O9), and the app
/// does not promise the licences it shows (spec L3). Both are always there.
#[test]
fn the_sheet_says_openverse_and_the_licence_caveat() {
    let w = window();
    w.set_find_music_open(true);
    assert!(shows(&w, OPENVERSE_LINE), "no Openverse attribution");
    assert!(shows(&w, LICENCE_LINE), "no licence caveat");
}

/// A row's Add hands back that row's id; and while one is adding, every Add
/// waits — one at a time, with no queue (spec S2).
#[test]
fn add_reaches_its_row_and_the_others_wait() {
    let w = window();
    w.set_find_music_rows(ModelRc::new(VecModel::from(vec![
        row("id1", "One"),
        row("id2", "Two"),
        row("id3", "Three"),
    ])));
    w.set_find_music_open(true);
    let added: Rc<RefCell<Vec<String>>> = Rc::default();
    w.on_add_music({
        let added = Rc::clone(&added);
        move |id| added.borrow_mut().push(id.into())
    });
    let adds = buttons(&w, "Add");
    assert_eq!(adds.len(), 3, "one Add per row");
    assert!(adds.iter().all(|b| b.accessible_enabled() == Some(true)));
    adds[1].invoke_accessible_default_action();
    assert_eq!(*added.borrow(), ["id2"]);

    w.set_find_music_adding(true);
    let enabled: Vec<_> = buttons(&w, "Add")
        .iter()
        .map(|b| b.accessible_enabled())
        .collect();
    assert_eq!(
        enabled,
        [Some(false); 3],
        "an Add is in flight and another can be pressed"
    );
}

/// **Esc in the genre field leaves the field and closes nothing; the next
/// closes Find music alone** (spec S10). The export sheet under it stays open:
/// Find music's branch is ahead of the export sheet's in `handle-key`, whose
/// Esc closes that sheet whatever has focus. And the `r` typed first is the
/// record key, which has to land in the field.
#[test]
fn esc_in_the_genre_field_leaves_both_sheets_open_and_the_next_closes_find_music_only() {
    let w = window();
    let closed = Rc::new(RefCell::new(0));
    w.on_find_music_closed({
        let closed = Rc::clone(&closed);
        move || *closed.borrow_mut() += 1
    });
    let recorded = Rc::new(RefCell::new(false));
    w.on_toggle_recording({
        let recorded = Rc::clone(&recorded);
        move |_slate| *recorded.borrow_mut() = true
    });
    w.set_find_music_open(true);
    ElementHandle::find_by_element_id(&w, "FindMusicSheet::genre-edit")
        .next()
        .expect("the genre field")
        .mock_single_click(PointerEventButton::Left);
    press(&w, "r");
    assert_eq!(w.get_find_music_genre(), "r");
    assert!(!*recorded.borrow(), "R started a take behind the sheet");

    press(&w, Key::Escape);
    assert!(w.get_find_music_open(), "the first Esc closed Find music");
    assert!(
        w.get_export_sheet_open(),
        "the first Esc closed the export sheet"
    );
    assert_eq!(*closed.borrow(), 0);

    press(&w, Key::Escape);
    assert!(
        !w.get_find_music_open(),
        "the second Esc left Find music open"
    );
    assert_eq!(*closed.borrow(), 1, "closing never told the window");
    assert!(
        w.get_export_sheet_open(),
        "the second Esc closed the export sheet as well"
    );
}

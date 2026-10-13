//! The scrubber's right-click menu — the snap (BACKLOG #102) and the door to
//! the Settings sheet's band colour (#131) — driven through the real window as
//! `slate_menu.rs` drives the slate row's: right-click, find the item by its
//! words, click it.
//!
//! **What it pins first is that the menu is reachable at all.** It sits inside
//! `Scrubber` above the drag's and the wheel's `TouchArea`s, either of which
//! grabs a press of any button: declared below one, the menu never opens. The
//! fixture therefore has something to play, so the drag area is enabled and
//! *would* take the right press if it were on top.

use crate::{AppWindow, RecordingPhase};
use i_slint_backend_testing::ElementHandle;
use slint::platform::{PointerEventButton, WindowEvent};
use slint::{ComponentHandle, SharedString};
use std::cell::Cell;
use std::rc::Rc;

const ITEM: &str = "Snap to match events";
/// The door to the Settings sheet (U+2026, as the menu spells it).
const BAND: &str = "Band colour…";

/// The window with the scrubber enabled, and how many times `toggle-snap`
/// fired.
fn window() -> (AppWindow, Rc<Cell<u32>>) {
    i_slint_backend_testing::init_no_event_loop();
    let w = AppWindow::new().unwrap();
    let toggled: Rc<Cell<u32>> = Rc::default();
    w.on_toggle_snap({
        let toggled = Rc::clone(&toggled);
        move || toggled.set(toggled.get() + 1)
    });
    w.set_can_play(true);
    w.show().unwrap();
    (w, toggled)
}

/// Right-clicks the scrubber and returns its menu area, which the popup's
/// items hang off.
fn open(w: &AppWindow) -> ElementHandle {
    let areas: Vec<ElementHandle> =
        ElementHandle::find_by_element_id(w, "Scrubber::snap-menu").collect();
    assert_eq!(areas.len(), 1, "one scrubber");
    let area = areas.into_iter().next().unwrap();
    area.mock_single_click(PointerEventButton::Right);
    area
}

/// The menu's descendant whose accessible label is `label`, if it shows one.
fn find(area: &ElementHandle, label: &str) -> Option<ElementHandle> {
    let label = label.to_string();
    area.query_descendants()
        .match_predicate(move |e| e.accessible_label().is_some_and(|l| l == label.as_str()))
        .find_first()
}

/// Escape, as a coach dismisses a menu: an open popup keeps the entries it
/// was opened with, so a reopen without it would read the old one.
fn close(w: &AppWindow) {
    let escape = SharedString::from(slint::platform::Key::Escape);
    w.window().dispatch_event(WindowEvent::KeyPressed {
        text: escape.clone(),
    });
    w.window()
        .dispatch_event(WindowEvent::KeyReleased { text: escape });
}

fn click_the_item(area: &ElementHandle) {
    find(area, ITEM)
        .unwrap_or_else(|| panic!("no {ITEM:?} in the menu"))
        .mock_single_click(PointerEventButton::Left);
}

#[test]
fn the_scrubbers_menu_toggles_the_snap() {
    let (w, toggled) = window();
    click_the_item(&open(&w));
    assert_eq!(toggled.get(), 1);
}

/// The item's ✓ is bound to the window's property, so it shows what Rust
/// last pushed rather than a state of its own.
#[test]
fn the_checkmark_follows_the_window() {
    let (w, _) = window();
    assert!(find(&open(&w), "✓").is_some(), "on: checked");
    close(&w);
    w.set_snap_to_events(false);
    assert!(find(&open(&w), "✓").is_none(), "off: unchecked");
}

/// A recording disables the drag, and the menu is not gated on it: it acts on
/// nothing playing.
#[test]
fn the_menu_opens_while_recording() {
    let (w, toggled) = window();
    w.set_recording_phase(RecordingPhase::Recording);
    click_the_item(&open(&w));
    assert_eq!(toggled.get(), 1);
}

#[test]
fn the_band_colour_item_opens_the_settings() {
    let (w, _) = window();
    find(&open(&w), BAND)
        .unwrap_or_else(|| panic!("no {BAND:?} in the menu"))
        .mock_single_click(PointerEventButton::Left);
    assert!(w.get_settings_sheet_open());
}

/// The menu opens mid-take for the snap, so the item carries the sheet's gate
/// itself — and `open-settings()` has no guard of its own, so this is the item's
/// `enabled` alone.
#[test]
fn the_band_colour_item_is_off_while_recording() {
    let (w, _) = window();
    w.set_recording_phase(RecordingPhase::Recording);
    find(&open(&w), BAND)
        .unwrap_or_else(|| panic!("no {BAND:?} in the menu"))
        .mock_single_click(PointerEventButton::Left);
    assert!(!w.get_settings_sheet_open());
}

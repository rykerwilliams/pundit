//! The touch affordance (#140 task 8): a tap path onto the three mouse-only
//! gestures the in-gym capture feature left behind — the clip row's menu-only
//! actions, the slate row's, and the scrubber's snap toggle.
//!
//! **One menu definition, two openers.** The clip row's `clip-menu` and the
//! slate row's `slate-menu` are each one `ContextMenuArea`; the right-click
//! opens it (`slate_menu.rs` drives that) and a kebab `⋯` on the row opens the
//! **same** menu through its `show()` function. So these tests tap the kebab
//! and then drive the menu exactly as `slate_menu.rs` does, and one of them
//! pins that the kebab and the right-click list the same items — the sabotage
//! target, because a kebab wired to a divergent copy would still open *a*
//! menu.
//!
//! The pattern is `snap_menu.rs`'s and `slate_menu.rs`': `find_by_element_id`
//! for the menu area, `mock_single_click` on the kebab's own `TouchArea`
//! (found by the accessible label on its glyph), and the item found by its
//! words under the menu area the popup hangs off.

use crate::{AppWindow, ClipRow, SlateRow};
use i_slint_backend_testing::{ElementHandle, ElementQuery};
use slint::platform::{PointerEventButton, WindowEvent};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

fn clip(id: &str) -> ClipRow {
    ClipRow {
        id: id.into(),
        name: "a clip".into(),
        duration: "0:12".into(),
        whiteboard: false,
    }
}

fn slate(id: &str) -> SlateRow {
    SlateRow {
        id: id.into(),
        range: "0:05–0:12".into(),
        video: SharedString::new(),
        name: "corner".into(),
        tags: SharedString::new(),
        shot: false,
        timed: true,
        title: SharedString::new(),
    }
}

/// A window with one clip and one slate, the game video present so both lists
/// show, and every command the menus can send recorded.
fn window() -> (AppWindow, Rc<RefCell<Vec<String>>>) {
    i_slint_backend_testing::init_no_event_loop();
    let w = AppWindow::new().unwrap();
    let sent: Rc<RefCell<Vec<String>>> = Rc::default();
    let record = |sent: &Rc<RefCell<Vec<String>>>, what: &'static str| {
        let sent = Rc::clone(sent);
        move |id: SharedString| sent.borrow_mut().push(format!("{what} {id}"))
    };
    w.on_jump_to_clip(record(&sent, "jump"));
    w.on_open_preview(record(&sent, "preview"));
    w.on_export_clip(record(&sent, "export"));
    w.on_add_to_basket(record(&sent, "basket"));
    w.on_delete_clip(record(&sent, "delete"));
    w.on_shoot_slate(record(&sent, "shoot"));
    w.on_preview_slate(record(&sent, "preview-slate"));
    w.on_jump_to_slate(record(&sent, "jump-slate"));
    w.on_delete_slate(record(&sent, "delete-slate"));
    w.set_can_play(true);
    crate::one_source(&w);
    w.set_clips(ModelRc::new(VecModel::from(vec![clip("c1")])));
    w.set_slates(ModelRc::new(VecModel::from(vec![slate("s1")])));
    w.show().unwrap();
    (w, sent)
}

/// The only menu area of `id` in the window (there is one clip and one slate).
fn menu(w: &AppWindow, id: &str) -> ElementHandle {
    let areas: Vec<ElementHandle> = ElementHandle::find_by_element_id(w, id).collect();
    assert_eq!(areas.len(), 1, "one {id}");
    areas.into_iter().next().unwrap()
}

/// Taps the kebab whose glyph carries `label` (left click, as a finger does).
fn tap_kebab(w: &AppWindow, label: &str) {
    let label = label.to_string();
    let kebab = ElementQuery::from_root(w)
        .match_predicate(move |e| e.accessible_label().is_some_and(|l| l == label.as_str()))
        .find_first()
        .expect("a kebab with that label");
    kebab.mock_single_click(PointerEventButton::Left);
}

/// The titles of every `MenuItem` the popup hanging off `area` shows, in tree
/// order — what the sabotage test compares across the two openers.
fn item_titles(area: &ElementHandle) -> Vec<String> {
    area.query_descendants()
        .find_all()
        .into_iter()
        .filter_map(|e| e.accessible_label())
        .map(|l| l.to_string())
        .collect()
}

/// Escape, to dismiss an open popup before another is opened — a popup keeps
/// the entries it was opened with, so a reopen without this reads the old one
/// (and a second `show()` over an open popup is swallowed). `snap_menu.rs`'s
/// `close`.
fn close(w: &AppWindow) {
    let escape = SharedString::from(slint::platform::Key::Escape);
    w.window().dispatch_event(WindowEvent::KeyPressed {
        text: escape.clone(),
    });
    w.window()
        .dispatch_event(WindowEvent::KeyReleased { text: escape });
}

/// Clicks the menu item reading `title` under `area`.
fn pick(area: &ElementHandle, title: &str) {
    area.query_descendants()
        .match_predicate({
            let title = title.to_string();
            move |e| e.accessible_label().is_some_and(|l| l == title.as_str())
        })
        .find_first()
        .unwrap_or_else(|| panic!("no {title:?} in the menu"))
        .mock_single_click(PointerEventButton::Left);
}

/// Tapping the clip row's kebab opens `clip-menu`, and an item in it fires its
/// command on the row's clip. The kebab is the tap path to the **menu-only**
/// actions; Jump and Preview already have one (single- and double-click).
#[test]
fn the_clip_row_kebab_opens_the_clip_menu() {
    let (w, sent) = window();

    tap_kebab(&w, "Clip actions");
    pick(&menu(&w, "AppWindow::clip-menu"), "Export video…");

    assert_eq!(*sent.borrow(), ["export c1"]);
}

/// The slate row's kebab opens the **same** `slate-menu` the right-click does.
#[test]
fn the_slate_row_kebab_opens_the_slate_menu() {
    let (w, sent) = window();

    tap_kebab(&w, "Slate actions");
    pick(&menu(&w, "AppWindow::slate-menu"), "Delete slate");

    assert_eq!(*sent.borrow(), ["delete-slate s1"]);
}

/// **The two openers list the same actions** — the whole point of "one menu
/// definition, two openers". A kebab wired to a divergent copy (an item
/// removed or renamed) would open *a* menu and pass every test above; this is
/// the one that fails. Done for both rows.
#[test]
fn both_openers_list_the_same_actions() {
    let (w, _sent) = window();

    tap_kebab(&w, "Clip actions");
    let clip_kebab = item_titles(&menu(&w, "AppWindow::clip-menu"));
    assert!(
        clip_kebab.contains(&"Export video…".to_string()),
        "{clip_kebab:?}"
    );
    close(&w);

    menu(&w, "AppWindow::clip-menu").mock_single_click(PointerEventButton::Right);
    let clip_right = item_titles(&menu(&w, "AppWindow::clip-menu"));
    close(&w);
    assert_eq!(
        clip_kebab, clip_right,
        "the clip kebab and right-click must open the same items"
    );

    tap_kebab(&w, "Slate actions");
    let slate_kebab = item_titles(&menu(&w, "AppWindow::slate-menu"));
    assert!(
        slate_kebab.contains(&"Delete slate".to_string()),
        "{slate_kebab:?}"
    );
    close(&w);

    menu(&w, "AppWindow::slate-menu").mock_single_click(PointerEventButton::Right);
    let slate_right = item_titles(&menu(&w, "AppWindow::slate-menu"));
    close(&w);
    assert_eq!(
        slate_kebab, slate_right,
        "the slate kebab and right-click must open the same items"
    );
}

/// The snap toggle is a visible checkbox, tap-reachable, and flips the
/// window's `snap-to-events` through the same `toggle-snap` the right-click
/// menu sends (`wire_snap` is the shared Rust half). The fixture flips the
/// property the way `wire_snap` does, so the tap's effect is observable.
#[test]
fn the_snap_toggle_is_tap_reachable() {
    i_slint_backend_testing::init_no_event_loop();
    let w = AppWindow::new().unwrap();
    let flips: Rc<Cell<u32>> = Rc::default();
    w.on_toggle_snap({
        let w = w.as_weak();
        let flips = Rc::clone(&flips);
        move || {
            flips.set(flips.get() + 1);
            let w = w.unwrap();
            w.set_snap_to_events(!w.get_snap_to_events());
        }
    });
    w.set_can_play(true);
    crate::one_source(&w);
    w.set_snap_to_events(true);
    w.show().unwrap();

    let toggle = ElementQuery::from_root(&w)
        .match_predicate(|e| {
            e.accessible_label()
                .is_some_and(|l| l == "Snap to match events")
        })
        .find_first()
        .expect("a visible snap toggle");
    toggle.mock_single_click(PointerEventButton::Left);

    assert_eq!(flips.get(), 1, "the toggle did not send toggle-snap");
    assert!(
        !w.get_snap_to_events(),
        "the toggle did not flip snap-to-events"
    );
}

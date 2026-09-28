//! The live self-view's placement against the **real** `AppWindow`, on Slint's
//! headless backend (#88 spec I6).
//!
//! `place-self-view` is a `pure` callback with no captures, so the take's size
//! and corner reach `core::layout` only if five sites agree: the two declared
//! enums, the two `in property`s, the callback's signature, the binding that
//! calls it and `start_self_view`'s feed. **Nothing about a missing one fails to
//! compile** — the self-view simply stays Medium in the bottom-right, and a
//! coach who set Large/BottomLeft sees the inset in the wrong corner and files
//! it as a bug. No bus-level test can see any of it: the placement never leaves
//! the window.
//!
//! What is under test is everything from the properties inward. The feed itself
//! is in `main.rs`, a `[[bin]]` no test can reach — the same limit
//! `tests/panel_widths.rs` records for the splitter's `state.json` write.

use std::cell::RefCell;
use std::rc::Rc;

use slint::{ComponentHandle, PhysicalSize};

// The window as `main.rs` builds it: the binding that calls the callback is the
// test, so a stand-in would prove nothing.
slint::include_modules!();

/// Every placement the window asked for, in order.
type Seen = Rc<RefCell<Vec<(InsetSize, InsetCorner)>>>;

/// A shown window with the self-view up, and what the binding passed.
fn window() -> (AppWindow, Seen) {
    i_slint_backend_testing::init_no_event_loop();
    let w = AppWindow::new().unwrap();
    let seen: Seen = Rc::default();
    w.on_place_self_view({
        let seen = Rc::clone(&seen);
        move |_content, _aspect, _level, _avatar, size, corner| {
            seen.borrow_mut().push((size, corner));
            PictureRect::default()
        }
    });
    w.set_has_project(true);
    w.set_can_play(true);
    // The self-view is only in the tree while a take is on screen.
    w.set_self_view_shown(true);
    w.show().unwrap();
    w.window().set_size(PhysicalSize::new(1280, 800));
    (w, seen)
}

/// Reads the placement, which is what evaluates the binding that calls
/// `place-self-view`.
///
/// **Reading it is the only way to make it run**, and the reason the binding
/// sits on a window property rather than on the image. The image is positioned
/// absolutely and takes no input, so no parent's layout wants its geometry, a
/// headless window never renders it and a pointer event never reaches it —
/// measured: with the binding on the image, neither `show()`, a resize nor a
/// `PointerMoved` over the player called the callback once.
fn place(w: &AppWindow) {
    let _ = w.get_self_view_rect();
}

/// **The take's own size and corner reach the placement.** Both properties
/// default to the values every inset the app has ever drawn had, so a wire left
/// out reads as a pass unless the test asks for something else — hence Large
/// and BottomLeft, neither of them a default.
#[test]
fn the_self_view_is_placed_with_the_takes_own_size_and_corner() {
    let (w, seen) = window();
    w.set_self_view_size(InsetSize::Large);
    w.set_self_view_corner(InsetCorner::BottomLeft);
    place(&w);

    let seen = seen.borrow();
    assert!(!seen.is_empty(), "the self-view was never placed");
    assert_eq!(
        seen.last(),
        Some(&(InsetSize::Large, InsetCorner::BottomLeft)),
        "every placement the window asked for: {seen:?}"
    );
}

/// **And the defaults are what they have always been**, so a project whose
/// preferences have never been touched records exactly as it did before v13.
#[test]
fn an_untouched_project_places_the_self_view_where_it_always_was() {
    let (w, seen) = window();
    place(&w);

    let seen = seen.borrow();
    assert!(!seen.is_empty(), "the self-view was never placed");
    assert_eq!(
        seen.last(),
        Some(&(InsetSize::Medium, InsetCorner::BottomRight)),
        "every placement the window asked for: {seen:?}"
    );
}

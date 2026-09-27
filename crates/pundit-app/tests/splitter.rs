//! The panel splitter's drag (spec W5), driven by pointer events on Slint's
//! headless testing backend.
//!
//! **The test window carries a real bound, and that is the point.** The
//! splitter emits a width; what stops a column going under its minimum is the
//! column's own layout constraints (spec W4). So the window below stands in
//! for `app.slint`'s columns — a `column` with `min-width`, a
//! `preferred-width`/`max-width` of `max(min, stored)` and no stretch, a 6px
//! `Splitter`, and a `player` that holds a minimum of its own — and the
//! `moved` handler assigns the emitted width to `stored` exactly as the real
//! wiring does. A `width-now` bound to a constant can never *be* bounded, and
//! a test written that way would catch neither of W5's two failures: both only
//! show once a drag has pushed past the minimum.
//!
//! The bound is written as layout constraints rather than as
//! `width: clamp(stored, min, root.width - …)`: that form reads `root.width`
//! from inside a child whose width feeds the layout that decides it, which
//! Slint deprecates with a warning `-D warnings` would fail on.

use std::cell::RefCell;
use std::rc::Rc;

use slint::platform::{PointerEventButton, WindowEvent};
use slint::{ComponentHandle, LogicalPosition};

/// The grip's own width, which is the splitter's and not this window's to set.
const GRIP: f32 = 6.0;

slint::slint! {
    import { Splitter } from "ui/splitter.slint";

    export component TestWindow inherits Window {
        width: 400px;
        height: 40px;
        // The raw property the drag writes, as `sidebar-width` is in the app.
        in property <length> stored: root.column-min;
        in property <bool> panel-on-the-right;
        out property <length> column-min: 80px;
        // What the layout actually gave the column: what the coach sees, and
        // what the splitter anchors on.
        out property <length> laid-out: column.width;
        out property <length> grip-x: splitter.absolute-position.x;
        out property <length> grip-width: splitter.width;
        callback moved(length);
        callback released();

        HorizontalLayout {
            spacing: 0px;
            padding: 0px;

            column := Rectangle {
                min-width: root.column-min;
                preferred-width: max(root.column-min, root.stored);
                max-width: max(root.column-min, root.stored);
                horizontal-stretch: 0;
            }

            splitter := Splitter {
                width-now: column.width;
                panel-on-the-right: root.panel-on-the-right;
                moved(w) => { root.moved(w); }
                released => { root.released(); }
            }

            // The player: it holds a minimum of its own, which is what caps
            // how wide the column can get.
            Rectangle {
                min-width: 100px;
                horizontal-stretch: 1;
            }
        }
    }
}

/// The window, the widths its `moved` emitted, and how often it released.
struct Rig {
    window: TestWindow,
    moved: Rc<RefCell<Vec<f32>>>,
    released: Rc<RefCell<usize>>,
}

impl Rig {
    /// `stored` is the width the column starts from, before the layout has its
    /// say; `None` leaves it at the column's minimum.
    fn new(stored: Option<f32>) -> Rig {
        i_slint_backend_testing::init_no_event_loop();
        let window = TestWindow::new().unwrap();
        if let Some(stored) = stored {
            window.set_stored(stored);
        }
        let moved: Rc<RefCell<Vec<f32>>> = Rc::default();
        let released: Rc<RefCell<usize>> = Rc::default();
        window.on_moved({
            let moved = Rc::clone(&moved);
            let weak = window.as_weak();
            move |w| {
                // The consumer's whole job (the app's own wiring): assign the
                // raw property and let the layout bound it.
                weak.unwrap().set_stored(w);
                moved.borrow_mut().push(w);
            }
        });
        window.on_released({
            let released = Rc::clone(&released);
            move || *released.borrow_mut() += 1
        });
        window.show().unwrap();
        Rig {
            window,
            moved,
            released,
        }
    }

    fn minimum(&self) -> f32 {
        self.window.get_column_min()
    }

    /// The middle of the grip, wherever the layout has just put it.
    fn grip_centre(&self) -> f32 {
        self.window.get_grip_x() + GRIP / 2.0
    }

    fn send(&self, event: WindowEvent) {
        self.window.window().dispatch_event(event);
    }

    /// Presses in the middle of the grip and returns the anchor that press
    /// took.
    fn press(&self) -> f32 {
        let x = self.grip_centre();
        self.move_to(x);
        self.send(WindowEvent::PointerPressed {
            position: LogicalPosition::new(x, 20.0),
            button: PointerEventButton::Left,
        });
        x
    }

    fn move_to(&self, x: f32) {
        self.send(WindowEvent::PointerMoved {
            position: LogicalPosition::new(x, 20.0),
        });
    }

    fn release(&self, x: f32) {
        self.send(WindowEvent::PointerReleased {
            position: LogicalPosition::new(x, 20.0),
            button: PointerEventButton::Left,
        });
    }

    fn laid_out(&self) -> f32 {
        self.window.get_laid_out()
    }

    fn last_moved(&self) -> f32 {
        *self.moved.borrow().last().expect("a width was emitted")
    }
}

fn close(got: f32, want: f32, what: &str) {
    assert!(
        (got - want).abs() < 1e-3,
        "{what}: got {got}, wanted {want}"
    );
}

/// W5, both rules at once, from a bounded state: a drag past the minimum stops
/// there, and a drag back picks the column up where the pointer left it — 30px
/// right of the anchor is 30px wider than the press, with nothing owed for the
/// distance the drag spent out of range. The incremental form passes the first
/// half of this and fails the second.
#[test]
fn a_drag_past_the_minimum_stops_and_comes_back_with_the_pointer() {
    let rig = Rig::new(None);
    let min = rig.minimum();
    let anchor = rig.press();
    // The grip sets its own 6px inside the layout, which is what the column
    // and the player are laid out around — and what `grip_centre` presses in
    // the middle of.
    close(rig.window.get_grip_width(), GRIP, "the grip's width");
    close(rig.laid_out(), min, "the column starts at its minimum");
    assert!(
        rig.moved.borrow().is_empty(),
        "a press alone is not a resize"
    );

    for past in [40.0, 60.0, 150.0] {
        rig.move_to(anchor - past);
        close(
            rig.laid_out(),
            min,
            "the column holds its minimum however far the drag goes",
        );
    }

    rig.move_to(anchor + 30.0);
    close(rig.last_moved(), min + 30.0, "the width emitted");
    close(
        rig.laid_out(),
        min + 30.0,
        "the column follows the pointer again",
    );

    rig.release(anchor + 30.0);
    assert_eq!(*rig.released.borrow(), 1);
}

/// W5's second rule: the press anchors on the column's **laid-out** width, so
/// a drag that begins while the bound is active moves the column at once.
/// Anchoring on the raw property — 20px here — would emit 50 and leave the
/// column sitting at its minimum until the pointer had dragged 60px of
/// nothing.
#[test]
fn a_drag_from_a_bounded_state_anchors_on_the_laid_out_width() {
    let rig = Rig::new(Some(20.0));
    let min = rig.minimum();
    close(rig.laid_out(), min, "the layout bounds the stored 20px up");
    let anchor = rig.press();
    rig.move_to(anchor + 30.0);
    close(rig.last_moved(), min + 30.0, "the width emitted");
    close(rig.laid_out(), min + 30.0, "the column moved at once");
}

/// The inspector's side: the grip is to the column's left, so dragging left
/// widens it. Only the sign is under test — which side of the player a column
/// sits on is the layout's business, not the splitter's.
#[test]
fn a_panel_on_the_right_widens_as_the_pointer_goes_left() {
    let rig = Rig::new(Some(120.0));
    rig.window.set_panel_on_the_right(true);
    let anchor = rig.press();
    rig.move_to(anchor - 40.0);
    close(
        rig.last_moved(),
        160.0,
        "left is wider for a panel on the right",
    );
    rig.move_to(anchor + 40.0);
    close(rig.last_moved(), 80.0, "and right is narrower");
}

/// A drag off the edge of the window is cancelled, not released (the reason is
/// in `scrubber.slint`'s header). The drag has to end there, or the width the
/// coach set is never stored.
///
/// The other half of that worry — a bare pointer move going on resizing the
/// column afterwards — **cannot be tested from here, and does not need to be.**
/// `i-slint-core`'s `input_items.rs` clears the grab on `MouseEvent::Exit` and
/// only calls `moved()` `if self.grabbed.get()`, so after a cancel no pointer
/// move reaches the component at all, whatever its own `dragging` says. A test
/// of it would pass with the cancel branch deleted and prove nothing.
#[test]
fn a_cancelled_drag_releases_and_ends() {
    let rig = Rig::new(Some(120.0));
    let anchor = rig.press();
    rig.move_to(anchor + 20.0);
    close(rig.last_moved(), 140.0, "the drag was under way");

    rig.send(WindowEvent::PointerExited);
    assert_eq!(*rig.released.borrow(), 1, "the drag never released");
}

/// **A bare click on the grip changes nothing.** The grip is 6px and easy to hit
/// by accident, and the consumer's `released` handler stores the width the layout
/// settled on — so without the `resized` flag a mis-click would quietly cut a
/// column the coach had widened on a bigger monitor down to whatever today's
/// window can give. A behaviour deliberately chosen deserves a test, which is
/// why this one exists even though nothing in the component looks fragile.
///
/// The third act is the one worth having: the flag is cleared on the press, so a
/// click *after* a real drag cannot inherit that drag's permission to write.
#[test]
fn a_bare_click_is_not_a_drag_and_never_releases() {
    let rig = Rig::new(Some(120.0));
    let anchor = rig.press();
    rig.release(anchor);
    assert_eq!(*rig.released.borrow(), 0, "a click wrote a width");
    assert!(
        rig.moved.borrow().is_empty(),
        "a click emitted a width: {:?}",
        rig.moved.borrow()
    );

    // A real drag still releases exactly once.
    let anchor = rig.press();
    rig.move_to(anchor + 20.0);
    rig.release(anchor + 20.0);
    assert_eq!(*rig.released.borrow(), 1, "a drag did not release");

    // And a click after it does not inherit the drag's flag.
    let anchor = rig.press();
    rig.release(anchor);
    assert_eq!(
        *rig.released.borrow(),
        1,
        "a click after a drag released again"
    );
}

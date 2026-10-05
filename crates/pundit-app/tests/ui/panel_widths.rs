//! The side columns' bound, against the **real** `AppWindow` on Slint's
//! headless backend (spec W10 test 6 of
//! `docs/superpowers/specs/2026-09-26-panels-and-fit-design.md`).
//!
//! The bound is not code, it is four layout constraints per column, so the only
//! thing that can check it is a layout — and it has to be this window's, because
//! what the columns are reconciled against is the rest of this window's row.
//! A stand-in would reconcile against a stand-in.
//!
//! **Nothing here derives the player's width by subtracting the columns from
//! the window.** The splitters are 12px of that row which such a sum forgets,
//! and an earlier draft of this test failed on exactly that. Every assertion is
//! against the window's own `player-min`, `sidebar-min` and `inspector-min`.
//!
//! Logical pixels throughout: this backend's scale factor is a hardcoded 1, so
//! a `PhysicalSize` and a `length` are the same number here. The widths are the
//! subject, never the window's floor — the backend honours any size it is
//! handed, under the declared minimum included, which is `fit.rs`'s note and
//! not this file's business.
//!
//! **Nothing here is a tripwire for the rows inside a column, and none can be.**
//! An explicit `min-width` *replaces* the children-derived constraint
//! (`i-slint-compiler/passes/lower_layout.rs`), and a preferred width is clamped
//! into `[min, max]` with the minimum winning
//! (`i-slint-core/layout.rs`), so a control too wide for its column
//! is clipped in silence — exactly as it was under the fixed 240px and 280px
//! this replaces. A test asserting a column sits at its own minimum therefore
//! cannot fail, whatever is put in it. Fitting a row to its column is still done
//! by looking, as the "One action, not two" note in `TranscriptRow` was.

use pundit_app::bus::PanelWidths;
use slint::platform::{PointerEventButton, WindowEvent};
use slint::{ComponentHandle, LogicalPosition, ModelRc, PhysicalSize, SharedString, VecModel};

// The window as `main.rs` builds it: the row the columns share is the test. The
// generated UI is compiled once for this whole test binary, in its `main.rs`
// (BACKLOG #121).
use crate::{AppWindow, ClipRow, SlateRow, SourceRow};

/// A window carrying what both columns were fitted to hold — a source, a clip,
/// a slate and a selected clip's inspector, with names long enough to want more
/// room than they get. **The content is the point**, not decoration: an empty
/// column has no children to have an intrinsic minimum, so an empty fixture
/// could not tell a column laid out at its declared minimum from one pushed
/// wider by something inside it.
fn window() -> AppWindow {
    i_slint_backend_testing::init_no_event_loop();
    let w = AppWindow::new().unwrap();
    w.set_has_project(true);
    w.set_can_play(true);
    w.set_sources(ModelRc::new(VecModel::from(vec![SourceRow {
        name: "a very long source file name indeed 2026-09-26.mp4".into(),
        duration: "1:45:30".into(),
        missing: false,
        referenced: true,
    }])));
    w.set_clips(ModelRc::new(VecModel::from(vec![ClipRow {
        id: "c1".into(),
        name: "an extremely long clip name nobody would type".into(),
        duration: "0:42".into(),
    }])));
    w.set_slates(ModelRc::new(VecModel::from(vec![SlateRow {
        id: "s1".into(),
        range: "14:05–14:40".into(),
        video: "a long source name".into(),
        name: "a long slate name as well".into(),
        tags: "corner, set-piece, defending".into(),
        shot: false,
        timed: true,
    }])));
    w.set_transcript_models(ModelRc::new(VecModel::from(vec![
        SharedString::from("base.en"),
        SharedString::from("small.en"),
    ])));
    // Which shows the clip inspector, the transcript row included — the row the
    // inspector's 280px was measured against.
    w.set_selected_clip("c1".into());
    w.set_clip_transcript("a transcript that runs on for a good while".into());
    w.show().unwrap();
    w
}

/// The layout solves in `f32` and a remainder can land on either side of a
/// whole pixel; half a pixel is under anything this file is about.
fn about(got: f32, want: f32, what: &str) {
    assert!(
        (got - want).abs() < 0.5,
        "{what}: expected {want}, laid out at {got}"
    );
}

/// **A stored pair too wide for the window puts nothing under its minimum.**
/// Widths come back from `state.json` unchecked — a bigger screen, a
/// hand-edited file — and the pair here asks for 800px of a 1100px window while
/// one of the two is also below its own floor. Both failures at once, because
/// both are the same declaration.
///
/// The player is what must survive it: it is the item with the stretch, so it
/// is where a column that overreaches takes its width from, and its own
/// minimum is the only thing that stops it. A `clamp` on `width` leaves it at
/// 308px here, which is the measurement in the comment above the sidebar's
/// constraints and the reason they are constraints.
#[test]
fn a_stored_pair_too_wide_for_the_window_keeps_every_minimum() {
    let w = window();
    w.window().set_size(PhysicalSize::new(1100, 700));
    w.set_sidebar_width(100.0);
    w.set_inspector_width(700.0);

    assert!(
        w.get_sidebar_width_now() >= w.get_sidebar_min(),
        "the sidebar was laid out at {} against a {} minimum",
        w.get_sidebar_width_now(),
        w.get_sidebar_min()
    );
    assert!(
        w.get_inspector_width_now() >= w.get_inspector_min(),
        "the inspector was laid out at {} against a {} minimum",
        w.get_inspector_width_now(),
        w.get_inspector_min()
    );
    about(
        w.get_player_width(),
        w.get_player_min(),
        "the player should be squeezed to its own minimum and no further",
    );
}

/// **A width the window can hold is honoured exactly.** Without this the test
/// above would pass on `min-width` alone, and a coach who dragged the inspector
/// wider would get 280px back on the next launch.
#[test]
fn a_stored_width_the_window_can_hold_is_what_the_column_gets() {
    let w = window();
    w.window().set_size(PhysicalSize::new(1600, 960));
    w.set_sidebar_width(400.0);
    w.set_inspector_width(500.0);

    about(w.get_sidebar_width_now(), 400.0, "the sidebar");
    about(w.get_inspector_width_now(), 500.0, "the inspector");
    assert!(
        w.get_player_width() > w.get_player_min(),
        "the window was meant to have room to spare: player {}",
        w.get_player_width()
    );
}

/// **The two copies of 240 and 280 are the same numbers, by test.** The layout
/// states its minima in `app.slint` because that is where every other layout
/// number lives; `state.json`'s defaults are in Rust because that is what
/// deserializes. Nothing binds them, and the comment on either one claiming they
/// agree is only true while this passes.
#[test]
fn the_stored_defaults_are_the_layouts_minima() {
    let w = window();
    let defaults = PanelWidths::default();
    assert_eq!(defaults.sidebar as f32, w.get_sidebar_min());
    assert_eq!(defaults.inspector as f32, w.get_inspector_min());
}

/// **A release stores the width the coach can see, not the one the pointer asked
/// for** (spec W5) — the one rule of this shipment nothing else can pin.
/// `tests/ui/splitter.rs` drives the component against a stand-in window that has
/// no write-back of its own, and `main.rs`'s `state.json` write is in a `[[bin]]`
/// no test can reach; what is left, and what is under test here, is `app.slint`'s
/// `released` handler on the real grip.
///
/// The drag deliberately ends **bounded** — 200px wider than a 1100px window can
/// give — because that is the only state in which the raw property and the
/// laid-out width differ at all. Drag to a width that fits and the two agree
/// whether the handler exists or not.
///
/// The inspector is left at a width the window can hold, so the sidebar is the
/// only column the bound is acting on. With both of them overreaching the layout
/// shares the shortfall between them, and writing one back changes what the
/// other is given — one more pass, converging, and nothing this rule is about.
///
/// The second drag is the other half: the stored width having been pulled back
/// to what was drawn, a drag from there moves by exactly the pointer's delta.
/// Nothing is owed for the distance the first drag spent out of range.
#[test]
fn a_release_stores_the_width_the_layout_settled_on() {
    let w = window();
    w.window().set_size(PhysicalSize::new(1100, 700));
    w.set_sidebar_width(400.0);
    w.set_inspector_width(w.get_inspector_min());
    about(
        w.get_sidebar_width_now(),
        400.0,
        "the sidebar starts honoured",
    );

    let asked = drag(&w, 200.0);
    assert!(
        asked > w.get_sidebar_width_now() + 1.0,
        "the drag was meant to end bounded: asked {asked}, laid out {}",
        w.get_sidebar_width_now()
    );
    about(
        w.get_sidebar_width(),
        w.get_sidebar_width_now(),
        "the release should store what was drawn, not what was asked for",
    );

    let settled = w.get_sidebar_width_now();
    drag(&w, -60.0);
    about(
        w.get_sidebar_width_now(),
        settled - 60.0,
        "a drag from a settled width should move by the pointer's delta",
    );
}

/// Presses the middle of the sidebar's grip — which begins where the column the
/// layout drew ends — drags `by` logical pixels and releases there. Returns the
/// raw width that drag asked for, which the bound may or may not have granted.
fn drag(w: &AppWindow, by: f32) -> f32 {
    let send = |event| w.window().dispatch_event(event);
    let x = w.get_sidebar_width_now() + 3.0;
    send(WindowEvent::PointerMoved {
        position: LogicalPosition::new(x, 20.0),
    });
    send(WindowEvent::PointerPressed {
        position: LogicalPosition::new(x, 20.0),
        button: PointerEventButton::Left,
    });
    send(WindowEvent::PointerMoved {
        position: LogicalPosition::new(x + by, 20.0),
    });
    let asked = w.get_sidebar_width();
    send(WindowEvent::PointerReleased {
        position: LogicalPosition::new(x + by, 20.0),
        button: PointerEventButton::Left,
    });
    asked
}

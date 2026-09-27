//! Fit window to video against the **real** `AppWindow`, on Slint's headless
//! backend (spec W10 of
//! `docs/superpowers/specs/2026-09-26-panels-and-fit-design.md`).
//!
//! **This file cannot test that `f` resizes the window.** `on_fit_window` lives
//! in `main.rs`, which is a separate `[[bin]]`; nothing under `tests/` can
//! reach it, and a test that installed a handler of its own would be testing
//! its own copy. Pressing `f` on a real window — maximised, fullscreen and
//! neither — is the runtime check in the plan's A4, and that is what covers the
//! wiring. What a test can own is the two halves either side of it: that the
//! numbers `fit_window` is fed really do describe the layout the app lays out,
//! and that the key is gated before it ever gets there.
//!
//! **An unwired `place-picture` returns `PictureRect::default()`.**
//! `content-width` and `content-height` are bound to it, so with no handler
//! installed every content-rect assertion below holds on 0×0 and this file
//! measures nothing while reporting fine. That is how the vacuous version of
//! this test gets written. The window here wires the production
//! [`Viewport`](pundit_app::zoom_input::Viewport) exactly as `main.rs` does,
//! and the first test asserts the rect is non-empty before it asserts anything
//! about it.
//!
//! **The backend can see neither the rounding nor the floor**: it honours any
//! size it is handed, fractional or under the declared minimum, and converts at
//! a hardcoded scale factor of 1. Those are `fit.rs`'s own unit tests and
//! nothing here restates them — the fact only a window knows is that
//! `window − player` is the chrome.

use std::cell::Cell;
use std::rc::Rc;

use pundit_app::fit::{fit_window, Fit};
use pundit_app::zoom_input::Viewport;
use pundit_core::zoom::Zoom;
use slint::platform::WindowEvent;
use slint::{ComponentHandle, ModelRc, PhysicalSize, SharedString, VecModel};

// The window itself, as `main.rs` builds it. A stand-in would have a stand-in's
// chrome, and the chrome is the whole subject.
slint::include_modules!();

/// The window with the parts of `main.rs` this file measures against: the
/// production letterbox, footage on screen, and a 16:9 frame.
fn window() -> AppWindow {
    i_slint_backend_testing::init_no_event_loop();
    let w = AppWindow::new().unwrap();
    // `main.rs`'s handler verbatim. The content rect is what the chrome is
    // being checked against, so a simplified letterbox here would check the
    // simplification.
    w.on_place_picture(|zoom, frame_w, frame_h, area_w, area_h| {
        let zoom = Zoom::new(zoom.scale.into(), zoom.pan_x.into(), zoom.pan_y.into());
        let Some(vp) = Viewport::new(frame_w.into(), frame_h.into(), area_w.into(), area_h.into())
        else {
            return PictureRect::default();
        };
        let r = vp.picture(zoom);
        PictureRect {
            x: r.x as f32,
            y: r.y as f32,
            width: r.width as f32,
            height: r.height as f32,
        }
    });
    w.set_can_play(true);
    w.set_frame_width(1920.0);
    w.set_frame_height(1080.0);
    w.show().unwrap();
    w
}

/// The four pairs `main.rs`'s `fit_now` assembles, and the only part of it
/// worth copying: all four are read off the live window, and whether they
/// describe its layout is exactly what `fit.rs`'s unit tests cannot know. The
/// `× scale_factor()` is identity on this backend and kept so the reads are the
/// app's.
///
/// Because this is *our* assembly, it cannot catch a wrong one in `main.rs` — a
/// dropped `× scale`, the wrong floor. What is under test is the chrome
/// subtraction against a real layout, nothing more.
fn fit_now(w: &AppWindow) -> Fit {
    let window = w.window();
    let scale = f64::from(window.scale_factor());
    let size = window.size();
    fit_window(
        (
            f64::from(w.get_content_width()) * scale,
            f64::from(w.get_content_height()) * scale,
        ),
        (
            f64::from(w.get_player_width()) * scale,
            f64::from(w.get_player_height()) * scale,
        ),
        (f64::from(size.width), f64::from(size.height)),
        (
            f64::from(w.get_min_window_width()) * scale,
            f64::from(w.get_min_window_height()) * scale,
        ),
    )
}

/// **The chrome subtraction predicts the real layout.** `window − player` is
/// the fit's one assumption about the app (spec W2), and a pure function can
/// only assume it: here the window is resized to the fit's own answer and asked
/// what it laid out.
///
/// The promise is spec W1's — the window closes up around the picture already
/// on screen, so the drawn size is unchanged — and the content rect is the
/// assertion that carries it. The second fit is idempotence in the real layout
/// rather than in arithmetic, and it is **weaker than it looks**: measured, a
/// target floored by one pixel still answers `NoSlack` the second time round,
/// because the re-fit's own `ceil` lands back on the window it was given. Only
/// the content rect catches a wrong target, so it is not the assertion to drop.
///
/// Nothing about the picture's *position*: it moves up by the bar that went
/// away, and that is the design. Nothing hard-coded either — the numbers here
/// are the window's own, because the column widths and the transport's height
/// change with a splitter or a button.
#[test]
fn the_chrome_subtraction_predicts_the_real_layout() {
    let w = window();
    w.window().set_size(PhysicalSize::new(1600, 960));

    let content = (w.get_content_width(), w.get_content_height());
    let player_width = w.get_player_width();
    // With `place-picture` unwired this is 0×0 and everything below passes on
    // nothing; see the module header.
    assert!(
        content.0 > 0.0 && content.1 > 0.0,
        "no picture to fit the window to: {content:?}"
    );

    let Fit::To(width, height) = fit_now(&w) else {
        panic!("16:9 footage in a 1600x960 window has black above and below it");
    };
    w.window().set_size(PhysicalSize::new(width, height));

    assert_eq!(
        (w.get_content_width(), w.get_content_height()),
        content,
        "the fit rescaled the picture instead of taking the bars off"
    );
    assert_eq!(
        w.get_player_width(),
        player_width,
        "the width was not the axis with the slack, so it should not have moved"
    );
    assert_eq!(
        fit_now(&w),
        Fit::NoSlack,
        "there is still black to take off at {width}x{height}, so the fit missed"
    );
}

/// **`f` is gated on `can-fit` and swallowed by a focused field.** The gate is
/// the failure the slates pass shipped: a letter shortcut that fires while the
/// coach is typing a name into a field. Only the callback is observed — what
/// `main.rs` then does with it is A4's check, per the module header.
#[test]
fn the_f_key_is_gated_and_a_field_swallows_it() {
    let w = window();
    let fits = Rc::new(Cell::new(0u32));
    w.on_fit_window({
        let fits = Rc::clone(&fits);
        move || fits.set(fits.get() + 1)
    });

    // Grey button, silent key: the one answer from Rust gates both.
    w.set_can_fit(false);
    press_f(&w);
    assert_eq!(fits.get(), 0, "f fitted a window with nothing to take off");

    w.set_can_fit(true);
    press_f(&w);
    assert_eq!(fits.get(), 1, "f did not reach the fit");

    // A slate's name is a place the coach types an f. The window folds the
    // field's focus into `text-editing`, which `handle-key` rejects on before
    // any letter branch. A slate row appears in a test about fitting only
    // because `focus-slate-name` is the cheapest seam that raises
    // `text-editing` — it is derived, so a test cannot set it. `slate_fields.rs`
    // pins the same fold for `i` and `o`; what this adds is that the **`f`**
    // branch sits below it, which is positional and could be got wrong on its
    // own.
    w.set_slates(ModelRc::new(VecModel::from(vec![SlateRow {
        id: "s1".into(),
        range: "0:05–0:12".into(),
        video: SharedString::new(),
        name: "corner".into(),
        tags: SharedString::new(),
        shot: false,
    }])));
    w.set_selected_slate("s1".into());
    w.invoke_focus_slate_name();
    press_f(&w);
    assert_eq!(fits.get(), 1, "an f typed into a field resized the window");
}

fn press_f(w: &AppWindow) {
    w.window()
        .dispatch_event(WindowEvent::KeyPressed { text: 'f'.into() });
    w.window()
        .dispatch_event(WindowEvent::KeyReleased { text: 'f'.into() });
}

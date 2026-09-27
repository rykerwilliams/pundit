//! Fitting the window to the footage's shape (spec W1, W6 and W8 of
//! `docs/superpowers/specs/2026-09-26-panels-and-fit-design.md`). The picture
//! is never re-fitted: the window closes up around the picture the player is
//! already drawing, so the letterbox bars go and nothing is rescaled.
//!
//! **Every input is read off the window, never derived from it** (spec W2). The
//! chrome is `window − player`; the picture is the window's own `content-width`
//! / `content-height`, which is `place-picture` at identity zoom — the very
//! expression the player draws with. An earlier version rebuilt that rect here
//! from the frame's size through [`Viewport`](crate::zoom_input::Viewport), which
//! was the same arithmetic twice and left this module having to argue that a
//! helper documented in *logical* pixels was safe to use in physical ones. It is
//! now the window's answer, and `Viewport` appears only in the tests, where it
//! is an independent oracle instead of a second copy of the implementation.
//!
//! **Physical pixels.** A window size is only ever asked for in physical ones,
//! because the two logical conversions disagree — `PhysicalSize::from_logical`
//! truncates where winit's path rounds — and at a fractional scale factor a
//! logical `ceil` is not enough to keep the picture width-limited. The caller
//! scales the logical pairs by `scale_factor()`, which **is** a conversion; its
//! round-trip error is ~1e-4 px and the `ceil` absorbs it. "No conversion after
//! this" is about the target, not the inputs.
//!
//! Unlike the rest of the input path, this module checks its own numbers rather
//! than trusting the window to have dropped the unusable ones (BACKLOG #28):
//! they are read off a window that may be mid-layout, which is a state no caller
//! can promise away. Anything unusable answers [`Fit::NoSlack`], which is also
//! the right answer for a layout that has not settled — it offers the coach
//! nothing.

/// What fitting the window to the displayed frame would do.
///
/// Three outcomes, not two, and the third is not spare: `can-fit` is
/// `!NoSlack`, so the same answer that gates the offer would also have to carry
/// the refusal. Collapsed into two, "too small to fit" could never be reached —
/// the button would already be grey and the key already silent — and the notice
/// explaining it would be dead code (spec W6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fit {
    /// Nothing to take off: the player is already at the frame's shape, or the
    /// numbers do not describe a settled layout.
    NoSlack,
    /// The fit would put the window under its own declared minimum. The caller
    /// says so and must not simply ask: that minimum is a hint the window
    /// manager clamps against rather than a request that fails, so asking
    /// leaves the coach with the bars *and* a moved window.
    TooSmall,
    /// Resize to this, physical pixels.
    To(u32, u32),
}

/// What fitting `window` to the picture it is drawing would do. `content` is the
/// letterboxed picture, `player` the area it is drawn in, `min` the window's
/// declared floor; all four are physical pixels.
///
/// The chrome is `window − player` per axis, measured rather than derived:
/// whatever is not the player **is** the chrome, so a new transport button or a
/// dragged panel cannot put it out of date (spec W2). `content` is read for the
/// same reason.
pub fn fit_window(
    content: (f64, f64),
    player: (f64, f64),
    window: (f64, f64),
    min: (f64, f64),
) -> Fit {
    if ![
        content.0, content.1, player.0, player.1, window.0, window.1, min.0, min.1,
    ]
    .iter()
    .all(|v| v.is_finite() && *v > 0.0)
    {
        return Fit::NoSlack;
    }
    let chrome = (window.0 - player.0, window.1 - player.1);
    if chrome.0 < 0.0 || chrome.1 < 0.0 {
        // A player larger than the window it sits in: the two numbers were
        // read a layout pass apart. Nothing to offer until they agree.
        return Fit::NoSlack;
    }

    // The letterbox itself says which axis has the slack: whatever the picture
    // does not fill is bar, and the fit is the window with that bar taken out.
    // The `else` covers a pillarboxed player and an exactly-fitted one alike —
    // the latter falls out as a no-op below rather than needing a third branch.
    let target = if content.1 < player.1 {
        (window.0, content.1 + chrome.1)
    } else {
        (content.0 + chrome.0, window.1)
    };

    // Rounded **up**, and that is load-bearing. Measured (spec W0): a 16:9 fit
    // at 1600 wide wants 715.5; at 716 the picture is still width-limited and
    // drawn at exactly the size it was, while a floored 715 makes it
    // height-limited and shrinks it by 0.89 px of width — the one thing the fit
    // promises not to do.
    //
    // Both axes are ceiled, though only one moved: the one that did not is the
    // window's own value, which the real caller reads from a
    // `PhysicalSize<u32>`, so the `ceil` is identity there. It is here for the
    // fractional sizes the headless backend allows, where truncating a
    // pass-through would shrink the picture on the axis that was already tight.
    // The cost is that a fractional window could be asked to grow by half a
    // pixel on the untouched axis — unreachable from a `PhysicalSize<u32>`, and
    // the alternative trades it for a half-pixel shrink in the same case.
    let fit = (target.0.ceil(), target.1.ceil());

    if fit.0 >= window.0 && fit.1 >= window.1 {
        // Strictly smaller or nothing. The axis that did not move is the
        // window's own value, so this is a test of the one that did, and it
        // carries three promises at once: the fit never grows (the `ceil` of an
        // exact fit lands a half-pixel above the window it came from), it is a
        // no-op once the player is at the frame's aspect, and it is therefore
        // idempotent — a stray second press cannot walk the window down.
        return Fit::NoSlack;
    }
    if fit.0 < min.0 || fit.1 < min.1 {
        // Either axis, not only the one that moved. The other is the window's
        // own width or height, and a window already under the floor — a window
        // manager that ignored the hint — must not be resized on one axis into
        // a size the same manager then clamps on the other: half-fitted is
        // worse than not fitted, being bars *and* a moved window.
        return Fit::TooSmall;
    }
    Fit::To(fit.0 as u32, fit.1 as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zoom_input::Viewport;
    use pundit_core::zoom::Zoom;

    /// 16:9, the shape every measurement in spec W0 was taken against.
    const FRAME: (f64, f64) = (1920.0, 1080.0);

    /// The window's declared floor (spec W4), at scale factor 1.
    const MIN: (f64, f64) = (1100.0, 700.0);

    /// Measured 2026-09-26, and constant at and above the floor: 520 is the two
    /// columns, 108 the transport bar (spec W0). Nothing here may depend on the
    /// values — they change with a splitter or a transport button — only on the
    /// subtraction being what the app does. **It has already changed, and this
    /// number is deliberately not chased:** #87's two 6px grips joined the same
    /// row, so the app's own chrome is now 532x108 (measured constant at
    /// 1100x700, 1600x960 and 1920x1080). These tests are arithmetic over a
    /// synthetic chrome, so the shipped value is `panel_widths.rs` and
    /// `fit_window.rs`'s business, not this constant's.
    const CHROME: (f64, f64) = (520.0, 108.0);

    fn player(window: (f64, f64)) -> (f64, f64) {
        (window.0 - CHROME.0, window.1 - CHROME.1)
    }

    /// The picture as the player draws it at that window size — `Viewport`'s
    /// answer, which is what the window hands the fit. Here it is an
    /// **independent oracle**: the fit no longer computes it, so a test built on
    /// it can disagree with the implementation.
    fn picture(frame: (f64, f64), window: (f64, f64)) -> (f64, f64) {
        let area = player(window);
        let rect = Viewport::new(frame.0, frame.1, area.0, area.1)
            .unwrap()
            .picture(Zoom::IDENTITY);
        (rect.width, rect.height)
    }

    /// The strongest assertion available without a window (spec W10): fitting
    /// closes the window up around the picture, so the drawn size is the same
    /// to the pixel. The *position* is deliberately not checked — the picture
    /// moves up by the bar that went away (spec W1).
    fn fits_without_touching_the_picture(frame: (f64, f64), window: (f64, f64)) -> (f64, f64) {
        let Fit::To(w, h) = fit_window(picture(frame, window), player(window), window, MIN) else {
            panic!("expected a fit for {window:?}");
        };
        let fitted = (f64::from(w), f64::from(h));
        let (before, after) = (picture(frame, window), picture(frame, fitted));
        assert!(
            (before.0 - after.0).abs() < 1e-9 && (before.1 - after.1).abs() < 1e-9,
            "picture {before:?} became {after:?} at {fitted:?}"
        );
        fitted
    }

    /// W0's first row: 1600×960 with 16:9 footage, 122 px of black above and
    /// below. The fit wants 715.5, so the window goes to 716 and the picture is
    /// drawn at exactly the size it already was.
    #[test]
    fn the_target_is_ceiled_and_the_picture_is_untouched() {
        let window = (1600.0, 960.0);
        assert_eq!(
            fit_window(picture(FRAME, window), player(window), window, MIN),
            Fit::To(1600, 716)
        );
        fits_without_touching_the_picture(FRAME, window);
        // 715 is the floored answer, and this is what it costs: a 607 px player
        // turns the picture height-limited and it is drawn 1079.11 × 607 —
        // 0.89 px of width and half a pixel of height gone, for nothing. That
        // is spec W0's last row; its prose calls the loss ~1.9 px, which the
        // table it sits under does not support.
        let floored = picture(FRAME, (1600.0, 715.0));
        let cost = picture(FRAME, window).0 - floored.0;
        assert!((0.8..1.0).contains(&cost), "a floor would cost {cost} px");
    }

    /// The other axis, and the same rounding: an ultrawide window pillarboxes
    /// the picture, so the width is what shrinks. The 892 px player is not a
    /// multiple of 9, so the target (2105.78) is fractional here too.
    #[test]
    fn a_pillarboxed_player_shrinks_the_width() {
        let window = (2560.0, 1000.0);
        assert_eq!(
            fit_window(picture(FRAME, window), player(window), window, MIN),
            Fit::To(2106, 1000)
        );
        fits_without_touching_the_picture(FRAME, window);
    }

    /// At the frame's own aspect there is nothing to take off, and the answer
    /// for a fitted window is the same answer again — which is what makes a
    /// stray second press harmless. The exact case also pins the never-grow
    /// half of the rule: the fit of 715.5 ceils to 716, and `NoSlack` is what
    /// stops that half pixel becoming a resize.
    #[test]
    fn at_the_frames_aspect_there_is_no_slack() {
        let exact = (1600.0, 715.5);
        assert_eq!(
            fit_window(picture(FRAME, exact), player(exact), exact, MIN),
            Fit::NoSlack
        );

        let fitted = fits_without_touching_the_picture(FRAME, (1600.0, 960.0));
        assert_eq!(
            fit_window(picture(FRAME, fitted), player(fitted), fitted, MIN),
            Fit::NoSlack
        );
    }

    /// Refused when the fit would need a window under its own floor — and
    /// refused as `TooSmall`, not `NoSlack`, because that distinction is the
    /// only thing that makes the notice reachable (spec W6).
    #[test]
    fn under_the_floor_on_either_axis_is_too_small() {
        // A 1200-wide window leaves a 680 px player, whose picture is 382.5
        // tall: a 491 px window, 209 under the floor.
        let narrow = (1200.0, 900.0);
        assert_eq!(
            fit_window(picture(FRAME, narrow), player(narrow), narrow, MIN),
            Fit::TooSmall
        );

        // The axis that does not move counts too. Here the fitted width (1395)
        // clears the floor and the untouched height does not, which is only
        // reachable from a window already under it — the headless backend, or a
        // window manager that ignored the hint (spec W0, measurement 3). Below
        // the floor the real chrome is not 520×108 either; what is under test
        // is which axes the comparison reads.
        let short = (1600.0, 600.0);
        assert_eq!(
            fit_window(picture(FRAME, short), player(short), short, MIN),
            Fit::TooSmall
        );
    }

    /// Junk in, `NoSlack` out: the offer withdrawn rather than a refusal
    /// explained, because none of these describe a layout the coach is looking
    /// at — one check over all eight numbers (BACKLOG #28).
    #[test]
    fn unusable_numbers_are_no_slack() {
        let window = (1600.0, 960.0);
        let area = player(window);
        assert_eq!(fit_window((0.0, 607.5), area, window, MIN), Fit::NoSlack);
        assert_eq!(
            fit_window(picture(FRAME, window), (area.0, f64::NAN), window, MIN),
            Fit::NoSlack
        );
        assert_eq!(
            fit_window(picture(FRAME, window), area, (f64::INFINITY, window.1), MIN),
            Fit::NoSlack
        );
        assert_eq!(
            fit_window(picture(FRAME, window), area, window, (MIN.0, 0.0)),
            Fit::NoSlack
        );
        // A player taller than its own window: two numbers read a layout pass
        // apart, not a shape to fit.
        assert_eq!(
            fit_window(
                picture(FRAME, window),
                (area.0, window.1 + 1.0),
                window,
                MIN
            ),
            Fit::NoSlack
        );
    }
}

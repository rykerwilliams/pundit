//! The arrowhead at the end of a stroke: where it points, how big it is, and
//! when there is no head to draw.
//!
//! **Everything here is checked in pixels**, by denormalizing through a
//! picture of a stated size, because that is the only space the head's shape
//! means anything in: a stroke's x is normalized to width and its y to height,
//! so the triangle `arrow_head` returns is deliberately skewed in normalized
//! space and comes out straight only once a drawer has applied
//! `(x * w, y * h)`. A test that read the normalized numbers would be pinning
//! the skew rather than the arrow.

use pundit_core::layout::{STROKE_LINE_WIDTH, STROKE_LINE_WIDTH_THICK};
use pundit_core::stroke::{
    arrow_head, StrokePoint, ARROW_HEAD_HALF_WIDTH_RATIO, ARROW_HEAD_LENGTH_RATIO,
};

/// The reference picture: 1080p, 16:9, where the default pen is 5.4 px.
const W: f64 = 1920.0;
const H: f64 = 1080.0;

/// Content-rect aspects an export or a preview actually letterboxes to —
/// 16:9, 4:3 (a pillarboxed source), 2.39:1 and square — plus one extreme, so
/// a skew would have nowhere to hide.
const ASPECTS: [f64; 5] = [16.0 / 9.0, 4.0 / 3.0, 2.39, 1.0, 0.5];

/// A stroke through `xy`, normalized, one point every 20 ms.
fn stroke(xy: &[(f64, f64)]) -> Vec<StrokePoint> {
    xy.iter()
        .enumerate()
        .map(|(i, &(x, y))| StrokePoint {
            x,
            y,
            t: i as f64 * 0.02,
        })
        .collect()
}

/// A stroke through points given in pixels of a `w` × `h` picture, which is
/// how every test here describes the shape it means.
fn in_pixels(xy: &[(f64, f64)], w: f64, h: f64) -> Vec<StrokePoint> {
    let normalized: Vec<(f64, f64)> = xy.iter().map(|&(x, y)| (x / w, y / h)).collect();
    stroke(&normalized)
}

/// The head's three corners in pixels of a `w` × `h` picture.
fn head_px(points: &[StrokePoint], line_width: f64, w: f64, h: f64) -> [(f64, f64); 3] {
    let head = arrow_head(points, line_width, w / h).expect("a head");
    head.map(|(x, y)| (x * w, y * h))
}

/// `(tip, base midpoint)` of a head, in pixels.
fn axis(head: [(f64, f64); 3]) -> ((f64, f64), (f64, f64)) {
    let [tip, a, b] = head;
    (tip, ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0))
}

/// How long the head is, tip to base, in pixels.
fn length(head: [(f64, f64); 3]) -> f64 {
    let (tip, mid) = axis(head);
    (tip.0 - mid.0).hypot(tip.1 - mid.1)
}

/// How wide its base is, in pixels.
fn base_width(head: [(f64, f64); 3]) -> f64 {
    let [_, a, b] = head;
    (b.0 - a.0).hypot(b.1 - a.1)
}

/// Which way it points, in degrees, screen coordinates: 0 is to the right and
/// 90 straight *down*, since y grows downward.
fn bearing(head: [(f64, f64); 3]) -> f64 {
    let (tip, mid) = axis(head);
    (tip.1 - mid.1).atan2(tip.0 - mid.0).to_degrees()
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

// ------------------------------------------------------- the tuned triangle

/// The exact triangle the constants produce, on the picture and the pen the
/// app draws at by default — the one place the numbers are written out, so a
/// retune shows up here and not as six vague failures elsewhere.
///
/// A stroke straight to the right at mid-height: the head is 5 × 5.4 = 27 px
/// long and 4 × 5.4 = 21.6 px across its base, and its **tip is half a line
/// width past the last point** (2.7 px), which is exactly where the line's own
/// round cap already ended.
#[test]
fn a_horizontal_stroke_gets_the_head_the_constants_describe() {
    let points = in_pixels(&[(400.0, 540.0), (1000.0, 540.0)], W, H);
    let [tip, a, b] = head_px(&points, STROKE_LINE_WIDTH, W, H);
    let pen = STROKE_LINE_WIDTH * H; // 5.4 px
    assert!(close(tip.0, 1000.0 + pen / 2.0), "{tip:?}");
    assert!(close(tip.1, 540.0), "{tip:?}");
    assert!(close(a.0, tip.0 - ARROW_HEAD_LENGTH_RATIO * pen), "{a:?}");
    assert!(
        close(b.0, a.0),
        "the base is square to the line: {a:?} {b:?}"
    );
    assert!(
        close(a.1, 540.0 + ARROW_HEAD_HALF_WIDTH_RATIO * pen),
        "{a:?}"
    );
    assert!(
        close(b.1, 540.0 - ARROW_HEAD_HALF_WIDTH_RATIO * pen),
        "{b:?}"
    );
    let head = [tip, a, b];
    assert!(close(length(head), 27.0), "{}", length(head));
    assert!(close(base_width(head), 21.6), "{}", base_width(head));
}

/// A stroke straight **down** gets the same triangle, turned.
///
/// This is the aspect ratio's test. The barbs are spread along x here and
/// along y in the test above, and x is normalized to width while y is
/// normalized to height — so a head computed in normalized space without the
/// aspect would come out 1.78× too wide at 16:9 and the two bases would not
/// match. They match to the picometre.
#[test]
fn a_vertical_stroke_gets_the_same_triangle_turned() {
    let down = head_px(
        &in_pixels(&[(960.0, 200.0), (960.0, 800.0)], W, H),
        STROKE_LINE_WIDTH,
        W,
        H,
    );
    let right = head_px(
        &in_pixels(&[(400.0, 540.0), (1000.0, 540.0)], W, H),
        STROKE_LINE_WIDTH,
        W,
        H,
    );
    assert!(close(bearing(down), 90.0), "{}", bearing(down));
    assert!(close(length(down), length(right)), "{down:?}");
    assert!(close(base_width(down), base_width(right)), "{down:?}");
    let [tip, a, b] = down;
    assert!(close(tip.1, 800.0 + STROKE_LINE_WIDTH * H / 2.0), "{tip:?}");
    assert!(
        close(a.1, b.1),
        "the base is square to the line: {a:?} {b:?}"
    );
}

/// Whatever the picture's shape and whichever way the stroke points, the head
/// is the same isosceles triangle in pixels: the stated length, the stated
/// base, the base square to the axis and the two barbs the same distance from
/// the tip. Nothing in the frame's aspect may reach the shape.
#[test]
fn the_head_is_one_triangle_in_pixels_whichever_way_it_points() {
    for aspect in ASPECTS {
        let (w, h) = (H * aspect, H);
        let pen = STROKE_LINE_WIDTH * h;
        for step in 0..24 {
            let angle = f64::from(step) * std::f64::consts::TAU / 24.0;
            // 200 px of straight line out of the middle of the picture.
            let from = (w / 2.0, h / 2.0);
            let to = (from.0 + 200.0 * angle.cos(), from.1 + 200.0 * angle.sin());
            let head = head_px(&in_pixels(&[from, to], w, h), STROKE_LINE_WIDTH, w, h);
            let why = format!("aspect {aspect}, {} deg", angle.to_degrees());
            assert!(
                close(length(head), ARROW_HEAD_LENGTH_RATIO * pen),
                "{why}: {}",
                length(head)
            );
            assert!(
                close(base_width(head), 2.0 * ARROW_HEAD_HALF_WIDTH_RATIO * pen),
                "{why}: {}",
                base_width(head)
            );
            // It points along the line it caps ...
            let turned = (bearing(head) - angle.to_degrees() + 540.0).rem_euclid(360.0) - 180.0;
            assert!(turned.abs() < 1e-9, "{why}: {turned} deg off the line");
            // ... and the barbs are a mirror pair about that axis.
            let [tip, a, b] = head;
            let (ta, tb) = (
                (tip.0 - a.0).hypot(tip.1 - a.1),
                (tip.0 - b.0).hypot(tip.1 - b.1),
            );
            assert!(close(ta, tb), "{why}: {ta} vs {tb}");
            let (_, mid) = axis(head);
            let dot = (tip.0 - mid.0) * (b.0 - a.0) + (tip.1 - mid.1) * (b.1 - a.1);
            assert!(dot.abs() < 1e-6, "{why}: base not square, dot {dot}");
        }
    }
}

// ------------------------------------------------------------ the direction

/// **The test that matters.** A quarter-circle flick: it starts off to the
/// left and ends travelling straight to the right, so the chord from the first
/// point to the last points up and to the right at 45° — nowhere the line
/// goes. The head follows the tangent at the tip instead, which is what the
/// window back from the last point buys.
#[test]
fn the_head_follows_the_tangent_at_the_tip_not_the_chord_of_the_whole_stroke() {
    // Centre (960, 540), radius 300, swept from 180° to 270°: from (660, 540)
    // up to (960, 240), travelling to the right at the end.
    let arc: Vec<(f64, f64)> = (0..=90)
        .map(|deg| {
            let t = (180.0 + f64::from(deg)).to_radians();
            (960.0 + 300.0 * t.cos(), 540.0 + 300.0 * t.sin())
        })
        .collect();
    let chord = {
        let (first, last) = (arc[0], arc[arc.len() - 1]);
        (last.1 - first.1).atan2(last.0 - first.0).to_degrees()
    };
    assert!(
        close(chord, -45.0),
        "the chord really is 45 deg off: {chord}"
    );

    let head = head_px(&in_pixels(&arc, W, H), STROKE_LINE_WIDTH, W, H);
    // The window is 3 × 5.4 = 16.2 px of a 300 px radius, so the chord across
    // it lags the true tangent by about 1.5°. Anything under 3° is the
    // tangent; the whole-stroke chord is 45° away.
    assert!(bearing(head).abs() < 3.0, "{} deg", bearing(head));
    assert!(
        (bearing(head) - chord).abs() > 40.0,
        "{} deg",
        bearing(head)
    );
}

/// And the window is why the head does not wobble: the last two points of a
/// freehand stroke can be a single logical pixel apart
/// (`drawing::MIN_DISTANCE`), so one sideways pixel of pointer jitter at the
/// tip would swing a last-pair direction 45° off a line the coach drew
/// straight.
#[test]
fn a_pixel_of_jitter_at_the_tip_does_not_swing_the_head() {
    let mut drawn: Vec<(f64, f64)> = (0..60)
        .map(|i| (400.0 + f64::from(i) * 10.0, 540.0))
        .collect();
    let last = *drawn.last().expect("drawn");
    drawn.push((last.0 + 1.0, last.1 - 1.0)); // one pixel up and across
    let head = head_px(&in_pixels(&drawn, W, H), STROKE_LINE_WIDTH, W, H);
    assert!(bearing(head).abs() < 6.0, "{} deg", bearing(head));
}

/// A stroke that curls back onto its own tip still gets a head, and it points
/// along the line the coach drew.
///
/// Four pixels out and the same four back, at the end of 500 px to the right:
/// the far end of the window is the tip itself, so there is no direction
/// there. The anchor walks further back rather than giving up — and what it
/// finds is the line, not the wobble, which is the right answer for a retreat
/// a twentieth of the window long. `None` stays reserved for a stroke with no
/// direction anywhere.
#[test]
fn a_tip_the_stroke_curls_back_onto_still_gets_a_head() {
    let points = in_pixels(
        &[
            (400.0, 540.0),
            (900.0, 540.0),
            (904.0, 540.0),
            (900.0, 540.0),
        ],
        W,
        H,
    );
    let head = head_px(&points, STROKE_LINE_WIDTH, W, H);
    assert!(close(
        length(head),
        ARROW_HEAD_LENGTH_RATIO * STROKE_LINE_WIDTH * H
    ));
    assert!(close(bearing(head), 0.0), "{} deg", bearing(head));
}

// ---------------------------------------------------------------- the scale

/// The head is sized off the pen alone, so the thicker pen (BACKLOG #116)
/// gets a bigger head with nothing wired up: twice the width is twice the
/// head, in both directions, pointing the same way.
#[test]
fn the_head_scales_with_the_line_width() {
    let points = in_pixels(&[(400.0, 540.0), (1000.0, 540.0)], W, H);
    let normal = head_px(&points, STROKE_LINE_WIDTH, W, H);
    let thick = head_px(&points, STROKE_LINE_WIDTH_THICK, W, H);
    assert!(
        close(STROKE_LINE_WIDTH_THICK, 2.0 * STROKE_LINE_WIDTH),
        "the pens really are 2x apart"
    );
    assert!(close(length(thick), 2.0 * length(normal)), "{thick:?}");
    assert!(
        close(base_width(thick), 2.0 * base_width(normal)),
        "{thick:?}"
    );
    assert!(close(bearing(thick), bearing(normal)), "{thick:?}");
}

// -------------------------------------------------------------- no head yet

/// No direction, no head. A single point draws a dot (the renderers' own
/// rule) and must not grow an arrow out of nothing, and neither must a stroke
/// whose points all land on one spot — which a held pen produces.
#[test]
fn a_stroke_with_no_direction_has_no_head() {
    let aspect = W / H;
    assert_eq!(arrow_head(&[], STROKE_LINE_WIDTH, aspect), None, "empty");
    let one = stroke(&[(0.5, 0.5)]);
    assert_eq!(
        arrow_head(&one, STROKE_LINE_WIDTH, aspect),
        None,
        "one point"
    );
    let twice = stroke(&[(0.5, 0.5), (0.5, 0.5)]);
    assert_eq!(
        arrow_head(&twice, STROKE_LINE_WIDTH, aspect),
        None,
        "a coincident pair"
    );
    let held = stroke(&[(0.25, 0.75); 40]);
    assert_eq!(
        arrow_head(&held, STROKE_LINE_WIDTH, aspect),
        None,
        "a held pen"
    );
}

/// A pen width or an aspect that is not a positive number is a caller's bug,
/// and the answer is no head rather than a triangle of `NaN`s that a drawer
/// would carry into a path.
#[test]
fn a_nonsense_pen_or_aspect_has_no_head() {
    let points = stroke(&[(0.2, 0.5), (0.8, 0.5)]);
    for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert_eq!(arrow_head(&points, bad, W / H), None, "line width {bad}");
        assert_eq!(
            arrow_head(&points, STROKE_LINE_WIDTH, bad),
            None,
            "aspect {bad}"
        );
    }
}

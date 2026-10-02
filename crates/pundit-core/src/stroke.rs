//! Freehand telestration strokes.
//!
//! Stroke coordinates are normalized **top-left** origin: capture flips out of
//! the platform's view space, and the compositor does not flip again.
//! `line_width` is normalized to frame **height**, not width.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Straight RGBA, components in `0.0..=1.0`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rgba {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Rgba {
    pub const RED: Rgba = Rgba {
        r: 1.0,
        g: 0.2,
        b: 0.2,
        a: 1.0,
    };
}

/// One sampled point of a stroke.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StrokePoint {
    /// 0...1 of frame width.
    pub x: f64,
    /// 0...1 of frame height.
    pub y: f64,
    /// Seconds since this stroke started.
    pub t: f64,
}

/// How a stroke ends (v14, BACKLOG #117).
///
/// **An enum rather than an `arrow: bool`**, and the rule it satisfies is
/// `project.rs`'s: a field added to an existing struct takes a field-level
/// `#[serde(default)]` only when its `Default` is what an older file *means*.
/// `Plain` is what every v7–v13 stroke was, so it qualifies on the same grounds
/// `Inset` defaults to `Camera` — and an enum says that in the type where a
/// defaulted `bool` would be leaning on the letter of the rule against its
/// stated reason.
///
/// It also leaves room for an end that is neither (a circle, a double head)
/// without a second field to disagree with this one.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StrokeEnd {
    /// A plain line, which is every stroke drawn before v14.
    #[default]
    Plain,
    /// An arrowhead at the last point, pointing where the pen was going.
    /// Drawn from [`arrow_head`], never by either drawer's own arithmetic.
    Arrow,
}

/// A complete freehand stroke.
///
/// The `.stroke` event that carries one is appended when the stroke
/// **finishes**, so replay back-computes its start time. See
/// `stroke_replay::visible_strokes`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stroke {
    pub id: Uuid,
    pub color: Rgba,
    /// Normalized to frame height.
    pub line_width: f64,
    pub points: Vec<StrokePoint>,
    /// Seconds after **pen-up** — the event's `record_time`, not the stroke's
    /// first point — at which this stroke disappears. `None` = persist until a
    /// `ClearAll`.
    pub auto_clear_after_seconds: Option<f64>,
    /// v14. How the stroke ends: a plain line, or an arrowhead at its last
    /// point. Field-level default because [`StrokeEnd::Plain`] is exactly what
    /// a v7–v13 stroke was.
    #[serde(default)]
    pub end: StrokeEnd,
}

// --------------------------------------------------------------- arrow head

/// How long the head is, as a multiple of the stroke's `line_width`.
///
/// **The head is sized off the pen and nothing else**, so the thicker pen
/// (BACKLOG #116) gets a bigger head for free and no second preference can
/// disagree with the one the coach already picked. At the default pen
/// (`layout::STROKE_LINE_WIDTH`, 5.4 px at 1080p) that is a 27 px head; at the
/// thick one, 54 px. Retuning the head is this constant and
/// [`ARROW_HEAD_HALF_WIDTH_RATIO`] — there is nothing else to turn.
///
/// Five is the ratio at which a head reads as an arrow at a glance without
/// swallowing the short flick a coach actually draws.
pub const ARROW_HEAD_LENGTH_RATIO: f64 = 5.0;

/// Half the head's base, as a multiple of `line_width` — so the base is four
/// line widths across.
///
/// With [`ARROW_HEAD_LENGTH_RATIO`] that is a barb half-angle of
/// `atan(2 / 5)` ≈ 21.8°, 43.6° included: sharp enough to read as a direction
/// rather than a blob, wide enough to be seen over match video.
pub const ARROW_HEAD_HALF_WIDTH_RATIO: f64 = 2.0;

/// How far back from the tip the direction is measured, as a multiple of
/// `line_width`.
///
/// **The direction is the tangent at the tip, not first-point-to-last.** A
/// coach's flick curves, so a chord across the whole stroke would point
/// somewhere the line never goes. It is not the final *pair* of points either:
/// those are as little as one logical pixel apart (`drawing::MIN_DISTANCE`),
/// which on a jittery stroke is mostly sampling noise. So the window is a
/// distance walked back along the path, which makes it independent of how
/// densely the pointer was sampled.
///
/// Three line widths is about 60% of the head's own length: long enough to
/// average the jitter out, short enough that the head still follows a curve
/// into its last few pixels. It scales with the pen for the same reason the
/// head does — a bigger head needs a longer base to sit straight on.
pub const ARROW_TANGENT_WINDOW_RATIO: f64 = 3.0;

/// The three corners of the filled triangle that caps the **end** of a stroke:
/// `[tip, barb, barb]`, in the same normalized space as [`StrokePoint`]
/// (x of frame width, y of frame height, top-left origin). The barbs come in
/// travel order, the right-hand one first; it is a filled triangle, so neither
/// drawer cares which way it winds.
///
/// `None` when the stroke has no direction — fewer than two points, or every
/// point coincident with the last. A single-point stroke draws a dot and no
/// head.
///
/// **This is one function in core because the head is drawn twice**: live in
/// Slint from SVG path commands, and by tiny-skia in `pundit_media`'s overlay
/// for previews and exports. Two hand-written arrowheads would drift and the
/// coach would see one thing while recording and another in the export — the
/// same reason [`crate::highlight::highlight_shapes`] lives here. It takes a
/// slice rather than a [`Stroke`] so the pen-down buffer, which is not a
/// `Stroke` yet, can draw its head live from the same arithmetic.
///
/// **`aspect` is the picture's width ÷ height, and it is not optional.** A
/// stroke's x is normalized to width and its y to height, so that pair is not
/// a space lengths or angles can be computed in: on 16:9 a direction read
/// straight out of it is skewed by 1.78, which would tilt every barb and make
/// the head's length depend on which way it pointed. So the work happens in
/// the height-normalized square space the pen itself lives in — the one where
/// `line_width` is a length (`layout::stroke_line_width`: height on both axes,
/// never width) — by scaling x up by `aspect` on the way in and back down on
/// the way out. Each drawer then applies exactly the `(x * w, y * h)` mapping
/// it already applies to every stroke point, and there is no second
/// convention. It is the **content rect's** aspect, like that function's
/// `picture_h`, never the output frame's. A non-positive or non-finite
/// `aspect` or `line_width` returns `None` rather than a `NaN` triangle.
///
/// The head is not clamped to the picture: a stroke that ends on the edge gets
/// a head that runs over it, and both drawers clip, which is what the line's
/// own round cap already does.
pub fn arrow_head(points: &[StrokePoint], line_width: f64, aspect: f64) -> Option<[(f64, f64); 3]> {
    // One negation over the whole conjunction: clippy's `nonminimal_bool` on
    // CI's pinned 1.92 refuses the two-negation form, and a newer local clippy
    // does not flag it.
    if !(line_width.is_finite() && line_width > 0.0 && aspect.is_finite() && aspect > 0.0) {
        return None;
    }
    // Into the square space: x scaled up by the aspect, y left alone.
    let square = |p: &StrokePoint| (p.x * aspect, p.y);
    let (tip_x, tip_y) = square(points.last()?);

    // Walk back from the tip, accumulating path length, and keep the earliest
    // point reached that is not coincident with the tip. The two conditions
    // are separate on purpose: the window is what makes the direction stable,
    // and the coincidence test is what makes it exist at all — so a stroke
    // that curls back onto its own tip reads from a point further back rather
    // than returning `None`.
    let window = ARROW_TANGENT_WINDOW_RATIO * line_width;
    let mut travelled = 0.0;
    let mut anchor = None;
    for pair in points.windows(2).rev() {
        let (ax, ay) = square(&pair[0]);
        let (bx, by) = square(&pair[1]);
        travelled += (bx - ax).hypot(by - ay);
        if (tip_x - ax).hypot(tip_y - ay) > 0.0 {
            anchor = Some((ax, ay));
        }
        if travelled >= window && anchor.is_some() {
            break;
        }
    }
    // A stroke shorter than the window gives its whole self, which is the best
    // direction there is. Every point coincident gives nothing.
    let (ax, ay) = anchor?;

    let (dx, dy) = (tip_x - ax, tip_y - ay);
    let length = dx.hypot(dy);
    let (dx, dy) = (dx / length, dy / length);
    // The tip sits half a line width past the last point — exactly where the
    // line's round cap already ends — so the cap is swallowed by the triangle
    // instead of poking a blunt nub out past its point, and the arrow is no
    // longer than the line was without it.
    let tip = (tip_x + dx * line_width / 2.0, tip_y + dy * line_width / 2.0);
    let head = ARROW_HEAD_LENGTH_RATIO * line_width;
    let half = ARROW_HEAD_HALF_WIDTH_RATIO * line_width;
    let base = (tip.0 - dx * head, tip.1 - dy * head);
    let barb = |sign: f64| (base.0 - dy * half * sign, base.1 + dx * half * sign);
    // Back out of the square space.
    let stroke_space = |(x, y): (f64, f64)| (x / aspect, y);
    Some([
        stroke_space(tip),
        stroke_space(barb(1.0)),
        stroke_space(barb(-1.0)),
    ])
}

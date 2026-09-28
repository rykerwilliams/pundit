//! Where the composite's furniture sits, as ratios of the frame it lands on.
//!
//! Every value here is a ratio, never a pixel count, so preview (1280×720) and
//! export (1920×1080) lay out identically from one set of numbers — the parent
//! spec's "Layout constants the port must reproduce". Phase 7 needs the stroke
//! and PiP rows; the text bar (Phase 8) and the scoreboard (Phase 9) joined
//! them when something drew them.
//!
//! **Two spaces, and they differ only on a non-16:9 source** (BACKLOG #20,
//! settled here the way the parent spec recommended):
//!
//! - **Strokes live in the content rect** — the letterboxed picture, which is
//!   [`crate::zoom::Zoom::IDENTITY`]'s `transform`. They were drawn on the
//!   picture, so `line_width` denormalizes against *its* height and the
//!   overlay is rasterized at *its* size. Against the output rect they would
//!   stretch across the letterbox bars.
//! - **The PiP lives in output space**, overlapping those bars like broadcast
//!   furniture. It is chrome: the coach never drew it, so nothing ties it to
//!   the picture. So does the text bar (Phase 8).

use crate::project::{InsetCorner, InsetPlacement, InsetSize};

/// An axis-aligned rectangle in pixels, top-left origin.
///
/// Sub-pixel by design: a mixer pad rounds it once, and rounding earlier would
/// make the same layout land differently at two output sizes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

/// The webcam inset's width at each [`InsetSize`], as a fraction of the
/// **output** width.
///
/// **The table is here, not on the enum.** Every ratio in the app lives in this
/// module with a doc comment of its own, and the enum whose shape `InsetSize`
/// copies argues the same way about its meaning: [`Resolution`] is a bare
/// stored enum whose pixels live in its consumer too.
///
/// `Medium` is what the inset has always been, which is what makes every
/// project written before v13 render to the pixel (spec I8). The three were
/// each measured against the longest caption the app produces, which a free
/// ratio could not be: at 1080p the bar fits about 74 / 68 / 61 characters, so
/// `Large` costs a caption roughly seven of them (spec I1).
///
/// [`InsetSize`]: crate::project::InsetSize
/// [`Resolution`]: crate::project::Resolution
pub fn inset_ratio(size: InsetSize) -> f64 {
    match size {
        InsetSize::Small => 0.16,
        InsetSize::Medium => 0.22,
        InsetSize::Large => 0.30,
    }
}

/// The text bar's height, as a fraction of the **output height** (macOS's
/// `size.height * 0.08`).
pub const BAR_HEIGHT_RATIO: f64 = 0.08;

/// The glyphs' inset inside the bar, as a fraction of the **bar's height** —
/// the same inset on both axes, so the text doesn't kiss its edges.
pub const BAR_INSET_RATIO: f64 = 0.15;

/// The bar's font size, as a fraction of the **bar's height**. Small enough
/// that a line plus its ascender and descender fits the inset rect.
pub const BAR_FONT_RATIO: f64 = 0.5;

/// A column of width `ratio × out_w` flush into `corner`, as `(x, w)`.
///
/// [`inset_span`] is this at an [`InsetSize`]'s own ratio, and it is the only
/// caller that wants a span rather than a rect: the bar needs the inset's
/// column without its height.
///
/// [`InsetSize`]: crate::project::InsetSize
fn corner_span(out_w: f64, corner: InsetCorner, ratio: f64) -> (f64, f64) {
    let w = ratio * out_w;
    let x = match corner {
        InsetCorner::BottomLeft => 0.0,
        InsetCorner::BottomRight | InsetCorner::TopRight => out_w - w,
    };
    (x, w)
}

/// The rect a column of `ratio × out_w` makes in `corner` at display aspect
/// `aspect`: [`corner_span`] given a height.
///
/// The inset is flush into its corner in both axes, so `aspect` decides only
/// how far the rect reaches back from the edge it hangs off — the bottom one,
/// or the top for a top corner.
///
/// **It takes the ratio and the corner, not a span and the corner again.** Both
/// callers — [`pip_rect`] at an [`InsetSize`]'s own ratio, [`avatar_box`] at a
/// ratio *of* one, which is the whole of the difference between the two
/// footprints — want the span this makes and nothing else, so passing the
/// corner twice bought only the chance of passing two different ones.
///
/// [`InsetSize`]: crate::project::InsetSize
fn corner_rect(out_w: f64, out_h: f64, ratio: f64, aspect: f64, corner: InsetCorner) -> Rect {
    let (x, w) = corner_span(out_w, corner, ratio);
    let h = w / aspect;
    let y = match corner {
        InsetCorner::TopRight => 0.0,
        InsetCorner::BottomLeft | InsetCorner::BottomRight => out_h - h,
    };
    Rect { x, y, w, h }
}

/// The column the inset stands in, in output space — and so where the text bar
/// has to stop.
///
/// **One function, two readers** ([`pip_rect`] and [`bar_rect`]), so the bar
/// meeting the inset exactly is the rule rather than two arithmetic
/// coincidences that a retuned ratio, or a corner one of them learned about
/// first, could part.
///
/// **`(x, w)`, never `(left, right)`.** The width is always the size's ratio of
/// `out_w` and the corner decides only `x`; a reader deriving the width as
/// `right − left` would not get the inset the app has shipped —
/// `1920 − (1920 − 0.22·1920)` is `422.4000000000001` where `0.22·1920` is
/// `422.4` — so every existing project would move by a ULP, and the two `f64`s
/// would be interchangeable at the call site into the bargain.
pub fn inset_span(out_w: f64, placement: InsetPlacement) -> (f64, f64) {
    corner_span(out_w, placement.corner, inset_ratio(placement.size))
}

/// The text bar's rect in output space: a strip along the bottom row, ending
/// where the inset stands — or reaching both edges when nothing stands in that
/// row.
///
/// **The whole bar stops, background and line alike** (the coach's corner-lock,
/// 2026-09-25). The inset is flush into the same corner ([`pip_rect`]), so a
/// bar that ran under it would put its 60% black over the coach's own face; a
/// bar that ends at the inset instead reads as the bottom row being shared
/// between the two. The line has to stop for a second reason of its own: it is
/// ellipsized and never shrunk (`media::overlay`), so a reel's three-part
/// caption or a basket's `<match> | <clip> | tags` would otherwise run
/// underneath the inset, and losing the tags to an ellipsis is a smaller loss
/// than losing them behind a picture.
///
/// Stopping the bar is also what keeps the **overlay the mixer's top layer**
/// (`media::composite::install_overlay_pad`), which is what the coach's pen
/// needs: a stroke into that corner stays visible, because nothing in this
/// layer washes the inset.
///
/// A **bottom-left** inset moves the bar's whole edge instead of shortening it:
/// a bar still starting at 0 would run under the inset, which is the accident
/// "flush, not inset" removed. A **top-right** one leaves the bottom row to the
/// bar alone, which is the full-width state a clip with no inset has always
/// had.
///
/// The height is the same in every case, so a caller that wants only the bar's
/// height — the glyphs' size and their inset, which are fractions of it — may
/// ask with any `inset`.
///
/// **The camera's column, whichever inset the clip has:** an avatar's circle is
/// flush into the same corner and narrower ([`avatar_box`]), and its pulse only
/// shrinks it further, so this column clears either kind in every corner. A
/// round avatar cannot finish the row the way a camera's rectangle does whatever
/// width is picked, so it is not worth a second edge.
///
/// **`None` means no inset is drawn, not "no clip".** It is
/// [`Clip::inset_placement`](crate::project::Clip::inset_placement), which
/// folds in the coach's own `show_pip` — and it is the coach's stored intent
/// rather than whether a recording opened: preview learns its camera's shape
/// from caps on a GStreamer thread, long after the bar is laid out, so anything
/// finer would have preview and export cut the same caption differently. A
/// `Some` for every clip, with the switch read at each call site instead, is
/// the bug this signature exists to make unwritable.
pub fn bar_rect(out_w: f64, out_h: f64, inset: Option<InsetPlacement>) -> Rect {
    let h = BAR_HEIGHT_RATIO * out_h;
    let (x, w) = match inset {
        // Nothing in the bottom row: both edges, as every clip's bar was
        // before there was an inset to give way to.
        None => (0.0, out_w),
        Some(p) => {
            let (inset_x, inset_w) = inset_span(out_w, p);
            match p.corner {
                // The bar runs from the left edge up to the inset.
                InsetCorner::BottomRight => (0.0, inset_x),
                // Its edge moves rather than the strip widening: a bar still
                // starting at 0 would run under the inset.
                InsetCorner::BottomLeft => (inset_x + inset_w, out_w - (inset_x + inset_w)),
                // The inset is out of this row, so the bar has it to itself.
                InsetCorner::TopRight => (0.0, out_w),
            }
        }
    };
    Rect {
        x,
        y: out_h - h,
        w,
        h,
    }
}

/// The webcam inset's rect in output space: flush into the corner `placement`
/// names, over the text bar.
///
/// Width comes from the output and `placement`'s size; height comes from
/// `cam_aspect`, so the camera is never stretched or letterboxed inside the
/// inset. `cam_aspect` is the **display** aspect (width ÷ height with the pixel
/// aspect ratio applied) and must be positive: a camera reporting neither is not
/// a camera.
///
/// **Flush, not inset** (the coach, 2026-09-25), in every corner: the decision
/// was about the strip of dead picture a margin left, which is
/// corner-independent. In a bottom corner the inset finishes the row the bar
/// starts, and the bar stops where it begins ([`bar_rect`]); macOS split the
/// bar's background from its glyphs across two layers to keep its PiP out of
/// its own caption, and a bar that ends at the inset needs neither the split
/// nor a fourth pad.
pub fn pip_rect(out_w: f64, out_h: f64, cam_aspect: f64, placement: InsetPlacement) -> Rect {
    corner_rect(
        out_w,
        out_h,
        inset_ratio(placement.size),
        cam_aspect,
        placement.corner,
    )
}

/// How large the avatar's circle is against the webcam inset a camera take of
/// the same size would fill — the user asked for smaller, 2026-09-23; it is one
/// line to retune.
pub const AVATAR_BOX_RATIO: f64 = 0.75;

/// The box an avatar's circle is drawn in at its loudest: the clip's inset
/// column at [`AVATAR_BOX_RATIO`] of its width, square, flush into the same
/// corner — so the circle keeps the webcam inset's own margins off the frame's
/// two edges and is simply smaller. That is the whole of the difference between
/// an avatar's footprint and a camera's.
///
/// **A ratio of a ratio, not a rect shrunk about a corner**, which is what this
/// was while there was one corner to shrink about. Shrinking a corner-flush
/// square about the corner it is flush in *is* a corner-flush square at
/// `AVATAR_BOX_RATIO ×` the ratio — identical to the pixel at Medium in the
/// bottom-right — while the rect form only worked in the corner it was written
/// for: applied to a bottom-left inset it drifted 105.6 px off the left edge at
/// Medium/1080p, and to a top-right one it hung the same 105.6 px below the top
/// — the box is square, so the two are one number.
///
/// The aspect is 1.0 and not a parameter because the avatar's image is square
/// (avatar spec A5), and the pulse still grows the circle concentrically
/// *inside* this box ([`avatar_rect`](crate::avatar::avatar_rect)), so nothing
/// else moves. **The pulse is the only part of the avatar's geometry that is
/// not here:** this is [`pip_rect`] at another ratio and a fixed aspect, which
/// is layout, while the pulse is a level and lives with the rest of the
/// estimator ([`crate::avatar`]).
pub fn avatar_box(out_w: f64, out_h: f64, placement: InsetPlacement) -> Rect {
    corner_rect(
        out_w,
        out_h,
        avatar_ratio(placement.size),
        1.0,
        placement.corner,
    )
}

/// The fraction of the output width an avatar's square box takes at `size`.
fn avatar_ratio(size: InsetSize) -> f64 {
    AVATAR_BOX_RATIO * inset_ratio(size)
}

/// The side of the square box an avatar's circle is drawn in, in output pixels.
///
/// **The one number a pixmap needs**, and the reason it is not
/// [`avatar_box`]`(…).w`: the corner moves the box without resizing it, so the
/// pixels depend on `size` alone. Export keys its avatar textures on
/// `(image, InsetSize)` and shares each one across the corners its entries use
/// (`media::composite::export`), so at that point there is no corner to name —
/// and asking for a rect there meant inventing one, which said something untrue
/// about a value nothing read.
///
/// Equal to `avatar_box(out_w, _, placement).w` for every corner, by
/// construction: both are [`avatar_ratio`] of `out_w`.
pub fn avatar_box_side(out_w: f64, size: InsetSize) -> f64 {
    avatar_ratio(size) * out_w
}

/// The composite's output shape. Every export resolution and the clip
/// preview are 16:9, and the source is letterboxed or pillarboxed into it.
pub const OUTPUT_ASPECT: f64 = 16.0 / 9.0;

/// A rect `f` computes in output space, carried into the space `picture` is
/// given in.
///
/// For the live self-view while recording: the UI draws the game video's
/// picture (the content rect, at 1×), not the output frame, and the export
/// fits that picture into a 16:9 frame before placing the inset in it. So the
/// output frame handed to `f` is the one the picture fits exactly, centred on
/// it — the export's fit, inverted — and on a non-16:9 picture the inset
/// reaches past the picture into where the export's bars would be, as it does
/// there. That mirrors in every corner: over a 4:3 picture a bottom-left inset
/// reaches past the left edge exactly as a bottom-right one reaches past the
/// right.
///
/// Two callers, the camera's inset and the avatar's smaller box, so the fit and
/// its inversion are written once.
fn over_picture(picture: Rect, f: impl FnOnce(f64, f64) -> Rect) -> Rect {
    let (w, h) = if picture.w / picture.h >= OUTPUT_ASPECT {
        (picture.w, picture.w / OUTPUT_ASPECT)
    } else {
        (picture.h * OUTPUT_ASPECT, picture.h)
    };
    let rect = f(w, h);
    Rect {
        x: picture.x + (picture.w - w) / 2.0 + rect.x,
        y: picture.y + (picture.h - h) / 2.0 + rect.y,
        ..rect
    }
}

/// Where the webcam inset lands **relative to the picture**, in whatever
/// space `picture` is given in: [`pip_rect`] carried from output space into
/// the picture's (`over_picture`, whose doc holds the reasoning).
pub fn pip_rect_over_picture(picture: Rect, cam_aspect: f64, placement: InsetPlacement) -> Rect {
    over_picture(picture, |w, h| pip_rect(w, h, cam_aspect, placement))
}

/// The live self-view's rect over `picture`, for a **camera** take whose
/// inset has display aspect `cam_aspect` (avatar spec G2).
///
/// The corner over the player is where the export puts the inset, so it is
/// [`pip_rect_over_picture`] and nothing else: the camera's inset lands where
/// it has always landed, at exactly the rect the render gives it.
/// `level` is the inset's size on the pulse's `0..=1` scale; a camera take
/// passes `1.0`, which [`avatar_rect`](crate::avatar::avatar_rect) maps to
/// `pip_rect_over_picture` itself. An avatar take has its own rule —
/// [`avatar_self_view_rect`] — because its box is smaller.
///
/// `None` before the first layout, or with no picture to place on.
pub fn self_view_rect(
    picture: Rect,
    cam_aspect: f64,
    level: f64,
    placement: InsetPlacement,
) -> Option<Rect> {
    (picture.w > 0.0 && picture.h > 0.0 && cam_aspect > 0.0).then(|| {
        crate::avatar::avatar_rect(pip_rect_over_picture(picture, cam_aspect, placement), level)
    })
}

/// The live self-view's rect over `picture` for an **avatar** take at `level`
/// (avatar spec G2): where the render draws the avatar, over the player.
///
/// One placement rule per kind of take. The avatar's image is square (avatar
/// spec A5), so its box is the clip's inset column at the avatar's own ratio
/// ([`avatar_box`]), breathed by the live level
/// ([`avatar_rect`](crate::avatar::avatar_rect)) — the same two functions, in the
/// same order, as the render.
///
/// `None` before the first layout, or with no picture to place on.
pub fn avatar_self_view_rect(picture: Rect, level: f64, placement: InsetPlacement) -> Option<Rect> {
    (picture.w > 0.0 && picture.h > 0.0).then(|| {
        crate::avatar::avatar_rect(
            over_picture(picture, |w, h| avatar_box(w, h, placement)),
            level,
        )
    })
}

// --------------------------------------------------------------- scoreboard
//
// The scoreboard's own ratios, deliberately **not** shared with the text bar's:
// `SCOREBOARD_HEIGHT_RATIO` happens to equal `BAR_HEIGHT_RATIO` today, and
// tying the two together would make one of them impossible to change.
//
// Everything below the bar itself is a fraction of the **cell height**
// (`bar.h` less the accent strip), not of the bar — the parent spec's table
// says `bar.h` and is ~9% too large.

/// The bar's width, as a fraction of the **output width**.
const SCOREBOARD_WIDTH_RATIO: f64 = 0.36;

/// The bar's height, as a fraction of the **output height**.
const SCOREBOARD_HEIGHT_RATIO: f64 = 0.08;

/// The accent strip's height, as a fraction of the **bar's height**.
const SCOREBOARD_ACCENT_RATIO: f64 = 0.08;

/// The cell widths, as fractions of the **bar's width**. The clock takes
/// whatever is left, so the four tile the bar exactly.
///
/// **The clock is the second-widest column, not the narrowest.** It carries
/// the longest string on the board: `BREAK` (3.77 em measured through the
/// shaping stack, and every break of every format but soccer's first reads it)
/// and `104:59` (3.88 em, the default soccer format plus overtime). When the
/// names took 0.30 each the clock was left 0.20, and at that width even
/// `00:00` (3.18 em) overflowed its 3.16 em cell — and a label is centred, so
/// an overflow spills *both* ways, into the away team's colour on one side and
/// past the bar's right edge on the other. The names give the width up: they are
/// fitted to their cells ([`SCOREBOARD_MIN_FONT_RATIO`]), so they lose size
/// rather than meaning.
const SCOREBOARD_HOME_RATIO: f64 = 0.27;
const SCOREBOARD_SCORE_RATIO: f64 = 0.20;
const SCOREBOARD_AWAY_RATIO: f64 = 0.27;

/// The stoppage tail's gap from the clock cell, as a fraction of the **cell
/// height** (macOS used an absolute 2 pt, which changes meaning with
/// resolution).
const SCOREBOARD_TAIL_GAP_RATIO: f64 = 0.025;

/// The four labels' font size, as a fraction of the **cell height**
/// ([`ScoreboardRects::home`]`.h`).
pub const SCOREBOARD_FONT_RATIO: f64 = 0.55;

/// The stoppage tail's font size, as a fraction of the **cell height**. It is
/// the one label that is not bold.
pub const SCOREBOARD_TAIL_FONT_RATIO: f64 = 0.45;

/// How small a scoreboard label may be shrunk to make it fit its cell, as a
/// fraction of the **cell height** — a quarter of [`SCOREBOARD_FONT_RATIO`].
///
/// A label that doesn't fit is shrunk to fit and only cut with an ellipsis
/// once it reaches this floor, because a smaller whole name carries more than
/// a full-size stub: `Manchester United` in a cell this wide is ellipsized to
/// `Manche…` but fits whole at 0.38 of the full size. Measured through the
/// shaping stack, every real club name tried — up to `Borussia
/// Mönchengladbach` and `Wolverhampton Wanderers`, 24 characters — fits at
/// 0.265, so the floor sits just under it at 0.25. In pixels that is 10.9 at
/// 1080p and 7.3 at 720p, both clear of the 6 px floor macOS used; below it a
/// line is cut rather than smeared to nothing (a pasted paragraph would
/// otherwise shape at 1.4 px).
pub const SCOREBOARD_MIN_FONT_RATIO: f64 = SCOREBOARD_FONT_RATIO / 4.0;

/// A team name's padding inside its cell, as a fraction of the **cell height**
/// (macOS used an absolute 4 pt).
pub const SCOREBOARD_NAME_PAD_RATIO: f64 = 0.05;

/// Where every piece of the scoreboard sits, in output space.
///
/// The five labels are centred in `home`, `score`, `away`, `clock` and `tail`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScoreboardRects {
    /// The whole bar: the accent strip plus the row of cells.
    pub bar: Rect,
    /// The accent strip, **above** the cells and spanning the bar. It is drawn
    /// over the **home and away columns only**, in each team's secondary
    /// color, so the drawer intersects it with those two cells' `x` and `w`.
    pub accent: Rect,
    pub home: Rect,
    pub score: Rect,
    pub away: Rect,
    pub clock: Rect,
    /// The `+M:SS` stoppage tail, which hangs **outside** the bar past the
    /// clock cell and is drawn only while the clock is in stoppage.
    pub tail: Rect,
}

/// The scoreboard's rects in output space, flush into the **top-left corner**.
///
/// **Flush, not inset** (the coach, 2026-09-25). The bar used to sit
/// `0.015 × outH` off both edges — 16 px at 1080p — which beside a caption bar
/// that lies on the bottom edge read as an accident rather than a decision, so
/// the board is locked to its corner the way the bar is to its own. Nothing
/// under it moves: the accent strip and the cells are placed off `bar`, and the
/// stoppage tail keeps its own gap off the clock **cell**, which is a gap
/// between two drawn things rather than off a frame edge.
pub fn scoreboard_rects(out_w: f64, out_h: f64) -> ScoreboardRects {
    let bar = Rect {
        x: 0.0,
        y: 0.0,
        w: SCOREBOARD_WIDTH_RATIO * out_w,
        h: SCOREBOARD_HEIGHT_RATIO * out_h,
    };
    let accent = Rect {
        h: SCOREBOARD_ACCENT_RATIO * bar.h,
        ..bar
    };
    let cell = |x: f64, w: f64| Rect {
        x,
        y: bar.y + accent.h,
        w,
        h: bar.h - accent.h,
    };

    let home = cell(bar.x, SCOREBOARD_HOME_RATIO * bar.w);
    let score = cell(home.x + home.w, SCOREBOARD_SCORE_RATIO * bar.w);
    let away = cell(score.x + score.w, SCOREBOARD_AWAY_RATIO * bar.w);
    // The clock closes the bar rather than taking a fourth ratio, so the cells
    // tile it exactly instead of to within a rounding error.
    let clock = cell(away.x + away.w, bar.x + bar.w - (away.x + away.w));
    let tail = cell(
        clock.x + clock.w + SCOREBOARD_TAIL_GAP_RATIO * clock.h,
        clock.w,
    );

    ScoreboardRects {
        bar,
        accent,
        home,
        score,
        away,
        clock,
        tail,
    }
}

/// The pen's width, as a fraction of the picture's height — the **one** pen
/// width. Every drawer takes it from here: the app's live stroke layer, the
/// [`Stroke::line_width`](crate::stroke::Stroke::line_width) a new drawing is
/// logged with, and a highlight's ring, which is stroked at the same weight so
/// it reads like a drawn ellipse.
pub const STROKE_LINE_WIDTH: f64 = 0.005;

/// A stroke's line width in pixels, from [`crate::stroke::Stroke::line_width`].
///
/// `picture_h` is the **content rect's** height, not the output frame's (see
/// the module comment). Height on both, never width: a stroke keeps its weight
/// when the picture's aspect changes, and the two axes would otherwise give a
/// pen that is oval rather than round.
pub fn stroke_line_width(line_width: f64, picture_h: f64) -> f64 {
    line_width * picture_h
}

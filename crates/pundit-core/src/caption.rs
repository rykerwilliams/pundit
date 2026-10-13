//! The strip along the bottom of the picture: what it says, and for how long
//! (BACKLOG #115, #133; spec `2026-10-12-titles-and-caption-bar-design.md`).
//!
//! Two things write into that one strip — a clip's caption bar, switched per
//! clip by [`CaptionBar`], and a slate's title, burned over the footage its
//! range covers — and [`strip_at`] is the one rule that picks between them per
//! frame. Both renderers call it and neither decides anything. Three rules
//! shape it:
//!
//! - **The fade is inside the window** (spec §S3). The strip is gone *at* the
//!   end of its window and fades over the window's last [`STRIP_FADE`]:
//!   `FirstSeconds` is full strength to 2.5 s and gone at 3.0, and a title is
//!   gone at its slate's out mark. Fading after the window would make "the
//!   first 3 seconds" mean 3.5, and spill a title past the coach's out mark
//!   into whatever he marked next.
//! - **No fade in** (§S4). A caption arrives with the entry's cut, which is
//!   already a hard change; a title arrives on the frame the coach marked, and
//!   a fade in would spend that frame and the next fourteen at partial
//!   strength.
//! - **A title hands back to the caption with a hard swap of the words, not a
//!   cross-fade** (§S5). While a caption is showing, a title is drawn at full
//!   strength to its out mark and the caption's words replace it on the next
//!   frame — the strip itself never moves. A cross-fade would draw two lines
//!   at once; fading the title over a caption that then pops back would read
//!   as a flicker. **The title fades only when nothing follows it**: the
//!   caption `Off`, past its 3 s, or empty.
//!
//! Two time bases, each the right one (§S6): the caption reads the entry's
//! **record time**, because "the first few seconds" is about the viewer's
//! clock; a title reads the frame's **source time**, because it is bounded on
//! the footage — so it freezes with the footage through a commentary pause, as
//! a highlight does.

use serde::{Deserialize, Serialize};

use crate::plan::PlanEntry;
use crate::project::Slate;

/// What a clip's caption bar does in an export or a preview (BACKLOG #115).
/// v18.
///
/// **An enum, not a defaulted `bool`**: three states, and `Whole` *is* what an
/// older file means — `Inset`'s and `StrokeEnd`'s rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CaptionBar {
    /// For the whole entry: what every v7–v17 clip did.
    #[default]
    Whole,
    /// For [`CAPTION_SECONDS`], then gone.
    FirstSeconds,
    Off,
}

impl CaptionBar {
    /// The record time at which this caption is gone — the one place the mode
    /// becomes a number, which [`crate::plan::PlanEntry::caption_until`]
    /// carries. `INFINITY` and `0.0` fall out of [`strip_at`]'s one formula,
    /// so the per-frame code has no branch on the mode.
    pub fn until(self) -> f64 {
        match self {
            CaptionBar::Whole => f64::INFINITY,
            CaptionBar::FirstSeconds => CAPTION_SECONDS,
            CaptionBar::Off => 0.0,
        }
    }
}

/// How long a `FirstSeconds` caption shows: the coach's own figure (#115).
///
/// **The inspector's picker spells it a second time**, as its "First 3
/// seconds" label (`app.slint`); change one and change the other. Pushing the
/// number from Rust for one word would cost more than the drift it prevents
/// (plan §0.5).
pub const CAPTION_SECONDS: f64 = 3.0;

/// How long the strip takes to leave (the coach, 2026-10-12).
pub const STRIP_FADE: f64 = 0.5;

/// The strip on one frame: the words, and how strongly they are drawn.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Strip<'a> {
    pub text: &'a str,
    /// In (0, 1], multiplying the bar's 60% black and its glyphs alike.
    pub opacity: f64,
}

/// 1 until `STRIP_FADE` before `end`, then linear to 0 at it.
fn leaving(end: f64, t: f64) -> f64 {
    ((end - t) / STRIP_FADE).clamp(0.0, 1.0)
}

/// The strip for one output frame of `entry`, or `None` for no strip at all.
///
/// `record_time` is the entry's own ([`PlanEntry::record_time`]) and
/// `source_time` the displayed frame's ([`crate::export::FrameSpec`]).
/// `slates` are the project's; only a **titled, closed** range on the entry's
/// own video counts, and of two covering the frame the later-starting wins.
/// A title on a half-marked slate is not drawn — it has no window yet.
///
/// **It never answers `Some` at opacity 0.** The title arm's window ends at
/// `out`, where [`leaving`] first reaches 0, and the caption arm is guarded by
/// `caption > 0.0`, so a drawer needs no "opacity 0 draws nothing" branch.
pub fn strip_at<'a>(
    entry: &'a PlanEntry,
    record_time: f64,
    slates: &'a [Slate],
    source_time: f64,
) -> Option<Strip<'a>> {
    let caption = if entry.text.is_empty() {
        0.0
    } else {
        leaving(entry.caption_until, record_time)
    };
    // The titled, closed range covering this frame; the latest-starting wins
    // an overlap.
    let title = slates
        .iter()
        .filter(|s| s.source_index == entry.source_index)
        .filter_map(|s| Some((s, s.title.as_deref()?, s.out_seconds?)))
        .filter(|(s, _, out)| s.in_seconds <= source_time && source_time < *out)
        .max_by(|a, b| a.0.in_seconds.total_cmp(&b.0.in_seconds));
    match title {
        // A caption to come back to: the words swap and the strip stays.
        Some((_, text, _)) if caption > 0.0 => Some(Strip { text, opacity: 1.0 }),
        // Nothing to come back to: the strip leaves with the title.
        Some((_, text, out)) => Some(Strip {
            text,
            opacity: leaving(out, source_time),
        }),
        None => (caption > 0.0).then_some(Strip {
            text: &entry.text,
            opacity: caption,
        }),
    }
}

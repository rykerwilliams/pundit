//! The strip along the bottom of the picture (`core::caption`, spec §S): what
//! it says on each frame, and how strongly.
//!
//! Entries are built through `CaptionBar::until()`, so these read the mode
//! rather than its number. **The title tests run at record time ≥ 5 s over an
//! `Off` caption** — except the hand-back, which needs a caption to hand back
//! to — so they stand apart from the mapping the first three pin.

use uuid::Uuid;

use pundit_core::caption::{strip_at, CaptionBar, Strip};
use pundit_core::plan::PlanEntry;
use pundit_core::project::Slate;
use pundit_core::timeline::{PlaybackSegment, SegmentKind};

const CAPTION: &str = "1 / 3 | corner";

/// An entry on video 0 captioned [`CAPTION`] under `mode`.
fn entry(mode: CaptionBar) -> PlanEntry {
    PlanEntry {
        clip_id: None,
        source_index: 0,
        segments: vec![PlaybackSegment {
            kind: SegmentKind::Play,
            source_start: 0.0,
            out_duration: 600.0,
        }],
        start_frame: 0,
        frames: 18_000,
        text: CAPTION.into(),
        caption_until: mode.until(),
    }
}

/// A closed slate on `source_index` over `[in, out)`, titled `title`.
fn titled(source_index: usize, range: (f64, f64), title: &str) -> Slate {
    Slate {
        id: Uuid::new_v4(),
        source_index,
        in_seconds: range.0,
        out_seconds: Some(range.1),
        name: String::new(),
        tags: Vec::new(),
        title: Some(title.into()),
    }
}

fn at(text: &str, opacity: f64) -> Option<Strip<'_>> {
    Some(Strip { text, opacity })
}

/// The record time the title tests run at: past any `FirstSeconds` window.
const LATE: f64 = 5.0;

#[test]
fn whole_shows_the_caption_for_the_whole_entry() {
    let e = entry(CaptionBar::Whole);
    for t in [0.0, 2.9, 600.0] {
        assert_eq!(strip_at(&e, t, &[], 0.0), at(CAPTION, 1.0), "at {t}");
    }
}

#[test]
fn first_seconds_fades_over_its_last_half_second_and_is_gone_at_three() {
    let e = entry(CaptionBar::FirstSeconds);
    assert_eq!(strip_at(&e, 2.4, &[], 0.0), at(CAPTION, 1.0));
    assert_eq!(strip_at(&e, 2.75, &[], 0.0), at(CAPTION, 0.5));
    assert_eq!(strip_at(&e, 3.0, &[], 0.0), None);
}

#[test]
fn off_shows_no_caption_from_the_first_frame() {
    let e = entry(CaptionBar::Off);
    assert_eq!(strip_at(&e, 0.0, &[], 0.0), None);
    assert_eq!(strip_at(&e, 1.0, &[], 0.0), None);
}

#[test]
fn an_empty_caption_draws_no_strip_whatever_the_mode() {
    let e = PlanEntry {
        text: String::new(),
        ..entry(CaptionBar::Whole)
    };
    assert_eq!(strip_at(&e, 0.0, &[], 0.0), None);
}

/// §S5: while a caption is showing, the title is at full strength to its out
/// mark and the caption is back, at full strength, on the frame at it.
#[test]
fn a_title_hands_back_to_the_caption_with_no_dip() {
    let e = entry(CaptionBar::Whole);
    let slates = [titled(0, (10.0, 12.0), "Pressing")];
    assert_eq!(strip_at(&e, LATE, &slates, 11.75), at("Pressing", 1.0));
    assert_eq!(strip_at(&e, LATE, &slates, 12.0), at(CAPTION, 1.0));
}

#[test]
fn a_title_with_nothing_to_hand_back_to_leaves_by_its_out_mark() {
    let e = entry(CaptionBar::Off);
    let slates = [titled(0, (10.0, 12.0), "Pressing")];
    assert_eq!(strip_at(&e, LATE, &slates, 11.75), at("Pressing", 0.5));
    assert_eq!(strip_at(&e, LATE, &slates, 12.0), None);
}

#[test]
fn a_title_on_another_video_is_ignored() {
    let e = entry(CaptionBar::Off);
    let slates = [titled(1, (10.0, 12.0), "Pressing")];
    assert_eq!(strip_at(&e, LATE, &slates, 11.0), None);
}

#[test]
fn an_untitled_or_half_marked_slate_draws_nothing() {
    let e = entry(CaptionBar::Off);
    let untitled = Slate {
        title: None,
        ..titled(0, (10.0, 12.0), "Pressing")
    };
    assert_eq!(strip_at(&e, LATE, &[untitled], 11.0), None);
    let half_marked = Slate {
        out_seconds: None,
        ..titled(0, (10.0, 12.0), "Pressing")
    };
    assert_eq!(strip_at(&e, LATE, &[half_marked], 11.0), None);
}

#[test]
fn the_later_starting_of_two_titles_wins() {
    let e = entry(CaptionBar::Off);
    // The later one listed first, so stored order cannot be what decides.
    let slates = [
        titled(0, (10.0, 20.0), "Second"),
        titled(0, (5.0, 20.0), "First"),
    ];
    assert_eq!(strip_at(&e, LATE, &slates, 11.0), at("Second", 1.0));
    assert_eq!(strip_at(&e, LATE, &slates, 7.0), at("First", 1.0));
}

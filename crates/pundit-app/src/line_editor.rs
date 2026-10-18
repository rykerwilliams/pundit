//! The line editor sheet's text, whichever list it is editing — kept out of
//! the UI code so it is tested headless.
//!
//! The sheet reads a pasted block back to the coach before Add is pressed
//! (match event editor spec B3). The block is a [`Batch`] of any grammar's
//! records; what a record *reads as*, and what the sheet calls one, are the
//! caller's.

use pundit_core::line_batch::{Batch, BatchVerdict};

/// One echoed line of the paste box (spec B3).
pub struct PasteLineText {
    /// `✓` it will add, `•` it is already there, `✗` it is refused.
    pub glyph: &'static str,
    pub text: String,
}

/// Everything the paste box says about the block it holds: a line each, the
/// summary above the button, and the button's own label.
pub struct PasteEcho {
    pub lines: Vec<PasteLineText>,
    pub summary: String,
    pub button: String,
}

/// Reads a parsed block back to the coach, line by line, before Add is pressed
/// (spec B3). `noun` is what one record is called (`"event"`), `duplicate`
/// what a skipped one is (`"already tagged"`), and `reads_as` how a record
/// that will add, or is skipped, reads.
///
/// Blank and comment-only lines say nothing, so they are not in `batch.lines`
/// and get no row here. A refusal quotes its line back with its number, which
/// is how the coach finds it in a block of twenty.
pub fn paste_echo<T>(
    batch: &Batch<T>,
    noun: &str,
    duplicate: &str,
    reads_as: impl Fn(&T) -> String,
) -> PasteEcho {
    let (mut adding, mut skipped, mut refused) = (0, 0, 0);
    let lines = batch
        .lines
        .iter()
        .map(|line| match &line.verdict {
            BatchVerdict::Added(item) => {
                adding += 1;
                PasteLineText {
                    glyph: "✓",
                    text: reads_as(item),
                }
            }
            BatchVerdict::Duplicate(item) => {
                skipped += 1;
                PasteLineText {
                    glyph: "•",
                    text: format!("{} — {duplicate}, skipped", reads_as(item)),
                }
            }
            BatchVerdict::Refused(reason) => {
                refused += 1;
                PasteLineText {
                    glyph: "✗",
                    text: format!("line {}: \"{}\" — {reason}", line.number, line.text),
                }
            }
        })
        .collect();

    let mut parts: Vec<String> = Vec::new();
    if adding > 0 {
        parts.push(format!("{adding} {} to add", plural(adding, noun)));
    }
    if skipped > 0 {
        parts.push(format!("{skipped} {duplicate}"));
    }
    if refused > 0 {
        parts.push(format!("{refused} {} refused", plural(refused, "line")));
    }
    PasteEcho {
        lines,
        summary: parts.join(" · "),
        // Nothing to add is a disabled button, which says nothing but its name.
        button: match adding {
            0 => "Add".to_string(),
            n => format!("Add {n} {}", plural(n, noun)),
        },
    }
}

fn plural(count: usize, noun: &str) -> String {
    match count {
        1 => noun.to_string(),
        _ => format!("{noun}s"),
    }
}

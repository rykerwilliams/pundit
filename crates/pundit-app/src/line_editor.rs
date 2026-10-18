//! The line editor sheet's text, whichever list it is editing — kept out of
//! the UI code so it is tested headless.
//!
//! The sheet reads a pasted block back to the coach before Add is pressed
//! (match event editor spec B3). The block is a [`Batch`] of any grammar's
//! records; what a record *reads as*, and what the sheet calls one, are the
//! caller's.

use pundit_core::line_batch::{Batch, BatchVerdict};
use pundit_core::match_entry::format_time;
use pundit_core::project::Project;
use pundit_core::slate_entry::{self, PendingSlate};
use uuid::Uuid;

use crate::match_panel;

/// What one kind's records are called, in the echo and in the bus's notice
/// alike, so the two cannot drift apart.
pub struct KindWords {
    /// One record: `"event"`. The plural is this plus `s`.
    pub noun: &'static str,
    /// A line skipped because its record is there already.
    pub duplicate: &'static str,
}

pub const EVENT_WORDS: KindWords = KindWords {
    noun: "event",
    duplicate: "already tagged",
};

pub const SLATE_WORDS: KindWords = KindWords {
    noun: "slate",
    duplicate: "already marked",
};

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
    /// How many lines will add: what Add is enabled on.
    pub adding: usize,
}

/// Reads a parsed block back to the coach, line by line, before Add is pressed
/// (spec B3), in `words`; `reads_as` is how a record that will add, or is
/// skipped, reads.
///
/// Blank and comment-only lines say nothing, so they are not in `batch.lines`
/// and get no row here. A refusal quotes its line back with its number, which
/// is how the coach finds it in a block of twenty.
pub fn paste_echo<T>(
    batch: &Batch<T>,
    words: &KindWords,
    reads_as: impl Fn(&T) -> String,
) -> PasteEcho {
    let KindWords { noun, duplicate } = words;
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
        adding,
    }
}

fn plural(count: usize, noun: &str) -> String {
    match count {
        1 => noun.to_string(),
        _ => format!("{noun}s"),
    }
}

/// One row of the line editor's list, whichever kind: what the sheet shows of
/// a record, and the line its field is seeded with when the row is selected.
#[derive(Debug, Clone, PartialEq)]
pub struct EditorRow {
    pub id: Uuid,
    /// "2 · 14:05.0", or "2 · 14:05.0–14:40.0" for a slate ("2 · 14:05.0–"
    /// open): the grammar's own numbering.
    pub where_text: String,
    /// What `interpret` calls an event; a slate's name, or "Untitled".
    pub label: String,
    /// Greyed: a start/stop the format has no period for, or an open range —
    /// stored, and not yet doing anything.
    pub role_less: bool,
    /// The record as a line of its grammar: what the row's field is seeded
    /// with, and what its edit is read against.
    pub line: String,
}

/// The match events as the line editor lists them: the Match panel's rows,
/// from the same builder, said the editor's way (spec T1).
pub fn event_rows(project: &Project) -> Vec<EditorRow> {
    match_panel::match_rows(project)
        .iter()
        .map(|row| EditorRow {
            id: row.id,
            where_text: match_panel::editor_row_where(row),
            label: row.label.clone(),
            role_less: row.role_less,
            line: match_panel::editor_row_line(row),
        })
        .collect()
}

/// The slates as the line editor lists them, in [`Project::slates_sorted`]'s
/// reading order.
pub fn slate_rows(project: &Project) -> Vec<EditorRow> {
    project
        .slates_sorted()
        .into_iter()
        .map(|slate| EditorRow {
            id: slate.id,
            where_text: slate_where(slate.source_index, slate.in_seconds, slate.out_seconds),
            label: label(&slate.name).to_owned(),
            role_less: slate.out_seconds.is_none(),
            line: slate_entry::format_line(slate),
        })
        .collect()
}

/// A slate that will add, or one already marked, as the paste echo reads it
/// back: "2 · 14:05.0–14:40.0 · corner routine · corners, set piece".
pub fn slate_reads_as(slate: &PendingSlate) -> String {
    let mut text = format!(
        "{} · {}",
        slate_where(slate.source_index, slate.in_seconds, slate.out_seconds),
        label(&slate.name)
    );
    if !slate.tags.is_empty() {
        text.push_str(&format!(" · {}", slate.tags.join(", ")));
    }
    text
}

fn slate_where(source_index: usize, in_seconds: f64, out_seconds: Option<f64>) -> String {
    format!(
        "{} · {}–{}",
        source_index + 1,
        format_time(in_seconds),
        out_seconds.map(format_time).unwrap_or_default()
    )
}

fn label(name: &str) -> &str {
    if name.is_empty() {
        "Untitled"
    } else {
        name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pundit_core::project::SourceRef;
    use pundit_core::slate_entry::parse_batch;

    fn project() -> Project {
        let mut p = Project::new("p");
        for i in 0..2 {
            p.source_videos.push(SourceRef {
                relative_path: format!("{i}.mp4"),
                display_name: format!("half {}", i + 1),
                duration_seconds: 3000.0,
                display_aspect: 16.0 / 9.0,
            });
        }
        p
    }

    /// The paste box's feedback for the slates, one line of each verdict: the
    /// glyphs, the sentences, the summary and the button, in the slates'
    /// words.
    #[test]
    fn the_slate_echo_reads_back_every_verdict() {
        let mut p = project();
        p.mark_slate_in(0, 200.0);
        let batch = parse_batch(
            &p,
            0,
            "2 14:05-14:40 corner routine [corners, set piece]\n\
             # notes\n\
             1 3:20-\n\
             2 14:05 home goal\n",
        );
        let echo = paste_echo(&batch, &SLATE_WORDS, slate_reads_as);

        assert_eq!(
            echo.lines.iter().map(|l| l.glyph).collect::<Vec<_>>(),
            ["✓", "•", "✗"]
        );
        assert_eq!(
            echo.lines[0].text,
            "2 · 14:05.0–14:40.0 · corner routine · corners, set piece"
        );
        assert_eq!(
            echo.lines[1].text,
            "1 · 3:20.0– · Untitled — already marked, skipped"
        );
        assert_eq!(
            echo.lines[2].text,
            "line 4: \"2 14:05 home goal\" — a slate is a range: 14:05-14:40, or 14:05- to \
             leave its end open"
        );
        assert_eq!(
            echo.summary,
            "1 slate to add · 1 already marked · 1 line refused"
        );
        assert_eq!(echo.button, "Add 1 slate");
    }

    #[test]
    fn an_open_range_is_a_role_less_row() {
        let mut p = project();
        let open = p.mark_slate_in(1, 30.0);
        let closed = p.mark_slate_in(0, 845.0);
        p.mark_slate_out(0, 880.0).unwrap();

        let rows = slate_rows(&p);
        assert_eq!(
            rows,
            [
                EditorRow {
                    id: closed,
                    where_text: "1 · 14:05.0–14:40.0".into(),
                    label: "Untitled".into(),
                    role_less: false,
                    line: "1 14:05.0-14:40.0".into(),
                },
                EditorRow {
                    id: open,
                    where_text: "2 · 0:30.0–".into(),
                    label: "Untitled".into(),
                    role_less: true,
                    line: "2 0:30.0-".into(),
                },
            ],
            "in reading order, the open range greyed"
        );
    }
}

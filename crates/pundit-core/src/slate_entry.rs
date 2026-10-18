//! One slate per line, typed or pasted — the grammar the line editor's slates
//! kind reads its row field and its paste box with (BACKLOG #97; spec
//! `2026-10-17-typed-slates-design.md`).
//!
//! A line is `[<video>] <in>-<out> <name> [<tags>]`:
//!
//! ```text
//! 2 14:05-14:40 corner routine [corners, set piece]
//! 3:20- free kick              # the default video; an open range
//! # a comment, ignored
//! ```
//!
//! **It shares three things with [`crate::match_entry`] and nothing else of its
//! grammar**: [`parse_time`], [`format_time`] and the rule that a leading bare
//! integer is a video number, with its refusal. `match_entry`'s `#` runs to the
//! end of the line and its words are a closed vocabulary — both would eat a
//! slate's name — so here **`#` is a comment only as a line's first character**
//! and the name is free text. Tags are a bracketed group that ends the line,
//! read by [`normalize_tags`], the tag field's own rule.
//!
//! **A lone time is refused**, not read as an open range: it is the line most
//! likely to arrive by mistake — every match event line has exactly one — and
//! an open range is one keystroke away (`14:05-`).

use crate::line_batch::{self, Batch, BatchVerdict};
use crate::match_entry::{format_time, is_video_number, parse_time, refuse_video};
use crate::project::{Project, Slate};
use crate::tag::normalize_tags;

/// A slate the coach typed, ready for the bus to add or retype. No id and no
/// title: the project mints the one, and a line never carries the other.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingSlate {
    pub source_index: usize,
    pub in_seconds: f64,
    /// `None` is an open range (`14:05-`).
    pub out_seconds: Option<f64>,
    /// Trimmed — `Project::edit_slate`'s rule for a name.
    pub name: String,
    pub tags: Vec<String>,
}

impl From<&Slate> for PendingSlate {
    /// A stored slate's five typed fields: what a line of it means.
    fn from(slate: &Slate) -> Self {
        Self {
            source_index: slate.source_index,
            in_seconds: slate.in_seconds,
            out_seconds: slate.out_seconds,
            name: slate.name.clone(),
            tags: slate.tags.clone(),
        }
    }
}

const NOT_A_RANGE: &str = "a slate is a range: 14:05-14:40, or 14:05- to leave its end open";

// ------------------------------------------------------------------ lines

/// One typed line, against `project`, for a line with no video number of its
/// own to belong to `default_source` (0-based). `None` for a blank line or a
/// comment, which say nothing.
pub fn parse_line(
    project: &Project,
    default_source: usize,
    line: &str,
) -> Option<Result<PendingSlate, String>> {
    parse_line_with(project, default_source, line, None)
}

/// The canonical rendering of a slate as a line: what the row's field is
/// seeded with, and what [`parse_line`] reads back into the same slate.
///
/// The video number is always written, so the line means the same whatever
/// the paste box's picker says; the dash is a bare `-`, and an open range's
/// name follows it after a space, so `5:00- 5:30 drill` reads back as an open
/// range named *5:30 drill*. **An empty `[]` is written when there are no tags
/// and the name holds a `[`**, because the tag group starts at the line's last
/// `[` — without it `a [b` would read as tags that were never closed.
pub fn format_line(slate: &Slate) -> String {
    let mut line = format!(
        "{} {}-",
        slate.source_index + 1,
        format_time(slate.in_seconds)
    );
    if let Some(out) = slate.out_seconds {
        line.push_str(&format_time(out));
    }
    if !slate.name.is_empty() {
        line.push(' ');
        line.push_str(&slate.name);
    }
    if !slate.tags.is_empty() {
        line.push_str(&format!(" [{}]", slate.tags.join(", ")));
    } else if slate.name.contains('[') {
        line.push_str(" []");
    }
    line
}

/// A row's field committed: `typed`, against `slate`, whose [`format_line`]
/// the field was seeded with.
///
/// **A line byte-identical to the seed is the slate as it stands**, which is
/// what keeps a row nobody edited from being rewritten when focus leaves it
/// even where its line does not round-trip (a tag holding a `[`). Otherwise
/// **an end typed as the time it displays keeps its stored seconds** — a
/// captured `845.06` is not re-rounded to the `14:05.0` the line shows because
/// the coach renamed the slate. "As it displays" is [`format_time`] of what was
/// typed against [`format_time`] of what is stored, so `14:05`, `14:05.0` and
/// either dash spacing all leave it alone, and a name is never in question.
pub fn edit_from_line(
    project: &Project,
    slate: &Slate,
    typed: &str,
) -> Result<PendingSlate, String> {
    let seed = format_line(slate);
    if typed == seed {
        return Ok(PendingSlate::from(slate));
    }
    parse_line_with(project, slate.source_index, typed, Some(slate))
        .unwrap_or_else(|| Err(format!("nothing on that line — a row reads \"{seed}\"")))
}

/// `line` split into its leading video number, if it has one, and the rest.
fn after_video(line: &str) -> (Option<&str>, &str) {
    let first = line.split_whitespace().next().unwrap_or("");
    if is_video_number(first) {
        (Some(first), line[first.len()..].trim_start())
    } else {
        (None, line)
    }
}

/// What the range scanner found: the two times **as typed**, and the text
/// after the range.
struct Range<'a> {
    in_text: &'a str,
    out_text: Option<&'a str>,
    rest: &'a str,
}

fn is_dash(c: char) -> bool {
    // A word processor turns a typed `-` between spaces into an en or em
    // dash; notes typed there must paste as they read.
    matches!(c, '-' | '–' | '—')
}

/// The run of characters a time is made of, at the start of `text`.
fn time_run(text: &str) -> &str {
    let end = text
        .find(|c: char| !(c.is_ascii_digit() || c == ':' || c == '.'))
        .unwrap_or(text.len());
    &text[..end]
}

/// The range at the start of `text` (spec S2). **The dash's spacing is what
/// tells a closed range from an open one whose name starts with a time**: no
/// space before it, and a time straight after it is `<in>-<out>` while a space
/// or the end is open; a space on both sides is closed; a space before it and
/// none after is refused.
fn scan_range(text: &str) -> Result<Range<'_>, &'static str> {
    let in_text = time_run(text);
    if in_text.is_empty() {
        return Err(NOT_A_RANGE);
    }
    let after_in = &text[in_text.len()..];
    let open = |rest| Range {
        in_text,
        out_text: None,
        rest,
    };
    let mut chars = after_in.chars();
    match chars.next() {
        // `14:05-…`: closed on a time, open on a space or the end.
        Some(c) if is_dash(c) => {
            let after_dash = chars.as_str();
            match after_dash.chars().next() {
                None => Ok(open(after_dash)),
                Some(c) if c.is_whitespace() => Ok(open(after_dash)),
                Some(_) => closed(in_text, after_dash),
            }
        }
        // `14:05 …`: a spaced dash, or a lone time.
        Some(c) if c.is_whitespace() => {
            let spaced = after_in.trim_start();
            let mut chars = spaced.chars();
            match chars.next() {
                Some(c) if is_dash(c) => {
                    let after_dash = chars.as_str();
                    match after_dash.chars().next() {
                        Some(c) if c.is_whitespace() => closed(in_text, after_dash.trim_start()),
                        Some(_) => Err("put a space on both sides of the dash or on neither"),
                        None => Err(NOT_A_RANGE),
                    }
                }
                _ => Err(NOT_A_RANGE),
            }
        }
        _ => Err(NOT_A_RANGE),
    }
}

/// A closed range's out time, at the start of `after_dash`.
fn closed<'a>(in_text: &'a str, after_dash: &'a str) -> Result<Range<'a>, &'static str> {
    let out_text = time_run(after_dash);
    if out_text.is_empty() {
        return Err(NOT_A_RANGE);
    }
    Ok(Range {
        in_text,
        out_text: Some(out_text),
        rest: &after_dash[out_text.len()..],
    })
}

/// The text after the range split into its name and its tags (spec S4): the
/// group starts at the line's **last** `[` and must end the line, which is
/// what lets a name hold brackets when tags follow it.
fn name_and_tags(rest: &str) -> Result<(String, Vec<String>), &'static str> {
    let rest = rest.trim();
    let Some(open) = rest.rfind('[') else {
        return Ok((rest.to_owned(), Vec::new()));
    };
    let group = &rest[open + 1..];
    if !group.contains(']') {
        return Err(
            "the tags aren't closed — end them with ] (a name with a [ in it needs [] after it)",
        );
    }
    let Some(inside) = group.strip_suffix(']') else {
        return Err("nothing may follow the tags (a name with a [ in it needs [] after it)");
    };
    Ok((rest[..open].trim().to_owned(), normalize_tags(inside)))
}

/// [`parse_line`], retyping `stored` when there is one (see
/// [`edit_from_line`]): each end that displays as the stored one keeps the
/// stored seconds, and **the order and the bound are checked on the seconds
/// that will be stored**, after that, so nothing can carry a stored time past
/// the end of a video or behind its own in point.
fn parse_line_with(
    project: &Project,
    default_source: usize,
    line: &str,
    stored: Option<&Slate>,
) -> Option<Result<PendingSlate, String>> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    Some(parse_body(project, default_source, line, stored))
}

/// `typed`, unless it displays as `stored`: then `stored`, to the last decimal.
fn kept(typed: f64, stored: Option<f64>) -> f64 {
    match stored {
        Some(stored) if format_time(typed) == format_time(stored) => stored,
        _ => typed,
    }
}

fn parse_body(
    project: &Project,
    default_source: usize,
    line: &str,
    stored: Option<&Slate>,
) -> Result<PendingSlate, String> {
    let (video, rest) = after_video(line);
    let source_index = match video {
        // Out of range it is refused as the video it is, rather than being
        // reinterpreted as anything else.
        Some(n) => match n.parse::<usize>() {
            Ok(n) if (1..=project.source_videos.len()).contains(&n) => n - 1,
            _ => return Err(refuse_video(n)),
        },
        None => default_source,
    };
    let Some(source) = project.source_videos.get(source_index) else {
        return Err(refuse_video(&(source_index + 1).to_string()));
    };

    let range = scan_range(rest)?;
    let in_seconds = parse_time(range.in_text).ok_or(NOT_A_RANGE)?;
    let out_seconds = match range.out_text {
        Some(text) => Some(parse_time(text).ok_or(NOT_A_RANGE)?),
        None => None,
    };
    let (name, tags) = name_and_tags(range.rest)?;

    let in_seconds = kept(in_seconds, stored.map(|s| s.in_seconds));
    let out_seconds = out_seconds.map(|out| kept(out, stored.and_then(|s| s.out_seconds)));

    // `mark_slate_out`'s `<=`: a zero-length range is not a range.
    if out_seconds.is_some_and(|out| out <= in_seconds) {
        return Err("the range ends before it starts".into());
    }
    // `match_entry`'s bound and wording: refused naming the length, never
    // clamped.
    if in_seconds.max(out_seconds.unwrap_or(0.0)) > source.duration_seconds {
        return Err(format!(
            "{} is {} long",
            source.display_name,
            format_time(source.duration_seconds)
        ));
    }
    Ok(PendingSlate {
        source_index,
        in_seconds,
        out_seconds,
        name,
        tags,
    })
}

// ----------------------------------------------------------------- batches

/// A whole pasted block, against `project`, read by
/// [`line_batch::read_batch`].
///
/// **A duplicate is the same text, not a tolerance**: two slates are the same
/// when their video, their two times as [`format_time`] writes them and their
/// name are equal — against the stored slates and against the lines this
/// block has already accepted. The name is in the key because two slates on
/// one range are legal (the same corner once for the attack, once for the
/// defence), and there is no tolerance because a range is typed from notes in
/// whole tenths at most: a range a tenth off is a different range, not a
/// doubled paste.
pub fn parse_batch(project: &Project, default_source: usize, text: &str) -> Batch<PendingSlate> {
    let stored: Vec<Key> = project
        .slates
        .iter()
        .map(|s| key(&PendingSlate::from(s)))
        .collect();
    line_batch::read_batch(text, |line, accepted: &[PendingSlate]| {
        Some(match parse_line(project, default_source, line)? {
            Err(reason) => BatchVerdict::Refused(reason),
            Ok(slate) => {
                let k = key(&slate);
                let seen = stored.contains(&k) || accepted.iter().any(|a| key(a) == k);
                if seen {
                    BatchVerdict::Duplicate(slate)
                } else {
                    BatchVerdict::Added(slate)
                }
            }
        })
    })
}

type Key = (usize, String, Option<String>, String);

fn key(slate: &PendingSlate) -> Key {
    (
        slate.source_index,
        format_time(slate.in_seconds),
        slate.out_seconds.map(format_time),
        slate.name.clone(),
    )
}

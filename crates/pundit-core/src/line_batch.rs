//! A pasted block of lines, read one line at a time by a grammar that is not
//! this module's — the container both paste boxes share.
//!
//! What is here is grammar-free: numbering every line of the block, skipping
//! the ones a grammar has nothing to say about, collecting what will add, and
//! joining the refused lines back into a block. What a line *means*, and
//! whether it repeats something already stored or already accepted from this
//! block, is the `judge`'s — `match_entry::parse_batch` and
//! `slate_entry::parse_batch` are the two.

/// What one line of a pasted block came to.
#[derive(Debug, Clone, PartialEq)]
pub enum BatchVerdict<T> {
    Added(T),
    /// The same record as one already in the project or already accepted from
    /// this block — by the grammar's own rule of "the same".
    Duplicate(T),
    /// A sentence for the coach, already naming what is wrong.
    Refused(String),
}

/// One echoed line, in input order.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchLine<T> {
    /// 1-based, counting every line of the block, so it names the line the
    /// coach can see.
    pub number: usize,
    /// The line as typed, trimmed — what a refusal quotes back.
    pub text: String,
    pub verdict: BatchVerdict<T>,
}

/// A pasted block, read.
#[derive(Debug, Clone, PartialEq)]
pub struct Batch<T> {
    /// Every line that will add, in input order: the command's payload.
    pub items: Vec<T>,
    /// One echo record per line that has anything to say. Blank and
    /// comment-only lines are not here.
    pub lines: Vec<BatchLine<T>>,
    /// The **refused** lines, joined back into a block — what the box keeps
    /// after Add, so the coach fixes them in place and presses Add again.
    ///
    /// Only the refusals: a line that was added is done with, and one skipped
    /// as [`BatchVerdict::Duplicate`] is not something editing can fix — the
    /// record it names is in the project already, so keeping it would ask the
    /// coach to clear a line whose only fault is being right. Blank and
    /// comment-only lines go with them, so a block that nothing refuses
    /// leaves the box empty.
    pub leftover: String,
}

/// Reads `text` line by line through `judge`.
///
/// `judge` gets one line and the items this block has accepted so far, and
/// answers `None` for a line with nothing to say (blank, or a comment) or the
/// line's verdict. It is `FnMut` so a grammar may keep a count of its own
/// across the block — `match_entry`'s start/stop cap does.
pub fn read_batch<T: Clone>(
    text: &str,
    mut judge: impl FnMut(&str, &[T]) -> Option<BatchVerdict<T>>,
) -> Batch<T> {
    let mut batch = Batch {
        items: Vec::new(),
        lines: Vec::new(),
        leftover: String::new(),
    };
    let mut leftover: Vec<&str> = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let Some(verdict) = judge(line, &batch.items) else {
            continue;
        };
        match &verdict {
            BatchVerdict::Added(item) => batch.items.push(item.clone()),
            BatchVerdict::Refused(_) => leftover.push(line.trim_end()),
            BatchVerdict::Duplicate(_) => {}
        }
        batch.lines.push(BatchLine {
            number: i + 1,
            text: line.trim().to_string(),
            verdict,
        });
    }
    batch.leftover = leftover.join("\n");
    batch
}

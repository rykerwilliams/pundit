//! The paste container, under a stub grammar. Its duplicate and leftover
//! rules are pinned by `match_entry`'s own batch tests, which run through it.

use pundit_core::line_batch::{read_batch, BatchVerdict};

/// One integer per line; `#…` and blank say nothing; anything else refused.
fn judge(line: &str, _accepted: &[u32]) -> Option<BatchVerdict<u32>> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    Some(match line.parse() {
        Ok(n) => BatchVerdict::Added(n),
        Err(_) => BatchVerdict::Refused("not a number".into()),
    })
}

#[test]
fn every_line_is_numbered_and_blanks_say_nothing() {
    let batch = read_batch("# c\n1\n\n2\nx\n", judge);
    assert_eq!(
        batch.lines.iter().map(|l| l.number).collect::<Vec<_>>(),
        [2, 4, 5],
        "numbered over every line of the block, the silent ones included"
    );
    assert_eq!(batch.items, [1, 2]);
    assert_eq!(batch.lines[2].text, "x");
    assert_eq!(batch.leftover, "x");
}

//! One slate per line (BACKLOG #97): the grammar behind the line editor's
//! slates kind — its row field and its paste box.

use pundit_core::line_batch::BatchVerdict;
use pundit_core::project::{Project, Slate, SourceRef};
use pundit_core::slate_entry::{
    edit_from_line, format_line, parse_batch, parse_line, PendingSlate,
};
use uuid::Uuid;

/// Two halves: an hour and 47:13 long.
fn project() -> Project {
    let mut p = Project::new("p");
    for (i, duration) in [3600.0, 2833.0].into_iter().enumerate() {
        p.source_videos.push(SourceRef {
            relative_path: format!("{i}.mp4"),
            display_name: format!("Half {}", i + 1),
            duration_seconds: duration,
            display_aspect: 16.0 / 9.0,
        });
    }
    p
}

fn pending(
    source_index: usize,
    in_seconds: f64,
    out_seconds: Option<f64>,
    name: &str,
    tags: &[&str],
) -> PendingSlate {
    PendingSlate {
        source_index,
        in_seconds,
        out_seconds,
        name: name.into(),
        tags: tags.iter().map(|t| t.to_string()).collect(),
    }
}

fn slate(
    source_index: usize,
    in_seconds: f64,
    out_seconds: Option<f64>,
    name: &str,
    tags: &[&str],
) -> Slate {
    Slate {
        id: Uuid::new_v4(),
        source_index,
        in_seconds,
        out_seconds,
        name: name.into(),
        tags: tags.iter().map(|t| t.to_string()).collect(),
        title: None,
    }
}

/// `line` read with the default video 1, which must be a slate.
fn reads(line: &str) -> PendingSlate {
    match parse_line(&project(), 0, line) {
        Some(Ok(slate)) => slate,
        other => panic!("{line:?} → {other:?}"),
    }
}

/// `line`'s refusal.
fn refusal(line: &str) -> String {
    match parse_line(&project(), 0, line) {
        Some(Err(reason)) => reason,
        other => panic!("{line:?} → {other:?}"),
    }
}

#[test]
fn every_shape_of_a_line_reads() {
    let cases: &[(&str, PendingSlate)] = &[
        (
            "2 14:05-14:40 corner routine [corners, set piece]",
            pending(
                1,
                845.0,
                Some(880.0),
                "corner routine",
                &["corners", "set piece"],
            ),
        ),
        // No video number: the default video.
        (
            "14:05-14:40 corner",
            pending(0, 845.0, Some(880.0), "corner", &[]),
        ),
        // Each dash, with no spaces and with a space on both sides.
        ("1 1:00-1:30 x", pending(0, 60.0, Some(90.0), "x", &[])),
        ("1 1:00 - 1:30 x", pending(0, 60.0, Some(90.0), "x", &[])),
        ("1 1:00–1:30 x", pending(0, 60.0, Some(90.0), "x", &[])),
        ("1 1:00 – 1:30 x", pending(0, 60.0, Some(90.0), "x", &[])),
        ("1 1:00—1:30 x", pending(0, 60.0, Some(90.0), "x", &[])),
        ("1 1:00 — 1:30 x", pending(0, 60.0, Some(90.0), "x", &[])),
        // Tenths and hours.
        (
            "1 0:52:10.5-0:52:31.5 x",
            pending(0, 3130.5, Some(3151.5), "x", &[]),
        ),
        // Open: a space or the end after the dash.
        (
            "1 3:20- free kick",
            pending(0, 200.0, None, "free kick", &[]),
        ),
        ("1 3:20-", pending(0, 200.0, None, "", &[])),
        ("1 3:20–", pending(0, 200.0, None, "", &[])),
        // An open range whose name starts with a time.
        (
            "1 5:00- 5:30 drill",
            pending(0, 300.0, None, "5:30 drill", &[]),
        ),
        // An empty name, with tags.
        (
            "1 52:10-52:31.5 [press]",
            pending(0, 3130.0, Some(3151.5), "", &["press"]),
        ),
        // Tags normalized by the tag field's own rule.
        (
            "1 1:00-1:30 x [Corners, corners ,, Set  Piece]",
            pending(0, 60.0, Some(90.0), "x", &["corners", "set  piece"]),
        ),
        // A name holding brackets, before the tags.
        (
            "1 1:00-1:30 [draft] corner [corners]",
            pending(0, 60.0, Some(90.0), "[draft] corner", &["corners"]),
        ),
        // `#` in the middle is part of the name.
        (
            "1 1:00-1:30 corner #3",
            pending(0, 60.0, Some(90.0), "corner #3", &[]),
        ),
        // Surrounding space is not part of anything.
        (
            "   1 1:00-1:30   corner   ",
            pending(0, 60.0, Some(90.0), "corner", &[]),
        ),
    ];
    for (line, want) in cases {
        assert_eq!(&reads(line), want, "{line:?}");
    }
    // A comment, only at the start of a line, and a blank line say nothing.
    for line in ["# notes", "   # notes", "", "   "] {
        assert_eq!(parse_line(&project(), 0, line), None, "{line:?}");
    }
}

#[test]
fn every_refusal_says_how_to_fix_it() {
    let range = "a slate is a range: 14:05-14:40, or 14:05- to leave its end open";
    let cases: &[(&str, &str)] = &[
        (
            "900 corner",
            "there is no video 900 — a time needs a colon (15:00)",
        ),
        (
            "3 14:05-14:40 x",
            "there is no video 3 — a time needs a colon (15:00)",
        ),
        ("2 corner", range),
        ("2 14:05 corner", range),
        ("2 14:05 - corner", range),
        ("2 1405-1440 corner", range),
        ("2 14:05-x", range),
        (
            "2 14:05 -14:40 x",
            "put a space on both sides of the dash or on neither",
        ),
        ("2 14:40-14:05 x", "the range ends before it starts"),
        ("2 47:00-47:20 x", "Half 2 is 47:13.0 long"),
        (
            "2 14:05-14:40 corner [corners",
            "the tags aren't closed — end them with ] (a name with a [ in it needs [] after it)",
        ),
        (
            "2 14:05-14:40 corner [corners] more",
            "nothing may follow the tags (a name with a [ in it needs [] after it)",
        ),
    ];
    for (line, want) in cases {
        assert_eq!(refusal(line), *want, "{line:?}");
    }
}

/// The one line most likely to arrive by mistake is a match event's — and a
/// block of them pasted into the slates box must add nothing.
#[test]
fn a_lone_time_is_refused_and_a_block_of_events_adds_no_slate() {
    assert!(refusal("2 14:05").starts_with("a slate is a range"));
    let batch = parse_batch(
        &project(),
        0,
        "2 14:05 home goal\n1 3:20 away goal\n2 30:00 period\n",
    );
    assert!(batch.items.is_empty());
    assert_eq!(batch.lines.len(), 3);
    assert!(batch
        .lines
        .iter()
        .all(|l| matches!(l.verdict, BatchVerdict::Refused(_))));
}

/// `match_entry` refuses them because a stray one moves every later period; a
/// slate never reaches `interpret`, and *kick-off routine* is a good name.
#[test]
fn kick_off_words_are_a_name_here() {
    assert_eq!(
        reads("2 14:05-14:40 kick-off routine").name,
        "kick-off routine"
    );
    assert_eq!(
        reads("1 1:00-1:30 ko whistle restart").name,
        "ko whistle restart"
    );
}

#[test]
fn each_end_is_bounded_and_the_range_must_run_forwards() {
    // Half 1 is an hour long: either end past it is refused.
    assert_eq!(refusal("1 60:01-60:30 x"), "Half 1 is 1:00:00.0 long");
    assert_eq!(refusal("1 59:00-60:01 x"), "Half 1 is 1:00:00.0 long");
    assert_eq!(refusal("1 60:01- x"), "Half 1 is 1:00:00.0 long");
    // To the very end is fine.
    assert_eq!(reads("1 59:00-60:00 x").out_seconds, Some(3600.0));
    // An empty range is not a range — `mark_slate_out`'s `<=`.
    assert_eq!(refusal("1 5:00-5:00 x"), "the range ends before it starts");
}

#[test]
fn a_line_reads_back_as_the_slate_it_was_written_from() {
    let p = project();
    for s in [
        slate(
            1,
            845.0,
            Some(880.0),
            "corner routine",
            &["corners", "set piece"],
        ),
        slate(0, 200.0, None, "free kick", &[]),
        slate(0, 3130.0, Some(3151.5), "", &["press"]),
        slate(0, 60.0, Some(90.0), "no tags", &[]),
        slate(0, 300.0, None, "5:30 drill", &[]),
        slate(0, 300.0, None, "", &[]),
    ] {
        let line = format_line(&s);
        let back = parse_line(&p, 0, &line).unwrap().unwrap();
        assert_eq!(
            back,
            pending(
                s.source_index,
                s.in_seconds,
                s.out_seconds,
                &s.name,
                &s.tags.iter().map(String::as_str).collect::<Vec<_>>()
            ),
            "{line:?}"
        );
    }
    assert_eq!(
        format_line(&slate(
            1,
            845.0,
            Some(880.0),
            "corner routine",
            &["corners", "set piece"]
        )),
        "2 14:05.0-14:40.0 corner routine [corners, set piece]"
    );
    assert_eq!(
        format_line(&slate(0, 300.0, None, "5:30 drill", &[])),
        "1 5:00.0- 5:30 drill"
    );
}

/// The tag group starts at the line's last `[`, so a name holding one needs
/// an empty `[]` after it when there are no tags — and only then.
#[test]
fn a_name_with_a_bracket_round_trips() {
    let p = project();
    for name in ["a [b", "a [b] c", "[x]"] {
        let s = slate(0, 10.0, Some(20.0), name, &[]);
        let line = format_line(&s);
        assert!(line.ends_with(" []"), "{line:?}");
        let back = parse_line(&p, 0, &line).unwrap().unwrap();
        assert_eq!(back.name, name, "{line:?}");
        assert!(back.tags.is_empty(), "{line:?}");
    }
    // With tags there is a group to find, and no `[]`.
    let line = format_line(&slate(0, 10.0, Some(20.0), "a [b", &["t"]));
    assert_eq!(line, "1 0:10.0-0:20.0 a [b [t]");
    assert_eq!(parse_line(&p, 0, &line).unwrap().unwrap().name, "a [b");
}

/// A tag holding a `[` does not survive the grammar, so without the
/// byte-identical rule a row nobody edited would be rewritten — its tags and
/// its name both — the moment focus left it.
#[test]
fn an_untouched_row_changes_nothing_even_with_a_bracket_in_a_tag() {
    let p = project();
    let s = slate(0, 845.06, Some(880.04), "corner", &["set [a] piece"]);
    let seed = format_line(&s);
    assert_eq!(
        edit_from_line(&p, &s, &seed),
        Ok(pending(
            0,
            845.06,
            Some(880.04),
            "corner",
            &["set [a] piece"]
        ))
    );
}

#[test]
fn an_unchanged_end_keeps_its_stored_seconds() {
    let p = project();
    let s = slate(0, 845.06, Some(880.04), "corner", &[]);
    assert_eq!(format_line(&s), "1 14:05.0-14:40.0 corner");

    // The out moved; the in, typed as it was seeded, keeps its 845.06 —
    // whether the dash is spaced or not.
    for typed in ["1 14:05.0-14:45.0 corner", "1 14:05.0 - 14:45.0 corner"] {
        let got = edit_from_line(&p, &s, typed).unwrap();
        assert_eq!(got.in_seconds, 845.06, "{typed:?}");
        assert_eq!(got.out_seconds, Some(885.0), "{typed:?}");
    }

    // Only the name changed, and it holds a dash: both ends are kept.
    let got = edit_from_line(&p, &s, "1 14:05.0-14:40.0 corner-kick").unwrap();
    assert_eq!(got.name, "corner-kick");
    assert_eq!((got.in_seconds, got.out_seconds), (845.06, Some(880.04)));

    // Another video under the same times keeps both.
    let got = edit_from_line(&p, &s, "2 14:05.0-14:40.0 corner").unwrap();
    assert_eq!(got.source_index, 1);
    assert_eq!((got.in_seconds, got.out_seconds), (845.06, Some(880.04)));
}

/// The order is a property of what is stored: an out typed at 14:05.05 is
/// after the 14:05.0 the in point's text reads, and still *behind* the kept
/// in point of 845.06.
#[test]
fn the_order_is_checked_on_the_seconds_that_will_be_stored() {
    let p = project();
    let s = slate(0, 845.06, Some(880.04), "corner", &[]);
    assert_eq!(
        edit_from_line(&p, &s, "1 14:05.0-14:05.05 corner"),
        Err("the range ends before it starts".into())
    );
}

#[test]
fn an_emptied_row_is_refused_naming_its_line() {
    let p = project();
    let s = slate(0, 845.0, Some(880.0), "corner", &[]);
    for typed in ["", "   ", "# gone"] {
        assert_eq!(
            edit_from_line(&p, &s, typed),
            Err("nothing on that line — a row reads \"1 14:05.0-14:40.0 corner\"".into()),
            "{typed:?}"
        );
    }
}

#[test]
fn a_pasted_slate_equal_to_a_stored_one_is_a_duplicate() {
    let mut p = project();
    p.slates
        .push(slate(1, 845.06, Some(880.0), "corner", &["corners"]));

    // The stored one renders as 14:05.0: the same text, so the same slate —
    // and its tags are not part of the question.
    let batch = parse_batch(&p, 0, "2 14:05-14:40 corner [other]\n");
    assert!(matches!(batch.lines[0].verdict, BatchVerdict::Duplicate(_)));
    assert!(batch.items.is_empty());
    assert_eq!(batch.leftover, "", "the slate it names is there already");

    // A block holding a line twice adds it once.
    let batch = parse_batch(&p, 0, "1 1:00-1:30 x\n1 1:00-1:30 x\n");
    assert_eq!(batch.items.len(), 1);
    assert!(matches!(batch.lines[1].verdict, BatchVerdict::Duplicate(_)));
    assert_eq!(batch.leftover, "");
}

#[test]
fn the_same_range_with_another_name_is_a_new_slate() {
    let mut p = project();
    p.slates.push(slate(1, 845.0, Some(880.0), "corner", &[]));
    for line in [
        "2 14:05-14:40 corner defence",
        "2 14:05.1-14:40 corner",
        "2 14:05-14:40.1 corner",
        "2 14:05- corner",
        "1 14:05-14:40 corner",
    ] {
        let batch = parse_batch(&p, 0, line);
        assert!(
            matches!(batch.lines[0].verdict, BatchVerdict::Added(_)),
            "{line:?}"
        );
    }
}

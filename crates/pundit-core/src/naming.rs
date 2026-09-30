//! What a name means, and what a name may contain.
//!
//! Two jobs, kept apart on purpose. **Reading** a name the coach already has:
//! the order a game's halves go in ([`order_videos`]), and the date and the
//! opponent a folder's name carries ([`parse_date_in`], [`opponent_from`]).
//! **Writing** one a filesystem will accept: [`safe_chars`] for a file name,
//! [`folder_slug`] for a folder. Those last two are deliberately not one
//! pipeline — [`safe_chars`] says what making them one would cost.
//!
//! Pure string and path rules, no clock and no I/O, like the rest of core.
//! [`parse_date_in`] taking the year it refuses above as an argument is that
//! rule showing up in a signature.

use std::cmp::Ordering;
use std::path::{Path, PathBuf};

use crate::metadata::CalendarDate;

/// Puts a picked set of videos into the order they should be added in.
///
/// **Sorting the names is right, but sorting them byte-wise is not.** Camera
/// and phone files are named by when they were shot, so a name sort is what
/// puts a game's halves in sequence — except that a browser or a file manager
/// marks a second copy with a ` (n)` suffix *before* the extension, and that
/// suffix sorts **earlier** than no suffix at all: `' '` is `0x20` and `'.'` is
/// `0x2E`, so `Hudson (1).mp4` came back before `Hudson.mp4` and a match's two
/// halves went in **reversed**, silently. That is the whole reason this function
/// exists rather than a `sort_by_key(file_name)`.
///
/// So the key is the stem with any ` (n)` taken off it, then `n` — which puts
/// the original first and its copies after it in numeric order, and leaves every
/// name without a suffix exactly where a name sort already had it.
///
/// A reversed pair is not a cosmetic fault: the source order *is* the match
/// timeline, so it decides the match clock, every clip's position and what a
/// burned-in scoreboard reads.
pub fn order_videos(paths: &mut [PathBuf]) {
    paths.sort_by(|a, b| compare_names(a, b));
}

/// [`order_videos`]'s rule for two paths, on their file names alone.
fn compare_names(a: &Path, b: &Path) -> Ordering {
    let key = |p: &Path| {
        let name = p.file_name().unwrap_or_default().to_string_lossy();
        let (stem, copy, ext) = split_copy(&name);
        (stem.to_owned(), copy, ext.to_owned())
    };
    key(a).cmp(&key(b))
}

/// `"Hudson (2).mp4"` → `("Hudson", 2, "mp4")`, and `"Hudson.mp4"` →
/// `("Hudson", 0, "mp4")`.
///
/// **`0` for no suffix, so the original leads its own copies.** A ` (0)` file
/// would tie with it, which is a name nothing produces and would order by the
/// extension.
fn split_copy(name: &str) -> (&str, u32, &str) {
    let (stem, ext) = match name.rsplit_once('.') {
        // A dotfile is all stem: `.hidden` has no extension to speak of.
        Some(("", _)) | None => (name, ""),
        Some((stem, ext)) => (stem, ext),
    };
    let Some(open) = stem.strip_suffix(')').and_then(|s| s.rfind(" (")) else {
        return (stem, 0, ext);
    };
    let digits = &stem[open + 2..stem.len() - 1];
    // Only a run of digits is a copy marker. `Final (edit).mp4` keeps its whole
    // stem, which is what a name sort would already have done with it.
    match digits.parse::<u32>() {
        Ok(copy) if !digits.is_empty() => (&stem[..open], copy, ext),
        _ => (stem, 0, ext),
    }
}

/// Every character a file name may not hold, replaced by `-`.
///
/// The app's export names have long cleaned two of them, and the comment
/// saying why is right: `/` is the path separator and `:` is what "a share to
/// a Mac or a Windows machine trips over". The set was short by seven. The
/// coach's projects live on a cloud-sync mount, and exFAT, NTFS and SMB reject
/// `" * ? < > |` and `\` as well, so a name this laptop's ext4 writes
/// perfectly well is a file the share it is going to cannot hold. **`\` is the
/// one that changes an existing behaviour:** on Linux a backslash in a clip
/// name is a legal file name today.
///
/// Control characters go too. They are legal in a POSIX name and there is no
/// dialog, terminal or player that shows one honestly.
///
/// **This replaces characters and does nothing else** — no collapsing, no
/// trimming, no truncation, no case change. It is what an export's file name
/// is cleaned with, and an export's basename is what its `.srt` and
/// `.chapters.txt` are named after, so any of those four would rename every
/// future export and orphan the sidecars beside the files already written —
/// and two long labels that differ only past byte 64 would collide *after*
/// de-duplication had already found them distinct. [`folder_slug`] is the
/// pipeline; this is the floor under it.
pub fn safe_chars(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '-',
            c if c.is_control() => '-',
            c => c,
        })
        .collect()
}

/// The longest prefix of `text` that is at most `bytes` long and does not
/// split a character.
///
/// The budget is a **byte** budget, because that is what a filesystem caps a
/// single name in — and slicing a `str` at a byte index inside a multi-byte
/// character panics, which is the whole reason this is a function and not a
/// range.
///
/// Two callers at two budgets: a film's typed name at 200 bytes (leaving room
/// for `.mp4`, a ` (10)` suffix and a straddling character inside the 255-byte
/// cap every filesystem here imposes) and [`folder_slug`] at 64. The mechanism
/// is shared and the budgets deliberately are not.
pub fn truncate_on_boundary(text: &str, bytes: usize) -> &str {
    if text.len() <= bytes {
        return text;
    }
    let mut cut = bytes;
    // Byte 0 is always a boundary, so this terminates.
    while !text.is_char_boundary(cut) {
        cut -= 1;
    }
    &text[..cut]
}

/// How many bytes of a folder name are kept. A project folder sits inside a
/// path that also holds a `recordings/` subdirectory and a `<uuid>.mkv`, so it
/// is budgeted well below the 255 bytes a single name may take.
const FOLDER_SLUG_BYTES: usize = 64;

/// A typed match name as the folder that holds it: [`safe_chars`], then lower
/// case, then every whitespace run to one `-`, then `-` runs collapsed, then
/// [`truncate_on_boundary`] at 64 bytes, then `-`, `.` and whitespace trimmed
/// off both ends.
///
/// `"Rovers United"` → `"rovers-united"`.
///
/// **Truncate before trimming.** A cut at byte 64 can land on a `-` or a `.`,
/// and Windows and SMB silently strip a trailing dot — the exact class of
/// failure this function exists to prevent, on the mount the coach's projects
/// live on. Trimming first and cutting second puts one back.
///
/// The trim's whitespace arm is belt and braces: turning whitespace runs into
/// `-` has already happened, so no cut can land on a space. It is one
/// `trim_matches` pattern either way, and it is what makes the rule read
/// whole.
///
/// **Only the characters this function removes can make a name vanish.**
/// `"///"` and `"..."` come back empty; `"!!!???"` comes back `"!!!"`, because
/// `?` is replaced and `!` is not. Stripping the rest of Unicode's punctuation
/// would need a category table, which is a dependency this crate is not
/// getting — and an accented name is not punctuation at all: it lowercases per
/// Unicode and keeps its letters, since every filesystem this app targets
/// stores UTF-8 names and transliterating one would need a crate too.
pub fn folder_slug(name: &str) -> String {
    let mut slug = String::with_capacity(name.len());
    for c in safe_chars(name).to_lowercase().chars() {
        if c.is_whitespace() || c == '-' {
            // One `-` per run, which collapses the replacement's own output
            // along with any the name already had.
            if !slug.ends_with('-') {
                slug.push('-');
            }
        } else {
            slug.push(c);
        }
    }
    truncate_on_boundary(&slug, FOLDER_SLUG_BYTES)
        .trim_matches(|c: char| c == '-' || c == '.' || c.is_whitespace())
        .to_owned()
}

/// The date a name carries, or `None`.
///
/// Four shapes — `YYYYMMDD`, `YYYY-MM-DD`, `YYYY_MM_DD` and `YYYY.MM.DD` —
/// each bounded by a non-digit or an end of the string, with the month
/// `1..=12`, the day `1..=31` and the year `2000..=max_year`. **Leftmost
/// wins**, which is what takes the right date out of a name carrying a date
/// and then a bare year.
///
/// **A whole date or nothing, and that is the whole rule.** It is what stops
/// two shapes the coach's own folders have: a team folder named after a
/// four-digit birth **year**, which a `YYYY` matcher would date every match in
/// that tree to; and a match folder opening with a six-digit age-group code,
/// which a `YYMMDD` matcher would read as month 15. Eight digits with a month
/// and a day that exist reject both, and the bound on the digit run is the
/// other half — an eight-digit window inside a longer run of digits is not a
/// date either.
///
/// A day the month does not have (`2026-02-31`) is accepted: the day is
/// range-checked, not calendar-checked. This is a prefill read out of a folder
/// name and nothing in core does arithmetic on it.
///
/// **`max_year` is an argument because core has no clock.** The app passes
/// this year plus one. It is not a constant here for the same reason
/// [`CalendarDate`] is passed into [`crate::metadata::file_tags`] rather than
/// derived — and note that a `SystemTime::now()` in this crate would sail
/// through the dependency audit, because a clock adds no dependency. This
/// signature is the only thing enforcing the rule.
pub fn parse_date_in(text: &str, max_year: i32) -> Option<CalendarDate> {
    let bytes = text.as_bytes();
    let digit = |i: usize| bytes.get(i).is_some_and(u8::is_ascii_digit);
    let mut i = 0;
    while i < bytes.len() {
        if !digit(i) {
            i += 1;
            continue;
        }
        let mut end = i;
        while digit(end) {
            end += 1;
        }
        let found = match end - i {
            8 => date_in(text, [i, i + 4, i + 6], max_year),
            // `YYYY<sep>MM<sep>DD`, one separator used twice, and the day is
            // the end of its own digit run.
            4 if matches!(bytes.get(i + 4), Some(b'-' | b'_' | b'.'))
                && bytes.get(i + 7) == bytes.get(i + 4)
                && digit(i + 5)
                && digit(i + 6)
                && digit(i + 8)
                && digit(i + 9)
                && !digit(i + 10) =>
            {
                date_in(text, [i, i + 5, i + 8], max_year)
            }
            _ => None,
        };
        if found.is_some() {
            return found;
        }
        // Past the whole run: a date can only start where one starts.
        i = end;
    }
    None
}

/// [`parse_date_in`]'s year, month and day, at the three byte offsets a shape
/// puts them, checked against the ranges a date has.
fn date_in(text: &str, at: [usize; 3], max_year: i32) -> Option<CalendarDate> {
    let [year, month, day] = at;
    let date = CalendarDate {
        year: text[year..year + 4].parse().ok()?,
        month: text[month..month + 2].parse().ok()?,
        day: text[day..day + 2].parse().ok()?,
    };
    let ok = (2000..=max_year).contains(&date.year)
        && (1..=12).contains(&date.month)
        && (1..=31).contains(&date.day);
    ok.then_some(date)
}

/// The opponent a match folder's name names, or `None`.
///
/// Strip a leading run of digits and separators — that is either a date or a
/// code, and in neither case a team — then a trailing one; turn `_`, `-` and
/// `.` into spaces; collapse and trim; and capitalize each word of whatever is
/// left. `"2026-09-21-athletic"` → `"Athletic"`,
/// `"170918-city_reserves"` → `"City Reserves"`, `"20260917"` → `None`.
///
/// **The folder's name and nothing else.** File names in the coach's trees do
/// contain the opponent — wrapped in a date, an age group, a club
/// abbreviation, a separator that is itself a hyphen-underscore sandwich and a
/// half marker. Anything that pulled a team out of that would be fitted to one
/// camera system and would quietly produce nonsense on the other, whose file
/// names carry no team at all. The folder name is the one the coach curates.
///
/// **Only a word's first letter is touched**, and the rest is left exactly as
/// typed, so `FC` stays `FC` and `McBride` stays `McBride`. Lowercasing the
/// tail is what "title case" usually means, and it would destroy a
/// capitalization the coach chose — which a prefill has no business doing.
pub fn opponent_from(folder_name: &str) -> Option<String> {
    let separator = |c: char| matches!(c, '-' | '_' | '.') || c.is_whitespace();
    let name = folder_name
        .trim_matches(|c: char| c.is_ascii_digit() || separator(c))
        .split(separator)
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    (!name.is_empty()).then_some(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ordered(names: &[&str]) -> Vec<String> {
        let mut paths: Vec<PathBuf> = names.iter().map(|n| PathBuf::from("/v").join(n)).collect();
        order_videos(&mut paths);
        paths
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect()
    }

    /// The bug this module exists for, in the shape the coach's own files have:
    /// a second half downloaded as a copy sorted **first** byte-wise, which put
    /// the halves in the project reversed and the match clock with them.
    #[test]
    fn a_copy_suffix_follows_the_file_it_is_a_copy_of() {
        assert_eq!(
            ordered(&["GAME 2016B vs Hudson (1).mp4", "GAME 2016B vs Hudson.mp4"]),
            ["GAME 2016B vs Hudson.mp4", "GAME 2016B vs Hudson (1).mp4"],
        );
        // And byte-wise this is the order that used to come back, so the test
        // fails against a plain name sort rather than passing either way.
        let mut bytewise = ["GAME 2016B vs Hudson (1).mp4", "GAME 2016B vs Hudson.mp4"];
        bytewise.sort_unstable();
        assert_eq!(bytewise[0], "GAME 2016B vs Hudson (1).mp4");
    }

    #[test]
    fn copies_run_in_numeric_order_not_lexicographic() {
        assert_eq!(
            ordered(&["h (10).mp4", "h (2).mp4", "h.mp4", "h (1).mp4"]),
            ["h.mp4", "h (1).mp4", "h (2).mp4", "h (10).mp4"],
        );
    }

    /// Every name without a copy suffix keeps the order a plain name sort gave
    /// it, which is what the camera-file case relies on.
    #[test]
    fn plain_names_are_unchanged() {
        assert_eq!(
            ordered(&["VID_002.mp4", "VID_001.mp4", "VID_003.mp4"]),
            ["VID_001.mp4", "VID_002.mp4", "VID_003.mp4"],
        );
    }

    /// A parenthesis that is not a copy marker is part of the name.
    #[test]
    fn a_parenthesis_that_is_not_a_number_is_part_of_the_stem() {
        assert_eq!(
            ordered(&["half (edit).mp4", "half (1).mp4", "half.mp4"]),
            // `half` then its copy, and `half (edit)` sorts as the stem it is.
            ["half.mp4", "half (1).mp4", "half (edit).mp4"],
        );
    }

    /// Two halves that differ only in extension still have a defined order, and
    /// a name with no extension at all does not panic.
    #[test]
    fn an_extension_breaks_a_tie_and_a_bare_name_is_fine() {
        assert_eq!(ordered(&["h.mp4", "h.mkv"]), ["h.mkv", "h.mp4"]);
        assert_eq!(ordered(&["second", "first"]), ["first", "second"]);
    }

    // ── What a name may contain ────────────────────────────────────────────

    #[test]
    fn safe_chars_replaces_every_character_a_share_refuses() {
        assert_eq!(safe_chars("a/b\\c:d*e?f\"g<h>i|j"), "a-b-c-d-e-f-g-h-i-j");
        // A tab and a bell are both control characters and both legal in a
        // POSIX name.
        assert_eq!(safe_chars("tab\there\u{7}"), "tab-here-");
    }

    /// The three axes [`folder_slug`] has and this function must not. An
    /// export's basename is cleaned with this one and its `.srt` and
    /// `.chapters.txt` are named after that basename, so a collapse, a trim or
    /// a cut here renames every future export.
    #[test]
    fn safe_chars_does_nothing_but_replace() {
        assert_eq!(
            safe_chars("Second half -- away at City"),
            "Second half -- away at City"
        );
        assert_eq!(safe_chars("Corners.. "), "Corners.. ");
        let long = "x".repeat(200);
        assert_eq!(safe_chars(&long), long);
    }

    #[test]
    fn folder_slug_lowercases_and_hyphenates_one_run_at_a_time() {
        assert_eq!(folder_slug("Rovers United"), "rovers-united");
        assert_eq!(folder_slug("  Rovers   United  "), "rovers-united");
        assert_eq!(
            folder_slug("2026-09-21-Rovers - Athletic"),
            "2026-09-21-rovers-athletic"
        );
    }

    /// An accented name is not punctuation: it lowercases per Unicode and keeps
    /// its letters, because mangling it would need a transliteration crate this
    /// crate is not getting.
    #[test]
    fn folder_slug_keeps_an_accented_name() {
        assert_eq!(folder_slug("Atlético Ñuñoa"), "atlético-ñuñoa");
    }

    /// Only the characters this function removes can make a name vanish.
    /// `"!!!???"` keeping its `!`s is the rule and not a defect: stripping the
    /// rest of Unicode's punctuation needs a category table core does not have.
    #[test]
    fn folder_slug_empties_only_what_it_removes() {
        assert_eq!(folder_slug("///"), "");
        assert_eq!(folder_slug("..."), "");
        assert_eq!(folder_slug("   "), "");
        assert_eq!(folder_slug("!!!???"), "!!!");
    }

    /// Truncate **before** trimming: the cut can land on a `-` or a `.`, and
    /// Windows and SMB silently strip a trailing dot. It can never land on a
    /// space, because whitespace runs are already `-` by then — which is why
    /// the trim's whitespace arm is belt and braces rather than reachable.
    #[test]
    fn folder_slug_truncates_before_it_trims() {
        let sixty_three = "a".repeat(63);
        let on_hyphen = format!("{sixty_three}-{}", "b".repeat(20));
        assert_eq!(folder_slug(&on_hyphen), sixty_three);
        let on_dot = format!("{sixty_three}.{}", "b".repeat(20));
        assert_eq!(folder_slug(&on_dot), sixty_three);
    }

    /// The budget is bytes, and a cut inside a multi-byte character would panic
    /// on the slice. Both callers' budgets are covered: a folder's 64 and a
    /// film's 200.
    #[test]
    fn truncate_on_boundary_never_splits_a_character() {
        // Three bytes each, so neither budget is a boundary.
        let euros = "€".repeat(30);
        assert_eq!(truncate_on_boundary(&euros, 64), "€".repeat(21)); // 63 bytes
        let many = "€".repeat(80);
        assert_eq!(truncate_on_boundary(&many, 200), "€".repeat(66)); // 198 bytes
        assert_eq!(truncate_on_boundary("héllo", 2), "h");
        assert_eq!(truncate_on_boundary("short enough", 64), "short enough");
    }

    // ── What a name means ──────────────────────────────────────────────────

    fn date(year: i32, month: u32, day: u32) -> Option<CalendarDate> {
        Some(CalendarDate { year, month, day })
    }

    #[test]
    fn parse_date_in_reads_each_accepted_shape() {
        assert_eq!(parse_date_in("20260921-rovers", 2027), date(2026, 9, 21));
        assert_eq!(parse_date_in("2026-09-21-rovers", 2027), date(2026, 9, 21));
        assert_eq!(parse_date_in("2026_09_21 rovers", 2027), date(2026, 9, 21));
        assert_eq!(
            parse_date_in("GAME 2026.09.21.mp4", 2027),
            date(2026, 9, 21)
        );
    }

    /// A bare year and a six-digit code are the two shapes the coach's own
    /// folders have, and neither is a date: a `YYYY` matcher would date every
    /// match in one tree to a birth year, and a `YYMMDD` one would read an
    /// age-group code as month 15.
    #[test]
    fn parse_date_in_needs_a_whole_date() {
        assert_eq!(parse_date_in("2016B-rovers", 2027), None);
        assert_eq!(parse_date_in("170918-rovers", 2027), None);
        assert_eq!(parse_date_in("2026-09-rovers", 2027), None);
    }

    #[test]
    fn parse_date_in_refuses_a_month_or_a_day_that_does_not_exist() {
        assert_eq!(parse_date_in("20261301", 2027), None);
        assert_eq!(parse_date_in("2026-13-01", 2027), None);
        assert_eq!(parse_date_in("2026-12-32", 2027), None);
        assert_eq!(parse_date_in("2026-00-10", 2027), None);
    }

    /// `max_year` is the caller's, so this test has no clock in it — which is
    /// the whole reason the bound is an argument.
    #[test]
    fn parse_date_in_refuses_a_year_outside_the_window() {
        assert_eq!(parse_date_in("19990921", 2027), None);
        assert_eq!(parse_date_in("20280921", 2027), None);
        assert_eq!(parse_date_in("20280921", 2028), date(2028, 9, 21));
    }

    /// Leftmost wins, which is what takes the right one out of a name carrying
    /// a date and then a bare year (both of the coach's trees).
    #[test]
    fn parse_date_in_takes_the_leftmost_date() {
        assert_eq!(
            parse_date_in("20260921-2016b-vs-20270105", 2027),
            date(2026, 9, 21)
        );
    }

    /// The run has to be bounded by a non-digit or an end of the string, so an
    /// eight-digit window inside a longer run of digits is not a date.
    #[test]
    fn parse_date_in_refuses_an_eight_digit_window_in_a_longer_run() {
        assert_eq!(parse_date_in("1202609215", 2027), None);
        assert_eq!(parse_date_in("2026-09-215", 2027), None);
    }

    #[test]
    fn opponent_from_strips_a_date_or_a_code_and_capitalizes_the_rest() {
        assert_eq!(
            opponent_from("2026-09-21-athletic").as_deref(),
            Some("Athletic")
        );
        assert_eq!(
            opponent_from("170918-city_reserves").as_deref(),
            Some("City Reserves")
        );
        assert_eq!(
            opponent_from("rovers.united.b").as_deref(),
            Some("Rovers United B")
        );
        // A trailing code goes the same way as a leading one.
        assert_eq!(opponent_from("athletic-2").as_deref(), Some("Athletic"));
    }

    /// A folder named only by its date or its code names no team, and the field
    /// is left empty rather than filled with a guess.
    #[test]
    fn opponent_from_a_name_of_digits_alone_is_none() {
        assert_eq!(opponent_from("20260917"), None);
        assert_eq!(opponent_from("2026-09-21"), None);
        assert_eq!(opponent_from(""), None);
    }

    /// Only a word's first letter is touched: `FC` is a capitalization the
    /// coach chose and a prefill has no business flattening it.
    #[test]
    fn opponent_from_keeps_a_capital_it_did_not_add() {
        assert_eq!(
            opponent_from("170918-fc united").as_deref(),
            Some("Fc United")
        );
        assert_eq!(
            opponent_from("170918-FC United").as_deref(),
            Some("FC United")
        );
    }
}

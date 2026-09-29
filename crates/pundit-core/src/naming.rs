//! What a file's name means — today, only the order a game's halves go in.
//!
//! Pure string and path rules, no clock and no I/O, like the rest of core.

use std::cmp::Ordering;
use std::path::{Path, PathBuf};

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
}

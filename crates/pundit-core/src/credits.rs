//! A fetched track's credit (BACKLOG #84 spec C1–C2): title, author, source and
//! licence, with links, ready to paste into a video's description.
//!
//! Worded **once, at Add** — the one moment the app knows the licence — and
//! stored as text beside the track. `pundit-media` copies that text beside the
//! reel at render time and knows nothing about what it says.

use crate::openverse::{licence_url, licence_words, Candidate, Licence};

/// CC BY's modification notice (BY 4.0 §3(a)(1)(B): a change must be
/// indicated). A reel cuts, ducks and fades the track, so every credit says so;
/// for CC0 and the mark it is owed nothing and costs nothing.
pub const MODIFIED: &str = "(edited and mixed under the footage)";

/// The credit for `candidate`, three lines (two when the catalogue gives no
/// page for the track):
///
/// ```text
/// Music (edited and mixed under the footage): "Title" by Creator
/// https://www.jamendo.com/track/1015339
/// Licensed under CC BY 3.0 — https://creativecommons.org/licenses/by/3.0/
/// ```
///
/// Openverse's own `attribution` sentence is not used: it has no link to the
/// track, which BY asks for where reasonably practicable, and its wording is
/// not ours to keep stable.
pub fn credit_text(candidate: &Candidate) -> String {
    let mut text = format!(
        "Music {MODIFIED}: \"{}\" by {}\n",
        candidate.title, candidate.creator
    );
    if let Some(page) = &candidate.landing_url {
        text.push_str(page);
        text.push('\n');
    }
    let words = licence_words(candidate.license, candidate.license_version.as_deref());
    let licence = match candidate.license {
        Licence::By => format!("Licensed under {words}"),
        Licence::Cc0 | Licence::Pdm => words,
    };
    text.push_str(&format!("{licence} — {}\n", licence_url(candidate)));
    text
}

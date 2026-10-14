//! Find music (BACKLOG #84 phase 2, spec S3): a search of Openverse, and a
//! result fetched into the music folder with its credit beside it.
//!
//! **Functions, not commands.** `main.rs` runs each on a short-lived thread and
//! takes the answer back on the event loop; the bus is not involved, holds no
//! candidate and has no command that could carry a URL. The only URL [`add`]
//! fetches is the `url` of a [`Candidate`] that [`search`] parsed.
//!
//! **What lands is what the mixer already reads**: `<folder>/<genre>/<title> —
//! <creator>.<ext>`, which `music::tracks` lists the moment it exists, and
//! `<that>.credit.txt` beside it, which media copies beside the reel. So the
//! export path never learns that a catalogue exists.
//!
//! Every failure is worded here for the coach — **never GStreamer's own
//! words**, which go to stderr.

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use pundit_core::credits::credit_text;
use pundit_core::metadata::UNTITLED;
use pundit_core::naming;
use pundit_core::openverse::{rank, results, search_url, Candidate};
use pundit_media::composite::CompositeError;
use pundit_media::{credit_beside, get, inspect_audio, GetError};

/// A page of twenty rows is 35–39 KB (measured); a page past a mebibyte is
/// not one.
const PAGE_LIMIT: usize = 1 << 20;

/// **Sized for MP3, which is what the catalogue serves**: a measured 3:24
/// Jamendo track was 4.5 MB, so a ten-minute `length=medium` song is about
/// 13 MB. A ten-minute WAV (about 106 MB) would not fit, and is refused in
/// [`TOO_LARGE`]'s words rather than half-fetched.
const TRACK_LIMIT: usize = 64 << 20;

/// What a track over [`TRACK_LIMIT`] is told.
const TOO_LARGE: &str = "the track is larger than pundit takes (64 MB)";

/// How far the decoded length may be from the catalogue's before the file is
/// not the song it claims to be (spec S7).
const LENGTH_SLACK_SECONDS: f64 = 2.0;

/// How many bytes of `<title> — <creator>` name the file: room for the
/// extension and `.credit.txt` inside the 255 every filesystem allows.
const STEM_BYTES: usize = 120;

/// What Openverse says when the anonymous quota is spent (spec S11, §O3).
const RATE_LIMITED: &str =
    "Openverse allows 20 searches a minute and 200 a day without an account — try again shortly.";

/// What a file that fails its check is called on its row.
const NOT_A_TRACK: &str = "not a track pundit can play";

/// Where a search lands: the folder name under the music folder, decided when
/// Search is pressed, and the words searched for. **An Add lands in the genre
/// its search was made under**, so this travels with the results.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Genre {
    /// The subfolder's name: an existing one's own, or the slug.
    pub name: String,
    /// The field's trimmed text, which is what Openverse is asked for.
    pub query: String,
}

impl Genre {
    /// The genre `text` names in `folder` (spec S5): an existing subfolder
    /// whose [`naming::folder_slug`] is the text's, **under its own name** —
    /// so `" ROCK"` finds `Rock/` — else the slug itself, which also means a
    /// `/` can never climb out of the music folder. An empty slug is refused
    /// before any request is made.
    pub fn resolve(folder: &Path, text: &str) -> Result<Genre, String> {
        let slug = naming::folder_slug(text.trim());
        if slug.is_empty() {
            return Err("Type a genre to search for".to_owned());
        }
        let existing = std::fs::read_dir(folder).ok().and_then(|entries| {
            entries
                .flatten()
                .filter(|e| e.path().is_dir())
                .filter_map(|e| e.file_name().into_string().ok())
                // Not a hidden one, which `music::genres` never lists: its
                // slug drops the dot, so `.rock/` would otherwise be "rock".
                .filter(|name| !name.starts_with('.'))
                .find(|name| naming::folder_slug(name) == slug)
        });
        Ok(Genre {
            name: existing.unwrap_or(slug),
            query: text.trim().to_owned(),
        })
    }
}

/// One page of Openverse's music for `genre`, the rows whose genres include it
/// first. `base` is the API's root (`main.rs`'s one `const`, or a test's local
/// server). **Not cancellable**: a superseded search finishes and is ignored.
pub fn search(base: &str, genre: &Genre) -> Result<Vec<Candidate>, String> {
    let url = search_url(base, &genre.query);
    let cant_read =
        |why: &str| format!("Openverse answered with something pundit can't read ({why})");
    let body = get(&url, PAGE_LIMIT, &AtomicBool::new(false)).map_err(|e| match e {
        GetError::Status(429) => RATE_LIMITED.to_owned(),
        GetError::Status(code) => cant_read(&format!("HTTP {code}")),
        GetError::Unreachable(_) => e.to_string(),
        GetError::TooLarge(_) => cant_read("an answer too long to be a page"),
        GetError::Failed(_) => cant_read("the answer broke off"),
        GetError::Cancelled => "Cancelled".to_owned(),
    })?;
    let text = String::from_utf8_lossy(&body);
    let mut candidates = results(&text).map_err(|e| cant_read(&e.to_string()))?;
    rank(&mut candidates, &genre.query);
    Ok(candidates)
}

/// The file stem `candidate` lands under: `<title> — <creator>`, cleaned by
/// [`naming::file_stem`]. **The same stem is the same song**: there is no
/// `" (2)"`, so [`already_in`] is one existence check.
pub fn stem(candidate: &Candidate) -> String {
    naming::file_stem(
        &format!("{} — {}", candidate.title, candidate.creator),
        STEM_BYTES,
        UNTITLED,
    )
}

/// Whether `stem` is already a track of `genre` in `folder`, in any extension
/// a track may have. A scan of the tracks and not of the credits, so a song
/// deleted from the folder can be added again.
pub fn already_in(folder: &Path, genre: &Genre, stem: &str) -> bool {
    let dir = folder.join(&genre.name);
    crate::music::EXTENSIONS
        .iter()
        .any(|ext| dir.join(format!("{stem}.{ext}")).is_file())
}

/// Fetches `candidate` into `folder/<genre>/` with its credit beside it, and
/// returns the track's path (spec S7):
///
/// 1. the genre folder, made if need be;
/// 2. the file, into memory, then `.<id>.part` — hidden, and with no track
///    extension, so `music::tracks` never lists it;
/// 3. **decoded whole**, which names its container and must come within 2 s
///    of the catalogue's length;
/// 4. the credit, through a temporary name and a rename;
/// 5. **the rename that makes the track exist, last** — so any track the
///    picker can choose already has its credit. If it fails, the credit just
///    written goes too.
///
/// **Every failure deletes the `.part`**, a cancel included. A crash between 4
/// and 5 leaves an orphan credit, which does no harm and a re-Add overwrites.
pub fn add(
    candidate: &Candidate,
    folder: &Path,
    genre: &Genre,
    cancel: &AtomicBool,
) -> Result<PathBuf, String> {
    let dir = folder.join(&genre.name);
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("could not create {}: {e}", dir.display()))?;
    // The id is the catalogue's: cleaned like any other name before it
    // becomes one, so it cannot name a path outside the genre folder.
    let id = naming::safe_chars(&candidate.id);
    let part = dir.join(format!(".{id}.part"));
    let landed = fetch_check_and_land(candidate, folder, genre, &id, &part, cancel);
    if landed.is_err() {
        let _ = std::fs::remove_file(&part);
    }
    landed
}

fn fetch_check_and_land(
    candidate: &Candidate,
    folder: &Path,
    genre: &Genre,
    id: &str,
    part: &Path,
    cancel: &AtomicBool,
) -> Result<PathBuf, String> {
    let dir = folder.join(&genre.name);
    let url = &candidate.url;
    let bytes = get(url, TRACK_LIMIT, cancel).map_err(|e| match e {
        GetError::Status(code) => format!("the track's server answered HTTP {code}"),
        GetError::Unreachable(_) => e.to_string(),
        GetError::TooLarge(_) => TOO_LARGE.to_owned(),
        GetError::Failed(_) => "the track's download broke off".to_owned(),
        GetError::Cancelled => "Cancelled".to_owned(),
    })?;
    std::fs::write(part, bytes).map_err(|e| format!("could not write {}: {e}", part.display()))?;
    let file = inspect_audio(part, cancel).map_err(|e| match e {
        CompositeError::Cancelled => "Cancelled".to_owned(),
        CompositeError::Failed(why) => {
            eprintln!("music: {url}: {why}");
            NOT_A_TRACK.to_owned()
        }
    })?;
    if (file.seconds - candidate.seconds).abs() > LENGTH_SLACK_SECONDS {
        eprintln!(
            "music: {url} decodes to {:.1} s, the catalogue says {:.1} s",
            file.seconds, candidate.seconds
        );
        return Err(NOT_A_TRACK.to_owned());
    }
    // **Never over another song**: two rows can share a stem, and the rename
    // below would replace the first one's file — and its credit just before
    // it. The same stem is the same song as far as the folder is concerned.
    let stem = stem(candidate);
    if already_in(folder, genre, &stem) {
        return Err(format!("\"{stem}\" is already in {}", genre.name));
    }
    let dest = dir.join(format!("{stem}.{}", file.extension));
    let credit = credit_beside(&dest);
    let credit_part = dir.join(format!(".{id}.credit.part"));
    std::fs::write(&credit_part, credit_text(candidate))
        .and_then(|()| std::fs::rename(&credit_part, &credit))
        .map_err(|e| {
            let _ = std::fs::remove_file(&credit_part);
            format!("could not write the credit {}: {e}", credit.display())
        })?;
    std::fs::rename(part, &dest).map_err(|e| {
        let _ = std::fs::remove_file(&credit);
        format!("could not move the track into {}: {e}", dir.display())
    })?;
    Ok(dest)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use pundit_media::fixtures::{audio_file, serve, Answer, AudioEncoder};

    use super::*;

    /// The core crate's page, trimmed from one real answer to `q=rock`: four
    /// rows a reel may use, three it may not. Its `url`s read `{AUDIO}`.
    const PAGE: &str = include_str!("../../pundit-core/tests/data/openverse_page.json");

    const BY_NONE: &str = "22a0cd91-ad66-4c36-ac70-c6d7a3b5c47d";
    const BY_ROCK: &str = "eb4a753d-b8d0-454c-a677-d838dcc2de1c";
    const CC0_POP: &str = "4bc3acc8-248b-429c-b707-9db9fd3965de";
    const PDM_ROCK: &str = "600f138f-0c8d-4299-995c-ba2ab98b4b04";

    /// The served track's length: every candidate's `seconds` is set to it
    /// (or, in the test that is about a disagreement, off it), because the
    /// page's real lengths are minutes.
    const SECONDS: u32 = 3;

    /// Two servers, as spec Testing says: one for the song, and one for the
    /// page whose `{AUDIO}` names it. Returns the page's base URL.
    fn servers(dir: &Path) -> String {
        gstreamer::init().unwrap();
        let song = audio_file(&dir.join("song.ogg"), AudioEncoder::Ogg, SECONDS);
        let song: Arc<[u8]> = std::fs::read(song).unwrap().into();
        let audio = serve(Answer::Body(song), Duration::ZERO);
        let page = PAGE.replace("{AUDIO}", &audio);
        let page_url = serve(Answer::Body(page.into_bytes().into()), Duration::ZERO);
        // `serve` answers every path with its one body, so the base is the
        // server's root.
        let (root, _) = page_url.rsplit_once('/').unwrap();
        format!("{root}/")
    }

    fn ids(candidates: &[Candidate]) -> Vec<&str> {
        candidates.iter().map(|c| c.id.as_str()).collect()
    }

    fn rock(folder: &Path) -> Genre {
        Genre::resolve(folder, "rock").unwrap()
    }

    /// The `Free (rock instrumental)` row, at the served track's length.
    fn by_rock(base: &str, folder: &Path) -> Candidate {
        let rows = search(base, &rock(folder)).unwrap();
        Candidate {
            seconds: f64::from(SECONDS),
            ..rows.into_iter().find(|c| c.id == BY_ROCK).unwrap()
        }
    }

    fn no_part_in(dir: &Path) {
        let parts: Vec<_> = std::fs::read_dir(dir)
            .unwrap()
            .flatten()
            .map(|e| e.file_name())
            .filter(|n| n.to_string_lossy().ends_with(".part"))
            .collect();
        assert!(parts.is_empty(), "left behind: {parts:?}");
    }

    #[test]
    fn search_returns_the_ranked_rows() {
        let dir = tempfile::tempdir().unwrap();
        let base = servers(dir.path());
        let rows = search(&base, &rock(dir.path())).unwrap();
        assert_eq!(ids(&rows), [BY_ROCK, PDM_ROCK, BY_NONE, CC0_POP]);
    }

    #[test]
    fn a_429_says_the_anonymous_limit() {
        gstreamer::init().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let base = serve(Answer::Status("429 Too Many Requests"), Duration::ZERO);
        assert_eq!(
            search(&base, &rock(dir.path())),
            Err(RATE_LIMITED.to_owned())
        );
    }

    /// From a music folder that does not exist yet, as on a first Add: the
    /// track and its credit land under the genre, and nothing else does.
    #[test]
    fn add_lands_the_track_and_its_credit() {
        let dir = tempfile::tempdir().unwrap();
        let base = servers(dir.path());
        let folder = dir.path().join("Music").join("pundit");
        let candidate = by_rock(&base, &folder);
        let track = add(&candidate, &folder, &rock(&folder), &AtomicBool::new(false)).unwrap();
        let genre_dir = folder.join("rock");
        assert_eq!(
            track,
            genre_dir.join("Free (rock instrumental) — Starfish Stories -- The Band.ogg")
        );
        assert!(track.is_file());
        assert_eq!(
            std::fs::read_to_string(credit_beside(&track)).unwrap(),
            credit_text(&candidate)
        );
        no_part_in(&genre_dir);
        assert_eq!(crate::music::tracks(&folder, "rock"), [track]);
    }

    /// The rename that makes the track exist fails for real — a directory is
    /// in the way — and the credit written just before it goes too.
    #[test]
    fn a_failed_rename_leaves_no_credit() {
        let dir = tempfile::tempdir().unwrap();
        let base = servers(dir.path());
        let folder = dir.path().join("music");
        let candidate = by_rock(&base, &folder);
        let dest = folder
            .join("rock")
            .join(format!("{}.ogg", stem(&candidate)));
        std::fs::create_dir_all(dest.join("in the way")).unwrap();
        add(&candidate, &folder, &rock(&folder), &AtomicBool::new(false))
            .expect_err("a directory holds the track's name");
        assert!(!credit_beside(&dest).exists(), "the credit was left");
        no_part_in(&folder.join("rock"));
    }

    #[test]
    fn a_track_whose_length_disagrees_is_refused_and_leaves_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let base = servers(dir.path());
        let folder = dir.path().join("music");
        let candidate = Candidate {
            seconds: f64::from(SECONDS) + 10.0,
            ..by_rock(&base, &folder)
        };
        assert_eq!(
            add(&candidate, &folder, &rock(&folder), &AtomicBool::new(false)),
            Err(NOT_A_TRACK.to_owned())
        );
        let left: Vec<_> = std::fs::read_dir(folder.join("rock"))
            .unwrap()
            .flatten()
            .map(|e| e.file_name())
            .collect();
        assert!(left.is_empty(), "left behind: {left:?}");
    }

    /// A song deleted from the folder is not "in rock" any more, and adding it
    /// again overwrites the credit its deletion orphaned.
    #[test]
    fn a_deleted_track_can_be_added_again() {
        let dir = tempfile::tempdir().unwrap();
        let base = servers(dir.path());
        let folder = dir.path().join("music");
        let genre = rock(&folder);
        let candidate = by_rock(&base, &folder);
        let stem = stem(&candidate);
        assert!(!already_in(&folder, &genre, &stem));
        let track = add(&candidate, &folder, &genre, &AtomicBool::new(false)).unwrap();
        std::fs::remove_file(&track).unwrap();
        std::fs::write(credit_beside(&track), "an orphaned credit\n").unwrap();
        assert!(!already_in(&folder, &genre, &stem));

        add(&candidate, &folder, &genre, &AtomicBool::new(false)).unwrap();
        assert!(already_in(&folder, &genre, &stem));
        assert_eq!(
            std::fs::read_to_string(credit_beside(&track)).unwrap(),
            credit_text(&candidate)
        );
    }

    #[test]
    fn a_genre_matches_an_existing_folder_by_slug() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("Rock")).unwrap();
        assert_eq!(
            Genre::resolve(dir.path(), " ROCK").unwrap(),
            Genre {
                name: "Rock".into(),
                query: "ROCK".into()
            }
        );
    }

    /// A hidden folder is no genre, as `music::genres` never lists one — and
    /// its slug drops the dot, so without the rule `.rock/` would be "rock".
    #[test]
    fn a_hidden_folder_is_not_a_genre() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".rock")).unwrap();
        assert_eq!(Genre::resolve(dir.path(), "rock").unwrap().name, "rock");
    }

    /// **Two rows with one stem are one song to the folder**, and the second
    /// Add is refused rather than renamed over the first — whose file and
    /// credit are left exactly as they were.
    #[test]
    fn a_second_song_with_the_same_stem_is_refused_and_the_first_is_kept() {
        let dir = tempfile::tempdir().unwrap();
        let base = servers(dir.path());
        let folder = dir.path().join("music");
        let genre = rock(&folder);
        let first = by_rock(&base, &folder);
        let second = Candidate {
            id: "another-id".into(),
            landing_url: Some("https://example.org/another-song".into()),
            ..first.clone()
        };
        assert_eq!(stem(&first), stem(&second));
        let track = add(&first, &folder, &genre, &AtomicBool::new(false)).unwrap();
        let bytes = std::fs::read(&track).unwrap();

        let refused = add(&second, &folder, &genre, &AtomicBool::new(false))
            .expect_err("a second song under the first one's name");
        assert!(refused.contains("already in rock"), "{refused}");
        assert_eq!(std::fs::read(&track).unwrap(), bytes);
        assert_eq!(
            std::fs::read_to_string(credit_beside(&track)).unwrap(),
            credit_text(&first)
        );
        no_part_in(&folder.join("rock"));
    }

    #[test]
    fn an_empty_genre_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            Genre::resolve(dir.path(), "  "),
            Err("Type a genre to search for".to_owned())
        );
    }
}

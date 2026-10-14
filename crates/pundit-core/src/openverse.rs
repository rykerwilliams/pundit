//! A goals reel's music search (BACKLOG #84 phase 2, spec S4): the request to
//! Openverse's `/v1/audio/`, and its answer read back as rows the coach may add.
//!
//! **Pure, like the rest of this crate**: no network and no URL crate. The app
//! fetches the string [`search_url`] builds and hands the body to [`results`].
//! Openverse names no genre field (`q` matches titles, spec §O6), so the genre
//! is the query text and [`rank`] puts the rows whose `genres` carry it first.
//!
//! **The licence is checked twice** (spec L2): the query names [`LICENCES`], and
//! [`results`] drops every row whose own `license` is not one of them. The
//! server's filter is the server's promise; what the sheet shows is the app's.

use serde::Deserialize;

use crate::naming;

/// The licences a reel's music may carry: CC0, the public-domain mark and
/// CC BY (spec L1). **The one list**, read by the query and by the re-check.
///
/// - **Synchronisation is an adaptation** under every Creative Commons licence
///   (BY-SA 4.0 §1(a): *"Adapted Material is always produced"* when music is
///   *"synched in timed relation with a moving image"*), so **ND is out by the
///   licence text** — a reel is exactly what ND forbids.
/// - **NC is out by policy**: a channel that is or becomes monetised would turn
///   the reel into a breach after the fact.
/// - **BY-SA is out by the coach's decision (2026-10-13)**: ShareAlike would
///   make the reel itself CC BY-SA — footage of identifiable players, released
///   for anyone to reuse commercially. That is the club's call, not a default.
pub const LICENCES: [&str; 3] = ["cc0", "pdm", "by"];

/// How many characters of the genre reach `q`, the API's own bound.
pub const MAX_QUERY_CHARS: usize = 200;

/// The anonymous page-size ceiling (spec §O3): above it Openverse answers `401`.
const PAGE_SIZE: u32 = 20;

/// What a row with no `creator` is credited to.
pub const UNKNOWN_ARTIST: &str = "Unknown artist";

/// The search for `genre` under `base` (`https://api.openverse.org/v1/`, with
/// its trailing slash): music of 2–10 minutes (`length=medium`, spec §O5) under
/// [`LICENCES`], one page of [`PAGE_SIZE`].
///
/// `q` is the trimmed text, cut to [`MAX_QUERY_CHARS`] characters and
/// percent-encoded. **Licences are named with `license`, never
/// `license_type`**, whose intersection lets the retired Sampling Plus through
/// (spec §O4).
pub fn search_url(base: &str, genre: &str) -> String {
    let q: String = genre.trim().chars().take(MAX_QUERY_CHARS).collect();
    format!(
        "{base}audio/?q={}&category=music&length=medium&license={}&mature=false&page_size={PAGE_SIZE}",
        encode(&q),
        LICENCES.join(","),
    )
}

/// RFC 3986's unreserved characters pass; every other byte of the UTF-8 is
/// `%XX`. Enough for one query value, which is all this crate builds.
fn encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for byte in text.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(char::from(byte));
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

/// One row the coach may add: what the sheet shows, what the credit says and
/// where the file is.
#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    /// Openverse's id, a UUID: the `.part`'s name and the sheet's row key.
    pub id: String,
    /// On one line ([`naming::one_line`]), never empty.
    pub title: String,
    /// On one line, or [`UNKNOWN_ARTIST`].
    pub creator: String,
    /// One of [`LICENCES`].
    pub license: String,
    pub license_version: Option<String>,
    /// As the catalogue gives it; [`licence_url`] builds one when it is absent.
    pub license_url: Option<String>,
    /// The track's page at its source, which the credit links.
    pub landing_url: Option<String>,
    /// The file.
    pub url: String,
    /// The catalogue's length, which an Add measures the decoded file against.
    pub seconds: f64,
    pub genres: Vec<String>,
}

/// Why a page could not be read at all. A row that can't be read is dropped,
/// never this.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct ParseError(String);

/// The fields of a result the app reads, and only those, so a field added
/// upstream costs nothing.
#[derive(Deserialize)]
struct Row {
    id: Option<String>,
    title: Option<String>,
    creator: Option<String>,
    license: Option<String>,
    license_version: Option<String>,
    license_url: Option<String>,
    foreign_landing_url: Option<String>,
    url: Option<String>,
    /// Milliseconds.
    duration: Option<f64>,
    genres: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct Page {
    results: Vec<serde_json::Value>,
}

/// The page's rows, in Openverse's order, that the coach may add.
///
/// A row is **dropped** — never shown greyed — when it has no `id`, no `url` or
/// no `duration` (an Add measures against it), a title that is blank on one
/// line, a licence outside [`LICENCES`], or a field of the wrong type. A
/// missing `creator` reads as [`UNKNOWN_ARTIST`].
pub fn results(json: &str) -> Result<Vec<Candidate>, ParseError> {
    let page: Page = serde_json::from_str(json).map_err(|e| ParseError(e.to_string()))?;
    Ok(page
        .results
        .into_iter()
        .filter_map(|value| candidate(serde_json::from_value(value).ok()?))
        .collect())
}

fn candidate(row: Row) -> Option<Candidate> {
    let license = row.license?;
    if !LICENCES.contains(&license.as_str()) {
        return None;
    }
    let title = naming::one_line(&row.title?);
    if title.is_empty() {
        return None;
    }
    let creator = row
        .creator
        .map(|c| naming::one_line(&c))
        .filter(|c| !c.is_empty())
        .unwrap_or_else(|| UNKNOWN_ARTIST.to_owned());
    Some(Candidate {
        id: row.id?,
        title,
        creator,
        license,
        license_version: row.license_version,
        license_url: row.license_url,
        landing_url: row.foreign_landing_url,
        url: row.url?,
        seconds: row.duration? / 1000.0,
        genres: row.genres.unwrap_or_default(),
    })
}

/// Rows whose `genres` include `genre` first, then the rest, **each group in
/// Openverse's order** — a stable sort, never a filter: a strict genre filter
/// keeps about a third of a page (spec §O6), and a rock song with no genres
/// listed is still a rock song.
pub fn rank(candidates: &mut [Candidate], genre: &str) {
    let genre = genre.trim().to_lowercase();
    candidates.sort_by_key(|c| !c.genres.iter().any(|g| g.to_lowercase() == genre));
}

/// A licence in words, for the sheet's row and the credit alike:
/// `CC BY 4.0`, `Public domain (CC0 1.0)`, `Public domain`.
pub fn licence_words(license: &str, version: Option<&str>) -> String {
    let versioned = |name: &str| match version {
        Some(v) => format!("{name} {v}"),
        None => name.to_owned(),
    };
    match license {
        "by" => versioned("CC BY"),
        "cc0" => format!("Public domain ({})", versioned("CC0")),
        "pdm" => "Public domain".to_owned(),
        other => versioned(&format!("CC {}", other.to_uppercase())),
    }
}

/// The licence's page: the catalogue's `license_url`, or — where that is
/// absent — one built from the licence and its version
/// (`creativecommons.org/licenses/by/<v>/`, and the `publicdomain/` forms for
/// CC0 and the mark).
pub fn licence_url(candidate: &Candidate) -> String {
    if let Some(url) = candidate.license_url.as_deref().filter(|u| !u.is_empty()) {
        return url.to_owned();
    }
    let version = candidate.license_version.as_deref();
    match candidate.license.as_str() {
        "cc0" => format!(
            "https://creativecommons.org/publicdomain/zero/{}/",
            version.unwrap_or("1.0")
        ),
        "pdm" => format!(
            "https://creativecommons.org/publicdomain/mark/{}/",
            version.unwrap_or("1.0")
        ),
        other => match version {
            Some(v) => format!("https://creativecommons.org/licenses/{other}/{v}/"),
            None => format!("https://creativecommons.org/licenses/{other}/"),
        },
    }
}

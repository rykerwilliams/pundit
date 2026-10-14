//! The Openverse music search's pure half (BACKLOG #84 phase 2, spec S4, L2):
//! the request built, and the answer read back, against a page trimmed from a
//! real `/v1/audio/` answer. No test here reaches the network.

use pundit_core::openverse::{
    licence_url, licence_words, rank, results, search_url, Candidate, UNKNOWN_ARTIST,
};

/// Seven rows from one real answer to `q=rock` (2026-10-13). Four are allowed;
/// an NC row, a BY-SA row and a BY row with a null duration were slipped in by
/// hand. The `url`s read `{AUDIO}`, which the app's tests replace.
const PAGE: &str = include_str!("data/openverse_page.json");

const BY_NONE: &str = "22a0cd91-ad66-4c36-ac70-c6d7a3b5c47d";
const BY_ROCK: &str = "eb4a753d-b8d0-454c-a677-d838dcc2de1c";
const CC0_POP: &str = "4bc3acc8-248b-429c-b707-9db9fd3965de";
const PDM_ROCK: &str = "600f138f-0c8d-4299-995c-ba2ab98b4b04";

fn ids(candidates: &[Candidate]) -> Vec<&str> {
    candidates.iter().map(|c| c.id.as_str()).collect()
}

#[test]
fn the_query_names_the_licences_length_and_category() {
    assert_eq!(
        search_url("https://api.openverse.org/v1/", "rock"),
        "https://api.openverse.org/v1/audio/?q=rock&category=music&length=medium\
         &license=cc0,pdm,by&mature=false&page_size=20"
    );
}

#[test]
fn a_genre_is_percent_encoded_and_cut_at_200() {
    let url = search_url("http://x/", "  drum & bass ");
    assert!(
        url.starts_with("http://x/audio/?q=drum%20%26%20bass&category="),
        "{url}"
    );
    // Two bytes a character: cut by characters, then encoded whole.
    let url = search_url("http://x/", &"é".repeat(300));
    let q = url
        .strip_prefix("http://x/audio/?q=")
        .and_then(|rest| rest.split('&').next())
        .unwrap();
    assert_eq!(q, "%C3%A9".repeat(200));
}

#[test]
fn the_page_keeps_exactly_its_allowed_rows() {
    let rows = results(PAGE).unwrap();
    // Dropped, in page order: 3c6bb63f… is NC (by-nc), 8fe7ca76… is BY-SA, and
    // 9ac627bf… is BY with a null duration — an allowed licence, so only the
    // duration can drop it.
    assert_eq!(ids(&rows), [BY_NONE, BY_ROCK, CC0_POP, PDM_ROCK]);
    let by = &rows[1];
    assert_eq!(by.title, "Free (rock instrumental)");
    assert_eq!(by.creator, "Starfish Stories :: The Band");
    assert_eq!(by.url, "{AUDIO}");
    assert_eq!(by.seconds, 265.0);
    assert_eq!(by.genres, ["rock"]);
    assert_eq!(
        by.landing_url.as_deref(),
        Some("https://www.jamendo.com/track/810513")
    );
}

/// 2.3's four rows, in page order, built by hand so that only the ranking is
/// under test here: a parse that kept a dropped row fails 2.3 alone.
#[test]
fn genre_matches_rank_first_in_their_own_order() {
    let row = |id: &str, genres: &[&str]| Candidate {
        id: id.into(),
        genres: genres.iter().map(|g| (*g).to_owned()).collect(),
        ..candidate("by", Some("3.0"))
    };
    let mut rows = vec![
        row(BY_NONE, &[]),
        row(BY_ROCK, &["rock"]),
        row(CC0_POP, &["pop"]),
        row(PDM_ROCK, &["rock"]),
    ];
    rank(&mut rows, " Rock");
    assert_eq!(ids(&rows), [BY_ROCK, PDM_ROCK, BY_NONE, CC0_POP]);
}

#[test]
fn licence_words_for_by_cc0_and_pdm() {
    assert_eq!(licence_words("by", Some("4.0")), "CC BY 4.0");
    assert_eq!(licence_words("cc0", Some("1.0")), "Public domain (CC0 1.0)");
    assert_eq!(licence_words("pdm", Some("1.0")), "Public domain");
}

fn candidate(license: &str, version: Option<&str>) -> Candidate {
    Candidate {
        id: "id".into(),
        title: "Song".into(),
        creator: "Artist".into(),
        license: license.into(),
        license_version: version.map(Into::into),
        license_url: None,
        landing_url: None,
        url: "http://x/song".into(),
        seconds: 200.0,
        genres: Vec::new(),
    }
}

#[test]
fn a_missing_licence_url_is_built_from_licence_and_version() {
    assert_eq!(
        licence_url(&candidate("by", Some("3.0"))),
        "https://creativecommons.org/licenses/by/3.0/"
    );
    assert_eq!(
        licence_url(&candidate("cc0", Some("1.0"))),
        "https://creativecommons.org/publicdomain/zero/1.0/"
    );
    assert_eq!(
        licence_url(&candidate("pdm", Some("1.0"))),
        "https://creativecommons.org/publicdomain/mark/1.0/"
    );
    // The catalogue's own, ported licence included, wins where there is one.
    let ported = Candidate {
        license_url: Some("https://creativecommons.org/licenses/by/2.5/it/".into()),
        ..candidate("by", Some("2.5"))
    };
    assert_eq!(
        licence_url(&ported),
        "https://creativecommons.org/licenses/by/2.5/it/"
    );
}

#[test]
fn a_title_with_a_newline_is_one_line_and_no_creator_is_unknown_artist() {
    let page = r#"{"results": [
        {"id": "a", "title": "Line one\n  line two ", "license": "by",
         "license_version": "4.0", "url": "http://x/a", "duration": 150000},
        {"id": "b", "title": " \n ", "license": "by", "url": "http://x/b",
         "duration": 150000}
    ]}"#;
    let rows = results(page).unwrap();
    // The second title is blank on one line, which drops the row.
    assert_eq!(ids(&rows), ["a"]);
    assert_eq!(rows[0].title, "Line one line two");
    assert_eq!(rows[0].creator, UNKNOWN_ARTIST);
}

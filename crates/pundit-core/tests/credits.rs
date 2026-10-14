//! A fetched track's credit (BACKLOG #84 spec C2).

use pundit_core::credits::credit_text;
use pundit_core::openverse::Candidate;

#[test]
fn a_by_credit_says_it_was_edited() {
    let candidate = Candidate {
        id: "id".into(),
        title: "I want you (pop rock remix)".into(),
        creator: "Lollita".into(),
        license: "by".into(),
        license_version: Some("3.0".into()),
        license_url: Some("https://creativecommons.org/licenses/by/3.0/".into()),
        landing_url: Some("https://www.jamendo.com/track/1015339".into()),
        url: "http://x/song".into(),
        seconds: 204.0,
        genres: Vec::new(),
    };
    assert_eq!(
        credit_text(&candidate),
        "Music (edited and mixed under the footage): \"I want you (pop rock remix)\" by Lollita\n\
         https://www.jamendo.com/track/1015339\n\
         Licensed under CC BY 3.0 — https://creativecommons.org/licenses/by/3.0/\n"
    );
}

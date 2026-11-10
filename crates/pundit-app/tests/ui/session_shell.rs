//! The session shell (#140 task 6): a session hides the match chrome, and the
//! one Record button's label follows the selection.
//!
//! **`is-match` is set directly on the window.** The bus-driven *meaning* of
//! Record in a session — capture a fresh clip, or review a selected one — is
//! task 5's and is proven in the harness; `tests/ui` cannot reach `main.rs`'s
//! event handling, so what it pins here is the window's own presentation:
//! which controls `is-match` gates, and which word the label follows.
//!
//! **What reaches the controls is the accessibility tree** (`export_sheet.rs`'s
//! pattern): a control inside an `if root.is-match` body is an element that
//! does not exist when the gate is false, so finding it by the words on it is
//! how "hidden" is asserted. The `#[test]`s share the process; each calls
//! `init_no_event_loop()` for itself (`tests/ui/main.rs`'s note).

use crate::{one_source, AppWindow};
use i_slint_backend_testing::ElementHandle;
use slint::ComponentHandle;

/// Every match control the session shell hides (spec S4), each named by the
/// words a coach reads on it — so this list **is** the §S4 enumeration, and the
/// gate cannot grow a control silently: a new one hidden by `is-match` adds a
/// line here, and one that should have been hidden but isn't fails below.
///
/// The two scoreboard rows live in the export sheet, so that sheet is opened
/// for this assertion too. The avatar/whiteboard sections, the Match panel's
/// buttons, the Slates `Type…` door and the reel are all in the window body.
const MATCH_CHROME: &[&str] = &[
    // The Match panel: its scoreboard setup and match-event editor.
    "Set up teams…",
    "Edit events…",
    // The Slates list's door (BACKLOG #97).
    "Type…",
    // The Devices popover's avatar inset and tactics whiteboard (needs the
    // popover shown; see the fixture).
    "Inset",
    "Whiteboard",
    // The scrubber's right-click *Band colour…* (#131, scrubber.slint): a
    // context-menu item, so like the popover controls it isn't in the headless
    // tree — listed for the enumeration, its match half skipped below.
    "Band colour…",
    // The export sheet's scoreboard picker and its subtitles switch (spec V2).
    "Scoreboard",
    "Scoreboard subtitles — an .srt beside the file, and a track inside a copy",
];

/// The window with a project open and a source, so every match control that
/// depends on footage would be shown in a match. The Devices popover and the
/// export sheet are forced open so their gated controls are reachable.
///
/// **The caller inits the backend**, not this — `init_no_event_loop()` installs
/// one backend per test thread and panics on a second call, and
/// `a_session_hides…` builds two windows (a session and a match) in one test.
fn shell(is_match: bool) -> AppWindow {
    let w = AppWindow::new().unwrap();
    w.set_has_project(true);
    w.set_can_play(true);
    one_source(&w);
    w.set_is_match(is_match);
    // The two scoreboard controls are in the export sheet; open it so they are
    // in the tree to be found (or found absent).
    w.set_export_sheet_open(true);
    w.show().unwrap();
    w
}

/// Whether any element reading exactly `words` is on screen.
fn present(w: &AppWindow, words: &str) -> bool {
    ElementHandle::find_by_accessible_label(w, words)
        .next()
        .is_some()
}

/// **The §S4 gate, counted in one assertion and in no prose.** With `is-match`
/// off, every control in `MATCH_CHROME` is absent; with it on, present. The
/// avatar and whiteboard sections live in a `PopupWindow` that a headless test
/// does not open, so they are asserted absent in the session (the gate's job)
/// and skipped for the match side — the popover's own contents are the Devices
/// test's concern, not this one's.
#[test]
fn a_session_hides_exactly_the_match_chrome() {
    // In-popup controls: present only once a popover or context menu is open,
    // which this headless fixture does not do — so the "present in a match" half
    // is not asserted for them. Their session half (absent) still is. "Band
    // colour…" is the scrubber's right-click item; the other two the Devices
    // popover's.
    let in_popover = ["Inset", "Whiteboard", "Band colour…"];

    i_slint_backend_testing::init_no_event_loop();
    let session = shell(false);
    for &words in MATCH_CHROME {
        assert!(
            !present(&session, words),
            "a session must hide the match control {words:?}"
        );
    }

    let game = shell(true);
    for &words in MATCH_CHROME {
        if in_popover.contains(&words) {
            continue;
        }
        assert!(
            present(&game, words),
            "a match must show {words:?} (the gate is too wide)"
        );
    }
}

/// The record button's accessible label, which a `Button` reports as its text.
/// There is one transport Record button, so exactly one of the two session
/// labels is on screen at a time.
fn record_label(w: &AppWindow) -> String {
    for label in ["Record clip", "Record", "Record range", "Stop"] {
        if present(w, label) {
            return label.to_owned();
        }
    }
    panic!("no record button on screen");
}

/// **Spec C4: one Record button, label follows the selection.** In a session
/// with nothing selected it reads "Record clip" (the free take); with a clip
/// selected it reads "Record" (review that clip). The capture-vs-review
/// *meaning* is the bus's (task 5) — this pins only the word.
#[test]
fn the_record_button_label_follows_selection_in_a_session() {
    i_slint_backend_testing::init_no_event_loop();
    let w = shell(false);
    // Not recording, and the export sheet is modal only over the scrim it
    // draws — the transport is still in the tree behind it. Close it so there
    // is no ambiguity about which "Record" is read.
    w.set_export_sheet_open(false);

    w.set_selected_clip(slint::SharedString::new());
    assert_eq!(
        record_label(&w),
        "Record clip",
        "nothing selected: the free take"
    );

    w.set_selected_clip("a-clip-id".into());
    assert_eq!(record_label(&w), "Record", "a clip selected: review it");
}

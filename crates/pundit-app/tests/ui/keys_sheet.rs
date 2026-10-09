//! **The Keys sheet, driven through the real sheet** (BACKLOG #96, plan task
//! 4): `F1` and the button open it, Esc closes it, and the 29 rows on screen
//! are the ones `keymap.rs` holds.
//!
//! **What this closes.** The sheet is a list of a Rust table, and the two ways
//! a list of a table goes wrong are both invisible to a window property: the
//! model can fail to reach the sheet (`rows: root.key-rows` left off, which the
//! plan named as the sabotage it expected to be unreachable) and the columns
//! can cross on the way in. So the rows are read back off the **accessibility
//! tree** — `KeyLine` carries one label for the whole row, three cells in
//! order — and compared with `Keymap::listing()`.
//!
//! **Two passes and a scroll, because the sheet scrolls.** The row list is
//! capped at 520px so the card fits the window's own 700px floor, and
//! `ElementHandle`'s walk skips any subtree the enclosing clip puts off screen
//! — so about two thirds of the rows are unreachable until the list is
//! scrolled, which is what a coach does too. Every assertion over the rows
//! therefore reads them in passes with `ElementHandle::scroll` between.
//!
//! **A handful of rows are spelled out here**, as `export_sheet.rs` spells out
//! its switches' words: a comparison that derives both sides from
//! `keymap.rs` cannot see a label change, so the test holds its own half of the
//! contract for the five rows that carry a shape — a named key, two bindings
//! with a modifier, this sheet's own key, an action bound to nothing, and one
//! with a `when` that is not "Any time".

use i_slint_backend_testing::{ElementHandle, ElementQuery};
use pundit_app::keymap::{Action, Keymap};
use slint::platform::{Key, WindowEvent};
use slint::{ComponentHandle, LogicalSize, Model, SharedString};
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;

use crate::key_action;
use crate::AppWindow;

/// The words on the button, which is also how it is found — the test's half of
/// a contract with `app.slint`.
const BUTTON: &str = "Keys";

/// Five rows spelled out: `(what, keys, when)`. Each carries a shape the sheet
/// has to get right rather than a row picked for being easy — a named key, a
/// pair of bindings with a modifier, the key that opens this sheet, an action
/// with **no** binding, and a `when` that is not "Any time".
const SPELLED_OUT: [(&str, &str, &str); 5] = [
    ("Play or pause", "Space", "Any time"),
    (
        "Skip back ten seconds",
        "shift+LeftArrow, shift+a",
        "Any time",
    ),
    ("Show the keys", "F1", "Any time"),
    ("Undo", "ctrl+z", "Any time, unless a recording is running"),
    // The one action that ships bound to nothing (#85's deferral, which #96
    // closes at the cost of this row): the sheet must say so rather than show
    // an empty cell.
    ("Open the recent projects list", "—", "Any time"),
];

/// The six rows the sheet writes itself, because they are not actions (spec
/// G4): `(what, keys, when)`, in the order they are shown.
const RESERVED: [(&str, &str, &str); 6] = [
    (
        "Back out",
        "Escape",
        "Whatever is innermost: a sheet, the tool, a take, the selection",
    ),
    ("Move the focus on", "Tab", "Any time"),
    ("Move the focus back", "Shift+Tab", "Any time"),
    (
        "Confirm the message",
        "Return",
        "While a message is on screen",
    ),
    (
        "Nothing",
        "Home",
        "Swallowed, so a touched slider cannot take it",
    ),
    (
        "Nothing",
        "End",
        "Swallowed, so a touched slider cannot take it",
    ),
];

/// How a row reads in the accessibility tree: `KeyLine`'s one label, its three
/// cells in the order the columns are in.
fn row_label(what: &str, keys: &str, when: &str) -> String {
    format!("{what} — {keys} — {when}")
}

/// The window with the production key wiring, which is what fills `key-rows`:
/// a copy made here would test the copy. The recorder catches any shortcut that
/// fires, which is how "the sheet swallows everything else" is checked.
fn window() -> (AppWindow, Rc<RefCell<Vec<String>>>) {
    i_slint_backend_testing::init_no_event_loop();
    let w = AppWindow::new().unwrap();
    key_action::wire_keys(&w, Keymap::defaults());

    let fired: Rc<RefCell<Vec<String>>> = Rc::default();
    w.on_toggle_play({
        let fired = Rc::clone(&fired);
        move || fired.borrow_mut().push("toggle-play".to_owned())
    });
    w.on_tag_match_event({
        let fired = Rc::clone(&fired);
        move |_| fired.borrow_mut().push("tag".to_owned())
    });
    // `toggle-play`'s gate, so Space is a live shortcut: the one key this
    // file presses to prove the sheet swallows what is behind it, and then
    // that it lets go. (`can-tag` and `can-draw` are `out` properties with no
    // setter, so the tag keys cannot be made live from here.)
    w.set_can_play(true);
    w.show().unwrap();
    (w, fired)
}

/// A press and its release, as a keyboard delivers them.
fn press(w: &AppWindow, text: impl Into<SharedString>) {
    let text: SharedString = text.into();
    w.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    w.window().dispatch_event(WindowEvent::KeyReleased { text });
}

/// The window with the sheet open and **big enough to read**. The card is
/// 840x666, and `ElementHandle` cannot reach a subtree the
/// window's own edge clips — at the headless default every row would be
/// unfindable and every assertion below vacuous.
fn open_sheet() -> AppWindow {
    let (w, _) = window();
    w.window().set_size(LogicalSize::new(1920.0, 1200.0));
    w.set_keys_sheet_open(true);
    w
}

/// Every row label the accessibility tree can reach right now, read in passes
/// with a scroll between: the list is capped at 520px, so the rows below it are
/// clipped out of the walk until it is scrolled.
///
/// **The scroll is dispatched on a row**, which is where a coach's wheel is,
/// and the handle is taken before the scroll so the event lands inside the
/// list either way.
fn row_labels(w: &AppWindow) -> BTreeSet<String> {
    let mut seen = BTreeSet::new();
    for pass in 0..6 {
        let lines = ElementQuery::from_root(w)
            .match_type_name("KeyLine")
            .find_all();
        assert!(
            !lines.is_empty(),
            "pass {pass} found no row of the sheet at all"
        );
        for line in &lines {
            if let Some(label) = line.accessible_label() {
                seen.insert(label.to_string());
            }
        }
        // The middle row rather than the first: `scroll` dispatches the wheel
        // at the element's centre, and the top row's centre can be above the
        // list once it is half scrolled out, where the event would miss it.
        lines[lines.len() / 2].scroll(0.0, -200.0);
    }
    seen
}

/// **`F1` opens it and Esc closes it**, and nothing else gets through while it
/// is up.
///
/// `keys.rs`'s census has `showKeys`' own row, so that the key reaches the
/// action is pinned there with the other 33 defaults; what this adds is the
/// half that is this sheet's — Esc closing it, and the modal layer swallowing
/// the keys behind it.
#[test]
fn f1_opens_the_keys_sheet_and_esc_closes_it() {
    let (w, fired) = window();
    assert!(!w.get_keys_sheet_open(), "the sheet starts open");

    press(&w, Key::F1);
    assert!(w.get_keys_sheet_open(), "F1 did not open the Keys sheet");

    // Modal over the window's shortcuts, the export sheet's shape: a key that
    // would otherwise play does nothing behind it.
    press(&w, " ");
    assert!(
        fired.borrow().is_empty(),
        "the sheet let a shortcut through: {:?}",
        fired.borrow()
    );

    press(&w, Key::Escape);
    assert!(!w.get_keys_sheet_open(), "Esc did not close the Keys sheet");

    // And the window is live again, which is what says the layer let go
    // rather than the property merely flipping.
    press(&w, " ");
    assert_eq!(
        *fired.borrow(),
        ["toggle-play".to_owned()],
        "the window stayed modal after the sheet closed"
    );
}

/// **The button opens it too**, which is the door for the coach who does not
/// know `F1` — and the only reason a key list is discoverable at all.
///
/// Uniqueness is asserted rather than assumed: the open sheet's title and its
/// middle column heading both read "Keys", so this has to be done with the
/// sheet shut.
#[test]
fn the_keys_button_opens_the_sheet() {
    let (w, _) = window();
    // The transport rows have to be on screen to be found: the walk skips a
    // subtree the window's edge clips.
    w.window().set_size(LogicalSize::new(1920.0, 1200.0));

    let mut found = ElementHandle::find_by_accessible_label(&w, BUTTON);
    let button = found.next().expect("no control reading \"Keys\"");
    assert!(
        found.next().is_none(),
        "\"Keys\" matches more than one element with the sheet shut"
    );
    assert_eq!(
        button.accessible_enabled(),
        Some(true),
        "the Keys button is greyed out"
    );

    button.invoke_accessible_default_action();
    assert!(
        w.get_keys_sheet_open(),
        "the Keys button did not open the sheet"
    );
}

/// **A row for every action, in `Action::ALL`'s order, with nothing blank** —
/// the cheap pin for "the list cannot go stale", read off the window property
/// `wire_keys` sets.
///
/// `Keymap::listing()` is on both sides of the order check, so what that half
/// catches is a model built any way but from the table: a count that drifts, a
/// sort, a filter. What it cannot catch is a label changing, which is what
/// [`SPELLED_OUT`] is for.
#[test]
fn the_listing_has_a_row_for_every_action() {
    let (w, _) = window();
    let rows = w.get_key_rows();
    let listing = Keymap::defaults().listing();

    assert_eq!(
        rows.row_count(),
        Action::ALL.len(),
        "the sheet's rows and the table's actions have drifted apart"
    );
    for (i, want) in listing.iter().enumerate() {
        let got = rows.row_data(i).expect("a row the count promised");
        assert_eq!(
            (got.what.as_str(), got.keys.as_str(), got.fires.as_str()),
            (want.what, want.keys.as_str(), want.when),
            "row {i} is not the table's row {i}"
        );
        assert!(
            !got.what.is_empty() && !got.keys.is_empty() && !got.fires.is_empty(),
            "row {i} has an empty cell: {got:?}"
        );
    }

    for (what, keys, when) in SPELLED_OUT {
        let row = listing
            .iter()
            .position(|r| r.what == what)
            .map(|i| rows.row_data(i).expect("a row the count promised"))
            .unwrap_or_else(|| panic!("no row reading {what:?}"));
        assert_eq!(
            (row.keys.as_str(), row.fires.as_str()),
            (keys, when),
            "the row for {what:?} does not read as it is supposed to"
        );
    }
}

/// **Every one of those rows is on screen, in its own three columns.** The
/// assertion the window property cannot make: `rows: root.key-rows` missing
/// from the sheet's instantiation opens an empty sheet and leaves
/// `the_listing_has_a_row_for_every_action` green, which is the gap the plan
/// expected to have to carry on the manual list.
///
/// A crossed column is caught by the same read: the label is the three cells
/// joined in order, so `keys` and `when` swapped is 29 failures.
#[test]
fn every_row_the_table_holds_is_on_screen() {
    let w = open_sheet();
    let seen = row_labels(&w);

    for row in Keymap::defaults().listing() {
        let want = row_label(row.what, &row.keys, row.when);
        assert!(
            seen.contains(&want),
            "no row on screen reads {want:?}; the rows found were {seen:#?}"
        );
    }
}

/// **The six reserved keys have a row each** (spec G4). They are not
/// `keymap::Action` variants — Escape is a key with a position rather than a
/// job, Tab and Return are the platform's, Home and End do nothing — so
/// `listing()` has no row for them and the sheet writes them itself. This is
/// what says it still does.
///
/// They sit under the actions, so reaching them is the whole reason
/// [`row_labels`] scrolls.
#[test]
fn the_reserved_keys_have_a_row_each() {
    let w = open_sheet();
    let seen = row_labels(&w);

    for (what, keys, when) in RESERVED {
        let want = row_label(what, keys, when);
        assert!(
            seen.contains(&want),
            "no row on screen reads {want:?}, so the reserved keys are not listed"
        );
    }
    // And they are the sheet's own rows rather than the table's, which is what
    // keeps `Binding`, `overrides()` and the rebind free of six rows they
    // would each have to filter back out.
    for (_, keys, _) in RESERVED {
        assert!(
            !Keymap::defaults()
                .listing()
                .iter()
                .any(|r| r.keys.split(", ").any(|k| k == keys)),
            "{keys:?} is in the rebindable table too"
        );
    }
}

/// **The card fits the smallest window the app opens** — the one thing a
/// 29-row table could have cost. `Sheet` is `height: body.preferred-height`
/// and **does not scroll** (BACKLOG #134), so a card taller than the window is
/// a card with its Close button off the bottom of it; this sheet answers that
/// by capping its row list, and this is the measurement of the cap.
///
/// Measured at the 1100x700 floor: **666px** of the 700 there are, which is
/// what the 520px cap on the row list buys.
#[test]
fn the_card_fits_the_smallest_window_it_opens_in() {
    let w = open_sheet();
    let (min_w, min_h) = (w.get_min_window_width(), w.get_min_window_height());
    w.window().set_size(LogicalSize::new(min_w, min_h));

    let height = ElementHandle::find_by_element_type_name(&w, "KeysSheet")
        .next()
        .expect("the open sheet's own element")
        .size()
        .height;

    assert!(
        height <= min_h,
        "the Keys sheet is {height}px tall and the smallest window this app \
         opens is {min_h}px: Close is off the bottom of it"
    );
}

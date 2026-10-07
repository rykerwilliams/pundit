//! **A letter typed into a text field must reach the field, not a shortcut**
//! — in every field the window has, and the first automated cover that rule
//! has ever had (BACKLOG #96 task 1, the class of #130).
//!
//! The window gates its letter shortcuts on `text-editing`, which until this
//! change was a seven-term disjunction of every field's focus that each new
//! field had to be added to. #130 is what happens when one is not: the slate
//! tag filter shipped as a bare `LineEdit` nobody joined up, so `i` and `o`
//! marked slates instead of typing, and `corner` and `possession` were
//! untypeable — found by the coach on first real use. Nothing could have
//! caught it, because no test in the repo typed into a field and looked at
//! what fired.
//!
//! `text-editing` is now Slint's own
//! `TextInputInterface.text-input-focused`, which `TextInput` sets on
//! focus-in and clears on focus-out *and* in its `deinit`. There is no list
//! left to join.
//!
//! **Between them these tests type into sixteen fields, which is every one of
//! the thirteen `LineEdit` / `TextEdit` / `TagField` sites in `ui/app.slint`.**
//! The match setup sheet's twelve `SetupField`s are the one group never folded
//! in even before this change — layer 5 of `handle-key` returns `reject` for
//! every non-Esc key ahead of the shortcuts, so they were already protected —
//! and the `SetupField::edit` they share is reached here through the New match
//! sheet's four.
//!
//! **Each field is driven by a click where a coach would click it**, not by a
//! `focus()` helper: what is under test includes the field being reachable at
//! all in the state the window is in, and several of these live inside an `if`
//! or behind a modal scrim.

use crate::{AppWindow, HighlightRow, MatchEditorRow, SlateRow};
use i_slint_backend_testing::ElementHandle;
use slint::platform::{Key, PointerEventButton, WindowEvent};
use slint::{ComponentHandle, ModelRc, PhysicalSize, SharedString, VecModel};
use std::cell::RefCell;
use std::rc::Rc;

/// The four keys dispatched into every field. Each is a bare-letter shortcut
/// with a recorded callback — `r` records, `i` and `o` mark a slate's in and
/// out, Space plays — so a key that misses the field has somewhere visible to
/// land. They also spell themselves into the field, which is the other half of
/// the assertion: the letters must *arrive*, not merely fail to fire.
const PROBE: [&str; 4] = ["r", "i", "o", " "];

/// What `PROBE` leaves in a field it reached.
const TYPED: &str = "rio ";

/// One field: how a test finds it, and the window property its text is bound
/// to through however many `<=>` wires. Paired here so a field that swallows
/// the letters and a field that lets them through to the shortcuts are two
/// different failures.
struct Field {
    /// The qualified element id, as `find_by_element_id` spells it.
    id: &'static str,
    /// The field's text, read at the window — the far end of the wire.
    text: fn(&AppWindow) -> SharedString,
}

/// Every text field reachable with no sheet open. The clip inspector's four,
/// the slate panel's three (the filter among them — #130's field) and the
/// project name.
fn window_fields() -> Vec<Field> {
    vec![
        Field {
            id: "AppWindow::name-edit",
            text: AppWindow::get_project_name,
        },
        Field {
            id: "Inspector::name",
            text: AppWindow::get_clip_name,
        },
        Field {
            id: "Inspector::tags",
            text: AppWindow::get_clip_tags,
        },
        Field {
            id: "Inspector::notes",
            text: AppWindow::get_clip_notes,
        },
        Field {
            id: "Inspector::transcript",
            text: AppWindow::get_clip_transcript,
        },
        Field {
            id: "HighlightsPanel::label-edit",
            text: AppWindow::get_highlight_label,
        },
        Field {
            id: "AppWindow::slate-name-edit",
            text: AppWindow::get_slate_name,
        },
        Field {
            id: "AppWindow::slate-tags-edit",
            text: AppWindow::get_slate_tags,
        },
        Field {
            id: "AppWindow::slate-filter-edit",
            text: AppWindow::get_slate_tag_filter,
        },
    ]
}

fn slate(id: &str) -> SlateRow {
    SlateRow {
        id: id.into(),
        range: "0:05–0:12".into(),
        video: SharedString::new(),
        name: SharedString::new(),
        tags: SharedString::new(),
        shot: false,
        timed: true,
    }
}

/// The window with every field it has on screen at once, and every shortcut
/// those four keys reach recorded.
///
/// **`can-play` matters**: `i` and `o` are gated on `can-tag`, so without it
/// those two would fire nothing whatever the fold did and half the probe would
/// be vacuous. `has-project` is the project name field's `enabled`, and a
/// disabled `LineEdit` refuses focus outright.
fn window() -> (AppWindow, Rc<RefCell<Vec<&'static str>>>) {
    i_slint_backend_testing::init_no_event_loop();
    let w = AppWindow::new().unwrap();
    let fired: Rc<RefCell<Vec<&'static str>>> = Rc::default();
    let record = |what: &'static str| {
        let fired = Rc::clone(&fired);
        move || fired.borrow_mut().push(what)
    };
    w.on_mark_in(record("mark-in"));
    w.on_mark_out(record("mark-out"));
    w.on_toggle_play(record("toggle-play"));
    w.on_toggle_recording({
        let fired = Rc::clone(&fired);
        move |_slate| fired.borrow_mut().push("toggle-recording")
    });
    // The tag picker's two, so typing into either `TagField` reads a real
    // vocabulary rather than an unset callback's default.
    w.on_suggest_tags(|_text| ModelRc::new(VecModel::<SharedString>::default()));
    w.on_take_suggestion(|text, _tag| text);

    w.set_can_play(true);
    w.set_has_project(true);
    // The inspector is shown by a selected clip, the highlight's label field
    // by a selected ring, the slate fields by a selected range — and the
    // filter needs a list for its section's `if` to hold.
    w.set_selected_clip("c1".into());
    w.set_highlight_rows(ModelRc::new(VecModel::from(vec![HighlightRow {
        id: "h1".into(),
        ink: slint::Color::from_rgb_u8(0xff, 0x00, 0x00),
        label: SharedString::new(),
    }])));
    w.set_selected_highlight("h1".into());
    w.set_slates(ModelRc::new(VecModel::from(vec![slate("s1")])));
    w.set_selected_slate("s1".into());
    w.show().unwrap();
    // **A tall window, and it is not decoration.** `ElementHandle` skips a
    // subtree its geometry puts outside the enclosing clip, so a field
    // scrolled out of the inspector column cannot be found or clicked at all.
    // At the headless default the column ends above the highlights panel.
    w.window().set_size(PhysicalSize::new(1920, 2400));
    (w, fired)
}

fn press(w: &AppWindow, key: impl Into<SharedString> + Clone) {
    w.window().dispatch_event(WindowEvent::KeyPressed {
        text: key.clone().into(),
    });
    w.window()
        .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
}

/// The one element with this id, or a message naming it: a field that cannot
/// be found is as much a failure as one that lets a letter through, and `None`
/// unwrapped bare says nothing about which of the sixteen it was.
fn one(w: &AppWindow, id: &str) -> ElementHandle {
    let mut found = ElementHandle::find_by_element_id(w, id);
    let it = found
        .next()
        .unwrap_or_else(|| panic!("no element {id:?} on screen"));
    assert!(
        found.next().is_none(),
        "{id:?} matches more than one element in this window"
    );
    it
}

/// Clicks `field`, types `PROBE` into it, and asserts both halves: the letters
/// arrived, and nothing fired.
fn probe(w: &AppWindow, fired: &Rc<RefCell<Vec<&'static str>>>, field: &Field) {
    one(w, field.id).mock_single_click(PointerEventButton::Left);
    for key in PROBE {
        press(w, key);
    }
    // **Firing first**, because it is the more diagnostic of the two: a key
    // taken as a shortcut is a key that never arrived, so the second
    // assertion would fail too and say nothing about where the letter went.
    assert!(
        fired.borrow().is_empty(),
        "{}: typing fired {:?}",
        field.id,
        fired.borrow()
    );
    assert_eq!(
        (field.text)(w),
        TYPED,
        "{}: the letters never reached the field",
        field.id
    );
}

/// **The regression test #130 never had.** Every field in the window, each
/// typed into where it stands, and after each one: the text is in the field
/// and no shortcut fired.
///
/// **A new text field needs no entry here to be safe** — that is the point of
/// the change this pins, and the difference from the list it replaced. An
/// entry is still worth adding, because what this also checks is that the
/// field is *reachable* by a click in the state the window is in.
#[test]
fn a_letter_typed_in_a_window_field_fires_no_shortcut() {
    let (w, fired) = window();

    for field in window_fields() {
        probe(&w, &fired, &field);
        // Off the field, so the next click is a fresh focus-in and the
        // assertions above are each about their own field. The window's own
        // Esc cascade is what a coach uses, and it is tested below.
        press(&w, Key::Escape);
        // `name-edit` reverts to the saved name when focus leaves it, and the
        // match editor's row field commits; neither is this test's business.
        fired.borrow_mut().clear();
    }
}

/// The same for the three sheets that hold fields, one at a time: the basket's
/// name, the match-event editor's row field and paste box, and the New match
/// sheet's four.
///
/// **A sheet rejects every non-Esc key ahead of the shortcut layer**, so what
/// this adds over the test above is the other half — the letters reach the
/// field behind a modal scrim. A field a coach cannot type into is the failure
/// #130 actually was.
#[test]
fn a_letter_typed_in_a_sheet_field_reaches_the_field() {
    let (w, fired) = window();

    w.set_basket_sheet_open(true);
    probe(
        &w,
        &fired,
        &Field {
            id: "BasketSheet::basket-name-edit",
            text: AppWindow::get_basket_name,
        },
    );
    w.set_basket_sheet_open(false);

    w.set_match_editor_rows(ModelRc::new(VecModel::from(vec![MatchEditorRow {
        id: "e1".into(),
        where_text: "1 · 14:05.0".into(),
        label: "Home goal".into(),
        role_less: false,
    }])));
    w.set_match_editor_selected("e1".into());
    w.set_match_editor_open(true);
    probe(
        &w,
        &fired,
        &Field {
            id: "MatchEditorSheet::line-edit",
            text: AppWindow::get_match_editor_line,
        },
    );
    probe(
        &w,
        &fired,
        &Field {
            id: "MatchEditorSheet::paste-edit",
            text: AppWindow::get_match_editor_paste,
        },
    );
    w.set_match_editor_open(false);

    // The New match sheet's four are `SetupField`s, so the field is the
    // `edit` inside each — and the sheet's own ids are how they are told
    // apart. **Folder name last**: typing a team name re-derives it, so an
    // earlier probe of it would be overwritten by a later one of them.
    w.set_new_match_sheet_open(true);
    for (sheet_id, text) in [
        (
            "NewMatchSheet::home",
            AppWindow::get_new_match_home_name as fn(&AppWindow) -> SharedString,
        ),
        ("NewMatchSheet::away", AppWindow::get_new_match_away_name),
        ("NewMatchSheet::dir", AppWindow::get_new_match_projects_dir),
        (
            "NewMatchSheet::folder",
            AppWindow::get_new_match_folder_name,
        ),
    ] {
        let edit = one(&w, sheet_id)
            .query_descendants()
            .match_id("SetupField::edit")
            .find_first()
            .unwrap_or_else(|| panic!("no field inside {sheet_id:?}"));
        edit.mock_single_click(PointerEventButton::Left);
        for key in PROBE {
            press(&w, key);
        }
        assert!(
            fired.borrow().is_empty(),
            "{sheet_id}: typing fired {:?}",
            fired.borrow()
        );
        assert_eq!(text(&w), TYPED, "{sheet_id}: the letters never arrived");
    }
}

/// **Esc leaves the field before it closes the sheet** (spec C6, basket spec
/// C6). The basket sheet's Esc tests `!text-editing`, so with the fold wrong
/// in either direction this cascade breaks in a different way: a constant
/// `false` closes the sheet on the first press and throws away the name being
/// typed, and a constant `true` never closes it at all.
///
/// The second press is also what exercises `TextInput::deinit`: it destroys
/// the sheet with the field inside it, and nothing in `close-basket` clears
/// the flag by hand any more.
#[test]
fn esc_in_a_sheet_field_leaves_the_field_before_it_closes_the_sheet() {
    let (w, _fired) = window();
    w.set_basket_sheet_open(true);
    one(&w, "BasketSheet::basket-name-edit").mock_single_click(PointerEventButton::Left);
    press(&w, "r");
    assert_eq!(
        w.get_basket_name(),
        "r",
        "the letter never reached the field"
    );

    press(&w, Key::Escape);
    assert!(
        w.get_basket_sheet_open(),
        "the first Esc should leave the field, not close the sheet over a half-typed name"
    );

    press(&w, Key::Escape);
    assert!(
        !w.get_basket_sheet_open(),
        "the second Esc, with no field focused, should close the sheet"
    );
}

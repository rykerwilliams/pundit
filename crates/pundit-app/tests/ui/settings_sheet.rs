//! **The Settings sheet, and the colour picker in each sheet that hosts it**
//! (BACKLOG #131).
//!
//! The sheet's one row is the slate band's colour: a field, a Default button
//! and the picker inline. What it pins is the window's half of the contract —
//! a typed colour reaches the band and is reported to be stored, a half-typed
//! one never is and does not outlive the sheet, Default takes the colour back
//! — and that `Settings…` is gated where a modal must not open. What it cannot
//! pin is that the band and the slate row actually **draw** the colour: both
//! are a `Rectangle.background` with no accessible property, which is the
//! manual list's (plan, manual checks 2–5), as is `main.rs`'s `wire_settings`.
//!
//! **To replace the field's text it selects all and types.** Never Backspace:
//! that passes through `""`, which is valid — it would report the default and
//! make a "nothing reported" assertion fail for the wrong reason.
//!
//! `ColorPicker` holds its own position — hue, saturation and brightness — and
//! follows its host's text, so the grey-hue logic exists once. Each host gets
//! two tests: a kit swatch pressed **by its name** lands in the field, and a
//! click in the saturation/brightness area reads the hue the field's text
//! stands on. The second is the one that reaches the moved sync: a kit press
//! writes its hex as it stands and never touches the held position, so it
//! would pass with the sync dead.
//!
//! **Every query is scoped to the sheet's element, not to its card.** The
//! setup sheet's popup is the card's *sibling* (it hangs over the card), so a
//! query inside the `Sheet` never finds a swatch.
//!
//! **`settle` after anything that is not a pointer or key event.** A
//! `changed` handler runs on the next change-handler pass, not at the
//! assignment; the pointer and key paths run that pass on the way in, but an
//! `invoke_accessible_default_action` and a Rust `set_*` do not.

use crate::{color_wiring, key_action};
use crate::{AppWindow, ColorMaths, RecordingPhase};
use i_slint_backend_testing::ElementHandle;
use pundit_app::color_picker;
use pundit_app::keymap::Keymap;
use slint::platform::{Key, PointerEventButton, WindowEvent};
use slint::{Color, ComponentHandle, LogicalSize, SharedString};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

/// The words on the drawing row's button (U+2026), which is how it is found.
const BUTTON: &str = "Settings…";

/// Every value the window reported to be stored, in order.
type Edits = Rc<RefCell<Vec<String>>>;

/// The window with the app's own colour maths and keys wired, at the floor size it declares, and what `slate-band-edited` hears.
fn window() -> (AppWindow, Edits) {
    i_slint_backend_testing::init_no_event_loop();
    let w = AppWindow::new().unwrap();
    // With `action-for` unwired every key reads as no action, so an Esc that
    // fell through the sheet would prove nothing about the sheet.
    key_action::wire_keys(&w, &crate::scratch_state(), Keymap::defaults());
    let edits: Edits = Rc::default();
    w.on_slate_band_edited({
        let edits = Rc::clone(&edits);
        move |text| edits.borrow_mut().push(text.to_string())
    });
    color_wiring::wire_colors(&w);
    w.show().unwrap();
    let (min_w, min_h) = (w.get_min_window_width(), w.get_min_window_height());
    w.window().set_size(LogicalSize::new(min_w, min_h));
    (w, edits)
}

/// Runs the change-handler pass a pointer or key event would have run.
fn settle() {
    i_slint_backend_testing::mock_elapsed_time(Duration::ZERO);
}

/// The one element of this type on screen.
fn sheet(w: &AppWindow, type_name: &str) -> ElementHandle {
    ElementHandle::find_by_element_type_name(w, type_name)
        .next()
        .unwrap_or_else(|| panic!("no {type_name} on screen"))
}

/// The first descendant of `root` with this element id.
fn by_id(root: &ElementHandle, id: &str) -> ElementHandle {
    root.query_descendants()
        .match_id(id)
        .find_first()
        .unwrap_or_else(|| panic!("no {id} under the sheet"))
}

/// The descendant of `root` whose accessible label is `label`.
fn by_label(root: &ElementHandle, label: &str) -> ElementHandle {
    let wanted = label.to_string();
    root.query_descendants()
        .match_predicate(move |e| e.accessible_label().is_some_and(|l| l == wanted.as_str()))
        .find_first()
        .unwrap_or_else(|| panic!("nothing labelled {label:?} under the sheet"))
}

/// Opens the setup sheet and the picker over its first colour field, the
/// home primary — the first `SetupField` that has a swatch to click.
fn open_the_setup_picker(w: &AppWindow) -> ElementHandle {
    w.set_match_sheet_open(true);
    settle();
    let setup = sheet(w, "MatchSetupSheet");
    by_id(&setup, "SetupField::touch").mock_single_click(PointerEventButton::Left);
    settle();
    assert_eq!(w.get_match_picker_field(), 0, "the home primary's picker");
    setup
}

#[test]
fn a_kit_colour_lands_in_the_setup_field() {
    let (w, _) = window();
    let setup = open_the_setup_picker(&w);
    by_label(&setup, "White").invoke_accessible_default_action();
    settle();
    // `KIT_COLORS[0]`.
    assert_eq!(w.get_match_home_primary(), "#ffffff");
}

/// The picker is created when it opens, so this pins the sync that runs as it
/// is made: the strip has to stand on the field's blue before the click, or
/// the area reads red's half-tone instead (`#804040`).
#[test]
fn the_setup_pickers_area_reads_the_fields_hue() {
    let (w, _) = window();
    w.set_match_home_primary("#0000ff".into());
    let setup = open_the_setup_picker(&w);
    by_id(&setup, "ColorPicker::area").mock_single_click(PointerEventButton::Left);
    settle();
    assert_eq!(
        w.get_match_home_primary(),
        color_picker::hex_of_hsv(240.0, 0.5, 0.5)
    );
}

// --- The Settings sheet ---------------------------------------------------

/// A key, down and up.
fn press(w: &AppWindow, key: impl Into<SharedString>) {
    let key: SharedString = key.into();
    w.window()
        .dispatch_event(WindowEvent::KeyPressed { text: key.clone() });
    w.window()
        .dispatch_event(WindowEvent::KeyReleased { text: key });
}

fn type_text(w: &AppWindow, text: &str) {
    for c in text.chars() {
        press(w, c.to_string());
    }
}

/// Ctrl+A, as a keyboard sends it: the modifier's own press is what Slint
/// reads the modifier from.
fn select_all(w: &AppWindow) {
    let ctrl = SharedString::from(Key::Control);
    w.window()
        .dispatch_event(WindowEvent::KeyPressed { text: ctrl.clone() });
    press(w, "a");
    w.window()
        .dispatch_event(WindowEvent::KeyReleased { text: ctrl });
}

/// The drawing row's `Settings…`.
fn the_button(w: &AppWindow) -> ElementHandle {
    ElementHandle::find_by_accessible_label(w, BUTTON)
        .next()
        .unwrap_or_else(|| panic!("no {BUTTON:?} button"))
}

/// Opens the sheet by its button, as the coach does.
fn open(w: &AppWindow) -> ElementHandle {
    the_button(w).invoke_accessible_default_action();
    settle();
    assert!(w.get_settings_sheet_open(), "{BUTTON} opened nothing");
    sheet(w, "SettingsSheet")
}

/// Clicks into the band field, ready to type.
fn into_the_field(settings: &ElementHandle) {
    by_id(&by_id(settings, "SettingsSheet::band"), "SetupField::edit")
        .mock_single_click(PointerEventButton::Left);
}

/// What the band field holds, read off the field itself: a half-typed value
/// never leaves the sheet.
fn field_text(settings: &ElementHandle) -> String {
    by_id(&by_id(settings, "SettingsSheet::band"), "SetupField::edit")
        .accessible_value()
        .unwrap_or_default()
        .to_string()
}

fn red() -> Color {
    Color::from_rgb_u8(255, 0, 0)
}

#[test]
fn the_settings_button_opens_the_sheet_and_close_shuts_it() {
    let (w, edits) = window();
    w.set_slate_band("#ff0000".into());
    let settings = open(&w);
    // Seeding the field from what is stored is not an edit.
    assert!(edits.borrow().is_empty(), "{:?}", edits.borrow());
    by_label(&settings, "Close").invoke_accessible_default_action();
    settle();
    assert!(!w.get_settings_sheet_open());
}

#[test]
fn esc_in_the_field_leaves_it_before_it_closes_the_sheet() {
    let (w, _) = window();
    into_the_field(&open(&w));
    press(&w, Key::Escape);
    assert!(
        w.get_settings_sheet_open(),
        "the first Esc leaves the field"
    );
    press(&w, Key::Escape);
    assert!(!w.get_settings_sheet_open(), "the second closes the sheet");
}

/// One gate for both doors: not over a take, not over a preview.
#[test]
fn the_settings_button_is_off_while_recording_or_previewing() {
    let (w, _) = window();
    assert_eq!(the_button(&w).accessible_enabled(), Some(true), "idle");
    w.set_recording_phase(RecordingPhase::Recording);
    settle();
    assert_eq!(
        the_button(&w).accessible_enabled(),
        Some(false),
        "recording"
    );
    w.set_recording_phase(RecordingPhase::Idle);
    w.set_previewing_clip("c1".into());
    settle();
    assert_eq!(
        the_button(&w).accessible_enabled(),
        Some(false),
        "previewing"
    );
}

#[test]
fn a_typed_colour_reaches_the_band_and_is_reported() {
    let (w, edits) = window();
    into_the_field(&open(&w));
    type_text(&w, "#ff0000");
    assert_eq!(w.get_slate_band_color(), red());
    assert_eq!(edits.borrow().last().map(String::as_str), Some("#ff0000"));
}

#[test]
fn a_typo_is_never_reported_and_does_not_outlive_the_sheet() {
    let (w, edits) = window();
    let settings = open(&w);
    into_the_field(&settings);
    type_text(&w, "#ff0000");
    let reported = edits.borrow().len();

    select_all(&w);
    type_text(&w, "#12");
    assert_eq!(field_text(&settings), "#12", "the typo is in the field");
    assert_eq!(edits.borrow().len(), reported, "{:?}", edits.borrow());
    assert_eq!(w.get_slate_band_color(), red(), "the band stays on red");

    press(&w, Key::Escape);
    press(&w, Key::Escape);
    assert!(!w.get_settings_sheet_open());
    let reopened = open(&w);
    assert_eq!(
        field_text(&reopened),
        "#ff0000",
        "what the sheet reopens on"
    );
}

/// The return to the default, not its identity: the colour read first *is*
/// the default, so a wrong default would pass here (manual check 4).
#[test]
fn default_takes_the_colour_back() {
    let (w, edits) = window();
    let before = w.get_slate_band_color();
    let settings = open(&w);
    into_the_field(&settings);
    type_text(&w, "#ff0000");
    assert_eq!(w.get_slate_band_color(), red());

    let default = by_label(&settings, "Default");
    default.invoke_accessible_default_action();
    settle();
    assert_eq!(w.get_slate_band_color(), before);
    assert_eq!(edits.borrow().last().map(String::as_str), Some(""));
    assert_eq!(default.accessible_enabled(), Some(false));
}

/// The inline picker was made on open over an empty field, so this pins the
/// sync that follows the text as it is typed: with it dead the area reads the
/// red the strip started on.
#[test]
fn the_inline_picker_follows_typed_text() {
    let (w, _) = window();
    let settings = open(&w);
    into_the_field(&settings);
    type_text(&w, "#0000ff");
    by_id(&settings, "ColorPicker::area").mock_single_click(PointerEventButton::Left);
    settle();
    assert_eq!(
        w.get_slate_band(),
        color_picker::hex_of_hsv(240.0, 0.5, 0.5)
    );
}

/// From the default there is no colour to stand on, so the picker starts at
/// full saturation and brightness: the strip gives that hue, not black.
#[test]
fn the_hue_strip_from_the_default_gives_a_colour() {
    let (w, _) = window();
    let settings = open(&w);
    by_id(&settings, "ColorPicker::strip").mock_single_click(PointerEventButton::Left);
    settle();
    assert_eq!(
        w.get_slate_band(),
        color_picker::hex_of_hsv(180.0, 1.0, 1.0)
    );
}

/// The selected slate's row is painted in the band's colour, so its words take
/// whichever of black and white reads over it. What reaches the row's `Text`s
/// is a binding no test can read (a `Text` exposes no colour); this pins the
/// answer they are bound to.
#[test]
fn the_band_ink_reads_over_white_and_black() {
    let (w, _) = window();
    let maths = w.global::<ColorMaths>();
    let (white, black) = (
        Color::from_rgb_u8(255, 255, 255),
        Color::from_rgb_u8(0, 0, 0),
    );
    assert_eq!(maths.invoke_band_ink(white), black, "over White");
    assert_eq!(maths.invoke_band_ink(black), white, "over Black");
}

#[test]
fn a_kit_colour_lands_in_the_settings_field() {
    let (w, _) = window();
    let settings = open(&w);
    by_label(&settings, "White").invoke_accessible_default_action();
    settle();
    // `KIT_COLORS[0]`.
    assert_eq!(w.get_slate_band(), "#ffffff");
}

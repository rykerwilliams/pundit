//! **The colour picker, in each sheet that hosts it** (BACKLOG #131).
//!
//! `ColorPicker` holds its own position — hue, saturation and brightness — and
//! follows its host's text, so the grey-hue logic exists once. Each host gets
//! two tests: a kit swatch pressed **by its name** lands in the field, and a
//! click in the saturation/brightness area reads the hue the field's text
//! stands on. The second is the one that reaches the moved sync: a kit press
//! writes its hex as it stands and never touches the held position, so it
//! would pass with the sync dead.
//!
//! **The fixture wires the picker's pure callbacks before it reads or sets
//! anything** — a copy of `main.rs`'s wiring over the library's own
//! `parse_hex` and `color_picker`, because `main.rs` is unreachable. Unwired,
//! `valid-hex` answers `false` and `hex-of-hsv` answers `""`, and every
//! assertion below would pass or fail on that instead.
//!
//! **Every query is scoped to the sheet's element, not to its card.** The
//! setup sheet's popup is the card's *sibling* (it hangs over the card), so a
//! query inside the `Sheet` never finds a swatch.
//!
//! **`settle` after anything that is not a pointer or key event.** A
//! `changed` handler runs on the next change-handler pass, not at the
//! assignment; the pointer and key paths run that pass on the way in, but an
//! `invoke_accessible_default_action` and a Rust `set_*` do not.

use crate::{AppWindow, Hsv, KitColor};
use i_slint_backend_testing::ElementHandle;
use pundit_app::color_picker;
use pundit_app::match_panel::parse_hex;
use slint::platform::PointerEventButton;
use slint::{ComponentHandle, LogicalSize, ModelRc, VecModel};
use std::time::Duration;

/// The window with the picker's conversions and kit colours wired, at the
/// floor size it declares.
fn window() -> AppWindow {
    i_slint_backend_testing::init_no_event_loop();
    let w = AppWindow::new().unwrap();
    w.on_hex_color(|text| match parse_hex(&text) {
        Some(c) => slint::Color::from_rgb_f32(c.r as f32, c.g as f32, c.b as f32),
        None => slint::Color::from_argb_u8(0, 0, 0, 0),
    });
    w.on_valid_hex(|text| parse_hex(&text).is_some());
    w.on_hsv_of_hex(|text| {
        let hsv = color_picker::hsv_of_hex(&text).unwrap_or(color_picker::Hsv {
            hue: 0.0,
            sat: 0.0,
            val: 0.0,
        });
        Hsv {
            hue: hsv.hue,
            sat: hsv.sat,
            val: hsv.val,
        }
    });
    w.on_hex_of_hsv(|hue, sat, val| color_picker::hex_of_hsv(hue, sat, val).into());
    w.set_kit_colors(ModelRc::new(VecModel::from(
        color_picker::KIT_COLORS
            .iter()
            .map(|kit| KitColor {
                name: kit.name.into(),
                hex: kit.hex.into(),
            })
            .collect::<Vec<_>>(),
    )));
    w.show().unwrap();
    let (min_w, min_h) = (w.get_min_window_width(), w.get_min_window_height());
    w.window().set_size(LogicalSize::new(min_w, min_h));
    w
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
    let w = window();
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
    let w = window();
    w.set_match_home_primary("#0000ff".into());
    let setup = open_the_setup_picker(&w);
    by_id(&setup, "ColorPicker::area").mock_single_click(PointerEventButton::Left);
    settle();
    assert_eq!(
        w.get_match_home_primary(),
        color_picker::hex_of_hsv(240.0, 0.5, 0.5)
    );
}

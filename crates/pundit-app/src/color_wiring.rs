//! The window's colour maths, wired onto the `ColorMaths` global (#131).
//!
//! **A module of two roots, the binary and `tests/ui`**, for `key_action.rs`'s
//! reason: the global is the Slint compiler's, nameable only from a root with
//! `include_modules!()`, and a test fixture that wired its own copy would test
//! the copy. `main.rs` has it as `mod color_wiring;` and `tests/ui/main.rs`
//! through `#[path]`.
//!
//! Every conversion is `color_picker`'s or `match_panel::parse_hex`, so the
//! picker lands on exactly the bytes the field stores and the scoreboard
//! draws — that module's header says why there is no second rounding.

use pundit_app::color_picker;
use pundit_app::match_panel::parse_hex;
use pundit_core::highlight::label_ink;
use pundit_core::stroke::Rgba;
use slint::{ComponentHandle, ModelRc, VecModel};

use crate::{AppWindow, ColorMaths, Hsv, KitColor};

/// Installs every `ColorMaths` callback and the kit colours. Called once, before
/// anything reads a colour: an unwired `valid-hex` answers `false`.
pub fn wire_colors(window: &AppWindow) {
    let maths = window.global::<ColorMaths>();
    maths.on_hex_color(|text| match parse_hex(&text) {
        Some(c) => slint::Color::from_rgb_f32(c.r as f32, c.g as f32, c.b as f32),
        None => slint::Color::from_argb_u8(0, 0, 0, 0),
    });
    maths.on_valid_hex(|text| parse_hex(&text).is_some());
    maths.set_kit_colors(ModelRc::new(VecModel::from(
        color_picker::KIT_COLORS
            .iter()
            .map(|kit| KitColor {
                name: kit.name.into(),
                hex: kit.hex.into(),
            })
            .collect::<Vec<_>>(),
    )));
    maths.on_hsv_of_hex(|text| {
        // Half-typed text has no position; the picker gates on `valid-hex`
        // before it moves, so this is never what it stands on.
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
    maths.on_hex_of_hsv(|hue, sat, val| color_picker::hex_of_hsv(hue, sat, val).into());
    maths.on_band_ink(|c| {
        let ink = label_ink(Rgba {
            r: f64::from(c.red()) / 255.0,
            g: f64::from(c.green()) / 255.0,
            b: f64::from(c.blue()) / 255.0,
            a: 1.0,
        });
        slint::Color::from_rgb_f32(ink.r as f32, ink.g as f32, ink.b as f32)
    });
}

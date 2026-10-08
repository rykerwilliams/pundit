//! The window's one key lookup: `keymap::Action` into the generated
//! [`KeyAction`], wired onto `action-for` (BACKLOG #96, plan task 3).
//!
//! **This file is a module of two roots, the binary and `tests/ui`**, and that
//! is deliberate rather than tidy. The mapping below is 29 arms of
//! boilerplate where a copy-paste — `ZoomOut => ZoomIn` — is exactly the bug
//! this task could ship, and the only thing that can catch one is a test that
//! presses the key and watches the callback. A handler written again in the
//! test fixture would test the fixture's copy (`fit_window.rs`'s header is the
//! record of that trap for `place-picture`), so the handler lives here and
//! both roots `mod` it: `main.rs` as `mod key_action;` and `tests/ui/main.rs`
//! through `#[path]`. [`KeyAction`] is the Slint compiler's, so it can only be
//! named from a root that has `include_modules!()` — which is why this cannot
//! be a module of the library beside `keymap.rs`.
//!
//! **A `match` on the Rust enum, so an action added without a member here
//! fails to compile** — `ScanStep`'s and `MatchTag`'s pattern in `main.rs`.
//! The other direction is not checkable: Slint's `if` chain in `handle-key`
//! can be missing a branch for a mapped action and nothing in either language
//! will say so, which is the plan's Risk 1 and why `tests/ui/keys.rs` is a
//! table of every action rather than a test per key.

use pundit_app::keymap::{Action, Keymap};

use crate::{AppWindow, KeyAction};

/// Answers `action-for` from `keymap`, which is read **once** — a key event is
/// a lookup over a few dozen rows, never a file read.
///
/// The keymap is moved into the callback rather than shared: nothing else
/// holds it, and nothing yet rewrites it. The Keys sheet's rebinding (plan
/// task 5) is what needs it behind a handle, and that is one line when it
/// arrives.
pub fn wire_keys(window: &AppWindow, keymap: Keymap) {
    window.on_action_for(move |text, ctrl, shift, alt| {
        key_action(keymap.action_for(&text, ctrl, shift, alt))
    });
}

/// One member per [`Action`], and `none` for a key that is bound to nothing.
fn key_action(action: Option<Action>) -> KeyAction {
    let Some(action) = action else {
        return KeyAction::None;
    };
    match action {
        Action::OpenProject => KeyAction::OpenProject,
        Action::Undo => KeyAction::Undo,
        Action::Redo => KeyAction::Redo,
        Action::ClearDrawings => KeyAction::ClearDrawings,
        Action::MarkIn => KeyAction::MarkIn,
        Action::MarkOut => KeyAction::MarkOut,
        Action::TagHomeGoal => KeyAction::TagHomeGoal,
        Action::TagAwayGoal => KeyAction::TagAwayGoal,
        Action::TagPeriod => KeyAction::TagPeriod,
        Action::HighlightTool => KeyAction::HighlightTool,
        Action::ToggleRecording => KeyAction::ToggleRecording,
        Action::DeleteClip => KeyAction::DeleteClip,
        Action::TogglePlay => KeyAction::TogglePlay,
        Action::SkipBack => KeyAction::SkipBack,
        Action::SkipForward => KeyAction::SkipForward,
        Action::SkipBackFar => KeyAction::SkipBackFar,
        Action::SkipForwardFar => KeyAction::SkipForwardFar,
        Action::StepBack => KeyAction::StepBack,
        Action::StepForward => KeyAction::StepForward,
        Action::PrevEvent => KeyAction::PrevEvent,
        Action::NextEvent => KeyAction::NextEvent,
        Action::ScanSlower => KeyAction::ScanSlower,
        Action::ScanFaster => KeyAction::ScanFaster,
        Action::FitWindow => KeyAction::FitWindow,
        Action::ZoomReset => KeyAction::ZoomReset,
        Action::ZoomOut => KeyAction::ZoomOut,
        Action::ZoomIn => KeyAction::ZoomIn,
        Action::ShowKeys => KeyAction::ShowKeys,
        Action::ShowRecents => KeyAction::ShowRecents,
    }
}

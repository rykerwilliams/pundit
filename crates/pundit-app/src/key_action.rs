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

use std::cell::RefCell;
use std::rc::Rc;

use pundit_app::bus::AppFiles;
use pundit_app::keymap::{Action, Binding, Keymap};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

use crate::{AppWindow, KeyAction, KeyRow};

/// Answers `action-for` from `keymap`, fills the Keys sheet's list from the
/// same map, and takes the sheet's two edits back into it and into `state`.
/// Read **once** — a key event is a lookup over a few dozen rows, never a file
/// read.
///
/// **The list and the lookup are one map read twice**, which is the whole of
/// why the sheet cannot go stale: there is no second table to keep in step, and
/// an edit sets both again from the one `Keymap` behind the handle.
///
/// **It writes through `AppFiles` and not the bus**, the same reason the panel
/// widths and the pen width do: a keymap reaches a key event on this thread, so
/// the bus has nothing to hold and no command to carry. What is stored is the
/// **diff** (`Keymap::overrides`), written whole on every edit.
pub fn wire_keys(window: &AppWindow, state: &AppFiles, keymap: Keymap) {
    let keymap = Rc::new(RefCell::new(keymap));
    window.set_key_rows(key_rows(&keymap.borrow()));
    window.on_action_for({
        let keymap = Rc::clone(&keymap);
        move |text, ctrl, shift, alt| {
            key_action(keymap.borrow().action_for(&text, ctrl, shift, alt))
        }
    });
    window.on_bind_key({
        let (weak, state, keymap) = (window.as_weak(), state.clone(), Rc::clone(&keymap));
        move |name, text, ctrl, shift, alt| {
            let Some(w) = weak.upgrade() else {
                return;
            };
            let Some(action) = Action::from_name(&name) else {
                // The sheet's own rows carry the names, so this is
                // unreachable; clearing rather than ignoring is what stops an
                // unreachable case from being a window stuck swallowing keys.
                return done(&w, "");
            };
            let Some(binding) = Binding::from_event(&text, ctrl, shift, alt) else {
                // A modifier on its own: the coach is still reaching for the
                // key, so the capture stays armed and the line stays up.
                return;
            };
            if binding.reserved() {
                // Still armed, so the next press is taken instead: the refusal
                // is a correction, not a cancellation.
                w.set_keys_message(
                    format!(
                        "{} is the app's own — press another key, or Escape to                          leave {} alone.",
                        binding.label(),
                        quoted(action)
                    )
                    .into(),
                );
                return;
            }
            let displaced = keymap.borrow_mut().rebind(action, binding);
            store(&w, &state, &keymap.borrow());
            done(
                &w,
                &match displaced {
                    Some(from) => format!(
                        "{} is now {}, taken from {}.",
                        quoted(action),
                        binding.label(),
                        quoted(from)
                    ),
                    None => format!("{} is now {}.", quoted(action), binding.label()),
                },
            );
        }
    });
    window.on_unbind_key({
        let (weak, state, keymap) = (window.as_weak(), state.clone(), Rc::clone(&keymap));
        move |name| {
            let (Some(w), Some(action)) = (weak.upgrade(), Action::from_name(&name)) else {
                return;
            };
            keymap.borrow_mut().unbind(action);
            store(&w, &state, &keymap.borrow());
            done(&w, &format!("{} has no key now.", quoted(action)));
        }
    });
}

/// An action by its words, in quotes — how the sheet's line names a row, which
/// is the only thing a coach ever saw it called.
fn quoted(action: Action) -> String {
    format!("“{}”", action.what())
}

/// Remembers the map and shows it: the file, then the sheet's rows, which are
/// rebuilt from the same map so the row the coach just changed reads back as
/// what it became.
fn store(window: &AppWindow, state: &AppFiles, keymap: &Keymap) {
    state.set_keymap(keymap);
    window.set_key_rows(key_rows(keymap));
}

/// Ends the capture and says what came of it. **The clear is here and not in
/// `app.slint`** — see `capturing-action`'s own note: a press that is not a key
/// has to leave the capture armed, and only this side can tell.
fn done(window: &AppWindow, message: &str) {
    window.set_capturing_action(SharedString::new());
    window.set_keys_message(message.into());
}

/// `Keymap::listing` as the sheet's model, one row per `Action::ALL` entry in
/// that order — which is the order the sheet shows and the order a collision is
/// resolved in, so it is carried rather than sorted here.
fn key_rows(keymap: &Keymap) -> ModelRc<KeyRow> {
    let rows: Vec<KeyRow> = keymap
        .listing()
        .into_iter()
        .map(|row| KeyRow {
            what: row.what.into(),
            keys: row.keys.as_str().into(),
            fires: row.when.into(),
            action: row.action.into(),
            bound: row.bound,
        })
        .collect();
    ModelRc::new(VecModel::from(rows))
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

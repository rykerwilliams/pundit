//! Every Slint test this crate has, in **one** test binary (BACKLOG #121).
//!
//! `build.rs` compiles `ui/app.slint` — 6,375 lines — into Rust, and every
//! `slint::include_modules!()` compiles that Rust again. One per test file meant
//! four rustc processes each holding the whole generated UI, and cargo runs test
//! binaries' compiles in parallel across all eight cores: on 2026-10-02 that
//! thrashed a 31 GiB laptop to a standstill and took it down uncleanly, with no
//! OOM kill logged. There is **one** `include_modules!()` in this crate's tests
//! — below — and every window test is a module of this binary. A new one joins
//! it; it does not add a file beside it.
//!
//! `scrubber` and `splitter` need no copy of the app's UI: they `slint!` a
//! single component against a stand-in window. They are modules here anyway,
//! because "every Slint test is a module of `tests/ui`" is one rule and "except
//! the two that only drive their own component" is two — and the next test to
//! be written against a stand-in is one `include_modules!()` away from being
//! written against the real window.
//!
//! **A test here can drive the real tree, not only poke properties** — find an
//! element, right-click it, read what a popup put on screen and click an item
//! in it (`slate_menu`). That is `i_slint_backend_testing`'s `ElementHandle`,
//! and it works only because `build.rs` compiles the UI with Slint's debug
//! info, which is where the measurement and the reason live (BACKLOG #127).
//!
//! **Sharing a process is free here, and the reason is where Slint keeps its
//! platform.** `init_no_event_loop()` ends in
//! `.expect("platform already initialized")`, which reads like a once-per-process
//! install and is the one thing that could have made this merge more than a move.
//! It is not: the slot `set_platform` claims is
//! `i_slint_core::context::GLOBAL_CONTEXT`, a `thread_local!`, and
//! `init_no_event_loop` asks for no event-loop proxy (`threading: false`, so
//! `TestingBackend`'s queue is `None`), which is the only process-wide slot the
//! backend has. So each fixture installs a backend on its own test's thread and
//! none of them can see another's — which is already how every file here worked,
//! each calling `init_no_event_loop()` once per fixture across two to five tests.
//! Measured on the pre-merge binaries: `--test-threads=1` passes too, with and
//! without `--nocapture`, because libtest spawns a thread per test either way.

slint::include_modules!();

/// **The production key wiring, shared with the binary** (BACKLOG #96). The
/// `Action` -> `KeyAction` map is 29 arms of boilerplate and the test is the
/// only thing that can catch a crossed one, so `keys.rs` presses keys against
/// *this* handler rather than a copy written in a fixture. `#[path]` because
/// the type it answers with is the Slint compiler's, nameable only from a root
/// with `include_modules!()`, so it cannot live in the library.
#[path = "../../src/key_action.rs"]
mod key_action;

/// The app's state file under a temporary directory, for the six fixtures that
/// wire the keys.
///
/// `wire_keys` writes a rebind through `AppFiles`, and the handle it is given
/// in the app is `$XDG_CONFIG_HOME/pundit/state.json` — **the coach's own**.
/// So no fixture may hand it that one, and this is what they hand it instead.
///
/// **One directory for the whole binary, and nothing reads it back.** The
/// keymap each window uses is the one passed to `wire_keys`, not one loaded
/// from here, so what a fixture writes is never read — by itself or by
/// another. The round trip through a real file is `bus::state`'s own
/// `a_rebind_survives_the_file`, with a directory of its own.
pub fn scratch_state() -> pundit_app::bus::AppFiles {
    static DIR: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    pundit_app::bus::AppFiles::in_config_dir(
        DIR.get_or_init(|| tempfile::tempdir().expect("a scratch directory"))
            .path(),
    )
}

mod export_sheet;
mod fit_window;
mod keys;
mod keys_sheet;
mod panel_widths;
mod scrubber;
mod self_view_placement;
mod sheet_scroll;
mod slate_fields;
mod slate_menu;
mod splitter;
mod tag_field;
mod text_editing;

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

mod fit_window;
mod panel_widths;
mod scrubber;
mod self_view_placement;
mod slate_fields;
mod slate_menu;
mod splitter;

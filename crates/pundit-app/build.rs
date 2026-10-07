fn main() {
    // **Debug info is what lets a test see the real window.**
    // `i_slint_backend_testing`'s `ElementHandle` — find an element by the
    // words on it, read what it shows, invoke its own action — refuses to do
    // anything without it ("The use of the ElementHandle API requires the
    // presence of debug info in Slint compiler generated code"), which is why
    // every test in `tests/ui` before these two could only poke a window
    // property or dispatch a key.
    //
    // **Two features needed it independently, and that is the argument.**
    // #127: a `MenuItem`'s `activated` is reachable from nowhere but its menu,
    // so the slate row's right-click items had no test at all. #78: the export
    // sheet's six controls are wired to the window with `<=>`, and the leaf
    // each wire feeds is a `CheckBox.checked` inside `ExportSheet` with no id
    // the window root can name — so a dead wire was invisible to the whole
    // suite. Neither could be tested any other way, and the second found the
    // accessibility tree is the way in (`accessible-label` is bound by every
    // style's `CheckBox`), which is position-independent and so works behind a
    // modal scrim.
    //
    // **Measured, so it is not a guess:** `app.slint` generates 15,779,200
    // bytes of Rust without it and 16,101,779 with — +322 KB, 2%. **And the
    // number that actually decides it, the shipped binary:** 59,211,608 bytes
    // without, 59,362,792 with — **+151 KB, 0.26%** of a release build (two
    // release builds, trees differing only by small features besides this, so
    // the debug-info share is ~150 KB give or take). Less than half what the
    // generated-Rust figure suggests, because most of what it adds is strings
    // the linker packs tightly. What it adds is element names and types beside
    // the item tree, not a code path, and it is on in every profile
    // deliberately: scoped to `PROFILE == "debug"` the UI tests would fail
    // under `cargo test --release` with a message about an environment
    // variable.
    slint_build::compile_with_config(
        "ui/app.slint",
        slint_build::CompilerConfiguration::new().with_debug_info(true),
    )
    .expect("compile ui/app.slint");
}

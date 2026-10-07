fn main() {
    // **Debug info is what lets a test see the real window** (BACKLOG #127).
    // `i_slint_backend_testing`'s `ElementHandle` — find an element, right-click
    // it, read what a popup put on screen — refuses to do anything without it
    // ("The use of the ElementHandle API requires the presence of debug info in
    // Slint compiler generated code"), which is why every test in `tests/ui`
    // until now could only poke a property or dispatch a key, and why the slate
    // row's menu had no test at all: a `MenuItem`'s `activated` is reachable
    // from nowhere else.
    //
    // **Measured, so it is not a guess:** `app.slint` generates 15,779,200
    // bytes of Rust without it and 16,101,779 with — +322 KB, 2%. **And the
    // number that actually decides it, the shipped binary:** 59,211,608 bytes
    // without, 59,362,792 with — **+151 KB, 0.26%** of a release build (two
    // release builds, trees differing only by small features besides this, so
    // the debug-info share is ~150 KB give or take). Less than half what the
    // generated-Rust figure suggests, because most of what it adds is strings
    // the linker packs tightly. What it adds
    // is element names and types beside the item tree, not a code path, and it
    // is on in every profile deliberately: scoped to `PROFILE == "debug"` the
    // UI tests would fail under `cargo test --release` with a message about an
    // environment variable.
    slint_build::compile_with_config(
        "ui/app.slint",
        slint_build::CompilerConfiguration::new().with_debug_info(true),
    )
    .expect("compile ui/app.slint");
}

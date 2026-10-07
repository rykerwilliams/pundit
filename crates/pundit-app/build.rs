fn main() {
    // **Debug info is what lets a test see the real window.**
    // `i_slint_backend_testing`'s `ElementHandle` — find an element by the
    // words on it, read what it shows, invoke its own action — refuses to do
    // anything without it ("The use of the ElementHandle API requires the
    // presence of debug info in Slint compiler generated code"), which is why
    // every test in `tests/ui` until `export_sheet` could only poke a window
    // property or dispatch a key. The export sheet's six controls are wired to
    // the window with `<=>`, and the leaf each wire feeds is a
    // `CheckBox.checked` inside `ExportSheet` with no id the window root can
    // name, so a dead wire was invisible to the whole suite (#78).
    //
    // **It is on in every profile deliberately.** Scoped to a debug build the
    // UI tests would fail under `cargo test --release` with a message about an
    // environment variable. What it adds is element names and types beside the
    // item tree, not a code path, and most of it is strings the linker packs
    // tightly — BACKLOG #127, which wants the same switch for the slate row's
    // menu (whose `MenuItem`s are reachable from nowhere else), carries the
    // measurement of what it costs the generated Rust and the shipped binary.
    // If both land, these two build scripts are one change.
    slint_build::compile_with_config(
        "ui/app.slint",
        slint_build::CompilerConfiguration::new().with_debug_info(true),
    )
    .expect("compile ui/app.slint");
}

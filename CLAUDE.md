# pundit — Project Conventions

## Workflow for non-trivial features

Each feature goes through a four-stage loop, with adversarial review at every artifact handoff:

1. **Brainstorm → spec** (`docs/superpowers/specs/YYYY-MM-DD-<topic>-design.md`)
2. **Adversarial review on the spec** — see "Review pattern" below. Apply fixes, then commit.
3. **Write plan** (`docs/superpowers/plans/YYYY-MM-DD-<topic>.md`)
4. **Adversarial review on the plan** — same pattern. Apply fixes, then commit.
5. **Compact the conversation before plan execution.** Plans get long; execution dispatches many subagents and consumes context fast. Start the execution phase with fresh context — re-read the plan + spec + this file rather than relying on accumulated chat history.
6. **Execute** via `superpowers:subagent-driven-development` (fresh subagent per task).
7. **Adversarial review on the shipped code changes** — apply fixes, then commit.
8. **Backlog deferred items** to `BACKLOG.md` at the worktree root.

## Review pattern (use for specs, plans, and shipped code)

For each review pass, spawn **two adversarial agents in parallel**:

- **Simplify agent** — find every place the design / plan / code is more complex than it needs to be. Recommended subagent: `general-purpose`. Frame as "adversarial simplification review."
- **Code-review / correctness agent** — find correctness bugs, fragile patterns, things that pass tests today but break tomorrow. Recommended subagent: `feature-dev:code-reviewer` for code; `general-purpose` for specs/plans.

Both agents get:
- The artifact under review (spec, plan, or diff range)
- The relevant codebase reference paths (so they can verify claims, not just trust the artifact)
- The full "user values" block (below)

After both reviews return:

1. **Group similar findings** across the two reviews.
2. **Spawn one deliberation agent per group** (in parallel). Each agent's job:
   - Research all issues in its group against the codebase
   - For each issue, decide the best long-term fix
   - Adversarial self-review of its own conclusions
   - **Defer to human** if the right fix isn't obvious
3. **Apply / skip per group**:
   - **APPLY** when the fix is strictly better than the original
   - **SKIP** when the fix is worse than the original issue (every change must earn its place)
   - **DEFER** when judgment is required from the human
4. Surface anything deferred at the end.

## User values (paste into every adversarial review prompt)

- Best long-term design over short-term tradeoffs
- It's OK to change adjacent code if it helps get to the best long-term design
- Simplicity — avoid over-engineered systems and fixes
- Don't care about effort or severity
- Care about long-term codebase quality and maintainability
- Don't need to fix every single race condition or edge case if they're super rare unless the fix has zero tradeoffs
- Pay close attention to fixes that add complexity — the fix needs to be worth it
- Every change must earn its place; if the fix is worse than the original issue, skip it
- Leave the code in a better place than we found it

## Project skills (`.claude/skills/`)

| Skill | Use it to |
|---|---|
| `port-swift-module` | Read a module out of the `macos-reference` tag into `pundit-core` without repeating past mistakes |
| `verify` | Run fmt, clippy, tests and the core dependency audit before committing |
| `measure-media` | Benchmark GStreamer decode/seek on real hardware without fooling yourself |
| `adversarial-review` | Run the review pattern below on a spec, plan, or diff |

`.claude/` is committed; personal overrides go in `.claude/settings.local.json` (gitignored).

## Build + test conventions

### Rust port (primary)

The Linux port is the active codebase. Spec: `docs/superpowers/specs/2026-09-19-linux-port-design.md`.

```bash
cargo test -p pundit-core   # pure logic -- needs NO GStreamer
cargo test --workspace      # everything -- needs GStreamer dev libraries
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```

**A clean local `clippy` is NOT the same gate as CI's.** `rust.yml`'s workspace
job pins **`dtolnay/rust-toolchain@1.92`** while this laptop is on 1.98.1, and
clippy's lints move between releases in both directions — measured 2026-10-02,
`nonminimal_bool` fired on 1.92 for a guard that 1.98 passed without comment, so
the branch was green locally and red on CI. When clippy fails on CI and not
here, **read the version in the error's own help URL**
(`…/rust-clippy/rust-1.92.0/index.html#<lint>`) before doubting the code. The
cheap way to check before pushing is `rustup run 1.92 cargo clippy …`, which
needs that toolchain installed; the pin is deliberate and is not the thing to
change.

**`pundit-app`'s Slint tests are one test binary, `tests/ui/`, and a new one is
a `mod` of it.** `build.rs` compiles `ui/app.slint` — 6,375 lines — and every
`slint::include_modules!()` compiles that generated Rust again, so one per test
file is one rustc per file holding the whole UI, and cargo compiles test
binaries in parallel across every core. Six of them thrashed this 31 GiB laptop
to a standstill on 2026-10-02 and took it down with no OOM kill logged (BACKLOG
#121). There is **one** `include_modules!()` in the crate's tests, in
`tests/ui/main.rs`; `scrubber` and `splitter` are modules of it too, although
they `slint!` only their own component, because one rule is simpler than one
with an exception. Measured on the merge: eight test binaries to three,
`rustup run 1.92 cargo clippy --workspace --all-targets -j 3` from 428.6 s to
227.2 s and `cargo test -p pundit-app --no-run -j 3` from 667.7 s to 337.9 s.

**`build.rs` compiles the UI with Slint's debug info, and that is what lets a
test reach inside a component.** `i_slint_backend_testing`'s `ElementHandle`
does nothing without it, and without `ElementHandle` a test can only poke a
window property or dispatch a key — which is why a control wired to the window
with `<=>` used to have no test at all: the leaf the wire feeds is a
`CheckBox.checked` inside a sheet, with no id the window root can name, so a
dead wire changed nothing any assertion could see. `tests/ui/export_sheet.rs`
is the pattern: **find the control by the words on it**
(`find_by_accessible_label` — every style's `CheckBox` binds
`accessible-label`, `accessible-checked` and `accessible-enabled`, and a `Text`
binds `accessible-label` by default, so an `if`-guarded line is findable or
absent), read it back, and tick it with
`invoke_accessible_default_action` rather than a synthesized click — a pointer
event would have to land on a computed position inside a sheet behind a modal
scrim, which is a layout measurement no wiring test wants to be. What a UI test
still **cannot** reach is `main.rs`'s seeding: `open_export_sheet` reads the
`UI` thread-local, so a missing `set_export_*` there passes everything.
**Sharing the process is free**: Slint's platform lives in a `thread_local!`
and libtest gives each `#[test]` a thread, so each fixture calls
`init_no_event_loop()` for itself and none may assume it is first.

**Speech recognition needs `cmake` and `libclang-dev`** (`sudo apt install
cmake libclang-dev`). `pundit-media` depends on `whisper-rs`
unconditionally — there is no feature gate, by decision — so without them
nothing builds but `pundit-core`. With them, the first build spends
**about three minutes** compiling the vendored whisper.cpp, and nothing
afterwards.

**whisper.cpp's instruction set is pinned to x86-64-v3** — Haswell-class:
AVX2, FMA, F16C, BMI2 — by `.cargo/config.toml`'s `[env]`, so dev, test, CI
and the `.deb` all build the same library. `GGML_NATIVE=OFF` stops
`-march=native` (a CI runner with AVX-512 would build a binary that `SIGILL`s
on the laptop); the explicit `GGML_*=ON` flags are what survive
`SOURCE_DATE_EPOCH`, which otherwise switches off *all* SIMD. **After changing
any `GGML_*`, run `cargo clean -p whisper-rs-sys` and again with `--release`**
— cargo does not rerun the build script when `[env]` changes, and each clean
removes one profile's copy only. `packaging/build-deb.sh` checks the release
build it packaged and fails on anything but `-mavx2` without `-march=native`.

**Which model runs is the coach's, machine-wide.** The inspector's transcript
row has a picker (`base.en` / `small.en`, default `small.en`), remembered in
`state.json` and never in `project.json`, because **which model is fast enough
is a property of this machine and not of the match** — a project carried to
another computer should not bring a model choice with it. (This used to say a
`Preferences` field would be "a format change every existing project fails
`store::read`'s version guard on", which is **false**: `read` accepts
`MIN_READABLE..=CURRENT`, so a newer build reads every older file and
`Preferences`' container default fills a key it hasn't got. The real cost of a
`Preferences` field is the **forward** direction — the first save re-stamps the
version and an *older* build then refuses the file, which is what the
`project.json.v<old>` backup exists for. The conclusion was right; the reason
was not.)
Switching models leaves a job *transcribing* alone (a whisper cancel costs
~12 s of CPU) but preempts one still *downloading* (that stops within 100 ms),
which restarts on the new model; the queue behind it picks the new one up. `$PUNDIT_WHISPER_MODEL`
still beats the picker, which says so by going grey and showing the file that
variable names. `WhisperModel` in `pundit-media/src/transcribe.rs`
carries each model's file name, size and **measured** sha256.

**The model downloads on first use, and only when the bus says it may.** It
lives in `$XDG_CACHE_HOME/pundit/models/`; a job whose model is absent
downloads it first (`souphttpsrc ! filesink` to a `.part`, glib's sha256 of
the file, rename), as `TranscribeMessage::Downloading` and its own
inspector line. **Permission is `TranscribeKind::Whisper`'s `fetch`, never
the path** (the reasoning lives on that variant): `bus::whisper` sets it only
under the cache directory, and a model switch moves only a path that has one.
**No test may reach Hugging Face** — serve from `pundit_media::fixtures::serve`.
The URL is pinned to a Hugging Face commit, not `main`.
The Transcribe button is the prompt ("Download 488 MB and transcribe"). A
cancel leaves the `.part`; every other failure deletes it; a failed download
drops the queue behind it. Downloads take turns process-wide, so a cancelled
one can't rename a `.part` its successor is writing.

**Transcription is asked for, never automatic.** `AUTO_TRANSCRIBE` in
`bus/transcribe.rs` is `false`: a preempted job restarts from zero, so a coach
recording faster than a job finishes would complete none of them. Measured:
`small.en` runs at **0.73x realtime** on the reference laptop
(`docs/superpowers/spikes/2026-09-21-whisper-throughput.md`).

**Recording always wins, and cancelling is not free.** Starting a recording
cancels the job in flight and puts its clip back at the **front** of the queue.
whisper only consults its abort flag once per encode and once per decode pass,
so a cancel costs **~12 s of CPU** — `Transcriber::drop` therefore cancels
*without* joining, because the bus thread used to block on it and froze Stop
Recording along with both deadlines.

**Whisper's progress percentage is nearly useless on a short clip.** It is
reported at the top of a loop advancing in <=30 s chunks and never reaches 100,
so a 20 s clip yields exactly one callback reading 0. The inspector shows an
elapsed clock and appends the percentage only once it moves off zero.

The whisper tests are **`#[ignore]`d**, because they need a model CI has
no copy of. Run them by pointing `$PUNDIT_WHISPER_MODEL` — the same
variable the app finds its model with — at one, and read the throughput line
off `--nocapture`:

```bash
PUNDIT_WHISPER_MODEL=~/.cache/pundit/models/ggml-small.en.bin \
  cargo test -p pundit-media transcribe -- --ignored --nocapture --test-threads=1
```

`--test-threads=1` is **not optional**: `--ignored` runs *only* the ignored
tests, and libtest would run them in parallel — two whisper contexts, ~1 GB
resident and sixteen threads on eight cores, which is not the machine the
throughput line describes.

**Running the app** (needs a display and GStreamer's runtime plugins incl.
`gstreamer1.0-gl`):

```bash
cargo run --release -p pundit-app               # restores the last project
cargo run --release -p pundit-app -- <folder>   # opens (or creates) a project there
```

The name is **`pundit`**, lower case, everywhere: the binary
(`target/*/pundit`, via `[[bin]]` — the package is `pundit-app`), the `.deb`,
the config and cache directories, `packaging/pundit.desktop` and its icons, the
window's `WM_CLASS` / Wayland `app_id` (`slint::set_xdg_app_id` in `main.rs`,
only valid after `BackendSelector::select()`), and `core::metadata::APP_NAME`,
which is what tags an export — read that constant rather than spelling the name
again. The backronym (*pundit Understands Nothing, Discusses It Thoroughly*)
belongs in the README and the package description, not in the UI.

**It was `coach-cuts` until 0.8.0**, a name inherited from the macOS app, and
0.8.0 also left the fork: the repository is `rykerwilliams/pundit`, with
`rykerwilliams/coach-cutups` deliberately left in place. Two shims carry an
existing installation across, each explained where it lives —
`state::adopt_old_name` (it renames the config and cache directories at
startup) and the `.deb`'s `conflicts = "coach-cuts"` (it is what makes one
`apt install` swap the packages; the other two fields of the conventional three
are measurably inert here, and the manifest says why). **Both are dated:**
`BACKLOG.md` #93 deletes them once no 0.7.x installation is left to upgrade.
They are transitional, not conventions.

The app must run on Slint's **Skia OpenGL** renderer (it selects it and fails
loudly otherwise): that renderer is EGL on X11 and Wayland, and EGL is what
lets GStreamer import decoded frames without a CPU copy. It logs the decoder,
the caps entering `glupload` and the GL platform on every source load (`bus:
loaded …` on stderr); that line is the zero-copy diagnostic, and on the
reference laptop it reads `vah265dec` / `memory:DMABuf` / `egl`.
`scripts/linux-gate-check.sh` measures decode throughput. **The projects the
coach has had open** (newest first, capped at 8), the chosen speech model, the
pen, the window's size, **the two side columns' widths** and **the keys the
coach rebound** live in
`$XDG_CONFIG_HOME/pundit/state.json` — every one of them a property of this
machine and none of the project's;
point `XDG_CONFIG_HOME` elsewhere when testing so the real one isn't
touched. **That file is read per field** (BACKLOG #100): a value this build can't
read falls back to that field's own default and costs the **document** nothing.
Two attributes do it — `bus/state.rs`'s `lenient` deserializer on each field of
`State`, and `#[serde(default)]` on the **container**, because `deserialize_with`
is not called for an absent key and a field-level default beside it would be the
same rule written twice. The container defaults on `WindowSize` and `PanelWidths`
**stay, and are not redundant**: they rescue a *partial object* —
`{"window": {"width": 1600}}` keeps the 1600 — which the per-field read does not,
and their hand-written `Default` impls are what stop a height of `0` reaching
`set_size`. **Two things this does not fix:** one malformed *element* still costs
a whole list, which is the bargain `bus/basket.rs` strikes for its `pieces`; and
a lost *update* between the bus's `AppFiles` handle and `main.rs`'s is untouched,
since every setter still rewrites the whole document.

**The keys live in one table, and `state.json` stores only the overrides**
(`keymap.rs`, BACKLOG #96; spec
`docs/superpowers/specs/2026-10-02-rebindable-keys-design.md`, plan
`2026-10-07-rebindable-keys.md`). The convention is VS Code's
`keybindings.json`: the defaults are in code (`Action::default_keys` — 29
actions, 34 bindings), the `keys` object holds the **diff** keyed by the
action's camelCase name, and an action it doesn't mention keeps its default —
so deleting the key is the reset path, and a later version that retunes a
default still reaches a coach who rebound something else. **Storing a copy of
the table would pin all 29 rows for ever**, which is why `Keymap::overrides`
emits nothing for an untouched map. A stored row **displaces the default** that
held its key (the loser is left unbound, logged, and shown so); two *stored*
rows claiming one key are resolved by `Action::ALL`'s order, which is why that
order is fixed. A bad label costs that label, an unknown action name costs that
row, and a JSON type error on `keys` costs the **whole keymap and nothing
else** — `lenient`'s bargain, and a re-pick rather than a project.
**Slint carries text, not scancodes** (`KeyEvent` is `text`/`modifiers`/
`repeat`, and winit's backend reads `logical_key` only), so a
layout-independent binding is not expressible and `NamedKey`'s characters come
from `slint::platform::Key` rather than a copied code point; the matcher
lowercases the text and compares the three modifiers **exactly**, which is what
leaves Caps Lock free and makes `shift+a` its own binding. A keymap is a
property of the coach's hands, so **nothing here is a `project.json` field and
no format version moves.**

**The Keys sheet is where they are rebound, and the capture is a window layer
rather than a focused field.** `F1` opens the sheet; each action's row carries
*Set…* and ✕. *Set…* arms `capturing-action` on the window and the **top of
`handle-key`** harvests the next key — it has to be there, because shortcuts are
dispatched window-to-focus-item *before* the focused element, so a `FocusScope`
inside the sheet would never see a key the table binds. **`Set…` replaces the
row's keys with the one captured** (the two-key defaults keep their pairs until
touched; adding a second is BACKLOG #135), and `Keymap::rebind` is G3's
last-wins: the key is taken off whatever held it, which keeps its *other* keys,
and the displaced action is named on the sheet's own line. That line is the
sheet's and not the status bar's, whose notice renders **behind the scrim**.
- **`AppFiles::set_keymap` writes the whole diff**, never one row — last-wins
  changes *two* rows, and a one-row write leaves the displaced binding in the
  file for the next load to resurrect as a collision. The spec's
  `set_binding(action, Vec<Binding>)` is wrong for that reason.
- **`capturing-action` is cleared by Rust, not by `app.slint`** — except for
  Escape, which cancels and needs no round trip. Only `keymap.rs` can say
  whether a press is a key at all: a **bare modifier leaves the capture armed**
  (Slint delivers a key event for Ctrl, and a capture that took the first press
  would store Ctrl and never see the letter), and one of the five **reserved**
  keys is refused with the capture still armed. `Binding::reserved` is Escape,
  Tab, Return, Home and End with any modifiers, and it is the same five the sheet
  lists as not rebindable — three of them because `handle-key` tests them *ahead*
  of the lookup, Tab because a coach who bound it away would have no keyboard
  path back to the sheet to undo it. **Delete is not one of them**, deliberately:
  the plan had Backspace and Delete *unbind* during a capture, but Delete is
  `deleteClip`'s own default, so that would have made a shipped binding
  unreachable. Unbinding is the ✕ instead.
- **Every one of the 29 actions now has a `handle-key` branch**, `showRecents`
  included — it still has no default, so the branch is unreachable until a coach
  binds it, which is the whole reason it exists.

**`last_project` is the derived head of `recent_projects`, never a second
stored copy** (#85): `push_recent_project` is the only writer, `set_last_project`
is deleted, and `==` on the canonical path `commit` stores is the whole
de-duplication. The old `lastProject` key survives as a **read-only seed** —
filled into an empty list once in `read`, and `#[serde(skip_serializing)]` so it
self-cleans out of the file on the first save. It is dated: BACKLOG #109 deletes
it. **The seeding belongs in `read` and not in the accessor**, measured: every
setter is read-then-save over the raw field, so seeded in the accessor the first
save of *any* setting — a pen change included — writes an empty list and no old
key, and the coach's projects are gone. With
the monitor off (DPMS), playback slows unless run with `vblank_mode=0`
(BACKLOG #36).

**Packaging** — a `.deb` for Ubuntu 24.04 / Mint 22 (needs `cargo install
cargo-deb`, `cargo install cargo-about --features cli`, `dpkg-dev` and docker):

```bash
packaging/build-deb.sh                                           # → target/debian/pundit_<version>_amd64.deb
packaging/smoke-test.sh target/debian/pundit_<version>_amd64.deb
```

- **Build with the script, never bare `cargo deb`.** It first generates the crate
  licence notices with cargo-about, which the asset list ships. Its `accepted` list
  (`packaging/about.toml`) is a **tripwire**: a crate licensed GPL-2.0-*only* can't
  be combined with this AGPL program, so an unlisted licence fails the build.
  Review it before adding one; never add GPL-2.0-only.
- **`packaging/copyright` is hand-written** and covers what crate tools can't see:
  whisper.cpp, the prebuilt Skia (with VulkanMemoryAllocator) and the DejaVu fonts.
  A new statically linked C/C++ library or embedded asset needs a stanza there.
- **`Depends:` is `$auto` plus a hand-kept list** (`[package.metadata.deb]`,
  commented). `$auto` is `dpkg-shlibdeps`; without `dpkg-dev`, cargo-deb only warns.
  Anything loaded at run time — a GStreamer element, a `dlopen`ed library — must
  be added by hand.
- **The smoke test is the only proof of the dependency list.** It runs in a clean
  `ubuntu:24.04` container, because the laptop already has every dev package. It
  checks the libc floor, installs without Recommends, looks up every software-path
  element, and launches the app under Xvfb. A new element the code names goes in
  its list.
- **Build dependencies are one list**, `packaging/build-deps.txt`, read by both
  workflows and the README.
- **Releasing:** bump `version` in the root `Cargo.toml`'s `[workspace.package]`,
  commit, then `git tag v<version> && git push origin v<version>`.
  `.github/workflows/release.yml` fails a tag that isn't `v` + that version, gates
  on `rust.yml` (called whole, via `workflow_call`), builds on `ubuntu-24.04` (the
  libc floor) with `build-deb.sh` (which asserts whisper's `-mavx2`), smoke-tests,
  and attaches the `.deb` to a GitHub Release. To check the
  pipeline without releasing, `gh workflow run release.yml --ref <branch>` runs
  everything and publishes nothing but a workflow artifact — **but only once
  `release.yml` exists on the default branch**; GitHub refuses to dispatch it
  otherwise (`HTTP 404: workflow … not found on the default branch`). Until then,
  a temporary `push: branches: [<branch>]` trigger does the same job; the release
  job requires a tag ref, so a branch run cannot publish. The `package` job uses
  no build cache on purpose (a cached whisper build can outlive an `[env]` change).

**Crate layout:**

| Crate | Holds |
|---|---|
| `pundit-core` | Pure logic: project format, playback timeline, zoom, stroke replay. |
| `pundit-media` | GStreamer: source player, capture, export frame driver, overlay rasterizer. |
| `pundit-app` | Slint UI, command bus, event layer. |
| `pundit-harness` | Headless integration tests driven over the bus. |

**`pundit-core` declares no media dependency** — not GStreamer, not an image
or font crate, not a feature that pulls one in. CI runs its tests on a runner
with no GStreamer installed, so adding one fails the build rather than passing
silently. If you need a media type in core, you need a different design.

**The project format reads `MIN_READABLE_FORMAT_VERSION..=CURRENT_FORMAT_VERSION`**
(`store.rs`; v7, the first the port wrote, onward). Below it is a macOS file
(`LegacyProject`), above it `TooNew`. See spec F in
`docs/superpowers/specs/2026-09-22-match-vision-design.md`.
- **Every phase that stores a new field bumps the version once**, even though
  serde ignores unknown keys: an older build would ignore them too, and drop them
  on its next save. The bump makes it refuse the file instead.
- **A field added to an existing struct is an `Option` or a `Vec` with a
  field-level `#[serde(default)]`**, which is exactly what an older file means. A
  new struct's fields get no default: a missing one is a malformed file. Never a
  field-level default on an `f64` or a `bool` (`project.rs`'s header).
- **Every bump comes with a test that every readable version still loads**
  (`project_format.rs::v7_to_v9_files_load_under_the_current_version`, and one
  per bump beside it).
- **v10 adds `Project.avatar` and `Clip.inset`.** `avatar` is the image's file
  name in the project folder and *is* avatar mode — `Some` means takes record
  commentary only, `None` means they record on camera, and there is no second
  flag to disagree with it. `inset` is what a clip was recorded with, defaulting
  to `Camera`, which is what every v7–v9 clip was. Both are additive, so the
  readable floor stays 7.
- **v11 adds `Preferences::last_export_scoreboard`** (the export sheet's
  Scoreboard picker, `None` = Default). A field added to `Preferences` takes
  **no** attribute: that container carries `#[serde(default)]` and fills from
  its hand-written `Default` impl, so a field-level one would be a second copy
  of the default.
- **v12 adds `Project.slates` and `Clip.slate_id`** — a range marked while
  watching, and the take shot from it.
- **v13 adds `Clip.inset_size` / `Clip.inset_corner`** — where the inset goes and
  how big, per clip — **and `Preferences::last_inset_size` / `last_inset_corner`**,
  the sticky last-used pair a new recording inherits. The two halves take
  **opposite serde rules and getting them the wrong way round is a bug**: the
  `Clip` fields take a **field-level** `#[serde(default)]`, because their
  `Default` (Medium, BottomRight) is exactly what an older file means; the
  `Preferences` fields take **none**, because that container already carries one
  and fills from its hand-written `Default`, so a field-level one would be a
  second copy of the default. Both are additive, so the readable floor stays 7.
- **v14 adds `Stroke.end`** (#117): `StrokeEnd::Plain` or `Arrow`. An **enum, not
  an `arrow: bool`**, and it takes a **field-level** `#[serde(default)]` for the
  `Clip` half of v13's reason — `Plain` is exactly what every v7–v13 stroke was,
  the same grounds `Inset` defaults to `Camera`. An enum is also what says that
  in the type, where a defaulted `bool` would be leaning on the letter of the
  rule against its stated reason, and it leaves room for an end that is neither.
- **v15 renames `Preferences::preview_source_volume` to `export_source_volume`**,
  with `#[serde(alias = "previewSourceVolume")]` — the export sheet's "Mute
  source audio" writes `0.0` or `1.0` into it. The old name was always wrong: a
  preview carries no source audio at all (#125), so that field has only ever
  reached the file. The alias is matched **verbatim** against the camelCase key
  the container's `rename_all` produces and is deserialize-only, so a save
  writes one key and the old spelling self-cleans out. **The bump's reason is
  the copy path, not the rename:** an older build's `carry_scoreboard` consults
  no stored field at all, so it would stream-copy a muted whole match *with its
  sound* whatever the key were called. (Had the key been kept it would at least
  have honoured the mute on the encoded path, reading gain `0.0` — which is why
  "an older build falls back to 1.0" is only half the story.) It takes **no**
  field-level `#[serde(default)]`, for `Preferences`' usual reason, and so keeps
  its place in `preferences_defaults_are_not_zero`.
- **v16 adds `Preferences::last_export_chapters` / `last_export_cues`** (#78) —
  the export sheet's two output switches, both **`true`** in the hand-written
  `Default`, both taking **no** field-level attribute for `Preferences`' usual
  reason (the container carries one), and both keeping their place in
  `preferences_defaults_are_not_zero`. Additive, so the readable floor stays 7.
  **The bump's reason is the forward direction**, which is the one that bites:
  serde ignores unknown keys, so an older build would read a v16 file, ignore
  both switches and drop them on its next save — exporting the chapters and the
  `.srt` a coach had turned off and saying nothing. The bump makes it refuse the
  file instead, and `project.json.v15` is what it can still be gone back to.
- **The first save after an upgrade keeps `project.json.v<old>`**, once, never
  overwritten, so the older build can still be gone back to. It is copied to a
  temporary name and renamed, like `project.json` itself, so a failed copy
  leaves no backup to block the next try; never a hard link (exFAT and FAT
  have none).

**A project folder is created by ONE command and never by a sequence**
(`Command::NewMatch` → `bus/project.rs::new_match`; spec
`docs/superpowers/specs/2026-09-24-new-match-flow-design.md`). The reason is the
aspect gate: it fires **between** sources, so as `OpenProject` + `AddSource` × n a
second half whose shape differs is refused *after* the first has been probed,
pushed, saved and published — leaving a named folder holding one of a game's two
halves, and the folder name is the one thing that flow cannot correct afterwards.
So nothing touches the disk until every video has been probed and accepted, and a
refused Create leaves the coach in the project he was in.
- **An existing folder is *adopted*; the refusal is keyed on `project.json`**, as
  `open_project` distinguishes the two. Keyed on `create_dir`'s `AlreadyExists`
  instead it would refuse the empty stranded folder a coach made by hand, which is
  one of the things the flow exists to fix. The two rules are tested **apart**, on
  purpose: a change that fails both has broken the command rather than proven the
  rule.
- **`create_dir`, never `create_dir_all`.** At most two directories are ever made
  — the projects folder's leaf and the match folder under it — so a typo in the
  hand-editable projects path leaves one stray directory under a folder that
  already existed rather than a tree.
- **The pre-disk gate is `project::aspects_match`,** the pairwise rule;
  `Project::check_aspect` is the stored-project wrapper over it and returns
  `Ok(())` when there is no stored source, so it would gate **nothing at all** on a
  project that does not exist yet.
- **The projects folder is found by walking up for `core::metadata::APP_NAME`** —
  the coach's own pattern, a `pundit` folder with projects inside it — four
  candidates, stopping at the filesystem root or at home inclusive. Then the
  folder the last project was in (**checked to exist**: `last_project` is a stored
  path, and an unchecked one silently re-creates a projects folder the coach
  deleted), then an app-named directory *beside* the footage, which is the one tier
  that carries a provenance line.
- **A project's name is `<Home> v <Away>`, and the bus builds it** from the
  scoreboard it has to validate anyway: `bus::scoreboard::storable` answers "may
  this be stored" and "what is it called" in one call, because they are one rule
  over the same two strings. `Command::NewMatch` carries **no** name field. Two
  spellings of it is not hypothetical — an untrimmed `!= ""` in the sheet against
  `match_name`'s trim left Create enabled, sending nothing, and dead until the
  coach cancelled and re-picked every video.
- **`core::naming` holds a name's rules, and `safe_chars` and `folder_slug` are
  deliberately not one pipeline.** `safe_chars` replaces the nine characters a
  share refuses plus controls and does **nothing else**; an export's file name and
  a basket film's stem take it, because an export's basename is what its `.srt` and
  `.chapters.txt` are derived from, so collapsing, trimming or truncating would
  orphan the sidecars beside files already written and make two long labels collide
  *after* de-duplication had found them distinct. `folder_slug` is the whole
  pipeline and only a folder takes it. `truncate_on_boundary` is shared; the
  budgets (200 for a film, 64 for a folder) are each site's own.
- **`naming::parse_date_in` takes its year bound as an argument** because core has
  no clock — and note that a `SystemTime::now()` in core would sail through the
  dependency audit, since a clock adds no dependency. That signature is the only
  thing enforcing the rule.
- **The flow stores nothing of its own and bumps no version.** No `state.json` key
  (W2 reads the `last_project` #85 already keeps) and no field on any stored struct.

**One avatar image per project, copied into the project folder.** It is
`<project>/avatar.<ext>` beside `project.json`, and `Project.avatar` holds its
**file name** — which *is* avatar mode (the format rules above).
- **Picking one is decode, copy, save** (`bus/project.rs::set_avatar`): the file
  is decoded first, so one that will not decode is refused before anything is
  copied; the bytes are then copied through a temp file and a rename in the same
  directory, as `store::write` writes `project.json`. Replacing one deletes
  **exactly the file `Project.avatar` names**, never an `avatar.*` glob over a
  folder the coach can also put files in. **Remove** deletes the project's copy
  alone — the coach's original is theirs.
- **`media::decode_still` is the one avatar decoder, and `media::avatar_drawn`
  the one rasterizer.** The pick validates with the decoder; the drawn pixels —
  cover-cropped, circle-masked, premultiplied — are made in one place and taken
  by the render, the Devices popover's thumbnail and the corner during a take
  alike, so a file that passes the pick cannot fail in an export, nothing has to
  keep a list of extensions in step, and the coach is never shown something the
  file won't get. The decoder runs
  `decodebin3 ! videoflip video-direction=auto`, which is what keeps an
  EXIF-rotated phone photo upright (`probe` *refuses* a rotated **source**
  instead: there a timeline and a stored aspect are at stake). One sample is
  pulled, so a multi-frame file yields its first frame and no error. The file
  dialog offers PNG and JPEG; what is accepted is what decoded.
- **GStreamer's `RGBA` is straight alpha and tiny-skia is premultiplied**, so
  the copy into the avatar's pixmap multiplies R, G and B by A — a memcpy leaves
  a cut-out PNG haloed. The pixmap is pre-scaled to the **square**
  `core::layout::avatar_box(out_w, out_h, placement)` — the clip's own size and
  corner (v13) — and masked **once, at open time** to the circle inscribed in
  it, so the per-frame draw is a plain blit. **Square whatever was
  picked, and the image cover-cropped into it** (scaled until its shorter side
  fills the box, then centred): what is drawn is always a circle, so a box of
  the image's own aspect would put that circle somewhere other than where the
  webcam inset sits — a 3:4 portrait's about 240 px above the corner at 1080p.

**Bus contract — caller-captured timestamps.** Any command that lands in the
commentary event log carries its timestamp (and source-position anchor) as a
field, captured at the input event on the UI thread, never assigned by the bus
handler. Queue delay would reintroduce the drift that puts drawings behind the
ball on replay. Querying position on a running pipeline is the only direct
pipeline access permitted outside the bus task.
- **One documented exception, and it is the shape of the rule rather than a hole
  in it:** the slate out-point stop (`bus::slates::stop_at_slate_out`) mints
  `now_ns()` itself. The contract is about the **queue delay between an input
  event and the handler that stamps it**, and a crossing has no input event —
  the poll *is* the event, so nothing was queued, and the time wanted is when
  the picture stopped, which is now. What keeps it honest is that the
  **position is still not a reading**: the pause is anchored at the stored
  `out_seconds`, exactly as `shoot_slate` seeks to a stored `in_seconds`. A new
  bus-minted timestamp has to clear that same bar — no queued input, and an
  anchor that is stored rather than queried. `transport.rs` declines to log a
  pause at a player error and at EOS, and **the reason there is not a bus-side
  clock either** — it is that neither has anything to anchor to; both comments
  used to say otherwise and were corrected with this change.

**Pixel work split.** GStreamer owns every full-frame pixel operation, on the
GPU. Rust owns the edit (which decoded frame lands at each output PTS) and the
vector overlay layer only. This is measured, not preferred — see
`docs/superpowers/spikes/2026-09-19-compositing-throughput.md`. Do not move
full-frame resampling into Rust.

**Decode path stays zero-copy, which needs `decodebin3` AND an EGL context.**
Use `decodebin3` (or `playbin3`) with the video stream selected by caps
(`video/x-raw(ANY)`), or an explicit `demux ! parse ! <hw decoder>` chain —
never `decodebin`. And the GL context must be **EGL**: on X11 GStreamer defaults
to GLX, where 1.24's DMABuf importer is unavailable and every frame is copied
through the CPU (~11× slower; seeks ~5× slower). In the app the EGL context is
Slint's Skia renderer's, shared with GStreamer.
Verify on real hardware with `scripts/linux-gate-check.sh <file>`; see
`docs/superpowers/spikes/2026-09-19-seek-latency.md`.

**A load that stalls is issued again, because GStreamer's sometimes do.**
`playbin3` occasionally never builds `playsink`'s chains, so a load's
READY → PAUSED never completes — measured on 1.24.2 (Ubuntu 24.04's, and
CI's) with four soaks pinned to **two** CPUs: `uridecodebin3` prerolls whole
and posts its `StreamCollection`, while `playsink` sits at
`Async/Ready/Paused` holding only its `audiotee` and `streamsynchronizer`.
**Two cores is the ingredient**, not load: on eight, with eight busy loops,
~6000 opens never showed it. **It is not our sink, our load sequence or our
dropping of the bus's messages** — each was swapped out two-core and the stall
stayed (BACKLOG #72's three controls); at a stall no sink is in the graph yet,
ours or any. The load's `ASYNC_DONE` is then never
posted, so `Flight::Loading` waits for a message nobody will send and nothing
recovers it (16 stalls, not one by itself, five of them given a full 40 s):
in the app a project that opens on a black frame with the scrubber stuck at
0, for ever, and in the harness BACKLOG #72's "timed out waiting for a
settled position".
So the bus bounds a load — `sources::LOAD_BOUND`, armed and cleared by
`watch_the_load` in the loop's tail — and `SourcePlayer::reload_stalled_load`
issues it again, **once**; a second stall is reported as a pipeline error
rather than retried for ever. The bound covers the load **alone** and never a
load the GL gate is holding (`SourcePlayer::loading()` is where both
exclusions live), and **the retry's bound is armed after the reload, never
before** — the take-down inside it spends up to `LOAD_SETTLE`'s 5 s letting a
preroll finish, so a bound armed first is already in the past and the retry is
taken for a second stall. Recovery costs about 8 s, which is what sets the
bound at 3 s: 3 + 5 + a load has to fit inside the harness's 15 s wait.
`player::tests::every_open_settles` is the `#[ignore]`d soak that produces the
stall (pin it with `taskset -c 0,1`, run several at once), and
`PUNDIT_SOAK_SABOTAGE=1` leaves the retry out, which is the measurement the
fix is argued from.

**Capture records on the system clock, from time 0 = `base_time`.**
- **Sources:** the camera is `v4l2src`, for kernel timestamps and the `exposure_dynamic_framerate=0` control that stops low-light drops to 7.5 fps. The mic is `pipewiresrc`.
- **Clock:** the recorder always forces `SystemClock` (CLOCK_MONOTONIC). `pulsesrc`'s clock was measured days off.
- **Time 0:** `matroskamux` writes running time as-is, so recording time 0 is the pipeline's `base_time`, read when `set_state(PLAYING)` returns. **Never wait for PLAYING:** the mux holds preroll until the camera's first frame.
- **Event times:** `host_ns` comes from `pundit_media::now_ns()`.
- **An avatar project records audio only.** `Project.avatar.is_some()` is the mode, so no camera is opened, no `video_%u` pad is requested on the mux, no H.264 encoder is chosen (a machine with neither VA-API nor `x264enc` still records commentary) and there is no self-view pipeline. `CaptureSources`' video side is `Option` on **both** arms, and `capture_sources`' early return for `CaptureKind::Test` honours the mode too — a branch written only into the `Devices` arm would leave every test recording with video whatever the project said.
- **`RecorderMessage::FirstBuffer` is the first-buffer gate** (not `FirstVideo`): a buffer reached the muxer, so there is a file worth keeping. It comes from the one pad there is — video where there is one, audio otherwise — and the start timeout says which was missing.
- **The `level` element carries two numbers and the recorder passes both on.** The meter draws `peak_db`; the avatar pulses on `rms_db`, because speech's 10–14 dB crest factor would peg a peak-driven inset while the export barely moved.
- **The corner during an avatar take is the project's image, pulsing.** It is the copy the Devices popover decoded (`media::avatar_drawn`, once per change of the file behind it — never at the start of a take, where a decode with a ten-second bound would sit on the UI thread the instant the coach presses R; and never `slint::Image::load_from_path`, since slint is built with no image decoder) and sized by the **live** estimator: `level`'s `rms_db` through `core::avatar::level_from_db`, smoothed at `dt = 0.1` — the message interval — by the same `core::avatar::smooth` the render uses at `1/30`. Previews and exports use the **rendered** estimator instead (RMS per output frame, read from the recording). Two estimators of one quantity, same constants, nothing persisted. `place-self-view` carries the level, **which kind of take it is, and the size and corner the next clip will get** (v13 — the two *preferences*, not a clip's fields: there is no clip during a take, so what it honestly shows is where the inset will land). There is one placement rule per kind: a camera take is `self_view_rect` → `pip_rect_over_picture` exactly, an avatar take is `avatar_self_view_rect` → `avatar_rect(avatar_box(w, h, placement) over the picture, level)`, the render's own two functions. **The self-view's quiet timer is the camera's alone:** an avatar take has no frames arriving and must not be hidden by it.
- **Tests:** they use injected test sources (`CaptureKind::Test`) and never the real camera or mic. See `docs/superpowers/specs/2026-09-19-linux-port-phase-4-design.md`.

**Export runs one GL graph everywhere, on its own GL display.**
- **The graph:** decode (`decodebin3` → the player's `gl_bin` → pull `appsink`) → a Rust pump → `appsrc` → `gltransformation` (zoom) → `glvideomixer` (letterbox, pinned to 1920×1080@30) → NV12 `gldownload` → encoder → `mp4mux`.
- **The GL display:** process-wide and surfaceless (`GLDisplayEGL::new_surfaceless()`), never the UI's. CI has no GPU, so Mesa's llvmpipe runs the same graph; there is no software variant.
- **Picking source frames:** use **stream time** (`segment.to_stream_time`), not raw PTS: MP4 edit lists offset raw PTS. Round seconds to ns (`seconds_to_clock`). Seek `KEY_UNIT|SNAP_BEFORE`, then pull forward: ACCURATE seeks drop frames in VFR or gapped files.
- **Quality is a constant quantizer, because nothing else is on offer.**
  `vah264lpenc`'s `rate-control` enum has exactly one member on the reference
  driver (`gst-inspect-1.0 vah264lpenc`); `rate-control=cbr` and `=vbr` fail to
  parse. `bitrate`, `target-usage` and `b-frames` exist as properties and are
  measurable no-ops, and `trellis=true` **doubles** the file. So there is no
  bitrate target and no size ceiling — a busy passage costs what it costs.
  Low/Medium/High are **VA QP 30/26/22**, and `x264enc` gets **QP − 4**
  (26/22/18), which matches the VA encoder's SSIM within 0.001. `x264enc` also
  needs **`vbv-buf-capacity=0`**: it hands `bitrate`'s 2048 kbit/s default to
  libx264 as a VBV maximum even in constant-quality mode, which silently capped
  every software export at ~1.7 Mbit/s. The ladder and its bitrates are
  tabulated on `quantizers` in `composite/export.rs`; `the_quality_ladder_reaches_both_encoders`
  pins it.
- **Never block a push or pull without a bound.** A blocking `appsrc` push hangs forever after a downstream error.
- **To test CI's path locally,** hide the GPU with `GST_REGISTRY=<scratch>/reg.bin bwrap --dev-bind / / --tmpfs /dev/dri cargo test …`. See `docs/superpowers/specs/2026-09-19-linux-port-phase-5-design.md`.

**The speakers are `autoaudiosink` with `pulsesink` demoted.** `keep_pulsesink_out()` drops `pulsesink`'s rank process-wide at `Bus::spawn`, so `autoaudiosink` picks `alsasink`, which reaches PipeWire through `pipewire-alsa`. Against Ubuntu 24.04's `pipewire-pulse` (PipeWire 1.0.5), `pulsesink` wedged the stream permanently after a quick burst of flushing seeks while playing — a dragged scrubber or a held skip key — and since it supplies the pipeline clock, picture and position froze with it (journal: `pipewire-pulse … [pundit]: stream … OVERFLOW`). Measured A/V offset is unchanged (~+1 ms, audio leading). **The test harness's `Harness::production()` runs the app's exact path** — the GL sink on a surfaceless display plus the real `autoaudiosink` — because the default harness (`fakesink` audio, silent WebM fixtures) can never reach the sound server, which is why this escaped. Real-footage checks are `#[ignore]`d: `PUNDIT_FOOTAGE=/path/to/game.mp4 cargo test -p pundit-harness --test real_footage -- --ignored --nocapture`.

**Every key the app binds is one row in one table** (`pundit-app/src/keymap.rs`, BACKLOG #96): 29 actions, 34 default bindings, overridden per machine by `state.json`'s `keys` — which holds the **diff and not a copy**, so deleting that key is the reset path and a default this build changes still reaches a coach who never touched that action. `handle-key` in `app.slint` asks `action-for` once per key event and acts on the `KeyAction` it answers with; `key_action.rs` is the `match` between the two, and it is a module of **both the binary and `tests/ui`** so `tests/ui/keys.rs` presses the shipped map rather than a copy of it. **A key is spelled in `keymap.rs` and nowhere else** — where a paragraph below names a letter it names today's *default*, to be read, not a binding to be edited. The far skip is its own pair of actions rather than a Shift read inside the near one, and the cost is named: a shifted **letter** now fires nothing of its own, so Shift+R no longer records and `Ctrl+Shift+Y` no longer redoes (`Ctrl+Y` still does).

**Context stays in three layers and none of them is the table.** Whether the window sees a key at all is `text-editing` — Slint's own `TextInputInterface.text-input-focused`, tested ahead of every binding. Whether an action fires *now* is the gate **in its branch** (`can-tag`, `can-draw`, `can-play`, `!event.repeat`), which is why a row carries no `when` clause: a clause makes a binding *not match*, so a gated key would fall through to whichever child has focus and `←` would reach a touched volume slider in exactly the case `can-play` is false. Whether the bus will do it while recording is the **recording allow-list**, a `!matches!` over `Command` variants — and that is **not reachable from a rebinding**: a rebound key sends the command it always sent, and a command added later is still refused by default. Two more rules the rewrite rests on. **Escape, Tab, Return, Home and End are not actions**, and the three the window handles are tested *ahead* of the lookup, so a hand-edited `state.json` cannot claim one and leave a sheet unleavable. And a bound key is swallowed **on release as well as on press**: `handle-key` runs for both, because a slider fires `released` on an arrow key's release — so the lookup is ahead of every `pressed` test and every `accept` is outside its guard.

**The Keys sheet is the seventh `Sheet`, and it is a list** (#96 task 4). `F1` opens it — that is `Action::ShowKeys`' own default, listed in the table like every other key — and so does the **Keys** button, which sits in the *drawing* row beside `Fit` and for `Fit`'s reason: the transport row is full at the window's 1100px minimum (measured, `Devices…` ends at x=1199, 99px past the right edge). Its rows are `Keymap::listing()` set on `key-rows` in `wire_keys`, **beside `action-for` and nowhere else**, so the list and the lookup are one map read twice and the list cannot go stale. **The six reserved rows — Escape, Tab, Shift+Tab, Return, Home, End — are written in `app.slint`, not in the table:** they are not `Action` variants, so `Binding`, `overrides()` and a rebind would each have to filter them back out of a table whose whole point is that every row in it is rebindable, and their words belong beside their behaviour — for the three the window handles, `handle-key`'s own reserved block in the same file. The rows are capped in a `ScrollView { height: 520px; }` because `Sheet` is `body.preferred-height` and **does not scroll** (#134): 29 rows at 22px is 638px and the card has to fit a 700px window, which it does at 666px. Rebinding is task 5 and nothing in the sheet anticipates it.

**Stepping one frame is `Command::StepFrame`** (`,` and `.` by default, the arrows skip), and it is refused while playing, recording or previewing. A step works from the shown frame's *end*, which a seek never clips: forward seeks to it, back to half a nominal frame before the frame's nominal start, or its own start where that is earlier (a frame held long). The readout shows tenths while paused, whole seconds while playing.

**Fitting the window to the footage** (BACKLOG #95; `F` by default): it only ever **shrinks**,
so the picture is never re-fitted — the window closes up around the picture
already on screen and the letterbox bars go, which is also why it can never put
the window off a screen whose size Slint will not report. Every input is read
off the window rather than derived from it: the chrome is `window − player` and
the picture is `content-width`/`content-height`, so a new transport button or a
dragged panel cannot put it out of date. `fit.rs` is the one place the
arithmetic lives, and it answers three outcomes because the refusal has to be
reachable — with two, the value that offers the action is the value that refuses
it, and the notice could never be shown. A maximised window is un-maximised
first and fitted on a later tick, watching the window's **size**: `is_maximized()`
goes false the instant `set_maximized` writes it. Fullscreen is refused outright,
because un-maximising cannot leave it.

**The two side columns are resizable** (BACKLOG #87): a 6px splitter either side
of the player, `ui/splitter.slint`, with the widths in `state.json`. Two things
about it are load-bearing and neither is obvious. The columns' bound is
**layout constraints** (`min-width` + `preferred-width`/`max-width` +
`horizontal-stretch: 0`, the player taking the stretch), not a
`width: clamp(stored, min, root.width − other − player-min)`: measured, the clamp
form put the player 12px **under** its own minimum, because each column computed
its headroom from the other's *raw* width and neither knew about the splitters —
and reading `root.width` from inside a child of the row it feeds also risks a
binding-loop deprecation that `-D warnings` fails on. And the drag **anchors on
the press** (`absolute-position.x + mouse-x`, invariant as the grip moves) rather
than accumulating `mouse-x − pressed-x`, which is what Slint's own
`tableview.slint` does and what sticks against a bounded consumer. **The panels
grow but do not shrink**: 280px is the width the inspector's transcript row was
fitted to. There is no keyboard path to them (#99).

**Every sheet is capped at the window and scrolls its body** (BACKLOG #134):
`Scrim` wraps its one child in a centred `VerticalLayout`, so Slint's box layout
hands the card `min(content, window)`, and `Sheet` puts its body in a
`ScrollView`. **A `Sheet` therefore states its height as
`preferred-height`/`max-height` and never as `height`** — an explicit height is
*fixed* in that layout and could not be capped, which is how the export sheet's
Export and Cancel came to sit off the bottom of a 700px window. The same goes
for `MatchSetupSheet`, the one sheet that *wraps* a `Sheet` rather than being
one (the colour picker's popup hangs off that wrapper). **A sheet's buttons and
its title scroll with the body**, which is the accepted trade: each sheet puts
its own action row in `@children`, and a pinned footer would mean a second child
slot on `Sheet` and an edit to all six. The keyboard is untouched, because
`handle-key` is `capture-key-pressed` and a `Flickable` reads no keys at all.
One thing it costs a test: `ElementHandle` skips anything clipped away, so a
button at the bottom of a tall sheet has to be flicked into view before it can
be found (`flick_to_the_end` in `tests/ui/sheet_scroll.rs`). It also **retired a
cap written hours earlier**: the Keys sheet's 29-row list had its own 520px
`ScrollView` for exactly this problem, and leaving it would have shown a coach
520px of rows in the middle of a 1200px card and scrolled in two places at
once.

**The scan speed is `Command::ScanSpeed(ScanStep)`** (`J`/`L` by default: the bus steps 1×–32×, while scanning only; any pause returns to 1×, so a recording starts at 1×). The player owns the rate, and **every scan seek carries it** (`pipeline.seek(rate, …)`, never `seek_simple`, whose 1.0 would drop it on the next scrub or skip). `set_rate` issues no seek: the bus does, through `load`, unless a seek still to be issued will carry it; after a pause it seeks to the frame on screen, not the position the picture trails at speed. Opening a preview returns to 1× with no seek. Every frame is decoded even at 32× and the scan sink's QoS (`max-lateness` 20 ms) drops what's late: measured on 1080p30 H.264, that showed 88 fps at 32× against key frames only's 16, with a tenth of the lag.

**Preview and export share one composite** (`pundit-media/src/composite/`).
- **Common:** `decode.rs` (`Decoder::frame_at`: reuse, pull ≤0.5 s, else `KEY_UNIT|SNAP_BEFORE` and walk forward), the pump, `frame_time`/`stamp`, `install_zoom`'s PTS-keyed probe, and the mixer geometry.
- **Tails:** `export.rs` encodes as fast as it can on a private surfaceless display; `preview.rs` ends in a `sync=true` appsink filling the shared `FrameMailbox`, on **Slint's** GL context (chosen by sink kind: the app never falls back to a private display, and tests pass `Gl::shared()`).
- **Preview's pads:** the pumped source through `gltransformation`, the recording played **natively** for PiP and commentary audio (record time *is* output time, so it needs no pump), and a second appsrc carrying the overlay.
- **The overlay rasterizes at the picture rect,** not the output frame: strokes are normalized to the content rect.
- **Measured:** 30.005 fps on 1440p HEVC, audio leading the picture by 2–7 ms. Don't measure the rate first-frame-to-last-frame; the mixer flushes its tail late. The UI budget is relative to a scanning control in the same session, not to an idle window.

**Export burns in the overlay and mixes the audio** (Phase 8).
- **Layers:** the pumped source (zoom, per-entry fit rect), then the inset, then one output-size overlay carrying strokes (mapped into the picture rect), the text bar and the scoreboard **over both**. The z-order and its reason live in `composite::install_overlay_pad`: the coach's pen is what must never be hidden, so nothing is ever mixed over the overlay. Pad rects are **PTS-keyed in probes**; set from the pushing thread they land up to `QUEUED` frames early.
- **Both furniture pieces are flush into a corner** (the coach, 2026-09-25): the scoreboard is locked into the top-left, and the inset goes into **whichever of three corners its clip names** — bottom-right (the default, and what every clip was before v13), bottom-left or top-right. **Top-left is not offered**, because the board is drawn over the inset and a coach choosing it would get a half-hidden face. The caption bar **stops where the inset stands**, background and line alike, so the bar's 60% black never washes over the coach's face and a caption — ellipsized, never shrunk — never runs under it. `layout::bar_rect` is the whole rule and it takes `Clip::inset_placement()`, an **`Option`** — **never `shows_inset`**: the accessor folds `show_pip` in and is `None` when no inset is drawn, so `clip.map(…)` would cut *every* clip's bar at a column nothing stands in, in export and preview alike. The bar's edge and the inset's come from one function (`layout::inset_span`), which is why they meet exactly rather than by two arithmetic coincidences.
- **The PiP pad is fed every frame,** with a **GL** 1×1 transparent filler when a clip has `show_pip` off or its recording is unusable. An unfed pad stalls the run, and a system-memory filler breaks `glupload` when a later entry has a real inset.
- **The avatar is that same inset pad, never the overlay.** An avatar clip's inset is the project's image: decoded and pre-scaled once **per image and `InsetSize`**, uploaded to GL once each (the filler's own hop, for the filler's own reason) and pushed as a re-stamped header over that one texture per frame; preview, whose recording has no video pad to play, feeds the pad from an `appsrc` of its own. **The texture is keyed on `(path, size)` and carries no rect:** the pixmap is square and sized by the size alone, so entries share it across corners, and **each entry's rect is built in `Pip::open` from its own clip** — carried along with the texture, one run's avatars all landed where the first one stood. **The avatar's box is `layout::AVATAR_BOX_RATIO` (0.75) of the webcam inset** at the clip's own size, square and flush into the clip's own corner, so it keeps that corner's margins and is simply smaller (the coach, 2026-09-23). `core::layout::avatar_box` is the whole of it, one pure function on the avatar paths only, and retuning the size is that one constant. **It is a ratio on a ratio, never a rect shrunk about a corner:** shrinking about the bottom-right only worked in the corner it was written for — applied to a bottom-left inset it drifted 105.6px off the left edge at Medium/1080p, and to a top-right one it hung the same 105.6px below the top, the same number because the box is square. The pulse is the **pad's rect**, `core::avatar::avatar_rect(box, level)`, set in the PTS-keyed probe from one level per output frame built at job setup from the recording's own audio — never in the frame loop, where an audio decode would stall the pump — and **bounded by the entry**: the reader is asked for exactly the samples the entry's frames cover, so a two-second entry of an hour-long take reads two seconds. `avatar_rect` is exactly the rect it is given at level 1.0, which is what every non-avatar frame carries, so a camera export is unchanged to the integer. Both raster pads blend `blend-function-src-rgb=one`: what they carry is a premultiplied tiny-skia pixmap. **Measured, and the reason:** drawing the inset in the overlay costs 4.2–4.8 ms a frame against the 3.2 ms the whole overlay costs — `tiny_skia` has no sprite fast path (`the_avatar_blit_costs`, `#[ignore]`d in `media/tests/avatar.rs`).
- **`Clip::camera_placement()` and `avatar_placement()` are the only readings of `show_pip × inset`, and each answers *where* in the same breath as *whether*.** `Some` means that kind of inset is drawn and carries its size and corner; they are never both `Some`, and there is **no bare predicate beside them** — every call site wants the placement the moment it knows it has one, which is why #88's review pass deleted `shows_camera_pip`, `shows_avatar` and `shows_inset` rather than leaving five sites to pair a bool with an accessor. (`inset_placement()` is the third reading and is **kind-blind**: it is `bar_rect`'s argument alone. The inspector's pickers read `inset` on its own, deliberately — `show_pip` is the checkbox beside them, not part of the question.) **A third `Inset` variant would need a third accessor:** nothing matches `Inset` exhaustively, and both media tails fall back to the filler or to no pad — safe, but silent. Preview reads the camera answer **once** and threads it to **three** users — the launch string's branch, the pad's placement, `decodebin3`'s pad-added link — which must agree or the mixer stalls; one answer rather than three reads is what makes them agree. And it includes a **probe of the recording**: `show_pip` says the coach wants an inset, but an avatar take's file has no video track and neither has a webcam take whose camera died. Export probes before it opens a decoder and falls back to the filler; preview has no filler and asks for no pad at all.
- **Audio:** one audio-only pipeline per file (flushing ACCURATE seeks per play segment, silence for a file with no audio), mixed in Rust from `core::audio`'s regions and envelope — and **`audio_regions` takes the source volume as an argument**, never reading it off the `Preferences` it is also handed: at `0.0` it emits **no game region at all** rather than a gain of zero, so nothing opens a reader to decode samples it would multiply away, and `bus/basket.rs` can keep mixing at `Preferences::default()` on purpose and still mute. The commentary's gain is still read from the struct (#124's remaining half) — pushed **at or ahead of** the video into an **unbounded** appsrc, then `avenc_aac` (needs `gstreamer1.0-libav`). **Drop the first 1024 samples** for the encoder's priming; shifting timestamps does nothing. A tone at 1.000 s must decode back within a millisecond.
- **Every denominator is `plan.total_frames()`,** never a duration sum: per-entry quantization can add a frame per entry.
- **A run is started by `begin(jobs)`** over jobs the caller built: the caller owns its own refusals (`refuse_if_busy` first, before any I/O), its own output directory and its own picker write-back. `begin` is **infallible** — by the time it is called there is nothing left to refuse — and the write-back still happens after it, so nothing on the way to a refusal can dirty the project.
- **Chapters are a hand-written `chpl`** (`media/src/chapters.rs`), one per plan entry (`CompilationPlan::chapters`, none under two entries), spliced into the reserved `moov` by shrinking the `free` after it, on the `.part` before the rename. `mp4mux` has no `GstTocSetter`. A chapter starts at `start_frame / OUTPUT_FPS`, never at a duration sum. **Chapters never cost an export:** any layout problem found before the write (no room, no `moov`, a box that doesn't fit) keeps the file whole without chapters, and `bus: exported …` says why. Only an I/O error opening the file or in the positioned write itself fails it.
- **A reel's chapters are worded twice, and mark the periods** (`core::reel::reel_plan`). The chapter reads as prose — `Goal 3 — Rovers 2-1` — where the bar burned across the picture reads `3 / 6 | Rovers goal | 2-1`; the same split as the Match panel's `labelled_events` and the whole match's `chapter_events`, and the two are meant to differ. Where the goals cross a period the boundary is marked **on the chapter already there**, as a prefix (`Second half: Goal 3 — Rovers 2-1`): a reel's entries run back to back, so a chapter of its own would share an instant with the next goal's — a zero-length chapter in the MP4, and a goal dropped by the ten-second rule in the pasteable list. A reel's entries and chapters are therefore built together, unlike every other target's, because the chapter needs the goal its entry was cut around.
- **`ffprobe` is the chapter test's reader** (`qtdemux` doesn't read `chpl`), so `ffmpeg` is a **test-only** build dependency: the test fails without it, never skips, and the `.deb` doesn't depend on it.

**Every exported MP4 says what it is, in its header** (`core::metadata::file_tags` → `media/src/composite/tags.rs`). The words live in **core**, beside the chapter and caption wording, as one pure function of the project, the export target and a date; media only sets them on `mp4mux`'s `GstTagSetter` before `PLAYING`, on **both** renderers. `ExportJob::tags` carries them, and `FileTags::default()` is an untagged file.
- **What actually reaches the file, measured on GStreamer 1.24.2** (mux, then `ffprobe -show_format`): `title`, `comment` (the final score), `keywords` (both team names, which GStreamer joins `", "`) and `encoder` (`pundit <version>`) land in `moov/udta` and read back by name; `date` lands in `©day`, unpadded, as `2026-9-21`; **`description` lands only in the XMP `uuid` box** as `<dc:description>` — `mp4mux` writes no `desc` atom, and nothing else on this muxer carries a description. **`datetime` is ignored**: it sets neither a tag nor `mvhd`'s creation time, so the date is a `glib::Date`.
- **The date is the footage's, not the export's** (a user decision): the first source file's mtime, read in the bus (`source_date`) because the bus knows the paths and media has no business stat-ing files. Resolved in the local zone there and handed to core as a `CalendarDate`; no date available writes no date.
- **The tag merge mode is `Keep`.** The encoder pushes an `ENCODER` tag of its own ("x264") into the same muxer; `Keep` is what leaves ours standing.
- **Tags cost the copy no losslessness and the chapters no room** (measured). They are header boxes beside the tracks, so not a sample changes; and `mp4mux` grows the reserved `moov` to fit them rather than spending the sample-table headroom — with tags, without them, and with a 40 KB payload, `reserved-duration-remaining` came back identical and the `free` box after `moov` that `chapters::splice` eats into stayed exactly 842 bytes.
- **Where a tag can't be told the truth it is left out, never guessed.** No scoreboard — or a kick-off not tagged yet — means no `comment`, and no teams means no `keywords` and a title that says what the export is rather than inventing "Home v Away".
- **The export target's labels live in core too** (`metadata::{ALL_CLIPS_LABEL, REEL_LABEL, WHOLE_MATCH_LABEL, UNTITLED, clip_label, reel_label}`), because the sheet row, the file name and the title are the same words: renaming a target renames it in all three.

**The whole match in track mode is a stream copy, not an encode** (`media/src/composite/copy.rs`). `ExportJob::render` picks the renderer — `Render::Encode` carries everything only the encoder reads, so a copy can't be handed a resolution or a cue-drawing scoreboard — and `composite::export::run` branches on it once, at the top; the `.part`, the chapters, the rename and the delete-on-failure stay in `run`, shared. **There is no `concat`:** Rust owns the ordering, one source at a time, as the encode path's pump does — two `concat`s, one per track, switch source independently and deadlocked one run in three under load. How the copy does it — the re-based segments, the `async=false` sinks, the header-reading caps gate, the single wait and its memory bound — is in that module's header, which is the one place it belongs.
- **The gate is also `pundit_media::can_copy`,** a header read per file that the bus asks before it chooses a renderer. Every refusal names the file and says to choose **Scoreboard: burned in**. **It takes the mute too** (`can_copy(&files, with_audio)`), because `can_copy` **is** `declare` and **four** of `declare`'s refusals are about sound: a non-AAC track, two halves disagreeing about having any, their caps disagreeing, and a rate that can't be read. The seam is one line — `if !with_audio { header.audio = None; }` as each header is read — and it is inside the loop rather than after it on purpose: with the sound forgotten, `absolutely` skips its AAC check, both of `agree`'s audio checks pass on `(None, None)` and `audio_rate` answers `Ok(None)`, so **no refusal is suppressed by hand — all four have nothing left to object to.** **Muting can therefore only *widen* what copies**: it removes checks, adds none and leaves every video rule alone, so two halves differing only in sound, or H.264 carrying AC-3, become copyable and nothing that copied before stops. Applied *after* `declare` instead — which is how the mute was first written — all three stay live, so a muted whole match is refused over sound it will not carry and "Default" spends an hour re-encoding to avoid it.
- **The scoreboard sidecar is `job.path.with_extension("srt")`** — `ExportJob::cues` as `core::cues::cues_to_srt`, written in `composite::export`'s `finish` **after** the rename, so it inherits the run's name cleaning and its `" (2)"` de-duplication and is the matching basename a player auto-loads. A failure is logged and reported as `ExportDone::sidecar: None`; a good `.mp4` is never thrown away over a text file, and a cancel, which never reaches the rename, leaves the last good export's `.srt` alone.
- **`cues` says what belongs beside the output, including nothing.** `Some(cues)` writes them, `Some(empty)` writes none **and removes a stale one**, and `None` is a target that carries no sidecar at all, whose `.srt` is the coach's own file and no export's business. `write_sidecar`'s doc says why the removal is not optional.
- **The chapters also go beside the file as pasteable text**, `job.path.with_extension("chapters.txt")` — `core::chapters::chapter_list` of `CompilationPlan::chapters`, written in the same `finish` as the `.srt` and so with the same name cleaning, `" (2)"` de-duplication, after-the-rename timing and never-fatal failure (`ExportDone::chapter_list`). **A YouTube upload can't read `chpl`**, so the list a coach pastes into the description is the only way those chapters reach a video anyone watches there. **Every target that has chapters gets one**, not just the whole match. There is no "leave the path alone" case as `cues` has: `.chapters.txt` is a name of ours, so a run with no list to write **removes** the stale one.
- **YouTube ignores the whole list silently if any rule is broken**, so `chapter_list` bends the app's chapters to them and says no when it can't: the first line is exactly `0:00` (a first chapter under 10 s in is **moved** there, keeping its own words; 10 s or more in gets a `0:00 Start` line above it), consecutive chapters are at least 10 s apart (a later one inside that is **dropped**, never merged — merging invents a title neither had), times are floored `m:ss` / `h:mm:ss` and ascending, and titles are flattened to one line (a newline would cost every chapter, not just its own). **Fewer than three survivors writes no file at all**: a two-line list is not a shorter list, it is one YouTube ignores, leaving loose timestamps in a description with no hint why.
- **The same cues also ride *inside* the copy, as a `tx3g` track** — the sidecar is what VLC loads without being asked, the embedded track is what survives the file being sent on. It is a third `mp4mux` pad (`subtitle_%u`, `trak-timescale=1000`) fed from an `appsrc` of `text/x-raw,format=utf8`, **requested only when there are cues** and **written whole before the first source is opened**, then ended: a requested pad that runs dry stalls the muxer, so the track is never trickled. **`mp4mux` writes an empty sample between cues** (3,400 cues → 6,799 samples); that is the muxer's, not ours. Only the copy carries it — the encoded path is unchanged.

**The export sheet's third picker is Scoreboard: Default / Burned into the picture / Separate track,** carried by `Command::Export`'s `scoreboard: Option<ScoreboardMode>` (`None` is Default) and remembered in `Preferences::last_export_scoreboard` (v11) by the same write-back as the other two. **There is a fourth control beside it, a checkbox: "Mute source audio"** (v15), remembered the same way and carried the same way, and the basket sheet has its own, in `basket.json` as a plain `bool` — a *label* there is the rule for an enum, which can gain variants a build cannot read, and a bool has none. (A downgrade therefore silently unmutes a **basket**, which is the right trade for machine state with no version; the code says so where the field is.) **Two more checkboxes under the picker say what the run leaves on disk: Chapters and Scoreboard subtitles** (v16, #78), remembered and carried the same way. `Pickers` is all six and **loses its `Eq`** derive, `f64` having none; the write-back's `!=` is all it needed. The command's payload is one struct, `ExportChoices`, because two more bools make `Bus::export` eight parameters counting `self` — one past where clippy's `too_many_arguments` fires — and because `mute_source`, `chapters` and `cues` are three adjacent same-typed values that would read as valid in any order. It has **no `Default`**, derived or written: both switches are `true` and a derived one is `false`. **One negation, at the bus's boundary:** `mute` is the coach's word and the checkbox's, `export_source_volume` and `Render::Copy`'s `with_audio` are everything below it. The two output switches are negated nowhere — `true` is "write it" at every layer.
- **One switch per output, each governing every form that output takes.** Chapters is the `chpl` box inside the file *and* the `.chapters.txt` beside it; Scoreboard subtitles is the `.srt` beside the file *and* the `tx3g` track inside a copy. A coach who turns one off and still finds it in the file has been told a half-truth, and the in-file form is the half he cannot see. Splitting either pair would be a third and fourth checkbox for a combination nobody has named.
- **Off reaches media by blanking the data media already reads, and `pundit-media` knows nothing about either switch.** Chapters off is one line in `job` — `compilation.plan.chapters.clear()` — and subtitles off is `job.cues` as `Some(Vec::new())`. Both are already media's one "no chapters" and one "no subtitles", so a `bool` on `ExportJob` would be the third state `carry_scoreboard`'s own rule exists to refuse. **If either switch ever needs a media change, the design has been abandoned.**
- **"Default" means the best available, never a silent trade.** It copies the whole match when `can_copy` agrees and burns the board in otherwise — a project of Matroska or HEVC sources re-encodes as it always did rather than failing at a gate the coach never asked for. Choosing **Separate track** by hand refuses instead, naming the file: there the coach asked for the copy. **And Default never trades the board away either:** with the subtitles off a copy would carry no board *anywhere*, so Default burns it in — one condition in the `Track` arm, **before** `can_copy`, which reads a header per file. *Separate track* chosen by hand still gets no board, because there the coach asked, which is what makes the sheet's "no scoreboard at all" line deterministic. **The sheet carries two lines under the picker and one derived property feeding both** — `would-copy` on `ExportSheet`, the whole match ticked and the board going anywhere but into the picture, which under Default depends on the subtitles too. The first line follows the *effective* mode, so Default says so too; the second is the "no scoreboard anywhere" warning, and *Separate track* with the subtitles off shows **both**, because it does copy and it does carry no board. **One property rather than the same boolean in two `if`s**, and with the subtitles on it is the old `scoreboard != 1` exactly, so a coach who touches neither switch sees no change.
- **Only the whole match carries the board beside the file.** A clip or a reel asked for on a separate track **burns it in** — it re-encodes either way, and the picker must never lose the board.
- **Independence costs one accepted redundancy** (the coach, 2026-10-06): with the subtitles defaulting on, **every burned whole match now writes an `.srt`** — including a *Default* run that fell back to burning because `can_copy` refused. Shown the three options he chose to accept it: the switch does what it says, and defaulting it off would have stopped today's whole-match copy writing an `.srt` at all.
- **The mapping is two functions over one picker, because they answer two questions.** `carry_scoreboard` in `bus/export.rs` is "how does this target carry the board **in the picture**" — mode + target into the renderer and the one job field that burns it in, with `Carry { copy, scoreboard }` and no cue slot. `board_cues` is "what goes **beside** the file", the three states `ExportJob::cues` has. **`carry_scoreboard` takes `with_audio` for one reason — it is the only caller of `can_copy`** — and `want_cues` for the same shape of reason, Default being unable to answer "the best available" without knowing what else the run carries; **`Carry` still has no field for either:** the renderer needs one bool, which `job` already holds, so a copy of it on `Carry` would be a second truth the encode arm ignores. `job` computes `with_audio` once and hands it to both readers, because the gate it asks and the copy it may choose have to be answering one question. **Track mode blanks `job.scoreboard`** rather than carrying a mode flag into media: `None` is already media's one "don't draw the board", so there is no third state to keep consistent and `overlay.rs` never learns a picker exists.
- **The run's rate window is cleared when a target finishes** (`Active::finish_target`), along with the rate the event carries. A copy runs at thousands of output frames a wall second against an encode's tens, so the clips queued behind one would otherwise inherit its rate and be promised they finish at once.

**The export queue is jobs frozen now and run later** (BACKLOG #77; spec
`2026-10-07-export-queue-design.md`, plan `2026-10-09-export-queue.md`). An
`ExportJob` is already self-contained, so enqueue is *"build the jobs now, run
them later"* and the queue is a `Vec<Queued>` **on `Bus`, not on `Open`** —
`Open` is replaced on every project open and spanning that is the whole
feature. **In memory only**, by the coach's call: closing the app loses it, and
what is lost is snapshots that are re-made by ticking and clicking again. If it
is ever persisted it is `queue.json` beside `basket.json` and **never** a
`state.json` key.
- **`jobs` is `start_run`'s middle, split out** — every refusal that can be made
  from the open project and none about whether anything may run now. An enqueue
  makes exactly those; **Start** makes `refuse_if_busy`'s and *"the queue is
  empty"* (a `CantExport` modal, as the basket's is: Start is a button the coach
  is standing in front of). **`create_dir_all` is `jobs`' last step**, which is
  what makes "created at enqueue, never at Start" structural rather than a rule
  to remember: Start drains jobs and never calls `jobs`, so it *cannot*
  re-create a project folder the coach has deleted and write a film into an
  otherwise-empty directory.
- **A queued row is a snapshot, which is the deliberate inverse of a basket
  piece.** A piece is `(folder, clip id)` resolved at Start, so a clip fixed
  after adding exports as it now stands; a job is frozen, so re-reading its
  label at display time could misdescribe the file about to be written. That is
  why `Queued` caches `match_label` and `label` where the basket's rule is
  *"never cached beside the reference"* — the reasons invert together.
- **"Already in the queue" is keyed on `(folder, target)`, never on the output
  path**, and the two halves of that rule are one rule. `ExportTarget: Eq`, so
  target identity is what the phrase means. A path key looks equivalent and is
  not: `de_duplicate` is per call, so two **different** targets of one project
  that share a name — a clip named after a tag — reach the same path across two
  clicks, and a path key would refuse the second as already queued when it is
  not queued at all, with no way out (there is no edit). The shared name is
  instead handled by **seeding `de_duplicate` with the labels already queued for
  that folder**, so they come out `t0` and `t0 (2)` exactly as they do inside
  one run. Pinned apart: keying on the label refuses the second target, dropping
  the seed makes both write `t0`.
- **Deleting a clip drops the queued jobs that needed it**, as `trash_clip`
  already does to the other two holders of `recordings/<file>` — but **below
  `remove_clip`, not beside them**. Both `?`s above can make that function a
  no-op, and a cancelled transcription and a closed preview are recoverable
  where **the queue has no undo**; `DeleteClip` is a public command with no
  gate, so an id from a project that is not open reaches it and must destroy
  nothing. The scan is **`ClipMedia`, not `PlanEntry::clip_id`**: that is the
  field which actually names the recording, and a `Render::Copy` job holds no
  `Encode` at all, so "a copy is never affected" is true by construction rather
  than by the data coincidence that a whole match's entries carry no clip.
- **`Event::Queue(Vec<QueueRow>)` is the whole view** — no wrapping struct,
  which `Event::Basket` has only because a basket carries a name and two
  pickers. It is published on every change to the queue and **nowhere else**: a
  project open does not emit it and must never clear the list.
- **One negation, in `impl From<ExportChoices> for Pickers`.** It was a few
  lines inside `Bus::export` until the enqueue needed the same mapping; a second
  copy would be a second place for the one negation to be got wrong.
  `settle_pickers` is the write-back, shared by the run and the enqueue, and
  called **after** the work in both for `a_refused_run_leaves_the_pickers_alone`'s
  reason.

**A run holds nothing of the open project, so the coach keeps working while one
goes** (BACKLOG #77 task 1; spec `2026-10-07-export-queue-design.md` §Q7, the
coach 2026-10-07: *"yes you should be able to keep working"*). `refuse_if_busy`
is a refusal to **render** — `start_run`, `basket_job` and the export queue's
Start take it whole — and opening a project is refused by
`refuse_if_previewing` alone, which is the same preview test under its own name
and its own message (`UserError::CantOpen`, modal: `new_match`'s doc argues its
refusals on exactly that and names this one). An **enqueue** makes neither
check, because it renders nothing.
- **What makes it safe is that a job reads its own files and no bus state.**
  `Active` reads `self.open` nowhere, export composites on `Gl::shared()`'s own
  surfaceless display, and `entry_media` deliberately `stat`s rather than
  consulting `Bus::missing` (basket spec V2) — so a project the bus no longer
  has open is safe to render from. §Q7 audited all eleven of `commit`'s steps
  against a rendering job; **nothing in `commit` changed**, and
  `opening_empties_the_trash_and_the_history` keeps its behaviour exactly.
- **The old guard's stated reason was false, both halves of it.** It read *"an
  export or an open preview is composing from the project the coach is leaving,
  and `commit` below empties that project's trash and clears the history"*. A
  job's `ClipMedia::recording` is `recordings/<file>` and **never** a path under
  `.trash`, so emptying it removes nothing a job reads; and `history.clear()`
  destroys only the *undo* of a clip delete, which no job reads either. What
  costs a job its recording is the **delete**, whose rename moves the file out
  from under it — true since Phase 8, in one project, with no queue and no
  switch. The hazard was real and attached to the wrong operation.
- **`refuse_if_busy`'s GL comment was false too** and is corrected: export runs
  on the surfaceless display, not the UI's context (`Gl::shared`'s doc says
  *"Export always runs here"*). The refusal stands on Phase 7 spec P5's
  exclusivity and on decode contention.
- **Two things this costs, both recorded rather than papered over.** A
  **preview** is still refused while a run goes (`preview.rs`), so "keep
  working" is narrower than it sounds — that follows from P5. And the sheet's
  run rows carry the bare target label for an ordinary run, so rows belonging to
  the project the coach left say nothing about which project that was
  (BACKLOG #136). The notice no longer lies about it — it reads *"Exported N
  videos"* with **no folder clause**, which is true of a run spanning any number
  of projects — but the rows stay ambiguous by decision.
- **Three harness tests asserted the refusal and therefore inverted**, not
  one: `export.rs`'s `a_project_open_is_refused_while_a_run_is_going` (which
  the spec named), `new_match.rs`'s
  `new_match_during_an_export_is_refused_and_the_run_finishes` (which it did
  not), and a new `a_project_open_is_refused_while_a_preview_is_open` for the
  clause that remains. **The way to find the second kind is to grep the
  message, not the function** — `"an export is running"` across `crates/` — and
  it is how a refusal's text being asserted verbatim in a test file nobody
  listed gets caught.
- **`main.rs` clears `export-run` on `ProjectOpened` only when nothing is
  running**, because those rows are the coach's one report of work he walked
  away from. The condition is sound rather than racy: `exporting` is written
  only in `show_export` from `Event::Export`, and `begin` publishes that before
  an `OpenProject` can be handled, over a channel that preserves order.
  **It is not testable here** — the harness has no window and a UI test cannot
  reach `main.rs`'s event arms — which is stated in the plan's Risk 1 rather
  than proved.

**The match clock is the displayed frame's source time** (Phase 9).
- **Never a per-clip constant.** `ScoreboardContext::state_at(entry.source_index,
  frame.source_time)` is called per frame, with `source_time` coming from
  `FrameSpec` — not `timeline::source_time`, and nothing cached on `PlanEntry`.
  macOS computed the clock as a per-clip constant plus the commentary's wall
  clock, so every pause and skip pushed the clock ahead of the footage; since
  every recording opens with a pause, that was nearly always (BACKLOG #27).
  A clip that pauses reads the same match time either side of the pause, and
  `core`'s pause test pins it.
- **The absolute events are derived per job** and must never be cached across a
  source add, move, remove or relink — a relink can change a duration, and so
  every later offset.
- **Every scoreboard label is fitted** (shrunk to a floor, then ellipsized).
  `draw_label` centres and does not clip, so an unfitted label spills out of
  both ends of its cell. The columns are sized so nothing realistic shrinks;
  fitting is what makes a spill impossible rather than unlikely.
- **The scan view shows the same board, from the same rasterizer.** Not a
  second drawing of it in Slint: `media::ScoreboardRenderer` renders the
  board's own corner of the frame and the window draws that image over the
  content rect (`show_board` in `main.rs`). It is a viewing aid — nothing is
  exported or stored — and the tick rasterizes only when the key
  `(config, state, device pixels)` changes, so a board costs **~0.3 ms at
  720p, about once a second**, not once a frame. It follows the displayed
  frame's source time as the highlight rings do, is dropped while `scrubbing`
  and restored on release, and is kept through a fast scan, where the shown
  frame's own time is as honest at 32× as at 1×.

**The basket is one film whose pieces come from several matches.** A piece is a
reference — `(project folder, clip id)` — added from the clip row's menu in
whatever project is open, and resolved at Start.
- **It lives in its own file**, `$XDG_CONFIG_HOME/pundit/basket.json`, and
  **not as a key in `state.json`**: `AppFiles`' own read discards that whole
  document on any parse error and every setter rewrites it, so one basket value
  a build couldn't read would take the last project, the pen and the speech
  model with it. Inside it, the two pickers are **string labels** for the same
  reason — an unknown one reads as the default rather than costing the pieces.
  (`AppFiles`, in `bus/state.rs`, is *where the app's own files are*: `state.json`
  with its accessors, its siblings, and — under the user's videos folder, which
  is neither state nor config — the folder a basket's film is written into.)
- **Each piece draws its own match's board and clock**, because the match's
  record hangs off `EntryMedia` and the clock is still
  `state_at(entry.source_index, frame.source_time)` per frame. `PlanEntry` knows
  nothing about matches, and `source_index` stays project-local.
- **The export's decoders are bounded**: one per distinct file, dropped as soon
  as neither the previous entry nor any later one reads it — the audio mixer's
  own rule, one entry wider because up to `QUEUED` of the previous entry's
  frames are still downstream and freeing their DMABuf pool is a hazard CI's
  llvmpipe would never show. At most two open. The zero-copy diagnostic is
  therefore taken when the **first** decoder opens, not after the loop, where it
  may be gone.
- **A basket's text bar is three parts** (`<match> | <clip> | tags`). The bar
  ellipsizes rather than shrinks, so a fourth part would spend the safe end of
  the line on a position that means nothing across matches; the position is in
  the chapters instead.
- **It mixes at the default volumes.** A project's preview volumes are a
  scanning convenience, and a film whose level jumps between pieces for an
  invisible reason is worse than one that doesn't.
- **The sheet is the fifth `Sheet`, and a row is a reference and a length.** A
  row's problem line covers a project that can't be read and a clip that has
  gone; **footage is checked at Start**, not per row, so a clean-looking row can
  still be refused — which is why the sheet says so above its own message line,
  and why that line exists at all (the status bar's notice renders *behind* the
  scrim). The row carries the short phrase and Start's refusal carries the
  sentence that names the folder. Both the sheet and Start read **one project
  per match, not one per piece** (`basket::distinct_matches`), and the sheet's name and
  pickers follow the bus only while the sheet is closed — never under the
  coach's hands.
- **The film's name is cleaned before anything runs**: trimmed, `naming::safe_chars`
  applied, cut to 200 bytes, and defaulted to `Basket` when what is left is
  empty or starts with a dot (which would give a hidden `.mp4`). Start is the
  "walk away" button, so a name must not fail at `File::create` after twenty
  pieces have resolved; an existing film is suffixed ` (2)`, never overwritten.

**Match events are tagged at the playhead or typed, in one grammar.** The
three tag keys (`z` / `x` / `v` by default) tag where the game video is; the editor sheet ("Edit events…" in the
Match panel) takes the same events as lines — `2 14:05 home goal` — in a row's
field and in its paste box alike, both read by `core::match_entry`
(`parse_line`, `parse_batch`, `format_line`, `edit_from_line`).
- **A time has a colon, and a leading bare integer is a video number** — never
  a time. `900 home goal` is refused as *"there is no video 900"*, because a
  bare number read as seconds puts an event minutes out in silence.
- **Kick-off words are refused, not read as period boundaries.** `kickoff`,
  `kick`, `ko`, `restart`, `whistle` each get a refusal naming the damage:
  `interpret` is positional, so one spurious start/stop moves every later
  period and the clock burned into every export. `kickoffs.txt` is by
  definition a list of *restarts*, and pasting it must add nothing.
- **The sheet's rows are rebuilt by its own committed edit and never from
  `show_project`** (`main.rs`'s `editor_rebuild` flag): a transcript landing
  (`bus/transcribe.rs`) and a source found missing (`bus/transport.rs`) both
  publish `ProjectChanged` with no command behind them, and a `LineEdit` inside
  a `for` can only be bound one way — a rebuild would overwrite what is being
  typed. Every commit drops focus, because `for` reuses its items by index and
  a re-timed event moves.
- **A new text field joins nothing, and that is load-bearing** (#96 task 1).
  The window's `text-editing` **is** Slint's own
  `TextInputInterface.text-input-focused` (`app.slint`), which `TextInput` sets
  on focus-in and clears on focus-out *and in its `deinit`* — so a field cannot
  be left out of it and a field destroyed while focused cannot leave the
  shortcuts switched off. It used to be a seven-term disjunction of every
  field's focus, which is how **#130** happened: the slate tag filter shipped as
  a bare `LineEdit` nobody added, so every letter typed into it fired its
  shortcut and `corner` was untypeable. The *reason* the gate exists is
  unchanged, and it is why a sheet must not test anything else: the sheet's key
  guard is `!text-editing`, so the first Esc leaves the field and the second
  closes the sheet, and without that the first Esc throws away a half-typed
  paste. `tests/ui/text_editing.rs` types a letter into every field there is and
  pins both halves.

**The goals reel** (`pundit-core/src/reel.rs`, spec R).
- **It is an `ExportTarget` (`Reel`), never a clip.** Its entries have
  `clip_id: None`, so its PiP is the GL filler and its audio is the game's alone.
- **It holds confirmed goals only** (every goal match event), in match order.
- **Each entry is one `Play` segment**, `[goal − lead-in, goal + tail]` on the
  goal's source, clamped to the source and to the previous entry's end on it. A
  goal at or before that end makes no entry of its own: it extends that one.
- **The defaults are 20 s and 6 s** (`REEL_LEAD_IN`, `REEL_TAIL`), overridden per
  side by the goal's trim. Never replace them with a guess that could be
  shorter: a cut-off assist is the one failure the reel must not have.

**Slates: a range marked now, its commentary recorded later** (`Project.slates`,
v12; spec `docs/superpowers/specs/2026-09-25-slates-design.md`).
- **A slate is not a clip and cannot become one.** A clip *is* a recording —
  that is what lets every clip replay, preview and export with no special case.
  Shooting a slate **produces** a clip, and the link runs from `Clip.slate_id`
  so nothing dangles: "has this been shot?" is a scan of the clips, which stays
  right across a delete, an undo and a re-record.
- **The in mark stores the slate on the first press** (`i` by default), with
  `out_seconds: None`. A
  half-marked range is then a row the coach can finish or delete rather than UI
  state that vanishes with the app — and marking needs no in-progress source
  index to invalidate when the source list changes underneath it. The out mark
  (`o`) closes the **most recently opened** range on that video, which is why the stored
  order is the marked order and `slates_sorted` is for reading only.
- **Both marks are on the recording allow-list**, beside a match tag and a
  highlight key, for the rule those two are there for: a record that belongs to
  the footage is placeable whenever the footage is on screen. So a refusal can
  land over a live take, and `UserError::Slate` is a notice, never a modal.
- **The shoot happens inside `start_recording`**, after `capture_sources`, and
  resets the skip coordinator first. Not because it is tidier: `NoCamera` comes
  from `resolve_camera` *after* `can_record` passes, so seeking from outside
  would move the game video and then refuse on a camera-less machine; and
  `heading` prefers the skip coordinator's pending target, so a skip burst still
  in the air would stamp the clip **where the arrows were heading** — two taps
  of the right arrow is 6 s, Shift-taps 20 s — pinned by
  `a_pending_skip_does_not_drag_the_take_off_the_in_point`, which fails without
  the reset.
- **A slate holds its source open** (`source_is_referenced`) and rides both
  remaps. `purge_for_source_change`'s staleness test is an **exhaustive match**
  for this reason — as a `matches!` it admitted new record types in silence, and
  a snapshot that survives a source move restores indices from before it.
- The tag **vocabulary** is clips ∪ slates (`tag_vocabulary`), so a tag invented
  on a slate autocompletes; the tag **overview** stays clips-only, because its
  columns are a clip count and a duration.
- **The themed pass is the Slates list itself, filtered by tag** (#120, spec T):
  the coach works through "all these are corner kicks" by shooting a range and
  letting the list hand on to the next. The whole advance is
  `slate_pass::next_slate` — **the first `timed && !shot` row at or after the
  selected one**, parked on with `JumpToSlate`, which pauses. Three things about
  it are load-bearing. **Nothing is snapshotted and the list is the queue:** a
  shot slate *stays* in the list, so "at or after" moves on by itself after a
  successful take and parks on the same range again after an aborted one, and
  the spec's T3 bug — a queue of unshot ids, `[A,B,C]`, shoot A, index + 1 is
  **C** and B is never offered — cannot be written. It also means a tag filter
  changed between takes is honoured with no state. **It advances on
  `Event::Recording(RecordingStatus::Idle)` and never on `ProjectChanged`:** an
  abort emits only the former (nothing changed to save), and the latter fires
  for things with no command behind them — a transcript landing — which would
  move the footage under the coach's hands. And **`R` carries the selected
  range** (`ToggleRecording { slate }`), with the *bus* choosing the shoot: the
  window's recording phase lags the bus's, and a `ShootSlate` sent into a
  running take is refused by the recording guard, so a window that branched on
  its own phase would swallow the second R that means "cancel". The transport
  button reads **"Record range"** while one is selected, which is how the mode
  stays visible rather than being state on the primary key.

**Player highlights** (`pundit-core/src/highlight.rs`, spec H).
- **A highlight belongs to the footage, not to a clip.** It is stored on the
  project (`Project.player_highlights`, v9) and keyed by `source_index` and the
  **displayed frame's stream time** (`Frame.stream_time`), never by record time,
  so it shows wherever that footage does — scanning, recording, a preview, every
  clip export that crosses it, and the reel — and it freezes with the footage
  through a commentary pause. Two keys on one frame are the same number, so a
  key replaces another by exact equality and no tolerance is stored or needed.
- **It is drawn from `highlight_shapes`**, the one piece of drawing geometry,
  which maps source-normalized rects through `Zoom::transform` — the affine the
  picture itself is drawn with. No new mapping function, in either the media
  overlay or the live Slint layer. Strokes stay zoom-agnostic (they live in the
  content rect); a highlight lives in source space and moves with the zoom.
- **The shape carries the label too** — its font size, the pill's height
  (`LABEL_PILL_RATIO`) and the pill's y, above the box or below it — and
  `highlight::label_ink` says whether the number is black or white. A drawer
  decides only how *wide* the pill comes out, because only a drawer shapes
  text; `app.slint` takes the rest as properties rather than repeating the
  ratios. The ring is stroked at `layout::STROKE_LINE_WIDTH`, the **one** pen
  width, which the live stroke layer and a logged `Stroke` also take from
  there.
- **The live ring is placed on the displayed frame**, `main.rs`'s
  `shown_position` — the same frame a key is placed on and "Delete key here"
  offers — not on `project.locate`. That is what makes the ring on screen the
  ring export burns in (spec H6).
- **A highlight may be placed outside a recording**, while pen drawings stay
  recording-only: a highlight describes the footage, a drawing the commentary.

**Match analysis is dB over a rolling median, never a level** (P3 — the
measurement phase: it stores nothing, suggests nothing and adds no command).
- **Core owns the maths, media owns the decode.** `core::signals` is
  `whistles` / `cheers` / `cheer_excess`, pure functions over a slice of
  16 kHz mono samples on 32 ms windows at a 16 ms hop.
  `media::analyze::audio::samples` is the **third caller of the export's own
  `Reader`** (`composite::audio::read_all`, which transcription now shares) and
  adds no decode path. A whole half is 27 MB of `f32`, which is what lets every
  rule stay a pure function of a slice.
- **Nothing absolute can work.** Venue gain differs by ~27 dB between the three
  tagged matches, so every threshold is dB over a **60 s rolling median of the
  signal's own band**, computed from a half-dB histogram — sorting each span
  over a hundred thousand windows costs minutes.
- **A whistle needs three terms**: level over that median, a ±150 Hz pitch
  hold, and **tonality**, the peak bin over the median of the other bins in the
  same window. Most broadband sound fails the pitch hold on its own — a noise
  burst's loudest bin hops. Tonality is what rejects sound that is broadband
  *and* steady (a horn, a buzzer), and the horn fixture in
  `core/tests/signals.rs` is the test that fails without it.
- **The picture is `core::motion` and `core::kickoff`**, read off the same
  decode: five frames a second of mean absolute luma difference, and one 32x18
  thumbnail a second. **Stillness is a quantile of the half's own motion**
  (`still_theta`), never a level — the median motion of a half runs 16–19 in two
  of the three venues and 4–8 in the third. On this footage the threshold has to
  land near the **median** (the camera's motion is bimodal), and the spec's
  "then motion above θ for 3 s" has to be read as the *median* of those 3 s: the
  literal reading found **0 candidates in 6 halves**.
- **P3's verdict is `docs/superpowers/spikes/2026-09-24-match-vision-measurements.md`,
  and it is negative. Read it before touching any of this.** Held out, the whole
  rule finds **7 goals of 9 with 29 false ones**, whose windows cover **57% of
  the match** — against a chance recall of 0.63, so the lift is **+0.15**.
  Periods are worse (start 0.50/0.50, end 0.00): the period whistles are audible
  and detected, but nothing tells them from the 41–85 other whistles in a half,
  not duration and not loudness. The cheer is the one real cue (8 of 9 held-out
  goals at 24 firings a half). **P4 is not justified and is not started**;
  nothing here is wired to the bus, the format or the UI.
- **Measured on the three tagged matches** (2026-09-24, release): the whole
  `Analyzer` is **67–84 s per file, ~24x realtime**, well inside G4's 5-minute
  bar.
- **The measurement run is `#[ignore]`d and needs `--release`** — an
  unoptimised Goertzel bank is about forty times slower:
  ```bash
  PUNDIT_GROUND_TRUTH=B=<folder>:A=<folder>:C=<folder> \
    cargo test --release -p pundit-harness --test ground_truth -- \
      --ignored --nocapture --test-threads=1
  ```
  **One run, not two** — sound and picture are scored together, off one
  `Analyzer` pass per source. The coach's folders are **the only copy of the
  footage and are read-only**:
  `store::read` plus a `kickoffs.txt` read, never a `Bus`, never a write. CI
  never sees them; every unit test is synthetic. **Nothing identifying goes in
  the repo or a pasted report** — no club, opponent, player, file or folder
  name. The matches are A, B and C.

### The macOS original (removed from the tree)

The Swift app this port replaces lived under `apple/` until 0.7.0. It is gone
from the working tree and kept whole at the annotated tag **`macos-reference`**:

```bash
git show macos-reference:apple/VideoCoachCore/Sources/VideoCoachCore/<file>.swift
git worktree add /tmp/macos-reference macos-reference   # the whole tree, read-only in practice
```

Read it only to answer "what did the original do?" — never to change it. It was
never built by CI and several known bugs were deliberately left in it
(`BACKLOG.md` #27); the port fixes them by construction. The behaviour worth
keeping is already written down in `docs/superpowers/specs/`, so reach for the
tag when a spec is silent, not as a first step.

Two of its conventions still bind this codebase, because the format is shared:

- **A project is a folder:** `project.json`, a `recordings/` subdir, an
  `exports/` one. `formatVersion` bumps on every additive schema change and
  migration happens at decode time, never at save. The port starts at v7 and
  refuses anything lower.
- **Project data and UI state stay apart.** Swift's `Workspace` held project
  data only, with mode flags on the view; here the same line runs between
  `project.json` and `state.json` — a preference is never a format change.

## Backlog

Carry deferred items in `BACKLOG.md` (worktree root). Format: numbered list under headings (Spec/plan corrections, Code follow-ups, UX gaps). Each entry includes "Why deferred" and "When to revisit."

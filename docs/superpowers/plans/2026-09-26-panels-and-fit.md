# Plan — Fit window to video (#95), then resizable panels (#87)

Spec: `docs/superpowers/specs/2026-09-26-panels-and-fit-design.md`. Read it
before any task here; this plan does not repeat its reasoning, and every task
cites the section it implements.

**Two shipments, in order, each green and committed on its own.** A is #95 and
needs nothing from B. Do not start B until A is committed.

Before anything: `CLAUDE.md`, and the two `app.slint` rules that produced the
worst bugs of the slates pass — a field is **hidden, never removed**, and an
`editing` flag is **derived, never assigned**. This work adds elements to that
same file.

Every `cargo` call except `fmt` goes through
`flock /tmp/claude-1000/cargo.lock nice -n 19 cargo …`. A `pundit-app` test
build is ~3 minutes. **Never pipe a clippy or test run to `tail`** — it masks
the exit status, which has already let a clippy failure be committed here;
redirect to a log and check `$?`.

## Traps that cost a working day if rediscovered

All four are measured, in the spec, and all four compile cleanly:

1. **`tick`'s body runs inside `UI.with_borrow_mut`.** Calling `show_notice`
   from it is a `BorrowMutError` panic. Set `notice` and `notice_until` by hand
   inside the borrow, or decide an action and act after it ends.
2. **`is_maximized()` flips synchronously** when you call `set_maximized(false)`
   — it reads the property that call writes. It is not a gate on the WM having
   restored anything. Watch the window's **size** instead.
3. **The headless testing backend honours any size**, fractional or below the
   declared minimum, and converts at a hardcoded scale factor of 1. It can see
   neither the rounding nor the floor. Those are unit-test facts.
4. **`ElementHandle` silently finds nothing** unless the crate was built with
   `SLINT_EMIT_DEBUG_INFO=1` — and the test still passes. Any measurement that
   walks the element tree must set it, or it measures nothing and says fine.

---

# Shipment A — Fit window to video (#95)

## A1. `fit.rs`: the arithmetic, pure, in the lib

**Files:** new `crates/pundit-app/src/fit.rs`; `pub mod fit;` in
`crates/pundit-app/src/lib.rs`.

Spec W8 says why this is in the lib: `main.rs` is a separate `[[bin]]`, so
nothing under `tests/` can call it, and a test that re-implements the arithmetic
tests its own copy. Follow `zoom_input.rs` for house style — a header stating
the units and the finite-input discipline (BACKLOG #28), doc comments that carry
the reason rather than restating the code.

```rust
pub enum Fit { NoSlack, TooSmall, To(u32, u32) }

/// Physical pixels; the target is rounded **up**.
pub fn fit_window(frame: (f64, f64), player: (f64, f64),
                  window: (f64, f64), min: (f64, f64)) -> Fit
```

**Three outcomes, not two** (spec W6): `can-fit` is `!NoSlack`, and `TooSmall` is
the only thing that shows the notice. A two-state answer makes the refusal
unreachable, because the same value gates the offer — the notice would be dead
code.

**Build it on `zoom_input::Viewport`** (spec W8): `Viewport::new(frame.0,
frame.1, player.0, player.1)` already rejects every non-finite and non-positive
value among those four, and `picture(Zoom::IDENTITY)` **is** the letterbox the
fit must agree with. Validate only `window`, `min` and the chrome yourself.
`Viewport`'s doc says only the shape matters, so physical pixels are fine — say
so in `fit.rs`'s header, because `zoom_input.rs`'s says logical.

The rules, in order:

1. `NoSlack` unless `Viewport::new` succeeds, `window` and `min` are finite and
   positive, and the chrome (`window − player`, per axis) is `>= 0` on both. An
   inconsistent pair is a layout that has not settled.
2. The chrome is `window − player`, per axis — **never** derived from column
   widths (spec W2).
3. Shrink the axis with the slack: player aspect below the frame's sets the
   **height** to `player_w / aspect + chrome_h`; above it sets the **width** to
   `player_h × aspect + chrome_w`. The other axis passes through unchanged.
4. **`ceil` the changed axis.**
5. **`NoSlack` unless the result is strictly smaller on the changed axis.** This
   one comparison delivers three of the spec's promises — never grow, the no-op
   at the right aspect, and idempotence — so do not also write an epsilon test
   for "within a pixel".
6. **`TooSmall` if the result is under `min` on *either* axis.** Either, not just
   the changed one: spec W6 and W10 both say either, and the divergence would
   make W10's test 4 fail against a "changed axis only" reading.

Rules 5 and 6 need doc comments carrying *why*.

**Unit tests in the same file** (spec W10, tests 1–5), fixture = W0's measured
table. The strongest assertion available, and the one to write for test 1, is
`Viewport`'s own: the content rect at the fitted size equals the content rect
before it, to the pixel. Assert that **and** `To(1600, 716)`, with a comment
that 715 is the floored answer, shrinks the picture 0.89px of width, and
breaks idempotence (a floored re-fit of 1600×715 asks for 1599).

**Verify:** `cargo test -p pundit-app --lib fit` — and **assert the count**
(`5 passed`, or run each test `--exact`). A filter that matches nothing exits 0,
so a forgotten `mod tests` or an unwired `pub mod fit;` would pass this step
silently. Then clippy, to a log, exit status checked.

## A2. Wire the fit: the floor, the gate, the key, the button, the call

**Files:** `crates/pundit-app/ui/app.slint`, `crates/pundit-app/src/main.rs`.

A2 and A3 of the first draft were one reviewable unit pretending to be two —
neither compiles or means anything without the other, and each paid a 3-minute
build. One task.

### In `app.slint`

1. **Expose the floor** (spec W4 — `min-width`/`min-height` are reserved and have
   no generated getter, so Rust would otherwise hold a fourth copy of the
   number). Verified to compile:

   ```slint
   out property <length> min-window-width: 1100px;
   out property <length> min-window-height: 700px;
   min-width: root.min-window-width;
   min-height: root.min-window-height;
   ```

   Keep the existing comment above them.

2. `in property <bool> can-fit;` and `callback fit-window();`.

3. **The key:** an `f` branch in `handle-key`'s letter section, behind the
   `text-editing` fold at `:3529`, gated `if (root.can-fit)`, and behind
   `!event.repeat` as the other letter branches are (`:3568`, `:3582`, `:3588`,
   `:3607`, `:3614`, `:3650`, `:3687`). `f` is free.

4. **The button, in the transport's *second* row** (`:4801`, the drawing/status
   row; `:4795` is the first row's closing brace) — **not the first**. Spec W3 has the measurement: the first row is
   exactly full at a 1100px window (buttons out to x=1092 of 1084px usable), and
   the only give is the readout, which clips rather than ellipsizes. `text:
   "Fit"`, `enabled: root.can-fit` (**never `visible`** — it would reflow the row
   under the cursor), `clicked => { keys.focus(); root.fit-window(); }`, and a
   `Tooltip`. A disabled Button keeps its tooltip, so make the text follow the
   state as the Record button's does: one wording when it will fit, another when
   there is nothing to fit.
   - **Check the notice line still reads at 1100px.** It shares that row. If it
     is squeezed, say so and stop rather than shipping a notice with no room —
     the fit's own refusal is what appears there.

### In `main.rs`

5. **`can-fit`, before `tick`'s early return.** The push goes in the same block
   as `notice_until`, *above* `let Some(project) = ui.snapshot… else { return; }`
   (`:3087`) — spec W8. Put it after that return and a project close leaves the
   property at its last value; since `frame-width` is never cleared, a stale
   `can-fit: true` lets `f` resize the window over the empty-project card.
   One helper assembling the four pairs, shared with 6:
   - `window`: `w.window().size()` — already physical.
   - `player`: `get_player_width()/_height()` × `scale_factor()`.
   - `frame`: `get_frame_width()/_height()` (a ratio; no scaling).
   - `min`: `get_min_window_width()/_height()` × `scale_factor()`.

   The helper also carries **`can_play`** (spec W3), so the gate is one call in
   Rust and never a second term in Slint.

   Note in `fit.rs`'s header that the `× scale_factor()` on the player *is* a
   logical→physical conversion; the round-trip error is ~1e-4px and the `ceil`
   absorbs it. The spec's "no conversion happens after this" is about the
   target, not the inputs.

6. **`on_fit_window`:**
   - **Maximised or fullscreen** → `set_maximized(false)`, arm
     `ui.pending_fit = Some((w.window().size(), Instant::now() + FIT_UNMAXIMIZE))`
     and return. **Do not compute the target here** (spec W6: the WM restores the
     pre-maximise geometry, so a target from the maximised one is a *grow* in
     both axes). Fullscreen is in the same branch because winit ignores
     `set_size` on a fullscreen window. **Correction from execution:**
     un-maximising cannot leave fullscreen, so fullscreen is its own immediate
     refusal with its own notice — see spec W6.
   - Otherwise `match fit_window(...)`: `To(w, h)` →
     `set_size(slint::PhysicalSize::new(w, h))`; `TooSmall` →
     `show_notice(&w, FIT_IMPOSSIBLE.into())`; `NoSlack` → nothing.

7. **The pending fit, in `tick`** (a second place in `tick`, separate from 5):
   fire on the first tick where `w.window().size()` **differs from the size the
   press recorded** — not on `!is_maximized()`, which flipped the instant
   `set_maximized(false)` was called (trap 2). On the deadline, clear it and set
   `notice`/`notice_until` **by hand** (trap 1 — `show_notice` panics here).

8. **`pending_fit` goes in `UiState` *and* in its hand-written
   `impl Default for UiState`** (`:243-272`), which is not derived. Declare
   `const FIT_UNMAXIMIZE: Duration` beside `NOTICE` and
   **`const FIT_IMPOSSIBLE: &str`** beside `DRAWING_HINT` (`:93`) and
   `HIGHLIGHT_PAUSE_HINT` (`:97`). Wording: the window is too small for the
   footage's shape and a bigger window is what makes it possible — the coach's
   next move, not a diagnosis. **No `UserError` variant, no `Command`**; this
   never touches the bus.

## A3. The window test, and the key's gate

**File:** new `crates/pundit-app/tests/fit_window.rs`, in `slate_fields.rs`'s
shape.

**It cannot test that `f` resizes the window.** `on_fit_window` is in `main.rs`,
a `[[bin]]`; a test can only install its own handler and would be testing its
own copy. Say that in the header, and name A4's runtime check as what covers the
wiring. Two tests:

1. **The chrome subtraction predicts the real layout** — the one fact no unit
   test knows. Wire `place-picture` to `pundit_app::zoom_input::Viewport` (the
   lib is public; `PictureRect`/`ZoomState` come from `include_modules!`), set
   `can-play`, a 16:9 frame and `PhysicalSize::new(1600, 960)`; read
   `player-width/height`, the content rect and `get_min_window_width/height()`;
   call `pundit_app::fit::fit_window` directly; `set_size` its answer; assert
   `player-width` and **both** content-rect dimensions are unchanged, and that a
   second `fit_window` on the new layout is `NoSlack`. Nothing about position
   (spec W1). Nothing hard-coding 520, 108 or 716.
   - **An unwired `place-picture` returns `PictureRect::default()`**, so every
     content-rect assertion passes on 0×0. The header must say so — that is how
     the vacuous version of this test gets written.
2. **The key's gate** — the failure that bit the slates pass: `f` invokes
   `fit-window` when `can-fit` is true, does not when it is false, and a focused
   field swallows it. Observe the callback through a closure, as
   `slate_fields.rs` does with `on_mark_in`.

## A4. Close out shipment A

- The `verify` skill: fmt, clippy, tests, the core dependency audit.
- **On the reference laptop**, the two things no test can reach: `f` on a
  **maximised** window (spec W6 calls this a runtime check, not an assumption —
  Cinnamon may drop the un-maximise), and that the bars are actually gone.
- Adversarial review on the diff (`adversarial-review`), then commit.
- `CLAUDE.md`: **one sentence** appended to the transport-keys paragraph — `f`
  fits the window, it only ever shrinks, the chrome is read off the window, and
  `fit.rs` is the one place the arithmetic lives.

---

# Shipment B — Resizable panels (#87)

## B1. `state.json`: the widths, and the `serde(default)` level

**File:** `crates/pundit-app/src/bus/state.rs`.

- `PanelWidths { sidebar: u32, inspector: u32 }` with a hand-written `Default` of
  today's widths, and **`#[serde(rename_all = "camelCase", default)]` on the
  *container*.** **Not field-level.** `project.rs:77-80` states the rule: "Field
  level `#[serde(default)]` is the hazard: it resolves to `Default::default()`",
  i.e. `0`. Field-level on `WindowSize` would turn `{"window":{"width":1600}}`
  into `1600×0` and `main.rs:313` would hand that straight to `set_size`.
- **Add the same container-level `default` to `WindowSize`**, which has the gap
  today: any parse error discards the whole document (`:196-200`), taking the
  last project, the pen and the model. A free adjacent fix, and the test below is
  what earns it.
- With a container default, `panels: PanelWidths` needs no `Option` —
  `#[serde(default)] panels: PanelWidths` is the whole of it. `window` keeps its
  `Option`, which it already has.
- `panel_widths()` / `set_panel_widths()` beside the window-size pair.
- Tests beside `remembers_the_window_size`: the widths round-trip; a file with no
  `panels` reads the defaults; and **a file with a partial `panels` or `window`
  object still yields the last project, the pen and the model — with the missing
  field at its real default (960, not 0).** Check that the `window` half fails
  before the change, and say so in the test's comment.

## B2. The columns: minima, widths, and the bound as layout constraints

**File:** `crates/pundit-app/ui/app.slint`.

```slint
out property <length> sidebar-min: 240px;
out property <length> inspector-min: 280px;
out property <length> player-min: 320px;
in-out property <length> sidebar-width: root.sidebar-min;
in-out property <length> inspector-width: root.inspector-min;
```

Replace the sidebar's `width: 240px` (`:3724`) and the inspector's `width: 280px`
(`:4469`) with the constraint form from spec W4 — `min-width`,
`preferred-width`/`max-width` of `max(own-min, stored)`, `horizontal-stretch: 0`
— and give the player `min-width: root.player-min; horizontal-stretch: 1;`.

**Do not use `width: clamp(stored, own-min, root.width − other − player-min)`.**
Measured at a 1100px window with the two splitters, stored `(100, 700)`: the
clamp form yields a player of **308px**, under its own minimum, because each
column computes its headroom from the other's *raw* value and neither knows about
the splitters. The constraint form yields **320**. Put that table in a comment
above the sidebar's constraints and point the inspector's at it.

Keep every comment already there. **Check that no child's intrinsic minimum now
shows through** — both columns wrap a `TouchArea` and (the inspector) a
`ScrollView`, and an explicit `min-width` is a local override, so it should not,
but confirm rather than assume.

**Its test, in this task** (spec W10 test 6): a stored pair too wide for the
window leaves **neither column under its own minimum** and the player at
`player-min`. Do not assert a player width derived by summing the columns — the
splitters are 12px that sum forgets, which is what made the first draft's
version of this test fail.

## B3. `ui/splitter.slint`

**New file**, imported by `app.slint` — the pattern `scrubber.slint` sets, and
the reason is the test: `tests/scrubber.rs` imports it into a small inline
`slint!` window, which is how this task's test avoids the whole `AppWindow`.

A 6px `Rectangle`, transparent until `has-hover`, `mouse-cursor: ew-resize`,
`in property <length> width-now` (the column's **laid-out** width),
`in property <bool> mirrored` (the inspector's splitter runs the other way — a
bool, not a `float` sign that could be set to 0 or 2), `callback moved(length)`,
`callback released()`.

**No `minimum` property.** The bound lives in B2's constraints, and spec W5 is
explicit that it lives in exactly one place; a `minimum` here invites clamping
inside the splitter, which re-creates the stick it exists to avoid.

The drag, per spec W5 — **both rules, each a bug if dropped:**

- On press record `anchor = self.absolute-position.x + area.mouse-x` **and**
  `start = root.width-now`, the laid-out width. Verified: `absolute-position` is
  readable in a `TouchArea` handler, the grab survives far outside the
  splitter's 6px, and `start` from the laid-out width picks the pointer up
  correctly from a clamped state.
- On `moved`, emit `start ± (self.absolute-position.x + area.mouse-x − anchor)`.
  The absolute x is invariant as the splitter moves under the drag. Do **not**
  write `width += mouse-x − pressed-x`: it is what Slint's own
  `widgets/material/tableview.slint` does, and against a bounded consumer each
  event adds an increment nothing cancels, so the raw value runs away from the
  pointer.

The header says what both rules are for; that is where the next reader looks
after a drag feels wrong.

**Its test, in this task** (spec W10 test 7): a small `slint!` window over this
file, `PointerPressed`/`Moved`/`Released`, **starting from a clamped state** — a
drag past the minimum stops there, a drag back picks the pointer up where it
left it. The test window therefore needs a stand-in column carrying a real
bound; a `width-now` bound to a constant can never be clamped, and the test
would catch neither failure. Use B2's constraint form in the stand-in, not a
`root.width` clamp.

## B4. Wiring the drag

**Files:** `app.slint`, `crates/pundit-app/src/main.rs`.

- A `Splitter` either side of the player, **inside the `HorizontalLayout`** — so
  they are chrome, which spec W2's subtraction absorbs without being told.
  `width-now` binds to the column's laid-out `width`; `moved` assigns the
  `root.*-width` property; `released` calls `root.panels-released()`.
- **On release, set the property to the column's effective (clamped) width
  before firing the callback** (verified: the new value is visible to Rust when
  the callback runs). Spec W5: the raw value would put a width the coach never
  saw into `state.json` and give the next drag a dead zone.
- `main.rs`: set both properties from `state.panel_widths()` at startup, beside
  the window size; `on_panels_released` reads both and calls `set_panel_widths`.
  **The callback takes no argument.** Nothing goes through the bus.

## B5. Close out shipment B

- The `verify` skill.
- Run the app: drag both splitters past their minima and back, quit, reopen,
  confirm the widths came back. Then press `f` and confirm the fit still lands
  with the panels moved — the one place A and B meet.
- Adversarial review on the diff, then commit.
- `CLAUDE.md`: fold the panel widths into the `state.json` sentence that already
  lists the last project, the model, the pen and the window size.
- `BACKLOG.md`: #87 and #95 resolved, with the date. **Leave #95's snap note
  standing** — it is the coach's open question.

---

## What this plan deliberately does not do

- **The splitter snap.** Spec W11 and BACKLOG #95: structurally unreachable for
  the bars the coach complained about, because the panels only grow. It needs
  their nod, not a task.
- **Coalescing the scoreboard raster**, which BACKLOG #87 asks for. Spec W9
  checked it: one call site inside the 30Hz tick, behind a key comparison,
  ~0.3ms for the board's own corner, and a window resize already does it today.
  A mechanism here would buy nothing.
- **Lowering the window's minimum, or letting a panel go under today's width.**
  280px is the width the inspector's transcript row was fitted to (`:694`).
- **Touching `place-picture`, the content rect, or anything the export reads**
  (spec W9).

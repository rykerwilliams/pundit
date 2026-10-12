# Plan — snap the scrubber to match events (#102)

Spec: `docs/superpowers/specs/2026-10-07-scrubber-snapping-design.md`. Read it and
`CLAUDE.md` first; neither is repeated here, except where §0 below corrects the
spec. **Every task starts with the `start-task` skill and ends with `verify`,
`sabotage-proof` and `recheck-claims`.**

Every `path:line` here was re-derived on **2026-10-12 at `d8311a0`**
(`claude/84-task-3`, #84's last task, not yet on `main`). That branch moves
`main.rs` and `app.slint` by up to ~100 and ~50 lines against `main` at
`d18865a`, so **re-derive every line at `start-task` §3** — `app.slint` drifts
fastest. Paths are under `crates/pundit-app/` unless they say otherwise.

## Where this stands (update it as tasks land)

- **1 — the scrubber's Slint half: the menu and `snap-seconds`.** Not started.
- **2 — the Rust logic: `nearest_chapter` and the `state.json` bool.** Not started.
- **3 — the wiring, the guide and the prose.** Not started.

**Shipping: two PRs, in order.** **Task 2 first, as its own PR**: pure functions
and a file field, reachable from unit tests, which nothing reads yet. **Then
tasks 1 and 3 together, as one PR with a commit each**: task 1 alone would put a
checked *Snap to match events* on `main` that does nothing, and task 3 needs both
the control and task 2's logic. Task 1 is still done first inside that PR,
because it is where the riskiest claims live.

---

## §0. Where the spec is out of date or wrong (checked 2026-10-12)

The codebase has moved since 2026-10-07 (#96 keys, #138 whiteboard, #77 queue,
#84 music). Design decisions are unaffected unless a line says otherwise.

1. **§P1's geometry is wrong, and so are the seconds built on it.** The spec
   measures the track off the **player column** (1068px at the default window).
   The transport bar is not in that column: it is a child of the window-wide
   `VerticalLayout` (`app.slint:5412`), sibling of the three-column
   `HorizontalLayout` (`:5413`), and its own comment says *"full width"*
   (`:6745`–`:6747`), padded 8px (`:6751`). So at the default 1600px window
   the track is 1584px and the travel **1564px**, not ~1032px. Re-derived:

   | Window | Travel | 45-min half, 6px | 90-min match, 6px | 90-min, 10px | 90-min, 4px |
   |---|---|---|---|---|---|
   | 1100 (minimum, `app.slint:3922`) | 1064px | ±15.2 s | ±30.5 s | ±50.8 s | ±20.3 s |
   | 1600 (default, `bus/state.rs:207`) | 1564px | ±10.4 s | ±20.7 s | ±34.5 s | ±13.8 s |
   | 1920 | 1884px | ±8.6 s | ±17.2 s | ±28.7 s | ±11.5 s |

   The spec's ±16 s / ±31 s are the **minimum** window's numbers, not the
   default's. **The decision survives**: 10px still swallows the reel's 20 s
   lead-in at every width that matters, and smaller is still the safer error.
   6px stands; Risk 2's "4px is ±21 s" is ±13.8 s at the default. §F4's "3 s
   before a goal is 1.1px" is 0.87px at the default.
2. **§S1: `CURRENT_FORMAT_VERSION` is 17, not 15** — still not bumped.
3. **§H4: "#84 is the sheet's first tenant and builds it" is no longer true.**
   #84's phase-1 spec §M6 (`2026-10-12-reel-music-design.md:132`) put the music
   folder on a **Folder…** button in the export sheet's Music row and built no
   sheet. That strengthens §H (nobody has built the sheet) and changes nothing
   here — but BACKLOG's #102 index line (*"which #84 does"*, `BACKLOG.md:33`)
   and #131's revisit line (`:4378`) repeat it; task 3 corrects #102's own line
   and #131's (§A).
4. **§H4 / §X: #96 has shipped**; a key for this toggle is still not built here.
5. **The spec's crate table misses a Scrubber callback.** The menu is *inside*
   `Scrubber`, so `activated` can only call a callback **of `Scrubber`**; the
   window's `toggle-snap` is a forward of it. Task 1 adds `callback
   toggle-snap()` to both.
6. **§P3's *"`marks.length` is the idiom used in nine places"*:** `.length` is on
   46 lines of `app.slint`. The point (it is an idiom) stands. The spec's line
   numbers have drifted too; re-derive them at `start-task`, never from the spec.
7. **One thing the spec leaves to luck, made explicit here: the release must
   snap too, not only the moves.** The handler writes the snapped value back
   as an `f32` (`position-seconds`), and `released(root.value)`
   (`scrubber.slint:161`) hands that `f32` back. Only a release that re-snaps
   against `match_abs`'s `f64` lands bit-identical to `]` (spec §L4). Re-snapping
   an `f32`-rounded mark finds the same mark — its distance is ~1e-4 s against
   a radius of seconds — so the release costs nothing extra.
8. **§H6's generator argument is no longer load-bearing.** `wire_snap` (task 3)
   flips the **window's** property on every activation, and the item's own
   write (`menus.rs:191`) flips its `checked` on every activation too, so the
   two agree whether or not Slint rebuilds the item tree per show. What §H6
   rests on reduces to *the property flips with every activation* — which is
   our code, not Slint's, and the checkmark test pins the binding it feeds.
9. **A right-click during a left drag leaves `scrubbing` stuck — pre-existing,
   and the menu makes it likely.** Verified in `i-slint-core-1.18.0`: the left
   press makes the pointer `TouchArea` the grabber (`items/input_items.rs:124`,
   `Pressed` → `GrabMouse`), and while it holds the grab the window runs
   `handle_mouse_grab` (`window.rs:1003`, `input.rs:1524`) before any
   hit-testing and hands it every event, the right press included — the menu
   area never sees it. The right **release** clears the `TouchArea`'s `grabbed`
   whatever the button (`input_items.rs:154`–`:177`) and returns
   `EventAccepted`, which ends the grab (`input.rs:1617`). The handler ignores
   that up (it tests for the left button), so `scrubbing` stays true; later moves
   no longer reach `moved` (it fires only while `grabbed`, `input_items.rs:186`),
   an `Exit` sends no cancel (nothing is grabbed), and the left up reaches the
   scrubber only if it lands on it by hit-testing. A left up off the track — the usual end of a drag — leaves the
   scrub stuck. Task 1 fixes it: any button's up ends a scrub.

---

## §A. The census, 2026-10-12 at `d8311a0`

Re-derive at each task (`start-task` §4).

| Changing | Sites |
|---|---|
| `Scrubber`'s interface (task 1) | its two instantiations: `app.slint:6754` and the stand-in `tests/ui/scrubber.rs:30`. No other `.slint` file imports it (`app.slint:8` only). |
| The scrub's end condition (task 1, §0.9) | `scrubber.slint:157`–`:159` (the up test); the file's header (`:1`–`:10`) says the area *"sees every end"* and gains the other buttons' ups. No scrubber test sends a non-left button (grepped `PointerEventButton::` in `tests/ui`: the one other is `slate_menu.rs:84`, on a slate row). |
| `State` (task 2) | **No struct literals**: `State` is built only by `State::default()` (`bus/state.rs:542`, `:545`, `:549`) and by serde. The JSON fixtures in `bus/state.rs`'s tests are strings and need no edit. |
| `AppFiles` accessors (task 2) | new; read by `main.rs` only (task 3) |
| Scrub handlers (task 3) | `main.rs:781`–`:782`; `cmd` (`:2347`) keeps three other users (`:763`, `:800`, `:804`), so it stays |
| Readers of `match_abs` | `on_jump_chapter` (`main.rs:1650`) and the new snap — two; `show_match` reads `match_rows` |
| A behaviour's prose (task 3) | `scrubber.slint:28`–`:29` (*"a click on a mark scrubs there like anywhere else"* — no longer true while snapping is on); `match_panel.rs:26`–`:28` and `:220`–`:222` (the marks and `[`/`]` read one list — the snap is a third reader); `match_panel.rs:20`–`:24` (`CHAPTER_TOLERANCE` — say it is not the snap's radius); `app.slint:4469`–`:4471` (`match-marks`' doc); `app.slint:6763`–`:6765` (the `marks:` comment) |
| CLAUDE.md (task 3) | the `state.json` list (`CLAUDE.md:249`–`:255`) gains *whether a drag on the scrubber snaps*; one or two sentences on the snap, beside the scan-speed and stepping paragraphs. (The list already omits the folds and the pen width — not this task's to fix; say so in the PR.) |
| User guide (task 3) | `docs/book/src/guide/watching.md` "Playing and skipping" (`:7`–`:18`) — a bullet; `docs/book/src/guide/match.md:29`–`:31` — one clause. `shortcuts.md` is untouched (no key). No other guide page names the scrubber's drag (grepped: `basket`, `clips`, `exporting`, `slates` mention right-click menus of *rows* only). |
| BACKLOG (task 3) | #102's entry (`:2870`) gains SHIPPED; its index line (`:29`–`:33`) drops *"which #84 does"*; **#131's "When to revisit"** (`:4377`–`:4383`) is a trigger this fires — the scrubber's menu now exists, not only in a spec — and its *"#84 does"* is stale too (§0.3) |
| Tests that assert what changes | none: grepped `tests/ui` and `src` for `scrubber`, `ContextMenu`, `find_by_element_id` — only `text_editing.rs` (fields) and `slate_menu.rs` (`AppWindow::slate-menu`, which keeps its count of two: the new area is `Scrubber::snap-menu`) |

**No site** in `pundit-core`, `pundit-media`, `pundit-harness` or `packaging`.
No `Command`, `Event`, `Preferences` field or format bump.

---

## 1. The scrubber's Slint half: the menu and `snap-seconds`

**First of its PR because it is where the spec's verified-but-unbuilt claims
live**: the menu's hit order against the drag, the bound `checked` (§H6, §0.8),
the element id a test finds it by, and the unit conversion only Slint can do.
It also fixes the stuck scrub of §0.9, which the menu makes likely.

### Files

- `ui/scrubber.slint`
- `ui/app.slint`
- `tests/ui/scrubber.rs`
- `tests/ui/snap_menu.rs` (new), `tests/ui/main.rs` (`mod snap_menu;`)

### Steps

1. **`scrubber.slint`**, beside `thumb` (`:75`):
   - `property <length> snap-radius: 6px;` with the doc from spec §P2, stating
     the rule rather than a table: the basin in seconds is `snap-radius ×
     maximum ÷ travel`, so it widens with the footage and narrows as the
     window widens — one example at most (±21 s on a 90-minute match at the default
     window, §0.1's number, never the spec's).
   - `in property <bool> snap-enabled;`
   - `out property <float> snap-seconds:` exactly spec §P3's expression:
     `root.snap-enabled && root.marks.length > 0 ? root.snap-radius *
     root.maximum / max(root.width - root.thumb, 1px) : 0;` — the denominator is
     `value-at`'s (`:72`), on purpose, so the radius and the mapping cannot
     disagree.
   - `callback toggle-snap();` (§0.5)
2. **The menu, as the last child** — after the pointer `TouchArea` (`:151`) and
   so after the wheel `TouchArea` (`:138`) as well, because a `TouchArea` grabs
   a press of any button and hit-testing is last-declared-first (spec §H2):
   ```slint
   snap-menu := ContextMenuArea {
       Menu {
           MenuItem {
               title: "Snap to match events";
               checkable: true;
               checked: root.snap-enabled;
               activated => { root.toggle-snap(); }
           }
       }
   }
   ```
   **No `enabled`** (spec §C3: it acts on nothing; it must open while a
   recording has disabled the drag). The comment above it carries the three
   reasons — order (above **both** `TouchArea`s: the wheel area is enabled
   during a recording and would take the right press too), no gate, the bound
   `checked` (§H6) — once each.
3. **Any button's up ends a scrub** (§0.9): in the pointer handler's up test
   (`:157`–`:159`), drop `&& event.button == PointerEventButton.left`, so it
   reads `root.scrubbing && (event.kind == PointerEventKind.up || event.kind ==
   PointerEventKind.cancel)`. The `down` test keeps its left-only condition — a
   right press must not start a scrub. A right-click mid-drag therefore ends the
   drag where it had got to and opens no menu (the grab holds the press); the
   left up that follows finds `scrubbing` false and does nothing. The header
   comment's *"sees every end: the pointer-up, and the cancel"* (`:9`–`:10`)
   gains *"of any button"*, and one line on why (the grab ends on any release).
4. **The stale doc**: `marks`' comment (`:28`–`:29`) says a click on a mark
   scrubs there "like anywhere else" — reword to: a mark is where a click
   scrubs to its value, and with `snap-enabled` a drag within `snap-radius`
   of one lands on it (Rust's half).
5. **`app.slint`**:
   - Beside `scrubbing` (`:4105`): `out property <float> scrub-snap-seconds:
     scrubber.snap-seconds;` with a one-line doc ("0 is don't snap").
   - Beside `match-marks` (`:4471`): `in property <bool> snap-to-events: true;`
     — `true` because that is what a fresh `state.json` reads (spec §S4), so the
     window and the file agree before Rust has pushed anything.
   - Beside `scrub-move` / `scrub-release` (`:4624`–`:4625`): `callback
     toggle-snap();` — *the scrubber's menu item; Rust flips the stored bool and
     pushes `snap-to-events` back.*
   - On the `Scrubber` (`:6754`): `snap-enabled: root.snap-to-events;` and
     `toggle-snap => { root.toggle-snap(); }`. The `marks:` comment (`:6763`–
     `:6764`) gains that the empty list in a preview is also what switches the
     snap off there (spec §P3) — so it is not written a second time.
6. **`tests/ui/scrubber.rs`'s stand-in** gains `in property <bool> snap-enabled:
   true;`, `out property <float> snap-seconds: scrubber.snap-seconds;` and the
   two bindings on its `Scrubber`.

### Tests

All in **`tests/ui`** (`cargo test -p pundit-app --test ui`).

- **`scrubber.rs` — `snap_seconds_is_six_pixels_of_track`**: the stand-in
  (Scrubber 400px wide, `maximum: 100`, so 380px of travel) with one mark and
  snapping on reads `6 × 100 / 380 = 1.5789…` (within 1e-4).
- **`scrubber.rs` — `snap_seconds_is_zero_with_nothing_to_snap_to_or_snapping_off`**:
  no marks → `0`; marks but `snap-enabled: false` → `0`.
- **`scrubber.rs` — `a_right_click_mid_drag_ends_the_scrub`** (§0.9): left
  down at 100, drag to 200, then a **right** press and a right release at 200
  (`WindowEvent::PointerPressed` / `PointerReleased` with
  `PointerEventButton::Right`) → `released` fired exactly once, at the value
  the drag reached, and `scrubbing` is false; the left release that follows
  leaves `released` at one.
- **The seven existing `scrubber.rs` tests are the "it cannot fight the drag or
  the wheel" proof** (spec §H2), unchanged: they drive the real `Scrubber`,
  which now carries the menu area above both `TouchArea`s. Run them; nothing to
  add.
- **`snap_menu.rs` (new, the real `AppWindow`, `slate_menu.rs`'s machinery)** —
  a fixture with `can-play` true (so the scrubber is enabled and its drag
  `TouchArea` *would* take a right press if it were on top), the window's
  `toggle-snap` counted, and a helper that right-clicks
  `ElementHandle::find_by_element_id(w, "Scrubber::snap-menu")` (assert exactly
  one — the component-qualified id is `text_editing.rs`'s `Inspector::name`
  form) and returns its descendants:
  - **`the_scrubbers_menu_toggles_the_snap`**: the item *Snap to match events*
    is found by its words and clicked (press + release, as `pick` does);
    `toggle-snap` fired once.
  - **`the_checkmark_follows_the_window`** (the binding,
    §0.8): with `snap-to-events` true, open → a `✓` is among the menu's
    descendants (`menu-base.slint:141` draws it as a `Text`, whose
    `accessible-label` is its text); set `snap-to-events` false and reopen → no
    `✓`. *Check first that a popup's `Text` is reachable by `accessible_label`;
    if it is not, say so in "Where this stands" and move the checkmark to the
    manual list rather than claim the test.*
  - **`the_menu_opens_while_recording`**: `recording-phase` set to
    `RecordingPhase::Recording` (the drag is disabled), right-click → the item is
    found and its click fires `toggle-snap`.

### Sabotage

1. `snap-seconds`' denominator `root.width` instead of `root.width - root.thumb`
   → `snap_seconds_is_six_pixels_of_track` fails (1.5 vs 1.579); the zero test
   stays green.
2. `root.marks.length > 0` dropped → the zero test fails at its no-marks
   assertion; the six-pixels test stays green.
3. `root.snap-enabled &&` dropped → the zero test fails at its off assertion.
   (2 and 3 share one test, at two named assertions; say which line failed.)
4. `snap-menu` moved to **directly between the wheel `TouchArea` and the
   pointer `TouchArea`** → the drag area grabs the right press and the menu
   never opens: `the_scrubbers_menu_toggles_the_snap` and the checkmark test
   fail; **`the_menu_opens_while_recording` stays green** (a disabled
   `TouchArea` ignores the press) — which is the two clauses pinned apart.
   (Not above the wheel area: that one is enabled during a recording, so it
   would grab the right press there too and fail the recording test as well.)
5. `enabled: root.enabled;` added to `snap-menu` → only
   `the_menu_opens_while_recording` fails.
6. `checked: true` instead of `checked: root.snap-enabled` → the checkmark test
   fails at its second assertion.
7. `activated` unwired (empty body) → `the_scrubbers_menu_toggles_the_snap`
   fails.
8. The up test's `&& event.button == PointerEventButton.left` restored → only
   `a_right_click_mid_drag_ends_the_scrub` fails (no release, `scrubbing`
   still true); the seven existing tests stay green, since none sends another
   button.

---

## 2. The Rust logic: `nearest_chapter` and the `state.json` bool

Pure functions and a file field, both in the **library**, both reachable from a
unit test.

### Files

- `src/match_panel.rs`
- `src/bus/state.rs`

### Steps

1. **`match_panel::nearest_chapter(abs: f64, chapters: &[f64], within: f64) ->
   Option<f64>`**, below `previous_chapter` (`:242`), spec §L7's doc plus:
   - **Nearest, within reach**: filter on the strict `(at - abs).abs() <
     within`, then the `at` minimising `|at − abs|` (a `min_by` on the
     distance). The strict `<` is also the whole of "do not snap": `within`
     `0`, negative or NaN admits nothing, so there is no separate guard
     (§P3's single-number contract). The mark under the playhead **is**
     returned — the opposite of `next_chapter`, and the doc says so (spec §C2).
   - **`chapters` are in order, as [`match_abs`] gives them** — the same
     sentence `next_chapter` and `previous_chapter` carry. `min_by` keeps the
     first of equal distances, so a tie goes to the earlier mark.
   - Its doc names the two other readers of `match_abs` (the marks via
     `match_rows`, and `[`/`]`) so the three stay one list (spec §L2).
2. **`CHAPTER_TOLERANCE`'s doc** (`:20`–`:23`) gains one line: it is not the
   snap's radius — wrong units, and the opposite rule.
3. **`State.snap_to_events: Option<bool>`** with `#[serde(deserialize_with =
   "lenient")]` and spec §S2's doc (why `Option`: `lenient` falls back to
   `T::default()`, which for a bare `bool` is **off**). Placed after
   `music_genre` (`bus/state.rs:149`–`:150`). Serialises as `snapToEvents`.
4. **`AppFiles::snap_to_events() -> bool`** (`unwrap_or(true)`, spec §S4's
   reason in its doc) and **`set_snap_to_events(bool)`** (read, set
   `Some(on)`, save — the folds setter's shape, `:441`–`:445`).

### Tests

All in the **`pundit-app` lib** unit tests (`cargo test -p pundit-app --lib`).

- `match_panel::tests`, beside `previous_and_next_chapter_skip_the_one_under_the_playhead`
  (`:534`), one test per rule so a mutation fails one name:
  - `nearest_chapter_takes_the_nearest_not_the_first` — chapters `[100, 104]`,
    `abs` 103, `within` 5 → 104 (distances 3 and 1: the first in order is also
    within reach, and neither is inside `CHAPTER_TOLERANCE`).
  - `nearest_chapter_includes_the_one_under_the_playhead` — chapters `[100,
    104]`, `within` 1: `abs` 104 (exactly on one) → 104, and `abs` 100.3 (inside
    `CHAPTER_TOLERANCE`) → 100. (`within` 1 keeps the other mark out of reach,
    so this test fails only on the rule it names.)
  - `nearest_chapter_breaks_a_tie_towards_the_earlier` — `[100, 104]`, `abs`
    102, `within` 5 → 100.
  - `nearest_chapter_is_none_out_of_reach_or_off` — `[100]` at `abs` 110,
    `within` 5 → `None`; empty slice → `None`; `abs` exactly on the mark with
    `within` `0.0`, `-1.0` and `NaN` → `None` each (the fixture sits *on* a mark,
    distance `0`, so a `<=` is visible at `0.0`).
- `bus::state::tests`:
  - `remembers_the_snap_and_defaults_it_on` (the folds test's shape, `:1067`):
    no file → `true`; `set_snap_to_events(false)` → a **fresh**
    `AppFiles::in_config_dir` reads `false`; `{"pen":"blue","snapToEvents":"yes"}`
    → `true` **and** `pen() == Pen::Blue`.
  - `the_settings_are_independent` (`:981`) gains `set_snap_to_events(false)`
    **after** `set_pen(Pen::Pink)`, `set_window_size` and `set_panel_widths`
    (`:986`–`:996`) have stored non-default values, followed by assertions that
    the pen, the size and the widths survived it — which is what a setter
    starting from `State::default()` would clobber; and, at the end, an
    assertion that the snap still reads `false` after the later setters (a
    clobbered value would read the default, `true`, which is why the stored one
    is `false`).

### Sabotage

1. `next_chapter`'s predicate copied in (`(at - abs).abs() > CHAPTER_TOLERANCE`
   as a second filter beside the reach one) → only
   `…includes_the_one_under_the_playhead` fails (both its assertions). The
   nearest test's distances (3, 1) and the tie test's (2, 2) are all past the
   tolerance, and the `None` cases stay `None`.
2. `.find(…)` (first within reach) instead of the minimum →
   only `…takes_the_nearest_not_the_first` fails (100, not 104). The tie
   test's first is its answer, and the includes test has one mark in reach.
3. Ties to the later (`.rev()` before the `min_by`) → only
   `…breaks_a_tie_towards_the_earlier` fails; every other fixture has a single
   nearest.
4. `<=` for the strict `<` in the reach filter → only
   `…is_none_out_of_reach_or_off` fails, at `within` `0.0` (distance `0 <= 0`).
   `-1.0` and NaN still admit nothing, and no other fixture sits at exactly its
   radius.
5. The accessor `unwrap_or(false)` → `remembers_the_snap_and_defaults_it_on`
   fails at its first assertion.
6. `lenient` removed from the new field → the same test fails at the pen
   assertion (the whole document then reads as default). `the_settings_are_independent`
   stays green.
7. `set_snap_to_events` starting from `State::default()` instead of `self.read()`
   → `the_settings_are_independent` fails at the pen assertion right after the
   snap's setter; `remembers_the_snap_and_defaults_it_on` stays green (it
   stores nothing else through a setter).

---

## 3. The wiring, the guide and the prose

### Files

- `src/main.rs`
- `CLAUDE.md`, `docs/book/src/guide/watching.md`, `docs/book/src/guide/match.md`
- `BACKLOG.md`
- The stale doc comments of §A's "behaviour's prose" row that tasks 1 and 2 did
  not already reword (`match_panel.rs:26`–`:28`, `:220`–`:222`)

### Steps

1. **`wire_snap(window, state)`** beside `wire_folds` (`main.rs:2663`), called
   beside it in `main` (`:438`), on its pattern (spec §S5/§C4): push
   `state.snap_to_events()` once; `on_toggle_snap` flips the **window's**
   property (`let on = !w.get_snap_to_events();`), writes that to `state.json`
   with `AppFiles::set_snap_to_events(on)` and sets it back on the window — no
   re-read of `state.json`. One writer of the window property, flipping it on
   every activation, which is all §H6 now rests on (§0.8). The bus never learns
   the option exists.
2. **`snap_scrub(w: &AppWindow, abs: f32) -> f64`**, one helper for both
   handlers: `within = f64::from(w.get_scrub_snap_seconds())`; under
   `UI.with_borrow`, `match_panel::nearest_chapter(abs, &match_abs(&project),
   within)` on the snapshot's project; when it snaps, `w.set_position_seconds(s
   as f32)` (spec §L5 — the thumb and the readout, `:4260`–`:4261`, follow the
   mark) and return `s`; else return `abs`. **No logic beyond that call** — the
   arithmetic is task 2's and the conversion task 1's.
3. **`on_scrub_move` / `on_scrub_release`** (`:781`–`:782`) stop being `cmd`
   one-liners: each takes a weak window handle, calls `snap_scrub`, sends
   `ScrubMove { abs }` / `ScrubRelease { abs }`. **Both** snap (§0.7).
4. **Prose** (§A): the two `match_panel.rs` docs; CLAUDE.md's `state.json` list
   and one or two sentences on the snap — **the drag alone snaps** (never the
   wheel, the arrows, `,`/`.` or `[`/`]`), and the radius is one constant in
   `scrubber.slint`. The rest lives in the code's own docs.
5. **The guide**: `watching.md` gains a bullet — *dragging the scrubber snaps to
   a goal or period mark once the pointer is within a few pixels of it, and lets
   go when you drag on past; right-click the scrubber to turn **Snap to match
   events** off or on*. `match.md:29`–`:31` adds that dragging the scrubber
   snaps to the marks, linking `watching.md`.
6. **BACKLOG** (§A): #102 SHIPPED with the PR numbers and the corrected
   geometry (§0.1); its index line; #131's revisit line.

### Tests

**None reachable.** `main.rs` is a `[[bin]]`; what is left there is wiring with
no branch of its own (`start-task` §5). `cargo build` proves the generated
setters/getters exist. The pieces it composes are tested in tasks 1 and 2.

### Manual checks (batched at the end, per the port-autonomy note)

On a two-half project at the default window, with a goal and two period marks:

1. Drag across a goal tick: the thumb sticks to it for about six pixels either
   side, the readout shows the goal's own time, then the drag lets go.
2. Release on the tick, then press `]` from before it and `[` from after: all
   three land on the same frame (§L2/§L4).
3. Right-click the scrubber: the item is checked; untick it; the drag no longer
   sticks; quit and relaunch: still unticked, and `state.json` holds
   `"snapToEvents": false`.
4. Right-click the scrubber during a recording: the menu opens.
5. Open a clip preview and drag: nothing snaps (no marks are drawn).
6. Wheel, arrows, `,`/`.` near a tick: exact distances, no snap.

### Sabotage

**None runnable** (no test reaches `main.rs`), and that is recorded rather than
claimed. The realistic bugs here — a handler left as `cmd(bus, …)`, the release
not re-snapping, the toggle not saving — are manual checks 1–3.

---

## Risks

1. **Task 3's wiring is untested** — the `[[bin]]` limit. Mitigated by keeping it
   to one call each way (`snap_scrub`, `wire_snap`) over tested pieces, and by
   manual checks 1–3. A handler left unsnapped fails nothing automated.
2. **The radius is a guess at the default width**: ±21 s on a 90-minute match at
   1600px, ±31 s at the 1100px minimum (§0.1). One constant in
   `scrubber.slint`; tune on real footage, not here. 4px would be ±14 s at the
   default.
3. **`position-seconds` is written from a handler during a drag** — safe because
   the tick treats it as the truth while `scrubbing` (`main.rs:4260`) and the
   mapping is absolute (`scrub-to`, `scrubber.slint:83`–`:86`). A future
   scrubber that anchors on the press invalidates the reasoning (spec Risk 3).
4. **Repeated identical seeks while the pointer moves inside a basin.** Each
   move sends `ScrubMove` at the same snapped `abs`; `Bus::scrub` is keyframe
   seeks, latest wins (`bus/transport.rs:224`–`:236`). Not deduplicated on
   purpose — dedupe is state that would need clearing; revisit only if a drag
   over a tick is visibly heavier than one elsewhere.
5. **`match_abs` allocates per pointer move** (spec Risk 4) — unchanged: a cache
   on `ProjectChanged` if it ever shows, never a second list.
6. **The open question (the selected slate's span) is unanswered.** The default
   ships — match events only; widening is a list passed to `nearest_chapter`
   and a renamed item, with no storage change.

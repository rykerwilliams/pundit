# Plan — the Settings sheet, and the slate band's colour in it (#131)

Spec: `docs/superpowers/specs/2026-10-13-settings-sheet-band-colour-design.md`.
Read it and `CLAUDE.md` first; neither is repeated here, except where §0 below
corrects the spec. **Every task starts with the `start-task` skill and ends with
`verify`, `sabotage-proof` and `recheck-claims`.**

Every `path:line` here was re-derived on **2026-10-13 at `114f2e4`**
(`claude/131-spec`: `main` at `c8eca27` plus the spec and its two BACKLOG index
lines). **Re-derive every line at `start-task` §3** — `app.slint` drifts
fastest, and each task here moves the lines of the next. Paths are under
`crates/pundit-app/` unless they say otherwise.

## Where this stands (update it as tasks land)

- **1 — `ColorPicker` holds its own position.** Not started.
- **2 — the `state.json` field.** Not started.
- **3 — the Settings sheet, its two doors, the band and the row.** Not started.
- **4 — the prose: CLAUDE.md, the guide, BACKLOG, #78's spec.** Not started.

**Shipping: one PR, a commit per task, in order.** No commit puts a control on
`main` that does nothing:

- Task 1 is a refactor of a shipped sheet plus labels on its swatches.
- Task 2 is a file field and two accessors nothing calls yet.
- Task 3 lands the button, the menu item, the sheet, the band and row binding
  and the Rust that stores the colour **together** — any one of them alone would
  be a door to nothing, or a colour that is forgotten or never shown.
- Task 4 is words.

Task 1 goes first: it moves code in a shipped sheet that no test covers today,
and its regression tests are written **before** the move.

---

## §0. Where the spec is out of date or wrong (checked 2026-10-13)

1. **`changed` handlers run on the next change-handler pass, not at the
   assignment.** §U3's `changed slate-band-hex` fills `slate-band-good` and fires
   `slate-band-edited`. Mouse and key dispatch already run that pass —
   `ensure_tree_instantiated` (`i-slint-core-1.18.0/window.rs:805`) is called at
   the top of the pointer and key paths (`:840`, `:1143`, `:1252`) — so a test
   that **types or clicks** and then reads needs nothing more. What does not
   flush is **`invoke_accessible_default_action`** and a **Rust `set_*`**. After
   those, a test calls `mock_elapsed_time(Duration::ZERO)`, which runs the pass
   (`i-slint-backend-testing-1.18.0/testing_backend.rs:58`); the fixture names it
   `settle(&w)`.
2. **The setup sheet's regression test cannot be written before the move as the
   spec words it.** It presses *White* by its label, which is spec §P4's and does
   not exist yet (today the name is only a `Tooltip`, `app.slint:2752`). So task 1
   adds the labels first, which changes no behaviour, then writes the tests, runs
   them green, and only then moves the code.
3. **The spec's tests never exercise the moved sync.** A kit press writes its hex
   as it stands (`app.slint:2750`) and never touches the held position, so
   nothing fails if `sync-from-text` or the `take` write-back is broken. Added:
   **one area test per host**. With the field at `#0000ff` (hue 240), a click at
   the centre of `ColorPicker::area` (`:2790`; the area is `root.span` × 110px,
   `:2759`–`:2761`) must give `color_picker::hex_of_hsv(240.0, 0.5, 0.5)`, which
   is `#404080`. With the hue never read it gives `#804040`. On the setup sheet
   the picker is created on open, so this pins the **`init`** sync. On the
   settings sheet the hex is typed in while the picker is showing, so it pins
   **`changed text`**.
4. **§P3's "chrome" range is two lines too wide.** `app.slint:2719`–`:2720` are
   `width`/`height: body.preferred-*` — the picker's sizing, which the inline one
   needs too — and stay. The chrome is `:2721`–`:2730`. **The clamp moves with
   it**: `x:`/`y:` (`:3161`–`:3162`) use `self.width`/`self.height` and go onto the
   `Rectangle` that now carries the chrome.
5. **Two same-value writes the spec calls harmless, located:** `wire_settings`'
   startup set, and `close-settings()`'s reseed, each fire `changed` with the
   value already stored. A test counting `slate-band-edited` calls counts what
   happens after its fixture has settled.
6. **The spec's Default test cannot catch a wrong default colour.** It compares
   against the colour read before anything was typed, which is the same default.
   The default's *identity* is manual check 4; the test pins the *return* to it.
7. **Three comments go stale with the move that the spec does not list:**
   `main.rs:1712`–`:1713` (*"the sheet gates on `valid-hex` before it moves the
   picker"* — the picker gates now), and `KitColor`'s doc
   (`app.slint:178`–`:179`) and `color_picker.rs:1`, both of which say *"the
   setup sheet's picker"*.

---

## §A. The census, 2026-10-13 at `114f2e4`

Only the entries that are not obvious from the spec. Re-derive at each task
(`start-task` §4).

| Changing | Sites |
|---|---|
| **Sheet counts in prose** (task 3) | **Go** — each loses its number, because the list is the count and every one went stale the same way: `app.slint:1917` (*"There are seven of these"*, whose list gains Settings); `:1929`–`:1930` (the parenthetical about *"this comment's own count above it"*, which has no count to refer to once `:1917` loses it); `:1935` (*"all six sheets"*); `:2988`–`:2989` (*"the other five sheets"*); `tests/ui/sheet_scroll.rs:4`, `:7`, `:13`, `:14`–`:15`, `:33`, `:45`; `tests/ui/export_sheet.rs:319` (*"of the six"*); `CLAUDE.md:738` (*"all six"*); `CLAUDE.md:845` (the export queue's *"no eighth `Sheet`"* → *"no `Sheet` of its own"*). **Stay**: `app.slint:1927` (*"two of the **seven**"* — a dated measurement); `CLAUDE.md:691` (*"the seventh `Sheet"* — still Keys); `app.slint:2150` (*"rather than on a Settings sheet"* — the music folder did not move). |
| Tests touched | `tests/ui/settings_sheet.rs` (new; `mod` in `tests/ui/main.rs:70`–`:84`); `snap_menu.rs` (its `ITEM`, `:18`, is matched exactly, so a second item leaves its three tests alone); `text_editing.rs` (`:252`, and its header's field count `:19`–`:20` — the site count does not move, `SetupField::edit` already being one); `sheet_scroll.rs`' `sheets()` (`:47`–`:108`), iterated by all four of its tests (`:190`, `:240`, `:280`, `:306`). |
| Side fix in `sheet_scroll.rs` | **The Keys sheet joins `sheets()` too.** It was never added when #96 shipped it, though the header calls the test *"a tripwire for the seventh"* (`:14`–`:15`) — #134's gap, fixed here because the Settings entry sits beside it. Opening it is `set_keys_sheet_open(true)`: the file's `window()` already wires the keys (`:121`). |

**No site** in `pundit-core`, `pundit-media` or `pundit-harness`. No `Command`,
`Event`, `Preferences` field or format bump.

---

## 1. `ColorPicker` holds its own position

### Files

- `ui/app.slint`
- `src/color_picker.rs` (header only), `src/main.rs` (one comment)
- `tests/ui/settings_sheet.rs` (new), `tests/ui/main.rs` (`mod settings_sheet;`)

### Steps

1. **Labels first (§0.2)**, which changes no behaviour. On the kit swatch
   `Rectangle` (`app.slint:2738`): `accessible-role: button; accessible-label:
   kit.name; accessible-action-default => { root.pick(kit.hex); }`.
2. **Write the two tests below and run them green** against the code as it
   stands. Record that in "Where this stands".
3. **Move the held position into `ColorPicker`** (spec §P2):
   - **It loses** `in property <float> hue/sat/val` (`:2704`–`:2706`) and
     `callback take` (`:2712`).
   - **It gains** `in property <string> text`, `pure callback hsv-of-hex`, `pure
     callback valid-hex`, the private `picker-hue`/`-sat`/`-val` and `text-hsv`,
     and `sync-from-text` as it stands (`:2953`–`:2968`). In that function
     `picker-text` is read as `text`, and `picker-field >= 0` is dropped.
   - It also gains `changed text => { sync-from-text(); }` and `init => {
     sync-from-text(); }`.
   - `take-area`/`take-hue` (`:2835`–`:2849`) set the position and call
     `root.pick(root.hex-of-hsv(…))` themselves.
4. **`MatchSetupSheet`** loses `:2943`–`:2970` and the `hue`/`sat`/`val`/`take`
   lines of its instantiation (`:3164`–`:3177`). Its popup becomes:
   - the backdrop `Rectangle` as now (`:3152`–`:3155`);
   - inside it, a `Rectangle` carrying the chrome (`:2721`–`:2730`) and the clamp
     `x`/`y` (`:3161`–`:3162`), sized to its child (§0.4);
   - that child is `ColorPicker { kit-colors; text: root.picker-text; the four
     colour callbacks forwarded; pick(hex) => { root.write(hex); } }`.
5. **Docs:**
   - `ColorPicker`'s says it is used inline and as a popup, and that it holds
     where it stands, with the grey-hue reason moving with the code.
   - `KitColor`'s and `color_picker.rs:1` say *"the colour picker's"*.
   - `main.rs:1712`–`:1713` says the picker gates (§0.7).

### Tests

These live in **`tests/ui`** (`cargo test -p pundit-app --test ui`), in
`settings_sheet.rs`.

The fixture installs `hex-color`, `valid-hex`, `hsv-of-hex`, `hex-of-hsv` and
`kit-colors` from `pundit_app::match_panel::parse_hex` and
`pundit_app::color_picker` **before** it sets or reads anything. That is a copy of
`main.rs:1690`–`:1725`'s wiring, because `main.rs` is unreachable. It also defines
`settle` (§0.1).

**Every query is scoped to the `MatchSetupSheet` element**
(`find_by_element_type_name`), not to its `card := Sheet`: the popup is the
card's *sibling* (`app.slint:3152`), so a query inside the card never finds
*White*.

- **`a_kit_colour_lands_in_the_setup_field`**:
  - set `match-sheet-open`;
  - click the first `SetupField::touch` under `MatchSetupSheet`, and assert
    `match_picker_field == 0` (the home primary);
  - press *White* with `invoke_accessible_default_action`, then `settle`;
  - expect `match_home_primary == "#ffffff"` (`KIT_COLORS[0]`,
    `color_picker.rs:39`–`:42`).
- **`the_setup_pickers_area_reads_the_fields_hue`** (§0.3):
  - set `match_home_primary` to `"#0000ff"` before the sheet opens;
  - open the picker as above;
  - single-click `ColorPicker::area`;
  - expect `match_home_primary == color_picker::hex_of_hsv(240.0, 0.5, 0.5)`.

### Sabotage

Each one is made **after** the move.

1. `init => { sync-from-text(); }` removed → only
   `the_setup_pickers_area_reads_the_fields_hue` fails (`#804040`).
2. `take-area` sets the position without calling `root.pick` → only the area
   test fails.
3. The kit swatch's `accessible-action-default` emptied → only
   `a_kit_colour_lands_in_the_setup_field` fails.

The chrome and the clamp are looks and placement, so no test covers them:
manual check 6.

---

## 2. The `state.json` field

### Files

- `src/bus/state.rs`

### Steps

1. **`State.slate_band_color: Option<String>`**, with `#[serde(deserialize_with =
   "lenient")]` and spec §S2's doc, after `snap_to_events` (`:157`–`:158`). It
   serialises as `slateBandColor`. `state.rs` imports
   `crate::match_panel::{hex, parse_hex}` and `pundit_core::stroke::Rgba`.
2. **`AppFiles::slate_band_color() -> Option<Rgba>`**:
   `self.read().slate_band_color.as_deref().and_then(parse_hex)`.
3. **`set_slate_band_color(Option<Rgba>)`**: read, set `color.map(hex)`, save —
   the shape of `set_snap_to_events` (`:531`–`:536`).

### Tests

These are **`pundit-app` lib** unit tests (`cargo test -p pundit-app --lib`) in
`bus::state::tests`.

- **`remembers_the_slate_band_and_defaults_to_the_palette`**, on the shape of
  `remembers_the_snap_and_defaults_it_on` (`:1096`):
  - no file → `None`;
  - store red → a fresh `AppFiles::in_config_dir` reads red;
  - `set_slate_band_color(None)` → `None`;
  - `{"pen":"blue","slateBandColor":5}` → `None` **and** `Pen::Blue`;
  - `{"pen":"blue","slateBandColor":"teal"}` → `None` and the pen.
- **`the_settings_are_independent`** (`:1004`) gains `set_slate_band_color(red)`
  after the setters already there. Its assertions check that the earlier values
  survived, and at the end that the band still reads red.

### Sabotage

1. `lenient` removed from the field → the new test fails at the `5` row's pen
   assertion.
2. A bad string read as black — `.map(|s| parse_hex(s).unwrap_or(Rgba { r: 0.0,
   g: 0.0, b: 0.0, a: 1.0 }))` (`Rgba` has no `Default`) → only the `"teal"` row
   fails.
3. The setter ignores `None` and keeps the old value → only the `set(None)` row
   fails.
4. The setter starts from `State::default()` → only
   `the_settings_are_independent` fails.

---

## 3. The Settings sheet, its two doors, the band and the row

### Files

- `ui/app.slint`, `ui/scrubber.slint`
- `src/main.rs`
- `tests/ui/settings_sheet.rs`, `snap_menu.rs`, `text_editing.rs`,
  `sheet_scroll.rs`, `export_sheet.rs` (one comment)
- `CLAUDE.md` (`:738` and `:845` only — §A's counts)

### Steps

1. **The window** — spec §U3 verbatim:
   - the five properties;
   - the `changed slate-band-hex` handler;
   - `slate-band-edited`;
   - `open-settings()`, whose doc says it **deliberately has no guard**: each
     door's `enabled` is the gate, and the menu item's test relies on that;
   - `close-settings()`.

   Place the band properties beside `slate-span-from`/`-to` (`:4153`–`:4154`), the
   sheet's beside `keys-sheet-open` (`:4590`–`:4592`), and the functions beside
   `close-basket` (`:4895`–`:4901`). `slate-band-color`'s doc takes the reasoning
   from `scrubber.slint:125`–`:133`.
2. **`SettingsSheet inherits Sheet`** (spec §U1–§U2), after `KeysSheet`:
   - `band := SetupField` with no swatch, and **Default** beside it;
   - the inline `ColorPicker`;
   - **Close**.
3. **The `Scrim`**, just before the Keys sheet's (`:7525`); **`handle-key`**:
   spec §U6's branch after the Keys sheet's (`:5007`–`:5012`).
4. **The button**, after `Keys` (`:7159`–`:7173`):
   - `Settings…`, with `enabled: root.can-open-settings` and `clicked => {
     root.open-settings(); }`;
   - tooltip *"Settings for this computer"*, which does not name the band and so
     does not go stale with a second tenant.
5. **The menu item**, spec §U9:
   - `Scrubber` gains `in property <bool> can-open-settings;` and `callback
     open-settings();` beside `toggle-snap` (`scrubber.slint:96`);
   - the `MenuItem` goes under the snap item in `snap-menu` (`:198`–`:209`);
   - the menu's comment (`:190`–`:197`) gains one sentence: the menu is ungated,
     so the item carries the sheet's gate;
   - the instance (`app.slint:6763`) forwards both, beside `toggle-snap`
     (`:6786`);
   - `snap_menu.rs`' header (`:1`–`:9`) widens from the snap to the menu.
6. **The band and the row:**
   - `Scrubber` gains `in property <color> band-color`;
   - `scrubber.slint:134` becomes `root.band-color.transparentize(0.25)`;
   - `:115`–`:117` (the teal sentence) and `:125`–`:133` go, leaving a pointer to
     the window's property;
   - the instance gets `band-color: root.slate-band-color;`;
   - the row's `background` (`app.slint:5692`–`:5694`) reads
     `root.slate-band-color`.
7. **`wire_settings(window, state)`** in `main.rs`, beside `wire_snap`
   (`:2710`–`:2721`), and called after it (`:439`), which is after
   `wire_callbacks` (`:427`):
   - register `on_slate_band_edited(|t| state.set_slate_band_color(parse_hex(&t)))`
     **first**;
   - then `set_slate_band_hex(state.slate_band_color().map(hex).unwrap_or_default().into())`.

   `hex` joins the `match_panel` import (`:38`–`:40`).
8. **The counts** — every "Go" in §A.

### Tests

These live in **`tests/ui`** (`cargo test -p pundit-app --test ui`).

**`settings_sheet.rs`.** The fixture also records `slate-band-edited`'s
arguments. The button is a `const` `"Settings…"` (U+2026), and the field is
`SettingsSheet::band` → `SetupField::edit`.

- **To replace the field's text**, select all with **Ctrl+A** dispatched through
  the window (press `Key::Control`, then `"a"`, then release both, as
  `keys.rs:270` builds a chord), then type. **Never clear with Backspace**: that
  passes through `""`, which is valid. It would report `""`, put the band back on
  the default, and make a "nothing reported" assertion fail for the wrong reason.
- **`the_settings_button_opens_the_sheet_and_close_shuts_it`**: button → open;
  **Close** → shut. Esc closing it is `sheet_scroll`'s, once `SettingsSheet`
  joins `sheets()`.
- **`esc_in_the_field_leaves_it_before_it_closes_the_sheet`**:
  - click the field and type `r`;
  - assert `get_slate_band_hex() == "r"`;
  - Esc → still open; Esc → shut.
- **`the_settings_button_is_off_while_recording_or_previewing`**:
  `accessible_enabled() == Some(false)` with `recording_phase` `Recording`
  (`settle`), then the same with `previewing_clip` `"c1"`, and `Some(true)` with
  neither.
- **`a_typed_colour_reaches_the_band_and_is_reported`**: type `#ff0000` →
  `get_slate_band_color()` is red, and the last call recorded is `"#ff0000"`.
- **`a_typo_is_never_reported_and_does_not_outlive_the_sheet`**:
  - after `#ff0000`, Ctrl+A and type `#12` → no new call, and the colour is
    still red;
  - Esc, Esc, then reopen → the field reads `#ff0000`.
- **`default_takes_the_colour_back`**:
  - read the colour, then type `#ff0000`;
  - press **Default** → the colour equals the first read, the last call is `""`,
    and Default reports disabled (§0.6).
- **`the_inline_picker_follows_typed_text`** (§0.3): type `#0000ff`, click
  `ColorPicker::area` → the field reads `hex_of_hsv(240.0, 0.5, 0.5)`. No
  `settle` is needed between the two: the click's dispatch runs the pass (§0.1).

**`snap_menu.rs`**, with its fixture and helpers (`window` `:22`, `open` `:37`,
`find` `:47`):

- **`the_band_colour_item_opens_the_settings`**: right-click, click *Band
  colour…* → `get_settings_sheet_open()`.
- **`the_band_colour_item_is_off_while_recording`**: with `recording_phase`
  `Recording`, the menu opens (that is already
  `the_menu_opens_while_recording`'s, `:92`), but clicking *Band colour…* leaves
  the sheet shut.

**`text_editing.rs`**: `a_letter_typed_in_a_sheet_field_reaches_the_field`
(`:252`) opens the settings sheet and probes `SettingsSheet::band`'s
`SetupField::edit`, reading `get_slate_band_hex`.

**`sheet_scroll.rs`**: `sheets()` gains `SettingsSheet`
(`set_settings_sheet_open(true)`) and `KeysSheet` (`set_keys_sheet_open(true)`,
§A's side fix). All four of its tests then cover both.

### Sabotage

1. The button's `enabled: true` → only
   `…button_is_off_while_recording_or_previewing` fails, at its recording
   assertion.
2. `can-open-settings: !root.recording` (previewing dropped) → the same test
   fails, at its previewing assertion.
3. The `changed` handler's validity test removed → only
   `a_typo_is_never_reported…` fails (`"#12"` is reported).
4. `close-settings()` without the reseed → only `a_typo_is_never_reported…`
   fails, at its reopen assertion.
5. **Default** writes `"#000000"` instead of `""` → only
   `default_takes_the_colour_back` fails.
6. `&& !root.text-editing` dropped from the settings branch → only
   `esc_in_the_field_leaves_it…` fails, at its first Esc.
7. `ColorPicker`'s `changed text => …` removed → only
   `the_inline_picker_follows_typed_text` fails. Task 1's area test is created
   on open, so `init` carries it.
8. **The settings branch removed from `handle-key`:**
   - What happens: Esc no longer closes the sheet, and the arrows reach the
     window behind it.
   - Fails: `sheet_scroll::esc_still_closes_every_sheet`,
     `sheet_scroll::a_scrolling_sheet_does_not_take_the_arrow_keys`,
     `esc_in_the_field_leaves_it…` (at its second Esc), and
     `a_typo_is_never_reported…`. The sheet never closes, so nothing reseeds,
     and the "reopened" field still reads `#12`.
   - Stays green: `…opens_the_sheet_and_close_shuts_it`, since Close does not go
     through `handle-key`.
9. The menu item's `enabled` removed → only
   `the_band_colour_item_is_off_while_recording` fails. `open-settings()` has no
   guard, so nothing else catches it.
10. The item's `activated` emptied → only
    `the_band_colour_item_opens_the_settings` fails.

**Not runnable:** `wire_settings` in `main.rs`, the band's `band-color` binding
and the row's `background`. A dead one passes everything above; see manual checks
2–5.

---

## 4. The prose: CLAUDE.md, the guide, BACKLOG, #78's spec

### Files

- `CLAUDE.md`
- `docs/book/src/guide/slates.md`, `watching.md`, `install.md`
- `BACKLOG.md`
- `docs/superpowers/specs/2026-10-02-app-settings-design.md`

### Steps

1. **CLAUDE.md**:
   - The `state.json` list (`:249`–`:255`) gains *the slate band's colour*.
   - After the Keys sheet's paragraph (`:691`ff), add two sentences:
     - The Settings sheet is a `Sheet` beside Keys, and its two doors — the
       drawing row's button and the scrubber's *Band colour…* — share one gate,
       `can-open-settings`, with no guard in `open-settings()`.
     - The colour is stored only on a valid change (a hex, or empty for the
       default), never a half-typed one.
2. **The guide**:
   - `slates.md:18`–`:19`: the range is drawn in the slate colour, which
     **Settings…** (or right-click the scrubber → **Band colour…**) changes, for
     the band and the selected row alike.
   - `watching.md:19`–`:23`: the snap bullet's right-click names both items.
   - `install.md:44`: the `state.json` row gains the colour.
3. **BACKLOG**:
   - **#131** (`:4381`–`:4414`): SHIPPED with the PR number, and its index line
     (`:34`–`:39`) leaves *Next*.
   - **#78**: the first-tenant paragraph (`:1620`–`:1627`) records that #131 built
     the sheet, and its index line (`:19`–`:26`) closes.
   - **#129's revisit trigger** (`:4347`, *"with any settings surface"*) has
     fired. It is answered: this sheet holds machine values and
     `pip_for_new_recordings` is the project's.
   - **#84** (`:1875`) and **#102** (`:2946`): *"unspent"* becomes "spent by
     #131".
4. **#78's spec**: the 2026-10-12 callout (`:320`–`:325`) becomes a dated note
   that #131 built the sheet, pointing at this spec.

### Tests

None, because this task is words. Run `recheck-claims` on every line it touches.

---

## Manual checks (batched at the end)

On a project with two slates and a highlight, at the 1100px floor and at the
default window:

1. **`Settings…` fits** beside `Keys` at 1100px, and the notice still shows. It
   is greyed during a take and during a preview.
2. Select a slate, open **Settings…** and press a kit colour. Behind the scrim,
   the band **and** the selected slate's row change at once. Close: both keep it.
3. Drag in the picker's area: the band follows. Quit and relaunch: the colour is
   back, and `state.json` holds `"slateBandColor": "#rrggbb"`.
4. **Default**: the band and the selected slate's row return to the app's
   selection colour. Compare by eye with the selected highlight's row in the
   Highlights panel (`app.slint:1613`), which draws the same
   `Palette.selection-background`. `state.json`
   holds `null`.
5. Type `#12`, close, reopen: the field shows the last colour, and the band never
   changed. Hand-edit `state.json` to `"FF0000"` → red; `"teal"` → default.
6. **The setup sheet's picker is unchanged to the eye**: border, shadow, clamped
   inside the card at the away column's swatches and the Text row's. Reopening
   on a grey field starts the strip at red (spec §P3's named change).
7. Right-click the scrubber: **Band colour…** opens the sheet. During a take it
   is greyed, and *Snap to match events* still toggles.
8. Pick **White** and read the selected row's words (spec Risk 1).

---

## Risks

1. **The wiring and both leaves are untested**: `wire_settings`, `band-color`'s
   binding and the row's `background`. They are kept to two calls and two
   bindings over tested pieces; manual checks 2–5 cover them.
2. **Per-drag writes** (spec Risk 3): one document rewrite per pointer move in
   the area, on the UI thread. Each also widens the existing lost-update window
   between the two `AppFiles` handles (`bus/state.rs:173`–`:177`) a little. If
   either shows, the single write goes on the area's pointer-up.
3. **The extraction touches a shipped sheet.** Task 1's tests run on both sides
   of the move; the looks are manual check 6.
4. **A test that reads too early after `invoke_accessible_default_action` or a
   Rust `set_*`** reads the previous value (§0.1). Typing and clicking flush on
   their own; those two do not, and the fixture's `settle` is for them.
5. **`Settings…` may crowd the drawing row at 1100px** with a recording's status
   showing. The notice is the stretch item with `min-width: 0` and gives way
   first (manual check 1).
6. **A pale colour washes out the row's text** (spec Risk 1). Manual check 8
   covers it; the cure, if it bites, is the shape of `highlight::label_ink`.

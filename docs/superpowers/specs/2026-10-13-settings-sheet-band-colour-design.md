# Design — the slate band's colour, and the Settings sheet it builds

**BACKLOG #131.** The coach (2026-10-05): *"slate bands look dope tho"*, then
*"i mean the band color should be in the settings"* — and, of the first colour
the band was given, *"the slider bar colour clashes a bit"*.

**Decided by the coach, 2026-10-13:**

- **A full colour picker, in a new Settings sheet** — over preset swatches on
  the scrubber's right-click menu. So #131 is the **first tenant of the Settings
  sheet** that #78's spec designed (`2026-10-02-app-settings-design.md` §U4) and
  nobody built.
- **The selected slate's row takes the colour too**, so the band and its row
  still match (§U8).
- **The scrubber's right-click menu gets a "Band colour…" item** that opens the
  sheet (§U9).

Read `CLAUDE.md` first; this spec does not repeat its `state.json`, sheet,
`text-editing` or UI-test rules. Every `path:line` below was re-derived from
`main` at `c8eca27`.

**The design, in one paragraph.** One `state.json` field, `slateBandColor`, a
`#rrggbb` string read leniently, where absent means today's palette colour. The
window holds it as text and derives one colour from it, which the band on the
scrubber and the selected slate's row both draw. **Settings…** in the drawing
row, after `Keys`, and **Band colour…** on the scrubber's menu open the eighth
`Sheet`, an ordinary one: the match setup sheet's `SetupField` (a hex field), a
**Default** button, and the `ColorPicker` **inline under them, always shown** —
there is one colour, so there is no popup. A change shows on the band at once and
is stored the moment it is a colour or empty; a half-typed one never is. **One
extraction:** the picker's held position and its sync with the text move from
`MatchSetupSheet` into `ColorPicker`, so the grey-hue logic exists once.

---

## §W. What is asked for

**W1. The band and its row — nothing else.** The band is one `Rectangle` with
one literal (`ui/scrubber.slint:118`–`:135`, the colour at `:134`,
`Palette.selection-background.transparentize(0.25)`); its row is one binding
(`app.slint:5692`–`:5694`). The track's other three marks (a goal's amber, a
start/stop's white, the rail's accent blue) and every other list's selection are
not touched: a palette is a bigger feature than was asked for.

**W2. Why a sheet and not the menu.** A `Menu` holds `MenuItem`s, not a hue strip
and a hex field — #78's own reason for #84's folder path (`BACKLOG.md:1621`–`:1623`:
*"a field and a file-chooser button need a surface, which a menu item cannot
be"*). So the picker clears #78 §U1's bar, and the menu gets a door to it
instead (§U9).

---

## §S. Storage: one `state.json` field, a hex string, absent is the palette

**S1. `state.json`, not `project.json`.** A colour the coach reads the scrubber
by is this machine's and his eyes', not the match's — `bus/state.rs`'s header
(`:1`–`:9`) is the test. No `Preferences` field, so **no format bump**.

**S2. The field** — one attribute, as #100 made every new key
(`bus/state.rs:56`–`:62`):

```rust
/// The slate band's colour (BACKLOG #131), `#rrggbb`. `None` is the palette's
/// selection colour — what the band was before this field existed.
#[serde(deserialize_with = "lenient")]
slate_band_color: Option<String>,
```

`lenient` (`bus/state.rs:178`) turns a JSON type error into `None` and costs this
field alone; the container's `default` (`:64`) covers an absent key. A string
that is not a colour (`"teal"`) is valid JSON, so it is the accessor's job —
`pen`'s shape (`:395`–`:401`).

**S3. A hex string, not the team colours' `Rgba` object.** Team colours are
`{"r":…,"g":…,"b":…,"a":…}` floats in `project.json`
(`pundit-core/src/scoreboard.rs:29`–`:36`) because the macOS format wrote them so;
`state.json` has no such inheritance and keeps hand-readable values. And the
picker's currency **is** the hex string — it converts only through
`match_panel::hex` / `parse_hex` (`match_panel.rs:371`, `:383`) so that it *"has to
land on exactly those bytes"* (`color_picker.rs:5`–`:10`). Storing that string
means the field, the band and the file cannot disagree in the last bit.

**S4. The accessors take and give a colour.**

```rust
/// The slate band's colour, if the coach chose one. `None` — absent, a
/// type error, or text that is not `#rrggbb` — is the palette's colour.
pub fn slate_band_color(&self) -> Option<Rgba>          // through parse_hex
pub fn set_slate_band_color(&self, color: Option<Rgba>) // Some(hex(c)) or None
```

One place parses, one place spells; the string stops at the file.

**S5. Absent means the palette's colour, and a stored colour is absolute.** The
window is pinned to dark (`Palette.color-scheme = ColorScheme.dark`,
`app.slint:4868`–`:4870`), so there is no light/dark interaction today. If a light
scheme ever comes, `None` follows it and `Some` does not: the coach picked it
against his footage, not the chrome. **Default** (§U2) is the way back to the
following colour.

**S6. Opaque; the band keeps its own translucency.** `parse_hex` always returns
alpha 1 (`match_panel.rs:367`–`:369`). `transparentize(0.25)` is how the *band* is
drawn — a wash under the rail and the ticks (`scrubber.slint:109`–`:117`) — so it
stays in `scrubber.slint`; the row stays opaque (§U8).

**S7. Stored on each change that is a colour or empty, through the UI thread's
`AppFiles`.** The bus never reads the band, so it goes through `machine_state`
(`main.rs:406`) as the pen width, panels, folds and snap do
(`main.rs:2645`–`:2721`, `:2752`–`:2778`), with no `Command`.

- **What is stored:** the window reports a value only when it is a hex or `""`
  (§U3); Rust stores `parse_hex(text)` — `""` is `None`. A half-typed `#12` is
  never stored, so a typo cannot cost the stored colour, and closing the sheet
  puts the field back on the last good value (`close-settings`, §U3), which is
  what reopening it shows.
- **When, and why this differs from `wire_panels`.** The panels write only on a
  drag's release (`main.rs:2645`–`:2670`, and `bus/state.rs:544`–`:546`: *"a drag
  is 30 events a second and every setter here rewrites the whole document"*),
  because a release is the one moment every resize ends at. The colour has four
  ways in — typing, a kit swatch, a drag in the area, Default — and only the drag
  has a release, so a release hook is one writer for one of four and three more
  besides. **One `changed` handler on the window covers all four.** The price is
  one temp-file-and-rename of a ~1 KB document per pointer move, on the UI
  thread, only while the coach drags inside a 306×110px area — accepted (Risk 3).

---

## §P. The picker: one extraction

**P1. Reused as it stands.** `SetupField` (`app.slint:2617`–`:2686`) — caption,
validity ✕ and a `LineEdit` bound out with `text <=>` — and `ColorPicker`'s
drawing (`:2688`–`:2850`: kit row, saturation/brightness area, hue strip, every
colour through Rust's conversions).

**P2. Extracted: the held position, into `ColorPicker`.** Today a host carries
`picker-hue`/`-sat`/`-val` and `text-hsv` (`app.slint:2943`–`:2951`), held rather
than read off the hex because grey has no hue and black neither hue nor
saturation; `sync-from-text` and its two `changed` handlers (`:2953`–`:2970`); and
the `take` write-back (`:3172`–`:3177`). About 25 lines, and the grey-hue logic is
the part a second copy would let drift. They move inside, and the interface
becomes:

```slint
component ColorPicker inherits Rectangle {
    in property <[KitColor]> kit-colors;
    /// The field the picker shows and writes: it follows this text.
    in property <string> text;
    pure callback hex-color(string) -> color;
    pure callback hsv-of-hex(string) -> Hsv;
    pure callback hex-of-hsv(float, float, float) -> string;
    pure callback valid-hex(string) -> bool;
    /// A colour for the field: a kit colour as it stands, or a point dragged.
    callback pick(string);
}
```

`take` becomes internal and ends in `pick(hex-of-hsv(…))`; `changed text` and
`init` call `sync-from-text`.

**P3. What `MatchSetupSheet` keeps, unchanged:** its backdrop, clamp and anchor
(`:2972`–`:2985`, `:3147`–`:3162`), its six-way `picker-text` / `write` dispatch
(`:2926`–`:2941`) and the window-owned `picker-field` (`:2876`–`:2882`). It passes
`text: root.picker-text` and `pick(hex) => { root.write(hex); }`.

- **The popup's chrome moves to it.** `ColorPicker`'s background, border, radius,
  drop shadow and swallowing `TouchArea` (`:2719`–`:2730`) are what make it a
  popup, and inline in a sheet a drop shadow would float. They go on a
  `Rectangle` round the `ColorPicker` inside the setup sheet's backdrop
  (`:3152`); the picker keeps only `body`.
- **One small behaviour change.** The setup sheet's picker sits in an `if`
  (`:3152`), so with the position inside it the remembered hue lasts one open
  instead of one visit to the sheet: reopening on a **grey** field starts the
  strip at red. The colour never moves — only the strip's marker on a colour that
  has no hue.

**P4. Kit swatches become findable by their words.** Each gets
`accessible-role: button`, `accessible-label: kit.name` and
`accessible-action-default => { root.pick(kit.hex); }` (the swatch is
`app.slint:2738`–`:2752`; its name is today only a `Tooltip`). That is what lets a
test press *White* rather than a computed position, in both sheets.

---

## §U. The sheet, the window, the scrubber and the row

**U1. `SettingsSheet inherits Sheet`** — the eighth, verified: export, basket,
match setup, the match event editor, New match, Keys and the error dialog are one
`Scrim` each (`app.slint:7331`–`:7552`), as `Sheet`'s doc lists them
(`:1917`–`:1918`). An **ordinary** `Sheet` — no popup, so none of
`MatchSetupSheet`'s wrapper — at `card-width: 440px` (the picker is
`14 × 22 − 2 = 306px` wide, `:2717`, with 14 kit colours, `color_picker.rs:38`–`:95`).
It states no `height`, per #134.

**U2. Its contents.**

```slint
HorizontalLayout {                        // the row: the next tenant is another
    spacing: 8px;
    SetupField {
        label: "Slate band";
        placeholder: "Default";
        ok: root.band-hex == "" || root.valid-hex(root.band-hex);
        text <=> root.band-hex;
    }
    VerticalLayout { alignment: end;
        Button { text: "Default"; enabled: root.band-hex != "";
                 clicked => { root.band-hex = ""; } }
    }
}
ColorPicker {
    kit-colors: root.kit-colors;
    text: root.band-hex;
    pick(hex) => { root.band-hex = hex; }
    /* the four colour callbacks forwarded */
}
HorizontalLayout { alignment: end;
    Button { text: "Close"; primary: true; clicked => { root.close(); } }
}
```

- **No swatch** (`has-swatch` stays `false`, its default — `:2628`). In `SetupField` the
  swatch *is* the way into a popup — lit on hover, tooltip *"Pick a colour"*
  (`:2658`–`:2679`) — and here there is no popup, so it would be a control that
  does nothing. The band and the row behind the scrim show the colour live.
- **Empty is the default**; the placeholder says *Default*, and the **Default**
  button is the way back, because clearing a field is not a gesture this coach
  would find.
- **No Cancel**: changes apply as made, as the pen and the panels do.
- At the `Scrim`: `band-hex <=> root.slate-band-hex`, `kit-colors`, the four
  colour callbacks, `close => { root.close-settings(); }` — the setup sheet's
  wiring shape (`:7400`–`:7417`).

**U3. The window: five properties, a callback and two functions.**

```slint
/// The Settings field's text (#131): a hex, half-typed text, or empty.
in-out property <string> slate-band-hex;
/// The last value of it that was a colour or empty — what is stored and drawn.
property <string> slate-band-good;
changed slate-band-hex => {
    if (root.slate-band-hex == "" || root.valid-hex(root.slate-band-hex)) {
        root.slate-band-good = root.slate-band-hex;
        root.slate-band-edited(root.slate-band-hex);
    }
}
/// What the band and the selected slate's row draw (§U4, §U8). **The one place
/// the default is written**, and why the band and row are one colour: the
/// comment now at `scrubber.slint:125`–`:133` moves here.
out property <color> slate-band-color: root.slate-band-good == ""
    ? Palette.selection-background : root.hex-color(root.slate-band-good);
in-out property <bool> settings-sheet-open;
/// Whether either door may open the sheet (§U5, §U9): one gate.
out property <bool> can-open-settings: !root.recording && !root.previewing;
/// A colour, or `""` for the default, to remember (§S7).
callback slate-band-edited(string);
/// Opens the sheet. **Deliberately no guard:** each door's `enabled` is the
/// gate, and `snap_menu.rs` relies on that — a check here as well would let a
/// broken item gate pass.
function open-settings() { keys.focus(); root.settings-sheet-open = true; }
/// Puts the field back on the last good value, so a typo left in it is not what
/// the sheet reopens on, then closes and gives the keys back.
function close-settings() {
    root.slate-band-hex = root.slate-band-good;
    root.settings-sheet-open = false;
    keys.focus();
}
```

Five properties: `slate-band-hex`, `slate-band-good`, `slate-band-color`,
`settings-sheet-open`, `can-open-settings`. A half-typed field leaves the band on
the last good colour. `valid-hex` and `hex-color` are the setup sheet's, wired in
`wire_callbacks` (`main.rs:1690`–`:1697`), and `wire_settings` runs after it
(`main.rs:427`). Rust's part is `wire_settings`: set `slate-band-hex` from
`state.slate_band_color().map(hex)` at startup — which fires `changed` once and
stores the same value back, harmlessly — and
`on_slate_band_edited(|t| state.set_slate_band_color(parse_hex(&t)))`.

**U4. The scrubber: `in property <color> band-color`**, bound to
`root.slate-band-color` beside `slate-span-from`/`-to` (`app.slint:6780`–`:6781`);
`:134` becomes `root.band-color.transparentize(0.25)`. The stale *"Teal: the three
things already on this track are…"* (`scrubber.slint:115`–`:117`), contradicted
ten lines below it since `edba6fd`, goes.

**U5. The button: `Settings…`, in the drawing row after `Keys`.** That row for
`Fit`'s and `Keys`' reason (`app.slint:7116`–`:7158`): the transport row already
runs to x=1199 at the 1100px floor, while here `Fit` ends at 268 (that comment's
2026-10-08 measurement) and the notice is the one stretch item, `min-width: 0`
(`:7267`–`:7276`). **An estimate** that it fits; the plan measures it as `Keys`
was. `enabled: root.can-open-settings` — as `Export…` and `Basket…` are gated
(`:6960`, `:6982`–`:6983`); `Keys` is ungated for a reason of its own
(`:7156`–`:7158`) that a colour does not share, and a modal over a take would hold
the keys that stop it. `clicked => { root.open-settings(); }`.

**U6. Esc: the basket's shape.** One branch after the Keys sheet's
(`app.slint:5007`–`:5012`); the Settings `Scrim` is drawn just before the Keys
sheet's (`:7525`), so the order mirrors the z-order:

```slint
if (root.settings-sheet-open) {
    if (pressed && event.text == Key.Escape && !root.text-editing) {
        root.close-settings();
        return accept;
    }
    return reject;
}
```

Esc in the field leaves it, the next closes — the basket's cascade
(`:5021`–`:5035`); `reject` delivers letters and Ctrl+V to the field and keeps the
shortcuts off. #78 §U4's "fold its own `editing`" no longer exists to do:
`text-editing` is Slint's own `text-input-focused` (`:4572`).

**U7. The next tenant is one row**, beside this one. No headings, no list model,
no registry.

**U8. The selected slate's row takes the same colour.** Today:
`background: slate.id == root.selected-slate ? Palette.selection-background :
transparent` (`app.slint:5692`–`:5694`), opaque, inside `AppWindow`. It becomes
`? root.slate-band-color : transparent` — no wire. The band and its row now read
one property rather than two literals that agree. The row stays opaque and the
band at 75%, so at the default both look exactly as today. Its text is
`Palette.foreground` / `alternate-foreground` (`:5836`–`:5856`), which a pale
chosen colour can wash out (Risk 1).

**U9. "Band colour…" on the scrubber's menu.** `snap-menu`
(`scrubber.slint:198`–`:209`) gains a plain item under *Snap to match events*:

```slint
MenuItem {
    title: "Band colour…";
    enabled: root.can-open-settings;
    activated => { root.open-settings(); }
}
```

- **It reaches the window as `toggle-snap` does:** `callback open-settings()` on
  `Scrubber` beside `toggle-snap` (`scrubber.slint:96`), forwarded beside
  `toggle-snap => { root.toggle-snap(); }` (`app.slint:6786`) to the window's
  function. No Rust.
- **Gated on the item, not the menu.** The menu has no `enabled` on purpose — it
  opens mid-take for the snap (`scrubber.slint:191`–`:197`; `snap_menu.rs:92`'s
  `the_menu_opens_while_recording`). So `Scrubber` gets
  `in property <bool> can-open-settings` — the window's name, one name for one
  gate — bound to `root.can-open-settings`. A disabled `MenuItem` greys out and
  is not activated (`i-slint-compiler-1.18.0/widgets/common/menu-base.slint:127`,
  `:184`–`:190`), which is the button's behaviour.

**No key opens the sheet** — Deferred.

---

## §E. Where the entries are out of date

To be corrected in the commit that carries this spec.

1. **#131** (`BACKLOG.md:4377`–`:4410`): its questions are answered (picker, band
   and row, the Settings sheet with a menu door) and its *"When to revisit"*
   (`:4405`–`:4410`) is superseded.
2. **#78** (`BACKLOG.md:1616`–`:1623`): *"The first tenant is #84"* — #84 built no
   sheet (`2026-10-12-reel-music-design.md` §M6; #84's entry `:1873`–`:1876`); it is
   **#131**. The "Next, in order" bullet (`:19`–`:25`) says *"still unbuilt"*.
3. **#78's spec §U4** (`2026-10-02-app-settings-design.md:317`–`:371`): superseded
   by this spec for the sheet's build — a note pointing here, in place of its
   2026-10-12 callout (`:320`–`:325`).
4. **#129** (`BACKLOG.md:4311`–`:4345`) waits *"with any settings surface"*; this
   sheet holds `state.json` values only and `pip_for_new_recordings` is the
   project's (`:4319`–`:4321`), so this is not its home.
5. **Sheet counts in prose, each re-derived:**
   - `app.slint:1935` — *"an edit to all six sheets"*: seven today, eight after.
   - `app.slint:2988`–`:2989` — *"the other five sheets, which *are* the
     `Sheet`"*: six today (export, basket, the editor, New match, Keys, the error
     dialog), seven after.
   - `CLAUDE.md:738` — *"an edit to all six"*: the same.
   - `tests/ui/sheet_scroll.rs:4`, `:7`, `:13`, `:33`, `:45` — *"six"*, and its
     `sheets()` (`:47`–`:107`) has no Keys sheet although `:14`–`:15` call it *"a
     tripwire for the seventh"*.

   Each is better **without** a number — the list is the count, and every one of
   these went stale the same way. `app.slint:1917` (*"There are seven of these"*)
   gains Settings in its list and loses the number; `:1927`'s *"two of the
   **seven**"* is a dated measurement and stays. `CLAUDE.md:691` (*"the seventh
   `Sheet"*) and `:845` (the export queue's *"no eighth `Sheet`"*) stay true.

---

## Crate responsibilities

| Crate | Change |
|---|---|
| `pundit-core` | **None.** |
| `pundit-media` | **None.** |
| `pundit-app` | `ui/app.slint`: `ColorPicker` takes the held position (§P2), loses its popup chrome to the setup sheet and labels its kit swatches (§P3–P4); `MatchSetupSheet` passes `text`/`pick` and holds the chrome (§P3); `SettingsSheet` (§U1–U2); the window's five properties, `changed` handler, callback and two functions (§U3); the `Settings…` button (§U5); the `handle-key` branch (§U6); the `Scrim`; the slate row's `background` (§U8); the counts (§E5). `ui/scrubber.slint`: `band-color`, the literal and the teal sentence (§U4); `can-open-settings`, `open-settings` and the item (§U9). `src/bus/state.rs`: the field and two accessors (§S2–S4). `src/main.rs`: `wire_settings`, after `wire_callbacks` (§U3). `tests/ui`: `settings_sheet` (new); `snap_menu`, `text_editing`, `sheet_scroll` grow. |
| `pundit-harness` | **None** — nothing crosses the bus. |
| Docs | `CLAUDE.md`: a Settings-sheet paragraph beside the Keys sheet's, the band in `state.json`'s list, and `:738`'s count (§E5). `BACKLOG.md` and #78's spec per §E. |

## Testing

1. **`src/bus/state.rs` unit tests** (`cargo test -p pundit-app --lib`), on
   `remembers_the_snap_and_defaults_it_on`'s shape (`:1096`): `None` with no file;
   a stored colour reads back after a reopen; `set(None)` reads `None`;
   `{"pen":"blue","slateBandColor":5}` and `…"slateBandColor":"teal"` both read
   `None` **and still read the pen** — the first `lenient`'s, the second the
   accessor's, so they fail under different sabotages. The field joins
   `the_settings_are_independent` (`:1004`).
2. **`tests/ui/settings_sheet.rs`, a new `mod` of `tests/ui`**, against the real
   `AppWindow`, finding controls by their words. **The fixture installs the
   pure-callback handlers first** — `valid-hex`, `hex-color`, `hsv-of-hex`,
   `hex-of-hsv` and `kit-colors`, a copy of `main.rs:1690`–`:1725`'s wiring over
   the library's `parse_hex` / `color_picker` — **before it reads a property or
   types anything**, since an unwired `valid-hex` answers `false` and every
   assertion below would pass or fail on that. It also records
   `slate-band-edited`'s arguments.
   - **Settings…** opens the sheet; Esc and **Close** each close it.
   - The button reports `accessible_enabled() == Some(false)` with
     `recording_phase` set, then with `previewing_clip` set (every style's
     `Button` binds `accessible-enabled: root.enabled`;
     `i-slint-compiler-1.18.0/widgets/fluent/button.slint:30` is this build's,
     having no Qt backend, root `Cargo.toml:20`).
   - **A typed colour reaches the band and is reported:** type `#ff0000` →
     `get_slate_band_color()` is red and `slate-band-edited` got `"#ff0000"`.
   - **A typo is never reported and does not survive the sheet:** then type
     `#12` → nothing more reported, the colour still red; Esc, Esc, reopen → the
     field reads `#ff0000`.
   - **Default:** → the colour equals the one read before anything was typed,
     `slate-band-edited` got `""`, and Default is disabled.
   - **A kit swatch lands in the field:** press *White* by its label
     (`invoke_accessible_default_action`) → the field reads `"#ffffff"`
     (`KIT_COLORS[0]`, `color_picker.rs:39`–`:42`).
   - **The setup sheet's picker still lands in its field** — one test, because
     §P2 moves its held position out of `MatchSetupSheet`: open the setup sheet,
     click the home primary's swatch (`SetupField::touch`), press *White* →
     `match_home_primary == "#ffffff"`. No test opens that picker today
     (`match_picker_field`, `SetupField::touch` and `kit_colors` appear in no file
     under `tests/`); written first and run either side of the move.
3. **`tests/ui/snap_menu.rs`**, with its fixture, `open` and `find`
   (`:19`–`:52`):
   - Clicking *Band colour…* opens the sheet.
   - With `recording_phase` set, then `previewing_clip`, the menu still opens but
     clicking *Band colour…* leaves the sheet shut. That pins the item's
     `enabled` alone, because `open-settings()` has no guard (§U3).
   - The header (`:1`–`:9`) widens from the snap to the menu.
4. **`tests/ui/text_editing.rs`**: the settings field joins
   `a_letter_typed_in_a_sheet_field_reaches_the_field` (`:252`); the header's
   field count (`:19`–`:20`) is re-derived — the site count does not move,
   `SetupField::edit` already being one.
5. **`tests/ui/sheet_scroll.rs`**: `SettingsSheet` and `KeysSheet` join `sheets()`
   and the header loses its counts (§E5).
6. **Not automated, and said:** that the band and the row actually take the
   colour. Both are a `Rectangle.background` with no accessible property, so a
   dead `band-color: root.slate-band-color` wire, or a row still reading the
   palette, passes everything above; the window's `slate-band-color` is pinned and
   the last hop to each leaf is the manual list's, with `Settings…`'s width
   (§U5).

## Risks

1. **A pale chosen colour can wash out the selected row's text** (§U8): the row
   is opaque and its words light on the dark scheme. Not engineered against — the
   coach picks by eye with the Slates list beside the sheet — and the cure if it
   bites is a known shape, `highlight::label_ink`'s black-or-white choice
   (`pundit-core/src/highlight.rs:406`) applied to the row's text.
2. **The picker shows black when the band is the default.** `""` is not a hex, so
   `sync-from-text` leaves the held position at its start. Cosmetic: the first
   kit press or drag puts it right, and seeding it would need a colour-to-hex call
   Slint does not have.
3. **Writes while dragging.** One small document rewrite per pointer move in the
   area (§S7), each a temp file and a rename on the UI thread; and every write
   widens, by a little, the existing lost-update window between the UI's and the
   bus's `AppFiles` handles (`bus/state.rs:173`–`:177`'s note). If either is ever
   seen, the area's pointer-up is where a single write goes.
4. **The extraction touches a shipped sheet**; Testing 2's last item is written
   first and run on both sides of the move.

**Moot with the popup gone:** both reviews' findings about a popup covering the
card's title and a live anchor binding moving the popup — there is no popup in
this sheet, and the setup sheet's backdrop, clamp and anchor are not changed.

## Deferred

- **A key for the sheet** — an unbound `ShowSettings` action is possible since
  #96 (`Action::ShowRecents` ships unbound, `keymap.rs:307`), so #78 §U4's
  "every letter is a global binding" reason is out of date; not built until a
  coach asks.
- **Cancel / "the colour I had before"**, if Default turns out not to be the way
  back he wanted.
- **The row's text following the colour's lightness** (Risk 1), when seen.

## Open questions for the coach

**None.** The first draft's two — whether the row takes the colour, and whether
the menu gets a door — were answered on 2026-10-13 (**row too; yes, both**) and
are §U8 and §U9.

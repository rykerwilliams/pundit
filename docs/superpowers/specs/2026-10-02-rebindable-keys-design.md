# Rebindable keys: one table, in one place, that the coach can read

**BACKLOG #96.** The coach (2026-09-25): *"we need to have all the hot keys
reassignable"*.

Today every key is an inline `event.text ==` comparison in one Slint function.
Nothing is data, so nothing can be changed without a rebuild and **nothing can
be listed** — which is half of why **#94** exists and the whole of why **#99**
and **#85**'s keyboard half are parked.

**#96's own framing is the right one and this spec keeps it:** the work is not a
settings UI, it is turning the branches into a table. The table is worth having
even if nothing is ever rebound, because it makes "what can I press here?"
answerable, and it stops the next feature inventing a key that is already taken
— a check #92 and #117 both had to do by grep.

**Three of #96's premises were checked against the source and two are wrong.**
Its branch count is stale (**A1**), and its "a coach must not be able to bind a
bare letter in a way that breaks typing" worry is already structurally
impossible (**G4**). Its Slint claim — the one that decides what "rebindable"
can even mean — is **right, and stronger than it says** (**K1**). Separately,
Slint 1.18 ships two things #96 predates and this spec had to weigh: a
first-class `keys` type and a `KeyBinding` element (**K3**, **B4**).

## Goal

Every key the app binds is one row in one table: its action, its default, what
it does and when it works. The coach can read that table in the app, and change
it without a rebuild.

## Scope

In: an `Action` enum and a `Binding` type in `pundit-app`, the defaults, the
`state.json` round-trip, the one lookup `handle-key` consults, and a read-only
list the coach can open.

Also in, because this change invalidates them: **`CLAUDE.md`'s transport-keys
paragraph** and the four other places it spells a key inline (the `J`/`L`, `F`,
`i`/`o` and match-tag paragraphs), which stop being the record of what is bound
the moment the table exists — they should point at the table and keep only their
*reasoning*. And **BACKLOG #35, #94, #96, #99**, each of which this spec either
resolves or re-premises.

Out: everything in **X**. In particular, **no rebind UI** — #96 itself says it
"should ship with a read-only view of the bindings first — that alone closes #94
and is most of the value", and this spec agrees (**D3**).

---

## A. The ground truth: every key the app binds today

Established by reading `crates/pundit-app/ui/app.slint`, which is **the only
place in the repo that handles a key**: `crates/pundit-app/src/` contains no
`KeyEvent`, no `key_pressed` and no `Key::` at all, and `ui/scrubber.slint` and
`ui/splitter.slint` contain no key handling either. Everything below is in
`app.slint`.

**Every line number in §A is against `4f418ae`** (the merge of #116), which is
what this spec was written on. **#113's folding sections are in flight on
`claude/folding-sections` and shift them**: at `4173ede`, `handle-key` is at
`:4182` rather than `:4092` and `text-editing` at `:3869` rather than `:3783` —
**+90 and +86**, and nothing in this section changes but the numbers. Stated
rather than left for a reader to discover, because the whole value of §A is that
every row can be checked, and a row that cannot be found reads as a row that is
wrong. **The plan should re-derive them against whatever is merged by then** and
is the only artefact that should carry them at all; this spec's own table is a
census, not a map.

**A1. The count.** #96 says "33 `event.text ==` branches". There are now
**61** such comparisons inside `handle-key` (`:4092`–`:4373`), 64 in the file,
across **29** `if` statements — six of which are the modal sheets' Esc guards
rather than bindings. The drift is #92's `i`/`o`, #95's `F` and the chapter
keys arriving after #96 was filed. **The number is not load-bearing; that it
grew by itself in five weeks is.**

### A2. The modal layers, in the order `handle-key` tests them

Each returns early. `accept` swallows the key; `reject` hands it to whatever
child has focus.

| # | Layer | Key | Effect | Line |
|---|---|---|---|---|
| 1 | `error-message != ""` | Return **or** Escape | clears the dialog | `:4095` |
| 1 | " | anything else | swallowed | `:4098` |
| 2 | `export-sheet-open` | Escape | closes the sheet | `:4103` |
| 2 | " | anything else | swallowed | `:4106` |
| 3 | `basket-sheet-open` | Escape, `!text-editing` | closes it | `:4115` |
| 3 | " | anything else | **`reject`** — delivers Ctrl+V to its field | `:4119` |
| 4 | `new-match-sheet-open` | Escape, `!text-editing` | closes it | `:4131` |
| 4 | " | anything else | `reject` | `:4135` |
| 5 | `match-sheet-open` | Escape, `match-picker-field >= 0` | closes the colour picker only | `:4146`, `:4147` |
| 5 | " | Escape otherwise | closes the sheet | `:4150` |
| 5 | " | anything else | `reject` | `:4154` |
| 6 | `match-editor-open` | Escape, `match-editor-line-focused` | re-seeds the row's line, returns focus to `keys` | `:4171` |
| 6 | " | Escape, `!text-editing` | closes it | `:4176` |
| 6 | " | anything else | `reject` | `:4181` |
| 7 | `text-editing` | **everything** | `reject` | `:4185` |

`text-editing` (`:3783`) is the disjunction of seven terms: the project-name
field, the clip inspector, the highlights panel, the match editor, the basket
sheet, a slate's fields and the New match sheet.

### A3. The window bindings

All of these are reached only past **A2**, so `!text-editing` holds throughout.
"Repeat" says whether a held key fires again: `!event.repeat` means it does not.

| Key(s) | Mods | Action | Guard | Repeat | Line |
|---|---|---|---|---|---|
| Home, End | — | **nothing** — swallowed so a touched slider cannot take them | — | n/a | `:4189` |
| O | Ctrl | `open-project()` | `!recording` | yes | `:4193` |
| 0 | Ctrl | `zoom-reset()` | `!previewing` | yes | `:4201` |
| Z | Ctrl | `undo()` | `!recording` | yes | `:4209` |
| Z | Ctrl+Shift | `redo()` | `!recording` | yes | `:4211` |
| Y | Ctrl (Shift ignored) | `redo()` | `!recording` | yes | `:4211` |
| *any other* | Ctrl | **`reject`** — this is what delivers every other Ctrl combo to a focused child | — | — | `:4219` |
| C | — | `clear-drawings()` | `can-draw` | no | `:4223` |
| I | — | `mark-in()` | `can-tag` | no | `:4237` |
| O | — | `mark-out()` | `can-tag` | no | `:4243` |
| Z | — | tag home goal | `can-tag` | no | `:4249` → `:4080` |
| X | — | tag away goal | `can-tag` | no | `:4252` |
| V | — | tag period start/stop | `can-tag && !match-at-cap` | no | `:4255` |
| H | — | toggles `highlight-tool` | `can-tag` | no | `:4262` |
| R | — | `toggle-recording()` | — | no | `:4269` |
| Escape | — | cascade: clear the highlight selection → leave the H tool → stop recording → close the preview → clear the clip selection | — | yes | `:4281` |
| Delete | — | `delete-clip(selected-clip)` | `!recording && selected-clip != ""` | yes | `:4299` |
| Space | — | `toggle-play()` | — | no | `:4305` |
| ←, A | — | `skip(-3)` | — | yes | `:4311` |
| ←, A | Shift | `skip(-10)` | — | yes | `:4311` |
| →, D | — | `skip(+3)` | — | yes | `:4317` |
| →, D | Shift | `skip(+10)` | — | yes | `:4317` |
| , | — | `step-frame(false)` | `!recording && !previewing && can-play` | yes | `:4325` |
| . | — | `step-frame(true)` | " | yes | `:4325` |
| [ | — | `jump-chapter(false)` | `!recording && !previewing && can-play` | yes | `:4334` |
| ] | — | `jump-chapter(true)` | " | yes | `:4334` |
| J | — | `step-scan-speed(slower)` | `can-scan-fast` | no | `:4342` |
| L | — | `step-scan-speed(faster)` | `can-scan-fast` | no | `:4342` |
| F | — | `fit-window()` | `can-fit` | no | `:4354` |
| 1 | — | `zoom-reset()` | `!previewing` | yes | `:4360` |
| 2 | — | `zoom-step(-0.25, …)` | `!previewing` | yes | `:4366` |
| 3 | — | `zoom-step(+0.25, …)` | `!previewing` | yes | `:4366` |
| *anything else* | — | `reject` | — | — | `:4372` |

**Every bare letter above also fires shifted**, because each test is
`event.text == "r" || event.text == "R"`. That is not a decision anyone took; it
is what writing both spellings gets you. **G2** decides what to do with it.

The guards, all on `AppWindow`:

| Guard | Definition | Line |
|---|---|---|
| `recording` | `recording-phase != idle` | `:3396` |
| `can-draw` | `recording-phase == recording` (narrower than `recording`) | `:3514` |
| `can-tag` | `can-play && !previewing && recording-phase != starting` | `:3708` |
| `can-play` | sources, none missing (from Rust) | `:3373` |
| `can-scan-fast` | `playing && !recording && !previewing && can-play` | `:3779` |
| `can-fit` | from Rust; `can-play` folded in | `:3345` |
| `previewing` | `previewing-clip != ""` | `:3680` |
| `in-highlight-tool` | `highlight-tool && can-play && !previewing` | `:3551` |
| `match-at-cap` | from Rust | `:3704` |

### A4. The three keys handled outside `handle-key`

| Where | Key | Effect | Line |
|---|---|---|---|
| `keys := FocusScope`'s own `key-pressed` | Escape while `text-editing` | `keys.focus()` — leaves the field, which commits it. The only keyboard way out of the notes field. | `:4385` |
| `ClipInspector`'s tags field | Tab while `suggesting` | takes the top suggestion | `:804` |
| " | Escape while `suggesting` | dismisses the suggestions | `:808` |

### A5. The keys Slint handles, which the app never sees

Verified in `i-slint-core-1.18.0/window.rs::process_key_input` (`:1248`). The
order is: **MenuBar accelerators** (`:1299`), then the **capture phase**
window→focus-item (`:1324`–`:1326`), then the **bubble phase** focus-item→window
(`:1336`), and only then:

| Key | What Slint does | Line |
|---|---|---|
| Tab | `focus_next_item()` | `:1359` |
| Shift+Tab, Backtab | `focus_previous_item()` | `:1368` |
| Escape | closes the top-most popup, if its policy is `CloseOnClick` or `CloseOnClickOutside` | `:1386`–`:1394` |

So while `recent-popup` or `devices-popup` is up, `show_popup` has taken the
focus item, `handle-key` never runs, and Escape dismisses the popover through
Slint — which is the recents spec's **P4**, confirmed here against the source
rather than inferred.

Inside a focused `LineEdit`, `InternalKeyEvent::shortcut` (`input.rs:1127`) and
`text_shortcut` hard-code Ctrl+C/X/V/A/F/S/P/Z/R and the cursor keys. They are
reachable only because **A2**'s layer 7 and layer 3–6 `reject`.

---

## K. What a key event actually carries — #35, checked

**K1. #35 is right, and the limitation is deeper than it records.** #35 says
*"Slint key events carry text, not scancodes"*. Verified three ways in Slint
1.18, which is what this workspace pins (`Cargo.toml:20`):

1. `KeyEvent` is a builtin struct with **exactly three fields** — `text:
   SharedString`, `modifiers: KeyboardModifiers`, `repeat: bool`
   (`i-slint-common-1.18.0/builtin_structs.rs:104`–`:111`). There is no
   scancode, no key code and no physical-key field, in the public struct or
   anywhere near it.
2. `InternalKeyEvent`, the runtime's own richer type, adds composition state and
   a Windows-only `text_without_modifiers` and **no physical key**
   (`i-slint-core-1.18.0/input.rs:1101`–`:1125`).
3. The winit backend reads **`event.logical_key` only**
   (`i-slint-backend-winit-1.18.0/winitwindowadapter.rs:1337`). `physical_key`
   is never mentioned in that crate. So the information is discarded at the
   boundary: no amount of plumbing on our side could recover it.

**Two consequences #35 does not record, both load-bearing:**

- **A key that produces no text never reaches the app at all.**
  `if text.is_empty() { return Ok(()) }` (`winitwindowadapter.rs:1407`). A dead
  key is not "received as nothing"; it is not received.
- **The text is the *shifted* reading.** `Key::Character(str)` is passed through
  as-is (`winitwindowadapter.rs:1366`), and winit's `logical_key` has the layout and the modifiers
  applied — which is why `app.slint` has to write `"r" || "R"` at all. That is
  the mechanism behind #35's AZERTY complaint, stated precisely.

**K2. So #35 cannot be fixed, and does not need to be: rebinding retires it.**
A physical binding is impossible without a change in Slint. But #35's two
symptoms are the two a keymap answers directly — a French coach binds the zoom
to the keys where their digits are, and `A`/`D` to `Q`/`D`, once, and
`state.json` remembers. **#35 should be closed as retired by this feature, not
left waiting for a Slint that may never expose scancodes.**

**K3. `slint::Keys` exists, is public, and is still not the representation —
because `matches` is not.** Slint 1.18 ships a `keys` primitive type with a
`@keys(…)` macro, a `Keys::from_parts(["Control", "Shift?", "Z"])` constructor,
`to_parts()`, and a platform-native `Display` (`input.rs:835`–`:969`, with `Display` at `:971`;
re-exported as `slint::Keys` at `slint-1.18.0/lib.rs:224`). It even solves the
AZERTY *digit* case properly: the digit keys carry
`ShiftBehavior::LocalizedShiftable` (`key_codes.rs:160`–`:169`), so a binding
written `Digit2` matches the key that *produces* `2` whether or not Shift is
needed to get there.

**It is rejected, on one fact: `Keys::matches` is `pub(crate)`**
(`input.rs:914`). Only `KeyBinding` and the MenuBar can use it. Adopting `Keys`
would mean writing our own matcher *anyway* and keeping a second, authoritative
spelling of each binding beside it — two values holding one truth, free to
disagree, which is the mistake this repo has now removed twice (`Command::NewMatch`'s
name field; `lastProject` beside `recentProjects`). Two further costs, for the
record: `to_parts` emits the **raw character** for a named key, which for
Escape or an arrow is an unprintable private-use code point that a text format
has to escape (its own doc says so), and the letters are `Unshiftable`
(`key_codes.rs:133`–`:158`), so `@keys(R)` would *stop* matching Shift+R, which
is a behaviour change we have to decide anyway (**G2**) and do not want decided
for us by a table in another crate.

**Revisit if Slint makes `matches` public.** Then `Keys` is strictly better than
**B1** and the migration is a drop-in: one type, its parser, its display and its
matcher, maintained upstream.

---

## B. The shape

**B1. A binding is `(key, modifiers)`, stored and shown as one string, in a new
`pundit-app/src/keymap.rs`.**

```rust
pub struct Binding { key: Key, ctrl: bool, shift: bool, alt: bool }
pub enum Key { Char(char), Named(NamedKey) }   // 'r', ',', '1' | Escape, LeftArrow, Delete, F1, Space
```

Its spelling is its label, on `Pen`'s and `WhisperModel`'s pattern
(`drawing.rs:71`/`:78`, `:104`/`:115`): `"r"`, `"ctrl+z"`, `"ctrl+shift+z"`,
`"shift+LeftArrow"`, `","`, `"Delete"`, `"F1"`. `Binding::label()` and
`Binding::from_label()` round-trip, and an unparseable label reads as **none**
rather than throwing anything away — the rule every stored label in this app
already follows.

**The named keys are a closed list of the ones the app binds or could sensibly
bind** — the arrows, Escape, Return, Tab, Delete, Backspace, Home, End, PageUp,
PageDown, Space, F1–F12 — not a copy of Slint's 90-entry table. Each carries
the `char` Slint actually delivers, taken from `key_codes.rs` (Escape
`\u{001b}`, LeftArrow `\u{F702}`, Delete `\u{007f}`, Space `\u{0020}`, F1
`\u{F704}`). **A test asserts each one against `app.slint`'s own `Key.*`
constant through the harness's key injection**, so the list cannot be wrong in
silence (**T2**).

**B2. The matcher lowercases the text and compares the three modifiers
exactly.** This is Slint's own rule (`input.rs:929`–`:938`) and it is what makes
Caps Lock free: under Caps Lock the backend reports uppercase text with
`shift: false`, so `"r"` still matches.

**B3. The lookup is one Rust callback, called once per key event.**

```slint
pure callback action-for(string, bool, bool, bool) -> KeyAction;   // text, ctrl, shift, alt
```

`handle-key` calls it once, past layer 7, and its chain becomes `if (act ==
KeyAction.record) { … }`. **Every guard, every `accept`/`reject` and the whole
Esc cascade stay exactly where they are** — this spec moves the *mapping* and
nothing else.

The argument is `(string, bool, bool, bool)` rather than the `KeyEvent` itself
because `KeyEvent` is not in `slint`'s public API: it is reachable only through
`slint::private_unstable_api::re_exports` (`private_unstable_api.rs:180`). Four
scalars are also the honest signature — `repeat` is read by the *guards*, which
stay in Slint, and the keymap has no business seeing it.

`KeyAction` is an `export enum` in `app.slint` with a `none` variant, mirrored
by `keymap::Action` in Rust, and `main.rs` maps between them — the established
pattern for `MatchTag`/`MatchEventKind` (`main.rs:1275`) and
`ScanStep`/`bus::ScanStep` (`:724`). That map is a `match` on the Rust enum, so
**adding an action without wiring it fails to compile**; the one hole is the
Slint `if` chain, which is not exhaustive (**Risk 1**).

**B4. `KeyBinding` elements are rejected, and the reasons are specific.** Slint
1.18 has a built-in `KeyBinding { keys; enabled; activated() }` element
(`builtin_elements.rs:1377`–`:1392`) whose entire purpose is this feature, with
an upstream `runtime_key_bindings` example for persisting one. It is the obvious
answer and it does not work here:

1. **Wrong phase.** `KeyBinding`s are consulted in `FocusScope::key_event`
   (`items/input_items.rs:723`), the **bubble** phase — after every focused
   child has had the key. This app's shortcuts are deliberately on
   `capture-key-pressed` (`:4376`), whose stated reason (spec D10, the comment
   at `:4088`) is *"every shortcut, ahead of whichever child has focus, so a
   touched slider never turns arrows into volume or scrub steps."* A `Slider`
   accepts the arrow keys, so under `KeyBinding` the skip keys would die the
   first time the coach touched the volume.
2. **`activated()` carries no event, so `repeat` is gone.** Ten of the bindings
   in **A3** are `!event.repeat` — `R`, Space, `C`, `I`, `O`, `Z`, `X`, `V`,
   `H`, `J`/`L`, `F` — each for a reason written beside it (a held `R` would
   toggle recording per repeat; a held `V` would insert a period per repeat).
   There is no way to express that in a `KeyBinding`.
3. **A letter binding cannot ignore Shift.** The letters are `Unshiftable`
   (**K3**), so every one of them would change behaviour.

**Revisit if Slint adds a capture-phase binding or an event argument to
`activated`.** Points 1 and 2 are the whole objection; the design in **B1**–**B3**
is deliberately shaped so that swapping in `KeyBinding` later replaces
`keymap`'s matcher and leaves the action table standing.

**B5. `handle-key`'s branch order stops being load-bearing, which is the real
simplification.** Today `c`, `i`, `o` and `z` must come *after* the Ctrl branch
or Ctrl+C would clear the drawings and Ctrl+O would mark an out point — two
comments in `app.slint` say so (`:4221`, `:4231`), and `i`/`o` inherit the
position without one of their own. Once the modifier is
part of the binding, `ctrl+o` and `o` are different table entries and the
ordering constraint is gone. The chain's order becomes cosmetic.

---

## G. The rules that have to be decided

**G1. Two bindings per action, and "unbound" is a real state.** `←` and `A` both
skip back; `1` and `Ctrl+0` both reset the zoom. So the map is action →
`Vec<Binding>`, and an **empty** vector is how a coach frees a key. That is not
a degenerate case to tolerate; it is the simplest answer to the conflict rule
(**G3**) and it is what lets an action ship with no default at all (**D4**).

**G2. Shift is part of the binding, and the far skip becomes its own action.**

Today Shift is two different things at once: part of the binding (Ctrl+Shift+Z),
an argument to the action (`skip(±10)` instead of `±3`), and silently ignored
(`"r" || "R"`). Three readings of one modifier is what makes a conflict rule
impossible to state.

**One reading: Shift is part of the binding.** Consequences:

- The **far skip gets its own four bindings** — `shift+LeftArrow`, `shift+a`,
  `shift+RightArrow`, `shift+d` on two new actions. This is strictly better
  than today: the 20-second Shift-tap becomes rebindable and *listable*, and the
  "Shift is an argument" special case disappears. The Slint branch calls
  `root.skip(-10)` where it called `root.skip(event.modifiers.shift ? -10 : -3)`.
- **Shift+letter stops firing the bare letter's action.** Shift+R no longer
  records. Nobody presses it on purpose, Caps Lock is unaffected (**B2**), and
  the one genuinely lost shape is Caps-Lock-on-plus-Shift+R, which is a
  lowercase `r` with `shift: true`. **Accepted** — the alternative is a
  three-valued Shift and a most-specific-wins resolution rule, which is real
  machinery bought for a key combination no coach presses.

**G3. A binding belongs to one action. The load rule drops the loser; the (future)
rebind rule unbinds it.**

- **On load**, the stored map is applied in `Action::ALL` order. A binding
  already taken is **dropped for the later action**, logged by name, and that
  action keeps whichever of its own bindings survive — possibly none. The
  surviving state is always consistent, and because "unbound" is legal
  (**G1**) there is no case to invent a recovery for.
- **On rebind**, when a rebind UI is ever built (**D3**): **last wins, and the
  action it was taken from loses that binding**, which the list then shows as
  `—`. Refusing is worse (the coach has to go find and clear the other action
  first, with no help finding it), and silent last-wins is worse (a key vanishes
  from another action with no word). Visible last-wins needs the list, which is
  why the list ships first and the UI second.
- **The defaults are conflict-free, and a test says so** (**T1**). That is the
  one property the whole rule rests on.

**G4. What is NOT rebindable, and why.** Four groups, and the first is the only
one that was ever in doubt.

1. **Escape, in all seven of its jobs.** It closes the error dialog, each of the
   five sheets (two of them through a cascade), leaves a text field (`:4385`),
   and runs the five-step cascade at `:4281` — and Slint itself uses it to
   dismiss a popover, *after* both dispatch phases, where nothing we write can
   reach (**A5**). Escape is not an action with a key; it is a key with a
   position: "back out of the innermost thing on screen". Rebinding it would let
   a coach make a sheet unleavable. **Reserved, and the cascade stays the
   hand-written chain it is.**
2. **Tab, Shift+Tab and Return.** Tab is window-level focus traversal
   (`window.rs:1359`, `:1368`) and the tags field's completion (`app.slint:804`); Return
   confirms the error dialog. Both are platform conventions, not app actions.
3. **Home and End**, which are *swallowed* and bound to nothing (`:4189`). They
   are in the table only as a row that says so.
4. **Everything a focused `LineEdit` does** — Ctrl+C/X/V/A, the cursor keys —
   which is hard-coded in `i-slint-core` (`input.rs:1127`) and arrives only
   because layer 7 rejects.

**And the worry #96 raises is already impossible.** It asks that "a coach must
not be able to bind a bare letter in a way that breaks typing in a sheet". The
`text-editing` reject (`:4185`) is **ahead of every binding** and is the
disjunction of all seven field groups (`:3783`), so no table entry, however
bare, is ever consulted while a field has focus. The design preserves this
structurally: `action-for` is called *after* that line and the spec forbids
moving it. Nothing to build.

---

## S. Storage

**S1. One new `state.json` key, `keys`: action name → list of spellings.**

```json
"keys": { "tagHomeGoal": ["z"], "skipBackFar": ["shift+LeftArrow", "shift+a"], "scanFaster": [] }
```

`state.json` is where #96 says to put it and the reasoning holds: a binding is a
property of the coach's hands, not of a match, so it must never be a
`project.json` field — **`CURRENT_FORMAT_VERSION` stays at 13**
(`store.rs:21`).

**Four failure shapes, three of which cost nothing:**

- **An action name this build does not know is ignored** — the whole reason it
  is a map keyed by name rather than a positional list, and the rule
  `whisper_model` and `pen` already follow (`bus/state.rs:86`, `:89`).
- **A spelling that does not parse is dropped**, logged, and its siblings
  survive. `from_label` returning `None` is the mechanism.
- **An action present with `[]` is unbound**, deliberately (**G1**).
- **An action absent keeps its default.** Which means `"keys"` deleted by hand
  is the reset path for as long as there is no button for it (**D3**).

**The one shape that costs the whole keymap is a JSON type error** —
`"keys": "x"`, or a map whose value is a number. `HashMap<String, Vec<String>>`
fails whole, so `lenient` (`bus/state.rs:121`) hands back an empty map and every
binding reverts to its default. **Accepted and stated, not fixed.** It is the
bargain `basket.rs` strikes for its `pieces` and the recents list strikes for
its paths, struck for the reason this file's header gives: what is lost is a
re-pick, never a project. Reading the map as `HashMap<String, Value>` to rescue
the good rows is machinery for a hand-edit.

**S2. One attribute, which is what #100 bought.** `#[serde(deserialize_with =
"lenient")]` and nothing else — `default` is on the container
(`bus/state.rs:53`). #100 landed on 2026-10-02 for exactly this trigger
("next time anything is added to `state.json`"), so this field is the first to
be added under the per-field read and is the first that costs nothing but
itself.

**What #100 does *not* buy, so this spec does not claim it:** lost updates are
untouched. Every setter is read-then-save over the whole document and there are
two `AppFiles` handles, the bus's and `main.rs`'s `machine_state` (`:380`). A
bus-side write interleaving with a keymap write loses one field. The write is a
temp file and a rename, so nothing tears.

**S3. Two accessors, on `set_pen`'s shape.** `AppFiles::keymap() -> Keymap`
applies the stored overrides over the defaults through **G3**'s load rule, and
`AppFiles::set_binding(action, Vec<Binding>)` writes one action's row. The
keymap is read **once, at startup**, into the `action-for` closure — not per key
event, which would be a file read per keypress. A rebind UI would have to
rebuild that closure; the read-only ship does not.

---

## D. The defaults, and how the coach finds them

**D1. The default table is today's bindings, unchanged, plus three rows.**
**29 actions, 34 bindings** — counted, not estimated.

| Action | Default | Action | Default |
|---|---|---|---|
| `openProject` | `ctrl+o` | `skipForward` | `RightArrow`, `d` |
| `undo` | `ctrl+z` | `skipBackFar` | `shift+LeftArrow`, `shift+a` |
| `redo` | `ctrl+shift+z`, `ctrl+y` | `skipForwardFar` | `shift+RightArrow`, `shift+d` |
| `clearDrawings` | `c` | `stepBack` | `,` |
| `markIn` | `i` | `stepForward` | `.` |
| `markOut` | `o` | `prevEvent` | `[` |
| `tagHomeGoal` | `z` | `nextEvent` | `]` |
| `tagAwayGoal` | `x` | `scanSlower` | `j` |
| `tagPeriod` | `v` | `scanFaster` | `l` |
| `highlightTool` | `h` | `fitWindow` | `f` |
| `toggleRecording` | `r` | `zoomReset` | `1`, `ctrl+0` |
| `deleteClip` | `Delete` | `zoomOut` | `2` |
| `togglePlay` | `Space` | `zoomIn` | `3` |
| `skipBack` | `LeftArrow`, `a` | `showKeys` | `F1` |
| | | `showRecents` | *(none)* |

**Four of the 29 are new**: the two far skips (**G2**), `showKeys` (**D2**) and
`showRecents` (**D4**). The other 25 are **A3**'s rows with `skip`'s Shift
argument lifted out, and nothing else moves.

**D2. The list is a sheet, opened by `F1`, and its rows are generated from the
same table the matcher uses.**

`Keymap::listing()` returns `(action label, bindings, one sentence)` for every
action in `Action::ALL`, so the list **cannot go stale**: a row that is listed
is a row the matcher holds. That is the whole of #94's remaining half, and #96
is right that it is most of the value.

- **A sheet**, which is the app's one established modal shape (five of them
  today, plus the error dialog), not a new window and not a panel section. It
  dims the window, Esc closes it through layer 2's pattern, and it needs no
  `editing` fold because it has no fields — the recents popover's **P4**
  reasoning, in the other direction.
- **`F1`**, which is layout-independent (a named key, **B1**), universally means
  help, and is free. It is also the one binding whose job is to prove the table
  works, so it is in the table like everything else.
- **Opened from a button too** — `Keys` beside `Recent ▾` in the transport row.
  **The row's budget at the window minimum is not measured** and is on the
  manual list (the recents spec made the same admission about the same row).
- **Three columns: what it does, the keys, and when.** The "when" is a sentence
  on the action, in prose — *"Tag a home goal — while the game video is on
  screen"* — **not** a second machine-readable copy of the Slint guard. A
  machine-readable guard would be a second source of truth for `can-tag` with
  nothing keeping the two in step, which is the hazard `CLAUDE.md` names for
  `Preferences`. One string, one reader.

**D3. No rebind UI in this spec**, on #96's own recommendation. What ships is
the table, the storage, the list, and a hand-editable `state.json`. **G3**'s
rebind rule is specified so that the UI, when it comes, has nothing left to
decide; **Deferred 1** carries it.

**D4. `showRecents` ships as an action with no default**, which closes #85's
deliberate deferral at the cost of one row. **#85 shipped its popover with no
keyboard path on the explicit grounds that #96 owns the question** (its spec's
**P3**, and #32's entry says the same). This is the answer: one table row, and
the coach picks the key — **Open question 1**.

**It unblocks *opening* the popover, not navigating it**, and the distinction is
structural, not an omission: `show_popup` takes the window's focus item, so
`handle-key` does not run while a popover is up and no binding in this table can
reach inside one (**A5**). Arrow-key navigation of the rows is a `FocusScope`
inside the popover and a separate piece of work (**Deferred 4**).

---

## N. What this unblocks for #99, and the premise it corrects

**#99's stated blocker is not the real one.** It says: *"`AppWindow` has
`forward-focus: keys` and every letter is a global binding, so a focusable grip
would swallow `z` / `x` / `v` while it held focus."*

Checked: it would not. The global letters are on **`capture-key-pressed`**
(`:4376`), dispatched window→focus-item **before** the focused item
(`window.rs:1324`–`:1326`). A focused splitter grip would never see `z`, because
`handle-key` accepts it first.

**The real blocker is the opposite problem, and this feature is what fixes it:**
the grip needs the **arrow keys**, and `handle-key` accepts those too
(`:4311`, `:4317`) for the skips. A grip can never be reached by arrow key as
long as a second dispatch path is the only way to give it one. With the table,
`resizeSidebar` / `resizeInspector` are **actions with a guard** — "a grip has
focus" — resolved in the same lookup as everything else, in one place, with the
precedence visible. That is what #96 owes #99, and #99's entry should be
re-premised.

---

## X. What this does not do

- **No format version.** No field on any stored struct;
  `CURRENT_FORMAT_VERSION` stays 13 (`store.rs:21`).
- **No new command, no bus change, no `pundit-core` change.** The keymap is a UI
  concern end to end, which is why there is no harness test (**T4**).
- **No rebind UI** (**D3**), no "restore defaults" button (**Deferred 2**).
- **No chords or sequences.** One `(key, modifiers)` pair per binding. Nothing
  in the app wants two-key sequences and they would need a timeout, a pending
  state and a way to show both.
- **It does not make Escape, Tab or Return rebindable** (**G4**).
- **It does not open the sheets by key.** Export…, Basket…, Match setup and the
  event editor have no keys today and inventing four is a feature, not this one.
- **It does not add a menu bar.** #32's remaining half, and **Risk 3** says what
  this spec found out about it.
- **It does not chase physical keys** (**K2**).

---

## Crate responsibilities

| Crate | Contents |
|---|---|
| `pundit-core` | **Nothing.** A keymap is neither a project nor a timeline. |
| `pundit-media` | **Nothing.** |
| `pundit-app` | New `pub mod keymap;`: `Action` (with `ALL`, `label`, `from_label`, the listing sentence), `Binding`/`Key`/`NamedKey` with `label`/`from_label`, `Keymap::{defaults, with_overrides, action_for, bindings_for, listing}` — including **G3**'s load rule. `bus/state.rs`: the `keys` field under `lenient`, `keymap()` and `set_binding()`. `main.rs`: reads the keymap once at startup, wires `action-for`, maps `keymap::Action` → `KeyAction`, and fills the sheet's rows from `listing()`. UI: `export enum KeyAction`, the `action-for` callback, `handle-key`'s chain re-keyed onto it with **every guard and every `accept`/`reject` unchanged**, the far-skip branches split out (**G2**), the `Keys` sheet, and the `Keys` button in the transport row. |
| `pundit-harness` | **Nothing new**, except the key-injection fixture **T2** needs. |

**The rules live in the app library, not `main.rs`**, which is wiring and has no
`#[cfg(test)]` module at all — `fit`, `match_panel`, `new_match`, `drawing`,
`recents` and `zoom_input` are the established pattern (`lib.rs:7`–`:17`).

## Testing

**No test writes outside a `tempfile::tempdir()`.**

- **`keymap` (unit, in-crate):**
  - **T1. The defaults are conflict-free** — no binding appears under two
    actions. This is the one property **G3** rests on, and it is one loop.
  - Every `Action::ALL` entry has a label that round-trips through
    `from_label`, and a non-empty listing sentence. `drawing.rs:425`–`:429` is
    the shape.
  - `Binding::label` round-trips for every default, and `from_label` returns
    `None` for `"ctrl+"`, `"shift"`, `"Escapé"`, `"ctrl+shift+"`, `""`.
  - `action_for` over a table: `("r", false,false,false)` → `ToggleRecording`;
    **`("R", …)` → `ToggleRecording`** (the Caps Lock shape, **B2**);
    `("r", _, shift: true, _)` → **none** (**G2**'s accepted change, pinned so a
    later build cannot reintroduce the three-valued Shift by accident);
    `("o", ctrl: true, …)` → `OpenProject` and `("o", …)` → `MarkOut` (the
    ordering dependency **B5** removes);
    `("z", ctrl: true, shift: true, _)` → `Redo`.
  - **G3**'s load rule: an override that collides with an earlier action's is
    dropped and the earlier one stands; an unknown action name is ignored; an
    unparseable spelling is dropped and its sibling survives; `[]` unbinds.
- **T2. The named-key codes, through the harness's key injection.** For each
  `NamedKey`, inject it and assert the window's binding fired. This is the only
  test that can catch a wrong code point, and without it `LeftArrow` being
  `\u{F702}` is a number copied out of another crate.
- **`bus::state` (unit):** `"keys": "x"`, `"keys": {"record": 5}`,
  `"keys": null` each cost **that field alone** — the pen, the speech model and
  the recents list survive. Three more rows on #100's existing five-shape table,
  which is the house idiom and already written.
- **T4. No harness test**, stated deliberately: no command, no bus state and no
  project field changes, so there is nothing for the harness to drive. The
  harness's only involvement is **T2**'s fixture.
- **Manual (batched):**
  - **Press every key in A3 and confirm it still does what A3 says.** This is
    the real gate: the table is a rewrite of the app's entire input surface and
    31 of its 32 rows must come out identical.
  - Confirm a bare letter still types in **all seven** `text-editing` field
    groups (`:3783`) — the project name, the clip inspector, the highlights
    label, the match editor's row and paste box, the basket's name, a slate's
    name and tags, the New match sheet's fields.
  - Confirm the Esc cascade and all six modal layers are untouched.
  - Confirm the transport row still fits with the `Keys` button added, at the
    window's minimum; and that the sheet reads at that size. **Both unmeasured.**
  - Hand-edit `state.json` to swap `z` and `x`, restart, confirm; then write a
    conflict and confirm the log says which binding was dropped.

## Risks

1. **The Slint `if` chain is not exhaustive on `KeyAction`.** Rust's side is —
   `main.rs`'s `Action` → `KeyAction` map is a `match` and will not compile with
   an action missing — but a new action with a default binding and no Slint
   branch is a **listed key that silently does nothing**. Mitigated only
   partially, and honestly: the listing is generated from the table, so the dead
   key is at least *visible* in the sheet rather than invisible everywhere.
   Nothing in Slint can check it. **This is the cost of keeping the guards and
   the UI effects in Slint, which is where they belong** — the alternative is
   moving `highlight-tool`, `selected-clip` and `zoom-step`'s hover arguments
   into Rust, which is a much larger change for a compile-time check.
2. **It is a rewrite of the one function every key goes through.** 61
   comparisons become one lookup and a chain. Nothing here is subtle, but
   nothing here is covered by an automated test either — the guards are Slint
   properties and the effects are Slint statements. That is why the manual list
   starts with "press every key" and why the far-skip split (**G2**) is the only
   behaviour change allowed in the same change.
3. **A MenuBar would silently outrank this whole table.**
   `process_menubar_shortcuts` runs **before** the capture phase
   (`window.rs:1299`–`:1302`), and `MenuItem.shortcut` is a `keys` property that
   *only* works inside a `MenuBar` (`builtin_elements.rs:1878`–`:1881`). So if
   #32's remaining half ever lands, every accelerator on it beats every binding
   here, with no warning but a `debug_log` on ambiguity. **Recorded for #32**,
   which is the entry that would hit it.
4. **A coach can hand-edit themselves into a keyless app**, by binding
   everything to one key or unbinding it all. The way back is deleting the
   `keys` key (**S1**), which is one line in a file they just edited. A button
   is **Deferred 2**; a guard against it would be machinery for a self-inflicted
   state with an obvious cure.

## Deferred

1. **The rebind UI** — a row per action in the sheet with a capture field, and
   **G3**'s last-wins behaviour made visible. #96's own order: the list first.
2. **"Restore defaults"**, which arrives with the UI and not before: without a
   UI the only way to a broken keymap is a hand edit, and the cure is the same
   hand edit.
3. **The splitter grip's arrows** (**#99**) — this spec says what shape they
   take and does not build them.
4. **Keyboard navigation inside the popovers** (#85's other half). Structurally
   separate: `handle-key` does not run while a popover is up (**A5**).
5. **Opening the sheets by key** (**X**).
6. **Moving to `slint::Keys`**, if `matches` is ever made public (**K3**).
7. **Moving to `KeyBinding`**, if Slint ever dispatches it in the capture phase
   and hands `activated` an event (**B4**).
8. **The AZERTY digit nicety.** Slint's `Digit0`–`Digit9` carry a
   layout-aware shift rule (**K3**) that would make the zoom keys work unshifted
   on AZERTY without a rebind. Adopting it means importing a 30-entry
   shifted-pairs table into our crate to buy what one rebind buys. Not worth it
   unless a non-QWERTY coach actually appears (**Open question 4**).

## Open questions for the coach

1. **What should open the Recent popover?** (**D4**) It ships bindable and
   unbound. Any free key works; `ctrl+shift+o` beside `ctrl+o` is the
   conventional reading.
2. **Is `F1` right for the list, and is a sheet right?** (**D2**) #113's folding
   sections are landing on `claude/folding-sections` as this is written, so a
   "Keys" section in a column becomes cheap — but a list read once and dismissed
   is a sheet's shape, not a panel's.
3. **Should the 20-second skip keep Shift?** (**G2**) The spec makes it its own
   rebindable action, which is a small behaviour change (Shift+R no longer
   records) in exchange for making the far skip listable and rebindable.
4. **Is there a non-QWERTY coach?** If not, **#35 closes as retired by
   rebinding** (**K2**) and **Deferred 8** never happens.
5. **Is a hand-edited `state.json` enough for now?** (**D3**) #96 says the
   read-only list is most of the value; this asks the coach to confirm before
   **Deferred 1** is scheduled.

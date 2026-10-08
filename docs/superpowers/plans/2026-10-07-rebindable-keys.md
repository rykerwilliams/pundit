# Plan — rebindable keys (#96)

Spec: `docs/superpowers/specs/2026-10-02-rebindable-keys-design.md`. **Read it
first**, with the corrections in "The spec is wrong, or now stale, about seven
things" below, and read `CLAUDE.md` — the transport-keys paragraph, the
recording allow-list, the `state.json` versus `project.json` rule and the
sheets' `editing` fold all bind here.

**What ships:** every key the app binds becomes one row in one table in Rust;
`handle-key` consults that table instead of 61 inline `event.text ==`
comparisons; the table is stored as overrides in `state.json`, listed in a sheet
the coach can open, and **rebindable from that sheet**. The scope is the whole
thing the coach asked for ("i did ask for a configuration system eh") in the
order #96 itself asked for it: the table, then the list, then the capture field.

**Gates.** Every `cargo` call but `fmt` under
`flock /tmp/claude-1000/cargo.lock nice -n 19`. Clippy must be
`rustup run 1.92 cargo clippy --workspace --all-targets -j 3 -- -D warnings` —
a clean local 1.98 is not the gate, and `-j 3` is required (#121). **Never
`cargo test --workspace`.** Never pipe clippy or tests to `tail`/`head`.

**Known flakes, not yours:** #72 (harness, "timed out waiting for a settled
position" — the retry landed in `7310a54`, so a sighting now is news) and #101
(`corrupted size vs. prev_size`, SIGABRT, no test named). **Log any sighting.**

## Where this stands (update it as tasks land)

- **1 — the `text-editing` fold becomes the platform's own property.** **Done**
  (`claude/96-task-1`). `app.slint:4031` is
  `property <bool> text-editing: TextInputInterface.text-input-focused;` and the
  list is gone. Deleted, counted off the diff: **13 properties** — the window's
  `basket-editing`, `match-editor-editing`, `new-match-editing`,
  `slate-editing` and `slate-filter-focused`; the three sheets' own `editing`;
  `MatchEditorSheet::paste-focused`; `HighlightsPanel::editing` and its
  `label-focused`; `NewMatchSheet::any-focused`; and `SetupField::focused` —
  **three `<=>` wires**, **three hand-clears** in the `close-*` functions, the
  `refresh-editing` function and its three calls, and **five** `changed`
  handlers that existed only to feed one of them. `app.slint` is +89 / −152,
  and of that the **code** is +7 / −50; the rest is comment, most of it the
  reasons the deleted flags carried. `tests/ui/text_editing.rs` is new: three
  tests over sixteen fields — every one of the thirteen `TextInput` sites in
  the file, the match setup sheet's shared `SetupField::edit` among them.
  **Four corrections for tasks 2–5:**
  - **Every line number in task 1's text was stale by ~+56**, the same drift §A
    predicted; the ones task 3 needs are re-derived in the list below.
  - **It was 13 properties, not the 10 step 1.2 names, and the three it misses
    are each one that only fed a named one:** `MatchEditorSheet::paste-focused`
    (into `refresh-editing`), `NewMatchSheet::any-focused` and
    `SetupField::focused` (into that sheet's `editing`). Grep per *term* is not
    enough — the sweep has to follow each term's own inputs. The three
    survivors were read right, though: `Inspector::editing` kept its second
    job, `HighlightsPanel::editing` had no other reader as the step guessed,
    and `MatchEditorSheet::line-focused` stays because `src/main.rs`'s
    `editor_rebuild` reads it.
  - **#132 is not of this class and the plan says it is.** Step 1.2's *"#130
    and #132 were both a field missing from the list this deletes"* is wrong:
    #132 was `on_show_slate` overwriting a focused field's text, fixed in
    `main.rs` by `show_clip`'s guard. **#123 is the second one** — the gate
    `slate-editing` repeated drifting from its section's own `if`. The comment
    that shipped cites #130 and #123.
  - **An `ElementHandle` test needs the window made tall** (`set_size`):
    `visit_descendants` skips any subtree whose geometry falls outside the
    enclosing clip, so at the headless default size the inspector column's
    fields are unfindable and unclickable. Tasks 4 and 5 will hit this.
- **Task 3's line numbers after task 1**, re-derived at `claude/96-task-1`:
  `handle-key` is `app.slint:4367`–`:4658` and **still holds 61 `event.text ==`
  of the file's 64**, so §A7's two counts are unchanged by this task. The five
  reads of `text-editing` task 3 must leave exactly as they are: `:4392`
  (basket), `:4408` (New match), `:4453` (match editor), `:4462` (layer 7's
  `reject`) and `:4670` (`keys.key-pressed`'s Esc).
- **2 — `keymap.rs`: the table, the labels, the defaults, the overrides.**
  **Done** (`claude/96-task-2`). 29 actions, 34 bindings, four per-action
  methods (`name`, `what`, `when`, `default_keys`) and 16 tests in the module,
  plus `reads_the_keymap_overrides` and three rows on
  `one_unreadable_value_costs_that_field_alone` in `bus/state.rs`. Everything
  the step asked for shipped; **five things the step left for the
  implementation to decide**, each pinned by a test:
  - **An override displaces a *default*; `ALL`'s order only breaks a tie
    between two *stored* rows.** Step 5 reads "the stored map is applied in
    `Action::ALL` order; a binding already taken is dropped for the later
    action", which taken literally over defaults-and-overrides together makes
    a hand-edited `{"tagHomeGoal":["o"]}` **lose to `markOut`'s own default** —
    the coach's edit dropped in favour of the default they were editing away
    from, which is the only rebind path there is until task 5. So the stored
    rows are placed first and the defaults fill in around them: the key moves,
    `markOut` is left unbound, and the drop is logged. Override-vs-override is
    still `ALL`'s order, which is what step 5 and **G3** are actually about.
    The result is a **fixpoint** — `with_overrides(k.overrides()) == k`,
    displaced rows included — which is what makes task 5's whole-diff write
    safe.
  - **The default table is written as *labels*** (`Action::default_keys`),
    parsed through `from_label`, so it reads as the sheet will show it and the
    spec's **D1** table can be checked against it by eye. The cost is a typo
    becoming a silently missing default; `every_default_label_parses` is the
    answer, and it asserts per action rather than on the total.
  - **`from_label` is order- and case-insensitive on the modifiers and rejects
    a modifier named twice.** A hand-edited file is the use case; `label()`
    stays canonical. It is a prefix-strip rather than a split on `+`, so a key
    that *is* `+` has a spelling.
  - **A character is folded to lower case at parse time**, not only in the
    matcher: a stored `"R"` that kept its case would be a binding no key event
    could ever match.
  - **`Action::ALL` is indexed with `self as usize` nowhere** — the keymap is
    `Vec<(Action, Vec<Binding>)>` in `ALL` order, so there is no parallel index
    to drift. `the_list_holds_every_action_in_declaration_order` pins the
    pairing anyway, because `ALL` is the sheet's order and the tie-break.

  **Two corrections for tasks 3–5:**
  - **`CURRENT_FORMAT_VERSION` is 16, not the 15 step 9 names** (`store.rs:21`,
    read). Untouched either way, as both say.
  - **`State` has gained `folds` since the plan was written** (#113), so the
    struct is `bus/state.rs:64`–`:133` and `keys` is its ninth field. The
    container `default` is at `:62`. Nothing else in step 9 moved: one
    attribute, `keymap()` on `pen()`'s shape, and no setter.

  **Not in this task, by the plan's own scope:** `Keymap::rebind`/`unbind` and
  `AppFiles::set_keymap` (task 5), and the reserved rows — Escape, Tab, Home,
  End — which are **not** `Action` variants, so task 4's sheet writes them as
  its own static lines if it wants them (its row-count test is
  `Action::ALL.len()`, which is the 29 rebindable ones).
- **3 — `handle-key` is re-keyed onto the table.** Not started.
- **4 — the Keys sheet.** Not started.
- **5 — rebinding from the sheet.** Not started.

**Shipping, and what is parallel.** Checked by opening every file named, not by
assertion:

- **1 and 2 are file-disjoint and may run in parallel.** Task 1 touches
  `crates/pundit-app/ui/app.slint`, `crates/pundit-app/tests/ui/` and
  `CLAUDE.md:723`. Task 2 touches `crates/pundit-app/src/keymap.rs` (new),
  `crates/pundit-app/src/lib.rs` (one `pub mod` line) and
  `crates/pundit-app/src/bus/state.rs`. No file is in both lists; the only
  shared file in the repo is `CLAUDE.md`, and task 2's paragraph (the
  `state.json` list at `:580`) is 140 lines from task 1's (`:723`).
- **1 ships alone, and should.** It is a bug-class fix (#130's class) with its
  own tests and no dependency on anything else here.
- **2 ships alone.** Nothing calls it yet, and nothing warns: every item it adds
  is `pub` in a library crate, so rustc cannot call it dead.
- **3 needs 1 and 2, and is sequential with 1 even though their lines are
  disjoint.** Both edit `app.slint`: task 1 edits the `text-editing` property at
  `:4036`–`:4038`, task 3 rewrites `handle-key`'s chain at `:4479`–`:4670`. Task
  3 leaves every *read* of `text-editing` (`:4405`, `:4421`, `:4466`, `:4475`,
  `:4683`) exactly as it is — the spec forbids moving them — so the edits do not
  overlap. They are still sequential: rebasing a 190-line rewrite of one
  function under another edit to the same file is not worth the parallelism, and
  the spec's Risk 2 is that this function is rewritten once, not twice.
- **3 and 4 ship together, one PR.** Task 3 gives `showKeys` a default of `F1`;
  without task 4 that is a listed key that does nothing, which is Risk 1 shipped
  on purpose.
- **5 ships separately, after.** It is the only task that writes `state.json`,
  and it is the one #96 says comes last.

---

## 0. Before task 1: the crate, and where "context" goes

Two things the coach asked for by name — *use a library or known pattern* — are
settled here rather than inside a task, because both tasks 2 and 3 depend on the
answer.

### 0a. No crate fits, and the reason is Slint's event model

Searched crates.io, read the sources, and the honest answer is **no dependency,
convention instead**. What was looked at:

- **`keybinds` 0.2.0** (MIT, 2.3k recent downloads, deps `bitflags` + `smallvec`
  — **both already in `Cargo.lock`** at 2.13.2 and 1.16.1, so it would add one
  crate to the graph, and MIT is already in `packaging/about.toml`'s `accepted`).
  It is the closest fit by shape: parser, generator, dispatcher, `Keybinds<A>`
  keyed by action, serde. **It is rejected on one documented fact:** its syntax
  forbids `Shift` on a character key — *"`Shift` modifier key is only available
  with named keys… when you want Shift+A you should use
  the logical input `A`"* (`doc/binding_syntax.md`). Two of this feature's
  default bindings are `shift+a` and `shift+d` (the far skip, spec G2), which in
  that grammar can only be written `A` and `D` — and `A` is **case-sensitively**
  a different key from `a`, which throws away the Caps-Lock rule (**0b** below).
  Its restriction is deliberate and ours is load-bearing; adopting it would mean
  overriding the thing it exists to provide.
- **`keymap` (keymap-rs) 1.0.0-rc.7** — pre-1.0, TUI/WASM-shaped (crossterm,
  termion, wasm backends), derive-macro-first for compile-time-fixed maps, and
  would need a Slint backend written by us anyway.
- **`keyboard-types` 0.8.3** — types only, hugely adopted. It would supply a
  100-variant W3C key enum and its spellings and leave the `+` grammar, the
  matcher and the Slint mapping to us. A closed 16-entry list is the spec's
  deliberate choice (**B1**), so the crate's value here is a vocabulary we would
  use a sixth of.

**The irreducible work no crate removes**, and the reason it is irreducible:
Slint hands the app a *character*, and for a named key that character is a
private-use code point. Every keybinding crate's core type is either a physical
code (unavailable — **0c**) or a logical key enum that has to be mapped *from*
that character, and only Slint knows that mapping. So the table is ours whatever
we depend on.

**The convention adopted instead is VS Code's `keybindings.json`**, which GTK's
accel maps and Emacs' keymaps reach from other angles: a **default table in
code**, a user file holding **only the overrides**, keyed by the **action's
name**, a **read-only list** of what is bound, and **last-wins** on a conflict
with the displaced binding shown as unbound. Spec **S1** and **D2** already
describe exactly this; what this plan adds is that the stored file is a *diff*
from the defaults and not a copy of them (step 2.6), which is the half of the
convention the spec left open and the half that lets a later version change a
default.

**What is *not* copied from VS Code is the `when` clause**, and the reason is
verified, not stylistic. In this app a bound key is swallowed **whether or not
its guard passes**: every branch of `handle-key` is
`if (event.text == …) { if (pressed && <guard>) { … } return accept; }` — the
`accept` is outside the guard, in all 29 branches. That is deliberate, and the
comment at `app.slint:4378`–`:4381` says why: *"every shortcut, ahead of
whichever child has focus, so a touched slider never turns arrows into volume or
scrub steps."* A `when` clause makes a binding **not match**, so the key falls
through to the focused element — which would hand `←` to a touched volume slider
in exactly the case `can-play` is false. So the lookup is `(key, modifiers) →
action` and nothing else.

### 0b. Where each kind of context lives, and why the two hazards survive

"Context" exists in this app in three layers, and each is already in the one
place it can be. The feature moves none of them.

1. **Routing — may the window see this key at all?** `text-editing`, tested at
   `app.slint:4475`, *ahead of every binding*. It must stay in Slint because
   only Slint can `reject`, and `reject` is what delivers the letter to the
   field. **Task 1 makes it the platform's own `TextInputInterface.text-input-focused`**,
   so it stops being a seven-term disjunction a new field has to remember to
   join — the #130 class of bug becomes unwritable rather than merely tested
   for. That is the answer to "a data-driven key map must not make the fold
   easier to get wrong": it makes it impossible to get wrong, by deleting the
   list.
2. **Effect gates — should this action fire now?** `can-tag`, `can-draw`,
   `can-play`, `can-scan-fast`, `can-fit`, `previewing`, `recording`,
   `match-at-cap`, `selected-clip != ""`, `!event.repeat`. These stay in Slint,
   in the branch, because the effects are in Slint (`highlight-tool`,
   `selected-clip`, `zoom-step`'s hover arguments) and because of **0a**: a gate
   that suppressed the *match* would change what the key does to a focused
   child. The listing's "when" column is a prose sentence on the action, as
   spec **D2** decided, not a second machine-readable copy of `can-tag`.
3. **Command refusal — may the bus do this while recording?** The recording
   allow-list, `bus/mod.rs:974`–`:1012`: a `!matches!` over an explicit list of
   `Command` variants, with `return eprintln!("bus: refused while recording")`
   for everything else.

   **The allow-list's exhaustiveness is untouched by this feature, and cannot be
   reached by it.** The list is keyed on **commands**, not on keys. A rebound
   key sends the same `Command` it sent before, so no rebinding can move an
   action onto or off the list, and **a command added later is still refused by
   default** because the guard is a negated explicit list. The keymap cannot
   invent a command — `Action` maps onto callbacks that already exist. The UI
   gates in (2) are the greying; the bus is the backstop, and its own comment
   says so (*"The UI greys these out, so reaching here is a UI bug"*).

   The one thing a coach *can* do is bind `z` to `fitWindow`, so a key that used
   to tag a goal mid-take now does nothing mid-take. That is the feature working.

### 0c. Slint carries text, not scancodes — verified, and what the coach gets instead

Checked four ways in Slint 1.18, which `Cargo.toml:20` pins:

| Claim | Where | Verdict |
|---|---|---|
| `KeyEvent` has exactly three fields: `text`, `modifiers`, `repeat` | `i-slint-common-1.18.0/builtin_structs.rs:104`–`:111` | **true** |
| the winit backend reads `logical_key` only | `i-slint-backend-winit-1.18.0/winitwindowadapter.rs:1337`, and `:1400` | **true** |
| `physical_key` appears zero times in that crate | `grep -rc` over `i-slint-backend-winit-1.18.0` | **true: 0** |
| a key that produces no text never reaches the app | `winitwindowadapter.rs:1407`–`:1410`, `if text.is_empty() { return Ok(()) }` | **true** |
| the text is the *shifted* reading | `Key::Character(str) => str.as_str().into()`, `winitwindowadapter.rs:1366` | **true** |

**So a layout-independent binding is not expressible**, and no plumbing on our
side can recover one: the information is discarded at the winit boundary. What
the coach gets instead is the thing a keymap gives directly — a French coach
binds the zoom and `a`/`d` to the keys where they are on their own keyboard,
once, and `state.json` remembers. **#35 closes as retired by this feature**, not
as waiting for a Slint that may never expose scancodes (spec **K2**).

**One consequence that is ours to get right:** the matcher must **lowercase the
event text and compare the three modifiers exactly**, which is Slint's own rule
for its internal matcher (`i-slint-core-1.18.0/input.rs:936`–`:938`, with the
comment *"the event text will be in uppercase if caps lock is active, even if
shift is not pressed"*). That is what keeps Caps Lock free, and it is why
`keybinds`' case-sensitive keys do not fit.

---

## 1. The `text-editing` fold becomes the platform's own property

**Files.** `crates/pundit-app/ui/app.slint`;
`crates/pundit-app/tests/ui/{main.rs, text_editing.rs}` (new module);
`CLAUDE.md:723`–`:726`.

1. **`text-editing` becomes one expression.** `app.slint:4036`–`:4038` is today
   a seven-term disjunction. It becomes:

   ```slint
   /// A text field has focus: every shortcut yields to it (C6).
   ///
   /// **Slint's own**, not a list of this window's fields. `TextInput`
   /// sets it on focus-in and clears it on focus-out *and on deinit*
   /// (`i-slint-core/items/text.rs:1234`, `:1257`, `:839`), so a field
   /// cannot be left out of it and a field destroyed while focused cannot
   /// leave the shortcuts switched off. #130 and #132 were both a field
   /// missing from the list this deletes.
   property <bool> text-editing: TextInputInterface.text-input-focused;
   ```

   `TextInputInterface.text-input-focused` is a built-in global
   (`i-slint-compiler-1.18.0/builtin_elements.rs:3602`–`:3608`) that lowers to
   `WindowInner::text_input_focused()` (`generator/rust.rs:5316`).
2. **What that deletes, and what it must not.** Checked by grepping every use of
   each term:
   - **Delete**, because the fold was their only reader: `basket-editing`
     (`:3925`, cleared by hand at `:4317`, wired at `:6571`),
     `match-editor-editing` (`:4015`, `:4305`, `:6644`), `new-match-editing`
     (`:3887`, `:4340`, `:6699`), `slate-editing` (`:3651`–`:3653`) and
     `slate-filter-focused` (`:3665`, assigned at `:4919`–`:4921`), and with
     them the `in-out property <bool> editing` on `BasketSheet` (`:2095`, set at
     `:2213`), `MatchEditorSheet` (`:2937`, `refresh-editing` at `:2967`, called
     at `:2917`, `:3082`, `:3129`) and `NewMatchSheet` (`:3258`, set at `:3289`).
   - **Keep `Inspector::editing`** (`:914`): it has a second job at `:5862`,
     where the inspector's focus-loss clears `editing-clip-id`.
   - **Keep `HighlightsPanel::editing`** (`:1522`) only if a reader survives;
     grep says the fold is its only one, so delete it and `label-focused` with
     it unless the implementation finds another.
   - **The three hand-clears in `close-basket` / `close-match-editor` /
     `close-new-match` go**, and the reason they existed is the reason the change
     is safe: *"an `if` cannot run code when it is destroyed"* — `TextInput`'s
     `deinit` clears the global for exactly that case
     (`items/text.rs:837`–`:842`).
3. **The `slate-editing` gate's duplication is what this is for.** It repeats its
   section's own `if` condition (`:4893`, `root.slates.length > 0 ||
   root.slate-tag-filter != ""`) because the flag would otherwise stick. #123 is
   the recorded instance of that pair drifting apart. One expression cannot
   drift.
4. **Nothing else changes.** Every read of `text-editing` stays: layers 3, 4 and
   6's Esc guards (`:4405`, `:4421`, `:4466`), layer 7's `reject` (`:4475`) and
   `keys.key-pressed`'s Esc (`:4683`).
5. **Verified no behaviour moves.** Every `TextInput` in the window was located
   (`LineEdit`/`TextEdit`/`TagField` at `:963`, `:973`, `:1046`, `:1142`,
   `:1608`, `:2207`, `:2403`, `:3076`, `:3121`, `:4749`, `:4911`, `:5157`,
   `:5183`) and each is either folded today or inside a modal layer that returns
   before layer 7 — the match setup sheet's `SetupField` fields (`:2403`) are the
   one group never folded, and layer 5 (`:4435`) returns `reject` for every
   non-Esc key ahead of layer 7, so they are already protected and stay so.
   **`ComboBox` is not a `TextInput`**: no `combobox.slint` in any of Slint's
   four styles contains one, so the nine `ComboBox`es do not newly suppress
   shortcuts.
6. **`CLAUDE.md:723`** — *"A sheet with fields folds its `editing` into the
   window's `text-editing`"* — becomes the opposite rule: the window asks Slint,
   a sheet folds nothing, and a new field joins nothing. Keep the *reason* (the
   sheet's Esc guard tests `!text-editing`, so the first Esc must leave the field
   and the second close the sheet).
7. **Two ids for the tests**, on `slate_menu`'s precedent: `basket-name-edit` on
   the basket sheet's `LineEdit` (`:2207`) and `slate-filter-edit` on the slate
   filter's (`:4911`).

### Tests

New `crates/pundit-app/tests/ui/text_editing.rs`, a module of the one test
binary (BACKLOG #121 — it does not add a file beside `main.rs`, it is listed in
it).

- **`a_letter_typed_in_a_field_fires_no_shortcut`.** One loop over the main
  window's fields, each by element id, with the window state that makes it
  visible: `AppWindow::name-edit`, `Inspector::name`, `Inspector::tags`,
  `Inspector::notes`, `AppWindow::slate-name-edit`, `AppWindow::slate-tags-edit`,
  `AppWindow::slate-filter-edit`. For each: `ElementHandle::find_by_element_id`,
  `mock_single_click(PointerEventButton::Left)`, then dispatch `r`, `i`, `o` and
  `" "` and assert **no** recorded callback fired (`on_toggle_recording`,
  `on_mark_in`, `on_mark_out`, `on_toggle_play`). This is the spec's manual
  bullet *"confirm a bare letter still types in all seven field groups"*,
  automated — and it is the first automated cover the #130 class has ever had.
- **`esc_in_a_sheet_field_leaves_the_field_before_it_closes_the_sheet`.** Open
  the basket sheet, click `BasketSheet::basket-name-edit`, press Escape: the
  sheet is still open. Press Escape again: it is closed. This is the path that
  used to need `basket-editing` and now needs the global, and the second press
  is what exercises `TextInput::deinit`.
- **Not automated, on the manual list with the reason:** the match editor's row
  field and paste box, and the New match sheet's fields. They share the basket
  test's mechanism exactly; reaching them costs opening a sheet whose rows come
  from a model the test would have to build.

### Sabotage proof

One line, both directions.

- Replace the binding with `property <bool> text-editing: false;`. The first
  test fails on the first field (a letter fires a shortcut) **and** the second
  fails on the first Escape (the sheet closes with a half-typed name in it).
- Replace it with `true`. The second test fails on the *second* Escape — the
  sheet never closes. The pair is what pins "the fold is this property and not a
  constant".

---

## 2. `keymap.rs`: the table, the labels, the defaults, the overrides

**Files.** `crates/pundit-app/src/keymap.rs` (new),
`crates/pundit-app/src/lib.rs` (one line), `crates/pundit-app/src/bus/state.rs`
(the field, the accessor, three test rows), `crates/pundit-app/Cargo.toml` (one
comment), `CLAUDE.md` (the `state.json` paragraph at `:580`).

1. **`Action`**, 29 variants, spec **D1**'s table unchanged. `ALL` in a fixed
   order (which is the load rule's order, step 5), and four methods on
   `drawing.rs:69`–`:117`'s shape — `name()` (the camelCase `state.json` key),
   `from_name()`, `what()` (what it does) and `when()` (the one prose sentence,
   spec **D2**).
2. **`Binding`, `Key`, `NamedKey`**, spec **B1**:

   ```rust
   pub struct Binding { key: Key, ctrl: bool, shift: bool, alt: bool }
   pub enum Key { Char(char), Named(NamedKey) }
   ```

   **`NamedKey`'s char comes from `slint::platform::Key`, never from a literal
   code point.** `slint::platform::Key` is public
   (`i-slint-core-1.18.0/platform.rs:322`, re-exporting `input::key_codes::Key`)
   and `impl From<Key> for char` is generated beside it (`input.rs:378`–`:385`),
   so `NamedKey::LeftArrow` is one `match` arm returning
   `slint::platform::Key::LeftArrow` and `char::from` does the rest. **This is a
   deviation from the spec and it deletes the spec's `T2` entirely** (see the
   corrections below). The closed list stays the spec's: the arrows, Escape,
   Return, Tab, Delete, Backspace, Home, End, PageUp, PageDown, Space, F1–F12.
   - **`keymap.rs` is the library's first use of `slint`.** The crate already
     depends on it unconditionally, and `platform::Key` is a plain enum that
     needs no display, so `cargo test -p pundit-app` on a headless box is
     unaffected. `Cargo.toml:17`–`:19`'s comment — *"The library (the bus) uses
     none of the UI dependencies"* — gains the exception and keeps the claim that
     matters: no windowing, no EGL, no file dialog, nothing needing a display.
3. **`label` / `from_label`**, the conventional grammar: modifiers in the fixed
   order `Ctrl+Shift+Alt+`, then the key — `"r"`, `"ctrl+z"`, `"ctrl+shift+z"`,
   `"shift+LeftArrow"`, `","`, `"Delete"`, `"F1"`. Parsing is
   case-insensitive on the modifier words and on a named key's name; emitting is
   canonical. An unparseable label is `None`, never an error — the rule every
   stored label in this app follows (`Pen::from_label`, `drawing.rs:115`).
   - **One canonicalization rule, because two spellings would reach one key:**
     `from_label` never yields `Key::Char(c)` for a `c` that a `NamedKey` owns,
     so `" "` is not a label and `"Space"` is. A test pins it.
4. **`Keymap::defaults()`**, spec **D1**: 29 actions, 34 bindings, which is
   `app.slint`'s 29 branches with `skip`'s Shift argument lifted into
   `skipBackFar` / `skipForwardFar`, plus `showKeys` (`F1`) and `showRecents`
   (no default — the open question).
5. **`Keymap::with_overrides(map)`**, spec **G3**'s load rule: applied in
   `Action::ALL` order; a binding already taken is **dropped for the later
   action** and logged by name; an unknown action name is ignored; an
   unparseable label is dropped and its siblings survive; `[]` unbinds; an absent
   action keeps its default.
6. **`Keymap::overrides()` returns only what differs from the defaults**, as a
   `BTreeMap<String, Vec<String>>`. Two reasons, both load-bearing:
   - **A diff, not a copy**, so a later version that changes a default reaches a
     coach who never touched that action. Writing the whole map would pin all 29
     for ever. This is VS Code's rule and the spec does not state it.
   - **`BTreeMap`, not `HashMap`**, so the key order in the file is stable.
     Every `AppFiles` setter rewrites the whole document (`bus/state.rs:477`),
     so a `HashMap` would shuffle `state.json`'s keys on every pen change.
7. **`Keymap::action_for(text, ctrl, shift, alt) -> Option<Action>`**, the one
   lookup: lowercase the text, compare the three modifiers exactly (**0c**). A
   linear scan over ≤34 rows, called once per key event.
8. **`Keymap::listing()`** → one row per `Action::ALL` entry: `what()`, the
   bindings joined `", "` or `"—"`, and `when()`. Generated from the same table
   the matcher reads, so a listed row is a row the matcher holds (spec **D2**).
9. **`state.json`**, spec **S1**: one new field on `State`
   (`bus/state.rs:62`–`:113`), `keys: BTreeMap<String, Vec<String>>`, with
   **one attribute**, `#[serde(deserialize_with = "lenient")]` — `default` is on
   the container (`:60`), and this is the first field added since #100 landed, so
   it is the first that costs one attribute rather than two.
   - **One accessor only: `AppFiles::keymap() -> Keymap`**, on `pen()`'s shape
     (`:351`–`:357`), read **once at startup** by task 3. **No `set_*` in this
     task:** it would have no caller until task 5, and task 5 needs a different
     signature anyway (step 5.4).
   - **What `lenient` means for an unreadable binding, stated and accepted:** a
     bad *label* costs that label, a bad *action name* costs that row, and a
     JSON **type** error on `keys` itself (`"keys": "x"`, or a value that is not
     an array of strings) costs **the whole keymap and nothing else** — every
     binding reverts to its default and the pen, the speech model, the recents
     list and the window size survive. That is the bargain `bus/basket.rs`
     strikes for its `pieces` and the recents list strikes for its paths, for
     the reason `bus/state.rs`' own header gives: what is lost here is a
     re-pick, never a project. Reading the map as `BTreeMap<String, Value>` to
     rescue the good rows is machinery for a hand-edit.
   - **`CURRENT_FORMAT_VERSION` does not move.** It is 15 (`store.rs:21` —
     read it, do not trust this line). No field on any stored struct, no bump,
     no `project_format.rs` test. A binding is a property of the coach's hands.
10. **`CLAUDE.md`'s `state.json` paragraph** gains the keymap beside the pen and
    the speech model, with the one sentence that matters: the file holds the
    **overrides**, so deleting the `keys` key is the reset path and a changed
    default still reaches a coach who never touched that action.

### Tests

`keymap.rs`'s own `mod tests` (the established place — `lib.rs:7`–`:19`'s
modules all test themselves; `main.rs` has no test module at all):

- **T1. The defaults are conflict-free** — no binding appears under two actions.
  One loop, and the one property **G3** rests on.
- Every `Action::ALL` entry: `from_name(name())` round-trips, and `what()` and
  `when()` are non-empty. `drawing.rs:656`–`:660` is the shape.
- `Binding::label` round-trips for every default, and `from_label` is `None` for
  `""`, `"ctrl+"`, `"shift"`, `"ctrl+shift+"`, `"Escapé"`, `" "` and
  `"ctrl+LeftArrow+a"`; `from_label("Space").label() == "Space"`.
- **Every `NamedKey` maps to a distinct char**, which is what catches a
  copy-paste in the `slint::platform::Key` match — the one failure mode the
  spec's `T2` was invented for, now a unit test with no window in it.
- `action_for`: `("r", f, f, f)` → `ToggleRecording`; **`("R", f, f, f)` →
  `ToggleRecording`** (Caps Lock, **0c**); `("r", f, **true**, f)` → `None`
  (**G2**'s accepted change, pinned so a later build cannot reintroduce a
  three-valued Shift); `("o", ctrl)` → `OpenProject` and `("o", …)` → `MarkOut`
  (the ordering dependency **B5** removes); `("z", ctrl, shift)` → `Redo`;
  `("y", ctrl, shift)` → `None`.
- `with_overrides`: a collision is dropped for the later action and the earlier
  one stands; an unknown action name is ignored; an unparseable label is dropped
  and its sibling survives; `[]` unbinds; an absent action keeps its default.
- `overrides()`: `defaults().overrides()` is **empty**; one changed action gives
  exactly one key; a round-trip through `with_overrides` is the identity.

`bus/state.rs`'s `mod tests`:

- **Three rows on `one_unreadable_value_costs_that_field_alone`** (`:1075`), the
  house idiom and already written: `"keys":"x"`, `"keys":{"toggleRecording":5}`,
  `"keys":null`, each with a `keys` predicate beside the existing `panels` and
  `window` ones.
- **`remembers_the_keymap`**, on `remembers_the_pen`'s shape (`:850`): write
  an override, read it back through `keymap()`, assert `action_for` moved.

### Sabotage proof

Three, each failing a different test:

1. **Drop the collision check from `with_overrides`** (take the later action's
   binding). The override-collision test fails and **T1 still passes** — which is
   what proves the two test different rules.
2. **Change one default to collide** (`fitWindow` → `"r"`). T1 fails and nothing
   else does.
3. **Drop the lowercase from `action_for`.** The `("R", …)` case fails and the
   `("r", …)` one passes — the Caps-Lock rule, pinned on its own.

---

## 3. `handle-key` is re-keyed onto the table

**Files.** `crates/pundit-app/ui/app.slint`, `crates/pundit-app/src/main.rs`,
`crates/pundit-app/tests/ui/keys.rs` (new module), `CLAUDE.md` (`:554`, `:556`,
`:586`, `:703`, `:742`).

**The anchors, re-derived at `2749e00`** — the spec's §A is against `4f418ae` and
says to do this. `handle-key` is `app.slint:4382`–`:4671` (**+290** on the spec's
`:4092`); `text-editing` is `:4036` (**+253** on `:3783`); the `FocusScope` is
`:4673` with `capture-key-pressed` at `:4674` and `capture-key-released` at
`:4677`; its own `key-pressed` Esc is `:4683`. The branch lines are `:4479`
(Home/End), `:4482` (the Ctrl branch), `:4513` (`c`), `:4527`/`:4533` (`i`/`o`),
`:4539`/`:4542`/`:4545` (`z`/`x`/`v`), `:4552` (`h`), `:4559` (`r`), `:4571`
(the Esc cascade), `:4597` (Delete), `:4603` (Space), `:4609`/`:4615` (the
skips), `:4623` (`,`/`.`), `:4632` (`[`/`]`), `:4640` (`j`/`l`), `:4652` (`f`),
`:4658`/`:4664` (the zoom) and `:4670` (the final `reject`). **61 `event.text ==`
comparisons inside that range, 64 in the file** — the spec's A1 count is right,
re-derived.

1. **One Slint enum and one callback**, spec **B3**:

   ```slint
   export enum KeyAction { none, open-project, undo, redo, … }   // 30 members
   callback action-for(string, bool, bool, bool) -> KeyAction;    // text, ctrl, shift, alt
   ```

   Four scalars rather than the `KeyEvent`, because `KeyEvent` is not in
   `slint`'s public API, and because `repeat` is read by the *guards*, which stay
   in Slint. **Not `pure`:** nothing binds on it, and the Rust side reads a
   `RefCell` that task 5 mutates.
2. **`handle-key`'s chain is re-keyed, and nothing else moves.** Layers 1–7
   (`:4383`–`:4477`) are untouched, Home/End stays its own branch, and past it:

   ```slint
   let act = root.action-for(event.text, event.modifiers.control,
                             event.modifiers.shift, event.modifiers.alt);
   if (act == KeyAction.clear-drawings) { … return accept; }
   ```

   **Every guard, every `accept`/`reject`, the Esc cascade and the branch bodies
   stay exactly as they are.** Three rules the rewrite must not break:
   - **The lookup runs for releases too.** `handle-key` is called from
     `capture-key-released` as well (`:4677`), and today a gated or ungated key
     returns `accept` on release — the comment at `:4380` says why: *"a slider
     fires `released` on an arrow key's release."* So `action-for` is called
     **before** any `pressed` test and the `accept` stays outside it. Guarding
     the whole lookup with `if (pressed)` would make every arrow release
     `reject` and hand it to a touched slider. This is the single most likely
     way to get this task wrong.
   - **`accept` is outside the guard**, in every branch, as **0a** records.
   - **The Ctrl branch's `return reject` at `:4509` stays**, as the fall-through
     that delivers every other Ctrl combo to a focused child. It is now reached
     when `act == KeyAction.none` and `event.modifiers.control`.
3. **The far-skip split** (**G2**), the one behaviour change allowed in this
   task: `root.skip(event.modifiers.shift ? -10 : -3)` becomes two branches
   calling `root.skip(-3)` and `root.skip(-10)`, and the same forward. The named
   costs: **Shift+R no longer records**, Shift+letter fires nothing, and
   `Ctrl+Shift+Y` no longer redoes (`Ctrl+Y` still does).
4. **`main.rs`: `wire_keys`**, on `wire_folds`' shape (`:2486`–`:2510`). It
   reads `state.keymap()` **once**, holds it in an `Rc<RefCell<Keymap>>`, and
   sets `on_action_for` to map `keymap::Action` → `KeyAction` through a **`match`
   on the Rust enum** — the `MatchTag`/`MatchEventKind` pattern at `:1410`–`:1412`
   and `ScanStep`'s at `:737`–`:739` — so **adding an action without mapping it
   fails to compile**. Called from `main` beside `wire_folds` (`:422`).
5. **`CLAUDE.md`, five paragraphs that stop being the record of what is bound**
   and keep only their reasoning, each pointing at the table: `:554` (the
   transport keys), `:556` (`F`), `:586` (`J`/`L`), `:703` (the match tags) and
   `:742` (`i`/`o`). Add one paragraph naming `keymap.rs` as the one place a key
   is spelled, and the three-layer context rule from **0b** — in particular that
   the recording allow-list is over commands and is not reachable from a
   rebinding.

### Tests

New `crates/pundit-app/tests/ui/keys.rs`. **This is the task's real gate and it
is automatable, which the spec says it is not** (its Risk 2 and its manual list
both assume otherwise). `fit_window.rs:202` already dispatches
`WindowEvent::KeyPressed { text: 'f'.into() }` against the real `AppWindow`, and
`tag_field.rs:72`–`:84` does it with `slint::platform::Key::Escape`.

- **`every_default_binding_still_does_what_it_did`.** One table of
  (key, modifiers, the window state its guard needs, the callback it must fire),
  one row per branch in the spec's §A3. Each row records its callback
  (`on_open_project`, `on_undo`, `on_redo`, `on_clear_drawings`, `on_mark_in`,
  `on_mark_out`, `on_tag_match_event`, `on_toggle_recording`, `on_delete_clip`,
  `on_toggle_play`, `on_skip`, `on_step_frame`, `on_jump_chapter`,
  `on_step_scan_speed`, `on_fit_window`, `on_zoom_reset`, `on_zoom_step`) or
  reads a property (`highlight-tool`). **27 of the 29 actions are covered**;
  `showKeys` is task 4's and `showRecents` has no default.
  - `on_skip`'s argument is the assertion for the far-skip split: `-3` for `←`
    and **`-10` for Shift+`←`**, which is the one behaviour change and must be
    asserted rather than described.
- **`a_shifted_letter_fires_nothing`** — Shift+`r`, Shift+`z`, Shift+`f`: no
  callback. **G2** pinned at the window, not only in the unit test.
- **`an_arrow_never_reaches_a_touched_slider`.** Find the window's one `Slider`
  (`:6195`, no id — `query_descendants().match_inherits("Slider").find_first()`),
  `mock_single_click` it to take focus, then dispatch `KeyPressed` **and**
  `KeyReleased` for `LeftArrow`: `on_skip` fired once, `on_volume_changed` and
  `on_volume_released` never. This is D10 and the release path in one test, and
  the app has never had either covered.
- **Still manual, and said rather than implied:** that the picture, the readout
  and the window actually move. The test proves the command was sent; a human
  proves the app does something with it.

### Sabotage proof

Four, each failing a different test:

1. **Wrap the lookup in `if (pressed)`.**
   `an_arrow_never_reaches_a_touched_slider` fails on the volume callback — the
   release reaches the slider — and nothing else does. This is step 2's first
   rule and the one the spec never names.
2. **Move one branch's `accept` inside its guard** (`f`, with `can-fit` false in
   the fixture). The arrow test's shape repeated for `f`: the key falls through.
   Covered by adding one `can-fit: false` row to the first test asserting
   `on_fit_window` did **not** fire *and* that no focused child saw it.
3. **Make the far skip ignore Shift again** (`root.skip(-3)` in both branches).
   `every_default_binding_still_does_what_it_did` fails on the `-10` row.
4. **Return `KeyAction::None` for one action in `main.rs`'s map** (not delete the
   arm — that is a compile error, not a failing test). That action's row fails
   and the other 26 pass.

---

## 4. The Keys sheet

**Files.** `crates/pundit-app/ui/app.slint`, `crates/pundit-app/src/main.rs`,
`CLAUDE.md`.

1. **The seventh `Sheet`** (`:1840`, whose doc says *"There are six of these"* —
   amend it). `in-out property <bool> keys-sheet-open` on the window, a `Scrim`
   wrapper at the end of the sheet stack (`:6536`'s pattern), and **layer 2's
   shape in `handle-key`**: Esc closes it, everything else is swallowed. It
   needs no `editing` fold because it has no fields — and after task 1 there is
   no fold to need.
2. **`export struct KeyRow { what: string, keys: string, when: string }`**, an
   `in property <[KeyRow]> key-rows` on the window, filled from
   `Keymap::listing()` once in `wire_keys` and again whenever task 5 rebinds.
   Three columns, spec **D2**.
3. **A `ScrollView`, with the arithmetic rather than an estimate.** `Sheet`'s
   height is `body.preferred-height` (`:1851`), and 29 rows at ~22px is ~640px
   against a window minimum of **700px** (`min-window-height`, `:3420`). So the
   rows go in a `ScrollView { height: 520px; }` inside a card of `card-width:
   640px`: 700 − 40 (the body's padding) − 24 (the heading) − 10 (spacing) − 40
   (the close row) = 586, and 520 is inside it with air. **Confirm on the manual
   list**; it is derived from the file's own numbers, not measured on screen.
4. **`F1` opens it**, as the table says, and so does a **`Keys` button beside
   `Recent ▾`** in the transport row (`:6081`–`:6096`, same `HorizontalLayout`).
   The button calls `keys.focus()` before opening, as `Recent ▾` and `Export…`
   both do. **The row's budget at the window's 1100px minimum is not measured**
   and is on the manual list — the recents spec made the same admission about
   the same row.

### Tests

- **`f1_opens_the_keys_sheet_and_esc_closes_it`**, in `tests/ui/keys.rs`:
  dispatch `slint::platform::Key::F1`, assert `keys-sheet-open`; dispatch
  Escape, assert it is false. `F1` through the table is also the one proof that
  a `NamedKey` binding resolves end to end.
- **`the_listing_has_a_row_for_every_action`**: `w.get_key_rows().row_count()
  == keymap::Action::ALL.len()`, and no row's `what` or `when` is empty. The
  cheap pin for "the list cannot go stale".
- **Manual:** the row still fits at 1100px, and the sheet reads at 700px.

### Sabotage proof

Remove the `key-rows: root.key-rows;` wire at the sheet's instantiation. The
sheet opens empty, `the_listing_has_a_row_for_every_action` **still passes**
(it reads the window's property, not the sheet's) — so add the assertion that
catches it: find the sheet's row container by element id and assert its
`query_descendants()` count. If that proves unreachable, say so in the commit
as the mute plan's task 2 did, and carry it on the manual list: *"open the
sheet, see 29 rows"*. Do not leave the gap unstated.

---

## 5. Rebinding from the sheet

**Files.** `crates/pundit-app/ui/app.slint`, `crates/pundit-app/src/main.rs`,
`crates/pundit-app/src/keymap.rs`, `crates/pundit-app/src/bus/state.rs`,
`CLAUDE.md`, `BACKLOG.md`.

**This goes beyond the spec, which defers it (`D3`, Deferred 1).** The deviation
is deliberate and dated: the coach, 2026-10-07, *"i did ask for a configuration
system eh"*. A hand-edited `state.json` is reassignable only in the sense that a
`.ini` file is; the ask was keys he can reassign. The spec's ordering is kept —
the list ships first, in task 4 — and **G3**'s rebind rule is implemented as the
spec wrote it, which is why nothing here has to be designed.

1. **Capture is a modal layer, not a focused field, and it has to be.** The
   window's shortcuts are on `capture-key-pressed`, dispatched window→focus-item
   **before** the focused element (`i-slint-core-1.18.0/window.rs:1324`–`:1326`),
   so a `FocusScope` inside the sheet would never see a key `handle-key` binds.
   So: `in-out property <string> capturing-action` on the window, tested at the
   **top** of the keys-sheet layer. While it is non-empty the whole keymap is
   inactive and the next key is harvested: Escape cancels, Backspace and Delete
   unbind, anything else is offered to `bind-key`. One mechanism — the modal
   layers — gains one more member.
2. **Each row gains two controls**: *Set…* (writes `capturing-action`) and *✕*
   (unbinds). **Set… replaces the action's bindings with the one key captured**,
   and the row then shows what it became. Replacing is the rule a coach can
   predict; an *Add* affordance for a second binding per action is deferred, and
   the two-key defaults (`←`/`a`, `1`/`Ctrl+0`) keep their pairs until touched.
3. **`Keymap::rebind(action, binding) -> Option<Action>`** — **G3**'s last-wins:
   the binding is removed from whatever action holds it, added to this one, and
   the displaced action is returned so the UI can say which key it just took.
   `Keymap::unbind(action)` empties a row. Both are pure functions on the map.
4. **`AppFiles::set_keymap(&Keymap)` writes `keymap.overrides()`** — the whole
   diff, not one row. **This corrects spec S3's `set_binding(action, Vec<Binding>)`:**
   last-wins changes *two* actions' rows, so a one-row writer would leave the
   displaced action's binding in the file and the next load would resurrect it
   as a collision.
5. **The notice is the status bar's**, naming the action the key was taken from
   — the sheet is not modal over the status bar, and `UserError`-style modals are
   for refusals. Nothing here refuses.
6. **`BACKLOG.md`:** #96 closes; **#35 closes as retired** (**0c**); **#94's
   remaining half closes** (`2`/`3`/`0` are now written down in the app); **#99
   is re-premised** — its stated blocker (*"a focusable grip would swallow
   `z`/`x`/`v`"*) is false, because the global letters are in the capture phase;
   the real blocker is that `handle-key` accepts the **arrows**, and the fix is a
   `resizeSidebar`/`resizeInspector` action with a grip-has-focus guard.
   **#128's keyboard half** gains its home: *select the first candidate* is one
   more row in this table.

### Tests

- `keymap.rs`: `rebind` moves a binding and returns the displaced action; the
  displaced action keeps its *other* bindings; `rebind` to an action that already
  holds the binding is a no-op and returns `None`; `unbind` then `overrides()`
  yields `[]` for that action; `set_keymap` → `keymap()` round-trips through a
  real `state.json` in a `tempfile::tempdir()`.
- `tests/ui/keys.rs`: `capturing_swallows_one_key_and_rebinds_it` — set
  `capturing-action` to `"toggleRecording"`, dispatch `q`, assert `bind-key` was
  called with `("toggleRecording", "q", …)` and that `q` did **not** toggle
  recording; then dispatch `q` again and assert it does.
  `escape_cancels_a_capture` — `capturing-action` is cleared and the old binding
  stands.

### Sabotage proof

1. **Make `rebind` skip the removal from the displaced action.** The round-trip
   test fails on the *next load*: `with_overrides` drops one of them as a
   collision, so the key ends up on whichever action `Action::ALL` reaches first
   — which is the bug a one-row writer causes, and it fails rather than silently
   working.
2. **Move the capture test out of `handle-key`'s top into a `FocusScope` in the
   sheet.** `capturing_swallows_one_key_and_rebinds_it` fails on the first half:
   `q` toggles recording, because the capture phase ran first.

---

## The spec is wrong, or now stale, about seven things

The plan above is right where it differs. Everything here was checked by opening
the file.

1. **`pundit-harness` cannot inject a key, so `T2` cannot live there.** The
   crate table says *"`pundit-harness`: Nothing new, except the key-injection
   fixture T2 needs"*. `crates/pundit-harness/Cargo.toml` has **no `slint`
   dependency** and no file in the crate mentions one — it drives the bus, which
   has no window. Key injection already exists where it belongs,
   `crates/pundit-app/tests/ui/` (`fit_window.rs:202`, `tag_field.rs:72`), and
   every test this plan adds is a module of that one binary.
2. **`T2` is not needed at all, because no code point has to be copied.**
   `slint::platform::Key` is public (`platform.rs:322`) with
   `impl From<Key> for char` (`input.rs:378`), so `NamedKey`'s chars come from
   Slint's own enum. The spec's premise — *"without it `LeftArrow` being
   `\u{F702}` is a number copied out of another crate"* — stops being true, and
   a unit test that every `NamedKey` maps to a distinct char covers what is
   left.
3. **`handle-key` is called for key *releases* too, and the spec's §A never says
   so.** `capture-key-released` at `app.slint:4677` calls
   `handle-key(event, false)`, and the comment at `:4380` gives the reason: a
   slider fires `released` on an arrow key's release. §A2/§A3's census has a
   "Repeat" column and no "released" one, so an implementer following it would
   put the lookup behind `if (pressed)` and break D10 on the release path. This
   plan makes it step 2's first rule and sabotage proof 3.1.
4. **A `when` clause in the table would be a regression**, so "action + binding +
   context" lands as the three-layer rule in **0b** and not as data on the row.
   Every branch's `accept` is **outside** its guard, in all 29 — so a gate that
   suppressed the *match* would hand the key to a focused child. The spec reaches
   the same answer for `D2` (one prose sentence, not a machine-readable guard)
   and gives the weaker of the two reasons.
5. **`S3`'s `set_binding(action, Vec<Binding>)` cannot implement `G3`'s rebind
   rule.** Last-wins changes two actions' rows; a one-row writer leaves the
   displaced binding in the file and the next load reads a collision. Task 5
   writes `overrides()` whole. And the spec does not say the stored map is a
   **diff** from the defaults at all — without that, a coach who rebinds one key
   pins all 29 against every future change.
6. **Two stale citations.** *"`drawing.rs:425`–`:429` is the shape"* for a label
   round-trip: that range is inside `arrow_commands`' assertions; the round-trip
   is `drawing.rs:656`–`:660`, inside a larger test. And *"`lib.rs:7`–`:17`"*
   for the established module pattern is now `:7`–`:19` (13 modules, and
   `pickers`/`video` are `main.rs`'s, not the library's).
7. **§A's line numbers are all +253 to +290**, as the spec predicted and asked
   the plan to re-derive. Task 3 carries the current set. **Two of its counts
   were re-derived and hold:** 61 `event.text ==` inside `handle-key`
   (`:4382`–`:4671`) and 64 in the file, at `2749e00`.

Verified and **true**, so nothing to correct, but worth recording because each
decides something: `Keys::matches` is `pub(crate)` (`input.rs:914`), so
`slint::Keys` cannot be the representation; `KeyBinding` is consulted in
`FocusScope::key_event`, the bubble phase (`items/input_items.rs:715`–`:722`),
so it cannot carry capture-phase shortcuts and `activated()` carries no event;
`process_menubar_shortcuts` runs before the capture phase (`window.rs:1299`), so
Risk 3 stands — and the app has **no `MenuBar` today**, checked.

## Risks

1. **The Slint `if` chain is not exhaustive on `KeyAction`.** Rust's side is —
   `main.rs`'s map is a `match` — but a new action with a default binding and no
   Slint branch is a listed key that does nothing. **Mitigated further than the
   spec assumed:** task 3's `every_default_binding_still_does_what_it_did` is a
   table of every action, so a new one added without a branch fails a test as
   soon as it is added to that table — which the test's own doc comment must
   say to do. Nothing in Slint can check it.
2. **It is a rewrite of the one function every key goes through**, 61
   comparisons into one lookup and a chain. The spec says *"nothing here is
   covered by an automated test"*; after task 3 that is no longer true for 27 of
   the 29 actions, for the arrow-release path and for D10. The residue is
   genuinely manual: that the picture moves.
3. **Task 1 deletes nine properties and three `<=>` wires in one file.** A
   deletion is the one edit that cannot be caught by a dead-code warning in
   `.slint`. The mitigation is step 1.2's grep-per-term, written out in the
   plan so a reviewer can repeat it, and the two tests, which fail if the fold
   is wrong in either direction.
4. **A coach can hand-edit themselves into a keyless app**, by unbinding
   everything. The way back is deleting the `keys` key, which is one line in a
   file they just edited — and after task 5, *Set…* on any row. A guard would be
   machinery for a self-inflicted state with an obvious cure.
5. **`showRecents` ships bindable.** It opens the popover; it cannot navigate
   it, because `show_popup` takes the window's focus item and `handle-key` does
   not run while a popover is up (spec **A5**, verified at
   `window.rs:1386`–`:1394`). That is structural, not an omission.

## What this does not do

- **No format version, no bus change, no `pundit-core` change.** A keymap is
  neither a project nor a timeline, and `CURRENT_FORMAT_VERSION` stays where
  `store.rs:21` has it.
- **No chords or sequences.** One `(key, modifiers)` pair per binding. They need
  a timeout, a pending state and a way to show both, and nothing in the app wants
  them. (This is also what makes `keybinds`' main feature worthless here.)
- **Escape, Tab, Shift+Tab and Return stay reserved** (**G4**), and Home/End stay
  bound to nothing — one row each in the list saying so.
- **It does not open the sheets by key**, add a menu bar (#32), reach inside a
  popover (#85's other half) or give the splitters their arrows (#99) — the last
  is designed in **5.6** and built by #99.
- **It does not chase physical keys** (**0c**), and it does not import Slint's
  `Digit0`–`Digit9` shift table for the AZERTY nicety (spec Deferred 8): one
  rebind buys what a 30-entry table would.
- **It adds no dependency** (**0a**).

## The one open question

**What key should open the Recent popover?** (spec **D4**.) `showRecents` ships
as a row in the table with **no default binding**, which is what closes #85's
deliberate deferral — that spec shipped the popover with no keyboard path *on
the explicit grounds that #96 owns the question*.

- **What it blocks:** one row's default value, and nothing else. The action, the
  row, the listing and the wiring all ship either way.
- **The default if unanswered:** it stays unbound and the coach binds it from the
  sheet in one press once task 5 lands. `ctrl+shift+o` beside `ctrl+o` is the
  conventional reading if an answer comes first.

Four of the spec's five open questions are **settled here rather than asked
again**, because each has an answer the code supplies: `F1` and a sheet are right
(`F1` is free, named and layout-independent; a list read once and dismissed is a
sheet's shape, and the sheet is the app's one modal idiom); the far skip's Shift
was **already answered by the coach on 2026-10-03** and the spec records it; there
is no non-QWERTY coach, so #35 closes as retired and Deferred 8 never happens;
and a hand-edited `state.json` is **not** enough, which is why task 5 exists.

## §R. What the brief got wrong, and what changed

1. **The brief asked whether a smaller change would do** — a map for the
   bindings with the dispatch left inline — and the coach's answer closed it:
   *"i did ask for a configuration system eh. and that is generally a solved
   space; use a library or known pattern/convention."* So the plan is the whole
   feature, and task 5 (which the spec defers) is in it.
2. **"Use a library" was taken literally and searched, not assumed away.**
   `keybinds`, `keymap-rs` and `keyboard-types` were read; **0a** records what
   each buys and the one documented fact that rules out the closest fit. The
   answer is a convention, not a dependency, and the convention is named.
3. **"Context" was asked for as data on the row and is not.** **0b** and
   correction 4 say why: a bound key is swallowed whether or not its guard
   passes, so a `when` clause would change what a gated key does to a focused
   child. What *did* come out of the steer is task 1 — the `text-editing` fold
   stops being a list each new field must join and becomes the platform's own
   property, which is the structural version of what the steer wanted.
4. **The recording allow-list needed no protection**, which the brief assumed it
   would. It is a `!matches!` over `Command` variants in the bus
   (`bus/mod.rs:974`–`:1012`), not over keys; a rebinding cannot reach it, and a
   command added later is still refused by default.

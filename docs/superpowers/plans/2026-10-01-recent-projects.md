# Plan — Recent projects: a popover that switches between them

Spec: `docs/superpowers/specs/2026-10-01-recent-projects-design.md`, revised
through two adversarial passes. **Read it before any task; this plan does not
repeat its reasoning** — it cites section letters (S1, S2, E2, D1, D2, D3, O4,
P1–P5) and moves on. Read `CLAUDE.md` too.

**Revised 2026-10-01 through its own two adversarial passes.** Three steps
**could not be implemented as written** — two of them compiler-proven, not
argued — and a fourth would have silently dropped the migration on the coach's
first switch. **§R** keeps the record. Where this plan and the spec disagree on a
fact, **this plan is right**, and the spec's §R carries the same corrections.

**Five tasks, ordered by their gate.** The pure, in-crate work first
(`cargo test -p pundit-app --lib`, seconds), the bus next, the UI last — so the
expensive build arrives when the logic is already proven.

**Gates.** Every `cargo` call **except `fmt`** goes through
`flock /tmp/claude-1000/cargo.lock nice -n 19 cargo …` — other Claude sessions
build here. **Never pipe clippy or a test run to `tail`**: it masks the exit
status, and a clippy failure has been committed here before. Write to a log and
read `$?` from it. **Compact the conversation before executing** (CLAUDE.md
workflow step 5).

**`pundit-core` is not touched by any task**, and the dependency audit must still
print exactly `serde`, `serde_json`, `thiserror`, `uuid` at close-out.
`metadata::match_label` and `store::read` already exist.

**Before each commit, grep every `path:line` and every quoted sentence in the
diff's doc comments and in the commit message, as one batch.** The spec's draft
shipped seven wrong citations; this plan's draft shipped six more, all of them in
prose rather than in the list that got checked — including one inherited from the
New match plan and repeated without re-deriving.

## Where this stands (update it as tasks land)

- **T1 — `state.json` reads per field (BACKLOG #100).** Not started.
- **T2 — `recents::rows`.** Not started.
- **T3 — the list, the restore, and `open_project`'s busy guard.** Not started.
- **T4 — the popover and the button.** Not started.
- **T5 — close out.** Not started.

---

## The traps

Each is a bug that compiles cleanly, or a step that does not compile at all.

**1. `PopupWindow` CANNOT be a child of a `Button`.** The draft said it could,
*"as this file already attaches `Tooltip`s"*. **False, and proven:** `Tooltip` is
a builtin flagged `@can_be_declared_without_children_slot`
(`i-slint-compiler-1.18.0/builtin_elements.rs:3417`) and is the only element
carrying it; `PopupWindow` is not. And **no style's `Button` has a `@children`
slot** — all five under `i-slint-compiler-1.18.0/widgets/` have zero occurrences.
The draft's shape fails with `'Button' cannot have children`. **T4 item 3 has the
form that compiles.**

**2. The migration must be seeded in `read()`, not in `recent_projects()`.**
Every `AppFiles` setter is `read()` → mutate → `save()`
(`bus/state.rs:181-185`). With the seeding in the *accessor*,
`push_recent_project` mutates `state.recent_projects` — **empty** on a
just-upgraded install — so the coach's previous project is gone from the list on
the very first switch after the upgrade, and because the list is then non-empty
the fallback never fires again. Seeding inside `read` closes it in one place and
removes the second read `push` would otherwise need.

**3. `last_project` is not "never written" unless you say so.** `save`
re-serialises the whole `State` (`bus/state.rs:277-287`) and `State` has no
`skip_serializing`, so every setter rewrites `lastProject` at its pre-upgrade
value, forever. The draft mandated a doc comment saying "never written" — a lie
the next reader would inherit. **`#[serde(skip_serializing)]` is what makes it
true**, and it self-cleans the key out of the file on the first save.

**4. The ticked row must not be clickable, and that needs a step rather than a
sentence.** `OpenProject` on the already-open folder reaches `commit`
unconditionally, which clears `self.history` and runs `clips::empty_trash`
(`clips.rs:325`) on both the outgoing and incoming folders — so a re-open of the
open project permanently destroys the coach's undo stack and the clip recordings
in it. **The `TouchArea` is gated, not just the cursor** (T4 item 2), and T5's
manual list checks it. The draft enumerated everything `RecentChoice` carries and
never said the touch area was conditional.

**5. Dimmed is not disabled, and the only disabled row is the ticked one.** An
implementer will conflate "did not resolve" with "cannot be clicked" — and then
the project on the drive they are about to plug in is the one row that refuses.
**A row that did not resolve stays clickable**: the click sends `OpenProject`,
whose refusal already names the folder and is already modal. #85's own wording
(*"greyed out rather than failing on click"*, `BACKLOG.md:1399`) reads as asking
for a disabled row; greyed **and** clickable is what it should have said.

**6. The inverted restore assertion cannot call `canonicalize()`.** The
assertion T3 inverts sits *after* `std::fs::remove_dir_all(dirs.project())`
(`project_and_sources.rs:193`), and its sibling tests write the positive form as
`Some(dirs.project().canonicalize().unwrap())` (`:109-110`) — which on a removed
path is `ENOENT` and **panics**. The `canonicalize()` cannot simply be dropped
either: `commit` stores the canonical form and `$TMPDIR` may be a symlink.
Capture the canonical path **before** the removal.

**7. `keys.focus()` goes before `show()`.** `show_popup` takes the window's focus
item for every window kind but `ToolTip`
(`i-slint-core-1.18.0/window.rs:2168`), so a half-typed clip name, tag or editor
row commits as a *side effect* of opening the popover. `Export…` and `Basket…`
both call `keys.focus()` first; `Devices…` does not, and T4 copies `Devices…` for
its *idiom* only.

**8. `devices-popup`'s position must not be copied.** It is
`x: parent.width - self.width - 8px`, right-anchored to a **full-width** bar —
correct for `Devices…` at the row's right end, ~1000px wrong for `Recent ▾` at
the left end. Only the `Rectangle`'s chrome transfers.

## What must be proven by sabotage, not asserted

Two. **Each pair must fail *apart*: a change that fails both has broken the
function rather than proven the rule.** Break it on purpose, run both, say what
you saw.

1. **T1 — drop `deserialize_with = "lenient"` from `recent_projects` only**, and
   confirm the **cross-case** test fails (the malformed value costs the whole
   document, so `lastProject` comes back `None`) **while the
   `lastProject`-alone migration test still passes** (an absent key still
   defaults). That pins what is worth pinning: leniency on *this* field is what
   the value-based fallback rests on.
2. **T2 — make the tick positional** (`i == 0`) and confirm the unresolved-head
   test fails while the normal ticked-row test still passes.

**The draft had a third and it was unsatisfiable.** It asked for `default` to be
dropped from one field so the partial-document test failed while the five-shapes
table passed — but every five-shapes document omits several keys, so every choice
of field fails both or neither. Moot now: T1's container `#[serde(default)]`
leaves no absent-key hazard to trap, and
`an_unknown_or_absent_model_reads_as_the_default` (`bus/state.rs:554`) already
pins it for free, its document omitting every field but one.

---

## T1. `state.json` reads per field (BACKLOG #100)

**Files:** `crates/pundit-app/src/bus/state.rs` only. **Gate:**
`flock … cargo test -p pundit-app --lib` for the loop, then
**`flock … cargo test --workspace`** before the commit — the spec's Risk 2 says
this change is gated by the existing suite, and five of the seven harness
`last_project()` assertions are outside this crate's `--lib`. Plus
`flock … cargo clippy --workspace --all-targets -- -D warnings` (to a log, `$?`
checked) and `cargo fmt --all --check`.

**It lands first and alone because the backlog says so, not because the feature
needs it** (spec S2): #100's trigger is *"next time anything is added to
`state.json`"*, and T3 adds something. The feature is safe without it, so the two
are reviewable apart.

1. **`fn lenient<'de, D, T>(d: D) -> Result<T, D::Error>` with
   `T: serde::de::DeserializeOwned + Default`.** Deserialize to
   `serde_json::Value`, try `T::deserialize(&value)`, and on failure log the value
   and return `T::default()`.

   **The spec prints this body with `T: serde::Deserialize<'de>` and it does not
   compile.** `'de` is the *outer* deserializer's lifetime, and
   `&'a Value: Deserializer<'a>` for a strictly shorter local borrow, which
   `Deserialize<'de>` does not supply — E0597, *"argument requires that `value` is
   borrowed for `'de`"*. `DeserializeOwned` is the fix, compiler-checked against
   this repo's own serde 1.0.229.

2. **`#[serde(default)]` on the CONTAINER, `#[serde(deserialize_with = "lenient")]`
   on each field.** `State` already derives `Default` (`bus/state.rs:32`):

   ```rust
   #[derive(Debug, Default, Serialize, Deserialize)]
   #[serde(rename_all = "camelCase", default)]
   struct State { … }
   ```

   **This is a net deletion**: the five existing field-level `#[serde(default)]`s
   go, a new field needs **one** attribute rather than two, and the draft's trap
   about `deserialize_with` without `default` making a missing key a hard error
   disappears rather than needing a trap, a test and a sabotage proof to guard it.
   The hazard is real — compiler-confirmed, `Err("missing field 'pen'")` — which is
   exactly why removing it beats documenting it.

3. **The container `#[serde(default)]` on `WindowSize` and `PanelWidths` stays**,
   and so do both hand-written `Default` impls. They rescue a *partial object* —
   `{"panels":{"sidebar":400}}` keeps 400 — which `lenient` does not, and the
   hand-written defaults are what stop a per-field `unwrap_or_default()`
   resurrecting the zero-height hazard `bus/state.rs`'s own comment warns about.
   Say this in a comment beside the attributes: the next person to add a field
   reads exactly that line.

4. **`read`'s existing `eprintln!` stays, and its scope is narrower than the
   draft claimed.** The draft called a non-object document *"the one case `lenient`
   cannot reach"* — false: a derived struct deserialises from a JSON **array**
   positionally, so `[1,2]` now parses as all-defaults and `lenient` logs twice.
   `read`'s own log still covers `"{not json"`, `"hello"`, `5`, `true`, `null`, a
   truncated file and an empty one. **The test must name one of those shapes**, not
   an array.

**Tests** (in `bus/state.rs`'s own `mod tests`):

- **New: one table-driven test over #100's five measured shapes** —
  `"panels":"wide"`, `{"sidebar":-5}`, `{"sidebar":1.5}`, `"panels":null`,
  `"window":{"height":-1}` (quoted from `BACKLOG.md:1834-1835`, which measured
  them) — each as a document that *also* carries a good `lastProject`, `pen` and
  `whisperModel`, asserting those three survive and only the bad field defaults.
- **Already in the tree, named rather than duplicated:**
  `a_partial_window_or_panels_object_costs_nothing_else` (`:615`) is the
  partial-object pin and is **better** than the draft's proposed version, covering
  `window` as well as `panels`; `corrupt_file_reads_as_none` (`:653`) is the
  non-object pin; `an_unknown_or_absent_model_reads_as_the_default` (`:554`) is the
  absent-key pin.
- **But `:615`'s doc comment must be fixed**, because it states the rule T1
  retires, verbatim: *"`read` throws the whole document away on **any** parse
  error"*. Leaving it keeps a test whose own documentation describes the pre-T1
  world. This is the one edit outside the attributes.

**#100 is not marked resolved here** — T5 does it, once the field it was blocking
has landed.

---

## T2. `recents::rows` (spec D1, D2, D3)

**Files:** `crates/pundit-app/src/recents.rs` (new), `src/lib.rs`. **Gate:**
`cargo test -p pundit-app --lib` is seconds; clippy and fmt as always. **Before
T3**, because it is cheap-gated pure logic and the plan's own principle is that
such work comes first — the draft scheduled it fourth, against its own rule.

**A new `pub mod recents;`** beside `fit`, `match_panel`, `new_match`, `drawing`
and `zoom_input`. The rules live in the app library because `main.rs` is wiring
and has no `#[cfg(test)]` module at all.

```rust
pub struct Row {
    pub path: PathBuf,
    /// Line one, **always populated**: the match's name where the project
    /// resolved, the folder's own file name where it did not.
    pub label: String,
    /// Line two, empty where it would only repeat `label`.
    pub second_line: String,
    /// The project read: `false` dims the row — and **nothing more**, because a
    /// dimmed row is still clickable (trap 5).
    pub resolved: bool,
    /// The open project: ticked, and the one row that is not clickable (trap 4).
    pub open: bool,
}

pub fn rows(paths: &[PathBuf], open: Option<(&Path, &str)>) -> Vec<Row>;
```

**`label` is always populated and `resolved` is its own flag.** The draft made
`label` empty for an unresolved row, which renders a **blank first line with the
folder name underneath** — the opposite of spec D2's *"shows the folder name
alone, dimmed"* — and made `label` carry two meanings, forcing `main.rs` to
re-derive `dimmed` as `label.is_empty()`: a second encoding of one fact, which is
what the New match review pass went out of its way to delete. With `label` always
holding the string to draw, D1's empty-when-equal rule covers the unresolved row
for free, since for it the two strings *are* the same.

1. **`store::read` is called per path, in list order — and skipped for the open
   project's path**, which is the same comparison as item 3's. Without the skip the
   implementation reads all eight and overwrites one label, which costs the eight
   reads spec Risk 1 prices as seven.
2. **The open project's row is labelled from `open`**, not from disk — the
   basket's rule (`bus/basket.rs:236-248`, *the open project from memory, every
   other from `store::read`*), and the only thing that stops the row disagreeing
   with the window title after a failed save.
3. **`open` is matched by path equality, not position** (sabotage proof 2), and it
   is an `Option`: a first launch, a refused `pundit <folder>` run and a launch
   whose restore failed all have a list and no open project.
4. **No injected reader.** Spec §R records why: a stub makes the one valuable test
   impossible, because "the refusal's kind does not matter" needs four real
   `StoreError`s. `new_match::lent_scoreboard` (`new_match.rs:178`) is the
   comparable function and takes a `&Path`.

**Tests**, over a `tempfile::tempdir()`:

- **The base fixture is two matches against the same opponent**, so one test
  carries the ordering and labelling assertions *and* D1's reason for the second
  line (same `match_label`, different folders). The draft had these as two tests
  and the second asserted a `pundit-core` property.
- A project whose name **is** its folder name has an empty `second_line` (D1's
  pre-flow case).
- **Four deliberately broken folders — absent, `{ this is not json`,
  `{"formatVersion": 6}`, `{"formatVersion": 99}` — are four rows with
  `resolved: false`, `label` equal to the folder name and an empty `second_line`**,
  not four dropped rows. `project_and_sources.rs:122-133` has the table idiom.
- The open project's row is labelled from the argument, not from disk: write one
  name to disk, pass another in, assert the argument's wins.
- No open project at all: no row is ticked.
- **A head that does not resolve while a *different* project is open: the head is
  dimmed, unticked and `resolved: false`, and the ticked row is the one whose path
  matches** — sabotage proof 2's target, and the design bug the spec review found.

---

## T3. The list, the restore, and `open_project`'s busy guard

**Files:** `crates/pundit-app/src/bus/{state.rs, project.rs, basket.rs}`,
`crates/pundit-harness/tests/{project_and_sources.rs, export.rs, new_match.rs}`.
**Gate:** the full workspace, clippy to a log with `$?` checked, fmt.

**The busy guard rides here rather than in a task of its own.** The draft gave it
one, citing the New match plan's export-rename split — but that split was for
*"the one change in this plan that silently alters something already on the
coach's disk"*, and a busy guard alters nothing. It is one line in the same file
as everything else here, under the same gate, and its test is the one T3 needs
anyway.

### The stored list (spec S1)

1. **`recent_projects: Vec<PathBuf>`** on `State` with T1's attribute, beside
   `last_project: Option<PathBuf>`, which gains **`#[serde(skip_serializing)]`**
   (trap 3) so it is genuinely a read-only seed and self-cleans out of the file on
   the first save. Its doc says it is a dated transitional read, naming the BACKLOG
   entry T5 files — `state::adopt_old_name`'s pattern. **The named cost:** a
   downgrade to a build predating this one, after any save, gets no restore at
   launch — one folder-pick, which is what `bus/state.rs`'s header already prices
   this whole file at.
2. **The seeding is inside `read`** (trap 2), right after the deserialize:
   `if state.recent_projects.is_empty() { state.recent_projects = state.last_project.clone().into_iter().collect(); }`.
   Then `recent_projects()` is `self.read().recent_projects`, `last_project()` is
   its head, and `push_recent_project` mutates a list that already carries the
   seed. **The fallback is on the empty value, from any cause** — a per-field read
   cannot tell a missing `recentProjects` from a malformed one, so keying on
   absence would lose the pointer for a document holding a good `lastProject` and a
   bad list: #100's symptom reintroduced by its own fix.
3. **`pub fn last_project(&self) -> Option<PathBuf>`** becomes the head of
   `recent_projects()`. Its doc says why it survives as an accessor:
   `restore_last_project` (`bus/project.rs:67`) and the New match flow's W2 tier
   (`main.rs:543`) both want exactly that, **and every existing harness
   `last_project()` assertion keeps passing untouched except the one E2 inverts** —
   `transcribe.rs:585`, `project_and_sources.rs:109`, `:165`, `:396`,
   `new_match.rs:222` pass; `project_and_sources.rs:205` is the inverted one. The
   draft listed `:205` among the untouched *and* inverted it four items later.
4. **`pub fn push_recent_project(&self, folder: &Path)`**: remove any entry equal
   to `folder`, insert at the head, truncate to `RECENT_PROJECTS` (8). Its doc
   carries S4 — `==` is the whole de-duplication because `commit` stores
   `canonicalize().unwrap_or(folder)` (`bus/project.rs:300`), and where
   `canonicalize` failed the list can hold two entries for one project, which costs
   one duplicate row and is not worth normalizing.
5. **`set_last_project` is deleted.** Its four in-crate test callers
   (`bus/state.rs:464`, `:467`, `:518`, `:534`) and `bus/basket.rs:664` move to
   `push_recent_project`. **`remembers_and_forgets` (`:459`) is renamed**: the
   `None` caller at `:467` and its assertion at `:468` go with E2, and a test whose
   name promises behaviour that was deliberately deleted is worse than no test.
   Fold it into the new `push_recent_project` test, which asserts the same first
   two things plus head/move/cap.
6. **`write`'s doc comment gains one clause.** It says *"Fails only for a non-UTF-8
   path, which then simply isn't remembered"* — still true, but with a list the
   failure now costs the *other* setters' fields too, since the whole document is
   serialised together. Self-limiting: the rename never happens, so the next setter
   reads a clean file. One clause, not a rewritten comment.

### The restore, and the guard

7. **`commit` calls `push_recent_project(&folder)`** in place of
   `set_last_project(Some(&folder))` (`bus/project.rs:301`).
8. **`restore_last_project` stops forgetting**: the `set_last_project(None)` at
   `bus/project.rs:75` goes. Its doc changes to carry the **forcing** argument —
   with `last_project()` derived there is no coherent meaning left for the setter —
   and **one consequence the draft was silent on**: `projects_dir_for` tests only
   that the head's *parent* `is_dir()` (`new_match.rs:119`), so a deleted project
   whose parent survives now feeds W2's `BesideLastProject` tier every launch
   rather than one. Nothing to build — W3's editable field with its provenance line
   is the answer, and spec Risk 3 says so — but the doc should say it.
9. **`self.refuse_if_busy()` at the top of `open_project`** (spec O4), as
   `if let Err(e) = self.refuse_if_busy() { return self.emit(Event::Error(e)); }` —
   matching the function's **two** existing early returns of that shape
   (`bus/project.rs:38`, `:40-43`). Today there is no guard, so a running export
   keeps rendering over the project the coach just left while `commit` empties that
   project's trash. The reason is written down at `bus/project.rs:171-172` — an
   inline comment inside **`built_new_match`** (`:162`), not a doc comment on
   `new_match`, which is what the draft said. `restore_last_project` is left
   alone: it runs at launch with nothing running.

**Tests.** In `bus/state.rs`'s `mod tests`:

- **The seeded push, which is trap 2's pin:** a document holding only
  `lastProject`, then one `push_recent_project`, holds **both**, newest first.
  Neither the draft's migration tests (which read without pushing) nor its push
  tests (which start empty) would have caught this.
- The migration, three cases: `lastProject` alone reads as a one-entry list; both
  keys present prefers `recentProjects`; neither is empty.
- **The cross-case** — a malformed `recentProjects` **and** a good `lastProject`
  still returns the last project. Sabotage proof 1's target.
- `push_recent_project`: a new path to the head, an existing one **moves** rather
  than duplicating, nine pushes leave eight with the oldest gone.
- **`skip_serializing` works**: a document with `lastProject`, one save, and
  `lastProject` is gone from the file while `recentProjects` carries it.

In `crates/pundit-harness/tests/project_and_sources.rs` — **no new file**, since
five `last_project()` assertions and the test E2 inverts already live here:

- **Open A, then B, then A, and the list is `[A, B]`** — one test proving that
  `commit` pushes, that the newest is the head, and that a re-open moves rather
  than duplicates. Beside `restore_reopens_the_last_project` (`:171`).
- **The inverted assertion** in
  `restoring_a_folder_that_no_longer_exists_does_not_create_it` (`:187`, assertion
  `:204-208`): the entry is **kept**, and the test is renamed. **Capture
  `let folder = dirs.project().canonicalize().unwrap();` before the
  `remove_dir_all` at `:193`** and assert against `folder` — trap 6, the
  difference between this test passing and panicking.
  (`:396`, in `opening_a_folder_that_does_not_exist_errors_and_creates_nothing`,
  is a different test about a different command and must **not** be touched; it now
  stands as the pin for "a refused open pushes nothing".)

In `crates/pundit-harness/tests/export.rs`, beside `Rig` — **where the export
scaffolding already is**, because `project_and_sources.rs` has none (no
`ExportTarget`/`Resolution`/`Quality`, no `wait_export`, no
clip-with-a-real-recording helper), and the draft priced rebuilding it as "one new
harness test":

- An `OpenProject` while a run is going is refused with
  `CantExport("an export is running")`, **nothing is pushed to the list**, the open
  project is unchanged, and the run reaches its outcome. One export run, one test,
  both halves.

**Changed elsewhere — three assertions in the whole tree**, not two as the draft
claimed: `project_and_sources.rs:204-208` (inverted), `bus/state.rs:468` (deleted
with its setter) and `new_match.rs:222` (reads the list instead of adding a test,
since it already asserts `last_project()` after a `NewMatch`).

**The cap is deliberately not re-tested in the harness.** It is a pure rule pinned
above, and the harness version would cost nine real folders and nine full
`OpenProject` round trips — each unloading, clearing history, emptying two trashes
and republishing — to prove it a second time through an expensive path.

**Before writing anything, grep the harness for `Command::OpenProject` sent after
an `Export` or a preview open**, in case item 9's guard refuses an existing test.

---

## T4. The popover and the button (spec P1–P5, O3)

**Files:** `crates/pundit-app/ui/app.slint`, `crates/pundit-app/src/main.rs`.
**Gate:** the full workspace, clippy to a log with `$?` checked, fmt. The manual
checks are **batched for the coach** at T5.

1. **`export struct RecentRow`** beside the others at the top of `app.slint`
   (there are 19 in the file, 16 before the components start — the draft said
   eleven): `label`, `second-line`, `ticked`, `dimmed`. Plus, on the window,
   `in property <[RecentRow]> recent-rows` and **`in property <bool>
   recents-none-resolved`** — Slint cannot fold over a model to learn that every row
   is dimmed, so P5's second empty line needs the flag computed in Rust
   (`rows.iter().all(|r| !r.resolved)`). The row's *path* does not go to Slint: the
   callback carries the index.
2. **A `RecentChoice` component on `DeviceChoice`'s shape** (`app.slint:398`) —
   the tick column, the label, the hover background — plus the second line, and
   `Palette.alternate-foreground` when `dimmed`. **Its `TouchArea` and its hover
   background are both `if !root.ticked`** (trap 4): clicking the open project's
   row would run `commit`'s `empty_trash` on its own `recordings/.trash`. A
   `RecentList` on `DeviceList`'s shape carries P5's two dimmed lines — one for an
   empty list, one for `recents-none-resolved`, which is E2's own morning and the
   difference between *"my projects are gone"* and *"my drive isn't mounted"*.
3. **The button, wrapped, with the popup as its sibling** — trap 1, and this is
   the form that compiles:

   ```slint
   HorizontalLayout {
       Button {
           text: "Recent ▾";
           enabled: !root.recording && !root.previewing;
           clicked => { keys.focus(); root.list-recents(); recent-popup.show(); }
       }
       recent-popup := PopupWindow {
           x: 0;
           y: -self.height - 8px;
           width: 300px;
           Rectangle { /* devices-popup's chrome: background, border, radius 8px */ }
       }
   }
   ```

   The wrapper holds **nothing but the button**, so `x: 0` anchors where the button
   is. It is safe: `lower_popups` runs before `lower_layouts`, so the popup is
   lifted out and contributes no layout constraint, and both `lower_layout` and
   `optimize_useless_rectangles` special-case `has_popup_child`, so neither the
   single-cell collapse nor the useless-rectangle pass can move the anchor out from
   under it. **A `HorizontalLayout`, not a bare `Rectangle`** — a `Rectangle` in a
   layout takes stretch and would not propagate the button's minimum. **`width:
   300px` explicitly**, as `devices-popup` has: without it the rows size the
   popover to the longest match name.

   **The handler is stated once, whole.** The draft gave it twice with two
   different bodies — `keys.focus(); show();` in one item and
   `refresh-recents(); show();` in another — so an implementer reading the first
   ships a popover that opens empty on first use.
4. **`callback list-recents()`, named after `list-devices`** — which the Devices
   button already does nine lines away:
   `clicked => { root.list-devices(); devices-popup.show(); }` (`app.slint:5424`).
   The draft reasoned its way to the same idiom from first principles about `pure
   callback` return types without noticing the shipped pattern. `main.rs` fills
   `recent-rows` and `recents-none-resolved` synchronously; the popup's component is
   instantiated inside `show_popup`, after the handler's earlier statements have
   run, so the ordering is sound.
5. **`callback open-recent(int)`**, which `main.rs` turns into
   `Command::OpenProject` by **re-reading `machine_state.recent_projects()` and
   `.get(index)`** — not from a cached `Vec`. Every other `AppFiles` accessor
   re-parses the whole file on every call (`pen()`, `window_size()`,
   `panel_widths()`, `last_project()`), so one more read on a click is in keeping
   and removes the only way an index and a list can disagree. The draft proposed an
   `Rc<RefCell<Vec<PathBuf>>>` on `new_match`'s `Draft` precedent, which does not
   transfer: `Draft` holds state with **no other source**, and this list is one line
   away. `.get()` rather than `[]`, so a stale index cannot panic the app.
6. **The rows come from `machine_state` (`main.rs:370`) and the open project from
   `UI`'s `snapshot` (`main.rs:138`)**, whose `Snapshot` carries `Arc<Project>` and
   a canonical `folder` (`bus/mod.rs:419-426`), so the label is
   `metadata::match_label(&s.project)` and costs nothing. **Reading `state.json`
   through `machine_state` is not a new use** — `open_new_match` already does it at
   `main.rs:543`, reached from `wire_new_match` at `:400`. The draft claimed it was
   new, inheriting that mistake from the New match plan; the honest instruction is
   to **widen the comment at `main.rs:367-369`**, which is already narrower than its
   uses.
7. **`!root.previewing` on both `Open Project…` buttons**, not one: the transport
   row's (`app.slint:5310`) and the empty card's `secondary` (`:4933`), whose
   preceding comment reads *"Gated exactly as the toolbar's is"* and which would
   otherwise become false the moment this lands. The card's gap is unreachable — it
   renders only when `!has-project`, and a preview needs a project — but leaving the
   next reader to re-derive that is what the comment exists to prevent.
8. **No keyboard shortcut** (P3): every letter is a global binding, `New match…`
   and `Open Project…` have no key between them, and #96 owns the question.
9. **No Esc cascade, no `editing` fold, no `recent-popover-open` property** (P4).
   `show_popup` takes the window's focus item, so `handle-key` cannot run while the
   popover is up; Escape is handled by `WindowInner` itself and closes the top
   popup, because `PopupClosePolicy`'s default is `CloseOnClick` — which is also
   what gives O3's close-on-click for free. **Adding the `Sheet` machinery here is
   how this task grows into the thing the popover was chosen to avoid.**

**One risk this task carries, stated rather than discovered:** the transport row
already holds **twelve** items and its only slack is the readout's
`min-width: 130px` (`app.slint:5366`), and the comment under it records that *"at
the window's 1100px minimum, sharing the transport's row pushed Export and Devices
off its end and left the notice no width at all."* One ~95px button is far less
than the drawing controls that measurement was about, so this is **expected** to
fit — but expected, not measured. **It is a manual check at the 1100px minimum
(`app.slint:3088`), with the fallback named in advance:** if it overflows, the
three project buttons move to the drawing row, where that comment's own precedent
put the overflow.

---

## T5. Close out

- **The `verify` skill**: fmt, clippy with `-D warnings`,
  `cargo test -p pundit-core`, `cargo test --workspace`, and the core dependency
  audit — which must still print exactly the four crates.
- **Adversarial review on the diff** (the `adversarial-review` skill), then
  commit. **Grep every citation in the diff before it goes out.**
- **`CLAUDE.md`**, in scope rather than a nicety: its `state.json` paragraph states
  the all-or-nothing contract verbatim — *"any `serde_json` error returns the
  defaults for the whole document… which is why each struct in it carries
  `#[serde(default)]` on the **container** — never on the fields"* — and enumerates
  the file's contents. T1 retires that reasoning and T3 lengthens the list. The
  rewrite must say: the read is per field now; **the container defaults stay**,
  because they rescue a *partial object* which the per-field read does not; one bad
  **element** still costs the whole list, which is the basket's own bargain
  (`bus/basket.rs:75-78`); and lost updates between the two `AppFiles` handles are
  **not** retired by any of it.
- **`BACKLOG.md`:**
  - **#100 resolved**, naming the five shapes and what the per-field read does and
    does not cover.
  - **#85 resolved**, recording that the coach offered two readings — "menu or
    similar" and "a project drawer" — and chose the popover, and that the bigger
    half turned out to be storage rather than UI.
  - **#32 "Recents list and a menu bar" (`BACKLOG.md:298`) amended** — its recents
    half is done, and its "when to revisit" (*"when the user works across several
    projects regularly"*) is this feature's premise. Left alone, the next person
    digs the same ground, which is what the backlog exists to stop. The menu bar
    stays open.
  - **#102 unblocked** — its entry says it needs #100's per-field read, which T1
    delivers.
  - **A new entry** for deleting the `last_project` seed field once no installation
    predating this version is left, on #93's dated-shim pattern.
  - **Spec Deferred 5**: deleted clips surviving a project switch, which is a
    feature and not a fix.
- **`CHANGELOG.md`**: an `## [Unreleased]` entry **written for a coach** — one
  button lists the matches you have been in, says what each one *is* rather than
  what its folder is called, and switches with one click; a project whose folder
  has gone greys instead of disappearing, and still opens if you plug the drive
  back in.
- **The batched manual pass, for the coach's eyes:**
  - the popover against the real tree: each row naming the match rather than the
    folder, and two matches against the same opponent tellable apart;
  - **the ticked row does not respond to a click** (trap 4 — the one failure here
    that destroys data);
  - a project whose folder has been renamed greys **and is still clickable**
    (trap 5), and the refusal names the folder;
  - **the transport row still fits at the 1100px window minimum**, and **eight rows
    stand over the player at the 700px minimum** (`app.slint:3089`) — both
    estimates, neither measured;
  - **time the popover opening against the real tree, and record the
    `project.json` sizes.** Spec Risk 1 says "seven few-KB reads… sub-millisecond",
    and `project.json` is not a few KB for a used project: `Clip` carries
    `events: Vec<CommentaryEvent>` — every pen point of every commentary take — and
    `transcript: String`, all of which `store::read` fully deserialises. The answer
    if it is felt is already specced (Deferred 2, resolve on the bus); nothing needs
    building, but the number should stop being quoted as measured;
  - switching between two projects twice and the order following.

**The hand-off to the coach is not part of this plan's close-out.** Three checks
are outstanding from *previous* features — the New match flow on screen, #88's two
inspector pickers and the live self-view, and whether any existing project has its
halves reversed. They belong in the message that hands this over, not in T5, where
they make one feature's close-out read as three.

---

**Non-goals** are spec Deferred 1–6 and X, at the length they are written there;
nothing in this plan reopens any of them.

---

## R. What this plan's first draft got wrong, kept as the record

Two adversarial passes ran in parallel on the draft. **All 21 of its `path:line`
citations were correct** — the batch-grep worked. The defects were in code it
printed, in claims about how Slint and serde behave, and in six smaller citations
made in prose, which the batch never covered.

**Three steps that could not be implemented as written:**

1. **`PopupWindow` as a child of a `Button`** — `'Button' cannot have children`.
   No style's `Button` has a `@children` slot, and `Tooltip` is placeable there
   only because it is flagged `@can_be_declared_without_children_slot`, which
   `PopupWindow` is not. The precedent the draft cited was a compiler special
   case.
2. **The `lenient` body** — E0597. `T: Deserialize<'de>` cannot deserialize from a
   local `&Value`; `DeserializeOwned` can.
3. **Sabotage proof 1** — no choice of field makes its pair fail apart, because
   every five-shapes document omits several keys. Moot once the container default
   removes the hazard.

**One design bug that would have shipped:** `push_recent_project`'s starting list
was unspecified, and the obvious implementation mutates the raw field — so the
coach's previous project vanishes from the list on the **first switch after the
upgrade**, in the feature whose point is going back and forth. Seeding in `read`
closes it.

**Two claims that would have become false comments:** that `last_project` is
"never written" (`save` re-serialises the whole struct), and that a non-object
document is the one case `lenient` cannot reach (a JSON array parses as
all-defaults).

**One internal contradiction:** `project_and_sources.rs:205` was listed among the
assertions that "keep passing untouched" and inverted four items later; and
"exactly two assertions in the whole tree" was three.

**Six citations in prose**, none of them in the list that was checked: eleven
`export struct`s (19 in the file, 16 before the components), `machine_state`
reading state as a "new use" (it already does, `main.rs:543`), `:173` as
`new_match`'s doc comment (it is `built_new_match`'s inline one), "three other
early `emit`s" (two), and `store.rs:98-99` for the NotFound comment (`:99-100`).

**Two tests the draft proposed that already exist** —
`a_partial_window_or_panels_object_costs_nothing_else` and
`corrupt_file_reads_as_none` — one of them better than the version proposed, and
carrying a doc comment that states the rule T1 retires.

**And one step too many:** the busy guard had a task of its own, on a precedent
(the New match plan's export-rename split) that existed because that change
altered files already on the coach's disk. This one alters nothing, so it rides
with T3 and the plan is five tasks rather than six.

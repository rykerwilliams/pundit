# Plan — Recent projects: a popover that switches between them

Spec: `docs/superpowers/specs/2026-10-01-recent-projects-design.md`, revised
2026-10-01 through both adversarial passes (`b4fa0ca`). **Read it before any task;
this plan does not repeat its reasoning** — it cites section letters (S1, S2, E2,
D2, D3, O4, P1) and moves on. Read `CLAUDE.md` too.

**Six tasks, and the order is the gate's.** The pure, in-crate work comes first
(`cargo test -p pundit-app --lib`, seconds), the bus next, the UI last — so the
expensive build arrives when the logic is already proven. Every task gates fully,
and a task that cannot is mis-staged.

**Gates.** Every `cargo` call **except `fmt`** goes through
`flock /tmp/claude-1000/cargo.lock nice -n 19 cargo …` — other Claude sessions
build here. **Never pipe clippy or a test run to `tail`**: it masks the exit
status, and a clippy failure has been committed here before. Write to a log and
read `$?` from it. **Compact the conversation before executing** (CLAUDE.md
workflow step 5): start each task from the spec, this plan and `CLAUDE.md`.

**`pundit-core` is not touched by any task here**, and the dependency audit
(`cargo tree -p pundit-core --edges normal --depth 1`) must still print exactly
`serde`, `serde_json`, `thiserror`, `uuid` at close-out. `metadata::match_label`
and `store::read` already exist; nothing is added to them.

**Before each task's commit, re-derive every `path:line` and every quoted
sentence the commit message or a doc comment makes.** The spec's own draft
carried seven wrong citations, two of them load-bearing, and they were written in
prose rather than measured — which is the failure mode this project has a
standing rule about.

## Where this stands (update it as tasks land)

- **T1 — `state.json` per-field, BACKLOG #100.** Not started.
- **T2 — `open_project`'s busy guard, spec O4.** Not started.
- **T3 — the stored list, and E2.** Not started.
- **T4 — `recents::rows`.** Not started.
- **T5 — the popover and the button.** Not started.
- **T6 — close out.** Not started.

---

## The traps

Each is a bug that compiles cleanly. Everything else is in the spec.

**1. `deserialize_with` is not called for an absent key, so `default` must stay
beside it.** `#[serde(deserialize_with = "lenient")]` alone makes a *missing*
field a hard error — which fails the whole document and leaves #100 **worse than
before the fix**. The attribute is `#[serde(default, deserialize_with = "lenient")]`
on every field. Only a test over a *partial* document catches it.

**2. The migration falls back on the empty VALUE, never on the absent KEY.** A
per-field read cannot tell a missing `recentProjects` from a malformed one — both
arrive as `Vec::default()`. Keying on absence loses the last-project pointer for
a document holding a good `lastProject` and a bad `recentProjects`, which is the
exact symptom #100 exists to stop, reintroduced by its own fix (spec §R, bug 2).

**3. The ticked row is a path match, not position 0.** After a launch whose
restore failed, the head is a project that is **not** open — `commit` was never
reached, and `self.open` is assigned in exactly one place, inside it
(`bus/project.rs:316`). `index == 0` compiles and is wrong (spec §R, bug 1).

**4. Dimmed is not disabled, and the only disabled row is the ticked one.** An
implementer will conflate "did not resolve" with "cannot be clicked" — and then
the one row the coach most needs, the project on the drive they are about to
plug in, is the one that refuses. **A row that did not resolve stays clickable**:
the click sends `OpenProject`, whose refusal already names the folder and is
already modal and already tested. #85's own wording (*"greyed out rather than
failing on click"*) reads as though it asks for disabling; greyed **and**
clickable is what it should have said, and this plan is where that is resolved.

**5. `devices-popup`'s position must not be copied.** It is
`x: parent.width - self.width - 8px`, right-anchored to a **full-width** bar,
which is correct for `Devices…` at the row's right end and ~1000px wrong for
`Recent ▾` at the left end. The `PopupWindow` is declared **as a child of its own
button**, `x: 0; y: -self.height - 8px`. Only the `Rectangle`'s chrome transfers
(spec P1).

**6. `keys.focus()` goes before `show()`.** `show_popup` takes the window's focus
item unconditionally, so a half-typed clip name, tag or editor row commits as a
*side effect* of opening the popover. `Export…` and `Basket…` both call
`keys.focus()` first and say why; `Devices…` does not, and P1 copies `Devices…`
for its chrome only.

Not a trap, but it decides a signature: **`recents::rows` takes the open
project's label as an argument, not a reader.** The open project's row comes from
the `Snapshot` the UI already holds (`main.rs:138`, carrying `Arc<Project>` and a
canonical `folder`, `bus/mod.rs:419-426`) — the basket's own rule
(`bus/basket.rs:236-248`), and the only thing that stops the popover disagreeing
with the window title after a failed save.

## What must be proven by sabotage, not asserted

Three, because two design bugs came out of the spec review and each has a test
whose job is to fail without the fix. **Each pair must fail *apart*: a change that
fails both has broken the function rather than proven the rule.** Break it on
purpose, run both, say what you saw.

1. **T1 — drop `default` from one field's attributes** and confirm the
   partial-document test fails while the five-shapes table still passes.
2. **T3 — key the migration on the absent key** (`contains_key`, or by removing
   the `is_empty` fallback) and confirm the **cross-case** test fails while the
   plain `lastProject`-only migration test still passes.
3. **T4 — make the tick positional** (`i == 0`) and confirm the failed-restore
   row test fails while the normal ticked-row test still passes.

---

## T1. `state.json` reads per field (BACKLOG #100)

**Files:** `crates/pundit-app/src/bus/state.rs` only. **Gate:**
`flock … cargo test -p pundit-app --lib`,
`flock … cargo clippy -p pundit-app --all-targets -- -D warnings` (to a log, `$?`
checked), `cargo fmt --all --check`. Nothing else is touched, so the workspace
stays green.

**It lands first and alone because the backlog says so, not because the feature
needs it** (spec S2): #100's trigger is *"next time anything is added to
`state.json`"*, and T3 adds something. The feature is safe without it — a recents
list is self-healing state worth one folder-pick — so the two are reviewable
apart, which is the whole point of separating them.

1. **`fn lenient<'de, D, T>(d: D) -> Result<T, D::Error>`**, the body in spec S2:
   deserialize to `serde_json::Value`, try `T::deserialize(&value)`, and on
   failure log the value and return `T::default()`. Its doc says what it buys and
   what it does not (the three bullets under S2).
2. **`#[serde(default, deserialize_with = "lenient")]` on every field of
   `State`** — `last_project`, `whisper_model`, `pen`, `window`, `panels`. **Trap
   1**: the `default` is not optional.
3. **The container `#[serde(default)]` on `WindowSize` and `PanelWidths` stays**,
   and so do both hand-written `Default` impls. They rescue a *partial object*,
   which is finer-grained than `lenient` and a different job; and the hand-written
   defaults are what stop a per-field `unwrap_or_default()` resurrecting the
   zero-height hazard `bus/state.rs`'s own comment warns about. Say this in a
   comment where the attributes are, because the next person to add a field will
   read exactly that line.
4. **`read`'s existing `eprintln!`** stays for a document that is not an object at
   all, which is the one case `lenient` cannot reach.

**Tests** (in `bus/state.rs`'s own `mod tests`, where the file's tests already
live):

- **One table-driven test over #100's five measured shapes** —
  `"panels":"wide"`, `{"sidebar":-5}`, `{"sidebar":1.5}`, `"panels":null`,
  `"window":{"height":-1}` — writing each as a document that *also* carries a
  good `lastProject`, a pen and a whisper model, and asserting those three
  survive and only the bad field defaults. **Quote the five from #100 rather than
  inventing them**: they are measured, and four of the five take the same path
  here, which is why one loop is the honest shape rather than five tests.
- **A partial `panels` object keeps its good field** (`{"panels":{"sidebar":400}}`
  → 400), which is **trap 1's** pin and sabotage proof 1's target.
- A document that is not an object at all still returns the defaults and logs.

**Close-out for the task:** BACKLOG #100 is **not** marked resolved here — T6
does it, once the field it was blocking has actually landed.

---

## T2. `open_project` refuses while busy (spec O4)

**Files:** `crates/pundit-app/src/bus/project.rs`, `crates/pundit-app/ui/app.slint`,
and one new harness test. **Gate:** the full workspace —
`flock … cargo test --workspace`,
`flock … cargo clippy --workspace --all-targets -- -D warnings` (to a log, `$?`
checked), `cargo fmt --all --check`.

**Its own task and its own commit, because it is a behaviour change to a shipped
command** — the same reason the New match plan split the export rename out. It is
*caused* by this feature (one-click switching is what makes the gap reachable)
and it is independent of everything else here.

1. **`self.refuse_if_busy()?` at the top of `open_project`**, matching
   `new_match` (`bus/project.rs:173`), whose doc already carries the reason: *"An
   export or a preview must not have the project swapped underneath it."* Today
   `open_project` has no guard, so a running export keeps rendering over the
   project the coach just left — and `commit` runs `clips::empty_trash`
   (`clips.rs:325`) on that project's `recordings/.trash` while it does.
   `open_project` returns `()`, so this is a small restructure: the body moves
   into a `Result`-returning helper, or the call becomes
   `if let Err(e) = self.refuse_if_busy() { return self.emit(Event::Error(e)); }`
   — **the second**, because the function has three other early `emit`s in that
   shape already and a helper would be a shape nothing else in the file uses.
2. **`restore_last_project` is left alone.** It runs once at launch with nothing
   running, and a guard there would be a refusal nobody can see.
3. **`!root.previewing` on `Open Project…` and on the new `Recent ▾`** (T5 adds
   the second), matching `Export…` and `Basket…`. `New match…` and
   `Add Source Video…` are **not** in scope: `NewMatch` already refuses in the
   bus, and `AddSource` is a different question with its own answer.

**Test** (`pundit-harness/tests/project_and_sources.rs`, beside the other open
refusals): an `OpenProject` while an export runs is refused with
`CantExport("an export is running")`, the open project is unchanged, and the run
reaches its outcome. `tests/export.rs`'s `Rig` is the shape to copy; the
new-match suite's `new_match_during_an_export_is_refused_and_the_run_finishes` is
the closest existing test and is worth reading first.

**Watch for:** an existing test that opens a project while a preview or export is
live and now gets refused. Grep the harness for `OpenProject` sent after an
`Export` or a preview open before writing anything.

---

## T3. The stored list, and the failed restore (spec S1, E2)

**Files:** `crates/pundit-app/src/bus/state.rs`,
`crates/pundit-app/src/bus/project.rs`, `crates/pundit-app/src/bus/basket.rs`
(one test line), `crates/pundit-harness/tests/{project_and_sources.rs,
new_match.rs}`, and a new `crates/pundit-harness/tests/recents.rs`. **Gate:** the
full workspace, as T2.

1. **`recent_projects: Vec<PathBuf>`** on `State`, with T1's attributes, beside a
   `last_project: Option<PathBuf>` that **stays as a read-only seed** — never
   written, with a doc comment saying so and naming the backlog entry that deletes
   it (T6 files it, on `state::adopt_old_name`'s dated-shim pattern).
2. **`pub fn recent_projects(&self) -> Vec<PathBuf>`**: the stored list, **or the
   seed when that list is empty** (**trap 2**). One `if list.is_empty()`.
3. **`pub fn last_project(&self) -> Option<PathBuf>`** becomes
   `self.recent_projects().into_iter().next()`. Its doc says it is the head of the
   list and why it survives as an accessor: `restore_last_project`
   (`bus/project.rs:67`) and the New match flow's W2 tier (`main.rs:543`) both
   want exactly that, and **every existing `last_project()` assertion in the tree
   keeps passing untouched** — `transcribe.rs:585`,
   `project_and_sources.rs:109`, `:165`, `:205`, `:396`, `new_match.rs:222`. That
   is the evidence for "one field, not two" and it belongs in the doc.
4. **`pub fn push_recent_project(&self, folder: &Path)`**: remove any entry equal
   to `folder`, insert at the head, truncate to `RECENT_PROJECTS` (8). Its doc
   carries S4 — `==` is the whole de-duplication because `commit` stores
   `canonicalize().unwrap_or(folder)` (`bus/project.rs:300`), and where
   `canonicalize` failed the list can hold two entries for one project, which
   costs one duplicate row and is not worth normalizing.
5. **`set_last_project` is deleted.** Its four in-crate test callers
   (`bus/state.rs:464`, `:467`, `:518`, `:534`) and the one in `bus/basket.rs:664`
   move to `push_recent_project`. The `:467` caller passes `None` and has no
   equivalent — that test's assertion about forgetting goes with E2.
6. **`commit` calls `push_recent_project(&folder)`** in place of
   `set_last_project(Some(&folder))` (`bus/project.rs:301`).
7. **`restore_last_project` stops forgetting**: the `set_last_project(None)` at
   `bus/project.rs:75` goes, and its doc comment changes — a failed restore still
   leaves the UI in its no-project state and still logs, and the entry stays so a
   drive that is not mounted yet costs a grey row rather than a lost project
   (spec E2). **The forcing argument goes in the doc**: with `last_project()`
   derived, there is no coherent meaning left for the setter.
8. **`write`'s doc comment changes subject.** It says *"Fails only for a non-UTF-8
   path, which then simply isn't remembered"*; with a list, **any** entry being
   non-UTF-8 fails the whole document, so that open is not remembered and nor is
   anything else that write would have carried. Self-limiting — the next setter
   re-reads a clean file — and worth stating rather than discovering.

**Tests.** In `bus/state.rs`'s `mod tests`:

- `push_recent_project`: a new path goes to the head; an existing one **moves**
  rather than duplicating; nine pushes leave eight with the oldest gone.
- The migration, three cases: `lastProject` alone reads as a one-entry list; both
  keys present prefers `recentProjects`; neither is empty.
- **The cross-case, which is sabotage proof 2's target:** a document with a
  malformed `recentProjects` **and** a good `lastProject` still returns the last
  project. This is the test the spec's draft could not have had, and it is the
  reason the fallback is on the value.

In `pundit-harness/tests/recents.rs` (new):

- **Open A, then B, then A, and the list is `[A, B]`** — one test, because it
  proves at once that `commit` pushes, that the newest is the head, and that a
  re-open moves rather than duplicates. Read back through
  `AppFiles::in_config_dir`, as `project_and_sources.rs:109` does.
- **A failed restore keeps the entry** — the one behaviour change, so it gets the
  test that would have failed before.
- **An open refused while an export runs pushes nothing** (T2's guard, seen from
  this side).
- **The cap is deliberately NOT re-tested here.** It is a pure rule already pinned
  in the unit tests, and the harness version would cost nine real folders and nine
  full `OpenProject` round trips — each unloading, clearing history, emptying two
  trashes and republishing — to prove it a second time through an expensive path.

Changed elsewhere, and **exactly two assertions in the whole tree**:

- `project_and_sources.rs:204-208`, in
  `restoring_a_folder_that_no_longer_exists_does_not_create_it` (`:187`) — the
  assertion reading *"a folder that can't be restored is forgotten"* becomes its
  opposite, and the test's name changes with it. **This is the only existing test
  whose behaviour changes.** (`:396`, in
  `opening_a_folder_that_does_not_exist_errors_and_creates_nothing`, is a
  different test about a different command and must **not** be touched — the spec's
  draft cited it by mistake and it now stands as the pin for "a refused open
  pushes nothing".)
- `new_match.rs:222` reads the list rather than adding a test, since it already
  asserts `last_project()` after a `NewMatch`.

---

## T4. `recents::rows` (spec D1, D2, D3)

**Files:** `crates/pundit-app/src/recents.rs` (new), `src/lib.rs`. **Gate:** as
T1 — `cargo test -p pundit-app --lib` is seconds and nothing else is touched.

**A new `pub mod recents;` in `src/lib.rs`**, beside `fit`, `match_panel`,
`new_match`, `drawing` and `zoom_input`. The rules live in the app library because
`main.rs` is wiring and has no `#[cfg(test)]` module at all.

```rust
pub struct Row {
    pub path: PathBuf,
    /// `metadata::match_label`'s name, or **empty when the project did not
    /// resolve** — which is what dims the row, and nothing more (trap 4).
    pub label: String,
    /// The folder's own file name, or empty where it would just repeat `label`.
    pub second_line: String,
    /// The open project: ticked, and the one row that is not clickable.
    pub open: bool,
}

pub fn rows(paths: &[PathBuf], open: Option<(&Path, &str)>) -> Vec<Row>;
```

1. **`store::read` is called directly**, per path, in list order. **No injected
   reader** — the spec's own §R records why the draft's version was wrong: a stub
   makes the one valuable test impossible, because "the refusal's kind does not
   matter" needs four real `StoreError`s and not four `None`s from a mock. The
   directly comparable function in the tree, `new_match::lent_scoreboard`, takes a
   `&Path` and calls the real reader.
2. **The open project's row is labelled from `open`, not from disk** — the
   basket's rule (`bus/basket.rs:236-248`, *the open project from memory, every
   other from `store::read`*), which is also the only thing that stops the row
   disagreeing with the window title after a failed save.
3. **`open` is matched by path equality, not by position** (**trap 3**), and it is
   an `Option`: a first launch, a refused `pundit <folder>` run and a launch whose
   restore failed all have a list and no open project.
4. **`second_line` is empty when it equals `label`**, computed here rather than
   compared in Slint, so it is testable (spec D1: a pre-flow project's name *is*
   its folder name, which is the one case where the second line is pure noise).

**Tests**, over a `tempfile::tempdir()` (a dev-dependency of `pundit-app`
already):

- Rows in list order, with `match_label`'s name and the folder's file name; and
  the second line **empty** for a project whose name is its folder name.
- **Four deliberately broken folders — absent, `{ this is not json`,
  `{"formatVersion": 6}`, `{"formatVersion": 99}` — are four rows with an empty
  label and `open: false`**, not four dropped rows. Build them as
  `project_and_sources.rs:122-133`'s table does.
- The open project's row is labelled from the argument and **not** from disk:
  write one name to disk, pass another in, assert the argument's wins.
- **No open project at all: no row is ticked.**
- **A head that does not resolve, with a *different* project open: the head is
  dimmed and unticked, and the ticked row is the one whose path matches** — this
  is sabotage proof 3's target and it is the design bug the spec review found.
- Two matches against the same opponent come back with the same label and
  different second lines, which is what D1's second line is for.

---

## T5. The popover and the button (spec P1–P5, O3)

**Files:** `crates/pundit-app/ui/app.slint`, `crates/pundit-app/src/main.rs`.
**Gate:** the full workspace, clippy to a log with `$?` checked, fmt. The manual
checks are **batched for the coach** at T6.

1. **`export struct RecentRow`** beside the other eleven at the top of
   `app.slint` — `label`, `second-line`, `ticked`, `dimmed` — and
   `in property <[RecentRow]> recent-rows` on the window. The row's *path* does
   not go to Slint: the callback carries the index, as the source and clip rows
   do, and `main.rs` holds the paths.
2. **A `RecentChoice` component on `DeviceChoice`'s shape** (`app.slint:398`) —
   the tick column, the label, the hover background — plus the second line, and
   `Palette.alternate-foreground` when `dimmed`. A `RecentList` on `DeviceList`'s
   shape carries the two empty lines (**P5**): one for an empty list, one for a
   list where **nothing resolved**, which is E2's own morning and the difference
   between *"my projects are gone"* and *"my drive isn't mounted"*.
3. **A `Recent ▾` button in the transport row, immediately right of
   `Open Project…`**, `enabled: !root.recording && !root.previewing` (T2 adds the
   second to `Open Project…` as well). Its handler is
   `keys.focus(); recent-popup.show();` — **trap 6**, in that order.
4. **`recent-popup := PopupWindow` as a child of that button** with `x: 0;
   y: -self.height - 8px` (**trap 5**), carrying `devices-popup`'s `Rectangle`
   chrome only: `Palette.background`, the house border and an 8px radius.
5. **`callback open-recent(int)`**, which `main.rs` turns into
   `Command::OpenProject(paths[index])`. The popover closes on the click, which
   Slint's default close policy already gives (**O3**) — there is nothing typed
   into it to lose, and the bus's refusals are modals that would draw over it.
6. **`main.rs` fills the rows when the popover opens**, not on a tick: a
   `pure callback`'s return value cannot carry a list, so the button's handler
   calls `root.refresh-recents()` before `show()`, and `main.rs` sets
   `recent-rows` and caches the paths in an `Rc<RefCell<Vec<PathBuf>>>` beside
   it — the shape `new_match`'s `Draft` uses, and for the same reason
   (`to_string_lossy` would corrupt a path that is not UTF-8).
   - The list comes from `machine_state` (`main.rs:370`). **That handle exists so
     the window's geometry is written through it** — reading recents through it is
     a **new use**, not its stated purpose, and the comment there should say so.
   - The open project comes from `UI`'s `snapshot` (`main.rs:138`), whose
     `Snapshot` carries `Arc<Project>` and a canonical `folder`
     (`bus/mod.rs:419-426`), so the label is `metadata::match_label(&s.project)`
     and costs nothing.
7. **No keyboard shortcut** (**P3**): every letter is a global binding,
   `New match…` and `Open Project…` have no key between them, and #96 owns the
   question.
8. **No Esc cascade and no `editing` fold** (**P4**): `show_popup` takes the
   window's focus item, so `handle-key` never runs while the popover is up and
   Escape falls through to Slint's own close. **Do not add a
   `recent-popover-open` property or a `handle-key` branch** — there is no state
   to clear, and adding the `Sheet` machinery here is how this task grows into the
   thing the popover was chosen to avoid.

**One risk this task carries, stated rather than discovered:** the transport row
already holds **twelve** items and its only slack is the readout's
`min-width: 130px`, and the comment under it records that *"at the window's 1100px
minimum, sharing the transport's row pushed Export and Devices off its end and
left the notice no width at all."* One ~95px button is far less than the drawing
controls that measurement was about, so this is **expected** to fit — but it is
expected, not measured. **It is a manual check at the 1100px minimum
(`app.slint:3088`), and the fallback is named in advance:** if it overflows, the
three project buttons move to the drawing row, which is where that comment's own
precedent put the overflow. Do not discover this at T6 and improvise.

---

## T6. Close out

- **The `verify` skill**: fmt, clippy with `-D warnings`,
  `cargo test -p pundit-core`, `cargo test --workspace`, and the core dependency
  audit — which must still print exactly the four crates, since nothing here was
  core's.
- **Adversarial review on the diff** (the `adversarial-review` skill), then
  commit. **Re-derive every citation in the diff's doc comments before it goes
  out** — the spec's draft carried seven wrong ones and that is the standing
  failure mode on this project.
- **`CLAUDE.md`**, and this is in scope rather than a nicety: its `state.json`
  paragraph states the all-or-nothing contract verbatim — *"any `serde_json` error
  returns the defaults for the whole document… which is why each struct in it
  carries `#[serde(default)]` on the **container** — never on the fields"* — and
  enumerates the file's contents. T1 retires that reasoning and T3 lengthens that
  list. The rewritten paragraph must say: the read is per field now; **the
  container defaults stay**, because they rescue a *partial object* which the
  per-field read does not; one bad *element* still costs the whole list, which is
  the basket's own bargain; and lost updates between the two `AppFiles` handles
  are **not** retired by any of it. A stale rule about which serde attribute goes
  where is the kind that produces a wrong field-level default on an `f64` six
  months from now.
- **`BACKLOG.md`:** #100 **resolved** (naming the five shapes it measured and what
  the per-field read does and does not cover); #85 **resolved**, recording that
  the coach offered two readings — "menu or similar" and "a project drawer" — and
  chose the popover, and that the bigger half turned out to be storage rather than
  UI. **A new entry** for deleting the `last_project` seed field once no
  installation predating this version is left, on #93's dated-shim pattern. And
  **Deferred 5** from the spec: deleted clips surviving a project switch, which is
  a feature and not a fix.
- **`CHANGELOG.md`**: an `## [Unreleased]` entry **written for a coach** — one
  button lists the matches you have been in, says what each one is rather than
  what its folder is called, and switches with one click; a project whose folder
  has gone greys instead of disappearing.
- **The batched manual pass, for the coach's eyes:**
  - the popover against the real tree: each row naming the match rather than the
    folder, and two matches against the same opponent tellable apart;
  - **the transport row still fitting at the 1100px window minimum** (T5's risk),
    and **eight rows standing over the player at the 700px minimum** — both
    estimates in the spec, neither measured;
  - switching between two projects twice and the order following;
  - renaming a project's folder and its row greying rather than vanishing — and
    **still being clickable** (trap 4), which is the behaviour #85's own wording
    would have got wrong.
- **Carried over, and still unlooked-at by anyone** — these are now two features
  deep and should go in front of the coach with this one:
  - **the whole New match flow on screen**: both real trees, ⇄ Swap rewriting the
    folder name live, typing in the folder field stopping it, Esc leaving a field
    before closing the sheet, and the empty card's new wording;
  - **#88's two inspector pickers and the live self-view**;
  - **whether any existing project has its halves reversed** (the copy-suffix sort
    fix repairs nothing already made; the symptom is a match clock out by about a
    half, and the correction is dragging the sources in the sidebar).

---

**Non-goals** are spec Deferred 1–6 and X, at the length they are written there;
nothing in this plan reopens any of them.

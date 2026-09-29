# Plan — New match: one sheet that makes the folder and names the project

Spec: `docs/superpowers/specs/2026-09-24-new-match-flow-design.md`, revised
2026-09-29 through both adversarial passes (`17c3a0a`). **Read it before any task;
this plan does not repeat its reasoning** — it cites section letters (E1, I5, W2, N2,
C2, S6, §R) and moves on. Read `CLAUDE.md` too.

**Five tasks. Every task gates fully, and a task that cannot is mis-staged.** Every
change is additive, and the two signature changes are small and in-crate:
`metadata::match_name` has exactly two callers (`metadata.rs:117`, `:196`), and
`Pick::ProjectFolder` gaining a field is two edits, the construction (`main.rs:445`)
and the match arm (`pickers.rs:78`). The order is the pure functions with their tests
first, the command next, the UI last, so the expensive build arrives when the logic is
already proven.

**Gates.** Every `cargo` call **except `fmt`** goes through
`flock /tmp/claude-1000/cargo.lock nice -n 19 cargo …` — other Claude sessions build
here. **Never pipe clippy or a test run to `tail`**: it masks the exit status, and a
clippy failure has been committed here before. Write to a log and read `$?` from it.
`pundit-core` is seconds and needs no GStreamer; anything reaching `pundit-media` pays
the ~3-minute whisper build, once for the plan. **Compact the conversation before
executing** (CLAUDE.md workflow step 5): start each task from the spec, this plan and
`CLAUDE.md`.

## Where this stands (update it as tasks land)

- **T1 — core naming and the aspect primitive.** Not started.
- **T2 — the shipped-code touches.** Not started.
- **T3 — `Command::NewMatch` and the harness proof.** Not started.
- **T4 — `new_match.rs`, the sheet and the wiring.** Not started.
- **T5 — close out.** Not started.

---

## The two traps

Each is a bug that compiles cleanly. Everything else is in the spec.

**1. An *empty* project folder is adopted, not refused; the refusal is keyed on
`project.json`.** `create_dir(project_dir)` returning `AlreadyExists` is **not** a
refusal on its own (spec C2 step 4, §R). Refuse iff
`project_dir.join(store::PROJECT_FILENAME)` exists — `OpenProject` has this exact
shape already (`bus/project.rs:45-57`). **Getting it backwards recreates the bug the
feature exists to fix:** the stranded empty folder the spec opens by observing.

**2. `create_dir`, never `create_dir_all`, and the parent must exist.** Two calls: the
projects folder (leaf only, and only if missing) and the project folder (spec W3, C2
steps 3–4). `create_dir_all` compiles, reads as friendlier, and turns a typo in a
hand-editable path into a tree.

Not a trap, but it decides a signature: **the `verify` skill's dependency audit reports
green on a `SystemTime::now()` in core**, because a clock adds no dependency.
`parse_date_in`'s `max_year` argument is the only thing standing between core and a
clock.

## What must be proven by sabotage, not asserted

**The adoption pair** (T3). Make `create_dir`'s `AlreadyExists` a refusal and confirm
the adoption test fails **while the `project.json`-refusal test still passes**. The two
must fail *apart*: a change that fails both has broken the command rather than proven
the rule. Break it on purpose, run both, say what you saw.

---

## T1. Core: the naming vocabulary and the aspect primitive

**Files:** `crates/pundit-core/src/naming.rs` (tests inline, in the `mod tests` at
`naming.rs:65`, as `order_videos`' are), `src/metadata.rs`, `src/project.rs`.

**Gate:** `flock … cargo test -p pundit-core`,
`flock … cargo clippy -p pundit-core --all-targets -- -D warnings`,
`cargo fmt --all --check`. Seconds, no GStreamer, workspace stays green. The dependency
audit must still print exactly `serde`, `serde_json`, `thiserror`, `uuid`: **no regex
crate** (the date shapes are a byte scan) and **no unicode crate** (spec N2).

**The tests are spec Testing's `core::naming` list — follow it there**, not a second
copy here; `tempfile` is already a dev-dependency (`pundit-core/Cargo.toml:22`) and
nothing here needs it. `naming.rs`'s module doc reads *"today, only the order a game's
halves go in"*; widen it to what a name means and what a name may contain.

1. **`pub fn safe_chars(&str) -> String`** — replace each of `/ \ : * ? " < > |` and
   every control character with `-`. **Nothing else**: no collapsing, no trimming, no
   truncation, no case change (T2). Its doc carries `bus/export.rs:192-194`'s reasoning
   forward and adds the seven the spec found missing, naming exFAT, NTFS and SMB.
2. **`pub fn truncate_on_boundary(&str, bytes: usize) -> &str`** — the last char
   boundary at or below `bytes`, replacing the hand-rolled `char_indices` walk at
   `bus/basket.rs:553-559`, its second caller at a different budget (200 there, 64
   here).
3. **`pub fn folder_slug(&str) -> String`** — spec N2's steps, truncate before the
   trim. Three properties are easy to write down wrongly:
   - **A name made only of the characters this function removes yields empty; other
     punctuation is kept.** `"..."` and `"///"` come back empty, `"!!!???"` comes back
     `"!!!"` — `?` is replaced, `!` is not. Stripping the rest needs the Unicode
     category table N2 forbids in the adjacent sentence, so "entirely punctuation
     yields empty" is a test to delete, not a bug to fix.
   - **Spec Testing's byte-64 case asserts a `.` or a `-`, never a space**: a space is
     unbuildable, since step 3 turns every whitespace run into `-` first.
   - **The final whitespace trim is therefore unreachable.** Keep it — one
     `trim_matches` pattern, and it makes the rule read whole — and say in the doc that
     step 3 is what makes it belt-and-braces.
4. **`pub fn parse_date_in(text: &str, max_year: i32) -> Option<CalendarDate>`** (spec
   I2, whose two traps the doc names). **The year bound is an argument, neither a
   constant nor a clock:** core has no clock (`metadata.rs:63-77` passes `CalendarDate`
   in for the same reason), and one parameter is cheaper than a second parse function
   or a century constant embedded in core — which is what the spec's review decided.
5. **`pub fn opponent_from(folder_name: &str) -> Option<String>`** — spec I4's rule,
   `None` when what is left is empty.
6. **`metadata::match_name` becomes pure** (spec N4):
   `pub fn match_name(home: &str, away: &str) -> Option<String>`, `None` when either
   side is blank after trimming. The `&Project` form stays as a private two-line
   wrapper — **this plan names it `project_match_name`; the spec does not** — keeping
   the two call sites one line each. `match_label`'s different fallback chain
   (`metadata.rs:195-202`) must not be conflated with it.
7. **`pub fn aspects_match(existing: f64, candidate: f64) -> bool`** in `project.rs`,
   the pairwise rule lifted out of `Project::check_aspect` (`pundit-core/src/project.rs:614-635`): both
   `> 0.0` and `(existing - candidate).abs() / existing.max(candidate) < 0.005`. **A
   `bool`, not a `Result`** — `AspectMismatch` carries both numbers, which the caller
   passed in. `check_aspect` keeps its signature and its whole doc (the 0.5% reasoning,
   the relink `excluding`, the NaN note) and becomes: find the reference source, call
   `aspects_match`, build `AspectMismatch` on false.

   **Why it has to exist:** `check_aspect` gates against the *stored* first source and
   returns `Ok(())` when there is none — and at probe time there is no `Project` and
   cannot be, since a `SourceRef` needs `relative_path`, which needs a canonical folder
   the command has not created yet. A probe loop written against `check_aspect`
   therefore gates **nothing at all** (T3 item 2), and an executor who notices instead
   hand-codes the tolerance in the bus: a second definition of the rule.

---

## T2. The shipped-code touches

**Files:** `crates/pundit-app/src/bus/{export.rs, basket.rs, sources.rs,
scoreboard.rs}`. **Gate:** the full workspace — `flock … cargo test --workspace`,
`flock … cargo clippy --workspace --all-targets -- -D warnings` (to a log, `$?`
checked), `cargo fmt --all --check`. Nothing is red at any point.

**Two commits**, gated each time: (a) the export rename, (b) the other three. The
export basename is the one change in this plan that silently alters something already
on the coach's disk, so it is reviewable on its own.

**Commit (a) — `bus/export.rs`'s `file_name` (`:195-198`) calls `naming::safe_chars`
and nothing else.** Spec N2 has the whole argument against `folder_slug`: orphaned
`.srt` and `.chapters.txt` sidecars, and long labels found distinct by `de_duplicate`
(`:510`, called at `:334`) and then colliding as files past byte 64.

Three new cases go in the existing `#[cfg(test)] mod tests` (`:812`): a label containing
`--`, one ending in `.` or a space, one over 64 bytes. They pin the collapse, trim and
truncate axes, which the shipped
`a_file_name_joins_the_label_and_the_project_without_path_characters` (`:817-823`) does
not. **That shipped test is already a partial guard, not a passes-either-way one** —
`folder_slug("All clips")` is `all-clips`, so both of its assertions fail on the
capitals and spaces alone. N2's "would have passed anyway" is about the *draft's
hypothetical* test; nothing about the shipped one is to be "fixed".

One behaviour *does* change and is meant to: `safe_chars` replaces `\`, which today
survives. On Linux a `\` in a clip name is a legal file name and writes a file an SMB
share cannot hold. Say that in `safe_chars`' doc.

**Commit (b) — the three that make the command honest rather than duplicated:**

1. **`bus/basket.rs`'s `file_stem` (`:549-565`) calls `naming::truncate_on_boundary`**
   in place of its `char_indices` walk (`:553-559`). **Its character set and its
   200-byte budget do not change** — N2 shares the mechanism and deliberately not the
   budgets. The asymmetries that leaves are T5's review queue's.
2. **`probed_source` splits** (`bus/sources.rs:211-237`). `relative_path` needs a
   canonical project folder, which does not exist until the command's step 4, so the
   probe cannot stay welded to the `SourceRef` construction.
   `source_ref(folder: &Path, path: &Path, probe: Probe) -> Result<SourceRef, UserError>`
   is the second half (canonicalize, `relative_path`, display name, struct);
   `probed_source` becomes the probe, the `check_aspect` at `:218` and a call to it.
   **`source_ref` is `pub(super)`**, not private as `probed_source` is, because
   `bus/project.rs` calls it. `add_source` (`:30`) and relink are unchanged.
3. **`pub(super) fn storable(config: &ScoreboardConfig) -> Result<(), &'static str>`**,
   lifted from `set_scoreboard`'s first two lines (`bus/scoreboard.rs:191-194`) and
   carrying `:188-190`'s reasoning: the one place that decides whether a scoreboard can
   be **stored**, which `NewMatch` would otherwise walk around by writing
   `scoreboard: Some(config)` onto a fresh `Project` (spec C2, §R).

   **It returns the reason, not a `UserError`: severity is not its to choose.**
   `set_scoreboard` wraps it exactly as today, `self.refuse(reason.into())` (`:207-209`,
   a `UserError::Scoreboard` notice); `new_match` wraps the same string in a **modal**
   one (T3 item 2). **Returning `UserError::Scoreboard` would make the refusal
   invisible:** that variant is in `is_notice` (`bus/mod.rs:564-574`), `main.rs`
   dispatches on the variant and not on the emitter (`:2727` against `:2739`), and a
   notice goes to the status line, which renders *behind* the scrim — as
   `main.rs:2730-2733` says out loud of the match editor's own line. Create would
   re-enable, nothing would be created, and the sheet would look unchanged.

---

## T3. `Command::NewMatch`, and the harness proof

**Files:** `crates/pundit-app/src/bus/{mod.rs, project.rs}`, and a new
`crates/pundit-harness/tests/new_match.rs`. **Gate:** the full workspace, as T2. Green
throughout: a new `Command` variant and a new `pub(super) fn` break nothing.

1. **The variant**, beside `OpenProject` (`bus/mod.rs:79`), carrying spec C1's four
   fields verbatim — **one path, not a `projects_dir` plus a `folder` the bus would
   rejoin**. Its doc comment is the spec's, including the note that nothing here is
   read from the pipeline, so the caller-captured-timestamp rule has nothing to bite
   on. The arm goes in `command`'s `match` (`bus/mod.rs:935`) and the variant is **left
   off the recording allow-list** (`:899-930`), which is deny-by-default, so a UI bug
   reaches a silent refusal rather than a half-built project mid-take.
2. **`bus/project.rs::new_match`**, in spec C2's order, and **nothing touches the disk
   until every video has been accepted**: `refuse_if_busy()` (`bus/export.rs:367`), the
   argument checks (including `storable(&scoreboard)`), **the probes, each gated
   against probe 0 through `project::aspects_match`**, then the two `create_dir`s,
   `Project::new(name)` (`pundit-core/src/project.rs:431-444`) with
   `scoreboard: Some(config)` and each `SourceRef` from T2's `source_ref`,
   `store::write` (`store.rs:175-206`) and `commit` (`bus/project.rs:93-126`),
   unchanged. Four things about it:

   - **The pre-disk gate is `aspects_match`, and spec C2 step 6's per-push
     `check_aspect` goes with it.** Gating the pushes runs the same rule over the same
     numbers a second time, *after* both `create_dir`s — so the refusal C1 calls
     decisive, "no folder holding one of a game's two halves", would fire too late and
     leave an empty folder behind, failing the harness test named for it. One rule, one
     place, and "nothing on disk until every video is accepted" becomes true rather
     than asserted.
   - **The modal channel is `UserError::Io(String)`** — `#[error("{0}")]`, a bare
     message, not in `is_notice`. It is already how `open_project` refuses a folder that
     is not there (`bus/project.rs:39-44`), which is likewise not an `io::Error`; the
     blank-team-name and `project.json` refusals go the same way.
   - **`project_dir` existing as a regular file is guarded**, with `folder.is_dir()` on
     the `AlreadyExists` branch, as `OpenProject` guards it (`bus/project.rs:39`).
     Without it `create_dir` returns `AlreadyExists`, the `project.json` probe finds
     nothing, and `store::write` fails later with a confusing `ENOTDIR`. One `is_dir` on
     a branch already reading the filesystem.
   - **No rollback** (spec C3, §R). The draft's version was two `remove_dir`s,
     `recordings/` and then the folder, and it was **unreliable rather than
     impossible** — which is worth stating precisely, because two reviews of this got
     it wrong in opposite directions. `store::write` creates `recordings/` first
     (`store.rs:176`) and writes `.project.json.tmp` before renaming it into place
     (`:202`). So a failure while **serializing** leaves only `recordings/`, which the
     rollback removes, and it works; a failure in the temp **write or rename** —
     `ENOSPC`, `EIO`, a dropped mount, the realistic ones — leaves the temp file
     behind, and the second `remove_dir` returns `ENOTEMPTY` with the error ignored.
     A rollback that works for the cheap failures and not the expensive ones is worse
     than none, and none is wanted anyway: a failed write leaves an empty folder the
     next Create adopts. **"Nothing created" has one honest exception:** a
     `create_dir(projects_dir)` that succeeds while `create_dir(project_dir)` fails
     leaves an empty projects folder, which is the directory W2's third tier would have
     proposed anyway. **No new undo action**: creating a project is not an undo step, as
     opening one is not, and `commit` clears the history (`bus/project.rs:104-106`).
3. **`tests/new_match.rs`** — **the cases are spec Testing's harness list.** What the
   spec does not say is where the fixtures are: `Dirs`
   (`project_and_sources.rs:18-49`, giving `<tmp>/config`, `<tmp>/project`,
   `<tmp>/media`, so stored paths climb out of the project folder as real ones do),
   `fixtures::webm` (`pundit-media/src/fixtures.rs:76`), `Harness::new(&config)`, and
   `ReadOnly` (`pundit-harness/src/lib.rs:652`), which reports whether the read-only
   mode was actually enforced. `tempfile` is already a dev-dependency
   (`pundit-harness/Cargo.toml:20`). Two assertion shapes are this plan's rather than
   the spec's: **"nothing created" is asserted by listing the projects folder**, never
   by testing one path; and the refused-`project.json` case asserts the first project's
   file **byte-identical** afterwards.

---

## T4. The module, the sheet and the two entry points

**Files:** `crates/pundit-app/src/new_match.rs` (new), `src/lib.rs`, `src/main.rs`,
`src/pickers.rs`, `ui/app.slint`. **Gate:** the full workspace, clippy to a log with
`$?` checked, fmt. The manual checks are **batched for the user** at T5.

**The module lands with its consumer.** A draft put `new_match.rs` in T2, whose own
header says it does the things that touch shipped code — and a module whose only
consumer is the sheet two tasks later would have had its shape settled by a subagent
that never has to use it (item 13 is that problem surfacing).

### The module: the rules, with the environment injected

A new `pub mod new_match;` in `src/lib.rs`, beside `fit`, `match_panel`, `drawing` and
`zoom_input`. **The rules live in the app library, not in `main.rs`**, which is wiring
and has no `#[cfg(test)]` module at all (verified: zero matches). **And the environment
is passed in**, as `bus/state.rs`'s `films_dir(videos, home)` (`:312`) and
`config_dir(xdg, home)` (`:301`) take theirs (`:305-311` says why). **The tests are
spec Testing's `pundit-app::new_match` list**, plus the two additions below.

1. **`projects_dir_for(match_folder: &Path, home: Option<&Path>, last_project: Option<&Path>) -> ProjectsDir`**,
   reaching the filesystem only through `is_dir` (spec W1, W2). `ProjectsDir` is an enum
   of three variants carrying the path — W1's hit, `last_project`'s parent, and
   `<match folder's parent>/<APP_NAME>` — because the spec names the type and not its
   shape, so the caller maps tier to provenance line in one place and a test names the
   tier rather than comparing paths. Only the third carries a line (spec S1).

   **The name is `core::metadata::APP_NAME` (`metadata.rs:30`), never the literal
   `"pundit"`** — `bus/state.rs:26`'s `APP_DIR` is the same string but `pub(super)`,
   invisible from here. **W1's "at most four `is_dir` calls" means four candidates:
   depth 0 is the match folder itself**, so the walk is the folder, its parent, its
   parent's parent and one more, which is what W1's depth table agrees with.
2. **`seed_scoreboard(projects_dir: &Path, home: &str, away: &str, max_year: i32) -> ScoreboardConfig`**
   (spec I5). `max_year` is threaded here because the ordering needs
   `naming::parse_date_in`. Two things the spec leaves open or contradicts itself on:
   - **Undated folders sort last, by name descending, and nothing is stat-ed at all.**
     I5 says both "no `stat` at all" and "falls back to its own mtime"; the first wins
     and its sentence is corrected. The cost is one degraded prefill in a folder where
     no project carries a date, which Risk 5 has already priced. Add a test: an undated
     folder sorts below every dated one.
   - **A missing or unreadable projects folder gives `match_panel::blank_config()`
     (`match_panel.rs:317-335`), not an error** — W2's third tier proposes a directory
     that does not exist yet, and `blank_config` is what the setup sheet already opens
     with on a project with no scoreboard (`main.rs:1436-1450`). Add a test for it.
3. **One prefill entry point** assembling what the sheet opens with: the match folder
   (spec I1), the date (I2's four tiers), the opponent (I4), the projects dir and the
   seeded config. **The guessed opponent prefills the AWAY slot**, the coach's club
   Home — the coach's decision, recorded in spec S3. The mtime tier reads a file, so it
   lives here and not in core. **No test reads the coach's folders and no test writes
   outside a `tempfile::tempdir()`**.

**`probe` is not called anywhere in this module** (spec S4): it blocks for up to ten
seconds per file (`pundit-media/src/probe.rs:15-16,41`) and this runs on the UI thread.
The bus probes, once, during Create.

### The sheet and the entry points

4. **`Pickers::open_many(&self, window: &AppWindow, pick: Pick, then: impl FnOnce(Vec<PathBuf>))`**,
   sharing `open`'s body: called back **once**, on the UI thread, with the ordered set,
   and **not at all on cancel** — the contract `open` already documents
   (`pickers.rs:53-56`). `open` keeps `impl FnMut(PathBuf)` and its four callers are
   untouched, which is what spec E4's objection was protecting. **A second method rather
   than "this flow's caller collects the paths":** `open` calls `then` once per path and
   nothing fires afterwards (`pickers.rs:104-106`), so there is no defined moment at
   which the set is complete, and relying on the callbacks draining inside one
   `spawn_local` task is accidental correctness. **No `Timer::single_shot`**: this
   repository does not build on incidental ordering. `Pick::ProjectFolder` also gains a
   `title: &'static str` (W3) — the construction (`main.rs:445`, keeping today's `"Open
   Project Folder"`) and the match arm (`pickers.rs:78`).
5. **`EmptyCard` gains an optional `secondary` string and `secondary-clicked()`**
   (`app.slint:340-383`), a plain (non-`primary`) `Button` beside the first in the
   existing `HorizontalLayout` (`:373-381`). The no-project card (`:4604-4609`) becomes
   **primary "New match…"**, **secondary "Open Project…"**; the other two (`:4610`,
   `:4616`) pass no `secondary` and are unchanged. **Its `message` (`:4606`) now
   describes the demoted button** — propose a wording, do not silently keep this one; it
   is the first thing a launch shows, so it goes in T5's manual list.
6. **A "New match…" button in the transport row**, immediately left of
   `Open Project…` (`app.slint:4972-4976`), gated **exactly** as that one is:
   `enabled: !root.recording` (`:4974`). No new rule (spec E1).
7. **`NewMatchSheet inherits Sheet`** (spec S1) — `card-width: 560px`, title
   `"New match"`, and **not** a `Rectangle` wrapper: `MatchSetupSheet`'s exception
   (`:2228`) exists only because a `ColorPicker` (`:2068`) hangs over the card as an
   absolutely-positioned sibling, and this sheet has none. **Update `Sheet`'s doc
   comment, which says there are five of these and names them (`:1524-1527`)** — six
   now. Rows per spec S1's table; every field is a `SetupField` (`:1997`) with no
   swatch; **nothing is extracted from `MatchSetupSheet` and it is not touched** (S2).
8. **The load-bearing sites `BasketSheet` has and this sheet needs too**, none optional:
   - `in-out property <bool> new-match-sheet-open` on the window (basket's `:3253`);
   - the sheet's exported `in-out property <bool> editing` (basket's `:1762`) and a
     window `property <bool> new-match-editing`, bound
     `editing <=> root.new-match-editing` in the `Scrim` (basket's `:5371`) and folded
     into `text-editing` (`:3386-3387`);
   - **a `close_new_match()` function on `close-basket`'s shape (`:3612-3623`)**, the
     most important of these: its comment says it clears `editing` **by hand** because
     the sheet that set it is about to be destroyed, and a field that had focus would
     otherwise leave the window's shortcuts switched off **permanently**. It clears the
     message too and ends in `keys.focus()`, and every close path goes through it —
     Cancel, Esc, `ProjectOpened`;
   - a `creating` flag, behind "Create disables on the click" (item 14);
   - the close on `ProjectOpened` (`main.rs:2562`'s arm) **and on nothing else** (C5).
9. **Its `Scrim` goes between `app.slint:5434`'s block and `:5489`'s.** Z-order is file
   order in Slint (spec S1), and an error raised by Create has to draw over the sheet
   that raised it (spec C4). Today: `:5338` export, `:5360` basket, `:5395` match setup,
   `:5434` match editor, `:5489` the error dialog.
10. **Its `handle-key` branch takes `BasketSheet`'s shape exactly** (`:3664-3670`) — Esc
    closes only when `!root.text-editing`, and the `reject` is what delivers `Ctrl+V`
    into the path field — placed **before** the setup sheet's branch at `:3679`. Without
    item 8's fold the guard is always true and the cascade is dead code; the comment at
    `:3658-3663` says this of `basket-editing` in as many words, and **spec S6 records
    the draft getting it backwards**, asserting both an unconditional Esc and the fold.

### The derivations — imperative, because a binding cannot do it

11. **The folder-name field cannot be driven by a pure callback**, and `SetupField`'s own
    doc says why: its text is two-way bound all the way out to the window
    (`app.slint:1995-1996`, the `LineEdit` at `:2052`) and *"a `text:` binding breaks the
    moment the user types into it"*. Same class as #88's B3, in the one task this plan
    calls wiring. So:
    - **The derivation is imperative**, on the pattern `app.slint:3011-3017` already uses
      for `saved-project-name`: `changed home-name`, `changed away-name` and the ⇄
      handler each write `root.new-match-folder-name = root.new-match-folder(…)`, guarded
      by `!root.new-match-folder-dirty`.
    - **`SetupField` gains an `edited()` callback**, forwarded from its `LineEdit`.
      Additive, and `MatchSetupSheet` passes nothing, so S2's "the setup sheet is not
      touched" still holds.
    - **The dirty flag is set only by the folder field's own `edited`** — never by a
      `changed new-match-folder-name` handler, which would self-trigger when the
      derivation writes the field and so kill the derivation on the first keystroke of a
      team name. That is the subtle half.
    - **`new-match-folder(date, home, away)` slugs the *joined* `<date>-<home>-<away>`
      string**, not each part separately: the `-` collapse then absorbs a blank team name
      or an absent date, and the 64-byte cut applies to the whole name. Backed by
      `naming::folder_slug`, with the date a separate `in property <string>` (empty when
      I2 found none) because **there is no date field** (spec N1). It feeds the field
      until the coach types in it and then stops — one dirty flag, never cleared while
      the sheet is open.
    - **The validators stay pure callbacks**, on `on_valid_periods`' pattern
      (`main.rs:1236-1241`): `valid-folder-name`, `valid-projects-dir`, and the target
      check — **each the same call Create will make** (*"a field can't read good and then
      fail to save"*, `main.rs:1237-1238`). The path line is
      `<projects folder>/<folder name>/`, concatenated in Slint. **⇄ Swap exchanges the
      two names**, and both follow from them (spec S3).
12. **`on_new_match`** opens `Pick::Videos` through `open_many`, hands the ordered set to
    item 3's prefill and seeds the sheet. `last_project` comes from **`machine_state`** —
    `main.rs:369`'s second `AppFiles` handle, which exists precisely so the UI thread can
    read this file (`main.rs:343`, `:369`); `home` from `std::env::var_os("HOME")`, as
    `AppFiles::default_location` reads it (`bus/state.rs:141`); `max_year` from
    `glib::DateTime::now_local().year() + 1` (`use gstreamer::glib`, as
    `bus/export.rs:34` has it).
13. **`Choose…` re-runs `seed_scoreboard` for the folder it just picked.** The seed is
    the *chosen* folder's (spec Q6, "the unit is the chosen folder"), but it is captured
    once at prefill and is invisible in the sheet, so without this a coach who picks a
    different projects folder silently keeps the old folder's kit. One `read_dir` plus
    one small `store::read`: the cost the sheet already pays on open.
14. **`on_create_match`** sends one `Command::NewMatch` and sets `creating`. The sheet
    closes on `ProjectOpened` and on nothing else; **`Event::Error` re-enables Create,
    wired in *both* arms** (`main.rs:2727` and `:2739`). Spec S5 wants any error arriving
    while the sheet is open to re-enable it, and the guarded arm at `:2727` takes **every**
    notice before `:2739` sees it — so wiring one leaves Create permanently dead after
    any notice that lands while the sheet is up.

**One risk this task carries, stated rather than discovered:** S5's target check is *"a
`try_exists` at prefill and on every edit"*, so a Slint binding `stat`s a path on every
keystroke, on a cloud-sync mount. The spec accepts it — *"a courtesy, not the
guarantee"*, and the guarantee is the bus's (C2), which cannot race. **If it is felt,
drop the courtesy rather than debounce it**; one `stat` of one path is the cheapest
thing in this sheet and a timer would be the fix being worse than the problem.

---

## T5. Close out

- **The `verify` skill**: fmt, clippy with `-D warnings`, `cargo test -p pundit-core`,
  `cargo test --workspace`, and the core dependency audit.
- **Adversarial review on the diff** (the `adversarial-review` skill), then commit. Two
  items are **queued for it by name**, neither decided here: `bus/basket.rs` still
  replaces only `['/', ':']` though the exFAT/NTFS/SMB argument applies to a film's name
  too, and it trims trailing whitespace after its cut (`basket.rs:560`) but not a
  trailing `.`.
- **The batched manual pass, for the user's eyes** (spec's Testing section): New match…
  against both real trees, confirming the projects folder, the date in the folder name,
  the opponent and the video order before pressing Create; ⇄ Swap rewriting the folder
  name and the path line live, and typing in the folder field stopping it; Esc leaving a
  field first and closing the sheet second; **and the empty card's new message**.
- **`CLAUDE.md`**: fold the flow into the project-lifecycle material — it is a convention
  now. What belongs there and nowhere else: that a project folder is created by **one**
  command and never by a sequence, and why (the aspect gate fires between sources, so a
  sequence can leave a named folder holding one of a game's two halves); that an empty
  folder is **adopted** and the refusal is keyed on `project.json`; that `core::naming`
  holds a name's rules, `safe_chars` for a file name and `folder_slug` for a folder, and
  **why they are not one pipeline**; that the pairwise aspect rule is
  `project::aspects_match` and `check_aspect` is the stored-project wrapper over it; that
  the projects folder is found by walking up for `APP_NAME`; and that `parse_date_in`
  takes its year bound because core has no clock.
- **`BACKLOG.md`: give this flow entry #108, resolved** (107 is the highest today). **It
  had no entry of its own, which is why it was invisible for five days** while the spec
  aged against ~115 commits — that is the thing worth recording, alongside the six
  defects §R keeps and the one bug that shipped ahead of the spec and is already fixed
  (`order_videos`, `df253ef`). Note that **#85's recents list is next and shares
  `last_project` with W2**, so neither needs a `state.json` key of its own; and record
  Deferred 3, now cheap (parse the export's date out of the folder name with
  `naming::parse_date_in`).
- **`CHANGELOG.md`**: an `## [Unreleased]` entry **written for a coach** — what changes in
  their hands, not the mechanism. It is one button that makes the folder, names the
  project from the two teams, finds the `pundit` folder beside the videos, guesses the
  date and the opponent, and leaves the footage exactly where it is.

---

**Non-goals** are spec Deferred 1–6, X, N3, N5 and Q3, at the length they are written
there; nothing in this plan reopens any of them.

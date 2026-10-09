# Plan — the export queue across projects (#77)

Spec: `docs/superpowers/specs/2026-10-07-export-queue-design.md`. Read it and
`CLAUDE.md` first; neither is repeated here.

**The spec is complete and has no open question.** The coach settled the only
one on 2026-10-07 (*"yes you should be able to keep working"*), §Q10 records what
the answer cost, and §R is the spec's own correction record. So this plan adds no
design — it is the order, the file lists, the tests and the sabotage proofs.

**This is the reviewed second draft.** §B records what the adversarial pass
changed, including five mistakes in the first draft's own "I re-derived
everything" section. Read §B before §A: one of the things it removes is a task.

## Where this stands (update it as tasks land)

- **1 — the coach keeps working while a run is going.** **Done**
  (`claude/77-task-1`). `refuse_if_previewing` beside `refuse_if_busy`,
  `UserError::CantOpen` as a unit variant, all five comment sites rewritten, the
  `ProjectOpened` clear conditional, and the notice's folder clause gone.
  `a_project_open_is_refused_while_a_run_is_going` **replaced** by
  `a_project_opens_while_a_run_is_going`, plus
  `a_project_open_is_refused_while_a_preview_is_open` which did not exist: 14
  harness export tests to 15.
  **A third test inverted, and neither review found it:**
  `new_match.rs`'s `new_match_during_an_export_is_refused_and_the_run_finishes`
  asserted the same `CantExport("an export is running")` from
  `built_new_match`'s gate. It is now
  `new_match_during_an_export_goes_through_and_the_run_finishes`: two of its
  three assertions invert (the folder *is* created, the open project *does*
  change) and the third stands unchanged and is the point — the run is not
  disturbed. **Both reviews checked that every test the plan *named* exists;
  neither searched for tests the plan did not name.** The grep that would have
  found it is `"an export is running"` across `crates/`, and task 2 should run
  the equivalent for `refuse_if_busy` before it moves anything.
  **One correction found in execution:** the first draft of the change had
  `refuse_if_busy` *call* `refuse_if_previewing`, which would have reported
  *"can't open that project"* to a coach who pressed Export. One condition, two
  refusals, each naming what it refused.
  **Sabotage:** the export clause put back into `refuse_if_previewing` fails
  `a_project_opens_while_a_run_is_going` alone, with
  `a_project_open_is_refused_while_a_preview_is_open` still green — which is
  what says the two clauses are pinned apart rather than together.
- **2 — the queue on `Bus`: four commands, one view, and the clip-delete rule.**
  Not started.
- **3 — the Queue section in the export sheet.** Not started.

**Shipping: three PRs, in order, none parallel.** 1 and 2 both edit
`bus/export.rs`; 1 and 3 both edit `main.rs`. Task 1 is a feature on its own —
the coach asked for it — and ships first because it is the one piece whose risk
is behavioural rather than structural (Risk 2).

**Re-derive `app.slint`'s line numbers at task 3, not from this plan.** Four
queued features (#102, #84, #115, #128) land in that file, and task 3's section
goes into the middle of `ExportSheet`.

---

## §A. The citations this plan rests on, re-derived at `8d63aa3`

The spec was written against `origin/claude/78-task-1` (merged as #61); #96's
five tasks and #134 have landed since. **Five of the spec's citations have
drifted:**

| Spec says | Actually |
|---|---|
| `start_run`'s `refuse_if_busy` at `export.rs:339` | **`:335`**; `:339` is the `targets.is_empty()` check |
| `ClipMedia::recording` built at `export.rs:753` | **`:838`–`:851`**, in `entry_media`; `:753` is `source_date`'s doc |
| `TRASH_DIRNAME` at `clips.rs:321` | the constant is **`:28`**, `trash_clip`'s join **`:265`**, `empty_trash`'s **`:321`**, `trash_path`'s **`:331`** |
| `app.slint:1835` names the six `Sheet`s | **`:1896`**, and it says **seven**, the seventh being **Keys** (#96 task 4, landed after the spec). The spec's six already counted the error dialog, so a queue sheet would be the **eighth** |
| nothing — but worth knowing | **`app.slint:1906` still says "two of the six sheets"**, six lines under the comment that says seven. The file contradicts itself; task 3 is in that component and fixes the line |

**Checked rather than trusted, and sound:** `refuse_if_busy` has exactly five
callers dividing 3 render / 2 project as §Q2 says (`export.rs:335`,
`basket.rs:422`; `project.rs:40`, `:96`, `:203` — `project.rs:174` is a doc link,
not a caller); its comment really carries the false clause *"Both composite on
the UI's GL context"* (`:391`); `start_run`'s order is `refuse_if_busy` → labels
→ `de_duplicate` → jobs → `create_dir_all` → `begin` → write-back, with nothing
between the job loop and `create_dir_all`; the recording guard is an allow-list
(`mod.rs:970`–`:1002`, `Export` absent) so four new commands need no edit;
`is_notice` is six variants (`:602`); `clear_run` has two callers
(`main.rs:915`, `:1177`); the `ProjectOpened` clear is `main.rs:3075`–`:3076`;
the end-of-run notice is `:3363`; `ExportSheet` carries **seven**
`enabled: !root.exporting` (`:2019` the targets list, `:2033`/`:2042`/`:2055`
the three pickers, `:2089` **Chapters**, `:2099` **Scoreboard subtitles**,
`:2111` **Mute source audio**) plus the Export/Cancel swap at `:2165`/`:2177`;
the Run list is `min(140px, root.run.length * 28px)` (`:2120`); `entry_media`
`stat`s rather than reading `Bus::missing`; and every test named below exists.

---

## §B. What the adversarial pass changed, and the five things §A had wrong

Both reviewers read the source. Fifteen findings each; the overlap was five, and
I verified every claim I applied against the tree rather than taking either
report's word for it. **Three findings removed work, four added it, and five were
mistakes in §A itself.**

**§A's own errors, all five found by the reviewers:**

1. **The seventh sheet is Keys, not the error dialog.** §A said *"(the error
   dialog counts)"*; the spec's six already included it. So task 3's headline is
   "no **eighth** sheet", not seventh.
2. **The three checkboxes were mapped in the wrong order** — `:2089` is
   Chapters, `:2099` Scoreboard subtitles, `:2111` Mute. §A wrote *"(mute,
   chapters, cues)"*, and the step instructs an editor **by line number**.
3. **`clips.rs:321` is `empty_trash`'s join, not `trash_path`'s** (`:331`).
4. **`ClipMedia::recording` starts at `:838`, not `:839`** (`:839` is the
   `.join` continuation).
5. **§A missed `two_targets_with_the_same_name_write_two_files`**
   (`harness/tests/export.rs:213`) while listing five citations it had checked —
   which left two sabotage proofs asking a question the tree already answers, and
   is the evidence base for the duplicate-refusal bug below.

**Work removed:**

- **Task 1 of the first draft is gone as a task.** It extracted `jobs` and
  shipped it paired with the project-open change, on the stated ground that *"2
  is the behaviour it makes room for"* — **which is false**: task 2 never calls
  `jobs`, and its only consumer is the enqueue. The split is now step 1 of the
  queue task, where its caller is. That also deleted its two sabotage proofs
  (both dead — see below) and a "no new tests" section.
- **`create_dir_all` folds into `jobs`**, which drops a tuple return that existed
  only to let two call sites repeat one line — and makes §R.5's rule
  (*"created at enqueue, never at Start"*) **structural**: Start calls `begin`
  over drained jobs and never `jobs`, so the order is no longer a step anyone can
  regress. One sabotage proof became unwritable, which is better than the test it
  was going to ask for.
- **`bus/queue.rs` is not a new file.** The basket earns its own file by *not*
  sharing export's job builder; the queue is a second entry point into it.
  A sibling module would force `Pickers` (private, `export.rs:592`) and `jobs`
  to widen to `pub(super)` and would need the picker write-back shared or
  copied. The queue goes in `export.rs`.
- **Harness test (d) is folded into (a).** A queue run **is** a run — Start is
  the same `begin` (`:413`) producing the same `Active`, and `open_project` reads
  nothing but `self.export.is_some()` — so (d) could observe nothing task 1's
  own test and (a) do not. A spec deviation, recorded here with its reason.
- **The duplicate-key bus unit test is gone.** It would assert
  `PathBuf == PathBuf`, and there is no `ExportJob` fixture anywhere in
  `export.rs`'s `mod tests` — building one twice to test `==` buys nothing the
  end-to-end notice test does not.
- **The 520px card widening is gone** *as a width change for the row's sake*.
  The queue row is the Run row's shape, and those rows already carry
  `"{target} — {match_label}"` at 480px with `overflow: elide`. The width is
  re-opened in task 3 for a **different and real** reason: the action row gains a
  fourth button.
- **Five numbered "steps" that were facts rather than work** moved into prose.

**Work added:**

- **The sheet had no way to enqueue.** The first draft specified the Queue
  *section* and its `Start queue`, and never an **Add to queue** control, its
  `ExportSheet` callback or `main.rs`'s handler. And it could not be recovered by
  analogy at task time: the section is *absent at zero rows*, so the control
  cannot live in it — with an empty queue there would be nothing to click and
  `Command::EnqueueExport` would be unreachable. This was the biggest gap in the
  draft.
- **The extraction does not compile as described.** `start_run`'s `refused`
  closure (`:332`) is used only inside the range being moved, so leaving it
  behind is an unused binding and `-D warnings` fails. *"Byte for byte what it
  does today"* hid it.
- **Enqueue needs an `ExportChoices` → `Pickers` conversion that does not
  exist as a function.** It is inline in `Bus::export` (`:310`–`:324`) and
  carries **the one negation CLAUDE.md names as living in exactly one place**.
  `jobs` therefore takes `ExportChoices`, and the conversion becomes
  `impl From<ExportChoices> for Pickers`.
- **`Rig` is single-project** (`harness/tests/export.rs:44`–`:120`): one project
  at `<tmp>/project`, one `exports`, one `clips`. The cross-project test needs a
  second, which is the only real test-infrastructure work here and is now a named
  step.

**Two sabotage proofs could not fail, and one behaviour had no test:**

- **"Leave the `ProjectOpened` clear unconditional"** was to be caught by a
  **harness** test. `pundit-harness` is `lib.rs`/`score.rs`/`truth.rs` with no
  Slint at all, and `tests/ui/` links the **lib** while `clear_run`,
  `show_export` and the `ProjectOpened` arm are all in `main.rs` — which
  CLAUDE.md already states is the one thing a UI test cannot reach. Its own
  fallback (*"it owes a `get_export_run()` row count"*) was impossible for the
  same reason. Replaced by an honest gap in Risks, and by extracting the part
  that has content (below).
- **"Call `refuse_if_busy` from enqueue"** was to be caught by the
  refusals test — in which nothing is running, so the added call returns `Ok` and
  the test passes. The draft even noticed that nothing covers enqueue-during-a-run
  and then did not fix it. **Enqueue while a run is going is §Q8's one documented
  divergence from the basket and §Q11's stated behaviour**, and it is now two
  lines on the end of test (a).
- **Task 1's two proofs asked questions the tree answers.** `de_duplicate` is
  pinned by `two_targets_with_the_same_name_write_two_files`; and
  `refuse_if_busy`-first is pinned by **nothing** —
  `a_refused_run_leaves_the_pickers_alone` starts no run at all, so moving the
  call cannot change it. The draft named that test as the pin, which is simply
  wrong. Accepted as a gap: the only observable difference is which of two
  refusal messages wins.

**And one bug in the design the draft called "coherent":**

- **The duplicate refusal would falsely refuse a *different* target.**
  `de_duplicate` is per call, so a clip named after a tag — the exact case
  `two_targets_with_the_same_name_write_two_files` exists for — enqueued in two
  clicks produces the same unsuffixed label and so the same `job.path`, and the
  second is refused *"already in the queue"* **for a target that is not in it**,
  with no way out (there is no edit, §Q11). Whether both can be queued depends on
  click order. §A called the mechanism coherent; the mechanism is, the
  **consequence** is not. Fixed in task 2 step 4 by keying the refusal on target
  identity and seeding `de_duplicate` from what is already queued.

**One finding is surfaced to the coach rather than fixed** — see "For the
coach" at the end.

---

## 1. The coach keeps working while a run is going

**Files.** `crates/pundit-app/src/bus/export.rs`, `bus/project.rs`,
`bus/mod.rs`, `crates/pundit-app/src/main.rs`,
`crates/pundit-harness/tests/export.rs`, `CLAUDE.md`, `BACKLOG.md`.

**The coach's own answer (2026-10-07) and a feature on its own**: today a project
open during an export run is refused, so a coach who starts a 40-minute whole
match cannot look at another match until it finishes.

1. **`refuse_if_busy` keeps its three render callers whole** — `start_run`,
   `basket_job`, and (task 2) queue Start — and its doc gains §Q2's clause: an
   enqueue makes neither check because it renders nothing, and a project open
   makes only the preview one.
2. **The three project callers get their own guard**, `refuse_if_previewing`,
   with **`UserError::CantOpen`** — a **unit variant** whose `#[error]` carries
   the whole sentence (*"can't open that project: a preview is open; close it
   first"*). Not `CantOpen(String)`: there is one producer and one message, and a
   `String` invites a second. Call sites `project.rs:40`, `:96`, `:203`.
3. **Five comment sites assert the clause being removed and must change with
   it.** This is the work the draft hid:
   - `project.rs:36`–`:39` (`open_project`): *"An **export** or an open preview
     is composing from the project the coach is leaving, and `commit` below
     empties that project's trash and clears the history"* — **both halves are
     now false**, and §Q7's audit is why: a job names no path under `.trash`, and
     what costs it its recording is the clip delete. It also forwards its
     reasoning to `built_new_match`'s gate, so the cross-reference dangles.
   - `project.rs:200`–`:202` (`built_new_match`) — the comment the one above
     points at, so **this** is the one that carries §Q7's result.
   - `project.rs:88`–`:94` (`restore_last_project`) — justifies its guard as
     *"`open_project`'s reason, not a reason of its own"*, which now means the
     preview only.
   - `project.rs:171`–`:176` (`new_match`'s doc) — names
     **`refuse_if_busy`'s `CantExport`** by name and argues *"Both are modal,
     which is what the reasoning above needs"*. Re-point it at `CantOpen`, which
     is **why step 2's variant must stay a modal** and not join `is_notice`.
   - `main.rs:3072`–`:3074` — **one** comment over **three** statements. Making
     the first two conditional splits it; the match-editor half of the reason
     (*"another project's block of match events, whose video numbers mean other
     videos"*) stays unconditional.
4. **Correct `refuse_if_busy`'s false clause** (§R.3): *"Both composite on the
   UI's GL context"* is wrong for export, which runs on `Gl::shared()`'s
   surfaceless display — `Gl::shared`'s own doc says *"Export always runs
   here"*. The refusal stands on Phase 7 spec P5's exclusivity and on decode
   contention.
5. **`main.rs`'s `ProjectOpened` clear of `export-run` becomes conditional** on
   `!w.get_exporting()` (`:3075`–`:3076`). This is correct and was checked:
   `set_exporting` is written only in `show_export` from `Event::Export`,
   `begin` publishes that event before `OpenProject` can be handled, and the
   bus→UI channel preserves order — so `exporting` is already true when this arm
   runs. A finished run still clears, which is the original comment's own case.
6. **The end-of-run notice drops its folder clause unconditionally**
   (`main.rs:3363`): *"Exported 3 videos"*, not *"… to the project's exports
   folder"*. Once a project open mid-run is allowed, that clause names **the
   wrong project's folder** for any run the coach has navigated away from — and
   it is one string, true everywhere, with no state and no comparison. The draft
   dropped it only for a queue run, which would have been a lie in the one case
   this very task creates.

### Tests

- **`a_project_open_is_refused_while_a_run_is_going` is replaced, not
  amended** (§Q9): it asserts the refusal and its exact
  `UserError::CantExport("an export is running")` text. The replacement,
  `a_project_opens_while_a_run_is_going`, re-asserts **of an open that
  succeeds** the three things the old one was guarding — `Event::ProjectOpened`
  **is** published, the folder **is** pushed to `recent_projects`, a
  `project.json` **is** written — and then that the run still reaches a terminal
  state.
- **`a_project_open_is_refused_while_a_preview_is_open`**, which does not exist
  today: the clause that remains, now reading `UserError::CantOpen`.
- `clips.rs`'s `opening_empties_the_trash_and_the_history` must pass unchanged,
  because `commit` is untouched — the audit's conclusion as a test.

### Sabotage proof

1. **Keep the export clause in `refuse_if_previewing`.** The replacement test
   fails on its first assertion (`ProjectOpened` never arrives) and
   `a_project_open_is_refused_while_a_preview_is_open` still passes — which is
   what says the two clauses are tested apart rather than together.
2. **No proof for step 5**, and that is a finding rather than an omission: the
   conditional clear is **not testable in this repo**. The harness has no
   window, and `tests/ui/` cannot reach `main.rs`'s event arms — CLAUDE.md states
   that limit in as many words. What pins it is the comment beside it and the
   first real run; Risk 1 carries it.

---

## 2. The queue on `Bus`: four commands, one view, and the clip-delete rule

**Files.** `crates/pundit-app/src/bus/export.rs`, `bus/mod.rs`, `bus/clips.rs`,
`crates/pundit-harness/tests/export.rs`, `CLAUDE.md`, `BACKLOG.md`.

**In `export.rs`, not a new module** (§B): the queue is a second entry point
into that file's own job builder, so `Pickers`, `jobs` and the write-back all
stay private.

1. **Split `start_run`'s middle out**, with the four corrections §B lists:
   ```rust
   /// The jobs `targets` would render, each with the label the sheet lists it
   /// under, and their `exports/` directory made — every refusal that can be
   /// made from the open project, and none about whether anything may run now.
   ///
   /// **Split out for the queue** (#77 §Q2): an enqueue makes exactly these
   /// refusals and not `refuse_if_busy`'s, because it renders nothing. The
   /// directory is created **here**, which is what makes §R.5's rule structural
   /// — Start drains jobs and never calls this, so it cannot re-create a
   /// project folder the coach has deleted.
   fn jobs(
       &self,
       targets: &[ExportTarget],
       choices: ExportChoices,
       queued_labels: &[String],
   ) -> Result<Vec<(String, ExportJob)>, UserError>
   ```
   - It takes **`ExportChoices`**, and the `Pickers` conversion — with the one
     negation CLAUDE.md says lives in exactly one place — becomes
     `impl From<ExportChoices> for Pickers`. `Bus::export` is then
     `self.start_run(targets, choices)` and nothing outside `export.rs` names
     `Pickers`.
   - The **`refused` closure at `:332` moves with the range it serves**, or
     `start_run` is left with an unused binding and `-D warnings` fails.
   - `queued_labels` seeds `de_duplicate`, which is step 4's other half; `Export`
     passes `&[]`.
   - It returns just the `Vec`: `create_dir_all` is its last step, with
     `start_run`'s comment carried verbatim.
2. **The state**, §Q2's with one field added: `queue: Vec<Queued>` on `Bus`
   (**not** on `Open`, the basket's own reason — initialise it wherever `Bus` is
   built), and `Queued { folder, target, match_label, label, job }`. `folder` and
   `target` are step 4's key; `target` is free (`ExportTarget: Eq`).
3. **Four commands and one event**: `EnqueueExport { targets, choices }`,
   `RemoveFromQueue { index }`, `ClearQueue`, `StartQueue`, and
   **`Event::Queue(Vec<QueueRow>)`** — no `QueueView` wrapper, which would be a
   newtype over one `Vec` (`BasketView` earns its struct by carrying a name and
   two pickers; this carries rows). `QueueRow` is `pub` and re-exported for
   `main.rs` and the harness. None goes on the recording allow-list, so all four
   are refused while recording with no edit to it.
4. **Enqueue** is the duplicate check → `jobs` → push → write back. **No
   `refuse_if_busy`.** The write-back stays after the push for its own stated
   reason.
   - **The refusal is keyed on `(folder, target)`, not on `job.path`** — the
     bug §B ends on. Target identity is what *"already in the queue"* actually
     means; a path key refuses a **different** target whose label collides. A
     notice, `UserError::Queue(String)`, listed in `is_notice` beside `Basket`.
   - **And `de_duplicate` is seeded** with the labels already queued **for that
     folder**, so two targets of one project that share a name get `t0` and
     `t0 (2)` across two clicks exactly as they do in one run. The check and the
     seed are two halves of one rule and neither works alone.
5. **Start** is `refuse_if_busy` → *"the queue is empty"* → `begin` over the
   drained jobs, each labelled `format!("{label} — {match_label}")` (§Q3). The
   empty-queue refusal is **`CantExport`, not `UserError::Queue`**: Start is a
   button the coach is standing in front of, matching the basket's *"the basket
   is empty"*, and `Queue` is in `is_notice`.
6. **`QueueRow`** is `{ match_label, label, seconds }`, read off the **job** and
   never re-resolved — §Q8's deliberate inversion of the basket's E3.
   `seconds` is **`plan.total_frames() as f64 / f64::from(OUTPUT_FPS)`**, the
   expression `export_targets` already uses three functions above
   (`export.rs:162`) and CLAUDE.md's stated rule. `OUTPUT_FPS` is a `u32`, so the
   draft's `frames.len() as f64 / OUTPUT_FPS` would not have compiled.
7. **`trash_clip` drops the queued jobs that need the clip** (§Q6), with two
   corrections to the draft:
   - **After `remove_clip` succeeds**, not after `cancel_transcription_of` and
     `close_preview_of`. Those two run *before* `remove_clip(id)?`, which can
     make the whole function a no-op — and a cancelled transcription and a closed
     preview are both recoverable while **the queue has no undo**. A delete that
     does nothing must not destroy queue entries. Zero cost.
   - **Scan `Render::Encode(e).entries` for
     `e.clip.as_ref().is_some_and(|c| c.clip.id == id)`**, not
     `plan.entries` for `clip_id`. `ClipMedia::recording` is the field that
     actually names `recordings/<file>`, and a `Render::Copy` job carries no
     `Encode` at all — so *"a copy is never affected"* becomes true **by
     construction** rather than by a data coincidence.
   - The notice names how many, through step 4's `UserError::Queue`.

### Tests

- **Bus unit** (`export.rs`'s `mod tests`): the clip scan as a predicate over
  `Render` — a job holding the clip is dropped, one holding another clip is kept,
  and a `Render::Copy` job is kept. Under step 7's scan that third case is
  **meaningful** rather than a second spelling of the first.
- **Harness** (`harness/tests/export.rs`), §Q9's four minus the one §B folds in:
  - **A named step first: `Rig` gains a second project.** It hard-codes
    `<tmp>/project` with one `exports` and one `clips`; a `second_project()`
    helper calls `pundit_harness`'s existing `write_project` + `add_clips` into
    `<tmp>/project2`. No lib change.
  - **(a)** enqueue two targets from one project, open a second, enqueue one from
    it, Start — one run writes three files into **two** `exports/` folders. Then
    **two more assertions folded in**: one more `EnqueueExport` **after Start**
    arrives as an `Event::Queue` row and leaves the run alone (§Q8's one
    documented divergence, otherwise untested), and a project opened **while it
    renders** goes through and the run still finishes.
  - **(b)** the enqueue-time refusals and the duplicate notice, each having
    started nothing (`no_export_events`) — **including the two-targets-one-name
    case**, which must enqueue **both** and not refuse the second.
  - **(c)** a queued job whose game video is **deleted between enqueue and
    Start** fails its own row and the jobs behind it still write.
- **No media and no core tests.** Neither crate changes, and every type needed
  (`ExportJob`, `Compilation`, `PlanEntry`, `ExportTarget: Eq`, `OUTPUT_FPS`,
  `metadata::match_label`) is already public. **If either crate is touched, this
  design has been abandoned** — the tripwire is an empty diff in both.

### Sabotage proof

1. **Key the duplicate refusal on `job.path`** (the draft's own design). (b)'s
   two-targets-one-name case fails on a refusal it should not have made — the
   bug §B ends on, now with a test.
2. **Drop the `de_duplicate` seed.** The same case fails the other way: both
   enqueue and both write `t0`, so one file is overwritten mid-run. The two
   halves of step 4 are tested apart.
3. **Make `trash_clip` leave the queued job in place.** The unit predicate
   fails. No harness test reaches it, deliberately: §Q6's argument is that the
   film would be *silently wrong*, which a predicate states better than a render.
4. **Move the drop above `remove_clip`.** A delete of an unknown id must then
   leave the queue alone and does not — and if no test covers it, say so: this
   is a zero-cost ordering fix, not a behaviour anyone has asked for.

---

## 3. The Queue section in the export sheet

**Files.** `crates/pundit-app/ui/app.slint`, `crates/pundit-app/src/main.rs`,
`crates/pundit-app/tests/ui/export_sheet.rs`, `CLAUDE.md`.

**No eighth sheet** (§Q5, and §A's corrected count): enqueue has to be where the
choices are, the Run list is already here, and the section needs no `editing`
fold because it has no text field.

1. **Add to queue**, in the action row beside `Export` (`:2160`–`:2186`) —
   **the control the draft forgot**, without which `Command::EnqueueExport` is
   unreachable. Gated on `any-ticked` and **not** on `exporting`, because
   enqueue during a run is allowed (§Q6). `main.rs` gains `on_add_to_queue`
   building `targets` and `choices` exactly as `on_start_export` does
   (`:910`–`:925`).
2. **The Queue section**, between the targets and the Run list: a `FieldLabel`,
   a `ListView` at `min(140px, rows * 28px)` on the Run list's own row shape,
   `[x]` per row, `Start queue` in the header. Absent at zero rows, as the Run
   section is — which is why step 1's control is **not** in it.
3. **The card stays 480px unless the action row says otherwise.** The draft
   widened it to 520px on the basket's reason (*"the widest thing in it is a
   row's match beside its clip name"*), which does not apply: the queue row is
   the Run row's shape, and those rows carry that same string at 480px with
   `overflow: elide`. **But step 1 adds a fourth button** to a row that holds
   Export, Cancel and Close — so measure *that* at the 1100x700 floor and widen
   only if it does not fit. If it does widen, `:2084`'s comment (*"a 480px card
   has no fourth column"*) goes stale, and `:2189`–`:2190` attributes the
   basket's 520px to the **setup sheet's** width rather than to a row — so do
   not propagate a second attribution.
4. **Seven `enabled: !root.exporting` come off** — `:2019` (targets),
   `:2033`/`:2042`/`:2055` (the three pickers), `:2089` (Chapters), `:2099`
   (Scoreboard subtitles), `:2111` (Mute) — because each job froze its own
   choices and the live controls describe the **next** enqueue. `Export` and
   `Start queue` are the only things a run disables. Re-derive all nine numbers
   at the task.
5. **`a_run_greys_out_every_switch` inverts** (`tests/ui/export_sheet.rs:190`),
   which the draft never said although it said exactly this for
   `a_project_open_is_refused_while_a_run_is_going`. It asserts
   `accessible_enabled() == Some(false)` for all three checkboxes and its doc
   argues the opposite of step 4 (*"**A run settles all three**… the easiest of
   the five touch points to leave off"*). It becomes
   `a_run_leaves_every_switch_live` — same `switches()` loop, `Some(true)`, and a
   doc carrying step 4's reason. **It is the replacement, not an addition**: do
   not write both.
6. **One `run-kind` property, not a second bool.** `clear_run(w, basket: bool)`
   becomes `clear_run(w, kind: RunKind)` over `Export | Basket | Queue`, as a
   Slint enum beside `MatchTag` and `ScanStep`. `basket-run`'s four readers
   (`:2383`, `:2384`, `:2414`, `:2441`) become `run-kind == RunKind.basket`, and
   `show_export` reads the one property. The draft kept two bools with a
   promise that one writer sets them; one property **removes** the state rather
   than policing it, and `queue-run` would have had no Slint reader at all.
   **The third call site is new** — the queue's Start clears the previous run's
   rows before publishing its own — and `clear_run`'s doc (`:1188`–`:1194`) says
   *"both Starts… one run and two sheets"*: three and three now.
7. **A queue run's failures go to the notice line, not the error dialog**
   (§Q7, §Q4): *"N of M exports failed — the queue's list says which"*. An
   `Export` run keeps its modal. **Extract the decision**: which kind gets a
   notice and what it says is a pure function of `(RunKind, &ExportRun)`, and it
   belongs in a lib module beside `fit.rs`, `recents.rs` and `slate_pass.rs`
   rather than inline in `main.rs`, **because nothing in `main.rs` is testable** —
   which is the same limit Risk 1 carries for step 5 of task 1. Three kinds ×
   (some failed / none failed) is six cases and a `#[cfg(test)] mod` away.
8. **The Queue list is never cleared on `ProjectOpened`** — it is published by
   `Event::Queue` alone. Said in a comment beside task 1's conditional clear so
   nobody adds one by analogy. **Spanning the switch is the feature.**

### Tests

`tests/ui/export_sheet.rs`, on its established pattern — find the control by the
words on it, read it back, `invoke_accessible_default_action` rather than a
synthesized click:

- **Add to queue exists and is live while `exporting`** — step 1's whole point,
  and the one assertion that would have caught its absence;
- the Queue section is **absent** at zero rows and present at one, by its
  heading;
- a row's `[x]` sends `RemoveFromQueue` with **that row's** index, scoped by type
  name so a `✕` elsewhere in the window cannot be picked up (`keys_sheet.rs`'s
  `row_buttons` is the pattern);
- `a_run_leaves_every_switch_live` (step 5's replacement) over all three
  checkboxes — **check whether a `ComboBox` reports `accessible_enabled` before
  promising the three pickers too**; CLAUDE.md's accessibility guarantee is
  stated for `CheckBox`. If it does not, the checkboxes are the honest assertion
  and the pickers ride on the same one-line-per-control argument;
- the outcome decision's unit tests, from step 7.
- **`the_sheet_an_export_is_set_up_in_needs_no_scrolling` needs no assertion
  change.** It reads **no number** — its doc says so, and its one assertion is
  that exactly one `"Close"` is findable at the floor. The 589/623/798/823px
  figures are in the doc comment. So this is a **doc update**, plus a clause
  saying there is deliberately no queue list in its case either, for the run
  list's own stated reason. The draft called it *"the number"* and asked for a
  re-measure; nobody should "fix" this test.

### Sabotage proof

1. **Leave the seven `enabled: !root.exporting` on.** `a_run_leaves_every_switch_live`
   fails on whichever control it reads first.
2. **Delete `Add to queue`'s `clicked` handler.** The first test fails. If it
   only checks the button *exists*, it owes the click — a button wired to nothing
   is exactly the gap that let this control be forgotten.
3. **Clear the Queue list on `ProjectOpened`.** Harness (a) fails on the second
   project's enqueue, which is the test that spans the switch.
4. **Return the wrong `RunKind` from the outcome function.** Its unit tests fail
   on the notice-vs-modal choice. This is the proof the draft could not have —
   its version reached for *"a `show_export` unit"*, which is unreachable from
   `tests/ui`, and step 7's extraction is what makes it writable.

---

## Risks

1. **Two `main.rs` changes ship with no test and no runnable sabotage** — task
   1's conditional clear, and anything left inline in `show_export`. The harness
   has no window and `tests/ui/` cannot reach `main.rs`; step 7 of task 3 moves
   the part with content into the lib, and the one-line condition stays
   unpinned. **Stated rather than papered over**, and the first real run is what
   checks it.
2. **Task 1 changes behaviour nobody asked to be able to observe.** §Q7's audit
   is a reading of `commit`, not a test, so a bug here would appear as a
   *rendered film* being wrong — which nothing automated in this repo would
   catch. **Watch for it on the first real queue run**, and the thing to watch is
   a film from project 1 rendered while project 4 was open.
3. **"Keep working" is narrower than it sounds**, and the plan should not claim
   otherwise: `preview.rs:49` still refuses opening a preview while
   `self.export.is_some()`. During a 40-minute queue run the coach can switch
   projects but cannot preview a clip. That follows from Phase 7 spec P5 and is
   almost certainly right; it is listed because §Q7's wording reads broader than
   what ships.
4. **The queue is in memory** (§Q11), so a crash during a long queue loses it.
   The coach's own call; `queue.json` beside `basket.json` is the way in, and
   **never** a `state.json` key.
5. **`app.slint` is contended** (#102, #84, #115, #128). Task 3 is the only one
   of these three tasks that touches it, which is why it is last and alone.

## What this does not do

§Q11, unchanged: no persistence, no editing a queued job, no reordering, no
adding to a run already going, no pausing, no per-job cancel, no format change,
no `state.json` key, no `pundit-media` change, no `pundit-core` change.

## For the coach

**One thing the review turned up that is a judgement call, not a defect, and it
is being shipped the cheap way unless you say otherwise.**

A plain **Export** run's rows are labelled with the bare target — `All clips`,
`Whole match` — while a **queued** job's carry `"{target} — {match_label}"`
(§Q3). Once task 1 lets you open another project mid-run, the sheet in project 4
shows rows belonging to project 1 **with nothing saying so**, and that is
permanent for non-queue runs rather than a stage this passes through. Task 1
step 6 fixes the part that was an outright lie — the notice no longer names "the
project's exports folder" — and leaves the rows as they are.

The alternative is to label an ordinary run's rows the same way, which costs
every single-project export a project name it does not need, in the sheet you are
standing in front of. **Not worth it on my read**, so it is going in `BACKLOG.md`
as the thing to revisit the first time a row's project is actually ambiguous in
use. Say so if you would rather have the names everywhere from the start.

# Design — the export queue across projects

The coach (2026-09-24), BACKLOG **#77**: *"i open project 1, do stuff, enqueue.
then project 2, do stuff, enqueue, then start the queue and walk away for a bit.
other apps have this sort of thing, like mkvtoolnix's muxer."*

Read `CLAUDE.md` first. This spec does not repeat the export, basket or bus
rules it states.

**The shape is already agreed with the coach and is not re-opened here**: an
`ExportJob` is a self-contained value, so enqueue is *"build the jobs now, run
them later"* and the queue is a list of them. The first version keeps the queue
**in memory** — closing the app loses it — and does **not** let a queued job be
edited. §Q11 states the rest of what it leaves out.

**And one more thing he settled, asked as this spec's open question
(2026-10-07): *"yes you should be able to keep working"*** — so the queue runs
while the coach opens another project and carries on. §Q7 is that, and §Q10
records what the answer cost: today's refusal goes, and the audit it was
deferring is in this spec.

**Written against `origin/claude/78-task-1`, not `main`.** #78 is mid-flight in
`bus/export.rs`: its task 1 (`57f8224`) replaces `Command::Export`'s five
positional fields with one `ExportChoices` struct, splits the cue decision out of
`carry_scoreboard` into `board_cues`, and adds two `Preferences` fields to the
write-back. Its task 2 touches only `ui/app.slint` and two `main.rs` sites, so
the bus shape this spec builds on is task 1's and will not move again. Every
`Pickers`/`ExportChoices` reference below is that shape.

---

## Why this is cheaper than the backlog entry says

**Two of the three things #77 lists as "the work" are already done**, and
verifying that is most of this spec's value.

1. **The run loop is not tied to the open project.** `export::Active` holds a
   `VecDeque<ExportJob>`, an index, an `ExportRun`, an `Exporter`, three timing
   fields and a `cancelled` flag — and nothing else. `Bus::export_message`,
   `Active::finish_target`, `Active::next_job` and `Bus::start` read `self.open`
   **nowhere**; `start` takes only `self.tx`. Export renders on
   `Gl::shared()`, the process's private surfaceless EGL display, not the UI's
   context. The only ties to the open project are in `start_run`: the refusals
   that *build* the jobs, and the pickers' write-back.
2. **One job's failure already fails only itself.** `finish_target` maps
   `Err(ExportError::Failed(e))` to `TargetState::Failed(e)` and `next_job`
   carries on to the next job regardless; `harness/tests/export.rs`'s
   `a_failed_target_does_not_stop_the_run` pins it with a real mid-run failure (a
   directory where the first target's file goes). Only a **cancel** stops the
   rest, deliberately.
3. **And the entry point a queue needs already exists**, built for the basket:
   `Bus::begin(jobs: Vec<(String, ExportJob)>)` is **infallible**, documented as
   *"the caller has already refused everything there is to refuse"*, with the
   caller owning its own refusals and its own output directory (basket spec C3).
   Starting the queue is `self.begin(drained_queue)` — the argument `begin`
   already takes.

So #77 is: a list on `Bus`, one more command to fill it, one more to drain it
into `begin`, a section in the export sheet, and the refusals in §Q6.

**That finding does not cover §Q7.** *"Keep working while it runs"* is the
coach's answer to the open question, it arrived after the three findings above,
and it is the one part of this feature that touches code outside the queue:
`refuse_if_busy`'s callers split, a clip delete has to deal with the jobs
holding its recording, and two `main.rs` conditions change. §Q10 counts it.
Nobody should read "smaller than it looks" as covering that.

---

## Q1. `ExportJob` is self-contained. Field by field

`pundit-media/src/composite/export.rs`. No lifetime parameters anywhere in the
tree below, so nothing can borrow the project by construction; what remains to
check is whether anything *resolves* against the open project at run time.

| Field | Owned? | Resolves against the open project? |
|---|---|---|
| `compilation: Compilation` | `Vec<FrameSpec>` + `CompilationPlan` | No. Every frame's entry, source index, record time and text are already computed. |
| `path: PathBuf` | yes | No — **absolute**: `open.folder.join(EXPORTS_DIRNAME).join(file_name(label, &project.name))`, and `Open::folder` is documented *"Absolute and canonical"* (`bus/mod.rs:671`), set by `commit`'s `canonicalize`. |
| `cues: Option<Vec<Cue>>` | yes | No. Built by `board_cues` from the frozen `ScoreboardContext`. |
| `tags: FileTags` | six owned fields | No. `file_tags` already resolved the project's name, teams, final score and the first source file's mtime. |
| `render: Render::Copy { files, with_audio }` | `Vec<PathBuf>` | No — each is `open.folder.join(&s.relative_path)`, absolute. |
| `render: Render::Encode(Encode)` | see below | No. |
| ↳ `Encode::audio: Vec<Region>` | yes | No. `audio_regions` took the gain as a parameter (spec M2). |
| ↳ `Encode::{resolution, quality}` | `Copy` enums | No. |
| ↳ `EntryMedia::source: PathBuf` | yes | No — absolute, and **a file rather than an index** by that field's own design note. |
| ↳ `ClipMedia::{recording, clip}` | absolute path + a **cloned** `Clip` | No. The clip is cloned, so its strokes and events travel. |
| ↳ `EntryMedia::match_media: Arc<MatchMedia>` | `Arc`, one per contributing project | No. Holds the frozen `ScoreboardContext`, a `Vec<PlayerHighlight>` clone and the avatar as an **absolute** path. |

**Nothing is not self-contained.** `PlanEntry::source_index` stays project-local,
but it indexes `Encode::entries`, which travels with the job — which is exactly
what `EntryMedia::source`'s doc says it is for. `MatchMedia` travels with the
job, by `Arc`, and `ScoreboardContext` inside it.

**Two things the job does not carry, and neither is a gap.**

- **The project's name is not a field.** It is inside `tags.title`,
  `tags.description` and the basename of `path` — in the last two through
  `naming::safe_chars`, so reading it back out is lossy. §Q3 is what the row
  stores instead.
- **The files themselves are opened lazily, per entry, at run time.** That is
  the whole of what "a snapshot" does not promise, and §Q4 and §Q6 are where it
  lands.

## Q2. The queue lives on `Bus`, beside the basket — and the run loop is untouched

```rust
/// What is waiting, in the order it was enqueued. **On `Bus` and not on
/// `Open`**, for the basket's own reason: `Open` is replaced on every project
/// open, and spanning that is the point (basket spec H3).
queue: Vec<Queued>,

struct Queued {
    /// `match_label(project)` at enqueue (Q3).
    match_label: String,
    /// The target's own label, as the sheet and the file name use it (spec E6).
    label: String,
    job: ExportJob,
}
```

A `Vec`, not a `VecDeque`: it is indexed for removal and drained whole, and
`Active` keeps the `VecDeque` it already has.

**`start_run` splits in two, and that is the only change to the existing file.**
Today it is: `refuse_if_busy` → build jobs (refusing per target) → create
`exports/` → `begin` → write back the pickers. It becomes

- `fn jobs(&self, targets, pickers) -> Result<Vec<(String, ExportJob)>, UserError>` —
  the middle, unchanged: no project open, nothing ticked, a target with nothing
  to export, a missing game video or recording, each naming the clip.
- **Export** (run now): `refuse_if_busy` → `jobs` → create `exports/` → `begin` →
  write back. Byte for byte what it does today.
- **Enqueue**: `jobs` → create `exports/` → push → write back. **No
  `refuse_if_busy`** (§Q6), and the write-back stays *after* the push for its own
  stated reason — nothing on the way to a refusal may dirty the project.

**`refuse_if_busy` is a refusal to *render*, so it guards Start and not
enqueue.** Its two members divide cleanly:

- *"an export is running"* — applies to **Start** (one `Active` at a time, which
  is what `Input::Export`'s *"there is only ever one"* rests on) and **not** to
  enqueue, which renders nothing.
- *"a preview is open; close it first"* — applies to **Start** and not to
  enqueue, for the same reason.

**It has five callers today and keeps three.** The three *render* callers —
`start_run` (`export.rs:339`), `basket_job` (`basket.rs:422`) and now queue
Start — keep it whole, and each still calls it **first**, before any I/O. The
three *project* callers — `open_project` (`project.rs:40`),
`restore_last_project` (`:96`) and `built_new_match` (`:203`) — lose the
export clause and keep only the preview one (§Q7). So the doc comment's rule
gains two clauses: enqueue makes neither check because it renders nothing, and
a project open makes only the preview one because a queue run holds nothing of
the open project.

**Those three get a guard of their own**, one line beside `refuse_if_busy`, and
with it the message they give. Today a refused open reads *"can't export: a
preview is open; close it first"* — `UserError::CantExport`, because they borrow
an export's refusal. One new variant, `UserError::CantOpen(String)` → *"can't
open that project: {0}"*, is the whole of the fix, and this change has to touch
that line anyway.

**One correction while we are here.** That second refusal's comment reads *"Both
composite on the UI's GL context, and an export would take the frames the preview
is pacing itself on (spec P5)"*. The first clause is **false**: `Gl::shared`'s own
doc says *"Export always runs here"*, on the process's surfaceless display, and
only a preview takes `Gl::wrapped`. The refusal is right — Phase 7 spec P5 states
the exclusivity, and the second clause is the live reason — but the comment should
lose the first clause rather than have a queue spec quote it.

Nothing else in the run loop moves. `Active`, `ExportRun`, `ExportTargetRun`,
`TargetState`, `finish_target`, `next_job`, `export_message`, `cancel_export` and
`de_duplicate` are all unchanged.

## Q3. A row is a match label, a target label and a length

`Queued` stores two strings and the job. The strings are captured at enqueue from
the project the job was built from:

- **`match_label(project)`** — `core::metadata`, the basket's own row label: the
  teams, else the project's name, else `UNTITLED`, and explicitly *"not the
  project's folder name"*. No new code.
- **the target's label** — the string `jobs` already produced for `begin`, which
  is what the sheet calls it and what names its file.
- **the length** is not stored: it is `job.compilation.frames.len()` divided by
  `OUTPUT_FPS`, which is also where `ExportTargetRun::frames` comes from.

**This is not a second copy of the project.** It is a copy of two labels the job
had already burned into its own `path` and `tags` — so a row cannot disagree with
the file it will write, which is the point. A row that re-read its project at
display time (the basket's rule) *could*: the job is frozen and the project is
not. §Q8 is where that contrast is argued.

`begin` is handed `format!("{label} — {match_label}")` for a queued job, so one
run of three projects' "All clips" reads as three distinguishable rows with no
change to `ExportTargetRun`. Two projects with the same `match_label` and the
same target give two identical row labels — they still write to their own
`exports/` folders, so nothing is lost but the ability to tell the rows apart,
and the duplicate refusal in §Q6 keys on the thing that actually collides.

## Q4. Failure isolation: already the behaviour, and what the coach sees

A queue run **is** a run. Per job:

- `Ok(ExportDone)` → `TargetState::Done(path)`, plus the `bus: exported …` line
  with the decoder, encoder, chapters, sidecar and `moov` reserve.
- `Err(ExportError::Failed(e))` → `TargetState::Failed(e)`, `bus: export failed:
  {e}` on stderr, **and the run goes on**. The frames it did render are credited
  to the rate, the ones it did not are not (`finish_target`'s `rendered`).
- `Err(ExportError::Cancelled)` → `Cancelled`, and every job after it is marked
  `Cancelled` without being started. Cancel is the one thing that stops a queue,
  which is what the button says.

`ExportDone` means for a queue exactly what it means today: one job's outcome.
There is no run-level success, and `ExportRun::is_running()` going false is still
the only "the run is over" signal.

**What the coach sees.** Every job is a row in the sheet's existing Run list with
its own status, and those rows survive the run: `main.rs` clears `export-run`
only on a Start (`clear_run`) and on `ProjectOpened` — and §Q7 makes that second
one wait for the run to finish — so a coach who walks away comes back to a row
per job saying `Done` or `Failed`. Two wordings in `show_export` are the open
project's and need the queue's:

- the end-of-run notice says *"Exported N videos to the project's exports
  folder"*, which is wrong for a run spanning projects — for a queue run it drops
  the folder clause.
- the error dialog reports the **first** failure and returns, so a run with two
  failures names one. For a queue run the rows are the report and the failure
  goes to the notice line instead of a modal (§Q7); a single-project `Export` run
  keeps the dialog it has, where *"a run of one target is still the common case"*
  still holds.

A **cancel** still names nothing: the rows carry it.

## Q5. The queue lives in the export sheet, not in a seventh sheet and not in a column

`app.slint:1835` names the six `Sheet`s (export, the basket, New match, match
setup, the match event editor, the error dialog), and `ExportSheet`'s own comment
calls itself *"the **only** export UI"*. Three reasons the queue belongs inside it
rather than beside it:

1. **Enqueue has to be where the choices are.** A queued job freezes all six of
   `ExportChoices` plus the ticks. Every one of those controls is on this sheet,
   and `Add to queue` is the same click as `Export` with a different destination.
   A separate surface would either duplicate the controls or enqueue with
   whatever the other sheet last had.
2. **The run list is already here**, rendering `Event::Export(ExportRun)`. A
   queue run publishes that same event, so Start's progress, its estimate, its
   per-job status and its Cancel button cost **no new UI at all**.
3. **It is not a panel and not a popover** — #78 spec §U4's argument, which
   applies unchanged: the side columns only grow and are permanently on screen,
   and a `PopupWindow` has no `editing` to fold into `text-editing`. (The queue
   section needs no fold either: unlike the basket's, it has no text field.)

```text
Queue (3)                                            [ Start queue ]
  All clips — Rovers v Athletic                4:12          [x]
  Corners — Rovers v Athletic                  1:06          [x]
  Whole match — U13 v Ash                     54:12          [x]
```

- A `ListView` with the Run list's own bounded height and row shape
  (`min(140px, rows * 28px)`, 28px rows), and `[x]` removing one row. The
  section is absent at zero rows, as the Run section is.
- **The card goes to 520px**, the basket sheet's width, *"because the widest thing
  in it is a row's match beside its clip name"* — a queue row is that same shape.
- **The ticks and the six controls stop being greyed by `exporting`.** Under a
  queue they are no longer *"settled for the run"*: each job froze its own
  choices at enqueue, so the live controls describe the **next** enqueue and
  nothing else. `Export` and `Start queue` are the only things a run disables.
- **`Export` ignores the queue and the queue ignores `Export`.** Export runs the
  ticked targets now; Start runs what is waiting. Both end in `begin`. An export
  and a queued job for the same target overwrite one file, which is already the
  rule for an export's name — *"safe to overwrite because it is derived from
  stable identity"* (`basket.rs`'s `output_path`) — so there is nothing to refuse.
- **Enqueue's own confirmation is the row appearing.** No notice.

Commands and events, mirroring the basket's set:

```rust
Command::EnqueueExport { targets: Vec<ExportTarget>, choices: ExportChoices },
Command::RemoveFromQueue { index: usize },
Command::ClearQueue,
Command::StartQueue,
Event::Queue(QueueView),   // published on every change; the rows, resolved
```

`Bus::command`'s recording guard is an allow-list — *"everything not listed is
refused, so commands added later are too"* — so all four are refused while
recording with no edit to it, as `Command::Export` is.

## Q6. What is refused, and the reason for each

**At enqueue** (the coach is in that project and can fix it, which is why this is
the moment):

| Refusal | Why here |
|---|---|
| no project open; nothing ticked; a target with nothing to export | `jobs`' existing checks, unchanged. |
| a missing game video or commentary recording, **naming the clip** | `entry_media`, unchanged: it `stat`s rather than reading `Bus::missing`, so a file deleted since the last refresh is caught. Refusing at Start instead would name a clip in a project the coach left hours ago. |
| `exports/` can't be created | As today. **Created at enqueue, never at Start**: `create_dir_all` at Start would silently re-create a project folder the coach has since deleted and write a film into an otherwise-empty directory — the hazard `CLAUDE.md` records for an unchecked `last_project`. |
| **the same output path is already queued** — a notice, *"already in the queue"* | Keyed on `job.path`, which is the thing that collides, so it covers two targets of one project and two same-named projects alike. Refused rather than replaced because the coach cannot tell two identical rows apart and the second render buys him nothing. Removing the row and enqueuing again is how a stale job is refreshed — there is no edit (§Q11). A **notice**, following the basket's *"already in the basket"*: one click refused with nothing to answer. One new `UserError::Queue(String)` variant, listed in `is_notice`. |
| **not** `refuse_if_busy` | Enqueue renders nothing (§Q2). So a coach can fill the queue while a run is going; `Start queue` is what waits. |

**At Start**:

| Refusal | Why |
|---|---|
| `refuse_if_busy`: *"an export is running"* | One `Active`, one `Exporter`, one FIFO of `Input::Export`. |
| `refuse_if_busy`: *"a preview is open; close it first"* | Phase 7 spec P5's exclusivity; the live reason is contention, not a shared GL context (§Q2). |
| *"the queue is empty"* | `begin`'s precondition: it logs a bug for an empty job list, and both existing callers refuse one. |

**Not refused, and each is a job failing itself at run time** (§Q4), because
nothing can be checked at Start that would still be true when the job's turn
comes:

- **the project folder deleted after enqueue** — `File::create` of the `.part`
  fails. Nothing recreates the folder.
- **a game video moved, renamed or on an unmounted drive** — the decoder fails to
  open it.
- **the disk filling** — the write fails part-way; the `.part` is deleted and the
  file already at `path` is untouched.
- **the project edited after enqueue** — the job is a snapshot and ignores it.
  That is the agreed shape, not a defect. A source **removed** from the list
  leaves the file on disk, so a queued whole-match job still renders from it; a
  source **relinked** is relinked because the old file is gone, so that job
  fails itself, as above.

**One edit is not left to fail itself: deleting a clip drops the queued jobs
that need it.** `trash_clip` moves `recordings/<file>` to
`recordings/.trash/<file>`, and a job's `ClipMedia::recording` is
`recordings/<file>` — so from that moment `Pip::open` cannot read it and the
entry falls back to the 1×1 GL filler. The job still writes a file: that entry's
inset is gone **and so is its commentary**, with its text bar and its chapter
still in place. A clip *is* its commentary, so that film is silently wrong, and
a silent quality loss is worse than a failure or a refusal.

- **The delete wins; the job goes.** `trash_clip` already does exactly this to
  the other two holders of that file — `cancel_transcription_of(id)` and
  `close_preview_of(id)`, in that order and for that reason. A queued job is a
  third holder, and it gets the same treatment: every queued job whose
  `compilation.plan.entries` contains `clip_id == Some(id)` is dropped, with a
  notice naming how many — `UserError::Queue`, the variant the duplicate refusal
  above already adds, not a second one. The test is a scan of the plan, so it
  needs no new state and a `Render::Copy` job (a whole match, no clips) is never
  affected.
- **Not a refusal**, although `source_is_referenced` is the precedent for one.
  That refusal exists because removing a source would leave stored indices
  pointing at the wrong file; deleting a clip corrupts nothing, it is undoable,
  and the coach's work on his project must not be held up by a queue he may
  have forgotten. Refusing an edit to protect a render inverts the app's own
  order of precedence.
- **Not a rebuild without that clip**, which would be editing a queued job
  (§Q11) and would change under the coach what he enqueued.
- **Undo restores the clip, not the job.** The queue has no undo, as the
  basket's clear has none: re-tick and enqueue again.

And two things a waiting queue does **not** block, because `refuse_if_busy` reads
`self.export` and a waiting queue is not an `Active`: **recording**
(`recording.rs:318`) and **transcription** (`transcribe.rs:330`). Only a running
queue does, as any run does.

## Q7. The coach keeps working while the queue runs — and the audit of `commit`

*"yes you should be able to keep working"* (2026-10-07). So `open_project`,
`restore_last_project` and `built_new_match` lose `refuse_if_busy`'s export
clause (§Q2) and refuse only on an open preview. A run cannot have started with
a preview open, so during a queue run a project open always goes through; the
clause that remains is for the coach who opens a preview and then clicks Open,
which is unchanged behaviour.

**Everything `commit` does, against a job that is rendering.** Audited line by
line rather than assumed, because this is the whole risk of the coach's answer.

| `commit` does | To a running job |
|---|---|
| `push_recent_project` | Nothing. |
| `unload()` → `close_preview`, `reset_skip`, `player.unload()`, `set_playing(false)` | Nothing. This is the **scan** player; a job's decoders live on the export thread, on `Gl::shared()`'s own display. |
| `armed_slate = None` | Nothing: bus state. |
| `history.clear()` | Nothing. Undo is in memory and a frozen job reads none of it. |
| `empty_trash(&open.folder)` and `empty_trash(&folder)` | **Nothing a job can read** — see below. |
| `player.set_volume(prefs.scan_volume)` | Nothing: the scan player again. |
| `current = 0`, `open = Some(..)` | Nothing: the job borrows neither (§Q1). |
| `refresh_missing()` | Nothing. `entry_media` deliberately `stat`s rather than reading `Bus::missing`, *because that flag says nothing about a project that is not open* (basket spec V2) — written for the basket, and it is what makes this safe. |
| `snapshot()` → `Event::ProjectOpened` | UI only; see below. |
| `reset_transcription()` | Cancels a transcription, as it does today. Independent of any job, and a queue run's Start does not refuse one. |
| `ensure_loaded(0.0)` | Decode contention with the run, which is true of any work the coach does during one. A performance matter, not a correctness one. |

**The trash emptying is not the sharp edge it looks like, and the reason
matters.** A job's `ClipMedia::recording` is `<folder>/recordings/<file>`
(`export.rs:753`); the trash is `<folder>/recordings/.trash/<file>`
(`clips.rs:321`, `TRASH_DIRNAME = ".trash"`). A job never names a path inside
the trash, so `remove_dir_all` of it removes nothing a job reads. **What costs a
job its recording is the *delete*, which moved the file out from under it** — and
that is true today, in one project, with no queue and no project switch.
`commit`'s emptying and its `history.clear()` destroy only the **undo** of that
delete, which no job reads. So the hazard belongs to the clip delete, it is
answered there (§Q6), and the project switch inherits nothing from it.

**So nothing in `commit` has to change.** The queue needs no exception in it,
and `clips.rs`'s `opening_empties_the_trash_and_the_history` keeps its
behaviour exactly.

**What the sheet shows with the coach in project 4 and rows from 1–3
rendering.** Most of this is already paid for by §Q3: every Run row's label is
`"{target} — {match_label}"`, captured at enqueue, so the rows describe
themselves in any project. The rest:

- **`main.rs`'s `ProjectOpened` arm must not clear `export-run` while the run is
  going.** Today it clears unconditionally, with the right reason for the case it
  was written for (*"Another project's run doesn't belong in this one's sheet"*) —
  which was a case where a run could only be **finished**. The clear becomes
  conditional on `!w.get_exporting()`, which the window already tracks from
  `show_export`. A **finished** run's rows still go when the project does: that
  reason survives untouched.
- **The Queue list is never cleared on `ProjectOpened`.** It is published by
  `Event::Queue` alone, which a project open does not emit, so there is nothing
  to do — stated here so that nobody adds a clear beside `export-run`'s by
  analogy. Spanning the switch is the feature.
- **The targets list, the ticks and the six controls are project 4's**, which is
  right: they describe the next enqueue, not the run.
- **`clear_run` now has three Starts**, not two; the queue's clears the previous
  run's rows before publishing its own, for the same stated reason.
- **A queue run reports its failures as a notice, not the error dialog.** A
  modal landing over project 4's work is exactly what `UserError::Scoreboard`
  and `UserError::Slate` are notices for, and the Run rows carry which job failed
  and why. So a finished queue run shows *"N of M exports failed — the queue's
  list says which"* on the notice line. A single-project `Export` run keeps the
  modal: the coach is standing in front of that one.

## Q8. Diff it against the basket — the explicit step

The basket is the nearest thing in the tree: it already resolves references from
several projects, reads one `project.json` per distinct match, builds one
`MatchMedia` per project shared by `Arc`, and renders a cross-project film. Every
rule of it was checked against this feature rather than assumed to carry.

| Basket rule | Here | |
|---|---|---|
| Lives on `Bus`, not on `Open`, because `Open` is replaced on every open (H3) | **Same.** The queue is precisely the thing that must survive a project switch. | ✓ take |
| A piece is a **reference**, resolved at Start, so a clip fixed after adding is exported as it now stands (E1) | **Deliberately opposite.** A queued job is a **snapshot**. The coach agreed to it, and it is what makes the run loop independent of the open project: a reference would have to re-read `project.json` at Start and could then refuse a job built hours ago. | ✗ invert, on purpose |
| One resolver, two callers, so the sheet can never show a piece Start then refuses (E3a) | **Not needed, and would be wrong.** The job is frozen, so the row is read off the job. A live-resolved row would be the only thing in the feature that *could* disagree with what gets rendered. | ✗ not applicable |
| Rows resolved each time the sheet is shown, **never cached beside the reference**, which goes stale the moment a clip is renamed (E3) | **Cached, for the inverse reason.** A rename after enqueue does not reach the job's `path` or `tags` either, so a fresh label would misdescribe the file about to be written. | ✗ invert, on purpose |
| `distinct_matches`: one `project.json` read per match, not per piece | **No reads at all.** Enqueue reads the project it already has in memory; Start reads nothing. | n/a |
| One `MatchMedia` per project, shared by `Arc` between its entries | **Same, per job**, via `job`'s existing `Arc::new(MatchMedia { … })`. Across jobs there is no sharing and should be none: two jobs are two snapshots. | ✓ already there |
| Its own file, **not a key in `state.json`**, because one unreadable value would take the pen and the last project with it (H1) | **No file at all** (§Q11): in memory, by the coach's own call. So the hazard does not arise — and if persistence ever lands it is `queue.json` beside `basket.json`, never a `state.json` key. | ✓ noted, deferred |
| Pickers stored as **string labels** so an unknown one reads as the default | n/a while nothing is stored. The same rule binds any future `queue.json`. | ✓ noted |
| `refuse_if_busy` **first**, before three projects' worth of I/O (C3) | **Start does; enqueue does not**, and §Q2 says why. The doc comment on `refuse_if_busy` says *"both job builders"* make them first and needs a third clause. | ~ diverge, documented |
| `begin` is infallible; the caller owns its refusals and its own output directory (C3) | **Taken whole.** This is the third caller and it adds no new refusal to `begin`. | ✓ take |
| Start writes the sheet's state **eagerly, before the refusals**, because the file *is* the sheet's memory | **Not taken.** The queue's write-back is the export sheet's `Preferences` one, which is deliberately late (*"nothing above this can have dirtied the project on its way to a refusal"*), and `a_refused_run_leaves_the_pickers_alone` pins it. | ✗ keep the export sheet's rule |
| All-or-nothing: one unresolvable piece refuses the whole film, because a film silently missing the piece the coach cared about is worse than no film (V1) | **Inverted, and this is #77's own requirement.** A queue is N films: one failing job must fail only itself. Within a job, V1's rule still holds — `jobs` refuses the whole job on a missing video. | ✗ invert, on purpose |
| A typed name, cleaned to 200 bytes, defaulted, suffixed `" (2)"` rather than overwritten | n/a: a queued job's path is an export's derived `<label> - <project>.mp4`, which overwrites by design. | n/a |
| Adding a duplicate is a **notice**, not a modal | **Taken** (§Q6). | ✓ take |
| Clear has no undo, because what is lost is references | **Taken, and weaker**: what is lost is snapshots, re-made by ticking and clicking again. | ✓ take |
| The sheet renders what it is handed and keeps nothing | **Taken**: `Event::Queue(QueueView)` is the whole of it, following `Event::Basket`. | ✓ take |
| Its own `Sheet`, *"and no fifth panel shape"* | **Not taken**: no new sheet (§Q5). The basket needed one because it has a name field, two pickers and a message line of its own; the queue has none of those and its controls are already on the export sheet. | ✗ diverge, argued |

Two rules of the basket's that are **not** reusable and could have been assumed
to be: its row resolution (E3/E3a) and its all-or-nothing refusal (V1). Both
invert here, and both inversions are the coach's agreed shape rather than a
shortcut.

## Q9. Tests — three groups

1. **Bus unit (`bus/export.rs`'s `mod tests`).** The duplicate key: two
   `Queued`s with the same `job.path` collide and two with different paths do
   not — a pure function on the list, asserted the way
   `the_pickers_are_read_back_from_the_project` asserts `Pickers::of`, rather
   than through three renders.
2. **Harness (`harness/tests/export.rs`, beside the existing run tests).** Four,
   no more: (a) enqueue two targets from one project, open a second project,
   enqueue one from it, Start, and the one run writes three files into **two**
   `exports/` folders — which is the whole feature, and the only test that needs
   two projects; (b) the enqueue-time refusals and the duplicate notice, which
   must not have started anything (`no_export_events`); (c) a queued job whose
   game video is **deleted between enqueue and Start** fails its own row and the
   jobs behind it still write — the queue's half of
   `a_failed_target_does_not_stop_the_run`, and the one behaviour §Q6 moves from
   refusal to failure; (d) **a project opened while the queue is rendering goes
   through, and the run finishes** — the coach's answer, and the test that
   replaces the one below.
3. **No media and no core tests.** Neither crate changes: no new `ExportJob`
   field, no format bump, no `state.json` key, nothing stored. If either crate
   is touched, this design has been abandoned.

**One existing test inverts.** `a_project_open_is_refused_while_a_run_is_going`
asserts exactly the refusal the coach's answer removes — including its
`UserError::CantExport("an export is running")` text — so it is **replaced** by
2(d), not amended. What it was really guarding (the refused open published
nothing, pushed no recent and wrote no `project.json`) now has to hold of an open
that **succeeds**: it publishes `ProjectOpened`, pushes its recent, and the run
goes on. Its sibling `a_preview_open_…` does not exist today; 2(d)'s second half
is that a project open **is** still refused while a preview is open, now reading
`UserError::CantOpen`.

Existing tests that must still pass unchanged, because they are what pins the
split in §Q2 and the audit in §Q7:
`a_run_renders_every_target_and_its_frames_only_fall`,
`a_failed_target_does_not_stop_the_run`,
`while_a_run_is_going_a_second_run_and_recording_are_refused`,
`a_refused_run_leaves_the_pickers_alone`,
`a_run_persists_the_resolution_quality_and_switches` (#78 task 1's rename of
`…_the_resolution_quality_and_mute`; the plan's *"renamed `…_and_switches`"*
reads as a suffix, and the name it actually took drops `mute`) and
`clips.rs`'s `opening_empties_the_trash_and_the_history`, which keeps its
behaviour because `commit` is unchanged.

## Q10. The question the coach answered, and what the answer cost

This spec was written with one open question — *may the coach open another
project while the queue runs?* — and a default of keeping today's refusal. **He
answered it on 2026-10-07: *"yes you should be able to keep working"***, so the
refusal goes and the audit the question was deferring is §Q7. **No open question
remains.**

The answer grew the feature, and the scope reduction at the top of this spec does
not cover it. Exactly what it added, so nobody has to guess:

| | |
|---|---|
| `refuse_if_busy` keeps its three render callers, loses its three project ones | §Q2 |
| one new `UserError::CantOpen(String)`, and a one-line guard for those three | §Q2 |
| `trash_clip` drops the queued jobs that need the clip, with a notice | §Q6 |
| `main.rs`: the `ProjectOpened` clear of `export-run` becomes conditional | §Q7 |
| `main.rs`: a queue run's failures go to the notice line, not the modal | §Q7, §Q4 |
| one harness test **replaced** (`a_project_open_is_refused_while_a_run_is_going`) and one added | §Q9 |

And what it did **not** add, which is the point of auditing rather than
defending: **no change to `commit`**, no exception in it for a running job, no
change to `Active` or the run loop, and still no `pundit-media`, `pundit-core`,
format or `state.json` change.

## §R. What my own first reading of this got wrong

1. **I took #77's "don't tie the run loop to the open project" as work to do.**
   The run loop is already untied: `Active` and `export_message` never touch
   `self.open`, and export has its own GL display. The ties are `start_run`'s
   refusals and the write-back, which is a split, not a rewrite. BACKLOG #77's
   entry overstates it and is corrected.
2. **I took failure isolation as work to do.** It has been the behaviour since
   Phase 8 and is pinned by `a_failed_target_does_not_stop_the_run`. What #77
   actually needs is for the *refusals* to move from "the whole run, up front" to
   "the one job, at enqueue" (§Q6) — a different change entirely.
3. **I read `refuse_if_busy`'s comment as fact.** *"Both composite on the UI's GL
   context"* is false for export, which runs on `Gl::shared()`'s surfaceless
   display; `Gl::shared`'s own doc says so. The refusal stands on P5's
   exclusivity and on contention, and the comment should lose the clause rather
   than be quoted.
4. **I first designed the queue row the basket's way** — a reference, resolved for
   display each time the sheet opens, because *"never cached beside the
   reference"* is the basket's explicit rule. That is wrong here and the reason
   inverts: the basket's piece is resolved at Start, so a fresh label is the
   truth; a queued job is frozen, so a fresh label would misdescribe the file
   about to be written. This is the shape-copying mistake §Q8 exists to catch,
   caught on the feature it was aimed at.
5. **I assumed `exports/` would be created at Start**, by symmetry with the
   basket's *"a run that can't start leaves no folder behind"*. At Start it would
   re-create a deleted project folder; at enqueue it cannot, and enqueue is when
   the coach is standing in that project anyway.
6. **I considered making the queue and `Active.jobs` one list**, so a job could be
   added to a run already going (mkvtoolnix does allow that). It would make
   `Active` and the queue two views of one `VecDeque` with two owners of the
   order, to buy adding a job to a run the coach has walked away from. Left out
   (§Q11).

Corrected after the coach answered the open question (2026-10-07):

7. **I named the trash emptying as the hazard of letting a project open land
   mid-run, and it is not one.** `commit` does empty **both** folders'
   `recordings/.trash`, but a job's `ClipMedia::recording` is
   `recordings/<file>` and never a path inside `.trash` — so emptying it removes
   nothing a job reads. What actually costs a job its recording is the **clip
   delete**, whose rename moves the file out from under it, and that has been
   true since Phase 8 in one project with no queue and no switch. Emptying the
   trash and `history.clear()` destroy only the *undo* of that delete, which no
   job reads. The hazard was real; it was attached to the wrong operation, and
   relocating it is what made §Q7 need no change to `commit` and §Q6 need a rule.
8. **I had the queue's failures keep the error dialog.** A modal is right for a
   run the coach is standing in front of and wrong for one he walked away from,
   which is the distinction `UserError::Scoreboard` and `UserError::Slate` are
   already notices for. §Q7 splits them.
9. **I listed `a_project_open_is_refused_while_a_run_is_going` among the tests
   that must pass unchanged.** It asserts the refusal the coach's answer removes,
   so it inverts and is replaced (§Q9). The three things it was really guarding
   have to be re-asserted of an open that succeeds.
10. **And I deferred the audit as "its own entry, not inside this one."** It took
    one reading of `commit`'s eleven steps, and nine of them were answerable from
    rules already written down — `entry_media`'s deliberate `stat` over
    `Bus::missing` (basket spec V2) being the one that carries it. Deferring an
    audit is only cheap when nobody has done it.

## Q11. What it does not do

- **No persistence.** Closing the app loses the queue. The coach's call, and the
  loss is snapshots that are re-made by ticking and clicking. If it is ever
  wanted it is `queue.json` beside `basket.json` — **never** a `state.json` key
  (basket spec H1) — and every `ExportJob` field would have to become
  serializable, which is most of the work and none of the value today.
- **No editing a queued job.** Remove it and enqueue again. The job is a frozen
  snapshot, so "edit" would mean rebuilding it, which is what re-enqueuing is.
- **No reordering.** The basket has `move_basket_entry`; a queue of three jobs
  whose order only changes which finishes first has no use for it yet.
- **No adding to a run already going** (§R.6). Enqueue during a run is allowed;
  it waits for the next Start.
- **No rebuilding a queued job when its clip is deleted** — the job is dropped
  instead (§Q6).
- **No pausing, and no per-job cancel.** Cancel stops the run, as it does today;
  the jobs it never started are marked `Cancelled` and are gone. Re-queuing them
  is the way back, and a per-job cancel is a row's third state nobody has asked
  for.
- **No format change, no `state.json` key, no `pundit-media` change, no
  `pundit-core` change.**

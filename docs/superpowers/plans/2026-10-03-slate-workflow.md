# Plan — the slate workflow's hard half: the out-point stop, the preview, the pass

Spec: `docs/superpowers/specs/2026-10-02-slate-workflow-design.md`. **Read it
first; this plan does not repeat its reasoning.** Read `CLAUDE.md` too.

**M and T1 have shipped** (PRs #29, #30): the mark buttons and the slates' tag
filter. What is left is the half the spec's review found a bug in nearly every
paragraph of.

**Revised through two adversarial passes, and the first draft's one original
idea was wrong.** §R keeps the record. The short version: the draft added a
"deadline as a hint, position check as truth" mechanism to make a missed re-arm
degrade to "pauses late" — but **"late" is unbounded**, which is
indistinguishable from "never", so it bought nothing. The replacement is smaller
and has no re-arms at all.

**Gates.** `CLAUDE.md`'s build conventions. Clippy must be
`rustup run 1.92 cargo clippy --workspace --all-targets -- -D warnings`: a clean
local 1.98 is not the gate.

**`pundit-core` and `pundit-media` are untouched by every task.**

## Where this stands (update it as tasks land)

- **A — the span on the scrubber.** **Landed.** Two floats on the `Scrubber`,
  one translucent `Rectangle` drawn *before the marks* as well as the slider (so
  a goal tagged inside the range still reads), fed from
  `main.rs::show_slate_span` on the two paths item 2 names. The concat mapping
  and the three "nothing to draw" cases are pinned in a new sibling module,
  `crates/pundit-app/src/slate_span.rs`. One addition the plan did not call for:
  the span is dropped while a preview is open, for the same reason `marks`
  already is — a preview's scale is its clip's, not the concat timeline's.
- **B — the stop.** **Landed.** `Bus.armed_slate: Option<Uuid>`, the bounded
  poll in the loop's `.min()` chain, and the check in the loop's tail, all as
  written. The lifecycle is the four sites item 11 names (`start_recording`
  after `recording = Some(…)`, `finish_recording` / `abort_recording`, and
  `project::commit`, which is what every open goes through), plus the disarm.
  **One deviation, and it is a simplification:** item 6's "one line at the
  skip/scrub/load landing sites" is **one** line, in `load` — every one of
  those sites funnels through it ("the one path every request takes"), and it
  takes the *request* after `load`'s clamp rather than a position reading, so
  the arm is spent before the seek is even issued. It also covers a frame step
  and a jump, which the three named sites do not. The detector lives in
  `bus/slates.rs` rather than `recording.rs`/`transport.rs`: it is slate
  behaviour, and that module's header is where the minted timestamp's exception
  is written down. Both of `transport.rs`'s contradictory comments are amended
  (item 7).
  **What the tests reach and what they do not:** the six harness tests include
  the forward-skip bug (which fails, with the measured wrong anchor, without
  the one-line disarm) and a take whose range is closed mid-take, which is the
  one reachable proof that the arm must be an id rather than a cached pair.
  Item 10's *inherited* arm and the `source_index` guard are **not reachable
  today** — the arm is only ever set where a take starts, and a take's source
  never changes — so both wait on C for a test; the unconditional assignment
  and the guard are in, with their reasons.
- **C — the preview.** **Landed.** `Command::JumpToSlate(Uuid)` and
  `Command::PreviewSlate(Uuid)` — **two commands, not one**: item 1's jump
  *pauses* (which is what D4 needs of it) and item 2's preview is that plus an
  arm plus a play, so one command cannot be both. The jump's body is
  `jump_to_clip`'s, **factored into `Bus::park_at` rather than copied**, so the
  two rows land the same way by construction. `ScanSpeed` takes the one extra
  condition (item 3), and the row selects on click — with Esc now clearing
  *both* lists' selections, which it had to gain, because the toggle was the
  only way out of a slate selection (item 4). The double-click previews, as spec
  P2 says, so #104's ask is met in a stronger form than it asked for.
  **One deviation, and it is a bug this task found in B's code:**
  `slate_out_reached` also requires `self.player.is_idle()` now. B's measured
  claim — after a flushing seek `query_position` returns `None` and then the
  seek's *target*, never the pre-seek value — does not cover a seek that has
  been **issued and not yet acted on**, which is exactly this iteration of the
  bus loop, because `preview_slate` parks and plays in one go. Measured: a range
  previewed from anywhere past its own out point read the position the coach was
  watching, fired at once, and stopped the preview on its first frame. A take
  can reach it too (space pressed inside the shoot's seek), so the guard belongs
  in the check and not in the preview. It is `step_frame`'s own condition, and
  it costs at most one poll.
  **Item 2's two unreachable cases are now tests** — the inherited arm and the
  `source_index` guard — each proved by sabotage to be the only test that fails
  when its line goes.
  **#104's right-click menu is not all shipped.** It gained *Jump to slate
  start*, which is also `JumpToSlate`'s live sender; the other three items are
  BACKLOG **#127**, because two of them (Record, Preview slate) *arm* a range
  from a row that may not be selected, and the span is the **selected** slate's
  — a rule that wants the review this plan got rather than a line written under
  task C.
- **D — the pass.** **Landed.** `slate_pass::next_slate` — *the first
  `timed && !shot` row at or after the selected one* — called from
  `main.rs::advance_pass` on `Event::Recording(RecordingStatus::Idle)`, which
  sets the selection and then parks with `JumpToSlate` (item 4), in that order,
  so the scrubber's span names the range the footage is heading for rather than
  the one just recorded. Items 1, 2, 3 and 6 as written, and `pundit-core` is
  untouched. The rule is a sibling lib module with its own tests for
  `slate_span.rs`'s reason — `main.rs` is wiring and has no `#[cfg(test)]`
  module at all — and `PassRow` exists because the *library* cannot see the
  window's generated `SlateRow`, which is in the binary: it is a borrowed
  projection of the two flags T1 put on that row, built at the one call site.
  **Two deviations. The first is the thing the plan did not reach, and without
  it the feature does not work:**
  1. **`R` has to shoot the selected range.** Item 5 says "`R` arms and space
     starts" and spec T5 says "a bounded take is two keys" — but `R` sent
     `ToggleRecording`, a plain take from where the player was heading. The
     right *frames*, under a clip with no `slate_id`: the range never read as
     `shot`, the footage never stopped at its out point (#114's arm is the
     shoot's, not `start_recording`'s), and item 2's "at or after" therefore
     parked on the same row **for ever**. So `Command::ToggleRecording` now
     carries `slate: Option<Uuid>`, the window passes whatever is selected, and
     **the bus** turns it into `shoot_slate`. Twenty-two harness call sites
     gained `slate: None`, which reads as what they are.
     **The window must not make that choice itself**, and this is measured by
     the code rather than guessed: `recording.rs` says "the UI's status can lag
     the bus's, so the bus decides: a second R during start-up cancels, as the
     user means", and `ShootSlate` is **refused** by the recording guard
     (`shooting_while_recording_is_refused`). A window branching on its own
     phase would send a `ShootSlate` into the take it meant to cancel, and the
     key would be swallowed. The *mode* stays visible, which is what the slates
     spec's §S6 asks of it: the highlighted row, and a transport button that
     reads **"Record range"**.
  2. **The advance needs no gate, as a consequence of 1.** The draft of this
     task had one: on `Idle` with *any* slate selected, a plain take would have
     thrown the footage to another range — the coach selects a row to rename it,
     records something unrelated at minute 70, and lands at minute 12. With `R`
     carrying the selection, **every take started while a range is selected is
     that range's**, so the selected row simply *is* the pass's cursor, a plain
     take has no cursor and advances nothing, and no flag says whether the take
     that ended was a slate's.
  **What it is tested by, and what it cannot be.** The plan's "Tests. Harness:
  shoot the first of three candidates and assert the second is next" is **not
  implementable**: the advance reads the row model and the selection, both UI
  state, and the harness has no window — which the spec's own crate table says
  ("`pundit-app`'s UI: … the pass's advance over ids"). So the rule is pinned by
  seven unit tests in `slate_pass.rs` (the advance, the abort, the half-marked
  skip, no going back, the end of the set, a filter change between takes, and a
  cursor that is not in the list), `R`'s two cases by the window test in
  `tests/slate_fields.rs`, and the bus's new branch by two harness tests in
  `tests/slates.rs`. The wiring in `advance_pass` is untested, as all of
  `main.rs` is.

---

## A. The span on the scrubber — first, because it is the only part the coach can see without the stop working

**Files:** `crates/pundit-app/ui/scrubber.slint`, `app.slint`, `main.rs`.

1. **Two floats on the `Scrubber`, not a field on `Mark`.** `in property <float>
   slate-span-from` / `slate-span-to`, and one `Rectangle` before the slider
   with `visible: to > from`. The draft put a `to` on `Mark` for "one field, one
   expression, one loop" and all three were wrong:
   - **the loop's `x` is `mark-x(mark.at) - self.width / 2`**, centred for a 2px
     tick, so a span would be drawn **half its own length to the left** — the
     left edge half a span before the in point. Two expressions, and the second
     is where the bug is.
   - **there is exactly one `Mark` construction** and it is a Rust struct
     literal in `show_match`, so "every existing construction keeps working"
     is false: it would not compile until `to: 0.0` were added. A compile error
     rather than a silent bug, but the opposite of what the draft promised.
   - **it is not a list.** The spec's S6 is one slate at a time. A span is two
     floats, and as its own element it is free to take its own height and
     opacity — which it wants, because a 12px tick-coloured bar over a region
     does not read like a tick.
2. **Drawn for the SELECTED slate, and the bus publishes nothing.** The draft had
   the bus publish the armed range *and* said it should serve the selected one —
   which cannot both hold, because the selection is UI-only. It does not need
   to: **the armed slate is always the selected slate** at every entry point (the
   Shoot button sends `root.selected-slate`, the preview comes from a row
   double-click, the pass selects the row it parks on). `on_show_slate` already
   fires on every selection change and already looks the slate up in the
   snapshot. So this is ~4 lines there plus the same two in the project-changed
   path, so a re-mark moves it — and **no new `Event`**.
3. **In concat time**, through `project.abs_seconds(source_index, …)`, because
   the scrubber's x is the concat timeline. `abs_seconds` is infallible and
   clamps the *index*, not the time — so an out-of-range index maps silently to
   `total + in_seconds`, off the end of the bar. Only reachable through a remap
   bug, but it fails quietly; worth a debug assert rather than a guess.

---

## B. The stop

**Files:** `crates/pundit-app/src/bus/{mod.rs, recording.rs, transport.rs}`.

### The mechanism

1. **`Bus.armed_slate: Option<Uuid>`, not a `(source_index, out)` tuple.** The
   tuple is a cache of two fields of a project the bus owns, and it goes stale:
   **`MarkSlateOut` is on the recording allow-list**, so the coach can press `o`
   mid-take and move the out point, and the cached value would stop at the old
   one. An id resolved at each check also makes a "cleared on a source change"
   clause unnecessary — a remapped slate resolves correctly, a deleted one
   resolves to `None` — and `Active.slate: Option<Uuid>` already exists, so a
   tuple beside it would be the second truth this task is meant to avoid.
2. **No deadline during a take: the bus already wakes 10 times a second.**
   `LEVEL_INTERVAL_NS` is 100 ms, and `bus/mod.rs`'s loop tail runs *"After every
   input, not only on a timeout, so a busy channel (level messages at 10 Hz)
   can't starve them."* So the check belongs in that tail and the take path needs
   no deadline at all.
3. **One bounded poll covers the preview path**, which is the only quiet one
   (plain playback posts nothing periodic). Recomputed each iteration in the
   `.min()` chain:
   ```rust
   .chain((self.armed_slate.is_some() && self.playing).then(|| Instant::now() + OUT_POLL))
   ```
   **No stored `Instant`, no `dispatch_deadlines` arm, no re-arm sites.** The
   timeout falls through `match input { None => {} }` to the tail, where the
   check lives. `OUT_POLL = LEVEL_INTERVAL`, so the constant says why it is that
   number. Worst case is one poll late — three frames — against the draft's
   unbounded "late".
4. **The check:** armed, playing, **no preview open**, the slate resolves, its
   source is `self.current`, and `query_position()` is at or past its out point.
   Then fire once and disarm.
   - `preview.is_none()` is not optional: `set_playing` acts on whichever
     pipeline is on screen while `query_position` always reads the **game**
     pipeline, so without it a stale arm would stop a *clip preview* dead, with
     no notice and no log.
   - **`query_position` is safe here, measured:** after a flushing seek it
     returns `None` for 0–15 ms and then the seek's **target** — never the
     pre-seek value — and the same across a source change. So there is no stale
     read to defend against, which is why there is no previous-position state.

### The bug the measurement exposed, and its fix

5. **A forward skip makes `query_position` read the skip's target within 5 ms, so
   a naive check fires and logs a pause where the footage is not.** Skip +6 s
   from 59.5 with `out = 60`: the position reads ~65.5, the check fires, and the
   log gets `Pause { source_time: 60.0 }` while the picture is at 65.5 — replay
   then freezes 5.5 s behind the footage the coach is talking over. The draft
   called an immediate fire "the honest answer, because the coach *is* past the
   range"; it is not honest, it writes a wrong anchor. BACKLOG #114 is explicit
   that a deliberate move past the out point must not trigger it.
6. **So the distinguisher is the EVENT, not the position: disarm without pausing
   when a landing's position is already past the out point.** One line at the
   skip/scrub/load landing sites. After it, the poll only ever sees a position
   reached by **playback**, anchoring at `out` is honest to the frame, and
   #114's rule is kept with no previous-tick state machine. A position tolerance
   cannot do this job — a missed poll and a deliberate skip look identical in
   position alone.

### The log, which is this feature's one genuine departure

7. **The bus mints the timestamp, and that contradicts a rule stated twice in the
   file being edited.** `transport.rs` declines to log a pause at `PlayerEvent::
   Error` and at EOS because *"it would need a bus-side time"*, and #114 repeats
   the demand for a caller-captured one. The spec argued `CLAUDE.md`'s contract
   away for **where the detector lives**; nobody carried it through to the
   **timestamp**. Carry it: the contract is about queue delay between an input
   event and its handler, **the deadline is itself the event**, nothing was
   queued, and the time wanted is "when the picture stopped", which is now. So
   `host_ns = pundit_media::now_ns()` on the bus thread — and **amend those two
   comments**, so the codebase ends with one rule rather than two contradictory
   ones. The distinction that makes it sound: those pauses have **nothing to
   anchor to**; this one has `out`.
8. **Call `active.log.pause(host_ns, out)` directly — NOT `log_playing`.**
   `log_playing` computes its anchor from `heading(ui_secs)`, which prefers
   `skip.target()`, then `player.target_secs()`, then the caller's value. So
   `log_playing(now, Some(out))` anchors at `out` only when the skip coordinator
   is idle *and* no seek is in flight — and `set_playing(false)` at a non-1×
   rate calls `change_rate(1.0)` → `load(...)` itself, making `target_secs()`
   `Some` **before** the log line runs. The anchor would be non-deterministic,
   and `EventKind`'s own doc says that anchor **overrides** the wall-clock cursor
   on replay. The draft's "as `toggle_play` does" was wrong twice: that guard
   lives in `toggle_play`, not in `set_playing` or `log_playing`.
9. **The state-change guard is the firing condition.** The check requires
   `self.playing`, so `set_playing(false)` always changes state and no second
   pause can be written. State that as the deliberate consequence rather than
   leaving it a coincidence a later edit can break. *(The draft cited
   `debug_assert_sorted` as the thing a duplicate would trip — it cannot:
   `RecordingLog::record_time` clamps every record time to at least the last
   event's, precisely so a late `host_ns` cannot unsort the log. The real cost of
   a duplicate is a redundant event and the tests that read the log's tail.)*

### The arm's lifecycle — four explicit sites, and there is no funnel

10. **`start_recording` assigns the arm unconditionally** — `self.armed_slate =
    from.map(|shot| shot.slate)` — so a plain `R` take always **clears** it.
    Without that, a live preview's arm is inherited and an `R` take pauses at a
    stale out point: the same bug as a refused shoot, through a second door.
    Assign it **after** `self.recording = Some(Active{…})`, since
    `create_dir_all` and `Recorder::start` can still refuse below that.
11. **Cleared on:** firing, a landing past the out point (item 6), wherever
    `recording` is cleared (`finish_recording` / `abort_recording`, which every
    stop path reaches), and a project open.
12. **The draft's advice here was unsafe and is retracted**: it said to prefer
    "one place they all pass through". **No such place exists.** The only hook
    project-open and every source change share is `reset_skip` — which is also
    called by a `J`/`L` press (disarming a live preview) and by
    `start_recording`'s own slate seek, *before* the arm is set. Clearing there
    breaks both features. Four explicit sites is the honest answer.

**Tests** (harness — reachable because the detector is in the bus, which it was
not in the spec's first draft):

- A take from a timed slate reaches its out point and **the clip's log ends with
  a pause anchored at `out`**. The assertion the feature exists for.
- **A forward skip past the out point disarms and never logs a pause** — item 6,
  and the draft's version of this test did not catch its own bug (it only
  asserted the pause was not *before* the skip target, which an immediate wrong
  fire satisfies).
- A half-marked slate's take never pauses.
- A plain `R` take never pauses: after a **refused** shoot, and after a **live
  preview arm**.
- A take on source 1 is not stopped by a slate on source 0 with a smaller out.

---

## C. The preview

**Files:** `crates/pundit-app/src/bus/{mod.rs, slates.rs}`, `main.rs`,
`app.slint`.

1. **`Command::JumpToSlate(Uuid)`**, which is BACKLOG **#104** — and its body is
   **`bus/clips.rs::jump_to_clip`**, not `transport.rs` (#104 names the function
   without its file and the obvious guess is wrong). That body is
   `reset_skip(); set_playing(false); if seekable() { load(index, secs, true,
   Origin::Scrub) }`, which is exactly what #104 describes. **#104 is resolved by
   this task.**
2. **Previewing is that, then play, then B's armed stop** — so the arm is not a
   take's alone.
3. **It forces 1× at the start, and `ScanSpeed` is refused while armed.** P3's
   "force 1×" is not enough on its own: `J`/`L` **are** allowed during a slate
   preview (`scan_speed` only requires `playing && preview.is_none()`), and at
   32× two things break — the poll overshoots by `rate × interval` (0.64 s at a
   100 ms poll), and `set_playing(false)` seeks back to `shown_secs()`, which
   "at 32x trails the position by up to 0.6 s", so the stop would land **before**
   the out point. One condition in `scan_speed`, for P3's own reason.
4. **A slate row selects on click**, the clip row's rule — it toggles today, so a
   double-click would jump and leave the row deselected. **#104 already settled
   this** ("Selecting on click, with deselection by Esc or by clicking empty
   space"), so follow it rather than re-opening it.

---

## D. The pass

**Files:** `crates/pundit-app/src/main.rs`, `app.slint`.

1. **No core predicate, and `pundit-core` stays untouched.** T1 already shipped
   the row model carrying both halves — `shot` and `timed` — with a comment that
   is the spec's T2 verbatim. A `core::is_unshot_pass_candidate` would be exactly
   the "second definition" that comment forbids.
2. **No id snapshot: "the next candidate row at or after the current one."** A
   shot slate **stays in the list**, so the displayed list is a stable frame of
   reference. On Stop, select the first row at or after the current slate's
   position that is `timed && !shot`. After a successful take the current row is
   now `shot`, so it moves on — the `[A,B,C]` skip-every-second bug is
   **structurally impossible rather than merely tested against**. After an
   aborted take the slate is still a candidate, so "at or after" parks on it
   again, which is the behaviour the draft needed a second sentence for. A slate
   marked mid-pass is picked up, and a filter change is honoured, because the
   list *is* the queue. It also removes a question the draft never raised: with a
   snapshot, something has to start and end the pass, and no task said what that
   UI was.
3. **Advance on `Event::Recording(RecordingStatus::Idle)`, not
   `ProjectChanged`.** The draft had it on `ProjectChanged` and that is wrong
   twice: `abort_recording` emits **only** `Recording(Idle)` — no
   `ProjectChanged`, because nothing changed to save — so the advance would never
   fire after an abort; and `ProjectChanged` fires on **every** unrelated change,
   including a transcript landing with no command behind it, which would move the
   footage under the coach's hands mid-pass. `finish_recording` emits
   `ProjectChanged` **then** `Recording(Idle)`, so the project is already updated
   when `Idle` arrives.
4. **The advance parks via `JumpToSlate`** (C1), which pauses. `ScrubRelease`
   would not, and the footage would run on through the next range.
5. **`R` arms and space starts, and the UI says so.** `start_recording` pauses
   the footage unconditionally and the log seeds a pause at record time 0, both
   deliberately. **This is the spec's open question for the coach**; do not
   change a documented rule to save a keypress unasked.
6. **Slates marked during the pass are not added to it** — `i`/`o` are on the
   recording allow-list, so this is reachable. Defensible (the pass is the set
   the coach started with, and item 2 picks up a new one at the next advance
   anyway), but say it rather than let it be an accident.

**Tests.** Harness: shoot the first of three candidates and assert the **second**
is next. Under item 2 that passes by construction, so the test is a regression
guard rather than a proof — which is the point of choosing a design where the bug
cannot be written.

*(As shipped this is a **unit** test and not a harness one: the advance reads the
row model and the selection, and the harness has no window. See "Where this
stands".)*

### What the whole feature leaves open

Recorded here because it is the last task, not because any of it blocks:

- **`R`'s new meaning is SETTLED, by the coach, 2026-10-04.** Asked directly
  ("r is for recroding i thought") and then offered the three ways out — keep
  it, revert the trigger to the slate row's Record button, or give the shoot a
  key of its own — he chose **keep it**: with a range selected, `R` shoots that
  range from its in point. So this is no longer an open question and should not
  be reopened without him.
  - **He had not approved it before.** Task D inferred it from spec T5 ("a
    bounded take is two keys") and from his earlier "keep R-then-space", which
    was an answer about *not auto-playing the footage* — a different question.
    The inference turned out to be right, but it was an inference, and D was
    right to flag it rather than let it pass as approved.
  - **It was also not optional**, which is the other half of why it stands: a
    plain `R` produced a clip with no `slate_id`, so the range never read as
    `shot`, #114's stop never armed, and "at or after" parked on the same row
    for ever. An infinite loop, not a nicety.
  - **The remaining risk is a stale selection** — select a row to rename it,
    press `R` without thinking, and the footage jumps to that range. The guards
    are the three already shipped: a second `R` always stops regardless of
    selection, the transport button reads "Record range", and Esc clears the
    selection. **If it bites in real use the trigger is a small revert** and the
    row's Record button already does the job; that is the fallback, written down
    so nobody has to re-derive it.

- **The spec's three open questions are still open**, and D touched none of
  them. **T5** — should the pass's own start play the footage, saving the space
  press — is the live one: the coach said "keep R-then-space" and both Record
  buttons' tooltips now say so, but nothing has been tried in anger.
  **T2's phrasing** (are the skipped rows marked, or filtered out) stands as
  shipped: half-marked rows render as `14:05–` and shot ones carry a `●`.
  **P** (does previewing want a key of its own) waits on #96.
- **The pass is silent at both of its ends** — it cannot be started from the
  keyboard (the first row is a click) and says nothing when it runs out of
  ranges. BACKLOG **#128**.
- **A shoot from an untimed row advances to the next timed one**, which falls
  out of "at or after" and is the one place the rule is presumptuous rather than
  obvious. Left alone: the row is in the list the coach is working, and the
  alternative is a case to remember.
- **#127**, the rest of the slate row's right-click menu, is untouched and still
  wants the review this plan got: *Record* and *Preview slate* from a row that
  may not be selected is the span invariant that D now leans on harder, since
  the selection is also the pass's cursor.

---

## One hazard that is not this feature's

**BACKLOG #72.** Every test above opens a project through the rig, and #72 is an
unresolved flake — thirteen sightings, every one `Rig::open`'s settle timeout,
including on branches with no Rust in them. These tests raise the exposure.
**The discriminator, so nobody re-debugs the detector:** the panic site is the
rig's settle wait, with `Position { source_index: 0, target_abs: Some(0.0) }` and
no settle inside the bound. If a CI run reddens there, rerun the job before
reading the diff. (#101 does not apply: it is the export binaries under load, and
nothing here touches an export path.)

---

## §R. What the first draft got wrong

1. **Its one original idea was unsound.** "The deadline is a hint; the position
   check is the truth" was justified by a table claiming a missed re-arm
   degrades to "pauses late" rather than "never". **"Late" is unbounded**: armed
   at `out = 100` with the playhead at 20, the deadline is `now + 80 s`; skip to
   95 and the out point passes 5 s later, but the first check lands **75 s late**
   — after the take ended. Indistinguishable from never. Capped, the estimate
   becomes pointless; the bounded poll is what remains.
2. **It did not notice the bus already wakes at 10 Hz during a take**, which is
   written down in the loop it proposed to extend.
3. **It cached `(source_index, out)`** when `MarkSlateOut` is on the recording
   allow-list, so the cache can go stale mid-take.
4. **It routed the log through `log_playing`**, which cannot anchor at `out`.
5. **It never said the bus must mint the timestamp**, against a rule stated
   twice in the file it edits.
6. **It called an immediate fire after a forward skip "the honest answer."** It
   writes a wrong anchor and replay freezes behind the footage — found by
   *measuring* `query_position` after a seek.
7. **It missed five preview-path endings**, including an `R` take inheriting a
   preview's arm — its own trap 4 through a second door.
8. **Its span would have been drawn half its length to the left**, and its
   "every existing construction keeps working" was false.
9. **It had a Traps section that was 8/8 restatement of its own task items**,
   one of which (`debug_assert_sorted`) named a guard that cannot trip. Deleted;
   the spec's §R records deleting such a section for being 5/6 restatement, and
   this one was worse.

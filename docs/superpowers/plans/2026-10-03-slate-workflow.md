# Plan — the slate workflow's hard half: the out-point stop, the preview, the pass

Spec: `docs/superpowers/specs/2026-10-02-slate-workflow-design.md`, revised
through two adversarial passes. **Read it before any task; this plan does not
repeat its reasoning.** Read `CLAUDE.md` too.

**M and T1 have shipped** (PRs #29, #30): the mark buttons and the slates' tag
filter. They were ordered first because they are independent of everything here.
What is left is the half the spec's review found a bug in nearly every paragraph
of — so this plan is mostly about the order in which the hazards are closed.

**Gates.** Every `cargo` call except `fmt` goes through
`flock /tmp/claude-1000/cargo.lock nice -n 19 cargo …` — other sessions build
here. **Never pipe clippy or a test run to `tail`**: it masks the exit status.
**And clippy must be run on CI's toolchain**: `rustup run 1.92 cargo clippy
--workspace --all-targets -- -D warnings`. A clean local 1.98 is not the gate
(`CLAUDE.md` says why, and it cost a red CI run on 2026-10-02).

**`pundit-media` is not touched by any task**, and `pundit-core` only by T3's
predicate. The dependency audit must still print exactly `serde`, `serde_json`,
`thiserror`, `uuid`.

## Where this stands (update it as tasks land)

- **S1 — the armed range and the position check.** Not started.
- **S2 — the deadline, and re-arming.** Not started.
- **S3 — the scrubber span.** Not started.
- **P — `JumpToSlate`, and the preview.** Not started.
- **T — the pass.** Not started.

---

## The one design decision this plan adds to the spec

**The deadline is a hint; the position check is the truth.**

The spec says the bus arms an out-deadline on the existing
`recv_timeout(min(deadlines))` loop. It does not say what happens when the
estimate is wrong — and it will be, because the wall-clock time to the out point
depends on the position *and* the rate, and six events change one or the other.

So: when the deadline fires, **query the position and decide**. If the out point
has not been reached, re-arm for the remaining estimate and do nothing else.

This turns the spec's "re-arming is the whole risk" from a correctness problem
into a *latency* problem:

| | with an exact deadline | with a hint plus a check |
|---|---|---|
| estimate too early | pauses early — **wrong** | re-arms — no harm |
| estimate too late | pauses late | pauses late |
| a re-arm is missed | **never pauses** | pauses late |

A missed re-arm becoming "late" instead of "never" is worth the one extra
position query per firing, and `PositionHandle::query_position` already exists
and already returns the position **within the current source**, which is the
quantity S1 needs.

**It also makes the arm cheap to reason about:** nothing has to be re-armed
*correctly*, only *eventually*. The re-arms are then an optimisation for
latency, and a forgotten one is a bug worth fixing but not a feature that
silently does nothing.

---

## S1. The armed range, and the check that fires the pause

**Files:** `crates/pundit-app/src/bus/{mod.rs, recording.rs, transport.rs}`.
**Gate:** the full workspace; clippy on 1.92.

1. **`Bus.armed_out: Option<(usize, f64)>`** — the source index and the out
   point. **Both halves are required**: a slate belongs to one source
   (`Slate::source_index`), and comparing a time alone pauses the footage in the
   *next* video wherever its time passes the out point. That is reachable: on the
   preview path `end_of_stream` advances to the next source at 0 s.
2. **Armed where the take is**, in `start_recording`, from the `Shot` it already
   has — **not from the UI**. `RecordingStatus` carries no slate, so an arm the
   UI kept from its own click is a parallel truth the bus can contradict:
   `start_recording` still refuses *after* the click, on `NoCamera`, on the
   recordings `create_dir_all`, and on `Recorder::start`. The spec's review
   traced the consequence — camera unplugged, Record on slate A, refusal, then a
   plain `R` take **pauses at A's out point**.
3. **Cleared on every exit**: the recording stopping or aborting, a project open,
   a source change. `armed_out = None` is the safe state, so prefer clearing it
   in one place that all of those pass through over three that might not.
4. **Only a *timed* slate arms.** `out_seconds: None` arms nothing and behaves
   exactly as today.
5. **The check, on the deadline firing:** `query_position()`, and fire when the
   source index matches **and** the position is at or past the out point **and**
   the footage is playing. Not `prev < out && cur >= out`: the spec's first draft
   used that and a **forward skip satisfies it perfectly**, pausing past a range
   the coach deliberately skipped into. With the position check there is no
   previous-tick state to be fooled, and a skip past the out point simply means
   the next firing sees it already past — which is the honest answer, because the
   coach *is* past the range.
6. **The pause is the bus's own `set_playing(false)` plus the log**, logged
   **only on a state change**, as `toggle_play` does. No new command: the spec's
   draft added `Command::Pause` and it would have been **silently refused** by
   the recording allow-list, which is exhaustive by design.
7. **Anchored at `out`**, not at the queried position: `out_seconds` is a stored
   field, so there is nothing to capture, and `ShootSlate` sets the precedent
   ("the in point is a stored field, not a reading of the playhead"). The queried
   position is how we know we got there; the anchor is where the range ends.
8. **Fires once**, then `armed_out = None`. A coach who plays on past the out
   point is not stopped again — they went there.

**Tests** (harness, which is now reachable because the detector is in the bus —
it was not in the spec's first draft, where the tick made it invisible to CI):

- A take from a timed slate crosses its out point and **the clip's log ends with
  a pause anchored at `out`**. This is the assertion the whole feature exists
  for.
- A **forward skip past** the out point does not pause before the skip's target.
- A **half-marked** slate's take never pauses.
- A plain **`R` take** never pauses — including the one the review found: after a
  refused shoot of a timed slate, with the camera gone.
- A take on **source 1** is not stopped by a slate on source 0 with a smaller out
  point.

## S2. The deadline, and the re-arms

**Files:** `crates/pundit-app/src/bus/mod.rs` (the loop and
`dispatch_deadlines`), plus wherever each re-arm site lives.

1. **`out_deadline: Option<Instant>`** joins the `.min()` chain beside
   `skip_deadline` and `start_deadline`, and `dispatch_deadlines` gains its arm.
   Existing machinery, one more entry.
2. **The estimate is `(out - position) / rate`**, floored at a small minimum so a
   bad estimate cannot spin the loop. Say the floor and why in the code.
3. **Re-armed on: play, pause, a skip landing, a scrub landing, a rate change,
   and a load.** Each is a latency fix, not a correctness one (see the decision
   above) — so **write the re-arms as one call at the end of the handlers that
   change position or rate**, rather than six scattered sites that have to be
   remembered.
4. **`ScanSpeed` is refused while recording**, so the rate cannot change mid-take
   — but it can on the preview path, which has no such rule. The estimate must
   therefore read the rate rather than assume 1×.

**Sabotage proof.** Delete one re-arm — the one after a skip lands — and confirm
the pause still happens, **late**, rather than not at all. That is the whole
claim of this plan's design decision, and if it fails, the decision is wrong and
the exact-deadline form is back on the table.

## S3. The scrubber span

**Files:** `crates/pundit-app/ui/scrubber.slint`, `app.slint`,
`crates/pundit-app/src/main.rs`.

1. **`Mark` gains `to: float`**, with `to <= at` meaning a point, and the
   existing loop's `width: max(2px, mark-x(to) - mark-x(at))`. **One field, one
   expression, one loop** — not a second struct, a second property and a second
   loop. Slint struct fields default to zero, so every existing `Mark`
   construction keeps working as a point.
2. **The armed *or selected* slate's range**, which is what makes it serve three
   features: the take's stop (it is the signal the coach said they expected), the
   re-marking from M (otherwise they re-mark against a text row), and P's preview
   (which has no take at all). One slate at a time, so the overlap question does
   not arise.
3. **In concat time.** `Mark.at` is `row.abs`, the concat timeline, so a span
   built from `in_seconds`/`out_seconds` raw lands at the wrong place for every
   slate not on the first video. Map through `project.abs_seconds(source_index,
   …)`.
4. **The bus publishes the armed range** so the UI can draw it, which is the same
   plumbing S1's arm needs and should be one thing, not two.

## P. `JumpToSlate`, and the preview

**Files:** `crates/pundit-app/src/bus/{mod.rs, slates.rs}`, `main.rs`,
`app.slint`.

1. **`Command::JumpToSlate(Uuid)`** — BACKLOG **#104**, which this subsumes
   rather than duplicates: *"`jump_to_clip`'s body with a slate's fields:
   `reset_skip`, pause, then `load(source_index, in_seconds, …, Origin::Scrub)`
   if `seekable()`"*. **That body is `bus/clips.rs::jump_to_clip`, not
   `transport.rs`** — checked, because #104 names the function without its file
   and the obvious guess is wrong. It belongs beside it or in `slates.rs`; put it
   where a reader looking for "jump to a thing" will find both.
2. **Previewing is that, then play, then S1's armed stop** — with no recording in
   flight. So the arm is **not** only a take's: it is `armed_out`, set by either.
3. **It forces 1×.** The clip preview already returns to 1× with no seek; a coach
   scanning at 16× would otherwise overshoot by half a second of footage.
4. **A double-click on a slate row previews it**, which is #104's own ask
   (it asked for a jump; a preview is a jump that plays). Note that a row's click
   **toggles** the selection, so a double-click must not leave the row
   deselected — #104 flags this and it is the one UI subtlety here.
5. **#104 is marked resolved by this task**, not left open.

## T. The pass

**Files:** `crates/pundit-core/src/project.rs` (the predicate), `main.rs`,
`app.slint`.

1. **A predicate in core**: timed, and unshot. `pub fn is_unshot_pass_candidate`
   or similar on `&Slate` + `&[Clip]` — **not a "queue"**, which would be a
   second definition of what `show_slates` already computes.
2. **The pass holds the ordered slate *ids* it started with**, and advances to the
   next id still a candidate. **An index-based next is wrong and the review
   traced it**: stopping a take adds a clip carrying `slate_id`, so the shot slate
   leaves the candidate set in the same `ProjectChanged` the advance reacts to —
   `[A,B,C]`, shoot A, set becomes `[B,C]`, index+1 is **C**, and **B is never
   offered**. An *aborted* take produces no clip, so its slate stays a candidate
   and must not be skipped either.
3. **The advance parks via `JumpToSlate`** (P1), which pauses. `ScrubRelease`
   would not, and the footage would run on through the next range.
4. **`R` arms and space starts, and the UI says so.** `start_recording` pauses
   the footage unconditionally — "Every clip starts on a still frame" — and the
   log seeds a pause at record time 0. **This is the spec's open question 1**; do
   not change the documented rule to save a keypress without the coach asking.
5. **The bound is in the data**, `out_seconds.is_some()`, not carried on a
   command. The spec's draft carried it and claimed two modes had to coexist;
   under S1's shape that is false, because pausing the picture costs the coach
   nothing.

**Tests:** core's predicate as a table. Harness: shoot the first of three
candidates and assert the **second** is the next one — the bug in item 2, which
an index-based implementation passes only by accident.

---

## Traps

Each is a bug that compiles cleanly, and every one is from the spec's review.

1. **A slate carries `source_index`.** Compare it, or the pause lands in the next
   video.
2. **`prev < out && cur >= out` is satisfied by a skip.** Do not write it; the
   position check has no previous state to fool.
3. **A new `Command::Pause` would be silently refused mid-take** by the
   exhaustive recording allow-list. Do not add one.
4. **An arm kept by the UI outlives a refused take.** Arm in the bus.
5. **`toggle_play` logs only on a state change.** An unconditional log writes a
   second pause event into a log that `debug_assert_sorted` and several tests
   read.
6. **An index-based advance skips every second slate.**
7. **`Mark.at` is concat time.** Map the span through `abs_seconds`.
8. **The harness cannot see the UI tick.** It can see the bus, which is why the
   detector is there — do not "simplify" it back into `main.rs`.

## What must be proven by sabotage

1. **S2's deleted re-arm** (above): the pause must come **late**, not never.
2. **T's index-based advance**: implement `index + 1` deliberately and confirm the
   second candidate is skipped, then fix it. The test must fail on the bug and
   pass on the id-based form, or it is not testing the rule.

## Open questions still with the coach

- **T4**: should the pass's start play the footage, or is `R`-then-space right?
- **T2**: are the skipped rows marked, or filtered out? (Both states are
  **already visible** — a `●` for shot, a trailing dash for an open range — so
  this may need nothing.)
- **P**: does previewing want a key? `b e g k m n p q s t u w` are free.

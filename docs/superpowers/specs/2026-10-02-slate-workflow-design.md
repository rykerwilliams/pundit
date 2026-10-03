# The slate workflow — stopping at the out point, adjusting the marks, and the themed pass

**BACKLOG #114, #119, #120 and #104**, specced together because they touch one
panel, one stored struct and — for two of them — one mechanism.

**Revised through two adversarial passes.** The first draft got the central
architecture wrong and shipped seven bugs on paper; **§R** keeps the record,
because several were wrong in ways worth not repeating.

The coach, over one session on 2026-10-02, and their choices:

| | |
|---|---|
| #114's shape | **Pause the footage at the out point, keep recording — and show the range** |
| #120's advance | **Advance to the next slate and park, waiting for `R`** |
| #119's control | Re-mark from the playhead — **as two buttons, not `i`/`o`** (see M1) |
| the detector's home | **The bus** (see S2) |

Read #114, #119, #120 and #104 first; this does not repeat their reasoning.

---

## What already exists, read rather than assumed

- **A slate is `{ id, source_index, in_seconds, out_seconds: Option<f64>, name,
  tags }`** (`project.rs:420`). **One source per slate** — which the first draft
  missed, and which the whole detector turns on.
- **`selected-slate` already exists** (`app.slint:3514`): UI-only, cleared on
  project open and when its slate leaves the project, driving the row highlight
  and the editor row below the list. A click **toggles** it.
- **The slate editor row already exists** with its two `LineEdit`s and the
  Shoot button (`app.slint:4803-4845`), already folded into `text-editing`.
- **`i` / `o` match `"i" || "I"` deliberately** (`app.slint:4355`), so a shifted
  or caps-locked press still marks. Shift+`i` is **not** free.
- **The slates spec's §S6 already decided** that `i`/`o` do not re-time a
  selected slate: "One key cannot both create and re-time, and the mode deciding
  which would be invisible state on the primary key."
- **`SlateEdit` is `Name | Tags`** and derives **`Eq`** (`project.rs:435`).
- **`SlateError::OutBeforeIn` and the `out <= in` rule already exist**
  (`project.rs:443-456`), surfaced by `mark_slate_out` as a notice.
- **`start_recording` pauses the footage unconditionally** — "Every clip starts
  on a still frame" (`recording.rs:124-127`) — and `RecordingLog::new` seeds a
  `Pause` at record time 0. So `R` parks a take **paused**.
- **A scrub is impossible during a take**: the scrubber is `enabled: can-play &&
  !recording`, and `ScrubMove`/`ScrubRelease` are absent from the recording
  allow-list. **Only skips** move the playhead mid-take.
- **The recording guard is an exhaustive allow-list** (`bus/mod.rs:914-951`):
  "everything not listed is refused, so commands added later are too."
- **`RecordingStatus` is `Idle | Starting | Recording { t0_ns }`** — it carries
  **no slate**. The bus knows (`Active.slate`); the UI does not.
- **The scrubber's `Mark` is a point** (`at`, `color`, a 2px rectangle drawn
  before the slider so the thumb paints over it, `scrubber.slint:16-19, 74-81`),
  and `at` is **concat time** (`main.rs:1699`).
- **`show_slates` already renders `slates_sorted()`** — source then in-point,
  i.e. footage order — and already computes `shot` per row (`main.rs:3423-3451`).

---

## S. Stopping at the out point (#114)

**S1. The footage pauses at the out point; the recording does not stop.**

The coach's choice, and it leaves the slates spec's §S5 standing: S5 refused to
bind `out_seconds` because "binding it would mean stopping a recording the coach
is still talking over" — an argument about the *recording*, not the *footage*.

**Verified by running code rather than argued:** a take with a pause at its end
replays as `[Freeze, Play, Freeze]` with `source_time` holding at the out point,
because `playback_segments`' tail covers the rest of the take; and a stroke
mid-draw when the pause lands is unaffected, since `visible_strokes` is keyed on
record time alone. That is why this shape is cheap.

**S2. The bus detects the crossing, and the first draft's argument for the UI
tick was wrong.**

The draft deduced the UI tick from `CLAUDE.md`'s bus contract. **That deduction
does not hold**: the contract is about queue delay between a user's *input
event* and the handler that stamps it, and a crossing has no input event — there
is nothing queued for the delay to affect.

The real reasons are better and they point the other way:

- **The bus owns the player, the rate, the position and `Active.slate`**, so it
  sees every seek, skip and rate change **first-hand**. The UI sees them late
  and through `shown_position`, which returns `locate(target_abs)` while a seek
  is outstanding — which is exactly how the draft's detector mistook a skip for
  a crossing (§R).
- **It already has the shape.** The run loop is `rx.recv_timeout` over
  `min(skip_deadline, start_deadline)` with `dispatch_deadlines`, so an
  **out-deadline is an existing mechanism**, not a new one.
- **It is testable.** `pundit-harness` drives the bus headlessly and never runs
  `main.rs`'s timer, so a tick-based detector is **invisible to CI** — which is
  how the draft's bugs survived its own review.

**The named cost is re-arming**: play, pause, a skip landing, a rate change and
a load all change when (or whether) the out point will be reached. That is real
added complexity and it is the price of the three bugs it removes.

**S3. The armed range is `(source_index, out_seconds)`, and both halves are
required.**

A slate belongs to one source. Comparing a time alone pauses the footage in the
*next* video wherever its time passes the out point — reachable on the preview
path, where `end_of_stream` advances to the next source at 0 s.

**S4. No new transport command.** The bus pauses the footage itself; it does not
need to send itself a command, and `Command::Pause` would have been **silently
refused mid-take** by the allow-list — the draft's one-line addition was a
feature that could never fire. The pause is logged exactly as `toggle_play` logs
one, **only on a state change**, so an already-paused take gains no second event.

**S5. The pause is anchored at `out`, not at a position reading.** `out_seconds`
is a stored field, so there is nothing to capture. This is `ShootSlate`'s own
precedent: it "carries **no** captured position: the in point is a stored field,
not a reading of the playhead."

**S6. The range is drawn on the scrubber for the armed *or selected* slate.**

The coach thought this already existed; it never did, and without it a take that
pauses is indistinguishable from one that stalled. Drawing it for the **selected**
slate too costs nothing extra and serves the two features that need it most —
re-marking (M) and previewing (P) both work on a range with no take in flight.

Two things it requires: a span shape (the existing `Mark` is a point), and
mapping through `project.abs_seconds(source_index, …)`, because `at` is concat
time. One slate at a time, so the overlap question does not arise.

### Deferred from S
A countdown or flash at the out point; drawing *every* slate's range, which
brings the overlap question with it.

---

## M. Adjusting the marks (#119)

**M1. Two buttons in the slate editor — "Set in" and "Set out" from the
playhead — not `i`/`o`.**

The coach chose "re-mark from the playhead"; this is that, by the cheaper route.
`i`/`o` are **not** available: they match `"i" || "I"` on purpose, so Shift+`i`
already marks, and splitting that branch would take behaviour away. **The slates
spec's §S6 already decided this** and its reasoning stands — one key cannot both
create and re-time. In this app Shift also already means *more* on the transport
(Shift+arrow is a 20 s skip), not *edit*.

The editor row, the selection and the `text-editing` fold all already exist, so
two buttons need no key branch, no modifier and no new state.

**M2. Two `SlateEdit` variants, no format change — and `Eq` has to go.**
`in_seconds`/`out_seconds` already exist and `edit_slate` is the one mutation
path. But `SlateEdit` derives `Eq`, and an `In(f64)`/`Out(f64)` variant cannot:
`f64: !Eq`. Nothing needs it. *(The draft claimed safety came from
`purge_for_source_change`'s exhaustive match — that match is over `UndoAction`,
not `SlateEdit`. Adding variants is safe because every slate edit funnels into
one `UndoAction::EditSlates` whole-list snapshot, which is already listed as
holding stale indices.)*

**M3. The ordering rule reuses `SlateError::OutBeforeIn` and its `<=`**, in
**both** directions — a "Set in" after the out point is the likelier mistake, and
nothing guards an in point today. A half-marked slate constrains nothing.

**M4. Where the check goes is the one real design choice in M.**
`Project::edit_slate` is infallible and `Bus::edit_slates` returns silently when
nothing changed — by design — so a refusal has nowhere to surface. Either
`edit_slate` becomes `Result<(), SlateError>` (two existing call sites) or the
check happens in `bus::edit_slate` before the mutation. **The spec picks the
bus**: core keeps its infallible setter, and the bus is already where
`UserError::Slate` notices come from.

---

## P. Previewing a slate (#119, #104)

**P1. It is not the preview pipeline.** `OpenPreview` takes a **clip** id and
composites that clip's recording; a slate has no recording.

**P2. It is #104's command plus a play.** #104 already specifies the park:
`reset_skip`, pause, then `load(source_index, in_seconds, …, Origin::Scrub)` —
i.e. `Command::JumpToSlate(Uuid)`. Previewing is that, then playing, then S's own
armed stop. **#104 is therefore subsumed**: a double-click on a slate row
previews its range.

**P3. It forces 1×.** The clip preview already returns to 1× with no seek; a
coach scanning at 16× would otherwise overshoot by half a second of footage.

**P4. It stops rather than pausing-and-continuing** — there is no sentence to
finish, and the last frame of the range is what they wanted to look at.

---

## T. The themed pass (#120)

**T1. The pass is the Slates list, tag-filtered**, on the clips list's shape. Its
order is the list's, which is already `slates_sorted` — footage order, which is
already right.

**T2. Only *timed* slates (`out_seconds.is_some()`) and only *unshot* ones, and
the rows must SAY so.** The draft claimed "the queue is the filtered list, so
what the coach sees is what they will work through" while excluding two classes
that stay in the list — rows the pass silently skips. `SlateRow` already carries
`shot`; nothing in the row reads it yet. **The rows show both states**, so the
list and the pass cannot disagree, and the queue reads the row model rather than
rescanning.

**T3. The pass holds the ordered slate *ids* it started with.** An index-based
"next" breaks the moment it is used: stopping a take adds a clip carrying
`slate_id`, so the shot slate leaves an unshot queue in the same
`ProjectChanged` the advance reacts to — `[A,B,C]`, shoot A, queue becomes
`[B,C]`, index+1 is **C**, and **B is never offered**. An *aborted* take produces
no clip at all, so its slate stays unshot and must not be skipped either.

**T4. After Stop it parks on the next slate's in point, via #104's
`JumpToSlate`** — which pauses, where `ScrubRelease` would not and the footage
would run on through the next range.

**T5. `R` arms and space starts, and that is stated rather than discovered.**
`start_recording` pauses the footage unconditionally and the log seeds a pause at
record time 0, both deliberately. So a bounded take is two keys. **Open question
for the coach:** should the pass's own start play the footage, saving a press per
slate, or is R-then-space right? Changing it argues against a documented rule.

**T6. The bound is in the data, not on the command.** `out_seconds.is_some()`
is the bound; an untimed slate has no end and behaves as today. The draft carried
a bound on the command and claimed two modes had to coexist — under S1 that is
false, because pausing the picture costs the coach nothing, so there is no
free-running mode to preserve.

**T7. The slates' tag filter is its own property, not the window's
`tag-filter`.** The clips list's is **written by the bus** on `Event::Select` to
un-hide a newly selected clip, which could clear the pass's filter mid-pass.

---

## Crate responsibilities

- **`pundit-core`**: the two `SlateEdit` variants (M2), and a predicate on
  `&Slate` + `&[Clip]` for "timed and unshot" — not a "queue", which would be a
  second definition of what the row model already computes.
- **`pundit-app`'s bus**: the out-deadline, the arm and its re-arming (S2, S3),
  the logged pause (S4, S5), `JumpToSlate` (P2), and the slate edits with their
  refusal (M4). The bus publishes the armed range so the UI can draw it.
- **`pundit-app`'s UI**: the span (S6), the tag filter (T7), the row states (T2),
  the two editor buttons (M1), and the pass's advance over ids (T3).
- **`pundit-media`**: **nothing.** No pipeline changes, no new pixels.

## Testing

Now that the detector is in the bus, the test that proves S1 end to end is
**reachable**, which it was not in the draft:

- Harness: a take from a slate crosses its out point and the clip's event log
  **ends with a pause anchored at `out`**; a forward skip past the out point does
  **not** fire it; a half-marked slate's take never pauses; a plain `R` take
  never pauses; a take on source 1 is not stopped by a slate on source 0.
- Harness: the refused mark move emits `UserError::Slate` and changes nothing.
- Core: the timed-and-unshot predicate, and the mark edits including both
  inversion directions.

## Risks

1. **Re-arming is the whole risk** (S2's named cost): six events change when the
   out point will be reached, and a missed one is either a pause that never comes
   or one that comes late. It is why the harness assertions above are per-event
   rather than one happy path.
2. **No format bump, no `state.json` key** — so nothing here can corrupt a
   project or a preference. Worth stating because it bounds the damage any of
   this can do.

## Open questions for the coach

1. **T5: should the pass's start play the footage**, or is `R`-then-space right?
2. **T2: are the skipped rows marked, or filtered out** — and if filtered, how
   do you get back to a half-marked one to finish it?
3. **P: does previewing want a key?** `b e g k m n p q s t u w` are free (`p` is
   the obvious one); #96 owns the keyboard question but the letters exist.

## R. What the first draft got wrong

Kept because several were wrong in instructive ways.

1. **The detector's home was deduced from the wrong premise.** The bus contract
   is about queue delay after an input event; a crossing has none. The right
   argument is that the bus sees seeks first-hand and is testable.
2. **A slate's `source_index` was omitted from its own description**, so the
   detector compared a time across a multi-video timeline.
3. **`prev < out && cur >= out` mistook a skip for a crossing**, because
   `shown_position` returns `locate(target_abs)` mid-seek. The draft *named* that
   hazard and then wrote the rule it breaks.
4. **`Command::Pause` could never have fired** — not on the recording
   allow-list.
5. **"There is no slate selection today"** — there is, and it already has the
   exact properties the draft proposed to invent.
6. **The arm outlived a refused take**, so a plain `R` take would have paused at
   a stale out point, violating the draft's own X2.
7. **The index-based advance skipped every second slate** (T3).
8. **Two fact errors relayed**: "every letter is taken" (twelve are free), and
   "a third of a frame at 30 fps" — 33 ms is exactly one frame.
9. **A section of refusals and a section of risks were 5/6 and 3/4
   restatement.** Both are now one short section each.

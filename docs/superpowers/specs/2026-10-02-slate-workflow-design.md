# The slate workflow — stopping at the out point, adjusting the marks, and the themed pass

**BACKLOG #114, #119 and #120, specced as one piece**, because all three want the
same thing underneath: *play this range and stop*. Written apart they would write
that three times.

The coach, over one session on 2026-10-02:

- "when i record a clip, it doesn't stop at the end of the clip" (#114)
- "preview and edit in out times" (#119)
- "i want to be able to filter slates on tags too, then record all the slates
  with that tag. the use case is like 'all these clips are corner kicks' or
  similar" — and, clarifying, "it should only show the timed slates, and not
  'free record' like the current slate recording action" (#120)

**Three decisions came from the coach before this was written**, and the spec is
built on them rather than proposing alternatives:

| | chosen |
|---|---|
| #114's shape | **Pause the footage at the out point, keep recording — and draw the range during the take** |
| #120's advance | **Advance to the next slate and park there, waiting for `R`** |
| #119's control | **Re-mark from the playhead: `i` / `o` with the slate selected** |

This spec does **not** repeat the three backlog entries' own reasoning. Read them
first; they carry the alternatives that were rejected and the coach's words.

---

## What already exists, checked rather than assumed

- **A slate is `(in_seconds, out_seconds: Option<f64>)` on `Project.slates`**,
  marked by `i` / `o`. `i` stores it with `out_seconds: None`, deliberately, so a
  half-marked range is a row that can be finished or deleted rather than UI state
  that dies with the app.
- **Shooting a slate seeks to its in point and then runs free** until Stop
  (`start_recording`'s `from`). Nothing watches the out point.
- **`SlateEdit` is `Name | Tags`.** There is no way to move a mark today —
  `in_seconds` and `out_seconds` are never written after `o` closes the range.
- **Slates are not on the scrubber at all.** `main.rs` shows them only in the
  Slates list, as text (`slate_range`). The scrubber's `match-marks` are match
  events.
- **"Has this slate been shot?" is already cheap**: a scan of the clips for
  `Clip.slate_id == slate.id`, which stays right across a delete, an undo and a
  re-record.
- **The clips list already filters by tag**, through the window's `tag-filter`,
  and `tag_vocabulary` is deliberately clips ∪ slates so a tag invented on a
  slate autocompletes.

---

## S. Stopping at the out point (#114)

**S1. The footage pauses at the out point; the recording does not stop.**

The coach's choice, and it keeps the slates spec's S5 reason intact: S5 refused to
bind `out_seconds` because "binding it would mean stopping a recording the coach
is still talking over" — which is an argument about the *recording*, not about the
*footage*. Pausing the picture takes nothing away from the sentence being spoken.

**A commentary pause is already an ordinary event in a take** (every recording
opens with one), so the clip replays, previews and exports with no special case.
That is the whole reason this shape is cheap.

**S2. The UI tick detects the crossing, not the bus — forced by the bus
contract.**

`CLAUDE.md`'s bus contract: any command that lands in the commentary event log
carries its timestamp and source anchor **as a field, captured at the input event
on the UI thread, never assigned by the bus handler**, because queue delay is the
drift that puts drawings behind the ball on replay.

A pause at the out point lands in the event log. So it cannot be a pause the bus
decides to issue with its own clock. The UI tick already reads the shown frame's
source time once a tick for the scoreboard and the highlight rings; it is the only
place that holds both halves — the frame on screen and a `now_ns()` to stamp it
with.

**The cost is the tick's granularity: 30 Hz, so up to 33 ms late.** Stated rather
than hidden. It is the same granularity the scan board and the rings already
follow the footage at, and a third of a frame at 30 fps.

**S3. The trigger is *crossing* the out point while playing forward, once per
take — not `position >= out`.**

A skip or a scrub during the take can move the footage past the out point or back
before it **on purpose**: the coach went there. `position >= out` would re-pause
every tick after the first, and would fight a coach who skipped back to re-watch
something.

So: remember the previous tick's source time, and fire when the previous was
`< out` and the current is `>= out`, while playing forward, and only if this take
has not already fired. A half-marked slate (`out_seconds: None`) has no end and
behaves exactly as today. A plain `R` take — not from a slate — has no range and
is untouched.

**S4. `TogglePlay` cannot be used for it, and this is the one new command.**

The existing pause/play command is `TogglePlay { host_ns, source_secs }`. A toggle
is wrong here: if the coach has already paused the footage themselves before the
crossing, a toggle would **start it playing** at the moment the range ended, which
is the opposite of the feature.

So `Command::Pause { host_ns, source_secs }` — an idempotent pause, carrying the
same two caller-captured fields for S2's reason. `TogglePlay` stays as it is; this
is not a refactor of it, because every keyboard path genuinely wants the toggle.

**S5. The range is drawn on the scrubber during the take, and this is half the
fix.**

The coach said "i thought we were signaling the end of the slate during
recording". It never existed. Without a visible end, a take that pauses is
indistinguishable from a take that stalled — which would turn S1 from a feature
into a bug report.

**The scrubber's `Mark` is a point, not a span** (`at`, `color`, drawn as a 2px
Rectangle), so a range needs a new shape: `export struct Span { from: float, to:
float, color: color }`, drawn before the marks for the same stated reason the
marks are drawn before the slider — so the thumb paints over them.

**Only the take's own slate is drawn, not every slate.** A match has dozens; the
scrubber is ~1000px wide at best, and the one range that matters during a take is
the one being shot. Outside a take, no span is drawn — which is also why this
needs no decision about overlapping ranges.

### Deferred from S

- **A countdown or a flash at the out point.** The span plus the pause is enough
  signal to test; a countdown is a second mechanism for the same job.
- **Drawing every slate on the scrubber outside a take.** Wanted eventually (it
  is how a coach would see the shape of a match), but it brings the overlap
  question with it and is not this feature's.

---

## M. Adjusting the marks (#119)

**M1. `i` and `o` move the selected slate's marks, from the playhead.**

The coach's choice, and it needs no new control: the same two keys that made the
slate move its marks when one is selected. Frame-accurate, because the playhead
already is.

**M2. Two new `SlateEdit` variants, and no format change.** `in_seconds` and
`out_seconds` already exist; `Project::edit_slate` is already the one mutation
path. `purge_for_source_change`'s staleness test is a deliberately **exhaustive**
match, which is what stops a new edit kind being admitted in silence — so adding
variants is safe there by construction.

**M3. The rule has to say what "selected" means, and there is no slate selection
today.** The Slates list has rows; the clips list has `selected-clip`. A slate
selection is new state, and it is UI state (not stored): which row the coach is
working on is not a property of the match.

**M4. `i` with a slate selected is ambiguous, and the ambiguity is resolved in
favour of the new slate.** Today `i` starts a *new* range. If a selected slate
made `i` move its in point, a coach who had clicked a row an hour ago would mark
nothing when they meant to.

So: **`i` always starts a new range.** Moving a mark is `i`/`o` **with a
modifier** — Shift+`i` / Shift+`o` — or an explicit "Set in from playhead" on the
row. **This is an open question for the coach**, because the choice is between a
modifier to remember and two buttons per row.

**M5. Moving a mark must keep the range ordered.** An out before its in is not a
range. Refuse it with a notice naming which mark, rather than silently swapping
them — a swap would move a mark the coach did not touch.

---

## P. Previewing a slate (#119)

**P1. Previewing a slate is NOT the preview pipeline**, and knowing that is what
makes it cheap.

`Command::OpenPreview(Uuid)` takes a **clip** id and composites that clip's
recording against the game video. A slate has no recording, so there is nothing
to composite. What the coach wants is the game video played from `in` to `out`
and stopped.

**P2. So it is a seek, a play, and S3's own crossing detector** — the same
machinery, with no recording in flight. Which is the strongest argument for
specifying these three entries together: a preview is a take's stop without the
take.

**P3. It stops rather than pausing-and-continuing**, because there is no sentence
to finish. The footage pauses on the range's last frame, which is where the coach
wanted to look.

---

## T. The themed pass (#120)

**T1. The queue is the filtered list, and the filter is the Slates list's own.**

Cheapest and clearest: what the coach sees is what they will work through. A tag
filter on the Slates panel, on the clips list's shape.

**T2. Only *timed* slates are in the queue — `out_seconds.is_some()`.**

The coach's clarification. A pass through the corners is a pass through ranges,
and a range with no end has nothing to work through. Half-marked slates **stay in
the list** to be finished; they are excluded from the queue alone. This is the
queue's filter, not a change to what a slate is.

**T3. Only *unshot* slates, by default.** The point is to get through the ones
not yet done, and "has this been shot?" is already the clip scan. A re-record of
one that came out badly is an obvious second need, so the queue carries a way to
include the shot ones — **an open question is whether that is a toggle or just
clicking the row directly**.

**T4. After Stop, it advances to the next slate and parks on its in point,
waiting for `R`.**

The coach's choice. It keeps the momentum of a pass without recording while they
are not ready, and it matches the rule every sheet in this app follows: nothing
moves under the coach's hands without being asked.

**T5. The bound belongs to the PASS, not to the slate — and both modes must keep
existing.**

Free-running is right when the coach marked an in point and wants to talk for as
long as it takes. Bounded is right for a pass. The same slate shot either way
behaves differently, and **nothing is stored to say which** — so the bound is a
property of how the take was started, carried on the command, not a field on the
slate.

**T6. Order is the footage's, not the marked order.** `slates_sorted` exists for
reading and is time order; the stored order is the marked order. A pass through a
match goes forwards through the match. Said rather than inherited from whichever
the list happens to use.

---

## X. What this does not do

- **X1. It does not stop the recording.** S1. The coach keeps Stop.
- **X2. It does not touch a plain `R` take.** No range, nothing to stop at.
- **X3. It does not make `out_seconds` binding on the stored slate.** The slates
  spec's S5 stands: the value is advisory, and a take that runs past it is still
  the take. What changes is that the *footage* stops offering more.
- **X4. It does not draw every slate on the scrubber.** S5's deferral.
- **X5. It does not add a second "record this range" command.** The pass starts
  takes through the same path a single slate does, with the bound carried on it
  (T5).
- **X6. It stores nothing new in `project.json`** — no format bump. The marks'
  fields exist (M2), the selection is UI state (M3), the filter is UI state, and
  the pass is UI state.

---

## Crate responsibilities

- **`pundit-core`**: the two `SlateEdit` variants and the ordering rule (M2, M5).
  The queue's shape — "timed, unshot, tagged, in footage order" — is a pure
  function over `Project`, which is where it can be tested without a bus.
- **`pundit-app`'s bus**: `Command::Pause`, the bound carried on a slate take, and
  the slate edits. It does **not** detect the crossing (S2).
- **`pundit-app`'s UI**: the crossing detector in the tick, the scrubber span, the
  slate selection, the tag filter, and the pass's advance.
- **`pundit-media`**: **nothing.** This feature draws no new pixels and changes no
  pipeline.

## Testing

- Core: the queue function over a project with timed/untimed, shot/unshot, tagged
  and untagged slates; the mark edits including the refused inversion.
- Harness: a take from a slate crosses its out point and the clip's event log
  **ends with a pause whose source time is the out point** — the one assertion
  that proves S1 end to end. And: a skip back before the out point does not
  re-fire (S3), a half-marked slate's take never pauses, and a plain `R` take
  never pauses.
- The crossing detector is pure arithmetic over (previous, current, out) and
  should be a function with its own table test, not a condition buried in the
  tick.

## Risks

1. **The 30 Hz tick means the pause can be up to 33 ms late** (S2). Accepted,
   stated, and the same granularity the board and rings already run at. If it
   ever matters, the fix is not a faster tick: it is the bus detecting the
   crossing and the UI stamping it, which is a bigger change than this feature.
2. **A slate selection is new UI state** (M3) and new state is where bugs live.
   Mitigated by it being UI-only and by the Slates list already having rows.
3. **M4's modifier is a guess at what the coach will find natural.** It is the
   one open question that will be felt every time it is used.
4. **The span on the scrubber is the first span that element has drawn.** Its
   marks are 2px points; a range needs its own shape and its own z-order
   decision. Small, but it is new drawing in a file that currently has none.

## Open questions for the coach

1. **M4: Shift+`i`/`o` to move a selected slate's mark, or two buttons on the
   row?** A modifier to remember against two more controls per row.
2. **T3: should the pass include already-shot slates behind a toggle**, or is
   clicking the row directly enough?
3. **Does the preview (P) want a key, or is a row button enough?** Every letter
   is taken and #96 owns the keyboard question.

## R. What this spec is not sure of

- **The crossing detector's home is argued from the bus contract, not measured.**
  The contract is explicit and the conclusion follows, but no one has shown that a
  bus-side pause would actually drift enough to notice. The argument is that the
  contract exists precisely so nobody has to find out.
- **The queue's "unshot" scan is O(clips × slates)** per rebuild. Fine at a
  match's scale (dozens of each) and stated so nobody assumes it was measured.

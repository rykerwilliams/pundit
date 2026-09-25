# Slates — a range tagged now, its commentary recorded later

**The coach, 2026-09-25:** "clips right now are recordings. but when i'm
watching the game live, i might want to note a time range, in order to do the
'clip' recording later. i want to highlight time range and 'clip it', with
tagging and such, just not do the commentary on it yet."

Watching a match through, the work of *finding* the moments and the work of
*talking over* them are separated. Today they are the same act: a clip exists
only once commentary has been recorded, so a coach who spots a corner routine
must stop, record, and only then move on. A slate is that first half on its
own — a range on the footage, named and tagged, waiting for its take.

> **Second draft.** Both adversarial reviews rejected parts of the first, and
> the rejected reasoning is kept in §S11 rather than quietly deleted: three of
> the decisions rested on claims that are false about this codebase.

## S1. A slate is a new record — and the alternative worth arguing about is `MatchEventRecord`

Not `Clip`: a clip *is* a recording (`recording_filename` and
`recording_duration` are required fields, and "no commentary-less clips" is the
rule that lets every clip replay, preview and export without a special case).
Nobody would have proposed otherwise.

The near miss is **`MatchEventRecord`**, which is already "an instant on a
source with a range around it" — `{ id, source_index, source_seconds,
reel_lead_in: Option<f64>, reel_tail: Option<f64> }` — and already has an
editor, a grammar, a snapshot-undo funnel and source remapping. A
`MatchEventKind::Slate` would reuse all of it. It is still wrong, for a reason
worth writing down:

- **`kind` is read positionally all over core.** `interpret` and
  `ScoreboardContext` build the match clock from it, `reel_goals` builds the
  reel, `start_stop_count` / `expected_start_stop_events` enforce the period
  cap, and `match_entry::word_kind` / `parse_kind` parse it. A fourth kind
  needs an exclusion at *every* one of those, and a single miss puts a slate in
  the burned-in scoreboard or the goals reel.
- **It has no `name` and no `tags`**, which are half of what a slate is for.

So: `Project.slates`, a new record.

## S2. The record (format v12, additive, floor stays 7)

```rust
pub struct Slate {
    pub id: Uuid,
    pub source_index: usize,
    pub in_seconds: f64,
    /// `None` until `o` closes it. A range left open is a visible row the
    /// coach can finish or delete, not a thing lost when the app closes.
    pub out_seconds: Option<f64>,
    pub name: String,
    pub tags: Vec<String>,
}
```

Six fields. `notes`, `created_at` and `sort_index` are **not** here:

- `created_at` is a field `Clip` carries and whose own doc says *"Nothing reads
  it — ordering is by `sort_index`"*. Copying an acknowledged dead field onto a
  new record is the clearest possible failure of "every change must earn its
  place".
- `sort_index` exists on `Clip` because clips are **hand-ordered** — drag
  reorder, `SortClipsBySource`, `ReorderClips`, `renumber()`, and the order is
  burned into `entry_text`'s `n / total`. Slates are never exported (§S8), so
  the only order that means anything is `(source_index, in_seconds)`, which is
  also the order a coach marking ranges in one pass produces. Keeping the field
  would commit us to a `MoveSlate` command, a `ReorderSlates` undo action and a
  read-time normalisation, for a list nobody needs to hand-order.
- `notes` had no reader anywhere in the first draft — no row, no grammar, no
  inspector. The coach asked for "tagging and such", not notes.

Per `project.rs`'s header rules: `Project.slates` is a `Vec` with a field-level
`#[serde(default)]` (exactly what a v7–v11 file means), and a new struct's
fields take **no** defaults. `CURRENT_FORMAT_VERSION` goes to 12, with a test
that every readable version still loads beside the existing ones.

**One source per slate.** A range cannot span two files any more than a clip
can; marking refuses it rather than storing something that needs a rule.

## S3. The link runs from the clip, not from the slate

`Clip` gains `slate_id: Option<Uuid>` — a field on an existing struct, so an
`Option` with a field-level `#[serde(default)]`, which is what the format rules
ask for and what a v7–v11 file means.

The first draft had `Slate.clip_id`, pointing from the one to the many. That
needs lifecycle rules in three places (write it back in `finish_recording`,
clear it in `delete_clip`, restore it on undo) and still dangles when a clip is
trashed. Pointing the other way, nothing needs maintaining:

- **"Has it been shot?"** is `clips.iter().any(|c| c.slate_id == Some(id))`,
  which is automatically right across delete, undo and re-record.
- **"Which clips did it make?"** is naturally plural, which is what a coach who
  re-records actually produces.
- **Nothing can dangle:** the pointer dies with the clip that holds it.

## S4. Marking: `i` and `o`, and the slate exists from the first press

`i` and `o` are free (`Ctrl+O` is Open Project; `o` alone and `i` are unbound).

**`i` creates the slate immediately**, complete and stored, with
`out_seconds: None`. **`o` closes the most recent open slate on the current
source.** There is no in-progress UI state, and that is the point:

- Nothing is lost when the app closes between the two presses — the coach sees
  a row reading `14:05–` and finishes or deletes it.
- "`o` before `i`" becomes "there is no open slate here", the same trivial
  refusal, and "`o` on a different source" disappears — `o` simply finds
  nothing on this source.
- `i` twice gives two slates, which is recoverable by deleting one; a silent
  move of an invisible in point is not.
- An in-progress in point would hold a `source_index` with **no invalidation
  rule**: mark `i` on video 2, drag video 2 to the front, press `o`, and the
  slate lands on whatever file is now at index 2.

The times come from the game video's position (`scan_source_position` in
`main.rs`, the same function a **match tag** uses), captured on the UI thread
per the bus contract — never queried in the handler. Not `shown_source_position`,
which a highlight key uses: that one exists because a ring must sit on exactly
the frame it was drawn over. A slate is a range, like a tag, and the two agree
to within a sub-frame anyway except mid-seek, where `shown_position` falls back
to the same `locate` this uses.

**Both keys are gated on the existing `can-tag`** (`can-play && !previewing &&
recording-phase != starting`), not on a new gate. Without `!previewing` the
keys would mark the game video's stale paused position while the coach is
looking at a preview. The gate is in the UI because the bus's refusal is only
an `eprintln!`.

**Marking is allowed while recording**, and joins `Command::TagMatchEvent` and
`Command::SetHighlightKey` on the allow-list. Those two are there because *a
record that belongs to the footage is placeable whenever the footage is on
screen*, and a slate is such a record. (The first draft refused it, on the
grounds that "the commentary event log has no room" — a slate never goes in
that log.)

## S5. Shooting a slate: refuse, reset, seek, start — in that order

The row's primary action is **Record**. The order is load-bearing, because
`start_recording` is built around *"Before anything changes: a refusal leaves
the player as it was"*:

**The shoot does not sequence this from outside.** `start_recording` gains two
parameters — `start: Option<(usize, f64)>` and `slate: Option<Uuid>` — and
`toggle_recording` passes `None, None`. Everything below then happens **inside**
the one function that already owns the refusal barrier:

1. **Every refusal fires before the player moves.** `can_record` checks the
   project, a running export, an open preview, the sources and the load; but
   **`NoCamera` is not among them** — it comes from `resolve_camera` inside
   `capture_sources`, after `can_record` has passed. So a slate shoot sequenced
   from outside would move the game video to the in point and *then* refuse on a
   camera-less machine. Moving the seek inside, after `capture_sources`
   succeeds, is what makes "a refusal leaves the player as it was" true for this
   path too — and it needs no change to `can_record`'s or `start_recording`'s
   visibility, both of which are private to `bus::recording`.
2. **`reset_skip()` before the seek.** `start_recording` takes its start from
   `heading(None)`, which prefers the **skip coordinator's pending target** over
   the player's. Two taps of the right arrow followed by Record would otherwise
   stamp the clip at the skip target rather than the slate's in point. `scrub`
   and `step_frame` already call `reset_skip` for exactly this reason.
3. **Then the seek**, which does not race: `load` sets `current` and
   `player.holds(uri)` synchronously, so `heading(None)` returns the in point
   immediately — no wait, and nobody should add one.

`Command::ShootSlate` therefore carries `{ id, zoom }` and **no captured
position**: the in point is a stored field, not a playhead reading. That is the
same reasoning `EditMatchEvent` records — "the time is typed, not captured … the
editor never reads the playhead at all". The `zoom` it does carry is the one
`RecordingLog::new` seeds the log with, which the UI owns.

**The inheritance mechanism**, which the first draft asserted without one:
`add_recorded_clip` hardcodes the name (`"2-01:02:05"`), empty notes and empty
tags, and `PendingClip` is `Copy` (two call sites rely on it). So the take
carries the slate's id on `Active`, and `finish_recording` — which already
mutates and saves the project — applies the slate's `name` and `tags` to the
new clip and sets its `slate_id`, in the same save. `PendingClip` stays `Copy`
and core's `add_recorded_clip` stays a pure function of the recording.

**An empty slate name does not become the clip's name.** A clip with no name
generates `"2-01:02:05"`, which is more informative than "Slate 3"; the slate
overrides it only when the coach actually typed one.

`out_seconds` is **advisory**: it is where the coach said the moment ends, and
a take that runs past it is still the take. Binding it would mean stopping a
recording the coach is still talking over.

## S6. Editing, and what is *not* reused

A slate row is selectable, and a selected slate edits its **name and tags** in
the same shape the clip inspector already uses — the two fields it already has.
Delete is a row action. Every edit funnels through a snapshot-undo action, as
match events and highlights do.

**`i` and `o` do not re-time a selected slate**, which an earlier draft of this
section said and §S4 contradicts. One key cannot both create and re-time, and
the mode deciding which would be invisible state on the primary key. Re-timing
is delete-and-re-mark: two keystrokes on a list whose rows are cheap.

**Any field added here joins the window's `text-editing` fold in the same commit
as the keys.** `handle-key` returns `reject` on that property *before* the
letter branches, so a slate tag field outside the fold would mean typing
"possession" marks two slates, starts a recording, clears the drawings and tags
two goals on the way through. The keys also need `!event.repeat`, as `tag-key`
and the `h`/`r` branches do, or a held key marks a slate per repeat.

**The typed-line editor is not part of this feature** (§S10), and the grammar
reuse the first draft promised does not exist:

- **`#` is a comment in `match_entry`** — `body()` does `line.split('#').next()`,
  "which is `kickoffs.txt`'s own convention". The first draft's own example,
  `2 14:05-14:40 corner routine #corners`, would have had its tag **silently
  discarded** by the grammar it claimed to reuse. There is no `#` tag sigil
  anywhere in this codebase; tags are comma-separated through
  `core::tag::normalize_tags`.
- **The words after the time are a closed vocabulary.** `parse_kind` refuses
  anything that is not an event word or a team name. A slate's tail is free
  text. There is no overlap.

What is genuinely reusable, if a typed editor is ever built, is `parse_time`,
`format_time` and the rule that a leading bare integer is a video number — in a
new `core::slate_entry`, not a second mode bolted onto `match_entry`.

## S7. The source list

Precisely, because "purges or remaps" was too vague to build from — nothing is
ever purged, and an add does nothing:

- **Remove refuses.** `source_is_referenced` gains a slate clause, so a source
  a slate points at cannot be removed, exactly as for clips, match events and
  highlights. The two user-facing strings that enumerate the kinds — in
  `project.rs` and `UserError::SourceReferenced` ("still used by a clip, a match
  event or a highlight") — gain "a slate".
- **Move and remove remap** every other source's indices: one more loop beside
  the three that are there.
- **Add and relink do nothing**, as `undo.rs` already says.
- **`purge_for_source_change` must drop stale slate snapshots.** Its
  `stale_snapshot` closure is a **non-exhaustive `matches!`**, so the compiler
  will *not* catch the omission: mark slates on video 2, drag video 2 above
  video 1, press Ctrl+Z once for an unrelated rename, and an `EditSlates`
  snapshot restores pre-move indices — every slate silently pointing at the
  wrong file, saved. While adding the variant, make that closure an exhaustive
  `match` returning `bool`, so the next record type cannot forget.

## S8. Tags

Slates share the tag **vocabulary**, and that is a small change in core rather
than a call-site tweak: `tag_suggestions` today takes `&[TagSummary]`, whose
rows carry `clip_count` and `total_seconds` — numbers a slate has not got. So it
takes a `&[String]` vocabulary instead, fed by a new
`tag::tag_vocabulary(project)` (sorted, deduped, clips ∪ slates). A tag invented
on a slate then autocompletes on the next one, and slate tags go through
`normalize_tags` like every other tag.

The tag **overview** stays clips-only. Its columns are `clip_count` and
`total_seconds` summed from `recording_duration` — a number a slate does not
have — and it is a view of exportable material. The slate section carries its
own filter, so a tag used only on slates is reachable there.

## S9. Out of scope, named so it is not drifted into

- **Exporting the slates** as a silent breakdown film with the scoreboard burned
  in. They are already an edit decision list, and `reel_plan` is close to the
  right shape, so this is cheap — but it is a second feature, and it is the one
  that would make `out_seconds` load-bearing rather than shown.
- **Marking at the pitch** with no footage loaded: a range is a source time, and
  wall-clock would need the match clock and a video that is not there.
- **Auto-suggesting slates** from the detection work (P4 is not justified).

## S10. Phases

- **A — the feature:** v12, `i`/`o`, the section, Record and the inheritance,
  name/tags editing, undo, the source-list work. This is all of it.
- **B — not the typed editor.** If there is a phase B it is §S9's slate export,
  which is cheaper and makes the out point mean something. The typed editor is
  backlogged: the coach asked to mark ranges while watching, not to type them.

## S11. What the first draft got wrong, kept as the record

- **"A clip's number is load-bearing in the sidebar"** — false. `ClipRow`
  carries `id`, `name`, `duration` and no number; the caption's `3 / 6` comes
  from `entry_text` in the export plan, over `project.clips`, which never sees
  the UI model. The separate section survives on its real merits (different
  columns, a different primary action, and a list that is not the export
  order), not on that claim.
- **The `#corners` tag syntax** — would have been eaten by the comment rule.
- **"The clip inherits the slate's name, notes and tags"** — had no mechanism:
  `add_recorded_clip` hardcodes all three and `PendingClip` is `Copy`.
- **"Refused while recording, because the commentary event log has no room"** —
  a slate never goes in that log, and the two analogous records are explicitly
  allowed while recording.
- **Eleven fields**, three of which nothing read.

## S12. Tests

- **core:** the record round-trips at v12 and v7–v12 all load; a slate blocks
  its source's removal; a source move remaps slate indices; `(source_index,
  in_seconds)` ordering.
- **undo, the one the compiler cannot catch:** mark slates on a source, move
  that source, undo an unrelated edit, and the slates still point at the same
  file.
- **harness (over the bus):** `i` creates an open slate at the displayed
  frame's time; `o` closes it; `o` with nothing open is refused and stores
  nothing; marking works while recording; Record on a slate refuses first when a
  preview is open, resets the skip coordinator, and the clip it produces starts
  at the in point and carries the slate's name, tags and `slate_id`; an empty
  slate name leaves the clip's generated name alone; undo restores what each
  edit changed.
- **No test touches the coach's projects**, and the shoot path uses
  `CaptureKind::Test`.

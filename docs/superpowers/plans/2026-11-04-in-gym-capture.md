# Plan — in-gym capture (#140), phase 1

Spec: `docs/superpowers/specs/2026-11-04-in-gym-capture-design.md`. Read it and
`CLAUDE.md` first; neither is repeated here. This plan covers **phase 1** (1a +
1b + 1c of the spec). **Phase 2 (Google Drive) is a separate spec** and is not
planned here.

**This is the reviewed draft.** §B records what the two adversarial passes
changed — and it is substantial: the first draft put a format edit in the wrong
crate, split a field from its only reader (dead code), leaned on an `f64` default
that the hand-written event codec ignores, and under-stated the recorder
machinery a capture must reuse. Read §B before the tasks. The spec was corrected
in step with this review (the `core/export.rs:209` citation and the `Rate`
newtype).

## Where this stands (update it as tasks land)

- **1 — core: session format (kind, created, v19 bump).** **DONE** — branch
  `claude/140-task-1`, commit `e56f0cb`. Gate green (core tests, fmt, pinned-1.92
  clippy, both sabotage proofs run). `a_v19_file_loads_under_the_current_version`
  follows the real sibling convention (there is no combined `v7_to_v19` test).
- **2 — core: recorded rate (field + behaviour, one PR).** **DONE** — branch
  `claude/140-task-2` (stacked on task 1), commit `1bde43f`. Gate green (35 core
  test binaries, fmt, pinned-1.92 clippy, 4 sabotage proofs). `Rate(f64)` newtype,
  `Default = Rate(1.0)`; the real-rate line is `export.rs:217`
  (`+ (t - out_start).max(0.0) * seg.rate`). **Caveat:** the 3 `EventKind::Play`
  sites outside `pundit-core` (`pundit-harness/tests/recording.rs`,
  `pundit-media/tests/export.rs` ×2) were edited and read-checked but **not
  compiled here** — fmt-checked only. Confirm them under the full workspace build.
- **3 — media: prove the rate survives the GL graph (test only).** Not started.
- **4 — bus: create an empty session.** Not started.
- **5 — bus + media: capture the camera as a source.** Not started.
- **6 — UI: the session shell.** Not started.
- **7 — UI + bus: the playback-speed control.** Not started.
- **8 — UI: the touch affordance.** Not started.

> **Environment note (2026-11-05):** tasks 1–2 were built and verified on a box
> with only a userspace Rust toolchain (no GStreamer, no sudo), so only the
> pure-`pundit-core` subset of `verify` ran. Tasks 3–8 touch
> `pundit-media`/`pundit-app`/`tests/ui` and need GStreamer dev libs — run them on
> a machine with the full toolchain and the whole `verify` gate. Task 2 stacks on
> task 1; rebase `--onto origin/main` once task 1 squash-merges (and retarget
> task 2's PR to `main` before merging task 1, per `start-task`).

## Dependency order

```
1 (session format) ── 4 (session) ── 5 (capture) ── 6 (session UI) ── 8 (touch)
2 (rate, core) ────── 3 (rate, media test) ── 7 (rate UI)
```

**Tasks 1 and 2 are both pure `pundit-core` and both land under v19 before the
1a/1b release** — the bump itself is in task 1, and task 2 extends task 1's
`v7_to_v19` load fixture with a rate-carrying event (it does **not** re-bump).
That shared version is the only coupling between the two tracks; otherwise the
session track (1→4→5→6→8) and the rate track (2→3→7) are independent. Within a
track the arrow is a hard dependency. **"Shares no file" is not "parallel"** (the
whiteboard lesson): task 5's `Active`/recorder lifecycle and task 2's
`PlaybackSegment` are shared construction sites that force their change through
every caller at once.

---

## Phase 1a — the session and capture-as-source

### Task 1 — core: session format

**Pure `pundit-core`. The session fields and the v19 bump.** Spec §S1–S3, §S2.

**The change:**
- `project.rs`: `pub enum ProjectKind { Match, Session }`, `Default = Match`,
  field-level `#[serde(default)]` on `Project.kind` (the v14 `StrokeEnd`
  precedent — an enum whose `Default` is what an older file means). Add
  `Project.created: Option<String>` (ISO date), field-level `#[serde(default)]`,
  `None` = every older file. Add `Project::is_match(&self) -> bool`.
- `store.rs`: `CURRENT_FORMAT_VERSION` 18 → **19**. Floor stays 7.
- **Literal fixups are small for `Project`:** there are only **two** full
  `Project { format_version: .. }` literals (the struct and `Project::new`,
  `project.rs`), because every test builds via `Project::new` + mutation. Fix
  `Project::new`; the fixture rule (`..stub()`) covers the rest. *(The wide
  literal fan-out is `EventKind::Play` — 20 sites — and that is **task 2**, not
  here.)*

**Tests:**
- `project_format.rs::v7_to_v19_files_load_under_the_current_version`, beside its
  siblings: a v19 fixture with `kind: "session"` and a `created` round-trips;
  every older fixture still loads **as `Match`** (the default). *(Task 2 extends
  this fixture with a rate-carrying `play`.)*
- `ProjectKind::default() == Match`, and a loaded v7–v18 fixture reads `Match`.

**Sabotage:**
- `CURRENT_FORMAT_VERSION` left at 18 → the v19 fixture fails the `TooNew` arm
  (`store.rs`), named exactly.
- Default `ProjectKind` to `Session` → **every older fixture in `v7_to_v19…load`
  now loads as a session**, failing that fixture's `kind == Match` assertion
  (the concrete, non-vacuous form — not "nothing else flips").

---

### Task 4 — bus: create an empty session

**`pundit-app` bus. Depends on task 1.** Spec §N1–N4.

**The change:**
- `Command::NewSession { name: String }` (`bus/mod.rs`), **not** on the recording
  allow-list. Handler in `bus/project.rs`: clean `name`
  (`naming::folder_slug` folder, `naming::safe_chars` stored name; default
  date-stamped `"Session"` when blank — the **bus** supplies the date, core has
  no clock), reuse `projects_dir_for` (`new_match.rs:100-123`), `create_dir`
  (never `_all`), build `Project { kind: Session, created: Some(<bus date>), .. }`,
  and go through the **same `commit` path `new_match` uses** (it empties trash,
  clears history, pushes recents, publishes `ProjectOpened` — skipping it would
  drop the recents push). `store::write` creates `recordings/` only; `sources/`
  appears at first capture (task 5).
- Shares no body with `new_match` (no scoreboard, no videos, no aspect gate);
  do not fold them (spec §N4).

**Tests (harness):**
- `new_session_creates_an_empty_session_and_opens_it`: folder + `project.json`
  with `kind: Session`, `created` set, `source_videos` empty, `ProjectOpened`
  published.
- `new_session_is_refused_while_recording`; `new_session_defaults_the_name_when_blank`.

**Sabotage:** drop `kind: Session` → the session assertion fails; stamp no
`created` → the `created` assertion (and task 6's recents test) fails.

---

### Task 5 — bus + media: capture the camera as a source

**The spine. `pundit-app` bus + `pundit-media`. Depends on task 4.** Spec §C0–C4,
§V1. Read §C and §X first — this is the task the spec review rewrote.

**The change:**
- `Command::CaptureSource { zoom: Zoom }` (`bus/mod.rs`), on the recording
  allow-list.
- **Share the recorder lifecycle, branch only at finish.** `start_recording`
  (`recording.rs:125-250`) supplies far more than guards: `self.self_view.take()`
  before start, `Recorder::start(sources, &path, self_view, callback)` with the
  bumped `self.generation`, the `Active { recorder, path, media_seen, .. }` state,
  the `FirstBuffer`/`media_seen` finish-vs-abort gate (`stop_recording`), the
  `STOP_TIMEOUT` stop, and the transcription preempt/resume. Capture **reuses all
  of that**; it forks only in (a) its guards, (b) no anchor, and (c) finish. So
  factor the shared lifecycle rather than duplicate the stall/timeout handling —
  a capture-vs-clip branch on the start inputs and the finish, not a parallel
  copy. `Active` gaining a capture variant is a shared construction site — change
  it through `stop`/`finish`/`abort`/start-timeout at once.
- **Its own guards**, not `can_record`: a camera is required; an empty/unloaded
  project is **not** refused (`can_record` refuses both,
  `recording.rs:420-437`). **No `self.heading(None)` anchor** (`recording.rs:194`)
  and **no player seek** — meaningless in an empty session.
- Records camera+mic (`CaptureSources::Devices`, `capture/recorder.rs`) to
  `sources/<id>.mkv`. `sources/` is a new dir created on demand, like
  `backdrops/` (not created by `store::write`).
- **Finish builds a `SourceRef`, not a `Clip`.** Use **`source_ref(&open.folder,
  &path, probe)`** directly (`bus/sources.rs`), **not** `add_source`/`probed_source`
  — that skips `check_aspect` (spec §V1: a session runs no aspect gate). First:
  **assert the captured MKV probes clean** right after `stop` (duration + a video
  stream, `probe.rs:41-96`) — a freshly finalised container must report a
  duration. Ensure `open.folder` is **canonical** before relativising, or a
  symlinked projects dir yields a `..` path.
- **`remove_source` deletes the file iff `relative_path.starts_with("sources/")`**
  — the exact "project owns this" test, no path-traversal/owned-vs-external
  machinery (a session's captured sources are all under `sources/` by
  construction). Today it deletes nothing (`sources.rs:59`). Delete **only on the
  core `Ok` branch** — core `remove_source` returns `Err(SourceReferenced)` if any
  clip/slate/highlight references the index (`project.rs:692`), so a referenced
  source is never deleted. Source deletion is **irreversible** (no `.trash`/undo),
  by design.

**Tests (harness, `CaptureKind::Test` sources — never the real camera):**
- `capture_source_into_an_empty_session_adds_a_source`: one `SourceRef` under
  `sources/`, no `Clip`, the file exists and probes.
- `capture_source_bypasses_the_empty_project_guard` (succeeds where `can_record`
  refuses).
- `removing_an_unreferenced_captured_source_deletes_its_file`, and
  `removing_an_imported_source_leaves_its_file` (the `starts_with("sources/")`
  guard, both ways).
- `a_review_take_over_a_captured_source_is_an_ordinary_clip` (§C3) — the proof
  D-2 costs no new code.

**Sabotage:**
- Route `CaptureSource` through `can_record` → the empty-session test fails on
  "add a game video to record over first".
- Issue a seek/`heading` during capture → a seek fires in an empty session; the
  capture test fails because a seek was issued (not "wrong source_index" — there
  is no clip stamped).
- Make `remove_source` delete unconditionally → the imported-source test fails.
- Delete before the `Ok` branch → a referenced-source removal deletes the file
  then errors; a "referenced removal keeps the file" test fails.

---

### Task 6 — UI: the session shell

**`pundit-app` Slint + `tests/ui`. Depends on task 5.** Spec §C4, §S4, §V2, §V3.

**The change:**
- **Expose the kind to Slint:** a new `in property <bool> is-match` on
  `AppWindow`, set from `main.rs`'s `show_project` (there is no kind property
  today). Chrome gates on `is-match`.
- A **New Session…** entry point opening a one-field name sheet → `NewSession`.
- **One Record button, meaning follows the bus** (§C4): the *bus* chooses
  `CaptureSource` (no source selected) vs the normal record (one selected), as
  `ToggleRecording { slate }` does; the label reads "Record clip" / "Record".
- **Hide match chrome when `!is-match`** (§S4, §V3): scoreboard setup, match-event
  editor, reel, clock readout, slate band, slate list. Export sheet offers Clip +
  basket only, no scoreboard picker (§V2).
- Devices popover: no avatar/whiteboard section in a session.
- **Recents shows `created`** via `recents.rs`'s existing `second_line` (it
  already reads one `project.json` per row — `main.rs:3063`); pull `created` from
  the project already read. **Not** a new `state.json` date cache (that would copy
  project data into machine state).

**Tests (`tests/ui`, setting `is-match` directly — the bus-driven Record meaning
is proved in the harness, task 5, since `tests/ui` can't reach `main.rs`):**
- `a_session_hides_exactly_the_match_chrome` — the §S4 gate test, enumerating the
  hidden controls.
- `the_record_button_label_follows_selection_in_a_session`.

**Sabotage:** show the scoreboard setup in a session → the chrome test fails,
naming the control; fix the label to "Record" always → the label test fails.

---

## Phase 1b — recorded playback speed

### Task 2 — core: recorded rate (field + behaviour, one PR)

**Pure `pundit-core`. Depends on task 1 (v19 exists). Field and behaviour land
together — no dead field.** Spec §R1, §R3, §R5.

**The change:**
- **The event field, through the hand-written codec.** `EventKind` does not
  derive serde (`event.rs:66`); it routes through the `KnownKind` proxy
  (`event.rs:98`), a hand-written `impl Serialize` (`:122`) and `deserialize`
  (`:153`). Add `rate: Rate` to `EventKind::Play` where `Rate(f64)` is a newtype
  with `Default = Rate(1.0)` — **not** a bare `f64` default (forbidden; resolves
  to `0.0`, which is the Freeze sentinel). Edit: `KnownKind::Play` (with the
  newtype's field-level default), the `From<KnownKind>`, the `Serialize` arm, the
  two `timeline.rs` matches (`:97`, `:233`), the producer `recording.rs:82`, and
  the wire-shape test `play_has_the_expected_wire_shape` (`event.rs:182`). The
  20-site `EventKind::Play` literal census lives here (`grep -rn 'EventKind::Play'
  crates/`).
- **Timeline behaviour** (`timeline.rs`): `PlaybackSegment` gains `rate: f64`
  (computed type, no format change; it derives `Copy`, safe). `emit`
  (`:154-182`) branches on `rate != 0.0` (Play) vs `== 0.0` (Freeze), carrying the
  segment's rate; its EOF-split math (`available`/`play_dur`/cursor-advance)
  scales by `rate` (source consumes `dur * rate` per record-second). The
  non-segment `source_time` walk (`:79-117`, `rate = 1.0` on Play at `:98`) takes
  the event's rate too.
- **Export walk** (`pundit-core/src/export.rs:209`, **core, not media**):
  `seg.source_start + (t − out_start) * seg.rate`. This is the one line that makes
  a non-unit rate real; `schedule → walk` is the single scheduler both export and
  preview use.
- **Audio** (`audio.rs::game_regions`, `:211-251`): a Play segment with a
  non-unit rate **emits no source-audio region** — the segment-level voice-over
  rule (§R3), the shape of the source-volume-0.0 path. `PlaybackSegment.rate`
  reaches audio.
- **Decode of `rate: 0.0` on a Play is pinned**: a floor (treat ≤0 as the Freeze
  it already is) with a test, so a malformed value can't silently become a frozen
  Play by a different route.

**Tests:**
- `a_half_speed_play_segment_advances_the_source_at_half_rate` — source cursor is
  `start + 0.5*N` after N record-seconds, **with a fixture whose 0.5× segment
  crosses `source_duration`** so the EOF-split is exercised.
- `a_slow_play_segment_is_a_play_not_a_freeze`; `a_slow_play_segment_contributes_no_source_audio`.
- A pre-existing 1.0 edit is unchanged to the integer (Rate defaulting).
- Extend `v7_to_v19…load` with a `play` carrying a `rate`.

**Sabotage:**
- Keep `emit`'s `== 1.0` → the slow segment becomes a Freeze; the "is a Play"
  test fails `left: Freeze right: Play`.
- Scale the walk but not the EOF-split → the source over-runs its duration; the
  cursor test fails at the boundary the fixture crosses.
- Drop the `* seg.rate` in `export.rs:209` → an export/driver test reads 1.0×.
- Emit a source region for the slow segment → the no-audio test fails.

---

### Task 3 — media: prove the rate survives the GL graph (test only)

**`pundit-media`, a verification task — no production change** (the edit is
task 2's `core/export.rs:209`). Depends on task 2. Spec §R2.

**The change:** none to production. A media export integration test on the GL
graph (llvmpipe), because the end-to-end path through `decodebin3 → mixer →
encode` is worth pinning even though the arithmetic is core's.

**Test:** `a_half_speed_export_plays_the_source_at_half_speed` — a counter fixture
exports such that output frame 60 shows source frame 30, the commentary track
real-time.

**Sabotage:** revert task 2's `* seg.rate` → the counter test reads source 60 at
output 60.

---

### Task 7 — UI + bus: the playback-speed control

**`pundit-app`. Depends on task 3.** Spec §R4, §R5.

**The change:**
- A playback-speed control in the recording transport, surfaced in clip settings
  for review.
- **`Command::TogglePlay` gains a `rate` field** and `RecordingLog::play`
  (`recording.rs:82`) gains a `rate` argument — a live rate change appends a
  `Play` at the current position carrying the rate (the window supplies it at the
  input event, caller-captured per the bus contract). The command **variant** is
  not new and needs no new allow-list entry (`TogglePlay` is already allowed), but
  its **payload changes** — the first draft's "no new command" hid this.
- Scan speed (`J`/`L`) is untouched and stays viewing-only (§R4).

**Tests:** `tests/ui` for the control; harness for the `Play` event appended at
the chosen rate.

**Sabotage:** set 0.5× but append `rate: 1.0` → the harness rate assertion fails.

---

## Phase 1c — touch

### Task 8 — UI: the touch affordance

**`pundit-app` Slint + `tests/ui`. Depends on task 6.** Spec §T1–T4.

**The change:**
- **Name the clip-row `ContextMenuArea`** (it is unnamed today; the slate's is
  `AppWindow::slate-menu`, `app.slint:6223`) and add a kebab `⋯` on the clip row
  (and, in a match, the slate row) that opens the **same** menu — one menu
  definition, two openers — for the **menu-only** actions (Export / Add to basket
  / Delete; Record / Delete on a slate). Preview and Jump already have a tap path
  (single/double-click, `app.slint:6200,6588`) and get no new control.
- The scrubber's snap becomes a **visible toggle** built on the existing
  `snap-enabled` property and `snap-menu` (`scrubber.slint:89,199`); "Band
  colour…" stays in Settings (#131). The scrubber right-click stays a power-user
  alias.
- Nothing else ships under this task (§T3).

**Tests (`tests/ui`, the `slate_menu.rs`/`snap_menu.rs` pattern —
`find_by_element_id` + `mock_single_click`, **not** the `export_sheet.rs`
accessible-label/invoke pattern, which is for `CheckBox`/`Text` leaves):**
- `the_clip_row_kebab_opens_the_clip_menu` — find the kebab, open it, drive the
  menu by its named id.
- `the_snap_toggle_is_tap_reachable`.

**Sabotage:** wire the kebab to a second, divergent menu → a test asserting both
openers list the same actions fails.

---

## Closing conventions (from CLAUDE.md)

- Each task is one branch (`claude/140-task-N`), one fresh subagent, `verify`
  before commit, `sabotage-proof` for every behaviour pinned.
- `pundit-core` declares no media dependency — tasks 1 and 2 must not pull one in.
- A new `Clip`/`Project` field goes into the test stub and `sample_*` and nowhere
  else; every other fixture spreads `..stub()`.
- Backlog any deferred item to #140's follow-ups.

---

## §B — what the adversarial review changed

The two passes (simplify + correctness) read the plan against the source; nearly
every finding was verified by me against the code before applying. Applied:

1. **The rate walk is `pundit-core/src/export.rs:209`, not
   `pundit-media/src/composite/export.rs`** (both reviewers; verified — media's
   `:209` is a `MatchMedia` impl). The `* seg.rate` edit moved into **task 2**
   (pure core); **task 3 became verification-only** (no production change). The
   spec's R1/R5 citation was corrected in step.
2. **Task 1/2 split created a dead field** (`Play.rate` with no reader until a
   later task). Re-split so **task 2 lands the field *and* its behaviour in one
   PR**; task 1 is session-format only. Both are core, both under one v19 (bump in
   task 1, fixture extended in task 2), shipped together.
3. **`EventKind` has a hand-written codec and an `f64` can't default to `1.0`**
   (verified: `event.rs:66` no derive, `KnownKind` proxy at `:98`). The rate is a
   **`Rate(f64)` newtype with `Default = Rate(1.0)`**, and task 2 enumerates the
   codec edits (`KnownKind`, `From`, `Serialize`, the wire test). The spec's R1
   was corrected. The `rate == 0.0` / Freeze-sentinel collision is pinned by a
   decode test.
4. **The literal census was wrong**: `Project` has **2** literals (not "19 like
   the whiteboard `Clip` field"); the 20-site fan-out is **`EventKind::Play`**,
   moved to task 2.
5. **Task 5 under-stated the shared recorder lifecycle** (self-view reset,
   `Recorder::start` generation/callback, `Active`, `media_seen`/`FirstBuffer`
   gating, `STOP_TIMEOUT`, transcription preempt). Now: share the lifecycle,
   branch only at guards/anchor/finish. `source_ref` (not `add_source`) to skip
   the aspect gate; assert the capture probes clean; canonical `open.folder`.
6. **The source-deletion guard was over-built.** Simplified to
   `relative_path.starts_with("sources/")` (no owned-vs-external path traversal),
   delete only on the core `Ok` branch (core refuses a referenced removal,
   `project.rs:692`), irreversible by design.
7. **`is_match()` isn't exposed to Slint** — task 6 adds `in property <bool>
   is-match` set from `show_project`. **Recents `created`** uses `recents.rs`'s
   existing `second_line`, not a new mechanism.
8. **Task 7 hid real plumbing** — `TogglePlay` gains a `rate` field and
   `RecordingLog::play` a `rate` arg; "no new command" was true only of the
   variant name.
9. **Task 8's test pattern was wrong** — menus are driven by `find_by_element_id`
   + `mock_single_click` (`slate_menu.rs`), not `export_sheet.rs`'s accessible
   label/invoke; the clip-row menu needs a named id.
10. **Three sabotage proofs couldn't fail as written** (task 1 "nothing flips",
    task 5 anchor, task 2 EOF-split) — each reworded to name an assertion that
    fails and a fixture that reaches the boundary.

Not changed (verified correct): the `Take` correction (§C0), `can_record`'s
refusals and the capture bypass, `game_regions` reading `PlaybackSegment`, the
`backdrops/`-style on-demand `sources/` dir, the v19 `TooNew` guard, and the snap
toggle building on the existing `snap-enabled`.

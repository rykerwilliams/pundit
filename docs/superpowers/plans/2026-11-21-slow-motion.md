# Plan — slow motion during a recording (#140)

Spec: `docs/superpowers/specs/2026-11-21-slow-motion-design.md` (after its
review fixes, `1c7416e`). Read it and `CLAUDE.md` first; neither is repeated
here except where §0 adds to the spec. **Every task starts with the
`start-task` skill and ends with `verify`, `sabotage-proof` and
`recheck-claims`.**

Every `path:line` here was re-derived on **2026-11-22 at `1c7416e`** (`main` at
`d742430` plus the spec's three commits, which touch only `docs/` and
`BACKLOG.md`). Paths are from the repository root; `core`, `media`, `app` and
`harness` mean `crates/pundit-<name>/`. **§A holds every citation**; the tasks
cite only what §A does not.

**This plan's `BACKLOG.md` edits are #140's entry and its "Next, in order"
line, and nothing else.**

## Where this stands (update it as tasks land)

- Nothing started. The spec and this plan are on `claude/slow-motion-spec`.

**Shipping: one PR, a commit per task, in order.** Task 1 bumps the format to
v19, and on its own it would re-stamp every project a coach saved with nothing
that needs v19 — an older build would then refuse files that hold no `Speed`
event at all. Tasks 2 and 3 are media changes that are passthrough at 1× (spec
M1) and have nothing to slow until task 4 logs a speed; reviewed apart they
would be reviewed without their reason. Each commit is still verifiable on its
own: its tests, and every existing one, pass at its own head, and no head
between them can record a slow span — the bus refuses every speed below 1×
until task 4.

---

## §0. What the spec does not say (checked 2026-11-22)

1. **`change_rate` gains the seek target as an argument**:
   `change_rate(rate, at: Option<f64>)`. `scan_speed` passes
   `Some(heading(source_secs).1)` (spec S5's "seeks to the anchor it logs");
   `set_playing`'s fast reset passes `None`, which keeps today's choice
   (`transport.rs:137`–`:144`: the target in flight, else the shown frame
   while paused, else the position).
2. **The cap is the whole of `scan_speed`'s refusals but the preview's.**
   `max` is 1× when recording, paused **or a slate is armed**, 32× otherwise —
   which is today's `!self.playing` refusal (a fast speed while paused) and the
   spec's narrowed armed-slate refusal (S10) both, as one value. The
   `!self.playing || … || self.armed_slate.is_some()` early return goes; the
   `preview.is_some()` one stays.
3. **The harness gains `Harness::scan(step)`** beside `toggle_play`
   (`harness/src/lib.rs:132`–`:137`), sending `ScanSpeed { step, host_ns:
   now_ns(), source_secs: self.position_secs() }`, and every existing
   `Command::ScanSpeed(…)` send in the harness becomes a call to it (§A).
4. **`Region`'s test literals take `rate: 1.0`** (`core/tests/audio.rs:107`,
   `:347`, `:415`); no other `Region {` exists outside `audio.rs:248`.
5. **The whiteboard notice is not asserted verbatim anywhere**:
   `harness/tests/whiteboard.rs:146` matches the variant and `is_notice` only.
   Adding "slowed" breaks no test.
6. **Slint's `ScanStep` gains `toggle-slow`** (`app.slint:242`–`:246`), so two
   exhaustive matches over it gain an arm: `main.rs:800`–`:804` and
   `app/tests/ui/keys.rs:226`–`:228`.
7. **`SegmentKind`'s `Eq` goes** (`timeline.rs:33`). Checked: nothing hashes it,
   bounds it on `Eq`, or uses it as a map key; `PlaybackSegment` already
   derives only `PartialEq` (`timeline.rs:42`).
8. **The book says J/L are "while playing… 1× up to 32×"**
   (`docs/book/src/guide/shortcuts.md:27`, `:30`–`:32`;
   `docs/book/src/guide/watching.md:26`–`:33`). Task 6 rewrites both and adds
   `S`.
9. **No CHANGELOG edit.** It is curated at release.

---

## §A. The census, 2026-11-22 at `1c7416e`

| Changing | Sites |
|---|---|
| The event (task 1) | `core/src/event.rs`: `EventKind` `:67`–`:87`, `KnownKind` `:98`–`:107`, `From` `:109`–`:120`, `Serialize` `:122`–`:144` (each gains a `Speed` arm); the wire-shape tests `:181`–`:236` (one more beside them) |
| The walks (task 1) | `core/src/timeline.rs`: `SegmentKind` `:33`–`:39` (derive `:33`, `Play` doc `:35`); `source_time` `:79`–`:116` (its match `:97`–`:110`); `Walk` `:132`–`:142` (`rate` `:136`); `emit` `:145`–`:185` (the `rate == 1.0` test `:154`, `Play` push `:163`); `playback_segments`' match `:232`–`:251`; docs `:67`–`:68`, `:121`–`:122` |
| `SegmentKind::Play` sites (task 1) | `core/src/export.rs:209` (the walk's map); `core/src/audio.rs:221` (task 2's, but it must compile in task 1 — see task 1 step 5); `core/src/whole_match.rs:37`; `core/src/reel.rs:206`. Tests: `core/tests/timeline.rs:133`, `:149`, `:163`, `:192`, `:207`, `:297`, `:386`; `core/tests/whole_match.rs:62`; `core/tests/reel.rs:56`; `core/tests/audio.rs:143`; `core/tests/caption.rs:24` |
| `EventKind` exhaustive matches | `timeline.rs:97`–`:110` and `:232`–`:251` (both gain `Speed`), `event.rs`'s codec. **No other** match names every variant (grepped `EventKind::Unknown(_)`) |
| The log (task 1) | `core/src/recording.rs`: `RecordingLog::new` `:60` (**unchanged**), `play` `:77`–`:86`, `pause` `:88`–`:97`; `speed` after `pause` |
| The format (task 1) | `core/src/store.rs:21` (18 → 19); `core/tests/project_format.rs`: `a_v17_file_loads_under_the_current_version` `:734` is the shape of the new `a_v18_…` |
| Regions (task 2) | `core/src/audio.rs`: header mapping `:23`–`:24`, `Region` `:94`, `game_regions` `:211`–`:231` (the `==` `:221`), `region()` `:240`–`:254` (`source_offset` `:251`); literals `core/tests/audio.rs:107`, `:347`, `:415` |
| The reader (task 2) | `media/src/composite/audio.rs`: the chain `:314`–`:350` (`audioresample` `:330`, link `:349`), the mixer's seek call `:176`, `Reader::seek` `:457`–`:477` |
| The player (task 3) | `media/src/player/mod.rs`: `PLAYBIN_FLAGS` `:129`, `new` `:165` (`audio-sink` set at `:173`), `set_rate` `:343`–`:346` (`mute` `:345`), the trick-mode flags `:624`–`:626`; `packaging/smoke-test.sh:64`–`:74` |
| The bus (task 4) | `app/src/bus/transport.rs`: `SCAN_SPEEDS` `:22`, `next_speed` `:26`–`:35` (`unwrap_or(0)` `:28`), `set_playing` `:74`–`:84` (reset `:81`), `scan_speed` `:116`–`:124` (doc `:102`–`:115`), `change_rate` `:132`–`:146`, `store_rate` `:151`–`:155`, `heading` `:316`–`:326`, the pure test `:479`. `app/src/bus/mod.rs`: `ScanStep` `:68`–`:77`, `Command::ScanSpeed` `:344`–`:349`, `Event::ScanSpeed` `:558`–`:559`, the recording allow-list `:1128`–`:1162`, the whiteboard list `:1164`–`:1188` (notice `:1184`), dispatch `:1262`. `app/src/bus/recording.rs`: `log_playing` `:309`–`:319` (the shape of the speed log), the pause `:181`–`:184`, `whiteboard` `:223`, the `Active` `:224`–`:233`, `armed_slate = slate` `:241`. `app/src/bus/slates.rs:116`–`:121` (the doc) |
| `ScanSpeed` users | `Command`: `main.rs:797`–`:806` (the only sender in the app); harness `tests/transport.rs:356`, `:357`, `:388`, `:414`, `:428`, `tests/slates.rs:1001`, `:1010`, `tests/real_footage.rs:193`. `Event`: `main.rs:3605`, `harness/src/lib.rs:207`–`:212` (`wait_speed`), `tests/transport.rs:393`, `:433`, `:526`, `tests/slates.rs:1020` — **all unchanged**: `Event::ScanSpeed(f64)` keeps its shape |
| The window (task 5) | `app/ui/app.slint`: `ScanStep` `:241`–`:246`, `KeyAction` `:261` (`scan-slower` `:286`, `scan-faster` `:287`), `scan-speed` `:4363`, `can-scan-fast` `:4884`–`:4885`, the J/L branches `:5761`–`:5774`, the speed button `:7400`–`:7411` (text `:7402`, enabled `:7403`, tooltip `:7410`). `app/src/main.rs`: `on_step_scan_speed` `:797`–`:806`, `on_toggle_play` `:780`–`:788` (the capture's shape), the readout `:4668`–`:4672` |
| Keys (task 5) | `app/src/keymap.rs`: `Action` `:74`–`:106` (after `ScanFaster` `:99`), `ALL` `:117`–`:149` (its length, at `:117`, is the **one** count, and no prose here or anywhere states it), `name` `:155` (`:181`), `what` `:196` (`:222`), `when` `:236` (`:260`), `default_keys` `:281` (`:307`); the counted tests `the_defaults_are_conflict_free` `:963`, `the_list_holds_every_action_in_declaration_order` `:1004`. `app/src/key_action.rs:182`–`:183`. `app/tests/ui/keys.rs:144`–`:145` (`fires` rows), `:226`–`:228` |
| Message texts asserted | None this changes. The whiteboard notice (`bus/mod.rs:1184`) is matched by variant only (§0.5); `"bus: refused while recording"` is an `eprintln!` |
| Existing tests that pin a rule this changes | `harness/tests/transport.rs:381` `fast_scanning_runs_at_the_chosen_speed` — its "refused while paused" (Faster from 1×) and "refused while recording" (Faster at 1× in a take) **both still hold** under the cap; its sends change shape only. `harness/tests/slates.rs` `the_scan_keys_are_refused_while_a_range_is_armed` (`:990`) — Faster while armed is still refused; sends change shape only. `transport.rs:479`'s pure test gains cases, and its existing six asserts keep their values (at `max = 32`) except `next_speed(1.0, Slower)`, **which becomes 0.5** |
| Prose (task 6) | `CLAUDE.md:498`–`:505` (a v19 bullet after v18's), `:771` (the scan-speed paragraph), `:1194`–`:1197` (*"minus six"*, and the six named); `docs/book/src/guide/shortcuts.md:27`, `:30`–`:32`; `docs/book/src/guide/watching.md:26`–`:33`; `docs/hands-on-checklist.md` (§17 after §16, `:223`) |
| BACKLOG (task 6) | #140: its "Next, in order" line and its entry, at the end of the file. **Nothing else** |

**No site** in `project.rs` (no struct field), `undo.rs`, `state.rs` (no
`state.json` key) or the copy path (`composite/copy.rs`: no clips).

---

## 1. Core: `EventKind::Speed`, `SegmentKind::Play(rate)`, the walks, v19

### Files

- `core/src/event.rs`, `core/src/timeline.rs`, `core/src/export.rs`,
  `core/src/whole_match.rs`, `core/src/reel.rs`, `core/src/audio.rs` (one
  line, step 5), `core/src/recording.rs`, `core/src/store.rs`
- `core/tests/timeline.rs`, `core/tests/export.rs`, `core/tests/recording.rs`,
  `core/tests/project_format.rs`, and the `Play` → `Play(1.0)` renames in
  `core/tests/whole_match.rs`, `reel.rs`, `audio.rs`, `caption.rs`

### Steps

1. **`EventKind::Speed { rate: f64, source_time: f64 }`**, in `KnownKind`,
   `From` and `Serialize` beside `Play`. Its doc says what S1 says: an `f64`,
   anchored, and a non-positive or non-finite rate is a hand-edited file's.
2. **`SegmentKind::Play(f64)`**, `Eq` dropped (§0.7), its doc *"Source advances
   at the rate"*.
3. **`Motion { rate: f64, playing: bool }`** (private to `timeline.rs`):
   `fn apply(&mut self, kind: &EventKind) -> Option<f64>` applies `Play`,
   `Pause` and `Speed` (the guard: a rate that is not finite and `> 0` is
   ignored and returns `None`) and returns the anchor they carry; `fn speed()`
   is `rate` while playing, else 0. Both walks call it; `Skip` stays in each.
   If the two walks' call sites read worse than today's three-line arms, write
   the arms twice instead and say so in "Where this stands" (spec S4's
   fallback).
4. **`emit`**: `speed > 0` replaces `rate == 1.0`; `play_dur =
   dur.min(available / speed)`, the source advancing by `play_dur × speed`;
   the push is `Play(speed)`. **`source_time`**'s accumulation multiplies by
   `speed()` as it does by `rate` today.
5. **`export::walk`**: `Play(rate) => seg.source_start + rate × (t −
   out_start).max(0.0)`. `whole_match.rs` and `reel.rs` write `Play(1.0)`.
   `audio.rs:221` becomes `if let SegmentKind::Play(_) = seg.kind` — **reading
   at 1× as today**: the rate reaches the region in task 2, and no head can
   record a slow span before task 4.
6. **`RecordingLog::speed(host_ns, source_seconds, rate)`**, `pause`'s shape.
7. **v19**: `store.rs:21`.

### Tests

**core** (`cargo test -p pundit-core`; the binaries named):

- 1.1 `--lib event`: `speed_has_the_expected_wire_shape` —
  `{"speed":{"rate":0.5,"sourceTime":1.5}}`, literal; and it round-trips.
- 1.2 `--test timeline`: `a_speed_event_slows_the_source_from_its_anchor` — a
  `Speed { 0.5, 10.0 }` at 2 s in a playing log gives `Play(0.5)` from 10 s,
  and `source_time` at 4 s is 11.0.
- 1.3 `--test timeline`: `a_pause_and_a_play_keep_the_speed` — `Speed 0.5`,
  `Pause`, `Play`: the segment after the play is `Play(0.5)`.
- 1.4 `--test timeline`: `a_slow_span_reaching_the_end_freezes_the_rest` — a
  5 s source, `Speed 0.5` anchored at 4 s, 4 s of record time after it:
  `Play(0.5)` for 2 s of record time, then `Freeze` capped at the end.
- 1.5 `--test timeline`: `a_speed_of_zero_or_nan_changes_nothing` — the
  segments equal the same log without the event.
- 1.6 `--test timeline`: the existing cross-check (`:297`'s walk) extended with
  a slow log — segments and `source_time` agree to 1e-9 everywhere before EOF.
- 1.7 `--test export`: `a_slow_span_ends_on_the_source_time_its_rate_says` —
  a `one_clip` at ½× and at ¼× from anchor 10 s over 2 s of record time: the
  last `FrameSpec::source_time` is `10 + rate × (59/30)`.
- 1.8 `--test recording`: `speed_is_logged_at_its_own_record_time` — after a
  play, `speed(t, 3.0, 0.5)` appends `Speed { 0.5, 3.0 }` at `t`'s record time,
  clamped like `pause`.
- 1.9 `--test project_format`: `a_v18_file_loads_under_the_current_version`.
- 1.10 every existing `timeline`, `export`, `whole_match`, `reel`, `caption`,
  `cues`, `scoreboard`, `highlight`, `zoom`, `stroke_replay` test, renamed only.

### Sabotage

1. `Motion::apply`'s `Play` arm resets `rate` to 1 → 1.3 only.
2. `emit` uses `available` for `play_dur` (no `/ speed`) → 1.4 only.
3. The guard removed → 1.5 only (a zero rate pushes a `Play(0.0)`).
4. `source_time` keeps multiplying by 1 while `emit` uses the speed → 1.6
   (and 1.2's `source_time` half).
5. `export::walk` ignores the rate → 1.7 only.
6. `Serialize` writes `"speed"` as `"rate"`'s key → 1.1 only.

---

## 2. Media: the export's game sound, slowed at its pitch

### Files

- `core/src/audio.rs`, `core/tests/audio.rs`
- `media/src/composite/audio.rs`, `media/tests/export.rs`

### Steps

1. **`Region { …, rate: f64 }`**: 1.0 for commentary and music, the segment's
   rate for a game region (`game_regions` reads `Play(rate)`). `region()` takes
   the rate and adds `rate × dropped / AUDIO_SAMPLE_RATE` to `source_offset`.
   The header's mapping (`:23`–`:24`) gains `rate ×`.
2. **`scaletempo`** between the `audioconvert` and the `audioresample`, with a
   second `audioconvert` after it (spec M1's chain): `make("scaletempo")`, added
   and linked with the others. Its comment cites M1's bit-exactness at 1.
3. **`Reader::seek(seconds, rate)`**: `pipeline.seek(rate, FLUSH | ACCURATE,
   SeekType::Set, position, SeekType::None, ClockTime::NONE)`. The mixer passes
   the region's rate (`:176`).

### Tests

**core** (`cargo test -p pundit-core --test audio`):

- 2.1 `a_slow_segment_is_a_game_region_at_its_rate` — a `Play(0.5)` segment's
  region has `rate: 0.5`; the commentary's has 1.0.
- 2.2 `the_priming_drop_is_scaled_by_the_rate` — the first region of a ½×
  entry: `source_offset` is `source_start + 0.5 × 1024 / 48000`.

**media** (`cargo test -p pundit-media --test export`):

- 2.3 `a_slow_clip_shows_each_source_frame_rate_times` — a `counter_video` of
  `CounterKind::H264Mp4BFrames`, 30 fps, exported through a clip whose log is
  one `Speed` at 0: at ½× every counter in the span decodes exactly twice, in
  order, and at ¼× four times; the last counter is the anchor plus `rate ×` the
  span. (Picture only, but it is the first test that exercises task 1's walk
  through a real decode, and it belongs beside the frame-counting tests.)
- 2.4 `a_slow_clip_stretches_the_game_sound_at_its_pitch` — a `tone_video` with
  a 440 Hz `Tone` windowed to `(0.0, 2.0)`, a clip at ½× and one at ¼× from 0:
  the tone's end, read from the decoded audio as the first 10 ms window below
  a tenth of its level, is at output 4 s and 8 s, early by at most one 30 fps
  source frame of source time (67 ms ½×, 133 ms ¼× of output) and late by at
  most a block; and `tone_level(…, 440.0)` over the span's middle second reads
  the tone's amplitude within 10%.
- 2.5 every existing `export`, `avatar`, `transcribe` test — the reader at 1×
  is bit-exact (spec M1), so none changes.

### Sabotage

1. `scaletempo` left out of the chain → 2.4 (the tone ends at output 2 s).
2. `Reader::seek` seeks at 1.0 → 2.4 (the same).
3. `game_regions` writes `rate: 1.0` → 2.1 and 2.4.
4. `region()` drops the `rate ×` on the priming → 2.2 only.

---

## 3. Media: the live player slows its sound

### Files

- `media/src/player/mod.rs`, `media/src/player/tests.rs`
- `packaging/smoke-test.sh`

### Steps

1. **`audio-filter`**: `SourcePlayer::new` builds a `scaletempo` and sets it
   on the `playbin3`, beside `audio-sink` (`:173`). Its comment cites M3:
   `playsink` inserts none, and without it the sink is starved at ½×.
2. **`set_rate` mutes only above 1**: `rate > 1.0`. The doc (`:337`–`:342`)
   already says so.
3. **`scaletempo`** in `smoke-test.sh`'s `elements=(…)`, beside `audioresample`.

### Tests

**media** (`cargo test -p pundit-media --lib player`):

- 3.1 `a_slow_rate_is_not_muted` — `set_rate(0.5)` leaves the pipeline's
  `mute` `false`; `set_rate(2.0)` sets it.
- 3.2 every existing `player::tests` test, unchanged.

### Sabotage

1. `rate != 1.0` restored → 3.1 only.

(The filter itself has no automated test, by the spec review's decision: what
would fail without it is the sound the sink is fed, which the harness's
`fakesink` cannot hear. It is manual check 2.)

---

## 4. The bus: the ladder, the log, the pause rule

### Files

- `app/src/bus/transport.rs`, `app/src/bus/mod.rs`, `app/src/bus/recording.rs`,
  `app/src/bus/slates.rs` (the doc)
- `app/src/main.rs` (the `ScanSpeed` send's new shape only, `ScanStep::Cycle`
  etc. mapped as now; task 5 adds the capture)
- `harness/src/lib.rs`, `harness/tests/transport.rs`, `harness/tests/slates.rs`,
  `harness/tests/whiteboard.rs`, `harness/tests/recording.rs`,
  `harness/tests/real_footage.rs`

### Steps

1. **The ladder**: `SCAN_SPEEDS = [0.25, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0, 32.0]`;
   `next_speed(rate, step, max)` per spec S6 — `Faster`/`Slower` stop at the
   ends and at `max`; `Cycle` wraps past `max` to 1×, or to ¼× when that would
   be no change; `ToggleSlow` → 1× from below 1, else ½×; the fallback index is
   1×'s.
2. **`ScanStep::ToggleSlow`**; **`Command::ScanSpeed { step, host_ns,
   source_secs }`**, its doc rewritten (spec §C). Dispatch passes all three.
3. **`scan_speed(step, host_ns, source_secs)`**: refuse under a preview; `max`
   per §0.2; `anchor = heading(source_secs).1`; the new rate; if changed,
   `change_rate(rate, Some(anchor))` (§0.1), then — while recording —
   `active.log.speed(host_ns, anchor, rate)`.
4. **`set_playing`'s reset**: `self.player.rate() > 1.0`, its doc (`:70`–`:73`)
   saying a pause returns from fast and keeps slow.
5. **The allow-list** gains `Command::ScanSpeed { .. }`, with a comment in its
   neighbours' voice (the footage's speed is part of the take); **the
   whiteboard list** gains it, the notice reading *"…so it can't be played,
   skipped, slowed, tagged, marked or ringed"*, and its comment's "minus six"
   becomes "minus seven".
6. **A take started slow**: after the `Active` is built (`:224`–`:233`), when
   `!whiteboard` and `self.player.rate() != 1.0`, `log.speed(t0_ns,
   start_source_seconds, rate)`.
7. **`slates.rs:116`–`:121`**: a slate preview plays at the standing speed,
   never above 1× (spec S12).
8. **`Harness::scan(step)`** (§0.3), and every harness send in §A moved onto
   it.

### Tests

**app** (`cargo test -p pundit-app --lib bus::transport`):

- 4.1 `scan_speeds_clamp_and_the_button_wraps` (`:479`), extended: `Slower`
  from 1× is ½× and from ¼× stays; `Faster` at `max = 1.0` stays; `Cycle` from
  32× (max 32) is 1×, from 1× under max 1 is ¼×, from ½× under max 1 is 1×;
  `ToggleSlow` from 1×, 4× → ½×, from ¼× → 1×; an off-ladder rate falls to 1×.

**harness** (`cargo test -p pundit-harness --test <name>`):

- 4.2 `--test recording`: `s_during_a_take_logs_a_speed_at_the_callers_moment`
  — play in a take, `scan(ToggleSlow)` with a captured `host_ns` and position:
  the clip's log holds `Speed { 0.5, ≈position }` at that record time, and the
  player's rate is 0.5.
- 4.3 `--test recording`: `a_take_caps_the_speed_at_1x` — `scan(Faster)` at 1×
  in a take: no `Event::ScanSpeed`, no `Speed` in the log.
- 4.4 `--test transport`: `a_pause_keeps_slow_motion_and_drops_fast` — ½×,
  pause, play: still 0.5 and no `ScanSpeed(1.0)`; 4×, pause: `ScanSpeed(1.0)`.
- 4.5 `--test recording`: `a_take_started_slow_opens_with_its_speed` — ½× while
  scanning, start a take, stop: the log's first events are the zoom, the pause
  and `Speed { 0.5 }` at 0.
- 4.6 `--test whiteboard`: `slow_motion_is_refused_in_a_whiteboard_take` — ½×,
  start a W take: no `Speed` in its log; `scan(ToggleSlow)` during it is a
  `UserError::Whiteboard` notice.
- 4.7 `--test slates`: `slow_motion_works_in_a_slate_take` — shoot a slate,
  `scan(ToggleSlow)`: the rate is 0.5 and logged; and `scan(Faster)` from 1×
  while armed is still refused.
- 4.8 `--test transport` `fast_scanning_runs_at_the_chosen_speed` and `--test
  slates` `the_scan_keys_are_refused_while_a_range_is_armed`, sends moved onto
  `scan` and **every assertion as it was**.

### Sabotage

1. The allow-list entry left out → 4.2 (no speed; the guard's `eprintln!`).
2. The bus logs `now_ns()` instead of the caller's `host_ns` → 4.2 only (the
   record time).
3. `max` ignores `recording` → 4.3 only.
4. `set_playing` back to `!= 1.0` → 4.4 only.
5. The take-start `speed` call left out → 4.5 only; its `!whiteboard`
   dropped → 4.6 only.
6. The armed refusal kept whole (`armed_slate.is_some()` returns) → 4.7 only.

---

## 5. The window: `S`, the gate, the capture, the readout

### Files

- `app/src/keymap.rs`, `app/src/key_action.rs`, `app/ui/app.slint`,
  `app/src/main.rs`
- `app/tests/ui/keys.rs`, `app/tests/ui/speed_button.rs` (new: no UI test
  covers the button today), `app/tests/ui/main.rs` (its `mod` list, between
  `snap_menu` `:102` and `splitter` `:103`)

### Steps

1. **`Action::SlowMotion`** after `ScanFaster` — `name` `"slowMotion"`, `what`
   *"Slow motion"*, `when` *"With the game video on screen"*, `default_keys`
   `&["s"]` — in `ALL` at the same place. `ScanSlower`/`ScanFaster`'s `when`
   becomes *"With the game video on screen"* too (spec §C).
2. **`KeyAction.slow-motion`**, `key_action.rs`'s arm, and a `handle-key`
   branch beside J/L's sending `step-scan-speed(ScanStep.toggle-slow)` under
   `can-scan-fast`.
3. **`can-scan-fast`** redefined in place: `root.can-play && !root.previewing`
   (spec S10), its comment and J/L's (`:5761`–`:5762`) rewritten; the button's
   tooltip *"Speed (J slower, L faster, S slow motion)"*.
4. **`main.rs`**: `on_step_scan_speed` captures `now_ns()` and
   `position.query_position()` as `on_toggle_play` does, and maps
   `ScanStep::ToggleSlow`; the readout's `s > 1.0` becomes `s != 1.0`.

### Tests

**app UI** (`cargo test -p pundit-app --test ui`):

- 5.1 `keys::…` — `fires(Action::SlowMotion, "s", "toggle slow")` in the
  existing table (`:144`–`:145`), with the `:226`–`:228` arm.
- 5.2 `speed_button::the_speed_button_is_live_while_paused_and_recording` — `can-play`
  true, `playing` false: the button (found by its words, `0.5×` after
  `set_scan_speed(0.5)`) is enabled; and with `recording` true.
- 5.3 `--lib keymap`: `the_defaults_are_conflict_free` and
  `the_list_holds_every_action_in_declaration_order`, passing with the new
  row — the one count is `ALL`'s length.

**Not reachable**: `main.rs`'s capture and readout (the binary; a UI test
links the library). The capture is the same three lines as `on_toggle_play`'s;
manual check 1 sees both.

### Sabotage

1. `SlowMotion` left out of `ALL` → 5.3 (`the_list_holds_every_action…`).
2. `can-scan-fast` keeps `root.playing` → 5.2 only.
3. `key_action.rs` maps `SlowMotion` to `ScanSlower` → 5.1 only.

---

## 6. Prose and the backlog

### Files

- `CLAUDE.md`, `docs/book/src/guide/shortcuts.md`,
  `docs/book/src/guide/watching.md`, `docs/hands-on-checklist.md`,
  `BACKLOG.md`

### Steps

1. **`CLAUDE.md`**: the v19 bullet after v18's (`:498`–`:505`), in its shape —
   `EventKind::Speed`, no struct field, the floor stays 7, the forward reason;
   the scan-speed paragraph (`:771`) — the ladder, the cap, pause keeps slow,
   `scaletempo` on both paths with M1's bit-exactness and M4's starvation; the
   whiteboard bullet (`:1194`–`:1197`) — minus seven, `ScanSpeed` named. One
   sentence in the export paragraph's audio bullet: the reader seeks each
   region at its rate.
2. **The book**: `shortcuts.md`'s J/L row and the paragraph after it, an `S`
   row; `watching.md`'s *Scanning fast* gains *Slow motion* — the speeds,
   pause keeps it, it records into the take and the export, the sound keeps
   its pitch.
3. **The checklist**: §17, the manual checks below.
4. **BACKLOG #140**: SHIPPED (#TODO), off "Next, in order" — at the
   coordinator's word, ahead of the merge.

### Tests

None; `recheck-claims` over every line changed.

---

## Manual checks (the checklist's §17)

1. **The keys and the readout**: `S` while playing shows ` · 0.5×` and the
   button `0.5×`; `J` to `0.25×`; pause and play keep it; `S` again is 1×.
2. **The sound** through the real speakers at 1×, ½× and ¼×: the pitch is the
   game's, and **the A/V offset** at each (spec Risk 2; CLAUDE.md's ~+1 ms at
   1× is the baseline).
3. **The hitch and the lag** (spec Risk 1, S5): during a take on real footage,
   press `S` with a player running and draw on him at once; stop, export, and
   measure how far ahead of him the stroke lands. Record the number in the
   spec.
4. **#72's stall with the filter** (spec Risk 3): `PUNDIT_SOAK_ROUNDS=500
   taskset -c 0,1 cargo test -p pundit-media player::tests::every_open_settles
   -- --ignored --nocapture`, four at once, against the numbers in its own doc
   (`media/src/player/tests.rs:740`–`:755`).
5. **A slow take's export and preview**: the clip previews slow, exports slow
   with the sound slowed, the scoreboard clock runs slow across the span, and a
   whole-match copy of the same project is untouched.
6. **A v19 project in the previous release** is refused, and
   `project.json.v18` opens there.

---

## Risks

1. **Task 1's walk change touches every export.** Mitigated by 1.10 — every
   existing timeline, export and overlay test passes with `Play(1.0)` — and by
   1.5/1.6, which pin a log with no `Speed` to today's segments.
2. **`Motion` may not be clean** (task 1 step 3). The fallback is written in
   the step; either is fine, and the choice is recorded.
3. **Task 2's onset bound is M2's numbers on synthetic audio.** If 2.4's bound
   fails on the fixture's raw-F32 Matroska where M2 measured WAV, the bound is
   re-measured on the fixture and the spec's M2 is corrected — the bound is
   never loosened past one source frame without the coach.
4. **The spec's Risks 1–3 are manual**, by nature (a real sink, real footage,
   two pinned cores). They are checks 2–4; none blocks the merge, and each
   result goes into the spec.

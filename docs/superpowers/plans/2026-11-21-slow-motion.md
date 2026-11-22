# Plan — slow motion during a recording (#140)

Spec: `docs/superpowers/specs/2026-11-21-slow-motion-design.md` (after its
review fixes and the plan review's amendments). Read it and `CLAUDE.md` first;
neither is repeated here except where §0 adds to the spec. **Every task starts
with the `start-task` skill and ends with `verify`, `sabotage-proof` and
`recheck-claims`.**

Every `path:line` here was re-derived on **2026-11-22 at `a94248c`** (`main` at
`d742430` plus this branch's commits, which touch only `docs/` and
`BACKLOG.md`). Paths are from the repository root; `core`, `media`, `app` and
`harness` mean `crates/pundit-<name>/`. **§A holds every citation**; the tasks
cite only what §A does not.

**This plan's `BACKLOG.md` edits are #140's entry and its "Next, in order"
line, and nothing else.**

## Where this stands (update it as tasks land)

- Nothing started. The spec and this plan are on `claude/slow-motion-spec`.
- **The plan's review (2026-11-22)** amended the spec in five places, each
  marked there: M5 (a re-seeked `scaletempo` leaks the old position's sound —
  re-measured), S1 (the bad-rate guard is in the decoder), S4 (no `Motion`
  type), S7 (a reader that has stretched is replaced), S10 (`can-scan`), and T2
  (the repeated-frames test dropped).

**Shipping: one PR, a commit per task, in order.** Task 1 bumps the format to
v19, and on its own it would re-stamp every project a coach saved with nothing
that needs v19 — an older build would then refuse files that hold no `Speed`
event at all. Tasks 2 and 3 are media changes that are passthrough at 1× (spec
M1) and have nothing to slow until task 4 logs a speed; reviewed apart they
would be reviewed without their reason. Each commit is still verifiable on its
own: its tests, and every existing one, pass at its own head, and no head
between them can record a slow span — the bus has no speed below 1× until
task 4. **Two manual checks gate the merge** (4 and 5 below: the #72 soak and
the 32× real-footage rerun), because each tests a change to the live player's
graph that nothing in CI can see.

---

## §0. What the spec does not say (checked 2026-11-22)

1. **`change_rate(rate, at: Option<f64>) -> Option<f64>`**, returning where it
   sent the footage, which is what a take logs as the anchor. Both branches:
   - **the early return** (`seek_waiting()`, not seekable, not loaded —
     `transport.rs:134`–`:136`) stores the rate and returns `at`: a seek still
     to be issued carries the rate, and its target is what `heading` answered;
   - **the seek**: the target in flight if there is one; **else, while paused,
     `shown_secs()` as today** (the frame on screen); else, while playing,
     `at.unwrap_or(current_secs())`. It `load`s there and returns it.
   `scan_speed` passes `Some(heading(source_secs).1)` and logs what comes back;
   `set_playing`'s fast reset passes `None` and ignores it, which is today's
   behaviour exactly (`transport.rs:137`–`:144`).
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
4. **`Region`'s literals take `rate: 1.0`** — `core/tests/audio.rs:107`,
   `:347`, `:415` and **`media/tests/export.rs:2576`** (a music region); the
   only other `Region {` is `core/src/audio.rs:248`.
5. **The whiteboard notice is not asserted verbatim anywhere**:
   `harness/tests/whiteboard.rs:146` matches the variant and `is_notice` only.
   Adding "slowed" breaks no test.
6. **Slint's `ScanStep` gains `toggle-slow`** (`app.slint:242`–`:246`), so two
   exhaustive matches over it gain an arm: `main.rs:800`–`:804` (task 4) and
   `app/tests/ui/keys.rs:225`–`:229` (task 5) — the latter's string is
   `format!("scan {which}")` (`:230`), so the new one reads `"scan toggle-slow"`.
7. **`SegmentKind`'s `Eq` goes** (`timeline.rs:33`). Checked: nothing hashes it,
   bounds it on `Eq`, or uses it as a map key; `PlaybackSegment` already
   derives only `PartialEq` (`timeline.rs:42`).
8. **The keymap's counts move**: `ALL: [Action; 31]` (`keymap.rs:117`) and
   `the_defaults_are_conflict_free`'s `36` and its message *"31 actions, 36
   bindings"* (`keymap.rs:979`–`:983`) become 32 / 37. Those two are the only
   places the counts are written.
9. **The book says J/L are "while playing… 1× up to 32×"**
   (`docs/book/src/guide/shortcuts.md:27`, `:30`–`:32`;
   `docs/book/src/guide/watching.md:26`–`:33`). Task 6 rewrites both and adds
   `S`.
10. **No CHANGELOG edit.** It is curated at release.

---

## §A. The census, 2026-11-22 at `a94248c`

| Changing | Sites |
|---|---|
| The event (task 1) | `core/src/event.rs`: `EventKind` `:67`–`:87`, `KnownKind` `:98`–`:107`, `From` `:109`–`:120`, `Serialize` `:122`–`:144` (each gains a `Speed` arm), `Deserialize` `:152`–`:164` (the bad-rate guard); the wire-shape tests `:181`–`:236`, the Unknown tests `:286`–`:302` |
| The walks (task 1) | `core/src/timeline.rs`: `SegmentKind` `:33`–`:39` (derive `:33`, `Play` doc `:35`); `source_time` `:79`–`:116` (`rate` `:84`, its match `:97`–`:110`); `Walk` `:132`–`:142` (`rate` `:136`); `emit` `:145`–`:185` (the `rate == 1.0` test `:154`, `Play` push `:163`); `playback_segments`' `Walk` literal and match `:222`–`:251`; docs `:67`–`:68`, `:121`–`:122` |
| `SegmentKind::Play` sites (task 1) | `core/src/export.rs:209` (the walk's map; docs `:4`–`:7`, `:176`–`:178`); `core/src/audio.rs:221` (task 2's, but it must compile in task 1 — task 1 step 4); `core/src/whole_match.rs:37`; `core/src/reel.rs:206`. Tests: `core/tests/timeline.rs:133`, `:149`, `:163`, `:192`, `:207`, `:297`, `:386`; `core/tests/whole_match.rs:62`; `core/tests/reel.rs:56`; `core/tests/audio.rs:143`; `core/tests/caption.rs:24` |
| `EventKind` exhaustive matches | `timeline.rs:97`–`:110` and `:232`–`:251` (both gain `Speed`), `event.rs`'s codec. **No other** match names every variant (grepped `EventKind::Unknown(_)`) |
| The log (task 1) | `core/src/recording.rs`: `RecordingLog::new` `:60` (**unchanged**), `play` `:77`–`:86`, `pause` `:88`–`:97`; `speed` after `pause`. `core/tests/recording.rs:50`–`:63` (`play_and_pause_log_caller_times_and_anchors`) |
| The format (task 1) | `core/src/store.rs:21` (18 → 19); `core/tests/project_format.rs`: `a_v17_file_loads_under_the_current_version` `:734` is the shape of the new `a_v18_…` |
| Regions (task 2) | `core/src/audio.rs`: header mapping `:23`–`:24`, `Region` `:94`, `audio_regions`' doc `:156`–`:159`, `game_regions` `:202`–`:231` (doc `:202`–`:203`, the `==` `:221`), `region()` `:240`–`:254` (`source_offset` `:251`); literals per §0.4 |
| The reader (task 2) | `media/src/composite/audio.rs`: `Mixer` `:80`–`:93` (`readers` `:92`), the region's arrival and seek `:168`–`:179` (seek `:176`), `reader()` `:234`–`:243`, `Reader` `:263`–`:283`, the chain `:314`–`:350` (`audioresample` `:330`, link `:349`), `Reader::seek` `:457`–`:477`, `mod tests` `:712`. `packaging/smoke-test.sh:64`–`:74` |
| The player (task 3) | `media/src/player/mod.rs`: `PLAYBIN_FLAGS` `:129`, `new` `:165` (`audio-sink` set at `:173`), `set_rate` `:343`–`:346` (`mute` `:345`), the trick-mode flags `:624`–`:626`; `media/src/player/tests.rs:630`–`:639` (`a_fast_rate_mutes_the_sound`), `:740`–`:757` (`every_open_settles`) |
| The bus (task 4) | `app/src/bus/transport.rs`: `SCAN_SPEEDS` `:22`, `next_speed` `:26`–`:35` (`unwrap_or(0)` `:28`), `set_playing` `:74`–`:84` (reset `:81`), `scan_speed` `:116`–`:124` (doc `:102`–`:115`), `change_rate` `:132`–`:146`, `store_rate` `:151`–`:155`, `heading` `:316`–`:326`, the pure test `:479`. `app/src/bus/mod.rs`: `ScanStep` `:68`–`:77`, `Command::ScanSpeed` `:344`–`:349`, `Event::ScanSpeed` `:558`–`:559`, the recording allow-list `:1128`–`:1162`, the whiteboard list `:1164`–`:1188` (notice `:1184`), dispatch `:1262`. `app/src/bus/recording.rs`: `log_playing` `:309`–`:319` (the shape of the speed log), the pause `:181`–`:184`, `whiteboard` `:223`, the `Active` `:224`–`:233`, `armed_slate = slate` `:241`. `app/src/bus/slates.rs:116`–`:121` (the doc). `app/src/main.rs:797`–`:806` (`on_step_scan_speed`, given `on_toggle_play`'s capture, `:780`–`:788`) |
| `ScanSpeed` users | `Command`: `main.rs:797`–`:806` (the only sender in the app); harness `tests/transport.rs:356`, `:357`, `:388`, `:414`, `:428`, `tests/slates.rs:1001`, `:1010`, `tests/real_footage.rs:193`. `Event`: `main.rs:3605`, `harness/src/lib.rs:207`–`:212` (`wait_speed`), `tests/transport.rs:393`, `:433`, `:526`, `tests/slates.rs:1020` — **all unchanged**: `Event::ScanSpeed(f64)` keeps its shape |
| The window (task 5) | `app/ui/app.slint`: `ScanStep` `:241`–`:246`, `KeyAction` `:261` (`scan-slower` `:286`, `scan-faster` `:287`), `scan-speed` `:4362`–`:4363`, `can-scan-fast` `:4884`–`:4885` → `can-scan`, its users `:5764`, `:5770`, `:7403`, the J/L branches `:5761`–`:5774`, the speed button `:7400`–`:7411` (text `:7402`, tooltip `:7410`). `app/src/main.rs`: the readout `:4668`–`:4672` |
| Keys (task 5) | `app/src/keymap.rs`: `Action` `:74`–`:106` (after `ScanFaster` `:99`), `ALL` `:117`–`:149`, `name` `:155` (`:181`), `what` `:196` (`:222`), `when` `:236` (`:260`), `default_keys` `:281` (`:307`); the counted test `the_defaults_are_conflict_free` `:963`–`:983` (§0.8), `the_list_holds_every_action_in_declaration_order` `:1004`. `app/src/key_action.rs:182`–`:183`. `app/tests/ui/keys.rs:144`–`:145` (`fires` rows), `:225`–`:230`, `the_first_range_key_waits_out_a_take_and_a_preview` `:513`–`:533` (5.2's pattern) |
| Message texts asserted | None this changes. The whiteboard notice (`bus/mod.rs:1184`) is matched by variant only (§0.5); `"bus: refused while recording"` is an `eprintln!`. The keymap's count message is §0.8 |
| Existing tests that pin a rule this changes | `harness/tests/transport.rs:381` `fast_scanning_runs_at_the_chosen_speed` — its "refused while paused" (Faster from 1×) and "refused while recording" (Faster at 1× in a take) **both still hold** under the cap; its sends change shape only. `harness/tests/slates.rs:990` `the_scan_keys_are_refused_while_a_range_is_armed` — Faster while armed is still refused; sends change shape only. `transport.rs:479`'s pure test gains cases, and its existing six asserts keep their values (at `max = 32`) except `next_speed(1.0, Slower)`, **which becomes 0.5**. `harness/tests/real_footage.rs:175` `real_footage_fast_scanning` (`#[ignore]`d) — manual check 5 |
| Prose (task 6) | `CLAUDE.md:498`–`:505` (a v19 bullet after v18's), `:771` (the scan-speed paragraph), `:1194`–`:1197` (*"minus six"*, and the six named); `docs/book/src/guide/shortcuts.md:27`, `:30`–`:32`; `docs/book/src/guide/watching.md:26`–`:33`; `docs/hands-on-checklist.md` (§17 after §16, `:223`) |
| BACKLOG (task 6) | #140: its "Next, in order" line and its entry, at the end of the file. **Nothing else** |

**No site** in `project.rs` (no struct field), `undo.rs`, `state.rs` (no
`state.json` key) or the copy path (`composite/copy.rs`: no clips).

---

## 1. Core: `EventKind::Speed`, `SegmentKind::Play(rate)`, the walks, v19

### Files

- `core/src/event.rs`, `core/src/timeline.rs`, `core/src/export.rs`,
  `core/src/whole_match.rs`, `core/src/reel.rs`, `core/src/audio.rs` (one
  line, step 4), `core/src/recording.rs`, `core/src/store.rs`
- `core/tests/timeline.rs`, `core/tests/export.rs`, `core/tests/recording.rs`,
  `core/tests/project_format.rs`, and the `Play` → `Play(1.0)` renames in
  `core/tests/whole_match.rs`, `reel.rs`, `audio.rs`, `caption.rs`

### Steps

1. **`EventKind::Speed { rate: f64, source_time: f64 }`**, in `KnownKind`,
   `From` and `Serialize` beside `Play`. **`Deserialize`**: a decoded
   `KnownKind::Speed` whose rate is not `> 0` (`!(rate > 0.0)`, which is NaN
   too) returns `EventKind::Unknown(value)`, as a malformed known kind already
   does (spec S1).
2. **`SegmentKind::Play(f64)`**, `Eq` dropped (§0.7), its doc *"Source advances
   at the rate"*.
3. **Each walk gains `playing: bool` beside `rate`** (spec S4): `source_time`'s
   locals (`:84`) and `Walk`'s fields. `Play`/`Pause` set `playing` and anchor;
   a new `Speed { rate, source_time }` arm emits (in `playback_segments`) or
   accumulates (in `source_time`) first, then sets `rate` and the cursor
   (`clamp_source(source_time)`). The speed used to advance is `rate` while
   `playing`, else 0. `emit` tests `speed > 0.0` where it tested `rate == 1.0`;
   `play_dur = dur.min(available / speed)`, the source advancing by
   `play_dur × speed`; the push is `Play(speed)`.
4. **`export::walk`**: `Play(rate) => seg.source_start + rate × (t −
   out_start).max(0.0)`, its doc (`:176`–`:178`) and the module header
   (`:4`–`:7`) saying the rate. `whole_match.rs` and `reel.rs` write
   `Play(1.0)`. `audio.rs:221` becomes `if let SegmentKind::Play(_) = seg.kind`
   — **reading at 1× as today**: the rate reaches the region in task 2, and no
   head can record a slow span before task 4.
5. **`RecordingLog::speed(host_ns, source_seconds, rate)`**, `pause`'s shape.
6. **v19**: `store.rs:21`.

### Tests

**core** (`cargo test -p pundit-core`; the binaries named):

- 1.1 `--lib event`: `speed_has_the_expected_wire_shape` —
  `{"speed":{"rate":0.5,"sourceTime":1.5}}`, literal, and it round-trips.
- 1.2 `--lib event`: `a_speed_that_is_not_positive_decodes_as_unknown` —
  `{"speed":{"rate":0,"sourceTime":1}}`, and the same with `-0.5` and `-0.0`
  (JSON cannot carry a NaN; `!(rate > 0.0)` covers it anyway), each decode as `Unknown` holding the value
  verbatim.
- 1.3 `--test timeline`: `a_speed_event_slows_the_source_from_its_anchor` — a
  `Speed { 0.5, 10.0 }` at 2 s in a playing log gives a `Play(0.5)` segment
  starting at source 10 s.
- 1.4 `--test timeline`: `a_pause_and_a_play_keep_the_speed` — `Speed 0.5`,
  `Pause`, `Play`: the segment after the play is `Play(0.5)`.
- 1.5 `--test timeline`: `a_slow_span_reaching_the_end_freezes_the_rest` — a
  5 s source, `Speed 0.5` anchored at 4 s, 4 s of record time after it:
  `Play(0.5)` for 2 s of record time, then `Freeze` capped at the end.
- 1.6 `--test timeline`: the existing cross-check (`:297`'s walk) extended with
  a slow log — segments and `source_time` agree to 1e-9 everywhere before EOF.
- 1.7 `--test export`: `a_slow_span_ends_on_the_source_time_its_rate_says` —
  `schedule(clip(10.0, 2.0, vec![speed(0.0, rate, 10.0)]), 60.0)` (the file's
  own helpers, `core/tests/export.rs:25`, `:59`, and a `speed` beside `play`
  `:63`) at ½× and ¼×: the last `FrameSpec::source_time` is
  `10 + rate × 59/30`.
- 1.8 `--test recording`: `play_and_pause_log_caller_times_and_anchors`
  (`:50`) gains a `speed(at(3.0), 12.0, 0.5)` and its `Speed` in the expected
  slice.
- 1.9 `--test project_format`: `a_v18_file_loads_under_the_current_version`.
- 1.10 every existing `timeline`, `export`, `whole_match`, `reel`, `caption`,
  `cues`, `scoreboard`, `highlight`, `zoom`, `stroke_replay` test, renamed only.

### Sabotage

1. The `Play` arm resets `rate` to 1 → 1.4 only.
2. `emit` uses `available` for `play_dur` (no `/ speed`) → 1.5 only.
3. The decode guard removed → 1.2 only.
4. `source_time` advances at `playing as f64` while `emit` uses the speed →
   1.6 only.
5. `export::walk` ignores the rate → 1.7 only.
6. `Serialize` writes the key as `"slow"` → 1.1 only.

---

## 2. Media: the export's game sound, slowed at its pitch

### Files

- `core/src/audio.rs`, `core/tests/audio.rs`
- `media/src/composite/audio.rs` (and its `mod tests`), `media/tests/export.rs`
- `packaging/smoke-test.sh`

### Steps

1. **`Region { …, rate: f64 }`**: 1.0 for commentary and music, the segment's
   rate for a game region (`game_regions` reads `Play(rate)`). `region()` takes
   the rate and adds `rate × dropped / AUDIO_SAMPLE_RATE` to `source_offset`.
   The header's mapping (`:23`–`:24`) gains `rate ×`; the two docs on play
   segments (`:156`–`:159`, `:202`–`:203`) say a play segment carries a rate.
   §0.4's literals take `rate: 1.0`.
2. **`scaletempo`** between the `audioconvert` and the `audioresample`, with a
   second `audioconvert` after it (spec M1's chain): `make("scaletempo")`, added
   and linked with the others. Its comment cites M1's bit-exactness at 1 and
   M5's leak.
3. **`Reader::seek(seconds, rate)`**: `pipeline.seek(rate, FLUSH | ACCURATE,
   SeekType::Set, position, SeekType::None, ClockTime::NONE)`, and `Reader`
   remembers it (`rate: f64`, from 1.0).
4. **A reader that has stretched is replaced** (spec S7, M5): where a region
   arrives (`:168`–`:179`), if the path's reader's last rate is not 1, the
   mixer removes it from `readers` and opens a fresh one through `reader()`
   before the seek. Nothing else reopens.
5. **`scaletempo`** in `smoke-test.sh`'s `elements=(…)`, beside `audioresample`
   — in this task, because this is the first one whose code names it.

### Tests

**core** (`cargo test -p pundit-core --test audio`):

- 2.1 `a_slow_segment_is_a_game_region_at_its_rate` — a `Play(0.5)` segment's
  region has `rate: 0.5`; the commentary's has 1.0.
- 2.2 `the_priming_drop_is_scaled_by_the_rate` — the first region of a ½×
  entry: `source_offset` is `source_start + 0.5 × 1024 / 48000`.

**media** (`cargo test -p pundit-media`):

- 2.3 `--lib composite::audio`: `a_reseek_after_stretching_hands_back_no_old_sound`
  — a `tone_video` (`fixtures.rs:177`) whose 440 Hz `Tone` is windowed to
  `(2.0, 4.0)`, silent before. Through the **mixer's** arrival path (two game
  regions on the one file, both at ½×): the first reads from source 2.5 s, in
  the tone; the second from source 0.5 s, in the silence. No 10 ms window of
  the second region's first 100 ms holds the tone above a fifth of its level
  (M5's measurement), and its onset — source 2.0 s, output 3.0 s into the
  region — is within M2's bound. Driven through the mixer rather than
  `Reader::seek` alone, because the replacement is what is under test.
- 2.4 `--test export`: `a_slow_clip_stretches_the_game_sound_at_its_pitch` — a
  `tone_video` (`fixtures.rs:177`) with a 440 Hz `Tone` windowed to
  `(0.0, 2.0)`, a clip at ½× and one at ¼× from 0. **The real assertion is the
  duration**: the tone's end, read from the decoded audio as the first 10 ms
  window below a tenth of its level, is at output 4 s and 8 s, early by at most
  one 30 fps source frame of source time (67 ms of output at ½×, 133 ms at ¼×)
  and late by at most one output frame; `tone_level` (`export.rs:1958`) at
  440 Hz over the span's middle second is the tone's amplitude within 10%.
  **A third clip pauses and plays inside the slow span** on the same source
  (`Speed 0.5`, `Pause` at 1 s, `Play` at 2 s): its second slow region is the
  M5 case, and the tone's end lands where the log says.
- 2.5 every existing `export`, `avatar`, `transcribe` test — the reader at 1×
  is bit-exact (spec M1), so none changes.

### Sabotage

1. `scaletempo` left out of the chain → 2.4 (the tone ends at output 2 s) and
   2.3.
2. `Reader::seek` seeks at 1.0 → 2.4 and 2.3.
3. The replacement removed (step 4) → 2.3, and 2.4's third clip.
4. `game_regions` writes `rate: 1.0` → 2.1 and 2.4.
5. `region()` drops the `rate ×` on the priming → 2.2 only.

---

## 3. Media: the live player slows its sound

### Files

- `media/src/player/mod.rs`, `media/src/player/tests.rs`

### Steps

1. **Measure first**: does a flushing seek at slow speed leak pre-seek sound
   through `playbin3`'s `audio-filter=scaletempo`, as M5's reader does? A
   `gi` script in the spec's M3 shape (appsink, `sync=true`): play at ½×, seek
   elsewhere at ½×, count 10 ms windows of the old tone. **The rule:** if it
   leaks ≤ ~50 ms per seek, accept it and record the number in the spec's M5
   and in "Where this stands"; if more, stop and defer to the coordinator
   before step 2. The live player is not given M5's replacement either way —
   the player is one long-lived pipeline (spec D1) and rebuilding its audio
   chain per seek is not a change this feature makes on its own.
2. **`audio-filter`**: `SourcePlayer::new` builds a `scaletempo` and sets it
   on the `playbin3`, beside `audio-sink` (`:173`). Its comment cites M3:
   `playsink` inserts none, and without it the sink is starved at ½×.
3. **`set_rate` mutes only above 1**: `rate > 1.0`. The doc (`:337`–`:342`)
   already says so.

### Tests

**media** (`cargo test -p pundit-media --lib player`):

- 3.1 `a_fast_rate_mutes_the_sound` (`player/tests.rs:632`), extended:
  `set_rate(0.5)` leaves `mute` `false`.
- 3.2 every existing `player::tests` test, unchanged.

### Sabotage

1. `rate != 1.0` restored → 3.1 only.

(The filter itself has no automated test: what would fail without it is the
sound the sink is fed, which the harness's `fakesink` cannot hear. It is manual
checks 2, 3 and 4.)

---

## 4. The bus: the ladder, the log, the pause rule

### Files

- `app/src/bus/transport.rs`, `app/src/bus/mod.rs`, `app/src/bus/recording.rs`,
  `app/src/bus/slates.rs` (the doc)
- `app/src/main.rs`: `on_step_scan_speed` (`:797`–`:806`) gets
  `on_toggle_play`'s capture (`:780`–`:788`) — `now_ns()` and
  `position.query_position()` — and maps `ScanStep::ToggleSlow`
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
   `main.rs`'s send per this task's Files.
3. **`scan_speed(step, host_ns, source_secs)`**: refuse under a preview; `max`
   per §0.2; the new rate; if changed, `let anchor = change_rate(rate,
   Some(heading(source_secs).1))` (§0.1), then — while recording —
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
  `ToggleSlow` from 1× and 4× → ½×, from ¼× → 1×; `Faster` from an off-ladder
  3.0 is 2× (the fallback lands on 1×, then steps).

**harness** (`cargo test -p pundit-harness --test <name>`):

- 4.2 `--test recording`: `the_speed_keys_in_a_take_are_logged_at_the_callers_moments`
  — on `recording.rs`'s `Rig` (`:42`), play, then raw sends in the pattern of
  `:181`–`:188`: `Command::ScanSpeed { step: ToggleSlow, host_ns: t0 +
  250_000_000, source_secs: … }`, then `Slower` at `+ 500 ms`, `Faster` at
  `+ 750 ms` and `+ 1000 ms`, and `Faster` again at `+ 1250 ms`. The clip's log
  holds `Speed` 0.5, 0.25, 0.5 and 1.0 at record times 0.25, 0.5, 0.75 and 1.0
  exactly, each anchored inside the source, and **nothing** for the last press
  (the take's cap).
- 4.3 `--test transport`: `a_pause_keeps_slow_motion_and_drops_fast` — ½×,
  pause, play: still 0.5 and no `ScanSpeed(1.0)`; 4×, pause: `ScanSpeed(1.0)`.
- 4.4 `--test recording`: `a_take_started_slow_opens_with_its_speed` — ½× while
  scanning, start a take, stop: the log's first events are the zoom, the pause
  and `Speed { 0.5 }` at 0.
- 4.5 `--test whiteboard`: `slow_motion_is_refused_in_a_whiteboard_take` — ½×,
  start a W take: no `Speed` in its log; `scan(ToggleSlow)` during it is a
  `UserError::Whiteboard` notice.
- 4.6 `--test slates`: `slow_motion_works_in_a_slate_take` — shoot a slate,
  `scan(ToggleSlow)`: the rate is 0.5 and logged; and `scan(Faster)` from 1×
  while armed is still refused.
- 4.7 `--test transport` `fast_scanning_runs_at_the_chosen_speed` and `--test
  slates` `the_scan_keys_are_refused_while_a_range_is_armed`, sends moved onto
  `scan` and **every assertion as it was**.

### Sabotage

1. The allow-list entry left out → 4.2, 4.5 (the guard refuses ahead of the
   whiteboard clause, with no notice) and 4.6.
2. The bus logs `now_ns()` instead of the caller's `host_ns` → 4.2 only (the
   record times).
3. `max` ignores `recording` → 4.2 (the last press logs 2×) and
   `fast_scanning_runs_at_the_chosen_speed` (its "refused while recording").
4. `set_playing` back to `!= 1.0` → 4.3, and 4.4 (the take's opening pause
   returns it to 1×, so no `Speed` at 0).
5. The take-start `speed` call left out → 4.4 only; its `!whiteboard`
   dropped → 4.5 only.
6. The armed refusal kept whole (`armed_slate.is_some()` returns) → 4.6 only.

---

## 5. The window: `S`, the gate, the readout

### Files

- `app/src/keymap.rs`, `app/src/key_action.rs`, `app/ui/app.slint`,
  `app/src/main.rs` (the readout only)
- `app/tests/ui/keys.rs`

### Steps

1. **`Action::SlowMotion`** after `ScanFaster` — `name` `"slowMotion"`, `what`
   *"Slow motion"*, `when` *"With the game video on screen"*, `default_keys`
   `&["s"]` — in `ALL` at the same place; §0.8's counts. `ScanSlower`/
   `ScanFaster`'s `when` becomes *"With the game video on screen"* too.
2. **`KeyAction.slow-motion`**, `key_action.rs`'s arm, and a `handle-key`
   branch beside J/L's sending `step-scan-speed(ScanStep.toggle-slow)` under
   `can-scan`.
3. **`can-scan-fast` → `can-scan`**, redefined in place as
   `root.can-play && !root.previewing` (spec S10), at its four sites (§A); its
   comment and J/L's (`:5761`–`:5762`) rewritten; `scan-speed`'s doc
   (`:4362`) *"¼ to 32"*; the button's tooltip *"Speed (J slower, L faster, S
   slow motion)"*.
4. **`main.rs`'s readout**: `s > 1.0` becomes `s != 1.0`.

### Tests

**app UI** (`cargo test -p pundit-app --test ui`):

- 5.1 `keys::…` — `fires(Action::SlowMotion, "s", "scan toggle-slow")` in the
  existing table (`:144`–`:145`), with §0.6's arm.
- 5.2 `keys::the_speed_keys_work_while_paused_and_recording` — in
  `the_first_range_key_waits_out_a_take_and_a_preview`'s pattern (`:513`):
  with `playing` false, `j` fires `scan slower` and `s` fires
  `scan toggle-slow`; with `RecordingPhase::Recording` set, both still fire;
  with a preview open, neither does.
- 5.3 `--lib keymap`: `the_defaults_are_conflict_free` and
  `the_list_holds_every_action_in_declaration_order`, passing with the new
  row and §0.8's counts.

**Not reachable**: `main.rs`'s readout (the binary; a UI test links the
library). Manual check 1 sees it.

### Sabotage

1. `can-scan` keeps `root.playing` → 5.2 only.
2. `key_action.rs` maps `SlowMotion` to `ScanSlower` → 5.1 only.

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
   `scaletempo` on both paths with M1's bit-exactness, M4's starvation and M5's
   replaced reader; the whiteboard bullet (`:1194`–`:1197`) — minus seven,
   `ScanSpeed` named.
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
   1× is the baseline). And **a skip during slow motion**: whether the old
   position's sound is heard after it (task 3 step 1's number, by ear).
3. **The hitch and the lag** (spec Risk 1, S5): during a take on real footage,
   press `S` with a player running and draw on him at once; stop, export, and
   measure how far ahead of him the stroke lands. Record the number in the
   spec.
4. **Before merge — #72's stall with the filter** (spec Risk 3):
   `PUNDIT_SOAK_ROUNDS=500 taskset -c 0,1 cargo test -p pundit-media
   player::tests::every_open_settles -- --ignored --nocapture`, four at once,
   against the numbers in its own doc (`media/src/player/tests.rs:740`–`:755`).
5. **Before merge — 32× on real footage with the filter**:
   `PUNDIT_FOOTAGE=… cargo test -p pundit-harness --test real_footage
   real_footage_fast_scanning -- --ignored --nocapture`, against the 88 frames a
   second at most 0.12 s behind that `player/mod.rs:612`–`:616` records — the filter is in the chain at every rate,
   muted or not.
6. **A slow take's export and preview**: the clip previews slow, exports slow
   with the sound slowed, the scoreboard clock runs slow across the span, and a
   whole-match copy of the same project is untouched.
7. **A v19 project in the previous release** is refused, and
   `project.json.v18` opens there.

---

## Risks

1. **Task 1's walk change touches every export.** Mitigated by 1.10 — every
   existing timeline, export and overlay test passes with `Play(1.0)` — and by
   1.6, which pins a slow log's two walks to each other.
2. **Task 2's onset bound is M2's numbers on synthetic audio.** If 2.4's bound
   fails on the fixture's raw-F32 Matroska where M2 measured WAV, the bound is
   re-measured on the fixture and the spec's M2 is corrected — never loosened
   past one source frame without the coach.
3. **M5 may have a second trigger** the replacement does not cover — a reader
   stretched by one region and seeked at 1× by the next is replaced too (the
   rule is "was stretching", not "will stretch"), so the remaining exposure is
   a reader seeked twice *within* one region, which the mixer never does.
4. **Task 3's measurement can stop the plan** (its rule defers a leak over
   ~50 ms). If it does, tasks 4–6 wait; tasks 1–3 are still a coherent head.
5. **The spec's Risks 1–3 are manual**, by nature (a real sink, real footage,
   two pinned cores). They are checks 2–5; 4 and 5 gate the merge, and each
   result goes into the spec.

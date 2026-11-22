# Design — slow motion during a recording

BACKLOG **#140**, at the coach's request (2026-11-21): slow motion *"instant
replay style"* — the footage slowed to ½× or ¼× **while a take is recording**,
so the slowed passage is in the clip, its preview and every export that carries
it, not only on the scan view.

The coach believed this had been specced before. It had not: nothing under
`docs/` or in `BACKLOG.md` mentions slow motion, a rate below 1× or
`scaletempo` (grepped 2026-11-21). This is written fresh.

Read `CLAUDE.md` first — the bus contract (caller-captured timestamps), the clip
model, the format rules and their v7–v18 bullets, the composite, preview and
audio-mixing paragraphs, the scan-speed paragraph, the recording allow-list and
the keymap rules. This spec does not repeat them. Every `path:line` below was
re-derived from the tree at `d742430` on 2026-11-21.

---

## Decided with the coach (2026-11-21)

1. **Slow motion works during a recording**, so it reaches the exported clip
   and its preview — not just scanning.
2. **Two ways in:** a slow-motion key that toggles slow motion at the footage's
   current position, and the `J`/`L` scan-speed keys stepping **below** 1×,
   both during a take and while watching.
3. **The speeds are ½× and ¼×.**
4. **Nothing jumps back:** slow motion starts where the footage is.
5. **The game's sound is slowed with its pitch kept** (GStreamer's
   `scaletempo` or equivalent), live and in the export.

---

## §O. What is true today, and what this changes

**O1. `J`/`L` stop at 1× and only work while scanning.** `SCAN_SPEEDS` is
`[1.0, 2.0, 4.0, 8.0, 16.0, 32.0]` (`pundit-app/src/bus/transport.rs:22`);
`scan_speed` returns unless the footage is playing with no preview and no armed
slate (`transport.rs:116`–`:124`); and `Command::ScanSpeed` is **not** on the
recording allow-list (`bus/mod.rs:1128`–`:1162`), whose doc says why: *"Not
while recording: the clip model, replay and export are 1x"*
(`bus/mod.rs:344`–`:349`). That sentence is the premise this feature retires.

**O2. Any pause returns the game video to 1×** (`transport.rs:81`–`:82`), and
the player mutes at any rate other than 1 (`player/mod.rs:343`–`:346`). Both
were written for **fast** scanning and both change (§S12, §S9).

**O3. Replay knows two rates, 1 and 0.** `source_time` and `playback_segments`
walk the log with `rate` set to `1.0` by `Play` and `0.0` by `Pause`
(`pundit-core/src/timeline.rs:98`, `:102`, `:235`, `:240`); `Walk::emit` tests
`self.rate == 1.0` (`timeline.rs:154`); and `SegmentKind::Play`'s own doc says
*"Source advances at 1x"* (`timeline.rs:35`). Every consumer reads `Play` as 1×:

| Site | What it assumes |
|---|---|
| `core/src/export.rs:209` | `Play` maps output `t` 1:1 onto the source |
| `core/src/audio.rs:221` | `seg.kind == SegmentKind::Play` → a game region read at 1× |
| `core/src/whole_match.rs:37`, `core/src/reel.rs:206` | build `Play` segments directly (correctly 1×: no clip, no log) |
| `core/src/timeline.rs:154` | the play-tail/EOF split is in source seconds = record seconds |

Nothing else matches on `SegmentKind` outside tests (grepped).

**O4. The export's audio reader cannot slow anything.** Its chain is
`filesrc ! decodebin3 ! audioconvert ! audioresample ! appsink sync=false`
(`pundit-media/src/composite/audio.rs:314`–`:350`), and `Reader::seek` is
`seek_simple` at rate 1 (`audio.rs:457`–`:477`). Measured below (§M1): a seek
at rate ½ into that chain changes **nothing** — the appsink is not synced, so
the same samples come out at the same count and pitch. The sound needs an
element that applies the segment's rate.

---

## §M. Measured (2026-11-21, GStreamer 1.24.2, the reference laptop)

Python `gi` scripts against synthetic files — a 440 Hz tone that switches to
880 Hz at 3.000 s (H.264 + AAC in MP4, and raw in WAV), and a WAV silent until
a 440 Hz onset at 3.000 s. Each seeks to 2.000 s with `FLUSH | ACCURATE` at
the rate shown.

**M1. The export reader's chain, with and without `scaletempo`**
(`audioconvert ! scaletempo ! audioconvert ! audioresample`, `sync=false`):

| chain | rate | 440→880 switch, output s | expected | pitch |
|---|---|---|---|---|
| today's | 1, ½, ¼ | 1.02 every time | 1.02 / 2.04 / 4.08 | 440 Hz |
| + `scaletempo` | 1 | 1.02 | 1.02 | 440 Hz |
| + `scaletempo` | ½ | 2.00 | 2.04 | 440 Hz |
| + `scaletempo` | ¼ | 3.98 | 4.08 | 440 Hz |

(The 1.02 is the AAC test file's own priming.) Without `scaletempo` a rate is
silently ignored; with it the sound is stretched to the rate and keeps its
pitch. **At rate 1 `scaletempo` is passthrough: the samples are bit-identical**
to today's chain (`max |diff| = 0.0` over 48,000 samples), so adding it changes
no existing export, transcription or avatar level. Cost: 4.5 s of ½× sound
decoded in 0.32 s wall against 0.12 s without it.

**M2. `scaletempo` leads by a fraction of a source frame.** The WAV onset
(threshold 0.05 and 0.4 of a 0.5 sine agree to within 23 ms output):

| rate | onset, output s | ideal | lead, output | lead, source time |
|---|---|---|---|---|
| 1 (with `scaletempo`) | 1.0000 | 1.0000 | 0 | 0 |
| ½ | 1.9750 | 2.0000 | 25 ms | 12.5 ms |
| ¼ | 3.9026 | 4.0000 | 97 ms | 24.4 ms |

Its window is consumed ahead of its output. In **source** time — which is what
the eye judges against slowed motion — the lead is under one 30 fps source frame
at both rates, and the live player has the same element at the same defaults
(M3), so what the coach heard during the take is what the export carries. §S7
accepts it and pins the bound.

**M3. The live player needs `scaletempo` named; `playsink` does not insert
it.** `playbin3` with the app's `PLAYBIN_FLAGS` (`player/mod.rs:129`) and an
appsink for audio, `sync=true`, 2.5 s wall:

| `audio-filter` | rate | samples reaching the sink | pitch | `scaletempo` in graph |
|---|---|---|---|---|
| none | ½ | 61,696 (half of real time) | 440 Hz | no |
| `scaletempo` | ½ | 123,840 | 440 Hz | yes |
| `scaletempo` | ¼ | 126,720 | 440 Hz | yes |
| `scaletempo` | 1 | 122,112 | 440→880 on time | yes |

Without the filter the sink receives half the samples real time needs — 1×
sound under a ½× segment. `scaletempo` is in `gstreamer1.0-plugins-good` (`audiofx`),
already a run-time `Depends` (`pundit-app/Cargo.toml:89`), so the `.deb` needs
no new package — but the smoke test's element list must name it (§S11).

**M4. An instant rate change works on `playbin3` 1.24.2** (`SeekFlags::
INSTANT_RATE_CHANGE`, no position, fakesinks): 1× → ½× with no flush. **Not
used** — §S5 says why.

---

## §S. Decisions

**S1. A speed is `Speed::{Normal, Half, Quarter}`, a core enum, not an `f64`.**
In `pundit-core/src/event.rs` beside `EventKind`, with `factor() -> f64`
(1, 0.5, 0.25) and serde names `"normal"`, `"half"`, `"quarter"`. An enum is
what says in the type that a take has three speeds and not a continuum, it
keeps `SegmentKind`'s `Eq` derive (S3), and a future speed this build cannot
name decodes as `EventKind::Unknown` — kept verbatim and re-emitted, never a
silent wrong number. The player keeps its `f64` rate, which also carries 2×–32×;
`Speed::from_rate(f64) -> Option<Speed>` is the one mapping, in core.

**S2. The log records a speed change as its own event:
`EventKind::Speed { speed: Speed, source_time: f64 }`**, wire
`{"speed":{"speed":"half","sourceTime":12.34}}`, anchored exactly as `Play` and
`Pause` are (the playhead at the keypress, `event.rs:59`–`:65`'s rule). It is
added to `KnownKind` (`event.rs:98`–`:107`) and both codec matches. Not a `rate`
on `Play`, for four reasons, any one sufficient:

- **A speed changes while the footage is already playing**, where there is no
  `Play` to hang it on — logging a second `Play` would make "play started"
  mean something else.
- **Under §S12 a speed outlives a pause**, so it is state of its own, not a
  property of the play that happened to be running.
- **serde ignores unknown fields**: an older build would read
  `{"play":{"sourceTime":…,"rate":0.5}}` as a plain `Play`, replay it at 1×
  and **drop the rate on its next save**. A new kind decodes there as
  `Unknown` and survives the round trip (`event.rs:80`–`:86`).
- A defaulted `rate: f64` on `Play` is a field-level default on an `f64`,
  which the format rules forbid.

**S3. A slow span is `SegmentKind::Play(Speed)`.** `Play` gains its speed;
`Freeze` is unchanged; there is no third variant. `PlaybackSegment` keeps its
three fields, and `out_duration` stays in record time. Making `Play` a tuple
variant is the point: **every 1× assumption in §O3's table stops compiling**,
including `audio.rs:221`'s `==`, which as a third `Slow` variant would have kept
compiling and silently made every slow span silent. `whole_match.rs` and
`reel.rs` write `Play(Speed::Normal)`, which is true — their entries have no
log.

**S4. Replay walks `speed` and `playing` apart.** Both `source_time` and
`playback_segments`' `Walk` hold `speed: Speed` (from `Normal`) and
`playing: bool` (from `true`, today's "an empty event list is all `Play`"); the
rate is `speed.factor()` while playing and 0 otherwise. `Play` and `Pause` set
`playing` and anchor as today and **do not touch `speed`**; `Speed` sets
`speed` and anchors; `Skip` is unchanged. `emit` at a speed advances the source
by `dur × factor`, and the EOF split becomes "the record time the remaining
source lasts at this speed": `play_dur = dur.min(available / factor)`, the
source advancing by `play_dur × factor`, then the `Freeze` tail as today. A log
with no `Speed` event walks to exactly today's segments — pinned (§T1). The
two functions keep their 50 ms disagreement past EOF and agree everywhere else
(the module header's contract, `timeline.rs:1`–`:14`).

**S5. A speed change is a flushing seek through `load`, as `J`/`L` are today.**
`change_rate` (`transport.rs:132`–`:146`) already stores the rate and seeks at
it, frame-accurately, from where the footage is heading; slow motion reuses it.
**During a take it seeks to the anchor it logs** — `heading(source_secs)`,
computed before the change (S6) — so the live picture and the replay start the
slow span on the same frame by construction, rather than one at the keypress
and the other at the handler's later `query_position`. That is decision 4:
nothing jumps back further than the input's own queue delay. **M4's instant
rate change is not used**: it issues no flush and so no preroll, while the
player's single-flight slot settles every seek on its `ASYNC_DONE`
(`player/mod.rs`'s `Flight`), so it would need a flight of its own — a second
way of changing rate beside the one every scan already takes.

**S6. The bus contract: `Command::ScanSpeed` carries its moment.** It becomes
`ScanSpeed { step: ScanStep, host_ns: u64, source_secs: Option<f64> }`, captured
in `main.rs`'s `on_step_scan_speed` (`main.rs:797`–`:806`) exactly as
`on_toggle_play` captures them (`main.rs:780`–`:788`). While recording the
handler takes the anchor from `heading(source_secs)` **first**, then changes the
rate, then calls `RecordingLog::speed(host_ns, anchor, speed)` — a new append
beside `play`/`pause` (`core/src/recording.rs:77`–`:97`), with the same
monotonic clamp. `ScanStep` gains `ToggleSlow` (the new key, S10): Normal → Half,
anything slow → Normal. One command, one handler, one allow-list entry.

**S7. The export's game sound is slowed by the reader, not by Rust.**
`scaletempo` goes into `Reader`'s chain after the first `audioconvert`
(`composite/audio.rs:330`–`:350`), always — M1 shows it is bit-exact at 1, so
transcription and the avatar's levels, which share `Reader::start`, are
unchanged, and one chain is simpler than two. `Reader::seek` takes the rate and
issues `pipeline.seek(rate, FLUSH | ACCURATE, Set, position, None, NONE)` in
place of `seek_simple`. `Region` gains `rate: f64` — 1 for the commentary and
the music, the segment's factor for a game region — and the mapping in
`core/src/audio.rs`'s module header becomes `source_offset +
rate × (s − out_samples.start) / AUDIO_SAMPLE_RATE`; `region()` scales the
priming drop it adds to `source_offset` by the same rate (`audio.rs:240`–`:251`).
The mixer seeks each region's reader at the region's rate as it comes in
(`audio.rs:176`), which it already does per region, so a ½× segment between two
1× ones on the same file is three seeks, exactly as a skip is today. **No
time-stretch is written in Rust**: WSOLA by hand is a DSP module to own for
ever, and M1 says GStreamer's already does it. `envelope` and the ramps are
unchanged — a ½× region fades in and out over 5 ms like any other.
M2's lead (≤ 24.4 ms of source time) is **accepted, not compensated**: a
compensating seek would be a per-speed constant tuned to one element's
defaults, and the live player has the same lead, so the take and its export
agree.

**S8. The picture needs nothing in media.** `core::export::walk` maps a
`Play(speed)` frame to `source_start + factor × (t − out_start)`
(`core/src/export.rs:209`), so at ½× consecutive output frames step 1/60 s of
source and at ¼× 1/120 s. The pump's rule — last decoded frame with PTS ≤
`source_time` (`core/src/export.rs:4`–`:7`) — then shows each frame of a 30 fps
source twice or four times, a 25 fps one in a 2,3,2,3… or 4,5… cadence, and a
60 fps one once each at ½×, which is true slow motion. `Decoder::frame_at`
reuses the current frame for a repeated target and only ever pulls forward
(`composite/decode.rs:136`–`:161`). **Preview is the same schedule**, so it is
free; its game audio is still absent (#54/#125), which this does not change.
The match clock, the cues, the highlights and a slate title all read the
displayed frame's source time, so through a slow span the clock runs slow, a
ring stays up four times as long at ¼×, and nothing needs a change. Strokes,
their auto-clear, zoom and the commentary are in record time and are untouched.

**S9. The live player slows its sound with `scaletempo`.** `SourcePlayer::new`
sets `audio-filter` to a `scaletempo` (M3), and `set_rate` mutes only **above**
1 (`player/mod.rs:345`), where `TRICKMODE_NO_AUDIO` already applies
(`player/mod.rs:624`–`:626`). The speakers path is unchanged — `scaletempo`
is upstream of `autoaudiosink`, which sees a normal segment — and the
recorder is its own pipeline on the system clock, so **the commentary is
captured in real time whatever the footage's speed**: record time is wall time,
which is what makes `out_duration` correct with no change.

**S10. The keys: one new action, and `J`/`L` reach below 1×.**

- **`Action::SlowMotion`, default `s`** — one row in `keymap.rs`'s table, a
  `KeyAction` and a `handle-key` branch as every action has. `s` is free
  (`keymap.rs:283`–`:313`; the letters bound are `a c d f g h i j l o r v w x
  z`), it is the mnemonic, and it sits between `a` and `d`, the skips: *back
  three seconds, slow* is two neighbouring keys.
- **The speed ladder is `[0.25, 0.5, 1, 2, 4, 8, 16, 32]`.** `J` steps down to
  ¼× and stops; `L` steps up. **During a take the ladder stops at 1×** — fast
  scanning in a take stays out of scope (it trick-modes and mutes the sound,
  and nothing asked for it) — so `L` at 1× in a take does nothing, as `J` at
  1× does today. The speed button's `Cycle` keeps today's 1×→…→32×→1× while
  scanning and from a slow speed goes to 1×; during a take it cycles
  1× → ½× → ¼× → 1×. `next_speed`'s fallback for a rate not on the ladder
  (`transport.rs:28`'s `unwrap_or(0)`) must be 1×'s index, not 0 — index 0 is
  now ¼×.
- **A speed above 1× still needs the footage playing; at or below 1× it does
  not** (S12's pause rule). `scan_speed`'s refusal while a slate is armed
  narrows to speeds above 1×: its reason is the out-point overshoot at
  `rate × OUT_POLL` (`transport.rs:106`–`:115`), which at ¼× is a quarter of
  1×'s.
- **`Command::ScanSpeed { .. }` joins the recording allow-list**, and the
  whiteboard take's refusal list (`bus/mod.rs:1164`–`:1188`) gains it beside
  `TogglePlay`: a whiteboard take has no footage to slow, and
  `playback_segments`' early return (`timeline.rs:214`) would ignore the event
  anyway. Its notice gains "slowed".
- **The window's gate** is a new `can-change-speed` — `can-play && !previewing`
  and not a whiteboard take — on `S`, `J`, `L` and the speed button, replacing
  `can-scan-fast` (`app.slint:4885`) there; the bus makes every finer refusal.

**S11. What the window shows.** The readout appends the speed whenever it is
not 1× — ` · ½×`, ` · ¼×` — where today it shows only speeds above 1
(`main.rs:4669`–`:4672`), and the speed button shows the same words instead of
`root.scan-speed + "×"` (`app.slint:7402`), which would print `0.5×`. Both
read one Rust formatter, `speed_label(f64)`, set on a `speed-label` property
beside `scan-speed`. During a take the button is enabled and the readout is
where the coach sees the mode; there is no badge on the picture (Q2).

**S12. Pause keeps slow motion; a take starts at the speed it was started at.**
*(Recommended in Q1; written here as decided so the rest of the spec is one
design. If the coach chooses otherwise, Q1 says what changes.)* `set_playing`'s
reset becomes `rate > 1.0` (`transport.rs:81`), so a pause from fast still
returns to 1× and a pause from slow does not. `start_recording` pauses first
(`bus/recording.rs:181`–`:184`), so a take started from slow motion stays slow:
`RecordingLog::new` (`core/src/recording.rs:60`) gains the speed and writes
`Speed` at record time 0, after its `Pause`, **only when it is not `Normal`** —
a normal take's log is byte-for-byte today's. Stopping a take leaves the speed
alone. The three things that reset it are the ones that already do: a pause
from fast, opening a preview (`bus/preview.rs:129`–`:131`), and the coach's own
keys.

**S13. Everything else, checked.**

- **Skip:** `apply_skip` already advances a live target by
  `elapsed × player.rate()` (`transport.rs:299`), and replay applies a `Skip`
  then carries on at the walk's speed. A speed change drops a burst in flight
  (`store_rate`'s `reset_skip`, `transport.rs:151`–`:155`), as today.
- **Frame step:** refused while playing, previewing or unsettled (`transport.rs:243`–
  `:250`) and while recording by the allow-list, unchanged; while paused at ½× it is an accurate seek at the stored
  rate, which lands the same frame.
- **The slate out-point stop** logs its `Pause` at the stored `out_seconds`
  (`bus/slates.rs:172`–`:183`) and, under S12, leaves the speed standing; at
  slow speed the poll's overshoot shrinks. Its doc's *"the rate is 1.0 by the
  time this plays"* (`bus/slates.rs:116`–`:121`) is no longer true of a slate
  preview started from slow motion, and is corrected to say a slate preview
  plays at the standing speed, which is never above 1×.
- **The themed pass** parks with `JumpToSlate`, a pause, so under S12 a pass
  run in slow motion stays in slow motion from take to take — what "work
  through all the corner kicks" in slow motion wants.
- **Avatar takes** play footage like a camera take; nothing differs.
- **The reel, the whole match and the stream copy** have no clip and no log:
  `Play(Normal)`, untouched. A **basket** piece replays its own clip's log, so
  its slow spans come with it.
- **`clip_source_duration`'s fallback** (`core/src/plan.rs:215`–`:221`) covers
  any position a clip visits at rate 1, so it covers one at ≤ 1 too.

**S14. v19.** `CURRENT_FORMAT_VERSION` moves 18 → 19 (`core/src/store.rs:21`);
the readable floor stays 7, since a v7–v18 log has no `Speed` events and walks
as today. **No struct gains a field**, so `clip_stub` and `sample_clip` are
untouched. **The bump's reason is the forward direction:** a v18 build would
decode every `Speed` event as `Unknown`, keep it verbatim — so the round trip is
safe — and **replay the slow span at 1×**, running the footage ahead of the
commentary and the drawings until the next anchor snaps it back, in every
preview and export it renders. The bump makes it refuse the file instead, and
`project.json.v18` is what it can still be gone back to. CLAUDE.md gains a v19
bullet in the v18 bullet's shape.

---

## §C. Sentences elsewhere this makes false, to be corrected with it

- `CLAUDE.md`'s scan-speed paragraph (*"the bus steps 1×–32×, while scanning
  only; any pause returns to 1×, so a recording starts at 1×"*), its
  whiteboard bullet's *"the recording one minus six"*, and its frame-step and
  keymap paragraphs where they name the speeds.
- `bus/mod.rs:344`–`:349` (`ScanSpeed`'s doc), `:558` (`Event::ScanSpeed`'s
  *"1 after every pause"*), `:68`–`:77` (`ScanStep`'s *"down to 1x"*), and
  `:1164`'s *"minus six"*.
- `transport.rs:20`–`:21`, `:70`–`:73`, `:102`–`:115`.
- `player/mod.rs:337`–`:342` (*"mutes the sound above 1"* is already the
  intent; the code is what changes).
- `timeline.rs:35` (*"Source advances at 1x"*), `:67`–`:68` and `:121`–`:122`
  (the events that move source time and split the timeline, which gain
  `Speed`); `core/src/audio.rs`'s module-header mapping (`:23`–`:24`).
- `keymap.rs:260` (*"While the game video is playing"*); `app.slint:4884` and
  `:5761`–`:5762` (*"down to 1x. Not while recording or previewing, which are
  1x"*); `app.slint:7410`'s tooltip.
- `bus/slates.rs:116`–`:121` (S13).

---

## Out of scope

- **Reverse**, **ramped speed** (an ease into slow motion) and any speed but
  ½× and ¼×.
- **Fast scanning during a take** (S10).
- **Changing a clip's speed after recording.** The speed is part of the log,
  as a skip is; re-recording is the edit.
- **A slow-motion badge** on the picture — Q2.
- **Game audio in the preview** (#54, #125), which slow motion would not fix.
- **M4's instant rate change** (S5).

---

## Risks

1. **The rate-change seek during a take is a visible hitch.** It is the same
   flushing accurate seek a `J`/`L` press already costs while scanning, but a
   take is where a stutter is recorded into the coach's attention. Mitigation:
   none designed — measure it in the hands-on check, and M4 is the measured
   fallback if it is bad.
2. **`scaletempo` on the live path at 1×.** M1/M3 show it passthrough and
   bit-exact on the appsink path; the real speakers (`autoaudiosink` →
   `alsasink` → `pipewire-alsa`) and the ~+1 ms A/V offset are not measured
   with it in place. Hands-on: play at 1× and at ½× through the speakers, and
   re-read the A/V offset.
3. **M2's lead at ¼×** is 97 ms of output time. Accepted on the ground that
   slowed motion is judged in source time; if the coach hears a whistle before
   the ref blows it, the fix is S7's rejected compensation, and the test's
   bound is where it would be tuned.
4. **A skip burst in flight at the keypress.** The anchor is the burst's
   target (`heading`, `transport.rs:316`–`:326`) and the speed change drops the
   burst, which is the same exposure `toggle_play` has today during a burst;
   not new, not widened.
5. **A third `Speed` variant** would need the ladder, `from_rate` and the
   label to learn it; `Speed`'s matches are exhaustive, so the compiler finds
   them.

---

## Testing

**T1. Core** (`pundit-core/tests/`, `src/event.rs`'s own tests):
- the wire shape `{"speed":{"speed":"half","sourceTime":…}}`, pinned literally
  (a round trip cannot catch a wrong name — `event.rs:208`–`:210`); an unknown
  speed string decodes as `Unknown` and re-emits verbatim;
- `timeline.rs`: a `Speed(Half)` at 2 s anchored at 10 s gives
  `Play(Half)` from 10 s whose source advances half as fast; a `Pause` then
  `Play` keeps `Half` (S12); a slow span reaching EOF plays the remaining
  source at its speed and freezes the rest; **every existing timeline test
  passes with `Play` read as `Play(Normal)`**; and the segment walk agrees with
  `source_time` across a slow span, as the existing cross-check does for 1×;
- `export.rs`: consecutive `FrameSpec::source_time`s in a ½× span differ by
  1/60 s and in a ¼× span by 1/120 s;
- `audio.rs`: a ½× segment yields a game region with `rate: 0.5`, and the
  priming drop's addition to `source_offset` is scaled by it; the commentary
  stays at 1;
- `recording.rs`: `RecordingLog::new` at `Normal` writes exactly today's two
  events; at `Half` a third, `Speed` at 0;
- `project_format.rs`: `a_v18_file_loads_under_the_current_version`, beside its
  siblings.

**T2. Media** (`pundit-media/tests/export.rs`):
- **repeated frames**: a `counter_video` (`fixtures.rs:430`) clip at ½× decodes
  back (`decode_counters`, `fixtures.rs:630`) to each counter exactly twice in
  order, and at ¼× four times;
- **pitch kept**: a `tone_video` (`fixtures.rs:177`) of 440 Hz through a ½×
  clip reads at 440 by `tone_level` (`export.rs:1958`) and not at 220, and the
  slow span consumes half the source — a tone windowed to end at source 2 s
  ends at output 4 s;
- **onset bound**: `a_tone_lands_where_the_picture_does`'s shape
  (`export.rs:2040`) at ½× and ¼×, with M2's bound — the onset within one
  30 fps source frame of source time, early side only;
- `Reader` at rate 1 with `scaletempo` in the chain is unchanged: every
  existing audio, transcription and avatar test is the proof, and none should
  need touching;
- the player: `set_rate(0.5)` leaves `mute` off and the graph holds a
  `scaletempo`.

**T3. Harness, over the bus**: `S` during a take logs `Speed(Half)` at the
caller's `host_ns` anchored at the caller's position; `L` at 1× in a take logs
nothing; a pause and play during slow motion keep it, in the player and in the
log; a pause from 4× still returns to 1×; a take started from ½× opens with
`Speed` at 0; `S` in a whiteboard take is refused with the notice and logs
nothing; `J` while scanning reaches ¼× and stops; a slow take's export runs the
length the log says.

**T4. `tests/ui`**: `keys.rs` presses the shipped `s` and gets the new
`KeyAction`; the speed button and readout show `½×` / `¼×` from `speed-label`
and nothing at 1×.

**T5. Hands-on** (batched with the release's checks): Risks 1 and 2 on the
reference laptop, on real footage, through the speakers.

---

## Open questions for the coach

**Q1. When you pause during slow motion and play again, should it carry on
slow?**
1. **Yes — slow motion stays on until you turn it off** (recommended). Slow,
   stop on the frame, draw, carry on slow is the instant-replay rhythm, and it
   costs nothing extra in the log. A take started in slow motion starts slow
   for the same reason. Fast scanning still drops back to 1× on a pause, as now.
2. **No — any pause returns to 1×**, slow included, as it does for fast speeds
   today. One rule, but you press `S` again after every pause, and a take
   always starts at 1×. (S12 becomes "every pause resets", `S` and `J` while
   paused are refused, and `RecordingLog::new` is unchanged.)

**Q2. Should an exported clip show that it is in slow motion?**
1. **No badge** (recommended). Slowed footage reads as slowed, you are talking
   over it, and a badge is a new overlay element with its own corner to find
   room for beside the scoreboard and the inset.
2. **A small "½×" / "¼×" badge** in encoded exports and previews while a slow
   span is on screen (never in a stream copy, which has no clips).
3. **Only on the scan view and during the take**, never in the file — the
   readout already does this, so this is option 1.

**Q3. What should `S` slow to?**
1. **Always ½×** (recommended): one key, one result; `J` takes it on to ¼×.
2. **Whichever slow speed you used last** — one press reaches ¼× if that is
   what you usually want, at the cost of `S` doing different things on
   different days.

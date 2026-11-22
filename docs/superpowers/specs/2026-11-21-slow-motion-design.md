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
re-derived from the tree at `d742430` on 2026-11-22, after the spec's two
reviews.

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

Answering the first draft's three questions, the same day, each the option
recommended:

6. **Slow motion stays on through a pause**, until the coach turns it off; a
   take started in slow motion starts slow. Fast scanning still drops back to
   1× on a pause, as now (§S11).
7. **No slow-motion badge** in exports or previews: slowed footage reads as
   slowed, and the coach is talking over it.
8. **`S` always slows to ½×**; `J` takes it on to ¼× (§S6).

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
were written for **fast** scanning and both change (§S11, §S9).

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

Nothing else names `SegmentKind` outside tests (grepped).

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
(thresholds of 0.05 and 0.4 on a 0.5 sine agree to within 23 ms of output):

| rate | onset, output s | ideal | lead, output | lead, source time |
|---|---|---|---|---|
| 1 (with `scaletempo`) | 1.0000 | 1.0000 | 0 | 0 |
| ½ | 1.9750 | 2.0000 | 25 ms | 12.5 ms |
| ¼ | 3.9026 | 4.0000 | 97 ms | 24.4 ms |

Its window is consumed ahead of its output. In **source** time — which is what
the eye judges against slowed motion — the lead is under one 30 fps source frame
at both rates. §S7 accepts it and pins the bound. Whether the live player shows
the same lead through the real speakers is **not** measured (Risk 2).

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
sound under a ½× segment. `scaletempo` is in `gstreamer1.0-plugins-good`
(`audiofx`), already a run-time `Depends` (`pundit-app/Cargo.toml:89`), so the
`.deb` needs no new package — but the smoke test's element list must name it
(§S9).

**M4. An instant rate change starves the sink with `scaletempo` in place.**
`SeekFlags::INSTANT_RATE_CHANGE` (no position, no flush) is accepted by
`playbin3` 1.24.2, and with fakesinks the position does slow to ~½×. But with
`audio-filter=scaletempo` and the appsink above, 1 s at 1× delivers 49,280
samples and the **2 s after** an instant change to ½× only 48,128, where real
time wants 96,000; the same change as a flushing seek delivers 97,920. So it is
**not an option** for this feature, and not a fallback for anything (§S5).

---

## §S. Decisions

**S1. A rate is an `f64`, everywhere.** The player already carries one
(2×–32× as well), the bus already emits one (`Event::ScanSpeed(f64)`,
`bus/mod.rs:559`), and the log stores what the player was set to. The ladder
(§S6) is what limits a take to ½× and ¼×; the format does not, so a future
speed is no format change. A `Speed` this build cannot decode (a malformed
payload) falls to `EventKind::Unknown` and is **kept verbatim**; the version
bump (§S13) is what stops a wrong replay, not the type. **One guard**, in the
walk (§S4): a `Speed` whose rate is not finite and positive — only a
hand-edited file can hold one — is ignored as `Unknown` is.

**S2. The log records a speed change as its own event:
`EventKind::Speed { rate: f64, source_time: f64 }`**, wire
`{"speed":{"rate":0.5,"sourceTime":12.34}}`, anchored exactly as `Play` and
`Pause` are (the playhead at the keypress, `event.rs:59`–`:65`'s rule). It is
added to `KnownKind` (`event.rs:98`–`:107`) and both codec matches. Not a `rate`
on `Play`, for four reasons, any one sufficient:

- **A speed changes while the footage is already playing**, where there is no
  `Play` to hang it on — logging a second `Play` would make "play started"
  mean something else.
- **A speed outlives a pause** (decision 6), so it is state of its own, not a
  property of the play that happened to be running.
- **serde ignores unknown fields**: an older build would read
  `{"play":{"sourceTime":…,"rate":0.5}}` as a plain `Play`, replay it at 1×
  and **drop the rate on its next save**. A new kind decodes there as
  `Unknown` and survives the round trip (`event.rs:80`–`:86`).
- A defaulted `rate: f64` on `Play` is a field-level default on an `f64`,
  which the format rules forbid.

**S3. A slow span is `SegmentKind::Play(f64)`**, the rate. `Freeze` is
unchanged and there is no third variant; `PlaybackSegment` keeps its three
fields, and `out_duration` stays in record time. Making `Play` a tuple variant
is the point: **every 1× assumption in §O3's table stops compiling**, including
`audio.rs:221`'s `==`, which beside a third `Slow` variant would have kept
compiling and made every slow span silent. `SegmentKind` loses its `Eq` derive
(`timeline.rs:33`), keeping `PartialEq`; nothing hashes or `Eq`-bounds it.
`whole_match.rs` and `reel.rs` write `Play(1.0)`, which is true — their entries
have no log.

**S4. Replay steps one `Motion { rate, playing }`, shared by both walks.**
`source_time` and `playback_segments`' `Walk` each hold one, starting at
`{ rate: 1.0, playing: true }` (today's "an empty event list is all `Play`").
`Motion` applies `Play`, `Pause` and `Speed` — `Play`/`Pause` set `playing`,
`Speed` sets `rate` (subject to S1's guard), **neither touches the other** —
and answers the anchor each carries and the source's speed, `rate` while
playing and 0 otherwise. `Skip` stays in each walk, as today. `emit` at a speed
advances the source by `dur × speed`, and the EOF split becomes the record time
the remaining source lasts at that speed: `play_dur = dur.min(available /
speed)`, the source advancing by `play_dur × speed`, then the `Freeze` tail as
today. A log with no `Speed` event walks to exactly today's segments — pinned
(§T1). The two functions keep their 50 ms disagreement past EOF and agree
everywhere else (`timeline.rs:1`–`:14`). If `Motion` turns out not to be
clean in the code, two copies of three lines is the fallback; the plan decides.

**S5. A speed change is a flushing seek through `load`, as `J`/`L` are today.**
`change_rate` (`transport.rs:132`–`:146`) already stores the rate and seeks at
it, frame-accurately; slow motion reuses it. **During a take it seeks to the
anchor it logs** — `heading(source_secs)`, computed before the change (S6) — so
the live seek and the replay's anchor name the same frame, rather than one at
the keypress and the other at the handler's later `query_position`. That is
decision 4: the footage is not sent back, save for the input's own queue delay
and for a skip burst in flight, where `heading` answers the burst's unadvanced
target (`transport.rs:316`–`:326`) — the same exposure `toggle_play` has today.
**The instant rate change is not used**: M4 measured it starving the sink.

**The seek's own gap is a lag the replay does not have.** A flushing seek
holds the live picture for its preroll, `L`, then plays from the anchor; the
replay plays from the anchor at once. So from an `S` press to the next anchor
(a pause, a play, the next speed change) the live picture trails its replay by
`L × rate` of source — and by `L` after a change back to 1×, which is a seek
too. A stroke drawn on the live picture then lands that much **ahead** of the
ball on replay. It is the same class as a skip during a take today, which also
seeks and is also anchored at the press; it is stated here, not fixed, and T5
measures it.

**S6. The bus contract and the ladder.** `Command::ScanSpeed` becomes
`ScanSpeed { step: ScanStep, host_ns: u64, source_secs: Option<f64> }`,
captured in `main.rs`'s `on_step_scan_speed` (`main.rs:797`–`:806`) exactly as
`on_toggle_play` captures them (`main.rs:780`–`:788`). While recording the
handler takes the anchor from `heading(source_secs)` **first**, then changes the
rate, then calls `RecordingLog::speed(host_ns, anchor, player.rate())` — a new
append beside `play`/`pause` (`core/src/recording.rs:77`–`:97`), with the same
monotonic clamp.

- **One ladder, `[0.25, 0.5, 1, 2, 4, 8, 16, 32]`**, and a pure
  `next_speed(rate, step, max)`. `Faster` and `Slower` step along it and stop
  at its ends, with `max` capping the top: **1× during a take or while
  paused**, 32× otherwise. Fast scanning in a take stays out of scope — it
  trick-modes and mutes the sound, and nothing asked for it — so `L` at 1× in a
  take does nothing, as `J` at 1× does today; and a speed above 1× still needs
  the footage playing, as today, because a pause would undo it.
- **`Cycle` (the speed button) is today's rule** — the next speed, wrapping
  past `max` to 1× — **except where that would be no change**: at 1× under a
  1× cap it wraps to ¼× instead, so the button is never a dead click. While
  scanning that is today's 1→…→32→1; in a take or paused it is 1→¼→½→1.
- **`ScanStep::ToggleSlow` is the new key**: a rate below 1 goes to 1×, any
  other — 1× or a fast speed — to ½× (decision 8).
- `next_speed`'s fallback for a rate not on the ladder (`transport.rs:28`'s
  `unwrap_or(0)`) becomes 1×'s index: index 0 is now ¼×.

One command, one handler, one allow-list entry; the ladder's cases go into the
existing pure test, `scan_speeds_clamp_and_the_button_wraps`
(`transport.rs:479`).

**S7. The export's game sound is slowed by the reader, not by Rust.**
`scaletempo` goes into `Reader`'s chain after the first `audioconvert`
(`composite/audio.rs:330`–`:350`), always — M1 shows it is bit-exact at 1, so
transcription and the avatar's levels, which share `Reader::start`, are
unchanged, and one chain is simpler than two. `Reader::seek` takes the rate and
issues `pipeline.seek(rate, FLUSH | ACCURATE, Set, position, None, NONE)` in
place of `seek_simple`. `Region` gains `rate: f64` — 1 for the commentary and
the music, the segment's rate for a game region — and the mapping in
`core/src/audio.rs`'s module header (`:23`–`:24`) becomes `source_offset +
rate × (s − out_samples.start) / AUDIO_SAMPLE_RATE`; `region()` scales the
priming drop it adds to `source_offset` by the same rate (`audio.rs:240`–`:251`).
The mixer seeks each region's reader at the region's rate as it comes in
(`composite/audio.rs:176`), which it already does per region, so a ½× segment
between two 1× ones on the same file is three seeks, exactly as a skip is
today. **No time-stretch is written in Rust**: WSOLA by hand is a DSP module to
own for ever, and M1 says GStreamer's already does it. `envelope` and the ramps
are unchanged. M2's lead (≤ 24.4 ms of source time) is **accepted, not
compensated**: a compensating seek would be a per-rate constant tuned to one
element's defaults.

**S8. The picture needs nothing in media.** `core::export::walk` maps a
`Play(rate)` frame to `source_start + rate × (t − out_start)`
(`core/src/export.rs:209`), so at ½× consecutive output frames step 1/60 s of
source and at ¼× 1/120 s. The pump's rule — last decoded frame with PTS ≤
`source_time` (`core/src/export.rs:4`–`:7`) — then shows each frame of a 30 fps
source exactly twice or four times; a 25 fps source's frames last 2.4 output
frames at ½× (a 2,2,3,2,3… cadence) and 4.8 at ¼×; a 60 fps source's frames
show once each at ½×, which is true slow motion. `Decoder::frame_at` reuses the
current frame for a repeated target and only ever pulls forward
(`composite/decode.rs:136`–`:161`). **Preview is the same schedule**, so it is
free; its game audio is still absent (#54/#125), which this does not change.

The match clock, the cues, the highlights and a slate title all read the
displayed frame's source time, so through a slow span the clock runs slow and a
ring stays up four times as long at ¼× — correct, and no change. **One visible
consequence, accepted:** a title's out-fade is `STRIP_FADE`, 0.5 s of
**source** time (`core/src/caption.rs:78`, read by `leaving` at `:89`–`:91`),
so over a slowed range it lasts 1 s at ½× and 2 s at ¼×. That is the title
fading with the footage it belongs to. Strokes, their auto-clear, zoom, the
caption's record-time window and the commentary are in record time and are
untouched.

**S9. The live player slows its sound with `scaletempo`.** `SourcePlayer::new`
(`player/mod.rs:165`) sets `audio-filter` to a `scaletempo` (M3), and
`set_rate` mutes only **above** 1 (`player/mod.rs:345`), where
`TRICKMODE_NO_AUDIO` already applies (`player/mod.rs:624`–`:626`) — the method's
doc already says "mutes the sound above 1" (`:337`–`:338`). `scaletempo` joins
`packaging/smoke-test.sh`'s `elements=(…)` (`:64`–`:74`), since the code now
names it. The recorder is its own pipeline on the system clock, so **the
commentary is captured in real time whatever the footage's speed**: record time
is wall time, which is what makes `out_duration` correct with no change.

**S10. The keys and the window.**

- **`Action::SlowMotion`, default `s`** — one row in `keymap.rs`'s table, a
  `KeyAction` and a `handle-key` branch, as every action has, sending
  `step-scan-speed(ScanStep.toggle-slow)`. `s` is free (`keymap.rs:283`–`:313`;
  the letters bound are `a c d f g h i j l o r v w x z`), it is the mnemonic,
  and it sits between `a` and `d`, the skips: *back three seconds, slow* is two
  neighbouring keys.
- **`Command::ScanSpeed { .. }` joins the recording allow-list**, and the
  whiteboard take's refusal list (`bus/mod.rs:1164`–`:1188`) gains it beside
  `TogglePlay`: a whiteboard take has no footage to slow, and
  `playback_segments`' early return (`timeline.rs:214`) would ignore the event
  anyway. The notice is the refusal, as it is for `TogglePlay` and `Skip`; its
  words gain "slowed".
- **`scan_speed`'s refusal while a slate is armed narrows to speeds above 1×.**
  The reason is not the overshoot: **a take shot from a slate arms it**
  (`bus/recording.rs:241`, `self.armed_slate = slate`), so the refusal as it
  stands would refuse every speed key in exactly the takes the themed pass
  makes. Its own reason — the out-point poll overshooting by `rate × OUT_POLL`
  and the pause's seek trailing at speed (`transport.rs:106`–`:115`) — is about
  fast speeds only, and at ¼× the overshoot is a quarter of 1×'s.
- **The window's gate is `can-scan-fast`, redefined in place**
  (`app.slint:4885`) as `can-play && !previewing` — dropping `playing` and
  `!recording` — at its existing three users (`J` `:5764`, `L` `:5770`, the
  button `:7403`) and the new `S` branch. No whiteboard clause: the bus's
  notice is the refusal, as for `TogglePlay`. The bus makes every finer
  refusal.
- **What it shows:** the readout appends the speed whenever it is not 1×
  (`main.rs:4669`–`:4672`'s `s > 1.0` becomes `s != 1.0`), so ` · 0.5×` and
  ` · 0.25×`; the button keeps `root.scan-speed + "×"` (`app.slint:7402`),
  which prints `0.5×` and `0.25×`. No new property, no badge (decision 7).

**S11. Pause keeps slow motion; a take started slow starts slow** (decision
6). `set_playing`'s reset becomes `rate > 1.0` (`transport.rs:81`), so a pause
from fast still returns to 1× and a pause from slow does not.
`start_recording` pauses first (`bus/recording.rs:181`–`:184`), so the player
is still slow when the take begins; **right after the `Active` holding
`RecordingLog::new` is built** (`bus/recording.rs:224`–`:233`, the log at
`:228`) it calls `log.speed(t0_ns, start_source_seconds, rate)` when the rate
is not 1 — no signature change, and a normal take's log is
byte-for-byte today's. **Not for a whiteboard take**, which has no footage to
be slow and whose log must say nothing about speed. Stopping a take leaves the
speed alone. What resets it: a pause from fast, opening a preview
(`bus/preview.rs:129`–`:131`), and the coach's keys.

**S12. Everything else, checked.**

- **Skip:** `apply_skip` already advances a live target by
  `elapsed × player.rate()` (`transport.rs:299`), and replay applies a `Skip`
  then carries on at the walk's speed. A speed change drops a burst in flight
  (`store_rate`'s `reset_skip`, `transport.rs:151`–`:155`), as today.
- **Frame step:** refused while playing, previewing or unsettled
  (`transport.rs:243`–`:250`) and while recording by the allow-list,
  unchanged; while paused at ½× it is an accurate seek at the stored rate,
  which lands the same frame.
- **The slate out-point stop** logs its `Pause` at the stored `out_seconds`
  (`bus/slates.rs:172`–`:183`) and leaves the speed standing; at slow speed the
  poll's overshoot shrinks. Its doc's *"the rate is 1.0 by the time this
  plays"* (`bus/slates.rs:116`–`:121`) is no longer true of a slate preview
  started in slow motion, and is corrected to say a slate preview plays at the
  standing speed, which is never above 1×.
- **The themed pass** parks with `JumpToSlate`, a pause, so a pass run in slow
  motion stays in slow motion from take to take.
- **Avatar takes** play footage like a camera take; nothing differs.
- **The reel, the whole match and the stream copy** have no clip and no log:
  `Play(1.0)`, untouched. A **basket** piece replays its own clip's log, so its
  slow spans come with it.
- **`clip_source_duration`'s fallback** (`core/src/plan.rs:215`–`:221`) covers
  any position a clip visits at rate 1, so it covers one at ≤ 1 too.

**S13. v19.** `CURRENT_FORMAT_VERSION` moves 18 → 19 (`core/src/store.rs:21`);
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

- `CLAUDE.md:771` (the scan-speed paragraph: *"the bus steps 1×–32×, while
  scanning only; any pause returns to 1×, so a recording starts at 1×"*),
  `:1194` (*"the recording one minus six"*), and a v19 bullet after `:498`'s.
- `bus/mod.rs:344`–`:349` (`ScanSpeed`'s doc), `:558` (`Event::ScanSpeed`'s
  *"1 after every pause"*), `:68`–`:77` (`ScanStep`'s *"down to 1x"*), and
  `:1164`'s *"minus six"*.
- `transport.rs:20`–`:21`, `:70`–`:73`, `:102`–`:115`.
- `timeline.rs:35` (*"Source advances at 1x"*), `:67`–`:68` and `:121`–`:122`
  (the events that move source time and split the timeline, which gain
  `Speed`); `core/src/audio.rs:23`–`:24` (the mapping).
- `keymap.rs:260` (*"While the game video is playing"*); `app.slint:4884` and
  `:5761`–`:5762` (*"down to 1x. Not while recording or previewing, which are
  1x"*); `app.slint:7410`'s tooltip.
- `bus/slates.rs:116`–`:121` (S12).
- `packaging/smoke-test.sh:64`–`:74` gains `scaletempo` (S9).

---

## Out of scope

- **Reverse**, **ramped speed** (an ease into slow motion) and any speed but
  ½× and ¼× in a take.
- **Fast scanning during a take** (S6).
- **Changing a clip's speed after recording.** The speed is part of the log,
  as a skip is; re-recording is the edit.
- **A slow-motion badge** on the picture (decision 7).
- **Game audio in the preview** (#54, #125), which slow motion would not fix.
- **An instant rate change** (M4).

---

## Risks

1. **The rate-change seek during a take is a visible hitch, and a lag the
   replay lacks** (S5). It is the flushing accurate seek a `J`/`L` press
   already costs while scanning, but a take is where a stutter is recorded
   into the coach's attention, and strokes drawn in the gap land ahead of the
   ball on replay. **No fallback is designed** — M4 rules out the one there
   was. T5 measures both.
2. **`scaletempo` on the live path, at every rate.** M1/M3 show it passthrough
   and bit-exact at 1× on the appsink path, but the real speakers
   (`autoaudiosink` → `alsasink` → `pipewire-alsa`), the ~+1 ms A/V offset at
   1× and the lead at ½× and ¼× are not measured with it in place — S7's "the
   take and its export agree" rests on the live path showing M2's lead too. T5
   reads the A/V offset at 1×, ½× and ¼× through the speakers.
3. **`audio-filter` changes how `playsink` builds its audio chain, and that is
   where #72's load stall lives** (`CLAUDE.md`'s "A load that stalls…"
   paragraph: `playsink` sitting at `Async/Ready/Paused` holding only its
   `audiotee` and `streamsynchronizer`). The filter may widen, narrow or not
   touch it. T5 reruns `player::tests::every_open_settles`
   (`player/tests.rs:757`) under `taskset -c 0,1`, several at once, with the
   filter, against its documented baseline.
4. **M2's lead at ¼×** is 97 ms of output time. Accepted on the ground that
   slowed motion is judged in source time; if the coach hears a whistle before
   the ref blows it, the fix is S7's rejected compensation, and the test's
   bound is where it would be tuned.

---

## Testing

**T1. Core** (`pundit-core/tests/`, `src/event.rs`'s own tests):
- the wire shape `{"speed":{"rate":0.5,"sourceTime":…}}`, pinned literally
  (a round trip cannot catch a wrong name — `event.rs:208`–`:210`);
- `timeline.rs`: a `Speed { 0.5 }` at 2 s anchored at 10 s gives `Play(0.5)`
  from 10 s whose source advances half as fast; a `Pause` then `Play` keeps
  0.5; a slow span reaching EOF plays the remaining source at its rate and
  freezes the rest; a `Speed` with rate 0 or NaN changes nothing; **every
  existing timeline test passes with `Play` read as `Play(1.0)`**; and the
  segment walk agrees with `source_time` across a slow span, as the existing
  cross-check does for 1×;
- `export.rs`: across a ½× span the last `FrameSpec::source_time` is the
  anchor plus `0.5 ×` the span's record length (and `0.25 ×` at ¼×) — the
  source time the picture ends on, which is what a wrong rate changes;
- `audio.rs`: a ½× segment yields a game region with `rate: 0.5`, and the
  priming drop's addition to `source_offset` is scaled by it; the commentary
  stays at 1;
- `project_format.rs`: `a_v18_file_loads_under_the_current_version`, beside its
  siblings.

**T2. Media** (`pundit-media/tests/export.rs`):
- **repeated frames**: a `counter_video` (`fixtures.rs:430`) of kind
  `CounterKind::H264Mp4BFrames` (`fixtures.rs:372`) — **not** `Vp8WebmWithAudio`,
  whose millisecond timecodes put 30 fps frames off the 1/60 s grid and make a
  repeat count 1 or 3 — exported at ½× decodes back (`decode_counters`,
  `fixtures.rs:630`) to each counter exactly twice in order, and at ¼× four
  times; and its last counter is the anchor plus `rate ×` the span, in frames;
- **the sound is stretched, at its pitch**: one test at ½× and ¼× over a
  `tone_video` (`fixtures.rs:177`) whose 440 Hz tone is windowed to end at
  source 2 s. Its **real assertion is the duration**: the tone ends at output
  4 s (½×) and 8 s (¼×), within M2's bound (one 30 fps source frame of source
  time, early side only). `tone_level` (`export.rs:1958`) at 440 Hz across the
  slowed span is the pitch half. Sabotage: remove `scaletempo` from the chain,
  or drop the rate from `Reader::seek` — either ends the tone at output 2 s;
- `Reader` at rate 1 with `scaletempo` in the chain is unchanged: every
  existing audio, transcription and avatar test is the proof, and none should
  need touching;
- the player: `set_rate(0.5)` leaves `mute` off.

**T3. Harness, over the bus**: `S` during a take logs `Speed { 0.5 }` at the
caller's `host_ns` anchored at the caller's position; `L` at 1× in a take logs
nothing (the take's cap); a pause and a play during slow motion keep it, in the
player and in the log; a pause from 4× still returns to 1×; a take started from
½× opens with `Speed` at 0, and a whiteboard take started from ½× does not;
`S` in a whiteboard take is refused with the notice and logs nothing; `S` in a
slate take is accepted while `L` above 1× is still refused while armed. The
existing `fast_scanning_runs_at_the_chosen_speed`
(`harness/tests/transport.rs:381`) and
`the_scan_keys_are_refused_while_a_range_is_armed` (`harness/tests/slates.rs`)
keep every assertion: both refusals they pin are speeds above 1×.

**T4. `tests/ui`**: `keys.rs` presses the shipped `s` and gets the new
`KeyAction`; the speed button is enabled while paused and while recording.

**T5. Hands-on** (batched with the release's checks), on the reference laptop,
real footage, the real speakers:
- the hitch at an `S` press during a take, and **stroke placement against the
  replay** after one: draw on a moving player just after pressing `S`, export,
  and see how far ahead of the ball the stroke lands (Risk 1);
- the A/V offset at 1×, ½× and ¼× (Risk 2);
- `every_open_settles` pinned to two CPUs with the filter (Risk 3).

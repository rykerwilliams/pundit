# Keeping libvorbis out of the test suite (#70) — design

BACKLOG **#70**: a glibc heap-corruption abort, four sightings across three test
binaries, always under load, never alone. Root cause, found 2026-09-27:
**libvorbis frees a pointer that is not a live allocation while clearing its
codebooks**, during Vorbis *decoder teardown* inside `decodebin3`. Every
`corrupted size vs. prev_size` abort is that damage discovered later by an
unrelated `malloc`.

We are not fixing libvorbis. We are stopping the suite from running it.

**This spec replaces a first draft that was measurably wrong, and the way it was
wrong is worth keeping.** That draft proposed selecting streams so no unwanted
decoder was built, and defended the scope by pointing at the test the captured
core came from. Both parts were refuted by measurement — see U3 — and the second
was the exact mistake #70's own entry warns about two paragraphs before the one
that matters: *"the backtrace does not name the culprit, and reading it as if it
did is the trap here."* I did it anyway.

## U0. The measurements, and which of them killed the first draft

Taken 2026-09-27 on the reference laptop, GStreamer 1.24.2.

1. **`decodebin3` builds a decoder for every stream, whatever you link or
   select.** With `GST_DEBUG=GST_ELEMENT_FACTORY:4` on a video-only decode of a
   VP8+Vorbis file: nothing → 1 `vorbisdec`; `caps=video/x-raw(ANY)` → still 1;
   **a `StreamCollection` off the bus plus a `SelectStreams` event → still 1**,
   created ~5 ms *after* the event, and `GST_DEBUG=vorbisdec:5` shows it then
   parsing all three headers — i.e. reaching exactly the state whose teardown
   calls `vorbis_book_clear`.
   **What decides it is *when* the event is sent**, and two reviews measured the
   two answers before the difference was spotted: sent after a polled wait, the
   `audio.rs` shape, the decoder is built (1 of 1, and 4 of 6 in a replica);
   **sent from a bus *sync handler*, on the thread that posts the collection, it
   is not** (0 of 12 idle, 0 of 8 at 8-way contention, and still 0 across three
   flushing seeks). `decodebin3`'s **`select-stream` signal** also gives 0 and
   needs no handler at all.
   **So `composite/audio.rs`'s selection does not prevent the element** — it
   arrives too late. That is a shipped bug, not just a wrinkle here; see U4a.
2. **The exposure is 24 decoders, not one.** `vorbisdec` constructions in one run
   of the media `export` suite: **24**. In
   `a_vp8_counter_fixture_decodes_to_its_frame_numbers`, the test the captured
   core came from and the only one the first draft touched: **1**. The other 23
   are the export path itself — `composite/decode.rs`'s pump, and
   `composite/audio.rs`'s `Reader`, which decodes the source's Vorbis **on
   purpose** and which no amount of stream selection can ever remove.
3. **The fixtures are the only source of Vorbis in the repo.** All four
   `vorbisenc` sites are in `fixtures.rs`; no test file mentions Vorbis at all.
4. **The bug needs several decoders coming down in one process under load.** Four
   concurrent copies of the `export` binary reproduce it (1 in 16 plain, **4 in
   24** under glibc's real heap checking — and `MALLOC_CHECK_=3` alone is inert on
   glibc 2.34+, which is how the entry sat on this for three sightings). One
   stock `gst-launch-1.0 uridecodebin3` over the same file, 6-way: **0 in 60**.

## U1. The fix: the fixtures stop being Vorbis

`vorbisenc` → **`opusenc`** at the four sites, and `AUDIO_RATE` 44 100 → 48 000.
That removes every Vorbis decoder the suite can build — all 24, including the
23 the first draft could never have reached and the `transcribe.rs` sighting
that started #70.

**And it makes the fixtures faithful, which is the better reason.** The recorder
writes **Opus** (`capture/recorder.rs:292`), so a commentary-recording fixture
encoded with Vorbis never resembled what the app produces. A WebM of VP8 and
Opus is what the app actually writes. The suite has been testing a format
nothing in production creates, and that is why nothing breaks when it stops.

Measured costs, each checked rather than assumed:

- **48 000 keeps the arithmetic.** `AUDIO_RATE.is_multiple_of(fps)` holds for
  every fps in use (25, 30, 60), and a rate that failed would fail **loudly** at
  that assert rather than silently.
- **`opusenc` is already a dependency** — `gstreamer1.0-plugins-base`, which the
  fixtures already require.
- **Container and file names are unchanged.** WebM carries Opus; the Ogg fixture
  carries Ogg Opus.
- **Duration is identical** (`ffprobe`: 2.000000 s). The only difference is a
  −7 ms audio `start_time` from Opus's pre-skip, which nothing in the suite
  measures.
- **The "tone at 1.000 s within a millisecond" check is not affected**, because
  `tone_video` is **raw F32LE in `matroskamux`** and keyed to `TONE_RATE`, not a
  codec and not `AUDIO_RATE`. The first draft claimed otherwise; that was false,
  and it was the main reason the draft talked itself out of this fix.

## U2. What is not the fix

- **Selecting streams.** U0.1: the event does not prevent construction. The
  signal does, but it cannot touch the 23 decoders the export builds on purpose
  (U0.2), so it is not a fix for #70 at all. It is worth doing for its own
  reason — see U4.
- **Fixtures with no audio at all.** The audio is load-bearing: the export mixer
  and transcription read it, and the track keeps the container duration honest.
  A codec change costs four words; surgery on which fixtures have sound does not.
- **Raw PCM in `matroskamux`** would build *no* audio decoder, which is even
  closer to the stated goal, but `webmmux` will not carry PCM — the `.webm`
  fixtures would become Matroska and their names would lie. Opus keeps the
  container honest and matches the recorder. If a future decoder bug makes any
  audio decode unwelcome, this is the fallback and it is already the pattern
  `tone_video` uses.
- **Blacklisting `vorbisdec`,** which would change what the app can open to work
  around one library's teardown. **Fixing libvorbis** upstream: right, and out
  of scope; #70 holds the backtrace, the heap forensics and the rates for the
  report.

## U3. The app's exposure is not closed by this, and was never closed by the plan

Say it plainly, because the first draft implied otherwise. The player is
`playbin3` with audio enabled, and the export's `Reader` decodes source audio
deliberately. **A Vorbis-audio source therefore builds and tears down a
`vorbisdec` on every load, close and export, whatever this spec does.** Nothing
short of the upstream fix changes that.

It is also narrow: it needs a coach to import a file whose audio is Vorbis, when
footage is MP4/MKV with AAC and the app's own recordings are Opus. Under "don't
fix super-rare edge cases unless the fix has zero tradeoffs", that is not a
justification for new machinery in five places — which is why U4a and U4b are
separate and are justified by something else entirely.

## U4a. A shipped bug this found on the way: `audio.rs` selects too late

`composite/audio.rs` goes to PAUSED, polls a `Mutex` slot every 10 ms for the
`StreamCollection`, then sends `SelectStreams`. By then `decodebin3` has built
the video decoder it did not want. **Measured in the real export binary:
`a_tone_lands_where_the_picture_does` creates three `vavp8dec` for a job with one
source and one recording — one legitimate, two from the two Readers losing that
race.** On coach footage that is a `vah264dec` constructed and torn down per
audio file, per export, and the same through `transcribe.rs` and
`analyze/audio.rs`, which both read through this `Reader`.

The comment at `audio.rs:384` is *literally true and still misleading*: its
2.2 s-against-46 ms measurement is real, because an unselected stream stops being
**decoded** — the element is simply still built, and being built is what #70
costs.

**The fix is to move that one `send_event` into the sync handler** that already
captures the collection — about five lines, no new machinery, and it corrects
three callers at once. **Its handler must return `BusSyncReply::Pass`**, not
`Drop`: `Drop` is why that module keeps an error *slot*, and copying it into a
caller that reads errors off the bus would silently swallow them.

This is a separate commit from U1. It is an efficiency fix, and it does **not**
fix #70 — the `Reader` asks for the audio stream, so on a Vorbis source it builds
a `vorbisdec` deliberately, whatever it selects.

## U4b. Separately again: the wasted decode, which is a different argument

Two sites decode audio nobody reads, on real coach footage, every time:
`analyze/motion.rs` (a whole half per run) and `composite/decode.rs` (the export
pump — pointlessly decoding the source's audio *while* `audio.rs` decodes the
same file's audio properly in a second pipeline). The `select-stream` signal
fixes that in ~12 lines apiece: synchronous, once per stream, before anything is
built, stateless so it survives seeking, and needing none of `audio.rs`'s mutex,
deadline or poll loop — that weight is about cancellation, not selection.

**This is not part of the #70 fix and must not be sold as one.** It wants its own
measurement — decode time for a whole half, and for one export, before and after
— and its own commit. Two sites the first draft listed do **not** belong:
`composite/avatar.rs` decodes PNG/JPEG stills, which have no audio stream to
drop, and `composite/preview.rs` deliberately links both streams.

To `BACKLOG.md`, not into this pass.

## U5. Acceptance

1. **The decoder count, which is the direct evidence and takes 33 seconds:**
   `GST_DEBUG=GST_ELEMENT_FACTORY:4 <binary> 2>&1 | grep -c 'creating element "vorbisdec"'`
   must go **24 → 0** for the media `export` suite, and be 0 for
   `harness/tests/transcribe.rs` — #70's original sighting, which the first draft
   did not touch.
2. **The suite passes unchanged.** The fixtures' *content* is what changes, so
   anything that moves is something that was depending on the audio codec, and
   that is worth knowing rather than papering over. Watch the export's audio
   assertions (tone position, loudness, silence) and the recorder's duration
   tests.
3. **One confirmation run of the repro**, after the count is 0: four concurrent
   copies of the `export` binary, `nice -n 19`, under
   `LD_PRELOAD=libc_malloc_debug.so.0 GLIBC_TUNABLES=glibc.malloc.check=3`,
   24 runs. Before: 4 crashes. Expected after: 0. This is confirmation, not the
   primary signal — the count is the primary signal, and it is 45× faster.

## U6. What must not change

- **`decode_each`'s contract**: every video sample, a loud failure on a decode
  error, and the EOS assert.
- **`TONE_RATE` and `tone_video`**: raw F32 in Matroska, for the reason its own
  comment gives (a lossy codec smears a burst's onset — Vorbis by ~21 ms). It is
  not part of this change.
- **Three comments that U0.1 falsifies** must not survive it: `fixtures.rs`'s
  two claims that the caps filter means "an audio stream is left alone" / "the
  other streams are left alone", and — now measured — the sense in which
  `audio.rs`'s "Only the audio stream" is true of the data flow but not of the
  elements. `motion.rs` and `decode.rs`'s "`decodebin3` tolerates an unlinked
  audio pad" are accurate, and misleading only by omission.

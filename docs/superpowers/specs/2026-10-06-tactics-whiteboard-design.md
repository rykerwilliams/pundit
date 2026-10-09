# Design — the tactics whiteboard

The coach (2026-10-06) asked whether an image could be a source, then said what
he wanted it for: *"yeah basically title card ability or simlar, but also,
sometime i'd want to draw over top a soccer field layout or similar"*. Offered
a choice of scope he took **the whiteboard only** — a clip whose backdrop is a
still image he talks and draws over.

**The title card is out of scope and is filed as BACKLOG #133**, under the
shape the coach gave it later the same day (*"'title maker' or similar, and it
would use a slate and a timing maybe"*), which is a different feature from the
one this spec would have had to grow into. That entry carries the analysis so
nobody re-derives it.

Read `CLAUDE.md` first. This spec does not repeat the clip-model, capture,
export, preview or format rules it states. §R keeps the record of what my own
earlier drafts got wrong.

---

## Why this is cheap, and where the work actually is

**The pixels are close to free, and the model is the whole job.** The avatar
already proves the rendering trick: a still decoded once (`decode_still`,
`media/src/composite/avatar.rs:64`), uploaded to GL once (`Texture::upload`,
`media/src/composite/export.rs:1005`) and then pushed as a re-stamped buffer
header over that one texture every frame (`Texture::stamped`, `:995`). A
whiteboard is that at the **picture** pad instead of the inset pad.

The frame loop makes this concrete. `composite/export.rs`'s loop needs exactly
one thing per output frame — a `gst::Sample` of GL memory — and today it gets
it from `Decoder::frame_at`. Everything after that reads the sample rather than
the decoder: `source_caps(sample)` (`:766`) stamps the output framerate onto
its caps, `fit_rect` (`composite/mod.rs:641`) letterboxes the picture from
those caps, `set_caps(&encoder.src, &caps)` hands them to the appsrc, and
`stamp(sample, n)` re-headers the buffer. A `Texture` can produce that sample.
So the picture source becomes "a decoder **or** one uploaded texture", and
nothing downstream of it changes shape.

Two measured facts make that safe rather than hopeful:

- **GL memory is not optional.** `Texture`'s own doc records the failure:
  a system-memory filler on a pad that had carried GL frames made `glupload`
  refuse the buffer (*"Failed to upload buffer"*), and *"any target whose
  first entry has no inset and whose second has one died there
  (reproduced)"*. The whiteboard's still must be uploaded for the same reason,
  which `Texture::upload` already does.
- **Caps may change GL to GL.** The same doc: *"the pad carries GL memory from
  the first frame to the last, and only the caps change — which GL to GL
  takes."* That is what lets a run mix a 16:9 half with a 4:3 pitch diagram on
  one picture pad.

**Where the avatar's machinery stops being reusable** is its pixel preparation,
and the line is sharp:

| Avatar | Whiteboard |
|---|---|
| `avatar.rs::circular` — cover-cropped into a **square**, masked to the circle inscribed in it | neither: the image is the picture, at its own aspect |
| pre-scaled to `layout::avatar_box_side(out_w, size)` | pre-scaled to the output, once (W9) |
| **premultiplied**, because its pad is told `blend-function-src-rgb=one` (`composite/mod.rs::premultiplied_over`) | **not** premultiplied: the picture is mixer pad 0, which is not told that, and straight-alpha RGBA is what it blends correctly |
| a rect that the pulse scales per frame | a fixed rect, `fit_rect` of its own caps |

So `decode_still` is shared verbatim and `drawn` / `open` / `circular` /
`premultiplied` are not. Getting the premultiply the wrong way round is a real
bug with a visible signature — `avatar.rs`'s module comment names it (*"a
bright halo around every soft pixel"*) — in the opposite direction here.

---

## W1. What it is

A **whiteboard clip**: a take whose picture is a still image the coach picked —
a pitch diagram — instead of the game video, with his drawings over it and his
voice on it.

- **It is a clip, in full.** It replays in the clip list, previews, exports in
  All Clips and in a tag export, carries a caption bar, carries its inset, and
  is burned into a basket film like any other clip.
- **It is recorded, never assembled.** There is no "make a 10-second title from
  this image" path. The length is the take (W3).
- **One diagram is armed at a time**, in a popover, and `W` records over it —
  the shape Devices already has for the avatar. `W` with nothing armed is a
  notice, not a modal, for `UserError::Avatar`'s stated reason: it refuses one
  click in a popover the coach still has open, and the next pick is the answer.
- **The pick is validated at pick time, with `decode_still`**, exactly as
  `bus/project.rs::set_avatar` does (`:376`): a file that will not decode is
  refused before anything is copied, so a diagram cannot fail twenty minutes
  into a take.

## W2. Where it lives in the model

**Chosen: a whiteboard is a `Clip` that names a backdrop image. `Clip` gains
one field — `backdrop: Option<String>`, the image's file name under the
project's `backdrops/` directory — and `Some` *is* whiteboard mode for that
clip.**

That last sentence is deliberately `Project.avatar`'s, whose doc says it of
itself (`core/src/project.rs:407`): *"It is the mode: `Some` and takes record
commentary only… There is no second flag to disagree with it (spec B1)."* The
same reasoning applies for the same reason — a `bool` beside a file name is two
values that can disagree, and `CLAUDE.md` forbids a field-level default on a
`bool` anyway, which would leave `Option<bool>` to express what one `Option<String>`
already says.

**`source_index` and `start_source_seconds` keep their types and keep a
meaning.** A whiteboard take is started while the coach is watching a moment of
a match, and it is *about* that moment. Storing where the player was is not a
vestige: it is this clip's place in the match, and it is what keeps three
existing rules correct without touching them.

- `Project::source_is_referenced` (`project.rs:578`) counts clips by
  `source_index`. A whiteboard clip genuinely does reference that video, for
  filing, so removing the video is refused — which is right, and needs no new
  arm.
- `Project::remove_source`'s remap and `UndoStack::purge_for_source_change`
  (`undo.rs:169`, an **exhaustive** match precisely so a new record type cannot
  be admitted in silence) both keep working, because nothing new holds a source
  index.
- The clip list can say which video a tactical explanation belongs to.

**What changes is what the entry's pixels come from, and that is one field's
doc.** `EntryMedia.source: PathBuf` becomes "the file this entry's picture
comes from" — the game video for a footage clip, the backdrop image for a
whiteboard — and its type does not change. `bus/export.rs::entry_media`
(`:710`) then stats and refuses **the image** for a whiteboard entry, which is
the refusal the coach can act on; refusing a whiteboard export because a game
video it does not draw a pixel of is missing would be a refusal with no
remedy.

*(The doc that has to be amended is on `Encode::entries` —
`export.rs:107`, **"Not an `Option`**: every entry has a game video and a match
behind it, even when it has no clip" — not on `EntryMedia::source` itself. The
field stays non-`Option`; the sentence becomes "a picture and a match".)*

### What this was weighed against

**An image as a source video — rejected, and the reason is the match clock.**
This is the coach's own first idea and it looks free: add the still to
`source_videos` with a chosen duration and nothing downstream changes. Four
things say no, and the third is disqualifying.

1. `probe` refuses it. GStreamer's `Discoverer` finds a video stream in a JPEG
   happily, so `ProbeError::NoVideo` is not what fires; the refusal is the
   duration, `probe.rs`'s closing `.filter(|d| *d > 0.0).ok_or_else(|| …"the
   file reports no duration")`. An image therefore cannot be a source today,
   and making it one means inventing a duration — a second duration authority
   beside `SourceRef::duration_seconds`, which `plan.rs::clip_source_duration`
   calls *"**The** duration authority"* and whose doc records an earlier design
   that reintroduced exactly that disagreement.
2. The **aspect gate** would refuse it. `project::aspects_match` (`:255`)
   demands every source share one display aspect within 0.5%; a pitch diagram
   is not 16:9, so the image would be refused — or, worse, a coach would crop
   his diagram to 16:9 to get past a gate that exists for the timeline's sake.
3. **It would break the scoreboard for the whole project.**
   `ScoreboardContext::for_project` (`scoreboard.rs:642`) builds
   `source_offsets` by accumulating `source.duration_seconds` over the source
   list, and `Project::absolute_match_events` puts every tagged event on that
   concatenated timeline. An invented duration anywhere but the end of the list
   shifts every later source's absolute time by it, so every match event, every
   period boundary and the clock burned into every export moves. A still that
   cost the coach his match clock is not a cheap feature.
4. `ExportTarget::WholeMatch` plays every source video whole, in order
   (`core::whole_match`), so the diagram would appear inside the match film,
   and `can_copy`'s header gate would refuse the stream copy the whole match
   depends on.

**A new record beside `Slate` — rejected.** A `Project.whiteboards` list that
is not a clip would need its own replay, preview and export path, which is the
precise special case the clip model exists to avoid:
`core/src/project.rs:424`'s comment on `Slate` states the rule — *"a clip **is**
a recording, which is what lets every clip replay, preview and export with no
special case."* A whiteboard take **produces** a recording, so it is a clip and
there is nothing a separate record would buy.

**`PlanEntry.source_index: Option<usize>` — rejected.** The tempting purist move
is to make a whiteboard entry sourceless and force every reader to decide.
Measured against the code, that is a worse trade than the one above: the field
is read by the scoreboard, `highlight_shapes`, `entry_media`, the audio mixer's
path table, the basket's per-match keying and `whole_match`, and making it
`Option` would make every one of those sites carry a branch for a case all but
one of them answers the same way. Keeping the anchor honest (W2's first half)
and reading the backdrop *once per entry* (W9) gets the same safety for one
accessor.

**A `Backdrop` enum with a payload — rejected as over-shaped.** `Option<String>`
is the project's existing idiom for "a file name that is the mode", and a second
variant has nothing to carry.

## W3. How long it is

**The take's own length, and nothing else is defensible.**

A clip's length is `recording_duration`, written by `Project::add_recorded_clip`
from what the recorder actually produced. Two alternatives were considered:

- **A stored duration on the clip.** It would be a second length authority
  beside the recording, and the first time they disagreed the preview and the
  export would disagree with each other: `composite/preview.rs`'s module
  comment is explicit that *"record time is output time"* and that the
  recording *"plays natively"* and **is the clock** (*measured:
  `GstPulseSinkClock`*). A clip longer than its recording would run the picture
  past the sound; shorter would cut the coach off mid-sentence.
- **A fixed duration with no recording at all.** That is the title card, which
  is #133 and which needs a plan entry with no commentary — a different feature.

So nothing in `core::timeline` or `core::plan` changes. `playback_segments`
walks the clip's events as it always did and `frame_count` quantizes the entry
as it always did. A `Play` or `Pause` logged during a whiteboard take produces
segments whose `source_start` nothing reads, because the picture is the same
texture at every `source_time` — which is why the transport is refused during
the take (W7) rather than special-cased here.

## W4. The image file: where it lives, and when it is written

`<project>/backdrops/<clip id>.<ext>`, copied in **at take start**, immutable
afterwards.

- **Under a directory of its own, named by the clip's id**, which is
  `recording_filename`'s shape — its doc reads *"`<uuid>.mkv`, relative to the
  project's `recordings/` directory"* — rather than `avatar.<ext>`'s. There is one avatar
  per project; there is one diagram per *clip*, so the name has to be unique and
  the clip's id is the only identifier that already is.
- **The extension is kept**, as `set_avatar` keeps it (`project.rs:382`) and for
  its stated reason: the file opens in a file manager as what it is, and the
  decoder sniffs regardless.
- **Copied through a temp file and a rename in the same directory**, as
  `set_avatar` does (`:387`) and as `store::write` does for `project.json`.
- **At take start, not at `finish_recording`.** This is the one piece of
  sequencing worth arguing. Copied at the end, a failed copy would leave a real
  recording whose `backdrop` is `None` — a clip that silently became a footage
  clip over an anchor it never showed, after the coach had said his piece.
  Copied at the start, the failure lands where `start_recording` already
  refuses for I/O (`recording.rs:151`, the `create_dir_all` arm) — and after
  `pending.id` is minted four lines above it, so the name is already in hand —
  before a word is spoken. `abort_recording` removes it alongside the recording it already
  removes.
- **Immutable: there is no "change this clip's diagram".** The commentary was
  recorded against that image; swapping it would leave the coach pointing at
  things that are no longer there. This is the same reason a clip cannot be
  re-anchored to different footage.
- **Nothing ever deletes it, including a clip delete.** A deleted clip's
  recording is moved to `recordings/.trash/<recording_filename>`
  (`bus/clips.rs:328`) and shredded on the next open; a backdrop is left where
  it is instead. The alternatives are both worse than the problem: a second
  trash path to keep in step with the first (this week's lesson — see W10), or a
  sweep of `backdrops/` against the live clips, which is a glob over a folder
  the coach can also write into and which `set_avatar`'s own comment refuses to
  do (*"Exactly the file the project named, never a `avatar.*` glob over a
  folder the coach can also put files in"*). An orphaned diagram costs a few
  hundred kilobytes, is named by a uuid so it can never be mistaken for the
  coach's own file, and is what makes delete-then-undo work with no code at
  all.

**The armed diagram is a path in `state.json`, not in the project.** It is "the
file on this machine I am drawing on today", which is the test
`bus/state.rs` states for that file — the same test that sent the whisper model
there. A project carried to another computer must not bring a path to a PNG in
someone else's home directory. It therefore costs no format bump of its own,
and it follows `bus/state.rs`'s per-field lenient read (#100), so an unreadable
value costs the arm and nothing else.

## W5. The camera: nothing changes, and that is the answer

**A whiteboard take opens the camera exactly when any other take of that
project would.** `Bus::avatar_mode` (`recording.rs:333`) is
`project.avatar.is_some()`, and `capture_sources` (`:342`) branches on it: with
an avatar, no camera is resolved, no `NoCamera` can be raised and a
camera-less machine records fine; without one, the camera is resolved as
always. The whiteboard changes what is on the **picture** pad, not what is
captured, so **`capture_sources` needs no change at all** — and
`CaptureSources::Test`'s `video: (!avatar).then_some(video_delay)` keeps test
takes the same shape as the coach's, which is the rule that branch exists for.

So the fork the brief flagged resolves itself, in the coach's favour on both
sides:

- **In a camera project**, a whiteboard take shows his face in the corner over
  the diagram. `Clip::show_pip` turns it off per clip and `inset_corner` (v13)
  moves it out of the part of the pitch he is drawing on — both existing
  controls, both already in the inspector, neither needing a word of new code.
- **In an avatar project**, it shows the avatar, pulsing, as every take of that
  project does.

**No new flag earns its place here.** A per-clip "camera on a whiteboard"
switch would be a third thing to disagree with `Project.avatar` and
`Clip::show_pip`, and `Clip`'s own comment on the placement accessors
(`project.rs:362`) is a warning against exactly that: `show_pip × inset` is
interpreted *in one place* and *"each answers where in the same breath as
whether"*. The whiteboard adds no reading of that pair.

**What does need work is the self-view during the take.** The window draws the
inset over the picture rect, and during a whiteboard take the picture is the
diagram rather than the mailbox's decoded frames. The player must therefore
show the armed image — from `media::decode_still`'s pixels into a
`slint::Image`, once per change of the armed file, never
`slint::Image::load_from_path` (slint is built with no image decoder) and never
a decode on the UI thread at take start, which is the rule `CLAUDE.md` already
states for the avatar corner. `layout::self_view_rect` /
`avatar_self_view_rect` then place the inset over that rect unchanged — and the
self-view's quiet timer stays the camera's alone, as it already must be.

## W6. The scoreboard: none, and no cues for its span

**A whiteboard entry draws no scoreboard.** The board's only job is to tell a
viewer where in the match the picture is, and a diagram is not in the match.

The mechanism is one `Option` at one site. `composite/export.rs`'s loop reads

```rust
let scoreboard = media.match_media.scoreboard.as_ref().and_then(|context| {
    let state = context.state_at(entry.source_index, frame.source_time)?;
    Some((context.config(), state))
});
```

and a whiteboard entry simply does not ask. The omission is **per entry**, not
per run: `MatchMedia` is built once and shared by `Arc` across every entry of a
project, so blanking it at the job would cost the footage clips in the same
export their board.

**The alternative was considered and rejected.** The clock *could* be drawn
frozen at the clip's anchor — `source_time` is constant across a whiteboard
entry, so it would show the minute the coach stopped to explain, and it would
be arithmetically honest. It is rejected because it reads as a claim about the
game: a board sitting at `58:12` over a two-minute tactical explanation tells
the viewer the match is paused there, which is not what is happening. "Where a
tag can't be told the truth it is left out, never guessed" is already this
codebase's rule for file tags; this is the same call.

**And no cues.** Only the whole match carries the board beside the file;
`bus/export.rs::carry_scoreboard` (`:608`) returns `cues: None` for every other
target, so a clip or basket export containing a whiteboard needs no change
here. A whiteboard clip simply contributes no cue line, which `core::cues`
expresses by having nothing to add.

**The caption bar stays**, and is the one piece of furniture a whiteboard
clip does want: `PlanEntry::text` is `"<n> / <total> | <name> | tags"`, and a
diagram's name is exactly what a viewer needs. `layout::bar_rect` takes
`Clip::inset_placement()` — kind-blind, an `Option`, and already correct — so
the bar stops where the inset stands as it does everywhere else.

## W7. What is refused, and why each

Worked through feature by feature rather than assumed.

| Feature | On a whiteboard | Why |
|---|---|---|
| **Pen strokes** | **allowed, unchanged** | The point of the feature. Strokes live in the content rect and are zoom-agnostic (`highlight.rs`'s module comment), and the content rect is the diagram's `fit_rect`. Nothing to change. |
| **Clear-all** | allowed | A stroke-log operation; knows nothing about the picture. |
| **Zoom** | **allowed, unchanged** | `install_zoom`'s PTS-keyed probe drives `gltransformation` on the picture pad, whatever that pad carries. Zooming into a corner of a set-piece diagram is plausible, and allowing it costs **no code** — a refusal would cost a guard. |
| **The inset (camera or avatar)** | allowed | W5. `show_pip` and `inset_corner` are the controls. |
| **Caption bar** | allowed | W6. |
| **Scoreboard** | **refused (not drawn)** | W6: there is no match time. |
| **Player highlights** | **refused**, both drawn and placed | A highlight *belongs to the footage* and is keyed by `(source_index, stream time of the frame it was placed on)` — `highlight.rs`'s module comment, and `highlight_shapes(…, entry.source_index, frame.source_time, …)` at the draw. A ring placed during a whiteboard take would be keyed to footage that is not on screen, and would then appear over the **game video** somewhere else entirely. The refusal is a notice at placement, and the draw simply does not ask. |
| **Transport: play/pause, skip, scan (`J`/`L`), step (`,`/`.`)** | **refused during a whiteboard take**, one guard, one notice | There is nothing on screen to move. Allowed, they would move the game video invisibly behind the diagram and log `Play`/`Pause`/`Skip` events into the clip, whose segments nothing then reads — the silent-wrong-value class this design is otherwise built to avoid. One refusal covering the group, not five, so the message can say the one true thing. |
| **Match tags (`z`/`x`/`v`), slate marks (`i`/`o`)** | **refused during a whiteboard take** | These are records *about the footage*, placeable "whenever the footage is on screen" — the stated reason they are on the recording allow-list. During a whiteboard take it is not. |
| **Preview** | allowed | W9. |
| **Transcription** | allowed, unchanged | It reads the recording's audio; the picture is irrelevant. |
| **The goals reel** | n/a | A reel's entries have `clip_id: None` and are cut from the goals; it never contains a clip. |
| **The whole match** | n/a | Built from `source_videos`, which a whiteboard never joins (W2). |
| **A basket piece** | allowed | A piece is `(project folder, clip id)`; a whiteboard clip resolves like any other, and `EntryMedia.source` carries its image. Each piece already draws its own match's board — and a whiteboard piece draws none, by W6. |

**The audio is commentary only, and core must say so.** `audio_regions`
(`core/src/audio.rs:123`) builds a game region per `Play` segment of every
entry, and the mixer builds its reader paths *from the regions*. A whiteboard
entry's `source` is a PNG, so a game region would have media open an audio
pipeline on an image. It must contribute **no game region** — a third guard
beside the two already in that function (`if source_volume != 0.0` and `if
entry.clip_id.is_none()`), of exactly their shape. The commentary region is
unchanged, which is the only sound a whiteboard clip should have.

## W8. The format

**One field on one existing struct: `Clip::backdrop: Option<String>`, with a
field-level `#[serde(default)]`.**

- `Option` with a field-level default is `CLAUDE.md`'s rule for a field added to
  an existing struct, and `None` is exactly what every v7–v15 clip means: no
  backdrop, the footage. It is the shape `Clip::slate_id` (v12) already takes.
- **No new struct**, so no required-field question arises.
- **`Preferences` gains nothing.** The armed diagram is a path on this machine
  and lives in `state.json` (W4), so this feature adds no `Preferences` field
  and the "container default, never a field-level one" rule has nothing to
  apply to.
- **Take the next free `formatVersion` from `store.rs` at implementation
  time.** It is **15** today (`store.rs:21`; the mute switch took it), so the
  next is 16 — but **both #78 and #115 want 16**, and only one bump can be. If
  two of the three land in the same stretch they should **share one bump**: one
  version, two fields, one every-readable-version test. Do not hard-code 16
  here.
- **The readable floor stays 7** (`store.rs:26`): the field is additive.
- **The bump is for the write, not the read.** An older build would ignore
  `backdrop` on read and drop it on its next save — turning a whiteboard clip
  into a footage clip over an anchor it never showed, silently, which is
  precisely the damage the version guard exists to prevent and what
  `project.json.v<old>` is the way back from. That backup is named from the
  project's **own** stored version, not from `CURRENT - 1`.
- **One test per bump**, `a_v15_file_loads_under_the_current_version` beside
  its siblings in `core/tests/project_format.rs`: the series is named for the
  *old* file version, and that per-bump test is the only thing pinning the
  constant.

## W9. Export and preview: what changes in each tail, exactly

**Common (`composite/`).** `Texture` moves from `export.rs` to `composite/mod.rs`,
beside `Decoder`, because both tails now need it; its three methods are
unchanged. The still's preparation joins `composite/avatar.rs` — which is
already *"one decoder, and one set of drawn pixels"* — as a second, shorter
path: `decode_still`, then **one** `tiny_skia` `draw_pixmap` to scale it down
to the output size if it is larger, and **no** crop, **no** circle mask and
**no** premultiply (see the table at the top). Pre-scaling once on the CPU
rather than letting the mixer resample a 4000×3000 texture every frame bounds
the VRAM and the per-frame GPU work to what footage already costs.

**One reading of the backdrop per entry, threaded to its users.** This is
`composite/preview.rs`'s own pattern, whose module comment and `CLAUDE.md`
both record why: preview reads the camera answer once and threads it to three
places *"which must agree or the mixer stalls; one answer rather than three
reads is what makes them agree."* A whiteboard entry's answer has four users —
the picture source, the board, the highlights, and `fit_rect`'s input — so it
is read once, from `Clip::backdrop()`, and passed.

**Export (`composite/export.rs`).** The loop keeps its shape; the picture
source becomes an enum of the two arms:

- today: `sources: HashMap<PathBuf, Decoder>`, `Decoder::start` on first use,
  `decoder.frame_at(seconds_to_clock(frame.source_time), &watch)` per frame.
- a whiteboard entry: one `Texture` per distinct image, keyed on the path, and
  a `gst::Sample` built from `stamped(n)`'s buffer and caps. `source_caps`,
  `fit_rect`, `set_caps` and `stamp` all take it unchanged.
- **the decoder bound is unaffected**: `close_unread` drops a decoder as soon
  as neither the previous nor any later entry reads it, and a whiteboard entry
  reads none, so it can only *reduce* the number open. The zero-copy diagnostic
  is still taken when the **first decoder** opens — which, for a run whose
  first entry is a whiteboard, is now the first *footage* entry's. That is the
  right reading of the line, and it is why the existing code takes it at the
  open rather than after the loop.
- `Pip::open` is untouched: it already falls through to the filler for anything
  the two placement accessors do not claim, and a whiteboard clip's `show_pip`
  and `inset` mean what they always meant.
- the `glvideomixer` geometry, the overlay pad's z-order and
  `install_overlay_pad`'s rule are untouched. **Nothing is ever mixed over the
  overlay**, and a whiteboard changes only what pad 0 carries — so the coach's
  pen is still the top layer, which on a whiteboard is the entire point.

**Preview (`composite/preview.rs`).** `PreviewJob.source` stops being "the game
video" and becomes "the picture's file", matching `EntryMedia.source`. The one
`Decoder::start(&job.source, gl, watch)` at `:407` takes the same two-arm
treatment, and because preview is always **one** clip there is no mid-run caps
change at all: a whiteboard preview pushes one texture for every frame of its
life. Everything else holds:

- the recording still plays natively for the inset and the commentary audio,
  and is still the clock — `decode.rs` links *"the video stream only. Other
  pads (audio) stay unlinked"*, which is why a whiteboard's missing source
  audio is not a new case: **a preview has never carried source audio at all**
  (BACKLOG #125).
- the inset pad is still requested only when there is something to feed it, by
  `camera_inset` and `AvatarInset::open`, both reading the same two accessors.
- it runs on **Slint's** GL context (`Gl::shared()`), so the texture uploads
  there as it does in the export's private surfaceless display.

**What does not change anywhere:** the pump, `frame_time`/`stamp`, the PTS-keyed
probes, `install_zoom`, the audio mixer's region machinery, the encoder ladder,
the chapters, the tags, the sidecars, and the stream-copy path (which a
whiteboard can never reach).

## W10. Diff it against what it resembles — an explicit step, not a hope

Three of the four defects found in the slates panel this week — #130, #132 and
a `J`/`L` hang — were **rules the clip inspector already had that the slates
copy did not**. This design resembles two shipped things, so the plan must
include a pass that lists each and checks it off rather than trusting the
resemblance.

**Against the avatar** (`2026-09-22-avatar-recording-design.md`,
`composite/avatar.rs`, `bus/project.rs::set_avatar`):

1. Decode **before** copying, so an undecodable file is refused with nothing
   written (`set_avatar:376`). → W1.
2. Copy through a temp file and a rename in the same directory
   (`set_avatar:387`). → W4.
3. Replace deletes **exactly the file the field names**, never a glob. → W4
   (nothing is deleted at all, for that reason).
4. Removing the project's copy never touches the coach's original. → W4.
5. The decoded pixels are prepared in **one** place, so the popover, the live
   corner and the export cannot disagree. → W9 (the still's second path in
   `composite/avatar.rs`).
6. Never decode on the UI thread at take start; never
   `slint::Image::load_from_path`. → W5.
7. A failure costs the inset, not the run (`open_reported`). → **this one
   inverts**: a missing *backdrop* is the whole picture, so it is
   `entry_media`'s refusal before the run starts, not a stderr line during it.
   Worth writing down precisely because it is where the resemblance stops.

**Against a footage clip** (`core::project::Clip`, the inspector,
`bus/clips.rs`):

8. Every per-clip control in the inspector: name, notes, tags, `show_pip`,
   inset size, inset corner, transcript. All apply unchanged; the pickers read
   `inset` on its own, deliberately, and that is unaffected.
9. Delete, undo, redo, trash and restore. → W4: the recording's path is
   unchanged and the backdrop is immune, so restore works with no new code.
10. Reorder (`sort_index`), the tag overview, the tag filter, the basket's
    "add to basket". All unchanged.
11. `source_is_referenced`, `remove_source`'s remap,
    `purge_for_source_change`'s exhaustive match. → W2: unchanged, by keeping
    the anchor honest.
12. Jump-to-clip-start, double-click-to-the-in-point and the transport that
    follows a selected clip: these take the clip to its `source_index` /
    `start_source_seconds`, which a whiteboard clip *has*. They should keep
    working — the anchor is where the explanation belongs — and the plan must
    confirm that, not assume it.
13. The transcript row and its model picker. Unchanged (W7).

## W11. Tests — four groups

1. **Core (`core/tests/`).** `audio_regions` over a compilation with a
   whiteboard entry yields **commentary regions only** for it, and the footage
   entries beside it are **byte-identical** to the all-footage run's — the
   second half is what pins "the guard is per entry", and it is sound because
   `region`'s priming drop is computed per region from the entry's own frames.
   Plus `project_format.rs`'s `a_v15_file_loads_under_the_current_version`, and
   a round-trip asserting an absent `backdrop` key reads as `None`.
2. **Media (`media/tests/`).** An export of one whiteboard clip decodes back to
   a file whose frames are the image, letterboxed to its own aspect inside
   1920×1080 — the assertion is the **picture rect**, from the bars, which is
   `fit_rect`'s answer and is checkable without a pixel hash. And a
   **mixed** compilation, whiteboard then footage, which is the case
   `Texture`'s doc says died when the caps feature changed: it must run to EOS
   with both entries' frames present.
3. **Harness (`harness/tests/`).** A whiteboard take end to end over
   `CaptureKind::Test`: `W` with nothing armed is a notice and records nothing;
   with a diagram armed it produces a clip whose `backdrop` is set, whose file
   is under `backdrops/`, and whose `source_index` is where the player was. An
   aborted take leaves **no** file in `backdrops/`. A copy failure refuses the
   take and leaves the player as it was — the rule `start_recording`'s doc
   already states for every refusal above its seek.
4. **Refusals.** One test per group in W7's table that refuses: the transport
   group, the match/slate marks, and a highlight placed during a whiteboard
   take. The highlight one matters most, because its failure mode is silent and
   remote — a ring appearing over the game video somewhere else.

## W12. The question the coach answered, and when

**One diagram at a time, or a small set kept with the project? — ANSWERED,
and it is the default below.** The coach, asked during this spec's own
drafting: *"Eventually yeah I would want a library of diagrams but for now I
guess you just choose one"*. So: **one at a time**, armed in `state.json`, and
the single-field format change.

**It was never written into this section, and that cost something.** The answer
arrived while the spec was being drafted and the section kept its "if he does
not answer" wording, so the plan stage opened 2026-10-09 with a question the
coach had already settled — the same shape of bookkeeping miss as the whiteboard
having no BACKLOG entry (#138). Recorded here 2026-10-09.

The reasoning, which stands either way:

What this spec assumes is **one at a time**: a path armed in `state.json`, a
popover to change it, and `W` to record over it. A coach with a pitch, a
half-pitch and a set-piece box would re-pick between takes.

The alternative is a small library kept **with the project** — a
`Project.backdrops` list and a picker with three thumbnails — which is a second
stored struct, a lifetime question this design currently has none of (W4), and
a reference count between the library and the clips that named its images.

- **What it blocks:** the picker's shape, and whether the format change is one
  `Clip` field or a field plus a new list. Nothing else in this spec moves
  either way — W2 through W11 are identical under both.
- **The default if he does not answer:** one at a time, `state.json`, the
  single-field format change. It is the cheaper half and it is strictly a
  subset: a library can be added later without changing `Clip::backdrop`'s
  meaning, because a clip already stores its **own copy** of the image it was
  recorded over.

Everything else the brief listed as a question is settled above and should not
be handed back as one: the model (W2), the length (W3), the file's home (W4),
the camera (W5), the scoreboard (W6), the refusals (W7), the format (W8) and
both tails (W9).

## §R. What earlier drafts of this got wrong

1. **I started from "an image is a source", and the match clock is why it
   fails.** The first two things I wrote down — `probe`'s duration refusal and
   the aspect gate — are both real but both *fixable*, and they are not the
   reason. `ScoreboardContext::for_project`'s running sum over
   `source_videos` is the reason, and it took reading that function to find it.
   A spec that had argued only the probe would have been answered with "then
   relax the probe", and the project's clock would have been the price. W2
   leads with the fatal one.
2. **I assumed `source_index` had to become optional**, and designed a
   propagation of `Option<usize>` through `PlanEntry` before asking whether the
   anchor was meaningless. It is not: the take is *about* a moment of the
   match, and keeping it honest leaves `source_is_referenced`,
   `remove_source`'s remap and `purge_for_source_change` correct with no new
   arms. The `Option` would have put a branch into six readers to serve one.
3. **I had the premultiply the wrong way round.** My first version of W9 reused
   the avatar's pixel path wholesale, which premultiplies — correct for the
   inset pad, which is told `blend-function-src-rgb=one`, and wrong for the
   picture pad, which is not. A transparent PNG would have come out haloed, the
   mirror image of the bug `avatar.rs`'s module comment records.
4. **I planned to copy the image at `finish_recording`.** A failed copy there
   leaves a finished recording with `backdrop: None` — a clip that silently
   became a footage clip over an anchor it never showed. Moving the copy to
   take start puts the failure where `start_recording` already refuses for I/O,
   before anything has been said.
5. **I designed a `backdrops/` sweep to collect orphans**, then deleted it: it
   is a glob over a folder the coach can write into, which `set_avatar`'s own
   comment refuses to do, and the problem it solves is a few hundred kilobytes.
   "Nothing ever deletes it" is also what makes delete-then-undo work for free.
6. **I had the scoreboard blanked on `MatchMedia`.** It is built once per run
   and shared by `Arc` across every entry of a project, so that would have cost
   the footage clips in the same export their board. The omission is per entry,
   at the draw.
7. **I nearly made the camera a new per-clip flag.** `Project.avatar.is_some()`
   plus `Clip::show_pip` plus `inset_corner` already answer it completely, and
   a fourth value in that conversation is what `Clip`'s comment on the
   placement accessors exists to warn against.
8. **Two claims in `CLAUDE.md` are wrong and are worth fixing while we are
   here.** `avatar_box`, `avatar_box_side` and `AVATAR_BOX_RATIO` live in
   **`core::layout`** (`layout.rs:252`, `:279`, `:228`), not `core::avatar`,
   which `CLAUDE.md` names three times; `core::avatar` holds `avatar_rect`,
   `level_from_db`, `smooth` and `pulse`. And the doc that says an entry's
   source is *"**Not an `Option`**: every entry has a game video and a match
   behind it"* sits on `Encode::entries` (`export.rs:107`), not on
   `EntryMedia::source` — which matters here, because `EntryMedia::source` is
   the field whose meaning W2 changes and `Encode::entries` is the doc that has
   to be amended.

# Plan — the tactics whiteboard (#138)

Spec: `docs/superpowers/specs/2026-10-06-tactics-whiteboard-design.md`. Read it
and `CLAUDE.md` first; neither is repeated here.

**The spec is complete and its one open question is answered.** The coach,
during its drafting: *"Eventually yeah I would want a library of diagrams but
for now I guess you just choose one"* — one diagram armed at a time, in
`state.json`, and the single-field format change. §W12 records it (written
2026-10-09: the answer had arrived and the section still read "if he does not
answer", the same bookkeeping miss as #138 having had no backlog entry).

**This is the reviewed second draft.** §B is the record of what the adversarial
pass changed, and it is long: the first draft's own §A found two drifts and
**missed twelve**, four of which made a task unimplementable and two of which
dropped spec requirements with correctness consequences. Read §B before the
tasks.

## Where this stands (update it as tasks land)

- **1 — core: the field, the bump, and the freeze.** Not started.
- **2 — media: the picture is a decoder *or* a still.** Not started.
- **3 — the bus: arm it, record over it, preview it, refuse the rest.** Not
  started.
- **4 — the window shows the diagram during the take.** Not started.
- **5 — the UI: the popover and `W`.** Not started.

**Shipping: five PRs.** 1 ships alone. **2 and 3 are parallel after it** —
task 2 is `pundit-media` and task 3 is `pundit-app/src/bus` plus the harness,
and they share no file; the first draft said *"nothing is parallel"* on the
grounds that both read the new field, which is a shared **dependency** on task
1, not a conflict. 4 needs 3, and 5 needs 4.

**Task 4 is new in this draft and is the one the feature's correctness rests
on** (§B.2).

---

## §A. The two drifts the spec has, re-derived

Checked at `a1c4b4f`, this branch's base.

| Spec says | Actually |
|---|---|
| the next free `formatVersion` is **16** (*"it is 15 today; the mute switch took it"*) | **17**. `CURRENT_FORMAT_VERSION` is **16** (`store.rs:21`) — #78's two output switches took it on 2026-10-07, after this spec was written. The spec's own instruction is why this was caught: *"take the next free from `store.rs` at implementation time… do not hard-code 16 here"* |
| §R.8: `CLAUDE.md` names `avatar_box` in `core::avatar` three times | **already fixed** — `CLAUDE.md:538` and `:718` read `core::layout::avatar_box`, corrected in #88's review pass. §R.8's *second* claim stands: `Encode::entries`' doc still reads *"every entry has a game video and a match behind it"* (`composite/export.rs:107`) and task 2 amends it |

**And the spec's `path:line` citations are stale in eight places**, which the
first draft asserted were "sound" without checking: `entry_media` is
**`bus/export.rs:1103`**, not `:710`; `carry_scoreboard` **`:966`**, not `:608`
(both moved when #77's export queue landed); `set_avatar` **`project.rs:396`**
with its decode at `:400` and its copy/rename at `:412`–`:413`, not
`:376`/`:382`/`:387`; `source_is_referenced` **`:600`**, not `:578`;
`aspects_match` **`:277`**, not `:255`; `Project.avatar`'s doc **`:429`**, not
`:407`; `Slate`'s comment **`:458`**, not `:424`; the placement-accessor comment
**`:385`**, not `:362`. **Every `bus/export.rs` number in the spec is ~390
lines out.** Re-derive at each task; do not cite the spec's numbers.

**One slip in the first draft's own framing, named because it is the "relayed
numbers" trap by another route.** It said *"checked at `54dc7cc`"* — a commit on
`claude/77-task-3`, **not an ancestor of this branch**. The one fact it carried
holds at both revisions, so nothing downstream was wrong; the rev was right and
the branch was not, and citing a tree the reader cannot check is the same
failure as citing a line from memory.

---

## §B. What the adversarial pass changed — two found, twelve missed

Both reviewers read the source. I verified every claim I applied against the
source myself rather than taking either report's word for it, and where they
disagreed I went to the code (§B.1).

**Four findings made a task unimplementable as written:**

1. **The audio guard had nothing to read.** `audio_regions(&Compilation,
   &Preferences, f64)` sees `Compilation { frames, plan }`, and `PlanEntry`
   carries `clip_id`, `source_index`, `segments`, `start_frame`, `frames`,
   `text` — **no clip and no backdrop**. Its two existing guards read an
   *argument* and a *`PlanEntry` field*; a third "of exactly their shape" had no
   third source of truth. §A checked that there were two guards and stopped one
   line short.
   - **The two reviews disagreed about the fix and the code settles it.** One
     said delete the guard (a game region on a PNG costs one short-lived
     pipeline and `Reader` yields silence, because `Mixer::readers` is
     `Option<Reader>` and a soundless file is recognised from `decodebin3`'s
     stream collection). The other said move it into `playback_segments`.
     **`playback_segments(clip: &Clip, …)` already has the clip**, and
     `game_regions` only emits for `SegmentKind::Play` — so **one `Freeze`
     segment spanning `recording_duration` emits no game region at all**, needs
     no new field, costs no pipeline, and makes "`source_time` is constant
     across the entry" true *by construction* rather than by assertion. Task 1
     takes that.
   - **It matters more than "a PNG is harmless" suggests.**
     `playback_segments` starts `Walk { rate: 1.0 }`, so a clip with an
     **empty** event list is all `Play` — which every media and core test
     fixture writes. Real takes open with a `Pause`, so the hazard is in the
     fixtures, which is exactly where a silent wrong value survives.
   - **Spec W3's "nothing in `core::timeline` changes" is therefore false**, and
     task 1 says so.
2. **W5's live picture was dropped, and it is what makes the strokes land.**
   The spec: *"What does need work is the self-view during the take … The player
   must therefore show the armed image."* No task carried it. Worse than a
   missing feature: `frame-width`/`frame-height` are set in `video.rs:160` from
   the decoded **game frame**, the window's `content` rect is
   `place-picture(identity, frame-width, frame-height, …)`, and **strokes are
   normalized against that rect** — so a 4:3 diagram over a 16:9 source means
   every stroke is replayed at the wrong aspect in the export, with the coach
   having watched the game video throughout. **Task 4 is now that.**
3. **A whiteboard preview would render the game video — and be refused when
   that video is missing.** `PreviewJob.source`'s only producer is
   `bus/preview.rs:94`, `open.folder.join(&video.relative_path)` from
   `source_videos[clip.source_index]`; and `open_preview` refuses first at
   `:58`–`:62` with *"the clip's game video is gone"* / *"missing; relink it
   first"*. **`bus/preview.rs` was in no task's file list**, so W7's *"Preview —
   allowed, unchanged"* is a fact error and the spec's own "refusal with no
   remedy" argument was left standing one layer up.
4. **`tiny_skia::Pixmap` is premultiplied storage**, so "one `draw_pixmap` and
   no premultiply" was self-contradictory — `avatar.rs::circular` premultiplies
   *first* for that reason, and feeding straight-alpha bytes to a `Pixmap`
   filters invalid data (r > a). `Still` is **already** straight-alpha packed
   RGBA and `Texture::upload` takes exactly that, so the preparation is one call
   and the premultiply question leaves the feature. **This deleted the draft's
   Risk 1, a sabotage proof and a "checked by eye" escape hatch** — the one
   thing it said could ship looking nearly right. *(The pad claim it rested on
   is true and kept: `premultiplied_over` touches `sink_2` and the two inset
   pads and never pad 0.)*

**Two things the draft claimed that are false:**

5. **Task 1 was not "pure core" and could not "ship alone."** A new `Clip` field
   breaks five exhaustive literals outside core — `media/tests/preview.rs`,
   `media/tests/export.rs`, `media/src/overlay.rs`,
   `media/src/composite/export.rs`, `harness/src/lib.rs` — and CI runs
   `clippy --workspace --all-targets -D warnings`, so that PR would be red.
6. **The copy's refusal cannot "leave the player as it was."**
   `start_recording`'s doc says the opposite in as many words: *"the two below
   the seek … leave it **at the in point**"*. Both reviews found this
   independently, and so had I before they reported. The copy goes **above** the
   seek, with the clip id minted there — which is the one place this feature
   changes the recording path rather than adding to it.

**Six things it left unsaid that an implementer would have had to invent:**

7. **Who writes `Clip.backdrop`.** `PendingClip` is `#[derive(Copy)]` over
   `{Uuid, usize, f64}`; a `String` would cost it `Copy` and touch four
   construction sites. **`slate_id` is the precedent**: `add_recorded_clip`
   writes `None` and `finish_recording` assigns beside `clip.slate_id =
   active.slate` — so **`Active` must gain the file name**, which is also what
   gives `abort_recording` something to delete.
8. **Nothing creates `backdrops/`.** `store::write` makes `recordings/` only,
   and `set_avatar` copies into a folder that already exists. One leaf under a
   folder that exists means **`create_dir`**, not `create_dir_all`, by the
   new-match flow's own argument. And the temp file must be **per clip**
   (`.<id>.tmp`), or two takes race one path; a failed copy removes it, as
   `set_avatar` does.
9. **How `W` reaches the bus**, and the consequence the draft never weighed: as
   a *new* command it is off the recording allow-list, so **`W` could not stop
   the take it started** — refused with a bare `eprintln!`, breaking the R/W
   symmetry `toggle_recording`'s doc argues for at length. Task 5 takes
   `ToggleRecording { slate, backdrop }` instead, which keeps the symmetry and
   needs no allow-list edit.
10. **No `UserError` variant was named, and `is_notice` is a hand-written
    `matches!`.** A variant added to the enum and not to `is_notice` renders as
    a **modal** — which `UserError::Scoreboard`'s and `::Slate`'s docs say is
    unacceptable because it *"could land over a live commentary take and swallow
    the transport keys"*. Named in task 3, with a test that asserts
    `is_notice()`.
11. **`Frame<'a> { sample: &'a gst::Sample }` is a borrow out of the decoder
    map**, and `Texture::stamped(n)` returns an **owned** `(Buffer, Caps)` — so
    the two arms have different ownership and the loop needs a local that
    outlives the push. Also `Encoder::push` already calls `stamp(sample, n)`, so
    building from `stamped(n)` would **stamp twice**: the picture path wants the
    texture's buffer and caps as one owned `Sample` built once per texture, with
    `push` doing the stamping.
12. **`Recorder::start` failing returns without reaching `abort_recording`**,
    so it would leave the backdrop behind — which the planned "an aborted take
    leaves no file" test would not have covered.

**And three sabotage proofs were deferred decisions dressed as proofs**, one of
which could not fail by construction. Each is now a real proof, an assertion in
a test, or deleted with the reason.

**The good news the correctness pass also established**, because it changes how
task 2 argues: **the mid-run GL→GL caps change on pad 0 is not a new
capability.** Two sources of different resolutions with the same display aspect
already change pad 0's caps mid-run, which is why `set_caps` and the per-entry
`fit_rect` exist; both the decoder's appsink and `Texture::upload` negotiate the
same `memory:GLMemory`/`RGBA`/`texture-target=2D`, and `Texture::upload` already
stamps `framerate=OUTPUT_FPS/1`. **Cite that precedent, not the inset pad's
doc**, which measured a different pad.

**Two things are deferrals for the coach**, at the end of this plan.

---

## 1. Core: the field, the bump, and the freeze

**Files.** `crates/pundit-core/src/project.rs`, `src/store.rs`,
`src/timeline.rs`, `crates/pundit-core/tests/project_format.rs`,
`tests/timeline.rs` — **plus the five exhaustive `Clip` literals outside core**
(§B.5): `media/tests/preview.rs`, `media/tests/export.rs`,
`media/src/overlay.rs`, `media/src/composite/export.rs`,
`harness/src/lib.rs`. Each gets `backdrop: None` and nothing else.

1. **`Clip::backdrop: Option<String>`** with a **field-level**
   `#[serde(default)]` — `CLAUDE.md`'s rule for a field on an existing struct,
   and `None` is exactly what every v7–v16 clip means. `Clip::slate_id` is the
   shape. Its doc says the thing the spec leads with: **`Some` *is* whiteboard
   mode for that clip**, as `Project.avatar`'s doc says of itself, so there is no
   second flag to disagree with it.
   - **No accessor.** The first draft wanted `Clip::backdrop()` "beside the
     placement accessors"; those exist because they fold **two** fields into one
     answer and `Clip`'s own comment says so. `backdrop` is one public `Option`
     with nothing to fold, and the directly comparable field —
     `recording_filename` — has no accessor either. A method with the field's
     name is a readability trap.
   - **`add_recorded_clip` writes `None`** (§B.7); the assignment is task 3's,
     beside `slate_id`'s.
2. **Bump `CURRENT_FORMAT_VERSION` to 17**, not 16 (§A). **Check `store.rs`
   again at the task**: #115 also wants a bump, and if the two land together
   they **share one** — one version, two fields, one per-bump test.
3. **`playback_segments` returns one `Freeze` for a whiteboard clip** (§B.1),
   spanning `recording_duration`. It already has the `Clip`, so the guard is one
   line. What it buys: **no game region at all**, because `game_regions` only
   emits for `Play`; the same entry length, because `frame_count` sums
   `out_duration` either way; and `frame.source_time` genuinely constant across
   the entry, which the spec asserts and this makes true by construction.
   **Spec W3's "nothing in `core::timeline` changes" is false and this is why.**

### Tests

- `project_format.rs::a_v16_file_loads_under_the_current_version`, beside its
  siblings and **named for the old version**, which is the series' rule and the
  only thing pinning the constant.
- A round-trip whose **unique** content is one line — `Some("pitch.png")`
  survives write-then-read under the key `backdrop`. The absent-key half is
  already the bump test's job (it strips the key from the raw JSON and asserts
  the default fills it), and `an_inset_size_and_corner_round_trip` is the
  precedent for keeping both; say which half is whose rather than restating.
- `timeline.rs`: a whiteboard clip's segments are **one `Freeze`** of
  `recording_duration`, and a footage clip's are byte-identical to today's.

### Sabotage proof

1. **Leave `CURRENT_FORMAT_VERSION` at 16.** `a_v16_…` must fail — but **not**
   "on its own name", which the first draft said and is wrong. Every version
   assertion in that file reads `CURRENT_FORMAT_VERSION`, so none can tell 16
   from 17; what fails is the **`project.json.v16` backup** assertion, because
   `store::write` keeps a copy only of a file *older* than what it writes. That
   test's own doc says so, and it is the sentence explaining what a per-bump
   test is for.
2. **Return `Play` instead of `Freeze`.** The timeline test fails; and if a
   core audio test is added later it would fail there too, which is the
   dependency to note rather than to build now.

---

## 2. Media: the picture is a decoder *or* a still

**Files.** `crates/pundit-media/src/composite/decode.rs`, `composite/mod.rs`,
`composite/export.rs`, `composite/preview.rs`, `crates/pundit-media/tests/`,
`CLAUDE.md`.

1. **`Texture`'s home is `composite/decode.rs`, not `mod.rs`** — `Decoder` is
   in `decode.rs:48`, so "beside `Decoder`" (the spec's words and the draft's)
   names the wrong file, and `decode.rs` is already what both tails import the
   picture side from.
   - **And decide the move on the merits, because the draft's reason is
     false**: *"both tails now need it"* assumed preview would use `Texture`, and
     preview's still path is `AvatarInset` — a **system-memory** buffer
     re-stamped per frame through its pad's own `glupload`, documented as needing
     no upload of its own *because a preview is one clip*. A whiteboard preview
     is exactly that case. So either `Texture` becomes the crate's one "a still
     on a mixer pad" type — and then **say whether `AvatarInset` should become
     one too**, or the crate ends with two patterns for one job, the state the
     move was meant to fix — or preview takes the still its own way and
     `Texture` stays put. **Deferred to the coach** at the end.
2. **There is no pixel preparation** (§B.4). `Still` is already straight-alpha
   tightly packed RGBA and `Texture::upload` takes exactly that, so it is
   `Texture::upload(gl, watch, still.w, still.h, still.rgba)` — no new function,
   no premultiply question anywhere in the feature.
3. **If a pre-scale earns its place it is `videoscale` + a caps filter in the
   still's own decode**, so GStreamer scales in straight-alpha RGBA and the
   bytes reach `Texture::upload` ready. **Measure before adding it**: the claim
   is that the mixer resampling a 4000×3000 texture per frame costs more than
   footage does, and nothing has measured that. **Note that three sizes want
   it** — the UI's, preview's 1280×720 and export's 1920×1080 — so if there is
   one preparation function it takes a size, which the draft did not say.
4. **The picture is one value, not a field plus a convention** (§B):
   `enum Picture { Footage(PathBuf), Backdrop(PathBuf) }` on `EntryMedia` and on
   `PreviewJob`, built **once** in `bus/export.rs::entry_media` (`:1103`, which
   already holds the project and the entry and already finds the clip) and in
   `bus/preview.rs`. `EntryMedia.source` keeping its type while
   `clip.clip.backdrop` separately says *whether* it is one is two values that
   can disagree, and the way they disagree is media opening a `Decoder` on a PNG
   or a `Texture` on an MP4. **`preview::InsetSource` is the codebase's own
   answer to this shape** — *"One value rather than two flags: the three cases
   are exclusive, and the pad is requested, placed, fed and linked from this one
   answer."*
5. **The export loop's two arms, with the ownership the draft hid** (§B.11).
   `Frame<'a>` borrows its `&gst::Sample` out of the decoder map and
   `Texture::stamped` returns owned values, so the picture path builds **one
   owned `Sample` per texture** and lets `Encoder::push` do the stamping it
   already does — `stamped(n)` would stamp twice and stays the inset pad's.
   Downstream nothing changes shape: `source_caps`, `fit_rect`, `set_caps`.
6. **Preview takes the same two arms** at its one `Decoder::start`. Preview is
   one clip, so there is **no mid-run caps change there at all**.
7. **The board and the highlights are omitted per entry, at the draw** — never
   by blanking `MatchMedia`, which is `Arc`-shared across a project's entries, so
   blanking it would cost the footage clips in the same export their board
   (§R.6). **The per-entry place exists in both tails** and was checked:
   `export.rs:664`–`:676` reads `media.match_media` with `media.clip` in hand,
   and preview has `job.clip`.
8. **Two docs to amend, one to leave alone.** `Encode::entries`' *"every entry
   has a game video"* becomes *"a picture"*; `EntryMedia`'s picture field says
   what it now is. **`install_overlay_pad`'s rule is untouched** — nothing is
   ever mixed over the overlay, and a whiteboard changes only pad 0, so the
   coach's pen is still the top layer, which on a whiteboard is the whole point.

### Tests

- **An export of one whiteboard clip** decodes back to a file whose frames are
  the image, letterboxed to its own aspect inside 1920×1080 — **the assertion is
  the picture rect, read off the bars**, which is `fit_rect`'s answer and
  checkable without a pixel hash. A **4:3** diagram, so the pillarbox is a real
  measurement.
- **A mixed compilation in both orders**, whiteboard-then-footage and
  footage-then-whiteboard: each must run to EOS with both entries' frames
  present. Both orders are one extra `PlanEntry`, so the draft's *"if only one
  is covered, say which and why"* was an escape hatch for nothing.
- **The mixed test also asserts the footage entry still carries its
  scoreboard**, pinning §R.6 — a correction the spec made, which the draft left
  to a sabotage proof to *discover*.

### Sabotage proof

1. **Push the still as system memory rather than GL.** The mixed test must fail
   at `glupload` with *"Failed to upload buffer"* — the failure `Texture`'s doc
   records, and why the upload is not optional.
2. **Blank the board on `MatchMedia`.** The mixed test's footage entry loses
   its board — now an assertion rather than a hope.
3. **Hand `Picture::Backdrop` to the decoder arm.** The export test must fail
   at the decode rather than producing a wrong-looking file: the two-values
   hazard §B removes, confirmed removed.

---

## 3. The bus: arm it, record over it, preview it, refuse the rest

**Files.** `crates/pundit-app/src/bus/state.rs`, `bus/mod.rs`,
`bus/recording.rs`, `bus/export.rs`, **`bus/preview.rs`** (§B.3),
`crates/pundit-harness/tests/`, `CLAUDE.md`. **Not `bus/drawing.rs`**, which the
draft named and **does not exist**; highlights are `bus/highlights.rs` and the
refusal is not there either (step 7).

1. **The armed diagram is a path in `state.json`**, through that file's
   per-field lenient read (#100), so an unreadable value costs the arm and
   nothing else. It is "the file on this machine I am drawing on today" — the
   test that file states, and the one that sent the whisper model there.
2. **The pick is validated with `decode_still` at pick time**, as `set_avatar`
   does (`project.rs:396`–`:413`), so a file that will not decode is refused
   before anything is armed.
3. **The copy happens at take start, above the seek** (§B.6). Into
   `<project>/backdrops/<clip id>.<ext>`, through a **per-clip** temp file
   (`.<id>.tmp`, or two takes race one path) and a rename in the same directory,
   with the temp removed on failure as `set_avatar` does. **`create_dir`, not
   `create_dir_all`** — one leaf under a folder that exists, by the new-match
   flow's own argument (§B.8).
   - **Above the seek** means minting `let clip_id = Uuid::new_v4();` as its own
     binding in `capture_sources`' nothing-has-changed-yet region, with
     `PendingClip { id: clip_id, … }` still built after `heading(None)` —
     `source_index` and `start_source_seconds` must be read after the seek and
     cannot move. **Widen `:125`'s comment** to say the backdrop copy joins that
     group, and leave `create_dir_all` where it is: moving *that* is a change to
     the recording path with no feature behind it.
4. **`Active` gains the file name and `finish_recording` assigns
   `clip.backdrop`**, beside `clip.slate_id = active.slate` (§B.7) —
   `PendingClip` is `Copy` over three scalars and a `String` would cost it that.
5. **Three paths remove the file, not one** (§B.12): `abort_recording`, and
   **`Recorder::start` failing**, which returns without reaching
   `abort_recording` and would otherwise leave an orphan. Nothing else ever
   deletes it — a backdrop is left where it is rather than gaining a second
   trash path to keep in step with the first, which is also what makes
   delete-then-undo work with no code.
6. **`bus/preview.rs` reads the backdrop once** (§B.3): when it is `Some`, skip
   **both** game-video refusals, point the job's picture at
   `<project>/backdrops/<name>` and stat **that**. Without this a whiteboard
   preview renders the game video, and is refused outright when that video is
   missing — the "refusal with no remedy" the spec rejects for `entry_media`,
   left standing one layer up.
7. **`entry_media` stats and refuses the image** for a whiteboard entry, for the
   same reason.
8. **The refusals are one clause in the allow-list that already exists**, not
   five guards in four files.
   - **Scan, step and scrub need nothing**: `ScanSpeed`, `StepFrame`, `ScrubTo`
     and `ScrubRelease` are **not on the allow-list**, so they are already
     refused for *every* take. W7's row is about the **keys**, not four
     commands, and the draft inherited the overstatement.
   - **What the whiteboard adds is six exclusions**, all already named in that
     one `matches!`: `TogglePlay`, `Skip`, `TagMatchEvent`, `MarkSlateIn`,
     `MarkSlateOut`, `SetHighlightKey`. One clause — "a whiteboard take's
     allow-list is the recording one minus these six" — keeps the property that
     guard advertises (*"everything not listed is refused, so commands added
     later are too"*), which five scattered guards throw away.
   - **The highlight is the one that matters most**: its failure is silent and
     *remote* — a ring keyed to footage that is not on screen, appearing over the
     **game video** somewhere else entirely.
9. **One `UserError::Whiteboard(String)`, listed in `is_notice`** (§B.10). A
   variant added to the enum and not to that hand-written `matches!` renders as
   a modal, which `::Scoreboard`'s and `::Slate`'s docs say is unacceptable
   because it could land over a live take and swallow the transport keys. **A
   test asserts `is_notice()`**, which is the only thing that can catch the
   missing arm.
10. **Nothing is refused that costs a guard to refuse.** Pen strokes,
    clear-all, **zoom**, the inset, the caption bar, preview, transcription and a
    basket piece are all allowed and unchanged — zoom especially, where
    `install_zoom`'s probe drives `gltransformation` on pad 0 whatever it
    carries, so allowing it costs no code and refusing it would cost a guard.

### Tests

Harness, `CaptureKind::Test` throughout:

- **A take end to end**: a clip whose `backdrop` is set, whose file is under
  `backdrops/`, and whose `source_index` is where the player was.
- **Nothing armed is a notice and records nothing**, and the notice
  `is_notice()`.
- **An aborted take leaves no file in `backdrops/`**, and **a recorder that
  fails to start leaves none either** (§B.12).
- **A copy failure refuses the take and leaves the player as it was** — true
  only because of §B.6, so **assert the player's position**, not just that no
  clip was written. The position is the half the obvious placement gets wrong.
- **A whiteboard clip previews while its anchored source is missing** (§B.3) —
  the refusal with no remedy, as a test.
- **One test per refusing group**, the highlight one carrying its reason in the
  doc comment.
- **W10's item 12**, which the spec asked the plan to confirm and the draft
  dropped: jump-to-clip-start and double-click-to-the-in-point on a whiteboard
  clip. The spec says *"the plan must confirm that, not assume it"*. **The other
  twelve items are "nothing changes" and are already pinned** —
  `source_is_referenced`, `remove_source`'s remap and
  `purge_for_source_change`'s exhaustive match all key off `source_index`, which
  W2 keeps honest, and all three were verified — so they are checked off in the
  PR body rather than made a thirteen-row checklist.

### Sabotage proof

1. **Move the copy to `finish_recording`.** With the copy made to fail, the
   take must produce a clip with `backdrop: None` rather than refusing — the
   silent failure §R.4 is about, as an assertion.
2. **Move the copy down beside `create_dir_all`.** The copy-failure test's
   **position** assertion must fail and its "no clip written" half must still
   pass, which is what says §B.6 is pinned rather than argued.
3. **Leave `Whiteboard` out of `is_notice`.** The `is_notice()` assertion
   fails. Nothing else can catch it, which is why it is its own test.
4. **Put the six exclusions back one at a time.** Each must fail its own
   group's test and nothing else — which says the single clause is six
   independent exclusions rather than one blunt gate.
5. **Skip the backdrop in `bus/preview.rs`.** The preview test fails on the
   refusal, not on the pixels — the bus half and the media half are pinned
   apart.

---

## 4. The window shows the diagram during the take

**Files.** `crates/pundit-app/src/video.rs`, `src/main.rs`,
`crates/pundit-app/ui/app.slint`, `crates/pundit-app/tests/ui/`, `CLAUDE.md`.

**The spec asked for this (W5) and the draft dropped it entirely** (§B.2). It is
not cosmetic — it is what the feature's correctness rests on, and W10's item 5
makes *"the popover, the live corner and the export cannot disagree"* a
checklist line.

1. **The player draws the armed diagram for the life of a whiteboard take**,
   from `decode_still`'s pixels into a `slint::Image` — `Still.rgba` is
   straight-alpha RGBA, which is what `SharedPixelBuffer<Rgba8Pixel>` wants, and
   both `decode_still` and `Still` are already `pub` from `pundit-media`.
   **Once per change of the armed file**, cached — never
   `slint::Image::load_from_path` (slint is built with no image decoder) and
   never a decode on the UI thread at take start, the rule `CLAUDE.md` states
   for the avatar corner. **Say where the decoded copy is cached.**
2. **`frame-width` and `frame-height` come from the diagram too**, and this is
   the half the spec did not spell out. They are set in `video.rs:160` from the
   decoded game frame's mapped width × PAR beside `set_frame`; `content` is
   `place-picture(identity, frame-width, frame-height, …)`; **strokes are
   normalized against that rect**. Three values, one source.
3. **`layout::self_view_rect` / `avatar_self_view_rect` place the inset over
   that rect unchanged**, and **the self-view's quiet timer stays the camera's
   alone**, as it already must be.

### Tests

`tests/ui/`, which can set the armed image as a window property — the harness
has no window and cannot reach this at all:

- with a whiteboard take armed and running, `frame-width`/`frame-height` are
  the **diagram's**, and the `content` rect that follows is the diagram's
  letterbox rather than the footage's;
- **a stroke's normalized coordinates against a 4:3 diagram differ from the
  same pointer position against a 16:9 source** — the assertion that makes step
  2 mean something rather than restating step 1;
- the inset still lands in `self_view_rect` over the new picture rect.

### Sabotage proof

1. **Leave `frame-width` on the game frame's shape** and draw only the image.
   The stroke-coordinate test must fail while the picture test passes — which
   says the two are separate claims, and is the exact bug this task prevents.
2. **Decode the still on the UI thread at take start.** Nothing fails, and
   that is the finding: the rule is stated in `CLAUDE.md` for the avatar corner
   and nothing enforces it for either. Record it rather than inventing a timing
   assertion.

---

## 5. The UI: the popover and `W`

**Files.** `crates/pundit-app/ui/app.slint`, `src/main.rs`, `src/keymap.rs`,
`src/key_action.rs`, `crates/pundit-app/tests/ui/`, `CLAUDE.md`, `BACKLOG.md`.
**Re-derive `app.slint`'s line numbers at the task** — four queued features edit
that file.

1. **A popover to arm a diagram**, on the shape Devices has for the avatar: a
   thumbnail of what is armed, a pick, a clear. Same decode-at-pick-time path,
   so the thumbnail is proof the file decoded.
2. **`W` is `Command::ToggleRecording { slate, backdrop }`, not a new command**
   (§B.9). A new variant would be off the allow-list, so **`W` could not stop the
   take it started** — refused with a bare `eprintln!`, breaking the R/W
   symmetry `toggle_recording`'s doc argues for. **And `W` with a slate selected
   is incoherent** — a slate is a *footage* range — so that arm refuses with the
   notice rather than being left undefined.
3. **A new `keymap::Action`, and `w` is free** (§A) so it displaces nothing.
   **It is more than "the three edits #96 promised"** (§B): the variant, **`ALL`'s
   array *and* its `[Action; 29]` length**, four match arms, the `KeyAction` arm,
   the `handle-key` branch, a `cases()` row in `keys.rs` — plus
   **`the_defaults_are_conflict_free`'s hard `assert_eq!(seen.len(), 34)`, which
   fails on the happy path** the moment the action is added, and five stale
   prose counts (`keymap.rs`, `key_action.rs`, `tests/ui/keys.rs`,
   `keys_sheet.rs`'s header, `CLAUDE.md`'s *"29 actions, 34 bindings"*).
   `keys_sheet.rs`'s assertions are safe — they read `Action::ALL.len()`.
   **This is the first new action since the table landed, so it is also the
   honest measurement of what one costs**, and that belongs in #96's record.
4. **`W` with nothing armed is a notice, not a modal** — `UserError::Avatar`'s
   stated reason: it refuses one click in a popover the coach still has open,
   and the next pick is the answer.

### Tests

- the popover's pick and clear are wired, and the thumbnail follows what is
  armed;
- **`W` reaches the record callback through the real `wire_keys`** — the
  `cases()` row;
- `W` with nothing armed fires **no** record and leaves a notice.

### Sabotage proof

1. **Leave the `handle-key` branch out** while keeping the `keymap` variant and
   the `KeyAction` arm. The test that catches it is
   **`every_default_binding_still_does_what_it_did`**, which presses the key —
   **not** `covers_every_action`, which the draft named and which only checks
   that a row *exists*. This is the pairing Slint cannot check, and the one proof
   that #96's machinery works for a *new* action and not only the 29 it shipped
   with.
2. **Give the new action `r`.** `the_defaults_are_conflict_free` must fail —
   but **it fails either way while the `34` literal is being updated in the same
   commit**, so run it *after* the count edit or the proof means nothing. Say
   which order it was run in.

---

## Risks

1. **The stroke normalisation is the thing that can ship looking nearly
   right.** Task 4 step 2 is the whole of it, and its sabotage proof is the only
   thing between a working whiteboard and one where the coach's drawings land
   somewhere else in the export. *(The draft's Risk 1 was the premultiply; §B.4
   removed it from the feature.)*
2. **`app.slint` is contended** (#78's settings sheet, #84, #102, #115). Task 5
   is the only one of these five tasks that touches it.
3. **The format bump collides with #115's.** Both want the next version; if
   they land together they share one bump, one version and one per-bump test.
   Whichever is second **checks `store.rs`, not this plan**.
4. **`source_is_referenced` will now pin a game video because of a clip that
   shows none of it.** Correct and needing no new arm — but a coach who wants to
   remove a half gets *"still used by a clip…"* because a whiteboard explanation
   was filed against it, and the only way out is deleting a clip whose picture is
   unaffected. **Accepted**; the better long-term answer is wording that refusal
   to say *which* clips, which is its own small change.
5. **An orphaned backdrop is accepted by design** — a deleted clip's diagram is
   left on disk, and an **evicted** undo entry is a second source of them. A few
   hundred kilobytes, named by a uuid so it can never be mistaken for the
   coach's own file, and it is what makes delete-then-undo work with no code. If
   it ever looks wrong, the fix is **not** a sweep of `backdrops/` (§R.5).

## What this does not do

- **No library of diagrams** — one armed at a time, the coach's own call
  (§W12). A library can be added later without changing `Clip::backdrop`'s
  meaning, because a clip stores its **own copy** of the image it was recorded
  over.
- **No title card** — a fixed-length still with no commentary is #133.
- **No changing a clip's diagram.** The commentary was recorded against that
  image; swapping it would leave the coach pointing at things that are no longer
  there, the same reason a clip cannot be re-anchored to other footage.
- **No scoreboard and no cues for a whiteboard's span**, and no `Preferences`
  field.
- **The pick-time decode is not the guarantee W1 sells.** *"A diagram cannot
  fail twenty minutes into a take"* is true of a file **deleted** between the
  pick and the take — the copy fails and the take is refused — and **false** of
  one *replaced* with something undecodable, which records fine and fails at
  export. Rare enough to leave; stated so the validation is not sold as more
  than it is. Note that `set_avatar`, the pattern being copied, decodes and
  copies **adjacently** — the whiteboard splits them by minutes, and that is
  where the resemblance stops.

## For the coach

**Two questions, neither blocking — task 1 can start either way.**

1. **When a whiteboard take refuses the play key, should it say so?** Every
   other command refused while recording logs to stderr and relies on the
   control being greyed, on the stated grounds that reaching the refusal is a UI
   bug. The spec asked for notices. Notices are a new error path and the greying
   has no home in this plan; the convention is free and silent. **Going with one
   notice for the group unless you say otherwise** — because unlike the others,
   a whiteboard take's keys are *not* greyed by anything today, so silence here
   would be a key that does nothing with no explanation.
2. **`AvatarInset` uses a system-memory buffer where the whiteboard will use a
   GL `Texture`** — two patterns for one job ("a still on a mixer pad"). Unify
   while in there, or leave? **Leaving it unless you say otherwise**: the
   avatar's path is measured and working, and a refactor with no feature behind
   it is the kind this plan has twice removed from itself.

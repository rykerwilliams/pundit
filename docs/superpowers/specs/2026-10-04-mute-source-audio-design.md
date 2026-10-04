# Design — muting the source audio in an export

The coach (2026-10-04): *"we need 'mute source audio' as an export option eh"*,
after finding a basket film where one piece carried the game's sound and another
did not.

Read `CLAUDE.md` first. This spec does not repeat the export, basket or format
rules it states. **Revised after two adversarial passes**, which found two
blocking faults in the first draft and corrected seven claims; §R keeps the
record.

---

## Why this exists, and the bug report it came out of

The coach reported two things, and **neither was the defect he thought it was**.
Both are worth writing down, because the second one is why this feature is cheap.

1. *"when i recorded a clip with zero volume, and preview it, the source audio
   is silenced."* — **A preview has never carried source audio at all.**
   `composite/preview.rs`'s only audio branch is
   `queue ! audioconvert ! audioresample ! volume ! autoaudiosink`, fed from the
   **recording's** `decodebin3` alone; the pumped source arrives as GL buffers
   through `composite/decode.rs`, which links *"the video stream only. Other
   pads (audio) stay unlinked"*. So this is true of every clip ever previewed.
   It is BACKLOG **#125**.
2. *"only the first one is muted; the second clip has the source audio."* —
   **Not a gain.** Game regions are built only from `Play` segments
   (`core::audio::game_regions`); a freeze contributes nothing, deliberately.
   **The only remaining explanation** — the gain having been ruled out below — is
   that the first clip's footage never ran: `R` pressed, space not, which is
   BACKLOG #110's confirmed behaviour wearing a different hat. **Nobody has
   inspected those two clips' segments**, so this is the surviving hypothesis
   and not an established fact. (Mic bleed-through was considered and ruled out
   by the coach: he monitors on a Bluetooth headset, so the room carries no game
   sound into the commentary track.)

**The knob already exists and is unreachable**, which is the real finding.
`audio_regions` applies `prefs.preview_source_volume` as the gain on every game
region of every encoded export, and **neither `preview_source_volume` nor
`preview_commentary_volume` has a writer or a UI control anywhere in production
code** — the only assignments are in `pundit-core/tests/audio.rs`. Both are
permanently `1.0`. Two consequences:

- The coach could not have set a source volume to zero even deliberately, so
  what he saw was never a volume.
- The basket's deliberate "mix at the **default** volumes, not the open
  project's" (spec J6) is **currently a no-op**, because the defaults are the
  only reachable values. The reasoning stays right; it has no teeth yet.

**This spec does not revive those preferences.** A remembered *slider* over
source gain is a different feature with the failure mode J6 exists to refuse — a
film quiet for an invisible reason. What the coach asked for is a switch. The
vestigial preferences are BACKLOG **#124**, and this feature is what would make
deleting `preview_source_volume` possible, since the switch becomes the one
reachable version of "make the game quieter".

---

## M1. What it is: a switch in the export sheet, remembered

A fourth control beside Resolution, Quality and Scoreboard, remembered like
those three (`Preferences::last_export_*`).

- **It mutes the game video's track only.** The commentary is untouched — that is
  the point of the feature.
- **It is a switch, not a level.** A level is #124's argument to have.
- **Wording:** the UI says **source audio**, because that is what the codebase
  calls the game video throughout (`EntryMedia::source`, `source_index`).
- **What the coach rejected was a separate *control*, not project storage.** He
  chose "export sheet, remembered" over "a project setting", meaning he wanted the
  switch in the sheet rather than behind #78's settings screen. Remembering it in
  `Preferences` **is** project storage (M5) — the two are not in tension, and the
  first draft's wording implied they were.

## M2. Each renderer expresses it in its own vocabulary, and the mute reaches the copy's *gate*

This is the `carry_scoreboard` pattern. One function maps the coach's choice into
each renderer's own terms; neither carries a flag the other reads.

**Encoded exports: no game regions at all.**
`core::audio::audio_regions` omits them, so there is nothing to express
downstream — `Mixer::new` builds `paths` *from the regions*, so no region means
no path, no `Reader` and no decode. **Not a gain of `0.0`**, which would decode
every sample to multiply it away and hold a reader open per source file for
nothing. An empty region list is already a handled, exercised case (`Encode::audio`'s
doc: *"Empty is a silent track, which is still a track"*, plus ~10 media tests
built with `audio: Vec::new()`).

**How it gets there: a parameter, never a field core reads.**
`audio_regions(&compilation, &prefs, mute_source: bool)`.

- **Core must not read `last_export_mute_source`.** No `last_export_*` field is
  read by core today — resolution, quality and scoreboard all flow sheet →
  `Pickers` → job, and core only stores them. Making the mute the first
  exception would break the pattern this spec claims to follow.
- **And the basket makes it a trap.** `bus/basket.rs` calls
  `audio_regions(&compilation, &Preferences::default())` **on purpose**, under a
  comment citing J6. If core read the field, a muted basket would have to
  synthesise `Preferences { last_export_mute_source: mute, ..Default::default() }`
  — a value that is no longer the default, at the one call site whose whole point
  is that it is untouched. With a parameter, `bus/export.rs::job` passes
  `prefs.last_export_mute_source`, the basket passes its own field, and J6's
  comment stays literally true.

**The stream copy: the audio track is not copied.** The coach's own answer, and
the better one — *"you can just copy the video stream"*. No re-encode.

- **`Render::Copy` gains the flag in the copy's vocabulary:**
  `Render::Copy { files: Vec<PathBuf>, audio: bool }`. The first draft said
  "one more condition" without saying where it could live; the tuple variant
  carried nothing but the files, so there was nowhere.
- **The mute must reach `declare`, not just the audio branch** — see M6. This is
  the first draft's second blocking fault.
- **It is the same *branch*, not the same path.** A silent source has no audio
  pad at all; a muted copy leaves a linked demuxer's audio pad **unlinked** and
  relies on `qtdemux`'s flow combiner tolerating `NOT_LINKED`. Measured in
  review: `filesrc ! qtdemux name=d d.video_0 ! h264parse ! mp4mux ! filesink`
  runs to EOS cleanly (exit 0) and yields one `h264` stream and no audio. The
  module's header is careful about this class of claim, so the precision matters.

## M3. The basket gets the same switch

A basket is clips from several matches, so commentary-only is its natural film,
and the coach's own trigger was a basket. It joins `basket.json` beside the name,
resolution and quality. A basket always encodes, so only M2's first half applies.

**Stored as a plain `bool`, not a string label.** The string-label rule exists
because an *enum* can gain variants an older build cannot read; a `bool` has none
to gain. **Not** because of a lenient per-field read — `basket.json` has none:
`read` is `from_str::<Stored>(&text).unwrap_or_else(|_| Stored::default())`, the
**whole document**, and its own doc says why (*"a file this build can't parse says
nothing trustworthy about any of its fields"*). So a malformed value there costs
the pieces, the name and both pickers. That is acceptable because no build of
this app writes a non-bool into it, and `#[serde(default)]` already covers an
absent key. **Do not add machinery for it:** `Option<bool>` buys nothing that
`#[serde(default)]` does not.

## M4. What it does not change

- **Preview.** A preview has no source audio to mute, so the switch is invisible
  there — with a pleasant consequence: **a muted export finally sounds like its
  preview**, where today a preview is always quieter than the file it predicts.
  That mismatch is #125 and this spec does not fix it.
- **Scanning.** The transport slider is `scan_volume`, the player's own output.
  It has never reached an export.
- **The commentary.** Muting touches one track. Anything already *inside* the
  commentary recording stays — not a live issue for this coach (he monitors on a
  headset) but the feature's honest boundary.
- **A freeze.** Still silent, muted or not. So this is **not** the fix for the
  coach's original report, and §R.3 says so rather than letting the entry close
  on a false claim.

## M5. The format: the next free version, which is contested

`Preferences::last_export_mute_source: bool`.

- **Take the next free `formatVersion`, and do not hard-code it here.** v14 is
  #117's arrowhead `StrokeEnd`, and **BACKLOG #115 already claims v15** for the
  caption-bar switch — also a `Preferences`-homed export-sheet checkbox. Only
  one can be v15 and #115 wrote it down first. **If the two land together they
  should share one bump**, which is cheaper and honest: one version, two fields,
  one every-readable-version test.
- **Why `Preferences` and not `state.json`.** #78 records the governing
  principle (*"a field added to `Preferences` is a `formatVersion` bump every
  time, which is why the whisper model picker went to `state.json`"*), and #115
  weighs this exact choice. It belongs in `Preferences` on the test
  `bus/state.rs` states — *"the reason a setting lives here rather than in
  `project.json` is **what it describes**"*. "Mute the game audio" is a property
  of the **match** (a half recorded in wind wants it every time), not of this
  machine's speed, which is what sent the whisper model to `state.json`. It is
  also the fourth picker of one sheet row, written by the write-back the other
  three already use and outside undo with them; a different file would mean
  different machinery and `Pickers` would stop being the sheet's state.
  **Note for the next reader:** `CLAUDE.md`'s *"a preference is never a format
  change"* sits in the macOS-conventions section and is already contradicted by
  `last_export_scoreboard` (v11) and `last_inset_*` (v13). The spec is not
  missing it.
- **No field-level `#[serde(default)]`.** The live reason is that `Preferences`
  carries a **container** default filling from its hand-written `Default`, so a
  field-level one would be a second copy. `CLAUDE.md`'s "never a field-level
  default on a `bool`" hazard does **not** bite here — `false` is exactly what an
  older file means. Worth one line, because
  `project_format.rs::preferences_defaults_are_not_zero` is the test that guards
  that hazard and **cannot** cover a field whose correct default *is* the zero
  value. This would be the first `Preferences` field of which that is true.
- **The backup is named from the project's own stored version**, not from
  `CURRENT - 1`: `store::write` uses `format!("{PROJECT_FILENAME}.v{}",
  project.format_version)`, so a project still at v12 gets `project.json.v12`.

## M6. Muting *removes* refusals rather than adding any

The first draft said "nowhere is it refused" and that was **false**. `declare` is
both the copy's gate *and* its rate reader, and four of its refusals are about
audio:

- `absolutely`: *"{name}'s sound isn't AAC, so it can't be copied"*
- `agree`: `first.audio.is_some() != header.audio.is_some()` — the two-halves case
- `agree`: `!renegotiable(&first.audio, &header.audio)` — caps disagreement
- `audio_rate`: an unreadable sample rate

Forcing `audio_rate = None` *after* `declare` leaves all four live, and
**`can_copy` is `declare`** (`copy.rs:171`). So:

- **Separate track + mute** would refuse, naming a file, over audio the coach
  just asked to drop — and the way out it offers spends an hour re-encoding.
- **Default + mute** would call `can_copy`, fail for the same reason, log *"the
  whole match can't be copied"* and re-encode a file that could have been copied
  in seconds. The coach never sees why.

**So the mute reaches the gate:** `declare(files, audio, watch)` skips the three
audio checks and returns `Ok(None)` when muted, and `can_copy(files, audio)`
takes it so the bus asks the right question.

**This can only widen what can be copied.** The gate loses checks and gains none,
so muting cannot make a copy fail that previously succeeded, and it legitimately
makes copyable a project that was not — two halves differing only in sound. That
is the answer to "could this change a refusal", and it is a better story than the
first draft's.

**Two consequences to state rather than fix:**

- **A muted whole match or reel is entirely silent.** Neither target has clips,
  so `audio_regions` gives them no commentary region either. That is a different
  proposition from a muted clip, which keeps the commentary that is the point of
  the feature.
- **The two renderers produce different files for the same request.** A muted
  whole match is a video-only MP4 when it copies and an MP4 with a **silent AAC
  track** when it encodes, decided by `can_copy`, which the coach never sees.
  Not worth equalising — teaching the encode path to drop its audio pad is new
  code for an invisible difference — but it sits next to "Default means the best
  available, never a silent trade", so it is named here on purpose.

**One quiet line in the sheet, for the silent case only.** The sheet already
spends exactly one grey line per trade. Where the result will carry **no sound at
all** (a muted whole match or reel), that line says so. A muted *clip* gets no
warning: it keeps the commentary, and a silent film is a legitimate thing to want
— footage to lay music over — so this is a telling, never a refusal. *(The coach
may veto the line; the refusal decision above does not depend on it.)*

## M7. Tests — three groups, not six

1. **Core (`core/tests/audio.rs`).** With the switch on: every region's track is
   `Commentary`, and the commentary regions are **byte-identical** to the unmuted
   run's — that second half is what pins "muting touches one track", and it is
   sound because `region`'s priming drop is computed per region from the entry's
   own frames, so entry 0's commentary loses the same 1024 samples either way.
   Same test asserts a **reel** entry (no clip) comes out with no regions at all;
   the mixer's silence rule is already pinned by ~10 existing `audio: Vec::new()`
   jobs and needs no test of its own.
2. **Media.** A muted encode decodes back to the commentary alone, extending
   **`the_game_track_is_gated_to_play_and_the_mix_fades_in`**
   (`media/tests/export.rs`), which already has `GAME = 1000.0`, `MIC = 300.0`
   and a single-bin DFT — exactly "the game's tone absent, the commentary's
   present". *(Not `a_tone_lands_where_the_picture_does`, which the first draft
   named: it has a tone in the source only, with no commentary to contrast.)*
   And a muted copy, in `media/tests/copy.rs`: **no audio stream**, plus the
   packet list and `video_stream()` tuple equal to the unmuted copy's — "packet
   for packet", the copy module's own phrase. `ffprobe` **cannot hash a
   bitstream**, so bit-identity is not the assertion; review measured that the
   video elementary stream *is* md5-identical and all 90 packets match, but
   proving it needs an `ffmpeg` helper `fixtures.rs` does not have. Add one only
   if the stronger claim is wanted.
3. **Harness + format.** The sheet's choice reaches
   `Preferences::last_export_mute_source` by the existing write-back, and a
   refused run does not dirty it — two assertions added to
   `export.rs`'s existing write-back and `a_refused_run_leaves_the_pickers_alone`
   tests, not a group of their own. The bus-level muted copy belongs in
   `harness/tests/whole_match.rs`, since the mute arrives through
   `Command::Export`. And **`a_v14_file_loads_under_the_current_version`** beside
   its siblings — the series is named for the *old* file version, there is no
   `v7_to_v14` test, and that per-bump test is the only thing pinning the
   constant.

## §R. What the first draft got wrong, and what review corrected

Two blocking faults, found independently by both reviewers:

1. **It had core read the `Preferences` field.** `audio_regions(compilation,
   prefs)` takes no other argument, so the only reading was that core consults
   `last_export_mute_source` — the first `last_export_*` core would ever read,
   and it would have forced the basket to fake a non-default `Preferences` at the
   one site whose comment says it is untouched. Now a parameter (M2).
2. **It designed the copy half as "no refusal anywhere" and that was false.**
   Four audio refusals in `declare` survive a post-hoc `audio_rate = None`, and
   `can_copy` *is* `declare`, so Default mode would have silently re-encoded over
   sound the mute discards. The mute reaches the gate, and muting *widens* what
   can be copied (M6).

Earlier errors, kept:

3. **It designed a refusal for the copy path at all**, assuming a stream copy
   cannot mute its audio, and offered the coach three bad choices. His answer —
   copy the video stream, leave the audio — needs no refusal. The lesson: *"a
   copy cannot change X"* is not the claim *"a copy must carry X"*.
4. **It proposed implementing the mute as `preview_source_volume = 0.0`**, which
   would decode every game sample to multiply it by zero and would quietly revive
   a preference J6 overrides.
5. **It read the coach's bug report as the thing to fix.** Both symptoms are
   correct behaviour. The feature is worth building on its own merits; M4 says it
   does not fix what he saw.

Claims review corrected:

6. **v15 was already claimed by #115**, and the spec engaged with neither #115 nor
   #78, which record this exact decision. M5 now takes "the next free version"
   and proposes sharing a bump.
7. **`basket.json` has no lenient per-field read** — the whole document is
   discarded on a parse error. M3's reasoning was right; its justification was not.
8. **M7 named a test that does not exist** (`v7_to_v14`) and the wrong media
   lever, and claimed an `ffprobe` assertion it cannot make.
9. **M1 contradicted M5** on whether a remembered switch is "a project setting".
10. **The `.v<old>` backup** is named from the project's own version.
11. **The bool default hazard does not bite**, and this would be the first
    `Preferences` field whose correct default is the zero value — which
    `preferences_defaults_are_not_zero` cannot cover.

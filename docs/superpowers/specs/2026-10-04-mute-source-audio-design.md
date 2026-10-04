# Design — muting the source audio in an export

The coach (2026-10-04): *"we need 'mute source audio' as an export option eh"*,
after finding a basket film where one piece carried the game's sound and another
did not.

Read `CLAUDE.md` first. This spec does not repeat the export, basket or format
rules it states.

---

## Why this exists, and the bug report it came out of

The coach reported two things, and **neither was the defect he thought it was**.
Both are worth writing down, because the second one is the whole reason this
feature is cheap.

1. *"when i recorded a clip with zero volume, and preview it, the source audio
   is silenced."* — **A preview has never carried source audio at all.**
   `composite/preview.rs`'s graph is `recording → audioconvert → audioresample →
   volume → autoaudiosink`: the only sound in a preview is the commentary. So
   this is true of every clip ever previewed, not of zero-volume ones.
2. *"only the first one is muted; the second clip has the source audio."* —
   **Not a gain.** Game audio regions are built only from `Play` segments
   (`core::audio::game_regions`); a freeze contributes nothing, deliberately. A
   take whose footage never ran — `R` pressed, space not — is one long freeze and
   so has no game audio to carry. That is BACKLOG #110's confirmed behaviour
   wearing a different hat.

**The knob already exists and is unreachable**, which is the real finding.
`audio_regions` applies `prefs.preview_source_volume` as the gain on every game
region, and an export reads it — but **neither `preview_source_volume` nor
`preview_commentary_volume` has a writer or a UI control anywhere in production
code**. Both are permanently `1.0`. Two consequences:

- The coach could not have set a source volume to zero even deliberately, so
  what he saw was never a volume at all.
- The basket's deliberate "mix at the **default** volumes, not the open
  project's" (spec J6) is **currently a no-op**, because the defaults are the
  only reachable values. The reasoning stays right; it just does not bite yet.

**This spec does not revive those preferences.** A remembered *slider* over
source gain is a different feature with a different failure mode (a film quiet
for an invisible reason), and J6 exists to refuse exactly that. What the coach
asked for is a switch. The vestigial preferences are BACKLOG **#124**.

---

## M1. What it is: a switch in the export sheet, remembered

A fourth control beside Resolution, Quality and Scoreboard. The coach chose
"export sheet, remembered" over a project setting: a per-run choice that sticks,
which is exactly what the other three already do
(`Preferences::last_export_resolution` / `_quality` / `_scoreboard`).

- **It mutes the game video's sound only.** The commentary is untouched — that
  is the point of the feature, a film of the coach talking over quiet footage.
- **It is a switch, not a level.** Two states. A level is #124's argument to
  have, not this one's.
- **Wording:** the UI says **source audio**, because that is what the codebase
  calls the game video everywhere (`EntryMedia::source`, `preview_source_volume`,
  `source_index`). "Game audio" would be a second name for one thing.

## M2. Each renderer expresses it in its own terms, and neither carries a flag the other reads

This is the `carry_scoreboard` pattern (`CLAUDE.md`: track mode **blanks**
`job.scoreboard` rather than carrying a mode flag into media, "so there is no
third state to keep consistent"). One function maps the coach's choice into each
renderer's own vocabulary:

- **Encoded exports: no game regions at all.** `core::audio::audio_regions`
  omits them, so there is nothing to express downstream — `Mixer::new` maps no
  path for `Track::Game`, opens no `Reader` for it, and decodes none of it.
  **Not a gain of `0.0`**, which would decode every sample and multiply it away:
  the same output for real work, and a reader held open per source file for
  nothing.
- **The stream copy: the audio track is not copied.** The coach's own answer,
  and the better one — *"you can just copy the video stream"*. **No re-encode
  and no refusal**, which is what the first draft of this spec had wrong: it
  proposed refusing a muted copy, or silently encoding it, on the assumption
  that a copy cannot change its audio. It does not have to change it; it can
  decline to carry it.

**The machinery for the copy half already exists and is already exercised.**
`copy::audio_rate` returns `Option<u32>` — `None` for a source with no audio
track — and that `None` already gates the whole audio branch:
`Source::play(file, …, audio_rate.is_some(), …)`, whose `with_audio` doc says
"with no sound to carry there is no audio branch, and so no sink that can never
finish". Muting is **the same path a silent source already takes**, reached by
one more condition rather than new code. A muted whole-match copy is therefore
still a lossless video copy, and its output has no audio track at all.

## M3. The basket gets the same switch

The coach chose this, and it is where the feature is most wanted: a basket is
clips from several matches, so commentary-only is the natural film. It joins
`basket.json` beside the name, resolution and quality.

**Stored as a plain `bool`, not a string label.** `CLAUDE.md` records that the
basket's two *pickers* are string labels so an unknown value reads as the
default rather than costing the pieces — that reasoning is about an enum
gaining variants, and a `bool` has none to gain. The lenient per-field read that
`basket.json` already gets is what covers a malformed value.

A basket always encodes, so only M2's first half applies to it.

## M4. What it does not change

- **Preview.** A preview has no source audio to mute (the finding above), so the
  switch is invisible there. Worth stating because it has a pleasant
  consequence: **muting makes an export finally sound like its preview**, where
  today a preview is always quieter than the file it predicts. That mismatch is
  BACKLOG **#125**; this spec does not fix it and must not be read as fixing it.
- **Scanning.** The transport slider is `scan_volume` and is the player's own
  output. It has never reached an export and still does not.
- **The scoreboard, the `.srt`, the chapters, the tags.** Orthogonal.
- **A freeze.** Still silent, muted or not. Muting does not make a take whose
  footage never ran any quieter, which is why it is **not** the fix for the
  coach's original report.

## M5. The format: v15, and a `Preferences` field takes no attribute

`Preferences::last_export_mute_source: bool`. v14 is the arrowhead's
`StrokeEnd`, so this is **v15**; the readable floor stays 7.

- **No field-level `#[serde(default)]`.** `CLAUDE.md` is explicit twice over:
  never a field-level default on a `bool`, and `Preferences` already carries a
  container `default` that fills from its hand-written `Default` impl — a
  field-level one would be a second copy of the default.
- **`Default` is `false`.** An older file means "the export carried the source
  audio", because that is what every export before v15 did.
- **The bump is required even though serde ignores unknown keys**, for the
  reason `CLAUDE.md` gives: an older build would ignore the key and drop it on
  its next save. The bump makes it refuse the file instead, and the first save
  after the upgrade keeps `project.json.v14`.

## M6. Where the choice is refused, and where it is not

**Nowhere.** There is no combination to refuse, which is the strongest argument
that the coach's copy answer was right. Specifically:

- A muted whole-match **copy** is a lossless video-only MP4 (M2).
- A muted clip or reel re-encodes as it always did, with no game regions.
- A muted export of footage that has **no** audio track is indistinguishable
  from an unmuted one, and costs nothing to ask for.
- A muted export whose clips are all freezes is silent either way (M4).

A silent film is a legitimate thing to want — footage to lay music over — so
"this will have no sound at all" is not a refusal and not a warning.

## M7. Tests

- **Core:** `audio_regions` with the switch on yields regions whose tracks are
  all `Commentary`, and the commentary regions are **byte-identical** to the
  unmuted run's. That second half is what pins "muting touches one track".
- **Core:** an entry with no clip and the switch on yields **no regions at all**
  for that entry, and the mixer's silence rule covers it.
- **Media:** a muted encode decodes back to the commentary alone — the existing
  tone-at-1.000s fixture is the lever, asserting the game's tone is **absent**
  while the commentary's is present within a millisecond.
- **Media:** a muted copy produces a file with **no audio stream**, read with
  `ffprobe` (already a test-only build dependency, for the chapter tests), and
  the video stream is bit-identical to the unmuted copy's — the claim that
  muting costs the copy nothing.
- **Harness:** the sheet's choice reaches `Preferences::last_export_mute_source`
  by the same write-back as the other three, and a refused run does not dirty it.
- **Format:** every readable version still loads, beside the existing
  `v7_to_v14` test.

## §R. What the first draft of this spec got wrong

1. **It designed a refusal for the copy path**, on the assumption that a stream
   copy cannot mute its audio, and offered the coach three bad choices
   (refuse / silently encode / Default-encodes). The coach's answer — copy the
   video stream and leave the audio behind — is correct, needs no refusal, and
   lands on a branch the copy path already has for silent sources. The lesson:
   "a copy cannot change X" is not the same claim as "a copy must carry X".
2. **It proposed implementing the mute as `preview_source_volume = 0.0`**, which
   would have decoded every game sample and multiplied it by zero, and would
   have quietly revived a preference that J6 deliberately overrides in baskets.
   Omitting the regions is both cheaper and honest about intent.
3. **It read the coach's bug report as the thing to fix.** The report's two
   symptoms are both correct behaviour (a preview has no source audio; a freeze
   is silent). The feature he asked for is worth building on its own merits, but
   it does **not** fix what he saw, and M4 says so rather than letting the entry
   close on a false claim.

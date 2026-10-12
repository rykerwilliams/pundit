# Design — music under a goals reel (BACKLOG #84, phase 1)

The coach (2026-09-24): *"pull in soundtrack files that are licensed for use, e.g.
rock, edm, etc. for the goals clips."* — and the same day, *"pick genre"*: choose
rock or EDM on the export and the app does the rest. A goals reel is the one export
with no commentary (every entry is `clip_id: None`, `core/src/reel.rs`), so it is
the one people expect music under.

Read `CLAUDE.md` and BACKLOG #84 first. #84 records the decisions this rests on:
**the coach's own folder of music is the mixer's only input**, and the search he
asked for (Openverse, decided 2026-09-25) is a *fetcher* that fills that same
folder — **phase 2, its own spec**, which is also where credits belong (§M5).

**What phase 1 ships:** a Music row on the export sheet, shown while a reel is
ticked — a genre picker and a **Folder…** button — and a track mixed under the reel
with the game sound ducked and the music faded. Nothing in the project format
changes, and no Settings sheet is built (§M6).

This is the reviewed draft; §B records what the review changed.

---

## §M1. The folder and its genres

- **The folder is a path in `state.json`** (`musicFolder`), set by the Music row's
  **Folder…** button. It is "the file on this machine I use", the whisper model's
  test, exactly as the whiteboard's armed diagram is (`AppFiles::backdrop`, whose
  accessor pair `music_folder` copies).
- **A genre is an immediate, non-hidden subfolder that holds at least one track.**
  `rock/`, `edm/`. Files at the folder's top level are not a genre — a genre the
  coach did not name would be a guess. Folders below a genre are not read.
- **A track is a file whose extension is one of** `mp3`, `ogg`, `oga`, `opus`,
  `flac`, `wav`, `m4a` (lower-cased) — which keeps `cover.jpg` from being chosen.
  The extension filters the listing; it is not the decoder's gate (`Reader` is
  `decodebin3`).
- **One module holds the rule**: `music::genres(folder)` and `music::tracks(folder,
  genre)` in `pundit-app`, called by the window to fill the picker and by the bus to
  build a job. One rule, not two readers of it. A listing is a `read_dir`, cheap
  on either thread; nothing caches it.

## §M2. The Music row on the export sheet

- **A `ComboBox` ("None" and the genres) and a Folder… button**, in their own row,
  shown only while a reel target is ticked — `reel-ticked`, fed from `main.rs`
  exactly as `whole-match-ticked` is.
- **With no folder, or a folder with no genres, the row says so and its button is
  the fix**: "Choose a music folder with a subfolder per genre". No other sheet to
  find.
- **Folder… opens the folder dialog** `pickers.rs` already has for a project
  folder (the variant is renamed `Folder`, since it now picks two kinds), writes
  `musicFolder` through the window's `AppFiles` handle — the bus only *reads* it,
  so no command (§U4's rule) — and re-reads the genres.
- **The genre reaches the bus on `ExportChoices` as `music_genre:
  Option<String>`**, its seventh field. That ends `ExportChoices`' `Copy`, so the
  call sites that pass it twice (`jobs`, then `settle_pickers`) clone or borrow.
  **It is not a `Pickers` field** — `Pickers` is what writes back to `Preferences`,
  and `From<ExportChoices> for Pickers` drops it deliberately.
- **The genre is remembered in `state.json` (`musicGenre`, a label)**, not in
  `Preferences`: it names a folder on this machine. **The bus writes it**, after the
  jobs are built and only if they were — beside `settle_pickers`, for
  `a_refused_run_leaves_the_pickers_alone`'s reason — so one click has one writer
  (`state.rs` records that two handles writing the same document can lose a
  field). The window only reads it, when the sheet opens; a remembered genre whose
  folder has gone reads as None.
- **"Mute source audio" keeps its word**: it mutes the game sound, so a muted reel
  with music is music alone. Its tooltip's *"comes out silent"* for a reel becomes
  *"carries only the music, or nothing with Music: None"*.

## §M3. Which track

- **A stable choice per match, not rotation state.** The genre's tracks are sorted
  by file name and the index is a stable hash (FNV-1a, written in core — no crate,
  no RNG) of the project's folder name, offset by the reel's side (All, Home, Away
  are 0, 1, 2) so one run's three reels differ. Different matches mostly get
  different songs; **re-exporting a match keeps its song**; nothing is stored, so
  nothing is written and no state can go stale.
- **Chosen in `job`** for a `Reel` target, and **frozen into the job** as a path,
  like everything a queued job holds. A file deleted before the run renders
  silence and a logged line — `Reader`'s existing rule.
- **A genre with no tracks when the job is built refuses the target**
  (`CantExport`, naming the folder): the coach asked for music, and shipping the
  reel without it would be the silent trade `CLAUDE.md`'s Default rule forbids.
  No folder at all with a genre remembered is the same refusal.
- **No probe.** The music region spans the whole reel; a track shorter than the
  reel plays to its own end and `Reader` pads past EOF with silence, and the fade
  is the reel's last seconds. Length would only buy fading a short track early,
  which a song's own outro already does.

## §M4. The mix

- **`Track` carries what names its file**: `Game { entry }`, `Commentary { entry }`
  and `Music`, with `Region.entry` removed. A music region spans every entry, so an
  index on it would be a lie, and moving the index into the variant lets the
  mixer's exhaustive `match` find every site. The music's file is
  `Encode::music: Option<PathBuf>`; the `Music` arm reads it.
- **`audio_regions` takes the music** (`Option<f64>`: the reel's music gain, or
  None) and **applies the duck itself**: with music, each game region's gain is
  `MUSIC_DUCK × source_volume`. The caller passes the source volume as today, so
  the duck cannot be applied twice, and "no game region at `source_volume == 0`"
  still holds — mute gives music alone.
- **The duck is static, for the whole reel**, including any tail after a short
  track ends (that tail is quiet crowd sound — §For the coach). A sidechain or a
  split at the music's end is not built until it is asked for.
- **Fades are chosen by track, not stored per region**: `envelope` takes `(in,
  out)` from the region's track — `MUSIC_FADE_IN` (1 s) and `MUSIC_FADE_OUT` (3 s)
  for `Music`, `RAMP_SAMPLES` (5 ms) for the rest — so game and commentary regions
  are untouched by construction. The edge formula becomes
  `min(rise / in, fall / out, 1)`; today's `min(rise, fall, RAMP)/RAMP` only works
  because both edges share one length. A reel under 4 s overlaps the two fades,
  which the formula handles and a test pins.
- **No clamp, and the reason is an invariant**: a reel has no commentary, one
  track's regions never overlap (the mixer's doc), and the volumes written are 0.0
  and 1.0, so the peak is `MUSIC_DUCK + MUSIC_GAIN`. A `const` assert beside the
  constants keeps that sum ≤ 1 (0.3 + 0.7). A clamp would be dead code with a test
  that cannot fail.
- **Only a reel gets music.** A clip, a tag and All clips carry commentary; the
  whole match is a stream copy; the basket mixes at the default volumes on purpose
  (`CLAUDE.md`). `job` adds the music only for `ExportTarget::Reel(_)`.

## §M5. Credits — phase 2

A CC track needs crediting (#84), and the credit belongs beside the film the way
`.chapters.txt` is. **Phase 1 does not write one**: the coach's own files carry no
licence the app can vouch for (ID3 has no licence frame; the tags would mostly
give "title by artist"), so a credits file would claim more than it knows. Phase
2's catalogue knows each track's licence, and its fetcher is where the credit is
captured and written.

## §M6. Why no Settings sheet

§U4 designed a Settings sheet for two tenants with "nothing to sit beside": #102's
bool and this folder. #102 found its home on the scrubber's own menu (the snapping
spec §H), and **the folder has one reader — the Music picker — so it sits beside
that**, on #116's precedent: a machine-wide value set at the thing it affects. The
sheet waits for a tenant that truly has nothing beside it; phase 2's search may
well sit beside the picker too. §U4 and #84's line that #84 builds the sheet are
corrected with this spec (§R).

## §M7. Packaging

`decodebin3` autoplugs a parser/demuxer and a decoder per format, and the `.deb`
already depends on the packages carrying them. **The smoke test does not check
them**, and `CLAUDE.md` makes it the only proof of the dependency list. Its element
list gains the eight it lacks (re-derived 2026-10-12): `mpegaudioparse`,
`mpg123audiodec` (mp3); `oggdemux`, `vorbisdec` (ogg, oga; opus already has
`opusdec`); `flacparse`, `flacdec`; `wavparse`; `avdec_aac` (m4a, beside the
listed `qtdemux`).

---

## Tests

- **core:** a reel compilation with music yields one `Music` region over the whole
  output and game regions at `MUSIC_DUCK × source_volume`; with music and a muted
  source, no game region; with Music: None, the regions equal today's. The music
  envelope rises over 1 s and falls over the last 3 s, and on a 3-second reel the
  overlapping fades stay within 0..=1 and meet.
- **core:** the stable choice — same folder name and side give the same index,
  sides differ, and it wraps on a one- and two-track genre.
- **core:** `MUSIC_DUCK + MUSIC_GAIN <= 1.0` as a `const` assert.
- **media:** a reel exported over a tone track decodes back with the tone present
  and fading to silence by the last frame (sabotage: the Music arm reading nothing
  must fail it).
- **app (unit):** `music::genres` lists only non-hidden subfolders with a track,
  and `music::tracks` only the listed extensions, sorted.
- **harness:** a reel export with a genre succeeds and records `musicGenre`; a
  refused run does not; a genre emptied before the job is built refuses the target
  naming the folder.
- **ui:** the Music row appears only while a reel is ticked; Folder… reaches its
  callback; the empty-folder line shows with no genres (by label, the accessible
  action — `tests/ui/export_sheet.rs`'s pattern).
- **No test reaches the network** — phase 1 has none.

## §R. What this corrects

1. **§U4 and #84 say #84 builds the Settings sheet.** It does not (§M6); the sheet
   waits for a tenant with nothing beside it.
2. **#84 says the fetcher fills "the same cache the mixer reads".** Phase 2 should
   fetch into the **music folder** (`<folder>/<genre>/`), not `$XDG_CACHE_HOME`: a
   cache can be evicted, and a track's credit with it.

## §B. What the review changed

- The Settings sheet and its button → a Folder… button in the Music row (§M6).
- Rotation with a stored `musicLast`, written by the bus while the window wrote
  `musicGenre` on the same click (the lost-update race `state.rs` names) → a
  stable per-match choice that stores nothing, and the bus as the one writer of
  `musicGenre`.
- A clamp whose test could not fail → a `const` assert on the gains.
- Per-region fade fields → fades chosen by track; `Region.entry` → in the `Track`
  variants; the envelope's shared-length shortcut called out.
- A probe and a credits file → no probe (Reader pads), credits to phase 2.
- An undefined failure for an empty genre → a named refusal.
- The smoke-test list was wrong (`opusdec` is already there) and incomplete
  (parsers and demuxers) → re-derived.

## What this does not do

- **No search, no download, no network, no credits** — phase 2.
- **No per-project music** and no format bump; no music under anything but a reel.
- **No Settings sheet**, no waveform, no track preview, no level control.
- **No beat-matching or cutting the reel to the music.**

## For the coach

Four calls #84 left open, each shipping with the default below unless you say
otherwise:

1. **Duck, don't replace:** the game sound stays under the music at about a third
   of its level, so a goal still sounds like a goal. "Mute source audio" gives you
   music only.
2. **A reel longer than the song:** the song plays to its end and the rest of the
   reel is the crowd, still at that lower level — no looping.
3. **Which song:** each match gets its own song from the genre, and re-exporting
   the same match keeps it. Home, Away and All reels of one match get different
   songs.
4. **The genre is remembered for the machine**, not per project.

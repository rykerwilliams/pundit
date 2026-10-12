# Design — music under a goals reel (BACKLOG #84, phase 1)

The coach (2026-09-24): *"pull in soundtrack files that are licensed for use, e.g.
rock, edm, etc. for the goals clips."* — and the same day, *"pick genre"*: choose
rock or EDM on the export and the app does the rest. A goals reel is the one export
with no commentary (every entry is `clip_id: None`, `core/src/reel.rs`), so it is
the one people expect music under.

Read `CLAUDE.md` and BACKLOG #84 first. #84 records the decisions this rests on and
they are not re-argued here: **the coach's own folder of music is the mixer's only
input**, and the search he asked for (Openverse, decided 2026-09-25) is a *fetcher*
that fills that same folder — **phase 2, its own spec**. #84 also builds the
Settings sheet that `2026-10-02-app-settings-design.md` §U4 designed, because the
folder's path has nothing else to sit beside (§U4, and the snapping spec's §H).

**What phase 1 ships:** a Settings sheet with one row (the music folder); a Music
picker on the export sheet, shown while a reel is ticked, listing the folder's
genres; a track chosen and mixed under the reel with the game sound ducked; and a
credits file beside the video. Nothing in the project format changes.

---

## §M1. The folder and its genres

- **The folder is a path in `state.json`** (`musicFolder`), set on the Settings
  sheet. It is "the file on this machine I use", the whisper model's test, exactly
  as the whiteboard's armed diagram is (`AppFiles::backdrop`, whose accessor pair
  `music_folder` copies). **No default is set for the coach**; the sheet offers
  the XDG Music directory as the dialog's starting point only.
- **A genre is an immediate, non-hidden subfolder that holds at least one track.**
  `rock/`, `edm/`. Files at the folder's top level are not a genre and are not
  offered — a genre the coach did not name would be a guess. Nested folders below
  a genre are not read.
- **A track is a file whose extension is one of** `mp3`, `ogg`, `oga`, `opus`,
  `flac`, `wav`, `m4a` (lower-cased). The extension is a *filter for the listing*,
  not the decoder's gate: `Reader` (`composite/audio.rs`) is `decodebin3`, and a
  file it cannot decode is already silence plus a logged line.
- **The listing is read when the export sheet opens**, by the window, with
  `read_dir` — a directory listing, not a decode, so it is cheap on the UI thread.
  Nothing caches it; a genre folder added while the app runs appears at the next
  open.

## §M2. The picker on the export sheet

- **One `ComboBox`, "Music": `None` and the genres**, in its own row, shown only
  while a reel target is ticked — `reel-ticked` beside `whole-match-ticked`, fed the
  same way from `main.rs`. With no folder set, or a folder with no genres, the row
  says so in one line and links nothing: the Settings sheet is one click away.
- **It is remembered in `state.json` (`musicGenre`, a label)**, not in
  `Preferences`: a genre is the name of a folder on this machine, so a project
  carried elsewhere must not bring it, and it costs no format bump (v17 stays). A
  remembered genre whose folder has gone reads as `None`.
- **It reaches the bus on `ExportChoices` as `music_genre: Option<String>`** — the
  seventh field. It is **not** a `Pickers` field: `Pickers` is what writes back to
  `Preferences`, and this does not belong there. The window writes `musicGenre`
  itself, beside the other `state.json` setters, when the sheet's Export or Add to
  queue is pressed.
- **"Mute source audio" keeps its word**: it mutes the game sound. With music on, a
  muted reel is music alone — which is a real choice (a reel cut to a song), so the
  tooltip's *"comes out silent"* for a reel becomes *"carries only the music, or
  nothing with Music: None"*.

## §M3. Which track

- **Rotation, not chance.** A coach exporting ten matches does not want ten
  identical soundtracks (#84), and the workspace has no RNG crate. The genre's
  tracks are sorted by file name and the next one after the last used is taken;
  `state.json` keeps `musicLast: { genre → file name }`, written by the **bus**
  after a job is built (it is the one that chose). A last-used name that has gone
  restarts at the first.
- **Chosen when the job is built** — at Export, or at Add to queue — and **frozen
  into the job** as a path, like everything else a queued job holds. A file deleted
  before the run renders silence and a logged line, which is `Reader`'s existing
  rule, and the run carries on.
- **Only the chosen track is probed**, for its length and its tags (§M5). A folder
  is never probed whole: `probe` is a `Discoverer` with a 10-second bound per file.
  **`probe` refuses an audio-only file** (`ProbeError::NoVideo`), so this is a new
  `pundit_media::probe_audio(path) -> (seconds, tags)`.

## §M4. The mix

- **The music starts with the reel and is faded out at its end**: a 1-second fade
  in, a 3-second fade out ending on the reel's last frame. A track **shorter** than
  the reel fades out over its own last 3 seconds and the rest of the reel is game
  sound alone. **No looping**: a loop is two regions of one file back to back,
  which breaks the mixer's stated invariant that a block never retires one region
  and starts another on the same file — and a hard loop point is audible anyway.
- **The game sound is ducked under it, statically.** With music on, the game's
  gain is `MUSIC_DUCK` (0.3) × the source volume, through the `source_volume`
  argument `audio_regions` already takes. No sidechain: a reel is a run of goals
  and crowd noise, and a constant duck is what an editor would set by hand. The
  music plays at `MUSIC_GAIN` (0.7). Both are constants in `core::audio`, retuned
  in one place.
- **`Track::Music`, and the file on `Encode`.** `Region.entry` names an entry, and
  a music region spans all of them, so the music's path is
  `Encode::music: Option<PathBuf>` and the mixer's exhaustive `match region.track`
  gains a `Music` arm that reads it — the compiler finds every site. Core builds
  the region (`audio_regions` gains the music's length and gain); media never
  learns where the file came from.
- **Fades are per region.** `envelope` ramps every region over `RAMP_SAMPLES`
  (5 ms) — a click guard. A region gains its own fade-in and fade-out lengths,
  defaulting to that guard, so game and commentary regions are byte-identical and
  only music gets seconds.
- **A limiter is now needed, and it is a clamp.** The mixer sums with no clamp; at
  these gains a goal's crowd roar plus a loud track can pass ±1.0. Samples are
  clamped to ±1.0 after the sum. A soft-knee limiter is not built until a clamp is
  heard.
- **Only a reel gets music.** A clip, a tag, All clips and the whole match carry
  commentary or are a stream copy; the basket mixes at the default volumes on
  purpose (`CLAUDE.md`). `job` adds the music only for `ExportTarget::Reel(_)`.

## §M5. Credits beside the film

- **`<film>.credits.txt`**, written in `composite::export`'s `finish` after the
  rename, exactly as `.chapters.txt` is: same name cleaning, same `" (2)"`
  de-duplication, never fatal, reported as `ExportDone::credits`. **A run with no
  music removes a stale one** — the name is ours, `.chapters.txt`'s rule.
- **The text is one line**, ready to paste under a video: `Music: "<title>" by
  <artist> — <licence>`, from the file's tags (title, artist, and the licence or
  copyright tag), falling back to the file name for a missing title and leaving out
  what is not there. **Phase 1 cannot vouch for a licence**: the coach's own files
  carry whatever their tags say. The sheet's Music row says so ("credits come from
  the file's own tags"), which is the honest scope until phase 2's catalogue
  supplies licences it can stand behind.

## §M6. The Settings sheet

Built per §U4, with three corrections (§R):

- **The eighth `Sheet`**, opened by a **Settings** button in the drawing row
  beside **Keys**, gated `!root.recording && !root.previewing` like the others.
- **One row: "Music folder"** — the path as read-only text, **Choose…** (the folder
  dialog `Pick::ProjectFolder` already opens) and **Clear**. No text field, so it
  needs no `editing` fold at all.
- **Written by the window** through its own `AppFiles` handle; the bus reads
  `music_folder` through its handle when it builds a job. No new command (§U4's
  rule: the bus only has to *read* it).
- It is built to hold what comes next — phase 2's search, #131's band colour —
  but **ships with one row**, because a container built before its contents is
  what §U4 exists to prevent.

## §M7. Packaging

`Reader` decodes whatever `decodebin3` finds an element for, and the `.deb`'s
dependencies already carry the decoders (plugins-good/ugly, libav). **The smoke
test does not check them**, and `CLAUDE.md` makes it the only proof of the
dependency list: `packaging/smoke-test.sh`'s element list gains the decoder for each
listed extension (`mpg123audiodec`, `vorbisdec`, `opusdec`, `flacdec`,
`wavparse`, `avdec_aac`).

---

## Tests

- **core:** a reel compilation with music yields one `Track::Music` region over
  the whole output, the game regions at `MUSIC_DUCK` × the source volume, the
  fades as specified for a track longer and one shorter than the reel; with
  `Music: None`, every region is byte-identical to today's (pins the default
  ramps).
- **core, rotation:** the next track after the last used, wrapping, and a vanished
  last-used name restarting at the first.
- **media:** a reel exported with a tone track decodes back with the tone present,
  fading out by the last frame, and no sample past ±1.0 with a full-scale track
  over full-scale game sound (the clamp).
- **media:** `probe_audio` reads a fixture's length and title tag, and an
  audio-only file no longer fails.
- **harness:** a reel export with a genre writes `<film>.credits.txt` naming the
  track, records it as `musicLast`, and the next export takes the following track;
  a re-export with `Music: None` removes the credits file.
- **ui:** the Music row appears only while a reel is ticked; the Settings button
  opens the sheet, and Choose…/Clear reach their callbacks (found by label,
  pressed with the accessible action — `tests/ui/export_sheet.rs`'s pattern).
- **No test reaches the network** — phase 1 has none.

## §R. Where §U4 and #84 are stale

1. §U4 calls the Settings sheet "the seventh `Sheet`"; Keys took seventh (#96), so
   it is the **eighth**.
2. §U4's Esc contract (fold the sheet's `editing` into `text-editing`) is obsolete:
   `text-editing` is now Slint's own `text-input-focused` (#96 task 1), so a field
   joins nothing — and this sheet has no field.
3. §U4 places the button "beside `Fit`"; the drawing row now holds **Keys** there
   too, and Settings goes beside Keys.
4. #84 says the fetcher fills "the same cache the mixer reads". Phase 2 should
   fetch into the **music folder** (`<folder>/<genre>/`), not `$XDG_CACHE_HOME`: a
   cache can be evicted, and a track's credits with it.

## What this does not do

- **No search, no download, no network** — phase 2, its own spec.
- **No per-project music** and no format bump; no music under anything but a reel.
- **No waveform, no track preview, no level control** — the genre is the control
  the coach asked for.
- **No beat-matching or cutting the reel to the music.**

## For the coach

Four calls #84 left open, each shipping with the default below unless you say
otherwise:

1. **Duck, don't replace:** the game sound stays under the music at about a third
   of its level, so a goal still sounds like a goal. "Mute source audio" gives you
   music only.
2. **A reel longer than the track:** the music fades out at the end of the track
   and the rest of the reel is crowd sound — no looping.
3. **Rotation:** each reel takes the next track in the genre, in file-name order,
   so ten matches get different songs.
4. **The genre is remembered for the machine**, not per project.

# Plan — music under a goals reel (#84, phase 1)

Spec: `docs/superpowers/specs/2026-10-12-reel-music-design.md`. Read it and
`CLAUDE.md` first; neither is repeated here. **Every task starts with the
`start-task` skill and ends with `verify`, `sabotage-proof` and `recheck-claims`**
— they are where this plan's own corrections will come from.

## Where this stands (update it as tasks land)

- **1 — core + media: the music track, its fades and the duck.** Not started.
- **2 — the bus: the folder, the genre, the choice, the refusal.** Not started.
- **3 — the window: the Music row.** Not started.

**Shipping: three PRs, in order, none parallel.** 1 changes types that 2 builds;
2 adds the `ExportChoices` field that 3's window fills.

## §A. The census, taken 2026-10-12 on `main` at `1eefc21`

Re-derive at each task (`start-task` §4); these are counts, not a file list to
trust.

| Changing | Sites |
|---|---|
| `Region` / `Track` (task 1) | `core/src/audio.rs` 4, `core/tests/audio.rs` 21, `media/src/composite/audio.rs` 2 (the `Mixer`'s exhaustive match and its `encode.entries[region.entry]` lookup) |
| `audio_regions` callers (task 1) | `bus/export.rs` (`job`), `bus/basket.rs`, `core/tests/audio.rs` 3, `media/tests/export.rs` 5 |
| `Encode` literals (task 1) | `media/src/composite/export.rs` 4, `media/tests/export.rs` 18, `bus/export.rs` 2, `bus/basket.rs` 1 |
| `ExportChoices` literals (task 2) | `bus/export.rs` 1, `main.rs` 2, harness `export.rs` 6, `whole_match.rs` 8, `reel.rs` 2, `whiteboard.rs` 2, `highlights.rs` 1, `new_match.rs` 1, `preview.rs` 1 |
| `ExportChoices` by value (task 2) | `export`, `start_run`, `jobs`, `enqueue_export`, `enqueued`, `settle_pickers` in `bus/export.rs`; `Command::Export` / `EnqueueExport` in `bus/mod.rs`; `sheet_asks` in `main.rs` |
| `Pick::ProjectFolder` (task 3) | `pickers.rs` 2, `main.rs` 2 |

**`Region.entry`'s other readers are not this field**: `frame.entry` (export,
preview) and `spec(pts).entry` (`composite/mod.rs`) are `FrameSpec`'s, and the
`.entry(...)` calls are `HashMap` entries.

---

## 1. Core + media: the music track, its fades and the duck

**One PR across both crates**, because `Track` gains a variant the `Mixer`'s
exhaustive match must handle — the lesson #138 task 2 recorded (a type change owns
its construction sites).

1. **`Track::{Game { entry }, Commentary { entry }, Music}`**; `Region.entry`
   removed. The `Mixer`'s lookup becomes a match on the track.
2. **`envelope` takes its edge lengths from the track**: `MUSIC_FADE_IN` (1 s) /
   `MUSIC_FADE_OUT` (3 s) for `Music`, `RAMP_SAMPLES` for the rest, and the formula
   `min(rise / in, fall / out, 1)`. Its doc's paragraph on "the smaller of the two
   ramps" is rewritten for unequal edges.
3. **`audio_regions(compilation, prefs, source_volume, music: Option<f64>)`** —
   `music` is the music's gain. With `Some`, each game region's gain is
   `MUSIC_DUCK × source_volume`, and one `Music` region spans the whole output.
   `MUSIC_DUCK` (0.3) and `MUSIC_GAIN` (0.7) live beside the other constants, with
   a `const` assert that their sum is ≤ 1 (the spec's §M4 invariant).
4. **`Encode::music: Option<PathBuf>`**; the `Mixer`'s `Music` arm reads it. A
   `Music` region with `music: None` is a bug and panics with a message, never
   silence — `job` is the only producer and always sets both.
5. **Every existing caller passes `None`** and every `Encode` literal gains
   `music: None`. No behaviour changes for anything but a music run.
6. **`packaging/smoke-test.sh`** gains the eight elements (spec §M7), with a
   comment naming why, beside the existing "the last five aren't named" note.

### Tests

- core: one `Music` region over the whole output; game gains at
  `MUSIC_DUCK × source_volume`; with a muted source, no game region; with `None`,
  the regions equal a hand-written expectation from before the change.
- core: the music envelope rises over 1 s and falls over 3 s; on a 3-second reel
  the fades overlap and stay within 0..=1.
- media: a reel exported over a tone fixture decodes back with the tone present
  mid-reel and below a threshold in the last frame.

### Sabotage

1. The `Mixer`'s `Music` arm returning no reader → the media tone test fails.
2. `envelope` using `RAMP_SAMPLES` for music → the core fade test fails, and
   **only** it.
3. The duck dropped (game at `source_volume`) → the core gain test fails; the
   `const` assert does not, which is correct — it pins a different claim.

---

## 2. The bus: the folder, the genre, the choice, the refusal

1. **`AppFiles::music_folder` / `set_music_folder`** and **`music_genre` /
   `set_music_genre`** — `backdrop`'s shape: `lenient`, `Option`, no container
   change.
2. **`pundit-app/src/music.rs`**: `genres(folder) -> Vec<String>` (non-hidden
   immediate subfolders holding a track) and `tracks(folder, genre) ->
   Vec<PathBuf>` (the spec's extensions, lower-cased, sorted by file name). A lib
   module, so a unit test reaches it.
3. **`core`: `music_index(seed: &str, side: ReelSide, tracks: usize) -> usize`**
   — FNV-1a over the seed plus the side's offset (All 0, Home 1, Away 2), modulo
   the count. Pure, so core tests pin it.
4. **`ExportChoices::music_genre: Option<String>`**; `Copy` removed, the by-value
   call sites clone or borrow (§A). `From<ExportChoices> for Pickers` ignores it,
   and **its doc ("the same six values") is corrected**.
5. **`job` for a `Reel` target with a genre**: list the tracks, refuse
   `CantExport("<genre> in <folder> has no music")` when there are none (or no
   folder), else pick `tracks[music_index(project folder name, side, n)]`, set
   `Encode::music` and call `audio_regions(..., Some(MUSIC_GAIN))`. Any other
   target ignores the genre.
6. **`settle_pickers`' neighbour writes `musicGenre`**, after the work and only on
   success — the run and the enqueue alike.
7. **The basket is untouched**: it passes `None`.

### Tests

- core: `music_index` — same seed and side, same index; sides differ; wraps at 1
  and 2 tracks.
- app unit (`music.rs`): hidden folders and empty folders are not genres; only the
  listed extensions are tracks; sorted.
- harness (new tests in `reel.rs`): a reel export with a genre succeeds and stores
  `musicGenre`; the same match re-exported picks the same file (read off the job
  via a log line or the event, whichever `job` already exposes — decide at the
  task, don't add an API for it); a genre emptied before Export refuses naming the
  folder; a refused run leaves `musicGenre` alone.

### Sabotage

1. `musicGenre` written before the jobs are built → the refused-run test fails.
2. `music_index` ignoring the side → the sides test fails.
3. The empty-genre refusal removed → the harness refusal test fails (the run would
   otherwise succeed silently, which is the trade the spec forbids).

---

## 3. The window: the Music row

1. **`reel-ticked`** on `ExportSheet`, forwarded from a window property
   `export-reel-ticked` exactly as `whole-match-ticked` is from
   `export-whole-match-ticked`, and set beside `set_export_whole_match_ticked` in
   its two `main.rs` places (by a `reel_ticked` beside `whole_match_ticked`).
2. **The row**: "Music" `ComboBox` (None + genres), **Folder…**, and the empty
   line when there is no folder or no genre; shown only while `reel-ticked`.
   **Measure the card at the 1100×700 floor** with the row shown — the export
   sheet scrolls its body (#134), so the row costs height, not a fit; record the
   number.
3. **`Pick::ProjectFolder` → `Pick::Folder`** (§A's four sites).
4. **`main.rs`**: on sheet open, read `music_folder` and `music_genre` from the
   window's `AppFiles`, list `music::genres`, preselect the remembered one;
   Folder… writes `musicFolder` and re-lists; `sheet_asks` fills `music_genre`
   from the combobox.
5. **The mute tooltip's reel clause** reworded (spec §M2).

### Tests

- ui: the row is absent with no reel ticked and present with one; Folder… reaches
  its callback; with no genres, the empty line is findable by its words. Found by
  label, pressed with the accessible action (`tests/ui/export_sheet.rs`).
- **Not reachable** (`main.rs`): the genre listing and preselection. Stated as a
  risk, not claimed.

### Sabotage

1. The row's `if` dropped (always shown) → the absent-with-no-reel test fails.
2. Folder…'s `clicked` unwired → the callback test fails.

---

## Risks

1. **`main.rs`'s listing and preselection are untested** (task 3) — the usual
   `[[bin]]` limit. The listing itself is `music.rs`'s, which is tested.
2. **The duck is a guess at a level** (0.3 / 0.7). It is one pair of constants;
   the first real reel is the measurement.
3. **A stable choice can collide** — two matches hashing to one song in a
   two-track genre. Accepted (spec §M3).
4. **The decoders are checked by the smoke test only** — CI's workspace job does
   not install the `.deb`. A format that fails to decode is silence plus a logged
   line, as today.

## What this does not do

The spec's list, unchanged: no search, download, credits, Settings sheet, per-
project music or format bump.

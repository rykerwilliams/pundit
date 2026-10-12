# Plan — music under a goals reel (#84, phase 1)

Spec: `docs/superpowers/specs/2026-10-12-reel-music-design.md`. Read it and
`CLAUDE.md` first; neither is repeated here. **Every task starts with the
`start-task` skill and ends with `verify`, `sabotage-proof` and `recheck-claims`.**

## Where this stands (update it as tasks land)

- **1 — the music track: core, media, and their callers.** **Done**
  (`claude/84-task-1`). `Track::{Game { entry }, Commentary { entry },
  Music(PathBuf)}`, `envelope`'s edges by track, `audio_regions(.., music)`
  owning `MUSIC_GAIN` / `MUSIC_DUCK` and the assert on their sum; both app
  callers pass `None`; a `tone_audio` fixture (Ogg Vorbis). Core audio tests 15 →
  19, media export 32 → 33. **The first "Done" was premature**: the new tests
  were added after the last `cargo fmt`, and the code review — not the gate I had
  claimed to run — found `fmt --check` red. The review also added `id3demux` to
  the smoke test (nearly every real mp3 carries ID3 tags), a fractional source
  volume to the duck test so the multiply is pinned, and the caller contract
  (music only without commentary) on `audio_regions`. **Sabotage, five, each where predicted:** the duck
  dropped fails the gain test; music on the 5 ms ramp fails both core fade tests
  (the short-reel one because the peak then reaches full level) and the media
  test's last frame (still at 0.322); the `Music` arm mapped to no file fails the
  media test (music at 0.000).
  **One mistake of my own, in the test:** `decode_audio` is mono and I indexed it
  as stereo, so "mid-reel" was really 4–5 s, inside the fade-out — the music read
  half its level (0.176) and the test failed for the wrong reason before it
  passed for the right one.
- **2 — the bus: the folder, the genre, the choice, the refusals.** **Done**
  (`claude/84-task-2`). `AppFiles::{music_folder, music_genre}` and setters,
  `pundit-app/src/music.rs` (`genres`, `tracks`, the path check),
  `core::reel::music_index`, and `reel_music` in `jobs`. Core reel tests 26 → 27,
  `music.rs` 3 unit tests, harness reel 7 → 11.
  **Two corrections to this plan:**
  - **Sabotage 1 as written cannot fail.** "The side added before the modulo" is
    `(h + k) % n == (h % n + k) % n` — the same number. The hazard the review
    meant is hashing the side **into** the seed, which collides at random; that
    is the sabotage run, and it fails the three-sides test.
  - **The user guide moves to task 3**: it would describe a Music row that only
    task 3 adds.
  **Sabotage, three:** the side hashed into the seed fails the three-sides test;
  the path check removed fails the `../outside` harness test and its unit test;
  the empty-genre refusal turned into "no music" fails the empty-genre test —
  **and the `../outside` test too**, since both lean on that refusal, so the path
  check is pinned apart only by the unit test and the first mutation.
  **Then the code review found what the sabotage list had missed**: no mutation
  dropped the *wiring* (`jobs` passing `None` to `job`), and the happy-path
  harness test only checked a file existed, so a reel with its music lost would
  have passed. **And the first fix was vacuous too**: both reviewers said the
  fixture game video was silent, I relayed it unchecked, and a "any sound in the
  output" assertion passed with the wiring cut — `fixtures::webm` leaves
  `audiotestsrc` at its default 440 Hz sine. The song is now 1500 Hz and measured
  at that frequency; with the wiring cut it reads 0.000 and the test fails.
  Also from review: the listing follows symlinks (a folder built from links into
  `~/Music` listed nothing), `tracks` is "listed by `genres`" and nothing else,
  `music_index` has a golden value (comparing it with itself could not catch a
  hasher change), `reel_music`'s insertion had stolen `job`'s doc comment, and
  the duplicate `../x` harness test went (the unit test is the proof).
- **3 — the window: the Music row.** Not started.

**Shipping: three PRs, in order, none parallel.** 1 changes `audio_regions` and
`Track`, which 2 calls; 3 writes the `state.json` keys 2 reads.

## §A. The census, recounted 2026-10-12 on `main` at `1eefc21`

Re-derive at each task (`start-task` §4). These were recounted after the plan's
review found the first draft's numbers wrong.

| Changing | Sites |
|---|---|
| `Region` literals (task 1) | `core/src/audio.rs` 1 (`region()`), `core/tests/audio.rs` 3 |
| `Region.entry` reads (task 1) | `media/src/composite/audio.rs` (the `Mixer`'s path lookup), `core/tests/audio.rs` 4 (lines 111, 263, 264, 281) and its private `region(entry, …)` helper |
| `Track::Game` / `Commentary` uses (task 1) | `core/src/audio.rs`, `core/tests/audio.rs`, `media/src/composite/audio.rs` — the compiler lists them once the variants carry fields |
| `audio_regions` callers (task 1) | `bus/export.rs` (`job`), `bus/basket.rs`, `core/tests/audio.rs` 3, `media/tests/export.rs` 5 |
| `Pick::ProjectFolder` (task 3) | `pickers.rs` 2, `main.rs` 2 |

**Not this field**: `frame.entry` and `spec(pts).entry` are `FrameSpec`'s; the
`.entry(...)` calls are `HashMap` entries. **No site** in `pundit-harness`,
`tests/ui` or `packaging` touches `Region`, `Track` or `audio_regions`.

---

## 1. The music track: core, media, and their callers

**Three crates in one PR** — core changes `audio_regions` and `Track`, media's
`Mixer` matches on `Track`, and `pundit-app`'s two callers pass the new argument.

1. **`Track::{Game { entry }, Commentary { entry }, Music(PathBuf)}`**;
   `Region.entry` removed; `Track` loses `Copy`. The `Mixer`'s path lookup becomes
   a total match on the track.
2. **`envelope`'s edges come from the track** (spec §M4): `MUSIC_FADE_IN` /
   `MUSIC_FADE_OUT` for `Music`, `RAMP_SAMPLES` otherwise, and
   `min(rise / in, fall / out, 1)`. Its doc's "smaller of the two ramps" paragraph
   is rewritten for unequal edges.
3. **`audio_regions(compilation, prefs, source_volume, music: Option<&Path>)`**,
   owning `MUSIC_GAIN` and `MUSIC_DUCK` and the `const` assert on their sum.
4. **Every existing caller passes `None`**: `bus/export.rs`, `bus/basket.rs`, and
   the tests. Nothing changes for a run without music.
5. **`packaging/smoke-test.sh`** gains the eight elements (spec §M7).
6. **Prose that states the old behaviour**: `core/src/audio.rs`'s module doc ("the
   two tracks") and `Region`'s doc ("regions of the two tracks"); CLAUDE.md's
   audio bullet. (The reel's "audio is the game's alone" line in CLAUDE.md waits
   for task 2, when it stops being true.)

### Tests

- core: one `Music` region over the whole output carrying the path; game gains at
  `MUSIC_DUCK × source_volume`; with a muted source, music and no game region.
  (With `None`, the existing tests pin today's regions — no new test.)
- core: the music envelope rises over 1 s and falls over 3 s; on a 3-second reel
  the fades overlap, stay within 0..=1 and meet.
- media: a reel exported over a tone fixture — **a listed extension**, e.g. an
  `.ogg` from `fixtures::audio_only`'s pattern — decodes back with the tone
  present mid-reel and below a threshold in the last frame.

### Sabotage

1. The `Mixer`'s `Music` arm mapped to no file → the media tone test fails.
2. `envelope` using `RAMP_SAMPLES` for music → the core fade test fails, and so
   does the media test's last-frame threshold; both are named.
3. The duck dropped (game at `source_volume`) → the core gain test fails.

---

## 2. The bus: the folder, the genre, the choice, the refusals

1. **`AppFiles::music_folder` / `set_music_folder`** and **`music_genre` /
   `set_music_genre`** — `backdrop`'s shape. The bus calls only the getters.
2. **`pundit-app/src/music.rs`** (a lib module): `genres(folder) -> Vec<String>`
   and `tracks(folder, genre) -> Vec<PathBuf>`, per spec §M1.
3. **core: `music_index(seed: &str, side: ReelSide, n: usize) -> usize`** =
   `(fnv1a(seed) % n + side) % n`.
4. **`jobs` reads the folder and genre once**, and for a `Reel` target: no genre →
   no music; a genre but no folder → refuse "no music folder is set"; a genre not
   in `music::genres(folder)` or with no tracks → refuse naming the genre and the
   folder; else `tracks[music_index(project folder's name, side, n)]` passed to
   `audio_regions`. Every other target ignores the genre.
5. **Prose**: CLAUDE.md's reel bullet ("audio is the game's alone") and the export
   section; the user guide's `docs/book/src/guide/exporting.md` (a muted reel
   "comes out silent"; no Music row yet); BACKLOG's index line for #84.

### Tests

- core: `music_index` — same seed and side, same index; with three tracks the
  three sides differ; wraps at one and two.
- app unit (`music.rs`): hidden and empty folders are not genres; only the listed
  extensions are tracks; sorted.
- harness (`reel.rs`): with a genre set in the scratch `state.json`, a reel export
  succeeds; a genre emptied before Export refuses naming it; no folder refuses in
  its own words; a genre `../x` refuses rather than reading outside the folder.
  **No "same file on re-export" test** — nothing exposes the chosen path, and
  `music_index`'s core test pins the determinism.

### Sabotage

1. `music_index` adding the side before the modulo → the three-sides test fails
   for some seed (the test uses one that shows it).
2. The listing check removed → the `../x` test fails.
3. The empty-genre refusal removed → its test fails (the run would otherwise
   succeed silently).

---

## 3. The window: the Music row

1. **`reel-ticked`** on `ExportSheet`, forwarded from a window property
   `export-reel-ticked` as `whole-match-ticked` is from `export-whole-match-ticked`,
   set by a `reel_ticked` beside `whole_match_ticked` in its two `main.rs` places.
2. **The row** (spec §M2): the genre `ComboBox`, **Folder…**, and the empty line;
   shown only while `reel-ticked`. Measure the card at 1100×700 with the row shown
   and record it (the body scrolls, #134).
3. **`Pick::ProjectFolder` → `Pick::Folder`** (§A).
4. **`main.rs`**: on sheet open, read `music_folder` / `music_genre`, list
   `music::genres`, preselect; a genre pick writes `musicGenre`; Folder… writes
   `musicFolder` and re-lists.
5. **The mute tooltip's reel clause** (spec §M2).
6. **The user guide**, `docs/book/src/guide/exporting.md` — the Music row, and a
   muted reel no longer "comes out silent" with music on (moved from task 2).
7. **A remembered genre whose folder has gone is cleared on read** (from task 2's
   review): the bus refuses a genre it cannot honour, so a sheet that showed
   "None" while `musicGenre` still named the gone folder would refuse on Export
   over music the coach can't see. The window clears the key when the listing no
   longer has it.

### Tests

- ui: the row is absent with no reel ticked and present with one; Folder…
  reaches its callback; **choosing a genre reaches the callback that remembers
  it** — the dead-wire class `tests/ui/export_sheet.rs` exists for; check first
  that Slint's `ComboBox` can be driven by the accessible API, and if it cannot,
  say so rather than claim the test; the empty line is findable by its words.
- **Not reachable** (`main.rs`): the listing on open and the preselection.

### Sabotage

1. The row's `if` dropped → the absent-with-no-reel test fails.
2. Folder…'s `clicked` unwired → its callback test fails.
3. The genre's `selected` unwired → the genre test fails (if the test exists).

---

## Risks

1. **`main.rs`'s listing and preselection are untested** — the `[[bin]]` limit;
   `music.rs` holds the logic and is tested.
2. **The levels are a guess** (0.3 / 0.7): one pair of constants, measured on the
   first real reel.
3. **A stable choice can collide**, and sides share a song below three tracks
   (spec §M3).
4. **Decoders are proven by the smoke test only**; CI's workspace job does not
   install the `.deb`.

## §B. What the plan's review changed

- The genre on `ExportChoices` (23 literals, ending `Copy`, and never reaching
  `job`, which takes `Pickers`) → read from `state.json`, written by the window on
  pick. This also fixed a bug: the hidden Music row would have cleared the genre on
  every clips-only export.
- `Encode::music` plus a panic for a music region without it, and a gain the caller
  chose → the path in `Track::Music`, the gain owned by core.
- The side offset hashed in (sides colliding at random) → added after the modulo,
  with "sides differ" true from three tracks.
- The census: `ExportChoices` 23 (not 24), `Encode` 20 (not 25), `Region` literals
  4 — moot after the first two changes, recounted anyway.
- Dropped: a "same file on re-export" harness test nothing could observe, a "None
  equals today" test the existing tests already are, and a sabotage clause that
  was explanation.
- Added: the genre-listing check against `../x`; the old-behaviour prose and the
  user guide; the combobox wire test.

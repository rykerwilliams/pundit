# Plan — music under a goals reel, phase 2: an Openverse search and credits (#84)

Spec: `docs/superpowers/specs/2026-10-13-reel-music-search-design.md`. Read it
and `CLAUDE.md` first. Neither is repeated here, except where §0 below corrects
the spec. **Every task starts with the `start-task` skill and ends with
`verify`, `sabotage-proof` and `recheck-claims`.**

Every `path:line` here was re-derived on **2026-10-13 at `51d3767`** (`main`
after #92, which the spec branch is rebased on). **#131 (the Settings sheet) is
being built on the main checkout now.** It edits the same `handle-key` sheet
chain, `Scrim` order and `tests/ui` `mod` list, so **this plan executes after
#131 merges**, and `start-task` §3 re-derives `app.slint` and `main.rs` first.
Paths are from the repository root; `core`, `media`, `app` and `harness` mean
`crates/pundit-<name>/`. **§A holds every citation**; the tasks cite only what
§A does not.

## Where this stands (update it as tasks land)

- **1 — the credit beside the reel, read at render time.** Done. Census
  re-taken at `4d43df9` (after #131/#94): every task 1 site in §A still at its
  line. **One addition:** `ExportDone` grew past clippy 1.92's
  `large_enum_variant` threshold inside `ExportMessage::Finished` (216 bytes);
  it takes the same reasoned `#[allow]` `Render` already carries, rather than a
  `Box` every reader would deref. `credit_beside` is re-exported from
  `pundit_media` for task 4 and `reel_music`. Sabotages 1–3 each failed exactly
  the tests the plan named.
- **2 — core: the query, the parse, the licence words, the credit and the stem
  rule.** Done; §A's task 2 sites unchanged at `4d43df9`. Deviations:
  - **The fixture page is `core/tests/data/openverse_page.json`**, not
    `tests/fixtures/`: the root `.gitignore`'s `fixtures/` (for local video)
    ignores any directory of that name. Built from one live `q=rock` answer
    fetched while writing it (2026-10-13; the test reads only the file). Every
    real row was CC BY, so the CC0 and PDM rows are real rows relabelled, as
    the NC and BY-SA ones are.
  - **`Candidate` carries `seconds: f64` and `landing_url`, and no
    `creator_url`**: nothing reads the creator's page (the credit links the
    track's, spec C2), so the parse reads ten fields, not eleven. A row is
    parsed on its own (`serde_json::Value` per row), so a field of the wrong
    type drops that row, not the page.
  - **The credit's first line carries the modification phrase for every
    licence**, not only BY: one rule, true of every reel, and owed only by BY.
    A missing `foreign_landing_url` drops the page line (two lines).
  - **2.4 ranks four hand-built rows** rather than 2.3's parse, so that
    sabotages 1 and 3 (a dropped row kept) fail 2.3 alone, as the plan says;
    on the parse they failed 2.4 too. Sabotage 4 is "`rank` does nothing":
    `rank` takes a slice, so it cannot filter.
  All eight sabotages then failed exactly the named test.
- **3 — media: `get`, `inspect_audio` and the fixture server's body.** Not
  started.
- **4 — app: `music_search.rs` and the default folder.** Not started.
- **5 — the Find music sheet, its keys, the wiring and the prose.** Not started.

**Shipping: one PR, a commit per task, in order.** Task 1 is now small. It adds
no `ExportJob` field, so there is no mechanical diff to keep out of the
feature's review. On its own it would ship a credits file that only a
hand-written credit could feed. Each commit stays verifiable on its own: its own
tests pass at its own head.

---

## §0. Where the spec is out of date or wrong (checked 2026-10-13)

The plan's review changed eight of the spec's decisions, and **the spec is
amended in place for each**, marked *"Amended 2026-10-13 after the plan's
review"*: C3, S3, S6, S7 (×2), S8, S10 and the Testing list. What is left here is
what the spec does not say.

1. **`inspect_audio` cannot be `read_all` itself.** `read_all`
   (`audio.rs:554`) builds and drops its `Reader` inside, and the container walk
   needs the pipeline between `start` and `rest`. So `inspect_audio` is
   `read_all`'s three lines inline, with the walk between them, at **8 kHz
   mono**. A ten-minute track is then 4.8 M samples, about 19 MB, held briefly.
   There is no `count_rest`.
2. **The basket's stem rule has a fallback of its own.** `file_stem`
   (`bus/basket.rs:602`) falls back to `DEFAULT_NAME`, which is `"Basket"`
   (`:47`), not `Untitled`, and its budget is `MAX_STEM_BYTES = 200` (`:576`).
   So the lifted helper takes **both** the budget and the fallback:
   `naming::file_stem(name, budget, fallback)`. The basket keeps its wrapper and
   its tests (`:746`–`:771`).
3. **`pundit-app` has no dev-dependency on media's `fixtures`**
   (`app/Cargo.toml`'s `[dev-dependencies]`). `music_search.rs`'s tests need it:
   `pundit-media = { path = "../pundit-media", features = ["fixtures"] }`, as
   the harness has it.
4. **`music::EXTENSIONS` is private** (`music.rs:14`). `already_in` needs it, so
   it becomes `pub(crate)`.
5. **The new genre field must join `tests/ui/text_editing.rs`**, which the
   spec's Testing does not name. Its sheet test says *"the three sheets that
   hold fields"* (`:249`), which becomes four, and it gains a probe for the
   field.
6. **Spec S10's key rule is the basket's branch exactly.** That branch
   (`app.slint:5080`–`:5086`) **rejects** every key but the closing Esc, which
   is what delivers letters to the field. The branch returns before the
   shortcut lookup, so R still never starts a take.
7. **The user guide already carries spec Risk 5's sentence**
   (`exporting.md:78`–`:80`). Task 5 adds only the Find music paragraph.
8. **No CHANGELOG edit.** It is curated at release.

---

## §A. The census, 2026-10-13 at `51d3767`

Re-derive at each task (`start-task` §4), and after #131 merges.

| Changing | Sites |
|---|---|
| `finish` and the sidecars (task 1) | `media/src/composite/export.rs`: `Render` `:88` (`Encode` arm `:90`), `Encode` `:106` (`audio: Vec<Region>` `:116`), `ExportJob` `:124` (**unchanged**), `ExportDone` `:247` (`chapter_list` `:265`; gains `credits`), its one literal `:483` (`chapter_list:` `:489`), `finish` `:469`, `write_chapter_list` `:555`. `Track::Music(PathBuf)` `core/src/audio.rs:87`, `Region` `:94` |
| `ExportDone` readers (task 1) | the run's log line `app/src/bus/export.rs:251`–`:265` gains `credits {:?}`. `media/tests/copy.rs:715` names `ExportDone { .. }` in a comment only |
| Existing tests extended (task 1) | `media/tests/export.rs:2281` (`cancel_leaves_nothing_and_keeps_an_existing_file`), `:2362` (`a_compilation_gets_a_chapter_per_entry`, its stale list at `:2388`, `audio: Vec::new()`) |
| `reel_music` (task 1) | `app/src/bus/export.rs:1203` (returns at `:1208`); its phase 1 refusals `:1213`–`:1221` are untouched |
| core (task 2) | `core/src/lib.rs`: `pub mod credits;` after `chapters` (`:12`), `pub mod openverse;` after `naming` (`:22`). `naming`: `safe_chars` `:104`, `one_line` `:121`, `truncate_on_boundary` `:137`, `folder_slug` `:178`, and the new `file_stem`. `metadata::APP_NAME` `:30`, `UNTITLED` `:40` |
| The basket's stem (task 2) | `app/src/bus/basket.rs:47` (`DEFAULT_NAME`), `:576` (`MAX_STEM_BYTES`), `:602` (`file_stem`, now a one-line wrapper), `:621` (`output_path`, unchanged), tests `:746`–`:771` (unchanged, now reaching the helper) |
| `download.rs` (task 3) | `unreachable_or` `media/src/download.rs:217` (now takes `&str`), its one caller `:166`; `fetch_to` `:138` (the loop `get` copies); the re-export `media/src/lib.rs:36` gains `get`. `download` (`:82`) and `check` (`:240`) **unchanged** |
| `Reader` (task 3) | `media/src/composite/audio.rs:263` (struct), `:302` (`start`), `:487` (`rest`), `:554` (`read_all`, its shape); `tone_audio` `media/src/fixtures.rs:259` |
| Fixture server (task 3) | `media/src/fixtures.rs:806` (`#[derive(Debug, Clone, Copy)]` → `Clone`), `:807` (`Answer`), `:823` (`serve`: `answer` cloned into each connection's thread), `:855`–`:868` (`respond`'s exhaustive `match answer`: one new arm). Callers of `Answer::` outside `fixtures.rs`: 11, all `Copy` uses to be checked under `Clone` |
| `AppFiles` (task 4) | `app/src/bus/state.rs:291` (struct), `:299`–`:311` (`default_location`, literal `:300`), `:316`–`:321` (`in_config_dir`, literal `:317`), `films_dir` `:651`; tests `:769` (`the_films_folder_follows_the_videos_directory`), `:786` (`a_test_state_file_keeps_its_films_beside_itself`) |
| app modules (task 4) | `app/src/lib.rs:16` (`pub mod music;`): `pub mod music_search;` after it; `app/src/music.rs:14` (`EXTENSIONS` → `pub(crate)`); `app/Cargo.toml` `[dev-dependencies]` |
| `app.slint` (task 5) | `ExportSheet`'s Music properties `:2062`–`:2071`; the Music row `:2180`–`:2207` (`Folder…` at `:2203`); the window's `export-music-*` `:4391`–`:4395`; `close-basket` `:4948` (the shape); `handle-key`'s export branch `:5064`–`:5071` (the new branch goes **ahead** of it), the basket's `:5080`–`:5086` (the shape); the export sheet's `Scrim` `:7433`–`:7464` (the new one goes after it), with its music wiring `:7447`–`:7451`; the error dialog's `Scrim` `:7653` |
| `main.rs` (task 5) | `UiState` `:155`, the `UI` thread-local `:349`–`:352`; `wire_music(…)` called at `:438`; `show_music` `:1342`; `wire_music` `:1359` (`set_music_genre` `:1369`); `wire_devices` `:2323`–`:2337` (spawn `:2332`, return `:2334`) |
| tests/ui (task 5) | `app/tests/ui/main.rs:70`–`:84` (the `mod` list: `find_music` in order); `sheet_scroll.rs:48` (`sheets()`, 7 entries: an eighth); `text_editing.rs:249`; `export_sheet.rs:557` (`NO_GENRES`), `:566` (the Music row test) |
| Refusal and notice texts | **None changes.** New texts live in `music_search.rs` and on the sheet |
| Exhaustive matches | `respond`'s `match answer` (task 3). No `Command`, `Event`, `UndoAction` or `KeyAction` changes |
| Prose | `CLAUDE.md:175` (*"No test may reach Hugging Face"*); a `.credits.txt` bullet after `:794` (the chapter list's YouTube rules, which end the `.chapters.txt` pair `:793`–`:794`); `:1055`–`:1062` (the goals reel's music clause); `docs/book/src/guide/exporting.md:69`–`:86` |
| BACKLOG | #84: the "Next, in order" line `BACKLOG.md:27`–`:34`, the entry `:1754` (its phase 2 bullet `:1881`, revisit `:1892`), as of this plan's commit |

**No site** in `packaging`, `keymap.rs`, `project.rs` or `store.rs`; no
`ExportJob` literal; no `state.json` key.

---

## 1. The credit beside the reel, read at render time

### Files

- `media/src/composite/export.rs`, `media/tests/export.rs`
- `app/src/bus/export.rs`

### Steps

1. **`credit_beside(music: &Path) -> PathBuf`**: `<music's file name>.credit.txt`
   in the same folder. This is the one sibling-name function. It is `pub`,
   because task 4's `add` writes to the same name.
2. **`write_credits(job) -> Option<PathBuf>`** after `write_chapter_list`.
   - For `Render::Encode`, it looks for a region in `audio` whose `track` is
     `Track::Music(path)`, and reads `credit_beside(path)` if there is one.
   - **With music and a credit**, it writes the text to
     `job.path.with_extension("credits.txt")`.
   - **Otherwise** (`Render::Copy`, no music region, or no credit file), it
     removes a stale one. NotFound is fine, and anything else is logged.
   - A failure to write is logged and returns `None`.
3. **`ExportDone::credits: Option<PathBuf>`**, filled in `finish` after
   `chapter_list`, and added to the run's log line.
4. **`reel_music`**: when the chosen track's `credit_beside` does not exist but
   any `*.credit.txt` does in its genre folder, log `bus: <track> has no credit
   beside it, but its folder holds some — a renamed track?` (spec C3). It never
   refuses.

### Tests

**media** (`cargo test -p pundit-media --test export`):

- 1.1 `a_reels_music_credit_lands_beside_the_film` (new). An `Encode` whose
  `audio` holds a `Track::Music` region over a `tone_audio` file with
  `credit_beside` written gives `<out>.credits.txt` with exactly that text, and
  `done.credits` names it.
- 1.2 `a_compilation_gets_a_chapter_per_entry` (`:2362`, extended): beside its
  stale `.chapters.txt`, a stale `out.credits.txt` is written first. The run has
  `audio: Vec::new()`, so no music, and the file is gone after it.
- 1.3 `cancel_leaves_nothing_and_keeps_an_existing_file` (`:2281`, extended): a
  stale `out.credits.txt` is written first and **survives** the cancel.

### Sabotage

1. `write_credits` writes only and never removes → only 1.2.
2. `finish` skips `write_credits` → 1.1 and 1.2. 1.3 stays green, because a
   cancel never reaches `finish`.
3. `credit_beside` returns `<stem>.credit.txt` → only 1.1.

1.3 has no sabotage of its own: a cancel never reaches `finish`, so the
credits are left alone by construction. The test pins that construction.

The `reel_music` log line has no test (a log line), and it is manual check 8.

---

## 2. Core: the query, the parse, the licence words, the credit and the stem rule

### Files

- `core/src/openverse.rs`, `core/src/credits.rs` (new), `core/src/lib.rs`,
  `core/src/naming.rs`
- `core/tests/openverse.rs`, `core/tests/credits.rs` (new),
  `core/tests/fixtures/openverse_page.json` (new)
- `app/src/bus/basket.rs` (the wrapper)

### Steps

1. **`openverse::LICENCES: [&str; 3] = ["cc0", "pdm", "by"]`**, the one list. Its
   doc carries spec L1's reasons and the BY-SA decision with its date.
2. **`search_url(base, genre) -> String`**: spec S4's query. `q` is the trimmed
   text, cut to 200 characters on a char boundary and percent-encoded by a small
   `encode` here (RFC 3986's unreserved set passes, everything else becomes
   `%XX` of its UTF-8 bytes).
3. **`Candidate`** and **`results(json) -> Result<Vec<Candidate>, ParseError>`**.
   - `serde` reads spec S4's fields only.
   - A row is dropped when it has no `id`, `url` or `duration`, a `title` that
     `naming::one_line` leaves empty, or a `license` not in `LICENCES`.
   - A missing `creator` becomes `"Unknown artist"`. `title` and `creator` are
     stored after `one_line`.
4. **`rank(&mut [Candidate], genre)`** is a stable sort, genre matches first.
   **`licence_words(license, version)`** gives the row and credit words.
   **`licence_url(&Candidate)`** gives `license_url`, or one built from
   licence and version.
5. **`credits::credit_text(&Candidate) -> String`**: spec C2's three lines, with
   the modification phrase as a `const`.
6. **`naming::file_stem(name, budget, fallback) -> String`** (§0.2), the
   basket's body moved here. The basket's `file_stem` becomes
   `naming::file_stem(name, MAX_STEM_BYTES, DEFAULT_NAME)`.
7. **The fixture page: seven results**, trimmed from one real `/v1/audio/`
   answer, every name a public Jamendo artist's.
   - The allowed rows: **BY** (`genres: ["rock"]`), **BY** (`genres: []`),
     **CC0** (`genres: ["pop"]`) and **PDM** (`genres: ["rock"]`).
   - Slipped in by hand: an **NC** row, a **BY-SA** row, and a **BY** row with
     `duration: null`. The null duration is on an allowed licence, so only the
     duration can drop it.
   - The `url`s read `{AUDIO}`, which **task 4's** tests replace. Core's tests
     leave them alone.

### Tests

**core** (`cargo test -p pundit-core --test openverse --test credits`):

- 2.1 `the_query_names_the_licences_length_and_category` — the exact URL for
  `"rock"`.
- 2.2 `a_genre_is_percent_encoded_and_cut_at_200` — `"drum & bass"` →
  `q=drum%20%26%20bass`; a 300-character `é…` is cut to 200 characters and is
  still valid UTF-8.
- 2.3 `the_page_keeps_exactly_its_allowed_rows` — `results` of the fixture, **in
  page order** and before `rank`, has exactly the four allowed ids. A comment
  beside each dropped id says why: NC, BY-SA, null duration.
- 2.4 `genre_matches_rank_first_in_their_own_order` — `rank` of 2.3's rows for
  `"rock"`: BY-rock, PDM-rock, then BY-none, CC0-pop.
- 2.5 `licence_words_for_by_cc0_and_pdm`.
- 2.6 `a_by_credit_says_it_was_edited` — the three lines, with the phrase.
- 2.7 `a_missing_licence_url_is_built_from_licence_and_version`.
- 2.8 `a_title_with_a_newline_is_one_line_and_no_creator_is_unknown_artist`.

The basket's `file_stem` tests (`basket.rs:746`–`:771`) pin the moved body
unchanged.

### Sabotage

1. The licence re-check removed from `results` → only 2.3.
2. `q` not encoded → only 2.2.
3. A row with a null `duration` kept → only 2.3.
4. `rank` filters instead of sorting → only 2.4 (2.3 is before `rank`).
5. The modification phrase dropped → only 2.6.
6. The `license_url` fallback returns `""` → only 2.7.
7. `one_line` not applied → only 2.8.
8. `naming::file_stem` ignores its `fallback` and returns `UNTITLED` → only
   `a_name_with_no_usable_stem_falls_back_to_the_default` (`basket.rs:746`).

---

## 3. Media: `get`, `inspect_audio` and the fixture server's body

### Files

- `media/src/download.rs`, `media/src/lib.rs`
- `media/src/composite/audio.rs`
- `media/src/fixtures.rs`

### Steps

1. **`get(url, limit, cancel) -> Result<Vec<u8>, GetError>`**.
   - The pipeline is `souphttpsrc location=<url> user-agent=pundit/<version>
     iradio-mode=false timeout=<TIMEOUT> ! appsink`, with `TIMEOUT` 20 s (1 s
     under `#[cfg(test)]`).
   - The loop is `fetch_to`'s, bus-driven: `timed_pop_filtered(EOS | ERROR)`
     with a short timeout, the cancel checked between pops, and the appsink
     pulled each turn (`max-buffers` set, so the bytes are counted as they come
     and `limit` holds).
   - **`GetError`** is `Status(u16)`, `Failed(String)` or `Cancelled`. `Status`
     comes from the error's `http-status-code` detail, **only when it is 400 or
     more**, and never from its text.
   - **There is no length check.** A short body is souphttpsrc's resource
     error (spec S7, measured), and it surfaces as `Failed`.
2. **`unreachable_or(url: &str, err)`**, with its caller at `:166` passing
   `&fetch.url`.
3. **`inspect_audio(path, cancel) -> Result<AudioFile { seconds: f64, extension:
   &'static str }, CompositeError>`** in `audio.rs` (§0.1).
   - It runs `Reader::start(path, 8000, 1, cancel)`, then walks
     `iterate_recurse()` for the fixed precedence list: `qtdemux` → `m4a`,
     `oggdemux` → `ogg`, `wavparse` → `wav`, `flacparse` → `flac`,
     `mpegaudioparse` → `mp3`.
   - With none of them present it refuses, and so it does with no audio
     (`Ok(None)` from `start`).
   - Then `rest`; `seconds` is the sample count over 8000.
   - It is re-exported from `lib.rs`.
4. **Fixtures:**
   - `Answer` derives `Clone` (not `Copy`) and gains `Body(Arc<[u8]>)` and
     `Cut(Arc<[u8]>, usize)`. `Cut` sends the headers for the whole body, then
     only the first `n` bytes, then closes. `Stall` cannot stand in for it,
     because it holds the connection open and reaches the timeout rather than a
     short body. `serve` clones the answer into each connection's thread, and
     the 11 outside callers are checked.
   - **`audio_file(path, encoder: AudioEncoder, seconds)`** is `tone_audio`'s
     sibling. `Ogg` is `vorbisenc ! oggmux`, `Mp3` is `lamemp3enc ! xingmux !
     id3v2mux` (ID3-tagged, so that `id3demux` is in the graph), and `Wav` is
     `wavenc`.

### Tests

**media** (`cargo test -p pundit-media --lib download` for `get`;
`--test export`'s audio neighbourhood, or a new `tests/inspect.rs`, for
`inspect_audio`):

- 3.1 `get_returns_the_body` — `Body` round-trips byte for byte.
- 3.2 `get_refuses_past_its_limit` — a 1 MB body under a 512 KB limit.
- 3.3 `get_gives_up_on_a_stall` — `Stall` fails within a few seconds under the
  test `TIMEOUT`.
- 3.4 `a_short_body_is_an_error` — `Cut(body, len / 2)` gives
  `Err(GetError::Failed(_))`, not `Ok` and not `Status`. Measured 2026-10-13
  with a raw socket server of exactly that shape: souphttpsrc posts *"Server
  does not support seeking."* with `http-status-code=200`.
- 3.5 `get_reads_the_status_from_the_details` — `Status("429 Too Many
  Requests")` gives `Status(429)`.
- 3.6 `get_cancels` — against `Stall`, a flag set after 0.5 s returns
  `Cancelled` within 2 s.
- 3.7 `inspect_names_ogg_mp3_and_wav` — the three `audio_file`s give their
  extension and a length within 0.1 s. The MP3 is ID3-tagged.
- 3.8 `an_html_page_is_not_audio` — fails.

### Sabotage

1. `Status` taken from details at any code (no `>= 400`) → only 3.4, whose
   `200` becomes `Status(200)`.
2. `timeout` not set → only 3.3, which then hangs to souphttpsrc's default.
   Run it with a test timeout.
3. `limit` not enforced → only 3.2.
4. The walk names the first element it finds, whatever it is → only 3.7.

The full decode is pinned by 4.5, a length that disagrees with the catalogue.
The list's **precedence** is not pinned: no supported file plugs two of its
elements (Risk 3).

---

## 4. App: `music_search.rs` and the default folder

### Files

- `app/src/music_search.rs` (new), `app/src/lib.rs`, `app/src/music.rs`,
  `app/Cargo.toml` (§0.3)
- `app/src/bus/state.rs`

### Steps

1. **`Genre::resolve(folder: &Path, text) -> Result<Genre, String>`** (spec
   S5).
   - An empty slug is `Err("Type a genre to search for")`.
   - Otherwise it is an existing subfolder whose `folder_slug` equals the
     text's, under the folder's own name, or the slug itself.
   - `Genre` holds the folder name and the query text.
2. **`search(base, &Genre)`**: `get` of `search_url` with a 1 MiB limit, then
   `results` and `rank`. `GetError`s become spec S11's words, with 429 matched
   on `Status(429)`.
3. **`add(&Candidate, folder, &Genre, cancel) -> Result<PathBuf, String>`**
   (spec S7):
   - `create_dir_all(folder/genre)`;
   - `get` with a 64 MiB limit, then write `.<id>.part`;
   - `inspect_audio`, then the 2 s check against `duration`;
   - the stem from `naming::file_stem(<title> — <creator>, 120, UNTITLED)`;
   - write the credit to `credit_beside(dest)` (task 1) through a temp file and
     a rename, then rename the `.part` to `dest`;
   - if that rename fails, delete the credit;
   - every failure deletes the `.part`.

   There is **no** rename seam.
4. **`already_in(folder, genre, stem) -> bool`**: `<stem>.<ext>` exists for any
   ext in `music::EXTENSIONS` (§0.4).
5. **`AppFiles`**: `music: PathBuf` beside `films`, with `default_location`
   resolving `glib::UserDirectory::Music` through `music_dir(music, home)` in
   `films_dir`'s shape. `in_config_dir` uses `config_dir.join("music")`, and
   `pub fn default_music_folder(&self) -> PathBuf`.

### Tests

**app lib** (`cargo test -p pundit-app --lib music_search` and `state`). Two
servers, as spec Testing says: the audio from one, and the page with `{AUDIO}`
replaced by its URL from the other. **The served track is an `audio_file`
generated at the length the page's candidate claims**, or the test overrides
the candidate's `duration`; whichever is used is stated in the test.

- 4.1 `search_returns_the_ranked_rows`.
- 4.2 `a_429_says_the_anonymous_limit`.
- 4.3 `add_lands_the_track_and_its_credit` — it starts from a music folder
  **that does not exist**, and ends with `<rock>/<title> — <creator>.ogg` and
  its `.credit.txt` holding `credit_text`, with no `.part` left.
- 4.4 `a_failed_rename_leaves_no_credit` — a **directory** made beforehand at
  the track's destination name makes the rename fail for real. The Add fails,
  there is no `.credit.txt`, and there is no `.part`. The order of the credit
  and the rename is not separately pinned (spec S7).
- 4.5 `a_track_whose_length_disagrees_is_refused_and_leaves_nothing` — the
  candidate's `duration` is 10 s over the served file's.
- 4.6 `a_deleted_track_can_be_added_again` — the orphan credit is overwritten,
  and `already_in` is false before and true after.
- 4.7 `a_genre_matches_an_existing_folder_by_slug` — `" ROCK"` against an
  existing `Rock/` lands in `Rock/`, and the test asserts that capitalisation.
- 4.8 `an_empty_genre_is_refused` — `Genre::resolve(dir, "  ")` alone.
- 4.9 `the_films_folder_follows_the_videos_directory` (`state.rs:769`, extended
  with `music_dir`'s two cases) and
  `a_test_state_file_keeps_its_films_beside_itself` (`:786`, extended:
  `default_music_folder()` is `/x/cfg/music`).

### Sabotage

1. The credit not deleted when the rename fails → only 4.4.
2. The 2 s check removed → only 4.5.
3. `already_in` looks for the credit instead of the track → only 4.6.
4. `resolve` compares names, not slugs → only 4.7.
5. `in_config_dir`'s music placed under `$HOME` → only 4.9's second test.

---

## 5. The Find music sheet, its keys, the wiring and the prose

### Files

- `app/ui/app.slint`, `app/src/main.rs`
- `app/tests/ui/find_music.rs` (new), `tests/ui/main.rs`,
  `tests/ui/sheet_scroll.rs`, `tests/ui/text_editing.rs`
- `CLAUDE.md`, `docs/book/src/guide/exporting.md`, `docs/hands-on-checklist.md`,
  `BACKLOG.md`

### Steps

1. **`FindMusicSheet inherits Sheet`**:
   - the genre `LineEdit` and **Search**;
   - a `for` over `rows: [MusicRow]` (`id`, `title`, `artist`, `length`,
     `licence`, `genres`, `state`: `""` / `"Adding…"` / `"In rock"`), with Add
     `enabled: row.state == "" && !root.adding`;
   - the message line;
   - Openverse's line and spec L3's line;
   - **Close**.

   It states `preferred-height` / `max-height`, never `height`. The window gets
   `find-music-open`, `-genre`, `-rows`, `-adding` and `-message`; callbacks
   `search-music(string)`, `add-music(string)` and `find-music-closed()`; and
   **`function close-find-music()`**, in `close-basket`'s shape. That function
   clears the flag, calls `keys.focus()` and fires `find-music-closed()`. The
   `Scrim` goes after the export sheet's.
2. **The Music row** gains **Find music…** before `Folder…`, forwarded through
   `ExportSheet` like `choose-music-folder`.
3. **`handle-key`**: a `find-music-open` branch **ahead of** the export sheet's,
   the basket's exactly (§0.6):
   `if (pressed && event.text == Key.Escape && !root.text-editing) {
   root.close-find-music(); return accept; } return reject;`.
4. **`main.rs`: `wire_find_music`**, beside `wire_music`.
   - **Its state lives in `UiState`**: `music_generation: u64`, `music_rows:
     Option<(Genre, Vec<Candidate>)>`, `music_added: Option<String>` and
     `music_cancel: Option<Arc<AtomicBool>>`. `OPENVERSE` is a `const`.
   - **Search:**
     - `folder = music_folder().unwrap_or(default_music_folder())`, then
       `Genre::resolve(&folder, text)`, with an `Err` going to the message line;
     - bump the generation and spawn with **only the number**;
     - in the `upgrade_in_event_loop` closure, `UI.with_borrow` drops the answer
       if the generation moved, and otherwise stores it and sets the rows, each
       row's state from `already_in(&folder, &genre, &stem)`.
   - **Add:** the same `folder`. Set the row to *"Adding…"* and `adding`, store
     a fresh cancel flag, and spawn `add`. On success:
     - when `music_folder()` is `None`, `set_music_folder(Some(&folder))`;
     - the row reads *"In rock"*;
     - `music_added` is set.

     On failure, the row's message.
   - **`find-music-closed`:** set the cancel flag, and if `music_added`,
     `set_music_genre(Some(name))` then `show_music` (spec S6).
   - Opening prefills the field from the picker's genre.
5. **`CLAUDE.md`**:
   - `:175` becomes *"No test may reach Hugging Face, Openverse or Jamendo"*;
   - a `.credits.txt` bullet after `:794`: read at render time beside the
     music, removed when there is none;
   - the goals reel's clause (`:1055`–`:1062`) gains a sentence: Find music
     fetches into the same folder, off the bus, with each track's credit beside
     it.
6. **The guide** gains a *Finding music* paragraph (§0.7). **The hands-on
   checklist** gains the manual checks.
7. **BACKLOG #84**: shipped, with the PR number. Its "Next, in order" line
   leaves.

### Tests

**tests/ui** (`cargo test -p pundit-app --test ui`, one binary; `find_music` is
a `mod` of `tests/ui/main.rs`):

- 5.1 `find_music_is_reached_from_the_music_row` — with a reel ticked, *Find
  music…*'s accessible action reaches its callback.
- 5.2 `the_sheet_says_openverse_and_the_licence_caveat` — both lines are found
  by label.
- 5.3 `add_reaches_its_row_and_the_others_wait` — row 2's Add gives
  `add-music("id2")`. With `find-music-adding`, every Add reads
  `accessible-enabled` false.
- 5.4 `esc_in_the_genre_field_leaves_both_sheets_open_and_the_next_closes_find_music_only`
  — the first Esc leaves both open; the second fires `find-music-closed`, and
  `export-sheet-open` stays true.
- 5.5 `sheet_scroll.rs`: an eighth `Sheet` entry, `FindMusicSheet` with 20 rows
  (the existing tests then scroll it and close it with Esc).
- 5.6 `text_editing.rs`: the sheet probe gains the genre field, and *"three
  sheets"* becomes four.

**`main.rs` is unreachable from any test** (`[[bin]]`; Risk 1). Its wiring is
manual checks 1–5.

### Sabotage

1. The Find music branch put **after** the export sheet's → only 5.4 (the first
   Esc closes the export sheet).
2. The branch's `!root.text-editing` dropped → 5.4 (the first Esc closes Find
   music) and 5.6 (Esc in the field closes the sheet).
3. Add's `enabled` ignores `adding` → only 5.3.
4. The Music row's button wired to `choose-music-folder` → only 5.1.
5. The sheet given a fixed `height` → only 5.5.

---

## Manual checks (batched at the end)

On a project with tagged goals, a reel ticked, and **no music folder set**:

1. **First search.** Find music…, `rock`, Search. Up to twenty rows appear, with
   rock-tagged rows first, and every licence reads CC BY or Public domain.
   Openverse's line is shown.
2. **First Add.** Add one. The row says *Adding…*, the other Adds grey, then it
   reads *In rock*. `~/Music/pundit/rock/` holds the track and its
   `.credit.txt`, and the folder is now remembered.
3. **Close selects it.** Close: the Music picker reads `rock`. Export: the reel
   has the song, and `<reel>.credits.txt` holds the credit with the modification
   phrase.
4. **A stale answer.** Search `rock`, then at once `jazz`. Only jazz rows show.
5. **Close mid-Add.** Close while a row is adding. Nothing is left in the
   folder, no `.part` included (`ls -a`), and with no folder set before, none is
   remembered.
6. **Offline.** Turn the network off and Search. The sheet's line names
   `api.openverse.org`.
7. **Keys.** In the field, `r` types an `r` and starts no recording. Esc leaves
   the field, a second Esc closes Find music, and the export sheet is still
   open.
8. **A renamed track.** Rename the added track and re-export the reel. There is
   no `.credits.txt`, and stderr has the *"a renamed track?"* line.

---

## Risks

1. **The `main.rs` wiring is untested** (spec Risk 1): the thread, the
   generation in `UiState`, the default folder written on success, and the
   re-selected genre. Each is a short mapping over tested pieces, and manual
   checks 1–5 cover them.
2. **#131 lands first and moves everything task 5 touches**: the sheet chain in
   `handle-key`, the `Scrim` order, `tests/ui/main.rs`'s `mod` list and
   `sheet_scroll.rs`'s `sheets()`. Task 5's census is re-taken after #131
   merges, and its branch order (Find music ahead of export) is checked against
   wherever the Settings sheet's branch went.
3. **Two orderings are argued, not pinned**: the credit before the rename (4.4
   pins only the cleanup), and the container precedence list (3.S4).
4. **The fixture page is a snapshot of Openverse's shape.** A renamed field
   upstream breaks the app and not the tests. Spec Risk 8 already accepts that
   Openverse can change, and the parse reads only eleven fields.

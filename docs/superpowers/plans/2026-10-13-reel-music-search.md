# Plan — music under a goals reel, phase 2: an Openverse search and credits (#84)

Spec: `docs/superpowers/specs/2026-10-13-reel-music-search-design.md`. Read it
and `CLAUDE.md` first. Neither is repeated here, except where §0 below corrects
the spec. **Every task starts with the `start-task` skill and ends with
`verify`, `sabotage-proof` and `recheck-claims`.**

Every `path:line` here was re-derived on **2026-10-13 at `51d3767`** (`main`
after #92, which the spec branch is rebased on). `main` moves daily, so
**re-derive every line at `start-task` §3**, starting with `app.slint` and
`main.rs`. Paths are from the repository root; `core`, `media`, `app` and
`harness` mean `crates/pundit-<name>/`. **§A holds every citation**; the tasks
cite only what §A does not.

## Where this stands (update it as tasks land)

- **1 — the credit beside the reel.** Not started.
- **2 — core: the query, the parse, the licence words and the credit.** Not
  started.
- **3 — media: `get`, `inspect_audio` and the fixture server's bodies.** Not
  started.
- **4 — app: `music_search.rs`, the default folder and the end-to-end test.** Not
  started.
- **5 — the Find music sheet, its keys, the wiring and the prose.** Not started.

**Shipping: two PRs, in order.**

1. **Task 1 alone, first.** It is useful by itself: a coach who writes
   `<track>.credit.txt` by hand for his own song gets `<film>.credits.txt`
   beside the reel. It touches the 22 `ExportJob` literals (§A), and that
   mechanical diff should not sit inside the feature's review.
2. **Tasks 2–5 together, as one PR with a commit each, in that order.** A search
   module with no sheet to reach it would be dead code on `main`, and so would a
   sheet with nothing behind it. Each commit stays verifiable on its own: its
   own tests pass at its own head.

---

## §0. Where the spec is out of date or wrong (checked 2026-10-13)

1. **`ExportJob` has 22 full literals**, which the spec does not count. 2 are in
   app, 1 in media's own tests module and 19 in media's integration tests. 6
   more spread `..job(…)` and need no edit. The full list is in §A.
2. **`Answer` is `Copy`** (`fixtures.rs:806`), so a variant cannot own a `Vec`.
   The body variants carry a `&'static [u8]`, made with `Vec::leak`, which is
   honest for a server that already lives as long as the test binary
   (`serve`'s doc). Two variants are needed, not one. `Body` sends the bytes
   whole. `Cut` sends the headers for the whole body and then half of it before
   closing, which is the truncation case. `Stall` holds the connection open, so
   it cannot stand in for it.
3. **`pundit-app` has no dev-dependency on media's `fixtures`**
   (`app/Cargo.toml`'s `[dev-dependencies]`). `music_search.rs`'s unit tests
   need it: add `pundit-media = { path = "../pundit-media", features =
   ["fixtures"] }`, as the harness has it.
4. **`Reader` has no counting read.** `rest` (`audio.rs:487`) collects every
   sample. A ten-minute track at 48 kHz mono is about 115 MB of `f32`, which is
   too much to hold just to count it. Task 3 adds `Reader::count_rest(cancel)
   -> Result<u64, CompositeError>`, which is `rest` without the `Vec` and keeps
   `rest`'s error rules.
5. **The container cannot come from `deep-element-added` on `Reader`'s
   pipeline.** `Reader::start` (`audio.rs:302`) builds the pipeline and prerolls
   it before returning, so a signal connected afterwards has already missed the
   demuxer. Task 3 walks `pipeline.iterate_recurse()` after `start` instead,
   which finds the elements `decodebin3` keeps. **To be measured first in task
   3.** If the walk misses them, `start` gains an optional element callback.
6. **Whether `souphttpsrc` already errors on a body shorter than its
   `Content-Length` is unmeasured.** The explicit check stays either way, and
   `Answer::Cut` pins it. If souphttpsrc does error first, the test still passes,
   and the sabotage (§3) shows which of the two caught it.
7. **The Find music sheet adds a text field**, and the spec's Testing does not
   name `tests/ui/text_editing.rs`. Its sheet test says *"the three sheets that
   hold fields"* (`:249`), which becomes four, and it gains a probe for the
   genre field.
8. **`reel_music` returns a path** (`bus/export.rs:1208`) and `job` takes
   `music: Option<&Path>` (`:1238`). The credit travels with that path, as
   `ReelMusic { path, credit: Option<String> }`. `job` takes
   `Option<&ReelMusic>`, which keeps its parameter count at six.
9. **Spec Risk 5's sentence for the user guide is already there.**
   `exporting.md:78`–`:80` says a match keeps its song *"as long as the genre's
   songs and the project's folder name stay the same"*. Task 5 adds only the
   Find music paragraph.
10. **CLAUDE.md's *"No test may reach Hugging Face"* (`:175`)** grows to name
    Openverse and Jamendo. The spec lists no CLAUDE.md edit; task 5 makes three
    (§A).
11. **No CHANGELOG edit.** It is curated at release.
12. **Spec S10's "every other key is swallowed" means "never reaches the
    shortcuts".** The basket's branch, which the spec names as the model,
    **rejects** every key but the closing Esc (`app.slint:5080`–`:5086`). That is
    what lets letters reach the field. A rejected R still never reaches the
    lookup, because the branch returns first, so the spec's conclusion holds:
    no take can start from the keyboard while the sheet is open. Manual check 7
    covers it.

---

## §A. The census, 2026-10-13 at `51d3767`

Re-derive at each task (`start-task` §4).

| Changing | Sites |
|---|---|
| `ExportJob` full literals (task 1) | **22.** App: `bus/export.rs:1300`, `bus/basket.rs:497`. Media src: `composite/export.rs:1523` (its tests). Media tests: `tests/export.rs:161`, `:372`, `:475`, `:624`, `:952`, `:1190`, `:1276`, `:1356`, `:1437`, `:1536`, `:1667`, `:1774`, `:1850`, `:1934`, `:2195`, `:2390`, `:2459`, `:2765`; `tests/copy.rs:104`. The 6 that spread `..job(…)` need no edit: `tests/export.rs:2302`; `tests/copy.rs:316`, `:418`, `:503`, `:568`, `:836`. |
| `ExportJob` / `ExportDone` (task 1) | struct `media/src/composite/export.rs:124` (`tags` at `:143`); `ExportDone` `:247` (`chapter_list` at `:265`), its one literal `:483` (`chapter_list:` `:489`); `finish` `:469`; `write_chapter_list` `:555` |
| The run's log line (task 1) | `app/src/bus/export.rs:251`–`:265` gains `credits {:?}` |
| `reel_music` / `job` (task 1) | `app/src/bus/export.rs`: the per-reel block `:389`–`:399` (the call at `:392`), `job(…)` called at `:400`; `reel_music` `:1203` (returns at `:1208`); `job` `:1232` (`music` at `:1238`), its literal `:1300` (`music,` at `:1312`) |
| core modules (task 2) | `core/src/lib.rs`: `pub mod credits;` after `chapters` (`:12`), `pub mod openverse;` after `naming` (`:22`). Read, not changed: `naming::one_line` `:121`, `safe_chars` `:104`, `truncate_on_boundary` `:137`, `folder_slug` `:178`; `metadata::APP_NAME` `:30`, `UNTITLED` `:40` |
| `download.rs` (task 3) | `POLL` `media/src/download.rs:34`; `unreachable_or` `:217` (now takes `&str`), its one caller `:166`; the re-export `media/src/lib.rs:36` gains `get`. `download` (`:82`) and `check` (`:240`) **unchanged** |
| `Reader` (task 3) | `media/src/composite/audio.rs:263` (struct), `:302` (`start`), `:464` (`read`), `:487` (`rest`), `:554` (`read_all`); `Stopper` `media/src/composite/mod.rs:249` |
| Fixture server (task 3) | `media/src/fixtures.rs:806`–`:815` (`Answer`, `Copy`), `:823` (`serve`), `:855`–`:868` (`respond`'s exhaustive `match answer`: two new arms); `tone_audio` `:259` |
| `AppFiles` (task 4) | `app/src/bus/state.rs:291` (struct), `:299`–`:311` (`default_location`, literal `:300`), `:316`–`:321` (`in_config_dir`, literal `:317`), `films_dir` `:651`. **Two literals**, both in this file |
| app modules (task 4) | `app/src/lib.rs:16` (`pub mod music;`): `pub mod music_search;` after it; `music::EXTENSIONS` `app/src/music.rs:14` (read); `bus/basket.rs:602` (`file_stem`, the rule copied, not called: it is private to the basket and its budget is 200) |
| Harness (task 4) | `harness/tests/reel.rs:90` (`music` helper), `:441` (`a_reel_with_music_carries_the_song`, the shape the end-to-end test follows). `:477` writes `rock/notes.txt` into a genre and expects a refusal, which stays true with `.credit.txt` beside tracks |
| `app.slint` (task 5) | `ExportSheet`'s Music properties `:2062`–`:2071`; the Music row `:2180`–`:2207` (`Folder…` at `:2203`); the window's `export-music-*` `:4391`–`:4395`; `handle-key`'s export branch `:5064`–`:5071` (the new branch goes **ahead** of it); the export sheet's `Scrim` `:7433`–`:7464` (the new one goes after it), with its music wiring `:7447`–`:7451`; the error dialog's `Scrim` `:7653` |
| `main.rs` (task 5) | `wire_music(…)` called at `:438`; `show_music` `:1342`; `wire_music` `:1359` (`set_music_genre` `:1369`, `on_choose_music_folder` `:1372`); `wire_devices` `:2319`–`:2337` (the pattern: spawn `:2332`, return `:2334`) |
| tests/ui (task 5) | `app/tests/ui/main.rs:70`–`:84` (the `mod` list: `find_music` in order); `export_sheet.rs:557` (`NO_GENRES`), `:566` (the Music row test); `text_editing.rs:249` (*"the three sheets that hold fields"*) |
| Refusal and notice texts | **None changes.** Phase 1's two refusals (`bus/export.rs:1213`–`:1221`) are untouched. New texts live in `music_search.rs` and on the sheet |
| Exhaustive matches | `respond`'s `match answer` (task 3). No `Command`, `Event`, `UndoAction` or `KeyAction` changes |
| Prose | `CLAUDE.md:175` (*"No test may reach Hugging Face"*); `:793` (the `.chapters.txt` bullet: a `.credits.txt` bullet after it); `:1055`–`:1062` (the goals reel's music clause); `docs/book/src/guide/exporting.md:69`–`:86` (*Music under a goals reel*) |
| BACKLOG | #84: the "Next, in order" line `BACKLOG.md:27`–`:34`, the entry `:1754` (the phase 2 bullet `:1881`, revisit `:1892`) — as of this plan's own commit |

**No site** in `packaging`, `keymap.rs`, `project.rs` or `store.rs`, and no
`state.json` key.

---

## 1. The credit beside the reel

### Files

- `media/src/composite/export.rs`, `media/tests/export.rs`, `media/tests/copy.rs`
- `app/src/bus/export.rs`, `app/src/bus/basket.rs`
- `harness/tests/reel.rs`

### Steps

1. **`ExportJob::credits: Option<String>`**, after `tags`, with a doc in
   `cues`' shape. `Some(text)` writes `<output>.credits.txt`, and `None` removes
   a stale one, because `.credits.txt` is a name of ours (spec C3).
2. **`write_credits(job) -> Option<PathBuf>`** after `write_chapter_list`, as a
   copy of its shape: remove on `None`, write on `Some`, log a failure and
   return `None`. **`ExportDone::credits: Option<PathBuf>`**, filled in
   `finish` after `chapter_list`.
3. **The 22 literals** gain `credits: None`. The basket's (`bus/basket.rs:497`)
   is `None` by design, since a basket mixes no music.
4. **`ReelMusic { path: PathBuf, credit: Option<String> }`** (§0.8).
   `reel_music` returns `Option<ReelMusic>` and reads
   `<track file name>.credit.txt` with `read_to_string`. An absent or unreadable
   file is `None`.
   - When the credit is `None` and the genre folder holds any `*.credit.txt`, it
     logs `bus: <track> has no credit, but its folder holds some — a renamed
     track?` (spec C3, Risk 3). It never refuses.
   - `job` takes `Option<&ReelMusic>`, passes `.path` to the mix and `.credit` to
     `credits`.
5. **The log line** (`:251`–`:265`) gains `credits {:?}`.

### Tests

**media** (`cargo test -p pundit-media --test export`):

- 1.1 `a_credit_is_written_beside_the_film` — `credits: Some("x")` gives
  `<out>.credits.txt` holding exactly `x`, and `done.credits` names it.
- 1.2 `no_credit_removes_a_stale_credits_file` — a `.credits.txt` written
  beforehand is gone after a run with `None`, and `done.credits` is `None`.
- 1.3 `a_cancelled_run_leaves_the_credits_alone` — the stale file survives a
  cancel, in the shape of `cancel_leaves_nothing_and_keeps_an_existing_file`
  (`tests/export.rs:2281`).

**harness** (`cargo test -p pundit-harness --test reel`):

- 1.4 `a_reel_with_a_credited_song_writes_its_credits` — `a_reel_with_music_carries_the_song`'s
  setup plus `rock/song.ogg.credit.txt`; the reel's `.credits.txt` equals it.
- 1.5 `a_reel_with_an_uncredited_song_writes_none` — the same setup with no credit
  file. The reel exports, and there is no `.credits.txt`.

### Sabotage

1. `write_credits` returns early on `None` without removing → only 1.2.
2. `finish` skips `write_credits` → 1.1 and 1.4; 1.2 stays green (nothing to
   remove), and so do 1.3 and 1.5.
3. `reel_music` reads `<stem>.credit.txt` (extension dropped) instead of
   `<file name>.credit.txt` → only 1.4.
4. `job` puts `Some(String::new())` for a missing credit → only 1.5, because a
   file is written.
5. `write_credits` runs before the rename → only 1.3.

---

## 2. Core: the query, the parse, the licence words and the credit

### Files

- `core/src/openverse.rs`, `core/src/credits.rs` (new), `core/src/lib.rs`
- `core/tests/openverse.rs`, `core/tests/credits.rs` (new)
- `core/tests/fixtures/openverse_page.json` (new: a trimmed real answer, see step 5)

### Steps

1. **`openverse::LICENCES: [&str; 3] = ["cc0", "pdm", "by"]`**, the one list. Its
   doc carries spec L1's reasons, and the BY-SA decision with its date.
2. **`search_url(base, genre) -> String`**: spec S4's query. `q` is the trimmed
   text, cut to 200 characters on a char boundary and percent-encoded by a
   small `encode` here (RFC 3986's unreserved set passes, everything else is
   `%XX` of its UTF-8 bytes). There is no `page`.
3. **`Candidate`** (spec S4's fields) and **`results(json) -> Result<Vec<Candidate>,
   ParseError>`**.
   - `serde` reads only those fields, with `Option` where Openverse may send
     `null`.
   - A result is dropped when it has no `id`, `url` or `duration`, or a
     `title` that `naming::one_line` leaves empty.
   - It is also dropped when its `license` is not in `LICENCES`.
   - A missing `creator` becomes `"Unknown artist"`. `title` and `creator` are
     stored already flattened by `one_line`.
4. **`rank(&mut [Candidate], genre)`**: a stable sort, with rows whose `genres`
   contain the lower-cased genre first. **`licence_words(license, version) ->
   String`** gives *"CC BY 4.0"*, *"Public domain (CC0 1.0)"* and *"Public
   domain"*. **`licence_url(&Candidate) -> String`** is `license_url`, or
   failing that one built from licence and version (spec C2).
5. **`credits::credit_text(&Candidate) -> String`**: spec C2's three lines, with
   the fixed modification phrase in a `const`.
6. **The fixture page**: one real `/v1/audio/` answer, trimmed to six results.
   - It holds two BY results, one CC0 and one with `genres: []`.
   - Two are slipped in by hand: an NC result and a BY-SA result.
   - One has `duration: null`.
   - The `url`s read `{AUDIO}`, which task 3's tests replace. Core's tests leave
     them alone.

   Nothing identifying the coach is in it. Every name in it is a public
   Jamendo artist's.

### Tests

**core** (`cargo test -p pundit-core --test openverse --test credits`):

- 2.1 `the_query_names_the_licences_length_and_category` — the exact URL for
  `"rock"`.
- 2.2 `a_genre_is_percent_encoded_and_cut_at_200` — `"drum & bass"` →
  `q=drum%20%26%20bass`; a 300-character `é…` cut to 200 characters and still
  valid UTF-8.
- 2.3 `the_page_parses_to_its_allowed_rows` — the fixture gives exactly the BY,
  BY and CC0 rows, plus the empty-genres row, which is BY.
- 2.4 `an_nc_and_a_by_sa_row_are_dropped` — the two slipped rows are absent.
- 2.5 `a_row_with_no_duration_is_dropped`.
- 2.6 `genre_matches_rank_first_in_their_own_order`.
- 2.7 `licence_words_for_by_cc0_and_pdm`.
- 2.8 `a_by_credit_says_it_was_edited` — the three lines, with the phrase.
- 2.9 `a_missing_licence_url_is_built_from_licence_and_version`.
- 2.10 `a_title_with_a_newline_is_one_line_and_no_creator_is_unknown_artist`.

### Sabotage

1. `LICENCES` gains `"by-sa"` → 2.1 (its literal URL no longer matches) and
   2.4.
2. The licence re-check removed from `results` → only 2.4.
3. `q` not encoded → only 2.2.
4. `duration` made `Option` and kept → only 2.5.
5. `rank` filters instead of sorting → 2.6 and 2.3 (the empty-genres row
   vanishes).
6. The modification phrase dropped → only 2.8.
7. The `license_url` fallback returns `""` → only 2.9.
8. `one_line` not applied → only 2.10.

---

## 3. Media: `get`, `inspect_audio` and the fixture server's bodies

### Files

- `media/src/download.rs`, `media/src/lib.rs`
- `media/src/composite/audio.rs`
- `media/src/fixtures.rs`

### Steps

1. **Measure §0.5 first.** After `Reader::start` on `tone_audio` and on an MP3,
   does `iterate_recurse()` list `oggdemux` and `mpegaudioparse`? Write the
   answer in "Where this stands". If not, add the element callback to `start`.
2. **`get(url, limit, cancel) -> Result<Vec<u8>, GetError>`**:
   `souphttpsrc location=<url> user-agent=pundit/<version> ! appsink`, pulled to
   EOS.
   - `limit` caps the bytes held.
   - **Stall bound: no buffer for 20 s** (`STALL`) fails, checked at every
     `POLL`.
   - Cancel is polled at the same interval.
   - After EOS, where the source's byte duration is known, it must equal the
     bytes held.
   - `GetError` is `Status(u16)`, `Failed(String)` or `Cancelled`. `Status` is
     read from the error message's structured details (`http-status-code`,
     measured on 1.24.2 for 401 and 404), **never from its text**.
3. **`unreachable_or(url: &str, err)`**, with its caller at `:166` passing
   `&fetch.url`.
4. **`Reader::count_rest`** (§0.4) and **`inspect_audio(path, cancel) ->
   Result<AudioFile { seconds: f64, extension: &'static str }, CompositeError>`**
   in `audio.rs`. It decodes at 48 kHz mono, finds the container (step 1) and
   maps it: `mpegaudioparse` → `mp3`, `oggdemux` → `ogg`, `wavparse` → `wav`,
   `flacparse` → `flac`, `qtdemux` → `m4a`. Any other container fails, and so
   does no audio track. Re-export it from `lib.rs:36`'s neighbourhood.
5. **Fixtures:** `Answer::Body(&'static [u8])` and `Answer::Cut(&'static [u8])`
   (§0.2), and `vbr_mp3(path, seconds)` (`audiotestsrc ! lamemp3enc
   target=quality ! xingmux`).

### Tests

**media** (`cargo test -p pundit-media --lib download` and `--test` for the
audio checks; place them beside the existing `download.rs` tests and in
`tests/export.rs`'s audio neighbourhood):

- 3.1 `get_returns_the_body` — `Body` round-trips byte for byte.
- 3.2 `get_refuses_past_its_limit` — a 1 MB body with a 512 KB limit fails.
- 3.3 `get_gives_up_on_a_stall` — `Stall` fails within `STALL + POLL`, with
  `STALL` lowered by a `#[cfg(test)]` constant.
- 3.4 `get_refuses_a_short_body` — `Cut` fails (§0.6).
- 3.5 `get_reads_the_status_from_the_details` — `Status("429 Too Many
  Requests")` gives `Status(429)`, and so does `Status("429 Slow Down")`.
- 3.6 `get_cancels` — a set flag returns `Cancelled` within `POLL` of setting it.
- 3.7 `inspect_names_ogg_mp3_and_wav` — `tone_audio`, `vbr_mp3` and a `wavenc`
  file give their extension and a length within 0.1 s.
- 3.8 `a_truncated_vbr_mp3_decodes_short` — `vbr_mp3` of 10 s cut to half its
  bytes reads under 6 s.
- 3.9 `an_html_page_is_not_audio` — fails.

### Sabotage

1. The length check after EOS removed → 3.4, unless souphttpsrc errors first;
   the result is recorded under §0.6.
2. `Status` parsed from the message text → only 3.5's second case.
3. The stall bound removed → only 3.3, which times out.
4. `count_rest` stops at the first buffer → 3.7 and 3.8.
5. `oggdemux` mapped to `opus` → only 3.7.
6. `limit` not enforced → only 3.2.

---

## 4. App: `music_search.rs`, the default folder and the end-to-end test

### Files

- `app/src/music_search.rs` (new), `app/src/lib.rs`, `app/Cargo.toml` (§0.3)
- `app/src/bus/state.rs`
- `harness/tests/reel.rs`

### Steps

1. **`Genre::resolve(folder: Option<&Path>, text) -> Result<Genre, String>`**
   (spec S5).
   - An empty slug is `Err("Type a genre to search for")`.
   - Otherwise it is an existing subfolder whose `folder_slug` equals the
     text's, under the folder's own name, or the slug itself.
   - `Genre` holds the folder name and the query text.
2. **`search(base, &Genre) -> Result<Vec<Candidate>, String>`**: `get` of
   `search_url` with a 1 MiB limit, then `results`, then `rank`. The
   `GetError`s become spec S11's words. `429` is matched on `Status(429)`.
3. **`add(&Candidate, folder, &Genre, cancel) -> Result<PathBuf, String>`**
   (spec S7):
   - `get` with a 64 MiB limit;
   - write `<genre>/.<id>.part`;
   - `inspect_audio`, then the 2 s check against `duration`;
   - the name, from spec S8's rule, as a `track_stem` here, in `file_stem`'s
     shape (§A);
   - then **`land(part, dest, credit, rename)`**, where `rename` is a seam
     (`fn(&Path, &Path) -> io::Result<()>`, production passes
     `std::fs::rename`). `land` writes the credit through a temp file and a
     rename, renames the track, and deletes the credit if the track rename
     fails.
   - Every failure deletes the `.part`.
4. **`already_in(&Candidate, folder, &Genre) -> bool`**: the file this candidate
   would land as exists (spec S8).
5. **`AppFiles`**: `music: PathBuf` beside `films`, with `default_location`
   using `glib::UserDirectory::Music` through `music_dir(music, home)` in
   `films_dir`'s shape. `in_config_dir` uses `config_dir.join("music")`.
   `pub fn default_music_folder(&self) -> PathBuf`.

### Tests

**app lib** (`cargo test -p pundit-app --lib music_search` and `state`):

- 4.1 `search_returns_the_ranked_rows` — two servers (spec Testing): the page
  with `{AUDIO}` replaced, served by `Body`.
- 4.2 `a_429_says_the_anonymous_limit`.
- 4.3 `add_lands_the_track_and_its_credit` — `<rock>/<title> — <creator>.ogg`
  and `.ogg.credit.txt` holding `credit_text`, with no `.part` left.
- 4.4 `a_failed_rename_leaves_no_credit` — the `land` seam fails the track
  rename.
- 4.5 `a_short_track_is_refused_and_leaves_nothing` — the served file's decode
  is more than 2 s off the candidate's `duration`.
- 4.6 `a_stem_taken_by_another_file_gets_2`.
- 4.7 `a_deleted_track_can_be_added_again` — the orphan credit is overwritten,
  and `already_in` is false before and true after.
- 4.8 `a_genre_matches_an_existing_folder_by_slug` — `" Rock"` lands in an
  existing `Rock/`.
- 4.9 `an_empty_genre_is_refused_before_any_request` — the base URL is
  `http://127.0.0.1:9/`, which nothing serves, and the error is the empty-genre
  one.
- 4.10 `the_default_music_folder_follows_films_dirs_rule` (`state.rs`'s tests
  beside `films_dir`'s, `:771`–`:780`).

**harness** (`cargo test -p pundit-harness --test reel`):

- 4.11 `a_song_found_by_search_reaches_the_reel_with_its_credit` —
  `music_search::add` into the scratch folder, `music::genres` lists the genre,
  and the reel export writes `.credits.txt` equal to `credit_text`.

### Sabotage

1. `land` writes the credit after the rename → only 4.4.
2. The 2 s check removed → only 4.5.
3. `already_in` scans for a credit instead of the track → only 4.7.
4. `resolve` compares names, not slugs → only 4.8.
5. The empty-slug refusal moved after the request → only 4.9, which then fails
   on the connection error.
6. `add` lands in `folder_slug(text)` and ignores `Genre`'s name → 4.8 and 4.11
   still pass (same slug), so **4.8 asserts the existing folder's
   capitalisation**, and then fails.
7. `in_config_dir`'s music under the real home → only 4.10.

---

## 5. The Find music sheet, its keys, the wiring and the prose

### Files

- `app/ui/app.slint`, `app/src/main.rs`
- `app/tests/ui/find_music.rs` (new), `tests/ui/main.rs`, `tests/ui/text_editing.rs`
- `CLAUDE.md`, `docs/book/src/guide/exporting.md`, `docs/hands-on-checklist.md`, `BACKLOG.md`

### Steps

1. **`FindMusicSheet inherits Sheet`**:
   - the genre `LineEdit` and **Search**;
   - a `for` over `rows: [MusicRow]` (`id`, `title`, `artist`, `length`,
     `licence`, `genres`, `state`: `""` / `"Adding…"` / `"In rock"`), with Add
     `enabled: row.state == "" && !root.adding`;
   - the message line;
   - Openverse's line and spec L3's line;
   - **Close**.

   The window gets `find-music-open`, `find-music-genre`, `find-music-rows`,
   `find-music-adding`, `find-music-message`, and callbacks `search-music(string)`,
   `add-music(string)` and `close-find-music()`. The `Scrim` goes after the
   export sheet's (§A).
2. **The Music row** gains `Button { text: "Find music…"; }` before `Folder…`,
   with `find-music()` forwarded through `ExportSheet` like
   `choose-music-folder`.
3. **`handle-key`**: a `find-music-open` branch **ahead of** the export
   sheet's, in the basket's shape:
   `if (pressed && event.text == Key.Escape && !root.text-editing) {
   root.close-find-music(); return accept; } return reject;` — the basket's
   branch (`app.slint:5080`–`:5086`) exactly (§0.12).
4. **`main.rs`: `wire_find_music`**, beside `wire_music`:
   - `OPENVERSE: &str = "https://api.openverse.org/v1/"`.
   - A `Rc<Cell<u64>>` generation, and a `Rc<RefCell<Option<(Genre,
     Vec<Candidate>)>>>` for the rows on screen.
   - **Search:** `Genre::resolve`; an `Err` goes to the message line. Otherwise
     bump the generation, spawn, run `search`, return through
     `upgrade_in_event_loop`, and drop the answer if the generation moved.
   - **Add:** if `music_folder()` is `None`, write
     `default_music_folder()` with `set_music_folder`. Set the row to
     *"Adding…"* and `adding`, spawn `add` with a fresh `Arc<AtomicBool>` kept
     for Close, and on return set the row and the message, and remember the
     genre added.
   - **Close:** set the cancel flag. If a genre was added, call
     `set_music_genre(Some(name))` and then `show_music` (spec S6). Close the
     sheet.
   - Opening prefills the field from the picker's genre.
5. **`CLAUDE.md`**:
   - `:175` becomes *"No test may reach Hugging Face, Openverse or Jamendo"*;
   - a `.credits.txt` bullet after `:793`'s, naming `ExportJob::credits` and the
     removal rule;
   - the goals reel's clause (`:1055`–`:1062`) gains one sentence: Find music
     fetches into the same folder, off the bus, with each track's credit beside
     it.
6. **The guide**: a *Finding music* paragraph in `exporting.md`'s music section
   (§0.9), naming CC0/CC BY only, the credits file, and Openverse's line.
   **The hands-on checklist** gains the manual checks below.
7. **BACKLOG #84**: shipped, with the PR number. Its "Next, in order" line
   leaves.

### Tests

**tests/ui** (`cargo test -p pundit-app --test ui find_music`, one binary;
`find_music` is a `mod` of `tests/ui/main.rs`):

- 5.1 `find_music_is_reached_from_the_music_row` — with a reel ticked, the
  accessible action on *Find music…* reaches `find-music`.
- 5.2 `the_sheet_says_openverse_and_the_licence_caveat` — both lines are found
  by label.
- 5.3 `add_reaches_its_row_and_the_others_wait` — the Add on row 2 reaches
  `add-music("id2")`. With `find-music-adding` set, every Add reports
  `accessible-enabled` false.
- 5.4 `esc_in_the_genre_field_leaves_both_sheets_open_and_the_next_closes_find_music_only`
  — Esc in the field: both open. Esc again: `close-find-music` fired and
  `export-sheet-open` is still true.
- 5.5 `text_editing.rs`: the sheet probe gains Find music's genre field, and
  *"three sheets"* becomes four.

**`main.rs` is unreachable from any test** (`[[bin]]`; Risk 1). Its wiring is
manual checks 1–5.

### Sabotage

1. The Find music branch put **after** the export sheet's → 5.4 (the first Esc
   closes the export sheet).
2. The branch's `!root.text-editing` dropped → 5.4 (the first Esc closes Find
   music).
3. Add's `enabled` ignores `adding` → only 5.3.
4. The Music row's button wired to `choose-music-folder` → only 5.1.
5. The `LineEdit` replaced by a `Text` → 5.5.

---

## Manual checks (batched at the end)

On a project with tagged goals, a reel ticked, and **no music folder set**:

1. **First search.** Find music…, type `rock`, Search. Up to twenty rows
   appear, with rock-tagged rows first, and every licence reads CC BY or Public
   domain. Openverse's line is shown.
2. **First Add.** Add one. The row says *Adding…*, the other Adds grey, then
   *In rock*. `~/Music/pundit/rock/` holds the track and its `.credit.txt`, and
   Folder… now shows that folder.
3. **Close selects it.** Close: the Music picker reads `rock`. Export: the reel
   has the song, and `<reel>.credits.txt` holds the credit with the modification
   phrase.
4. **A stale answer.** Search `rock`, then at once `jazz`. Only jazz rows show.
5. **Close mid-Add.** Close while a row is adding. Nothing is left in the folder,
   no `.part` included (`ls -a`).
6. **Offline.** Turn the network off and Search. The sheet's line names
   `api.openverse.org`.
7. **Keys.** In the field, typing `r` types an `r` and starts no recording. Esc
   leaves the field, a second Esc closes Find music, and the export sheet is
   still open.
8. **A renamed track.** Rename the added track, re-export the reel. There is no
   `.credits.txt`, and stderr has the *"renamed track?"* line.

---

## Risks

1. **The `main.rs` wiring is untested** (spec Risk 1): the thread, the
   generation, the default folder's write and the re-selected genre. Each is a
   short mapping over tested pieces, and manual checks 1–5 cover them.
2. **§0.5 can fail.** If `iterate_recurse` does not see `decodebin3`'s inner
   elements, `Reader::start` gains a callback, which is a wider change to a
   shared reader. Measured first, in task 3 step 1.
3. **Task 1's 22 literals** are mechanical. The risk is a literal that should
   carry a credit and does not. Only `bus/export.rs:1300` should, and 1.4 pins
   it.
4. **The fixture page is a snapshot of Openverse's shape.** A renamed field
   upstream breaks the app and not the tests. Spec Risk 8 already accepts that
   Openverse can change, and the parse reads only eleven fields.

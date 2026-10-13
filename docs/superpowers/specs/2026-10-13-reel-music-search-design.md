# Design — music under a goals reel, phase 2: an Openverse search, and credits (BACKLOG #84)

The coach (2026-09-25): *"what i'd rather have is like some interface to a free
music search? like free music archive or similar"*. Phase 1
(`2026-10-12-reel-music-design.md`) shipped the mixer's half: a music folder with
a subfolder per genre, a Music row on the export sheet, and one track chosen per
match and mixed under the reel. It left two things to this spec (its §M5 and
"What this does not do"): **the search that fills the folder**, and **the credit
each track it fetches is owed**.

Read `CLAUDE.md` and #84 first. #84 records the decisions this rests on:
Openverse, anonymous, as the catalogue (2026-09-25); the fetcher lands tracks
where the mixer already reads, so **the export path never learns that a
catalogue exists**; the credit is an output written beside the film the way
`.chapters.txt` is; and the model download's network discipline is the
precedent. Phase 1's §R2 moved the landing place from a cache to the music
folder (`<folder>/<genre>/`), because a cache can be evicted and a credit with
it.

Every `path:line` below was re-derived from `main` at `f27cb38`. Every claim
about Openverse was **measured against the live API on 2026-10-13** unless it
says otherwise, and the sources are listed under §Sources.

**The coach decided the six product questions on 2026-10-13**, each as
recommended; they are recorded at the end under *Decided with the coach*, and
the design below is written to those answers.

**The design, in one paragraph.** A **Find music…** button in the export sheet's
Music row opens a modal sheet of its own. It has a genre field, **Search**, and
up to twenty results (title, artist, length, licence, genres), each with
**Add**. Search is one `GET` of Openverse's `/v1/audio/`, for music of 2–10
minutes under **CC0, the public-domain mark or CC BY**. Core builds the request,
parses the answer and re-checks the licences. **Neither Search nor Add goes
through the bus.** Both are functions of a library module, `music_search.rs`,
which `main.rs` runs on a short-lived thread, as it lists the Devices popover's
devices. Add fetches the file into memory through `souphttpsrc`, writes it to a
`.part`, decodes all of it to check it, writes the credit as
`<track>.credit.txt`, and renames the track into `<musicFolder>/<genre>/` last.
Phase 1's picker then lists the genre. When a reel's chosen track has a credit,
the bus reads the text into the job, and `finish` writes `<film>.credits.txt`
beside the reel, as it writes `.chapters.txt`. Nothing reaches the network until
the coach presses Search, no test reaches Openverse, and nothing in the project
format changes.

---

## §O. What Openverse actually offers (measured 2026-10-13)

**O1. The endpoint.** `GET https://api.openverse.org/v1/audio/` with, among
others, `q` (≤ 200 characters), `license` (a comma list of `by`, `by-nc`,
`by-nc-nd`, `by-nc-sa`, `by-nd`, `by-sa`, `cc0`, `nc-sampling+`, `pdm`,
`sampling+`), `license_type` (`all`, `all-cc`, `commercial`, `modification`),
`category` (`audiobook`, `music`, `news`, `podcast`, `pronunciation`,
`sound_effect`), `length` (`shortest`, `short`, `medium`, `long`), `page`,
`page_size`, `mature`, `extension`, `source`. The parameter list is the API's own
OpenAPI schema (`/v1/schema/`). **There is no genre parameter.**

**O2. Each result carries** `id` (a UUID), `title`, `creator`, `creator_url`,
`license`, `license_version`, `license_url`, `foreign_landing_url` (the track's
page at its source), `url` (the file), `provider`/`source`, `category`,
`genres` (a list, often empty), `tags`, `duration` (milliseconds), `filetype`,
`filesize`, `attribution` (a ready-made sentence), `mature`, and more. Measured
on a Jamendo result: **`filesize` is `null` and `filetype` is `"mp32"`**, which is
no extension a player knows; the `url` (`prod-1.storage.jamendo.com/?trackid=…&format=mp32`)
answers `200`, `audio/mpeg`, with a `Content-Length`.

**O3. The limits, read off the response headers:** `x-ratelimit-limit-anon_burst:
20/min` and `x-ratelimit-limit-anon_sustained: 200/day`. **An anonymous
`page_size` above 20 is refused** — `401 {"detail":"page_size may not exceed 20
for anonymous requests"}` — and the result depth is capped too: every broad query
reported `result_count: 240`, `page_count: 12` (12 × 20). A registered client
(OAuth2 client credentials, `POST /v1/auth_tokens/register/` then
`/v1/auth_tokens/token/`, then `Authorization: Bearer …`) gets 100/min and
10000/day **in the repository's defaults** (`api/conf/settings/rest_framework.py`);
production's values were not measured. An unverified registration gets the
anonymous limits.

**O4. `license_type` is an intersection, and it lets `sampling+` through.**
`validate_license_type` (`api/api/serializers/media_serializers.py`) intersects
the groups, and `commercial` is "no `nc` in the name", `modification` "no `nd`"
(`api/api/constants/licenses.py`). So `commercial,modification` is `{cc0, pdm,
by, by-sa, sampling+}` — and Sampling Plus is a retired licence this app has no
business reasoning about. **The search names licences with `license`, never
`license_type`.**

**O5. `length` buckets, measured** (music, the four buckets, 20 results each):
`shortest` 10–28 s, `short` 43–119 s, `medium` 128–436 s, `long` 632–1812 s —
consistent with < 30 s, 30 s–2 min, 2–10 min, > 10 min. The bucket boundaries
were not found in the API's source; the measured ranges are what this spec
relies on.

**O6. `q` matches titles, not genres.** Every result of `q=rock`, `q=edm`,
`q=electronic` and `q=eurodance` reported `fields_matched: ["title"]`. Of the
first twenty music results of `q=rock` (CC0/PDM/BY/BY-SA, medium), **7** had
`rock` in `genres`; of `q=electronic`, 8; of `q=dance`, 8. `tags=rock` found 5
results in all, because Jamendo's genres are not its tags. **#84 predicted this**
(*"genre lives in tags… Jamendo's own API has explicit genre, mood and tempo
filters"*); it is now measured, and decision 2 accepts it.

**O7. `category=music` is Jamendo in practice.** Every `category=music` result
in every query above came from `jamendo`; without it, `q=rock` under CC0/BY
returned Freesound clips (category `null`). **`/v1/audio/stats/` lists three
sources today — Jamendo, Wikimedia Commons, Freesound** — not the five #84
recorded (§R1).

**O8. The licence filter costs most of the catalogue.** The same `q`, music,
medium: `rock` reports 240 (the cap) with BY-SA and **58** without; `electronic`
190 and **36**; `dance` 240 and **177**. In the first twenty with BY-SA allowed,
16–18 were BY-SA. This is decision 3's cost.

**O9. Openverse's terms** (`docs.openverse.org/terms_of_service.html`): no
scraping; respect the rate limits; the licence data is **not** warranted —
Openverse *"does not verify its licensing status or make any representations or
warranties"*, and the user is *"responsible for independently verifying"*; and an
app using the API must *"prominently indicate that it was made using Openverse
but is not endorsed or certified by Openverse"* (S2).

**O10. `souphttpsrc` reaches it.** `gst-launch-1.0 souphttpsrc
location='https://api.openverse.org/v1/audio/?…' ! filesink` returned the JSON
on this laptop, with GStreamer's default User-Agent and with one set. So the
search, like the download, adds no crate (S7).

## §L. Which licences, and why

**L1. Synchronisation is the right in question, and CC grants it** — as an
*adaptation*. CC BY-SA 4.0 §1(a): where the material is a musical work or sound
recording, *"Adapted Material is always produced"* when it is *"synched in timed
relation with a moving image"*; 3.0 §1(a) says the same of *"synching"*. Every
4.0 licence carries that definition. So:

- **ND is excluded by the licence text**: a reel *is* an adaptation, which ND
  forbids. Not a policy choice.
- **NC is excluded by policy**: a coach may upload to a channel that is or
  becomes monetised, and NC turns that into a breach after the fact.
- **CC0, PDM and CC BY are allowed.** BY asks for credit, which §C provides.
- **CC BY-SA is excluded** (decision 3): its ShareAlike clause
  (4.0 §3(b)(1): *"The Adapter's License You apply must be a Creative Commons
  license with the same License Elements"*) makes **the reel itself** CC BY-SA —
  the coach's footage of identifiable players, released for anyone to reuse,
  commercially included. That is the club's decision to make, not a default.

**L2. The filter is enforced twice.** The query names the licences L1 allows
(S4, and §O4 for why not `license_type`), and core **re-checks each result's
`license` field** against the same `const` list before a row is shown — because the app
promises the coach the licence, and a server-side filter is the server's
promise, not ours. A result whose licence is not in the list is dropped, not
shown greyed.

**L3. What the app does not promise.** Openverse's licence data is unwarranted
(§O9), so the sheet says once, under its results: *"Licences are as the
catalogue reports them — check the track's page before you publish."* The page
is in the credit (§C2), so the check is one click from the file the coach is
about to paste anyway.

## §S. Decisions

**S1. Where it lives: a Find music sheet, opened from the Music row** (decision
1). The row (`app.slint:2149`–`:2176`) gains **Find music…** beside
**Folder…**. It opens a sheet of its own, a `Scrim` declared after the export
sheet's (`:7343`), so it stacks over it the way the error dialog does (`:7563`).
It does not go on the Settings sheet #131 is building. That sheet is for
machine-wide values with nothing to sit beside (its §U7: *"the next tenant is one
row"*), and the search fills the very folder this picker reads, which is phase
1's §M6 argument for putting the folder path here. It is not inline in the
export sheet either: a results list would double a sheet that already scrolls
(#134), and every switch in that sheet describes the next export, which a search
does not.

**S2. The sheet's contents.**

- A **genre** field, prefilled with the Music picker's genre (or empty), and
  **Search**.
- **One page of up to twenty rows**: title, artist, `m:ss`, the licence in words
  (S4's one function), and the result's `genres`. Rows whose genres include the
  searched word come first and the rest follow, each group in Openverse's order.
  This ranks rather than filters: a strict genre filter keeps 7 of 20 (§O6), and
  a rock song with no genres listed is still a rock song. **There is no More and
  no paging.** A different word does the same job, and paging would add state
  and a second request per look.
- **Add** on each row. While one Add is in flight its row says *"Adding…"* and
  **every other Add is disabled**: one at a time, with no queue and no progress
  figure. Afterwards the row says *"In rock"*. A result whose file is already in
  the genre folder (S8) shows *"In rock"* from the start.
- A message line. It belongs to the sheet, because the status bar renders behind
  the scrim (`CLAUDE.md`, the basket sheet's reason).
- **The attribution Openverse's terms require, always visible:** *"Search by
  Openverse. pundit is not endorsed or certified by Openverse."* — and L3's line.
- **Close.** There is no Cancel, because an added track stays added. Close does
  cancel an Add still in flight (S10).

**S3. Search and Add are functions, not commands.**
`pundit-app/src/music_search.rs`, a library module beside `music.rs`
(`lib.rs:16`), holds two functions:

```rust
pub fn search(base: &str, genre: &Genre) -> Result<Vec<Candidate>, String>;
pub fn add(candidate: &Candidate, folder: &Path, genre: &Genre,
           cancel: &AtomicBool) -> Result<PathBuf, String>;
```

- **`main.rs` runs each one on a short-lived thread** and returns the answer
  with `upgrade_in_event_loop`. That is `wire_devices`' pattern
  (`main.rs:2278`–`:2295`): it spawns at `:2290` and comes back at `:2293`.
- **The base URL is a `const` in `main.rs`** (`https://api.openverse.org/v1/`),
  passed to `search` as an argument. No other file names the host. Tests pass a
  local server's URL.
- **The bus is not involved.** It has no command, no event, no store of
  candidates and no catalogue setting. Nothing the bus does waits on the
  network. A whole layer of permission is unnecessary because no command can
  carry a URL. The only URL `add` fetches is the `url` field of a `Candidate`
  that `search` parsed.
- **The staleness rule is a generation counter.** A `Cell<u64>` in `main.rs` is
  bumped by each Search, and an answer that comes back under an older generation
  is dropped. A search is never cancelled; a superseded one just finishes and is
  ignored.

**S4. The query, and the parse, are core's** (`core::openverse`), as pure
functions:

```text
GET <base>audio/?q=<genre text>&category=music&length=medium&license=cc0,pdm,by
    &mature=false&page_size=20
```

- **`q` is the field's trimmed text**, percent-encoded by a small encoder in
  that module (core has no URL crate and wants none) and cut to 200 characters,
  the API's own bound.
- **`length=medium`** (2–10 min, §O5). A reel of six goals at phase 1's 20 s +
  6 s is about 2.6 minutes. A `short` track would end before most reels do and
  leave the crowd sound alone, and `long` results are mostly mixes and sets.
- **`category=music`**, so that a search for "rock" does not return a recording
  of a rock being dropped (§O7).
- **`page_size=20`**, the anonymous ceiling (§O3).
- **`results(json) -> Result<Vec<Candidate>, ParseError>`** reads only the
  fields the app uses: `id`, `title`, `creator`, `creator_url`, `license`,
  `license_version`, `license_url`, `foreign_landing_url`, `url`, `duration`
  and `genres`. A field added upstream therefore costs nothing.
  - A result with **no `id`, `title`, `url` or `duration`** is dropped.
    `duration` is required because the check in S7 measures against it.
  - A missing `creator` reads as *"Unknown artist"*.
  - Then comes L2's licence check, and S2's ranking.
- **One licence-to-words function**, `openverse::licence_words(license,
  version)`, gives *"CC BY 4.0"*, *"Public domain (CC0 1.0)"* and *"Public
  domain"*. The result row and the credit (C2) both read it.

**S5. A genre is matched by its slug, and its name comes from the folder.**

- **`Genre`** is the folder name a search lands in, decided **when Search is
  pressed**. The field is trimmed and `naming::folder_slug`ged
  (`naming.rs:167`). An empty slug refuses the search on the sheet's line
  (*"Type a genre to search for"*) before any request is made.
- **An existing genre folder whose `folder_slug` equals the field's** is used
  under its own name. So *"Rock "* finds `rock/`, and so does *"ROCK"*. Failing
  that, the slug is the new folder's name, which also means a `/` cannot climb
  out of the music folder.
- **An Add lands in the genre its search was made under**, not in whatever the
  field says now. The `Genre` travels with the results.

**S6. The default folder belongs to `AppFiles`, and the window writes it.**

- `AppFiles` (`bus/state.rs:291`) gains a `music` path beside `films`, resolved
  the same way: `default_location` (`:299`) uses glib's `UserDirectory::Music`
  joined with `pundit`, falling back to `$HOME/Music` as `films_dir` (`:651`)
  falls back to `$HOME/Videos`. `in_config_dir` (`:316`) uses a scratch path, so
  no test writes into the coach's own music folder.
- **The window resolves and writes it.** When an Add finds no `musicFolder`
  set, `main.rs` takes `AppFiles::default_music_folder()`, writes it with
  `set_music_folder`, and passes it to `add`. The bus stays a reader of both
  music keys, which is phase 1's rule (§M2 there) and `CLAUDE.md`'s. A folder
  that is already set is never changed by the search. **Folder…** still chooses
  another (decision: `~/Music/pundit`).
- **When the sheet closes after an Add, the picker re-lists and selects the
  genre added last**, written to `musicGenre` through the same `set_music_genre`
  the picker uses (`main.rs:1349`). This is needed, not cosmetic. The bus reads
  `musicGenre` at job build (`bus/export.rs:393`–`:394`), so a coach who picks
  nothing on Find music, searches, adds, closes and exports would otherwise get
  Music: None, which is phase 1's "no music" and not a refusal. With no Add,
  closing changes nothing.

**S7. Add: into memory, to a `.part`, checked, credited, renamed.**

1. **Fetch.** `download::get(url, limit, cancel)` is new in `pundit-media`, the
   transport the search also uses. It is `souphttpsrc ! appsink` read to EOS
   into memory.
   - **The cap is 64 MiB.** A measured 3:24 track was 4.5 MB, and a twenty-row
     page is 35–39 KB.
   - **The bound is a stall, not a deadline:** no new bytes for 20 s fails the
     request. A slow but moving transfer is never cut off.
   - **The cancel is `download`'s cancel**, polled at the same interval
     (`POLL`).
   - **Truncation:** where `souphttpsrc` reports a byte length (the server's
     `Content-Length`, queried as the source's duration in bytes), the bytes
     received must equal it.
   - **HTTP failures are read from the error's structured details, never from
     its text.** Measured on 1.24.2, an error from `souphttpsrc` carries
     `details, http-status-code=(uint)401` (and `404` for a missing path). So a
     429 is matched on that field, and gets S11's words.

   `download()` (`download.rs:82`) and its `check` are left exactly as they
   are. The model needs its exact length and sha256, and nothing here has
   either.
2. **Write `<genre folder>/.<openverse id>.part`.** It is hidden and named by the
   id, so two different results can never share it, and it has no track
   extension, so `music::tracks` never lists it.
3. **Check it by decoding it, all of it**, with `pundit-media`'s
   `inspect_audio(path, cancel)`. That function is built on the mixer's own
   `Reader` (`composite/audio.rs:263`, the `read_all` shape at `:554`), and it
   counts samples to EOS.
   - **Pass:** the decoded length is within 2 s of the candidate's `duration`.
   - **Why a full decode and not a duration query:** a parser's estimate is not
     what the mixer will get, and a full decode is. Measured on 1.24.2, with an
     encoded VBR MP3 (`lamemp3enc ! xingmux`) of 10.03 s cut to half its bytes,
     the decode counted 4.94 s. In this case the duration query also said about
     5 s, so the Xing over-report was not reproduced. The check still does not
     rely on the query.
   - **The same pass names the container.** `deep-element-added` on the pipeline
     sees the first parser or demuxer that `decodebin3` plugs, and it measured:
     `mpegaudioparse` → `mp3`, `oggdemux` → `ogg`, `wavparse` → `wav`, with
     `flacparse` → `flac` and `qtdemux` → `m4a` by the same rule. Those
     extensions are phase 1's `EXTENSIONS` (`music.rs:14`).
   - **What fails:** anything else plugged, nothing plugged at all (an HTML page
     served as `200` plugged nothing and decoded 0 s), or a length outside the
     2 s. Each refuses with *"not a track pundit can play"* and deletes the
     `.part`.
   - **No separate `typefind`**, and so nothing new on the smoke test's list.
4. **Write the credit**, `<final file name>.credit.txt` (C1), through a temp file
   and a rename.
5. **Rename the `.part` to the track's name, last.** The rename is what makes the
   track exist to `music::tracks`, so any track the picker can choose already has
   its credit. If the rename fails, the credit just written is deleted.
   - The landing step (steps 4–5) is one function that takes a rename seam, so
     a test can make the rename fail.
   - A crash between steps 4 and 5 leaves an orphan `.credit.txt`, which does no
     harm, and a re-Add overwrites it.

**S8. Names.** The track's stem follows the basket's film-name rule
(`bus/basket.rs:602`, `file_stem`). The input is `<title> — <creator>`. It is
trimmed, passed through `naming::safe_chars`, cut on a boundary at **120 bytes**,
and replaced by `Untitled` (`metadata::UNTITLED`) when it comes out empty or
begins with a `.`.
- **A different file with the same stem gets `" (2)"`**, `" (3)"` and so on,
  before the extension.
- **"Already in rock" means that file exists**, judged by the name this
  candidate would get. It is not a scan of the credits. So a deleted track can
  be added again, and the orphan credit beside its name is overwritten.

**S9. Nothing is automatic.** There is no request at startup, when the sheet
opens or while the coach types. The first network traffic is the coach's Search,
which is transcription's rule of *"asked for, never automatic"*.

**S10. The sheet is modal, and its keys are its own.**
- **`handle-key` gets a Find music branch *ahead* of the export sheet's**
  (`app.slint:5017`–`:5024`), because this sheet is drawn over that one.
  - **Esc while not `text-editing`** closes Find music only.
  - **Esc in the genre field** leaves the field and closes nothing. That is the
    basket sheet's cascade (`:5025`–).
  - **Every other key is swallowed**, so R cannot start a recording while the
    sheet is open.
- So an Add never overlaps a take that started from the keyboard, and there is
  no recording rule to state. **Closing the sheet cancels an Add in flight.** A
  track is a few megabytes (4.5 MB measured for 3:24), so re-adding costs
  seconds, and a cancel leaves nothing behind (S7's cleanup).

**S11. Failures** are shown on the sheet's line for a search, and on the row for
an Add, which offers Add again.
- Offline, DNS and TLS failures use `unreachable_or`'s host wording
  (`download.rs:217`). That function changes to take the URL rather than a
  `Fetch`, so `get` and `download` share it.
- **429:** *"Openverse allows 20 searches a minute and 200 a day without an
  account — try again shortly."*
- Any other non-200 status, or JSON that will not parse: *"Openverse answered
  with something pundit can't read (…)"*.
- There is no retry anywhere. Pressing the button again is the retry, as the
  model download's doc says of Transcribe.
- Quitting during an Add leaves a hidden `.part`, which the next Add of that id
  truncates.

## §C. Credits

**C1. The credit is worded at Add and stored as text beside the track:
`<track file name>.credit.txt`**, for example `Song — Artist.mp3.credit.txt`.
- **The wording is `core::credits::credit_text(&Candidate)`.** The credit is the
  one moment the app knows the licence, so the words are fixed then.
- **Its extension keeps it out of the listing by construction**: `.txt` is not
  in `EXTENSIONS`, so `music::genres` and `music::tracks` read it as nothing.
- **A coach can write one by hand for his own tracks**, and it reaches the
  export the same way.

**C2. The words** give title, author, source and licence (TASL), with links,
ready to paste into a description:

```text
Music (edited and mixed under the footage): "I want you (pop rock remix)" by Lollita
https://www.jamendo.com/track/1015339
Licensed under CC BY 3.0 — https://creativecommons.org/licenses/by/3.0/
```

- **The fixed phrase *"(edited and mixed under the footage)"* is CC BY's
  modification notice.** BY 4.0 §3(a)(1)(B) asks that a change be indicated, and
  a reel cuts, ducks and fades the track. Core's test for BY pins the phrase.
- **The licence line is S4's `licence_words`.** CC0 and PDM keep the credit
  with their own words: nothing is owed, but it costs nothing and tells a viewer
  where the song came from.
- **A null `license_url` falls back to one built from licence and version**
  (`https://creativecommons.org/licenses/<license>/<version>/`, and the
  `publicdomain/` forms for CC0 and PDM).
- **Title and creator are flattened to one line** by `naming::one_line`, which
  is today `chapters.rs:55`'s private `one_line` moved to `naming` and made
  public, with chapters calling it there.
- **Openverse's own `attribution` string is not used.** It has no link to the
  track, which BY asks for where reasonably practicable (4.0 §3(a)(1)(A)(v)),
  and its wording is not ours to keep stable.

**C3. The bus reads the text when it builds the job; media writes it in
`finish`.** `reel_music` (`bus/export.rs:1203`) already returns the chosen
track's path. Beside it, the bus reads `<track>.credit.txt` into
**`ExportJob::credits: Option<String>`**.
- It is read at job build, so a queued job keeps the credit of the track it
  froze (`CLAUDE.md`'s queue: *"a queued row is a snapshot"*).
- An absent credit is `None`, which is never a refusal. The coach's own tracks
  usually have none.
- **When the chosen track has no credit but its genre folder holds some
  `.credit.txt`, the bus logs it.** That pattern is what a renamed fetched
  track looks like (Risk 3). It is logged beside the run's existing `bus:
  exported …` line (`bus/export.rs:251`–`:265`) and not shown in the export
  outcome. That outcome carries no per-target notes today (`TargetState::Done`
  holds a path), and a sidecar that failed to write is not surfaced there
  either.
- **`finish` (`composite/export.rs:439`) gains `write_credits(job)`** after
  `write_chapter_list` (`:525`), under the same rules:
  - it writes `job.path.with_extension("credits.txt")`, so it inherits the run's
    name cleaning and `" (2)"`;
  - it runs after the rename, so a cancelled or failed run neither writes nor
    removes anything;
  - a failure is logged, reported as `ExportDone::credits: None`, and never
    fatal.

  **Like `.chapters.txt`, it is a name of ours**, so a job carrying `None`
  **removes** a stale file. A reel re-exported with Music: None must not leave
  the old song's credit beside it.
- **`pundit-media` learns nothing about catalogues or licences.** It receives a
  string and writes it, as `cues` is all it knows of the scoreboard picker.

**C4. One file per output, not one combined description.** A single
`<film>.description.txt` with both the chapters and the credit would be one
paste instead of two. But `.chapters.txt` has its own rule, no file under three
surviving chapters (`CLAUDE.md`), which a credit must not inherit. A third
sidecar is cheaper than one file with two rules for whether it exists.

**C5. No credit in the picture or in the file's tags** (decision 4). In-file, it
would be the `description` tag, which on this muxer lands only in the XMP box
(`CLAUDE.md`), where no player shows it. **The gap is accepted:** a reel sent on
as a file arrives without its `.credits.txt`.

## Crate responsibilities

| Crate | Gains |
|---|---|
| `pundit-core` | `openverse` (the query, the parse, the licence list and check, `licence_words`, the ranking); `credits::credit_text`; `naming::one_line` (moved). Pure: no network, no clock. |
| `pundit-media` | `download::get`; `inspect_audio` (full decode and container); `ExportJob::credits` and `write_credits`. Nothing about catalogues. `download()` unchanged. |
| `pundit-app` | `music_search.rs` (`search`, `add`, the landing step); `AppFiles`' default music folder; the reel's credit read in `reel_music`; the Find music sheet, its `handle-key` branch and its `main.rs` wiring. **The bus gains no command.** |
| `pundit-harness` | One end-to-end test (Testing). |

**Packaging:** nothing changes. `souphttpsrc` is already on the smoke test's
element list (the `elements` array at `smoke-test.sh:64`), and so are the parsers
and decoders `inspect_audio` plugs, which phase 1 added. No music ships, so
`packaging/copyright` is untouched.

## Testing

**No test reaches Openverse or Jamendo.** Every test that searches or fetches
does it against `pundit_media::fixtures`' local server.

- **The fixture server, with no router.** `Answer` (`fixtures.rs:807`) gains a
  variant that carries its own body. A test starts **one server for the audio
  file**, substitutes that server's URL into the checked-in page JSON (a trimmed
  copy of a real `/v1/audio/` answer), and serves the page from **a second
  server**. `serve` (`:823`) already answers any path with its one body, so this
  needs no routes.
- **core:**
  - the query: the licence list, `length`, `page_size`, percent-encoding of a
    genre with a space and an `&`, and the 200-character cut;
  - the parse of the fixture page, including that rows with no `url` or no
    `duration` are dropped;
  - **NC and BY-SA rows slipped into the page are dropped** (L2; sabotage:
    remove the re-check);
  - the ranking puts genre matches first and keeps Openverse's order within each
    group;
  - `licence_words` for BY, CC0 and PDM;
  - the credit's words, including **the modification phrase for BY**, the
    licence-URL fallback, *"Unknown artist"*, and a newline in a title flattened.
- **media:**
  - `get` honours its byte cap, its stall bound (against `Answer::Stall`), its
    cancel, and the Content-Length check;
  - a 429 is recognised from `http-status-code` (sabotage: match on the message
    text instead, against a 429 whose reason phrase is changed);
  - `inspect_audio` names the container of the `tone_audio` fixture (Ogg) and of
    a generated MP3 and WAV;
  - **a truncated VBR MP3 fails the 2 s check.** The fixture is
    `audiotestsrc ! lamemp3enc target=quality ! xingmux`, then cut to half its
    bytes; `lamemp3enc` is in the build dependencies' plugin sets.
  - an HTML body fails as not audio;
  - `write_credits` writes, removes a stale file on `None`, and leaves the path
    alone on a cancel.
- **app (unit, `music_search.rs`, against the fixture servers):**
  - `search` returns the fixture's rows, ranked;
  - `add` lands `<folder>/rock/<title> — <creator>.ogg` and its `.credit.txt`,
    and leaves no `.part`;
  - with the rename seam failing, **no credit is left** (sabotage: write the
    credit after the rename);
  - a stem collision gets `" (2)"`;
  - a re-Add after the track is deleted overwrites the orphan credit;
  - a genre field of `" Rock"` lands in an existing `rock/`;
  - an empty slug refuses before any request is made.
- **app (unit, `AppFiles`):** the default music folder's resolution, in
  `films_dir`'s shape.
- **harness, one end-to-end test:** `music_search::add` into a scratch music
  folder; the phase 1 picker (`music::genres`) lists the genre; a reel export
  with that genre writes `.credits.txt` with C2's words.
- **ui:**
  - the Music row's **Find music…** reaches its callback;
  - the sheet shows Openverse's attribution line and L3's line;
  - **Add** on a row reaches its callback with that row's id, and the other
    rows' Add is disabled while one row is adding;
  - **Esc typed in the genre field leaves both sheets open; a second Esc closes
    Find music only**, and the export sheet is still open.

## Risks

1. **The `main.rs` wiring is untested.** The thread, the generation counter,
   the folder default being written and the re-selected genre all live in
   `main.rs`, which no test reaches (`CLAUDE.md`: a UI test links the library).
   What can be is in `music_search.rs` and `AppFiles`, and the rest is the manual
   check list.
2. **Genre by title is coarse** (§O6): about a third of a page is the genre
   asked for. Ranking hides the worst of it. If the coach finds the results
   useless, the cure is Jamendo's own API (decision 2's option B), which `search`
   is shaped to take: it is one function with a base URL.
3. **A renamed track loses its credit and silently drops a CC BY attribution.**
   The credit is keyed by the track's file name. C3's log line is the cheap
   mitigation. The coach is never refused, because his own files have no credit
   either, and refusing would be phase 1's silent trade turned into a false
   alarm.
4. **Content ID does not read licences.** #84's warning stands: a CC BY track
   can still draw a claim on YouTube. The `.credits.txt`, with licence and link,
   is what a dispute is made of. Nothing in the design guards against this.
5. **Adding a track changes which song a match gets.** Phase 1's choice is
   `music_index` over the genre's sorted tracks (`reel.rs:75`), so when `n`
   changes, most matches move to a different song. Phase 1 promised a stable
   song *"while its folder keeps its name"*; with a fetcher, that also needs the
   genre to keep its tracks. The user guide says so.
6. **Jamendo's file URLs are Jamendo's**, not Openverse's (§O2). A change there
   breaks Add while Search still works. The row's error names the host.
7. **The anonymous quota is per address**, so a club's shared connection shares
   200 searches a day. Each Search is one request.
8. **Openverse may change its terms or end anonymous access** (§O9: *"at any
   time for any reason"*). Phase 1's folder keeps working whatever happens; only
   the sheet stops.

## §R. What this corrects

1. **#84 says Openverse indexes *"Jamendo, Freesound, ccMixter, Wikimedia
   Commons and the Free Music Archive"*.** On 2026-10-13, `/v1/audio/stats/`
   lists three: Jamendo, Wikimedia Commons and Freesound. Every music result
   measured here was Jamendo's. So "Openverse includes FMA, whose own API is
   dead" no longer holds, and in practice this is a Jamendo search behind
   Openverse's front door.
2. **#84's *"a checksum before use"*** cannot be had for a catalogue track,
   because no digest is published. The check is a full decode against the
   catalogue's duration instead (S7).
3. **#84's *"downloads only under `$XDG_CACHE_HOME/pundit/`"*** was already
   moved to the music folder by phase 1's §R2. #84's *"permission carried on the
   job type"* has no counterpart here, because the network is reached from a
   function no command can call with a URL (S3).

## What this does not do

- **No listening before Add** (decision 6). A track the coach dislikes is
  deleted from the folder like any other file.
- **No credit in the picture or the tags** (decision 4).
- **No Jamendo API and no registered Openverse client** (decisions 2 and 5).
- **No paging, no download queue, no progress figure** (S2).
- **No change to the mix, the choice of track or the project format.** No
  `state.json` key beyond phase 1's two: the default folder is written to the
  existing `musicFolder`, and the added genre to `musicGenre`.
- **No BY-SA** (decision 3).

## Sources

- Openverse API reference and its OpenAPI schema: <https://api.openverse.org/v1/>,
  <https://api.openverse.org/v1/schema/> (the `/v1/audio/` parameters, the
  `auth` section on rate limits and pagination, `/v1/auth_tokens/register/`,
  `/v1/auth_tokens/token/`).
- Sources indexed: <https://api.openverse.org/v1/audio/stats/>.
- Terms of service: <https://docs.openverse.org/terms_of_service.html>.
- Licence groups: <https://github.com/WordPress/openverse/blob/main/api/api/constants/licenses.py>;
  `validate_license_type`, the anonymous `page_size` bound:
  <https://github.com/WordPress/openverse/blob/main/api/api/serializers/media_serializers.py>;
  throttle defaults: <https://github.com/WordPress/openverse/blob/main/api/conf/settings/rest_framework.py>.
- CC BY-SA 4.0 legal code (§1(a) synching, §3(a)(1), §3(b)(1)):
  <https://creativecommons.org/licenses/by-sa/4.0/legalcode.en>; CC BY-SA 3.0
  (§1(a), §4(b)): <https://creativecommons.org/licenses/by-sa/3.0/legalcode>.
- The live measurements in §O were made with `curl` and `gst-launch-1.0` on
  2026-10-13; about thirty anonymous requests in all. S7's `souphttpsrc` error
  details and decode measurements were made the same day with GStreamer 1.24.2's
  Python bindings.

---

## Decided with the coach (2026-10-13)

All six questions were answered with the recommended option; the design above
is written to them.

1. **Where the search lives — (A) a "Find music…" button in the export sheet's
   Music row, opening its own sheet** (S1). Not chosen: (B) inline in the export
   sheet; (C) a row in #131's Settings sheet.
2. **Openverse's weak genre search — (A) ship Openverse, genre matches listed
   first** (S2, §O6). Not chosen: (B) Jamendo's own API with each coach's key;
   (C) both.
3. **CC BY-SA — (A) no: CC0, the public-domain mark and CC BY only** (§L1).
   Not chosen: (B) allow it, with the reel declared CC BY-SA; (C) a checkbox, off
   by default.
4. **The credit on the video — (A) a credits text file beside the reel only**
   (C3, C5). Not chosen: (B) also burned over the last seconds; (C) also in
   the file's metadata.
5. **Registering with Openverse — (A) no; search anonymously** (§O3, S3). Not
   chosen: (B) each coach's own client ID and secret in Settings.
6. **Listening before adding — (A) no preview; add, and delete the file if
   unwanted.** Not chosen: (B) streaming in the app; (C) a link to the track's
   page.

**Also confirmed:** with no music folder set, the first Add uses `~/Music/pundit`
(S6).

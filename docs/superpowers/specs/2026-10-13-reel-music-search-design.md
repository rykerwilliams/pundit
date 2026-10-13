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
Music row opens a sheet of its own: a genre field, **Search**, and up to twenty
results — title, artist, length, licence, genres — each with **Add**. Search is
one `GET` of Openverse's `/v1/audio/`, filtered to music of 2–10 minutes under
**CC0, the public-domain mark or CC BY**, issued through `souphttpsrc` on a worker
thread, parsed and re-checked in core. **Add** downloads that result's file
through the model download's own `download()` — `.part`, verified, renamed —
into `<musicFolder>/<genre>/`, beside a `<track>.credit.json` written first.
Phase 1's picker then lists the genre with no change. When a reel's chosen track
has a credit, the bus words it in core and freezes it into the job, and
`finish` writes `<film>.credits.txt` beside the reel, exactly as it writes
`.chapters.txt`. Nothing reaches the network until the coach presses Search, no
test reaches Openverse, and nothing in the project format changes.

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
but is not endorsed or certified by Openverse"* (§S2).

**O10. `souphttpsrc` reaches it.** `gst-launch-1.0 souphttpsrc
location='https://api.openverse.org/v1/audio/?…' ! filesink` returned the JSON
on this laptop, with GStreamer's default User-Agent and with one set. So the
search, like the download, adds no crate (§S5).

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

**L2. The filter is enforced twice.** The query names `license=cc0,pdm,by`
(§O4), and core **re-checks each result's `license` field** against the same
list before a row is shown — one `const` list, read by both — because the app
promises the coach the licence, and a server-side filter is the server's
promise, not ours. A result whose licence is not in the list is dropped, not
shown greyed.

**L3. What the app does not promise.** Openverse's licence data is unwarranted
(§O9), so the sheet says once, under its results: *"Licences are as the
catalogue reports them — check the track's page before you publish."* The page
is in the credit (§C2), so the check is one click from the file the coach is
about to paste anyway.

## §S. Decisions

**S1. Where it lives: a Find music sheet, opened from the Music row.** The row
(`app.slint:2149`–`:2176`) gains **Find music…** beside **Folder…**. It opens a
sheet of its own — a `Scrim` declared after the export sheet's (`:7343`), so it
stacks over it the way the error dialog does (`:7563`) — and closing it returns to
the export sheet with the picker re-listed (`show_music`, `main.rs:1322`).
**Not the Settings sheet** #131 is building: that sheet holds machine-wide values
with nothing to sit beside (its §U7 — *"the next tenant is one row"*), and the
search fills exactly the folder this picker reads, which is phase 1's §M6
argument for the folder path sitting here at all. **Not inline in the export
sheet** either: a results list and a download line would double a sheet that
already scrolls (#134), and every switch in it describes the next export, which
a search does not. (Decision 1.)

**S2. The sheet's contents.**

- A **genre** field, prefilled with the Music picker's genre (or empty), and
  **Search**. The field's text is both the query and the folder name (S6).
- Up to twenty **rows**: title, artist, `m:ss`, the licence as words
  (*"CC BY 4.0"*, *"Public domain"*), the result's `genres` — rows whose genres
  include the searched word first, the rest after, each group in Openverse's
  order. **Ranking, not filtering**: a strict genre filter keeps 7 of 20
  (§O6), and a rock song whose genres are empty is still a rock song.
- **Add** on each row, which becomes the row's progress and then *"In rock"*.
  A result already in the folder (by `id`, S7) shows *"In rock"* from the start.
- **More** for the next page, while `page < page_count`.
- A message line — the sheet's own, because the status bar renders behind the
  scrim (`CLAUDE.md`, the basket sheet's reason).
- **The attribution Openverse's terms require, always visible:** *"Search by
  Openverse. pundit is not endorsed or certified by Openverse."* — and L3's line.
- **Close.** No Cancel: an added track is added.

**S3. What is searched** — one request per Search or More:

```text
GET <base>audio/?q=<genre>&category=music&length=medium&license=cc0,pdm,by
    &mature=false&page_size=20&page=<n>
```

- **`length=medium`** (2–10 min, §O5): a reel of six goals at phase 1's 20 s +
  6 s is about 2.6 minutes; `short` tracks would end under most reels and leave
  the crowd alone, and `long` ones are mostly mixes and sets.
- **`category=music`**, so a search for "rock" does not return a sample of a rock
  being dropped (§O7).
- **`page_size=20`**, the anonymous ceiling (§O3).
- **The URL is built in core** — `core::openverse::search_url(base, genre,
  page)` — with `q` percent-encoded by a small encoder in that module (core has
  no URL crate and wants none). The genre is trimmed and cut to 200 characters,
  the API's own bound.

**S4. Parsing and checking are core's**, as pure functions over the response
bytes: `core::openverse::results(json) -> Result<Page, ParseError>` into a
`Candidate { id, title, creator, creator_url, license, license_version,
license_url, landing_url, url, duration, genres }` — `serde` with the fields the
app reads and nothing else, so an added field upstream costs nothing — then L2's
licence check and S2's ranking. A result missing `url`, `title` or `id` is
dropped; a missing `creator` reads as *"Unknown artist"*, which the credit then
says (§C2).

**S5. The transport is GStreamer's, as the download's is.** Search is
`souphttpsrc location=<url> ! appsink`, read to EOS into a buffer **bounded at
1 MiB** (a twenty-result page measured 35–39 KB) and **20 s** overall — the
"never block without a bound" rule — on a worker thread the bus spawns, with the
answer returned over the bus's own channel as transcription's messages are. One
function in `pundit-media`, `download::get(url, limit, deadline, cancel) ->
Result<Vec<u8>, CompositeError>`, beside `download` and sharing its error
wording (`unreachable_or`, `download.rs:217`). **No HTTP crate**: `souphttpsrc`
already ships (`smoke-test.sh`'s element list) and brings TLS. A non-200 answer
is an error naming the status; **429 is worded** (*"Openverse allows 20 searches
a minute and 200 a day without an account — try again shortly"*). User-Agent:
`pundit/<version>` from `core::metadata::APP_NAME`.

**S6. Where a track lands: `<musicFolder>/<genre>/`.**

- **The genre folder's name is `naming::folder_slug` of the field, lower-cased**
  (`naming.rs:167`), so *"Rock "* and *"rock"* are one genre and a `/` cannot
  climb out. An existing genre whose name the field matches is that genre.
- **With no music folder set, the first Add sets one**: `<Music>/pundit`, glib's
  `UserDirectory::Music` with `$HOME/Music` as the fallback — `films_dir`'s shape
  (`bus/state.rs:651`) for the user's music folder instead of the videos one —
  so "pick genre, search, add" works with no folder chosen first, as #84's
  *"semi-automatic"* asked. **Folder…** still chooses another. A folder that is
  set is never changed by the search.
- **The file name is `<title> — <creator>.<ext>`**, through
  `naming::safe_chars` and cut at 120 bytes by `truncate_on_boundary`
  (`naming.rs:104`, `:126`), with `" (2)"` on a collision — human-readable,
  because the coach sees this folder and phase 1 sorts tracks by name.
- **The extension is what the bytes are, not what the catalogue says**:
  `filetype` is `"mp32"` (§O2). After the transfer, `typefind` on the `.part`
  names the container — `audio/mpeg` → `mp3`, `application/ogg` → `ogg`,
  `audio/x-flac` → `flac`, `audio/x-wav` → `wav`, an MP4 audio type → `m4a` —
  which are phase 1's `EXTENSIONS` (`music.rs:14`). Anything else refuses the
  track (*"not an audio file pundit can play"*), so `music::tracks` never has to
  be told about a fetched file.

**S7. The credit is captured at the fetch, beside the track:
`<track>.credit.json`.** One file per track, so it moves and dies with the
track when the coach handles the pair, and there is no shared document for two
writers to race over. It holds the `Candidate`'s identity and licence fields —
`openverse` (the id), `title`, `creator`, `creatorUrl`, `license`,
`licenseVersion`, `licenseUrl`, `landingUrl`, `source` — camelCase, as
`state.json` is. **Its extension keeps it out of the listing by construction**:
`.json` is not in `EXTENSIONS`, so `music::genres` and `music::tracks` read it
as nothing. "Already in rock" (S2) is a scan of the genre's `*.credit.json` for
the id. A coach who wants his own files credited can write one by hand; nothing
requires it.

**S8. The order of writes is what keeps "a listed track has its credit" true.**
`.part` → verified (S9) → **credit written** (temp file and rename, as
`store::write` writes `project.json`) → **track renamed into place**. The rename
is what makes the track exist to `music::tracks`, so a track the picker can
choose always has its credit. A failed track rename deletes the credit it just
wrote. A crash between the two leaves an orphan `.credit.json`, which is inert.

**S9. The download is the model's `download()`, with the check made the
caller's.** `download` (`download.rs:82`) already does the `.part`, the
cancel, the one-at-a-time lock (`ONE_AT_A_TIME`, `:50`), the cleanup and the
rename; what it cannot do is `check` (`:240`), which needs a sha256 and a length
**known in advance** — and Openverse gives neither (`filesize: null`, no
digest). So `Fetch` (`:55`) carries the check as a value:

```rust
pub enum Check {
    /// The model's: exact length, then sha256 — today's `check`, unchanged.
    Digest { sha256: String, bytes: u64 },
    /// A fetched track's: it types as audio and decodes to a duration within
    /// 2 s of the catalogue's.
    Audio { duration: Duration },
}
```

`Audio` is what catches a truncated transfer and an HTML error page served as
`200`, which are the two failures a catalogue file realistically has. The probe
is the mixer's own decoder (`decodebin3`, as `Reader` reads), so what passes is
what the mixer can read. Progress is bytes over `souphttpsrc`'s byte duration
when the server sends a length, and a running byte count when it does not.
**The lock is shared with the model download on purpose**: one transfer at a
time, process-wide — so an Add waits behind a 488 MB model download, and the
row says *"Waiting for the speech model's download"*. Rare and honest; a second
lock would reopen the race `ONE_AT_A_TIME`'s doc closes.

**S10. Permission is the candidate, never a URL** — `TranscribeKind::Whisper`'s
rule (`transcribe.rs:190`), carried across. `Command::AddMusic` carries the
result's **id and the genre**, and the bus downloads only a `Candidate` it holds
from its own latest search; no command carries a URL, so no command can make the
app fetch anything a search did not return. The search's base URL is a
**`Catalogue`** value handed to `Bus::spawn` (`bus/mod.rs:912`), as `capture` and
`transcribe` are: `Catalogue::Openverse { base }`, or `Catalogue::Off`, which
answers Search with *"music search is not available"*. **`main.rs` is the only
place `https://api.openverse.org/v1/` is written**; the harness passes a local
server's URL or `Off`, so no test can reach Openverse even by accident.

**S11. Nothing is automatic, and recording wins.** No request at startup, at
sheet open or on typing — the first network traffic is the coach's Search
(transcription's *"asked for, never automatic"*). Neither command is on the
recording allow-list, so both are refused during a take by the default the
allow-list already gives. An Add runs while the coach keeps working, as an
export does; **closing the sheet cancels an Add in flight** (the file is a few
megabytes — measured 4.5 MB for a 3:24 track — so a cancel costs seconds of
re-download), and the cancel leaves nothing, as `download`'s does.

**S12. One search at a time, and the latest wins.** A Search or More while one
is in flight cancels it, and each answer carries the generation it was asked
under, so a slow first answer cannot overwrite a second. One Add at a time
(S9's lock); a second Add queues in the sheet's rows rather than in the bus.

## §C. Credits

**C1. The credit is worded in core, frozen into the job by the bus, and written
by media** — the cues' path exactly. `reel_music` (`bus/export.rs:1203`) already
returns the chosen track's path; beside it the bus reads
`<track>.credit.json` and hands `core::credits::credit_text(&credit)` to the job
as **`ExportJob::credits: Option<String>`**. Read at job build, so a queued job
keeps the credit of the track it froze (`CLAUDE.md`'s queue: *"a queued row is a
snapshot"*). An absent or unreadable `.credit.json` — the coach's own files,
phase 1's case — is `None` and a logged line, never a refusal: phase 1 refuses a
genre it cannot honour, and a missing credit is not that.

**C2. The words.** TASL — title, author, source, licence — with links, ready to
paste into a description:

```text
Music: "I want you (pop rock remix)" by Lollita
https://www.jamendo.com/track/1015339
Licensed under CC BY 3.0 — https://creativecommons.org/licenses/by/3.0/
```

CC0 and PDM say *"Public domain (CC0 1.0)"* / *"Public domain"* in place of the
licence line, and keep the credit: owed nothing, but it costs nothing and tells a
viewer where the song came from. **Openverse's own `attribution` string is not
used**: it carries no link to the track, which CC BY asks for where reasonably
practicable (4.0 §3(a)(1)(A)(v)), and its wording is not ours to keep stable.

**C3. `<film>.credits.txt`, written in `finish`.** `finish`
(`composite/export.rs:439`) gains `write_credits(job)` after
`write_chapter_list` (`:525`), with the same rules: `job.path.with_extension(
"credits.txt")`, so it inherits the run's name cleaning and `" (2)"`;
after the rename, so a cancelled or failed run writes and removes nothing; a
failure is logged, reported as `ExportDone::credits: None`, and never fatal.
**And like `.chapters.txt` it is a name of ours**, so a run whose job carries
`None` **removes** a stale one — a reel re-exported with Music: None must not
leave the old song's credit beside it.

**C4. One file per output, not one combined description.** A single
`<film>.description.txt` holding the chapters and the credit would be one paste
instead of two, but `.chapters.txt` has its own rule — no file under three
surviving chapters (`CLAUDE.md`) — that a credit must not inherit, and a third
sidecar is cheaper than a file with two rules for whether it exists.

**C5. Not burned into the picture, and not in the file's tags — in this
phase** (decision 4). #84 called burning a credit *"a second step, not the
first one"*. The in-file tag would be `description`, which on this muxer lands
only in the XMP box (`CLAUDE.md`), where no player shows it — a credit no viewer
sees is not a credit. **The gap is real**: a reel sent on as a file (a group
chat) arrives without its `.credits.txt`; the coach accepted that (decision 4).

**C6. `pundit-media` learns nothing about Openverse or licences.** It receives a
string and writes it; `ExportJob::credits` is media's whole knowledge of this
feature, as `cues` is of the scoreboard picker.

## §F. When the network fails

| What happens | What the coach sees | What is left on disk |
|---|---|---|
| Offline / DNS / TLS | *"Couldn't reach api.openverse.org"* on the sheet's line (`unreachable_or`'s host wording) | nothing |
| 429 | S5's worded limit | nothing |
| Other non-200, or JSON that will not parse | *"Openverse answered with something pundit can't read (…)"* | nothing |
| 20 s without an answer | *"Openverse didn't answer in 20 seconds"* | nothing |
| A download that stalls, drops, or fails its check | the row: *"Couldn't add this track (…)"*, Add offered again | no `.part`, no `.credit.json` |
| Cancel (Close mid-Add) | — | nothing |
| Quit mid-Add | — | a `.part` the next Add of that track truncates; never listed (`.part` is no track extension) |

No retry loop anywhere: pressing the button again is the retry, as the model
download's doc says of Transcribe.

## Crate responsibilities

| Crate | Gains |
|---|---|
| `pundit-core` | `openverse` (the URL, the parse, the licence list and check, the ranking), `credits` (the `Credit` record and its words). Pure, no network, no clock. |
| `pundit-media` | `download::get`; `Check` on `Fetch`, with `Audio`'s probe; `ExportJob::credits` and `write_credits`. Nothing about catalogues. |
| `pundit-app` | `Catalogue` on `Bus::spawn`; `Command::{SearchMusic, AddMusic}` and their events; the `.credit.json` read/write and the folder default; the Find music sheet. |
| `pundit-harness` | the tests below, against a local server. |

**Packaging:** no new package. `typefind` is a core element; add it to
`smoke-test.sh`'s element list (the `elements` array at `:64`), which `CLAUDE.md` makes the only
proof of the dependency list. `souphttpsrc` is already there. No music ships, so
`packaging/copyright` is untouched.

## Testing

**No test reaches Openverse or Jamendo.** Every test that searches or fetches
does it against `pundit_media::fixtures`' local server, and the harness's
default `Catalogue` is `Off`.

- **The fixture server learns routes.** `serve` (`fixtures.rs:823`) answers every
  request, whatever its path, with one body (`served_body`). It gains a form that answers by path
  — a checked-in search page (a trimmed copy of a real `/v1/audio/` answer whose
  `url`s are rewritten to point back at the server), the `tone_audio` fixture
  (phase 1's Ogg Vorbis), an HTML page served as `200` audio, and the existing
  `Answer::Stall` / `Status`. One server, so the search's URLs and the files they
  name agree.
- **core:** the URL (licence list, `length`, `page_size`, percent-encoding of a
  genre with a space and an `&`, the 200-character cut); the parse of the
  fixture page; **an NC and a BY-SA row slipped into the page are dropped** (L2;
  sabotage: the re-check removed); ranking puts genre matches first and keeps
  order within each group; a row with no `url` is dropped; the credit's words
  for BY, CC0 and PDM, and *"Unknown artist"*.
- **media:** `download` with `Check::Audio` — a good tone lands; the HTML page
  and a truncated body fail the check and leave no `.part`; a duration off by
  more than 2 s fails. `Check::Digest`'s existing tests stay as they are
  (sabotage: `Audio` always passing). `get` honours its byte bound and its
  deadline against `Stall`. `write_credits` writes, and removes a stale file on
  `None`, and leaves the path alone on a cancel.
- **harness:** search → an event with the fixture's rows, the NC row absent;
  Add → `<folder>/rock/<title> — <creator>.ogg` and its `.credit.json`, written
  credit-first (a test that fails the track rename finds no credit left); the
  phase 1 picker now lists `rock`; a reel export with that genre writes
  `.credits.txt` with C2's words; re-exported with Music: None, the
  `.credits.txt` is gone; a track with no credit exports with no credits file
  and no refusal; Add with no folder set lands in the default folder and sets
  `musicFolder`; **`AddMusic` with an id the last search did not return fetches
  nothing** (S10; sabotage: look the id up in any page ever seen); 429 and a
  stall each give their line and leave nothing; a search during a take is
  refused; a second Search's answer wins over a slower first.
- **ui:** the Music row's **Find music…** reaches its callback; the sheet shows
  Openverse's attribution line and L3's line (by label, `tests/ui/export_sheet.rs`'s
  pattern); **Add** on a row reaches its callback with that row's id; the
  sheet's message line shows a refusal.

## Risks

1. **Genre by title is coarse** (§O6): about a third of a page is the genre
   asked for. Ranking hides the worst of it; the cure, if the coach finds the
   results useless, is Jamendo's own API (decision 2's option B), which #84 named as
   the upgrade and which this design's `Catalogue` seam is shaped to take.
2. **Content ID does not read licences.** #84's warning stands: a CC BY track
   can still draw a claim on YouTube. The `.credits.txt` — licence and link — is
   what a dispute is made of. Not engineered against.
3. **Adding a track changes which song a match gets.** Phase 1's choice is
   `music_index` over the genre's sorted tracks (`reel.rs:75`), so `n` changing
   moves most matches to a different song. Phase 1 promised a stable song
   *"while its folder keeps its name"*; with a fetcher, also *while the genre
   keeps its tracks*. The user guide says so.
4. **Jamendo's file URLs are Jamendo's**, not Openverse's (§O2): a change there
   breaks Add while Search still works. The row's error names the host.
5. **The anonymous quota is per address**, so a club's shared connection shares
   200 searches a day. Measured traffic is one request per Search or More; a
   coach is unlikely to come near it. Anonymous use was decided (decision 5).
6. **Openverse may change its terms or end anonymous access** (§O9: *"at any
   time for any reason"*). Phase 1's folder keeps working whatever happens;
   only the sheet stops.

## §R. What this corrects

1. **#84 says Openverse indexes *"Jamendo, Freesound, ccMixter, Wikimedia
   Commons and the Free Music Archive"*.** `/v1/audio/stats/` lists three on
   2026-10-13 — Jamendo, Wikimedia Commons, Freesound — and every music result
   measured here was Jamendo's. So "Openverse includes FMA, whose own API is
   dead" no longer holds, and in practice this is a Jamendo search behind
   Openverse's front door.
2. **#84's *"a checksum before use"*** cannot be had for a catalogue track: no
   digest is published. The check is a decode instead (S9).
3. **#84's *"downloads only under `$XDG_CACHE_HOME/pundit/`"*** was already
   moved to the music folder by phase 1's §R2; this spec follows that, and the
   permission rule (S10) is what replaces "only under the cache".

## What this does not do

- **No listening before Add** — no streaming preview (decision 6). A track
  the coach dislikes is deleted from the folder like any other file.
- **No credit in the picture or the tags** (C5, decision 4).
- **No Jamendo API, no registered Openverse client** (decisions 2 and 5).
- **No change to the mix, the choice of track or the project format**; no
  `state.json` key beyond phase 1's two (the default folder is written to the
  existing `musicFolder`).
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
  2026-10-13; about thirty anonymous requests in all.

---

## Decided with the coach (2026-10-13)

All six questions were answered with the recommended option; the design above
is written to them.

1. **Where the search lives — (A) a "Find music…" button in the export sheet's
   Music row, opening its own sheet** (§S1). Not chosen: (B) inline in the export
   sheet; (C) a row in #131's Settings sheet.
2. **Openverse's weak genre search — (A) ship Openverse, genre matches listed
   first** (§S2, §O6). Not chosen: (B) Jamendo's own API with each coach's key;
   (C) both.
3. **CC BY-SA — (A) no: CC0, the public-domain mark and CC BY only** (§L1).
   Not chosen: (B) allow it, with the reel declared CC BY-SA; (C) a checkbox, off
   by default.
4. **The credit on the video — (A) a credits text file beside the reel only**
   (§C3, §C5). Not chosen: (B) also burned over the last seconds; (C) also in
   the file's metadata.
5. **Registering with Openverse — (A) no; search anonymously** (§O3, §S5). Not
   chosen: (B) each coach's own client ID and secret in Settings.
6. **Listening before adding — (A) no preview; add, and delete the file if
   unwanted.** Not chosen: (B) streaming in the app; (C) a link to the track's
   page.

**Also confirmed:** with no music folder set, the first Add uses `~/Music/pundit`
(§S6).

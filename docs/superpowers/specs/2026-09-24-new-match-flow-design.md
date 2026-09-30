# New match: one sheet that makes the folder and names the project

**Date:** 2026-09-24. **Revised 2026-09-29**, after both adversarial passes.
**Status:** Reviewed. The decisions the reviews changed are applied below and the
reasoning they rejected is kept in **§R**, not deleted.
**Builds on:** the project lifecycle (`bus/project.rs`, spec D6), the source list (`bus/sources.rs`, spec D7), the match setup sheet and `ScoreboardConfig` (Phase 9 spec S5), the shared `Sheet` / `Scrim` extracted for the match event editor (`docs/superpowers/specs/2026-09-23-match-event-editor-design.md` P1), and the export file-name rules (Phase 8 spec E6).
**Evidence:** the code as it stands on `origin/main` at `40d996d`, and a read-only inspection of the coach's two real footage trees on 2026-09-24. Every claim carries a `file:line` or an observation.

Labels: **[cited]** points at a file in this repository or a decision recorded in another spec. **[observed]** is something read off the coach's own disks on 2026-09-24 — described structurally (`<club>`, `<season>`, `<match>`, `<opponent>`), never by name. There is no **[measured]** claim in this document.

> **The sheet asks for names, not for setup.** The colours are one click away in
> `Setup…`, the video order is drag-and-drop in the sidebar, the project name is
> the title field — **only the folder name has no correction path once Create is
> pressed**, and the coach asked about the folder and a naming scheme. Everything
> else the first draft put in the sheet was scope the draft added, and cutting it
> takes the colour fields, their validators, the periods and minutes fields, the
> `TeamColumn` extraction and its regression risk with it.

---

## Goal

The coach asked for this in these words:

> *"the project creation flow is a bit wonky. in general i will have a folder with the game files. so like, i'd want the app to help create the folder? note the pattern of having a 'pundit' folder and then projects inside it? so if the app could help with a uniform naming scheme."*

Today, starting a match takes four separate acts, in a fixed order, none of which knows about the others:

1. **`Open Project…`** — a folder picker (`Pick::ProjectFolder`, `pickers.rs:17`) whose result becomes `Command::OpenProject` (`main.rs:440-449`). The folder must **already exist**: `open_project` refuses one that doesn't, because "saving never creates one" (`bus/project.rs:30-33`, the refusal at `:39-44`). So the coach creates the folder in the picker's own *New Folder* button, or in a file manager, before the app is involved at all. The project's name is then whatever that folder is called (`bus/project.rs:47-52`).
2. **`Add Source Video…`** — `Pick::Videos`, one `Command::AddSource` per file (`main.rs:450-459`), each one a separate probe, gate, save and publish (`bus/sources.rs:26-43`).
3. **`Set up teams…`** — the match setup sheet (`app.slint:1035`), `Command::SetScoreboard` (`main.rs:1187-1197`).
4. **Renaming the project** — the title field, `Command::RenameProject` (`main.rs:507-522`), which a coach who got the folder name right never does.

**What that produces on the coach's own disk** [observed]:

| | Tree A | Tree B |
|---|---|---|
| footage | `<club>/<team>/game-videos/<match>/P1.mp4, P2.mp4` | `<club>/<season>/<match>/<two files from one camera app>` |
| projects | `<club>/<team>/pundit/<match>/project.json` | `<club>/pundit/<match>/project.json` |
| folder names | `2026-09-19-<opponent>` | `<8-digit date>-<opponent>` |
| project names | `"<club> <team> - <opponent>"` and `"<team> vs <opponent> <date>"` — **two different schemes in two sibling projects** | `"<8-digit date>-<opponent>"` — the raw folder name, i.e. nobody ever renamed it |

And one of Tree B's two project folders **contains nothing at all** — no `project.json`, no `recordings/`. It is a folder made in a picker or a file manager that the app was never pointed at. That is the wonkiness, on disk: the folder is created by hand, outside the app, and nothing guarantees the app ever finishes the job.

This spec collapses steps 1–4 into **one sheet**, prefilled from the footage the coach picked, and one all-or-nothing command. The four existing routes stay exactly as they are (**E3**).

---

## Scope

**The user's decisions of 2026-09-24. This spec follows them and does not reopen them:**

- **D-1. The project folder goes in a `pundit/` folder beside the videos.** The app finds the sensible parent and creates the match folder in it. **The footage is left where it is** — never moved, never copied.
- **D-2. The name carries the date and both teams.** Folder `2026-09-17-<home>-<away>`; project name `<Home> v <Away>`. The date comes from the video files. The coach can edit everything before anything is created.

**In scope:** the entry points, the inference, the sheet, the one command, the naming rules, and the failure behaviour. **Out of scope and unchanged:** the match setup sheet (it stays the way to set and edit teams, colours and format); `Open Project…`; `Add Source Video…`; relink; export; anything to do with clips, recording or the match clock.

---

## Decisions

### E. How the flow is entered

**E1. A "New match…" button in the transport row, immediately left of `Open Project…`, and the primary action of the no-project empty card.** Both, not one.

The transport row already holds `Open Project…` and `Add Source Video…` side by side (`app.slint:4969-4981`), and the empty card `"No project open"` offers `Open Project…` as its single action (`app.slint:4604-4609`). A coach who has just launched the app sees the card, not the toolbar; a coach who has a project open and wants the next match sees the toolbar, not the card. Putting it in one place only would miss one of those two.

`EmptyCard` takes one `action` and one `clicked()` (`app.slint:340-383`). It gains an optional `secondary` string and `secondary-clicked()`, rendered as a plain (non-`primary`) button beside the first. The no-project card becomes **primary "New match…"**, **secondary "Open Project…"** — new is the common case, opening an old one is the exception. The other two cards pass no `secondary` and are unchanged. **Gating is exactly `Open Project…`'s**: `enabled: !root.recording` (`app.slint:4974`). No new rule. The command is not on the recording allow-list, which is deny-by-default (`bus/mod.rs:895-934`), so a UI bug reaches a silent refusal rather than a half-built project mid-take.

**E2. It opens the existing multi-select video picker first, and the videos decide the folder.** `Pick::Videos` already exists, already filters on `VIDEO_EXTENSIONS` in both cases (`pickers.rs:36-39`), already multi-selects, and already parents itself to the window on the portal without blocking the event loop (`pickers.rs:1-4`). Nothing new is needed.

Picking the **files** rather than the folder is deliberate, and it is what lets this spec answer *"which files are game videos at all"* honestly: **the coach's selection is the answer, and the app never guesses.** A folder-first flow would have to scan and then decide which of a folder's files are halves and which are a stray phone clip, a `.txt` of kick-off times or a thumbnail — a guess with no good failure mode. The desktop dialog is already the right place to look at names and sizes. The match folder is then derived rather than asked for: **the deepest common ancestor directory of the chosen files** (**I1**), which in both of the coach's trees is the `<match>` folder [observed].

**E3. Nothing is removed.** `Open Project…`, `Add Source Video…`, `Set up teams…` and the rename field all stay, unchanged and un-deprecated. They are how an existing project is opened, how a third camera angle is added a week later, and how the teams are corrected. The new flow is a fifth route, not a replacement.

**E4. The picker's ordering is already fixed, and `Pickers::open` keeps its per-path callback.** [cited] The draft proposed both, and the first half shipped ahead of this spec: `core::naming::order_videos` (`naming.rs:26`) keys each path on **the stem with any ` (n)` copy suffix removed, then `n`**, and `pickers.rs:64` calls it before handing the paths out. That is the whole bug and its whole fix — `' '` is `0x20` and `'.'` is `0x2E`, so a byte sort put `<Stem> (1).mp4` ahead of `<Stem>.mp4` and reversed a game's halves silently (`naming.rs:81`, the test that fails against a plain name sort).

**The signature stays `impl FnMut(PathBuf)`.** Handing back a `Vec<PathBuf>` would make a cancelled dialog call back with an empty vec, which the project-folder, single-video, relink and avatar pickers would all have to learn to ignore — three of those flows would raise an error on it. The one caller that wants the whole set (this flow's) collects it itself; `order_videos` is already in core, where it is tested without a picker.

### I. What the app infers, and how sure it is

**I0. Every inference is a prefilled, editable field. Nothing is decided silently, and nothing is created until Create is pressed.** Where an inference is weak the field carries one short provenance line under it; where it is strong it carries nothing, because a hint on every field is a hint on none.

**I1. The match folder is the deepest common ancestor of the chosen files.** With every file in one directory — both of the coach's trees, every time [observed] — that is that directory. With files from two directories it is their shared parent, and the opponent inference (**I4**) will usually find nothing there, which is the correct outcome: a blank half of a folder name rather than a wrong one.

**I2. The date, in this order. A bare year is never a date.**

1. **A full date in a file's name.** The shapes accepted are `YYYYMMDD`, `YYYY-MM-DD`, `YYYY_MM_DD` and `YYYY.MM.DD`, each bounded by a non-digit or an end of string, with month `1..=12`, day `1..=31` and year `2000..=max_year`. Leftmost wins. If every chosen file yields the same date, that is the answer.
2. **A full date in the match folder's name**, same shapes, same bounds.
3. **The first-in-order file's mtime**, resolved in the local zone. **The first in `order_videos`' order, not the earliest mtime**, so there is one rule for "the game's date" and not two: `bus/export.rs:638` already reads `sources.first()`, and after `order_videos` the two usually coincide but not always.
4. Nothing. The folder name is then prefilled from the team slugs alone, and Create is not blocked: a folder of two team names is a legal, distinct folder.

**Requiring a full date is the whole rule**, and it is what stops two real traps in the coach's own trees [observed]: a team folder whose name is a **four-digit birth year**, and a match folder whose name begins with a **six-digit age-group code**. A `YYYY` matcher would date every match in Tree A to a decade ago; a `YYMMDD` matcher would read that six-digit code as a month of 15. Both are rejected by requiring eight digits with a valid month and day. Tree A's file names carry a leading `YYYYMMDD` **and** a bare year later in the same name; leftmost-first plus the eight-digit rule takes the right one.

**Why mtime is third**, rather than second: in Tree A the footage mtime is **two days** after the date the coach's own folder name gives; in Tree B it is **one day** after [observed]. These are downloads, and they are downloaded when the coach gets to it. `bus/export.rs:626-634`'s doc says mtime *"is within a day of the match"*; Tree A shows two. It is close enough to be a useful prefill and not close enough to be trusted (**Deferred 3**).

**`parse_date_in` takes the year bound as an argument**, not from a clock: `pub fn parse_date_in(text: &str, max_year: i32) -> Option<CalendarDate>`, with the app passing `this year + 1`. Core has no clock, which is exactly why `CalendarDate` is passed in rather than derived (`metadata.rs:63-77`); the draft wrote the bound as "`<this year + 1>`" inside core, and its test *"a year outside the window is refused"* would have been time-dependent.

**I3. The container's creation time is unusable, and the probe is not extended to read it.** Both of the coach's camera systems write `creation_time = 1904-01-01T00:00:02Z` into every file — the QuickTime epoch with a two-second offset, i.e. a field that was never set [observed]. Taking it would date every match in both trees to 1904. `pundit_media::probe` returns duration and display aspect and nothing else (`probe.rs:18-25`); adding a creation-time field to it would be a media change made to serve a value that is provably garbage on 100% of the coach's footage. **Not done.** If a coach ever shoots on a phone, whose containers do carry a real one, revisit (**Deferred 2**).

**I4. The opponent comes from the match folder's name, and only from there.**

Take the match folder's file name; strip a leading run of digits and separators (that is either a date or a code, and in neither case a team); strip a trailing run of digits and separators; replace `_`, `-` and `.` with spaces; collapse spaces; trim. If what is left is non-empty, **capitalize each word's first letter and leave the rest as typed** — not full title case, which would turn `FC` into `Fc` and `McBride` into `Mcbride`, destroying a capitalization the coach chose for a value this is only guessing at. Otherwise the field is left empty. (Corrected in T1, which shipped the honest rule; the draft said "title-case each word".)

Both of the coach's trees yield the opponent's name cleanly under this rule [observed]: one after stripping a `YYYY-MM-DD-` prefix, the other after stripping a six-digit code and its hyphen. **File names are deliberately not mined for it.** Tree A's file names do contain it, wrapped in a date, an age group, a club abbreviation, a separator that is itself a hyphen-underscore sandwich, and a half marker [observed]. Anything that pulled a team out of that would be a pattern fitted to one camera system, and it would quietly produce nonsense on the other, whose file names carry no team at all. The folder name is what the coach already curates.

**I5. The scoreboard is the most recent neighbouring project's, with the two typed names substituted.** One `read_dir` of the chosen projects folder, its entries in recency order, and the first that `store::read` (`store.rs:97`) accepts — so one read in practice. A file that is unreadable, legacy or too new is passed over in silence: this is a prefill, not an operation. With no readable neighbour, `match_panel::blank_config()` (`match_panel.rs:317-335`) — which is what the setup sheet already opens with on a project that has no scoreboard (`main.rs:1436-1450`) — and `MatchFormat::default()` (`scoreboard.rs:60-70`). What is inherited is a **whole valid config**, so the new project starts with the previous match's kit in each slot and the format the coach plays; only the two names are replaced, and `Setup…` corrects the rest.

**Recency is the date parsed out of the folder name, not mtime.** `project.json` is rewritten on every edit (`bus/project.rs:187-190`), so its mtime means "last opened", not "last match" — a coach who reopens last season's game to re-export a clip would seed the next match from it. And because **N1** puts `YYYY-MM-DD` at the front of every folder this flow creates, sorting the folder names descending **is** recency order: no `stat` at all. A folder with no parseable date sorts below every dated one, ordered by name descending — still no `stat`.

**The UI-thread budget, stated rather than assumed.** This runs on the UI thread while the sheet opens, on a cloud-sync mount. **S4** refuses to call `probe` there because it blocks for up to ten seconds per file (`probe.rs:15-16,41`); the seeding is one `read_dir`, up to four `is_dir` calls for **W1**, and one read of a file that is a few kilobytes (rarely two, if the newest neighbour does not parse). That is the same order of work as opening any folder in a file manager, and it is bounded by the number of *matches in one season*, not by the number of files. The draft's histogram read **every** `project.json` in the folder, which is the version that would have been felt.

**I6. The video order is `order_videos`, and the sheet shows it read-only.** The list is `<n>. <file name>`, in the order the command will store them, and that is all it is: a check that the halves are the right way round. There are no `↑ ↓ ×` controls, because **the sidebar already reorders and removes sources by drag** (`app.slint:4007` → `Command::MoveSource`, `:4053` → `RemoveSource`), which is a correction path that exists, is tested and works a week later as well as it works now. Two orderings of one list in two places is the kind of duplication this codebase pays for later.

The draft also proposed a half-marker layer, an mtime layer and a line warning when name order and mtime order disagree. None of it is needed: a plain name sort already puts `-P1` before `-P2` (Tree A [observed]), the copy suffix was the only real failure, and the shipped rule has no clock in it at all.

**I7. Which files are game videos is the coach's selection (E2).** The app applies no size floor, no duration check and no content sniff at pick time. The bus probes them during Create (**C2**), which is the app's existing and only definition of "can this be a source" (`bus/sources.rs:211-218`).

### W. Where the projects folder is

**W1. Walk up from the match folder for an existing app-named directory, and use the first one found.**

From the match folder itself, then its parent, then its parent's parent, up to **four** levels, stopping at the filesystem root or at the coach's home directory (inclusive of home, never above it): test whether `<candidate>/<APP_NAME>` is a directory. The first hit is the projects folder.

**The name is `core::metadata::APP_NAME` (`metadata.rs:30`), never the literal `"pundit"`** — `CLAUDE.md` is explicit that the app's name is read from that constant rather than spelled again, and `bus/state.rs:26`'s `APP_DIR` is the same string but `pub(super)`, invisible to a module outside `bus`. This is the decisive rule and it answers the coach's actual question — *"note the pattern of having a 'pundit' folder and then projects inside it"*. It finds the right answer in **both** of his trees, at depth 2 in each, despite the trees having different shapes [observed]:

| | match folder | depth 1 | depth 2 |
|---|---|---|---|
| Tree A | `<club>/<team>/game-videos/<match>` | `game-videos/` — no | `<team>/pundit` — **hit** |
| Tree B | `<club>/<season>/<match>` | `<season>/` — no | `<club>/pundit` — **hit** |

It also handles the coach reorganizing, it needs no configuration, and it is at most four `is_dir` calls.

**W2. With no such directory above the videos, use `last_project()`'s parent.**

`AppFiles::last_project()` (`bus/state.rs:176`) already holds the folder of the project the coach last opened, written on every commit (`bus/project.rs:98`). **Its parent is the projects folder** — that is what a projects folder *is*. So the second tier is that parent, with no inference and no clamping: it is a directory that demonstrably held a project a moment ago.

With no last project either — a first launch, before this app has ever opened one — the field is prefilled with an **app-named directory beside the footage**, `<match folder's parent>/<APP_NAME>`, under the hint *"No projects folder found nearby — choose where projects go."* The coach presses `Choose…` (**W3**) if that is wrong. **Not the bare parent:** that would propose projects as *siblings* of the match folder, which is the opposite of the pattern this whole flow exists to keep — the coach's own words are *"the pattern of having a 'pundit' folder and then projects inside it"*, and both their trees have one [observed]. Proposing the directory that pattern wants, in the only place there is evidence for (beside the footage), is a guess the coach can see and change before anything is created; proposing its parent would be a guess that quietly breaks the pattern. It is also the one tier that creates a directory the coach has never seen, which is why **W3**'s parent-must-exist rule matters most here. This tier **shares one field with BACKLOG #85's recent-projects list** rather than re-deriving what `state.json` already remembers: #85 grows `last_project` into a short `Vec<PathBuf>` and this flow reads the head of it, so neither needs a key of its own.

**W3. The field is an editable path with a Choose… button that opens the existing folder picker, prefilled.** `Pick::ProjectFolder` (`pickers.rs:17`) with its title changed to name what it is choosing. That is the "ask, with a picker prefilled" answer for every case W1 and W2 get wrong, and it is the same control the coach already knows. **Because it is hand-editable, its parent must exist:** a typo must not create a tree, so **C2** requires `projects_dir.parent()` to be an existing directory and then makes at most the one leaf with `create_dir`, never `create_dir_all`. The sheet checks the same thing so the button and the command agree.

**W4. The footage never moves.** Nothing is copied, nothing is renamed, nothing is hard-linked. The project stores `relative_path`, computed by the existing helper, which already produces a `../`-climbing path from a canonical file to a canonical project folder and is already tested for exactly that (`bus/sources.rs:246-258`, `:283-292`). Both of the coach's trees store `../../<group>/<match>/<file>` today [observed], which is what this flow will keep writing.

### N. Naming

**N1. The folder name is one editable field, prefilled `<YYYY-MM-DD>-<home slug>-<away slug>`.**

Home then away, in the slot order the sheet shows, so ⇄ Swap rewrites it. It is **re-derived from the two name fields until the coach types in it**, and left alone from then on: one dirty flag, set by the field's own edit and never cleared while the sheet is open.

**There is no separate date field, and the date is better off inside this one.** A wrong date is corrected where its consequence is visible: the coach reads `2026-09-21-rovers-athletic` and sees both the date and what it does. A date field beside a read-only preview shows the same thing twice and gives the coach two places to look.

**There is no project-name field either.** The name is `<Home> v <Away>`, unconditionally, from the two fields already on screen. `Command::RenameProject` and the title field exist and are one click away (`main.rs:507-522`), and a sticky third name field in a sheet whose whole job is the folder earns nothing. Under the folder field is **one read-only line: the full path** it will create. That is the whole preview.

**N2. Two functions in `core::naming`, not one pipeline.**

`bus/export.rs:192-198` has the app's only export-name cleaning today: `part.replace(['/', ':'], "-")`, with the comment that `/` is the path separator and `:` is what *"a share to a Mac or a Windows machine trips over."* That reasoning is right and its character set is short by seven. The coach's projects live on a cloud-sync mount [observed], and exFAT, NTFS and SMB all reject `" * ? < > |` and `\` as well.

So:

- **`naming::safe_chars(&str) -> String`** — replace each of `/ \ : * ? " < > |` and every control character with `-`. Nothing else: no collapsing, no trimming, no truncation, no case change. `bus/export.rs`'s `file_name` calls **this**, so the only export file name that changes is one that contains a character which today produces a file that fails to write on a shared drive.
- **`naming::folder_slug(&str) -> String`** — `safe_chars`, then lowercase, then whitespace runs to `-`, then collapse `-` runs, then `truncate_on_boundary(_, 64)`, then trim `-`, `.` and whitespace. `Rovers United` → `rovers-united`. An accented name lowercases per Unicode and keeps its letters: a coach's own language is not punctuation, every filesystem this app targets stores UTF-8 names, and mangling it would need a transliteration crate `pundit-core` will not be getting.

**Pointing export's `file_name` at the whole pipeline — which the draft did — would have silently renamed every future export.** Collapsing `-` runs, trimming leading dots and truncating to 64 bytes all apply to labels the coach already has: the export basename is what `.srt` and `.chapters.txt` are derived from (`composite/export.rs:464`, `:512`), so every sidecar beside a file already written would orphan on the next run, and two long labels that differ only past byte 64 would collide *after* `de_duplicate` had already run on the labels (`bus/export.rs:334`, `:510`) and found them distinct. The draft's own proposed regression test — "`file_name`'s output is unchanged for a label with no forbidden character" — would have passed while all of that shipped. **The test points at a label with `--` in it, one with trailing dots, and one over 64 bytes.**

**Truncate before trimming, not after**, which is the order above: a cut at byte 64 can land on a `.` or a space, and Windows and SMB silently strip trailing dots and spaces — the exact class of failure this rule exists to prevent, on the mount the coach uses. `bus/basket.rs:549-565` already does it this way round.

**There is a third name-cleaner in the tree and this spec does not pretend otherwise.** `bus/basket.rs:528-565` cleans a film's typed name: a 200-byte cap with a documented 255-byte rationale, char-boundary truncation, a leading-dot guard. Its budget is right for its job and 64 is right for a folder that sits inside a path, so they are not one function. What **is** shared is the mechanism: `naming::truncate_on_boundary(s, bytes)`, called with each site's own budget, replacing the hand-rolled `char_indices` walk at `basket.rs:553-559`.

**N3. A folder name that is already taken is refused, not suffixed.** The draft had both — a `-2`/`-3` rule in `next_free` and a refusal in the bus — which contradict each other. The refusal wins, and `next_free` is not written: the coach is looking at an editable folder-name field with the full path under it, so a collision is one he can see and resolve in the place he is already typing. A double-header against the same opponent on the same day wants `…-athletic-2` typed by the person who knows it is the second game, not a suffix the app invented.

The **project name is not de-duplicated either**: two projects may legitimately be called the same thing, `Project::name` is not a key, and nothing in the app looks a project up by it.

**N4. The project name is `<Home> v <Away>`, built by core.** `metadata::match_name` already derives exactly that (`metadata.rs:210-214`) — but it takes `&Project`, and at Create time there is no `Project`: the sheet needs the name live from two `LineEdit` strings. So it is refactored to `pub fn match_name(home: &str, away: &str) -> Option<String>`, with the existing `&Project` caller as a two-line wrapper. `match_label(project)` (`metadata.rs:195-202`) is a **different** fallback chain — teams, else the project's own name, else `UNTITLED` — and must not be conflated with it.

A project created by this flow therefore has the name the exporter would have derived for it anyway, and one place decides how a match is written down. Names keep the capitalization the coach typed; only the folder slug lowercases.

**N5. The date is not stored in `project.json`, and nothing bumps.** A `Project::date` field would be a format change for a value already visible in the folder name and already derivable at export time. `CURRENT_FORMAT_VERSION` is **13** (`store.rs:21`) and stays there; `MIN_READABLE_FORMAT_VERSION` is 7 (`:26`) and stays there. **This feature adds no field to any stored struct**, which is the whole of the rule in `CLAUDE.md`: a bump is what a new field costs. See **Deferred 3** for the one thing that would like a stored date.

### S. The sheet

**S1. One `Sheet`, at 560 px, titled "New match".** `Sheet` is the card every modal in this app is drawn as — width, chrome, heading, body (`app.slint:1528-1558`) — inside a `Scrim` (`app.slint:1513-1519`). The widest thing in this one is the folder path line; 560 sits between the setup sheet's 520 (`app.slint:2359`) and the editor's 640 (`:2636`).

It **`inherits Sheet`**, as `ExportSheet` (`:1566`), `BasketSheet` (`:1731`) and `MatchEditorSheet` (`:2552`) do. The draft wrapped it in a `Rectangle` instead, copying `MatchSetupSheet`'s exception (`:2228`) — but that exception exists only because a `ColorPicker` hangs over the card as an absolutely-positioned sibling, and **this sheet has no colour picker**. `Sheet`'s own doc comment says there are five of these and names them (`:1524-1527`); this makes **six**, and that count is updated with it. Its `Scrim` goes in the window **before the error dialog's** (`app.slint:5489`): z-order is file order in Slint, and an error raised by Create has to be drawn over the sheet that raised it.

Top to bottom:

| Field | Prefilled from | Control |
|---|---|---|
| **Home team** | blank — the coach's own club | `SetupField`, name only |
| **⇄** | — | a button between the two; swaps the names |
| **Away team** | **I4** / blank | `SetupField`, name only |
| **Projects folder** | **W1** / **W2** | a path `SetupField` + `Choose…`, with a provenance line on the last tier only |
| **Folder name** | **N1**, then editable | `SetupField`, re-derived until touched |
| **Path** | derived | one read-only line: `<projects folder>/<folder name>/` |
| **Videos** | `order_videos` | a read-only numbered list, one row per file |

**S2. It reuses `SetupField` and extracts nothing.** `SetupField` is already a caption, an `✕` mark, an optional swatch and a two-way-bound `LineEdit` (`app.slint:1997-2067`), and it is used here with no swatch. The draft extracted a `TeamColumn` from `MatchSetupSheet` so the two sheets could share a name-plus-colours column; with no colours in this sheet there is nothing to extract, `MatchSetupSheet` is not touched, and the risk of regressing a shipped sheet goes with it. `ColorPicker` (`:2068`) and the by-hand picker dispatch it forces (`:2291-2294`) stay where they are, used by one sheet.

**S3. ⇄ Swap exchanges the two names**, and with them the folder name and the path line. It is the answer to home/away being unknowable: in the coach's own tree the opponent sits in the **home** slot and his club in **away** — an away fixture [observed] — and the user's own example for this spec has the same shape. **The guessed opponent (I4) nevertheless prefills the *away* slot, with the coach's own club in *home*:** that is the coach's decision, taken against this observation rather than inferred from it, and ⇄ is one click either way. Which team was at home is a fact about where the match was played, which is in no file and in no folder name. Watching the folder name flip is the fastest way to see which way round the coach wants it.

**S4. The video list does not probe.** No durations and no aspect: reading either means `probe`, which blocks for up to ten seconds per file (`probe.rs:15-16,41`) and would do it on the UI thread, stalling the video behind the scrim. **The bus probes, once, during Create** (**C2**), which is also the only place the aspect gate can honestly run.

There is no "Add files…" and no `×`: the picker that opened the sheet is the way in, Cancel plus New match… again is the way to change the selection, and the sidebar is the way to change it afterwards (**I6**).

**S5. Create is enabled only when everything it needs is good**, on the setup sheet's own model — *"a field can't read good and then fail to save"* (`main.rs:1237-1238`). It needs: **both team names non-blank** (the scoreboard guard, **C2**); a folder name that is non-blank and is exactly one `Component::Normal` — no `/`, no `..`, not `.`; an absolute projects-folder path whose **parent exists**; at least one video; and a target folder that does not already hold a `project.json`. Each is marked in its own field by the same call Create will make.

**The target check is a courtesy, not the guarantee:** a `try_exists` at prefill and on every edit, where the guarantee is the bus's own (**C2**), which cannot race.

**Create disables on the click and the sheet closes on `ProjectOpened` (C5), so `Event::Error` is what re-enables it** — any error arriving while the sheet is open does, because the alternative is a dead button after a refused Create and a coach who has to Cancel and re-pick every video. An unrelated error — a source found missing, a transcription failure — re-enables it too; that is benign, since the bus is single-threaded, a second `NewMatch` can only land after the first has finished, and it then refuses on the `project.json` the first one wrote.

**S6. The key guard takes the basket/editor shape, not the setup sheet's.** This sheet's `editing` folds into the window's `text-editing` (`app.slint:3386`), and Esc closes it **only from outside a field**; inside one, Esc leaves the field first and the next Esc closes. That is `BasketSheet`'s branch exactly (`:3664-3669`, whose comment says the fold is what stops the cascade being dead code), and the `reject` is also what delivers `Ctrl+V` into the path field. It goes before the setup sheet's branch in `handle-key` (`:3642-3716`), since only one sheet can be open at a time and the order is just a chain.

**The draft had this backwards.** It said the sheet *"takes the setup sheet's shape exactly"* — an Esc that closes unconditionally (`:3679-3689`) — **and** that its `editing` folds into `text-editing` as the others' do. Those contradict: with an unconditional Esc the fold is never read. And the justification was inverted. Re-opening the setup sheet costs one click on a shipped project; re-opening **this** sheet costs re-walking the file picker and re-picking every video. Esc-discards-everything is far more expensive here than it is there.

### C. The command

**C1. One new command, `Command::NewMatch`, not a sequence of the existing four.**

```rust
/// Creates `project_dir`, writes a project into it naming `videos` in the
/// order given, and opens it. All or nothing: nothing is created until every
/// video has been probed and accepted.
///
/// Every field is the sheet's, captured when Create was pressed. Nothing here
/// is read from the pipeline, so the caller-captured rule has nothing to bite
/// on.
NewMatch {
    project_dir: PathBuf,
    name: String,
    scoreboard: ScoreboardConfig,
    videos: Vec<PathBuf>,
},
```

**One path, not two.** The draft carried `projects_dir: PathBuf` and `folder: String` — the joined path taken apart so the bus could rejoin it and re-validate both halves, when the sheet has already computed and displayed exactly that join. The bus validates the one path it is given. (The draft's doc comment also read *"Create `pundit/<folder>/` under `projects_dir`"*, which, taken literally, specifies a second `pundit` level: `projects_dir` **is** that folder.)

Sending `OpenProject` + `AddSource` × n + `SetScoreboard` + `RenameProject` instead would be wrong in five separate ways, each of which is in the existing code:

- **The aspect gate fires between sources** (`bus/sources.rs:30`, `pundit-core/src/project.rs:614`). A second half whose shape differs is refused **after** the first half has been probed, pushed, saved and published — leaving a named folder holding one of a game's two halves, and the folder name is the one thing this flow cannot correct afterwards. Tree B's camera even hands out files whose durations are identical to the microsecond [observed]; nothing about a second file is guaranteed by the first. **This is the decisive argument.**
- **`OpenProject` refuses a folder that does not exist** — *"saving never creates one"* (`bus/project.rs:30-33`). The UI would have to create the directory itself, putting filesystem writes on the UI thread and splitting the "who makes folders" rule in two.
- **Each step saves and publishes on its own** (`bus/project.rs:187-190`). 4 + n writes of `project.json` and 4 + n `ProjectChanged` rebuilds, for one act.
- **`RenameProject` silently does nothing on a blank or unchanged name** (`bus/project.rs:79-89`), so the last step of the sequence has no failure the coach would see.
- **The half-built states are all reachable and all persistent.** The coach's own tree already contains a project folder with nothing in it [observed]. A half-built project is worse than none: it has a name, a `recordings/` folder and a `project.json` the coach will open next week expecting a match.

One command makes the whole thing one transaction, and it reuses every existing piece. No new undo action — **creating a project is not an undo step**, as opening one is not; `commit` clears the history (`bus/project.rs:104-106`).

**C2. The order of operations. Nothing touches the disk until every video has been accepted.**

1. **Refuse.** `refuse_if_busy()` first (`bus/export.rs:367-378`): an export or a preview must not have the project swapped underneath it, and the export sheet is explicitly designed to be closed while a run continues, so `New match…` is clickable mid-export. Then: `project_dir` absolute, its file name exactly one `Component::Normal`, and its parent — the projects folder — either present or itself having a parent that is (**W3**), `name` non-blank after trimming, `videos` non-empty, and **both team names non-blank**.
2. **Probe every video, in order, and gate each against the ones before it.** `probe` then `check_aspect` (`bus/sources.rs:211-218`), accumulating `(path, Probe)`. The first failure aborts: `Event::Error` naming the file and the reason, **nothing created**.
3. **`create_dir(projects_dir)` if it is missing** — the leaf only, never `create_dir_all`, so a typo in the hand-editable path (**W3**) makes one directory under an existing parent rather than a tree.
4. **`create_dir(project_dir)`.** `AlreadyExists` is **not** a refusal on its own: an existing folder is refused only if a `project.json` is inside it, and an empty one is **adopted**. That is exactly what `OpenProject` already does (`bus/project.rs:45-57`, with `StoreError::MissingProjectJson` there to distinguish the two), and it is what fixes the empty stranded folder this spec opens by observing [observed] — the coach who made one by hand can now point the flow at it.
5. **Canonicalize the project folder**, then compute each `SourceRef`. Canonical on both sides because the kernel resolves `..` physically, and a cloud-sync mount may well be reached through a symlink (`bus/sources.rs:241-245`).
6. **Build the `Project`** in memory: `Project::new(name)` (`pundit-core/src/project.rs:431-444`), `scoreboard: Some(config)`, and each `SourceRef` pushed as it passes `project.check_aspect(...)`. The gate needs no new logic — the in-memory project answers it exactly as the incremental path does.
7. **`store::write(&project_dir, &mut project)`** — creates `recordings/`, writes `project.json` through a temp file and a rename (`store.rs:175-205`).
8. **`commit(project_dir, project)`** — the existing function, unchanged (`bus/project.rs:93-126`).

**Two small refactors make steps 6 and 1 honest rather than duplicated:**

- **`probed_source` splits** (`bus/sources.rs:211-237`). `relative_path` needs a canonical folder that does not exist until step 4, so the probe cannot stay welded to the `SourceRef` construction. `source_ref(folder, path, probe) -> Result<SourceRef, UserError>` is the second half, called by both paths.
- **The blank-team-name guard moves out of `set_scoreboard`.** `bus/scoreboard.rs:191-194` refuses a blank name *there* precisely so *"the render path never has to guard one"* (`:185-190`) — and `NewMatch` writes `scoreboard: Some(config)` straight onto the project, walking around it. One `storable(&ScoreboardConfig) -> Result<(), UserError>` that both call is what makes one place decide whether a scoreboard can be stored.

**C3. What each failure does.**

| Failure | What happens |
|---|---|
| Busy: an export or preview is running | `Event::Error(UserError::CantExport(_))`. Nothing done, nothing probed. |
| A video can't be probed (missing, not video, rotated, needs a plugin — `probe.rs:29-37`) | `Event::Error(UserError::Source(_))` naming the file. **Nothing created.** The sheet stays open with its fields intact. |
| A video's shape differs from the first (`UserError::AspectMismatch`, `bus/mod.rs:478-482`) | The same: named, nothing created, sheet open. |
| `create_dir` fails — read-only disk, no permission, a missing parent | `Event::Error(UserError::Io(_))` naming the path. Nothing created. |
| The target folder already holds a `project.json` | Refused, with a message naming it. An **empty** folder is adopted instead (**C2** step 4). |
| `store::write` fails after the folder was made | `Event::Error`. **Nothing is rolled back**, and nothing needs to be: the folder is now an empty one, which the next Create adopts. The draft's two `remove_dir`s were **unreliable rather than impossible**: `store::write` creates `recordings/` first (`store.rs:176`) and writes `.project.json.tmp` before renaming it (`:202`), so a failure while serializing leaves only `recordings/` and the rollback works, while a failure in the temp write or the rename — `ENOSPC`, `EIO`, a dropped mount — leaves the temp file and the second `remove_dir` returns `ENOTEMPTY`. A rollback that covers the cheap failures and not the expensive ones is worse than none. |
| A project was already open | Untouched until step 8. Every mutation saves as it happens (`bus/project.rs:187-190`), so there is nothing unsaved to lose, and a Create that fails at any step leaves the coach in the project he was in. |

**C4. Errors here are modal, not notices.** `UserError::Io`, `UserError::Source` and `UserError::CantExport` are not in `is_notice`, so they raise the error dialog, which is right: the coach pressed a button and is looking at a sheet waiting for an answer. Unlike the match event editor's refusals (spec C5) there is no live take to protect, and the status line would be hidden behind the scrim anyway.

**C5. The sheet closes on `ProjectOpened` and on nothing else.** Not on the click. The one event that says the project exists is the one that dismisses the sheet, so a failure leaves the sheet up with the coach's typing in it.

### X. What this does not do

- **It does not move, copy, rename or delete any footage.** The project points at the files where they are (**W4**).
- **It does not write another project's `project.json`.** It refuses a folder that holds one (**C3**), and it adopts an empty one. It does, through `commit`, **empty the previously open project's trash** (`bus/project.rs:107-109`) — permanently deleting the clip recordings that project had in undo. That is by design and predates this flow; the draft's *"it does not touch an existing project"* was simply false.
- **It does not scan for videos** (the coach picks them, **E2**, **I7**) **and it does not write outside the chosen projects folder** — one leaf directory and one match folder under it, and nothing else, ever.
- **It stores no state of its own.** No new `state.json` key: W1 re-derives from the footage and W2 reads the field #85 already keeps.
- **It does not change the project format.** No new field, no version bump (**N5**).

---

## Crate responsibilities

| Crate | Contents |
|---|---|
| `pundit-core` | `naming.rs` grows `safe_chars`, `folder_slug`, `truncate_on_boundary`, `parse_date_in(text, max_year)` and `opponent_from` beside the shipped `order_videos` — all pure, all over `&str` and `&Path`, all tested with no filesystem and no clock. `metadata::match_name` becomes `pub fn match_name(home, away)`. **No new dependency:** the audit still lists exactly `serde`, `serde_json`, `thiserror`, `uuid` — no regex crate (the date shapes are a byte scan), no unicode crate (**N2**). |
| `pundit-media` | **Nothing.** `Probe` gains no field; the container's creation time is not read (**I3**). |
| `pundit-app` | `bus/mod.rs`: `Command::NewMatch`, off the allow-list. `bus/project.rs`: `new_match`. `bus/sources.rs`: `probed_source` split, yielding `source_ref`. `bus/scoreboard.rs`: the blank-name guard extracted as `storable`. `bus/export.rs`: `file_name` calls `naming::safe_chars`. `bus/basket.rs`: `file_stem`'s hand-rolled truncation calls `truncate_on_boundary`. **A new `pub mod new_match;`**: the `APP_NAME` walk, the neighbour read and the prefills. `main.rs`: wiring and `on_create_match`. UI: `EmptyCard`'s secondary action, `NewMatchSheet` and its `handle-key` branch. |
| `pundit-harness` | The command end to end, and each of its refusals. |

**The rules live in the app library, not in `main.rs`**, which is wiring and has no `#[cfg(test)]` module at all; `fit.rs`, `match_panel.rs`, `drawing.rs` and `zoom_input.rs` are this crate's established pattern of "a testable rule is a `pub mod` with its tests beside it". **And the environment is passed in**, exactly as `bus/state.rs`'s `films_dir(videos, home)` and `config_dir(xdg, home)` take theirs — whose doc says the rule is worth a test and that is why (`bus/state.rs:305-311`). So `projects_dir_for(match_folder: &Path, home: Option<&Path>, last_project: Option<&Path>) -> ProjectsDir`, reaching the filesystem only through `is_dir`.

## Testing

**No test writes outside a `tempfile::tempdir()`, and no test reads the coach's folders.** Every layout below is reconstructed structurally in a temp dir with empty files.

- **`core::naming`:**
  - `safe_chars` over each forbidden character and a control character, and — the regression pin that matters — **`file_name`'s output unchanged for a label with `--` in it, one with trailing dots, and one over 64 bytes**. Those three are exactly what the draft's shared pipeline would have silently rewritten, and a test that only checks a clean label passes either way.
  - `folder_slug` lowercases, turns whitespace runs into one `-`, collapses, truncates and trims; a name that is entirely punctuation yields empty; a name whose byte-64 cut lands on a `.` or a space comes back with **neither**, which is the truncate-before-trim order; and `truncate_on_boundary` never splits a multi-byte character, at 64 or at 200.
  - `parse_date_in`: each accepted shape; a **bare four-digit year is not a date**; a **six-digit run is not a date**; month 13 and day 32 refused; a year above `max_year` refused (with `max_year` passed, so the test has no clock in it); the leftmost of two dates wins; an eight-digit run inside a longer digit run refused.
  - `opponent_from`: a `YYYY-MM-DD-` prefix stripped; a digit-code prefix stripped; underscores and dots to spaces; title-cased; a name of digits alone yields `None`.
  - `match_name(home, away)` builds `"A v B"` and is `None` when either side is blank; the `&Project` wrapper agrees with it.
- **`pundit-app::new_match`:**
  - the `APP_NAME` walk over **both reconstructed layouts**, finding it at depth 2 in each; the walk stopping at four levels, and at the injected home directory;
  - the fall to `last_project`'s parent when no such directory is above, and to an app-named directory **beside** the footage (`<match folder's parent>/<APP_NAME>`, never the bare parent — **W2**) when there is no last project either;
  - the neighbour seeding: three reconstructed `project.json`s whose folder names carry different dates, the newest one's config cloned with the two typed names substituted; an unreadable and a too-new file skipped; **a neighbour with a newer mtime but an older folder date not chosen**; no neighbour at all giving `blank_config`.
- **Harness (`tests/new_match.rs`), all inside a temp dir:**
  - `NewMatch` with two fixture videos creates `<folder>/project.json` and `recordings/`, publishes **one** `ProjectOpened`, stores both sources in the order given with `../` relative paths, and reads back through `store::read` with the scoreboard and the name;
  - **an existing empty folder is adopted** and an existing folder holding a `project.json` is **refused**, the first project's file byte-identical afterwards;
  - **a blank team name is refused** and nothing is created — the guard `set_scoreboard` has and `NewMatch` would otherwise have bypassed;
  - a video that can't be probed: an error, **and no directory created** — asserted by listing the projects folder;
  - a second video with a different aspect: the same, and in particular **no folder holding one of the two**;
  - a projects folder whose parent does not exist: an error and **no tree created**; a read-only one: an error, nothing created;
  - `NewMatch` during an export: refused, the run unaffected; and the previously open project still open and unchanged after each refusal;
  - `NewMatch` while recording is dropped (the deny-by-default allow-list, `bus/mod.rs:895-934`).
- **Manual (batched, needs the user's eyes):**
  - run New match… against both real trees and confirm the projects folder, the date in the folder name, the opponent and the video order are right before pressing Create;
  - confirm ⇄ Swap rewrites the folder name and the path line live, and that typing in the folder field stops it;
  - confirm Esc leaves a field first and closes the sheet second.

## Risks

1. **The W2 tiers are weaker than W1.** `last_project`'s parent is a fact rather than an inference — but a coach who works on two clubs and was last in the other one gets the other club's folder. One editable field, a picker beside it, and a path line showing the consequence.
2. **The order inference can be silently wrong.** `order_videos` covers both of the coach's camera systems, but a coach who renames files by hand can defeat it. The list in the sheet is the check, and the sidebar's drag is the fix (**I6**).
3. **The date is cosmetic here and load-bearing elsewhere.** It names the folder and nothing else (**N5**), so a wrong date costs a badly-named folder. The export's date tag is still mtime's (**Deferred 3**), so the two can now disagree.
4. **Probing several large files on a network mount blocks the bus for the length of the Create.** `probe` allows ten seconds each (`probe.rs:15-16`); the existing `AddSource` has exactly this property one file at a time. Create is disabled while it runs and the sheet is up, so the coach sees a modal rather than a frozen window — but there is no progress indication. **Q3.**
5. **The seeded colours are the previous match's opponent's**, so one slot is wrong every match — but it is a valid kit corrected in `Setup…`, not a blank the coach must invent, and the draft's histogram was a lot of machinery for a slightly better wrong answer.

## Deferred

1. **Scanning a folder for candidate videos**, as an alternative to picking files. It needs a rule for which files are halves, which is the guess E2 exists to avoid.
2. **Reading the container's creation time.** Useless on 100% of the coach's current footage (**I3**). Revisit only if footage from a phone or a camcorder turns up, and then as a `Probe` field.
3. **Making the export's date tag agree with the match date.** `bus/export.rs:639` derives it from the first source's mtime, which the coach's own trees show is **one to two days** after the match [observed] — the doc comment's *"within a day"* is already optimistic. Once folders carry a real date, the cheapest fix is to parse it out of the project folder's name at export time: pure, no format change, and it reuses `naming::parse_date_in`. **Revisit first.**
4. **Creating a whole season's projects at once** from a folder of match folders. The per-match sheet has to be right before a batch of them can be. **A "Duplicate this match's setup" action** for a second recording of the same game is the same deferral; the neighbour seeding (**I5**) already gets most of the way there.
5. **Recent projects and a project drawer** — BACKLOG #85, which shares W2's field with this flow. Build it after this.
6. **Renaming a project's folder** to match a corrected name after the fact. It means moving a directory with a `project.json`, a `recordings/` tree and every source's `relative_path` pointing back out of it — a different feature with its own failure modes. **It is also the reason the folder name is the one field this sheet cannot afford to get wrong.**

## Open questions for the user

- **Q1. Answered by W2:** where the projects folder goes when none is above the videos is `last_project()`'s parent — what the app already remembers — not an inference from the footage's depth. No clamps, no `st_dev`, no two-levels-up guess.
- **Q2. Answered "neither":** the colours are not in this sheet at all, in two fields or in three. `TeamConfig::new` still defaults the font colour to the secondary (`scoreboard.rs:38-47`), which is what the seeded config carries.
- **Q3. Should Create show progress while it probes?** On a cloud-sync mount, two 1 GB files could take a few seconds each.
  **Default:** no — a disabled button and the modal sheet. Progress means a second event stream for an operation that is usually instant.
- **Q4. Should the date go in the project name as well as the folder?**
  **Default:** no, as decided. The folder carries it and the export title is derived from the name (`metadata.rs:210-214`).
- **Q5. Should `New match…` be the primary action of the no-project card, demoting `Open Project…` to a secondary button?**
  **Default:** yes (**E1**). Creating is the common case; opening an existing project is what the toolbar and `Ctrl+O` are for (`app.slint:3726-3732`).
- **Q6. Should the flow reuse the teams of the last opened project when the chosen projects folder has no readable neighbour?**
  **Default:** no. A coach who works on two clubs would get the other club's kit, and `blank_config` is honest about knowing nothing.

  **The unit is the chosen folder, not the tree, and that is what makes this consistent.** **I5** seeds from the newest readable project *in the projects folder the sheet is pointing at* — never from "the last project" as a privileged source. So where **W2**'s second tier has made the last project's own folder the projects folder, the neighbour read lands there by the ordinary rule: the coach is putting this match into that folder, and a match in a folder inherits that folder's most recent kit. This answer forbids reaching *across* to a folder the coach did not choose; it does not forbid reading the one they did. The correction, if the kit is wrong, is `Setup…` — one click, and the two names were typed by hand either way.

## R. What the first draft got wrong, kept as the record

- **It put the kit colours and the match format in the sheet.** Four hex fields with validators, two number fields with theirs, and a `TeamColumn` extracted out of a shipped sheet to share them — all for values `Setup…` edits in one click. The draft never asked which of its fields had *no* correction path; only one does.
- **Its citations had drifted ~60%.** The spec was five days old against ~115 commits. `CURRENT_FORMAT_VERSION` was cited as 11 and is 13 (v12 slates, v13 the per-clip inset); the conclusion — nothing bumps — holds, but for the real reason, which is that this feature stores no field, not an arithmetic of 11 → 12.
- **`next_free` and the refusal contradicted each other.** N4 suffixed a taken folder name `-2`; C2 and S5 refused it. Both were in the same document.
- **The rollback worked only for the failures that did not matter.** Its two `remove_dir`s cleared `recordings/` and then the folder, which succeeds when `store::write` fails while serializing and returns `ENOTEMPTY` when it fails in the temp write or the rename — the realistic ones (`store.rs:176`, `:202`). The fix is not a better rollback: an empty folder is now **adopted**, as `OpenProject` adopts one, which makes the coach's stranded empty folder usable instead of an error. The draft's own manual test asked for both behaviours in one sentence.
- **`create_dir_all` on a hand-typed path** would have made a whole tree out of a typo, and **`NewMatch` walked around the scoreboard guard**, writing `scoreboard: Some(config)` past the blank-name refusal that exists so the render path never has to guard one.
- **It pointed export's `file_name` at the whole new naming pipeline**, which would have renamed every future export, orphaned the sidecars beside every file already written, and collided long labels past byte 64 — and its proposed regression test would have passed anyway.
- **It trimmed before truncating**, leaving a name that can end in a `.` or a space on the one filesystem class that silently strips them — and it claimed export's cleaner was the app's only one, when `bus/basket.rs` has a second with its own budget and its own documented reason.
- **`metadata::match_name` takes `&Project`**, so "it becomes `pub`" was not implementable from two `LineEdit`s.
- **Its key guard was self-contradictory and back-to-front**, and its `NewMatch` carried a path in two halves so the bus could rejoin what the sheet had already joined. **§S6** and **§C1** have both.
- **It said it "does not touch an existing project":** `commit` empties the old project's trash.
- **Its club histogram** read every `project.json` in the folder, counted names case-insensitively, required two appearances and kept per-slot bookkeeping across four tiers — on the UI thread, in a sheet that elsewhere refuses to call `probe` for exactly that reason. One read of the newest neighbour gets a whole valid config.

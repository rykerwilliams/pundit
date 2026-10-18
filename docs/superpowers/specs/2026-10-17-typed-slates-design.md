# Design — typing slates in as lines, as match events can be typed

**BACKLOG #97.** The coach asked for it on 2026-10-17, which meets the entry's
own "when to revisit" (`BACKLOG.md:2396`–`:2397`): *"if a coach arrives with
times already written down — which is exactly how `kickoffs.txt` came about"*.

**Status: decided with the coach (2026-10-17); both spec reviews applied.**
Read `CLAUDE.md` first — its *"Match events are tagged at the playhead or
typed, in one grammar"* and *"Slates"* sections — and the two specs this one
builds on: `2026-09-25-slates-design.md` (§S6 is why this was dropped, §S9 what
stayed out) and `2026-09-23-match-event-editor-design.md` (the editor this
mirrors). Every `path:line` below was re-derived at `5a4c707` (`main` after
#96, whose tree is the same as `aa39d50`'s).

**The design, in one paragraph.** A new pure module, `core::slate_entry`, reads
one slate per line — `[<video>] <in>-<out> <name> [<tags>]`, e.g.
`2 14:05-14:40 corner routine [corners, set piece]` — reusing `match_entry`'s
`parse_time`, `format_time` and its "a leading bare integer is a video number"
rule, and nothing else of its grammar. **`#` is a comment only at the start of
a line**, so it cannot eat a tag or a name; tags are a bracketed,
comma-separated group at the end, fed through `normalize_tags`. A range is
required (`14:05-` leaves it open); a lone time is refused, which is also what
catches a block of match events pasted into the wrong box. **The paste loop is
lifted out of `match_entry` into a generic `core::line_batch`**, which both
grammars use. It lives in the match event editor's sheet, generalized to **one
`LineEditorSheet` with a kind** (events or slates) — the list as retypable
lines, a paste box, a live echo, Done — opened for slates from a **Type…**
button in the Slates header. A committed row edits its slate **in place** (id,
title and place in the stored order kept), an Add is one `EditSlates` undo
step, and nothing new is stored: **no format bump**.

---

## §W. What is asked for, and what §S6 said stood in the way

**W1. The ask** is the match editor's, for ranges: a coach with a list of
moments on paper should not have to scrub to each one and press `i` and `o`.

**W2. §S6's two objections, and how each is met** (slates spec §S6;
`BACKLOG.md:2381`–`:2388`):

- **`#` is a comment to end of line in `match_entry`** — `body()` is
  `line.split('#').next()` (`crates/pundit-core/src/match_entry.rs:294`–`:296`).
  The obvious `#corners` would be silently discarded. → This grammar is not
  `match_entry`'s: tags are not marked with `#` at all (S4), and `#` is a
  comment only as a line's first character (S5).
- **The words after the time are a closed vocabulary** — `parse_kind` refuses
  any word that is not an event word or a team name (`match_entry.rs:191`–`:226`),
  and a slate's tail is free text. → Nothing of `parse_kind`, `tokens`,
  `REFUSED_WORDS` or the team matching is reused (S1).

**W3. What the entry said *is* reusable** (`BACKLOG.md:2389`–`:2391`):
`parse_time`, `format_time` and the video-number rule, "in a new
`core::slate_entry`, not a second mode bolted onto `match_entry`". That is a
rule about the **line grammar**, and this spec keeps it. It is not a rule about
the **batch container** — numbering lines, skipping blanks, judging each
against what is stored and what the block has already accepted, joining the
refused lines back up — which is grammar-free and is shared (S12).

---

## §S. Decisions

### The grammar

**S1. A new module, `core::slate_entry`, beside `match_entry` and sharing three
things of its grammar.**

- `match_entry::parse_time` (`match_entry.rs:63`) and `format_time` (`:111`) —
  already `pub`.
- `is_video_number` (`:299`) and `refuse_video` (`:368`) — private today; they
  become `pub(crate)`. The refusal's words (*"there is no video 3 — a time needs
  a colon (15:00)"*) are then the same sentence in both boxes.

Nothing else of the grammar is shared, deliberately. `body()` is the `#` rule
this grammar rejects; `tokens()` turns `-` into a space, which would split a
range; `parse_kind` is the closed vocabulary.

**S2. A line is `[<video>] <range> [<name>] [[<tags>]]`, read left to right:**

| Part | Rule |
|---|---|
| **video** | Optional. A leading bare integer, **always** a video number (`match_entry`'s rule, CLAUDE.md). Out of `1..=source_videos.len()` it is `refuse_video`'s sentence. Absent, the sheet's default-video picker decides (S12). |
| **range** | `<in>-<out>` or `<in> - <out>` for a closed range; `<in>-` for an open one. Each time is exactly what `parse_time` accepts (`m:ss`, `h:mm:ss`, optional tenths, a colon required). **The dash has a space on both sides or on neither** — see below. It may be `-`, `–` or `—`, because a word processor autocorrects a typed `-` between spaces into an en or em dash, and notes typed there must paste as they read. |
| **name** | The trimmed text between the range and the tag group. Free text, **no vocabulary**: may be empty (the row shows "Untitled", as today). |
| **tags** | Optional. A `[…]` group that **ends the line**; its inside goes through `tag::normalize_tags` (`crates/pundit-core/src/tag.rs:24`), so it is the tag field's own comma rule — trimmed, lowercased, de-duplicated, and a tag may contain spaces. |

**The dash's spacing is what separates a closed range from an open one whose
name starts with a time.** After the in time:

- a dash with **no space before it**: if the next character is the start of a
  time, it is `<in>-<out>`; if it is a space or the end of the line, the range
  is **open** and whatever follows is the name. So `5:00- 5:30 drill` is an open
  range named *5:30 drill*, and `5:00-5:30 drill` is closed and named *drill*.
- a dash with **a space before and after it**: `<in> - <out>`, closed. A time
  must follow.
- a dash with a space **before** it and none after (`5:00 -5:30`): refused,
  *"put a space on both sides of the dash or on neither"*. (A space after it
  and none before is the open case above.)

So: `2 14:05-14:40 corner routine [corners, set piece]` is video 2, 845.0 to
880.0 s, named *corner routine*, tagged `corners` and `set piece`.

**S3. What is refused, each in words that name the fix.** A refusal is a
`Refused(String)`, as `match_entry`'s is; these are the sentences:

| Line | Refusal |
|---|---|
| `900 corner` | *there is no video 900 — a time needs a colon (15:00)* (shared, S1) |
| `2 corner`, `2 14:05 corner`, `2 14:05 - corner` | *a slate is a range: 14:05-14:40, or 14:05- to leave its end open* |
| `2 1405-1440 corner` | the same sentence — a time without a colon is not a time |
| `2 14:05 -14:40 x` | *put a space on both sides of the dash or on neither* |
| `2 14:40-14:05 x` | *the range ends before it starts* (also an equal pair: `mark_slate_out`'s `<=`, `crates/pundit-core/src/project.rs:897`) |
| either end past the video | *Half 2 is 47:13.0 long* — `match_entry`'s bound and wording (`match_entry.rs:347`–`:353`): refused, never clamped |
| `… corner [corners` | *the tags aren't closed — end them with ] (a name with a [ in it needs [] after it)* |
| `… [corners] more` | *nothing may follow the tags (a name with a [ in it needs [] after it)* |

**The bound and the order are checked on the seconds the line will store** —
after S9's per-end override, not on the parse of the text — so nothing can
carry a stored time past the end of a video or behind its own in point. That
is `match_entry`'s order (`match_entry.rs:333`–`:353`: the override, then the
bound).

**A lone time is refused, not read as an open range.** It is the one line most
likely to arrive by mistake — every match event line has exactly one time, so a
block of `2 14:05 home goal` pasted into the wrong box would otherwise add a
column of open slates named *home goal*. An open range is one keystroke away
(`14:05-`), and it is the Slates list's own rendering of one. The coach chose
this (Q3, below).

**The kick-off words are not refused here.** `match_entry` refuses them because
a spurious start/stop shifts every later period (`match_entry.rs:128`–`:143`);
a slate never reaches `interpret`, and *kick-off routine* is a perfectly good
name.

**S4. Tags are a bracketed group, not a sigil.** `[corners, set piece]`:

- It **cannot collide with a comment** — §S6's whole objection.
- It carries **a tag with a space in it**, which `normalize_tags` allows and a
  `#word` sigil cannot express.
- Its inside is **the tag field's own text** — comma-separated, the one rule
  (`normalize_tags`) — so nothing new is learned and nothing is normalized
  twice.
- **The group starts at the last `[` on the line, and it must end the line.**
  That is what lets a *name* contain brackets when tags follow it
  (`[draft] corner [corners]` is named `[draft] corner`). A name containing a
  `[` with **no** tags needs an empty `[]` after it, which is what S9's
  `format_line` writes and what both bracket refusals say.

The coach chose the brackets over `#corners` and a `|` separator (Q1, below).

**S5. `#` is a comment only as a line's first non-blank character.** A line that
is blank or a comment is `Nothing` — no echo row, as `match_entry`'s. A `#`
anywhere else is part of the name: `corner #3` is named *corner #3*. A trailing
`kickoffs.txt`-style comment therefore lands in the name — visible in the echo
before Add, so a surprise and not a loss. `match_entry`'s end-of-line rule
would cut every name that numbers something.

**S6. The title (v18) is not in the line.** `Slate.title` is words burned over
the footage (`project.rs:521`–`:530`); it is edited in the Slates panel's
*Title in exports* field (`crates/pundit-app/ui/app.slint:6297`–`:6315`) and
**a line edit leaves it exactly as it was** (S9). A third part of the line
would need a third delimiter, and the title is the one field a coach composes
rather than copies from notes. That is the default the coach accepted (Q2,
below).

### Where it lives

**S7. One sheet, the match event editor's, generalized to a kind — not a copy,
not a section, not a second instance.** `MatchEditorSheet`
(`app.slint:3357`–`:3655`) is already list-agnostic in shape: rows of `{id,
where-text, label, role-less}` (`MatchEditorRow`, `:160`–`:172`), one retypable
`line` for the selected row, `→` and `×`, a paste box with a default-video
picker, a live `PasteEcho` (`:184`–`:193`), a message line, Done. What is
event-specific is words, and one callback's doc. So:

- **`MatchEditorSheet` → `LineEditorSheet`, `MatchEditorRow` →
  `LineEditorRow`**, with a new `export enum LineEditorKind { events, slates }`.
- **One instance** (`app.slint:7874`'s `Scrim`), and **one set of window
  properties**, renamed `match-editor-*` → `line-editor-*`
  (`app.slint:4816`–`:4838`), plus `line-editor-kind`. Only one of the two can
  be open at a time, so a second set would be state that is never live.
- **Two paste strings, so each box keeps its own block** for the session
  (editor spec F2): `events-paste` and `slates-paste`. The sheet's box binds
  `line-editor-paste`; opening copies the kind's string in, and
  `close-line-editor()` copies it back. A project open clears both, as it
  clears the one today (`crates/pundit-app/src/main.rs:3545`–`:3548`).
- **The words.** One `noun` property (`"event"` / `"slate"`) for the three
  strings that differ by a noun alone, and one whole-sentence property per
  string that differs in more — all set in `app.slint` from the kind, so Rust
  sets no UI text. The census of the sheet's event-specific text:

  | Line | Today | Slates | Property |
  |---|---|---|---|
  | `:3441` | title `"Match events"` | `"Slates"` | `title` (sentence) |
  | `:3444` | empty text, *"Nothing tagged yet. Paste a block below, or tag with Z, X and V while the game video plays."* | *"No slates yet. Paste a block below, or mark one with I and O while the game video plays."* | `empty-text` (sentence) |
  | `:3498` | `→` tooltip, *"Play the game video from here. The picture is behind this sheet, so it closes."* | *"Show this slate's first frame. The picture is behind this sheet, so it closes."* — it **parks**, it does not play (S17) | `go-tooltip` (sentence) |
  | `:3510` | `×` tooltip, *"Delete this event (undoable)"* | *"Delete this slate (undoable)"* | from `noun` |
  | `:3552` | heading *"Paste events"* | *"Paste slates"* | from `noun` |
  | `:3574` | placeholder `"2 14:05 away goal"` | `"2 14:05-14:40 corner routine [corners]"` | `placeholder` (sentence) |
  | `:3578` | hint, *"One event per line: …"* | *"One slate per line: an optional video number, a range — 14:05-14:40, or 14:05- to leave it open — a name, then tags in [brackets]. # at the start of a line is a comment."* | `hint` (sentence) |
  | `:3398`–`:3400` | `valid-line`'s doc names `bus::editor_line` and "cap and all" | — | doc reworded: "the very call the command makes, for either kind" |

  The `Add` and `Done` tooltips (`:3629`, `:3651`) and `"Lines with no number
  are:"` (`:3557`) are already kind-free. The echo's summary and button come
  from Rust (S12).
- **`role-less`** greys a row; for a slate it marks an **open** range, the same
  job (a row that is stored but not yet doing anything), and what the Slates
  list's `timed` flag already says (`app.slint:40`–`:44`).
- **The window's callbacks become kind-free** — `line-editor-select`,
  `-reseed`, `-commit`, `-go`, `-remove`, `-add`, and the pure `valid-line` and
  `check-paste` — and **branch on `line-editor-kind` in Rust**. The Match
  panel's own `seek-match-event` and `delete-match-event` stay (they are the
  panel's); the events arms of `-go` and `-remove` call the same Rust as they
  do.
- **One `handle-key` branch** (`app.slint:5430`–`:5448`, renamed), **one close
  function** (`close-match-editor`, `:5193`–`:5198`), **one notice arm** that
  routes by kind (S15), **one rebuild gate** (S17).

A copy would be ~300 lines of Slint to keep in step by hand; a second instance
would be a second set of nine window properties of which one set is always
dead. A *shared* paste box that dispatched each line by whether its time is a
range was Q4's option C, which the coach did not choose.

**S8. The door is a "Type…" button in the Slates header — and the header shows
once the project has a video, even with no slates.** Today the whole section is
hidden until a slate exists (`app.slint:5964`–`:5983`), on the grounds that *"a
third list in a 240 px column would otherwise shrink the clip list"*. But the
first paste is this feature's main case and comes *before* any slate exists, so
a door inside that section would never be seen when it is needed.

**So the section is restructured, not just re-gated.** Today one `if`
(`:5983`) holds a `VerticalLayout` whose first child is the header
`HorizontalLayout` — the "Slates" label, the tag filter `LineEdit` and its `✕`
(`:5986`–`:6032`) — followed by the empty text and the list. After:

- the outer `if` becomes `root.sources.length > 0`;
- the header row holds the label, then an **inner** `if` (today's condition,
  `root.slates.length > 0 || root.slate-tag-filter != ""`) around the filter
  field and its `✕`, then **Type…** outside that inner `if`;
- the empty text and the list keep today's condition, now as their own `if`s.

The cost is one header row in a project that never uses slates — not the list
the original reason was about. The button follows the house rule
(`keys.focus()` first) and is enabled on `can-play && !recording &&
!previewing`, the Match panel's `can-edit` (`app.slint:7098`). **No key**: the
events editor has none either, and the keymap's `showRecents` shows how one is
added later at no cost.

The coach chose this door (Q4, below).

### Round trip

**S9. `slate_entry::format_line(&Slate) -> String`, and an edit keeps what it
was not asked to change.**

```text
2 14:05.0-14:40.0 corner routine [corners, set piece]
1 3:20.0- free kick
1 52:10.0-52:31.5 [press]
1 5:00.0- 5:30 drill
1 0:10.0-0:20.0 a [b []
```

- **The video number is always written**, as `match_entry::format_line`'s is
  (`match_entry.rs:248`–`:259`), so a row's line means the same thing whatever
  the paste box's picker says.
- **Times are `format_time`'s floored tenths, and the dash is a plain `-` with
  no spaces**: closed `in-out`, open `in-` followed by a space and the name
  (S2's rule, so an open range named *5:30 drill* comes back as one).
- **No tags, no brackets** — *except* when the name contains a `[`, where an
  empty `[]` is written so that S4's last-`[` rule finds a group that ends the
  line. One `if`, pinned by round-trip tests on `a [b`, `a [b] c` and `[x]`.
- **`edit_from_line(project, &Slate, typed)` keeps the stored seconds per
  end** — `match_entry::edit_from_line`'s discipline (`match_entry.rs:268`–`:278`).
  **The identity test compares the in time and the out time as text**, split
  on the dash and trimmed — not whitespace tokens, because `14:05.0 - 14:40.0`
  and `14:05.0-14:40.0` hold the same two times. An unchanged in keeps a
  captured `845.06` rather than re-rounding it to `845.0`; an unchanged out
  likewise; a changed one takes what was typed. S3's order and bound are then
  checked on those effective seconds.
- **A line byte-identical to the seed changes nothing at all.** Enter's double
  commit (editor spec T4) and a focus loss over an untouched row are already
  free without this — the per-end rule keeps both times, the name and tags read
  back the same, and `edit_slates` drops a no-op (`crates/pundit-app/src/bus/slates.rs:316`–`:318`).
  **The rule exists for the `[`-in-a-tag hole**: a tag containing `[`, which the
  panel's tag field allows, does not survive S4, so without it a row nobody
  edited would be rewritten — its tags and its name both changed — the moment
  focus left it.
- **An emptied row** is refused with `nothing on that line — a row reads "<seed>"`
  — `bus::scoreboard::editor_line`'s words (`bus/scoreboard.rs:70`–`:73`).
  Deleting is the row's `×`.
- **The title is not in the seed and not touched** (S6). Neither is
  `Clip::slate_id` on any clip (S11).

Because slates carry no cap and no cross-record rule, there is **no
`bus::editor_line` equivalent**: the row field's ✕ mark (in `main.rs`) and the
bus command both call `slate_entry::edit_from_line`. One call, two callers.

### Mapping a line to a `Slate`

**S10. A new slate is built by one constructor, and a retyped one is edited in
place.**

- **`PendingSlate { source_index, in_seconds, out_seconds: Option<f64>, name,
  tags }`** in `slate_entry` — no id, which the project mints, as
  `PendingMatchEvent` (`match_entry.rs:37`–`:42`). Not `Copy` (it holds
  `String`s), which nothing needs.
- **`Project::add_slate(PendingSlate) -> Uuid`** pushes it with a fresh id and
  `title: None`. **`mark_slate_in` becomes a call to it**
  (`project.rs:866`–`:878`), so there is one place a `Slate` literal is built
  outside tests.
- **A retype** is a new `Project::retype_slate(id, PendingSlate) -> bool`,
  `#[must_use]` as `edit_match_event` is (`crates/pundit-core/src/scoreboard.rs:813`),
  writing the five fields and keeping `id`, `title` and the slate's **position
  in the stored `Vec`**. Not five `SlateEdit`s: a retype can move the slate to
  another video, which no `SlateEdit` does, and can reopen a range (`14:05-`),
  which `SlateEdit::Out` cannot express (`project.rs:569`–`:571`). Name
  trimming stays `edit_slate`'s rule (`project.rs:921`), applied in the parser.
- **Order needs nothing.** Display is `slates_sorted` (`project.rs:943`–`:951`)
  and the stored order is only the marked order. Keeping a retyped slate's
  position keeps `o`'s meaning — *"the range most recently opened"*
  (`project.rs:880`–`:885`) — for slates that were marked by key.
- **A typed open range joins that order at the end.** So after pasting
  `2 14:05- corner`, the next `o` on video 2 closes *that* one. That is the
  rule doing what it says, named in Risks rather than special-cased.
- **Moving a slate to another video by typing its number is allowed**, unlike
  *Set in*/*Set out* from the playhead, which refuse it out loud (BACKLOG #122,
  `main.rs:1754`–`:1781`). Those buttons refuse because the *playhead* happened
  to be on another video; a typed number is the coach saying which video.

**S11. A shot slate edits like any other, and its clip is not touched.** The
link runs from the clip (`Clip::slate_id`, `project.rs:340`–`:349`) and is set
once, at the take; the clip's anchor is its recording's, not the slate's.
Retyping a shot slate's range is therefore what *Set in*/*Set out* already do
to one (both enabled on `!recording && !previewing` with no `shot` test,
`app.slint:6350`–`:6373`), and renaming one does not rename its clip — the
inheritance is at the take only (slates spec §S5). The row keeps reading *shot*
because "has it been shot?" is a scan of the clips. The one visible effect is
v18's: a titled slate's words burn over **its** range, so moving the range
moves them — as a Set-in does today. The coach chose no note on the row (Q5,
below).

### The batch

**S12. The paste loop becomes a generic `core::line_batch`, and both grammars
use it.** `match_entry::parse_batch`'s loop (`match_entry.rs:424`–`:469`) is
grammar-free apart from two calls: the line's parse and the judgement of a
parsed line against what is stored and what the block has accepted. So:

```rust
pub enum BatchVerdict<T> { Added(T), Duplicate(T), Refused(String) }
pub struct BatchLine<T> { pub number: usize, pub text: String, pub verdict: BatchVerdict<T> }
pub struct Batch<T> { pub items: Vec<T>, pub lines: Vec<BatchLine<T>>, pub leftover: String }
pub enum Admit { Add, Duplicate, Refuse(String) }

/// `parse`: `None` for a blank or comment line, else the line's verdict.
/// `admit`: judges a parsed line against the items this block has accepted.
pub fn read_batch<T: Clone>(
    text: &str,
    parse: impl Fn(&str) -> Option<Result<T, String>>,
    admit: impl Fn(&T, &[T]) -> Admit,
) -> Batch<T>
```

- **The loop moves whole**: 1-based numbering over every line, blank and
  comment lines skipped with no echo, the leftover as the refused lines alone
  joined by `\n` (editor spec B5).
- **Match events move onto it with their behaviour unchanged.**
  `match_entry::parse_batch` keeps its signature and becomes a call: its
  `admit` is today's duplicate test (`already_tagged`, `:475`–`:490`, within
  `SAME_EVENT_SECONDS`) and today's start/stop cap, **counted from `accepted`**
  — `project.start_stop_count()` plus the accepted start/stops — rather than a
  counter mutated in the loop, which is the same number. Its existing tests
  (`crates/pundit-core/tests/match_entry.rs:378`–`:464`) pin that, changed only
  where they name a renamed item: `AlreadyTagged` → `Duplicate`, `events` →
  `items`.
- **A slate duplicate is canonical text, not a tolerance.** Two slates are the
  same when `(source_index, format_time(in), out.map(format_time), name)` are
  equal, the name compared exactly (it is already trimmed, S2) — against the
  stored slates rendered the same way and against the block's accepted ones.
  **The name is in the key** because two slates on one range are legal and
  plausible — the same corner once for the attack, once for the defence. No
  `SAME_EVENT_SECONDS`: a range is typed from notes in whole tenths at most,
  so "the same text" is the same slate, and a second either side is not a
  doubled paste but a different range.
- **The echo and the notice take their words as arguments.**
  `match_panel::paste_echo` (`crates/pundit-app/src/match_panel.rs:143`) becomes
  generic over `T`, taking the noun (`"event"` / `"slate"`), the duplicate's
  words (`"already tagged"` / `"already marked"`) and a `reads_as` closure; it
  moves, with its tests (`:688`ff.), into a new `line_editor.rs`.
  `bus::scoreboard::batch_notice` (`bus/scoreboard.rs:245`) takes the
  duplicate's words likewise and becomes `pub(super)` for `bus/slates.rs`.
- **The slate echo** reads `✓ 2 · 14:05.0–14:40.0 · corner routine · corners,
  set piece`; `• … — already marked, skipped`; `✗ line 4: "…" — <reason>`.
  Summary *"3 slates to add · 1 already marked · 2 lines refused"*, button
  *"Add 3 slates"*.
- **Default video** — the same picker, reset to the first video on every open
  (editor spec F2, `main.rs:2283`–`:2285`).
- **Best-effort, not all-or-nothing; the box keeps the refused lines alone**,
  handed back by the bus's own parse (editor spec B5). `Event::MatchPasteLeftover`
  (`crates/pundit-app/src/bus/mod.rs:584`–`:591`) **folds into
  `Event::PasteLeftover { kind, text }`**, `kind` a bus enum `PasteKind {
  MatchEvents, Slates }`; `main.rs` writes it to the sheet when that kind is
  open and to the kind's paste string otherwise.
- **No overlap rule.** Overlapping ranges are legal today (`i` twice).

### Commands, undo, the bus

**S13. Two commands in `bus/slates.rs`, both through `edit_slates`.**

```rust
/// Retype one slate from a line of `core::slate_entry`. Typed, not captured.
EditSlateLine { id: Uuid, line: String },
/// A pasted block, parsed by the bus against the project as it lands.
AddSlates { text: String, default_source: usize },
```

The shapes of `EditMatchEvent` and `AddMatchEvents` (`bus/mod.rs:192`–`:210`),
for their reasons: **the bus parses** rather than trusting slates parsed at the
UI, so the bound and the duplicate rule are read once, from core, against the
project as it is. Typed times satisfy the caller-captured rule trivially — the
sheet never reads the playhead (editor spec C2).

**S14. One undo step per committed row and per Add.** `Bus::edit_slates`
(`bus/slates.rs:309`–`:322`) snapshots the whole list around a closure, drops a
no-op, saves, records `UndoAction::EditSlates` and publishes once — so twenty
pushes in one closure are one save, one step and one `ProjectChanged`. No new
`UndoAction`, and so no new arm in `purge_for_source_change`'s exhaustive match
(`crates/pundit-core/src/undo.rs:178`–`:184`): `EditSlates` is already purged
on a source move or removal, which is what keeps a typed video number honest
across a reorder.

**S15. Refusals are `UserError::Slate` notices, and the sheet repeats them.** A
notice already (`bus/mod.rs:694`–`:697`, `:719`). The status bar renders behind
the scrim, so `main.rs`'s notice arm — which copies `UserError::Scoreboard` to
the match editor's line while it is open (`main.rs:3759`–`:3765`) — becomes one
test: the sheet is open and the error is **its kind's** (`Scoreboard` for
events, `Slate` for slates).

**S16. Not on the recording allow-list, and the door is greyed during a take.**
The allow-list is deny-by-default, so this is nothing to write; `MarkSlateIn`
and `MarkSlateOut` stay on it (`bus/mod.rs:1128`–`:1129`) because they are
placed from the footage, which a typed line is not.

**S17. The rows follow the project except while the row's field has focus,
and Go parks.**

- **The rebuild gate is today's** (`main.rs:3981`–`:3985`): `show_project`
  rebuilds the open editor's rows on every `ProjectChanged` unless its row
  field has focus, now for whichever kind is open. Every commit drops focus
  (editor spec T4).
- **A slate's Go is `park_on_slate`** (`main.rs:4249`–`:4256`) — **select, then
  jump** — and the sheet closes, because the picture is behind it. Selecting
  first is what makes `R` then shoot the slate on screen
  (`ToggleRecording { slate }` carries the selection) and what moves the
  scrubber's span to it, which a bare `JumpToSlate` (`bus/slates.rs:88`–`:99`)
  would not. It parks; it does not play.

**S18. CLAUDE.md is wrong about the rebuild, and is corrected with this
feature.** Its editor bullet (`CLAUDE.md:1047`–`:1053`) says the rows are
*"rebuilt by its own committed edit and never from `show_project`
(`main.rs`'s `editor_rebuild` flag)"*. There is no such flag: `show_project`
rebuilds the rows on every project change **unless the row's field has focus**
(`main.rs:3981`–`:3985`), and the editor spec's T3 explains why a one-shot flag
was rejected (two commands in flight; a refused command publishes nothing). The
bullet is rewritten to say that in this spec's commit; the plan's prose task
extends it to the slates kind when the code does.

### Format

**S19. Nothing new is stored; `CURRENT_FORMAT_VERSION` stays 18**
(`crates/pundit-core/src/store.rs:21`). A typed slate writes exactly the fields
`Slate` has had since v12 (and leaves v18's `title` as it was). The paste boxes'
text is window state, session-only, cleared when a project opens — the match
editor's F2.

---

## Crate responsibilities

| Crate | Contents |
|---|---|
| `pundit-core` | `line_batch.rs` (new): `Batch<T>`, `BatchLine<T>`, `BatchVerdict<T>`, `Admit`, `read_batch`. `match_entry.rs`: `parse_batch` a call to it, its own `Batch` types gone; `is_video_number` and `refuse_video` `pub(crate)`. `slate_entry.rs` (new): `PendingSlate`, `parse_line`, `format_line(&Slate)`, `edit_from_line`, `parse_batch`. `project.rs`: `add_slate` (which `mark_slate_in` calls), `retype_slate`. No new dependency. |
| `pundit-media` | Nothing. |
| `pundit-app` | Bus: `EditSlateLine`, `AddSlates` in `bus/slates.rs` through `edit_slates`; `Event::PasteLeftover { kind, text }` replacing `MatchPasteLeftover`; `batch_notice` generic in its words. `line_editor.rs` (new): the generic `paste_echo` (moved from `match_panel.rs`) and the slate rows and `reads_as`. `main.rs`: the window's `line-editor-*` callbacks branching on kind, `open_line_editor(kind)` / `show_line_editor`, the notice arm, the rebuild gate, Go through `park_on_slate`. UI: `LineEditorSheet` + `LineEditorKind`, the renamed properties and the two paste strings, the per-kind words, the Slates header's restructure and **Type…**. |
| `pundit-harness` | The batch, the retype and their undo, over the bus; `match_events.rs`'s `MatchPasteLeftover` reader renamed. |

## Testing

- **Core — `tests/line_batch.rs` (new):** a stub grammar (one integer per line)
  shows the numbering over blank and comment lines, a duplicate judged against
  the accepted items, a refusal in the leftover and a duplicate not.
- **Core — `tests/match_entry.rs`:** every existing batch test still passes
  with only the two renames — that is the proof the lift changed nothing.
- **Core — `tests/slate_entry.rs` (new):**
  - every S2 shape: with and without a video number; `-`, `–`, `—`, each with
    no spaces and with a space on both sides; an open range; an empty name;
    tags with a space in one; tags normalized (`[Corners, corners ]` →
    `["corners"]`); a name containing `[`…`]` before the tags; `#` at the start
    (a comment) and in the middle (part of the name);
  - **every S3 refusal by its words**, including the lopsided dash, and **a
    lone time refused** — a block of `match_entry` lines yields no slate;
  - **the kick-off words are a name here**: `2 14:05-14:40 kick-off routine`;
  - the bound on **each** end, named; `out <= in` refused at equality;
  - `parse_line(format_line(s))` gives back `s`'s fields for a closed range,
    an open one, no tags, **an open range named `5:30 drill`**, and names
    **`a [b`, `a [b] c` and `[x]`** with no tags (S9's `[]`);
  - `edit_from_line`: an untouched line changes nothing, **including a slate
    with a tag containing `[`**; an untouched **in** keeps a stored `845.06`
    while the out moves, **both when typed `14:05.0-14:45.0` and
    `14:05.0 - 14:45.0`**; a changed video number under unchanged times keeps
    both stored seconds; **an order check on the effective seconds** (stored
    in `845.06`, out typed `14:05.0` = 845.0 is refused); an emptied line
    refused;
  - `parse_batch`: a block containing a line twice adds it once; a line equal
    to a stored slate's canonical text is a duplicate; **the same range with
    another name is added**; a range a tenth off is added; `leftover` holds the
    refused lines alone.
- **Core — `tests/slates.rs`:** `retype_slate` keeps `id`, `title` and the
  stored position, can reopen a range and move a video, and returns false for
  an unknown id; `o` after a pasted open range closes it (the rule, pinned so a
  later change to it is deliberate).
- **App, headless (`line_editor.rs`):** the moved `paste_echo` tests for events,
  unchanged in their expected strings; the slate echo's three verdicts, its
  summary and button; the slate rows' `where`, label and open flag.
- **UI — `tests/ui/`:**
  - `text_editing.rs`'s sheet probe (`a_letter_typed_in_a_sheet_field_reaches_the_field`,
    `:258`; ids `"MatchEditorSheet::line-edit"` `:284` and `::paste-edit`
    `:292`, renamed) runs **once per kind**;
  - **Esc, per kind**: Esc in the row field puts the seeded line back and
    leaves the sheet open; a second Esc closes it (a new test, beside
    `esc_in_a_sheet_field_leaves_the_field_before_it_closes_the_sheet`);
  - `sheet_scroll.rs`'s list (`:90`, `card: "MatchEditorSheet"`, renamed)
    gains the slates kind as an entry of its own;
  - a new `slate_editor.rs`: **Type…** is found by its label **with zero
    slates** and opens the sheet on the slates kind; the sheet reads *"Slates"*
    and the events kind still reads *"Match events"*; a slate's Go selects it
    (`selected-slate`) before it jumps.
- **Harness — `tests/slates.rs`:** an `AddSlates` of N publishes one
  `ProjectChanged` and one `Undo` removes all N; a partly refused block adds
  the rest, emits one `UserError::Slate` and one `PasteLeftover { Slates, … }`
  holding the refused lines alone; `EditSlateLine` moves a **shot** slate's
  range and the clip's `slate_id`, anchor and name are unchanged; a retype of a
  titled slate keeps its title; the same line sent twice records one undo step.
- **Manual (batched):** paste a real list of ranges, filter by a pasted tag,
  `g`, and shoot two — the themed pass (#120) over typed slates is the
  workflow this is for; Go on a row, then `R`, shoots that row; Esc in the row
  field cancels and in the box leaves it first; paste a block typed in a word
  processor (autocorrected dashes).

## Risks

1. **A wrong time is silent**, as in the events editor. Go is the check.
2. **The `[`-in-a-tag hole.** A tag containing `[` (the panel allows it) does
   not round-trip through S4. An untouched row is safe (S9); **an edited row
   silently changes both its tags and its name** — the last `[` is now inside
   the tag, so the tag group starts there and the rest of the old group joins
   the name. Rare enough not to forbid `[` in tags, which would be a new rule
   on a shipped field.
3. **`o` closes a typed open range.** S10 says why that is the rule working;
   a coach who pastes open ranges and then marks by key may still be surprised.
   Q3's answer (a lone time is refused) keeps open ranges rare.
4. **Generalizing a shipped sheet** risks the match editor. The per-kind UI
   tests, `sheet_scroll.rs` and `text_editing.rs` over both kinds, and the
   unchanged `match_entry` and `paste_echo` expectations are the guard.
5. **The Slates header now always shows.** A layout change in a column whose
   space was argued over (#87, #123); measured in the plan, not assumed.
6. **A slate preview can be retyped under the sheet.** `PreviewSlate` arms the
   range (`bus/slates.rs:124`–`:132`) and the out-point stop re-reads the
   stored out on each poll (`:218`–`:235`), so retyping that slate's out while
   it plays moves where it stops — the same exposure *Set out* has today.
   Accepted: the sheet covers the picture, and the arm is a preview's.
7. **Retyping a titled slate open stops its title drawing** until it is closed
   again: a title is drawn only over a closed range
   (`crates/pundit-core/src/caption.rs:124`). That is the title's own rule, and
   the row shows the range open.

## Out of scope

- **Exporting the slates as a film** (BACKLOG #98) — unchanged by this.
- **Copying the slate list out as text**, the events editor's own top deferral
  (editor spec, *Deferred*). `format_line` makes it a button; nobody asked.
- **A hint in the events box** when a line there holds a range. The events box
  today says *"no time (use m:ss)"*, which is true, and a cross-reference
  between the boxes is a coupling to add only if it bites.
- **Loading from a file**, multi-select, a whole session as one undo step —
  out for the events editor's reasons.
- **Typing a title** — Q2, below.

## Decided with the coach (2026-10-17)

The coach took the recommended answer to every question. The options he was
shown are kept so the record says what was turned down.

**Q1. How are tags written on a line? — A, a bracket group at the end:
`corner routine [corners, set piece]`.** It can't be confused with a comment,
it carries a tag with a space in it, and the inside is exactly what the tag
field takes (S4). Turned down: B, `#corners` with `#` no longer a comment (no
spaces in a tag, and `corner #3` couldn't be a name); C, a `|` separator.

**Q2. Should a line carry the title burned into exports? — A, no: the title
stays the panel's field, and a line edit never touches it (S6).** *Taken as the
default and not put to the coach.* Turned down: B, a quoted title at the end of
the line.

**Q3. What does a line with only a start time mean? — A, it is refused; `14:05-`
leaves the end open (S3).** That is what stops a pasted list of goals becoming
open slates, and `14:05-` is how the Slates list already shows an open range.
Turned down: B, a lone time read as an open range; C, a fixed default length.

**Q4. Where do you go to type slates in? — A, a "Type…" button in the Slates
header, which shows whenever the project has a video (S8).** It sits beside the
list it fills. Turned down: B, a button beside "Edit events…" in the Match
panel; C, one paste box in the events editor taking both kinds.

**Q5. When a slate that has already been recorded is retyped, does its row say
anything? — A, no: it edits like any other, and the recorded clip is untouched
(S11).** That is what Set in / Set out already do to a recorded slate. Turned
down: B, a note under the row; C, refusing range changes on a recorded slate.

# Design — typing slates in as lines, as match events can be typed

**BACKLOG #97.** The coach asked for it on 2026-10-17, which meets the entry's
own "when to revisit" (`BACKLOG.md:2396`–`:2397`): *"if a coach arrives with
times already written down — which is exactly how `kickoffs.txt` came about"*.

**Status: draft, before review.** Read `CLAUDE.md` first — its *"Match events
are tagged at the playhead or typed, in one grammar"* and *"Slates"* sections —
and the two specs this one builds on:
`2026-09-25-slates-design.md` (§S6 is why this was dropped, §S9 what stayed out)
and `2026-09-23-match-event-editor-design.md` (the editor this mirrors). Every
`path:line` below was re-derived on `claude/97-spec` at `aa39d50`.

**The design, in one paragraph.** A new pure module, `core::slate_entry`, reads
one slate per line — `[<video>] <in>-<out> <name> [<tags>]`, e.g.
`2 14:05-14:40 corner routine [corners, set piece]` — reusing `match_entry`'s
`parse_time`, `format_time` and its "a leading bare integer is a video number"
rule, and nothing else of it. **`#` is a comment only at the start of a line**,
so it cannot eat a tag or a name; tags are a bracketed, comma-separated group at
the end, fed through `normalize_tags`. A range is required (`14:05-` leaves it
open, the row's own rendering); a lone time is refused, which is also what
catches a block of match events pasted into the wrong box. It lives in a sheet
that **is** the match event editor's component, generalized and instantiated a
second time — the slates as retypable lines, a paste box, a live echo, Done —
opened from a **Type…** button in the Slates header. A committed row edits its
slate **in place** (id, title and place in the stored order kept), an Add is one
`EditSlates` undo step, and nothing is stored that is not stored today: **no
format bump**.

---

## §W. What is asked for, and what §S6 said stood in the way

**W1. The ask** is the match editor's, for ranges: a coach with a list of
moments on paper should not have to scrub to each one and press `i` and `o`.

**W2. §S6's two objections, and how each is met** (slates spec §S6;
`BACKLOG.md:2381`–`:2388`):

- **`#` is a comment to end of line in `match_entry`** — `body()` is
  `line.split('#').next()` (`crates/pundit-core/src/match_entry.rs:294`–`:296`).
  The obvious `#corners` would be silently discarded. → This grammar is not
  `match_entry`'s: tags are not marked with `#` at all (§S4), and `#` is a
  comment only as a line's first character (§S5).
- **The words after the time are a closed vocabulary** — `parse_kind` refuses
  any word that is not an event word or a team name (`match_entry.rs:191`–`:226`),
  and a slate's tail is free text. → Nothing of `parse_kind`, `tokens`,
  `REFUSED_WORDS` or the team matching is reused (§S1).

**W3. What the entry said *is* reusable** (`BACKLOG.md:2389`–`:2391`):
`parse_time`, `format_time` and the video-number rule, "in a new
`core::slate_entry`, not a second mode bolted onto `match_entry`". This spec
takes that literally.

---

## §S. Decisions

### The grammar

**S1. A new module, `core::slate_entry`, beside `match_entry` and sharing three
things with it.**

- `match_entry::parse_time` (`match_entry.rs:63`) and `format_time` (`:111`) —
  already `pub`.
- `is_video_number` (`:299`) and `refuse_video` (`:368`) — private today; they
  become `pub(crate)`. That is the whole of the change to `match_entry`, and
  the refusal's words (*"there is no video 3 — a time needs a colon (15:00)"*)
  are then the same sentence in both boxes.

Nothing else is shared, deliberately. `body()` is the `#` rule this grammar
rejects; `tokens()` turns `-` into a space, which would split a range;
`parse_kind` is the closed vocabulary. A shared "line reader" with a mode flag
is exactly the "second mode bolted onto `match_entry`" W3 rules out.

**S2. A line is `[<video>] <range> [<name>] [[<tags>]]`, read left to right:**

| Part | Rule |
|---|---|
| **video** | Optional. A leading bare integer, **always** a video number (`match_entry`'s rule, CLAUDE.md). Out of `1..=source_videos.len()` it is `refuse_video`'s sentence. Absent, the sheet's default-video picker decides (S12). |
| **range** | `<time>-<time>`, or `<time>-` for a range left open. Each time is exactly what `parse_time` accepts (`m:ss`, `h:mm:ss`, optional tenths, a colon required). The dash may be `-`, `–` (en dash, which is what the Slates list renders — `main.rs:4155`–`:4161` — so a range copied off a row reads back) or `—`, with or without spaces around it: `14:05-14:40`, `14:05 – 14:40`. |
| **name** | Everything after the range up to the tags, trimmed. Free text, **no vocabulary**: may be empty (the row shows "Untitled", as today). |
| **tags** | Optional. A `[…]` group that **ends the line**; its inside goes through `tag::normalize_tags` (`crates/pundit-core/src/tag.rs:24`), so it is the tag field's own comma rule — trimmed, lowercased, de-duplicated, and a tag may contain spaces. |

So: `2 14:05-14:40 corner routine [corners, set piece]` is video 2, 845.0 to
880.0 s, named *corner routine*, tagged `corners` and `set piece`.

**S3. What is refused, each in words that name the fix.** A refusal is a
`Refused(String)`, as `match_entry`'s is; these are the sentences:

| Line | Refusal |
|---|---|
| `900 corner` | *there is no video 900 — a time needs a colon (15:00)* (shared, S1) |
| `2 corner`, `2 14:05 corner` | *a slate is a range: 14:05-14:40, or 14:05- to leave its end open* |
| `2 1405-1440 corner` | the same sentence — a time without a colon is not a time |
| `2 14:40-14:05 x` | *the range ends before it starts* (also an equal pair: `mark_slate_out`'s `<=`, `project.rs:897`) |
| either end past the video | *Half 2 is 47:13.0 long* — `match_entry`'s bound and wording (`match_entry.rs:347`–`:353`): refused, never clamped |
| `… corner [corners` | *the tags aren't closed — end them with ]* |
| `… [corners] more` | *nothing may follow the tags* |

**A lone time is refused, not read as an open range.** It is the one line most
likely to arrive by mistake — every match event line has exactly one time, so a
block of `2 14:05 home goal` pasted into the wrong box would otherwise add a
column of open slates named *home goal*. An open range is still one keystroke
away (`14:05-`), and it is the Slates list's own rendering of one. **Q3** asks
the coach whether that is right.

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
- **The group is the last `[` on the line, and it must end the line.** That is
  what lets a *name* contain brackets (`[draft] corner [corners]` is named
  `[draft] corner`), and S8 says how a name that *ends* in `]` round-trips.

`#corners` and a `|` separator are the alternatives; **Q1** puts them to the
coach.

**S5. `#` is a comment only as a line's first non-blank character.** A line that
is blank or a comment is `Nothing` — no echo row, as `match_entry`'s. A `#`
anywhere else is part of the name: `corner #3` is named *corner #3*. A trailing
`kickoffs.txt`-style comment therefore lands in the name — visible in the echo
before Add, so a surprise and not a loss. The alternative, `match_entry`'s
end-of-line rule, would cut every name that numbers something.

**S6. The title (v18) is not in the line.** `Slate.title` is words burned over
the footage (`project.rs:521`–`:530`); it is edited in the Slates panel's
*Title in exports* field (`app.slint:6297`–`:6315`) and **a line edit leaves it
exactly as it was** (S9). A third part of the line would need a third
delimiter, and the title is the one field a coach composes rather than copies
from notes. **Q2** asks whether that is right.

### Where it lives

**S7. A sheet, and it is the match event editor's sheet, generalized — not a
copy and not a section of it.** `MatchEditorSheet` (`app.slint:3357`–`:3655`)
is already list-agnostic in shape: rows of `{id, where-text, label, role-less}`
(`MatchEditorRow`, `:160`–`:172`), one retypable `line` for the selected row,
`→` and `×`, a paste box with a default-video picker, a live `PasteEcho`
(`:184`–`:193`), a message line, Done. Everything event-specific is words —
the title, the empty-list text, the placeholder, the hint line. So:

- **Rename it `LineEditorSheet`** and lift those four strings into `in
  property <string>`s; the match editor sets them to today's text.
- **Instantiate it a second time**, `if root.slate-editor-open: Scrim {
  LineEditorSheet { … } }`, beside the match editor's (`app.slint:7874`), with
  its own `slate-editor-*` window properties — so each sheet's paste box keeps
  its own block for the session (editor spec F2) and neither can clobber the
  other's.
- **`role-less`** greys a row; for a slate it marks an **open** range, which is
  the same job (a row that is stored but not yet doing anything) and the
  Slates list's `timed` flag already says it (`app.slint:40`–`:44`).
- **The `handle-key` guard** (`app.slint:5430`–`:5448`) becomes one branch for
  *either* editor — `if (root.match-editor-open || root.slate-editor-open)` —
  with its row-field test reading whichever is open. The two branches would be
  identical otherwise, and the Esc-cancels-in-the-row rule (editor spec P3) is
  the subtle one to keep in one place.

A copy would be ~300 lines of Slint to keep in step by hand. A *section* of the
events sheet would make one sheet with two paste boxes and two echoes, and put
slates behind a button labelled "Edit events…". A *shared* paste box that
dispatched each line by whether its time is a range is tempting — and it is
**Q4**'s option C — but it couples two undo lists into one Add and two comment
rules into one box.

**S8. The door is a "Type…" button in the Slates header — and the header shows
once the project has a video, even with no slates.** Today the whole section is
hidden until a slate exists (`app.slint:5964`–`:5983`), on the grounds that *"a
third list in a 240 px column would otherwise shrink the clip list"*. But the
first paste is this feature's main case and comes *before* any slate exists, so
a door inside that section would never be seen when it is needed.

So the gate splits: the **header row** (the "Slates" label and the new button)
shows whenever there is a source; the **filter field and the list** keep
today's gate exactly. The cost is one header row in a project that never uses
slates — not the list the original reason was about. The button follows the
house rule (`keys.focus()` first) and is enabled on `can-play && !recording &&
!previewing`, the match editor's `can-edit` (editor spec P4). **No key**: the
events editor has none either, and the keymap's `showRecents` shows how one is
added later at no cost (CLAUDE.md, *every action has a `handle-key` branch*).

**Q4** asks the coach where he would look for it.

### Round trip

**S9. `slate_entry::format_line(&Slate) -> String`, and an edit keeps what it
was not asked to change.**

```text
2 14:05.0-14:40.0 corner routine [corners, set piece]
1 3:20.0- free kick
1 52:10.0-52:31.5 [press]
```

- **The video number is always written**, as `match_entry::format_line`'s is
  (`match_entry.rs:248`–`:259`), so a row's line means the same thing whatever
  the paste box's picker says.
- **Times are `format_time`'s floored tenths**, and the dash is a plain `-` —
  typeable, where the list's `–` is not.
- **No tags, no brackets** — *except* when the name itself ends in `]`, where
  an empty `[]` is written so that S4's "last group ends the line" rule does
  not read the name's brackets as tags. One `if`, pinned by a round-trip test.
- **`edit_from_line(project, &Slate, typed)`** is `match_entry::edit_from_line`'s
  discipline, per end: **each time token byte-identical to the seed's keeps the
  stored seconds**, so renaming a slate whose in point is a captured `845.06`
  does not re-round it to `845.0`. And **a line byte-identical to the seed
  changes nothing at all** — which is what makes Enter's double commit (editor
  spec T4) and a focus loss over an untouched row free, and what keeps a slate
  whose tags contain `[` (allowed by the panel's field, unreadable by S4) from
  being rewritten by a row nobody edited.
- **An emptied row** is refused *"nothing on that line — a row reads "<seed>""*
  — `bus::scoreboard::editor_line`'s words (`bus/scoreboard.rs:70`–`:73`).
  Deleting is the row's `×`.
- **The title is not in the seed and not touched** (S6). Neither is
  `Clip::slate_id` on any clip (S11).

Because slates carry no cap and no cross-record rule, there is **no
`bus::editor_line` equivalent**: the row field's ✕ mark (in `main.rs`) and the
bus command call `slate_entry::edit_from_line` itself. One call, two callers —
the editor spec's T5 discipline with one layer fewer.

### Mapping a line to a `Slate`

**S10. A new slate is `mark_slate_in`'s slate with every field given, and a
retyped one is edited in place.**

- **`PendingSlate { source_index, in_seconds, out_seconds: Option<f64>, name,
  tags }`** in `slate_entry` — no id, which the project mints, as
  `PendingMatchEvent` (`match_entry.rs:37`–`:42`). Not `Copy` (it holds
  `String`s), which nothing needs.
- **Add** pushes each in input order with a fresh `Uuid`, `title: None` — the
  record `mark_slate_in` builds (`project.rs:866`–`:878`) with the out point,
  name and tags filled in. A core `Project::add_slate(PendingSlate) -> Uuid`
  does it, and `mark_slate_in` becomes a call to it.
- **A retype** is a new `Project::retype_slate(id, PendingSlate) -> bool`,
  `#[must_use]` as `edit_match_event` is, writing the five fields and keeping
  `id`, `title` and the slate's **position in the stored `Vec`**. Not five
  `SlateEdit`s: a retype can move the slate to another video, which no
  `SlateEdit` does, and can reopen a range (`14:05-`), which `SlateEdit::Out`
  cannot express (`project.rs:569`–`:571`). Name trimming stays `edit_slate`'s
  rule (`project.rs:921`), applied in the parser.
- **Order needs nothing.** Display is `slates_sorted` (`project.rs:943`–`:951`)
  and the stored order is only the marked order. Keeping a retyped slate's
  position keeps `o`'s meaning — *"the range most recently opened"*
  (`project.rs:880`–`:885`) — for slates that were marked by key.
- **A typed open range joins that order at the end.** So after pasting
  `2 14:05- corner`, the next `o` on video 2 closes *that* one. That is the
  rule doing what it says, and it is named in Risks rather than special-cased.
- **Moving a slate to another video by typing its number is allowed**, unlike
  *Set in*/*Set out* from the playhead, which refuse it out loud (BACKLOG #122,
  `main.rs:1754`–`:1781`). Those buttons refuse because the *playhead* happened
  to be on another video; a typed number is the coach saying which video.

**S11. A shot slate edits like any other, and its clip is not touched.** The
link runs from the clip (`Clip::slate_id`, `project.rs:340`–`:349`) and is
set once, at the take; the clip's anchor is its recording's, not the slate's.
Retyping a shot slate's range is therefore what *Set in*/*Set out* already do
to one (both enabled on `!recording && !previewing` with no `shot` test,
`app.slint:6350`–`:6373`), and renaming one does not rename its clip — the
inheritance is at the take only (slates spec §S5). The row keeps reading
*shot* because "has it been shot?" is a scan of the clips. The one visible
effect is v18's: a titled slate's words burn over **its** range, so moving the
range moves them — as a Set-in does today. **Q5** asks whether the row should
say so.

**S12. The paste box takes the match editor's rules, re-worded for ranges.**

- **Default video** — the same picker, reset to the first video on every open
  (editor spec F2, `main.rs:2283`–`:2285`).
- **Echo** — one line per line, three glyphs: `✓ 2 · 14:05.0–14:40.0 · corner
  routine · corners, set piece`; `• … — already marked, skipped`;
  `✗ line 4: "…" — <reason>`. Summary *"3 slates to add · 1 already marked · 2
  lines refused"*, button *"Add 3 slates"*.
- **A duplicate is the same video, both ends within `SAME_EVENT_SECONDS`
  (`match_entry.rs:33`) — an open end matching only an open end — and the same
  name**, against existing + accepted-so-far, for the editor spec's V3 reason
  (a block pasted twice). **The name is in the key** because two slates on one
  range are legal and plausible — the same corner watched once for the attack
  and once for the defence — where two goals in one second are not.
- **Best-effort, not all-or-nothing**; **the box keeps the refused lines and
  nothing else** after Add, handed back by the bus's own parse (editor spec
  B5) as `Event::SlatePasteLeftover(String)`, the sibling of
  `MatchPasteLeftover` (`bus/mod.rs:584`–`:591`).
- **No overlap rule.** Overlapping ranges are legal today (`i` twice), and the
  list shows them.

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
the match editor's line while it is open (`main.rs:3759`–`:3765`) — copies
`UserError::Slate` to the slate editor's line the same way. A batch's notice is
one aggregated sentence, `batch_notice`'s shape (`bus/scoreboard.rs:230`ff.).

**S16. Not on the recording allow-list, and the door is greyed during a take.**
The allow-list is deny-by-default, so this is nothing to write; `MarkSlateIn`
and `MarkSlateOut` stay on it (`bus/mod.rs:1128`–`:1129`) because they are
placed from the footage, which a typed line is not.

**S17. The rows follow the project except while the row's field has focus** —
the editor spec's T3 invariant, exactly: `show_project` rebuilds the slate
editor's rows on every `ProjectChanged` while it is open and its
`line-focused` is false, as it does the match editor's (`main.rs:3981`–`:3985`).
Every commit drops focus (T4), and Go — `JumpToSlate`, which parks without
playing (`bus/slates.rs:88`–`:99`) — closes the sheet, because the picture is
behind it.

### Format

**S18. Nothing new is stored; `CURRENT_FORMAT_VERSION` stays 18**
(`crates/pundit-core/src/store.rs:21`). A typed slate writes exactly the fields
`Slate` has had since v12 (and leaves v18's `title` as it was). The paste box's
text is a window property, session-only, cleared when a project opens — the
match editor's F2.

---

## Crate responsibilities

| Crate | Contents |
|---|---|
| `pundit-core` | `slate_entry.rs`: `PendingSlate`, `parse_line`, `format_line(&Slate)`, `edit_from_line`, `parse_batch` with its own `Batch` / `BatchLine` / `BatchVerdict` (namespaced, not generic over `match_entry`'s). `match_entry::is_video_number` and `refuse_video` become `pub(crate)`. `project.rs`: `add_slate`, `retype_slate`; `mark_slate_in` calls `add_slate`. No new dependency. |
| `pundit-media` | Nothing. |
| `pundit-app` | Bus: `EditSlateLine`, `AddSlates`, `Event::SlatePasteLeftover`, all in `bus/slates.rs` through `edit_slates`. A headless `slate_editor.rs` beside `match_panel.rs`: the rows, the echo and the summary wording (sharing `match_panel`'s `plural`, made `pub(crate)`). `main.rs`: `wire_slate_editor` / `open_slate_editor` / `show_slate_editor`, the notice routing, the rebuild rule. UI: `MatchEditorSheet` → `LineEditorSheet` with four text properties, its second instance, the joined `handle-key` branch, the Slates header's new gate and **Type…** button. |
| `pundit-harness` | The batch, the retype and their undo, over the bus. |

## Testing

- **Core — `tests/slate_entry.rs` (new):**
  - every S2 shape: with and without a video number, `-` / `–` / `—` with and
    without spaces, an open range, an empty name, tags with a space in one,
    tags normalized (`[Corners, corners ]` → `["corners"]`), a name containing
    `[`…`]` before the tags, `#` at the start (a comment) and in the middle
    (part of the name);
  - **every S3 refusal by its words**, and in particular **a lone time
    refused** — a block of `match_entry` lines yields no slate;
  - **the kick-off words are a name here**: `2 14:05-14:40 kick-off routine`
    parses;
  - the bound on **each** end, named; `out <= in` refused at equality;
  - `parse_line(format_line(s))` gives back `s`'s fields within a tenth, for a
    closed range, an open one, no tags, and **a name ending in `]`** (S9's
    `[]`);
  - `edit_from_line`: an untouched line changes nothing; an untouched **in**
    token keeps a stored `845.06` while the out moves; a changed video number
    under unchanged times keeps both stored seconds; an emptied line refused;
  - `parse_batch`: the duplicate rule at exactly `SAME_EVENT_SECONDS`, an open
    end matching only an open end, **the same range with another name added**,
    a block containing a line twice adding it once, `leftover` holding exactly
    the refused lines.
- **Core — `tests/slates.rs`:** `retype_slate` keeps `id`, `title` and the
  stored position, can reopen a range and move a video; `add_slate` and
  `mark_slate_in` build the same record; `o` after a pasted open range closes
  it (the rule, pinned so a later change to it is deliberate).
- **Core — `tests/match_entry.rs`:** unchanged, and must stay green through the
  `pub(crate)` change.
- **App, headless (`slate_editor.rs`):** row `where`/label/open flag, the three
  echo verdicts' wording, the summary and button.
- **UI — `tests/ui/`:** `text_editing.rs`'s sheet-field probe
  (`a_letter_typed_in_a_sheet_field_reaches_the_field`, `:258`) and its Esc
  test gain the slate editor's row field and paste box — **that is where a new
  field is proved typeable** (CLAUDE.md); a new `slate_editor.rs` finds the
  **Type…** button by its label **with zero slates** (S8's reason for
  existing) and confirms the match editor still reads its own title and hint
  after the generalization.
- **Harness — `tests/slates.rs`:** an `AddSlates` of N publishes one
  `ProjectChanged` and one `Undo` removes all N; a partly refused block adds
  the rest, emits one `UserError::Slate` and one `SlatePasteLeftover` holding
  the refused lines alone; `EditSlateLine` moves a **shot** slate's range and
  the clip's `slate_id`, anchor and name are unchanged; a retype of a titled
  slate keeps its title; the same line sent twice records one undo step.
- **Manual (batched):** paste a real list of ranges, filter by a pasted tag,
  `g`, and shoot two — the themed pass (#120) over typed slates is the
  workflow this is for; Esc in the row field cancels and in the box leaves it
  first; copy a row from the Slates list, paste it, and see it marked
  *already marked*.

## Risks

1. **A wrong time is silent**, as in the events editor. Go is the check.
2. **The `[`-in-a-tag hole.** A tag containing `[` (the panel allows it) does
   not round-trip through S4. An untouched row is safe (S9); an edited one
   would store the tag differently, shown in the row once it commits. Rare
   enough not to forbid `[` in tags, which would be a new rule on a shipped
   field.
3. **`o` closes a typed open range.** S10 says why that is the rule working;
   a coach who pastes open ranges and then marks by key may still be surprised.
   Q3's answer may remove most of this by keeping open ranges rare.
4. **Generalizing a shipped sheet** risks the match editor. The UI test that it
   still reads its own words, plus the existing `text_editing.rs` probes over
   both, is the guard.
5. **The Slates header now always shows.** A layout change in a column whose
   space was argued over (#87, #123); measured in the plan, not assumed.

## Out of scope

- **Exporting the slates as a film** (BACKLOG #98) — unchanged by this.
- **Copying the slate list out as text**, the events editor's own top deferral
  (editor spec, *Deferred*). `format_line` makes it a button; nobody asked.
- **A hint in the events box** when a line there holds a range. A cheap nicety,
  but the events box today says *"no time (use m:ss)"*, which is true, and a
  cross-reference between the boxes is a coupling to add only if it bites.
- **Loading from a file**, multi-select, a whole session as one undo step —
  out for the events editor's reasons.
- **Typing a title** — Q2.

## Open questions for the coach

**Q1. How are tags written on a line?**
- **A. A bracket group at the end: `corner routine [corners, set piece]`** —
  *recommended.* It can't be confused with a comment, it carries a tag with a
  space in it, and the inside is exactly what the tag field takes.
- B. `#corners #set-piece`, with `#` no longer a comment — what social media
  taught everyone, but a tag can't hold a space and `corner #3` can't be a
  name.
- C. A bar: `corner routine | corners, set piece` — no brackets, but a `|` is
  harder to find on a keyboard than to read.

**Q2. Should a line carry the title burned into exports?**
- **A. No — the title stays the panel's field, and a line edit never touches
  it** — *recommended.* Notes have names and times; a title is composed later,
  and it would need a third delimiter.
- B. Yes, in quotes at the end: `corner routine [corners] "Watch the near post"`.

**Q3. What does a line with only a start time mean?**
- **A. Refused — write `14:05-` to leave the end open** — *recommended.* It is
  what stops a pasted list of goals becoming open slates, and `14:05-` is how
  the Slates list already shows an open range.
- B. An open range, as if `14:05-` had been typed.
- C. A range of a fixed length (say 20 s) — but that invents an end the notes
  never gave.

**Q4. Where do you go to type slates in?**
- **A. A "Type…" button in the Slates header, which then shows even before the
  first slate** — *recommended.* It sits beside the list it fills.
- B. A button beside "Edit events…" in the Match panel.
- C. The events editor takes both: one paste box, where a line with a range
  becomes a slate and a line with one time becomes an event.

**Q5. When you retype a slate that has already been recorded, should its row
say anything?**
- **A. No — it edits like any other; the recorded clip is untouched** —
  *recommended.* It is what Set in / Set out already do to a recorded slate.
- B. A one-line note under the row: "already recorded — the clip keeps the take
  as it was".
- C. Refuse range changes on a recorded slate (name and tags only).

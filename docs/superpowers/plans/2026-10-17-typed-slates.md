# Plan — typing slates in as lines, as match events can be typed (#97)

Spec: `docs/superpowers/specs/2026-10-17-typed-slates-design.md`. Read it and
`CLAUDE.md` first. Neither is repeated here, except where §0 below corrects
the spec. **Every task starts with the `start-task` skill and ends with
`verify`, `sabotage-proof` and `recheck-claims`.**

Every `path:line` here was re-derived on **2026-10-17 at `5a4c707`** (`main`
after #96), which the spec branch is rebased on. **Other branches are in
flight**: #101 (media's decode), #115 (the caption bar) and #93 (the rename
shims) each touch `BACKLOG.md`, and #115 may touch `core/src/caption.rs` and
the export path. None is known to touch `app.slint`'s line editor, the Slates
section or `main.rs`'s editor wiring, but `start-task` §3 re-derives §A before
each task all the same. **This plan's `BACKLOG.md` edits are #97's entry and
its "Next, in order" line, and nothing else.** Paths are from the repository
root; `core`, `app` and `harness` mean `crates/pundit-<name>/`. **§A holds
every citation**; the tasks cite only what §A does not.

## Where this stands (update it as tasks land)

- **1 — `core::line_batch`, and match events onto it.** Not started.
- **2 — `core::slate_entry`, `add_slate` and `retype_slate`.** Not started.
- **3 — the bus: `EditSlateLine`, `AddSlates`, `PasteLeftover`.** Not started.
- **4 — one `LineEditorSheet` with a kind, events only.** Not started.
- **5 — the slates kind: the door, the wiring, Go.** Not started.
- **6 — prose and the backlog.** Not started.

**Shipping: one PR, a commit per task, in order.** Tasks 1 and 4 are
refactors that change no behaviour — the match editor before and after reads
and writes exactly what it did — and on their own they would be reviewed
without the reason for them, which is tasks 2, 3 and 5. A separate refactor PR
would also leave `main` with a `LineEditorKind` that has one live value. Each
commit is verifiable on its own: its own tests, and every existing one, pass at
its own head.

---

## §0. Where the spec is out of date or wrong (checked 2026-10-17)

1. **Go is a Slint function, not `main.rs`'s `park_on_slate`.** Spec S17 has
   the sheet's Go call `park_on_slate` (`app/src/main.rs:4249`–`:4256`), which
   **no test can reach** — the UI tests link the library, and `main.rs` is the
   binary (CLAUDE.md). Its body is two window calls, so it moves into
   `app.slint` as `public function park-on-slate(id: string)` (set
   `selected-slate`, then `jump-to-slate(id)`), and `main.rs`'s two callers
   (`advance_pass` `:4211`, `start_pass` `:4229`) call
   `w.invoke_park_on_slate(…)`. One copy, and the spec's "Go selects before it
   jumps" becomes a UI test.
2. **Go and × branch on the kind in Slint, not in Rust.** Spec S7 makes every
   sheet callback branch in Rust. Go and × already have window callbacks for
   both kinds — `seek-match-event` / `delete-match-event` (the Match panel's,
   `app.slint:5053`–`:5054`), `park-on-slate` (above) / `delete-slate`
   (`:4990`) — so branching in the instance's binding costs no new callback,
   where branching in Rust would add two that only forward. The rest
   (select, reseed, commit, add, `valid-line`, `check-paste`) need the project
   and stay in Rust, branching on `get_line_editor_kind()`.
3. **The match cap moves into `admit` as a count over `accepted`.**
   `parse_batch` counts start/stops in a mutable local (`core/src/match_entry.rs:429`,
   `:450`–`:452`); `read_batch`'s `admit` is `Fn(&T, &[T])`, so the count is
   `project.start_stop_count()` (`core/src/scoreboard.rs:841`) plus the
   start/stops in `accepted`. The same number; the cap test pins it.
4. **`text_editing.rs`'s doc names the sheet** (*"the match-event editor's row
   field and paste box"*, `app/tests/ui/text_editing.rs:249`–`:251`) and grows
   a slates clause.
5. **`app/tests/ui/main.rs`'s `mod` list gains `slate_editor`** (between
   `sheet_scroll` `:85` and `slate_fields` `:86`), which the spec does not
   name.
6. **The spec's Risk 5 "measured in the plan"** is manual check 3: the clip
   list's visible rows at the 700 px minimum window, with no slates, before and
   after.
7. **No CHANGELOG edit.** It is curated at release.

---

## §A. The census, 2026-10-17 at `5a4c707`

Re-derive at each task (`start-task` §4).

| Changing | Sites |
|---|---|
| core modules (tasks 1, 2) | `core/src/lib.rs`: `pub mod line_batch;` after `layout` (`:19`), `pub mod slate_entry;` after `signals` (`:30`) |
| The batch loop (task 1) | `core/src/match_entry.rs`: `PendingMatchEvent` `:37`, `LineVerdict` `:46`, `BatchVerdict` `:378` (`AlreadyTagged` `:382`), `BatchLine` `:388`, `Batch` `:399` (`events` `:401`), `parse_batch` `:424`–`:469` (the loop `:437`–`:466`, the cap counter `:429`/`:450`–`:452`), `already_tagged` `:475`–`:490`; `is_video_number` `:299` and `refuse_video` `:368` → `pub(crate)` (task 2) |
| Batch readers (task 1) | `core/tests/match_entry.rs`: `added` `:373` (`batch.events`), `:410` and `:437` (`AlreadyTagged`), `:413` (`batch.events`); its four batch tests `:378`, `:404`, `:429`, `:443` change **only** those names. `app/src/match_panel.rs:9` (the import), `PasteLineText` `:123`, `PasteEcho` `:131`, `paste_echo` `:143`–`:191` (`AlreadyTagged` `:156`, `"already tagged"` `:160`/`:178`), `plural` `:194`, `reads_as` `:203`, `pending_label` `:215`, the test `the_echo_reads_back_every_verdict` `:687`–`:750`. `app/src/main.rs:2046`–`:2066` (`on_check_paste`; `batch.events` `:2064`). `app/src/bus/scoreboard.rs:31` (import), `:151`–`:161` (`add_match_events`: `batch.events` `:152`, the leftover `:158`, `batch_notice` `:159`), `batch_notice` `:245`–`:274` (`AlreadyTagged` `:254`) |
| `Project` (task 2) | `core/src/project.rs`: `Slate` `:509`–`:531`, `mark_slate_in` `:866`–`:878` (its literal `:868`, the only `Slate {` outside tests), `edit_slate` `:916`, `slates_sorted` `:943`. Test literals of `Slate` (`core/tests/caption.rs`, `project_format.rs`, `tags.rs`, `sources.rs`, `undo.rs`, `media/tests/export.rs`) are **unchanged**: no field changes |
| The bus (task 3) | `app/src/bus/mod.rs`: `Command` `EditMatchEvent`/`AddMatchEvents` `:192`–`:210` (the shape), `EditSlate` `:255`; `Event::MatchPasteLeftover` `:584`–`:591` → `PasteLeftover { kind, text }` (doc ref at `:203`); dispatch `:1185`–`:1189`, `:1207`–`:1208`; the recording allow-lists `:1128`–`:1129`, `:1149`–`:1150` (**unchanged**: deny by default). `app/src/bus/slates.rs`: `edit_slate` `:288`, `edit_slates` `:309`–`:322`. `bus/scoreboard.rs:158` (the emit) |
| Leftover readers (task 3) | `app/src/main.rs:3752` (the `Event` arm); `harness/tests/match_events.rs:263` |
| `app.slint` (task 4) | `MatchEditorRow` `:160`–`:172`, `PasteLine` `:176`, `PasteEcho` `:184`–`:193` (doc `"7 events to add"` `:186`); `MatchEditorSheet` `:3357`–`:3655` — its event words `:3441`, `:3444`, `:3498`, `:3510`, `:3552`, `:3574`, `:3578` and `valid-line`'s doc `:3398`–`:3400`; the window's `match-editor-*` `:4816`–`:4838`; callbacks `:5061`–`:5079`; `close-match-editor` `:5187`–`:5198`; `handle-key`'s editor branch `:5422`–`:5448`; the Match panel's `edit-events` `:7122`–`:7125`; the `Scrim` `:7873`–`:7921` |
| `main.rs` (task 4) | `wire_match_editor` call `:875`, `wire_match_editor` `:1977`–`:2069`, `editor_verdict` `:2074`, `seed_match_line` `:2087`–`:2105`, `open_match_editor` `:2266`–`:2295`, `show_match_editor` `:2307`–`:2330`, the project-open clear `:3545`–`:3548`, the notice arm `:3755`–`:3767`, the rebuild gate `:3981`–`:3985` |
| The Slates section (task 5) | `app.slint:5964`–`:6254` (the outer `if` `:5983`, the header `HorizontalLayout` `:5986`–`:6032`, the empty text `:6038`, the list `:6045`); `delete-slate` `:4990`, `jump-to-slate` `:4993`; `sources` `:4336`. `main.rs`: `park_on_slate` `:4249`–`:4256`, its callers `:4211`, `:4229` |
| tests/ui (tasks 4, 5) | `main.rs:76`–`:92` (the `mod` list); `sheet_scroll.rs:23` (import), `:90`–`:97` (`card: "MatchEditorSheet"`), `:416`–`:425` (`editor_rows`); `text_editing.rs:34` (import), `:249`–`:251` (doc), `:272`–`:296` (the two ids `"MatchEditorSheet::line-edit"` `:284`, `"MatchEditorSheet::paste-edit"` `:292`), `esc_in_a_sheet_field_leaves_the_field_before_it_closes_the_sheet` `:380` |
| Refusal and notice texts | **Every existing one is kept verbatim**, and these are asserted: `"already tagged"` (`match_panel.rs:712`, `:735`; `harness/tests/match_events.rs:273`, `"2 already tagged"`), `"no time (use m:ss)"` (`core/tests/match_entry.rs:395`), the cap's `"already tagged"` (`:460`). New texts live in `slate_entry.rs` and `line_editor.rs` |
| Exhaustive matches | `Bus::command`'s dispatch (`bus/mod.rs`, two new arms); `main.rs`'s `Event` match (`:3752`, one arm renamed). `BatchVerdict`'s matches in `paste_echo` and `batch_notice` (renamed variant). `UndoAction` **unchanged**; no `KeyAction` |
| Prose (task 6) | `CLAUDE.md:1034`–`:1058` (the match-events grammar paragraph and its bullets, including the rebuild bullet the spec's commit rewrote), `:1133` (the themed pass); `docs/book/src/guide/slates.md` (a section after *Marking a range*, `:6`–`:22`); `docs/book/src/guide/match.md:36`ff. (*Typing events in*, unchanged but cross-referenced); `docs/hands-on-checklist.md` (§16 after §15 `:209`) |
| BACKLOG (task 6) | #97: the "Next, in order" line `BACKLOG.md:28`–`:30` and the entry `:2382`–`:2398`, as of this plan's commit. **Nothing else** |

**No site** in `pundit-media`, `store.rs`, `undo.rs`, `keymap.rs` or
`packaging`; no `Slate` field; no `state.json` key; no format bump.

---

## 1. `core::line_batch`, and match events onto it

### Files

- `core/src/line_batch.rs` (new), `core/src/lib.rs`, `core/src/match_entry.rs`
- `core/tests/line_batch.rs` (new), `core/tests/match_entry.rs`
- `app/src/line_editor.rs` (new), `app/src/lib.rs`, `app/src/match_panel.rs`,
  `app/src/main.rs`, `app/src/bus/scoreboard.rs`

### Steps

1. **`line_batch.rs`**: `BatchVerdict<T> { Added(T), Duplicate(T),
   Refused(String) }`, `BatchLine<T>`, `Batch<T> { items, lines, leftover }`,
   `Admit { Add, Duplicate, Refuse(String) }`, and `read_batch(text, parse,
   admit)` — `match_entry.rs:437`–`:467` (the loop and the leftover join) moved whole, with `parse` returning
   `Option<Result<T, String>>` (`None` for nothing) and `admit` judging
   against `&batch.items`. Its doc carries `parse_batch`'s and `Batch::leftover`'s
   reasoning (`:405`–`:423`), which is about the container.
2. **`match_entry::parse_batch` becomes a call**, keeping its signature and
   returning `Batch<PendingMatchEvent>`. `parse` maps `parse_line`'s three
   verdicts; `admit` is `already_tagged` (unchanged) then the cap, counted as
   §0.3 says. `match_entry`'s own `BatchVerdict`, `BatchLine` and `Batch` go;
   `match_entry` re-exports nothing.
3. **`line_editor.rs`** (app, `pub mod` after `keymap` in `app/src/lib.rs:14`):
   `PasteLineText`, `PasteEcho` and `paste_echo` move from `match_panel.rs`,
   generic: `paste_echo<T>(batch, noun, duplicate, reads_as)`. `plural` moves
   with it. `match_panel.rs` keeps `reads_as` and `pending_label` for events,
   `pub(crate)`, and its echo test moves with `paste_echo`, its expected
   strings untouched.
4. **`batch_notice(lines, duplicate)`** in `bus/scoreboard.rs`, `pub(super)`
   for task 3; the events call passes `"already tagged"`.
5. **The renames at every reader in §A's "Batch readers" row**, and nothing
   else at them.

### Tests

**core** (`cargo test -p pundit-core --test line_batch --test match_entry`):

- 1.1 `every_line_is_numbered_and_blanks_say_nothing` (new): a stub grammar (one
  integer a line, `#` a comment, anything else refused) over
  `"# c\n1\n\n2\nx\n"` gives lines numbered 2, 4, 5 and no echo for 1 and 3.
- 1.2 `a_duplicate_is_judged_against_what_the_block_accepted` (new): with an
  `admit` that calls equal integers duplicates, `"1\n1\n"` adds one and
  echoes the second as `Duplicate`.
- 1.3 `the_leftover_is_the_refused_lines_alone` (new): `"1\nx\n1\ny\n"` leaves
  `"x\ny"` — not the duplicate.
- 1.4 the four existing batch tests (`match_entry.rs:378`, `:404`, `:429`,
  `:443`), renamed only.

**app** (`cargo test -p pundit-app --lib line_editor`):

- 1.5 `the_echo_reads_back_every_verdict` (moved from `match_panel.rs:688`),
  every expected string as it was.

**harness** (`cargo test -p pundit-harness --test match_events`):

- 1.6 `a_pasted_batch_is_one_undo_step` (`:185`) and
  `a_partly_refused_batch_adds_the_rest` (`:237`, its `"2 already tagged"`),
  unchanged.

### Sabotage

1. `read_batch` numbers only the lines it echoes → 1.1,
   `a_pasted_block_echoes_every_line_in_input_order` (its `number == 2`) and
   1.5 (its `line 4` / `line 6`).
2. `admit` is handed an empty slice instead of `&batch.items` → 1.2,
   `the_same_line_twice_in_one_block_adds_once` and the cap test (`:443`),
   whose fourth start/stop then lands.
3. The leftover keeps duplicates too → 1.3,
   `a_line_already_tagged_is_skipped_within_a_second_either_side` and
   `the_same_line_twice_in_one_block_adds_once` (both assert `""`).
4. `paste_echo` ignores its `duplicate` argument and writes `"already
   marked"` → 1.5 only.

---

## 2. `core::slate_entry`, `add_slate` and `retype_slate`

### Files

- `core/src/slate_entry.rs` (new), `core/src/lib.rs`,
  `core/src/match_entry.rs` (two `pub(crate)`), `core/src/project.rs`
- `core/tests/slate_entry.rs` (new), `core/tests/slates.rs`

### Steps

1. **`PendingSlate`**, and **`parse_line(project, default_source, line) ->
   Option<Result<PendingSlate, String>>`** in S2's order: comment and blank →
   `None`; the video number (`is_video_number`, `refuse_video`); the range by
   S2's dash rule, written as a small scanner over `char`s (the in time is the
   run of digits, `:` and `.`; then the dash and its spacing decide closed,
   open or lopsided); the tags as the last `[` to the end of the line; the
   name as the trimmed text between. Every S3 sentence is a `const` beside the
   function, so the tests assert the same bytes.
2. **`parse_line_with(…, keep: (Option<f64>, Option<Option<f64>>))`**, the one
   body: the per-end override is applied **before** the order and the bound
   (spec S3, S9), as `match_entry::parse_line_with` does (`:333`–`:353`).
3. **`format_line(&Slate)`**: S9's shapes, `[]` when there are no tags and the
   name contains `[`.
4. **`edit_from_line(project, &Slate, typed) -> Result<PendingSlate, String>`**:
   byte-identical to `format_line(slate)` → the slate's own fields; else the
   two time strings of seed and typed compared **split on the dash and
   trimmed**, each equal one kept as stored; `None` → the emptied-row
   sentence.
5. **`parse_batch(project, default_source, text) -> Batch<PendingSlate>`** over
   `read_batch`, `admit` the canonical-text key (spec S12) against the stored
   slates and `accepted`.
6. **`Project::add_slate(PendingSlate) -> Uuid`** and
   **`retype_slate(id, PendingSlate) -> bool`** (`#[must_use]`), beside
   `mark_slate_in`, which becomes `self.add_slate(PendingSlate { …, out_seconds:
   None, name: String::new(), tags: Vec::new() })`.

### Tests

**core** (`cargo test -p pundit-core --test slate_entry --test slates`), in
`slate_entry.rs` unless named:

- 2.1 `every_shape_of_a_line_reads` — spec Testing's S2 list, table-driven.
- 2.2 `every_refusal_says_how_to_fix_it` — each S3 sentence by its `const`,
  including `2 14:05 -14:40 x`.
- 2.3 `a_lone_time_is_refused_and_a_block_of_events_adds_no_slate`.
- 2.4 `kick_off_words_are_a_name_here`.
- 2.5 `each_end_is_bounded_and_the_range_must_run_forwards` (equality refused).
- 2.6 `a_line_reads_back_as_the_slate_it_was_written_from` — closed, open, no
  tags, an open range named `5:30 drill`.
- 2.7 `a_name_with_a_bracket_round_trips` — `a [b`, `a [b] c`, `[x]`, no tags.
- 2.8 `an_untouched_row_changes_nothing_even_with_a_bracket_in_a_tag`.
- 2.9 `an_unchanged_end_keeps_its_stored_seconds` — stored in `845.06`, out
  moved, typed both `14:05.0-14:45.0` and `14:05.0 - 14:45.0`; and a changed
  video number under unchanged times.
- 2.10 `the_order_is_checked_on_the_seconds_that_will_be_stored` — stored in
  `845.06`, out typed `14:05.0` (845.0): refused.
- 2.11 `an_emptied_row_is_refused_naming_its_line`.
- 2.12 `a_pasted_slate_equal_to_a_stored_one_is_a_duplicate` — and a block
  holding a line twice adds it once.
- 2.13 `the_same_range_with_another_name_is_a_new_slate` — and a range a tenth
  off is too.
- 2.14 `slates.rs::a_retype_keeps_the_id_the_title_and_the_place` — can reopen
  a range and move a video; unknown id → `false`.
- 2.15 `slates.rs::o_closes_a_typed_open_range` — the rule, pinned.

### Sabotage

1. `format_line` writes `[]` only when the name ends in `]` → 2.7's `a [b` and
   `a [b] c` cases only.
2. The time identity compares whitespace tokens → 2.9's spaced case only.
3. The order check reads the parsed in, not the effective one → 2.10 only.
4. The byte-identical early return removed → 2.8 only.
5. A lone time read as an open range → 2.3 only.
6. The lopsided dash accepted as closed → 2.2 only.
7. The duplicate key drops the name → 2.13 only.
8. `retype_slate` removes and pushes → 2.14 only (the place).

---

## 3. The bus: `EditSlateLine`, `AddSlates`, `PasteLeftover`

### Files

- `app/src/bus/mod.rs`, `app/src/bus/slates.rs`, `app/src/bus/scoreboard.rs`
- `app/src/main.rs` (`:3752` only — the renamed arm, writing the match editor's
  paste as today)
- `harness/tests/slates.rs`, `harness/tests/match_events.rs`

### Steps

1. **`Command::EditSlateLine { id, line }` and `Command::AddSlates { text,
   default_source }`**, docs in `EditMatchEvent`'s words; dispatch arms beside
   `EditSlate`.
2. **`Bus::edit_slate_line`**: find the slate (absent → `eprintln!`, as
   `edit_match_event`), `slate_entry::edit_from_line`, a refusal →
   `UserError::Slate`, else `edit_slates(|p| retype_slate)`.
3. **`Bus::add_slates`**: `slate_entry::parse_batch`, one `edit_slates`
   pushing every item with `add_slate`, `Event::PasteLeftover { kind:
   PasteKind::Slates, text }`, `batch_notice(&lines, "already marked")` as a
   `UserError::Slate`.
4. **`Event::PasteLeftover { kind: PasteKind, text }`** replaces
   `MatchPasteLeftover`; `PasteKind { MatchEvents, Slates }` beside `Event`.
   `add_match_events` emits `MatchEvents`.

### Tests

**harness** (`cargo test -p pundit-harness --test slates --test match_events`):

- 3.1 `a_pasted_block_of_slates_is_one_undo_step` — three lines, one
  `ProjectChanged`, one `Undo` leaves none.
- 3.2 `a_partly_refused_block_of_slates_adds_the_rest` — one `UserError::Slate`
  naming the count, one `PasteLeftover { Slates, … }` holding the refused line
  alone, the saved project matching.
- 3.3 `retyping_a_shot_slate_leaves_its_clip_alone` — shoot a slate
  (`CaptureKind::Test`, as `:272`), retype its range and name: the clip's
  `slate_id`, `start_source_seconds` and name are as they were.
- 3.4 `retyping_a_titled_slate_keeps_its_title`.
- 3.5 `the_same_line_twice_is_one_undo_step`.
- 3.6 `match_events.rs::a_partly_refused_batch_adds_the_rest` (`:237`), its
  reader `:263` now matching `PasteLeftover { kind: PasteKind::MatchEvents, .. }`.

### Sabotage

1. `add_slates` calls `edit_slates` once per item → 3.1 only.
2. `add_slates` emits `kind: MatchEvents` → 3.2 only.
3. `edit_slate_line` writes through `SlateEdit`s and a `Title(String::new())`
   → 3.4 only.
4. `edit_slate_line` parses with `parse_line` instead of `edit_from_line` →
   3.5 (the re-rounded seconds make the second send an edit) — and 3.3 stays
   green, which is why 3.5 exists.

---

## 4. One `LineEditorSheet` with a kind, events only

### Files

- `app/ui/app.slint`, `app/src/main.rs`
- `app/tests/ui/text_editing.rs`, `app/tests/ui/sheet_scroll.rs`,
  `app/tests/ui/slate_editor.rs` (new), `app/tests/ui/main.rs`

### Steps

1. **Rename** `MatchEditorSheet` → `LineEditorSheet`, `MatchEditorRow` →
   `LineEditorRow`; add `export enum LineEditorKind { events, slates }`.
2. **The sheet's words** become `in property`s — `title`, `noun`,
   `empty-text`, `go-tooltip`, `placeholder`, `hint` — per spec S7's table;
   the × tooltip and the *"Paste …s"* heading are built from `noun`;
   `valid-line`'s doc reworded.
3. **The window's properties** `match-editor-*` → `line-editor-*`, plus
   `line-editor-kind`, `events-paste`, `slates-paste`. The instance sets every
   word from `line-editor-kind` (both kinds' words written now; only events is
   reachable until task 5).
4. **One open, one close**: `open-line-editor(LineEditorKind)` (the Match
   panel's `edit-events` passes `events`); `close-line-editor()` copies
   `line-editor-paste` back into the kind's string, then does today's three
   clears.
5. **Go and ×** branch on the kind in the instance (§0.2). For events, today's
   `seek-match-event` / `delete-match-event`.
6. **`handle-key`'s branch** reads `line-editor-open` /
   `line-editor-line-focused` / `line-editor-reseed()`.
7. **`main.rs`**: `wire_line_editor` (today's wiring, each callback matching
   on `get_line_editor_kind()` with a `Slates` arm that does nothing yet);
   `open_line_editor(w, kind)` copies the kind's paste in; `show_line_editor`
   behind the same gate (`:3983`); the project-open clear empties both paste
   strings; the notice arm tests the kind's `UserError`.

### Tests

**app UI** (`cargo test -p pundit-app --test ui`):

- 4.1 `text_editing::a_letter_typed_in_a_sheet_field_reaches_the_field` — the
  ids `"LineEditorSheet::line-edit"` / `"::paste-edit"`, events kind.
- 4.2 `sheet_scroll` — the entry renamed (`card: "LineEditorSheet"`), opened
  on the events kind; every assertion as it was.
- 4.3 `slate_editor::the_events_kind_reads_its_own_words` (new): title *"Match
  events"*, heading *"Paste events"*, the Z/X/V empty text — found by label.
- 4.4 `slate_editor::esc_in_an_events_row_reseeds_and_the_second_closes` (new):
  focus the row field, type, Esc → `line-editor-reseed` fired (a recording
  mock) and the sheet open; Esc again → shut.
- 4.5 `slate_editor::each_box_keeps_its_own_block` (new): type in the events
  box, close, set kind slates and open, the box is empty; back to events, the
  block is there.

### Sabotage

1. The instance binds `title` to the slates words unconditionally → 4.3 only.
2. `handle-key`'s Esc closes from inside the row field (the reseed branch
   removed) → 4.4 only.
3. `close-line-editor` does not copy the box back → 4.5 only.
4. The paste box's id renamed → 4.1 only (4.2 finds the card, not the box).

`main.rs`'s kind arms, `open_line_editor` and the notice arm are unreachable
from a test (CLAUDE.md); manual checks 1 and 6 cover them.

---

## 5. The slates kind: the door, the wiring, Go

### Files

- `app/ui/app.slint`, `app/src/main.rs`, `app/src/line_editor.rs`
- `app/tests/ui/slate_editor.rs`, `app/tests/ui/text_editing.rs`,
  `app/tests/ui/sheet_scroll.rs`

### Steps

1. **The Slates section restructured** per spec S8: the outer `if`
   `root.sources.length > 0`; the header's filter field and ✕ under an inner
   `if` on today's condition; **Type…** after it (`keys.focus()` first, then
   `open-line-editor(LineEditorKind.slates)`; enabled on the Match panel's
   `can-edit` expression); the empty text and the list as their own `if`s.
2. **`park-on-slate`** as a `public function` (§0.1); `main.rs`'s two callers
   invoke it and `park_on_slate` goes.
3. **Go and × for slates**: `park-on-slate(id)` then close; `delete-slate(id)`.
4. **`line_editor.rs`**: `slate_rows(project) -> Vec<LineEditorRowText>`
   (`slates_sorted`, `where` as `<n> · <in>–<out>`, label the name or
   "Untitled", `role_less` for an open range) and `slate_reads_as`.
5. **`main.rs`'s `Slates` arms**: select seeds `slate_entry::format_line`;
   `valid-line` is `edit_from_line(…).is_ok()`; commit sends `EditSlateLine`;
   add sends `AddSlates`; `check-paste` is `slate_entry::parse_batch` through
   `paste_echo(…, "slate", "already marked", slate_reads_as)`; the leftover
   arm writes the slates string, or the box when that kind is open.

### Tests

**app** (`cargo test -p pundit-app --lib line_editor`):

- 5.1 `the_slate_echo_reads_back_every_verdict` — the spec's three lines, the
  summary *"1 slate to add · 1 already marked · 1 line refused"*, the button.
- 5.2 `an_open_range_is_a_role_less_row`.

**app UI** (`cargo test -p pundit-app --test ui`):

- 5.3 `slate_editor::type_is_there_with_no_slates` — a source, no slates:
  **Type…** found by label, and pressing it opens the sheet on the slates
  kind. With no source it is absent.
- 5.4 `slate_editor::the_slates_kind_reads_its_own_words`.
- 5.5 `slate_editor::go_on_a_slate_selects_it_before_it_jumps` — a mock
  `jump-to-slate` records `selected-slate` at the call; it is the row's id,
  and the sheet is shut.
- 5.6 `slate_editor::esc_in_a_slate_row_reseeds_and_the_second_closes`.
- 5.7 `text_editing` probe on the slates kind; `sheet_scroll` gains a slates
  entry.

### Sabotage

1. **Type…** inside the inner `if` → 5.3 only.
2. `park-on-slate` jumps before it selects → 5.5 only.
3. The slates Go calls `jump-to-slate` without selecting → 5.5 only.
4. `slate_rows`' `role_less` reads `out_seconds.is_some()` → 5.2 only.
5. The slates words' `hint` bound to the events hint → 5.4 only.

---

## 6. Prose and the backlog

### Files

- `CLAUDE.md`, `docs/book/src/guide/slates.md`, `docs/hands-on-checklist.md`,
  `BACKLOG.md` (#97 only)

### Steps

1. **`CLAUDE.md`**: the match-events paragraph (`:1034`ff.) names the sheet
   `LineEditorSheet`, its two kinds and `core::line_batch`; a bullet for the
   slate grammar (the bracket tags, `#` only at a line's start, the dash's
   spacing, the lone time refused, the canonical-text duplicate); the rebuild
   bullet covers both kinds.
2. **The guide's `slates.md`**: *Typing ranges in*, after *Marking a range*:
   **Type…**, one example line, `14:05-` for an open end, tags in brackets,
   and that a retyped slate keeps its title and its recorded clip.
3. **The checklist**: §16, manual checks 1–7 below.
4. **BACKLOG #97**: the entry gains *SHIPPED* with the PR as `#TODO`; the
   "Next, in order" line goes at merge.

No tests; `recheck-claims` over every line.

---

## Manual checks (batched at the end)

1. Paste a real list of a match's ranges, with tags; filter by one; `G`, then
   `R`, talk, stop — the pass walks the typed slates.
2. Go on a row, then `R`: the take is that row's range, and the scrubber's band
   is on it.
3. **The column cost** (§0.6): at the 700 px minimum window, a project with no
   slates — count the visible clip rows before and after. Record it in *Where
   this stands*.
4. Esc in a slate row puts the line back; Esc in the box leaves it; the next
   closes. Reopen: the box still holds its block, and the events box its own.
5. Paste a block written in a word processor (autocorrected dashes).
6. A refused Add's sentence on the sheet's line, for each kind.
7. Retype a titled slate open: its title stops showing in a preview; close it
   again and it is back.

## Risks

1. **Task 1 is a refactor of a shipped parser.** The match editor's four batch
   tests, its echo test and its two harness tests are unchanged apart from
   renames; a change to any expected string is a regression, not a fix.
2. **Task 4 is a rename across `app.slint`, `main.rs` and two UI tests.** §A
   lists every site; a grep for `match-editor`, `match_editor` and
   `MatchEditor` must come back empty at task 4's head.
3. **The dash scanner is the one piece of new parsing with edge cases** (S2).
   Tests 2.1, 2.2 and 2.6 cover each spacing; anything else is a refusal by
   construction.
4. **`main.rs`'s kind arms are untested** (§0, task 4). They are thin — each
   is one core call and one window write — and manual checks 1, 4 and 6 run
   both kinds through them.
5. **A branch in flight may move §A's lines** (#101, #115, #93). `start-task`
   re-derives before each task.

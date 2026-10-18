# Plan — typing slates in as lines, as match events can be typed (#97)

Spec: `docs/superpowers/specs/2026-10-17-typed-slates-design.md`. Read it and
`CLAUDE.md` first. Neither is repeated here, except where §0 below adds to the
spec. **Every task starts with the `start-task` skill and ends with `verify`,
`sabotage-proof` and `recheck-claims`.**

Every `path:line` here was re-derived on **2026-10-17 at `5a4c707`** (`main`
after #96), which the spec branch is rebased on. Paths are from the repository
root; `core`, `app` and `harness` mean `crates/pundit-<name>/`. **§A holds
every citation**; the tasks cite only what §A does not.

**Other branches are in flight, and three of them touch files this plan does**
— so `start-task` §3 re-derives §A before each task, and whichever lands second
rebases:

- **#93** (`claude/93-shims`, PR #97 open): `app/src/main.rs` (four lines out,
  at `:391`, so every `main.rs` line below that moves up by four),
  `CLAUDE.md`, `docs/hands-on-checklist.md`.
- **#115**: `docs/hands-on-checklist.md`.
- **#101**: `CLAUDE.md` (near `:787`).

**This plan's `BACKLOG.md` edits are #97's entry and its "Next, in order" line,
and nothing else.**

## Where this stands (update it as tasks land)

- **1 — `core::line_batch`, and match events onto it.** Done. §A re-derived
  at `484f70f` (`main` after #93, #115 and this plan): every core and app
  citation in task 1's rows held except `main.rs`, four lines up after #93
  (`on_check_paste` `:2040`–`:2062`). `paste_echo` takes its `reads_as` as
  `impl Fn(&T) -> String`, so `match_panel::reads_as` is passed by name;
  `main.rs` imports `pundit_app::line_editor`. Sabotage 1–4 as listed (1 also
  fails `a_pasted_block_echoes_every_line_in_input_order`, as predicted).
- **2 — `core::slate_entry`, `add_slate` and `retype_slate`.** Done. Task
  2's citations held at `c93a9d0` (`project.rs` `:866`/`:868`/`:916`/`:943`;
  `match_entry.rs`' two helpers one line down, after task 1's import).
  **2.10 types its out as `14:05.05`, not `14:05.0`**: with the in re-rounded
  to 845.0, an out of 845.0 is refused by `<=` either way, so the plan's line
  could not fail under sabotage 4. **Sabotage 2 also fails 2.10** — that test
  needs the in kept too, so it rightly fails with the identity broken. The
  other eight sabotages failed exactly the test listed.
- **3 — the bus: `EditSlateLine`, `AddSlates`, `SlatePasteLeftover`.** Done.
  §A's bus row held at `e5b4bbe` to the line; `main.rs`'s leftover arm is
  `:3750` (four up, #93). `Event::SlatePasteLeftover(_) => {}` is the arm task
  5 fills. The five sabotages failed exactly the test listed; 3's
  (`SlateEdit`s and an empty `Title`) trips 3.3 at its video assertion first,
  so a second mutant — `retype_slate`, then `Title(String::new())` — was run
  and fails 3.3 alone, on the title.
- **4 — `MatchEditorSheet` → `LineEditorSheet`, a rename.** Done. §A's
  task-4 rows held at `de7cad0` except the sheet itself, which starts at
  `app.slint:3367` (ten lines below the census) and `main.rs`, four up
  (`wire_match_editor` `:1974`, `open_match_editor` `:2264`,
  `show_match_editor` `:2305`, the rebuild gate `:3984`). The grep for
  `match-editor-`, `match_editor_` and `MatchEditor` is empty at this head.
  Sabotage 1 fails 4.3 alone.
- **5 — the slates kind: the words, the dispatch, the door, Go.** Not started.
- **6 — prose and the backlog.** Not started.

**Shipping: one PR, a commit per task, in order.** Tasks 1 and 4 are refactors
that change no behaviour — the match editor before and after each reads and
writes exactly what it did — and on their own they would be reviewed without
the reason for them, which is tasks 2, 3 and 5. **Task 4 carries no slate
scaffolding**: it is the rename alone, with the sheet's words left as the
literals they are; the kind, the word properties and the second paste string
arrive in task 5 with the code that uses them. Each commit is verifiable on its
own: its own tests, and every existing one, pass at its own head.

---

## §0. What the spec does not say (checked 2026-10-17)

The plan's review changed five of the spec's decisions, and **the spec is
amended in place for each**, marked *"Amended 2026-10-17 after the plan's
review"*: S7 (`title` is `Sheet`'s; one dispatch rule, in the instance;
`open-line-editor` a Slint function), S9 (the time identity reads the
scanner's tokens), S12 (one `judge` closure; `SlatePasteLeftover` beside the
existing variant; the echo test stays in `match_panel.rs`), S17 (`park-on-slate`
in Slint) and Testing (kind-blind code tested once). What is left here is what
the spec does not say.

1. **Task 5's header gate breaks three UI fixtures** that set slates and no
   sources: the list and the filter field will not exist without a source.
   `tests/ui/slate_menu.rs`'s `window()` (`:43`–`:58`, its four tests),
   `slate_fields.rs`'s `a_titled_slate_shows_its_marker_and_an_untitled_one_none`
   (`:294`, its own window, slates at `:297`), and `text_editing.rs`'s
   `window()` (`:131`–`:168`), whose `window_fields` probes
   `"AppWindow::slate-filter-edit"` (`:105`). Each gains `set_sources(…)` with
   one row. **`fit_window.rs:191` and `tag_field.rs:63` need nothing**: they
   reach a slate *field*, and the fields sit outside the gated section
   (`app.slint:6256`, after its close at `:6254`).
2. **What `main.rs` calls must be `pub`**, because `main.rs` is the binary and
   reaches the library as `pundit_app::…`: `line_editor::paste_echo`,
   `match_panel::reads_as`, `line_editor::slate_rows` and
   `line_editor::slate_reads_as`.
3. **`refuse_video` returns `String`**, not a `LineVerdict`, once it is shared:
   `match_entry` wraps it (`LineVerdict::Refused(refuse_video(n))`) and
   `slate_entry` returns it as its `Err`.
4. **The Slates section's comment is rewritten** (`app.slint:5964`–`:5982`): it
   argues the old one-`if` gate, which task 5 replaces. And the empty text's
   condition is written out —
   `root.slates.length == 0 && root.slate-tag-filter != ""` — because today it
   leans on the outer `if` to imply the filter (`:6036`–`:6037` says so).
5. **`text_editing.rs`'s doc names the sheet** (*"the match-event editor's row
   field and paste box"*, `:249`–`:251`); task 4 renames the words.
6. **`tests/ui/main.rs`'s `mod` list gains `slate_editor`** (between
   `sheet_scroll` `:85` and `slate_fields` `:86`).
7. **Risk 5's "measured in the plan"** is manual check 3.
8. **No CHANGELOG edit.** It is curated at release.

---

## §A. The census, 2026-10-17 at `5a4c707`

| Changing | Sites |
|---|---|
| core modules (tasks 1, 2) | `core/src/lib.rs`: `pub mod line_batch;` after `layout` (`:19`), `pub mod slate_entry;` after `signals` (`:30`) |
| The batch loop (task 1) | `core/src/match_entry.rs`: `PendingMatchEvent` `:38`, `LineVerdict` `:46`, `BatchVerdict` `:378` (`AlreadyTagged` `:382`), `BatchLine` `:388`, `Batch` `:399` (`events` `:401`), `parse_batch` `:424`–`:469` (the loop `:437`–`:466`, the cap counter `:429`/`:450`–`:452`), `already_tagged` `:475`–`:490`; `is_video_number` `:299` and `refuse_video` `:368` → `pub(crate)` (task 2) |
| Batch readers (task 1) | `core/tests/match_entry.rs`: the import `:3`–`:6` (`BatchVerdict` from `line_batch`), `added` `:373` (its `Batch` → `line_batch::Batch<PendingMatchEvent>`, `batch.events`), `:410` and `:437` (`AlreadyTagged`), `:413` (`batch.events`); its four batch tests `:378`, `:404`, `:429`, `:443` change **only** those names. `app/src/match_panel.rs`: the import `:9`, `PasteLineText` `:123`, `PasteEcho` `:131`, `paste_echo` `:143`–`:191` (`AlreadyTagged` `:156`, `"already tagged"` `:160`/`:178`), `plural` `:194`, `reads_as` `:203` (→ `pub`), `pending_label` `:215`; its test `the_echo_reads_back_every_verdict` `:687`–`:750` **stays**, calling `line_editor::paste_echo`. `app/src/main.rs:2046`–`:2066` (`on_check_paste`; `batch.events` `:2064`). `app/src/bus/scoreboard.rs`: the import `:31`, `add_match_events` `:151`–`:161` (`batch.events` `:152`, `batch_notice` `:159`), `batch_notice` `:245`–`:274` (`AlreadyTagged` `:254`) |
| `Project` (task 2) | `core/src/project.rs`: `Slate` `:509`–`:531`, `mark_slate_in` `:866`–`:878` (its literal `:868`, the only `Slate {` outside tests), `edit_slate` `:916`, `slates_sorted` `:943`. Test literals of `Slate` (`core/tests/caption.rs`, `project_format.rs`, `tags.rs`, `sources.rs`, `undo.rs`; `media/tests/export.rs`) are **unchanged**: no field changes |
| The bus (task 3) | `app/src/bus/mod.rs`: `Command` `EditMatchEvent`/`AddMatchEvents` `:192`–`:210` (the shape), `EditSlate` `:255`; `Event::MatchPasteLeftover` `:584`–`:591` (**unchanged**; `SlatePasteLeftover` after it); dispatch `:1207`–`:1208`; the recording allow-lists `:1128`–`:1129`, `:1149`–`:1150` (**unchanged**: deny by default). `app/src/bus/slates.rs`: `edit_slate` `:288`, `edit_slates` `:309`–`:322` |
| `app.slint` (task 4) | `MatchEditorRow` `:160`–`:172`; `MatchEditorSheet` `:3357`–`:3655`; the window's `match-editor-*` `:4816`–`:4838`; `close-match-editor` `:5187`–`:5198`; `handle-key`'s editor branch `:5422`–`:5448`; the `Scrim` `:7873`–`:7921`. The window's callbacks (`open-match-editor`, `select-match-event`, `reseed-match-line`, `edit-match-event`, `add-match-events`, `valid-match-line`, `check-paste`, `:5061`–`:5079`) **keep their names** |
| `main.rs` (task 4) | `wire_match_editor` `:1977`–`:2069`, `seed_match_line` `:2087`–`:2105`, `open_match_editor` `:2266`–`:2295`, `show_match_editor` `:2307`–`:2330`, the project-open clear `:3545`–`:3548`, the leftover arm `:3752`, the notice arm `:3755`–`:3767`, the rebuild gate `:3981`–`:3985`. The comment at `:2029` names `MatchPasteLeftover` and stays true |
| `app.slint` (task 5) | `Sheet`'s `title` `:1992` (bound, not redeclared); the sheet's event words `:3441`, `:3444`, `:3498`, `:3510`, `:3552`, `:3574`, `:3578` and `valid-line`'s doc `:3398`–`:3400`; the Match panel's `edit-events` `:7122`–`:7125`; the Slates section `:5964`–`:6254` (comment `:5964`–`:5982`, outer `if` `:5983`, header `:5986`–`:6032`, empty text `:6038`, list `:6045`); `sources` `:4336`, `delete-slate` `:4990`, `jump-to-slate` `:4993`, `seek-match-event`/`delete-match-event` `:5053`–`:5054` |
| `main.rs` (task 5) | `park_on_slate` `:4249`–`:4256` (deleted), its callers `advance_pass` `:4211` and `start_pass` `:4229` |
| tests/ui | `main.rs:76`–`:92` (the `mod` list); `sheet_scroll.rs:23` (import), `:89`–`:97` (`card: "MatchEditorSheet"`), `:416`–`:425` (`editor_rows`); `text_editing.rs:34` (import), `:105` (the filter probe), `:131`–`:168` (`window()`), `:249`–`:251` (doc), `:272`–`:296` (ids `"MatchEditorSheet::line-edit"` `:284`, `"::paste-edit"` `:292`), `:380` (the basket's Esc test, the shape); `slate_menu.rs:43`–`:58`; `slate_fields.rs:294`–`:297` |
| Refusal and notice texts | **Every existing one is kept verbatim**, and these are asserted: `"already tagged"` (`match_panel.rs:712`, `:735`; `harness/tests/match_events.rs:273`, `"2 already tagged"`), `"no time (use m:ss)"` (`core/tests/match_entry.rs:395`), the cap's `"already tagged"` (`:460`). The slate sentences are new and spelled out in test 2.2 |
| Exhaustive matches | `Bus::command`'s dispatch (two arms); `main.rs`'s `Event` match (one arm, `SlatePasteLeftover`). `BatchVerdict`'s matches in `paste_echo` and `batch_notice` (renamed variant). `UndoAction` **unchanged**; no `KeyAction` |
| Prose (task 6) | `CLAUDE.md:1034`–`:1058` (the match-events grammar paragraph and its bullets, the rebuild bullet among them), `:1133` (the themed pass); `docs/book/src/guide/slates.md` (a section after *Marking a range*, `:6`–`:22`); `docs/hands-on-checklist.md` (§16 after §15, `:209`) |
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
   and `read_batch(text, judge: impl FnMut(&str, &[T]) ->
   Option<BatchVerdict<T>>)`. The loop is `match_entry.rs:437`–`:467` with its
   body replaced by the call: numbering, `None` skipped, `Added` pushed onto
   `items`, `Refused` into the leftover. Its doc carries `Batch::leftover`'s
   reasoning (`:405`–`:415`), which is about the container.
2. **`match_entry::parse_batch` becomes a call**, keeping its signature and
   returning `line_batch::Batch<PendingMatchEvent>`. `judge` is today's loop
   body as it stands — `parse_line`, `already_tagged` against `accepted`, the
   cap with `start_stops` captured `mut`. `match_entry`'s own batch types go.
3. **`line_editor.rs`** (app, `pub mod` after `keymap`, `app/src/lib.rs:14`):
   `PasteLineText`, `PasteEcho`, `plural` and `pub fn paste_echo<T>(batch,
   noun, duplicate, reads_as)` move from `match_panel.rs`. `match_panel::reads_as`
   becomes `pub`; the echo test stays in `match_panel.rs`, its expected strings
   untouched.
4. **`batch_notice(lines, duplicate)`**, `pub(super)` for task 3; the events
   call passes `"already tagged"`.
5. **The renames at every reader in §A's "Batch readers" row**, and nothing
   else at them.

### Tests

**core** (`cargo test -p pundit-core --test line_batch --test match_entry`):

- 1.1 `every_line_is_numbered_and_blanks_say_nothing` (new): a stub judge (an
  integer is added, `#…` and blank are `None`, anything else refused) over
  `"# c\n1\n\n2\nx\n"` numbers its lines 2, 4 and 5.
- 1.2 the four existing batch tests (`match_entry.rs:378`, `:404`, `:429`,
  `:443`), renamed only: the duplicate, the leftover and the cap are theirs.

**app** (`cargo test -p pundit-app --lib match_panel`):

- 1.3 `the_echo_reads_back_every_verdict` (`match_panel.rs:688`), every
  expected string as it was.

**harness** (`cargo test -p pundit-harness --test match_events`):

- 1.4 `a_pasted_batch_is_one_undo_step` (`:185`) and
  `a_partly_refused_batch_adds_the_rest` (`:237`, its `"2 already tagged"`),
  unchanged.

### Sabotage

1. `read_batch` numbers only the lines it echoes → 1.1,
   `a_pasted_block_echoes_every_line_in_input_order` (`number == 2`) and 1.3
   (`line 4` / `line 6`).
2. `read_batch` hands `judge` an empty slice → `the_same_line_twice_in_one_block_adds_once`
   only (the cap keeps its own counter).
3. The leftover keeps duplicates → `a_line_already_tagged_is_skipped_within_a_second_either_side`
   and `the_same_line_twice_in_one_block_adds_once` (both assert `""`).
4. `paste_echo` ignores `duplicate` and writes `"already marked"` → 1.3 only.

---

## 2. `core::slate_entry`, `add_slate` and `retype_slate`

### Files

- `core/src/slate_entry.rs` (new), `core/src/lib.rs`,
  `core/src/match_entry.rs` (§0.3), `core/src/project.rs`
- `core/tests/slate_entry.rs` (new), `core/tests/slates.rs`

### Steps

1. **`PendingSlate`** and **`parse_line(project, default_source, line) ->
   Option<Result<PendingSlate, String>>`**, in S2's order: comment and blank →
   `None`; the video number; the range, by a small scanner over `char`s (the in
   time is the run of digits, `:` and `.`; then the dash and its spacing decide
   closed, open or lopsided), which **returns the two time tokens as it found
   them**; the tags as the last `[` to the end; the name as the trimmed text
   between. The refusal sentences are written inline, as `match_entry`'s are;
   only the video refusal is shared (§0.3).
2. **One body, `parse_line_with(…, keep: (Option<f64>, Option<f64>))`**: an
   end is overridden only when the parse produced a time for it, and the
   override is applied **before** the order and the bound (spec S3, S9), as
   `match_entry::parse_line_with` does (`:333`–`:353`).
3. **`format_line(&Slate)`**: S9's shapes, `[]` when there are no tags and the
   name contains `[`.
4. **`edit_from_line(project, &Slate, typed) -> Result<PendingSlate, String>`**:
   byte-identical to `format_line(slate)` → the slate's own fields; else the
   scanner's in and out tokens of the seed and the typed line compared, each
   equal one kept as stored; `None` → the emptied-row sentence.
5. **`parse_batch(project, default_source, text) -> Batch<PendingSlate>`** over
   `read_batch`, the judge keying on canonical text (spec S12) against the
   stored slates and `accepted`.
6. **`Project::add_slate(PendingSlate) -> Uuid`** and
   **`retype_slate(id, PendingSlate) -> bool`** (`#[must_use]`), beside
   `mark_slate_in`, which becomes a call to `add_slate`.

### Tests

**core** (`cargo test -p pundit-core --test slate_entry --test slates`), in
`slate_entry.rs` unless named:

- 2.1 `every_shape_of_a_line_reads` — spec Testing's S2 list, table-driven.
  **No lopsided dash in it**: that is 2.2's.
- 2.2 `every_refusal_says_how_to_fix_it` — each S3 sentence spelled out in full
  in the test, including `2 14:05 corner` and `2 14:05 -14:40 x`.
- 2.3 `a_lone_time_is_refused_and_a_block_of_events_adds_no_slate`.
- 2.4 `kick_off_words_are_a_name_here`.
- 2.5 `each_end_is_bounded_and_the_range_must_run_forwards` (equality refused).
- 2.6 `a_line_reads_back_as_the_slate_it_was_written_from` — closed, open, no
  tags, an open range named `5:30 drill`.
- 2.7 `a_name_with_a_bracket_round_trips` — `a [b`, `a [b] c`, `[x]`, no tags.
- 2.8 `an_untouched_row_changes_nothing_even_with_a_bracket_in_a_tag`.
- 2.9 `an_unchanged_end_keeps_its_stored_seconds` — stored in `845.06`, out
  moved, typed both `14:05.0-14:45.0` and `14:05.0 - 14:45.0`; stored out
  `880.04` kept when only a name `corner-kick` is edited; a changed video
  number under unchanged times keeps both.
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

1. `format_line` writes `[]` only when the name ends in `]` → 2.7 (`a [b`,
   `a [b] c`) only.
2. The identity compares whitespace tokens → 2.9 (the spaced case) only.
3. The identity splits the line on its first dash → 2.9 (`corner-kick`) only.
4. The order check reads the parsed in, not the effective one → 2.10 only.
5. The byte-identical early return removed → 2.8 only.
6. A lone time read as an open range → 2.2 (`2 14:05 corner`) and 2.3.
7. The lopsided dash accepted as closed → 2.2 only.
8. The duplicate key drops the name → 2.13 only.
9. `retype_slate` removes and pushes → 2.14 only (the place).

---

## 3. The bus: `EditSlateLine`, `AddSlates`, `SlatePasteLeftover`

### Files

- `app/src/bus/mod.rs`, `app/src/bus/slates.rs`
- `app/src/main.rs`: one arm, `Event::SlatePasteLeftover(_) => {}`, because
  its `Event` match is exhaustive (`:3752`–`:3779`, no wildcard) and there is
  no slates box to write to until task 5, which fills the arm
- `harness/tests/slates.rs`

### Steps

1. **`Command::EditSlateLine { id, line }` and `Command::AddSlates { text,
   default_source }`**, docs in `EditMatchEvent`'s words; dispatch arms beside
   `EditSlate`.
2. **`Bus::edit_slate_line`**: find the slate (absent → `eprintln!`, as
   `edit_match_event`), `slate_entry::edit_from_line`, a refusal →
   `UserError::Slate`, else `edit_slates(|p| retype_slate)`.
3. **`Bus::add_slates`**: `slate_entry::parse_batch`, one `edit_slates`
   pushing every item with `add_slate`, `Event::SlatePasteLeftover(leftover)`,
   `batch_notice(&lines, "already marked")` as a `UserError::Slate`.
4. **`Event::SlatePasteLeftover(String)`** after `MatchPasteLeftover`.

### Tests

**harness** (`cargo test -p pundit-harness --test slates`):

- 3.1 `a_pasted_block_of_slates_is_one_undo_step` — three lines, one
  `ProjectChanged`, one `Undo` leaves none.
- 3.2 `a_partly_refused_block_of_slates_adds_the_rest` — one `UserError::Slate`
  naming the count, one `SlatePasteLeftover` holding the refused line alone,
  the saved project matching.
- 3.3 `retyping_a_shot_titled_slate_leaves_its_clip_and_its_title_alone` —
  title a slate, shoot it (`CaptureKind::Test`, as `:272`), retype its range
  and name: the clip's `slate_id`, `start_source_seconds` and name are as they
  were, and the slate's title survives.
- 3.4 `a_name_only_retype_keeps_a_captured_in_point` — `mark_in(0, 0.53)`, an
  `EditSlateLine` changing only the name: `in_seconds == 0.53`.
- 3.5 `the_same_line_twice_is_one_undo_step`.

### Sabotage

1. `add_slates` calls `edit_slates` once per item → 3.1 only.
2. `add_slates` emits `MatchPasteLeftover` → 3.2 only.
3. `edit_slate_line` writes `SlateEdit`s and a `Title(String::new())` → 3.3
   only.
4. `edit_slate_line` parses with `parse_line` instead of `edit_from_line` →
   3.4 only (0.53 re-rounds to 0.5).
5. `edit_slates`' no-op return removed → 3.5 only (the existing
   `each_edit_is_one_undo_step`, `:214`, sends only edits that change
   something).

---

## 4. `MatchEditorSheet` → `LineEditorSheet`, a rename

### Files

- `app/ui/app.slint`, `app/src/main.rs`
- `app/tests/ui/text_editing.rs`, `app/tests/ui/sheet_scroll.rs`,
  `app/tests/ui/slate_editor.rs` (new), `app/tests/ui/main.rs`

### Steps

1. **Rename** `MatchEditorSheet` → `LineEditorSheet`, `MatchEditorRow` →
   `LineEditorRow`, the window's `match-editor-*` → `line-editor-*`,
   `close-match-editor` → `close-line-editor`. **Nothing else**: the words stay
   literals, the window's callbacks keep their names, and no kind exists yet.
2. **`main.rs`**: the generated setters and getters renamed at every §A
   `main.rs` (task 4) site. `wire_match_editor` keeps its name and shape.
3. **The tests' ids, imports and setters** renamed (§A tests/ui), and
   `text_editing.rs`'s doc (§0.5).

### Tests

**app UI** (`cargo test -p pundit-app --test ui`):

- 4.1 `text_editing::a_letter_typed_in_a_sheet_field_reaches_the_field` — ids
  `"LineEditorSheet::line-edit"` / `"LineEditorSheet::paste-edit"`.
- 4.2 `sheet_scroll` — the entry's `card: "LineEditorSheet"`; every assertion
  as it was.
- 4.3 `slate_editor::esc_in_the_row_field_reseeds_and_the_second_closes`
  (new): focus the row field, type, Esc → `reseed-match-line` fired (a
  recording mock) and the sheet open; Esc again → shut. Tested **once**:
  `handle-key`'s branch does not know the kind (spec Testing).

### Sabotage

1. `handle-key`'s branch loses its reseed arm, so the first Esc in the row
   field falls to the cascade → 4.3 only.

4.1 and 4.2 are renames of shipped tests and have no sabotage of their own:
the rename is what they pin, and a missed one fails to compile.

---

## 5. The slates kind: the words, the dispatch, the door, Go

### Files

- `app/ui/app.slint`, `app/src/main.rs`, `app/src/line_editor.rs`
- `app/tests/ui/slate_editor.rs`, `slate_menu.rs`, `slate_fields.rs`,
  `text_editing.rs` (§0.1)

### Steps

1. **`export enum LineEditorKind { events, slates }`**; the window's
   `line-editor-kind`, `events-paste`, `slates-paste`.
2. **The sheet's words** become `in property`s — `noun`, `empty-text`,
   `go-tooltip`, `placeholder`, `hint` — and `Sheet`'s own `title` is bound
   (spec S7's table). The instance sets each from the kind.
3. **`public function open-line-editor(kind)`** (spec S7): sets the kind,
   copies the kind's paste string into `line-editor-paste`, calls
   `open-match-editor()` or `open-slate-editor()` (Rust: rows, sources, picker,
   selection — `open_match_editor` minus its `set_..._open(true)`), then sets
   `line-editor-open`. `close-line-editor()` copies the box back to the kind's
   string first. The Match panel's `edit-events` calls
   `open-line-editor(LineEditorKind.events)`.
4. **The instance dispatches every callback on the kind** (spec S7): `select`,
   `commit`, `add`, `valid-line`, `check-paste`, and the `handle-key` branch's
   reseed, to the match editor's callbacks or to new `select-slate-line`,
   `reseed-slate-line`, `edit-slate-line`, `add-slates`, `valid-slate-line`,
   `check-slate-paste`; Go to `seek-match-event` or `park-on-slate`; × to
   `delete-match-event` or `delete-slate`.
5. **`public function park-on-slate(id)`**: `selected-slate = id`, then
   `jump-to-slate(id)`. `main.rs`'s `park_on_slate` is deleted and its two
   callers call `w.invoke_park_on_slate(…)`.
6. **`main.rs`'s `wire_slate_editor`**, beside `wire_match_editor` and the same
   shape: select seeds `slate_entry::format_line`; `valid-slate-line` is
   `edit_from_line(…).is_ok()`; commit sends `EditSlateLine`; add sends
   `AddSlates`; `check-slate-paste` is `slate_entry::parse_batch` through
   `paste_echo(…, "slate", "already marked", line_editor::slate_reads_as)`;
   `show_slate_editor` from `line_editor::slate_rows`. The leftover arms write
   the box when their kind is open and their kind's string otherwise; the
   notice arm and the rebuild gate test the open kind. The project-open clear
   empties both strings.
7. **`line_editor.rs`**: `pub fn slate_rows(project)` (`slates_sorted`, `where`
   as `<n> · <in>–<out>`, label the name or "Untitled", `role_less` for an open
   range) and `pub fn slate_reads_as`.
8. **The Slates section** per spec S8 and §0.4: the outer `if`
   `root.sources.length > 0`; the filter and its ✕ under an inner `if` on
   today's condition; **Type…** after it (`keys.focus()`, then
   `open-line-editor(LineEditorKind.slates)`; enabled on the Match panel's
   `can-edit` expression, `app.slint:7098`); the empty text and the list as
   their own `if`s, the empty text's condition written out; the comment
   rewritten.
9. **The three fixtures of §0.1** gain one source row.

### Tests

**app** (`cargo test -p pundit-app --lib line_editor`):

- 5.1 `the_slate_echo_reads_back_every_verdict` — the spec's three lines, the
  summary *"1 slate to add · 1 already marked · 1 line refused"*, the button.
- 5.2 `an_open_range_is_a_role_less_row`.

**app UI** (`cargo test -p pundit-app --test ui`):

- 5.3 `slate_editor::type_is_there_with_no_slates` — `set_can_play(true)`, one
  source, no slates: **Type…** found by label, the *"No slates tagged"* line
  absent; pressing it opens the sheet on the slates kind. With no source,
  **Type…** is absent.
- 5.4 `slate_editor::each_kind_reads_its_own_words` — *"Match events"* and
  *"Paste events"* on one, *"Slates"* and *"Paste slates"* on the other, each
  found by label.
- 5.5 `slate_editor::go_on_a_slate_selects_it_before_it_jumps` — a mock
  `jump-to-slate` records `selected-slate` at the call; it is the row's id, and
  the sheet is shut.
- 5.6 `slate_editor::each_box_keeps_its_own_block` — type into the events box,
  close, `invoke_open_line_editor(Slates)`: the box is empty; reopen events:
  the block is there.
- 5.7 the §0.1 fixtures' tests, all passing — they are the guard on the gate.

### Sabotage

1. **Type…** inside the inner `if` → 5.3 only.
2. The empty text keeps the bare `slates.length == 0` → 5.3 only.
3. `park-on-slate` jumps before it selects → 5.5 only.
4. `open-line-editor` does not copy the kind's string in → 5.6 only.
5. The instance binds `hint` and `title` to the events words → 5.4 only.
6. `slate_rows`' `role_less` reads `out_seconds.is_some()` → 5.2 only.
7. The outer `if` left at today's condition → 5.3 only; the §0.1 fixtures
   stay green, which is why 5.3 has a no-slates case.

`main.rs`'s `wire_slate_editor`, the leftover and notice arms and the rebuild
gate are unreachable from a test (CLAUDE.md); they are one core call and one
window write each, and manual checks 1, 4 and 6 run them.

---

## 6. Prose and the backlog

### Files

- `CLAUDE.md`, `docs/book/src/guide/slates.md`, `docs/hands-on-checklist.md`,
  `BACKLOG.md` (#97 only)

### Steps

1. **`CLAUDE.md`**: the match-events paragraph (`:1034`ff.) names
   `LineEditorSheet`, its two kinds and `core::line_batch`; **one sentence** for
   the slates — they are typed through `core::slate_entry`, whose `#` is a
   comment only at a line's start so that the bracket tags survive, and its
   module doc holds the rest; the rebuild bullet covers both kinds.
2. **The guide's `slates.md`**: *Typing ranges in*, after *Marking a range*:
   **Type…**, one example line, `14:05-` for an open end, tags in brackets,
   and that a retyped slate keeps its title and its recorded clip.
3. **The checklist**: §16, manual checks 1–6 below.
4. **BACKLOG #97**: the entry gains *SHIPPED* with the PR as `#TODO`; the
   "Next, in order" line goes at merge.

No tests; `recheck-claims` over every line.

---

## Manual checks (batched at the end)

1. Paste a real list of a match's ranges, with tags; filter by one; `G`, then
   `R`, talk, stop — the pass walks the typed slates.
2. Go on a row, then `R`: the take is that row's range, and the scrubber's band
   is on it.
3. **The column cost** (§0.7): at the 700 px minimum window, a project with no
   slates — count the visible clip rows before and after. Record it in *Where
   this stands*.
4. Esc in a slate row puts the line back; Esc in the box leaves it; the next
   closes. Reopen: the box still holds its block, and the events box its own.
5. Paste a block written in a word processor (autocorrected dashes).
6. A refused Add's sentence on the sheet's line, for each kind.

## Risks

1. **Task 1 is a refactor of a shipped parser.** The four batch tests, the
   echo test and the two harness tests are unchanged apart from renames; a
   change to any expected string is a regression, not a fix.
2. **Task 4 is a rename across `app.slint`, `main.rs` and two UI tests.** A
   grep for `match-editor-`, `match_editor_` and `MatchEditor` must come back
   empty at task 4's head (the callbacks' `-match-` names stay, by design).
3. **Task 5's gate moves three UI fixtures** (§0.1). They are listed; a slate
   test outside them that touches the list will fail loudly, not silently.
4. **The dash scanner is the one piece of new parsing with edge cases** (S2).
   2.1, 2.2, 2.6 and 2.9 cover each spacing and a dash in a name.
5. **`main.rs`'s slate wiring is untested** (task 5). Thin by design — the
   dispatch is in the instance, where 5.4–5.6 reach it.
6. **A branch in flight may move §A's lines** (#93's `main.rs`, #93/#101's
   `CLAUDE.md`, #93/#115's checklist). `start-task` re-derives before each
   task.

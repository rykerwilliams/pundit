# Plan — titles over the footage, and a switchable caption bar (#133, #115)

Spec: `docs/superpowers/specs/2026-10-12-titles-and-caption-bar-design.md`. Read
it and `CLAUDE.md` first; neither is repeated here, except where §0 below
corrects the spec. **Every task starts with the `start-task` skill and ends with
`verify`, `sabotage-proof` and `recheck-claims`.**

Every `path:line` here was re-derived on **2026-10-13 at `4436b5d`**
(`claude/133-115-spec`, the spec's own commit on top of `main` at `c8eca27`).
Nothing in `crates/` differs from `main` there, but `main` moves daily —
**re-derive every line at `start-task` §3**, and `app.slint` and `main.rs` first.
Paths are from the repository root; `core`, `media`, `app` and `harness` mean
`crates/pundit-<name>/`. **§A holds every citation**; the tasks cite only what
§A does not.

## Where this stands (update it as tasks land)

- **1 — #106 first: one `clip_stub()` for `pundit-core`'s tests.** Done
  2026-10-13 on `claude/106-clip-stub`: 548 core tests pass before and after,
  core's full `Clip` literals 14 → 3. The sabotage (stub `show_pip: false`)
  fails `clip_order.rs`'s `set_sets_one_field_and_returns_the_old_value`, the
  one core test that reads `show_pip` — not `plan.rs` or `export.rs`, which
  never read it (PiP is media's).
- **2 — the v18 format and the strip rule, in core.** Done 2026-10-13 on
  `claude/133-115-titles`: core's tests 548 → 564, the workspace green. The
  census matched §A exactly — `Clip` 8 (3 core, 4 media, 1 harness), `Slate`
  7, `PlanEntry` 4. All 20 sabotages went red where predicted but one: #15
  (`Default`'s `last_caption_bar: Off`) also fails 2.13 and 2.16, which the
  list missed — the container default is what fills a v17 file's missing key,
  and `Project::new` seeds from it. Deviations: `sample_project` gains **no**
  slate (two v12 tests read `slates[0]` and the whole sorted list as their
  own); 2.13 and 2.14 push a `titled_slate` instead. 2.10 also asserts a
  basket's entries take each piece's own mode (spec Testing 4).
- **3 — drawing the strip, in export and preview.** Done 2026-10-13 on
  `claude/133-115-titles`: six new tests (media lib 128 → 129, `tests/export`
  33 → 36, `tests/preview` 5 → 6, harness `tests/export` 19 → 20), the
  workspace green but for `recorder.rs`'s wall-clock bounds under other
  sessions' load (untouched code; green alone 2 runs in 3). The census matched
  §A, `overlay.rs`'s test literals one line lower. All 7 sabotages went red
  exactly where predicted. Deviations: `draw_label`'s premultiply put
  `coverage²/255` in the alpha channel, and now puts the alpha itself — every
  label's anti-aliased edge, measured by no test; preview's **strokes** keep
  `n/30` and only the strip reads `entry.record_time(n)` (the same number in a
  one-entry run); 3.4 gives its entry a real line under `Off`, so `Off` is
  what keeps the caption away; the three-clip test's glyph count became the
  shared `bar_glyphs`.
- **4 — the controls, the wiring, the guide and the prose.** Done 2026-10-13
  on `claude/133-115-titles`: three new tests (`tests/ui` slate_fields 4 → 6,
  harness `tests/clips` 11 → 12) and the title field joined to
  `text_editing.rs`'s list. The census matched §A — `SlateRow` 7, the
  `SlateField` match, the one `_ => false` arm — at drifted lines. Deviations:
  4.1 leaves the field with **Esc**, not a click, because Slint raises the new
  item's focus before lowering the old one's, so a click into another slate
  field would test that ordering instead of this commit; the copy sentence
  had a third copy the census missed, `exporting.md`'s *"warns that a copy
  cannot carry highlights or drawings"*, now naming titles; BACKLOG #133/#115
  move to "Closed, kept for the reasoning" with no PR number yet.

**Shipping: two PRs, in order.**

1. **Task 1 alone, first.** It is a test-only refactor with no behaviour change.
   BACKLOG #106's revisit trigger is *"at the **start** of the next format bump,
   before its fields are added"*, and its reason is a **readable format diff**:
   inside task 2 it would bury the format change under twelve mechanical edits.
2. **Tasks 2, 3 and 4 together, one PR with a commit each, in that order.** The
   format bump, the drawing and the controls ship as one. So `main` never holds
   a v18 that nothing can set, nor a control that draws nothing, and no release
   can be cut in between. Each commit stays independently verifiable: its own
   tests pass at its own head.

---

## §0. Where the spec is out of date or wrong (checked 2026-10-13)

1. **`SlateRow` literals: 7, not "two fixtures"** (spec Risk 6) — one in
   `main.rs` and six `tests/ui` fixtures.
2. **The copy sentence is asserted verbatim** by `tests/ui/export_sheet.rs`'s
   `COPY_LINE` (three users) and quoted in `docs/hands-on-checklist.md`. The spec
   names neither; both change with the sentence.
3. **BACKLOG #106's trigger fires, and the spec's Risk 6 misses it.** After task
   1, the `Clip` literals task 2 edits fall from **19 to 8**: 3 in core
   (`add_recorded_clip`, the stub and `project_format.rs`'s `sample_clip`, which
   keeps its full literal on purpose), 4 in media and 1 in harness.
4. **`Slate` literals: 7, not 8** — the spec counted `fn slate_on(…) -> Slate {`
   as a literal.
5. **The picker's "First 3 seconds" is a second spelling of `CAPTION_SECONDS`**
   (spec §U5: *"nothing else on screen has to"* state it). Accepted: each side
   carries a comment naming the other, and manual check 4 reads the label.
   Pushing a model from Rust for one word costs more than the drift it prevents.
6. **Spec Testing 10–12's fixtures are too short or too bright.**
   - The media test `Source` fixture is **5 s**, so 3.2, 3.3 and 3.5 build their
     own counter source of at least 16 s.
   - The whiteboard test's `PITCH_GREEN` diagram has a luma of about 139, under
     the detector's 150 — too close. 3.4 uses a **black** diagram.
7. **Spec Testing 13 — the harness undo of a title — is dropped.** `EditSlates`
   snapshots whole `Slate`s, so a title rides the slate undo that
   `each_edit_is_one_undo_step` already pins. Adding a step there is not a
   one-line change, and the bus path for `SlateEdit::Title` is exercised by 3.6
   anyway.
8. **Spec Testing 15 — "runs to EOS" — cannot see what it claims to test.** A
   job missing its slates still runs to EOS. 3.6 decodes the export and counts
   title glyphs instead. A new `MatchMedia::of` is the one function both bus
   fills call, so that one test covers both (§3 step 5).
9. **The harness's default fixture is wrong for a glyph count.**
   `write_project`'s SMPTE bars put a white block inside the bar's rows. 3.6 uses
   a black `solid_video` through `write_one_source_project`.
10. **Two functions named `one_line` would follow from spec §B.** The shared
    flattener is `naming::one_line`, and `chapters.rs`'s wrapper is renamed
    `chapter_title`.
11. **Stale prose the spec does not list:**
    - preview's *"simply `n/30`"* module doc, and `Composite::text`'s doc;
    - `tests/ui/text_editing.rs`'s counts — *sixteen fields*, *thirteen sites*,
      *the slate panel's three* — which the title field makes wrong.
12. **`Strip` derives `Debug, Clone, Copy, PartialEq`** (the spec shows none);
    the core tests compare it with `assert_eq!`.
13. **No CHANGELOG edit.** `[Unreleased]` is empty although features have landed
    since 0.11.0: the changelog is curated at release.

---

## §A. The census, 2026-10-13 at `4436b5d`

Re-derive at each task (`start-task` §4).

| Changing | Sites |
|---|---|
| `Clip` full literals (tasks 1, 2) | **19**, each naming `backdrop: None`. **Core:** `src/project.rs:795` and 13 test files — `highlight.rs:278`, `tags.rs:30`, `project_format.rs:42`, `export.rs:42`, `audio.rs:46`, `undo.rs:36`, `clip_order.rs:28`, `scoreboard.rs:729`, `timeline.rs:30`, `cues.rs:118` (inside a test, `:97`–`:119`), `plan.rs:32`, `stroke_replay.rs:29`, `sources.rs:58`. **Media:** `src/overlay.rs:1133`, `src/composite/export.rs:1404`, `tests/export.rs:128`, `tests/preview.rs:76`. **Harness:** `src/lib.rs:629`. **App:** none. Every other `Clip {` in the tests spreads off one of these. |
| What those 12 core helpers share (task 1) | All 12 set `show_pip: true`, `inset: Camera`, `inset_size: Medium`, `inset_corner: BottomRight`, `notes`, `transcript` empty, `slate_id: None`, `backdrop: None`; 11 set `sort_index: 0` (`plan.rs` takes it as a parameter). They **differ** in `id` (`new_v4` in 7, `nil` in 3, `from_u128` in 1, a parameter in `undo.rs`), `name`, `created_at` (four values) and `recording_filename` (five spellings). |
| `Slate` literals (task 2) | 7: `core/src/project.rs:829`; `core/tests/tags.rs:161`, `project_format.rs:411`, `:766`, `:790`, `undo.rs:75`, `sources.rs:169` |
| `PlanEntry` literals (task 2) | 4: `core/src/reel.rs:202`, `core/src/whole_match.rs:33`, `core/src/plan.rs:213`, `core/tests/audio.rs:147` |
| `SlateRow` literals (task 4) | 7: `app/src/main.rs:3876`; `app/tests/ui/text_editing.rs:107`, `slate_menu.rs:28`, `fit_window.rs:191`, `panel_widths.rs:63`, `slate_fields.rs:25`, `tag_field.rs:63` |
| `OverlayFrame` literals (task 3) | 7: `media/src/composite/export.rs:737`, `media/src/composite/preview.rs:915`; `overlay.rs`'s tests at `:1188` (inside `render_at`, `:1179`), `:1566`, `:1707`, `:1856`, `:2047` |
| `MatchMedia` (task 3) | full literals at `app/src/bus/export.rs:1295` and `app/src/bus/basket.rs:435`, both replaced by `MatchMedia::of`. The three in `media/tests/export.rs` (`:147`, `:1024`, `:1080`) spread `..MatchMedia::default()` and need no edit. |
| `PreviewJob` (task 3) | `app/src/bus/preview.rs:109`, `media/tests/preview.rs:136`. **Not** a `MatchMedia`: it carries `scoreboard`, `highlights` and `avatar` as fields of its own (`media/src/composite/preview.rs:130`–`:160`), so `of` does not fit it and it gains `slates` beside them. |
| Exhaustive matches | Task 2: `Clip::set` on `ClipEdit` (`core/src/project.rs:372`–`:386`); `Slate::would_invert` (`:513`–`:517`) and `Project::edit_slate` (`:880`–`:885`) on `SlateEdit`. Task 4: `main.rs`'s `SlateField` match (`app/src/main.rs:1465`–`:1468`). **No new `UndoAction`**, so `purge_for_source_change` (`core/src/undo.rs:176`–`:190`) is untouched. |
| Non-exhaustive `ClipEdit` readers | `app/src/bus/clips.rs:58`–`:65` (`_ => false`; task 4 adds an arm); `bus/transcribe.rs:449`, `:458` (one variant; untouched) |
| Refusal texts | **none change** (spec §M1): `SourceReferenced` (`core/src/project.rs:248`), `app/src/bus/mod.rs:633`–`:636`, `app/ui/app.slint:5594` |
| `.length` readers | none: a title adds no row |
| Prose asserting old behaviour | `core/src/project.rs:478`–`:482` (`Slate`'s *"never exported"*, task 2); `core/src/chapters.rs:47`–`:54` (`one_line`'s doc, task 2); `media/src/overlay.rs:124`–`:127` (`OverlayFrame::text`), `:143`–`:145` (`fitted`), `:158`–`:160` (`TextSlot`) (task 3); `media/src/composite/preview.rs:20` and `:680`–`:681` (task 3); `app/ui/app.slint:2107` with `app/tests/ui/export_sheet.rs:47` (users `:277`, `:300`, `:371`) and `docs/hands-on-checklist.md:185` (task 4); `app/tests/ui/text_editing.rs:19`–`:20` (*"sixteen fields … thirteen … sites"*), `:63` (*"the slate panel's three"*), `:182` (*"which of the sixteen"*) (task 4) |
| CLAUDE.md | task 1: one sentence in the format list. Task 2: a v18 bullet after the v17 one, before `:495`. Task 4: `:759` (the bar rule), `:984`–`:987` (a basket's text bar), and a slates bullet after `:1112` (the whiteboard heading is `:1114`). |
| User guide (task 4) | `docs/book/src/guide/slates.md` (a section after "Recording over one", `:22`–`:31`); `exporting.md:17`–`:19` and `:41`–`:42`; `basket.md:19`–`:20` |
| BACKLOG | #106 (`:3133`, trigger `:3153`): RESOLVED in task 1. #133 (`:4450`, index `:30`–`:35`) and #115 (`:3436`, index `:42`–`:48`): SHIPPED in task 4, and they leave "Next, in order"; #115 keeps its `1 / 1` variant as its one open remainder. |
| Fixtures | `media/tests/export.rs:60`–`:65` (`Source`: 5 s of counter video); `media/src/fixtures.rs:388` (`counter_video(path, w, h, fps, frames, kind)`), `:106` (`solid_video(path, w, h, fps, frames, rgb, with_audio)`); `media/tests/export.rs:2418` (`PITCH_GREEN`), `:2456` (the whiteboard export test); `harness/src/lib.rs:427` (`write_project`, SMPTE bars), `:449` (`write_one_source_project`), `:636` (`add_clips`); `core/tests/project_format.rs:46` (`sample_project`, which has no slate) |
| Glyph detector | `media/tests/export.rs:566`–`:575` (luma `> 150` in `bar_rect`'s rows, x < 640) |

**No site** in `packaging`, `keymap.rs`, `Command` or `Event`.

---

## 1. #106: one `clip_stub()` for `pundit-core`'s tests

### Files

- `core/tests/common/mod.rs` (new)
- The 12 core test files of §A's first row, **all but `project_format.rs`**
- `BACKLOG.md` (#106), `CLAUDE.md` (one sentence)

### Steps

1. `common/mod.rs`: `pub fn clip_stub() -> Clip`, which fixes exactly the fields
   all 12 helpers share (§A's second row): `show_pip: true`, the default inset
   placement, empty `notes` and `transcript`, `slate_id` and `backdrop` `None`,
   `sort_index: 0`, `source_index: 0`. Everything else is a placeholder — nil
   `id`, empty `name`, `tags`, `events`, `created_at`, `recording_filename`,
   zero times — that **every helper overrides**. Its doc carries #106's reason,
   and why this is not a `Default for Clip` in production code.
2. Each helper becomes `Clip { <its own fields>, ..common::clip_stub() }`. It
   keeps its own `id`, `name`, `created_at`, `recording_filename`, `tags`,
   `events`, times and anything it takes as a parameter, so **no value a test
   sees changes**. `cues.rs`'s in-test literal takes the same form.
3. `project_format.rs` keeps `sample_clip` whole, with a comment: the format
   test is where naming every field is the point.
4. `BACKLOG.md` #106: RESOLVED. `CLAUDE.md`'s format list gains one sentence: a
   new `Clip` field goes into `clip_stub` and `sample_clip`, and nowhere else in
   core's tests.

### Tests

None new: it is a refactor. The proof is that `cargo test -p pundit-core` passes
the **same count** before and after (both counts go in the PR), and that §A's
core sites fall from 14 to 3.

### Sabotage

Set the stub's `show_pip: false`: tests that read `show_pip` through a helper
must fail — `core/tests/plan.rs` and `export.rs`, whose PiP and audio
expectations assume it. **If nothing fails, some helper still sets its own
`show_pip`**, and the stub is not what it seems; find it. Restore.

---

## 2. The v18 format and the strip rule, in core

### Files

- `core/src/caption.rs` (new), `core/src/lib.rs`, `core/src/{project, undo, plan, reel, whole_match, naming, chapters, store}.rs`
- `core/tests/caption.rs` (new); `core/tests/{common/mod.rs, plan.rs, whole_match.rs, project_format.rs, slates.rs, clip_order.rs}` and the `Slate` / `PlanEntry` literals of §A
- The `Clip` literals in media and harness (one field each)
- `CLAUDE.md`

### Steps

1. **`caption.rs`** (`pub mod caption;` beside `chapters`, `core/src/lib.rs:11`),
   exactly spec §S2:
   - `CaptionBar { Whole (default), FirstSeconds, Off }`, with
     `pub fn until(self) -> f64` — the one place the enum becomes a number;
   - `CAPTION_SECONDS = 3.0` (its doc names the inspector's label, §0.5) and
     `STRIP_FADE = 0.5`;
   - `Strip<'a>` with `#[derive(Debug, Clone, Copy, PartialEq)]` (§0.12),
     `leaving`, and `strip_at`.

   The module doc carries spec §S3–§S5's three rules.
2. **`project.rs`**:
   - `Clip.caption_bar` after `backdrop` (`:364`), with a field-level default;
   - `Preferences.last_caption_bar` after `last_inset_corner` (`:190`), with no
     attribute, and `Whole` in `Default` (`:201`–`:218`);
   - `add_recorded_clip` seeds it beside `inset_corner` (`:809`);
   - `Slate.title` after `tags` (`:496`), with a field-level default, and
     `Slate`'s doc amended (spec §M1);
   - `title: None` in `mark_slate_in`;
   - `SlateEdit::Title(String)` after `Out` (`:537`);
   - `would_invert`'s `false` arm gains `Title(_)`;
   - `edit_slate`: `let line = naming::one_line(&text); slate.title =
     (!line.is_empty()).then_some(line);`.
3. **`undo.rs`**: `ClipEdit::CaptionBar` after `InsetCorner` (`:50`), with the
   v13 doc above it (`:43`–`:49`) reworded to name all three. `Clip::set` gains
   its arm.
4. **`plan.rs`**: `PlanEntry.caption_until` after `text` (`:96`), with spec §S1's
   doc. `clip_entry` (`:209`–`:221`) writes `clip.caption_bar.until()`. The other
   three literals write `f64::INFINITY`.
5. **`naming::one_line`**: `split_whitespace` joined by single spaces, taking the
   newline half of `chapters.rs`'s doc. `chapters.rs`'s wrapper becomes
   `chapter_title` (§0.10), keeping the `UNTITLED` half; its callers are
   `chapters.rs:105`, `:110` and `:113`.
6. **`store.rs:21`**: `17` → `18`.
7. **The literals of §A.** `sample_project` (`project_format.rs:46`) gains a
   slate with `title: None`, so that 2.12 has a key to remove (§A, last row).
8. **CLAUDE.md**, a v18 bullet after v17's (before `:495`):

   > - **v18 adds `Slate::title`, `Clip::caption_bar` and
   >   `Preferences::last_caption_bar`** (#133, #115). `title` is an
   >   `Option<String>` and `caption_bar` an enum defaulting to `Whole`, each
   >   with a field-level `#[serde(default)]`, since `None` and `Whole` are what
   >   an older file means; `last_caption_bar` takes none, as `Preferences`
   >   always does. The floor stays 7. **The bump's reason is the forward
   >   direction:** an older build would drop every title and reset every
   >   caption on its next save.

### Tests

**`pundit-core`** (`cargo test -p pundit-core`).

`tests/caption.rs` (new). Entries are built through `CaptionBar::until()`.
**2.5–2.9 run at record time ≥ 5 s over an `Off` caption**, except 2.5, which
needs a caption to hand back to; that makes them independent of the mapping
that 2.1–2.3 pin.

- 2.1 `whole_shows_the_caption_for_the_whole_entry` — opacity 1 at record time
  0, 2.9 and 600.
- 2.2 `first_seconds_fades_over_its_last_half_second_and_is_gone_at_three` — 1
  at 2.4, ½ at 2.75, `None` at 3.0.
- 2.3 `off_shows_no_caption_from_the_first_frame` — `None` at 0 and at 1.
- 2.4 `an_empty_caption_draws_no_strip_whatever_the_mode` — `Whole` with empty
  `text` is `None`.
- 2.5 `a_title_hands_back_to_the_caption_with_no_dip` — `Whole`, titled slate
  on [10, 12): at 11.75 the title at 1; at 12.0 the caption at 1.
- 2.6 `a_title_with_nothing_to_hand_back_to_leaves_by_its_out_mark` — `Off`:
  ½ at 11.75, `None` at 12.0.
- 2.7 `a_title_on_another_video_is_ignored` — `Off`; `None`.
- 2.8 `an_untitled_or_half_marked_slate_draws_nothing` — `Off`;
  `title: None` → `None`, and `out_seconds: None` → `None`.
- 2.9 `the_later_starting_of_two_titles_wins` — `Off`.

Elsewhere in core:

- 2.10 `tests/plan.rs`: `each_entry_carries_its_clips_caption_mode` — three clips
  `Whole` / `FirstSeconds` / `Off`; each entry's `caption_until ==
  clip.caption_bar.until()`, and the three values differ.
- 2.11 `tests/plan.rs`: `the_chapters_keep_their_words_with_every_caption_off` —
  two `Off` clips under `AllClips`; the chapters' titles equal the entries' text,
  which is not empty.
- 2.12 `tests/whole_match.rs`: `the_reel_and_the_whole_match_keep_their_caption`
  — `INFINITY` on both.
- 2.13 `tests/project_format.rs`: `a_v17_file_loads_under_the_current_version`.
  Write `caption_bar: Off`, `last_caption_bar: FirstSeconds` and the slate titled
  `"Corner"`, then remove the three keys. The file loads with `Whole`, `Whole`
  and `None`, and the write keeps `project.json.v17`.
- 2.14 `tests/project_format.rs`: `a_title_and_a_caption_round_trip` — the
  written JSON holds `"title"`, `"captionBar": "firstSeconds"` and
  `"lastCaptionBar"`.
- 2.15 `preferences_defaults_are_not_zero` (`project_format.rs:144`) gains
  `last_caption_bar == Whole`.
- 2.16 `tests/project_format.rs`:
  `the_caption_of_a_new_clip_comes_from_the_sticky_preference` (beside `:1250`).
- 2.17 `tests/slates.rs`: `a_title_is_one_line_and_an_empty_one_is_none` (beside
  `:158`) — `"two\nlines"` becomes `Some("two lines")`, `"  \n "` becomes `None`.
  `a_mark_moves_and_an_inverting_move_is_caught_in_both_directions` (`:88`) gains
  `(SlateEdit::Title("x".into()), "a title")` in its not-inverting list
  (`:111`–`:115`).
- 2.18 `tests/clip_order.rs`: `set_sets_one_field_and_returns_the_old_value`
  (`:170`) gains a `CaptionBar(Off)` / `CaptionBar(Whole)` row.

`naming::one_line` has no test of its own: `chapters.rs`'s
`a_title_is_flattened_to_one_line` (`core/tests/chapters.rs:160`) and 2.17
reach it.

### Sabotage

One mutation each. Predicted siblings are named; any other red is a finding.

1. `CAPTION_SECONDS = 3.5` → only 2.2.
2. `CaptionBar::Off.until()` → `CAPTION_SECONDS` → 2.3 **and 2.10** (the three
   values no longer differ — `FirstSeconds` and `Off` agree). Predicted.
   2.6–2.9 stay green: they run at record time ≥ 5.
3. `CaptionBar::Whole.until()` → `CAPTION_SECONDS` → 2.1 at 600, **2.5** at
   12.0 (record time ≥ 5, so no caption to hand back to, and the title fades),
   and **2.10** (two values agree). Predicted. 2.7 and 2.8 stay green: they are
   `Off`.
4. The `entry.text.is_empty()` branch dropped → only 2.4.
5. The `Some(…) if caption > 0.0` arm deleted → only 2.5, at 11.75.
6. The title-fading arm's opacity set to `1.0` → only 2.6, at 11.75.
7. The `source_index` filter dropped → only 2.7.
8. `s.out_seconds?` → `s.out_seconds.unwrap_or(f64::INFINITY)` → only 2.8, at
   its half-marked assertion.
9. `max_by` → `min_by` → only 2.9.
10. `clip_entry` writes `CaptionBar::Whole.until()` → only 2.10.
11. `clip_entry` writes an empty `text` for `Off` → only 2.11.
12. `reel.rs` writes `0.0` → only 2.12.
13. `CURRENT_FORMAT_VERSION` left at 17 → only 2.13, at the `.v17` assertion.
14. `rename_all = "camelCase"` removed from `CaptionBar` → only 2.14.
15. `Default`'s `last_caption_bar: Off` → only 2.15.
16. `add_recorded_clip` writes `CaptionBar::default()` → only 2.16.
17. `edit_slate` stores `Some(text)` unflattened → only 2.17, at both its
    assertions.
18. `would_invert`'s `Title(_) => true` → only
    `a_mark_moves_and_an_inverting_move_is_caught_in_both_directions`.
19. `Clip::set`'s new arm returns the new value → only 2.18.
20. `naming::one_line` splits on `' '` → `chapters.rs`'s
    `a_title_is_flattened_to_one_line` **and** 2.17. That is one shared function
    reached by two tests, which is §0.10's point; predicted.

---

## 3. Drawing the strip, in export and preview

### Files

- `media/src/overlay.rs`, `media/src/composite/export.rs`, `media/src/composite/preview.rs`
- `media/tests/export.rs`, `media/tests/preview.rs`
- `app/src/bus/export.rs`, `app/src/bus/basket.rs`, `app/src/bus/preview.rs`
- `harness/tests/export.rs`

### Steps

1. **`OverlayFrame::text` → `bar: Option<Strip<'a>>`**, with a new doc.
   - `draw`: `if let Some(strip) = frame.bar` replaces both empty-text guards.
     The fill is `BAR_ALPHA × opacity`, the label white at
     `round(255 × opacity)`.
   - `debug_assert!(strip.opacity > 0.0)`, and no zero-opacity branch (spec §S2).
2. **`draw_label` honours the colour's alpha** (`overlay.rs:655`). The callback
   computes `a = coverage × label.color.a() / 255` before premultiplying, citing
   `cosmic-text-0.19.0/src/swash.rs:212`–`:217`.
3. **The two memo docs** (§A) say the bar's line changes at most twice per title
   and re-fits once per change.
4. **`overlay.rs`'s tests**: `render_at` keeps its `text: &str` argument and maps
   `""` to `None`, anything else to `Some(Strip { text, opacity: 1.0 })`. The
   four `text: ""` literals become `bar: None`.
5. **`MatchMedia::of(project: &Project, folder: &Path, scoreboard:
   Option<ScoreboardContext>) -> MatchMedia`**, beside the struct (§A).
   - It builds `highlights`, `slates` and `avatar` from the project, and takes the
     board as given. The export's board depends on the picker
     (`carry.scoreboard`); the basket's is the project's own.
   - Both bus fills call it, replacing their literals.
   - **One constructor is what lets one harness test cover both fills**, and it
     removes the copy of "what a match contributes" that the two literals were.
6. **Export** (`composite/export.rs:737`–`:745`): `bar: strip_at(entry,
   record_time, slates, frame.source_time)`. `slates` is an empty slice for a
   backdrop picture — one `match` beside the highlights' (`:725`–`:735`), citing
   their comment rather than repeating it.
7. **Preview**:
   - `PreviewJob.slates` beside `highlights`; delete `Composite::text`;
   - compute the strip in the pump beside the board (`preview.rs:493`), from
     `entry` — an `Option` (`:488`), where `None` gives no strip — with
     `entry.record_time(n as usize)` and the same backdrop slice;
   - carry it on `Frame` and pass it on in `push`;
   - reword the module doc's *"simply `n/30`"*;
   - fill it at `bus/preview.rs:109` from the open project.

### Tests

**`pundit-media`** (lib unit tests; `cargo test -p pundit-media --test export --test preview`).

3.2, 3.3 and 3.5 build a **16 s** counter source directly:
`fixtures::counter_video(path, 640, 360, 30, 480, CounterKind::Vp8WebmWithAudio)`.
The `Source` fixture is 5 s (§0.6). Glyphs are counted with the existing
detector (§A).

- 3.1 `overlay.rs`: `a_half_faded_strip_halves_the_bar_and_its_glyphs` — at
  opacity ½, a glyph-free bar pixel's alpha is within 1 of 76.5
  (`0.6 × 0.5 × 255`), and the brightest glyph pixel's red is 128 ± 10.
- 3.2 `tests/export.rs`:
  `a_first_seconds_caption_leaves_the_export_after_three_seconds` — one clip,
  `FirstSeconds`, `start_source_seconds: 10`, 5 s long. More than 50 glyph
  pixels at 1 s, and 0 at 4 s.
- 3.3 `tests/export.rs`: `a_title_replaces_an_off_caption_inside_its_range` — an
  `Off` clip from source 10, and `MatchMedia { slates, ..default() }` with one
  slate titled on [11, 12). Glyphs at source 11.5; none at 10.5 or 12.5.
- 3.4 `tests/export.rs`: `a_whiteboard_entry_draws_no_title` — the job of
  `a_whiteboard_clip_exports_its_diagram_letterboxed` with a **black**
  `solid_png` diagram, `Off`, and a titled slate over the anchor: 0 glyph pixels.
- 3.5 `tests/preview.rs`:
  `a_first_seconds_caption_leaves_the_preview_after_three_seconds` — 3.2's clip
  previewed, sought to 1 s and to 4 s, `settled_picture` each time.

**`pundit-harness`** (`cargo test -p pundit-harness --test export`).

- 3.6 `an_exported_clip_burns_in_its_slates_title`. A **black**
  `solid_video(path, 640, 360, 30, 300, 0x000000, true)` (10 s) goes through
  `write_one_source_project` and `add_clips` (§0.9). Then, over the bus: a slate
  over the clip's middle, `EditSlate { Title }`, `EditClip { CaptionBar(Off) }`,
  and an export. The decoded frames (`pundit_media::fixtures`, as the file
  already does) hold glyphs inside the slate's range and none outside it. This
  also exercises the bus path for `SlateEdit::Title` (§0.7).

### Sabotage

1. The background's `× opacity` dropped → 3.1's background assertion only.
2. The alpha multiply in `draw_label` dropped → 3.1's glyph assertion only. The
   scoreboard and label tests draw opaque colours and stay green.
3. Export passes `frame.source_time` as the record time → only 3.2 (the clip
   starts at 10 s, so the caption is gone from frame 0). 3.3 is `Off`.
4. Export passes `&[]` for the slates → 3.3 **and** 3.6. Predicted.
5. The backdrop guard on the slates dropped → only 3.4.
6. Preview passes `frame.source_time` as the record time → only 3.5.
7. `MatchMedia::of` leaves `slates` empty → only 3.6. 3.3 builds its own
   `MatchMedia`.

The preview's bus fill is reached by no test: manual check 6.

---

## 4. The controls, the wiring, the guide and the prose

### Files

- `app/ui/app.slint`, `app/src/main.rs`, `app/src/bus/clips.rs`
- `app/tests/ui/{slate_fields, text_editing, export_sheet, slate_menu, fit_window, panel_widths, tag_field}.rs`
- `harness/tests/clips.rs`
- `CLAUDE.md`, `docs/book/src/guide/{slates, exporting, basket}.md`, `docs/hands-on-checklist.md`, `BACKLOG.md`

### Steps

1. **The title field** (spec §U1):
   - `SlateField` gains `title` (`app.slint:267`–`:270`);
   - `AppWindow` gains `in-out property <string> slate-title` beside
     `slate-name` (`:4157`);
   - `slate-title-edit := LineEdit` goes under `slate-name-edit`
     (`:5897`–`:5912`), with its handlers copied using `SlateField.title`,
     placeholder *"Title in exports"* and `enabled: !root.recording`;
   - `main.rs`: the `SlateField` match maps `Title` to
     `SlateEdit::Title(text.to_string())`, and `on_show_slate` (`:1555`,
     `:1574`–`:1575`) sets the title field under the existing #132 guard
     (`:1570`).
2. **The row marker** (spec §U2):
   - `SlateRow` (`app.slint:24`–`:45`) gains `title: string`, empty for none;
   - beside the `●` (`:5859`–`:5868`):

     ```slint
     if slate.title != "": Text {
         text: "T";
         color: Palette.alternate-foreground;
         font-size: 11px;
         vertical-alignment: center;

         Tooltip {
             text: @markdown("\{slate.title}");
         }
     }
     ```

   - `show_slates` fills `title`, and the six `tests/ui` fixtures gain
     `title: SharedString::new()`.
3. **The caption picker** (spec §U5), on the inset pickers' path at every hop:
   - the `Inspector` gets `in-out property <int> caption-index` and
     `callback caption-chosen(int)`, beside `inset-size-index` (`:943`) and
     `inset-size-chosen` (`:1005`);
   - the row goes under the inset row (`:1080`–`:1107`), with
     `min-width: 0; horizontal-stretch: 1` and the model
     `["Whole clip", "First 3 seconds", "Off"]`, whose comment names
     `caption::CAPTION_SECONDS` (§0.5);
   - `AppWindow` gets `clip-caption-index` and `callback set-caption-bar(string,
     int)`, beside `:4193` and `:4801`, bound on the `Inspector` instance
     (`:6599`–`:6625`);
   - in `main.rs`, `caption_bar_index` / `caption_bar_at` go beside
     `inset_size_index` (`:1087`); `on_set_caption_bar` sends
     `EditClip { edit: ClipEdit::CaptionBar(..) }`, beside the inset handlers
     (`:2182`–`:2205`); and `show_clip` sets the index (`:4013`).
4. **Sticky** (`bus/clips.rs:58`–`:65`): a `CaptionBar(v)` arm writing
   `last_caption_bar`. The comment's *"The inset's size and corner"* gains
   *"and the caption bar"*.
5. **The copy sentence**: `app.slint:2107` becomes *"…player highlights, titles
   and pen drawings can't ride a copy."*, along with `export_sheet.rs:47`'s
   `COPY_LINE` and `docs/hands-on-checklist.md:185`'s *"highlights and drawings
   can't ride a copy"* (*"highlights, titles and drawings…"*).
6. **`text_editing.rs`'s prose** (§A): reword `:19`–`:20`, `:63` and `:182` to
   drop the counts — *"every text field in `ui/app.slint`"*, *"the slate
   panel's fields"*, *"which one it was"*. The list in `window_fields()` is the
   count, as CLAUDE.md says of the keymap's one assertion; a number in prose
   beside it goes stale with every field, as it does here.
7. **CLAUDE.md**:
   - `:759`: the bar's line is the caption, or a slate's title while its range
     is on screen (`caption::strip_at`), under one inset rule.
   - `:984`: a basket piece's bar follows its own clip's mode.
   - After `:1112`, one slates bullet. A slate may carry a title (v18), drawn
     over its range in encoded exports and previews and never in the scan view.
     Re-timing it is re-marking the slate. The themed pass offers a titled slate
     like any other (spec §U4).
8. **The guide**:
   - `slates.md`: a section, *"Words over the range"*. Type a title under the
     slate's name; any export or preview showing that stretch of the game shows
     the words along the bottom in place of the clip's caption, fading as the
     range ends. Clear the field to remove it. A range with no end shows nothing
     until it has one.
   - `exporting.md:17`–`:19`: the inspector's **Caption bar** — whole clip, the
     first 3 seconds, or off.
   - `exporting.md:41`–`:42`: *"Highlights and titles cannot ride a copy."*
   - `basket.md:19`–`:20`: a piece's caption follows its clip's setting.
9. **BACKLOG**: #133 and #115 SHIPPED, with the PR number. Both index lines
   leave "Next, in order".

### Tests

**`pundit-app` `tests/ui`** (`cargo test -p pundit-app --test ui`).

- 4.1 `slate_fields.rs`: `the_title_field_commits_to_the_slate_it_opened_on`.
  **The first test of the `editing-slate-id` rule for any slate field**: the
  name field's #132 guard lives in `main.rs`, and nothing pins the Slint half.
  1. Select `s1`.
  2. **Click** into `AppWindow::slate-title-edit`, as `text_editing.rs`
     reaches its fields, and type `Corner`.
  3. Set `selected-slate` to `s2` and click elsewhere.
  4. Assert: `edit-slate-field` fired once, with `SlateField::Title` and
     `"Corner"`, and `editing-slate-id` read `s1` inside the callback.
- 4.2 `slate_fields.rs`:
  `a_titled_slate_shows_its_marker_and_an_untitled_one_none` — two rows, one
  titled; exactly one `T`, found by accessible label.
- 4.3 `text_editing.rs`: `window_fields()` (`:63`–`:104`) gains
  `AppWindow::slate-title-edit` / `get_slate_title`, which the existing
  `a_letter_typed_in_a_window_field_fires_no_shortcut` (`:228`) then covers.
- 4.4 `export_sheet.rs`: the three `COPY_LINE` users, unchanged in logic.

**`pundit-harness`** (`cargo test -p pundit-harness --test clips`).

- 4.5 `a_caption_edit_is_the_mode_the_next_recording_inherits`, on
  `an_inset_edit_is_the_placement_the_next_recording_inherits`'s shape
  (`harness/tests/clips.rs:186`). The preference follows the edit, including an
  edit to the value the clip already has; undo restores the clip and not the
  preference.

### Sabotage

1. The title field's handlers send `SlateField.name` → only 4.1.
2. The title field's focus-in no longer sets `editing-slate-id` → only 4.1, at
   its id assertion.
3. The marker's guard set to `if true` → only 4.2.
4. `bus/clips.rs`'s new arm removed → only 4.5.
5. **None for 4.3**: a `TextInput` joins `text-input-focused` by construction
   (CLAUDE.md, #130), so no mutation of ours can make the field miss.
6. **None for the `main.rs` wiring**, which no test can reach: manual checks.

---

## Manual checks (batched at the end)

On a two-half project with a scoreboard, a camera clip and a slate:

1. **A title over a clip.** Type a title on a closed slate, then preview a clip
   whose span crosses it.
   - The words replace the caption over the range, and the caption returns with
     no dip.
   - Set the clip to **Off**: the words fade over the range's last half second.
2. **Clearing a title.** Clear it: the row's `T` goes and the preview shows the
   caption throughout. Undo brings the title back.
3. **A half-marked slate.** Type a title on one: nothing shows in the preview.
   Press `o`: the title appears.
4. **The caption picker.**
   - In the inspector, **Caption bar** reads *Whole clip / First 3 seconds /
     Off*.
   - *First 3 seconds* shows for 2.5 s and fades by 3 s, in the preview and in
     an export.
   - A new take inherits the last choice.
5. **The whole match.** Export it on Default, with subtitles on: it is copied,
   the sheet's line names titles, and the file has none. With *Burned into the
   picture*, the titles are there.
6. **The preview's bus fill**, which no test reaches. Preview a clip under a
   titled slate: the title shows.
7. **A whiteboard take.** Record one inside a titled range: there is no title in
   its preview or its export, and its caption follows its own mode.
8. **A basket.** One piece whose clip is **Off** shows no caption; the others
   keep theirs.

---

## Risks

1. **The `main.rs` wiring and the preview fill are untested** — the `[[bin]]`
   limit, and a fill no harness test can see. Each is one mapping over tested
   pieces, and manual checks 1–6 cover them.
2. **The picker's label is a second spelling of `CAPTION_SECONDS`** (§0.5).
3. **Task 1 can change a value a test sees** if a helper leaves a field to a
   stub that disagrees with it. The unchanged test count and the sabotage are
   the guard. A red test in task 1 is a stub mismatch, not a bug in the test.
4. **The spec's accepted visual risks are unpinned by decision**: the fade
   freezing through a pause, an entry opening on a part-faded strip, and a
   one-frame jump when two windows end within half a second. Manual check 1 is
   where the coach judges them.

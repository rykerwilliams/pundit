# Design — titles over the footage, and a caption bar that can be switched

Two BACKLOG entries, one spec and **one format bump**, at the coach's request
(2026-10-12):

- **#133, the title maker** — typed words burned over the footage for a marked
  range of it (*"it would use a slate and a timing maybe"*, 2026-10-06).
- **#115, the caption bar switchable** — per clip, *off*, *for the whole
  entry* or *for the first few seconds* (*"this title should be toggleable in
  exported clips"*, then *"show title for the first 3 seconds or first `d`
  seconds"*, then *"per clip or in general"*, all 2026-10-02).

They are one feature from two ends: both decide **what the strip along the
bottom of the picture says, and for how long**. The coach's decisions, written
as decided:

**2026-10-12**

1. **A title replaces the caption bar for its window.** One strip: the title's
   words instead of the caption while the displayed frame's source time is
   inside the title's range; the caption comes back after.
2. **A title looks like the caption bar** — the same strip
   (`core::layout::bar_rect`), the same 60% black, the same font, the same
   fitting, the same stop short of the inset.
3. **Titles show in exports and previews only** — every encoded target whose
   footage crosses the range. They are not drawn live in the scan view.
4. **One spec, one format bump.**
5. **The strip leaves with a short fade (~0.5 s)**, both at the end of "first
   few seconds" and when a title's window ends.
6. **A basket piece honours its own clip's caption setting.** A reel has no
   clip, so it keeps its bar.

**2026-10-13**, answering the first draft's questions after its review:

7. **A title is part of the slate, not a record of its own.** A slate gains an
   optional title; its range is the slate's range. The first draft's separate
   `Title` record, its list, its undo step, its four commands and its panel are
   gone (§O7 says why #133's case against this no longer holds).
8. **"The first few seconds" is a constant, 3 s** — not a preference and not a
   control.
9. **A copied whole match carries no titles**, and "Default" still copies it:
   the sheet names titles beside highlights and pen drawings as things a copy
   cannot carry.

Read `CLAUDE.md` first: this spec does not repeat its format, composite,
overlay, basket, reel, slate, highlight or whiteboard rules. Every claim below
was re-derived from the source on 2026-10-13; §O lists where the two BACKLOG
entries no longer match it.

---

## §O. Where the entries are out of date

**O1. The format version.** #115 says "v15 to the mute switch… this is not
v15"; #133 says "#78, #115 and the whiteboard all want 16". Both are stale:
v16 went to #78's two export switches and v17 to the whiteboard's
`Clip::backdrop`. `CURRENT_FORMAT_VERSION` is **17** (`pundit-core/src/store.rs:21`),
so this is **v18**.

**O2. "Nothing in the overlay knows about time" (#115) is false, and #133's
correction of it overshoots.** #133 is right that `OverlayFrame` carries
`record_time` (`pundit-media/src/overlay.rs:114`) and that an empty `text`
suppresses the bar (`overlay.rs:124`–`:127`, guarded at `:367` and `:377`).
But it concludes the windowing *"needs no new plumbing in `overlay.rs` at
all"*, and the fade (decision 5) makes that untrue in two places:

- the bar's background is a fixed `BAR_ALPHA` (`overlay.rs:73`, filled at
  `:368`–`:372`), so a fade needs an opacity on it;
- the glyphs **cannot** be faded by handing `draw_label` a translucent colour.
  cosmic-text 0.19 (`Cargo.lock`) builds each mask pixel as
  `Color((coverage << 24) | base.0 & 0xFF_FF_FF)` — the base colour's alpha is
  thrown away, under its own `//TODO: blend base alpha?`
  (`cosmic-text-0.19.0/src/swash.rs:212`–`:217`, reached from `Buffer::draw`,
  `buffer.rs:1604`, through `LegacyRenderer`). Our callback
  (`overlay.rs:655`) then sees coverage only. §D2 is the fix.

**O3. #133's citation of the `Slate` doc has moved:** the *"slates are never
exported"* reasoning is now `pundit-core/src/project.rs:478`–`:482`, not
`:429`–`:433` — and decision 7 makes it untrue, so it is amended (§M1).

**O4. #115's storage advice contradicts itself, and its later half is
right.** Its first bullet says the field is *"an `Option`… never a defaulted
`bool`"*; its third-state bullet says an enum whose `Default` is "whole entry"
with a field-level `#[serde(default)]`. The second is what `project.rs`'s
header now says in so many words — *"any type whose `Default` is what an older
file means, which is why `Inset` defaults to `Camera`"* (`project.rs:12`–`:15`)
— and what v14's `StrokeEnd` already did. §M2 takes it.

**O5. "The reel and the basket have no clip to carry the switch" (#115) is
half wrong.** A reel's entries have `clip_id: None` (`pundit-core/src/reel.rs:203`);
a basket piece **is** a clip (`BasketPiece::clip`, `pundit-core/src/plan.rs:286`).
Decision 6 settles both.

**O6. The whole match has no caption bar on the encoded path either**, not
only the copy (#115 says only *"the whole match in track mode… has no bar
already"*). `whole_match_entries` writes `text: String::new()`
(`pundit-core/src/whole_match.rs:43`). It matters here because an encoded
whole match **will** show titles (§T).

**O7. #133's argument that a slate must not carry a title no longer holds.**
It made three points; each was about a slate *being* a title, and decision 7
makes a title a field a slate *has*:

- *"Slates are never exported"* (`project.rs:478`–`:482`). The doc used that
  to explain why a slate carries no `created_at` and no `sort_index`: nothing
  hand-orders slates, because nothing exported them in an order. That
  conclusion survives — a title is drawn where its range falls on the footage,
  so the only order that means anything is still `(source_index, in_seconds)`.
  Only the premise changes, and the doc is amended to say so (§M1).
- *"`o` would be closing a title."* `o` closes the most recently opened range
  on that video (`mark_slate_out`, `project.rs:846`). A title on a slate whose
  range is still open is simply **not drawn** until the range is closed
  (§S2), so `o` finishing the range is what makes the title appear — the right
  behaviour, not a collision. And re-timing a title is now exactly re-marking
  its slate (*Set in* / *Set out*), which is one control where the first draft
  had two.
- *"A row simultaneously un-shot and a title."* That is now a feature, not a
  contradiction: "shot" is about the commentary (`Clip::slate_id`,
  `project.rs:337`), the title is about words on the footage, and a range can
  want both. The two answer different questions about one range.

What decision 7 removes from the first draft — a `Title` struct,
`Project.titles`, a fourth term in `source_is_referenced` and two remap loops,
`UndoAction::EditTitles` and its purge arm, `bus/titles.rs`, `MakeTitle` /
`EditTitle` / `DeleteTitle` / `JumpToTitle`, and a Titles panel — is the
measure of how much a second record cost.

---

## §M. The model

**M1. `Slate.title: Option<String>`, with a field-level `#[serde(default)]`:**

```rust
/// v18. Words burned over the footage for this range in every export and
/// preview whose footage crosses it (BACKLOG #133), in place of the caption
/// bar. `None` is no title — every v12–v17 slate. Drawn only once the range
/// is closed: an open slate has no window to draw it in.
#[serde(default)]
pub title: Option<String>,
```

- **An `Option` added to an existing struct**, CLAUDE.md's rule for it
  (`project.rs:12`–`:13`); `None` is exactly what every older slate means. The
  field-level default is written even though serde treats a missing `Option`
  as `None` anyway (`project.rs:20`–`:25`), so the rule reads where it is
  applied.
- **Stored trimmed and on one line, and never `Some("")`**: an edit to empty
  stores `None`, so "has a title" is `is_some()` with no second test.
- **It belongs to the footage, through its slate**, which is already keyed by
  `source_index` and source seconds — `PlayerHighlight`'s rule and its reason
  (`pundit-core/src/highlight.rs:4`–`:10`): it shows wherever that footage
  does, in every clip export that crosses it, in the reel and in a basket
  piece.
- **Nothing else about a slate changes.** It already holds its source open
  (`source_is_referenced`, `project.rs:634`), already rides both remaps
  (`remove_source` `:672`–`:674`, `move_source` `:715`–`:717`), and its undo is
  already a snapshot of the whole list, `UndoAction::EditSlates`
  (`pundit-core/src/undo.rs:92`), taken by `Bus::edit_slates`
  (`pundit-app/src/bus/slates.rs:309`–`:322`) and applied by undo and redo in
  `bus/clips.rs:223`–`:226`. So a title edit is undoable, a source move purges
  its stale snapshots (`undo.rs:176`–`:180`), and deleting the slate deletes
  the title — with **no new code on any of those paths**.
- **The texts of `SourceReferenced`** (`project.rs:248`) and the bus's refusal
  (`pundit-app/src/bus/mod.rs:633`–`:636`) — *"a clip, a match event, a
  highlight or a slate"* — **stay as they are**: a title is a slate's, so
  "delete the slate" is still the instruction. So does the source row's
  tooltip (`pundit-app/ui/app.slint:5594`).
- **`Slate`'s doc is amended** (`project.rs:478`–`:482`). Today: *"slates are
  never exported, so the only order that means anything is
  `(source_index, in_seconds)`"*. After: a slate is never exported **as a film
  of its own** — its title is burned over the footage of any export that
  crosses its range, which places it on the footage and still needs no
  `created_at` and no `sort_index`.

**M2. `Clip.caption_bar: CaptionBar`, with a field-level `#[serde(default)]`:**

```rust
/// What a clip's caption bar does in an export or a preview (BACKLOG #115). v18.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CaptionBar {
    /// For the whole entry: what every v7–v17 clip did.
    #[default]
    Whole,
    /// For `CAPTION_SECONDS`, then gone.
    FirstSeconds,
    Off,
}
```

An enum, not `Option<bool>` or a defaulted `bool`, for O4's reason: three
states, and `Whole` *is* what an older file means — `Inset`'s and `StrokeEnd`'s
rule. It lives in a new module, `pundit-core/src/caption.rs`, with the strip
rule it feeds (§S), as `PlayerHighlight` lives beside `highlights_at`. Edited
through `ClipEdit::CaptionBar(CaptionBar)` beside `ShowPip` (`undo.rs:42`),
undoable like it, and applied by `Clip::set`.

**M3. `CAPTION_SECONDS` is a constant** (decision 8):

```rust
/// How long a `FirstSeconds` caption shows: the coach's own figure (#115).
pub const CAPTION_SECONDS: f64 = 3.0;
```

beside `STRIP_FADE` in `caption.rs`. Being a constant is what retires the
correctness review's findings 2, 3 and 6 on the first draft — every one of
them was about the seconds being a per-project preference (whose project a
basket piece reads it from, and what a spin box writes) — and with them
`Preferences::caption_seconds`, `Command::SetCaptionSeconds`, the spin box and
`BasketPiece::caption_seconds`.

**M4. One `Preferences` field, `last_caption_bar: CaptionBar`**, default
`Whole`, with **no field attribute** (the container carries `default`,
`project.rs:133`, and fills from the hand-written `Default` at `:201`). It is
the sticky last-used mode — "in general", in the coach's words — on exactly
`last_inset_size`'s mechanism and doc (`project.rs:179`–`:189`): written back
by `Bus::edit_clip` **before** its no-op guard
(`pundit-app/src/bus/clips.rs:58`–`:65`, for the reason argued there), never
reverted by undo, and read by `add_recorded_clip` beside `inset_size`
(`project.rs:808`). It joins `preferences_defaults_are_not_zero`
(`pundit-core/tests/project_format.rs:144`).

**M5. The bump: v18, readable floor unchanged at 7.** Three additions, all
additive: `Slate.title`, `Clip.caption_bar`, `Preferences.last_caption_bar`.
**The reason is the forward direction**, as for v16 and v17: serde ignores
unknown keys, so a v17 build would read a v18 file, drop every title and reset
every clip's caption to `Whole` on its next save — words the coach typed, gone
without a message. The bump makes it refuse the file; `project.json.v17` is
what it can go back to. Wire spellings: `title`, `captionBar` (`"whole"` /
`"firstSeconds"` / `"off"`), `lastCaptionBar`. CLAUDE.md's format list gains a
v18 bullet in the same change.

---

## §S. The strip: one rule, in core

**S1. The plan resolves each entry's caption to one number.** `PlanEntry`
gains

```rust
/// The record time at which this entry's caption bar is gone: `INFINITY` for
/// the whole entry, `CAPTION_SECONDS` for `FirstSeconds`, and `0.0` for `Off`.
/// `text` is untouched either way — it is also the chapter.
pub caption_until: f64,
```

- **Why not blank `text` for `Off`:** `entry_chapters` titles each chapter
  with `e.text` (`plan.rs:152`–`:160`), so blanking it would cost the chapters
  their words. #115 asks for exactly the opposite — *"the chapters keep their
  words"* — and this keeps them with no rule to remember.
- **Why a number and not the enum:** one formula (§S3) covers all three
  states, `Off` included, so the per-frame code has no branch on the mode.
- **Who sets it:** `clip_entry` (`plan.rs:209`), the one builder of a clip's
  entry, reads `clip.caption_bar` — so `compilation_plan` (`:234`) and
  `basket_plan` (`:304`) both get it with no new argument, and a basket piece
  honours its own clip (decision 6) because the piece *is* that clip.
  `reel_plan` (`reel.rs:202`) and `whole_match_entries` (`whole_match.rs:33`)
  write `f64::INFINITY`: they have no clip, and a reel keeps its bar.
- **The time base is the entry's own record time**, `PlanEntry::record_time`
  (`plan.rs:109`), which *is* elapsed output time inside the entry ("record
  time is output time", `plan.rs:83`–`:86`) — so "the first 3 seconds" means
  the first 3 seconds the viewer sees, pauses included.

**S2. `caption::strip_at` picks the words and their opacity per frame:**

```rust
/// How long the strip takes to leave (decision 5).
pub const STRIP_FADE: f64 = 0.5;

pub struct Strip<'a> {
    pub text: &'a str,
    /// In (0, 1], multiplying the bar's 60% black and its glyphs alike.
    pub opacity: f64,
}

/// 1 until `STRIP_FADE` before `end`, then linear to 0 at it.
fn leaving(end: f64, t: f64) -> f64 {
    ((end - t) / STRIP_FADE).clamp(0.0, 1.0)
}

pub fn strip_at<'a>(
    entry: &'a PlanEntry,
    record_time: f64,
    slates: &'a [Slate],
    source_time: f64,
) -> Option<Strip<'a>> {
    let caption = if entry.text.is_empty() {
        0.0
    } else {
        leaving(entry.caption_until, record_time)
    };
    // The titled, closed range covering this frame; the latest-starting wins
    // an overlap.
    let title = slates
        .iter()
        .filter(|s| s.source_index == entry.source_index)
        .filter_map(|s| Some((s, s.title.as_deref()?, s.out_seconds?)))
        .filter(|(s, _, out)| s.in_seconds <= source_time && source_time < *out)
        .max_by(|a, b| a.0.in_seconds.total_cmp(&b.0.in_seconds));
    match title {
        // A caption to come back to: the words swap and the strip stays.
        Some((_, text, _)) if caption > 0.0 => Some(Strip { text, opacity: 1.0 }),
        // Nothing to come back to: the strip leaves with the title.
        Some((_, text, out)) => Some(Strip { text, opacity: leaving(out, source_time) }),
        None => (caption > 0.0).then_some(Strip { text: &entry.text, opacity: caption }),
    }
}
```

- **A title on a half-marked slate is not drawn** — `s.out_seconds?` drops it.
  It is stored, shown on the slate, and appears the moment `o` (or *Set out*)
  closes the range.
- **`None` is "no strip at all"**, which replaces `OverlayFrame::text`'s
  "empty means none" convention (§D1) with a type.
- **It never answers `Some` at opacity 0.** The title arm's window ends at
  `out`, where `leaving` first reaches 0, and the caption arm is guarded by
  `caption > 0.0`. So the overlay needs no "opacity 0 draws nothing" branch; a
  `debug_assert!(strip.opacity > 0.0)` at the draw is the whole of it.
- It is core's, pure, and tested there (§Testing) — both tails call it and
  neither decides anything.

**S3. The fade, and why it is *inside* the window.** The strip is gone **at**
the end of its window and fades over the window's last half second:
`FirstSeconds` is 2.5 s at full strength and gone at 3.0; a title is gone at
its slate's out mark. The alternative — full strength to the end, then a fade
after it — was rejected: "the first 3 seconds" would mean 3.5, and a title
would spill past the coach's out mark into whatever he marked next. `leaving`
is one clamp, and `INFINITY` and `0.0` fall out of it: `Whole` is 1 throughout
and `Off` is 0 from the first frame.

**S4. No fade in.** A caption arrives with the entry's cut, which is already a
hard change; a title arrives on the frame the coach marked, and a fade in would
spend that frame and the next fourteen at partial strength. Decision 5 asked
for a fade *out*; this adds nothing it did not ask for.

**S5. A title ending mid-entry hands back with a hard swap of the words, not a
cross-fade.** While the caption is showing (`caption > 0`), a title is drawn at
full strength to its out mark and the caption's words replace it on the next
frame — the strip itself never moves. A cross-fade would draw two lines at
once (two shapings a frame, and the `TextSlot::Bar` memo, `overlay.rs:167`,
thrashing between them for fifteen frames); fading the title out over a
caption that then pops back in would dim the strip and re-brighten it, which
reads as a flicker. **The title fades only when nothing follows it** — the
caption `Off`, past its 3 s, or empty (an encoded whole match, O6). The
review looked for something simpler than these three arms and found nothing;
they stay as written.

- **Two docs become false and change with this:** `OverlayRenderer::fitted`'s
  *"none of those lines changes inside one"* (`overlay.rs:143`–`:145`) and
  `TextSlot`'s *"a line that holds still for a whole entry"*
  (`overlay.rs:158`–`:160`). The bar's line now changes inside an entry at
  most twice per title it crosses, so the memo still pays — it re-fits once
  per change rather than once per frame — and the docs should say exactly that.

**S6. Two time bases, each the right one.** A title reads `frame.source_time`
(`pundit-core/src/export.rs:106`), because it is bounded on the footage — so it
freezes with the footage through a commentary pause and reappears if the
commentary rewinds over it, as a highlight does. The caption reads record time
(§S1), because "the first few seconds" is about the viewer's clock. Neither
`record_time` nor `source_time` is new to either tail: export has both at
`pundit-media/src/composite/export.rs:663` and `:693`, preview at
`composite/preview.rs:487` and `:917`.

**S7. Not over a whiteboard.** A whiteboard clip's `source_index` and anchor are
real (CLAUDE.md, v17) and its one `Freeze` segment holds `source_time` at the
anchor, so a title covering the anchor would be drawn over the diagram for the
whole take. Both tails already hand the highlights an **empty slice** for a
backdrop picture (`composite/export.rs:725`–`:735`, `composite/preview.rs:518`–`:529`);
the slates take the same slice, at the same two sites, for the same reason —
*"one drawn over a diagram would be a ring placed on footage that is not on
screen"*. The overlay never learns a whiteboard exists. **The whiteboard's own
caption is unaffected**: a whiteboard clip carries a caption bar (whiteboard
spec W1) and its `caption_bar` applies like any clip's.

**S8. The inset rule is unchanged.** The strip is `bar_rect(out_w, out_h,
clip.inset_placement())` whatever words it carries (`overlay.rs:366`), so a
title stops short of the coach's face exactly as the caption does
(decision 2).

---

## §D. Drawing it

**D1. `OverlayFrame::text: &str` becomes `bar: Option<Strip<'a>>`**
(`overlay.rs:124`–`:127`). Its doc's *"Empty draws no bar at all… Neither
shipping caller does"* goes with it: both callers now pass whatever
`strip_at` answers. `draw` (`overlay.rs:352`–`:396`) keeps macOS's order —
background, strokes, glyphs, board — and only reads its line and opacity from
the `Strip`: the background is `BAR_ALPHA × opacity`, the glyphs as D2.

**D2. `draw_label` honours its colour's alpha** (`overlay.rs:655`): the
callback multiplies each pixel's coverage by `label.color.a()` before
premultiplying, putting back what cosmic-text drops (O2). The bar passes white
at `round(255 × opacity)`. **This is the smaller fix**: a `Label::opacity`
field would add a value to all seven `Label` constructions to say what the
colour already says. It changes nothing else drawn today — every other label is
opaque in practice: the highlight ink is black or white (`label_ink`,
`highlight.rs:406`), and every team colour the port writes comes from
`match_panel::parse_hex`, which always sets `a: 1.0`
(`pundit-app/src/match_panel.rs:394`–`:399`). A hand-edited translucent team
font colour would now draw translucent, which is what it says.

**D3. Both tails compute the strip and nothing else changes in them.**

- **Export** (`composite/export.rs:737`–`:745`): `bar: strip_at(entry,
  record_time, slates, frame.source_time)`, with `slates` the entry's own
  match's — `MatchMedia` (`:186`) gains `slates: Vec<Slate>` beside
  `highlights` (`:193`), filled by `bus/export.rs::job` (`:1295`–`:1297`) and
  by the basket (`bus/basket.rs:435`–`:440`), each from its own project. A job
  is a snapshot, so a title edit or a source change never reaches a running
  export — `MatchMedia`'s existing guarantee (`:181`–`:185`). The whole list
  is carried and `strip_at` filters it; pre-filtering to titled slates in the
  bus would be a second place for the "has a title" rule.
- **Preview**: `PreviewJob` gains `slates` beside `highlights`
  (`composite/preview.rs:156`), filled at `bus/preview.rs:116`. The pump's
  entry is an **`Option`** (`preview.rs:488`, `plan.entries.get(frame.entry)`),
  as it already is for the board and the highlights; **`None` gives no strip**.
  `Composite::text` (`preview.rs:682`, set once at `:881`–`:886`, *"fixed for
  the run, since the entry is"*) is **deleted**: the strip is no longer fixed
  for the run, so it is computed in the pump beside the board (`:493`) and
  carried on `Frame`, and `push` passes it through (`:915`–`:922`). Preview's
  record time becomes `entry.record_time(n)`, export's own expression, rather
  than `n / 30` — equal for a one-entry compilation, and one fewer place for
  the two tails to differ.

---

## §T. Which targets show what

| Target | Caption bar | Titles |
|---|---|---|
| One clip, All clips, a tag | the clip's `caption_bar` | yes |
| Basket piece | **its own clip's** `caption_bar` (decision 6) | yes, its own match's |
| Goals reel | always (`clip_id: None`, `reel.rs:203`) | yes |
| Whole match, encoded | none — empty text (`whole_match.rs:43`) | yes |
| Whole match, copied | none | **no** (T1, decision 9) |
| Clip preview | the clip's `caption_bar` | yes |
| Scan view | none | **no** (decision 3) |
| Whiteboard clip, anywhere | the clip's `caption_bar` | **no** (S7) |

**T1. The copied whole match carries no titles, and "Default" still copies**
(decision 9). A copy joins the sources' packets (`Render::Copy`,
`composite/export.rs:91`–`:99`) and draws nothing — highlights and pen
drawings already ride no copy, and the sheet says so under the picker:
*"player highlights and pen drawings can't ride a copy"* (`app.slint:2107`).
Titles take the highlights' rule: the sentence becomes *"player highlights,
titles and pen drawings can't ride a copy"*, and `carry_scoreboard`
(`bus/export.rs:978`) is untouched. The scoreboard's *"Default never trades the
board away"* rule (the `Track` arm, `bus/export.rs:1010`–`:1021`) was the
alternative; it would have made one five-second title cost an hour-long
re-encode of the whole match.

**T2. The chapters keep their words** in every row of the table, because
`text` is never blanked (§S1). A title is not a chapter.

---

## §U. The UI

**U1. The title is a third field among the slate's fields** — the smallest UI
there is, because the fields already exist and already edit the selected
slate. Under the name field (`slate-name-edit`, `app.slint:5897`), in the
block that lives **outside the list** for that block's own reason
(`:5876`–`:5892`: a field inside the `for` is destroyed by every rebuild,
including the one its own commit triggers, and the typing is lost):

```text
[ corner2                          ]   <- name (filing)
[ Title in exports                 ]   <- title: the words a viewer reads
[ tags, comma separated ] [Set in] [Set out] [Record]
```

- Its placeholder is *"Title in exports"*, which is the whole explanation the
  field needs: the name above it is the coach's filing, and the title is what a
  viewer reads.
- It commits exactly as the name does — on Enter and on focus-out, to the slate
  the field **opened on** (`editing-slate-id`, `:5901`–`:5911`) — through the
  same callback, `edit-slate-field`, with a third `SlateField` member, `title`
  (`app.slint:267`–`:270`).
- **`enabled: !root.recording`**, the name field's own gate (`:5900`): editing a
  slate waits while recording, and the bus refuses it anyway (§B).
- **No "Make title" menu item.** Typing words into the field *is* making a
  title, and clearing it is deleting one; the first draft's menu item, Titles
  panel, row selection and row menu all go. The slate row's menu
  (`slate-menu`, `app.slint:5753`) is unchanged.

**U2. The slate row says when it has a title**: a small `T` beside the existing
`●` shot marker (`app.slint:5860`), with the words as its tooltip, shown only
when the title is set. `SlateRow` gains `title: string`, empty for none, built
in `show_slates` (`pundit-app/src/main.rs:3869`). **An empty title shows
nothing** — no marker and no text — so a project that never uses titles sees
no change in the list. A title on a half-marked slate gets its marker too: the
row's `14:05–` already says the range is unfinished, and a finished range is
what makes it draw.

**U3. Re-timing a title is re-marking its slate** — *Set in* / *Set out*
(`app.slint:5945`–`:5968`) — and **the selected slate's span on the scrubber
already shows its range** (slates spec S6), so where a title falls is visible
with no band of its own.

**U4. The themed pass offers a titled slate like any other**, unchanged.
`slate_pass::next_slate` (`pundit-app/src/slate_pass.rs:75`) parks on the first
`timed && !shot` row. A `title.is_none()` term was considered and rejected: a
title says nothing about whether the range also wants commentary — a corner
can carry both words and a take — so the term would wrongly skip ranges that
want both. And the pass runs over a tag-filtered list in practice (BACKLOG #120),
where a range marked only for its words, untagged, is not in it. The
cost is that an unfiltered pass parks on a title-only range, and the coach
clicks the next row.

**U5. The inspector's caption control**, under "Inset size and corner"
(`app.slint:1080`) and in its shape:

```text
Caption bar
[ Whole clip                        v ]
```

- A `ComboBox` over `["Whole clip", "First 3 seconds", "Off"]` sending
  `EditClip { edit: ClipEdit::CaptionBar(..) }`, as the inset pickers send
  theirs (`main.rs:2182`–`:2205`). The middle label states the constant, so
  nothing else on screen has to.
- `min-width: 0; horizontal-stretch: 1`, for the inset row's measured reason
  (`app.slint:1066`–`:1076`): a default-width `ComboBox` declares 160px in a
  280px column.

---

## §B. The bus

| Command | Does | While recording |
|---|---|---|
| `EditSlate { id, edit: SlateEdit::Title(text) }` | the existing command, a new variant | refused (not on the allow-list, `bus/mod.rs:1128`–`:1129` lists only the marks) |
| `EditClip { CaptionBar(..) }` | the existing command; sticky write-back of `last_caption_bar` | as today |

**No new command.**

- **`SlateEdit::Title(String)`** joins `Name` and `Tags` (`project.rs:527`).
  `Project::edit_slate` (`:876`) stores it flattened to one line and turns an
  empty result into `None` (§M1); `Slate::would_invert` (`:512`) gains it in
  its `false` arm with `Name` and `Tags`; and `main.rs`'s `on_edit_slate_field`
  (`:1457`–`:1469`) maps `SlateField::Title` to it. `Bus::edit_slate`
  (`bus/slates.rs:288`) and `edit_slates` (`:309`) are untouched: the snapshot,
  the "nothing changed costs neither a save nor a step" rule and the undo
  already cover a third field.
- **One flattener, not two.** `chapters.rs`'s private `one_line` (`:55`) joins
  `split_whitespace()` with single spaces and answers `UNTITLED` for an empty
  title. The joining half moves to `core::naming` as `pub fn one_line(&str) ->
  String` — that module already holds "what a name may contain"
  (`pundit-core/src/naming.rs:1`–`:8`) — and `chapters.rs` keeps its
  empty-to-`UNTITLED` rule on top of it, so
  `chapters.rs::a_title_is_flattened_to_one_line`
  (`pundit-core/tests/chapters.rs:160`) still pins it. The slate title takes
  the same function and maps empty to `None` instead. The bar needs one line
  for the chapter list's own reason: it draws one row with `Wrap::None`
  (`overlay.rs:783`–`:788`).
- **Shooting a slate does not copy the title to the clip.** `shoot_slate` gives
  the take the slate's name and tags; the title stays on the slate, which is
  where the footage-keyed rule reads it, so the take's own export shows it
  anyway when its footage crosses the range.

---

## Crate responsibilities

| Crate | Change |
|---|---|
| `pundit-core` | **New `caption.rs`**: `CaptionBar`, `CAPTION_SECONDS`, `STRIP_FADE`, `Strip`, `strip_at`. `project.rs`: `Slate.title` and its amended doc, `SlateEdit::Title` through `edit_slate` and `would_invert`, `Clip.caption_bar`, `Preferences.last_caption_bar`, `add_recorded_clip`'s seed. `undo.rs`: `ClipEdit::CaptionBar`. `plan.rs`: `PlanEntry::caption_until`, set in `clip_entry`; `reel.rs` and `whole_match.rs` write `INFINITY`. `naming.rs`: `one_line`, moved from `chapters.rs`. `store.rs`: 17 → 18. |
| `pundit-media` | `overlay.rs`: `bar: Option<Strip>`, the background's opacity, `draw_label` honouring alpha, the two memo docs. `composite/export.rs`: `MatchMedia::slates`, `strip_at` per frame, the whiteboard's empty slice. `composite/preview.rs`: `PreviewJob::slates`, the strip per frame, `Composite::text` deleted. **No new element, pad or decode.** |
| `pundit-app` (bus) | The sticky write-back in `bus/clips.rs`; `slates` into `MatchMedia` in `bus/export.rs` and `bus/basket.rs`, and into `PreviewJob` in `bus/preview.rs`. **No new command and no new module.** |
| `pundit-app` (UI) | `app.slint`: the title field, `SlateField.title`, the row's `T` marker and `SlateRow.title`, the inspector's caption picker, the copy sentence. `main.rs`: the field mapping, the row's title, the inspector's value. |
| `pundit-harness` | Tests only. |

`pundit-core` gains no dependency.

---

## Testing

Which binary can reach each claim. **`main.rs` is reachable by none of them**,
so it holds wiring and a row mapping and no rule.

**`pundit-core` (`tests/caption.rs`, new, and the existing files).**

1. `strip_at`, as a table: `Whole` is opacity 1 throughout; `FirstSeconds` is
   1 at 2.4, ½ at 2.75, `None` at 3.0; `Off` is `None` from frame 0; empty
   text is `None` whatever the mode.
2. A title replaces the caption inside `[in, out)` at opacity 1 and the caption
   is back on the first frame at `out` — **no dip** (§S5). Over `Off`, the same
   title is ½ at `out − 0.25` and `None` at `out`.
3. A titled slate on another `source_index` is ignored; an untitled slate is
   ignored; **a titled slate with no out point is ignored** and the caption
   shows through; of two overlapping titles the later-starting one wins.
4. `compilation_plan` writes `caption_until` from each clip's mode;
   `basket_plan` from each piece's clip (two pieces, two modes); `reel_plan`
   and the whole match write `INFINITY`; and `entry_chapters`' titles are
   identical with every clip `Off` — the "chapters keep their words" pin.
5. Format (`project_format.rs`): `a_v17_file_loads_under_the_current_version`
   (writes each of the three new keys away from its default, removes them,
   asserts the defaults and `project.json.v17`), a titled slate round-tripping
   at its wire spelling, and one new line in `preferences_defaults_are_not_zero`.
6. `add_recorded_clip` takes `last_caption_bar`.
7. `edit_slate(Title)` flattens a pasted two-line string to one line, and an
   all-whitespace title stores `None`; `would_invert` is `false` for it.
8. `naming::one_line` (moved), and the existing
   `a_title_is_flattened_to_one_line` still passing over it.

**`pundit-media` (`overlay.rs`'s unit tests, `tests/export.rs`,
`tests/preview.rs`).**

9. Overlay: a `Strip` at opacity ½ fills the bar at half of `BAR_ALPHA`'s alpha,
   and its **glyphs** at half their alpha — the second assertion is the one
   that fails without §D2's fix, which is the point of writing it.
10. Export of one clip with `FirstSeconds`, **asserted on the glyphs, not the
    tint**: the counter fixture is black, so the bar's tint over it is
    invisible, and the existing bar test counts white glyph pixels inside
    `bar_rect` instead (`pundit-media/tests/export.rs:558`–`:578`, the count at
    `:570`). A frame at 1 s has glyphs in the bar and a frame at 4 s has none.
    An export of an `Off` clip with a titled slate covering part of it: glyphs
    inside the window, none outside it.
11. **The whiteboard exclusion**: a one-entry export with a backdrop picture,
    caption `Off` and a titled slate covering its anchor has **no** glyphs in
    `bar_rect` — the only test that can see S7, since the rule lives in the
    tail.
12. Preview of the same `FirstSeconds` clip agrees with the export at the same
    two instants, by the same glyph count ("the preview is the export").

**`pundit-harness` (over the bus).**

13. `EditSlate { Title }` stores the title, undo removes it and redo restores
    it; clearing it stores `None`; deleting the slate deletes it and undo brings
    both back. All of it through the existing `EditSlates` step.
14. `EditClip CaptionBar` writes `last_caption_bar` — including when it equals
    the clip's (the sticky rule) — and the next recording over
    `CaptureKind::Test` inherits it; undo restores the clip and not the
    preference.
15. An All Clips export over a project with a `FirstSeconds` clip and a titled
    slate runs to EOS (the full path, with `MatchMedia::slates` filled by the
    bus).

**`pundit-app` `tests/ui` (modules of the one binary).**

16. `slate_fields.rs`: the title field commits to the slate it **opened on**
    when focus leaves after the selection changed — the name field's pinned
    failure, for the third field.
17. `text_editing.rs`: the title field gates the shortcuts — the test types a
    letter into every field there is, so it joins its list.
18. The slate row's `T` marker is present for a row with a title and absent for
    one without, found by its accessible label.

---

## Risks

1. **A title's fade freezes through a pause in its last half second.** The
   fade is on source time (§S6), so a commentary pause inside it holds the
   strip at partial strength for the length of the pause. Accepted: it is a
   pause within 0.5 s of a mark, the strip is still legible, and the fix —
   fading on output time — needs state across frames that the pure `strip_at`
   does not have.
2. **An entry that starts inside a title's last half second opens on a
   partly faded strip.** Rare, honest (the footage's title is leaving), and the
   same shape as a highlight cut by an entry boundary.
3. **Two windows ending close together on different clocks give a one-frame
   jump**, in either direction:
   - the caption's 3 s run out inside a title's last half second, so the title
     drops from 1 to its fading value on one frame;
   - a title **starts** while the caption is fading out, so the strip jumps
     from partial strength back to 1 with the title's words.
   Each needs a title edge within 0.5 s of the 3 s mark; not worth state.
4. **`draw_label` honouring alpha** changes any label with a stored alpha below
   1, which the port never writes (§D2). A hand-edited project could see a
   translucent team name; that is what its file says.
5. **A title on a half-marked slate is invisible in every export** until the
   range is closed. The row's open range (`14:05–`) is the only sign; a coach
   who types a title and never presses `o` gets no words.
6. **The literals the compiler will find**, measured: **19** `Clip` literals
   name every field and gain `caption_bar` — 14 in `pundit-core` (13 test
   helpers and `add_recorded_clip`), 4 in `pundit-media` (`overlay.rs`,
   `composite/export.rs`, `tests/export.rs`, `tests/preview.rs`) and 1 in
   `pundit-harness`; `pundit-app` has none, and every other `Clip { .. }` in
   the tests spreads off one of those helpers. **4** `PlanEntry` literals gain
   `caption_until` (`reel.rs:202`, `whole_match.rs:33`, `plan.rs:213`,
   `core/tests/audio.rs:147`). **8** `Slate` literals gain `title`, all in
   `pundit-core` (`mark_slate_in` and seven in tests). Two `SlateRow` fixtures
   in `tests/ui` (`slate_menu.rs`, `slate_fields.rs`) gain `title`.

---

## Deferred

- **A title over a range that is not a slate**, or several titles on one
  slate. Decision 7 makes one slate, one title; nothing has asked for more.
- **A fade in** (§S4), **title styling** (a position other than the bottom
  strip, a size) and **titles in the `.srt`** — none has been asked for.
- **#115's lighter variant — dropping `1 / 1` from a one-entry export's
  line.** With the switch shipped the coach can turn the bar off or shorten it,
  which is what he asked for; the variant would also change a one-goal reel's
  line (`reel.rs`) and six assertions in `tests/plan.rs` and `tests/reel.rs`.
  Offer it when he next exports one clip.
- **Setting the mode on many clips at once** — #45's multi-select, as #115
  already says.

---

## Open questions for the coach

None. The three the first draft asked — whether a title is its own record, how
the seconds are set, and whether a copied whole match should carry titles —
were answered on 2026-10-13 (decisions 7–9).

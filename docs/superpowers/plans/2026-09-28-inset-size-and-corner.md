# Plan — the inset's size and corner, per clip (#88)

Spec: `docs/superpowers/specs/2026-09-28-inset-size-and-corner-design.md`. Read it
before any task; this plan does not repeat its reasoning. Read `CLAUDE.md` too.

**Four tasks, one crate each, so each pays one build.** A first draft had six, split
so that changing a signature and fixing its callers were different tasks — which
cannot work here: the geometry signatures have ~35 call sites across three crates
and their tests, so the "signature" task's own verify step would have had to do the
"callers" task as well. The only way to stage them would be a `pip_rect` /
`pip_rect_at` shim pair, i.e. exactly the second source of truth this change exists
to remove.

Every `cargo` call except `fmt` goes through
`flock /tmp/claude-1000/cargo.lock nice -n 19 cargo …`. `pundit-core` is fast and
needs no GStreamer; media and app builds are ~3 minutes. **Never pipe clippy or a
test run to `tail`** — it masks the exit status, which has let a clippy failure be
committed here before.

## The four traps

Each is a bug that compiles cleanly. Everything else is in the spec.

1. **Two serde rules in one change.** The `Clip` fields take a **field-level**
   `#[serde(default)]` — their `Default` is what an older file means. The
   `Preferences` fields take **none**: that container already carries one and fills
   from its hand-written `Default`, so a field-level one is a second copy.
2. **v13, not v12.** `store.rs` is at 12 (slates). BACKLOG #88 says "bump to 12";
   trusting it writes a version every 0.9.0 project already claims.
3. **The `show_pip` check must survive, and the accessor is where to guarantee
   it.** Today the call is
   `bar_rect(out_w, out_h, frame.clip.is_some_and(Clip::shows_inset))`. The obvious
   translation — `frame.clip.map(Clip::inset_placement)` — **compiles and is
   wrong**: `Option::map` is `Some` for *every* clip, so one with the inset
   switched off gets a bar cut at the inset's column instead of full width, in
   export **and** preview. So **`Clip::inset_placement()` returns
   `Option<InsetPlacement>`, `None` when `!shows_inset`** — then `bar_rect` takes
   it verbatim and the bug is unwritable, rather than depending on a `filter` being
   remembered at two call sites. That also keeps `show_pip × inset` "interpreted
   here and nowhere else", which is the rule the two `shows_*` predicates exist
   for. `bar_rect`'s doc says `None` means **"no inset is drawn"**, not "no clip".
   The regression test that would catch this already exists and builds a
   `show_pip: false` clip for the purpose
   (`overlay.rs::a_long_caption_stops_where_the_inset_stands`) — but B2 rewrites
   its `pip_left` line, so it is in the hands of whoever edits it.
4. **`inset_span` returns `(x, w)`, never `(left, right)`.** Deriving the width as
   `right - left` is **not** today's rect: measured,
   `1920 − (1920 − 0.22·1920) = 422.4000000000001` against `0.22·1920 = 422.4`,
   and likewise at 1280/0.16 and 3840/0.22. That alone would break the
   pixel-identity pin and the two shipped exact-equality tests. `w` is always
   `inset_ratio(size) · out_w`; the corner decides only `x`.

---

## B1. Core: the fields, the format, the geometry, and all of core's tests

**Files:** `crates/pundit-core/src/{project.rs, store.rs, layout.rs, avatar.rs,
undo.rs}` and `crates/pundit-core/tests/{layout.rs, avatar.rs, project_format.rs}`.
(A draft listed `tests/recording.rs`; it has no `Clip` literal, no
`add_recorded_clip` and no geometry. The preference-seeding test belongs in
`project_format.rs` beside `show_pip_comes_from_preferences`.)

**This task knowingly leaves the workspace red downstream**, and the radius is
wider than the geometry: `Clip` has **no `Default`**, so adding two fields breaks
**every full `Clip` struct literal** — about 20 of them across 19 files, findable
with `grep -rn "recording_filename:"` (two in `project.rs`, fifteen in
`pundit-core/tests/`, plus `pundit-harness/src/lib.rs`, `pundit-media/src/overlay.rs`,
`pundit-media/src/composite/export.rs` and `pundit-media/tests/{export,preview}.rs`).
Fix core's own; leave the other crates' to B2 and B3. Do **not** add a shim or a
`Default` to keep them building — a shim is the second source of truth this change
exists to remove.

Gate on `cargo test -p pundit-core`: fast, no GStreamer, and the only gate
available until B2 lands.

1. **`InsetSize`** (Small / **Medium** / Large) and **`InsetCorner`**
   (**BottomRight** / BottomLeft / TopRight), shaped exactly like `Inset`:
   `#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]`,
   `#[serde(rename_all = "camelCase")]`, a `#[default]` variant.
2. **`InsetPlacement { size, corner }`** — a plain `Copy` struct with a derived
   `Default`, **not serialized** (it is read off the clip). One type threaded
   through six signatures is less code than two arguments, and it gives
   Medium/BottomRight one home. **`Clip::inset_placement() -> Option<InsetPlacement>`,
   `None` when `!shows_inset`** (trap 3) — in the spirit of `shows_inset`, and the
   reason the bar's bug is unwritable.
3. **`Clip::inset_size` / `inset_corner`** with field-level `#[serde(default)]`
   (trap 1), documented as v13 the way `inset` documents v10.
   **`Preferences::last_inset_size` / `last_inset_corner`** with **no field
   attribute** (trap 1), their values in `Preferences`' hand-written `Default`,
   beside the `last_export_*` fields they are modelled on.
4. **`CURRENT_FORMAT_VERSION` → 13** (trap 2); `MIN_READABLE_FORMAT_VERSION` stays
   7.
5. **Two `ClipEdit` variants** and their arms in `Clip::set` — "the one definition
   of a field change". Undoable, like `ShowPip`. (`Clip::set`'s match is
   exhaustive, so this belongs with the fields, not a later task.)
6. **`Project::add_recorded_clip` seeds a new clip** from the two preferences,
   beside `show_pip: self.preferences.pip_for_new_recordings` — that is the core
   function holding the line; `finish_recording` is only its bus caller.
7. **`layout::inset_span(out_w, placement) -> (x, w)`** replacing `pip_left` —
   `(x, w)` rather than `(left, right)` so `pip_rect` needs no arithmetic and the
   two `f64`s are not interchangeable. Carry `pip_left`'s "one function, two
   readers" doc onto it: that invariant is the point. **`pip_left` has to change
   signature regardless** — it cannot express a left corner — so this widens one
   function rather than adding an abstraction.
8. **The ratio table lives in `layout.rs`**, not on the enum:
   `fn inset_ratio(size) -> f64` beside `BAR_HEIGHT_RATIO`. `layout.rs` owns every
   ratio in the app with a doc comment each, and the precedent the enum's *shape*
   comes from argues the same way about its *meaning* — `Resolution` is a bare
   stored enum whose pixels live in the consumer. **Delete `PIP_WIDTH_RATIO`**
   (four readers; one of them, `overlay.rs`'s test, re-derives `pip_left` by hand
   as `1280.0 * (1.0 - PIP_WIDTH_RATIO)` — the exact drift a kept constant
   enables).
9. **`pip_rect`** takes the placement and reads `inset_span`; `y` is the top or
   bottom row by corner; flush in every corner.
10. **`bar_rect`** takes `Option<InsetPlacement>` and reads the **same**
    `inset_span`: bottom-right → `0 .. x`; bottom-left → `x + w .. out_w`;
    top-right → full width. Its doc says `None` is "no inset drawn" (trap 3), and
    keeps its existing reasoning about the argument being the coach's stored intent
    rather than a pipeline observation — now more true, not less.
11. **Delete `avatar_box`.** It is only applied to a square `pip_rect`, and
    shrinking a corner-flush square about its flush corner *is* a corner-flush
    square at `AVATAR_BOX_RATIO ×` the ratio. **This is the fix, not a tidy-up:**
    left as a rect operation, a bottom-left avatar drifts **105.6px** off the left
    edge and a top-right one hangs **105.6px** below the top edge at Medium/1080p —
    two of the three new corners, not one, and the *same* number on both axes
    because the avatar's box is square (aspect 1.0). (A draft said 118.8 for the
    top; that is `0.22·1080·0.5`, which corresponds to nothing on this path.) Keep `AVATAR_BOX_RATIO` and its reasoning,
    now as a ratio on a ratio. The avatar path asks for the inset rect at
    `AVATAR_BOX_RATIO * inset_ratio(size)` with aspect 1.0.
12. **`pip_rect_over_picture`, `self_view_rect`, `avatar_self_view_rect`** take the
    placement through. Their comment about a non-16:9 picture and the inset
    reaching into where the export's bars would be stays true and mirrors in every
    corner.

**Core's tests, in this task:**
- **One golden rect, then a property loop** — not a nine-row table. `tests/layout.rs`
  already has that shape. Keep exactly one exact rect (Medium + BottomRight at
  1080p, today's values, the pixel-identity pin), then loop 3×3 asserting
  `w == ratio·out_w`, `x == 0` or `x + w == out_w` by corner, `y == 0` or
  `y + h == out_h` by corner, and `h == w / aspect`. That pins the *intent* and
  cannot rot into "retune the expected numbers".
- **The bar meets the inset**, asserted against `inset_span` in one 3×3 loop, plus
  full width for top-right and for `None`.
- **Extend the existing `pip_rect_over_picture` identities** across placements.
- **`avatar_box`'s own tests must be rewritten, not epsilon'd.** A draft of this
  plan said `tests/avatar.rs` had exact-equality assertions needing an epsilon;
  that was wrong twice. Those tests already use `< 1e-12`, and their `pip()`
  helper is a hardcoded `Rect` that composes no ratios — but `avatar_box` is
  **deleted**, so `the_avatar_box_keeps_the_insets_corner_and_only_shrinks` and
  `the_avatar_never_reaches_past_its_box` have to be rewritten against the folded
  ratio or removed. `tests/layout.rs::an_avatar_take_is_placed_in_the_smaller_box`
  also calls it. The fold's own error is harmless and measured: `x` and `y` are
  **bit-identical**, `w`/`h` differ by ≤1.14e-13, and every case rounds and ceils
  to the same integer — so the pixmap size and the mixer's rect are unchanged.
- **The v12-loads test written to the existing pattern is exactly the test that
  bites**, and a draft of this plan was wrong to say otherwise. `a_v10_…` and
  `a_v11_…` each end by asserting a `project.json.v<old>` backup exists, and
  `store::write` only keeps a backup of a file *older* than the version it writes —
  so at `CURRENT_FORMAT_VERSION = 12` the v12 test fails on that line. **Proven:**
  reverting the constant to 12 failed exactly
  `a_v12_file_loads_under_the_current_version` on
  `project.json.v12 … .exists()`. No extra assertion grafted onto
  `an_upgrade_keeps_the_old_file_once` is needed; it would be a second copy. And
  there was no v12 test at all — slates skipped it — so this closes a real gap.
- **The preferences seed a new clip**: a clip edited to Large/BottomLeft leaves the
  next `finish_recording` at Large/BottomLeft.

## B2. Media: per-entry placement, and the bug only a run can show

**Files:** `crates/pundit-media/src/{overlay.rs, composite/avatar.rs,
composite/export.rs, composite/preview.rs, composite/mod.rs}` and
`crates/pundit-media/tests/{export.rs, preview.rs, avatar.rs}`.
**`composite/avatar.rs` is where items 3–5 actually bite**: `avatar::open` computes
the box, and its doc ("shrunk about its bottom-right corner — keeping the inset's
own right and bottom margins") becomes false and needs **replacing**, like
`overlay.rs`'s board comment. Both `AvatarInset`s go through it.
**Also fix `crates/pundit-harness/src/lib.rs`'s `Clip` literal**, which B1 breaks
and which no task owned.

**Expect ~14 call sites in media's own tests** (`tests/export.rs` alone has ten) on
top of the source changes. B1 left them red; that is this task's work.

1. **`overlay.rs`** passes `frame.clip.and_then(Clip::inset_placement)` to
   `bar_rect` — trap 3, the one line to get right, and `None` now comes from the
   accessor rather than a `filter` here. Good news, verified: `overlay.rs` hands the
   whole `bar` rect to `fill` and to `Label { rect: bar, … }` and never assumes
   `bar.x == 0`, so a moved left edge needs **no drawing change**.
2. **`overlay.rs`'s falsified decision comment** — *"The board cannot reach it: it
   is 0.36 of the width from the left edge and the inset starts at 0.78"* — is
   **replaced, not retuned**: bottom-left is separated from the board
   **vertically**, top-right **horizontally**. Word the vertical one as holding
   for every camera aspect the app has seen, not as a proof: a portrait camera
   (below ≈0.58) at Large would reach the board's rows, where the board simply
   draws on top — degraded, not corrupt, and not worth code.
3. **`Pip::open` computes the avatar's rect** from the clip's placement, as the
   camera path already does per entry. `Layout::pip` is already per-entry; it is
   the avatar path that bypassed it.
4. **Key the avatar texture cache on `(PathBuf, InsetSize)`.** `export.rs` already
   holds `HashMap<PathBuf, Option<AvatarInset>>`, so this is a key change, and
   every texture is then `ceil()`ed to **its own** box — keeping "the drawn box is
   never short of the rect it stands for" true for every size, not just the
   largest. It also removes an ambiguity a draft had ("the run's largest" — global,
   or per image path? the basket has several) and a silent GPU downscale of a Small
   entry's texture built at Large. Cost: up to three still-decodes of one local
   image per run, against an export of an hour of video.
5. **Preview has its *own* `AvatarInset`** (`composite/preview.rs`), calling the
   same `avatar::open_reported` and using `avatar.rect` for the pad and the image's
   size for the appsrc caps. It reads `job.clip`'s placement — a `PreviewJob` is
   one clip, so it needs none of the cache reasoning. **If it is left on the
   default while export takes the clip's, a Large clip previews at Medium** — the
   preview/export drift trap 3 exists to prevent.

**Its test, in this task:** a run of **mixed avatar sizes gives each entry its own
rect**. This is the bug the spec found and nothing else catches. **Prove it fails
first** against the pre-change behaviour (one rect for the run) and say what you
saw.

## B3. App: the commands, the write-back, the controls, the self-view

**Files:** `crates/pundit-app/src/main.rs`, `crates/pundit-app/src/bus/`,
`crates/pundit-app/ui/app.slint`.

1. **Two commands** on the `on_set_show_pip` template. **Each also writes
   `Preferences::last_inset_size` / `last_inset_corner`** — that write-back is what
   makes the next recording inherit the choice, and `bus/export.rs`'s
   `last_export_resolution` write is the pattern. **It goes inside
   `Bus::edit_clip`, before `self.save()`**: after it, the preference is lost until
   some unrelated save. That function's existing `if before == edit { return; }`
   guard correctly means a no-op edit writes no preference.
   **Undo does not revert the preference, and that is right** — `last_export_*` sit
   outside the undo history for the same reason: a sticky last-used value is not
   part of the document's meaning. Said here so a reviewer does not file it.
2. **Two declared Slint enums** (Slint cannot take a Rust enum; `ClipField` and
   `ScanStep` are the precedent). Not the `int`-index pattern — a stale index is
   the hazard `state.json`'s string labels exist to avoid.
3. **Two controls in the Inspector**, beside "Show avatar in export", as **one
   `HorizontalLayout` with `min-width: 0; horizontal-stretch: 1` on both
   `ComboBox`es.** That makes overflow **impossible by construction** rather than
   something to measure: the column is 280px with a documented scar (its transcript
   caption row records a fourth control running 76px past it at the window's
   minimum), and the app already uses `min-width: 0` for exactly this. Worst case
   is a clipped label, visible the moment anyone opens the app — not a control off
   the edge. **Vertical space is no longer scarce**: the inspector column went
   inside a `ScrollView` in the panels work (0.10.0), so extra rows scroll and
   nothing has to give up pixels. A draft of this plan said otherwise.
4. **The live self-view reads the two preferences** (spec I6) — and this is real
   wiring, not argument threading: `on_place_self_view` is a *pure* Slint callback
   with no captures, so it needs two new `in property`s on the window, the two
   enum declarations from 2, a callback signature change, the binding site, and a
   feed in `show_project`. Five sites. If any of it slips, say so rather than
   leaving the self-view silently at Medium/BottomRight — a coach who set
   Large/BottomLeft and sees the self-view bottom-right will file it as a bug.

**Gate:** the workspace, clippy (to a log, `$?` checked), fmt.

## B4. Close out

- The `verify` skill: fmt, clippy, tests, the core dependency audit.
- **The acceptance gate is already committed**, not a separate step: B1's one
  golden rect pins Medium + BottomRight to today's values, and B2's mixed-size test
  pins the per-entry rect.
- Adversarial review on the diff, then commit.
- `CLAUDE.md`: fold the placement into the export/inset material — including that
  the bar's edge and the inset's come from one function, and why that argument is a
  placement and not a rect.
- `BACKLOG.md`: #88 resolved, recording the two corrections it needed (v12→v13, and
  the per-run avatar rect its own list was missing).
- `CHANGELOG.md`: an `## [Unreleased]` entry written for a coach — what changes in
  their hands, not the mechanism.

---

## What this plan deliberately does not do

- **Top-left.** The scoreboard is locked there and is drawn over the inset, so it
  would be a half-hidden face.
- **A slider.** Unguessable without a live composite preview, which the app has
  not got.
- **A UI for `pip_for_new_recordings`.** It has none today; #78 is where it and
  the two new preferences would get one.
- **Touching `shows_camera_pip` / `shows_avatar` / `shows_inset`.** They answer
  *whether*; this is *where* and *how big*.

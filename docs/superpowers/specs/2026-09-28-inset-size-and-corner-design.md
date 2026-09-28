# The inset's size and corner, per clip (#88) — design

The coach (2026-09-25): *"avatar sizing and position should be settable per clip
i think?"*

Today both are fixed: `layout::PIP_WIDTH_RATIO` is 0.22 of the output width and
`layout::pip_rect` is flush into the **bottom-right** corner. A clip records only
*whether* it shows an inset (`Clip::show_pip`) and *which kind* (`Clip::inset`).

**Two corrections to BACKLOG #88 before anything:**

1. It says this is *"a `formatVersion` bump to 12"*. **v12 is taken** — slates
   took it (`store.rs`: `CURRENT_FORMAT_VERSION = 12`). **This is v13.** An
   implementation trusting the entry would write a version every 0.9.0 project
   already claims.
2. Its list of what must follow is missing the item most likely to ship a bug:
   **the export builds one avatar rect for the whole run** (I5).

## I1. The size is three named steps

| step | ratio of output width | bar keeps |
|---|---|---|
| Small | 0.16 | 0.84 |
| **Medium (default)** | **0.22** | 0.78 |
| Large | 0.30 | 0.70 |

Medium is exactly today's `PIP_WIDTH_RATIO`, which is what makes every existing
clip render to the pixel (I8).

**Why steps and not a slider — the honest reason.** A first draft argued that a
free ratio would make the caption bar jitter between clips. That argument is
wrong and would have been caught by anyone who checked: steps jitter too (the bar
is 0.84 / 0.78 / 0.70), and a per-clip *corner* moves the bar's whole edge, a far
larger discontinuity than any width. Taken seriously it argues against #88
itself.

The real reasons: a continuous ratio would be **unguessable**, because this app
has no live preview of the composite — a coach dragging a slider is guessing
until they export — and three steps can each be **measured** against the longest
line the app produces, where a slider cannot. At 1080p the bar's font is
`0.5 × 0.08 × 1080` = 43.2px with a 13px pad, so the usable width is ≈1613 /
1497 / 1318px: about **74 / 68 / 61 characters** at a 0.5em average advance.
Large costs roughly seven characters against today, on a bar that ellipsizes
rather than shrinks. The basket's three-part bar (`<match> | <clip> | tags`) is
the case to shape-check, not a reason to drop Large.

**Where the ratios live:** `InsetSize::ratio()`, and **`PIP_WIDTH_RATIO` goes**.
Leaving it as "Medium's value" would be a second source of truth. Note
`overlay.rs`'s test re-derives `pip_left` by hand as
`1280.0 * (1.0 - PIP_WIDTH_RATIO)`; it must ask `layout` instead.

## I2. Three corners, and top-right is the cheap one

Bottom-right (default, today's), bottom-left, top-right. **Top-left is not
offered**: the scoreboard is locked into it (the coach, 2026-09-25), and the
export's z-order puts the scoreboard **over** the inset (verified: picture pad z
0, inset z 1, overlay including the board z 2). So a top-left inset would not
corrupt anything — it would just be drawn under the board, and a coach choosing
it gets a half-hidden face and has to work out why. The app's live board is drawn
after the self-view too, so app and export agree.

Rejected variation: offering top-left only when no scoreboard is configured. A
control that appears based on a setting two panels away is worse than one that is
simply absent.

**The costs are the opposite way round from what you would guess**, and it
matters for staging:

- **Top-right is nearly free.** It reuses `bar_rect`'s existing
  `has_inset == false` branch — a full-width bar is already a shipped, tested
  state. Measured clear of the board (the board's stoppage tail ends at x=872.9
  of 1920; the widest top-right inset starts at x=1344) and clear of the app's
  only top furniture, the centred `ZoomIndicator`.
- **Bottom-left is the expensive one.** It is the only genuinely new bar
  arithmetic, and it puts a 60%-black strip that no longer touches the left edge
  behind text still aligned left from mid-frame — a look nobody has seen, in the
  component whose recent history is exactly "flush, not an accidental gap".

If scope is ever cut, cut **bottom-left**, not top-right. Keeping it costs the
one new visual state; it also serves the real case ("the action is on the right,
put my face on the left"), which is why it is in.

## I3. The bar takes the *placement*, never a rect

`pip_left` generalises into one aspect-free span both readers share:

```rust
fn inset_span(out_w: f64, size: InsetSize, corner: InsetCorner) -> (f64, f64)
```

`pip_rect` reads it for x and width; `bar_rect` reads it to cut itself:

- inset **bottom-right** → bar spans `0 .. inset_left` (today).
- inset **bottom-left** → bar spans `inset_right .. out_w`. This *moves* the
  bar's edge rather than widening it — a bar starting at 0 would run under the
  inset, the accident "flush, not inset" removed.
- inset **top-right** → bar spans the full width; nothing is in the bottom row.

**`bar_rect` must not take the inset's `Rect`, and a first draft's proposal to do
that was impossible.** Two independent reasons, both verified:

- **Preview cannot supply one.** The `OverlayFrame` is built in the pump, and the
  inset's rect is only computed later, in `place_pip`'s **caps probe on a
  GStreamer thread**. That is precisely the hazard `bar_rect`'s own doc names:
  preview learns the camera's shape "long after the bar is laid out, so anything
  finer would have preview and export cut the same caption differently".
- **Export would supply a wrong one.** When the probe fails it falls back to
  `Pip::filler()` — a **1×1** rect. Passing the real rect there would give a
  full-width bar for a clip whose camera died in export while preview cut it
  short: the exact drift that doc forbids.

Size and corner are **stored fields**, known before either pipeline starts, so
they carry none of that. A rect's *height* needs `cam_aspect`; the bar only ever
needed a horizontal span. That distinction is the whole of this section.

## I4. The format: v13, two fields, the `Inset` precedent

```rust
#[serde(default)] pub inset_size: InsetSize,      // Small | Medium | Large
#[serde(default)] pub inset_corner: InsetCorner,  // BottomRight | BottomLeft | TopRight
```

Both follow `Inset` exactly: `#[derive(…, Default, Serialize, Deserialize)]`,
`#[serde(rename_all = "camelCase")]`, a `#[default]` variant, field-level
`#[serde(default)]` on the `Clip` field. `project.rs`'s header allows that for
"any type whose `Default` is what an older file means, which is why `Inset`
defaults to `Camera`" — this is the permitted side of the rule, where an `f64`
would be the forbidden side. `Quality` is a second precedent for the exact shape
`InsetSize` wants.

`MIN_READABLE_FORMAT_VERSION` stays 7; both fields are additive. The bump brings
its test, named by the convention already in `project_format.rs`: **a v12 file
loads under the current version**.

A `Clip::inset_placement() -> (InsetSize, InsetCorner)` accessor, in the spirit
of `shows_inset`, so geometry takes one value rather than two arguments.

## I5. What has to follow — including the one that would have shipped a bug

- **The export's avatar rect is built once per *run*, and must become per entry.**
  `AvatarInset::open(path, out_w, out_h)` computes `rect = avatar_box(pip_rect(…))`
  **once** and hands that single rect to every avatar entry, while the camera
  path computes its rect per entry from the probe. So a run mixing a Small
  bottom-left avatar clip with a Large bottom-right one would draw **both in one
  place**, silently, with no test failing — I8's pixel-identity test passes,
  because single-size projects are unaffected. The rect moves into `Pip::open`,
  which already holds the `clip`.
  **The texture is sized to the run's largest avatar box**, not to Large
  unconditionally: the pixmap is deliberately `ceil()`ed so "the drawn box is
  never short of the rect it stands for", and always building at Large would
  change the pixels of every existing Medium-only project and break I8.
- **`avatar_box` folds into the ratio and stops being corner arithmetic.** It is
  only ever applied to a **square** `pip_rect`, and shrinking a corner-flush
  square about the corner it is flush in *is* a corner-flush square at
  `AVATAR_BOX_RATIO ×` the ratio — verified identical to the pixel
  (x=1603.200, w=316.800 both ways at Medium/1080p). So the avatar path passes
  `AVATAR_BOX_RATIO * size.ratio()` with aspect 1.0 and `avatar_box(pip: Rect)`
  is deleted. Left as a rect operation it is a real bug: unfixed, a bottom-left
  avatar drifts **105.6px** off the left edge at Medium/1080p. `avatar_rect`'s
  pulse is concentric about the box and is unaffected in every corner.
  Watch one detail: multiplying ratios before the width can differ from the
  current order by a ULP, so `tests/avatar.rs`'s exact-equality assertions want
  an epsilon.
- **`overlay.rs`'s decision comment is falsified**: *"The board cannot reach it:
  it is 0.36 of the width from the left edge and the inset starts at 0.78."* The
  inset can now start at 0.70, at 0.0, or be in the top row. The separation still
  holds everywhere, but for different reasons: bottom-left is separated
  **vertically** (the board occupies `y ∈ 0..0.08·H`; the tallest offered inset's
  top edge is at y=504 of 1080), top-right **horizontally**. Replace the
  reasoning, do not just retune the number.
- **The GL 1×1 filler** needs nothing: it feeds a pad whose rect is set in the
  PTS-keyed probe.
- **`shows_camera_pip` / `shows_avatar` / `shows_inset`** are untouched — they
  answer *whether*, and a reel entry (`clip: None`) correctly still reads as no
  inset and gets a full-width bar.

## I6. Both readings, from one sticky preference

The coach, asked whether they wanted to choose this before recording or fix it
afterwards: **"can pick before or fix after, either."** Both, then — and there is
a pattern in the same struct that gives both for the price of one.

**There is no clip during a take.** `finish_recording` builds the `Clip`
*afterwards* and takes the sibling field from a preference
(`show_pip: self.preferences.pip_for_new_recordings`). So "pick before" cannot be
a per-clip field by definition, and a first draft's claim that the live self-view
should follow the clip's size and corner was unimplementable.

**But `pip_for_new_recordings` is the wrong precedent to copy: it has no UI at
all.** It is a format field with a `true` default, written only by tests — the
coach cannot reach it. Adding `inset_*_for_new_recordings` beside it would add two
more inert fields and leave "pick before" no more reachable than it is today.

**The right precedent is three fields further up the same struct.**
`last_export_resolution`, `last_export_quality` and `last_export_scoreboard` are
**sticky last-used** values: written back when the coach picks one
(`bus/export.rs`), read to seed the next sheet. Applied here:

- `Clip::inset_size` / `inset_corner` (v13) — **"fix after"**: edit any clip in
  the Inspector, check it in Preview.
- `Preferences::last_inset_size` / `last_inset_corner` — written back whenever
  the coach changes a clip's, and read by `finish_recording` to seed the next
  clip. **"Pick before"**: set it once on any clip and every later recording
  inherits it.

One mechanism, both readings, an established naming and write-back pattern, **no
new UI beyond I7's two controls**, and no dependency on #78 (app settings) — which
is where `pip_for_new_recordings` would finally get a control, and where these two
would then appear for free.

**The live self-view comes back into scope, cheaply**: it reads the two
*preferences* — the values the next clip will get — so it honestly shows where
the inset will land, without needing a clip that does not exist. That is
`pip_rect_over_picture` and the two `self_view_rect` helpers taking a size and a
corner, which is a plain argument-threading change, not the impossible one the
draft implied.

**Two different serde rules in one change, and getting them the wrong way round
is a bug.** The `Clip` fields take a **field-level** `#[serde(default)]` (I4).
The `Preferences` fields take **none at all**: that container already carries
`#[serde(default)]` and fills from its hand-written `Default` impl, so a
field-level one would be a second copy of the default — the rule `CLAUDE.md`
states for `last_export_scoreboard`.

## I7. The surface the coach actually touches

Absent from the draft entirely, and it is more new code than the geometry:

- **Two `ClipEdit` variants** and their arms in `Clip::set`, "the one definition
  of a field change". These are undoable, like `ShowPip`; anything else would be
  inconsistent with the control sitting beside them.
- **Two commands**, on the `on_set_show_pip` template.
- **Slint cannot take a Rust enum**, so either a declared `enum` in `app.slint`
  (precedent: `ClipField`, `ScanStep`) or the `int` index pattern the export sheet
  uses (`resolution_index` / `resolution_at`). Pick the declared enum: the index
  pattern exists because a `ComboBox` needs indices, and a stale index is the
  hazard `state.json`'s string labels were chosen to avoid.
- **Two controls in the Inspector, beside "Show avatar in export"** — and the
  width must be **checked at the window's 1100×700 minimum**, not assumed. That
  column is 280px and has a scar: its transcript caption row records
  `[Transcript] [model] [Transcribe] [Cancel]` running **76px past** it at that
  size. A captioned row of two combos is ≈252px of the 256px available — inside
  by 4px, which is the margin that failed last time. The scarce axis is also
  **vertical** ("this column is 280px of a window that may be only 700px tall",
  with `notes` already floored at 64px), so the plan names which control gives up
  the pixels.

## I8. What must not change

- **Existing clips render identically** — Medium is `PIP_WIDTH_RATIO` and
  BottomRight is today's corner. This is the acceptance test, not a hope, and I5's
  texture-sizing rule exists to keep it true.
- **The inset stays flush** in every corner: the margin-free decision was about
  the strip of dead picture a margin left, which is corner-independent.
- **One `Zoom::transform`** and the content rect: this is output-space chrome.
- **The scoreboard's corner**, locked — which is what costs us top-left.

## I9. Tests

- **Every step and corner against a golden rect**, in `pundit-core` (no
  GStreamer). Medium + BottomRight must equal today's `pip_rect` exactly.
- **The bar meets the inset**, asserted against the shared `inset_span` rather
  than a recomputed ratio, so the invariant is what is under test: adjacent with
  no gap or overlap in both bottom corners, full width for top-right.
- **A run of mixed avatar sizes gives each entry its own rect** — I5's bug, which
  nothing else would catch.
- **v13 loads every readable version**, as "a v12 file loads under the current
  version".
- **Extend `tests/layout.rs`'s existing `pip_rect_over_picture` identities**
  across the sizes and corners rather than adding a harness test: the property is
  a pure `f64` identity core already pins at 16:9 and both off-aspect cases, and a
  bus round trip would only re-prove it.

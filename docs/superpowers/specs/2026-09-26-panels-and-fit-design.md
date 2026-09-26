# Resizable panels, and Fit window to video — design

Backlog **#87** (resizable panels) and **#95** (the player area at the
footage's aspect). One spec, **two shipments**: #95 first and alone, #87 after.

What the coach asked for, in their words:

- "resizing all the panels" (2026-09-25).
- "i'd like the preview window to be the right aspect ratio? i don't like the
  black bars in the view" (2026-09-25, on 0.8.0) — and, given the choice
  between an automatic resize and an action, **"Fit window to video"**: the
  window moves when asked and never on its own.

**They need nothing from each other.** #95 reads the player area's size off the
window (W2); #87 changes what that size is. That is the whole of the coupling,
and it is why the fit must *read* the area rather than derive it from the column
widths. Entry #87 claimed #95 needed it ("panels that can use the slack") and
#95's entry already records that as measurably wrong — the slack is vertical and
the panels are horizontal. So #95 ships first: it is one pure function, one key,
one button and one notice.

## W0. The measurements this is built on, and the one that proved nothing

Taken 2026-09-26 against the real `AppWindow` on `i-slint-backend-testing`,
with `place-picture` wired to the production `zoom_input::Viewport` (an
unwired callback returns `PictureRect::default()`, which is how a careless
measurement here reports a content rect of 0×0 and concludes nothing):

| window | player area | content rect | chrome |
|---|---|---|---|
| 1600×960 | 1080×852 | 1080×607.5 | 520 × 108 |
| 1100×700 | 580×592 | — | 520 × 108 |
| 1920×1080 | 1400×972 | — | 520 × 108 |
| 1600×715.5 | 1080×607.5 | 1080×607.5 | 520 × 108 |
| 1600×715 | 1080×607 | **1079.1×607** | 520 × 108 |

1. **The chrome is 520 × 108 — at and above the window's 700px floor.** 520 is
   the two columns (240 + 280), 108 the transport bar. Below the floor it is
   not constant (measured: `1100×400` → player `580×334`, so 66), which the
   headless backend will happily let a test reach. With #87 the horizontal half
   becomes whatever the columns and splitters are; the vertical half stays put.
2. **The exact target is fractional, and the rounding is load-bearing.** A 16:9
   fit at 1600 wide wants `1080 / (16/9) + 108` = **715.5**. The last two rows
   are why this spec says *ceil*: at 716 the picture is still width-limited and
   unchanged; at 715 it becomes height-limited and the picture **shrinks by
   ~1.9px**, which is the one thing W1 promises won't happen.
3. **`min-width` / `min-height` are not a floor a test can see, and not one the
   app can read.** The headless backend's `set_size` dispatches the resize
   unconditionally and consults layout constraints only when the current size is
   zero (`testing_backend.rs:512-531`), so it honours 200×100 and any fractional
   size, with or without the declarations. On the **real** path the declared
   pair *replaces* the content-derived minimum
   (`lower_layout_expression.rs:1999-2030`) and is published to winit as
   `set_min_inner_size` (`winitwindowadapter.rs:2008-2010`), i.e. as
   `WM_NORMAL_HINTS` — which a Mutter-family WM applies to a client resize
   request, not only to a user drag. **So a fit below the floor does not fail
   cleanly: the WM clamps it, and the coach is left with bars *and* a moved
   window.** W6's refusal must therefore be an explicit check in the app, and
   W6 says so.

(An earlier draft of this spec concluded from the headless backend that "Slint
enforces no intrinsic minimum" and that the player would be crushed to zero
width. That was an artefact of the backend, and it is recorded here so nobody
measures window constraints that way again.)

## W1. Fit never re-fits the picture. It closes the window up around it.

This is the design; the rest follows from it.

The bars exist because the player area's aspect is not the footage's, and
`place-picture` letterboxes the frame inside whatever shape it is given. Two
ways to make the two aspects agree: **shrink** the dimension with the slack, or
**grow** the other one. Fit shrinks.

At 1600×960 with 16:9 footage the picture is drawn 1080×607.5 with 122px of
black above and below. Fit sets the window to 1600×716, and the picture is
still drawn **1080×607.5, at exactly the same size** — nothing is re-fitted,
nothing is rescaled, and the only change is that the black is gone. (It does
move *up*, by the bar that went away: the player is at the top of the window
and every WM anchors a resize at the top-left. The picture's size is the
promise; its position on screen is not.)

Growing instead (1600×960 → 2120×960) would make the picture **bigger**. That
is a different request — "give me a bigger picture" — and dragging the window
is already how it is asked for. It is also the only version of this action that
could push the window off a screen whose size the app cannot read (W6).

> **Fit = shrink the dimension that has the slack. Never grow, never move.**

- Player too **tall** (bars above and below — the coach's case): the height
  becomes `player_w / aspect + chrome_h`.
- Player too **wide** (bars left and right — a 4:3 source, or a short window):
  the width becomes `player_h × aspect + chrome_w`.
- Already at the footage's aspect, within a pixel: nothing happens, and nothing
  is said. The action is idempotent, which is what makes a stray second press
  harmless.

**The target is an integer number of physical pixels, rounded up.** Measurement
2 is the reason, and physical rather than logical because the two conversions
disagree: `PhysicalSize::from_logical` truncates (`api.rs:190-195`) where
winit's path rounds (`dpi-0.1.2:97-100`), and at a fractional scale factor a
logical ceil is not enough to keep the picture width-limited.

## W2. The chrome is read off the window, never computed

`chrome_h = window_h − player_h` and `chrome_w = window_w − player_w`, from
`player-width` / `player-height`, which `AppWindow` already publishes
(`app.slint:2809-2810`).

Deriving it instead — 240 + 280, plus a transport bar of 108 — would be a
fourth copy of the layout's numbers, wrong the first time a button joins the
transport row or a panel is dragged. The subtraction cannot be wrong: whatever
is not the player **is** the chrome. It is also what lets #95 ship before #87
and stay correct afterwards.

Nothing in the spec may hard-code 520, 108, 716 or a threshold derived from
them — not the code and not the tests. Measurement 1 already shows two of those
numbers changing.

## W3. Which aspect, when the action is offered, and where it is offered from

**The aspect is the displayed frame's** — `frame-width / frame-height`, the same
pair `place-picture` letterboxes against, set by `video.rs` from the decoded
frame's **display** size with the pixel aspect applied (`video.rs:157-162`). Not
`SourceRef::display_aspect`: a project may hold sources of different shapes, and
the bars on screen belong to the frame on screen. Reading the same numbers the
letterbox is computed from is what makes "the bars are gone afterwards" true by
construction rather than by agreement.

**Offered when, and only when, there is slack to remove**, which is:
`can-play` (`app.slint:2848` — sources, none missing) **and** a player whose
aspect differs from the frame's by more than a pixel's worth.

It is **not** `frame-width > 0`. Those properties are declared
`in property <float> frame-width: 16;` / `frame-height: 9;` and `video.rs` only
ever sets them — never clears them on close — so that test is **always true**,
including with no project open, and would put the action over the empty-project
card.

It is **not** gated on recording or previewing. Resizing the window during a
take touches neither the camera nor the recording, and a preview is exactly
when a coach is looking hard at the picture.

### The two entry points

- **`f`**, in `handle-key`'s letter branches, behind the `text-editing` fold at
  `app.slint:3529` that every letter is behind. `f` is free: the letters in use
  are a, c, d, h, i, j, l, o, r, v, x, z, plus Ctrl+o/0/z/y. It is one more row
  for #96's rebinding table.
- **A "Fit" button in the transport's *second* row** — the drawing/status row at
  `app.slint:4795` — with a `Tooltip` naming the key, the row idiom already in
  use (`"Clear the drawings (C)"`, `"Speed while playing (J slower, L faster)"`).
  **`enabled`-gated, never `visible`-gated**: a button appearing and vanishing as
  the window is dragged would reflow the row under the coach's cursor. A
  disabled button keeps its `Tooltip`, which is how the Record button explains
  its own greying.
  - **Not the first row, and this is measured.** At a 1100px window that row is
    exactly full: the seven buttons run to x=1092 of a 1084px content width, and
    the only give is the readout (`horizontal-stretch: 1`, `min-width: 130px`,
    laid out at ~204px). A ~50px button fits only by taking ~50px from the
    readout, which **clips rather than ellipsizes** — and the readout is the
    position display. `app.slint:4797-4800` records that this row already
    overflowed once, which is why a second row exists at all. The second row's
    unconditional content is about 300px of that 1084, so the button is free
    there; it costs the notice line some width, and the plan checks the notice
    still reads at 1100px.
  - The rejected alternative was a `ZoomIndicator`-styled pill in the player's
    **bottom-left**. It is prettier — it would sit *in* the black bar it removes
    — but it costs a component, it is the only affordance in the app that would
    live over the picture, it flickers in and out as the window is dragged, and
    it must be declared **after** `zoom-area` and the two content-rect
    `TouchArea`s (`:4374`, `:4420`) or the press never reaches it.

## W4. The panel widths (#87)

Two properties on `AppWindow`, logical pixels, plus the minima and the window's
own floor as `out` properties — because **Slint's `min-width`/`min-height` are
reserved layout properties with no generated getter**, and W6 needs the floor in
Rust. One literal each, in the `.slint`, where every other layout number lives:

```slint
out property <length> min-window-width: 1100px;
out property <length> min-window-height: 700px;
min-width: root.min-window-width;
min-height: root.min-window-height;

out property <length> sidebar-min: 240px;
out property <length> inspector-min: 280px;
out property <length> player-min: 320px;

in-out property <length> sidebar-width: root.sidebar-min;
in-out property <length> inspector-width: root.inspector-min;
```

**The bound is expressed as layout constraints, not as a `clamp` on `width`.**
Each column states what it wants and what it will accept, and the layout is
what reconciles them:

```slint
min-width: root.sidebar-min;
preferred-width: max(root.sidebar-min, root.sidebar-width);
max-width: max(root.sidebar-min, root.sidebar-width);
horizontal-stretch: 0;
```

with the mirror for the inspector, and `min-width: root.player-min;
horizontal-stretch: 1;` on the player. Three things about that:

- **It is declarative, so it is unconditional.** A width restored from a wider
  screen, a window dragged narrow, a hand-edited state file — all read as "as
  wide as there is room for". Nothing is fixed up at startup, and the drag (W5)
  writes the raw property and lets the layout bound it, so the bound lives in
  exactly one place.
- **The obvious alternative is measurably wrong.** A
  `width: clamp(stored, own-min, root.width − other − player-min)` on each
  column has each one computing its headroom from the other's *raw* property,
  and neither knows about the two 6px splitters W5 puts in the same layout.
  Measured at a 1100px window with the two splitters present, stored
  `(sidebar 100, inspector 700)`:

  | | sidebar | inspector | player |
  |---|---|---|---|
  | `clamp` on `width` | 240 | 540 | **308 — under its minimum** |
  | layout constraints | 240 | 528 | **320** |

  and at `(400, 400)`: 380/380/328 against 384/384/**320**. At the defaults the
  two are identical (240/280/568). The layout form holds the player's minimum in
  every case, needs no `max(other, other-min)` term to do it, and accounts for
  the splitters without being told — which is W2's rule applied to the columns.
- **It also cannot close a binding loop**, because it never reads `root.width`.
  The `clamp` form does, from inside a child whose width feeds the layout that
  determines `root.width` — and Slint deprecates that with a warning that
  `-D warnings` would fail on. I could **not** reproduce the warning in a
  scratch window, so treat the loop as unproven rather than as the reason; the
  308 above is reason enough.

**The minima are today's widths: the panels grow, they do not shrink.** Not
timidity — measured: the inspector's transcript row is
`[Transcript] [model] [Transcribe]` and `app.slint:694` records that a fourth
control there "runs 76px past this 280px column at the window's 1100x700
minimum". 280 is the width that row was fitted to; 240 is the same kind of
number for the Clips rows. A narrower panel means re-verifying every row in both
columns against a width nobody asked for, and the coach wants *more* room on a
big screen, which is the direction this allows. `player-min` of 320px is below
the 568 the player gets at the window's own minimum once the splitters are in
(520 of columns + 12 of splitters), so it is slack rather than a constraint —
and it is what the layout reconciles against when the stored widths do not fit.

## W5. The drag (#87)

A 6px `Splitter` **inside the `HorizontalLayout`**, between each column and the
player: transparent until hovered, `mouse-cursor: ew-resize`, and a `TouchArea`
that does the arithmetic **in window coordinates**. Being in the layout means
they are part of the chrome — `chrome_w` becomes 532 — which is exactly why W2
subtracts instead of adding up column widths.

Two rules, and both are bugs if broken:

- **Anchor on the press, in absolute coordinates.** On press, record
  `self.absolute-position.x + self.mouse-x` and the column's width; while moved,
  `width = start + (absolute pointer x − anchor)`. The pointer's absolute x is
  invariant as the splitter moves under the drag. The incremental form
  (`width += mouse-x - pressed-x`) is what Slint's own
  `widgets/material/tableview.slint` does, and it **sticks at the clamp**: each
  `moved` adds an increment that the clamped consumer never cancels, so the raw
  property runs away from the pointer and a drag back does nothing until it
  returns.
- **`start` is the column's laid-out width, not the raw property.** If a drag
  begins while the clamp is active, the raw value is off in the distance and the
  column will not move until it comes back into range — the same stick, in a
  second shape.

On release the property is set to the column's **effective** (clamped) width and
that is what is stored. Storing the raw value would put a width the coach never
saw into `state.json`, give the next drag a dead zone, and hand W4 the only
out-of-range input its clamp pair is not safe for.

The release callback takes **no argument** — Rust reads both properties. The
width is written to `state.json` on release and nowhere else: a drag is 30
events a second and `AppFiles` rewrites the whole document per setter. Unlike
the window size, which the app can only learn on the way out
(`closing_window_size`), a panel width is known the instant the coach lets go,
and a release is the last thing that can happen to it — so there is no on-close
write to add.

Nothing goes through the bus. A panel width is not project data, not undoable
and not the bus's business; it is the window's own geometry, like the window
size, which `main.rs` already reads and writes directly.

## W6. When the fit cannot be done: the floor, and the maximised window

**One rule, both axes:** if the size the fit computes is below the window's
minimum on either axis, **do nothing and say so**. Half-fitting is the worst
outcome available — bars *and* a moved window — and measurement 3 says that is
exactly what the WM would hand us if the app simply asked, because the minimum
is a hint the WM clamps against rather than a request that fails. No threshold
is written down: the check is a comparison against `min-window-width` /
`min-window-height` in the pure function, and the numbers it involves change
when a splitter or a transport button does.

The refusal is `main.rs`'s **`show_notice`** with a `const` string, as
`DRAWING_HINT` (`main.rs:93`) and `HIGHLIGHT_PAUSE_HINT` (`:97`) already are.
**No `UserError` variant and no `Command`** — `UserError::is_notice` is the
bus's route, and W5's principle holds here too: this is the window's own
geometry.

**But it cannot be said at all unless the fit has three outcomes, not two.**
A two-state answer (`Some(size)` / `None`) collapses "already the right shape"
and "too small to fit" into one value, and since that same value is what offers
the action (W8), a refusal can never be reached: the button is already greyed
and the key already does nothing, silently. So the answer is:

```rust
pub enum Fit { NoSlack, TooSmall, To(u32, u32) }
```

`can-fit` is `!NoSlack`; `To` resizes; **`TooSmall` is the only thing that ever
shows the notice**, and it is reachable because the action is still offered. A
`Tooltip` that follows the same distinction is what the transport's Record
button already does for its three states. **This is not an extra mechanism —
it is the one that makes W6 exist at all.** A two-state version of this spec
should have deleted the notice instead.

Fullscreen belongs to the same refusal: winit ignores `set_size` on a
fullscreen window (`winitwindowadapter.rs:1935-1944`). The app never asks for
fullscreen, but a WM keybinding can, and without the check `f` would silently
do nothing.

**The maximised window is the coach's most likely state** — `main.rs:311-322`
reopens a maximised session by asking for the maximised *size*, and says so. A
maximised window cannot change shape, so fit must un-maximise first. Computing
the target *before* doing so is wrong twice over: the WM restores the
pre-maximise geometry (say 1100×700) while the target was computed from
1920×1080, which makes the "shrink" a **grow in both dimensions** — W1 violated
and W6's own safety argument with it — and on X11 the un-maximise is a
`_NET_WM_STATE` message while the resize is a bare `ConfigureWindow`, two
independent requests whose order the WM decides.

So: `f` on a maximised window calls `set_maximized(false)`, arms a **one-shot
pending fit with a deadline**, and the fit is computed and applied on a later
`tick`, from the geometry then in effect.

**What that later tick waits for is the window's size changing — not
`is_maximized()`.** `set_maximized(false)` writes the very property
`is_maximized()` reads (`i-slint-core/window.rs:2444-2454`), so the flag goes
false *synchronously*, before winit is told and long before the WM has restored
anything. A gate on the flag therefore fires on the very next tick and computes
the fit from the **maximised** geometry — the grow in both axes this paragraph
exists to prevent. The pending fit carries the window size at the moment of the
press and fires on the first tick where `w.window().size()` differs from it.
That is one field instead of two, it is the observable that actually matters,
and it survives a WM that restores in stages.

The deadline (500ms, the shape of the existing `notice_until`) is what stops a
WM that drops the un-maximise from leaving the fit armed to fire minutes later
when the coach un-maximises by hand; on expiry it becomes the `TooSmall`
notice. Whether Cinnamon honours the un-maximise at all is **a runtime check
for execution, not an assumption** — `main.rs` already carries the scar that
winit's maximise request "straight after mapping the window, before the window
manager has taken it on" is dropped by Cinnamon.

**The notice cannot be raised from `tick`.** `tick`'s whole body runs inside
`UI.with_borrow_mut` and `show_notice` borrows `UI` mutably too, so calling it
from there is a `BorrowMutError` — a panic that compiles cleanly and only
appears when a coach presses `f` on a maximised window the WM refuses to
restore. The pending-fit block sets `notice` and `notice_until` directly, or
decides an action and acts on it after the borrow ends.

The screen's own size is never consulted: Slint exposes no monitor geometry
(1.18's `Window` has size, position, scale factor, maximized and fullscreen,
and nothing about the display). W1's shrink-only rule is what makes that absence
harmless.

## W7. Where the widths live

`state.json`, beside the window size, as one optional field:

```rust
#[serde(default)]
panels: Option<PanelWidths>,
```

and `PanelWidths { sidebar: u32, inspector: u32 }` with
**container-level `#[serde(default)]`**. That attribute is not decoration: `read`
returns `State::default()` on **any** `serde_json` error for the whole document
(`bus/state.rs:196-200`) and every setter rewrites it, so one field a build
can't parse takes the last project, the pen and the speech model with it. It is
the same hazard `CLAUDE.md` gives as the reason the basket lives in its own
file. `WindowSize` has the same gap today — `{"window": {"width": 1600}}`
already costs the whole file — and closing it is a free adjacent improvement
that belongs in this pass.

The **home** is decided, not weighed: a column width describes the coach's
screen, not the match — the line that already puts the window size, the pen and
the speech model here and `Preferences` in `project.json`, where a new field is
a version bump every existing project would fail `store::read`'s guard on.

One field holding both widths, not two: they are read and written together, and
a file with one but not the other is a state nobody wants to reason about.

## W8. Where the arithmetic lives, and what it is

**A pure module in the lib** — `pundit-app` has a library target
(`src/lib.rs`) and `main.rs` is a separate `[[bin]]`, so **nothing under
`tests/` can call `main.rs`**. Anything left in the binary can only be tested
through the window, and a test that re-implements the arithmetic tests its own
copy. `zoom_input.rs` is the model to follow: `Viewport` already holds the four
numbers this needs (`frame_w`, `frame_h`, `area_w`, `area_h`), `picture(IDENTITY)`
already returns the content rect, `drawing_hint` is already a pure predicate the
window asks, and its header already states the "every function takes finite
input; the window drops non-finite values (BACKLOG #28)" discipline the fit
inherits.

One entry point, taking physical pixels, returning W6's three outcomes — and
**one call drives both the offer and the action**, so they can never disagree:

```rust
/// What fitting the window to the frame would do. Physical pixels, and the
/// target is rounded **up** (W1).
pub fn fit_window(frame: (f64, f64), player: (f64, f64),
                  window: (f64, f64), min: (f64, f64)) -> Fit
```

**It is built on `zoom_input::Viewport`, not beside it.** `Viewport::new`
already rejects every non-finite and non-positive value among the frame and
area, and `picture(Zoom::IDENTITY)` **is** the letterbox the fit has to agree
with — so taking the aspect from the same type that draws the picture makes
"the bars are gone afterwards" a property of one expression rather than of two
that must be kept in step. It also buys the strongest assertion available to a
unit test, with no window at all: the content rect that `Viewport` reports for
the *fitted* size is the same rect, to the pixel, as for the size before it.
`Viewport`'s doc already says only the shape matters, so physical pixels are
fine — `fit.rs`'s header says so, because `zoom_input.rs`'s says logical.

`main.rs` then holds three things and no arithmetic: the call, the un-maximise
path (W6), and one `in property <bool> can-fit` pushed on the existing tick.
**That push goes before `tick`'s `let Some(project) … else { return }`**: with
no project open the property would otherwise keep its last value, and since
`frame-width` is never cleared (W3) a stale `can-fit: true` would let `f`
resize the window over the empty-project card. `can-play` is part of the gate
in Rust, inside the same call — never as a second term in Slint, which is what
"one call drives both" means.

## W9. What this must not change

- **The content rect.** Strokes are normalized to it, highlights are placed in
  it, the scan scoreboard is drawn over it, the export crops to it — one
  `Zoom::transform`, spec D9. This work changes how much *area* the player is
  given and nothing about how the frame is placed inside it. `place-picture` is
  not touched.
- **The zoom's pan clamp.** `main.rs`'s `update_zoom` (~`:1855`) builds its
  `Viewport` from `player-width`/`player-height`, so the player's area already
  feeds every scroll, drag and 2/3 press. A resize changes it today; this
  changes nothing about how.
- **The export.** 1920×1080@30 regardless. A window's shape has never had a say
  in it and must not acquire one.
- **The scoreboard's raster cost.** #87's entry worried that the board is
  rasterized per device-pixel size and would re-raster on every drag frame, so
  it "needs to raster on release, or on a coalesced size". **Checked, and there
  is nothing to fix:** `show_board` has one call site, inside the 30Hz tick,
  behind `if ui.board_key != board` where the key includes the device size;
  the raster is the board's own corner only, measured ~0.3ms at 720p; and a
  window resize already does exactly this today. Coalescing would buy nothing.
  The `scrubbing` drop stays as it is. (Recorded because the backlog says
  otherwise, and a plan that "fixed" it would be adding a mechanism for no
  reason.)
- **The window's minimum.** 1100×700 stays, now behind a readable property.

## W10. What gets tested, and where

**Pure unit tests, in the lib** (`fit_window`, no backend, no window) — this is
where the facts the headless backend *cannot* see are pinned, and measurement 3
is why that distinction matters:

1. **The target is ceiled**, and the picture stays width-limited: the 715.5 case
   resolves to 716, and a floored implementation fails. Measurement 2's table is
   the fixture, and the assertion worth making is `Viewport`'s — the content rect
   at the fitted size equals the content rect before it, to the pixel — not the
   number 716.
2. **A pillarboxed player shrinks the width instead**, same rounding.
3. **At the frame's aspect the answer is `NoSlack`**, and feeding a computed
   target back in stays `NoSlack` (idempotence).
4. **Under the window's minimum on either axis the answer is `TooSmall`** — and
   `TooSmall`, not `NoSlack`, because that distinction is the only thing that
   makes W6's notice reachable.
5. **Non-finite and zero inputs are `NoSlack`**, which `Viewport::new` already
   decides (BACKLOG #28).

**One headless window test** (`pundit-app/tests/`, beside `slate_fields.rs`).
It cannot test that `f` resizes the window: `on_fit_window` lives in `main.rs`,
which is a `[[bin]]`, so a test can only install its own handler and would be
testing its own copy. What it *can* pin is the fact no pure function knows —
**that the chrome subtraction predicts the real layout**: wire `place-picture`
to the production `zoom_input::Viewport`, set 1600×960 and a 16:9 frame, call
`fit_window` directly, `set_size` its answer, and assert `player-width` and
**both** content-rect dimensions are unchanged, and that a second `fit_window`
on the new layout is `NoSlack` — idempotence in the real layout rather than in
arithmetic. Nothing about position (W1). Nothing hard-coding 520, 108 or 716.

An unwired `place-picture` returns `PictureRect::default()`, so every
content-rect assertion silently passes on 0×0. That is how the vacuous version
of this test gets written, and the test's header should say so.

**Separately, the `f` key's gate** belongs in the same file and is the fact that
bit the slates pass: `f` invokes `fit-window` when `can-fit` is true, does not
when it is false, and a focused field swallows it — `slate_fields.rs`'s own
shape, observing the callback through a closure.

**#87's own tests:**

6. **The columns' bounds** (the real window): a stored pair too wide for the
   window leaves **neither column below its own minimum** and the player at its
   own minimum. Do not assert an exact player width from arithmetic over the
   column widths — the two splitters are 12px the naive sum forgets, which is
   what made the first version of this test fail against the first version of
   W4.
7. **The drag anchors on the press** (a small `slint!` window over
   `ui/splitter.slint`, as `tests/scrubber.rs` does it): starting **from a
   clamped state**, a drag past the minimum stops there and a drag back picks
   the pointer up where it left it. Written from an unclamped state it catches
   neither failure in W5 — and the test window therefore needs a stand-in
   column carrying the real bound, because a `width-now` bound to a constant
   can never be clamped.

**In `bus/state.rs`'s own tests**, beside `remembers_the_window_size`:

8. **The widths round-trip**, a file that doesn't mention them reads the
   defaults, and — the one that matters — **a file with a partial `panels` or
   `window` object still yields the last project, the pen and the model**. That
   is the `#[serde(default)]` fix, and it fails today for `window`.

## W11. Deferred, on purpose

- **The splitter snap, and why it is not built.** The coach's decision came
  with "the splitter snap comes with it so the shape is reachable by hand too" —
  and it isn't, structurally. Because panels only grow (W4), a drag can only
  make the player **narrower**, which makes a letterboxed player *more*
  letterboxed. The snap could only ever fire on a **pillarboxed** player, which
  for 16:9 footage at 1600 wide means window heights in `[700, 715.5)` — a 15px
  band. It cannot touch the bars the coach complained about: that would need a
  player 1514px wide in a 1600px window, leaving 86px for two columns. So the
  snap would be a tint, a test and the only place a panel drag reads the
  footage, in exchange for nothing in the case that motivated it. **This is the
  one thing here the coach asked for that this spec declines to build, and it
  needs their nod.**
- **A panel narrower than today.** W4 says why: 280 is measured, not chosen. If
  a coach asks, the work is re-fitting the rows, not the splitter.
- **Panel widths per screen.** A coach who docks a laptop gets one pair for
  both. Not worth a second dimension in `state.json` until someone has two
  screens and says so.
- **A vertical splitter** (the transport bar's height, the Match panel's share
  of its column). Nobody asked; "all the panels" meant columns in a window
  whose panels are columns.

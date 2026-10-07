# Design — snapping the scrubber to the match marks

The coach (2026-09-28): *"snap to events in the scrubber as an option."* Dragging
the scrubber lands on whatever frame the pixel under the cursor works out to, so
getting the playhead onto a goal he tagged means dragging and then nudging with
the arrows. Asked what shape the option should take, he answered (2026-10-03):
**an on/off setting**, not snap-always with Shift to suppress.

Read `CLAUDE.md` first. This spec does not repeat the transport, `state.json` or
sheet rules it states. BACKLOG **#102** is the entry; every claim below was
re-derived from the source rather than copied from it, and **#102 was wrong about
one thing** (§L).

---

## §H. Where the option lives — and why it does **not** build the settings sheet

This is the decision the rest of the document hangs off, so it goes first.

**#102 does not build #78's settings sheet. The option is a checkable item in a
context menu on the scrubber itself.**

**H1. #78's own rule is what decides it.** `2026-10-02-app-settings-design.md`
§U1: *"Every configurable thing in this app is reached beside the thing it
configures, and that is the rule a new setting has to pass… the bar for a
settings sheet is not 'there is an option' — it is **'this option has no thing to
sit beside.'**"* §U4 then justified the sheet on **two** tenants with nothing to
sit beside: #102's bool and #84's local music folder path.

**That premise is half wrong, and this is the correction.** #102's bool has
exactly one thing to sit beside, and it is the most specific "beside" in the
whole app: the option changes the behaviour of **one control**, and that control
is a component of its own (`ui/scrubber.slint`). A snap setting on the scrubber's
own right-click menu is #116's precedent (a machine-wide value, read back
leniently, at the thing it affects, no screen) applied to a control rather than
to a row.

**This is not an override of a reviewed spec.** §D1 wrote the licence into #78
itself: *"It is the sheet's natural first tenant and builds it per **U4** — or,
if it finds a cheaper home beside the scrubber itself, that is its own call."*
This is that call, taken with the evidence §U4 did not have.

**H2. The home already exists in the tree, verified, not assumed.**

- `ContextMenuArea` is already the window's idiom for per-object options: the
  slate row's menu (`app.slint:5019`, named `slate-menu`) and the clip row's
  (`app.slint:5345`). The coach uses both.
- A `MenuItem` can carry a checkmark: `checkable` and `checked` are properties of
  the builtin (`i-slint-compiler-1.18.0/builtin_elements.rs:1877`/`:1883`).
- `ContextMenuArea` is `@expands_to_parent_geometry`
  (`builtin_elements.rs:2061`–`:2062`), so a bare one inside `Scrubber` covers
  the track with no geometry of its own.
- **It cannot fight the drag.** `ContextMenu::input_event` accepts a **right**
  press (`i-slint-core-1.18.0/items.rs:1630`) and returns `EventIgnored` for
  everything else (`:1658`), while its `input_event_filter_before_children`
  forwards (`:1616`).
- **Order is load-bearing, and the clip row already has it right.** `TouchArea`
  grabs a press of **any** button (`items/input_items.rs:124`–`:138`), so the
  menu area must be the **later** sibling: hit-testing is `FrontToBack`
  (`input.rs:1792`–`:1794`), last-declared first. In the clips row the
  `ContextMenuArea` follows the `TouchArea` for this reason. In `Scrubber` the
  pointer `TouchArea` is the last child today (`scrubber.slint:151`), so the menu
  area goes after it.
- **It still opens while the scrubber is disabled** (a recording): the drag
  `TouchArea` returns `EventIgnored` when not enabled (`input_items.rs:120`), and
  it is behind the menu area anyway. So the item needs no `enabled` gate — it
  writes a preference and acts on nothing, like the fold chevrons and the pen
  width.
- **It is testable through the real menu.** `tests/ui/slate_menu.rs` right-clicks
  a row, finds the item by its words and clicks it, through
  `i_slint_backend_testing`'s `ElementHandle`. That machinery exists because of
  #127 and costs this feature nothing.

**H3. A sheet would be worse here, on #78's own terms.** #78 refuses to build the
sheet because *"a container built before its contents is the thing this spec
exists to prevent"*. Having #102 build it inverts that into the same mistake with
a different owner: a seventh `Sheet`, a button squeezed into the drawing row
(§U4's placement is *"an estimate, not a measurement"*), an Esc/`text-editing`
fold it needs no field for — all to hold one checkbox that §U1 says belongs on
the scrubber. Every change must earn its place; this one would be paid for by
#102 and used by #84.

**H4. What this means for the other citers.**

- **#84 (music) is the sheet's first tenant and builds it per §U4, unchanged.**
  Its entry already names the reason: a **local music folder path** is
  machine-wide, cannot be a picker beside anything, and needs a file-chooser
  button — a surface, not a menu item. Nothing in §U4's design is being spent or
  altered here; it is simply not being built yet.
- **#96 (rebindable keys) is unaffected.** Its spec's D2 already ships the key
  list as **its own sheet on `F1`**, and §D5's rule stands: a keys **tab** in the
  settings sheet is a reasonable future, a tab-less sheet with a key map poured
  into it is not. #96 waits on nothing here.
- **#131 (the slate band's colour)** is a *scrubber* setting too, and if it is
  ever built the menu this adds is the obvious place to look first — a submenu of
  named colours sits beside the band exactly as this item sits beside the drag.
  **Not decided here:** #131's own open question is whether a picker is wanted at
  all.
- So the sheet's tenant list is #84 (the path), #131 (if it becomes a picker) and
  #96's future tab — and #78's design waits for the one that needs a surface.

**H5. The item.** One item, no submenu:

```text
[x] Snap to match events
```

The title names what §E settled, so widening the rule (the open question) renames
it. The menu area is **named** — `snap-menu := ContextMenuArea` — for
`slate_menu.rs`'s stated reason: every `LineEdit` carries a `ContextMenuArea` of
its own, so the type name alone finds six in this window.

**H6. `checked` is bound, and the self-toggle is harmless — verified.**
`MenuFromItemTree::activate` writes `checked` directly
(`i-slint-core-1.18.0/menus.rs:190`–`:191`), which is the move that breaks a
Slint binding; `app.slint`'s own comment on the match-setup fields records the
same hazard. It does not bite here because the `MenuItem` tree is **instantiated
per show**: the generated show-popup code constructs a fresh item tree before it
reads the entries (`i-slint-compiler-1.18.0/generator/rust.rs:4797`–`:4802`). The
direct write lands on an instance that is thrown away with the popup, and the
next open re-reads the binding. So `checked: root.snap-to-events` is sound, and
the window property stays the one thing the snap reads.

---

## §E. What "events" covers: the marks that are drawn

**E1. Match events only, which is #102's own defensible first cut.** The marks
the scrubber draws are `match-marks` (`app.slint:3954`), built in
`main.rs::show_match` (`:1825`–`:1842`) from `match_panel::match_rows` — every
goal in its team's colour and every start/stop in white. That is the whole list,
and it is the list the snap searches.

**E2. The rule behind the cut: you can only snap to a mark you can see.** It is
worth stating as a rule rather than as a scope line, because it answers the
extension question and bounds the feature. Player highlights and clip starts are
**not drawn on the scrubber** — including them would snap the drag onto marks
nothing on screen shows, which is a different feature (draw them first). The
slate span **is** drawn (`scrubber.slint:97`–`:114`, shipped this week), which is
why it is the open question and not a refusal.

**E3. One bool, never a set.** #102 notes the shape to know about: a malformed
*element* still costs a whole list under #100's per-field read, so a stored
**set** of mark kinds would be a list that one bad entry empties. Nothing here
stores a set — the kinds are a compile-time decision and the option is one bool
(§S).

---

## §P. The radius is in pixels, and the number is six

**P1. Pixels, for #102's stated reason, re-derived.** The scrubber's
seconds-per-pixel changes with the window's width and with the footage's length,
so a radius in seconds would snap from half a screen away on a short project and
never on a long one. Measured from the layout: at the default 1600×960 window the
player column is **1068px** (`bus/state.rs:190`, measured there), less the
transport bar's 8px padding either side is ~1052px of track, less the 20px thumb
is **~1032px of travel** (`value-at`, `scrubber.slint:71`–`:73`). A 45-minute half
is then **2.6 s/px** and a two-half match **5.2 s/px** — a factor of two between
two projects of the same coach, on the same screen.

**P2. Six pixels.** `property <length> snap-radius: 6px;` beside `thumb`
(`scrubber.slint:75`), because the radius is a geometry fact about this control.

- **The floor:** a mark is drawn 2px wide and 12px tall (`scrubber.slint:117`–
  `:123`). The radius has to make "click the tick I can see" reliable rather than
  a matter of aim, and 6px gives a 14px-wide target — wider than the tick is
  tall.
- **The ceiling is the reason it is not larger.** The basin in *seconds* grows
  with the footage: ±16 s on a 45-minute half, **±31 s** on a 90-minute match.
  Half the thumb (10px) — the tidier derivation, and the one considered first —
  would make it ±52 s, swallowing the reel's own 20 s lead-in (`REEL_LEAD_IN`)
  and most of a build-up. Smaller is the safer error here, because the escape for
  "I wanted to be near it, not on it" is the arrows (§F), while the escape for
  "I cannot get onto it" is the whole feature.
- It is the one tuning knob, it is one number, and the spec expects it to be
  tuned once on real footage rather than argued about here.

**P3. What crosses into Rust is the radius *in seconds*, not the radius.** Only
Slint knows the track's width; only Rust can search the marks (**Slint has no
loop construct — a model cannot be iterated in a function**, which is why the
arithmetic cannot all live beside `mark-x`). So `Scrubber` publishes one derived
number:

```slint
in property <bool> snap-enabled;
/// The snap radius in the scrubber's own units: `snap-radius` of track,
/// as seconds, or 0 for "do not snap" — off, or nothing to snap to.
out property <float> snap-seconds:
    root.snap-enabled && root.marks.length > 0
        ? root.snap-radius * root.maximum / max(root.width - root.thumb, 1px)
        : 0;
```

`length * float / length` is a float, as `value-at` already relies on, and
`marks.length` is the idiom used in nine places in `app.slint`. **A single
non-negative number is the whole contract**, and `0` carries every reason not to
snap at once — including a **preview**, where `marks` is already `[]` because a
preview's scale is its clip's (`app.slint:6027`). That is inherited from the one
binding that states the rule rather than written out a second time beside it.
`AppWindow` forwards it (`out property <float> scrub-snap-seconds:
scrubber.snap-seconds;`) and `main.rs` reads it in the scrub handlers.

---

## §L. What the snap lands on, and in which time base

**L1. The scrubber's axis is the concat timeline**, in seconds
(`app.slint:3586`, `:6024`–`:6027`), via `Project::abs_seconds` — so the marks,
the snap and the landing are all absolute concat seconds, and `Bus::scrub`'s
`locate(abs)` turns that back into `(source, seconds)` exactly as it does for an
unsnapped drag (`bus/transport.rs:226`–`:236`, `:339`–`:348`).

**L2. It lands on the mark's stored instant — which is the same value `]` already
seeks to.** `match_panel::match_abs` (`:223`–`:229`) is the list `[` and `]` step
through and the list `show_match` draws, *"the same list, so they cannot drift
apart"*. The snap joins them as a third reader of it, so one match event has one
landing for all three controls, and a snapped release is indistinguishable from
`on_jump_chapter`'s (`main.rs:1469`–`:1491`) — both end in
`Command::ScrubRelease { abs }`.

**L3. #102's claim about the time base is half false, and nothing depends on the
false half.** The entry says *"the highlight-ring and slate work both key marks
by the **displayed frame's** time, which is what a snap should land on too."*
Checked:

- **Highlights do** — `shown_source_position` (`main.rs:1724`–`:1732`) uses
  `ui.shown_stream_time`, the displayed frame's own stream time, "so the box and
  the time describe one frame".
- **Slate marks do not.** `i`/`o` use `scan_source_position`
  (`main.rs:1712`–`:1718`, wired at `:1218`/`:1229`), i.e. `scan_abs`, which
  prefers a pending skip target and otherwise queries the player.
- **Match events do not either** — `on_tag_match_event` (`:1402`–`:1417`) uses
  the same `scan_source_position`.

It does not matter, and the reason is worth writing down: a snap lands on the
**stored** instant whatever captured it. The displayed-frame rule governs where a
mark is *put*; the snap only has to agree with the two controls that already go
*to* one, and L2 is how it agrees.

**L4. f64 in Rust, f32 only for drawing.** `Mark.at` is an `f32`
(`main.rs:1833`), which is a drawing. Rust re-derives from `match_abs`'s `f64`,
so the snapped landing is bit-identical to `]`'s rather than a few hundred
microseconds off it.

**L5. The snapped value is written back, and that is not cosmetic.** The handler
sets `position-seconds` to the snapped value. Without it three things disagree:
the picture sits on the mark while the thumb sits under the pointer, and the
**readout** — which reads `position-seconds` while `scrubbing`
(`main.rs:4035`–`:4037`) — shows a time no frame is at. With it, the thumb sticks
to the tick and the readout reads the mark's own time, which is the whole visual
feedback of the feature.

**L6. The write-back cannot drift, and that is a property of this scrubber.**
Every pointer move recomputes the value **absolutely** from `mouse-x`
(`scrub-to` → `value-at`, `scrubber.slint:83`–`:86`), with no press anchor and no
accumulated delta. So writing a snapped value into `value` cannot compound: the
next move overwrites it from the pointer. (The **splitter** is the control that
anchors on the press, `ui/splitter.slint` and CLAUDE.md's panel rules — not this
one. A snap on an accumulating drag would have needed the snap offset carried in
the anchor; here it needs nothing.)

**L7. Where the search lives: `match_panel.rs`, beside `[` and `]`.**

```rust
/// The chapter nearest `abs`, if one is within `within` seconds: what a
/// snapping scrub lands on.
pub fn nearest_chapter(abs: f64, chapters: &[f64], within: f64) -> Option<f64>
```

One module already owns "where the match events are, and how a control lands on
one", which is what stops a fourth reading of the list from drifting. `within <=
0.0` returns `None`, which is how §P3's single-number contract reaches Rust.

---

## §S. Storage: one `state.json` bool, default on

**S1. `state.json`, by that file's own test.** Its header asks what a setting
*describes*: "None is a project's." Whether the drag snaps is how this coach
works, not a fact about a match, and a project carried to another machine must
not bring it. It is therefore **not** a `Preferences` field and bumps nothing:
`CURRENT_FORMAT_VERSION` stays at **15** (`store.rs:21`).

**S2. One field, one attribute** — #100's per-field read is already in place:

```rust
/// Whether a scrub snaps to the match marks (BACKLOG #102).
#[serde(deserialize_with = "lenient")]
snap_to_events: Option<bool>,
```

`Option<bool>`, not `bool`, because the stored default is **on** and `lenient`
falls back to `T::default()` — `false` for a bare `bool`, which would turn the
feature off on any unreadable value. With `Option`, "absent", "unreadable" and
"this build cannot read it" all read as `None`, and the accessor turns `None`
into **on**. One unreadable value costs this field alone and never the coach's
last project (`bus/state.rs:128`–`:143`).

**S3. Accessors mirror the folds** (`bus/state.rs:399`–`:411`):
`snap_to_events() -> bool` and `set_snap_to_events(bool)`, read once at startup
and written on the toggle.

**S4. The default is on, and the reason is §H.** The `Folds` default follows
"what the app looked like before it" because a fold is layout; this is the
behaviour the coach asked for, and an opt-in behind a right-click he has not been
told about would ship the switch without the feature. The switch exists because
he wanted it optional — an escape — not because he wants it off. Overturning it
is one line in the accessor.

**S5. Written through the UI thread's `AppFiles`, never the bus** — `wire_snap`,
on `wire_folds`' pattern (`main.rs:2486`–`:2518`), with one function pushing the
window property so the stored value and the checkmark cannot disagree. **The bus
never learns the option exists:** it receives an `abs` it cannot tell from a
hand-dragged one. No new `Command`, no new `Event`, no change in `pundit-core`
and none in `pundit-media`.

---

## §F. What the snap must not fight, and what the escape is

**F1. The escape is the pixels, and it is already paid for.** The radius is 6px
and the mapping is absolute, so **the snap never holds the playhead**: drag more
than six pixels past the mark and the drag lands where the pointer is. There is
no state to clear and no gesture to learn.

**F2. Shift is not the escape, twice over.** The coach rejected
snap-always-with-Shift-to-suppress as the *shape* of the feature, so re-adding
Shift inside the opt-in puts back what he turned down. And Shift over this
control already means something: `scrub-scroll(dx, dy, shift)` carries it to
`wheel.rs` as the **far** skip, as it does on the arrows.

**F3. The frame-accurate paths are untouched, which is #102's own rule.** `,` and
`.` step exactly one frame, the arrows skip exactly 3 s and 10 s, the wheel skips
by whole notches, and `[` / `]` land exactly on an event. All are promises about
exact distances; snapping belongs to the **drag** alone — the press, the moves and
the release of the one `TouchArea` (`scrubber.slint:151`–`:169`) and nothing else.

**F4. So "near a mark, not on it" is the arrows' job and gets better, not
worse.** Landing 3 s before a goal is 1.1px on a two-half timeline: today it is
luck, and after this it is the snap plus one left arrow. The combination is the
argument for the feature as much as the escape from it — the snap gives exactness
*at* a mark, the arrows give exactness *from* one.

**F5. And the switch is two clicks away on the control itself**, which is the
other half of §H's case: turning it off never means leaving the scrubber.

---

## §C. Diffed against the shapes this copies

Three bugs in the slates panel this week were rules the clip inspector already
had that the copy did not. So, explicitly, what was checked against what:

**C1. `Zoom::snapped` (`core/src/zoom.rs:121`–`:137`) — the app's existing snap,
and both of its rules are wrong here.**
- Its tolerance is **relative** (3% of the notch) because zoom notches are
  multiplicative. Copied onto a linear time axis, 3% of a 90-minute match is
  **162 s**. The radius must be pixels (§P1).
- It returns the **first** notch in table order and documents that first and
  nearest coincide *"only by property of this table"*. Match marks are
  coach-placed and can be arbitrarily close, so the rule must be **nearest**,
  with a deterministic tie-break (the earlier mark).
- Its *third* rule is the one worth keeping: *"Interactive commits only. Replay
  never snaps."* That is §F3 in other words.

**C2. `CHAPTER_TOLERANCE = 0.5 s` (`match_panel.rs:24`) must not be reused, and
its rule is the opposite of this one.** It is how near the playhead a chapter can
be and still count as *the one under it*, which `[` and `]` **step past** —
pinned by `previous_and_next_chapter_skip_the_one_under_the_playhead` (`:534`).
`nearest_chapter` must **include** the mark under the playhead; reusing either the
constant or the predicate would make the snap refuse the mark it is aimed at.

**C3. The clip and slate row menus** (`app.slint:5345`, `:5019`): the
`ContextMenuArea` goes **after** the `TouchArea` (C-order is hit order, §H2); it
is **named** so a test can find it among the six a window full of `LineEdit`s
has; its items are found by their words. Differences taken deliberately: **no
`enabled` gate** (it acts on nothing — the row menus gate on `!recording`
because theirs act), and **no selection side-effect** (`slate_menu.rs`'s whole
subject; there is nothing selected here).

**C4. `wire_folds` / `show_folds` (`main.rs:2486`–`:2518`)** — read at startup,
one writer pushing the window property, written through the UI thread's own
`AppFiles`, the bus untouched. Difference: a single bool needs no name-keyed
callback, so there is no "unknown section" arm to get wrong.

**C5. A sheet's Esc contract (`CLAUDE.md`, `§U4`) does not apply**, and checking
that is what rules a field out of this home: a menu can hold a checkmark and
cannot hold a path, which is precisely why #84 still needs the sheet (§H4).

---

## §X. What it does not do

- **No settings sheet** (§H), no new `Sheet`, no new button in any row.
- **No key**, and no `keymap.rs` action. If the coach later wants one, #96's table
  is where it goes — one row, default *(none)* — and that would make the menu no
  longer the only writer of the flag, which is the one thing §H6 depends on.
- **No snap on the wheel, the arrows, `,` / `.` or `[` / `]`** (§F3).
- **No snap in a preview** (§P3), where the scale is the clip's.
- **No snap to player highlights, clip starts or slate marks** (§E, and the open
  question).
- **Nothing drawn differently**: no magnet cursor, no highlighted mark, no second
  colour for a snapped tick. The thumb landing on the tick is the feedback (§L5).
- **No `project.json` change** (§S1), **no bus command**, **no `pundit-core` or
  `pundit-media` change**.
- **It does not make the marks easier to see.** A tick is 2px; if the complaint
  turns out to be "I cannot see where the goals are", that is a drawing change and
  a different entry.

---

## Crate responsibilities

| Crate | Change |
|---|---|
| `pundit-core` | **None.** |
| `pundit-media` | **None.** |
| `pundit-app` | `ui/scrubber.slint`: `snap-radius`, `snap-enabled`, `snap-seconds`, the named `ContextMenuArea`. `ui/app.slint`: forward `snap-seconds`, the `snap-to-events` property, the `toggle-snap` callback. `src/match_panel.rs`: `nearest_chapter`. `src/main.rs`: `wire_snap`, and the two scrub handlers snap + write back (they stop being `cmd(bus, …)` one-liners, `:743`–`:744`). `src/bus/state.rs`: the field and its two accessors. |
| `pundit-harness` | **None** — nothing crosses the bus (§S5). |

## Testing

1. **`match_panel.rs` unit tests, beside `[`/`]`'s own** (`:534`): nearest wins
   over first; **the mark under the playhead is returned** (the C2 difference, and
   the test that fails if `next_chapter`'s predicate is copied); a tie takes the
   earlier mark; nothing within the window gives `None`; `within <= 0.0` gives
   `None`; an empty list gives `None`.
2. **`tests/ui/scrubber.rs`** — the existing stand-in window (400px wide, maximum
   100) gains the two properties: `snap-seconds` is `6 × 100 / 380 = 1.579` with
   marks and snapping on, and **0** with no marks and 0 with snapping off. That
   pins the unit conversion, which is the half of §P3 no Rust test can see.
3. **`bus/state.rs`**, on `remembers_the_folds_and_defaults_to_every_section_open`'s
   shape (`:963`): default **on** with no file; a stored `false` survives a
   reopen; `{"pen":"blue","snapToEvents":"yes"}` reads **on** and still reads the
   pen. And the new field joins `the_settings_are_independent` (`:877`).
4. **Not automated, and said rather than implied:** the right-click opening the
   menu over the track (and while a recording has disabled the drag), the
   checkmark surviving a reopen of the menu, and the thumb sticking to a tick and
   letting go six pixels later. `slate_menu.rs`'s machinery could drive the first
   two against the real window; the third is a feel, and feel goes on the manual
   list.

## Risks

1. **The coach dislikes snapping and does not find the right-click.** The real
   risk of §H, and the reason the default is on (§S4) — he meets the behaviour he
   asked for, and the gesture he would try on a control in this app is a
   right-click, which two other controls already answer. Bounded cost: the drag
   behaves exactly as it does today six pixels further on.
2. **±31 s on a long match feels like a magnet.** Mitigated by the radius being
   one constant in one file (§P2) and by the arrows (§F4). If it has to come
   down, 4px is ±21 s and still a 10px target.
3. **`position-seconds` is written from a handler during a drag.** Safe because
   the tick treats it as the truth while `scrubbing` (`main.rs:4035`) and because
   the mapping is absolute (§L6) — but it is the one place this feature writes a
   property the UI also writes, and a future scrubber that anchors on its press
   would invalidate the reasoning, not just the code.
4. **`match_abs` allocates per pointer move** (a `Vec` plus a label `String` per
   event, ~20 events, 30 moves a second). `on_jump_chapter` already does it per
   keypress; if it ever matters, the answer is a cache rebuilt on
   `ProjectChanged`, not a leaner list — the shared list is what stops the drift
   (§L2).

## Deferred

- **Widening the snap** beyond match events is the open question below, not a
  deferral: it is a one-line change to which list is passed.
- **Snapping to a highlight or a clip start** needs those marks drawn first (§E2).
- **A key for the toggle** waits for #96 and for the coach asking (§X).

## Open question for the coach

**Should the selected slate's in and out points snap too?** The scrubber now
draws that span (`scrubber.slint:97`–`:114`), and §E2's rule — you can only snap
to a mark you can see — admits it. It is the one extension that costs nothing:
two more `f64`s in the list Rust already searches, the same bool, the same radius,
no storage and no UI change (the item's words would widen to *"Snap to marks"*).

- **Default if he does not answer:** match events only, which is what ships.
- **What it blocks:** nothing. It can be added later with no migration and
  nothing to undo.

---

## §R. What my own first reading got wrong

1. **It assumed #102 had to build the settings sheet**, because both BACKLOG #102
   and §U4 say so. What overturned it was §D1's own licence (*"or, if it finds a
   cheaper home beside the scrubber itself"*) plus two facts in the tree: the
   window already right-clicks for per-object options, and a `MenuItem` can carry
   a checkmark. §U4's premise — "two options with nothing to sit beside" — was
   half wrong, and the half that was right is #84's.
2. **It assumed the scrubber's drag anchors on the press**, as the splitter's
   does. It does not: every move recomputes the value absolutely from `mouse-x`
   (§L6). The assumption would have added anchor bookkeeping to keep a snapped
   drag from drifting; the real design needs none, and the write-back that makes
   the thumb stick is only safe *because* of it.
3. **It repeated #102's claim that slate marks are keyed by the displayed frame.**
   Only highlights are (§L3); slates and match events both use the scan position.
   Corrected in the entry.
4. **It planned to do the whole snap in `scrubber.slint`, beside `mark-x`.** Slint
   has no loop, so a model cannot be searched in a function. The split — geometry
   in Slint, nearest-mark in Rust — follows from that rather than from taste, and
   it is why the contract across the boundary is one number (§P3).
5. **It reached for `CHAPTER_TOLERANCE` as the radius.** Wrong units and the
   opposite rule: it exists to *skip* the mark under the playhead (§C2).
6. **It suspected a checkable `MenuItem` could not be bound**, since `activate`
   sets `checked` directly and a direct set kills a binding. True, and harmless:
   the item tree is built fresh on every show (§H6). Verified in the generator
   rather than assumed either way.
7. **It read the radius as a comfort question only.** The number that matters is
   the basin in *seconds*, which the footage's length sets and which made the
   tidier derivation (half the thumb) the wrong one (§P2).

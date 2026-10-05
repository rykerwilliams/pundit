# App settings: what an export writes, and where a setting lives

**BACKLOG #78** (`BACKLOG.md:1229-1255`). The coach (2026-09-24): *"add to the
backlog an appsettings? we don't need a screen for it yet. maybe we already have
it. e.g. the srt file gen, the other chapter track, etc. these are general app
config settings to be turned off or on."*

**The entry was filed the same day `.chapters.txt` shipped** (0.6.0,
`CHANGELOG.md:214-221`), and "the srt file gen, the other chapter track" names
exactly the two things that had started appearing beside a whole-match export.
What they were looking at when they wrote it is an inference, not something the
entry says — which is why **Open question 3** asks rather than assumes.

**#78 is cited as a prerequisite by four other entries** — #84, #102, #115
and #116 — and in three of the four that citation is stale or wrong. So this
spec has two jobs: answer what the coach asked, and stop #78 being the hook
every unhoused option gets hung on. **"A settings screen is where features go to
hide" is the standing risk here, and §X is the half of this document that earns
its place.**

**What the audit found** (§W2). #78's entry names six things that could become a
setting. **Two already have one** (the `.srt` and the `tx3g`, both on the
Scoreboard picker — §W3), **a third already has a better one** (the reel's
lead-in and tail, per goal — §X2), **two are not settings at all** (the file
tags, the avatar's pulse constants — §X1, §X3), and **one is real** (the
chapters). So the answer is a single checkbox, on the sheet that already carries
three like it, and **no new UI surface, no new file and no new command**.


## Overturned by the coach, 2026-10-03 — read this before the sections below

Two answers change this spec's headline decision, and the sections after this one
have **not** been rewritten yet; they are the reasoning as it stood, which is
still worth reading for the audit in §W2.

**1. "I think all of them should be settings?"** — asked which files in
`exports/` were unwanted when #78 was filed. So the spec's collapse of #78 to a
single chapters checkbox is **not what was asked for**: the coach wants every
export output switchable.

**And the coach answered the fork it raises (2026-10-03): fully independent
switches.** The question was put — the `.srt` and the embedded `tx3g` are not
independently controlled today, they *follow* from the Scoreboard picker (burned
in writes neither and removes a stale `.srt`; separate track writes both),
because a subtitle track of a board already painted into the picture is the
board twice. Asked whether to leave them following the picker, make them
independent, or grey them when redundant, the coach chose **independent**.

**So two combinations become reachable that are not today, and both are
legitimate — but neither may be silent.**

- **Board burned in *and* a subtitle track of it.** The board twice. The coach
  asked for the control knowing this; it is their file.
- **Scoreboard: Separate track with the subtitle outputs off** — which means
  **no board anywhere**: not burned, not beside the file, not embedded. Read
  charitably this is a thing a coach may well want (a clean copy with no board
  at all), so it is not a state to refuse. But it is **the one combination where
  what you asked for and what you get look nothing alike**, so the export
  sheet's explanatory line — which already follows the *effective* mode — has to
  say it in words before the run, not leave it to be discovered in `exports/`.

**What this does to the code**, so the plan does not discover it: `job.cues`
stops being derived from the picker. `carry_scoreboard` currently maps mode +
target into **both** the renderer and the cue slot in one place; the cues become
their own input. That is a coupling removed rather than added, but the two rules
it currently states together ("track mode blanks `job.scoreboard`" and "a clip
or reel asked for on a separate track burns it in rather than dropping it") have
to survive separately — the second especially, because it exists so the picker
can never lose the board, and a clip's cue slot is **not** where its board can
live.

**Amended 2026-10-05: `carry_scoreboard` has moved under this paragraph's feet,
and in a helpful direction.** The mute-source-audio feature
(`2026-10-04-mute-source-audio-design.md`) now passes it a `with_audio: bool`,
because it is `can_copy`'s only caller and the gate has to be asked the muted
question. Three consequences for whoever plans #78:
- **The precedent for "an input, not a carried field" is now set in that exact
  function.** The mute is handed *in* and `Carry` gained **no** field, on the
  reasoning that `job` already holds the value and a carried copy would be two
  truths for one fact. Prising the cues out of `carry_scoreboard` is the same
  move in the opposite direction, and the two should read consistently.
- **The function is no longer only about the board**, so its name is already
  approximate. If #78 removes the cue coupling, that is the moment to rename it
  rather than grow a third concern under a board-shaped name.
- **`job.cues` is still derived from the picker as of `d585d24`** — the
  overturned decision above has been recorded, not implemented. Read the code,
  not this paragraph, when the plan is written.

**2. #102 is an on/off setting**, not the snap-always-Shift-to-suppress shape
§U1 recommended.

**Together these retire §U1's conclusion that no settings screen is justified.**
That conclusion rested on "this option has nothing to sit beside" — and there are
now **three** tenants: #102's bool, the export-output switches, and #84's local
music folder path. The sheet designed in §U3–U4 should be built, and §U1's rule
("a setting sits beside what it configures") survives as the test a *new* setting
has to pass, not as an argument against the screen.

**What has not changed:** §X's refusals (the header tags, the reel trim, the
avatar constants, anything already carrying a control, anything belonging to one
clip), and the format-cost correction in §W3, which was independently verified
and is now in `CLAUDE.md`.

## Goal

The coach can stop an export writing anything beside the video, from the sheet
they are already looking at; and every candidate setting has a decided home, so
the next one does not arrive as a sixth checkbox on a screen nobody designed.

## Scope

In: the audit of what an export writes (**§W2**), one `Preferences` field and the
v14 bump it costs (**§S2**), one checkbox on the export sheet (**§U2**), and the
**design** of the Settings sheet for whoever needs it first (**§U3**) — specified
here, built by #84 or #102, not by this.

Also in, because this change invalidates them:

- **`bus/state.rs:8-11`**, whose header states the cost of a `Preferences` field
  as *"`store::read`'s exact-version guard would make every existing project
  unreadable"*. `store::read` is a **range** (`store.rs:124-137`), and §S3 states
  the real cost. A wrong price on the only decision #78 turns on is the kind of
  claim that sends the next three settings to the wrong file.
- **`CLAUDE.md`'s speech-model paragraph**, which carries the same claim in
  shorter form — *"a `Preferences` field would be a format change every existing
  project fails `store::read`'s version guard on"* — and is the more-read copy of
  the two. Both say the same untrue thing about the same guard, which is why they
  are corrected together rather than one at a time. (The paragraph's *conclusion*
  is unaffected: the speech model is the coach's, not the match's, so `state.json`
  is still right for it. Only the price is wrong.)
- **BACKLOG #78 itself**, resolved, with §W3's correction recorded in it; and
  **#84, #102, #115, #116**, whose "needs #78" lines are rewritten per §D.

Out: everything in **§X**.

---

## Decisions

### W. What #78 actually is

**W1. #78 is two requests under one number, and this spec splits them.**

- **(A) What the coach asked for:** switches for the files an export puts on
  disk. Concrete, scoped, and answered here.
- **(B) What #84 and #102 cite #78 for:** *a place for a machine-wide option to
  live* — a music folder, an API key, a snap toggle. None of those is a thing an
  export writes.

Reading them as one request is how #78 becomes a preferences system. They are
answered separately: (A) on the export sheet (**§U2**), (B) by a Settings sheet
that is designed in **§U3** and built by its first real tenant.

**The entry's own "Why deferred" asks for exactly this**: *"it should arrive with
its home already decided rather than as six checkboxes."* A spec that decides the
home and builds one checkbox is that sentence taken literally.

**W2. What an export writes today, in full.** Verified against the code rather
than against the entry; this table is the whole of (A)'s subject matter.

| Written | Where from | Which runs write it |
|---|---|---|
| the `.mp4` | both renderers | all |
| `chpl` chapters, inside the file | `chapters::splice` (`composite/export.rs:435`) from `plan.chapters` | any target with ≥2 entries (`plan.rs:152-155`) |
| `.chapters.txt` beside it | `write_chapter_list` (`composite/export.rs:511`) → `core::chapters::chapter_list` | any target whose chapters survive YouTube's rules — ≥3 after the 10 s rule (`chapters.rs:28-31`) |
| `.srt` beside it | `write_sidecar` (`composite/export.rs:462`) from `job.cues` | **only** the whole match on a separate track |
| a `tx3g` subtitle track, inside the file | `composite/copy.rs:623-637` | **only** the whole match on a separate track |
| header tags | `job.tags` → `composite/tags.rs`, both renderers | all |

Per target, the `cues` slot is decided in one place, `carry_scoreboard`
(`bus/export.rs:579`): a non-whole-match target gets `cues: None` (`:602` — "no
sidecar at all, so nothing at that path is written **or removed**"), a burned-in
board gets `Some(empty)` (`:590` — writes none, removes a stale one), and only
the track mode gets cues (`:628`). A basket film is `cues: None` too
(`bus/basket.rs:482-484`).

**W3. The `.srt` and the `tx3g` already have their switch, and #78's entry is
out of date about it.** The entry says of everything it lists: *"All are 'on'
today with no way to say otherwise."* For two of them that is false, and has been
since 0.4.0 (`CHANGELOG.md:267-272`), a day **before** the entry was filed.

Both ride `job.cues`, and `job.cues` is the Scoreboard picker
(`app.slint:1774-1778`). **Scoreboard: Burned into the picture** writes neither,
and removes a stale `.srt`. Nothing to build; the control the coach wanted exists
and is one row above where they were looking.

**The one gap this leaves is not worth a control.** A coach who wants the board
as a separate track *inside* the file but no `.srt` beside it cannot have that.
The embedded track and the sidecar are deliberately one decision —
`composite/copy.rs:60-63` gives the reason (the track carries *"the same lines
the `.srt` gets — so the board survives the file being copied to a phone or sent
on, where a sidecar does not"*), and `CLAUDE.md:528` states the pair as one rule
— and splitting them would be a second picker for a combination nobody has
named.

**W4. What is left is the chapters, and one switch covers both forms of them.**

The `.chapters.txt` is the only file an export puts beside the video with no way
to say no — and the `chpl` box inside it is the same list. **One switch, both
forms**, because a coach who turns "Chapters" off and still finds chapters in the
file has been told a half-truth, and the in-file box is the half they cannot see.

Default **on**. It costs nothing measurable (CLAUDE.md: *"Tags cost the copy no
losslessness and the chapters no room"*, measured), a chapter list is the only
way those chapters reach a YouTube viewer at all, and the coach asked to be able
to turn it off — not for it to be off.

### S. Where a setting lives

**S1. The rule is applied per setting, and the question is whose property it
is.** CLAUDE.md's line, which #78's entry restates as the decision to take first:
the *machine's* (`state.json`, no format change) or the *project's*
(`Preferences` in `project.json`, a version bump). Applied to each candidate:

| Candidate | Whose | Home |
|---|---|---|
| chapters on/off | the **export's** — "this one is going to YouTube" | `Preferences`, beside its three row-mates (**S2**) |
| `.srt` / `tx3g` | the export's | already the Scoreboard picker (**W3**) |
| header tags | nobody's: not a setting (**X1**) | — |
| reel lead-in / tail | the **goal's**, and already stored there (**X2**) | — |
| avatar pulse constants | nobody's: tuning (**X3**) | — |
| snap to events (#102) | the **coach's** | `state.json` (**D1**) |
| a music folder, an API key (#84) | the **coach's** | `state.json` (**D2**) |

**S2. The chapters switch is a `Preferences` field, and it rides the export
sheet's existing write-back.**

`Preferences` already holds `last_export_resolution`, `last_export_quality` and
`last_export_scoreboard` (`project.rs:138-142`), and the sheet's whole
stickiness is one mechanism: `Pickers::of(prefs)` reads them
(`bus/export.rs:550-556`), `Command::Export` carries them
(`bus/mod.rs:334-342`), and one write-back after `begin` stores them
(`bus/export.rs:360-370`). A fourth field is four lines in that mechanism and
nothing else.

**The alternative was `state.json`, and it is rejected on one structural
ground.** Not on cost — `state.json` is cheaper, and #78's entry is right that
it is the cheap one. It is rejected because **splitting one sheet's four pickers
across two files gives that sheet two write-back paths**, and the next person
adding a picker to it has to guess which. The three that are there set the
precedent, and consistency inside one control group is worth more than one
avoided version bump.

**It is also the better semantics, narrowly.** "Does this match go to YouTube"
is a property of the match more than of the laptop: a coach filming for one club's
channel and another's parents' WhatsApp wants them to differ, which `state.json`
cannot express. That argument is *secondary* — the sticky-last-used shape makes
either home work in practice — and it is stated second so nobody mistakes it for
the load-bearing one.

**The field takes no serde attribute.** `Preferences` carries
`#[serde(default)]` on the **container** (`project.rs:126-133`) and fills from
its hand-written `Default` impl (`:165-181`), so a field-level one would be a
second copy of the default — CLAUDE.md's rule, and the same call v11 and v13
already made for their `Preferences` fields.

**S3. What a `Preferences` bump actually costs — and the doc comment that
overstates it.**

`bus/state.rs:8-11` says a new `Preferences` field is *"a format change that
`store::read`'s exact-version guard would make every existing project unreadable
for"*, and `CLAUDE.md`'s speech-model paragraph says *"every existing project
fails `store::read`'s version guard on"*. **There is no exact-version guard.**
`read` accepts
`MIN_READABLE_FORMAT_VERSION..=CURRENT_FORMAT_VERSION` — `found < MIN` is
`LegacyProject` and `found > CURRENT` is `TooNew` (`store.rs:124-137`) — so a v14
build reads every v7–v13 file, and the container default fills the new key. **No
existing project becomes unreadable.**

The real cost, in full:

1. `CURRENT_FORMAT_VERSION` 13 → 14 (`store.rs:21`); `MIN_READABLE_FORMAT_VERSION`
   stays 7, since the field is additive.
2. One new test, `a_v13_file_loads_under_the_current_version`, on
   `a_v12_file_loads_under_the_current_version`'s shape
   (`tests/project_format.rs:439`) — CLAUDE.md's every-bump rule.
3. **The forward direction, which is the one that actually bites:** the first
   save re-stamps the project to v14 (`store.rs:194`), after which 0.11.x refuses
   it as `TooNew`. Mitigated already — that same save keeps `project.json.v13`,
   once, never overwritten (`store.rs:182-192`, pinned by
   `an_upgrade_keeps_the_old_file_once`, `:732`).
4. **Not** the eighteen-file edit of BACKLOG #106. That is a **`Clip`** field's
   cost. `Preferences` is constructed as a literal in exactly one place in the
   whole tree — its own `Default` impl — so the field costs one line there and
   nothing in any test.

Correcting both copies is in scope because that sentence is what this whole
decision was going to be made against, and it is currently the app's written
reason for sending a setting to `state.json`. The *conclusions* it was used to
reach are all still right — the speech model, the pen, the recents and the panel
widths are all the coach's rather than the match's — so nothing moves file; only
the price changes.

**S4. What #100's per-field read bought, and what it did not.** BACKLOG #100
landed 2026-10-02 (`BACKLOG.md:1922-1964`): `#[serde(default)]` moved to
`State`'s container and a `lenient` deserializer went on each field
(`bus/state.rs:53-54`, `:121`). It matters here because (B)'s settings go to
`state.json`, so the next three options are priced against it.

**It buys:** a new `state.json` key is **one attribute**, and a value this build
cannot read costs **that key alone** rather than the last project, the pen, the
speech model, the window size and the panel widths.

**It does not buy, and these are the shapes to know before choosing a type:**

- **One malformed *element* still costs a whole list**, because a `Vec` fails
  whole. So an option stored as a **set** — #102's "which kinds of mark to snap
  to", if it ever becomes one — loses the whole set to one bad entry, where a
  bool or a label-string loses only itself. #102's defensible first cut (match
  events only, a bool) sidesteps it; a set is a thing to decide with eyes open.
- **Lost updates are untouched.** Every setter is `read` then `save` over the
  whole document and there are two `AppFiles` handles — the bus's
  (`bus/mod.rs:669`) and `main.rs`'s `machine_state` (`:380`) — so a bus-side
  write interleaving with a UI-side one still loses a field. The write is a temp
  file and a rename, so nothing tears, and the cost is one option's value. **Not
  fixed, and no machinery for it**: stated so that a Settings sheet writing
  several keys from the UI thread while the bus writes a recent project is
  understood to cost at most one of them, and so nobody later claims #100 retired the class.
- **The container defaults on `WindowSize` and `PanelWidths` are still doing a
  different job** (rescuing a partial object), so a new *struct* in `state.json`
  needs one of its own. A new scalar does not.

### U. Where the UI lives

**U1. This app has no settings screen, and the absence is a pattern rather than
a gap.** Every configurable thing in it is reached beside the thing it
configures — and four of these are machine-wide values with no screen at all:

| Setting | Where its control is | Stored |
|---|---|---|
| the pen's colour | the drawing row's swatches | `state.json` |
| the pen's width | two dots beside the swatches (`app.slint:5692-5696`) | `state.json` |
| which speech model runs | the inspector's transcript row (`app.slint:904-921`) | `state.json` |
| camera, mic, avatar | the Devices popover | `Preferences` / the project |
| resolution, quality, scoreboard | the export sheet | `Preferences` |
| a clip's inset size and corner | the clip inspector | `Clip` + sticky `Preferences` |
| the panels' widths, the folds, the window's size | dragged / clicked directly | `state.json` |

**#116 is the sharpest precedent and it shipped this morning** (`b6b93e8`): a
machine-wide value, a two-value picker *at the thing it affects*, read back by
label so an unknown value reads as the default, written through the UI thread's
own `AppFiles` (`main.rs:2343-2350`, wired at `:409`) with **no bus command**,
because the bus has nothing to hold. It needed no screen, and the entry that
proposed one for it was answered without building it.

So the bar for a settings sheet is not "there is an option" — it is **"this
option has no thing to sit beside."** Two of #78's dependents clear that bar
(**§D1**, **§D2**); #78's own content does not.

**U2. The chapters switch is a `CheckBox` on the export sheet, under the
Scoreboard row.**

```text
Scoreboard  [ Default                      ▾ ]
            The whole match is copied, not re-encoded: …
[x] Chapters, in the file and as a list beside it
```

- **Its own row, not a fourth column.** The Scoreboard picker already took its
  own row for the stated reason that *"'Burned into the picture' doesn't fit a
  third of this sheet"* (`app.slint:1769-1771`); a 480px card
  (`app.slint:1728`) has no fourth column.
- **A `CheckBox`, not a two-item `ComboBox`.** Three combo boxes and a checkbox
  reads as "three choices and one switch", which is what it is. `Auto-clear`
  (`app.slint:5736-5739`) is the house's own checkbox idiom.
- **The label says both halves**, because the switch governs both and the coach
  can only see one. "Chapters" alone would read as the sidecar.
- **`enabled: !root.exporting`**, as all three pickers are: a run's settings are
  settled for it.
- **It is sticky, so it is a once-ever decision in practice.** This is what makes
  a per-export control the right place rather than a tax: set it once and every
  later export inherits it, by the same write-back that already does this for
  resolution and quality.

**How "off" reaches media: by blanking the data media already reads, not by a new
flag.** `job()` (`bus/export.rs:349`) clears `compilation.plan.chapters` when the
switch is off, and **media is untouched**. Both readers already handle empty:
`splice` returns `Written(0)` without opening the file (`media/src/chapters.rs:158`),
and `chapter_list` returns `None`, which makes `write_chapter_list` remove a
stale list (`composite/export.rs:511-521`). The `bus: exported …` line
(`bus/export.rs:250`) already reports that outcome; it is what a single-clip
export does today.

**This is `carry_scoreboard`'s own rule, reused rather than reinvented.** CLAUDE.md:
*"Track mode blanks `job.scoreboard` rather than carrying a mode flag into media:
`None` is already media's one 'don't draw the board', so there is no third state
to keep consistent and `overlay.rs` never learns a picker exists."* An empty
chapter list is already media's one "no chapters". A `bool` on `ExportJob` would
be the third state that rule exists to refuse.

**U3. The Settings sheet is designed here, and built by its first tenant rather
than by #78.**

(B) needs somewhere, and there is no honest way for #78 to build it: #78's own
content does not belong in it (**U2**), so #78 would ship a sheet holding one
checkbox that should be somewhere else, or a sheet holding nothing. **A container
built before its contents is the thing this spec exists to prevent.**

So: **the design is settled here, and whichever of #84 or #102 lands first builds
it** — one sheet, one control, on a shape the app already has six of. That costs that entry almost nothing and costs #78 nothing at all, and
it is what #78's "with its home already decided" asks for.

**U4. If and when it is built: the seventh modal, on the shared `Sheet`.**
Specified now so it is not re-litigated, and so the first tenant does not invent
it.

- **A `Sheet`, like the other six.** `app.slint:1661-1662` names them — export,
  the basket, New match, match setup, the match event editor, the error dialog —
  and all six are built on the one `Sheet` component (`:1666`; match setup wraps
  it in a `Rectangle` only so its colour picker can float over the card,
  `:2504-2509`, `:2656-2660`). Not a panel: the two side columns already hold
  Sources, Clips, Match and Highlights and only grow (#87, #113), and a settings
  column would be permanently on screen for something touched twice a year. Not
  a popover: the Devices and Recent popovers are *lists you pick from and
  dismiss*, where a settings surface has fields that are typed into — and a
  `PopupWindow` has no `editing` to fold into `text-editing`, which is exactly
  the machinery a typed field needs (**below**).
- **Reached from the window's menu-less toolbar, and the transport row cannot
  take it.** That row is measured full: *"at the window's 1100px minimum,
  sharing the transport's row pushed Export and Devices off its end and left the
  notice no width at all"* (`app.slint:5641-5644`), against a 1100px minimum
  (`:3214-3215`), and the recents work already spent its last slack on
  `Recent ▾`. **So the button goes on the drawing row, beside `Fit`** — which is
  where that comment's own precedent put the overflow, and `Fit` is there for
  exactly this reason. This is an **estimate, not a measurement**, and it is on
  the manual list.
- **The Esc contract, which is the one thing a sheet with fields gets wrong.**
  A sheet whose fields can hold focus must fold its own `editing` into the
  window's `text-editing` (`app.slint:3783`), or the first Esc closes the sheet
  and throws away what is half-typed — CLAUDE.md's rule, and the reason the
  basket and New match sheets each carry the fold (`:1894`, `:3047`). A
  settings sheet with a path or a key in it is in that class **from its first
  field**, which is the single biggest reason it is a `Sheet` and not a popover.
- **It is gated like the other sheets:** `!root.recording && !root.previewing`.
  Nothing in it touches a running export, so no busy guard in the bus.
- **It holds `state.json` values only**, written through `machine_state` on the
  UI thread as the pen width and the panel widths are. A setting the **bus** must
  read is read through its own handle (`bus/mod.rs:669`), as the whisper model
  already is — **no new command**, unless the bus has to *act* on the change.

**U5. No keyboard shortcut, and no menu bar.** Every letter in this window is a
global binding, and which ones are rebindable is #96's question, not this one.
There is no menu bar to put a Preferences item in, and adding one for a single
item is a second navigation model.

### D. The dependents, and what each actually needs

**D1. #102 (snap the scrubber to events) needs one `state.json` bool and a
control — and there is a shape that needs neither.**

What it needs from a settings surface is genuinely small: one bool — one
attribute, under #100 (**S4**) — and one checkbox. The marks are already on the
scrubber — `Mark { at, color }` at absolute timeline seconds
(`scrubber.slint:14-19`, `:30`), built from the match events in
`main.rs:1691-1707` — and the drag that would snap is one `TouchArea`
(`scrubber.slint:108-126`). So #102 is a small build **plus** a home for its
switch, and today that home is the whole of its blockage.

**The shape that frees it: snap always, with a modifier to suppress.** Shift-drag
scrubs raw; a plain drag snaps. Every timeline editor does this, it is
discoverable by accident, it needs no stored value, no screen and no #78 — and it
is reversible mid-gesture, which a checkbox in a sheet is not. **Recommended to
the coach (§Open questions).** If they want the switch anyway, #102 is the
Settings sheet's first tenant and builds it per **U4**.

**Either way #102 is unblocked by this document**, which is the point: what it
was waiting for was a decision, not a screen.

**D2. #84 (music under a goals reel) is the one entry that genuinely needs the
sheet — and not for the reason its own entry gives.**

Its entry says the sheet is *"where the key field lives"*, and the key is
**optional**: the same entry's 2026-09-25 decision is Openverse, *"anonymous
queries need no key"*, with Jamendo-direct as an upgrade. So the API key is not
a blocker.

**What is a blocker is the local music folder** — the entry's shape (a), *"the
mixer's only input"*, decided to be built first. A folder path is machine-wide
(it is where the coach keeps their music, not a property of a match), it cannot
be a picker beside anything, and a file-chooser button needs a surface. That is
a real settings field, and #84 is big enough to carry building the sheet for it.

**What does *not* go in the sheet:** the genre pick. That is an export choice —
*"pick rock or EDM"* on the reel's export — and it belongs on the export sheet
with resolution and quality, for **U1**'s reason.

**D3. #115 (the caption bar, switchable off) is NOT #78's, and folding it in
would undo a decision the coach already took.**

Its entry lays out three homes and records the answer (`BACKLOG.md:2386`): *"The
coach answered (same day): 'per clip or in general'"* — explicitly **not** the
export sheet, and explicitly the shape #88 shipped for the inset's size and
corner: a `Clip` field with an inspector control, plus a sticky
`Preferences` pair that seeds the next recording. That is **U1**'s pattern
exactly, and it is already specified in the entry down to the serde attributes.

It also is not a bool — off, whole entry, or the first few seconds — and the
timed variant is a real change in the overlay. Nothing about it gets easier for
being near a settings screen, and a settings checkbox would be a fourth place
the bar could be switched.

**#115's "needs #78" line is removed.** It is independent and buildable now.

**D4. #116 (the pen's width) is NOT #78's, is already shipped, and the backlog's
own entry is stale.** `b6b93e8` shipped it this morning as a two-value picker
beside the swatches, in `state.json`, with no screen. The entry still reads as
open (`BACKLOG.md:2551`, and the top-of-file list at `:27`); that is a
bookkeeping miss, not a decision, and it is corrected with #78's own resolution.
It is kept in this spec as **U1**'s best evidence.

**D5. #96 (every hot key reassignable) is adjacent and must not be folded in.**
A key map is a table with capture, conflict detection and a reset — its own
surface, not a row in a settings sheet — and #96 is open on the coach's own
terms. If the Settings sheet ever exists, a keys **tab** in it is a reasonable
future; a tab-less sheet with a key map poured into it is not.

### X. What must not go in it

This is the half of the spec that earns its place. Each of these is a candidate
that has been named — three of them by #78's own entry — and each is refused with
its reason.

**X1. The header tags.** #78 names them (*"whether it writes the file tags"*).
Refused:

- **The complaint was files, not metadata.** The coach's words name the `.srt`
  and the chapter track — things that appear in a folder. `title`, `comment`, `keywords`, `date` and `encoder` are
  boxes in `moov/udta` (`core::metadata::FileTags`, `metadata.rs:86-103`); no
  coach has ever seen one without running `ffprobe`.
- **They cost nothing** — measured, and written down in CLAUDE.md: *"not a
  sample changes"*, and the `free` box the chapters eat into *"stayed exactly
  842 bytes"*.
- **Where a tag can't be told the truth it is already left out** (`metadata.rs`,
  and CLAUDE.md's rule): no scoreboard, no `comment`; no teams, no `keywords`.
  The honest cases are already handled, so a switch would only let a coach ship
  files that say nothing about themselves.

A switch here is a control with no symptom behind it, which is the definition of
a settings screen feature.

**X2. The reel's lead-in and tail.** #78 names them (*"possibly the reel's
default lead-in and tail (20 s / 6 s today, constants)"*). Refused, and this one
is not close: **they already have a per-goal control.** `REEL_LEAD_IN` and
`REEL_TAIL` (`reel.rs:30`, `:34`) are only the *defaults*; each goal stores its
own override in `MatchEventRecord::reel_lead_in` / `reel_tail`
(`scoreboard.rs:219-221`), and the Match panel's goal rows carry the trim
buttons and the reel span beside them (`app.slint:1286-1326`). A global default
beside a per-goal override is **two ways to say one thing, free to disagree** —
and CLAUDE.md's rule for the reel is explicit: *"Never replace them with a guess
that could be shorter: a cut-off assist is the one failure the reel must not
have."* A coach who can lower the global default can give every untrimmed goal
that failure in one click.

**X3. The avatar's pulse constants.** #78 names them. Refused outright: they are
six tuning constants (`PULSE_GROWTH`, `PULSE_FLOOR_DB`, `PULSE_CEILING_DB`,
`PULSE_ATTACK`, `PULSE_RELEASE`, `PULSE_RATE` — `avatar.rs:20-36`) shared by
**two estimators of one quantity** (the live one at `dt = 0.1`, the rendered one
at `1/30`), and CLAUDE.md's rule for them is *"same constants, nothing
persisted"*. Exposing them makes them a stored value that the two estimators can
disagree about, for a visual effect nobody has asked to change. If the pulse is
wrong, the constant is wrong, and that is a commit.

**X4. Anything that already has a control.** Resolution, quality, the scoreboard
mode, the pen, the pen's width, the speech model, the camera, the mic, the
avatar, the panel widths, the folds, the preview and scan volumes. A settings
sheet listing these would be a second place each can be changed. **This is the
rule that keeps the sheet small, and it is stated as a rule because every one of
them is individually plausible.**

**X5. Anything that is a property of one clip.** The inset's size and corner
(v13), `show_pip`, the caption bar (#115), a clip's tags and notes. CLAUDE.md's
line between `project.json` and `state.json` is about *who* a value belongs to,
and a per-clip value belongs to the clip; a machine-wide default for it takes the
sticky-`Preferences` shape #88 shipped, which needs no screen.

**X6. The export's output directory.** Plausible, unasked, and not free: the
directory is `open.folder.join(EXPORTS_DIRNAME)`, created on demand after every
refusal (`bus/export.rs:341`, `:355-357`), and both sidecars hang off the output
path it builds (`job.path.with_extension(…)`). A configurable one has to answer
what happens when it is missing, unwritable or on another filesystem — and the
project-is-a-folder convention says an export belongs in the project. Refused
because nobody has asked, not on principle.

**X7. A "reset to defaults" button, an import/export of settings, or a
`settings.json` of its own.** Everything lives in two files that already exist,
and both already fall back to their own written defaults on anything they cannot
read — `lenient` per field in `state.json`, the container default in
`Preferences`. Deleting the file *is* the reset, and a third settings file would
be a third thing to keep in step with the two that are correct.

**X8. A format version for any of (B).** Everything in **§U3**'s sheet is
`state.json`. `CURRENT_FORMAT_VERSION` moves **once**, for **S2**'s one field,
and not again for the sheet.

---

## Crate responsibilities

| Crate | Contents |
|---|---|
| `pundit-core` | `Preferences::last_export_chapters: bool`, `true` in the `Default` impl, **no serde attribute** (`project.rs:126-181`); `CURRENT_FORMAT_VERSION` 13 → 14 (`store.rs:21`), `MIN_READABLE_FORMAT_VERSION` unchanged. Nothing else — `plan::CompilationPlan::chapters` and `chapters::chapter_list` are untouched, because the switch is expressed as an empty list, which both already mean. |
| `pundit-media` | **Nothing.** `splice` already returns `Written(0)` for an empty list (`media/src/chapters.rs:158`) and `write_chapter_list` already removes a stale file when `chapter_list` says no (`composite/export.rs:511-521`). **If this table ever grows a media change, the design in §U2 has been abandoned.** |
| `pundit-app` | `bus/mod.rs`: `chapters: bool` on `Command::Export` and its dispatch arm (`:334-342`, `:1029-1034`). `bus/export.rs`: the field on `Pickers` and in `Pickers::of` (`:540-556`) — including its doc comment, which says *"The export sheet's three pickers"* (`:536-538`) — the write-back (`:360-370`), and one line in `job` clearing `compilation.plan.chapters` when it is off (`:349`). `bus/state.rs`: the header's `store::read` claim corrected (**S3**) — a documentation fix, no code. UI: one `CheckBox` and one `in-out property <bool>` on `ExportSheet` (`app.slint:1704-1788`); `main.rs` passes it with the other three. |
| `pundit-harness` | That an export with the switch off writes no `.chapters.txt` and removes a stale one, and that one with it on still writes it. |

**No new module, no new file, no new command, and no new UI surface.**

**`CLAUDE.md` changes in three places**, all of them paragraphs that already
exist and none of them a new section:

- the **format-rules list** gains a `v14` bullet, on v11's and v13's shape —
  the field, that `Preferences` takes **no** attribute, and that the readable
  floor stays 7;
- the **export-sheet paragraph** (*"The export sheet's third picker is
  Scoreboard…"*) gains the fourth control and **W4**'s one-switch-both-forms
  rule, beside `carry_scoreboard`'s own "blanks the field rather than carrying a
  mode flag into media", which it is a second instance of;
- the **speech-model paragraph**, per **S3**, loses the "every existing project
  fails" price and keeps its conclusion.

**`bus/state.rs`'s header** takes the matching correction. Both are
documentation; no code moves for them.

## Testing

**No test writes outside a `tempfile::tempdir()` and no test reads the coach's
folders.**

- **`pundit-core` (`tests/project_format.rs`):**
  - **`a_v13_file_loads_under_the_current_version`**, on
    `a_v12_file_loads_under_the_current_version`'s shape (`:439-476`): serialize
    `sample_project`, remove `preferences.lastExportChapters`, stamp
    `formatVersion: 13`, read, assert the field is `true` from the container
    default, assert `project.json.v13` exists after the save, assert the file is
    re-stamped to `CURRENT_FORMAT_VERSION`. The literal `13` is what fails if
    the constant never moved — that test's own stated reason (`:429-437`).
  - `preferences_defaults_are_not_zero` (`:143`) **amended** rather than joined:
    it is already the test that a `Preferences` field's default is the written
    one and not `Default::default()`, and a `bool` whose real default is `true`
    is exactly its subject.
- **Harness (`pundit-harness/tests/export.rs`):**
  - **An All Clips export with the switch off writes no `.chapters.txt`, and
    removes one already at that path.** The removal is the half an
    implementation can miss, because "off" takes the same branch a single clip
    already takes.
  - **With it on, the same export writes one** — so the test pair proves the
    switch and not just the absence.
  - **The in-file chapters follow it**: `ExportDone::chapters` is
    `Written(0)` with the switch off and `Written(n)` with it on. This is what
    pins **W4**'s "both forms, one switch"; without it the two halves can drift.
  - **The write-back**: an export with the switch flipped stores it in
    `Preferences`, and the next sheet open reads it back. Amends the existing
    picker write-back test rather than adding a fourth.
  - **A refused run stores nothing** — the existing rule
    (`bus/export.rs:360-362`: the write-back is after `begin`, so nothing on the
    way to a refusal dirties the project). Amended, not added.
- **Not tested, deliberately:** that `splice` and `chapter_list` handle an empty
  list. Both are already covered by the single-clip path and by
  `core/tests/chapters.rs`; a third assertion of the same rule through a slower
  path earns nothing.
- **Manual (batched):**
  - export a whole match with the switch off and confirm `exports/` holds the
    `.mp4` alone;
  - confirm the export sheet still fits its 480px card with the fourth row, at
    the 1100px window minimum — **an estimate, not a measurement**;
  - confirm **W3** by hand, which is the claim this spec most depends on:
    Scoreboard → *Burned into the picture* on a whole match writes no `.srt`
    and removes one already there.

## Risks

1. **W3 is a code reading, and the whole "two already have a switch" conclusion
   rests on it.** `carry_scoreboard` is one function and its three `cues` arms
   are explicit (`bus/export.rs:590`, `:602`, `:628`), but the coach's actual
   experience of it has not been checked — they may have had a `.srt` appear on
   a run where they believed they had chosen *Burned*, which would mean the
   picker is not discoverable as the off switch even though it is one. **On the
   manual list, and it is the check that could change this spec's answer** (to:
   the Scoreboard picker's explanatory line should say the `.srt` goes with it).
2. **One switch covering both forms of chapters may be one switch too few.** A
   coach who wants `chpl` for their own mpv playback but no text file beside the
   video cannot have that. **Accepted**: the in-file box is invisible and free,
   so the combination is implausible, and two checkboxes for it would be worse
   than the gap. Revisit only if asked.
3. **A version bump for a bool will look disproportionate to the next reader.**
   It is: one field, one test, and a project that an older build then refuses.
   **S2** states the structural reason (one sheet, one write-back path) rather
   than leaving it to be re-derived, and **S3** prices the bump honestly so the
   trade is visible. The reader who disagrees has everything they need to.
4. **The Settings sheet, designed and unbuilt, is a design that can go stale.**
   If neither #84 nor #102 lands for months, **U4** is a specification of a
   component against an `app.slint` that has moved. Accepted: it is forty lines
   of guidance against a `Sheet` component that has been stable across six
   users, and the alternative — building it now — is the container-before-contents
   mistake this spec refuses.

## Deferred

1. **The Settings sheet itself** (**U3**, **U4**) — built by #84 or #102,
   whichever lands first.
2. **A chapters switch on the basket sheet.** That sheet has its own two
   pickers, stored as string labels in `basket.json` (basket spec H1's reason),
   and a basket film gets chapters one per piece. Adding a third picker there is
   a second storage path for the same question and is not what the coach asked
   about. Revisit when a basket film's sidecars are complained about.
3. **Separating the `.srt` from the embedded `tx3g` track** (**W3**). One
   decision today, deliberately. Revisit if a coach names the combination.
4. **A keys tab** (**D5**, #96).
5. **Splitting the Scoreboard picker's explanatory line to mention the `.srt`**
   — depends on Risk 1's manual check.

## Open questions for the coach

1. **#102: a switch, or Shift to suppress the snap?** (**D1**.) The modifier
   needs no stored value, no screen and no #78, is reversible mid-drag, and is
   what every timeline editor does. A switch is what was asked for and is a
   small build once a sheet exists. **Recommended: the modifier.** It decides
   whether the Settings sheet is built soon or waits for #84, which needs it
   either way (**D2**).
2. **Does "turn the chapters off" mean the text file alone, or the chapters
   too?** (**W4**, Risk 2.) This spec says both, on the grounds that the in-file
   box is invisible. Worth one sentence of confirmation, because it is the only
   place the switch does more than was literally asked.
3. **Which files were in `exports/` when #78 was filed, and which of them were
   unwanted?** (**W3**, Risk 1.) The entry names *"the srt file gen, the other
   chapter track"*, and this spec reads that as the two sidecars beside a whole
   match — but if the `.srt` was there on a run where *Burned into the picture*
   was chosen, then the problem is the picker's discoverability rather than a
   missing setting, and the fix is a word in its explanatory line instead of a
   checkbox.

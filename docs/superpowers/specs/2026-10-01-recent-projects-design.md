# Recent projects: a popover that switches between them

**BACKLOG #85.** The coach (2026-09-24): *"'recent projects' menu or similar?
also could have a project drawer to switch between recents? good for working
with several project and going back and forth."*

They have three tagged matches across two clubs and move between them. Today
every switch is **Open Project…** and a folder picker — two clicks, a dialog, and
a walk down a tree whose folders are called `20260917-canfield`.

**The shape is decided** (the coach, 2026-10-01, asked against three options): a
**popover off a toolbar button**, beside `New match…` and `Open Project…`. One
click to open, one to switch, and it dismisses. Not a sidebar panel — that column
already holds Sources and Clips and only grows (**#87**) — and not a seventh
`Sheet`, which dims the window and needs dismissing for something done to go back
and forth.

**This is mostly storage.** The switch itself already exists and is already
all-or-nothing: `Bus::open_project` (`bus/project.rs:35`) reads first and only
then commits the folder and the project together (`:46-47`), so **no new command
and no new refusal** — with one exception this spec takes on deliberately
(**O4**). What is new is a list where there is one path today, and a popover to
show it.

**Revised 2026-10-01 through both adversarial passes.** Two design bugs and
seven wrong citations came out of them; **§R** keeps the record.

## Goal

Switching between the matches the coach is working on costs one click, and the
list says what each project *is* rather than what its folder is called.

## Scope

In: the stored list, its rows, the popover, the toolbar button, the per-field
`state.json` read (**#100**), and the busy guard `open_project` turns out not to
have (**O4**).

Also in, because this change invalidates them: **`CLAUDE.md`'s `state.json`
paragraph**, which states the all-or-nothing contract verbatim and enumerates the
file's contents — **S2** retires its reasoning and **S1** lengthens its list — and
**BACKLOG #100 and #85**, both resolved. A stale rule about which serde attribute
goes where is the kind that produces a wrong field-level `#[serde(default)]` on an
`f64` six months from now.

Out: everything in **X**.

---

## Decisions

### S. Storage

**S1. `last_project` grows into `recent_projects: Vec<PathBuf>`, in
`state.json`, and `lastProject` is read once to seed it.**

`state.json` already holds the last opened folder (`bus/state.rs:36`). It is
machine state, not a project's: which matches *this laptop* has been in says
nothing about the match, so it is `state.json` and not a format change
(`bus/state.rs`'s header has the rule).

**It is written in two places today**, and the order of this spec's own sections
is the reason that matters: `commit` (`bus/project.rs:301`, `Some(&folder)`) and
the failed restore (`:75`, `None`). **E2 removes the second**, and only then is
`push_recent_project` the sole writer. That dependency is stated rather than
asserted away — the first draft claimed "written in exactly one place", which was
false, and used it as the premise for this whole section.

**One field, not two.** `recent_projects()` is the list and `last_project()`
becomes `recent_projects().first().cloned()` — derived, and kept because
`restore_last_project` (`bus/project.rs:67`) and the New match flow's W2 tier
(`main.rs:543`) both want exactly "the head". Keeping `lastProject` written *as
well* is the mistake the New match review pass removed from
`Command::NewMatch`: two fields holding one value, free to disagree.

**This choice is cheap and the evidence is countable:** because
`last_project()` survives as the derived head, **every existing `last_project()`
assertion in the repo keeps passing untouched** — `transcribe.rs:585`,
`project_and_sources.rs:109`, `:165`, `:205`, `:396`, `new_match.rs:222` — and
exactly **one** existing test in the tree changes (**E2**).

**`set_last_project` is deleted**, not left with no callers.
`push_recent_project(folder)` replaces it. A surviving setter would be a second
way to write one field, and nobody could say what it meant (write a one-element
list? replace the head? truncate the rest?). Its four in-crate test callers
(`bus/state.rs:464`, `:467`, `:518`, `:534`) and one in `bus/basket.rs:664` move
to the new setter.

**The migration is on the VALUE, not the key.** `recent_projects()` falls back to
`lastProject` whenever the parsed list comes back **empty, from any cause**. The
first draft said "a file with no `recentProjects`", which does not fire for the
likeliest hand-edit of all: a document holding a good `lastProject` *and* a
malformed `recentProjects`. Under **S2** a malformed field is indistinguishable
from a missing one, so keying on absence would have lost the last-project pointer
— the exact symptom #100 exists to stop.

**The `last_project` struct field stays, as a read-only seed**, with a doc
comment saying so: it is never written, and removing it would break the upgrade
it exists for. It is **dated**, on `state::adopt_old_name`'s pattern — a new
BACKLOG entry deletes it once no installation predating this version is left,
the shape `CLAUDE.md` already uses for #93.

**S2. `AppFiles::read` becomes per-field, which is BACKLOG #100 — an independent
change that lands first because the backlog says so, not a prerequisite of this
feature.**

`read` (`bus/state.rs:254`) gives up on any `serde_json` error for the whole
document and returns `State::default()`, so **one bad value costs the last
project, the pen, the speech model, the window size and the panel widths.** #100
measured five shapes that still do it after #87's container defaults.

**The first draft argued this was a prerequisite, and that argument was wrong.**
It claimed *"a list is a strictly bigger target than a path"* — but
`"lastProject": 5` already costs the whole document today, so the list adds more
malformed *shapes* and no new risk class. And a recents list is **derived,
self-healing state**: losing it costs one folder-pick, which is exactly what
`bus/state.rs`'s header already prices as the cost of this whole file. The
feature is safe without #100.

What is true is simpler: **#100's trigger is "anything added to `state.json`",
and this adds something.** Two other entries are already routing around it
(#96, #102). It lands first so the two can be reviewed apart.

**The mechanism is a lenient field deserializer, not a hand-written
`serde_json::Map` walk.** A Map walk would put every key name in `read` as a
camelCase string literal beside a `#[serde(rename_all = "camelCase")]` struct
that already declares them — a second copy of the schema with nothing keeping it
in step, which is the hazard `CLAUDE.md` names for `Preferences`. Instead, one
helper and one attribute per field, leaving `struct State` as the single schema:

```rust
/// Any value this build can't read falls back to the field's default, so one
/// bad value costs that field and not the document (BACKLOG #100).
fn lenient<'de, D, T>(d: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de> + Default,
{
    let value = serde_json::Value::deserialize(d)?;
    if let Ok(parsed) = T::deserialize(&value) {
        return Ok(parsed);
    }
    eprintln!("bus: ignoring an unreadable state.json value: {value}");
    Ok(T::default())
}
```

with `#[serde(default, deserialize_with = "lenient")]` on each field of `State`.
The `default` is still needed: `deserialize_with` is not called for a key that is
**absent**. Every field type already satisfies `Default`.

Three things about it that are easy to get wrong:

- **The container `#[serde(default)]` on `WindowSize` and `PanelWidths` stays.**
  It rescues a *partial object* — `{"panels":{"sidebar":400}}` keeps 400 — which
  is finer-grained than the lenient wrapper and a different job. Both lose a
  whole `panels` object when one of its fields is bad. Both hand-written
  `Default` impls also stay, which is what stops a per-field
  `unwrap_or_default()` resurrecting the zero-height hazard `bus/state.rs`'s own
  comment warns about.
- **It moves the loss from the document to the field, and no further.** One
  malformed *element* still costs the whole list, because `Vec<PathBuf>` fails
  whole. That is the basket's own bargain for `pieces` (`bus/basket.rs:75-78`
  says so out loud) and it is accepted for the same reason: a re-pick, not a
  format change. Reading the list as `Vec<serde_json::Value>` to keep the good
  strings is machinery for a hand-edit and does not earn its place.
- **It does not retire lost updates.** Every setter is `read()` then `save()`
  over the whole document, and there are two `AppFiles` handles — the bus's and
  `machine_state` (`main.rs:370`). A per-field read fixes *parse* robustness and
  nothing else: a bus-side push interleaving with a UI-side `set_panel_widths`
  loses one field. The write is a temp file and a rename, so nothing tears, and
  the cost is one list entry. **Not fixed** — stated so that nobody later builds
  machinery for it, and so "retires the whole class" is not claimed for
  something that does not.

**S3. The cap is 8, which is an estimate and is on the manual list.** A row is
two lines on `DeviceChoice`'s 28px, so ~40px, so eight rows is ~320px standing
over the player — but **nothing has been measured, because nothing has been
built**, and the bar's own height at the 700px window minimum
(`app.slint:3089`) is not stated anywhere in the tree. The first draft called
this "the popover's measurement, not a guess", which is how a wrong number gets
into a plan. The cap of 8 stands on its own merits — the coach has three
projects, a season has twenty-odd, and a list you scroll is a picker, which they
already have.

**S4. Entries are canonical where the kernel could say so, de-duplicated by
`==`.** `commit` stores `folder.canonicalize().unwrap_or(folder)`
(`bus/project.rs:300`), with `open_project` having made it absolute first
(`:36`). So `==` is the whole de-duplication and re-opening a project moves it to
the head. Where `canonicalize` fails — EACCES on a component, a mount dropped
between the read and the commit — a non-canonical path is stored and the list can
hold two entries for one project. **No normalization of our own**: new machinery
for a case that costs one duplicate row. (`bus/basket.rs` already makes the
stronger claim about the same value; this spec does not repeat it.)

**Entries are absolute by construction**, and a hand-edited relative one resolves
against the **process** working directory — in the row's read, in `store::read`
and in `open_project`'s `std::path::absolute` — so the row and the open agree
with each other and neither agrees with the coach. Exactly as `last_project`
already behaves. Nothing to build.

### E. What is in the list, and what takes something out of it

**E1. An entry is added on `commit` and nowhere else.** `commit` has four call
sites (`bus/project.rs:47`, `:55`, `:72`, `:154`), covering `OpenProject`, the
folder-with-no-`project.json` creation, `RestoreLastProject` and `NewMatch`; and
`self.open` is assigned in exactly one place, inside it (`:316`). So every way a
project becomes the open project goes through it and there is no second place to
keep in step.

**This overrides #85's own description**, which says the write belongs *"in
`open_project`, which is the one place a project is opened."* It is not —
`new_match` commits too.

**A refused open pushes nothing**, and that is already pinned:
`opening_a_folder_that_does_not_exist_errors_and_creates_nothing`
(`project_and_sources.rs:378`) asserts `last_project()` is `None` at `:396`,
which under a list becomes the guarantee that a refused open adds no entry.

**E2. A project that fails to open is NOT removed. This change is forced by S1,
not chosen for the coach's convenience.**

Once `last_project()` is the derived head of `recent_projects()`, there is no
coherent meaning for `set_last_project(None)` at `bus/project.rs:75` — pop the
head? clear the list? The setter stops making sense, so the call has to go
whatever one thinks about removable drives. The first draft justified this purely
on the cloud mount and then conceded in a risk that it could be called the other
way; the forcing argument makes that concession unnecessary.

It is *also* the right behaviour, for the reason the draft gave: the coach's
projects live on a cloud-sync mount, and a drive not mounted at launch would
otherwise drop a project they still have, on one failed read, silently. The entry
stays and the popover greys it (**D2**).

The cost is honest: a project genuinely deleted sits greyed until it falls off the
end of eight. Better than the alternative, and it is *information*.

**The test this changes is
`restoring_a_folder_that_no_longer_exists_does_not_create_it`
(`project_and_sources.rs:187`), whose assertion is at `:203-208` and reads "a
folder that can't be restored is forgotten".** The first draft cited `:396`,
which is a different test about a different command and which E2 must **not**
touch. What does not change: a failed restore still leaves the UI in its
no-project state, and still logs. Only the forgetting goes.

**E3. Nothing else removes an entry.** No "clear recents", no per-row remove.
Eight entries rotate on their own, and a list the coach has to curate is a list
with a management UI — **Deferred 1**.

### D. What a row shows

**D1. Two lines: what the match *is*, then the folder it is in.**

The name is `core::metadata::match_label` (`metadata.rs:195`) — the teams, else
the project's own name, else `Untitled` — the function the basket's text bar and
chapters already use, and whose doc already says why it is not the folder name:
*"the coach's folders are called things like `20260917-canfield`, which names
nothing a viewer knows."*

**The second line is the folder's own file name, and the collision it resolves is
two matches against the same opponent.** A New match project is named
`<Home> v <Away>` and carries those teams, so `match_label` is **identical** for
every match against that opponent — the normal case across a season — and the
folder, which that flow names `<date>-<home>-<away>`, is what tells them apart.
Not the whole path: long, identical in every row, and the coach knows where their
projects are.

The first draft justified this line with a case that does not arise: it claimed a
project whose teams are not set up has a `match_label` that *is* its project
name, so two such projects could read identically. In fact such a project's name
**is its folder name** (`open_project` builds `Project::new(folder.file_name())`,
`bus/project.rs:49-53`, pinned by `project_and_sources.rs:103`), so the draft's
own two examples differ on line one already — and each renders the same string
twice, which is the one case where the second line is pure noise.

**So the second line is shown only when it differs from the first.** One `if` in
Slint, and the duplicated-string row goes.

**D2. A row is resolved by reading its `project.json` when the popover opens —
except the open project's row, which comes from the snapshot already in hand.**

Reading beats storing a name beside the path for one sufficient reason: a stored
name goes stale the moment the coach renames a project or sets the teams up.
(Greying a missing row needs a filesystem check either way, so once you are
reading, the stored name earns nothing.)

**The open project's row is the snapshot's**, not a read. The UI thread already
holds `Snapshot`, which carries the `Arc<Project>` and the canonical folder, so
the label is free — and this is the basket's own rule, written down at
`bus/basket.rs:236-248`: *the open project from memory, every other from
`store::read`*. It is not only cheaper but more correct: `project_and_sources.rs`
pins that a rename stands in memory while a failed save leaves the old name on
disk, so a re-read would show the popover disagreeing with the window title.

A row whose project cannot be read — gone, unreadable, legacy or too new — shows
the folder name alone, dimmed. It is not an error: the popover is a list, not an
operation.

**The first draft cited the New match sheet as precedent for eight reads, and
that citation argued the opposite.** The quote is real — *"the same order of work
as opening any folder in a file manager"* — but prices **one** read, and the very
next sentence rejects the many-read version: *"The draft's histogram read every
`project.json` in the folder, which is the version that would have been felt."*
See **Risk 1**, which now states the cost rather than borrowing comfort from a
sentence that denies it.

**D3. The ticked row is the one whose path equals the open project's folder — if
there is one. Not "the head".**

The draft said the open project is at the head, *"so leaving it out would mean
the list disagreeing with the field"*. **E2 makes that false:** after a launch
whose restore failed, the head is a project that is not open, because `commit`
was never reached and nothing else moved it. The same holds on a first launch and
on a refused `pundit <folder>` run, where there is no open project at all. An
implementer following "tick the head" would put a tick and a disabled state on a
greyed, unreachable row — and the one row the coach most needs to click, to retry
once the drive is mounted, is the one that would refuse.

So the tick is an `Option<&Path>` match, and every other row is clickable
including the head.

**The ticked row is not clickable, and the reason is load-bearing rather than
cosmetic.** Clicking it would send `OpenProject` on the already-open folder, and
`commit` clears `self.history` and calls `clips::empty_trash` (`clips.rs:325`) on
both the outgoing and incoming folders — so a re-open of the open project
silently and permanently destroys the coach's undo stack and the clip recordings
in it. Without that written down, "the head row might as well be clickable, it's
a harmless no-op" is a plausible simplification someone makes later.

### P. The popover

**P1. A `PopupWindow` off a `Recent ▾` button in the transport row, immediately
right of `Open Project…`** — the third of the three ways into a project, beside
the other two.

**That row is already full, and this spec prices it rather than assuming.** It is
one `HorizontalLayout` holding twelve items — New match…, Open Project…, Add
Source Video…, Play, the speed button, Record, the stretch readout, "Volume", the
volume slider, Export…, Basket…, Devices… — whose only slack is that readout,
which carries a hard `min-width: 130px`. And the comment immediately after it
records the measurement that matters: *"at the window's 1100px minimum, sharing
the transport's row pushed Export and Devices off its end and left the notice no
width at all."* One ~95px button is far less than the drawing controls that
measurement was about, so this is expected to fit — **and it is a manual check at
the 1100px minimum (`app.slint:3088`), with the fallback named in advance:** if
it overflows, the three project buttons move to the drawing row, which is where
that comment's own precedent put the overflow.

**The chrome is `devices-popup`'s, and the position is NOT.** `devices-popup` is
`x: parent.width - self.width - 8px`, right-anchored — correct for `Devices…`,
which sits at the row's right end, and ~1000px wrong for a button at the left
end, because the bar is full width. So the `PopupWindow` is declared **as a child
of the `Recent ▾` button**, which is how this file already attaches `Tooltip`s,
with `x: 0; y: -self.height - 8px`. What is copied from `devices-popup` is the
`Rectangle` of `Palette.background` with the house border and radius.

**The button's handler is `keys.focus(); recent-popup.show();`** — in that order,
as `Export…` and `Basket…` both do and `Devices…` does not. `show()`
unconditionally takes focus from whatever had it, so without the explicit call a
half-typed clip name, tag or editor row commits as a *side effect* of opening a
popover. The draft claimed the `app.slint:879` caveat did not apply because there
is "no field in play"; a field can be in play, since a click on a toolbar button
does not by itself take focus from one.

**P2. Gated as `Open Project…` is:** `enabled: !root.recording`. Plus
`!root.previewing`, which that button is missing and **O4** adds to both.

**P3. No keyboard shortcut, deliberately.** Every letter in this window is a
global binding, `New match…` and `Open Project…` have no key between them, and
**#96** is where rebindable hot keys get decided. A key for this one and not its
two siblings would be an inconsistency invented by whoever built it. **#99** is
the related gap and is also #96's.

**P4. Dismissal is `PopupWindow`'s own**, and there is genuinely nothing to
build: `show_popup` takes the window's focus item, so the window's `keys`
`FocusScope` is not focused while the popover is up, `handle-key` never runs, and
Escape falls through to Slint's own popup close. Focus is restored on close.
There is **no `editing` to fold into `text-editing`** and so no Esc cascade to
get wrong — which is the machinery every `Sheet` in this app needs and this does
not, and most of why the popover is the smaller build.

**P5. Two empty cases, both one dimmed line** in `Palette.alternate-foreground`,
which is `DeviceList`'s own idiom for "Looking for devices…"
(`app.slint:446`):

- **Nothing in the list** — a first launch.
- **Nothing in the list resolves**, which is **E2's whole scenario**: on the
  morning the coach's drive is not mounted, the popover would otherwise open
  showing eight grey unclickable rows and no explanation. One line is the
  difference between "my projects are gone" and "my drive isn't mounted."

With exactly one entry — the open project — the list is that row, ticked.

### O. Opening one

**O1. A click sends `Command::OpenProject(folder)` and nothing else.** No new
command. The bus's open is already read-first-then-commit-together, *because*
macOS set the folder before reading and the next autosave wrote the old project
over the file it had just refused (`bus/project.rs`'s header). Its refusals are
already modal and already tested.

**O2. The switch clears the undo history and empties both projects' trash**,
through `commit` — permanently deleting clip recordings the outgoing project had
in undo. This is existing behaviour of every open, pinned by
`clips.rs`'s `opening_empties_the_trash_and_the_history`.

**What changes is not the frequency but the absence of a confirmation.** The
folder picker was an implicit one: a dialog, a tree walk, a double-click on a
named folder. A row in an eight-row popover is one click with a neighbour 40px
away, and a misclick now costs the current project's undo history and trash with
no dialog and no undo.

**Accepted, and stated out loud rather than inherited.** To lose anything the
coach must have deleted a clip in this session and not undone it; and the
alternative — refusing the click, or confirming it, when the trash is non-empty —
puts a modal into the one flow whose whole point is that it has none. The real
answer is for deleted clips to survive a project switch at all, which is a
different and larger question: **Deferred 5**.

**O3. The popover closes on the click, not on `ProjectOpened`.** The opposite of
the New match sheet (spec C5), for the opposite reason: there is nothing typed
into this popover to lose, and the bus's refusals are modals that would draw over
it. A refused open leaves the coach where they were, with the popover shut and a
dialog to read. Slint's default close policy already gives this.

**O4. `open_project` gains `refuse_if_busy()?`, and both buttons gain
`!root.previewing`.** Scoped in deliberately, as its own task with its own test.

`open_project` calls no busy guard, while its sibling `new_match` opens with one
for a reason it writes down: *"An export or a preview must not have the project
swapped underneath it."* So today a running export keeps rendering over the
project the coach just left — and `commit` runs `empty_trash` on that project's
`recordings/.trash` while it does. The UI gating has the matching gap:
`Open Project…` tests `!root.recording` only, where `Export…` and `Basket…` also
test `!root.previewing`.

**This flow is what makes a latent gap reachable**: its whole purpose is to make
switching cheap. One line in the bus and one condition per button, and
`OpenProject` then refuses exactly as `NewMatch` does.

### X. What this does not do

- **It does not add a format version.** No field on any stored struct, so
  `CURRENT_FORMAT_VERSION` stays at 13 (`store.rs:21`).
- **It does not touch a project.** It reads `project.json` to label a row and
  otherwise only sends an existing command.
- **It does not manage the list** — no clear, no pin, no per-row remove
  (**Deferred 1**) — and it is not a second window listing projects (the "drawer"
  reading the coach offered and did not choose).
- **It does not scan for projects.** The list is what has been opened, not what
  exists.
- **It does not probe or load anything on hover.** A row is a name and a folder;
  a thumbnail or a duration means `probe`, which blocks for up to ten seconds a
  file on the UI thread (the New match spec's S4 has the argument).

---

## Crate responsibilities

| Crate | Contents |
|---|---|
| `pundit-core` | **Nothing new.** `metadata::match_label` is already the one way a match is written down; `store::read` (`store.rs:97`) already answers "can this project be read". |
| `pundit-media` | **Nothing.** |
| `pundit-app` | `bus/state.rs`: `lenient` and the per-field attributes (**S2**), `recent_projects` with its empty-list fallback, `push_recent_project`, `last_project` as the derived head, and `set_last_project` **deleted**. `write`'s doc comment changes subject: *any* path in the list being non-UTF-8 fails the whole document, so that open is not remembered — self-limiting, since the next setter re-reads a clean file. `bus/project.rs`: `commit` pushes; the failed restore stops forgetting (**E2**); `open_project` gains `refuse_if_busy` (**O4**). A new `pub mod recents;`: resolving a stored list into rows. `main.rs`: the popover's rows, read through `machine_state` (`main.rs:370`) — which exists so the window's *geometry* is written through it, so this is a **new use** of that handle and not its stated purpose. UI: the button, the `PopupWindow` as its child, and `!root.previewing` on two buttons. |
| `pundit-harness` | That `commit` pushes, and that a failed restore keeps the entry. |

**The rules live in the app library, not `main.rs`**, which is wiring and has no
`#[cfg(test)]` module at all — `fit`, `match_panel`, `new_match`, `drawing` and
`zoom_input` are the established pattern.

**`recents::rows` calls `store::read` directly** —
`rows(paths: &[PathBuf], open: Option<(&Path, &str)>) -> Vec<Row>` — and is
tested over a `tempfile::tempdir()`. **No injected reader.** The draft specified
one, citing `new_match`'s environment injection, but that injects what a test
*cannot* control (`$HOME`, the year); the directly comparable function,
`lent_scoreboard`, takes a `&Path` and calls the real `store::read` with tests
that build real folders. And a stub makes the one valuable test impossible: the
rule worth pinning is that **the refusal's kind does not matter** — gone,
unreadable, legacy, too new all look the same to a list — which against a stub is
one assertion written four ways over your own mock, and against the real reader
is four honestly different `StoreError` variants.

## Testing

**No test writes outside a `tempfile::tempdir()` and no test reads the coach's
folders.**

- **`bus::state` (unit, in-crate):**
  - **One table-driven test over #100's five measured shapes** — `"panels":"wide"`,
    `{"sidebar":-5}`, `{"sidebar":1.5}`, `"panels":null`,
    `"window":{"height":-1}` — asserting the *other* fields survive. Quoted from
    #100 rather than invented. Four of the five take the same path under a
    per-field read, so one loop is the honest shape, and it is the house idiom.
  - The same table for the new field: `"recentProjects": "x"`, a list with a
    non-string element, `"recentProjects": null`.
  - **The cross-case, which is the one the draft could not have caught:** a
    document with a malformed `recentProjects` **and** a good `lastProject` still
    restores the last project. This is what the value-based migration buys.
  - The migration: `lastProject` alone reads as a one-entry list; both present
    prefers `recentProjects`; neither is empty.
  - `push_recent_project`: a new path goes to the head; an existing one **moves**
    rather than duplicating; the cap drops the oldest.
  - A partial `panels` object still keeps its good field, so the container
    defaults are not quietly lost to the new wrapper.
- **`pundit-app::recents` (unit, over a tempdir):**
  - rows in list order, with `match_label`'s name and the folder's file name, and
    the second line absent when it equals the first;
  - **four deliberately broken folders — absent, `{ this is not json`,
    `{"formatVersion": 6}`, `{"formatVersion": 99}` — are four rows with no name
    and not clickable**, not four dropped rows;
  - the open project's row is labelled from the argument, not from disk, and is
    ticked and not clickable;
  - **no open project at all, and a head that does not resolve: the head is
    greyed, unticked and *clickable*** (**D3**'s bug).
- **Harness (`tests/recents.rs`):**
  - open A, then B, then A, and the list is `[A, B]` — which proves at once that
    `commit` pushes, that the newest is the head, and that a re-open moves rather
    than duplicates;
  - **a failed restore keeps the entry** (**E2**) — the one behaviour change, so
    it gets the test that would have failed before;
  - **an open refused while an export runs** (**O4**), the run unaffected and no
    entry pushed.
  - The cap is **not** re-tested here: it is a pure rule already pinned in the
    unit tests, and the harness version would cost nine real folders and nine
    full `OpenProject` round trips — each of which unloads, clears history,
    empties two trashes and republishes — to prove it a second time through an
    expensive path.
  - `NewMatch`'s push **amends the existing assertion** at `new_match.rs:222`
    rather than adding a test; it already asserts `last_project()` after a
    `NewMatch`.
- **Manual (batched):**
  - open the popover against the real tree and confirm each row names the match
    rather than the folder, and that two matches against the same opponent are
    tellable apart;
  - **confirm the transport row still fits at the 1100px window minimum**
    (**P1**), and that eight rows stand over the player at the 700px minimum
    (**S3**) — both estimates, neither measured;
  - switch between two projects twice and confirm the order follows;
  - rename a project's folder and confirm its row greys rather than vanishing.

## Risks

1. **Up to seven reads on popover open, on the UI thread.** The open project's row
   is free (**D2**), so a full list is seven small reads. **Accepted, not
   mitigated.** The draft proposed checking `project.json` exists before reading,
   which buys nothing and is worse: on an unresponsive mount `exists()` blocks in
   the same place `read_to_string` does, on an absent folder both return `ENOENT`
   at once, and it is **exactly what `store::read` deliberately rejected** —
   *"Map NotFound on the read itself rather than testing `exists()` first: one
   syscall, no TOCTOU window, and the distinction is made in one place"*
   (`store.rs:98-99`). Honest comparison: the New match sheet reads one file on
   open and its spec declined the read-every-project version. Seven few-KB reads
   on a local disk are sub-millisecond; on a stalled mount they are seven stalls.
   If it is ever felt the answer is **Deferred 2** (resolve on the bus), not a
   cached name.
2. **#100's fix touches every reader of `state.json`** — the last project, the
   pen, the speech model, the window size and the panel widths. It is small but
   it is load-bearing, so it is gated by the five shapes above and the existing
   suite, not by inspection.
3. **One-click switching makes the New match flow's own W2 weakness more
   common.** That spec's Risk 1 is *"a coach who works on two clubs and was last
   in the other one gets the other club's folder"* — and this feature is a machine
   for making the head flap between clubs. Nothing to build: W3's editable field
   with its provenance line is already the answer, and `projects_dir_for` already
   falls a stale head through to `Proposed` (`new_match.rs:119`, pinned at
   `:593`). Noted because the draft claimed the sharing was cost-free.

## Deferred

1. **Managing the list** — clear, pin, per-row remove. Eight rotate on their own
   and the coach has not asked.
2. **Resolving rows on the bus.** If Risk 1 is felt, the rows become something the
   bus publishes rather than something the UI reads. Strictly more machinery for a
   popover that opens in well under a millisecond today.
3. **A key for it**, with `New match…`'s and `Open Project…`'s — **#96**'s
   (**P3**).
4. **Showing the date.** `naming::parse_date_in` on the folder name would give a
   row a real date for free, but a project made before the New match flow has no
   date in its folder, so some rows would have one and some not — which reads
   worse than none having one. The second line already delivers it for New match
   folders, which are named `<date>-…`.
5. **Deleted clips surviving a project switch** (**O2**). `commit` empties the
   outgoing project's trash because its undo history is being cleared and the
   trash is then unreachable. Making trashed clips outlive a switch would need
   somewhere for them to be restorable *from*, which is a feature, not a fix.
   Revisit the first time a coach loses one.
6. **Deleting the `last_project` seed field** (**S1**), once no installation
   predating this version is left — the shape #93 uses for the rename shims.

## Open questions for the coach

None blocking. The shape question was asked and answered (**the popover**). Four
manual checks are batched, two of which are the unmeasured numbers this spec is
careful to call estimates.

---

## R. What the first draft got wrong, kept as the record

Two adversarial passes (simplification, correctness) ran in parallel on the
draft. **Both opened with the same class of problem: citations.** Seven were
wrong, and two of those would have sent the plan at the wrong artefact. The
design bugs were found independently of each other.

**Two design bugs:**

1. **D3's "the open project is at the head" is false the moment E2 lands.** After
   a failed restore the head is a project that is not open, so "tick the head"
   would tick a greyed, unreachable row and disable the one row the coach needs
   to retry. The tick is now a path match against an `Option`.
2. **S1's migration keyed on the absent key, which a per-field read cannot
   distinguish from a malformed one.** A document with a good `lastProject` and a
   bad `recentProjects` would have lost the pointer — the symptom #100 exists to
   stop, reintroduced by the fix for it. The fallback is now on the empty *value*.

**Seven wrong citations:**

| The draft said | Actually |
|---|---|
| `project_and_sources.rs:396` pins the failed-restore forgetting | `:396` is in `opening_a_folder_that_does_not_exist…` (`:378`), a different command; the pin is `:187`, assertion `:203-208`. **A plan would have edited the wrong test.** |
| The New match spec priced this read cost as acceptable precedent | Its quote prices **one** read and the next sentence rejects reading every project as *"the version that would have been felt."* **The citation argued the opposite of the claim.** |
| `last_project` is "written in exactly one place" | Two: `bus/project.rs:301` and `:75`. True only after E2. |
| A project with no teams set up has a `match_label` that collides | Such a project's name **is** its folder name (`bus/project.rs:49-53`), so the draft's own examples differ on line one. The real collision is two matches against one opponent. |
| `machine_state` "exists so the UI thread can read this file" | `main.rs:367-369` says it exists so the window's *geometry* is written through it. **Inherited** — the New match plan made the same claim and it was repeated without checking. |
| The popover is `devices-popup`'s shape "exactly" | Its position is right-anchored to a full-width bar, ~1000px from this button. Only the chrome transfers. |
| `bus/project.rs:33` for read-then-commit | `:33` is a doc-comment line; the function is `:35` and the sequence is `:46-47`. |

**Three arguments that were wrong rather than merely imprecise:**

- **#100 was presented as a prerequisite** on the grounds that "a list is a
  strictly bigger target than a path". `"lastProject": 5` already costs the whole
  document, and a recents list is self-healing state. It lands first because the
  backlog's trigger fired, which is a smaller and true claim.
- **Risk 1's mitigation was a no-op** that contradicted `store.rs:98-99` by name
  and would have added a syscall and a TOCTOU window.
- **`recents`' injected reader** was justified by a precedent that injects
  uncontrollable environment, and it would have made the one test worth writing
  impossible.

**Four things the draft left for an implementer to invent**, now specified: the
popover's anchor, `keys.focus()` before `show()`, the transport row's budget, and
whether `set_last_project` survives (it does not).

**And two it priced as measurements when nothing had been measured** — the cap's
320px and the transport row's fit. Both are now called estimates and are on the
manual list. That is this repo's standing rule about numbers, and the draft broke
it twice in its own voice.

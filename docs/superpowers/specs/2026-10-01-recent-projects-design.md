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
all-or-nothing: `Command::OpenProject` reads first and commits the folder and the
project together (`bus/project.rs:33`), so **no new command, no new refusal, and
no new risk to an open project**. What is new is a list where there is one path
today, and a popover to show it.

## Goal

Switching between the matches the coach is working on costs one click, and the
list says what each project *is* rather than what its folder is called.

## Scope

In: the stored list, its rows, the popover, the toolbar button, and the
`state.json` read that makes adding a field safe (**#100**, whose own "when to
revisit" is this moment).

Out: everything in **X**.

---

## Decisions

### S. Storage

**S1. `last_project` grows into `recent_projects: Vec<PathBuf>`, in
`state.json`, and `lastProject` is read once to seed it.**

`state.json` already holds the last opened folder (`bus/state.rs:36`), written in
exactly one place — `Bus::commit` (`bus/project.rs:301`), which is the one place a
project becomes the open one. A recents list is that field grown, written where it
is written now. It is machine state, not a project's: which matches *this laptop*
has been in says nothing about the match, so it is `state.json` and not a format
change (`bus/state.rs`'s header has the rule).

**One field, not two.** `recent_projects()` is the list and
`last_project()` becomes `recent_projects().first().cloned()` — a derived
accessor, kept because `restore_last_project` (`bus/project.rs:67`) and the New
match flow's W2 tier (`main.rs:543`) both want exactly "the head". Keeping
`lastProject` written *as well*, so a downgrade finds it, is the mistake the New
match review pass just removed from `Command::NewMatch`: two fields holding one
value, free to disagree. A downgrade costs one re-pick, which `bus/state.rs`'s
header already prices as the cost of this whole file.

**The migration is one-way and happens at read.** A file with no
`recentProjects` but a `lastProject` reads as a one-entry list; the next open
rewrites the document with the new key, and the old one goes. Hand-readable,
and no code path writes both.

**S2. `AppFiles::read` becomes per-field first (BACKLOG #100), and that comes
before the new field, not after.**

`read` (`bus/state.rs:254`) gives up on any `serde_json` error for the whole
document and returns `State::default()` — so **one bad value costs the last
project, the pen, the speech model and the window size**. #100 measured five
shapes that still do it after #87's container defaults, and names the fix: parse
to a `serde_json::Map`, take each key independently, keep the default for
whatever fails.

This is a prerequisite rather than a nice-to-have, for a reason this spec can
state precisely: **a list is a strictly bigger target than a path.** `"recentProjects": "x"`,
a list with one non-string element, or a list of nulls each cost the whole file
today, and the list is the field most likely to be hand-edited (it is the one a
coach would prune). #100's own "when to revisit" is *"next time anything is added
to `state.json`"*. It is also why `CLAUDE.md` says the basket lives in its own
file — and fixing it is what makes a second file unnecessary here.

**S3. The cap is 8, and it is the popover's measurement, not a guess.** A row is
two lines (**D1**), about 40px; eight rows is ~320px, which stands over the
player without scrolling at the window's minimum height. The coach has three
projects today and a season has twenty-odd, so the cap is doing real work and
should not be large enough to need a scroll view — a list you scroll is a
picker, and they already have one of those.

**S4. Entries are canonical paths, de-duplicated by exact equality.** `commit`
canonicalizes before it stores (`bus/project.rs:300-301`), so every path in the
list is already canonical and `==` is the whole de-duplication: re-opening a
project moves it to the head rather than adding a second row. No path
normalization of our own, and nothing compares a symlink to its target.

### E. What is in the list, and what takes something out of it

**E1. An entry is added on `commit` and nowhere else.** That is where
`set_last_project` is called today, and it is reached by `OpenProject`
(including the folder-with-no-`project.json` case, which creates one),
`RestoreLastProject` and `NewMatch`. Every way a project becomes the open project
goes through it, so there is no second place to keep in step.

**E2. A project that fails to open is NOT removed, and this changes shipped
behaviour.** `restore_last_project` today forgets the path on any failure
(`bus/project.rs:75`, `set_last_project(None)`), and a test pins it
(`project_and_sources.rs:396`). Under a list, forgetting is the wrong call and
the coach's own setup is why: **their projects live on a cloud-sync mount.** A
drive not mounted at launch, or a folder not yet synced, would drop a project
the coach still has — on one failed read, silently. The entry stays and the
popover greys it (**D2**), so an unplugged drive costs a grey row for an hour
instead of a list entry forever.

The cost is honest and small: a project genuinely deleted sits greyed until it
falls off the end of eight. That is better than the alternative, and it is also
*information* — "I did have that match open" is a true thing the coach may want
to see.

The failed-restore test therefore changes, and the reason goes in its name. What
does **not** change: a failed restore still leaves the UI in its no-project
state, and still logs. Only the forgetting goes.

**E3. Nothing else removes an entry.** No "clear recents", no per-row remove.
Eight entries rotate on their own, and a list the coach has to curate is a list
with a management UI — see **Deferred 1**.

### D. What a row shows

**D1. Two lines: what the match *is*, then the folder it is in.**

The name is `core::metadata::match_label` (`metadata.rs:195`) — the teams, else
the project's own name, else `Untitled` — which is the function the basket's text
bar and chapters already use, and whose doc already says why it is **not** the
folder name: *"the coach's folders are called things like `20260917-canfield`,
which names nothing a viewer knows."* One function decides how a match is
written down, here as everywhere else.

The folder's own file name is the second line, smaller and dimmed, because it is
the disambiguator: two of the coach's projects are `20260917-canfield` and
`2016B vs Hudson 2026-09-19`, and a project whose teams are not set up yet has a
`match_label` that *is* its project name — so without the folder, two
not-yet-set-up projects can read identically. Not the whole path: it is long, it
is the same prefix for every row, and the coach knows where their projects are.

**D2. A row is resolved by reading its `project.json` when the popover opens,
not by storing a name beside the path.** #85 offers both. Reading is right for
three reasons, in order: a stored name goes stale the moment the coach renames a
project or sets the teams up; reading is the only way to know a project is
**missing**, which is what greys the row rather than letting a click fail; and
there is already a precedent at this exact cost — the New match sheet reads a
neighbouring `project.json` when it opens, and its spec priced that as *"the same
order of work as opening any folder in a file manager."*

A row whose project cannot be read — gone, unreadable, legacy or too new — shows
the folder name alone, dimmed, and **is not clickable**. It is not an error: the
popover is a list, not an operation, and the same silence `seed_scoreboard`
keeps (`new_match.rs`) is right here.

**D3. The open project is in the list, at the head, with a tick and not
clickable.** It is what `commit` just stored, so leaving it out would mean the
list disagreeing with the field. The tick is the Devices popover's own mark
(`DeviceChoice`, `app.slint:396`), so a coach who has seen one has seen this.

### P. The popover

**P1. A `PopupWindow` off a `Recent ▾` button in the transport row, immediately
right of `Open Project…`.** `New match…` and `Open Project…` are already there,
in that order; this is the third of the three ways into a project and goes
beside them. The popover is `devices-popup`'s shape exactly — a `PopupWindow`
positioned over the player, a `Rectangle` of `Palette.background` with the
house border and radius (`app.slint:5636-5650`) — and it is opened by the
button's `clicked` with `.show()`, as the Devices button does
(`app.slint:5425`).

**`PopupWindow` is right here and the one caveat does not apply.** `app.slint:879`
explains that the tag suggestions are *not* a `PopupWindow`, because `show()`
takes focus from the field being typed into and commits it. This popover is
opened from a toolbar button with no field in play, which is the Devices
popover's situation and not the suggestion list's.

**P2. Gated exactly as `Open Project…` is:** `enabled: !root.recording`
(`app.slint`'s transport row). No new rule — `OpenProject` is off the recording
allow-list, which is deny-by-default, so a UI slip reaches a silent refusal
rather than a project swap mid-take.

**P3. No keyboard shortcut, and that is deliberate.** Every letter in this
window is a global binding (`handle-key`), `New match…` and `Open Project…` have
no key between them, and **#96** is where rebindable hot keys get decided. A key
for this one and not its two siblings would be an inconsistency invented by
whoever happened to build this. **#99** is the related gap and is also #96's.

**P4. Dismissal is `PopupWindow`'s own**, i.e. a click outside or Esc, with no
state of its own to clear. There is no `editing` to fold into `text-editing`
(**no fields**), and so no Esc cascade to get wrong — which is the one piece of
machinery every `Sheet` in this app needs and this does not. That is most of why
the popover is the smaller build.

**P5. Empty is a line, not an empty box.** A first launch has no recents; the
popover says so in one dimmed line (`DeviceList`'s own idiom for "Looking for
devices…", `app.slint:446`) rather than opening a 4px rectangle. With exactly
one entry — the open project — the list is just that row, ticked, which is
honest.

### O. Opening one

**O1. A click sends `Command::OpenProject(folder)` and nothing else.** No new
command. The bus's open is already the all-or-nothing path: it reads first and
only then commits the folder and the project together, *because* macOS set the
folder before reading and the next autosave wrote the old project over the file
it had just refused (`bus/project.rs`'s header). Every refusal it has is
already the right one, already modal, and already tested.

**O2. The switch empties the previous project's trash**, through `commit`
(`bus/clips::empty_trash`), permanently deleting the clip recordings that
project had in undo. That is existing, by-design behaviour of every open, and it
is called out here only because this flow makes opening a project *cheap* and so
makes that consequence more frequent. It is the same trade `Open Project…` has
always made.

**O3. The popover closes on the click, not on `ProjectOpened`.** The opposite of
the New match sheet (spec C5), and for the opposite reason: there is nothing
typed into this popover to lose, the click cannot fail in a way the coach would
want the list still up for, and the bus's refusals are modals that would draw
over it. A refused open leaves the coach where they were, with the popover shut
and a dialog to read.

### X. What this does not do

- **It does not add a command, a refusal, or a format version.** No field on any
  stored struct, so `CURRENT_FORMAT_VERSION` stays at 13.
- **It does not touch a project.** It reads `project.json` to label a row and
  otherwise only sends an existing command.
- **It does not manage the list.** No clear, no pin, no per-row remove
  (**Deferred 1**), and no second window listing projects (that is the "drawer"
  reading the coach offered and did not choose).
- **It does not scan for projects.** The list is what has been opened, not what
  exists; the New match flow's `projects_dir_for` walk is a different question
  with a different answer.
- **It does not probe or load anything on hover.** A row is a name and a folder;
  a thumbnail or a duration means `probe`, which blocks for up to ten seconds a
  file on the UI thread (the New match spec's S4 has the whole argument).

---

## Crate responsibilities

| Crate | Contents |
|---|---|
| `pundit-core` | **Nothing new.** `metadata::match_label` already exists and is already the one way a match is written down; `store::read` (`store.rs:97`) already answers "can this project be read". |
| `pundit-media` | **Nothing.** |
| `pundit-app` | `bus/state.rs`: the per-field read (**S2**), `recent_projects` and its one-way migration, `push_recent_project`, and `last_project` as the head. `bus/project.rs`: `commit` pushes instead of setting, and `restore_last_project` stops forgetting (**E2**). A new `pub mod recents;`: resolving a stored list into rows, which is a pure rule over `(paths, a reader)` and so testable with no `state.json`. `main.rs`: the popover's rows, read on open through `machine_state` (`main.rs:370`'s second `AppFiles` handle, which exists so the UI thread can read this file). UI: the button and the `PopupWindow`. |
| `pundit-harness` | The list's behaviour end to end over the bus: an open pushes, a re-open moves to the head, the cap holds, a failed restore keeps the entry. |

**The rules live in the app library, not `main.rs`**, which is wiring and has no
`#[cfg(test)]` module at all — `fit`, `match_panel`, `new_match`, `drawing` and
`zoom_input` are this crate's established pattern. **And the reader is injected**,
as `new_match`'s environment is and `bus/state.rs`'s `films_dir(videos, home)` is:
`recents::rows(paths, |path| …)` takes a closure that resolves one path, so every
rule below is a test with no filesystem in it.

## Testing

**No test writes outside a `tempfile::tempdir()` and no test reads the coach's
folders.**

- **`bus::state` (unit, in-crate):**
  - the per-field read: **each of #100's five measured shapes** (`"panels":"wide"`,
    `{"sidebar":-5}`, `{"sidebar":1.5}`, `"panels":null`, `"window":{"height":-1}`)
    now keeps the last project, the pen and the speech model, and loses only the
    field that was bad. These are the regression pins that matter, and they are
    quoted from #100 rather than invented.
  - the same for the new field: `"recentProjects": "x"`, a list with a non-string
    element, and `"recentProjects": null`.
  - the migration: a document with `lastProject` and no `recentProjects` reads as
    a one-entry list; one with both prefers `recentProjects`; one with neither is
    empty.
  - `push_recent_project`: a new path goes to the head; an existing one **moves**
    to the head rather than duplicating; the cap drops the oldest; and the file is
    rewritten each time.
- **`pundit-app::recents` (unit):**
  - rows come back in list order, with `match_label`'s name and the folder's file
    name;
  - a path the reader refuses is a row with **no name and not clickable**, not a
    dropped row, and the refusal's kind does not matter (gone, unreadable, legacy,
    too new all look the same to a list);
  - the open project's row is ticked and not clickable, and it is the head;
  - an empty list is an empty `Vec` and the caller's problem, not an error.
- **Harness (`tests/recents.rs`):**
  - `OpenProject` on two folders in turn leaves the second at the head and both in
    the list, read back through `AppFiles`;
  - re-opening the first moves it to the head and the list stays length two;
  - nine opens leave eight entries, oldest gone;
  - **a failed restore keeps the entry** (**E2**) — the one behaviour change, so it
    gets the test that would have failed before;
  - a `NewMatch` pushes its folder, since it reaches `commit` too.
- **Manual (batched, needs the coach's eyes):**
  - open the popover against the real tree and confirm each row names the match
    rather than the folder, and that a project whose teams are not set up yet is
    still tellable from its neighbours;
  - switch between two projects twice and confirm the order follows;
  - unplug or rename a project's folder and confirm its row greys rather than
    vanishing.

## Risks

1. **A row's read can block on a dead mount.** `store::read` is a small file, but
   eight of them on an unmounted cloud path are eight stats that can hang on the
   UI thread when the popover opens. The New match spec accepted one such read and
   said so; this is up to eight. **Mitigation, and it is cheap:** resolve a row by
   checking `project.json` exists first and reading only if it does, so a folder
   whose mount is gone costs one stat rather than an open. If it is still felt, the
   answer is to resolve on the bus and publish rows, **not** to cache names — see
   **Deferred 2**. Stated rather than discovered.
2. **The behaviour change in E2 is a behaviour change.** A coach who deletes a
   project sees it greyed for a while. Judged better than dropping a project on a
   cloud mount's bad morning, but it is a judgement and it is the one thing here a
   reasonable person could call the other way.
3. **#100's fix touches every reader of `state.json`.** It is ~15 lines in one
   function, but that function feeds the last project, the pen, the speech model,
   the window size and the panel widths — so it is gated by the five shapes above
   and by the existing suite, not by inspection.

## Deferred

1. **Managing the list** — clear, pin, per-row remove. Eight entries rotate on
   their own and the coach has not asked. Revisit if they prune `state.json` by
   hand.
2. **Resolving rows on the bus.** If Risk 1 is ever felt, the list of rows becomes
   something the bus publishes (a `RecentsChanged` event) rather than something the
   UI reads. That is strictly more machinery for a popover that opens in under a
   millisecond today, and it is the right answer only once it is needed.
3. **A key for it**, with `New match…`'s and `Open Project…`'s — **#96**'s, not
   this one's (**P3**).
4. **Showing the date.** With the New match flow naming every folder
   `YYYY-MM-DD-…`, `naming::parse_date_in` on the folder name would give a row a
   real date for free. Left out because a project made before that flow has no date
   in its folder, so some rows would have one and some not — which reads worse than
   none having one. Revisit once most of the coach's folders come from the flow.

## Open questions for the coach

None blocking. The shape question was asked and answered (**the popover**), and
every other decision above is either a precedent already in the tree or a cost
this spec has priced. The three manual checks are batched for the close-out.

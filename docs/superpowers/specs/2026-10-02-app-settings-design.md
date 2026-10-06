# App settings: what an export writes, and where a setting lives

**BACKLOG #78.** The coach (2026-09-24): *"add to the backlog an appsettings? we
don't need a screen for it yet. maybe we already have it. e.g. the srt file gen,
the other chapter track, etc. these are general app config settings to be turned
off or on."*

Asked on **2026-10-03** which of the files in `exports/` were unwanted, he
answered *"I think all of them should be settings?"*, and chose **fully
independent** switches over leaving any of them to follow the Scoreboard picker.
This spec is written to those answers. **Its first version concluded the
opposite** — one chapters checkbox, and no settings screen — and §R keeps that
record.

Read `CLAUDE.md` first; this spec does not repeat the export, format or
settings-location rules it states.

**The design, in one paragraph.** Two switches on the export sheet, beside the
four controls already there: **Chapters** and **Scoreboard subtitles**. Each
governs one output in both the forms it takes, each is remembered in
`Preferences` by the write-back the other four already use, and each reaches
media by **blanking the data media already reads** rather than by a new flag —
so `pundit-media` changes nothing at all. Independence makes two combinations
reachable that are not today; one is "no scoreboard anywhere", and §S4 is how it
is said in words before the run rather than discovered in `exports/`. Three of
the six things #78's entry names are **not** switched, each refused in **§X** —
the half of this document that earns its place, and which the coach's *"all of
them"* makes sharper rather than moot.

---

## §W. What #78 is, and what an export actually writes

**W1. #78 is two requests under one number, answered separately.**

- **(A) What the coach asked for:** switches for what an export puts on disk.
  Decided here (**§S**, **§U1–U3**).
- **(B) What five other entries cite #78 for:** *a place for a machine-wide
  option to live* — a music folder, a snap toggle, a key map. None of those is a
  thing an export writes.

Reading them as one request is how #78 becomes a preferences system. (A) goes on
the export sheet; (B) gets a Settings sheet, **designed** in §U4 and **built by
its first real tenant**. §D says what #78 owes each citer, which is less than
each currently claims.

**W2. What an export writes today, in full.** Verified against the code; this
table is the whole of (A)'s subject matter.

| Written | Where from | Which runs write it |
|---|---|---|
| the `.mp4` | both renderers | every run |
| `chpl` chapters, inside the file | `chapters::splice` in `composite::export`'s `finish`, from `plan.chapters` | a compilation of two or more clips (`plan::entry_chapters`); the whole match's are the match's own moments, a reel's its goals |
| `.chapters.txt` beside it | `write_chapter_list` → `core::chapters::chapter_list` | any target whose chapters survive YouTube's rules — at least `MIN_CHAPTERS` (3) after the `MIN_GAP_SECONDS` (10) rule |
| `.srt` beside it | `write_sidecar`, from `job.cues` | **only** the whole match on a separate track |
| a `tx3g` subtitle track, inside the file | `composite::copy`'s third `mp4mux` pad, from the same `job.cues` | **the copy only** — the whole match on a separate track |
| header tags | `job.tags` → `composite::tags::apply`, both renderers | every run |

Two facts from that table do most of the work below.

- **`job.cues` is one input feeding two outputs, and each renderer carries what
  it can.** `Some(cues)` writes the `.srt`; the copy additionally requests the
  subtitle pad (`CLAUDE.md`: *"Only the copy carries it — the encoded path is
  unchanged"*). `Some(empty)` writes neither **and removes a stale `.srt`**.
  `None` is a target that carries no sidecar at all, so nothing at that path is
  written or removed.
- **`plan.chapters` is one input feeding two outputs too**, with exactly two
  readers, both in `finish`. `splice` returns `Written(0)` for an empty list
  *without opening the file*; `chapter_list` returns `None`, which makes
  `write_chapter_list` **remove** a stale list.

**W3. The Scoreboard picker is the board's delivery, not an off switch for these
files.** Its three values decide where the board goes *in the picture*: burned in
(forcing a re-encode), kept out of it (letting the whole match be a stream copy),
or Default, the best available. Today the subtitle outputs ride along: *Burned*
writes neither, *Separate track* writes both — a real coupling with a reason, in
that a subtitle line of a board already painted into the picture is the board
twice.

It is not the switch the coach asked for, because it cannot express either
*"keep the board out of the picture and write no sidecar either"* (a clean stream
copy with no board) or *"burn the board in and leave me an `.srt`"*. Offered the
choice of leaving them to follow the picker, making them independent, or greying
them when redundant, **the coach chose independent**. So the picker keeps its own
job and the subtitle outputs get their own switch.

The entry's *"All are 'on' today with no way to say otherwise"* is therefore
**half right**: the `.srt` and the `tx3g` have been suppressible since 0.4.0, but
only as a side effect of a control about something else, and not without moving
the board into the picture.

---

## §S. The switches

**S1. One switch per output, and each governs every form that output takes.**

| Switch | Governs | Default |
|---|---|---|
| **Chapters** | `plan.chapters` — the `chpl` box inside the file *and* `.chapters.txt` beside it | on |
| **Scoreboard subtitles** | `job.cues` — the `.srt` beside the file *and* the `tx3g` track inside a copy | on |

**Why both forms ride one switch.** A coach who turns "Chapters" off and still
finds chapters in the file has been told a half-truth, and the in-file box is the
half they cannot see. The same argument covers the cues, and `composite/copy`'s
header already states that pair as one decision (the `.srt` is *"what VLC loads
without being asked"*, the embedded track *"what survives the file being sent
on"*). Splitting either pair would be a third and fourth checkbox for a
combination nobody has named — **deferred**, not refused.

**Defaults stay on.** Both cost nothing measurable (`CLAUDE.md`: *"Tags cost the
copy no losslessness and the chapters no room"*, measured), a chapter list is the
only way those chapters reach a YouTube viewer, and the coach asked to be *able*
to turn them off — not for them to be off.

**A fourth value on the Scoreboard picker was the alternative and is rejected.**
*Default / Burned in / Separate track / None* is cheaper by a row, but it is the
*dependent* shape the coach declined, and it cannot express "burned in **and** an
`.srt`" at all — one of the two things independence was chosen for.

**S2. How "off" reaches media: by blanking the data media already reads.**
Neither switch becomes a flag on `ExportJob`, and **media changes nothing**.

- Chapters off → `job()` clears `compilation.plan.chapters`. Both readers already
  mean "no chapters" for an empty list (**W2**), and the removal of a stale
  `.chapters.txt` comes free, being the branch a single-clip export already takes.
- Subtitles off → `job.cues` is `Some(Vec::new())` for the whole match: no `.srt`,
  a stale one removed, and no subtitle pad requested by the copy (*"an empty cue
  list leaves the output with no subtitle track at all"*, `composite/copy`).

This is `carry_scoreboard`'s own rule reused. `CLAUDE.md`: *"Track mode blanks
`job.scoreboard` rather than carrying a mode flag into media: `None` is already
media's one 'don't draw the board', so there is no third state to keep consistent
and `overlay.rs` never learns a picker exists."* An empty chapter list and an
empty cue list are already media's one "no chapters" and one "no subtitles". A
`bool` on `ExportJob` would be the third state that rule exists to refuse.

**S3. The cue slot leaves `carry_scoreboard`, which gets simpler for it.** That
function currently maps the picker into the renderer *and* the cue slot, stating
two rules together. Both must survive the split:

- **`carry_scoreboard` keeps:** *a clip or a reel asked for on a separate track
  burns the board in rather than dropping it* — the picker must never lose the
  board, and a clip's cue slot is not where its board can live.
- **A new `board_cues(target, want, compilation, context)` takes:** *only the
  whole match carries the board beside the file.* Three arms, in order: not the
  whole match → `None` (a `.srt` beside a clip is the coach's own file and no
  export's business); `!want` → `Some(Vec::new())`; otherwise the cues, or
  `Some(Vec::new())` where the project has no `ScoreboardContext` to derive them
  from.

**`carry_scoreboard` then loses a parameter rather than gaining one**:
`compilation` was passed *only* for `scoreboard_cues`. It returns `Carry { copy,
scoreboard }` — exactly "how this target carries the board" — so it fits its name
better than it does today, and **the rename the 2026-10-05 amendment proposed is
not needed** (§R.5).

**S4. The two newly reachable combinations, and the one that must be said out
loud.**

- **Board burned in *and* an `.srt` beside it.** Legitimate; the coach asked for
  it knowing what it is. Note it is the `.srt` **alone**: a burned export
  re-encodes, and the `tx3g` track rides the copy only, so there is no embedded
  track to disagree with the picture.
- **Separate track with the subtitles off: no board anywhere.** Not in the
  picture, not beside the file, not inside it. Read charitably a coach may want
  exactly that — a clean copy, no board — so it is **not a state to refuse**. But
  it is the one combination where what you asked for and what you get look
  nothing alike, so the sheet says so before the run (**§U2**).

**And "Default" never arrives there silently.** `CLAUDE.md`: *"'Default' means
the best available, never a silent trade."* Default is the mode where the app
chooses, so with the subtitles off it **burns the board in** instead of copying a
film with no board on it — one condition in the `Track` arm, beside the
`can_copy` fallback already there and for the same reason. Independence is
untouched: the switch always does what it says, and what adapts is Default's own
resolution, which is the whole of what Default is for. *Separate track* chosen by
hand still gets no board, because there the coach asked.

Both extra arguments `carry_scoreboard` now takes — `with_audio` (the mute, v15)
and the subtitles bool — are there for this one reason: **Default cannot answer
"the best available" without knowing what else the run carries.**

---

## §F. The format: one bump, two fields

Both switches are `Preferences` fields riding the export sheet's existing
write-back: `Pickers::of(prefs)` reads the sheet's controls, `Command::Export`
carries them, one write-back after `begin` stores them. Two more fields are a few
lines in that mechanism and nothing else.

- **`last_export_chapters: bool`**, `true` in the hand-written `Default`.
- **`last_export_cues: bool`**, `true`. Named for `job.cues` and `core::cues`,
  the vocabulary the code already uses for exactly this pair of outputs —
  deliberately not `last_export_scoreboard_subtitles`, which reads as a qualifier
  on `last_export_scoreboard` beside it.

**Neither takes a serde attribute.** `Preferences` carries `#[serde(default)]` on
the **container** and fills from its hand-written `Default`, so a field-level one
would be a second copy — `CLAUDE.md`'s rule, and the call v11, v13 and v15 each
made. The *"never a field-level default on a `bool`"* hazard is about that second
copy resolving to `false`; here the container default is the only copy, and
`project_format.rs::preferences_defaults_are_not_zero` guards it.

**`state.json` was the alternative and is rejected on one structural ground.**
Not cost — `state.json` is cheaper and #78's entry is right that it is the cheap
one. It is rejected because **splitting one sheet's controls across two files
gives that sheet two write-back paths**, and the next person adding one has to
guess which. Four are already in `Preferences`. The semantics agree more weakly:
"does this match go to YouTube" is a property of the match rather than of the
laptop, which is the test `bus/state.rs`'s header states.

**Take the next free `formatVersion` from `store.rs`, not a number written
here.** It is **16** as of this writing (v14 is #117's `StrokeEnd`, v15 the
mute's `export_source_volume`), and **BACKLOG #115 also claims 16**. Whichever
lands first takes it, and **if they land together they share one bump** — one
version, one every-readable-version test.

**What the bump costs**, in full, because #78's entry and two load-bearing doc
comments priced it wrong:

1. `CURRENT_FORMAT_VERSION` to the next number; `MIN_READABLE_FORMAT_VERSION`
   stays 7, both fields being additive.
2. One new test on `a_v12_file_loads_under_the_current_version`'s shape —
   `CLAUDE.md`'s every-bump rule.
3. **The forward direction, which is the one that bites:** the first save
   re-stamps the project, after which an older build refuses it as `TooNew`.
   Already mitigated — that save keeps `project.json.v<old>`, once, never
   overwritten, named from the project's own stored version.
4. **Not** BACKLOG #106's eighteen-file edit, which is a **`Clip`** field's cost.
   `Preferences` is constructed as a literal in exactly one place in the tree —
   its own `Default` impl — so a field costs one line there and nothing in any
   test.

**There is no "exact-version guard", and the two docs that claimed one are
already fixed.** `store::read` accepts `MIN_READABLE..=CURRENT`, so a new build
reads every v7-onward file and the container default fills the key it hasn't got.
`bus/state.rs`'s header and `CLAUDE.md`'s speech-model paragraph were corrected
on 2026-10-02 in `65f3e47`, so **this spec owes that edit nothing**. The fact is
kept because it is the price #78's entry still quotes, and a wrong price on the
only decision #78 turns on is what sends the next three settings to the wrong
file.

---

## §U. Where the UI lives

**U1. Every configurable thing in this app is reached beside the thing it
configures, and that is the rule a new setting has to pass.**

| Setting | Where its control is | Stored |
|---|---|---|
| the pen's colour and width | the drawing row's swatches, and two dots beside them | `state.json` |
| which speech model runs | the inspector's transcript row | `state.json` |
| camera, mic, avatar | the Devices popover | `Preferences` / the project |
| resolution, quality, scoreboard, mute | the export sheet | `Preferences` |
| a clip's inset size and corner | the clip inspector | `Clip` + sticky `Preferences` |
| the panels' widths, the folds, the window's size | dragged or clicked directly | `state.json` |

**#116 is the sharpest precedent**: a machine-wide value, a two-value picker at
the thing it affects, read back by label so an unknown value reads as the
default, written through the UI thread's own `AppFiles` with no bus command
because the bus has nothing to hold. It needed no screen, and the entry that
proposed one for it was answered without building it.

So the bar for a settings sheet is not "there is an option" — it is **"this
option has no thing to sit beside."** #78's own switches do not clear it: what an
export writes is configured per export, and the export sheet is where its other
four controls and their write-back already are. Two of #78's citers *do* clear it
(**§D1**, **§D2**), which is why §U4 exists.

**U2. Two `CheckBox`es on the export sheet, under the Scoreboard row.**

```text
Scoreboard  [ Default                      ▾ ]
            The whole match is copied, not re-encoded: …
[x] Chapters, in the file and as a list beside it
[x] Scoreboard subtitles — an .srt beside the file, and a track inside a copy
[ ] Mute source audio
```

- **`CheckBox`es, not two-item `ComboBox`es**, and the sheet already has the
  idiom with its own tooltip: "Mute source audio" shipped as one with v15.
- **Each label says both halves**, because each switch governs both and the coach
  can only see one. "Chapters" alone would read as the file beside the video.
- **Their own rows.** The Scoreboard picker already took its own for the stated
  reason that *"'Burned into the picture' doesn't fit a third of this sheet"*, and
  a 480px card has no fourth column.
- **`enabled: !root.exporting`**, as the other four are.
- **A tooltip on the subtitles switch carries the general warning**: off means the
  scoreboard appears only if it is burned into the picture.

**The explanatory line under the picker gains the combination that has to be said
in words, and loses a case where it would now be wrong.**

- It reads *"The whole match is copied, not re-encoded: player highlights and pen
  drawings can't ride a copy"* whenever the whole match is ticked and the picker
  is not *Burned*. **Under §S4's Default rule that is false for
  Default-with-subtitles-off**, which burns the board in. The condition becomes
  "this run would copy": the whole match is ticked **and** (the picker is
  *Separate track*, **or** it is *Default* and the subtitles are on) — one derived
  property on the sheet, documented once, rather than the same boolean in two
  `if`s.
- A **second line** when the whole match is ticked, the picker is *Separate track*
  and the subtitles are off: this export carries no scoreboard at all — not in the
  picture, not beside the file, not inside it. Deterministic, because *Separate
  track* chosen by hand refuses rather than falling back.

The `can_copy` gate can still send *Default* back to a re-encode and only the bus
knows that, which is as true of the existing line as of the new one; the bus says
so on stderr, as today.

**U3. Nothing else moves.** No new UI surface, no new file, no new command, no new
module, and — the load-bearing one — **no change in `pundit-media`**.

**U4. The Settings sheet: now justified, designed here, built by its first
tenant.**

The first version of this spec concluded no settings screen was warranted,
because the one option in hand had something to sit beside. The coach's answer
to #102 (**an on/off setting**) retired that: there are **two** options with
nothing to sit beside — #102's bool and #84's local music folder path — and
§U1's rule survives as the test a *new* setting must pass rather than as an
argument against the screen.

**#78 still does not build it.** #78's own content belongs on the export sheet
(**U2**), so #78 would ship a sheet holding one checkbox that should be elsewhere,
or a sheet holding nothing. **A container built before its contents is the thing
this spec exists to prevent.** The design is settled here, and whichever of #102
or #84 lands first builds it. That costs that entry almost nothing, and it is
what the entry's own *"it should arrive with its home already decided"* asks for.

When it is built:

- **A `Sheet`, the seventh.** That component's own comment names the six (export,
  the basket, New match, match setup, the match event editor, the error dialog)
  and this is the shape they share. **Not a panel:** the side columns already hold
  Sources, Clips, Match and Highlights and only grow, and a settings column would
  be permanently on screen for something touched twice a year. **Not a popover:**
  a `PopupWindow` has no `editing` to fold into `text-editing`, which is exactly
  the machinery the next bullet needs.
- **The Esc contract, the one thing a sheet with fields gets wrong.** It must fold
  its own `editing` into the window's `text-editing`, or the first Esc closes the
  sheet and throws away what is half-typed — `CLAUDE.md`'s rule, and why the
  basket and New match sheets each carry the fold. A sheet with a path in it is in
  that class from its first field.
- **Reached from the drawing row, beside `Fit`.** The transport row is measured
  full (*"at the window's 1100px minimum, sharing the transport's row pushed
  Export and Devices off its end and left the notice no width at all"*) and the
  recents work spent its last slack on `Recent ▾`; the drawing row is where that
  comment's own precedent put the overflow. **An estimate, not a measurement**,
  and on the manual list.
- **Gated like the other sheets** (`!root.recording && !root.previewing`), and
  **holding `state.json` values only**, written through `machine_state` on the UI
  thread as the pen width and panel widths are. A value the **bus** must read is
  read through its own handle, as the whisper model is — **no new command**, unless
  the bus has to *act* on the change. No keyboard shortcut: every letter in this
  window is a global binding, and there is no menu bar to put a Preferences item
  in.
- **Under #100 a new `state.json` key is one attribute**, and an unreadable value
  costs that key alone. Two shapes it does not cover, so a tenant picks its type
  knowing them: a malformed *element* still costs a whole `Vec`, and a lost update
  between the two `AppFiles` handles still costs one option's value. Neither is
  machinery to add.

---

## §D. The citers, and what #78 actually owes each

Six entries cite #78. **Not one needs #78 to ship**, and every correction below
has been applied to `BACKLOG.md` in the same change as this spec.

**D1. #102 (snap the scrubber to events) — owed the sheet's design, nothing
else.** The coach answered **an on/off setting** (2026-10-03), so the
Shift-to-suppress shape this spec's first version recommended is withdrawn
(§R.4). What #102 needs is small: one `state.json` bool and one checkbox; the
marks are already on the scrubber and the drag that would snap is one
`TouchArea`. It is the sheet's natural first tenant and builds it per **U4** —
or, if it finds a cheaper home beside the scrubber itself, that is its own call.

**D2. #84 (music under a goals reel) — owed the sheet's design, and its entry was
wrong twice about why.** Its *"When to revisit: after #78, which is where the key
field lives"* was stale on both halves: the API key is **optional** (the entry's
own 2026-09-25 decision is Openverse, *"anonymous queries need no key"*), and #78
does not build the sheet. **The real blocker is the local music folder path** —
shape (a), the mixer's only input, decided to be built first. A folder path is
machine-wide, cannot be a picker beside anything, and a file-chooser button needs
a surface; #84 is big enough to carry building the sheet for it. **Not in the
sheet:** the genre pick, an export choice belonging on the export sheet with
resolution and quality, for **U1**'s reason.

**D3. #115 (the caption bar, switchable off) — owed nothing.** Its entry records
the coach's *"per clip or in general"* — explicitly **not** the export sheet and
explicitly #88's shape: a `Clip` field with an inspector control plus a sticky
`Preferences` pair seeding the next recording. It is also not a bool (off, whole
entry, or the first few seconds), and the timed variant is a real change in the
overlay that being near a settings screen does not help. **Corrected in its
entry:** its body said the bump was v15 in two places; v15 went to the mute, so
it is the next free number from `store.rs` — which its own top-of-file line
already said.

**D4. #116 (the pen's width) — owed nothing, shipped, correctly filed.** A
two-value picker beside the swatches, in `state.json`, with no screen
(2026-10-02). It cites #78 nowhere, its entry reads RESOLVED and it is not in the
open list — so this spec's first version was wrong both to count it as a citer
and to call its bookkeeping stale (§R.7, §R.8). Kept here as **U1**'s best
evidence.

**D5. #96 (every hot key reassignable) — owed the sheet's design and one rule: a
key map is not a row in it.** #96 says it *"wants #78's settings screen to land
on"* and is the entry immediately after #78 in the open order, so this matters. A
key map is a table with capture, conflict detection and a reset — its own
surface. A keys **tab** in the sheet is a reasonable future; a tab-less sheet with
a key map poured into it is not.

**D6. #88 (the inset's size and corner, closed) — owed nothing, and its line is
corrected.** It said the two new preferences *"have no control of their own,
which is #78's job, where `pip_for_new_recordings` would finally get one too"*.
They are not #78's job: `last_inset_size` and `last_inset_corner` are the
sticky **memory** of the clip inspector's own two controls, which is §X4.
`pip_for_new_recordings` is genuinely unhoused — no control at all, written only
by tests — but it is the **project's**, not the machine's, so a machine-wide sheet
is the wrong home; its control belongs beside the inset's two.

---

## §X. What must not go in it

The half of the spec that earns its place, and the coach's *"all of them"* makes
it sharper rather than moot: the question he answered was **which files in
`exports/` were unwanted**, and three of the six things #78's entry names were
never files an export writes.

**X1. The header tags.** #78 names them (*"whether it writes the file tags"*).
Refused on three grounds, the third decisive:

- **The complaint was files, not metadata.** `title`, `comment`, `keywords`,
  `date` and `encoder` are boxes in `moov/udta`; no coach has seen one without
  running `ffprobe`.
- **They cost nothing** — measured, and in `CLAUDE.md`: *"not a sample changes"*,
  and the `free` box the chapters eat into *"stayed exactly 842 bytes"*. Where a
  tag can't be told the truth it is already left out, so a switch would only let a
  coach ship files that say nothing about themselves.
- **A switch could not even deliver an untagged file on the software encoder.**
  `x264enc` pushes an `ENCODER` tag of its own into the same muxer and the merge
  mode is `Keep` precisely so ours wins; with ours blank, x264's stands. "Tags
  off" would be a control that does not do what its label says on one of the two
  encoders.

A switch here is a control with no symptom behind it. **The open question below
puts it back to the coach anyway**, because it is the only place this spec reads
*"all of them"* narrowly — and it is cheap to answer either way:
`FileTags::default()` is already an untagged file, so it would be the same
blanking move as the other two.

**X2. The reel's lead-in and tail.** #78 names them. Refused, and not close:
**they already have a per-goal control.** `REEL_LEAD_IN` and `REEL_TAIL` are only
the *defaults*; each goal stores its own override in
`MatchEventRecord::reel_lead_in` / `reel_tail`, and the Match panel's goal rows
carry the trim buttons and the reel span beside them. A global default beside a
per-goal override is two ways to say one thing, free to disagree — and
`CLAUDE.md`'s rule is explicit: *"Never replace them with a guess that could be
shorter: a cut-off assist is the one failure the reel must not have."* A coach who
can lower the global default can give every untrimmed goal that failure in one
click.

**X3. The avatar's pulse constants.** #78 names them. Refused outright: six tuning
constants (`PULSE_GROWTH`, `PULSE_FLOOR_DB`, `PULSE_CEILING_DB`, `PULSE_ATTACK`,
`PULSE_RELEASE`, `PULSE_RATE`) shared by **two estimators of one quantity**, where
`CLAUDE.md`'s rule is *"same constants, nothing persisted"*. Exposing them makes
them a stored value the two estimators can disagree about, for a visual effect
nobody has asked to change. If the pulse is wrong, the constant is wrong, and that
is a commit.

**X4. Anything that already has a control.** Resolution, quality, the scoreboard
mode, the mute, the pen and its width, the speech model, the camera, the mic, the
avatar, the inset's size and corner, the panel widths, the folds, the preview and
scan volumes. A settings sheet listing these would be a second place each can be
changed. **Stated as a rule because every one is individually plausible** — and
because a sticky `Preferences` pair behind an existing control is not an unhoused
option looking for a screen (**D6**).

**X5. Anything that is a property of one clip.** `show_pip`, the inset's size and
corner, the caption bar (#115), a clip's tags and notes. `CLAUDE.md`'s line
between `project.json` and `state.json` is about *who* a value belongs to, and a
per-clip value belongs to the clip; a machine-wide default for it takes the
sticky-`Preferences` shape #88 shipped, which needs no screen.

**X6. The export's output directory.** Plausible and unasked. The directory is
`open.folder.join(EXPORTS_DIRNAME)`, created on demand after every refusal, and
both sidecars hang off the path it builds; a configurable one has to answer what
happens when it is missing, unwritable or on another filesystem — and the
project-is-a-folder convention says an export belongs in the project.

**X7. A "reset to defaults" button, an import/export of settings, or a
`settings.json` of its own.** Every value lives in a file that already falls back
to its own written defaults on anything it cannot read — `lenient` per field in
`state.json`, the container default in `Preferences`. Deleting the file *is* the
reset, and a third settings file is a third thing to keep in step with the two
that are correct.

**X8. A format version for anything in §U4's sheet.** It is all `state.json`.
`CURRENT_FORMAT_VERSION` moves **once**, for §F's two fields, and not again for
the sheet.

---

## Crate responsibilities

| Crate | Contents |
|---|---|
| `pundit-core` | `Preferences::last_export_chapters` and `last_export_cues`, both `bool`, both `true` in the `Default` impl, **no serde attribute**; `CURRENT_FORMAT_VERSION` to the next free number, `MIN_READABLE_FORMAT_VERSION` unchanged. Nothing else: `plan::CompilationPlan::chapters`, `chapters::chapter_list` and `cues` are untouched, both switches being expressed as an empty list, which all three already mean. |
| `pundit-media` | **Nothing.** `splice` already returns `Written(0)` for an empty list, `write_chapter_list` already removes a stale file when `chapter_list` says no, `write_sidecar` already removes a stale `.srt` for `Some(empty)`, and `copy.rs` already requests no subtitle pad for an empty cue list. **If this row ever grows a media change, the design in §S2 has been abandoned.** |
| `pundit-app` | `bus/mod.rs`: two `bool`s on `Command::Export` and its dispatch arm. `bus/export.rs`: the two fields on `Pickers` and `Pickers::of` (including the doc comment that counts the sheet's pickers), the write-back, one line in `job` clearing `compilation.plan.chapters`, the new `board_cues`, and `carry_scoreboard` losing its `compilation` parameter and its `cues` arm while gaining §S4's Default condition. UI: two `CheckBox`es and two `in-out property <bool>`s on `ExportSheet`, one derived "would copy" property, one new explanatory line; `main.rs` reads both on open and sends both on Start, beside the four it already does. |
| `pundit-harness` | The switch pair, both ways, for both outputs — including the stale-file removals, which are the halves an implementation can miss. |

**`CLAUDE.md` changes in two existing paragraphs:** the **format-rules list**
gains a bullet for the new version on v11's, v13's and v15's shape (the two
fields, no attribute, the floor stays 7); the **export-sheet paragraph** gains the
two switches, §S1's one-switch-both-forms rule and §S4's "Default never trades
the board away", beside `carry_scoreboard`'s own "blanks the field rather than
carrying a mode flag into media", which both switches are further instances of.
Nothing is owed to `bus/state.rs` or the speech-model paragraph (**§F**).

## Testing

**No test writes outside a `tempfile::tempdir()` and no test reads the coach's
folders.**

- **`pundit-core` (`tests/project_format.rs`):** one new every-readable-version
  test on `a_v12_file_loads_under_the_current_version`'s shape — serialize
  `sample_project`, remove both new keys, stamp the previous version, read, assert
  both fields are `true` from the container default, assert `project.json.v<old>`
  exists after the save and that the file is re-stamped. Its literal version is
  what fails if the constant never moved, which is that test's own stated reason.
  `preferences_defaults_are_not_zero` is **amended**, not joined.
- **Harness.** Extend the existing whole-match tests, which already assert on
  `exports/`' contents and already count subtitle streams with `ffprobe`:
  - **Chapters off** on an All Clips export writes no `.chapters.txt` and removes
    one already at that path; **on**, the same export writes one. The pair proves
    the switch rather than an absence.
  - **The in-file chapters follow it**: `ExportDone::chapters` is `Written(0)` off
    and `Written(n)` on. This pins §S1's "both forms, one switch"; without it the
    two halves can drift.
  - **Subtitles off on a copied whole match**: no `.srt`, a stale one removed, and
    **no subtitle stream in the file** — the half that lives in `copy.rs`, and the
    one an implementation can get right beside the file and wrong inside it.
  - **Burned with the subtitles on writes the `.srt`** — unreachable today, and
    the cheapest proof that independence is real.
  - **Default with the subtitles off burns the board in** rather than copying. The
    one behavioural inference in the spec, so it is pinned.
  - **The write-back** stores either switch when flipped and reads it back on the
    next open; **a refused run stores nothing.** Both amend
    `a_run_persists_the_resolution_quality_and_mute` and
    `a_refused_run_leaves_the_pickers_alone` rather than adding a pair beside them.
- **Not tested, deliberately:** that `splice`, `chapter_list` and `write_sidecar`
  handle an empty list. All three are already covered — by the single-clip path, by
  `core/tests/chapters.rs`, and by `a_burned_whole_match_removes_a_stale_sidecar`.
- **Manual (batched):** export a whole match with both switches off and confirm
  `exports/` holds the `.mp4` alone; confirm the sheet still fits its 480px card
  with two more rows at the 1100px window minimum (**an estimate, not a
  measurement**); confirm the no-board-anywhere line appears for *Separate track*
  with the subtitles off and not for *Default*.

## Risks

1. **§S4's Default rule is an inference, not a coach answer.** He asked for
   independent switches; "Default burns the board in when the subtitles are off"
   is this spec reading *"Default means the best available, never a silent
   trade"* as governing. The alternative is Default copying a film with no board
   while the sheet warns about something it cannot predict (`can_copy` is the
   bus's answer, not the UI's). **Accepted**: the rule is one condition in the arm
   that already holds the `can_copy` fallback, it costs the hand-chosen modes
   nothing, and it is the only version where every line the sheet shows is true.
   Reversible in one line.
2. **One switch per output may be one too few.** A coach wanting `chpl` for their
   own mpv playback but no text file beside the video, or the embedded track but
   no `.srt`, cannot have either. **Accepted:** the in-file forms are invisible
   and free, so both are implausible, and four checkboxes would be worse than the
   gap.
3. **A version bump for two bools will look disproportionate.** It is: two
   fields, one test, and a project an older build then refuses. §F states the
   structural reason and prices the bump honestly, so a reader who disagrees has
   what they need to.
4. **The Settings sheet, designed and unbuilt, can go stale.** If neither #102 nor
   #84 lands for months, §U4 specifies a component against an `app.slint` that has
   moved. **Accepted:** it is guidance against a `Sheet` that has been stable
   across six users, and building it now is the container-before-contents mistake
   this spec refuses.

## Deferred

1. **The Settings sheet itself** (§U4) — built by #102 or #84, whichever lands
   first.
2. **Splitting `.srt` from `tx3g`, and `chpl` from `.chapters.txt`** (Risk 2). One
   switch each, deliberately. Revisit if a coach names a combination.
3. **A chapters switch on the basket sheet.** The mute went to both sheets, so the
   precedent is not that the basket is exempt — but only the chapters half could
   apply (a basket's board is burned in and its `cues` are `None` by design), and
   `basket.json` is a separate storage path with no version and a whole-document
   read. Revisit when a basket film's sidecars are complained about.
4. **A keys tab** (§D5, #96).
5. **A tags switch** — on the open question below.

## Open question for the coach

**The header tags: did *"all of them"* include those?** (§X1.) This spec switches
the two outputs a coach can see in a folder — the chapters and the scoreboard
subtitles — and leaves the `moov/udta` tags on, because they are not files, they
cost nothing, and `x264enc`'s own `ENCODER` tag means a switch could not even
produce an untagged file on the software encoder. **It is the one place his words
are read narrowly, so it is asked rather than assumed.** A yes is a third
checkbox and the same blanking move, so it is cheap either way.

Everything else this spec once asked has been answered or settled: #102's shape
is the coach's on/off (§D1); what "turn the chapters off" covers is decided by
§S1, with Risk 2 recording the cost; and which files were in `exports/` when #78
was filed no longer changes the design now that every output is switchable.

---

## §R. What the first version got wrong, and what overturned it

The first version (2026-10-02) concluded that #78 was a single chapters checkbox
and that no settings screen was justified. Both are wrong. The record, because a
spec that quietly rewrites its own conclusion teaches the next reader nothing:

**Overturned by the coach (2026-10-03):**

1. **The headline.** Asked which files in `exports/` were unwanted: *"I think all
   of them should be settings?"* So collapsing #78 to one checkbox was not what
   was asked for (§S1).
2. **The `.srt` and the `tx3g` were read as "already having their switch"** in the
   Scoreboard picker. Offered the choice of leaving them to follow the picker,
   making them independent, or greying them when redundant, he chose
   **independent** (§W3). The picker is the board's delivery, and reaching it as
   an off switch required also moving the board into the picture.
3. **"No settings screen is justified"** rested on "this option has nothing to sit
   beside". #102's answer gave the screen a second tenant, so it is justified,
   designed in §U4 — and still not built by #78.
4. **#102's shape.** The first version recommended snap-always with Shift to
   suppress. The coach answered **an on/off setting**; the recommendation is
   withdrawn.

**Claims that were wrong on their own terms, found by re-checking the code and the
backlog:**

5. **The 2026-10-05 amendment said `carry_scoreboard` should be renamed** once it
   lost the cue coupling, because `with_audio` had made its name approximate. The
   opposite: without the cue slot it returns `{ copy, scoreboard }` — exactly how
   a target carries the board — and it **loses** its `compilation` parameter,
   which was passed only for `scoreboard_cues`. No rename (§S3).
6. **"Board burned in *and* a subtitle track of it"** overstated the first new
   combination. The `tx3g` track rides the **copy** only; a burned export
   re-encodes and gets the `.srt` alone (§S4, and `CLAUDE.md`'s own *"Only the
   copy carries it"*).
7. **"#78 is cited as a prerequisite by four other entries — #84, #102, #115 and
   #116."** #116 cites #78 nowhere; #115 canvassed #78 as one of three homes and
   then recorded the coach's answer against it; and two citers were missed, **#96**
   and **#88** (§D5, §D6). Six entries name #78 and none needs it to ship.
8. **"#116's entry still reads as open, and so does the top-of-file list."** Both
   had already been corrected: the entry reads RESOLVED and #116 is not in the open
   list.
9. **The two doc corrections were listed as this spec's scope.** They landed
   2026-10-02 in `65f3e47`, so §F records the fact and owes no edit.
10. **The version was "13 → 14".** `CURRENT_FORMAT_VERSION` is **15** (v14 is
    #117's `StrokeEnd`, v15 the mute's `export_source_volume`). §F takes the next
    free number from `store.rs` rather than writing one down — the instruction
    #115's own top-of-file line already gives, and which its body contradicted
    until this change (§D3).

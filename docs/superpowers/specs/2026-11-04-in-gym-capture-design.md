# In-gym capture: record live, review, and share from a session

**Date:** 2026-11-04. **Revised 2026-11-04** after the two adversarial passes
(simplify + correctness). **Status:** Reviewed. The decisions the reviews changed
are applied below; the reasoning they rejected is kept in **§X**, not deleted.
The one decision still owed to the coach is called out in **Open questions**.
**Builds on:** the project format and versioning (`pundit-core/src/project.rs`,
`store.rs`, spec F), the recording path (`bus/recording.rs`, Phase 4 spec), the
timeline walk (`pundit-core/src/timeline.rs`), the export/preview composite
(`pundit-media/src/composite/`), the new-match flow
(`docs/superpowers/specs/2026-09-24-new-match-flow-design.md`), the export queue
(BACKLOG #77, `docs/superpowers/specs/2026-10-07-export-queue-design.md`), and
the Devices popover (`app.slint`).
**Evidence:** the code on `origin/main` at `d742430`, read 2026-11-04. Every
claim carries a `file:line` or is marked **(design)** where it proposes
something new. Backlog entry **#140** records the brainstorm this spec executes;
**#139** is the Android editor this was chosen over.

> **The app assumes footage is imported and the camera is only ever
> commentary.** This feature inverts that one assumption — the live camera
> becomes primary, scrubable footage — and then reuses everything *downstream*
> unchanged. Two mechanisms are genuinely new and carry real work: **capturing
> the camera as a source** (§C) and **a recorded playback rate** (§R). But the
> review-and-draw flow, the export, the queue and the overlay do **not** change:
> once a captured clip is an ordinary `SourceRef`, the coach records a normal
> commentary take over it, and the rest of the app can't tell it apart from
> imported footage. The review's correction to the first draft: the two new
> mechanisms are not "almost free" — they touch `start_recording`, the timeline
> walk and `core::audio`. The *downstream* claim survives; the *cheapness* claim
> did not.

---

## Goal

The coach asked for this:

> *"i'm thinking about 'in gym usage' for like tumbling instruction. we have a
> cart mounted machine with a usb camera, they can record live, and then
> annotate with the kid right there, and add it to a queue to be shared on
> google drive or something."*

And, on what "annotate" means:

> *"it is just like our review and drawing on screen. e.g. the coach records a
> clip right after."*

And on two refinements:

> *"playback speed could be added to the current clip settings. we can already
> pause and scrub during clip recording … playback speed is a natural extension
> of that."*
> *"we will need to redesign the ui a bit around touchscreen compatibility
> here."*

Today, the only way footage enters a project is `Add Source Video…` →
`Command::AddSource(PathBuf)` → `probe` → a `SourceRef` pushed onto
`Project::source_videos` (`bus/sources.rs`, `probe.rs:41-96`). The live USB
camera is captured only as a **commentary take**: `Command::ToggleRecording`
records camera+mic to `recordings/<id>.mkv` and `finish_recording` calls
`Project::add_recorded_clip`, which appends a `Clip` that **references** an
existing source and plays back *over* it (`bus/recording.rs:489-539`,
`project.rs:802-858`). There is no path from the camera to a *source*.

The gym workflow needs exactly that path, plus a project shape that isn't a
match, plus slow-motion review, plus a touch-reachable UI, plus a share step.

---

## Scope

**The coach's decisions of 2026-11-04 (from #140). This spec follows them:**

- **D-1. A new "session" project kind** — a loose bag of captured clips, with no
  scoreboard, no `<Home> v <Away>` name, no two-half aspect gate, no periods.
- **D-2. Annotation is the existing review-and-draw flow, unchanged.** Once a
  captured clip is a *source*, the coach records a commentary take over it
  exactly as over match footage. No new annotation mechanism.
- **D-3. Playback speed is a recorded value**, baked into a clip's edit so
  preview and export replay it — a generalisation of the per-segment `rate` the
  timeline already carries.
- **D-4. Sharing is a real Google Drive upload with a pending-share queue**
  (phase 2), not an export into a synced folder.
- **D-5. A bounded touchscreen pass** — give the mouse-only gestures the gym
  flow uses a tap path. Not a full tablet UI rewrite (that is #139).

**In scope:** the session kind and its creation flow; capture-as-source; the
recorded playback rate end-to-end (event → timeline → export/preview → audio);
the touch affordance for the menu-only actions; and a phase-2 sketch of the
Drive queue.

**Out of scope and unchanged:** the match project in every respect; the
scoreboard, match clock, slates, reel, basket and whiteboard (a session doesn't
use them — §V); the export composite's pixel work; the capture encoder/container
(`vah264lpenc`/`x264enc` → MKV H.264/Opus, `capture/recorder.rs`).

---

## Decisions

### S. The session project kind (format)

**S1. A `kind: ProjectKind` field on `Project`, an enum `Match | Session`,
defaulting to `Match`.** (design) Today `Project` has no type discriminant
(`project.rs:461-490`); every project is a match. Add:

```rust
pub enum ProjectKind { Match, Session }
impl Default for ProjectKind { fn default() -> Self { ProjectKind::Match } }
```

It takes a field-level `#[serde(default)]`, which is `StrokeEnd`'s v14 rule
exactly (`project.rs` header, verified against `event.rs`): an enum whose
`Default` is *what every older file means* — and every v7–v18 project is a match.
This is the sanctioned "a type whose `Default` is what an older file is" case,
not a forbidden field-level default on a `bool`; a `bool is_session` would be the
two-values-free-to-disagree smell, and would foreclose a third kind.

**S2. This is v19 and the readable floor stays 7.** (design) Additive. The bump's
reason is the **forward direction**: serde ignores unknown keys, so an older
build would read a v19 session, ignore `kind`, treat it as a match, and on its
next save drop the field — silently turning a session into a match and trying to
draw a scoreboard clock over footage that has no match time. The v18→v19 bump
makes an older build refuse the file (the version guard reads
`MIN_READABLE..=CURRENT` against the raw JSON, `store.rs`), and `project.json.v18`
is the way back. **v19 carries three additive changes together** — `kind`
(S1), `Project.created` (S3) and §R's `Play.rate` — one bump, one
`v7_to_v19_files_load_under_the_current_version` test beside its siblings in
`project_format.rs`. The first-draft "its own bump if they ship apart" hedge is
cut, because the phasing ships them in one release window.

**S3. `kind` exists for one reason: to forbid the scoreboard forever.** (design,
revised per both reviews) A session is otherwise *data-identical to a fresh empty
match* — `scoreboard: None`, empty `match_events`/`slates`/`player_highlights`
(all already `Vec`/`Option` with field defaults, `project.rs:472-489`), ordinary
`Clip`s. The *only* thing `scoreboard.is_some()` cannot express is "this project
must never acquire a board," because an empty match legitimately can. That is
what `kind` encodes. The one other session-level field is a **stored created
date** (coach, 2026-11-04): `Project.created: Option<String>`, an ISO date set by
the bus at creation (core has no clock) and shown in the recents list, so a gym
running many same-named sessions in a day can tell them apart. It is additive —
`None` is every v7–v18 file, a field-level `#[serde(default)]`, and it rides the
same v19. There is still no `periods`, no second name, no session struct.

**S4. One predicate, pinned by one test — not a growing `match kind {}`.**
(design, revised per correctness #6) Everything that varies between a session and
a match reads a **single helper**, `Project::is_match()`, at a small set of
sites: the creation flow refuses a scoreboard on a session (N), the capture
control is offered (C), the match chrome is hidden (the scoreboard setup, the
match-event editor, the reel, the clock readout — V3), and export names/targets
(V2). A test — `session_hides_exactly_the_match_chrome`, in the shape of the
keymap's "counted in one assertion and no prose" rule — enumerates those sites,
so the gate can't grow in silence. Everything else already branches on data
(`scoreboard.is_some()`), not on `kind`.

### N. Creating a session

**N1. A session starts *empty* and is captured into — the inverse of a match.**
The new-match flow requires the videos *first*, because they decide the folder
name and the aspect gate fires between them (`new_match.rs`,
`bus/project.rs:190-346`). A session has neither constraint: the footage doesn't
exist yet — the coach makes it with the camera. So session creation is **name →
create folder → open an empty session → capture.**

**N2. The name is the coach's, not derived.** (design) A match is
`metadata::match_name(home, away)` → `Some("{home} v {away}")` when both are
non-empty (`metadata.rs:220`); a session has no teams. The creation sheet takes a
free-text name, cleaned by `naming::folder_slug` (folder) and `naming::safe_chars`
(stored `name`), as the export file-name path cleans. Default when empty: a
date-stamped `"Session"` — the **bus** supplies the date (core has no clock;
`naming::parse_date_in` takes its year as an argument), as `source_date` does for
export tags. The same bus call stamps `Project.created` (S3), so the recents list
can show it.

**N3. The folder goes where a match's would — the `pundit/` walk is reused
whole.** `projects_dir_for` (`new_match.rs:100-123`) walks up for
`APP_NAME`/`pundit`, else `last_project`'s parent, else beside the footage. A
session has no footage to sit beside, so the first two tiers do the work;
unchanged. `create_dir`, never `create_dir_all` (`project.rs:161`).

**N4. One command, `Command::NewSession { name }` — smaller than `NewMatch`,
not a mirror of it.** (design, revised per simplify #3) It carries no videos and
no scoreboard, so it shares almost no code with `new_match`, whose body is
*entirely* scoreboard validation, video probing and the aspect gate — none of
which a session runs. `NewSession`'s whole body is `folder → empty Project {
kind: Session } → commit`. Like `NewMatch`, it is **not** on the recording
allow-list (deny-by-default, `bus/mod.rs:1124`). The plan should not try to fold
the two commands together; they share a goal, not an implementation.

### C. Capture-as-source — the new primitive (rewritten per both reviews)

**C0. What the first draft got wrong.** It modelled capture-as-source as a
`Take::Source` variant, on the false premise that `Take` discriminates capture
kinds. It does not: `Take` is `Footage | Slate(Uuid) | Whiteboard`
(`bus/mod.rs:87`), every variant answers *"what does this commentary take play
**over**?"*, and avatar mode isn't a `Take` at all (it's `project.avatar.is_some()`,
`recording.rs:441`). Capture-as-source plays over **nothing** — it *is* the
footage — so it is a different operation, and `start_recording`'s machinery
actively fights it (C2). It is its own command.

**C1. `Command::CaptureSource { zoom }` with its own short start path and its own
finish.** (design) It records camera+mic to a file with the existing pipeline
(`capture/recorder.rs`, the `CaptureSources::Devices` arm), but:
- **its own guards**, not `can_record`'s. `can_record` refuses
  `"add a game video to record over first"` when `source_videos` is empty and
  `"the game video isn't loaded"` when nothing is loaded (`recording.rs:420-437`)
  — both of which an empty session trips on its first move. `CaptureSource`
  requires only a camera (and, in a session, nothing loaded);
- **no player seek and no `heading` anchor.** `start_recording` derives
  `(source_index, start_source_seconds)` from `self.heading(None)`
  (`recording.rs:194`), which reads the current source and live player position —
  meaningless when capturing into an empty session. `CaptureSource` stamps no
  such anchor;
- **finish probes the file in as a source**, not a clip. On stop, the finished
  MKV is probed (`probe.rs:41-96`) for duration and aspect and pushed onto
  `source_videos` as a `SourceRef`. **No commentary `Clip` is created**: there is
  no source to play over, the event log is empty, `source_time` is identity.

**C2. The captured file lives in a dedicated `sources/` directory, not
`recordings/`.** (design, per correctness #5) `recordings/` is owned by the clip
lifecycle — `trash_clip` moves `recordings/<file>` to `recordings/.trash/`, and
`empty_trash` wipes that on every open. A *source* file has no clip and no trash
integration, so co-tenancy there is a shared-ownership hazard. A captured source
is `sources/<id>.mkv`; its `SourceRef::relative_path` points there (a free POSIX
path, so representable with no format change, `project.rs:243`). **Removing a
project-owned source deletes its file** — a small, honest addition to
`remove_source` (`bus/sources.rs:59`), which today deletes nothing because an
imported source is the coach's own file elsewhere. "Owned" = a relative path
under the project folder with no `..`, the same test `avatar`/`backdrop` copies
imply. (Plan detail: the exact deletion guard.)

**C3. The review take is the existing flow, verbatim — this is D-2.** After C1 a
session holds a source. The coach selects it, presses R, and records a commentary
take over it (play/pause/scrub/draw, `show_pip` off) through the **unchanged**
`add_recorded_clip` path (`recording.rs:489-539`), producing an ordinary `Clip`.
The pen, step-frame, scan, highlights all work. The new code is in C1; C3 is
reuse.

**C4. One Record button, and the bus decides what it means — decided (coach,
2026-11-04).** (design) In a session, R captures a new source (`CaptureSource`)
when **no source is selected**, and records a review take over the selected
source when **one is** — the meaning follows the bus's own state, the slate-pass
precedent (R's meaning follows the bus, not the window; CLAUDE.md,
`ToggleRecording { slate }`). One control, no second button on a touch screen,
and no window-side mode to drift out of step with the bus. The transport button's
label follows suit ("Record clip" with nothing selected, "Record" over a
selection), as it already reads "Record range" for a slate. The Devices popover
lists camera and mic only (no avatar/whiteboard section in a session).

### V. Review and export — what a session reuses and what it skips

**V1. Export is unchanged.** A session clip exports like any clip over any
source: the composite pins 1920×1080@30, letterboxes, burns the overlay, mixes
audio. With `scoreboard: None` no board is drawn and no cues are generated — a
legal, tested state (`carry_scoreboard` returns the burned-`None` path for a
clip target; `ScoreboardContext::for_project` short-circuits `None` at
`scoreboard.rs:643`). The export **queue** (#77) carries session jobs with no
change. **A session does not run the pairwise aspect gate** (`aspects_match`): its
sources are independent clips exported one at a time, with no two-half continuity
to protect, and the export letterboxes any aspect to 1080p regardless.

**V2. A session's export targets are the per-clip ones only — decided, not
confirmed.** (revised per simplify #7) `WholeMatch`, `Reel` and the
match-flavoured labels are match concepts. A session offers **Clip** export and
the **basket** (already cross-project, board-per-piece). There is **no
"whole-session" target**: the basket already makes cross-clip films, so a
session-flavoured `WholeMatch` analog would be redundant. No scoreboard picker,
no "Separate track" board; the Mute/Chapters/Subtitles switches behave as today.

**V3. A session hides match chrome and does not offer slates in phase 1 —
decided.** (revised per simplify #7) Rendered only when `is_match()`: the
scoreboard setup, the match-event editor, the reel, the clock readout, the
slate-band colour. **Slates are a match-review tool and a session does not
surface them in phase 1** — leaving the data path live while hiding the controls
is the worst of both, and S1's whole point is "no match furniture." If a coach
later wants to mark-then-reshoot a skill in a session, that is its own entry,
specced then.

### R. Recorded playback speed (rewritten per correctness #2/#3/#4)

**R1. Playback speed is a rate carried on the play event, walked into the edit.**
(design) The timeline computes a per-segment `rate`, but only ever `1.0` (on
`EventKind::Play`) or `0.0` (on `Pause`) (`timeline.rs:235,240`), advancing the
source cursor as `source + (ev.record_time − record_cursor) * rate`
(`timeline.rs:92`). Slow motion generalises that constant below 1.0. **The
first draft called this "one line"; it is not.** The honest edit list:

- **Event:** `EventKind::Play` gains a `rate`, as a **`Rate(f64)` newtype with
  `Default = Rate(1.0)`** — not a bare `f64` field default, which the format
  rules forbid (it would resolve to `0.0`) and which `EventKind`'s **hand-written
  codec** would ignore anyway (`EventKind` does not derive serde; it routes
  through the `KnownKind` proxy and a hand-written `impl Serialize`/`deserialize`,
  `event.rs:98-164`). The newtype's `Default` is the sanctioned "a type whose
  `Default` is what an older file means" pattern (the v14 `StrokeEnd` precedent,
  which is an enum with a `Default` for exactly this reason). The codec edit —
  `KnownKind::Play`, the `From`, the `Serialize` arm, and the
  `play_has_the_expected_wire_shape` test — is part of the work, not free.
  *(Rate-on-`Play`, not a `SetRate` event — see R5.)*
- **Timeline `emit`:** today `emit` branches `if self.rate == 1.0 { Play } else
  { Freeze }` (`timeline.rs:154-182`), so a 0.5 segment would wrongly become a
  **Freeze**. The branch becomes `rate != 0.0 { Play(rate) } else { Freeze }`,
  and `PlaybackSegment` gains `rate: f64`.
- **The EOF-split arithmetic:** `emit`'s available-source / play-duration /
  cursor-advance math assumes the source consumes 1 s per record-second; at rate
  0.5 it consumes `dur * rate`. Each of those three expressions scales by `rate`.
- **Both walks:** the segment walk (`playback_segments`) and the non-segment
  `source_time` walk (`timeline.rs:79-117`, which also hard-codes `rate = 1.0` on
  Play at `:98`) need the same generalisation.
- **Export/preview:** the walk is in **core** — `pundit-core/src/export.rs:209`
  (`schedule → walk`, the one scheduler both export and preview use) — not in
  `pundit-media`, which only consumes `FrameSpec.source_time`. It becomes
  `seg.source_start + (t − out_start) * seg.rate`; preview, being the same core
  walk with a different events closure, follows with no media change.
- **Audio:** see R3 — `core::audio::game_regions` must read `seg.rate`. The rate
  threads into `core::audio`, not only the export walk.

`out_duration` is unchanged — it is record time, real either way; only the
*source* advance scales.

**R2. The commentary stays real-time; only the source footage slows.** Record
time is wall-clock, so the coach's voice and pen are real-time while a rate of
0.5 makes the footage crawl under them — "walk me through this slowly." No
commentary resampling, because commentary time *is* output time.

**R3. A slowed segment is voice-over: it contributes no source-audio region.**
(design, revised per simplify #5 + correctness #4) `game_regions` emits a region
per `SegmentKind::Play` and reads the file at 1:1 (`audio.rs:211-251`), so a
slowed picture would drift against unslowed source audio. The rule is framed at
the **segment**, not as a `rate != 1.0` special-case bolted on: *a Play segment
with a non-unit rate is definitionally voice-over, so it emits no source-audio
region* — the same shape as the source-volume-0.0 "no region" path (#124). The
commentary is untouched. Pitch-corrected time-stretch is a deferral, not phase 1.

**R4. Scan speed stays a separate concept, correctly.** Scan speed
(`SCAN_SPEEDS = [1,2,4,8,16,32]`, `transport.rs`) is **viewing only**: it speeds
seeking during review, is refused while recording, returns to 1.0 on pause. The
recorded rate is **sub-1.0 and persistent**, part of the edit. They do not merge;
the smell to avoid is a *third* rate. The recorded-rate control is shown in the
recording transport (so it reads as a transport action, like pause/scrub) and
surfaced in the clip's settings for review.

**R5. The format reasoning, corrected.** (per correctness #2) The first draft
said a new play-rate "survives an old build via `EventKind::Unknown`." That is
**false**: `Unknown` catches an unrecognised *outer key* (a `SetRate` event would
survive), but a new *field* on `Play` is dropped by an old build's
`Play { source_time }` deserialize — not preserved. The real protection is the
**v19 bump itself**: an older build refuses the whole file as too-new and never
deserialises it, so the field is never dropped. Within the readable range the
field-level `rate: 1.0` default reads every v7–v18 file correctly. This is why
**rate-on-`Play` is chosen over a `SetRate` event**: with the bump, rate-on-`Play`
has no forward-compat disadvantage, it is the smaller change (one field, one
`emit` condition, no new event variant and no new recording-allow-list entry),
and "change speed" naturally implies a play boundary. A live mid-take rate change
appends a `Play` at the current position (which `TogglePlay` already does on the
allow-list, `bus/mod.rs:1124`), so no new command is needed.

### T. The touchscreen pass (D-5, tightened per simplify #6)

**T1. Three mouse-only gestures, and only the menu-only *actions* need an
affordance.** The sweep of 2026-11-04 found everything else already tap/drag: the
transport row, the pen row, the scrubber **drag** (`scrubber.slint:171`,
left-button), the player pan/draw/highlight drags, zoom, every sheet button, and
all popup/sheet dismissal (click-outside works with a tap). Hover is
supplementary everywhere, never the only affordance. The three `ContextMenuArea`
right-clicks:

| Gesture | Where | Items |
|---|---|---|
| Right-click scrubber | `scrubber.slint:199-216` | Snap toggle; "Band colour…" |
| Right-click slate row | `app.slint:6223-6295` | Jump / Preview / Record / Delete |
| Right-click clip row | `app.slint:6593-6625` | Jump / Preview / Export / Add to basket / Delete |

**T2. The pass gives a tap path only to actions with no other one.** (revised) It
does **not** reflexively add a kebab to two dense row types. First check the
existing taps: a row's **single-click selects** and **double-click previews/jumps**
(`app.slint:6200,6588`), so *Preview* and *Jump* already have a tap path. What is
**menu-only** — Record (slate), Export / Add to basket / Delete (clip), Delete
(slate) — gets a visible affordance: a kebab `⋯` on the row opening the **same**
menu (one menu definition, two openers). The scrubber's menu **dissolves with no
new control**: Snap becomes a visible toggle in the scrubber area (a persistent
setting, not a per-object action), and "Band colour…" is already in the Settings
sheet (#131); its right-click stays as a power-user alias. In a session, the clip
row is the one that matters; the slate row isn't shown (V3).

**T3. Bounded by that table, and must not grow.** (design) The pass touches the
clip-row (and, in a match, slate-row) menu-only actions and the snap toggle, and
**nothing else**. A genuinely new touch surface (on-screen keyboard, gesture
zoom, a tablet layout) is #139, needs Android, and is not here. The review
rejects any T-item that isn't one of these.

**T4. Resolve against #139 so the touch surface isn't designed twice.** The
kebab-menu pattern and the snap toggle are controls a future Android UI would
want; keeping them in plain Slint means #139 inherits them. Noted in #139.

### D. Google Drive upload + share queue (phase 2 — sketch only)

**D1. The most external, most separable chunk; phase 1 ships without it.** A
burned `.mp4` in `exports/` is a complete deliverable; Drive is an additional
sink. The sketch, to be its own spec:
- a share queue mirroring the export queue (#77): a `Vec` of pending uploads on
  `Bus`, in memory by the same reasoning (persisted only if asked, then
  `shares.json` beside `basket.json`, never a `state.json` key). Sink is Drive;
- OAuth with token storage off `project.json` and off `state.json` — a secret is
  neither project data nor a machine preference — in a dedicated, mode-restricted
  token file, with a refresh-token flow for the shared/headless cart box;
- the configured Drive folder and whether the upload returns a shareable link for
  the parent — the product questions phase 2 opens;
- no test reaches Google (the whisper/Openverse fixture rule): a fake upload sink
  behind a trait, served locally.

**D2. The seam phase 1 leaves clean:** export completion already emits an
`ExportDone` with the output path (`composite::export`'s `finish`). Phase 2
subscribes to that; it does not reach into the composite. Phase 1 owes phase 2
only a stable "a file was written here" event, which exists.

---

## Phasing

| Phase | Delivers | Done when |
|---|---|---|
| **1a** | The session kind (S) + creation (N) + capture-as-source (C) + review/export reuse (V). **v19.** | A coach makes a session, records a clip from the USB camera, it becomes a source, records a review take over it, and exports an `.mp4`. |
| **1b** | Recorded playback speed (R), **on the same v19**. | A review take recorded at 0.5× exports with the footage at half speed and the commentary real-time. |
| **1c** | The touch affordance (T) — the menu-only actions, the snap toggle. | Every control the gym flow uses is tap-reachable; `tests/ui` presses the clip-row kebab. |
| **2** | The Google Drive share queue (D) — its own spec first. | A finished export uploads to Drive and the coach gets a link. |

1a is the spine. 1b touches the timeline/event/export/audio walk; 1c touches
`app.slint` rows; they are independent of each other. 2 is gated behind its own
design. **1a and 1b share the v19 bump** (S2), so if they ship in one release the
format changes once.

---

## Open questions for the coach

All resolved. The first draft's `(confirm)` marks were settled by the review
(rate lives on `Play`; whole-session and slates-in-session are cut; the audio
rule is segment-level), and the two the review left to the coach were answered
2026-11-04:

1. **The capture gesture (C4):** one Record button, the bus deciding capture vs
   review from selection (the slate-pass precedent). **Resolved.**
2. **Session identity (S3, N2):** a session stores a created date, shown in
   recents. **Resolved.**

Nothing here is built. The next step is the plan.

---

## X. Reasoning the review rejected (kept, not deleted)

- **`Take::Source`** (first-draft §C1): rejected — `Take` discriminates
  play-over target, which capture-as-source has none of; the justification was
  factually wrong (`bus/mod.rs:87`). Replaced by `Command::CaptureSource` (§C).
- **"Capture-as-source costs no new code beyond letting the source be a captured
  one"** (first-draft §C4): false — it bypasses `can_record`'s guards and
  `heading`'s anchor (§C1). Kept as the thesis correction.
- **"The rate change is one line at `export.rs:209`"** (first-draft §R1): false —
  `emit` emits a Freeze for any non-unit rate, and the EOF-split math and both
  walks and `game_regions` all scale (§R1).
- **"A new play-rate survives an old build via `EventKind::Unknown`"**
  (first-draft §R5): false for a field; the v19 bump is the real protection (§R5).
- **A `SetRate` event / a clip-wide `Clip` rate field:** rejected in favour of
  rate-on-`Play` (§R5) — the version bump removes `SetRate`'s only advantage, and
  a clip-wide field can't slow one segment and not another.
- **Whole-session export target; slates offered in a session; a kebab on every
  row reflexively:** all cut as scope (§V2, §V3, §T2).

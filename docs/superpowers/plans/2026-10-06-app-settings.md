# Plan — the two export-output switches (#78)

Spec: `docs/superpowers/specs/2026-10-02-app-settings-design.md` (rewritten whole
on 2026-10-06; the date in its filename is the first version's). **Read it
first**, with the five corrections in "The spec is wrong, or now stale, about
five things" below. Read `CLAUDE.md` too — the format rules, the export-renderer
split and the basket section all bind here.

**What ships:** two `CheckBox`es on the export sheet, **Chapters** and
**Scoreboard subtitles**, each governing its output in both the forms it takes,
each a `Preferences` field on the write-back the other four controls already use,
each reaching media by blanking data media already reads. **`pundit-media`
changes nothing** — if it does, §S2's design has been abandoned.

**Gates.** Every `cargo` call but `fmt` under
`flock /tmp/claude-1000/cargo.lock nice -n 19`. Clippy must be
`rustup run 1.92 cargo clippy --workspace --all-targets -j 3 -- -D warnings` —
a clean local 1.98 is not the gate, and `-j 3` is required (#121). **Never
`cargo test --workspace`.** Never pipe clippy or tests to `tail`/`head`.

**Known flakes, not yours:** #72 (harness `lib.rs:164`, "timed out waiting for a
settled position" — nine sightings, two of them on docs-only branches) and #101
(`corrupted size vs. prev_size`, SIGABRT, no test named). **Log any sighting.**

## Where this stands (update it as tasks land)

- **1 — the switches reach the outputs.** **Done** (2026-10-07). The version
  taken was **16**, read off `store.rs`, which was still 15: #115 had not landed
  and nothing else had claimed it. Four deviations, each small:
  - **`ExportChoices` carries `Debug, Clone, Copy, PartialEq`.** `Command`
    derives `Debug`, so the struct needs it; `PartialEq` is what the new
    `Pickers::of` unit test asserts with. No `Default`, as the plan requires.
  - **Only three test files got a `choices()` helper** — `export.rs`,
    `whole_match.rs` and `reel.rs`, which have several senders each.
    `preview.rs`, `highlights.rs` and `new_match.rs` send once apiece, so a
    one-use helper would not have earned its place; each writes the literal.
  - **`whole_match.rs`'s rig takes its tags as a parameter** (`open_tagged`,
    with `DEFAULT_TAGS` for the two every other test wants) rather than growing
    a second constructor body. The chapters test is the only caller that needs
    three moments ten seconds apart.
  - **`a_run_persists_the_resolution_quality_and_mute` is renamed**
    `…_and_switches`, since it now covers all three.
- **2 — the sheet.** Not started.

**They are sequential, and they ship together — one PR, one release.**

- **Sequential, not parallel, and the overlap is named.** Task 1 adds two
  required fields to `Command::Export`, and `main.rs:919` constructs that
  command, so task 1 **must** touch `main.rs` or the crate does not compile. It
  touches exactly one site there, sending `chapters: true, cues: true` as
  literals, which task 2 replaces with the window's two properties. That is the
  whole of the overlap and it is one line. (The mute plan's first draft claimed
  two tasks were file-disjoint when both rewrote `job` 23 lines apart; this claim
  was checked the way that one should have been — by opening
  `crates/pundit-app/src/main.rs` and reading the `Command::Export` send at
  :919-930. Task 2's other files, `ui/app.slint` and the rest of `main.rs`, task
  1 does not open.)
- **Together, because task 1 bumps `formatVersion`.** A build between 1 and 2
  stamps the next version on the coach's project and gives him no control over
  the two fields it added — the naked-bump failure the mute plan's §R.2 records.
  Worse here than there: task 1 alone also changes what a burned whole match
  writes (see Risk 1), with no switch on screen to say otherwise.
- **Task 1 is the one with the sabotage proofs.** Everything it does is visible
  from the bus, so the harness can see all of it. Task 2 is `.slint` plus two
  `main.rs` sites, and its proof is the manual batch — honestly, and the reason is
  in task 2.

---

## 1. The switches reach the outputs

**Files.** Core: `crates/pundit-core/src/{project.rs, store.rs}`,
`crates/pundit-core/tests/project_format.rs`. App:
`crates/pundit-app/src/bus/{mod.rs, export.rs}`,
`crates/pundit-app/src/main.rs` (the one send, literals). Harness:
`crates/pundit-harness/tests/{export.rs, whole_match.rs, reel.rs, preview.rs,
new_match.rs, highlights.rs}` — every sender of `Command::Export`, and **not**
`basket.rs`, whose senders are `Command::ExportBasket`. Docs: `CLAUDE.md`.

### The stored fields

1. **Two `bool`s on `Preferences`**, beside `last_export_scoreboard`:
   `last_export_chapters` and `last_export_cues`, **both `true`** in the
   hand-written `Default` impl, **neither taking a serde attribute** — the
   container carries `#[serde(default)]` and fills from that impl, so a
   field-level one would be a second copy of the default (`project.rs`'s own
   header, and the call v11, v13 and v15 each made).
   - `last_export_cues`, not `last_export_scoreboard_subtitles`: it is named for
     `job.cues` and `core::cues`, the words the code already uses for this pair
     of outputs, and the longer name would read as a qualifier on
     `last_export_scoreboard` sitting right above it.
2. **Bump `CURRENT_FORMAT_VERSION`.** Take the **next free number from
   `store.rs` at implementation time**. It is 15 there today (v14 is #117's
   `StrokeEnd`, v15 the mute's `export_source_volume`), so the next is **16** —
   but read the constant, do not trust this line. `MIN_READABLE_FORMAT_VERSION`
   stays 7; both fields are additive. **BACKLOG #115 also wants 16: if it is
   being built in the same stretch, share one bump** — one version, one
   every-readable-version test.
3. **`preferences_defaults_are_not_zero` is amended, not joined**: two
   `assert!(p.last_export_chapters)` / `assert!(p.last_export_cues)` lines. That
   test is the one that bites a wrong default, and it already covers every
   other field of this container.

### The arity wall, and the one struct that answers it

4. **`Bus::export` cannot take two more arguments.** It has five plus `&mut
   self` today (`bus/export.rs:310`); two bools make **eight**, and clippy's
   `too_many_arguments` fires at eight. **Measured, not assumed:** `rustup run
   1.92 clippy-driver --edition 2021 --crate-type lib` over a stand-in with a
   seven-parameter and an eight-parameter method reports `this function has too
   many arguments (8/7)` on the eight only, `#[warn(...)] on by default` — so
   `-D warnings` fails the build. `self` is counted.
5. **So the command's payload becomes one struct**, `ExportChoices`, in
   `bus/export.rs` and re-exported from `bus/mod.rs` beside `ExportRun`
   (`pub use export::{…}`, :59) so the harness can name it:

   ```rust
   pub struct ExportChoices {
       pub resolution: Resolution,
       pub quality: Quality,
       pub scoreboard: Option<ScoreboardMode>,
       pub mute_source: bool,
       pub chapters: bool,
       pub cues: bool,
   }
   ```

   `Command::Export { targets, choices }` replaces the four flat fields
   (`bus/mod.rs:355-367`), the dispatch arm becomes `self.export(targets,
   choices)` (:1082-1088), and `Bus::export` is three parameters.
   - **Why a struct and not three more positional bools.** `mute_source`,
     `chapters` and `cues` are three adjacent values of the same type at every
     site that passes them — `Bus::export`, `main.rs`'s seeding tuple, and the
     two harness `export_with` helpers. Transposing two of them compiles and
     every test still passes. The rule is already written down in this tree, ten
     lines above the function this plan edits in `main.rs`: *"a pair of `i32`s
     would read as valid either way round, which is the mistake
     `InsetPlacement` exists to make unwritable"* (`main.rs:1020-1022`).
   - **The cheaper alternative, and why it loses.** Leave `Command::Export` flat
     and build `ExportChoices` in the dispatch arm instead: a smaller diff (the
     eight harness senders keep their shape), but then the six field names are
     written twice for no gain and the three transposable bools survive in
     `main.rs` and in the test helpers. The struct is the version where no site
     has three adjacent same-typed values.
   - **`Command::ExportBasket` keeps its four flat fields.** The basket gets no
     chapters switch (spec Deferred 3), so it has nothing to group and its
     `export_basket` is five parameters with `self`. The asymmetry is the
     absence of a feature, not a style drift.
6. **`Pickers` gains the two bools** (six fields), `Pickers::of` reads them, and
   the write-back assigns them (`bus/export.rs:371-380`). Its doc comment says
   *"The export sheet's four pickers"* — it is now **three pickers and three
   switches**, and that comment is load-bearing enough that the spec's crate
   table names it. The `PartialEq`-not-`Eq` note stays: `source_volume`'s `f64`
   is still what forbids `Eq`.
7. **The one negation stays where it is.** `Bus::export` builds `Pickers` from
   `ExportChoices`, turning `mute_source` into `source_volume`. The two new
   switches are **not** negated anywhere: they are `true` for "write it" at every
   layer, so there is nothing for a reader to track.

### The cue slot leaves `carry_scoreboard`

8. **`Carry` loses its `cues` field** and becomes `{ copy, scoreboard }` —
   exactly "how this target carries the board in the picture".
9. **A new `board_cues(target, want, compilation, context) -> Option<Vec<Cue>>`**,
   three arms in order:
   - not `ExportTarget::WholeMatch` → `None`. A `.srt` beside a clip is the
     coach's own file and no export's business, so nothing at that path is
     written **or removed** (`a_clip_on_a_separate_track_burns_the_board_in`
     pins it).
   - `!want` → `Some(Vec::new())`: no `.srt`, a stale one removed, and no
     subtitle pad requested by the copy.
   - otherwise `Some(scoreboard_cues(compilation, context))`, or
     `Some(Vec::new())` where `context` is `None` — there is nothing to derive
     cues from.
10. **`carry_scoreboard` loses `compilation` and gains `want_cues`.** Verified:
    `compilation` is read at **exactly one place** in that function,
    `scoreboard_cues(compilation, context)` (`bus/export.rs:659`), so taking the
    cue slot out takes that parameter's only use with it. Its arity is therefore
    unchanged at six, not reduced — what falls is the number of questions it
    answers, and `Carry` going from three fields to two is where that shows. (An
    earlier note of mine said this was the moment to *rename* the function for
    growing a third concern. Backwards: it grows no concern, and the spec's §R.5
    records it. No rename.)
11. **§S4's Default rule is one condition, and it goes before `can_copy`:**

    ```rust
    ScoreboardMode::Track => {
        if picked.is_none() {
            // "Default means the best available, never a silent trade": with
            // the subtitles off, a copy would carry no board anywhere, so
            // Default burns it in instead.
            if !want_cues {
                return burned(context);
            }
            if let Err(why) = pundit_media::can_copy(files, with_audio) { … }
        }
        …
    }
    ```

    **Before** `can_copy`, which reads a header per file: a question already
    settled should not cost that I/O. *Separate track* chosen by hand still gets
    no board, because there the coach asked — which is also what makes §U2's
    second line deterministic.
12. **Chapters off is one line in `job`.** `let compilation` becomes `let mut
    compilation` and, where the two switches are read,
    `compilation.plan.chapters.clear()` when `!pickers.chapters`. Both readers
    already mean "no chapters" for an empty list, and both live in media's own
    `finish`: `chapters::splice` returns `Written(0)` **without opening the
    file** (`media/src/chapters.rs:157-160`) and `chapter_list` returns `None`
    for an empty slice, which makes `write_chapter_list` **remove** a stale
    `.chapters.txt` (`media/src/composite/export.rs:519-523`). Nothing else
    reads `plan.chapters`, and `total_frames` — every denominator — is untouched.
13. **No `bool` reaches `ExportJob`.** An empty chapter list and an empty cue
    list are already media's one "no chapters" and one "no subtitles"; a flag
    beside them would be the third state `carry_scoreboard`'s own rule exists to
    refuse (`CLAUDE.md`: *"Track mode blanks `job.scoreboard` rather than
    carrying a mode flag into media"*).
14. **Every sender of `Command::Export` needs the new shape.** `main.rs:919`
    (literals, until task 2), and in the harness: `export.rs:116`,
    `whole_match.rs:137`, `preview.rs:109`, `new_match.rs:594`,
    `reel.rs:116/253/379`, `highlights.rs:312`. The two `export_with` helpers
    (`export.rs:115`, `whole_match.rs:131`) take `ExportChoices` in place of
    their trailing bools, and each file gets a three-line helper returning its
    rig's standard choices (720p, Low, Default, unmuted, both outputs on), so a
    test that cares about one switch writes
    `ExportChoices { chapters: false, ..rig_choices() }` and names that one
    switch.
    - **Do not `#[derive(Default)]` on `ExportChoices`.** Both new fields'
      correct value is `true` and a derived `Default` is `false`, so every
      helper built with `..Default::default()` would silently export with both
      outputs off and the tests would be asserting the wrong baseline. This is
      `project.rs`'s own header hazard — *"never a field-level default on an
      `f64` or a `bool`"* — in a struct serde never sees. A hand-written
      `Default` is the other option and is worse: it would be a second copy of
      `Preferences::default()`'s four `last_export_*` values, free to drift from
      it, for a convenience two test files need.
    - **`basket.rs`'s senders are `Command::ExportBasket` and are untouched.**
      Check this rather than assume it: the basket's own `cues` are `None` by
      design (`bus/basket.rs:520`) and its board is burned in.
15. **`CLAUDE.md`, the two paragraphs this task makes true:**
    - the **format-version list** gains a bullet on v11's, v13's and v15's shape
      — the two fields, no attribute, the floor stays 7, and the bump's real cost
      is the forward direction that `project.json.v<old>` exists for. **While you
      are in that list, v14 has no line at all** (the jump is v13 → v15); the
      mute plan noted the slip and did not fix it. Add v14's line (#117's
      `StrokeEnd`) or say in the commit why not.
    - the **`.chapters.txt` paragraph** (:589) says *"Every target that has
      chapters gets one"* and *"There is no 'leave the path alone' case as `cues`
      has"*: both stay true, with the Chapters switch as the one thing that makes
      a target have none.
    - the **export-sheet paragraph** (:593) counts the sheet's controls and names
      `Pickers`; it gains the two switches, §S1's one-switch-both-forms rule and
      §S4's "Default never trades the board away".

### Tests

- **`bus/export.rs`'s own `mod tests` — one unit test on `Pickers::of`.** A
  `Preferences` with all six values away from their defaults, asserting every
  field of `Pickers::of(&prefs)` matches. This is the sharp, free pin for "the
  write-back read the stored value", and it covers the mute's field too.
  - **Why this and not three more harness runs.** A wrong `Pickers::of` shows up
    only as a *missed* write-back, so pinning it through the harness needs, per
    switch, one run that stores "off" and a second that differs from what is
    stored in that switch **alone** — three extra renders for what one
    `assert_eq!` on a private function says exactly. The existing two-run dance
    in `a_run_persists_the_resolution_quality_and_mute` stays (it also pins the
    round-trip through `store`); it does not grow.
- **`core/tests/project_format.rs` — `a_v15_file_loads_under_the_current_version`**,
  on `a_v14_…`'s shape (the series is named for the **old** version; there is no
  `v7_to_v15` test). Serialize `sample_project` with both switches set to
  `false`, remove both keys from `preferences`, stamp the previous version, read,
  assert both are `true` **from the container default**, then write and assert
  the file is re-stamped and `project.json.v15` exists. **The literal
  `project.json.v15` is the only thing in it that fails if the constant never
  moved** — every other version assertion reads `CURRENT_FORMAT_VERSION`.
- **`crates/pundit-harness/tests/whole_match.rs` — the copy is where both
  switches are cheap**, because it encodes nothing and that rig already has a
  scoreboard, a kick-off and a goal tagged, `outputs(&m.exports())` for the
  folder's contents and `streams(path, "s")` for the subtitle track.
  - **Chapters, both forms, one test.** A new rig variant with **one source of
    900 frames** (30 s at the file's `FPS`) and three match events tagged at
    0.2 s, 12 s and 25 s: `whole_match_chapters` takes the match's own moments
    whatever the entry count (`core/src/whole_match.rs:76-85`), so one source is
    enough, and the three are ≥ `MIN_GAP_SECONDS` apart so `chapter_list`
    actually writes a file. Export it on *Separate track* twice:
    - **on** → `outputs` holds the `.mp4`, the `.srt` and the
      `Whole match - Game.chapters.txt`, whose text is
      `0:00 Kick-off\n0:12 Rovers goal 1-0\n0:25 Half time\n`, and
      `ffprobe_chapters(path)` (copy `reel.rs:304`'s helper) reads three.
    - **off** → the `.chapters.txt` the first run left is **gone**, and
      `ffprobe_chapters` reads none. The pair proves the switch rather than an
      absence, and asserting both forms in one test is what stops the two halves
      drifting.
    - **Why a 900-frame fixture rather than the existing ones.** No rig in the
      tree can produce a `.chapters.txt` at all: `chapter_list` needs
      `MIN_CHAPTERS` (3) survivors `MIN_GAP_SECONDS` (10) apart, and
      `export.rs`'s clips are 1 s while `whole_match.rs`'s source tuples are
      `(name, w, h, **frames**)` — `("first half", 640, 360, 60)` is 2.0 s. On
      the encoded path three chapters ten seconds apart is 900+ output frames on
      llvmpipe; on the copy path it is a header read and a packet copy.
  - **Subtitles off on a copied whole match**: no `.srt`, a stale one removed
    (export once with them on first), and `streams(path, "s") == 0` — the half
    that lives in `copy.rs` and the one an implementation can get right beside
    the file and wrong inside it.
  - **Burned with the subtitles on writes the `.srt`.** Unreachable today, the
    cheapest proof that independence is real, and the test that must be
    **rewritten rather than extended**: `a_burned_whole_match_removes_a_stale_sidecar`
    asserts `outputs == ["Whole match - Game.mp4"]`, which is no longer what a
    burned run with the default switches writes (Risk 1). Keep its removal half
    by driving it with the subtitles **off**, and add the on-case beside it.
  - **Default with the subtitles off burns the board in** rather than copying.
    **The discriminator is the frame size, not the packet count:** the rig's
    sources are 640×360 and the sheet sends `R720`, so a copy reads back 640×360
    and an encode 1280×720 — while an encode of a whole match at `OUTPUT_FPS`
    over 30 fps sources writes the *same* number of video packets as the copy,
    which is why `the_whole_match_is_copied_with_a_sidecar`'s packet assertion
    cannot carry this one. Add a `video_size(path)` helper beside `streams`.
- **`crates/pundit-harness/tests/export.rs` — the write-back.** Amend
  `a_run_persists_the_resolution_quality_and_mute` to assert both new fields are
  stored from a run that sends them off, and
  `a_refused_run_leaves_the_pickers_alone` **must send them off**, or asserting
  the stored values are still `true` says nothing (the trap the mute plan's §R.5
  records, in its own words).
- **Not tested, deliberately:** that `splice`, `chapter_list` and `write_sidecar`
  handle an empty list. All three are already covered — `splice` by the
  single-clip path, `chapter_list` by `core/tests/chapters.rs`,
  `write_chapter_list`'s removal by
  `media/tests/copy.rs::a_single_source_is_copied_with_no_chapters`, and
  `write_sidecar`'s by `a_burned_whole_match_removes_a_stale_sidecar`.

### Sabotage proof

Four, each failing a different test:

1. **Delete `compilation.plan.chapters.clear()`.** The chapters-off half of the
   whole-match test must fail on **both** forms — a `.chapters.txt` still beside
   the file and three chapters still in it. Then make the clear
   unconditional: the chapters-on half must fail. One line, both directions.
2. **Make `board_cues` ignore `want`** (always return the cues). The
   subtitles-off test must fail on the `.srt` **and** on `streams(path, "s")`.
3. **Drop the `!want_cues → burned` arm from the `Track` branch.** The
   Default-with-subtitles-off test must fail on the frame size: 640×360, i.e.
   copied, where the sheet promised a burned board.
4. **Have `Pickers::of` return `true` for both new fields regardless.** The new
   unit test must fail. (Not "remove the fields from `Pickers::of`" — that is a
   compile error, not a failing test.)

---

## 2. The sheet

**Files:** `crates/pundit-app/ui/app.slint`, `crates/pundit-app/src/main.rs`,
`CLAUDE.md` (the export-sheet paragraph, if task 1 left the UI half of it).

1. **Two `CheckBox`es, under the Scoreboard row and above "Mute source audio"**
   (`app.slint:1962-1980`), in the spec's §U2 order:

   ```text
   [x] Chapters, in the file and as a list beside it
   [x] Scoreboard subtitles — an .srt beside the file, and a track inside a copy
   [ ] Mute source audio
   ```

   Each label says **both halves**, because each switch governs both and the
   coach can only see one: "Chapters" alone reads as the file beside the video.
   Their own rows, as the Scoreboard picker took its own for the stated reason
   that *"'Burned into the picture' doesn't fit a third of this sheet"* — a 480px
   card (`:1906`) has no fourth column.
2. **A tooltip on the subtitles switch** carrying the general warning: off means
   the scoreboard appears only if it is burned into the picture. Same idiom as
   the mute's (`:1972-1981`).
3. **Diff the two new checkboxes against the mute checkbox, line by line, and
   tick off all five of its touch points.** This is an explicit step, not a
   reading: three of the four bugs found in the slates panel this week (#130,
   #132 and a `J`/`L` hang) were rules the clip inspector already had that the
   slates copy did not. The mute's five are:
   - `in-out property <bool> mute-source;` on `ExportSheet` (`:1897`) → two new
     properties, `chapters` and `cues`;
   - `enabled: !root.exporting;` on the `CheckBox`, as all four controls have;
   - `checked <=> root.mute-source;` — two-way, not `checked: root.…`;
   - `in-out property <bool> export-mute-source;` on the window (`:3844`);
   - `mute-source <=> root.export-mute-source;` at the `ExportSheet`
     instantiation (`:6452`) — **the wire that, missing, leaves a checkbox that
     moves and changes nothing.**

   And two the mute did *not* need, so do not copy them in: **no `editing` fold**
   into `text-editing` (the export sheet has no text field — that rule is the
   basket and New-match sheets'), and **nothing in the basket sheet** (spec
   Deferred 3).
4. **`main.rs`, two sites, and both are needed or the control is half-dead:**
   - the send (`:919-930`): the literals task 1 put there become
     `chapters: w.get_export_chapters(), cues: w.get_export_cues()`;
   - the seeding (`:1042-1095`): the closure's `picked` tuple is already five
     values and would become seven, three of them adjacent `bool`s. **Return
     `ExportChoices` and the rows instead** — the type task 1 added, named
     fields, and the same argument as step 5 of task 1. Then
     `w.set_export_chapters(…)` / `set_export_cues(…)` beside
     `set_export_mute_source` (`:1095`).
5. **The explanatory line's condition changes, and a second line appears.** One
   derived property on `ExportSheet`, so the same boolean is not written into two
   `if`s:

   ```slint
   // This run would copy rather than re-encode: the whole match is ticked and
   // the board is going anywhere but into the picture — which, under Default,
   // now depends on the subtitles too (spec S4).
   property <bool> would-copy: root.whole-match-ticked
       && (root.scoreboard == 2 || (root.scoreboard == 0 && root.cues));
   ```

   - the existing line (`:1962`, today `whole-match-ticked && scoreboard != 1`)
     becomes `if root.would-copy:`. **It is false today for
     Default-with-subtitles-off**, which burns the board in: with `cues` on the
     new condition is the old one exactly, so nothing changes for a coach who
     touches neither switch.
   - a **second** line when `root.whole-match-ticked && root.scoreboard == 2 &&
     !root.cues`: this export carries no scoreboard at all — not in the picture,
     not beside the file, not inside it. Deterministic, because *Separate track*
     chosen by hand refuses rather than falling back.
   - `export-whole-match-ticked`'s doc comment on the window (`:3845-3846`) says
     it decides *"the Scoreboard picker's explanatory line"*, singular. It now
     feeds two.
   - The `can_copy` gate can still send *Default* back to a re-encode and only
     the bus knows that, which is as true of the existing line as of the new one;
     the bus says so on stderr, as today.

### Tests, and why there is no automated one

**There is no UI test for this task, and that is a decision rather than an
omission.** `crates/pundit-app/tests/ui/` drives the real `AppWindow` on Slint's
headless backend, and its own module doc says what it is for: failures *"neither
of which any bus-level test can see"*. A dead `<=>` wire is such a failure — but
it is not reachable from there. The window root can only read its own
`export-chapters`; the leaf the wire feeds is `CheckBox.checked` inside
`ExportSheet`, which has no id the test can name, so observing the break needs a
synthesized pointer click at a computed position inside an open sheet. Every test
in that binary either sends a `WindowEvent` key or reads a root property. And
`main.rs`'s seeding is not reachable at all: `open_export_sheet` reads the `UI`
thread-local. **The four controls already on that sheet have no test of their
seeding either**, so this is an existing gap, not one this change opens —
reported for the backlog rather than papered over with a new mechanism.

**Manual, batched:**

- Export a whole match with both switches off and confirm `exports/` holds the
  `.mp4` alone.
- Turn Chapters off, export, reopen the sheet: it comes back off. (The seeding,
  which nothing else proves.) Same for the subtitles.
- Confirm the sheet still fits its 480px card with two more rows at the window's
  1100px minimum — **an estimate in the spec, a measurement here.**
- Confirm the no-board-anywhere line appears for *Separate track* with the
  subtitles off, and **not** for *Default* with them off (which burns the board
  in and shows neither line).

### Sabotage proof

Remove the `cues <=> root.export-cues;` wire at the `ExportSheet` instantiation.
The checkbox still ticks, the sheet still exports, every test still passes, and
the second manual check above is the only thing that catches it. **That is the
proof, and it is a manual one** — which is the honest statement of where this
task's risk lives, and the reason step 3 is a tick-list rather than a reading.

---

## The spec is wrong, or now stale, about five things

The plan above is right where it differs.

1. **`ExportDone::chapters` is not visible to the harness.** Its Testing section
   asks for *"`ExportDone::chapters` is `Written(0)` off and `Written(n)` on"*.
   The bus keeps only the path — `TargetState::Done(PathBuf)` — and spends
   `done.chapters` on the `bus: exported …` log line and nowhere else
   (`bus/export.rs:245-264`). A harness test reads the `chpl` box with `ffprobe`
   (`reel.rs:304`), or the assertion belongs in `pundit-media`.
2. **No fixture in the tree can produce a `.chapters.txt`**, so *"Chapters off on
   an All Clips export writes no `.chapters.txt` … on, the same export writes
   one"* is not writable as stated: `MIN_CHAPTERS` is 3 and `MIN_GAP_SECONDS` is
   10, while `export.rs`'s clips are 1 s and `whole_match.rs`'s source tuples
   count **frames**. Task 1's tests move that pair onto the copy path with a
   30-second source, where it costs no encode.
3. **`carry_scoreboard`'s arity does not fall.** §S3 says it *"loses a parameter
   rather than gaining one"*, and §S4 then hands it the subtitles bool: six in,
   six out, `compilation` for `want_cues`. The true claim — verified at
   `bus/export.rs:659`, the function's only read of `compilation` — is that
   taking the cue slot out takes that parameter's only use with it, and that
   `Carry` goes from three fields to two. Nothing here is a reason to rename it.
4. **Nothing in the spec prices the arity wall.** Two bools make `Bus::export`
   eight parameters and clippy's `too_many_arguments` fires at eight, counting
   `self` — measured on 1.92, `-D warnings` fails. The spec's crate table
   (*"two `bool`s on `Command::Export` and its dispatch arm"*) cannot be
   implemented literally without something grouping; task 1 groups once, with a
   struct that also removes two other adjacent-bool hazards.
5. **A behaviour change the spec frames as a *choice* arrives as the *default*.**
   §S4 presents "board burned in **and** an `.srt`" as a combination the coach
   asked for knowing what it is. With `board_cues`' three arms and
   `last_export_cues` defaulting to `true`, it is also what **every** burned
   whole match now writes — including a *Default* run that fell back to burning
   because `can_copy` refused, which is every Matroska or HEVC project. That is
   the literal reading of independence and this plan takes it (Risk 1) —
   **and the coach confirmed it on 2026-10-06**, shown the three options
   (accept the redundant `.srt`, default the switch off, or go back to following
   the picker) and choosing **accept**: the switch does what it says, and the
   rule that matters more is that the board is never *lost* — defaulting the
   switch off would have stopped today's whole-match copy writing an `.srt` at
   all. So Risk 1 is a recorded, accepted cost rather than an open risk, and
   `a_burned_whole_match_removes_a_stale_sidecar` is rewritten knowing it. The
   spec should still have said so where it says the picker's old coupling had a reason
   (§W3: *"a subtitle line of a board already painted into the picture is the
   board twice"*).

**And one count, corrected in `BACKLOG.md` in this change:** §D says six entries
cite #78. **#131** (the slate band's colour, filed 2026-10-05) waits on *"#78's
settings surface"* too, which makes seven — and #129, filed the same day, is
deliberately **not** one: its own words are *"with any settings surface"*, for
the reason its entry gives. #131's entry also still describes the band as
`#2ec4b6.transparentize(0.3)`, which `edba6fd` replaced with
`Palette.selection-background.transparentize(0.25)` on the same day it was filed.

## Risks

1. **Every burned whole match now writes an `.srt`** (see above), and a coach
   whose sources cannot be copied sees a new file beside his export without
   having asked for one. **Accepted**, because the alternative is the dependent
   shape the coach declined: a switch that says "Scoreboard subtitles: on" and
   then writes none, in the one case where the app chose to burn the board in,
   is the switch not doing what it says. It is pinned by two tests rather than
   left to be discovered, and it is reversible in one arm of `board_cues`.
2. **§S4's Default rule is an inference, not a coach answer.** The spec says so
   and accepts it; the plan adds only that it is one condition in the arm that
   already holds the `can_copy` fallback, before that call rather than after it.
3. **One switch per output may be one too few** (a coach wanting `chpl` for mpv
   but no text file). **Accepted:** the in-file forms are invisible and free, and
   four checkboxes would be worse than the gap. Spec Deferred 2.
4. **A format bump for two bools will look disproportionate.** It is. §F states
   the structural reason — splitting one sheet's controls across two files gives
   that sheet two write-back paths — and prices the bump honestly.

## What this task does not do

- **It does not build the Settings sheet** (§U4). It is designed in the spec and
  built by whichever of #102 or #84 lands first. A container built before its
  contents is the thing that spec exists to prevent.
- **It does not switch the header tags, the reel's lead-in and tail, or the
  avatar's pulse constants.** Each is refused with reasons in §X; the tags are
  the one open question below.
- **It adds no chapters switch to the basket sheet** (spec Deferred 3): only the
  chapters half could apply, and `basket.json` is a separate storage path with no
  version and a whole-document read.
- **It changes nothing in `pundit-media`.** If the implementation reaches for a
  media change, stop: §S2's design has been abandoned and the plan is wrong, not
  the crate.

## The coach's open question — ANSWERED 2026-10-06: no third switch

**"The header tags: did *'all of them'* include those?"** (§X1.) Put to the coach
with the timing spelled out — cheap now, its own format bump later — and he chose
**no: leave them on, unswitched.** So this plan ships **two** switches and §X1's
refusal stands as the design rather than as a default nobody confirmed. The
reasoning below is kept because it is why the answer is the right one, not
because the question is still open.

**It blocked nothing either way.** The plan ships two switches regardless.

- **If it is unanswered, the default is the spec's:** the `moov/udta` tags stay
  on, unswitched, because they are not files, they cost nothing (measured: *"not
  a sample changes"*), and `x264enc` pushes an `ENCODER` tag of its own into the
  same muxer under a `Keep` merge — so "tags off" could not even produce an
  untagged file on the software encoder.
- **If it is answered "yes" before task 1 lands**, it is cheap and it rides this
  bump for free: a third `Preferences` bool, a third checkbox, and
  `job.tags = FileTags::default()` — already an untagged file, so it is the same
  blanking move as the other two, and `ExportJob::tags` is read in one place.
- **If it is answered "yes" afterwards**, it costs a bump of its own. That is the
  only thing the timing changes, and it is worth asking before task 1 for exactly
  that reason.

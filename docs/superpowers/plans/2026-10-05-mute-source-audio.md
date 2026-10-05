# Plan — muting the source audio in an export

Spec: `docs/superpowers/specs/2026-10-04-mute-source-audio-design.md`. **Read it
first**, with the two corrections in "The spec is wrong about two things" below.
Read `CLAUDE.md` too.

**Revised after two adversarial passes.** The first draft had three tasks, two of
which edit the same 23 lines of `bus/export.rs`; a grey line that contradicted
its own spec; and a write-back test that pinned nothing. §R keeps the record.

**Gates.** Every `cargo` call but `fmt` under
`flock /tmp/claude-1000/cargo.lock nice -n 19`. Clippy must be
`rustup run 1.92 cargo clippy --workspace --all-targets -j 3 -- -D warnings` —
a clean local 1.98 is not the gate, and `-j 3` is required (#121). **Never
`cargo test --workspace`.** Never pipe clippy or tests to `tail`/`head`.

**Known flakes, not yours:** #72 (harness `lib.rs:164`, "timed out waiting for a
settled position") and #101 (`corrupted size vs. prev_size`, SIGABRT, no test
named). **Log any sighting** — #101's count now schedules the work, by the
coach's rule.

## Where this stands (update it as tasks land)

- **1 — the mute, end to end.** Done. `CURRENT_FORMAT_VERSION` took **15**
  (#115 had not landed, so no bump was shared). **Two deviations**, both
  recorded here rather than left to be rediscovered:
  - **`carry_scoreboard` did *not* take the bool** (step 10's first clause). In
    task 1 `can_copy` keeps its signature, so the argument would have had no
    reader and `-D warnings` fails an unused parameter. Everything else in
    step 10 stands — `Carry` gains no field, `job` builds
    `Render::Copy { files, with_audio }` from `pickers`, and the one negation
    is at the bus's boundary. Task 2 adds the argument along with
    `can_copy(&files, with_audio)`, which is its only reader.
  - **The write-back test runs muted *first*, then unmuted** — the reverse of
    the Tests section's order, which does not catch its own sabotage proof.
    `Rig::export` sends 720p/Low, so a run that *adds* the mute differs from
    the stored prefs in three fields and writes back whatever `Pickers::of`
    read; with `of`'s volume hardcoded `1.0` that run still differs and still
    passes. Asking a **muted** project for an **unmuted** run is the direction
    the stored value has to be read to notice: `of` == `pickers`, no write-back,
    and the project stays muted. §R.5 twice over, and the test's own doc
    comment says why.
- **2 — the copy gate's widening.** Not started.

**They are sequential and ship together.** The first draft claimed two of its
three tasks were file-disjoint and parallel; they both rewrite `job` in
`bus/export.rs`, 23 lines apart. The seam that *does* work is behaviour, not
layer: task 1 is the whole feature, task 2 adds what the gate makes newly
possible.

**One PR, one release.** Task 1 bumps `formatVersion`: a build between 1 and 2 is
fine (1 is self-contained), but no build may reach the coach between them that
stamps v15 without the control, because then any 0.11.x build refuses the file
and he is back to `project.json.v14` for nothing.

---

## 1. The mute, end to end

**Files.** Core: `crates/pundit-core/src/{project.rs, store.rs, audio.rs}`,
`crates/pundit-core/tests/{audio.rs, project_format.rs}`. Media:
`crates/pundit-media/src/composite/{export.rs, copy.rs}`,
`crates/pundit-media/tests/{copy.rs, export.rs}`. App:
`crates/pundit-app/src/bus/{export.rs, basket.rs, mod.rs}`,
`crates/pundit-app/src/main.rs`, `crates/pundit-app/ui/app.slint`. Harness:
`crates/pundit-harness/tests/{export.rs, whole_match.rs, basket.rs, preview.rs,
new_match.rs, reel.rs, highlights.rs}`. Docs: `CLAUDE.md`.

### The stored level

1. **Rename `Preferences::preview_source_volume` → `export_source_volume`**, with
   `#[serde(alias = "previewSourceVolume")]`. **Verified by running it:** under
   the container's `rename_all = "camelCase"` the alias is matched **verbatim**
   (so the camelCase spelling is required — `preview_source_volume` would not
   match), aliases are deserialize-only so saves write one key, and `{}` or a
   partial object still fills from the hand-written `Default`. A document
   carrying **both** keys is a hard duplicate-field error; unreachable, since
   every save rewrites the whole document.
2. **No field-level `#[serde(default)]`** — the container has one and `Default`
   keeps `1.0`. The field therefore **keeps its place in
   `project_format.rs::preferences_defaults_are_not_zero`** (it asserts `== 1.0`
   today) and in `partial_preferences_keep_real_defaults_for_missing_keys`; both
   need the new name.
3. **Bump `CURRENT_FORMAT_VERSION`.** Take the **next free number at
   implementation time** — check `store.rs`; it is 15 unless #115 landed first.
   Floor stays 7. **If #115 is being built in the same stretch, share one bump.**
   - **The reason is the copy path, not the rename.** An older build's
     `carry_scoreboard` consults **no stored field at all**, so a muted whole
     match would be **stream-copied with its sound** whatever the key were
     called. (Had the key been kept, an older build would at least honour the
     mute on the *encoded* path, reading gain `0.0` — which is why "an older
     build falls back to 1.0" is only half the story and the rename is not what
     forces the bump.)
   - `store::write` names the backup from the project's **own** version, so a
     project at v12 gets `project.json.v12`.

### Core's parameter

4. **`audio_regions(&compilation, &prefs, source_volume: f64)`**, and core
   **stops reading the stored field**. The rule is
   `if source_volume == 0.0 { no game region } else { gain = source_volume }`.
   - **Why a parameter:** `bus/basket.rs` passes `Preferences::default()` *on
     purpose* (J6). While core reads the gain out of the struct, a muted basket
     can only fake a non-default `Preferences` at the one call site whose comment
     says it is untouched. Spec M2 has the argument.
   - **`prefs` stays an argument**: `preview_commentary_volume` is still read
     from it (spec M2 claims otherwise and is wrong). Do not narrow to two
     floats — #124's territory.
   - **Rewrite `audio_regions`' doc comment.** It opens *"Gains are the
     **preview** volumes: one pair of numbers decides how loud a clip is in the
     app and in the file"*, which is now false in both halves.
   - `0.0` and `1.0` round-trip exactly through JSON and the checkbox writes
     literals, so the equality is exact. **#126's slider must quantize its
     bottom detent to exactly `0.0`**, because the zero branch is an equality.
5. **Call sites:** `bus/export.rs::job` passes `prefs.export_source_volume`;
   `bus/basket.rs` passes its own stored value; the **five** sites in
   `media/tests/export.rs` (367, 470, 1653, 1815, 2012) and the two in
   `core/tests/audio.rs` pass `1.0`.

### The copy path honours it

6. **`Render::Copy` becomes a struct variant**:
   `Copy { files: Vec<PathBuf>, with_audio: bool }` — **`with_audio`, not
   `audio`**, which is the word `Source::play` already uses, and it reads
   unambiguously beside `files`. A tuple variant had nowhere for the choice to
   live, and a field on `ExportJob` would put copy-only data where both renderers
   see it, inverting the rule that `Render::Encode` carries only what the encoder
   reads.
7. **Touch points:** `composite/export.rs`'s variant (`:94`), the `debug_assert`
   in `Exporter::spawn` (`:263`), **`run`'s match arm (`:414`), which must
   destructure and thread it into `copy()` — whose signature changes too**,
   `bus/export.rs::job` (`:801`), and `media/tests/copy.rs::job` (`:88`).
8. **In `copy()`, the one-line form is
   `let audio_rate = declare(&files, &watch)?.filter(|_| with_audio);`** and
   nothing else changes. Every downstream rule already handles `None`:
   `Source::play` builds no audio branch and leaves the demuxer's audio pad
   unlinked, `Output::start` requests no `audio_%u` pad. **Task 2 moves the mute
   into `declare` itself**; this task deliberately does not, so task 1 is one
   self-contained change. *(Unlinked `qtdemux` pads are already an exercised path
   here — `read_header` and `Source::play` both leave non-A/V pads unlinked with
   comments saying qtdemux is happy with it. Review reproduced the muted mux end
   to end on GStreamer 1.24.2: EOS, exit 0, 90/90 video packets, and the video
   elementary stream md5-identical to the source.)*

### The sheet, the basket and the write-back

9. **`Pickers` gains `source_volume: f64`** and **loses its `Eq` derive** — `f64`
   is not `Eq`. `PartialEq` is all the write-back's `!=` needs, and the only
   values are `0.0` and `1.0`. Keep the `f64` rather than a `bool` so #126's
   slider is pure UI.
10. **`carry_scoreboard` takes the bool as an argument; `Carry` does not gain a
    field.** It is `can_copy`'s **only** caller, in the Default-Track branch, so
    without it "Default" asks the unmuted question and loses task 2's widening.
    But a `Carry { with_audio }` would be a verbatim copy of something `job`
    already holds — two truths for one fact, and a field the encode arm ignores.
    `job` builds `Render::Copy { files, with_audio }` from `pickers` directly.
    **One negation, at the boundary:** `mute` at the picker and UI layer,
    `with_audio` at the renderer layer.
11. **`main.rs` is not optional** and the first draft omitted it: the
    `Command::Export` send (`:919`), the `Command::ExportBasket` send
    (`:1152-1156`), the picker seeding when the sheet opens (`:1084-1090`), and
    `show_basket`'s "follow the bus **only while the sheet is closed**" block
    (`:3350-3354`), which the basket checkbox must sit inside per `CLAUDE.md`'s
    basket rule. `BasketView` carries the bool so `show_basket` can seed it.
12. **Two checkboxes, "Mute source audio"** — one in the export sheet beside the
    three pickers, one in the basket sheet. Ticked writes `0.0`, unticked `1.0`.
    **No slider** (#126), and **no explanatory line** (see "cut from the first
    draft").
13. **`basket.json` gets a plain `bool`.** Not a string label: that rule is about
    an enum gaining variants. Its `read` discards the **whole document** on a
    parse error (there is no per-field lenient read — do not claim one), which is
    acceptable because no build writes a non-bool. **Say in the code that a
    downgrade silently unmutes a basket**: a build without the field ignores the
    key and drops it on its next save. That is the right trade for machine state
    with no version, but it should be written down rather than discovered.
14. **The basket's eager save is existing, deliberate asymmetry.**
    `export_basket` calls `basket.save()` *before* `basket_job()`'s refusals, so
    a refused Start still remembers the mute — the opposite of the sheet's
    `a_refused_run_leaves_the_pickers_alone`. Losing `basket.json` costs a
    re-gather, which is why. Note it so nobody "fixes" it.
15. **Every `Command::Export` / `ExportBasket` sender needs the field.** In the
    harness: `export.rs:111`, `whole_match.rs:127`, `preview.rs:109`,
    `new_match.rs:594`, `reel.rs:116/252/377`, `highlights.rs:312`, and
    `basket.rs:133/187/222/310/421`. `whole_match.rs`'s and `export.rs`'s
    `export()` helpers take a new parameter.
16. **`CLAUDE.md` edits**, which the first draft skipped entirely: the
    format-version list gains a v15 line; "The export sheet's **third** picker is
    Scoreboard…" becomes wrong as written; `can_copy`'s description ("a header
    read per file that the bus asks before it chooses a renderer") now also
    carries the mute; and the export-audio bullet needs the mute. *(v14 shipped
    without a `CLAUDE.md` line, so the convention has slipped once — this change
    is more visible than an arrowhead.)*

### Tests

- **Core (`core/tests/audio.rs`).** At `0.0`: every region's track is
  `Commentary`, **and the commentary regions equal the `1.0` run's** —
  compared as `regions.iter().filter(|r| r.track == Track::Commentary)` of each
  run, **not** the whole `Vec`, which interleaves game regions and would fail.
  `regions_at` and `track` already exist. Same test asserts a reel entry
  (`clip_id: None`) yields **no regions at all**. The assertion is `PartialEq` on
  a gain that is literally the same number both runs.
- **Media (`media/tests/copy.rs`).** A muted copy has **no audio stream**, and
  its packet list and `video_stream()` tuple equal the unmuted copy's — "packet
  for packet", the module's own phrase. `ffprobe` cannot hash a bitstream; review
  measured that the elementary stream *is* md5-identical, so an `ffmpeg -f md5`
  helper is three lines if the stronger claim is wanted (`ffmpeg` is already a
  test-only build dependency).
- **Harness.** The write-back test must be built properly, because the obvious
  version pins nothing: `Rig::export` always sends `R720`/`Low`, which already
  differ from the project's defaults, so `Pickers::of(prefs) != pickers` is true
  **regardless of the new field** and an added assertion would pass even if
  `Pickers::of` read the wrong thing. **Export once unmuted at R720/Low and let
  it persist, then export muted and assert `ProjectChanged` plus `0.0` saved** —
  a run where the mute is the only difference from the stored prefs. And in
  `a_refused_run_leaves_the_pickers_alone` the rig must send `0.0`, or asserting
  `== 1.0` is vacuous. Plus a bus-level muted copy in `whole_match.rs`, which
  already builds `H264AacMp4` sources and needs no new fixture.
- **Format.** `a_v<prev>_file_loads_under_the_current_version` beside its
  siblings — the series is named for the **old** version, there is no
  `v7_to_v14` test, and **the literal `project.json.v<prev>` filename in it is
  the only thing that fails if the constant never moved** (every version
  assertion reads `CURRENT_FORMAT_VERSION`). Add one asserting a file written
  with the old `previewSourceVolume` key still reads.

### Sabotage proof

Set the core rule to `gain = source_volume` with no zero branch: the "no game
region" assertion must fail while the commentary one passes. And have
`Pickers::of` return a constant `1.0`: the **rebuilt** write-back test must fail.
*(Not "drop the field from `Pickers::of`" — that is a compile error, not a
failing test.)*

---

## 2. The copy gate's widening

**Files:** `crates/pundit-media/src/composite/copy.rs`,
`crates/pundit-media/tests/copy.rs`, `crates/pundit-app/src/bus/export.rs`.

1. **The mute reaches `declare`**: `declare(&files, with_audio, &watch)` and
   **`can_copy(&files, with_audio)`** — `can_copy` **is** `declare`
   (`copy.rs:171`), so the bus must ask the right question or Default mode
   re-encodes a file it could have copied.
2. **The seam is `if !with_audio { header.audio = None; }`** as each header is
   read. **Verified: there is no sixth reader.** `header.audio` is set at
   `copy.rs:302` and read only by `absolutely` (`:353`), `agree` (`:372`,
   `:377`) and `audio_rate` (`:415`). With it `None`: `absolutely` skips its
   audio arm, both `agree` checks pass (`renegotiable(None, None)` is `true`),
   `audio_rate` returns `Ok(None)`. Nothing caches a rate; `Copying::crowded`
   and `over_ceiling` take their existing `audio: None` branches. A
   **single**-source copy behaves identically to a multi-source one — `agree` is
   never called for one file, and `audio_rate` reads `first` either way.
3. **This can only widen.** Every audio check becomes trivially true or is
   skipped and the video checks are untouched, so muting cannot make a copy fail
   that previously succeeded — and it legitimately makes copyable two halves
   that differ only in sound, or H.264 video with an AC-3 track.
4. **Why task 1 does not do this**, recorded so the two signature changes do not
   look unmotivated: task 1's `.filter(|_| with_audio)` ships a muted copy with
   no change to `declare` or `can_copy` at all. What it loses is exactly the
   widening — `absolutely` would still refuse a non-AAC track, and `agree` still
   refuse mismatched halves, on a copy carrying no sound. The widening is worth
   its two signatures; the cheap version is worth recording as what was passed
   over.

**Tests.** The headline is **a pair of halves differing only in audio: refused
unmuted, copied when muted.** No existing helper can build it — `whole_match_with`
applies **one** `CounterKind` to every source. The pair is `H264AacMp4` +
`H264Mp4BFrames` at matching w/h/fps, whose video legs are the same launch
fragment; `counter_video_with` is `pub`, so build the `Match` by hand in the
test. **This is the one unbudgeted piece of work in task 2.** If `renegotiable`'s
`codec_data` subset check makes two separate x264 runs disagree, the test fails
for the wrong reason and the fixture wants an AAC leg added to a copy of the
existing kind instead.

**Sabotage proof** — the one that earns its place, because it reproduces a bug
that was actually written: move `header.audio = None` to *after* `declare` and
confirm the widening test fails (the gate refuses) while the no-audio-stream test
still passes.

---

## Cut from the first draft

**The grey line in the export sheet.** It contradicted its own spec: M6 says a
silent film is "a legitimate thing to want … **not a refusal and not a
warning**", and the draft added the warning anyway while admitting the coach
might veto it. It is also not free — there is no reusable "one line per trade
slot"; the existing line is a single `Text` inside the Scoreboard column
conditioned on `whole-match-ticked`, and the muted-silent case is whole match
**or reel**, for which no `reel-ticked` property exists. **Backlog it** and let
the coach ask, having used the checkbox.

## The spec is wrong about two things

Both inherited from its bool draft, and the plan above is right where it differs:

1. **M2's "no `Preferences` field is read inside core's audio path at all"** is
   false: `audio_regions` still reads `preview_commentary_volume`.
2. **M5's bullet about the `bool` default hazard** and "the first `Preferences`
   field whose correct default is the zero value" is left over — the field's
   default is `1.0`, so `preferences_defaults_are_not_zero` covers it and must
   simply be renamed. M7's `v7_to_v14` test name does not exist either, which
   the plan already corrects.

## What this feature does not do

- **No slider** (#126): a partial level cannot ride a stream copy and needs a
  fourth branch in `copy.rs` — demux, decode, `volume`, `avenc_aac`. It costs
  **one AAC generation on the sound**, not a re-encode of the picture. Deferred
  at the coach's direction. Its bottom detent must quantize to exactly `0.0`.
- **No preview change** (#125): a preview has no source audio to mute. Muting
  makes an export finally *match* its preview rather than fixing the mismatch.
- **It does not fix the coach's original report.** Both symptoms were correct
  behaviour. Spec §R.5 says so; do not let the entry close on a false claim.
- **`preview_commentary_volume` stays unreachable** (#124's remaining half).
- **Stale doc references** to `preview_source_volume` survive in four specs
  (`2026-09-22-match-vision`, `2026-09-24-clip-basket`, `2026-09-19-linux-port`,
  `2026-09-19-linux-port-phase-8`). Not worth a sweep; noted so a reader is not
  misled.

## §R. What the first draft got wrong

1. **Three tasks, two of them claimed file-disjoint and parallel.** Tasks 2 and
   3 both rewrote `job` in `bus/export.rs`, 23 lines apart, and task 2 was not
   independently meaningful — `Render::Copy`'s bool would have been hardcoded
   `true` until task 3. Reseamed by behaviour.
2. **A format bump landing naked.** The bump's whole purpose is to stop an older
   build dropping a stored value, and between the old task 1 and task 3 there
   was no value to protect — only a coach's project stamped v15 that 0.11.x
   would refuse.
3. **The bump's reason was half the story** — true only because of the rename.
   The rename-independent reason is the copy path, which no older build knows to
   drop.
4. **A grey line that contradicted the spec** and assumed a UI slot that does not
   exist.
5. **A write-back test that pinned nothing**, because `Rig::export` already
   differs from the project's defaults, and a sabotage proof that was a compile
   error rather than a failing test.
6. **"Byte-identical commentary regions"** read as comparing the whole `Vec`,
   which interleaves game regions — a trap that would push an implementer to
   weaken the assertion.
7. **`main.rs` missing from the file list**, along with eleven harness senders of
   `Command::Export` / `ExportBasket` and `CLAUDE.md`.
8. **`Carry` gaining a field** that `job` already holds, and `Render::Copy`'s
   flag named `audio` rather than the `with_audio` already in the file.
9. **A tautological sabotage proof** (drop the alias, watch the alias test fail).

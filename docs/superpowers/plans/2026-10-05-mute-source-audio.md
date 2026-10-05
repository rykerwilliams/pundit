# Plan — muting the source audio in an export

Spec: `docs/superpowers/specs/2026-10-04-mute-source-audio-design.md`. **Read it
first; this plan does not repeat its reasoning.** Read `CLAUDE.md` too.

**Gates.** `CLAUDE.md`'s build conventions. Every `cargo` call but `fmt` under
`flock /tmp/claude-1000/cargo.lock nice -n 19`. Clippy must be
`rustup run 1.92 cargo clippy --workspace --all-targets -j 3 -- -D warnings` —
a clean local 1.98 is not the gate, and `-j 3` is required (#121). **Never
`cargo test --workspace`.** Never pipe clippy or tests to `tail`/`head`.

**Known flakes, not yours:** #72 (harness `lib.rs:164`, "timed out waiting for a
settled position") and #101 (`corrupted size vs. prev_size`, SIGABRT, no test
named). **Log any sighting** — #101's count now schedules the work, by the
coach's rule.

## Where this stands (update it as tasks land)

- **1 — the stored level and core's parameter.** Not started.
- **2 — the copy path's gate.** Not started.
- **3 — the sheet, the basket and the write-back.** Not started.

**Order:** 1 first — 2 and 3 both depend on its signature. 2 and 3 are
**file-disjoint** (`pundit-media` vs `pundit-app`) and can run in parallel after
1 lands.

---

## 1. The stored level, the rename, and core's new parameter

**Files:** `crates/pundit-core/src/project.rs`, `store.rs`, `audio.rs`,
`crates/pundit-core/tests/{audio.rs, project_format.rs}`, plus the mechanical
call-site updates in `crates/pundit-app/src/bus/{export.rs, basket.rs}` and
`crates/pundit-media/tests/export.rs`.

1. **Rename `Preferences::preview_source_volume` → `export_source_volume`**, with
   `#[serde(alias = "previewSourceVolume")]` so every existing project reads. No
   field-level `#[serde(default)]` — the container already has one, and the
   hand-written `Default` keeps `1.0`.
2. **Bump `CURRENT_FORMAT_VERSION`.** **Take the next free number at the time you
   implement**: it is 15 unless BACKLOG #115 has landed first, in which case 16.
   Check `store.rs` rather than trusting this sentence. Floor stays 7.
   **If #115 is being built in the same stretch, share one bump** — one version,
   both fields, one per-bump test.
   - The bump is for the **write**, not the read: the alias means older files
     load, but once this build saves `exportSourceVolume` an older build would
     not know the key, fall back to `1.0`, and export at full volume a film the
     coach had muted.
3. **`audio_regions` gains a parameter**:
   `audio_regions(&compilation, &prefs, source_volume: f64)`, and **stops reading
   the stored field**. The rule is
   `if source_volume == 0.0 { no game region } else { gain = source_volume }`.
   - `prefs` stays an argument — `preview_commentary_volume` is still read from
     it. Do **not** narrow it to two floats; that is #124's territory.
   - **Why a parameter and not the field:** `bus/basket.rs` passes
     `Preferences::default()` *on purpose* (J6). While core reads the gain out of
     the struct, a muted basket can only fake a non-default `Preferences` at the
     one call site whose comment says it is untouched. Spec M2 has the whole
     argument.
4. **Call sites:** `bus/export.rs::job` passes `prefs.export_source_volume`;
   `bus/basket.rs` passes its own stored value (task 3); the five
   `media/tests/export.rs` sites pass `1.0`.

**Tests.** `core/tests/audio.rs`: at `0.0` every region's track is
`Commentary`, **and the commentary regions are byte-identical to the `1.0`
run's** — that second half is what pins "muting touches one track". Same test
asserts a reel entry (no clip) yields **no regions at all**. A non-zero,
non-one gain still reaches the regions (the behaviour that exists today).
`project_format.rs`: **`a_v<prev>_file_loads_under_the_current_version`** beside
its siblings — the series is named for the *old* version, there is no
`v7_to_v14` test, and that per-bump test is the only thing pinning the constant.
Add one asserting a file written with the old `previewSourceVolume` key still
reads through the alias.

**Sabotage proof.** Drop the alias and confirm the old-key test fails and the
others pass. Set the rule to `gain = source_volume` without the zero branch and
confirm the "no game region" assertion fails while the commentary one passes.

---

## 2. The copy path's gate

**Files:** `crates/pundit-media/src/composite/{copy.rs, export.rs}`,
`crates/pundit-media/tests/copy.rs`.

1. **`Render::Copy` becomes a struct variant**:
   `Copy { files: Vec<PathBuf>, audio: bool }`. The tuple variant carried only
   the files, so there was nowhere for the choice to live. Touches
   `bus/export.rs::job`, the `debug_assert` in `Exporter::spawn`, and
   `media/tests/copy.rs::job`.
2. **The mute reaches `declare`, which is the whole point of this task.**
   `declare(&files, audio, &watch)` and **`can_copy(&files, audio)`** — `can_copy`
   *is* `declare` (`copy.rs:171`), so the bus must ask the right question or
   Default mode silently re-encodes.
   - **The one-line form is `if !audio { header.audio = None; }`** on each header
     as it is read. Every downstream rule then takes the *already handled*
     no-audio path with **no new conditions**: `absolutely` skips its audio arm,
     `agree`'s two audio checks pass, `audio_rate` returns `Ok(None)`,
     `Source::play` builds no audio branch, `Output::start` requests no pad.
   - **This can only widen what can be copied.** The gate loses checks and gains
     none, so muting cannot make a copy fail that previously succeeded — and it
     legitimately makes copyable a project whose halves differ only in sound.
3. **It is the same *branch*, not the same path.** A silent source has no audio
   pad at all; a muted copy leaves a linked demuxer's audio pad **unlinked**,
   relying on `qtdemux`'s flow combiner tolerating `NOT_LINKED`. Measured in
   review: `filesrc ! qtdemux name=d d.video_0 ! h264parse ! mp4mux ! filesink`
   runs to EOS cleanly. The module header is careful about this class of claim;
   keep it so.

**Tests** (`media/tests/copy.rs`). A muted copy has **no audio stream**, and its
packet list and `video_stream()` tuple equal the unmuted copy's — "packet for
packet", the module's own phrase. **Do not assert bit-identity via `ffprobe`**:
it cannot hash a bitstream. (Review measured the elementary stream *is*
md5-identical and all 90 packets match, but proving it needs an `ffmpeg` helper
`fixtures.rs` does not have. Add one only if the stronger claim is wanted.)
Also: a project whose two halves differ only in audio **is refused unmuted and
copies when muted** — the widening, which is the task's real claim.

**Sabotage proof.** Move the `header.audio = None` to *after* `declare` and
confirm the widening test fails (the gate refuses) while the no-audio-stream test
still passes. That is exactly the first draft's bug.

---

## 3. The sheet, the basket, and the write-back

**Files:** `crates/pundit-app/src/bus/{export.rs, basket.rs, mod.rs}`,
`crates/pundit-app/ui/app.slint`, `crates/pundit-harness/tests/export.rs`.

1. **`Pickers` gains `source_volume: f64`**, read in `Pickers::of(prefs)` from
   `export_source_volume`, so the existing "only when it changed" write-back
   covers it with no new machinery. It stays outside undo with its siblings.
2. **`Command::Export` carries it**, and **`carry_scoreboard` is where it maps**
   — it already returns `Carry { copy, cues, scoreboard }` and already decides
   `copy`, so it is the function that must know, or it will choose a copy without
   knowing whether to ask `can_copy` about audio. Rename it if its name stops
   fitting; `Carry` gains `audio: bool`.
3. **The sheet's control is a checkbox, "Mute source audio".** A switch over
   level-shaped storage: ticked writes `0.0`, unticked `1.0`. The slider is #126
   and is **not** built here.
4. **The basket gets the same checkbox**, and `basket.json` a plain `bool`
   beside the name, resolution and quality. **Not** a string label: that rule is
   about an enum gaining variants. Note `basket.json`'s read discards the
   **whole document** on a parse error (it has no per-field lenient read), which
   is acceptable because no build writes a non-bool — but do not claim the
   lenient read exists.
5. **One grey line in the sheet, for the silent case only.** Where the result
   will carry **no sound at all** — a muted whole match or reel, neither of which
   has clips and so has no commentary region either — the sheet's existing
   one-line-per-trade slot says so. A muted **clip** gets no warning: it keeps the
   commentary, and a silent film is a legitimate thing to want. **This is a
   telling, never a refusal**; the coach may veto the line.

**Tests** (`harness/tests/export.rs`). Two assertions added to the **existing**
write-back test and to `a_refused_run_leaves_the_pickers_alone` — not a new
group. Plus a bus-level muted copy in `harness/tests/whole_match.rs`, since the
mute arrives through `Command::Export`.

**Sabotage proof.** Drop `source_volume` from `Pickers::of` and confirm the
write-back test fails while the refusal test still passes.

---

## What this feature does not do

- **No slider** (#126): a partial level cannot ride a stream copy, and needs a
  fourth branch in `copy.rs` — demux, decode, `volume`, `avenc_aac`. It costs
  **one AAC generation on the sound**, not a re-encode of the picture. Deferred
  at the coach's direction.
- **No preview change** (#125): a preview has no source audio to mute. Muting
  makes an export finally *match* its preview rather than fixing the mismatch.
- **It does not fix the coach's original report.** Both symptoms were correct
  behaviour. Spec §R.5 says so; do not let the entry close on a false claim.
- **`preview_commentary_volume` stays unreachable** (#124's remaining half).

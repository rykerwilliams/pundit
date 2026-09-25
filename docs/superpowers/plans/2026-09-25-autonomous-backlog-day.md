# A day of backlog, unattended — 2026-09-25

The coach is using 0.8.0 for real work while this runs. That sets the rules
before it sets the order.

## Rules while they are using the machine

- **Their footage and projects are untouchable.** No `Bus`, no writes, nothing
  under `/pcloud` or `~/pCloudDrive`. Every test uses fixtures.
- **Every cargo call stays under `flock /tmp/claude-1000/cargo.lock nice -n 19`.**
  They are running an app that decodes video on this machine; a release build at
  full tilt would show up as dropped frames in their scan.
- **`main` stays green and installable.** Work lands on the branch, is verified
  (fmt, clippy, core, workspace), then fast-forwards `main`. A `.deb` is built at
  the end of each batch so they can install a whole batch at once, never
  mid-flight.
- **No question blocks progress.** Anything needing their judgement is written
  into "Waiting on the coach" below and skipped, not guessed at.

## Order, and why this order

Value to the person using the app today, then risk, then size.

### Batch 1 — the day-of-use annoyances (small, visible)

1. **#94 zoom and pan are undiscoverable.** They asked how to pan within an hour
   of installing. The `ZoomIndicator` is already over the picture: say how to
   zoom while the zoom is at identity, in the shape of the existing
   `DRAWING_HINT`.
2. **#56 the score label overflows its cell at double-digit scores.** Measured,
   fixed by fitting, which every other scoreboard label already does.
3. **#73 `,` looks stuck across a timestamp gap longer than half a frame.**
4. **#68 the volume slider is Slint's stock one**, so a drag commits per pixel.

### Batch 2 — the layout they have now asked for twice

5. **#87 resizable panels** and **#95 the player area taking the footage's
   aspect**, in one pass, because #95's slack has to go somewhere and #87 is
   where it goes.

### Batch 3 — the next feature, whole

6. **#92 slates**, through the full workflow in `CLAUDE.md`: spec → adversarial
   review → plan → adversarial review → execute → review. Its four open
   questions are answered with the defaults recorded in the entry (keep the
   slate after its take; the out point advises rather than binds; in-app
   ranges only; one list with the clips, greyed) — each one reversible, each
   one called out at hand-back.

### Batch 4 — correctness, with a test each

7. **#28 non-finite floats silently corrupt a project on save.**
8. **#58 `scan_abs` can pair a new source's index with the old source's offset.**
9. **#74 a reel trim can be a silent no-op.**
10. **#63 a truncated recording transcribes as complete.**

### Batch 5 — the flakes that cost releases

11. **#70 `GST_REGISTRY` per test binary**, the experiment the entry now names:
    three faces of one pattern, all under parallel workspace runs, the newest a
    segfault *inside* GStreamer's registry fork.
12. **#72 a bounded load/seek in the bus** — an unbounded wait is a silent wedge
    in the app, not only in CI. The entry says ~30 lines and that it will not
    make the test pass; it is still right for the app.

## Waiting on the coach — not started, not guessed

- **#84 music under a goals reel** — which library, and the licence terms. No
  amount of code answers that.
- **#79 what the app does over a forwarded X11 display** — needs a remote
  display to test against.
- **#82 P4, showing detection suggestions** — the measurement says it is not
  justified; that verdict is theirs to overturn, not mine.
- **#80 the sixteen restarts** — their data to write down.
- **#53 2160p export** and **#24 AppImage/Flatpak** — product decisions.
- **#93 delete the rename shims** — deliberately dated to after 0.9.0.

## Done means

fmt, clippy `-D warnings`, `cargo test -p pundit-core`, `cargo test --workspace`,
the core dependency audit, a commit whose message says what was decided and what
was rejected, and the backlog entry closed with the evidence. A batch ends with
a `.deb` and a one-line note of what to look at.

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

### Batch 1 — the day-of-use annoyance (done)

1. **#94 zoom and pan are undiscoverable.** They asked how to pan within an hour
   of installing. The drag that does nothing already raises a notice, so that
   notice now answers both readings of the gesture rather than only "press R".

**Six items were triaged out of this batch, and the reason is the same one:**
#56 (the score label at double digits), #73 (`,` across a real timestamp gap),
#58 (`scan_abs` pairing), #63 (a truncated recording transcribing as complete),
#68 (the stock volume slider) and #74 (a reel trim that is a silent no-op) each
carry a "when to revisit" that has **not** happened — a format that scores in
double digits, a coach reporting the symptom, a source with real gaps. Their
fixes all cost more than the problem today, which is what the entries already
say. Doing them now would be inventing work against this project's own rule
that every change must earn its place. They stay as written.

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

---

## Where it stands (2026-09-26, written for the session that picks this up)

**Shipped and released.** `v0.9.0` is published from `rykerwilliams/pundit`
with its `.deb` attached. Done since this plan was written: the rename to
pundit and the split out of the fork (0.8.0), the wheel's direction and its
reach over the picture, the zoom/pan hint (#94), the backlog's index and four
stale entries corrected, and **slates end to end** (#92 — spec, plan, both
reviews at every handoff, six tasks, 943 tests green).

**Next, and it is the top of the index:** **#87 resizable panels together with
#95's fit-to-video.** Nothing is started — no spec, no branch state to inherit.

Read before starting: `BACKLOG.md` #87 and #95 (in that order — #95 carries the
layout finding that changes what it is), then `CLAUDE.md`. The two things a
fresh session would otherwise rediscover:

- **Panels cannot remove the black bars.** The window is
  `sidebar (240px) | player | inspector (280px)` in a `HorizontalLayout`; the
  coach's bars are above and below the picture. The slack is vertical and the
  panels are horizontal. Narrowing a panel widens the player, which reduces
  letterboxing only until the player area is 16:9, then pillarboxes.
- **So the coach chose "Fit window to video"** (2026-09-25): an action that
  resizes the *window* so the player area is the footage's aspect exactly.
  Never automatic — the window moves when asked. A splitter snap at that shape
  comes with it.

Then #88 (per-clip inset), #85 (recents), #78 (settings), #96 (rebindable
keys, which wants #78 first), #77 (export queue), #84 (music, wants #78).

**Watch for:** `#87` touches `app.slint`, where the slates section, the
inspector and the clip list all live — and where two of the three worst bugs of
the last pass came from putting a field in the wrong place. The rules that
caught them are in `CLAUDE.md` and worth re-reading before editing that file:
a field is hidden rather than removed, and an `editing` flag is derived, never
assigned.


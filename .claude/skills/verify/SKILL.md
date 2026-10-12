---
name: verify
description: Run the Rust verification gate the way CI does — fmt, clippy on CI's pinned 1.92 toolchain, the tests, and the core dependency audit — under this machine's shared cargo lock, and tell a known flake from a regression. Use before every commit to crates/, and whenever asked whether the build is green.
---

# Verify

Run from the repo root. **Collect everything, then report.**

## The commands

```bash
L="flock /tmp/claude-1000/cargo.lock nice -n 19"   # every compiling cargo call
S=<scratchpad>

cargo fmt --all --check
$L rustup run 1.92 cargo clippy --workspace --all-targets -j 3 -- -D warnings
$L cargo test --workspace -j 3 --no-fail-fast > $S/test.log 2>&1
grep -E 'test result|FAILED|panicked|SIGABRT|^error' $S/test.log
cargo tree -p pundit-core --edges normal --depth 1
```

- **The lock**: other Claude sessions build on this laptop (memory
  `cargo-coordination`). `fmt` compiles nothing and stays unlocked.
- **Clippy on 1.92**, because CI pins it and lints move between releases
  (CLAUDE.md, "A clean local `clippy` is NOT the same gate as CI's").
- **`-j 3`** on every compiling call (BACKLOG #121).
- **`--no-fail-fast`**, or one aborting binary silently hides every suite after
  it.
- **Output to a file, never through `tail`/`head`**: a SIGABRT or a compile
  error scrolls away.
- **While iterating**, run just the binary you touched (`-p pundit-app --test
  ui`, `-p pundit-harness --test <name>`); the whole workspace is the final
  gate.
- **fmt runs on stable here and on 1.92 in CI.** They rarely disagree; if CI's
  fmt fails and local passes, run it through 1.92 (that toolchain needs
  `rustup component add rustfmt --toolchain 1.92`, which is the user's call).
- **No GStreamer dev headers** (`pkg-config --modversion gstreamer-1.0` fails):
  run `cargo test -p pundit-core -j 3` instead and report "core green; workspace
  not run" — never "green". Installing the headers needs `sudo`; ask first.

**Core audit:** `pundit-core`'s direct normal dependencies are exactly `serde`,
`serde_json`, `thiserror`, `uuid`. Anything else fails the gate even though it
compiles.

## A failure: regression or known flake?

**Classify by signature first.**

- **`SIGABRT` with `corrupted size vs. prev_size` or `malloc(): mismatching …`,
  no failing test named, in an export or whole-match binary** — BACKLOG #101, a
  known write fixed upstream after GStreamer 1.24.2; the coach chose "document
  and watch". It **can** fire with its suite run alone, so passing alone proves
  little. **Append a dated sighting to #101**; don't investigate.
- **Harness: `timed out waiting for a settled position`** — BACKLOG #72, which is
  **fixed**. A sighting is a regression: reopen #72 with the run.
- **Anything else**:
  1. Does the diff touch the failing code? A docs-only branch cannot cause a
     media abort.
  2. Re-run that binary alone at the same commit.
  3. Grep `BACKLOG.md` for the failure's message text. A match gets a dated
     sighting on **that** entry — never a new entry (2026-10-12 filed #101 again
     as #139 and withdrew it).
  4. An allocator abort's backtrace names whichever `malloc` tripped over the
     damage, **not** the writer.
  5. Log it and move on (memory `no-detours`) unless the user asks for a hunt.

Report a known flake as one — "N passed; one SIGABRT in `pundit-media --test
export`, #101's signature" — never folded into "green".

## Reporting

- Counts per binary and the total. **A drop in the number of suites or tests
  against the last stated total** (the plan's "Where this stands" block, or the
  last commit body) is a finding to explain.
- A failing test's assertion output, not just its name.
- An intentional lint is `#[allow(lint, reason = "…")]` and mentioned.
- **Say what only CI checks**: core without GStreamer, Windows (advisory), the
  docs book. The PR's checks are the gate (memory `pr-first-then-merge`).
- Never report "green" for a step you didn't run.

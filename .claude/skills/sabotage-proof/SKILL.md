---
name: sabotage-proof
description: Prove each new test can fail — put the bug back (or break the feature) one mutation at a time, confirm exactly the named test fails and its siblings stay green, then restore. Use after a task's tests pass and before committing, for every behaviour the task claims to pin, and whenever a plan lists sabotage proofs.
argument-hint: "<the behaviours / tests to prove>"
---

# Sabotage proofs

A passing test is not evidence until it has been seen to fail. In this
project's history, sabotage is what caught tests that could not fail — and they
recur.

## 1. Write the list before mutating

One line per claimed behaviour: **mutation → the one test expected to fail →
why that test and no other.** Good mutations are the realistic bug: a line moved
below a pause, a guard removed, a callback unwired or **crossed with its
neighbour**, a flag ignored, the old behaviour put back.

## 2. One mutation at a time

```bash
S=<scratchpad>; f=<file>; b="$S/$(basename "$f").bak"
cp "$f" "$b" &&
python3 <apply the mutation; assert it matched exactly once> &&
flock /tmp/claude-1000/cargo.lock nice -n 19 \
  cargo test -j 3 -p <crate> --test <binary> > "$S/sabotage.log" 2>&1
grep -E 'test result|FAILED|panicked|SIGABRT|^error' "$S/sabotage.log"
cp "$b" "$f" && cmp "$f" "$b" && echo restored
```

- Back up **every** file the mutation touches, each to its own name.
- **`cmp` against the backup is the check**, not `git diff --stat`: the file
  already carries the task's own edits, so a mutated file and a restored one
  look the same in a stat.
- Mutations with **disjoint** expected failures may share one build; say so,
  and check each failure landed where predicted.

## 3. Judge the result

- **Exactly the named test fails** — the proof holds.
- **No `test result` line at all** — the mutation didn't build or the binary
  aborted. That is not a result; read the log.
- **Others fail too** — fine if they are the same claim seen from another angle
  (say which); otherwise the tests aren't pinned apart. Where two clauses are
  tested separately, a mutation to one must leave the other's test green.
- **Nothing fails** — the test is vacuous. Fix the test.
- **It can't be run** — a compile error, correct by construction, or code no
  test reaches (`start-task` §5). **Say so; never claim it.** Record it as a
  risk, or move the logic where a test can reach it.

## 4. Test-shape traps seen here

- A UI test that **finds** a control must also **press** it (CLAUDE.md:
  `invoke_accessible_default_action`) — a button wired to nothing ships dead.
- `find_by_accessible_label` matches **every** element with that label — a
  sheet's title and its button, two "Choose…" rows. Assert the count, or press
  all and assert each callback fired once.
- A key test must press a **bound** key; an unbound one passes vacuously.
- The harness's `wait_for_error` skips past everything before the error — read
  earlier events first.
- A fixture must differ from the expected value in the dimension under test (a
  violet fixture once made a "no green" assertion pass on nothing).

## 5. Record

In the plan's "Where this stands" and the commit body: each mutation and the
test it failed, and any proof that couldn't fail, by name, with why.

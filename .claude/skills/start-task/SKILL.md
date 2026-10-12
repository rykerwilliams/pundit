---
name: start-task
description: Preflight before writing any code for a plan task, a backlog item or a fix — check nobody (including an earlier session) already did it, branch from fresh main, re-read the plan's status, re-derive its citations, and take a census of everything the change will touch. Use at the start of every task, and when resuming one.
argument-hint: "<plan path and task number, or backlog #N>"
---

# Start a task

## 1. Is it already done, or in flight?

Search by the backlog **number and** the topic's **name** — branches use both
(`claude/138-task-4`, `claude/recents-t1`).

```bash
git fetch -q origin
git status -sb
git branch -a --list '*<N>*' '*<name>*'
gh pr list --state all --search "<N>" --json number,title,state,headRefName
git worktree list
grep -nE '^<N>\. |\*\*<N>\.\*\*|^### <N>\.' BACKLOG.md
```

- An **open PR** for the task: rebase and finish it; don't redo it. (2026-10-12
  nearly rebuilt #77 task 3, open since 2026-10-09 as PR #70.)
- A branch with commits and no PR: read `git log origin/main..<branch>` first.
- The backlog entry may say RESOLVED or carry a status the plan doesn't.

## 2. Branch from fresh main

```bash
git switch -c claude/<N-or-name>-task-<n> origin/main
```

Never write on `main`. A task stacked on a prerequisite still in CI may branch
from that branch, then `git rebase --onto origin/main <prereq>` once it
squash-merges.

## 3. Re-read; don't remember

- The plan's **"Where this stands"** — earlier tasks record corrections that
  change this one.
- The task's section, the spec sections it cites, CLAUDE.md for the area.
- **Every `path:line` the task cites, re-derived** (`recheck-claims`). Line
  numbers in `app.slint` drift fastest.

## 4. The census — the plan's list of sites is a claim

Write the results down before editing; they are the task's real file list.

| Changing… | Find |
|---|---|
| a struct's fields | `cargo check --all-targets` lists every literal exactly; then `rg 'Type \{' crates/` for struct-update (`..base`) sites and JSON/serde fixtures the compiler won't flag. (#138 task 1: 19 literals where review predicted 5; `EntryMedia` is built in `bus/export.rs`, `bus/preview.rs` **and** `bus/basket.rs`.) |
| an enum's variants | every exhaustive `match`, and hand-written `matches!` lists the compiler won't flag — `UserError::is_notice`, the recording allow-list in `Bus::command` |
| a refusal, notice or message | its **text** across `crates/` — tests assert it verbatim (#77 task 1) |
| a behaviour | comments and CLAUDE.md prose asserting the old one |
| a model or list | every reader of its length/count in `app.slint` and `main.rs` (`rg '<name>\.length\|<name>\.len\(\)'`) — #123 |
| a component copied from another | a diff against the original (#132) |

Also grep `BACKLOG.md` for **"When to revisit"** triggers this work fires (#106
waited through four format bumps).

## 5. Where can each claim be tested?

Name the binary before promising a test:

- **`pundit-harness`** — the bus end to end; no window.
- **`pundit-app`'s `tests/ui`** — the real `AppWindow` and its properties, plus
  `key_action.rs`, shared by `#[path]`.
- **`main.rs`** — a `[[bin]]`; nothing reaches it. Logic with content goes in a
  lib module (`fit.rs`, `slate_pass.rs`, `slate_span.rs`, `recents.rs`) or in
  the bus, where a test can; what stays in `main.rs` is a stated risk, not a
  claimed test.

## 6. Then build

Tell the user in a line or two what the census found beyond the plan's list
and any deviation you're taking — a status line, not a wait for approval.

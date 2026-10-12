---
name: recheck-claims
description: Re-derive every citation, number, version, count and quoted claim in a document before it is handed on — a spec, plan, commit message, PR body, backlog entry, CLAUDE.md edit or report to the user. Use after drafting and before committing or sending; claims typed from memory or relayed from a review or subagent are the ones that are wrong.
argument-hint: "<file, or 'the commit message' / 'the PR body'>"
---

# Recheck claims before handing on

The most frequent failure in this project's history, and it keeps recurring
after being written down (memory `relayed-numbers`). A measurement carries its
reasoning when taken and loses it when copied; relayed, it reads as fact and
survives review.

## 1. Extract

List every checkable claim in the draft:

- `path:line` citations and file paths;
- quotes — from code, docs, a spec, the coach;
- numbers — counts, measurements, test totals, rates, sizes, **versions**;
- **quantifiers** — "all", "every", "never", "only", "both": the common failure
  is a true claim about one case widened to all of them;
- claims about behaviour — "X is refused", "nothing reads Y";
- commit hashes, PR and backlog numbers;
- **causes** — "X causes Y" is a claim too.

## 2. Re-derive, in one batch

One `grep`, `rg`, `sed -n`, `git show` or `python3 -c` per claim, run together:

- a `path:line` must still hold **what the sentence says**, not just exist;
- a count is recounted, and the match list eyeballed — a grep can double-count
  a multi-line site;
- a format version comes from `store.rs`, never from a plan or backlog entry;
- a quote is read **with the sentence after it**;
- a cause: grep `BACKLOG.md` for the symptom's text first — it may be a known
  entry;
- a subagent's or reviewer's claim is a lead, not a fact.

## 3. Fix, attribute or delete

- Wrong → fix it; say where the wrong version came from.
- Not cheaply re-derivable → attribute it ("the review measured…").
- A count nothing checks and that will go stale → **delete it**: put the number
  in a test assertion and say "every row" in prose.

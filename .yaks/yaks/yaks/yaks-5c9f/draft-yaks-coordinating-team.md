---
name: yaks-coordinating-team
description: Farm-mode companion to yaks-coordinating for a TEAM farm (`.yaks/` committed to git): per-branch farms, claim commits, shorn moves riding with the code, squash landing, provenance, PR-driven integration. Load after yaks-coordinating when `git ls-files .yaks` lists files. Repo-internal; not shipped.
---

# Coordinating a team farm (`.yaks/` is committed)

Read `yaks-coordinating` first. This file is only what changes when the farm lives in git.

## What the farm is
Every checkout carries its own committed copy of `.yaks/`, so farms are **per-branch** and
reconcile when work **lands**, not live. A worker's shave, note or ask is invisible to you
until its commit reaches your branch. Coordinate with disjoint scopes plus landing, not by
watching each other. Yak ids are fine in commit messages and in-repo references.

## Claim
Create or select the leaf yaks, `shave` each, add the assignment note (owner, scope, evidence
contract, judge), and **commit them together** on your branch (`kick off run: shave A, B, C`)
before fanning out. That makes your branch's `shaving` set honest and each yak self-describing.
Some environments let a worker see an uncommitted claim; commit anyway, so the claim is
durable and your branch tells the truth.
A human's `yaks answer` on your branch is an uncommitted edit until you commit it. Commit
answers before you cut a lane or spawn a worker, or the worker will not see them.

## A worker's finish
One commit contains the code, the yak move `shaving > shorn` and any evidence. `yaks attach`
creates a NEW directory under `.yaks/artifacts/<id>/`; it must be `git add`ed or it is lost.
Stage with paths that exist: `git add` aborts and stages NOTHING if one pathspec does not
match (a transient `shaving/` path after the move is the usual culprit). Use
`git add -A -- <code paths> ':(glob).yaks/*/<id>.md' .yaks/artifacts/<id>` and then check
`git show --stat` lists every file you expect before you rely on the commit.

## Landing
- **Squash-merge** each lane; the yak id appears in the message, so `yaks commits <id>` can
  join on it. Use a merge commit only for a lane that needs several commits. No cherry-picking
  across branches. To bring main-side updates into a live branch, `git merge main`,
  all-or-nothing.
- **Commit human drift first.** Before ANY squash checkpoint, commit or stash the human's
  uncommitted `.yaks/` edits on your branch, especially answers to files the branch moves: a
  conflicted `merge --squash` leaves a half-staged index, and the following `&& git commit`
  silently does nothing.
- **Gate re-syncs on success.** Run `reset --hard` or a branch re-sync only after you have
  checked that the commit landed and `git show --stat` contains every expected file. Never
  unconditionally.
- Run `yaks doctor`. A stalled lane left `shaving` on your branch is `regrow`n or re-run.

## Asks
A worker's `yaks ask` lives on its lane until it lands, so it does not appear in your
`yaks inbox` before then. When the code is gated green and only a subjective sign-off remains,
land the lane first, then route the human's answer to the ask that is now visible.

## Provenance
`yaks commits <id>` joins on the yak file followed across the squash plus the id in messages.
After a PR squash with an id-free message the message half breaks and the file-follow
survives: anchor on the file move.

## PR-driven landing (coordinator owns `gh`, workers never touch it)
Yak moves ride inside the PR diff, so reviewers see `.yaks/` churn; that is the tradeoff.
Yak ids may appear in commit messages but NEVER in a PR title or body or an external tracker:
put the external key in with `yaks rollup --keys`, and preflight the text with
`printf '%s' "$body" | yaks scan-ids` (non-zero on any leaked id). Prefer a private farm when
the repo also has an external tracker (two competing shared layers otherwise).
If you land through a PR-style branch and a squash, keep the original per-lane history on a
named archive branch so worker SHAs cited in messages stay reachable.

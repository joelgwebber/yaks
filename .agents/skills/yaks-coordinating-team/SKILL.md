---
name: yaks-coordinating-team
description: "Farm-mode companion to yaks-coordinating for a TEAM farm (the farm in git: on its own branch, or committed beside the code): one farm per machine on a farm branch; per-branch farms, claim commits, shorn moves riding with the code, squash landing, provenance, PR-driven integration. Load after yaks-coordinating when the farm is on a branch or `git ls-files .yaks` lists files. Opt-in skill, installed by `yaks skills install --with coordination`."
---

# Coordinating a team farm (the farm is in git)

Read `yaks-coordinating` first. This file is only what changes when the farm lives in git: on its
own branch (first section, the default), or committed on the code branches (the rest).

## The farm on its own branch (the default here)
When the farm lives on an orphan branch (`yaks`, or `git config yaks.branch`) in its own
worktree, there is **one farm per machine**, reached by discovery from every checkout (yours,
the human's, every worker clone), and `main` carries no yak files. Then the per-branch rules
below (claim commits, shorn moves riding in the worker's commit, `answer <id>@<shed>`, `inbox
--sheds`) do not apply; the private skill's live rules do (live claims and asks, one writer per
yak, workers stage nothing under `.yaks/`), plus:
- Yak ids are fine in commit messages (the farm is in git). Workers' briefs say "Finish (live
  farm" and need no `YAKS_DIR` on this machine; `yaks path` (no arguments) must print the farm
  worktree's `.yaks`: make it the worker's first check.
- **You commit the farm** with `yaks commit` (on the farm branch, in its worktree, whatever your
  cwd), at least at every landing. Code lands by `land.sh` (a squash); nothing else touches
  `.yaks/`, so landings never conflict on yak files.
- **Machines and teammates:** the farm crosses as `origin/<branch>`. Push it only on the
  human's say-so; once pushed: push before a machine switch, pull after. A clone whose host is
  Delta's bare managed repo cannot see the worktree: give its brief `--yaks-dir <farm>`.
- `yaks sheds` still lists every checkout with its git facts (branch, ahead/behind, dirty,
  merged, liveness); its farm-diff columns read `shares this farm` for every shed, by design.
  "What is worker X doing" is `yaks log --by <actor>` and `yaks inbox`.

## What the farm is
Every checkout carries its own committed copy of `.yaks/`, so farms are **per-branch** and
reconcile when work **lands**, not live. A worker's shave, note or ask is invisible to your
plain `list`/`show`/`inbox` until its commit reaches your branch; the cross-shed commands
(core section 3: `sheds`, `changes`, `inbox --sheds`, `show --sheds`) read it in its checkout
before then. Coordinate with disjoint scopes plus landing; watch with those to notice asks and
stalls, never to merge state by hand. Yak ids are fine in commit messages and in-repo references.

## Claim
Create or select the leaf yaks, `shave` each, add the assignment note (owner, scope, evidence
contract, judge), and **commit them together** on your branch (`kick off run: shave A, B, C`)
before fanning out. That makes your branch's `shaving` set honest and each yak self-describing.
Some environments let a worker see an uncommitted claim; commit anyway, so the claim is
durable and your branch tells the truth.
A human's `yaks answer` on your branch is an uncommitted edit until you commit it. Commit
answers before you cut a shed or spawn a worker: a `git worktree` shed starts from a commit and
would not see them, and even where a worker's checkout copies your working tree (Delta) the
commit is what keeps your branch's history honest.

## A worker's finish
One commit contains the code, the yak (shorn: its `status:` line changed in place in
`.yaks/<herd>/yaks/<id>.md`) and any evidence. `yaks attach` creates a NEW directory `.yaks/<herd>/yaks/<id>/`; it
must be `git add`ed or it is lost. Stage with paths that exist (`git add` aborts and stages
NOTHING if one pathspec does not match): `git add -A -- <code paths> .yaks/<herd>/yaks/<id>.md`, plus
`.yaks/<herd>/yaks/<id>/` if you attached something, then check `git show --stat` lists every file you
expect before you rely on the commit.

## Landing
- **Squash-merge** each shed; the yak id appears in the message, so `yaks commits <id>` can
  join on it. Use a merge commit only for a shed that needs several commits. No cherry-picking
  across branches. To bring main-side updates into a live branch, `git merge main`,
  all-or-nothing.
- **Commit human drift first.** Before ANY squash checkpoint, commit or stash the human's
  uncommitted `.yaks/` edits on your branch (`yaks commit` does it in one command, never touching
  code), especially answers to files the branch moves: a
  conflicted `merge --squash` leaves a half-staged index, and the following `&& git commit`
  silently does nothing.
- **Gate re-syncs on success.** Run `reset --hard` or a branch re-sync only after you have
  checked that the commit landed and `git show --stat` contains every expected file. Never
  unconditionally.
- Run `yaks doctor`. A stalled shed left `shaving` on your branch is `regrow`n or re-run.

## Asks
A worker's `yaks ask` lives in its shed's copy until it lands, so plain `yaks inbox` does not show
it; `yaks inbox --sheds` does (the shed's name and the question). Answer scope and mechanics in the
shed with `yaks answer <id>@<shed>` (core section 8): the reply stays uncommitted there and the
worker commits it with its yak file, so the yak keeps one committed history. For a design fork,
tell the human the shed name and id; they can answer the same way from their checkout (not from
a machine the thread was only shared to: there their checkout cannot see the clones). When the
code is gated green and only a subjective sign-off remains, you may instead land the shed first
and route the human's answer to the ask that is now visible.
Your OWN ask (raised in your checkout) is answered the same way: tell the human
`yaks answer <id>@<your shed>`, never plain `yaks answer <id>`, and do not push the commit holding
the open ask to their checkout before it is answered. The push copies the ask into their farm, a
plain answer lands in that copy, and yours stays `needs: human` (seen on yaks-7204; the fix then is
to record their answer in your yak with `yaks answer`, quoting it, and have them discard their
uncommitted copy before your next push).

## Provenance
`yaks commits <id>` joins on the yak file followed across the squash plus the id in messages.
After a PR squash with an id-free message the message half breaks and the file-follow
survives: anchor on the file move.

## PR-driven landing (yakherd owns `gh`, workers never touch it)
Yak moves ride inside the PR diff, so reviewers see `.yaks/` churn; that is the tradeoff.
Yak ids may appear in commit messages but NEVER in a PR title or body or an external tracker:
put the external key in with `yaks rollup --keys`, and preflight the text with
`printf '%s' "$body" | yaks scan-ids` (non-zero on any leaked id). Prefer a private farm when
the repo also has an external tracker (two competing shared layers otherwise).
If you land through a PR-style branch and a squash, keep the original per-shed history on a
named archive branch so worker SHAs cited in messages stay reachable.

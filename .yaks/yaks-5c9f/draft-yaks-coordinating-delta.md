---
name: yaks-coordinating-delta
description: Environment companion to yaks-coordinating for coordinating in Delta (parent thread as coordinator, workers via spawn_subagent): the worker brief template, what a worker sees, how its work lands and how to check it, asking and resuming a worker, landing in the human's checkout. Load after yaks-coordinating and its farm-mode skill when you can call spawn_subagent. Repo-internal; not shipped.
---

# Coordinating in Delta

Read `yaks-coordinating` and your farm-mode skill first. This file is how work MOVES in Delta.

## The model
Your thread is the coordinator. Each worker (`spawn_subagent`) is its own thread with a thin git
clone (`<repo>/.delta/clones/<id>/<repo>.git`) and a checkout (`<repo>/.delta/worktrees/<id>/<repo>`);
those finished clones stay on disk. A fresh checkout has no `target/`: building takes 15-25 s.
Extra repos are attached to the thread by the human as more Delta worktrees; with several attached,
file tools and `skill` need the worktree argument. Never clone a repo or add a path dependency.
Project skills load only from REAL files under `.agents/skills/` (symlinked skill dirs are skipped),
and `skill` needs the `worktree` argument; still put the critical rules in the brief.

## Before you spawn
- Claim in the farm (mode skill). A worker's checkout is a snapshot of your WORKING TREE,
  uncommitted changes included, so it sees an uncommitted claim; in a team farm commit it anyway.
- Read the model from the spawn confirmation and record it in the claim note. The profile default
  has changed between runs; pass `model` only when the human asked for one.
- Scope disjointly and give each worker its own yak (core sections 3-4). Parallel workers on one
  repo are fine: four landed within 10 seconds of each other.

## The brief (fill in every blank)
```
You are worker `<name>`. You own exactly ONE yak: `<id>` - "<title>". It is already `shaving`.
Do not shave, shear, regrow or edit any other yak.
Prefix EVERY yaks command with `YAKS_ACTOR=<name>` (each terminal call is a fresh shell).
First: `ls .yaks/*/<id>.md` and `date -u` (team) or `ls -d .yaks` (private); record them in your
first note. Build your own binary (`cargo build --release`, then ./target/release/yaks) or use
<absolute path>; never npx or an installer. `yaks show <id>` and read every note.
Task: <what and what not>. Scope by symbol: <files, functions>. Out of scope: everything else;
if you must touch another file, `yaks ask` instead.
Evidence (I re-run it and reject what I cannot reproduce): gate `<verify cmd>`; <test that fails
before and passes after>; <attachment>. Paste real output in notes, claim nothing unobserved.
Forbidden: edits to Cargo.toml/Cargo.lock/.gitignore/config, path dependencies, `git push`,
installs, history rewrites. <mode rule: team: stage `':(glob).yaks/*/<id>.md'` and
`.yaks/artifacts/<id>` too; private: no yak ids or the word yaks in commits, stage only your files>
Finish: shear after the gate passes; ONE commit, explicit paths, message `<id>: ...` (team) or
plain English (private). Blocked or a decision needed: `yaks ask <id>`, leave your edits in the
working tree (never revert, never park in $TMPDIR), say which in your final message, return.
Final message: commit SHA and `git show --stat`; gate command + last 15 lines; start/finish
`date -u`; docs-parity grep you ran; anything surprising.
```

## What lands, and how (never trust the working tree after a landing)
Delta merges a worker's result against what IT last applied to your tree, not against your HEAD.
Expect any of: a fast-forward of your HEAD; a PENDING MERGE (`MERGE_HEAD` set, files in the working
tree); only modified files (its commit exists as an object); a worker's unfinished edits as modified
files; conflict markers. Once a landing silently reset files you had committed after spawning.
After every worker returns, in this order:
1. `git status` (the LONG form) and `git rev-parse --verify MERGE_HEAD`. Do not commit anything
   unrelated while a merge is pending: it becomes the merge commit.
2. Take the worker's SHA from its final message. `git show <sha> --stat`.
3. `git merge-tree --write-tree HEAD <sha>` (git 2.38 or newer; check `git --version`) prints the correct
   merged tree; `git diff <tree>` shows how your working tree differs from it. On an older git, make a
   scratch worktree and run `git merge --no-commit --no-ff <sha>` there, then compare. Differences in files the worker never touched are Delta's mistake.
4. Land by SHA, one worker at a time: `git merge --abort`, `git checkout -- <that worker's files>`,
   then `git merge --no-ff <sha>`. A merge refuses to overwrite untracked files, so for each untracked
   leftover first check it equals the worker's version, `git show <sha>:<path> | cmp - <path>`, and
   remove it only if it does. Conflict markers in a file mean the same thing: do not hand-edit them;
   restore the file and merge by SHA. Never `git add -A` after a landing: Delta's applied state can mix
   files from several workers and unfinished edits, and `-A` would commit all of it as one change.
5. Re-run the gate yourself, reproduce the key scenario, review the diff, `yaks doctor` (core section 7).
   A resumed worker's second landing can leave a yak in two status dirs.
6. Do not hand-revert Delta's applied state and expect that worker's NEXT landing to merge cleanly; it
   will come back as a conflict. Use steps 1-4 again.

## Asking and resuming
A blocked worker has run `yaks ask` and returned. Answer in the yak (core section 8): in a team farm have
the worker run `yaks answer` on your stated authority, so the yak keeps one writer; in a private farm
answer directly while the worker is idle. Then wake the SAME worker with `send_agent_message` to its
agent id (it is in the spawn confirmation, and is the `sender_id` of its completion message): it resumes with its notes, findings and worktree (and may need to redo edits that were lost). The
message is the nudge; the note is the record. Ask the worker whether it saw the answer without a sync step.

## Landing in the human's checkout
Every commit in every thread is mirrored into the human's repo as `refs/delta/<thread>/<repo>/<sha>`, on no
branch, so their `main` does not move by itself and there is no agent-side accept tool. Moving it is your
explicit `git push local <branch>:main`: refused if the human's checkout is on that branch and dirty (their
yak drift counts), always possible to a branch that is not checked out. Team-mode asks and notes reach their
`yaks inbox` only once `main` moves. Default flow: form logical per-lane commits on a PR-style branch, push the
branch, then fast-forward `main`; put the yak id and the checks run in each message. Never push `origin` or open
a PR without the human's say-so; GitHub needs `origin`, not `local`. You may delete your own merged `pr/*`
branches on `local`.

## Hazards
- Private farm: every thread shares ONE live farm by walk-up (mode skill); same-yak writes lose notes.
- Attribution: notes carry the inline `YAKS_ACTOR`; commits are authored by the human; status moves have no
  actor. Delta terminals expose `DELTA_THREAD_TITLE` and `DELTA_CURRENT_THREAD_ID`.
- Each clone has its own `target/` (0.3-1.3 GB).

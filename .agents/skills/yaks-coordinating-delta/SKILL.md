---
name: yaks-coordinating-delta
description: "Environment companion to yaks-coordinating for coordinating in Delta (parent thread as yakherd, workers via spawn_subagent): the worker brief template, what a worker sees, how its work lands and how to check it, asking and resuming a worker, landing in the human's checkout. Load after yaks-coordinating and its farm-mode skill when you can call spawn_subagent. Opt-in skill, installed by `yaks skills install --with coordination`."
---

# Coordinating in Delta

Read `yaks-coordinating` and your farm-mode skill first. This file is how work MOVES in Delta.

## The model
Your thread is the yakherd. In Delta (the agent environment where every thread owns an isolated git
checkout) each worker (`spawn_subagent`) is its own thread with a thin git clone plus a checkout; those finished clones stay on disk. Where they live depends on the machine:
on the human's own machine `<repo>/.delta/clones/<id>/<repo>.git` and `<repo>/.delta/worktrees/<id>/<repo>`;
on a machine that only has the thread shared to it, `~/.local/share/delta/worktrees/<id>/<repo>` with the
git dir as its sibling `<repo>.git` and NO `.delta/`. Either way a checkout is `<root>/<id>/<repo>`, and
`git worktree list` inside it shows only that clone; `yaks lanes` lists every sibling checkout (git worktrees
and Delta clones) with its HEAD, ahead/behind, dirty count, who is working in it and what its farm changed,
read-only. Your clone's `objects/info/alternates` names the repo that holds Delta's `refs/delta/<id>/<repo>/<sha>`
pins. Delta marks the commits it imports from another machine as shallow boundaries, so a clone (yours or the
human's) can be shallow (`git rev-parse --is-shallow-repository`): ancestry questions then have unreliable
answers, `yaks lanes` shows `?` and `land.sh` prints a note; `git fetch --unshallow origin` (or `local`) fixes it. A worker can
read your checkout and your `target/` (it is not isolated), so the brief forbids touching them. A fresh checkout has no `target/`: building takes 15-25 s.
Extra repos are attached to the thread by the human as more Delta worktrees; with several attached,
file tools and `skill` need the worktree argument. Never clone a repo or add a path dependency.
Project skills load only from REAL files under `.agents/skills/` (symlinked skill dirs are skipped),
and `skill` needs the `worktree` argument; still put the critical rules in the brief.

## Before you spawn
- Claim in the farm (mode skill). A worker's checkout is a snapshot of your WORKING TREE,
  uncommitted changes included, so it sees an uncommitted claim; in a team farm commit it anyway.
- Read the model id from the spawn confirmation (for example `anthropic/claude-sonnet-5-5`) and record it AFTER
  spawning, one line (worker name, agent id, model id, the base commit) on the umbrella yak or in the
  landing note, never on the worker's own yak: the worker moves that file, and your edit then conflicts with
  its landing. Once spawned, do not edit a worker's yak at all. The profile default has changed between
  runs; pass `model` only when the human asked for one.
- Start every spawn `title` with the worker's name (`lanes-1: build yaks lanes CLI`). Delta shows the title in
  its thread list and exports it as `DELTA_THREAD_TITLE`, which yaks stamps as `delta:<title>` when no actor is
  set; with the name first the thread, the yak's notes and moves, and the label `yaks lanes` shows for the lane
  (its actors and in-progress yaks) line up by eye. Keep `YAKS_ACTOR=<name>` in the brief too: it is stable
  when a thread is renamed.
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
Finish: `yaks shorn <id>` (there is no `shear` subcommand) after the gate passes; ONE commit, explicit paths, message `<id>: ...` (team) or
plain English (private). Blocked or a decision needed: `yaks ask <id>`, leave your edits in the
working tree (never revert, never park in $TMPDIR), say which in your final message, return.
Decisions: record every choice a reviewer could argue about as a note starting `Decision:` that names
the alternatives you rejected. Write multi-line notes as markdown with real line breaks: pipe them in with `--note - <<'EOF'`, not one long argument.
Final message: commit SHA and `git show --stat`; gate command + last 15 lines; start/finish
`date -u`; the `Decision:` notes in one line each; docs-parity grep you ran; anything surprising.
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
4. `sh .agents/skills/yaks-coordinating-delta/land.sh <sha> --dry-run` reports steps 1-3 (and what in your tree the
   commit does not explain); without `--dry-run` it does step 4 and checks the result equals the computed tree.
   It refuses while your tree holds changes the commit does not explain (`--force-discard` overrides) and never
   pushes. By hand, land by SHA, one worker at a time: `git merge --abort`, `git checkout -- <that worker's files>`,
   then `git merge --no-ff <sha>`. A merge refuses to overwrite untracked files, so for each untracked
   leftover first check it equals the worker's version, `git show <sha>:<path> | cmp - <path>`, and
   remove it only if it does. Conflict markers in a file mean the same thing: do not hand-edit them;
   restore the file and merge by SHA. Never `git add -A` after a landing: Delta's applied state can mix
   files from several workers and unfinished edits, and `-A` would commit all of it as one change.
5. Re-run the gate yourself, reproduce the key scenario, review the diff, `yaks doctor` (core section 7).
   A resumed worker's second landing can leave a yak in two status dirs.
6. Do not hand-revert Delta's applied state and expect that worker's NEXT landing to merge cleanly; it
   will come back as a conflict. Use steps 1-4 again.
7. **A parked or long-running worker pins your history.** Its commit is based on some commit of yours. If that
   base is no longer an ancestor of your branch (you squashed, then `git reset --hard local/main`, or rebased),
   `git merge <sha>` finds a very old merge-base and reports add/add conflicts, and Delta's applied tree is the
   worker's old snapshot: it reverted every newer file in your working tree and resurrected deleted yaks. Test it
   with `git merge-base --is-ancestor $(git log --format=%p -1 <sha>) HEAD`. If it fails, land with
   `git cherry-pick <sha>` (only the worker's own change) and resolve its conflicts by hand. Better: while any
   worker is running or parked do not rewrite your branch; after a squash landing use `git merge local/main`,
   not `git reset --hard`.

## Asking and resuming
A blocked worker has run `yaks ask` and returned. Answer in the yak (core section 8): in a team farm have
the worker run `yaks answer` on your stated authority, so the yak keeps one writer; in a private farm
answer directly while the worker is idle. Then wake the SAME worker with `send_agent_message` to its
agent id (it is in the spawn confirmation, and is the `sender_id` of its completion message): it resumes with its notes, findings and worktree (and may need to redo edits that were lost). The answered yak is `needs: agent`: tell the worker to run `yaks pickup <id>` when it resumes. The
message is the nudge; the note is the record. Ask the worker whether it saw the answer without a sync step.

## Landing in the human's checkout
Every commit in every thread is mirrored into the human's repo as `refs/delta/<thread>/<repo>/<sha>`, on no
branch, so their `main` does not move by itself and there is no agent-side accept tool. Moving it is your
explicit `git push local <branch>:main`: refused if the human's checkout is on that branch and dirty (their
yak drift counts), always possible to a branch that is not checked out.
Team-mode asks and notes reach their
`yaks inbox` only once `main` moves. Default flow: form logical per-lane commits on a PR-style branch, push the
branch, then fast-forward `main` (and do not reset your own branch to it while workers are parked: step 7); put the yak id and the checks run in each message. Never push `origin` or open
a PR without the human's say-so; GitHub needs `origin`, not `local`. You may delete your own merged `pr/*`
branches on `local`.

**Before any push to `main`, check that `local` is the human's checkout.** On a thread shared to another machine `local` is a
Delta-managed BARE repo with no branches (workers' clones always are); a push there is accepted and lands where
the human never looks. Run `yaks preflight --push-main`: it exits non-zero, and prints what to do instead, when `local`
is missing, bare, has no `main`, is not a path on this machine, or is a checkout on `main` with uncommitted changes
(yak edits count; the push would be refused). By hand that is `git ls-remote --heads local main` printing a line and
`git -C "$(git remote get-url local)" rev-parse --is-bare-repository` saying `false`. If it fails, never push `main`:
push `pr/<name>` to `local`, and give the human the command `git fetch <managed repo path> pr/<name>` (or ask them to
repoint `local`).

## One landing point, and when your clone goes stale
Land from ONE place: the yakherd's thread. If another thread (a human-created subthread, a second
yakherd) or the human pushed to `main` while you worked, your clone is behind and `git push` is
rejected or would drop their work. `git fetch local`, then `git merge local/main` (or rebase your
unpushed branch), re-run the gate, and only then push. Do not `git reset --hard local/main` unless
`git status` is clean and `git log local/main..main` shows nothing you still need: the reflog is the
only net. The human may also push `origin`; Delta refreshes your `origin/main` for you.

## Hazards
- Private farm: every thread shares ONE live farm, but a Delta checkout is its own git top-level
  (`<repo>/.delta/worktrees/<id>/<repo>`), so discovery does not walk out to the user's farm: put
  `YAKS_DIR=<abs path to the farm>` in the worker brief and on every command (mode skill). Without
  it the worker gets an error naming the git top-level, not the live farm. Same-yak writes lose notes.
- Attribution: notes carry the inline `YAKS_ACTOR`; commits are authored by the human; status moves carry
  the actor too (a `moved: <from> -> <to>` entry, same `--as`/`YAKS_ACTOR`). Delta terminals expose `DELTA_THREAD_TITLE` and `DELTA_CURRENT_THREAD_ID`.
- Each clone has its own `target/` (0.3-1.3 GB).

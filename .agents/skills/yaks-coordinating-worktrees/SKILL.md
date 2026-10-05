---
name: yaks-coordinating-worktrees
description: "Environment companion to yaks-coordinating for coordinating with plain `git worktree` checkouts (one worktree per lane): creating and removing worktrees, the file-tool pitfalls of some agent harnesses, serial arcs, a human-driven lane, crash recovery. Load after yaks-coordinating and its farm-mode skill when your workers run in `git worktree` checkouts. Opt-in skill, installed by `yaks skills install --with coordination`."
---

# Coordinating with git worktrees

Read `yaks-coordinating` and your farm-mode skill first. This file is how work MOVES when each
lane is a `git worktree` of one repository.

## Lanes
- Cut a lane from your freshest state: `git worktree add wt/<name> -b <branch>`. Put it in a
  **non-gitignored in-project directory** (`wt/`, not `.worktrees/`): some harnesses' file tools
  consult `git check-ignore` and refuse ignored paths, and neither `.gitignore` nor
  `.git/info/exclude` fixes that. The cost is `?? wt/` in `git status`; always stage explicit
  paths and never `git add .`. `git worktree remove` it after the run.
- Team farm: the lane has its own committed copy of `.yaks/`, so cut it AFTER the claim commit
  and after committing any human answers. Private farm: discovery stops at the git top-level, and a
  worktree is its own top-level (`.git` file), so even an in-tree `wt/<name>` does NOT find the
  one live farm by walk-up: give each lane `YAKS_DIR=<abs path to the farm>` in its commands, or a
  `.yaks` pointer file / symlink in the worktree.
- Each lane builds and tests its OWN binary (`./target/release/yaks` in the lane), never the main
  checkout's.

To see all lanes at once (branch, ahead/behind, dirty files, who is working there, what each lane's farm
changed) run `yaks lanes` from any checkout; it is read-only.

## File-tool rules (harnesses whose file tools root at the main checkout)
1. Edit through the EXPLICIT worktree path (`wt/<name>/src/foo.rs`), never a bare `src/foo.rs`:
   a bare path silently edits the main tree.
2. After each edit, `git -C <main> status --short` must show no stray source edits in main (a
   `?? wt/` line is expected).
3. If a harness still refuses the path, edit through the terminal with the lane as cwd, replacing a
   unique multi-line anchor string (never a line number) so a wrong match fails loudly.
A harness that addresses files by worktree id (Delta) does not have this problem.

## Landing from a worktree
Land from the main checkout, one lane at a time: squash-merge by default (team: id in the
message; private: plain English, then stamp the SHA on the yak). Verify the lane is disjoint from
the others first, and run the checks in `yaks-coordinating` section 7. A lane with several
commits that matters as history uses a merge commit instead.

## Serial arc (one long-lived lane, phases in order)
For a large refactor whose phases edit the same file and cannot run in parallel: cut ONE lane,
keep it for every phase, and remove it at the end. Drive each phase yourself for delicate,
behaviour-preserving work, or spawn a worker per phase for bulk mechanical moves (it then obeys
the file-tool rules above). **Checkpoint** every completed phase to main (squash, id in the
message), then `git merge main` back into the lane. Do not hoard the arc on a branch.
Human questions are simpler here: you work at the lane, so asks and answers land on the branch
and merge with the work.

## A human-driven lane (parallel, no fan-out)
A design problem the human and one agent iterate on is a peer lane the human drives. The human
opens `wt/<name>` as the harness root (an agent cannot re-root itself), and the yak is deferred
until there is an artifact to make. Team farm: land the farm-only commit early so you can fan the
emitted yaks out while the code work continues. Private farm: the yaks are visible at once; only
claim what you hold. When the human says "merge up": if no yakherd is running they land it;
otherwise you do, from a committed branch.

## Recovery
A worker's work lives in its worktree and branch, so a lost session is not lost work. Look first:
`git -C wt/<name> status` and `git -C wt/<name> diff`, build and test that worktree's own binary,
then commit and shear it yourself or discard and re-run.

## Pitfalls
- A `git add <list>` with one path that does not exist stages NOTHING; check `git show --stat`.
- A squash checkpoint over uncommitted human edits to files the branch moved leaves a half-staged
  index; commit human drift first, and re-sync the lane only after the checkpoint demonstrably landed.
- Never `git clean -fdx` in the outer repo with a private farm (it deletes the ignored `.yaks/`).

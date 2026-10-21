# Worker brief template (Delta, team-mode farm) — draft for yaks-2f9b

Status: DRAFT, first used in trial 1 (yaks-fe33). Fill every <placeholder>. Each
section exists because a first-run failure needed it (log entry in parentheses).
Paste the filled brief as the `spawn_subagent` task.

## 1. Who you are and which yak (O4, O5)
You are worker `<name>`. You own exactly one yak: `<yak-id>` — "<title>".
It is already in `shaving`. Do not shave, shear, regrow or edit any other yak.
Prefix EVERY yaks command with `YAKS_ACTOR=<name>` (each terminal call is a fresh
shell; an `export` does not carry over). Notes without it are attributed to the
human.

## 2. First actions (O3, fe33 measurements)
1. Before building anything run `ls .yaks/*/<yak-id>.md` and `date -u`; record in
   your first note which status directory the yak is in and the time.
2. Build your own binary: `cargo build --release` in your yaks worktree, then use
   `./target/release/yaks`. Never use `npx`, a `yaks` on PATH, or any install.
   Note whether `target/` existed and how long the build took.
3. `yaks show <yak-id>` and read every note (a human may have left feedback).

## 3. Task and scope (O5)
<what, in the yak's own terms, plus the behaviour wanted and not wanted>
Scope (by file and symbol, not line number): <files / fn names>
Out of scope: everything else. If you believe you must touch another file, stop
and `yaks ask` instead.

## 4. Evidence contract and judge (O6)
Gate (a script): `<verify command>`. Also required: <test that fails before and
passes after / an attached frame or transcript / ...>.
Judge: the coordinator re-runs the gate in its own checkout. Anything it cannot
reproduce is rejected, so paste real command output, never a summary.
Attach evidence with `yaks attach <yak-id> <file>`; it creates a new directory
under `.yaks/artifacts/` that must be `git add`ed (untracked dirs are not
replicated, O9).

## 5. Forbidden (O5, 5fe9 #3)
- No edits to Cargo.toml / Cargo.lock / .gitignore / config of any kind, and no
  path dependencies. If a dependency must change, `yaks ask`.
- No `git push`, no network installs, no deleting or rewriting history.
- No claims in notes you did not observe.

## 6. Finish (O2, O6)
Shear when the gate passes: `YAKS_ACTOR=<name> ./target/release/yaks shorn <yak-id>`
after a short note (what changed, what you learned, the evidence). The
coordinator may regrow it.
Make ONE commit containing the code, the shorn yak move and the artifacts. Stage
explicit paths only: `git add -A -- <code paths> ':(glob).yaks/*/<yak-id>.md'
.yaks/artifacts/<yak-id>` (the glob form works whether or not the claim was ever
committed, trial 1B). Message: `<yak-id>: <summary>`.
Your commit reaches the coordinator automatically when you finish.
If you hit a decision only a human can make, or a scope boundary: `yaks ask
<yak-id> --note "..."` and return. Do NOT revert your work and do NOT park it in
$TMPDIR (it is deleted when your session ends, O17). Leave the edits in the
working tree, or make one clearly named `WIP:` commit; uncommitted edits reach
the coordinator too. Say which in your final message. The coordinator may answer
and message you; you keep your context.

## 7. Final message format (O6)
- Commit SHA and `git show --stat` output.
- The gate command and the last 15 lines of its output.
- Start/finish `date -u`, whether `target/` pre-existed, build time.
- Files changed; anything not done; anything surprising.
- Docs parity: the grep you ran for the old behaviour across `README.md docs
  skills src/main.rs` and what it showed (O25).

## 8. Coordinator checklist after a worker returns (not part of the brief)
Learned in trial 1 (log O2, O22, O25):
1. `git status` (LONG form) and `git rev-parse --verify MERGE_HEAD`: a worker
   commit landing on a diverged parent is a pending merge. Do not commit
   anything unrelated first (it becomes the merge commit); do not cherry-pick.
2. `git log --graph --oneline` and `git diff --cached <worker-sha> --stat`
   (staged tree vs the worker's commit, expecting only your own side's files).
3. Re-run the gate yourself and reproduce the key scenario with your own build.
   Cannot reproduce = reject.
4. Read the diff for quality (duplication, missed docs); make your changes as
   follow-up commits so the worker's commit stays intact.
5. `yaks doctor`. A resumed worker's second landing can leave a yak in two
   status dirs (stage the deleted side too).
6. If the worker asked: answer scope/mechanics yourself (record that you did),
   route design forks to the human, then `send_agent_message` the worker.
7. Right after any landing (before any `git add`), compute the correct result:
   `git merge-tree --write-tree HEAD <worker-sha>` and compare it with the
   working tree (`git diff <tree>`); Delta's applied state has silently reverted
   newer parent changes once (O31) and wrote conflict markers once (O33).
8. Land workers ONE AT A TIME by SHA (from each final message): `git merge
   --abort`, `git checkout -- <that worker's files>`, `git merge --no-ff <sha>`.
   Parallel workers land in different shapes (fast-forward, pending merge, plain
   file edits, WIP files; O30), so never `git add -A` after a landing. Do not
   hand-revert an applied landing and then expect the same worker's next landing
   to merge cleanly (O33): use the by-SHA procedure for that one too.

## 9. Private-farm variant (Delta x private, trial 3)
Used for a repo whose `.yaks/` is gitignored (the checkout has no `.yaks/`; every
`yaks` command resolves the one LIVE farm by walk-up, O27):
- Claim LIVE before spawning; there is nothing to commit. The brief gives the
  absolute path of a built binary (the checkout has no `target/`), `YAKS_ACTOR=`
  on every command, and the farm location for reference only.
- No yak ids and no `yaks` wording in commit messages, code or comments; stage
  only the one source file, never `git add -A`; never edit yak files by hand.
- Evidence attaches to the shared farm (no `git add` rule applies); verify with a
  stored `verify:` command.
- Ask / answer is live: the coordinator may answer directly in the farm while the
  worker is idle (single writer at a time), then `send_agent_message` to wake it;
  the note is the record, the message is the nudge (O34).
- Same-yak contention loses notes (yaks-800d): keep one writer per yak.

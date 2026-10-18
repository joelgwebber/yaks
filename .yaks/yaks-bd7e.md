---
id: yaks-bd7e
title: 'yaks migrate --to-branch: move an in-tree committed farm onto an orphan yaks branch (the yaks-87a3 cutover as one command)'
status: shorn
type: feature
priority: 1
created: '2026-10-18T21:39:13Z'
updated: '2026-10-18T22:57:24Z'
parent: yaks-73f4
labels:
- cli
- git
---

What yaks-87a3 did by hand for this repo, as a command for every other team repo: `yaks migrate --to-branch [--branch yaks] [--worktree <path>] [--dry-run]`, run from a checkout whose committed `.yaks/` is the farm. Steps: refuse unless the tree is clean and no merge is pending; create the branch from HEAD in a new worktree (default `../<repo>-yaks`, later the XDG home, yaks-0ef9); in it, one commit that keeps only `.yaks/` (history of every past farm change kept, so `yaks commits` and `git log --follow` work); in the code checkout, one commit removing `.yaks/` and the leftover dir; print what to push (`git push -u origin yaks`, then main) and the one-line setup for other machines/teammates (`git worktree add <path> yaks`). Never pushes. Idempotent: a repo already on a branch says so. Evidence: a temp repo with a committed farm and history -> after: main has no .yaks/, the branch tree is only .yaks/, `yaks path` from main/subdir/a --shared clone resolves the worktree, `yaks commits <old id>` finds its history; dry run changes nothing.

---
▸ 2026-10-18T22:32:58Z [delta-farm]

moved: hairy -> shaving

---
▸ 2026-10-18T22:32:58Z [delta-farm]

Claim (delta-farm, yakherd). Owner: `migrate-1`. Includes the worktree-location slice of yaks-0ef9 (Joel: get farms where they belong before migrating more repos).

Goal: `yaks migrate --to-branch [--branch <name>] [--worktree <path>] [--dry-run]` turns a repo whose farm is a committed in-tree `.yaks/` into a farm on an orphan-tree branch, in one command, never pushing. It extends the existing `Migrate` command in src/main.rs (today: schema migration); the new flag selects this mode.

Decided (not yours to fork):
- **Default worktree location** = `store::farm_home(repo)`: `$XDG_DATA_HOME/yaks/farms/<repo-dir-name>-<8 hex>` (else `$HOME/.local/share/yaks/farms/...`, on every platform, matching how src/tui/views_store.rs already picks `$XDG_CONFIG_HOME`/`~/.config`; Windows `%LOCALAPPDATA%\yaks\farms` when HOME is unset). The hex is a stable hash of the canonical host git dir (`sheds::host`), like views_store's `slug`. `--worktree` overrides. One function; a later `yaks init --mode branch` (yaks-b1e9) reuses it.
- **Branch** = `--branch`, else `git config yaks.branch`, else `yaks`. When it is not `yaks`, set `git config yaks.branch <name>` in the repo so discovery finds it.
- **History kept**: the branch starts at HEAD; its first commit removes everything but the top-level `.yaks/`, so `git log <branch> -- .yaks/<id>.md` and `yaks commits <id>` still reach old commits. The farm worktree must end up holding ONLY `.git` and `.yaks/`: a NESTED `.yaks/` elsewhere in the tree (this repo has `tests/fixtures/farm/.yaks`) or any other path must not survive on disk.
- **The code side**: one commit on the current branch removing `.yaks/` (message `yaks: the farm moves to branch <b>`), then remove the leftover directory (ignored `.lock`/`.gitignore`).
- **Refuse** (exit 1, naming why, nothing changed): uncommitted changes anywhere, a merge/cherry-pick pending, a farm that is not tracked (private), the farm already on a branch (say so, exit 0), the branch or the worktree path already existing.
- **Print** at the end: where the farm now is, `git -C <worktree> push -u origin <branch>` and the main push to do, and the setup line for another machine or teammate (`git worktree add <path> <branch>`). Never push.
- `--dry-run` prints the plan and changes nothing.

Scope: new module (e.g. src/branchfarm.rs, or beside src/migrate.rs), `store::farm_home`, the `Migrate` command's args + dispatch in src/main.rs, docs: docs/cli.md (`migrate` row and the farm-as-a-branch rule's setup lines), README (the farm-on-its-own-branch paragraph), the yaks skill (that section + command table row), clap help. Out of scope: TUI, sheds, the rest of yaks-0ef9, pushing.

Evidence (fail first): Rust tests with real temp repos: a committed farm with several commits of history + a nested `x/.yaks/` -> after: main has no `.yaks/`, the branch tree is only `.yaks/`, the worktree holds only `.git` + `.yaks/`, `yaks path` (discovery) from main, a subdir and a `--shared` clone resolves to the worktree, the old commits of a yak are reachable on the branch; each refusal; dry run changes nothing (HEAD, branches, worktrees, files identical); `farm_home` honours XDG_DATA_HOME. A transcript of the real binary migrating a scratch COPY of a real repo, attached. Gate `cargo test --workspace`. Judge: delta-farm, then Joel on one of his repos.

---
▸ 2026-10-18T22:33:34Z [migrate-1]

migrate-1 start: Sun Oct 18 22:33:29 UTC 2026; yaks path yaks-bd7e = /home/joel/src/yaks-farm/.yaks/yaks-bd7e.md

---
▸ 2026-10-18T22:51:59Z [migrate-1]

Tests written in `src/branchfarm.rs` (real temp repos, data dir passed in, no `set_var`). Seen failing first: with `branchfarm::run`/`render` and `store::farm_home_with` stubbed out, `cargo test --release branchfarm` gave

```
test branchfarm::tests::another_branch_name_is_recorded_for_discovery ... FAILED
test branchfarm::tests::moves_the_farm_keeping_history ... FAILED
test branchfarm::tests::farm_home_honours_xdg_data_home_then_home ... FAILED
test branchfarm::tests::default_worktree_is_the_farm_home ... FAILED
test branchfarm::tests::dry_run_changes_nothing ... FAILED
test branchfarm::tests::refusals_change_nothing ... FAILED
test branchfarm::tests::a_shared_clone_is_sent_to_its_host ... FAILED
test result: FAILED. 0 passed; 7 failed
```

and with the implementation restored: `test result: ok. 7 passed; 0 failed`.

- `moves_the_farm_keeping_history`: farm with 2 commits + nested `x/.yaks/` + on-disk `.lock` -> main has no `.yaks/` (tracked or on disk), nested fixture stays on main, branch tree only `.yaks/`, worktree holds exactly `.git`+`.yaks`, `git log --follow` on the branch reaches both old commits, discovery from main / `src/` / a `--shared` clone resolves the worktree, a re-run (from main and from the worktree) says already-on-a-branch.
- `refusals_change_nothing`: modified file, untracked file, existing branch, existing worktree path, pending merge, private farm; HEAD/refs/worktrees/files/local config identical after each.
- `dry_run_changes_nothing`, `default_worktree_is_the_farm_home`, `farm_home_honours_xdg_data_home_then_home`, `another_branch_name_is_recorded_for_discovery`, `a_shared_clone_is_sent_to_its_host`.

[migrate-to-branch-transcript](yaks-bd7e/migrate-to-branch-transcript.txt)

---
▸ 2026-10-18T22:54:04Z [migrate-1]

Transcript attached (`migrate-to-branch-transcript.txt`): `./target/release/yaks migrate --to-branch` on `/tmp/mig-t/surveyor`, a scratch `git clone` of `/home/joel/src/fs/subtext/surveyor` (190 tracked farm files; the original was only read). `XDG_DATA_HOME=/tmp/mig-t/data` so the real data dir was not touched. What it shows:

- dry run: the plan printed, `git status` empty, data dir never created.
- the real run: `main` 6be5f8d `yaks: the farm moves to branch yaks` on top of 8d95c1b; `yaks` 5143f0d `yaks: branch yaks keeps only the farm (.yaks/)` on top of the same 8d95c1b.
- `ls -A` of the worktree gives `.git .yaks`; the worktree tracks 0 files outside `.yaks/`; `git status --ignored` is clean in both checkouts, so no leftover `.yaks/` or `.lock` on main.
- `yaks path` from the clone root, from `docs/` and from a `--shared` clone (`-C shared`): all `/tmp/mig-t/data/yaks/farms/surveyor-d707600d/.yaks`.
- `yaks commits survey-e0c0` lists 12 commits that touched the file, all older than the migration.
- re-run, and re-run with `--branch other`: "already on the branch `yaks` ... nothing to do", exit 0.
- on a fresh clone with an untracked file: refused, exit 1, no branch made, data dir not created.

Found while making the transcript, and fixed (a test now covers it): on an already-migrated repo, `--branch other` said "no committed .yaks/ farm" and exited 1. It now checks the configured branch too and says the farm is already on a branch.

---
▸ 2026-10-18T22:54:04Z [migrate-1]

Decision: the worktree is created with `git worktree add --no-checkout -b <b> <path> HEAD`, then `read-tree HEAD` + `git rm -r --cached ':(top)' ':(top,exclude).yaks'` + commit + `checkout HEAD -- .`. Nothing outside `.yaks/` is ever written to the worktree, so a nested `x/.yaks/` cannot survive there (the yaks-87a3 surprise).

- Rejected: a normal checkout followed by deleting the extra files. That writes the whole tree to disk and has to clean it up.
- Rejected: `git worktree add --orphan`. It loses the farm's history.

---
▸ 2026-10-18T22:54:04Z [migrate-1]

Decision: from a Delta clone or any `--shared` clone (`sheds::host(..).via_alternates`), the command refuses and names the host's main checkout.

- Why: a worktree added from a clone is registered in the clone's own repository, so the human's other checkouts' `git worktree list` would not show it, and discovery from them would fail.
- Rejected: creating the branch and worktree in the host repository from inside the clone. It acts on another checkout's repository without being asked to.

---
▸ 2026-10-18T22:54:04Z [migrate-1]

Decision: `store::farm_home` keys on the host's canonical git dir. The `<repo>` part is the directory name of the host's main checkout, falling back to the bare git dir's name (with `.git` stripped). The hash is FNV-1a, the same hash views_store's `slug` uses, cut to 8 hex. Tests call `store::farm_home_with(repo, env)`, which takes the environment as a closure, and `branchfarm::run` takes a `home: FnOnce(&Path) -> PathBuf`. So no test reads the real XDG dirs or calls `set_var`.

- Rejected: hashing the checkout path. Each Delta clone or worktree would get a different farm home.

---
▸ 2026-10-18T22:54:04Z [migrate-1]

Decision: more refusals than the claim lists: a detached HEAD; `--branch` equal to the current branch; an invalid branch name; `origin/<b>` already existing (the message says to add that branch's worktree instead); a `.yaks` pointer file or symlink. Untracked files count as uncommitted changes, because the removal commit and the deletion of the leftover directory must not race with stray files.

`git config yaks.branch` is set only when the chosen branch differs from what discovery already resolves. So the default `yaks` writes no config.

Both commits use `--no-verify`. They are mechanical moves, and a repo's pre-commit hook (lint, test gates) could otherwise abort the run halfway. If the farm side fails, it is undone (worktree removed, branch deleted). If the code side fails, the error names the finishing command.

---
▸ 2026-10-18T22:54:32Z [migrate-1]

verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-18T22:54:43Z [migrate-1]

Summary (migrate-1). `yaks migrate --to-branch [--branch <b>] [--worktree <p>] [--dry-run]` is in.

- `src/branchfarm.rs` (new): `run`, `render`, 7 tests.
- `store::farm_home` / `farm_home_with` (new). `store::only_leftovers` is now `pub(crate)`.
- `Migrate` gains the args `to_branch`, `branch` and `worktree`, plus a dispatch that runs before any farm is opened. The schema-migrate path is unchanged.
- Docs: docs/cli.md (a new `migrate --to-branch` row, and the set-up block in the farm-as-a-branch rule), README (farm-on-its-own-branch paragraph), yaks skill (setup bullet + command-table row), clap help.

Reuses `sheds::host`, `sheds::host_checkout`, `sheds::worktree_of_branch`, `sheds::canonical`, `store::configured_farm_branch`, `store::farm_branch`, `store::expand_path` and `changes::pending_operation`.

Evidence:

- 7 tests that failed against stubs, then passed.
- Attached transcript on a scratch clone of surveyor.
- `yaks verify` PASS.

Not done: no run on any real repo; the coordinator or Joel judges on one.

---
▸ 2026-10-18T22:54:43Z [migrate-1]

moved: shaving -> shorn

---
▸ 2026-10-18T22:57:24Z [delta-farm]

Landed (delta-farm): worker commit 56068fc squashed onto main as 8cecd3e by land.sh (base an ancestor, tree identical to the computed merge, doctor clear). Full `cargo test --release` 874 passed. Read the design: worktree created `--no-checkout` and filled from the keep-only-.yaks commit (so nested `.yaks/` dirs never land), every refusal before the first change, farm side undone on failure, `--no-verify` on both commits (accepted: a hook must not stop the move halfway). Accepted the extra refusals (detached HEAD, origin/<b> exists, pointer/symlink farm, a Delta/--shared clone pointed at its main checkout).

My own run, release binary, a fresh scratch clone of ~/src/canon (186 farm files), temp XDG_DATA_HOME: dry run printed the plan and changed nothing (no branch, no worktree, data dir empty); the real run created `yaks` 9f2a775 (only .yaks/) and `main` a34ef54 (removes .yaks/), worktree at `<data>/yaks/farms/repo-6de69669` holding only `.git .yaks`, main clean with no `.yaks/`; `yaks path` from the root and from crates/canon-api resolves the worktree; `yaks commits canon-0109` 8 lines before and after; `yaks list` 174 rows; a re-run says "already on the branch".

Then moved THIS repo's farm to the new default with one `git worktree move ~/src/yaks-farm ~/.local/share/yaks/farms/yaks-539d3edb` (the farm_home of /home/joel/src/yaks): discovery from ~/src/yaks, a subdir, this Delta clone and the new worktree all follow it; `migrate --to-branch` there says "already on the branch". AGENTS.md and the yaks skill updated to say `yaks path` instead of naming ~/src/yaks-farm.

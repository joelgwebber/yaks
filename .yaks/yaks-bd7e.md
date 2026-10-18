---
id: yaks-bd7e
title: 'yaks migrate --to-branch: move an in-tree committed farm onto an orphan yaks branch (the yaks-87a3 cutover as one command)'
status: shaving
type: feature
priority: 1
created: '2026-10-18T21:39:13Z'
updated: '2026-10-18T22:32:58Z'
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

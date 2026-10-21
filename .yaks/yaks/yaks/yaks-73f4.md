---
id: yaks-73f4
title: 'The farm is a branch: an orphan yaks branch replaces committed .yaks/ on main (shared, personal or local)'
status: shaving
type: feature
priority: 2
created: '2026-10-18T15:54:02Z'
updated: '2026-10-18T21:15:00Z'
parent: yaks-6a34
labels:
- git
- design
---

Joel (2026-10-18), from the live-farm trial (yaks-6a34): with the farm on its own branch, option (b) is a real mode: the farm lives ONLY on a perpetual branch (`yaks`/`farm`) pushed to origin, never merged into main. Backed up, shareable with teammates who opt in (fetch the branch, add the worktree), invisible to everyone else: no `.yaks/` on main, no yak churn in PRs or `git log`. Wanted for some team repos.

Design questions:
- Discovery with no `.yaks/` on main: `live_branch:` lives in a committed config today. Quiet mode needs a non-committed signal: the presence of a worktree on the branch, a per-user registry (see the client-metadata yak), or `git config yaks.liveBranch` in the repo.
- Setup for a teammate: `yaks init --mode quiet` = fetch `origin/<branch>`, worktree it (sparse), done.
- Ids in commit messages: allowed (teammates resolve them from the branch) or kept out like private mode? Probably a per-repo choice.
- Provenance: `yaks commits <id>` joins on ids in code commits only; the farm's history is on its own branch.
- Relation to private mode (untracked) and to the nested-repo pattern, which quiet mode largely supersedes.

---
▸ 2026-10-18T19:35:20Z [delta-farm]

Rescoped (Joel + delta-farm, 2026-10-18): "quiet mode" becomes THE model, subsuming public mode and (nearly) private. The farm is an ORPHAN branch (`yaks`, or `yaks/<team>` for isolated farms in a monorepo) holding only `.yaks/`, checked out as its own worktree (one live farm per machine, as in yaks-6a34). Nothing about yaks is ever committed to `main`. Shared = pushed to origin; personal = pushed elsewhere (`branch.yaks.pushRemote`); local/private = never pushed, guarded by a pre-push hook. Joel: every farm lives in SOME repo, even a never-pushed `git init`, so every operation has one shape.

Why: no yak commits on main to explain to a team; locked main + PR workflows need nothing special; trusted contributors with Write role can herd without main access (GitHub permissions are per repo, so not for outsiders); isolated farms per team via named branches.

Experiment (scratch repos, git 2.56), all observed:
- `git worktree add --orphan -b yaks <dir>`: the farm worktree holds only `.yaks/`; main has no `.yaks/`; histories unrelated.
- Discovery: a fresh `git clone` already has `origin/yaks` (no extra fetch); `git worktree add <dir> yaks` tracks it. A `clone --shared` (Delta-style) sees the human's farm worktree through its host's `git worktree list`, exactly like `live_branch` today.
- Concurrency: two clones writing DIFFERENT yaks: push rejected, `pull --rebase`, 0 conflicts. Same yak: 1 conflict by default. With `.yaks/*.md merge=union` committed ON the branch, 0 conflicts and every note kept, BUT the frontmatter ends up with two `updated:` lines: union makes merges succeed into an INVALID file. So a frontmatter-aware merge driver is required, not optional (revives yaks-74f5's design; filed as a child).
- Private guard: a pre-push hook keyed on `git config yaks.private` refused both `git push --all` from the code checkout and an explicit push from the farm worktree (hooks are shared by worktrees); origin's `yaks` unchanged. (The hook must read its stdin refs; a no-op if the branch has no new commits.)

Costs to accept/solve: team push races (yaks sync + the driver); yak state no longer versioned with each main commit (old tags do not show it; code->yak relative links break, yak->code ids still work); every clone and CI needs the farm worktree (auto-create on first use under the XDG home, yaks-0ef9); ids-in-commit-messages becomes a per-farm setting.

Breaks today already (found while checking): source links `[x](../src/foo.rs)` resolve against `farm.root().parent()` (src/tui/source.rs `open_source`), which under the live farm is the sparse farm worktree, so they no longer open. Filed.

Plan: design note + migration for this repo (move `.yaks/` history onto an orphan branch, drop `.yaks/` from main in one commit), retiring the main-tracking parts of yaks-2355 (`sync_live`, `--land`) since an orphan farm never shares history with main. One migration covering yaks-b32e's layout decision too, if any.

---
▸ 2026-10-18T20:02:13Z [delta-farm]

Design for the build (delta-farm, before code):

**Discovery** (one added step; YAKS_DIR and the walk unchanged): when the walk reaches a git top-level with no `.yaks` entry, look up the farm BRANCH: name from the host repo's `git config yaks.branch`, default `yaks`; its worktree from the host's `git worktree list` (the same lookup `live_branch` uses, so Delta clones work). Found: that worktree's `.yaks/`. Branch exists (local or `origin/`) but no worktree: an error naming the two commands to create it. Neither: today's error. A `.yaks/` whose config has `live_branch:` stays a stale copy and redirects through the same lookup (covers old checkouts of pre-migration commits).

**One helper**, `store::farm_branch(root)`: the farm branch a farm root is checked out on, when that root is a dedicated farm worktree (its HEAD is the configured farm branch, or its config names `live_branch:`). `brief` (Mode::Live) and source links (`source_base`) key on it instead of the config line.

**Retire** (they assume the farm shares history with main and must be merged into it): `commit::sync_live`, `yaks commit --land`. With the farm off main, `yaks commit` in the farm worktree is the whole story; landing code is `land.sh` (squash) and nothing else touches `.yaks`.

**This repo's migration** (after round 2 lands, no worker in flight): commit the farm; in `~/src/yaks-farm` create branch `yaks` from `farm` and commit `git rm -r --cached` of everything but `.yaks/` (so the branch's tree is only the farm, while its history still contains every past `.yaks/` commit: `git log --follow` and `yaks commits` keep working); drop `live_branch:` from its config; `sparse-checkout disable`; on main, one commit removing `.yaks/`. Fresh farms elsewhere use `git worktree add --orphan`. Pushing `yaks` to origin waits for Joel.

**Deferred to children/siblings**: `yaks init --mode branch` + XDG default path (yaks-b1e9, yaks-0ef9), `yaks sync` + merge driver (yaks-4a53), the private pre-push hook, named farms per team via a pointer `branch:` key, and yaks-b32e's layout (decided before migrating if it changes paths).

---
▸ 2026-10-18T21:15:00Z [delta-farm]

moved: hairy -> shaving

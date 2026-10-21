---
id: yaks-472e
title: 'Farm branch sync lifecycle: commit, pull and push the farm (yaks sync), design first'
status: hairy
type: feature
priority: 2
created: '2026-10-21T19:54:29Z'
updated: '2026-10-21T19:54:35Z'
parent: yaks-6a34
labels:
- git
- cli
- design
---

Joel (2026-10-21): with the farm always on its own branch and worktree, `yaks commit` commits there but nothing moves the branch between machines or teammates; today that means `git -C "$(dirname "$(yaks path)")" push` / `pull` by hand. Pieces of this were scattered: `yaks sync` is a one-line wrapper at the end of yaks-4a53 (merge driver), the first-publish scope note and the auto-commit question (from yaks-78ff) are notes there, and "is the farm pushed / behind" is half of yaks-b1e9's `yaks farm status`. This yak owns the lifecycle as a whole; yaks-4a53 stays the merge driver it depends on for same-yak concurrency.

## Today
- `yaks commit`: commits the farm on its branch in the farm worktree, from any cwd. Never pushes.
- First clone: the farm worktree is bootstrapped from `origin/<branch>`, tracking it (yaks-a260).
- `yaks migrate --to-branch`: prints the `push -u origin <branch>` line for the first publish.
- Nothing pulls, nothing pushes, nothing tells you the farm is ahead or behind.

## Design questions (to discuss before building)
1. **Command shape.** One `yaks sync` (commit what is dirty, fetch, rebase/merge, push, retry on a race)? Or thin `yaks push` / `yaks pull`, or `commit --push`? Lean: one `sync`, plus `--no-push` / `--no-commit` if needed.
2. **Pull timing.** Explicit only, at session start (skills), on TUI start, or a cheap background fetch with a "behind" badge? Network on every command is out.
3. **Auto-commit.** Should yaks commit by itself (timer, TUI exit, every mutation, or only inside `sync`)? The farm worktree is the only writer of `.yaks`, so it cannot collide with code commits. Commit granularity vs. log noise.
4. **Merge strategy.** Rebase vs. merge on the farm branch; concurrent edits to DIFFERENT yaks merge cleanly already; the SAME yak needs the frontmatter-aware driver (yaks-4a53). Ship `sync` before the driver (and stop with a clear message on conflict), or only after?
5. **First publish and remotes.** `push -u origin <branch>` when origin has none (yaks-4a53 scope note); which remote (`branch.<b>.pushRemote`, a personal remote for a personal farm).
6. **Private farms.** A never-pushed farm: `sync` refuses or no-ops, and the pre-push guard keyed on `git config yaks.private` from the yaks-73f4 experiment keeps a plain `git push --all` from leaking it.
7. **Where it runs.** Like `commit`: in the farm worktree whatever the cwd, so from a Delta clone too (the host's worktree). A clone whose host is Delta's bare repo has no farm worktree: refuse with the reason.
8. **Visibility.** `yaks farm status` (yaks-b1e9) and/or the TUI status bar: uncommitted count, ahead/behind `origin/<branch>`, last sync time. A TUI key to sync?
9. **Skills.** The yakherd's routine (sync at session start and after landings; before a machine switch) goes into the skills rewrite (yaks-b048).

Evidence when built: two clones of a scratch repo with a farm branch; edits on both (different yaks, then the same yak); `sync` on each converges with no hand git; a private farm refuses to push.

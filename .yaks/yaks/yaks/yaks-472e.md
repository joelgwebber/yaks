---
id: yaks-472e
title: 'Farm branch sync lifecycle: commit, pull and push the farm (yaks sync), design first'
status: shaving
type: feature
priority: 2
created: '2026-10-21T19:54:29Z'
updated: '2026-10-21T20:34:01Z'
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

---
▸ 2026-10-21T20:12:06Z [Joel Webber]

Design discussion:
1. I dig `sync` with options for the half-sync special cases.
2. Agreed. We can offer skill guidance to ensure it happens at the right moments.
3. Inclined to auto-sync, but open to convincing. If not, do we need to start sprouting tui affordances to show dirty status, so you remember that you need to? Also, what if there are manual edits on the filesystem? Should probably grab those too -- otherwise things get weird quickly.
4. I'm assuming rebase should be fine, given that we have a strategy for (mostly?) conflict-free merges anyway. We can always predicate `sync` on the merge driver. No reason to add extra failure modes we could avoid.
5. I'm inclined to just create the remote automatically. We want the local/remote branches to match, or things can get confusing. So for personal farms, I think we should require something at farm creation time to rename it (though we could also add a branch-rename affordance later?).
6. That makes sense. It sounds like the `yaks.private` trick should avoid the answer to (5) mucking things up.
7. Makes sense.
8. This comes back to (3) -- if we *do* auto-commit, when do we do it? If we *don't*, you need affordances for managing a dirty herd from the CLI/TUI. The CLI _does_ already have status/commit/sync, so it's not a huge leap to say "we need the same in the TUI". Worth thinking about -- see below.
9. Yes, that seems unavoidable. We'll need to pin down the auto-sync behaviors first.

On the auto-commit flow (or lack thereof), we need to decide on the tradeoff between "automatic" and "extra state for the user/ui to manage". I was initially inclined towards "automatic", but I see a strong argument for making it explicit, so that the human and agent both have the option of futzing with local changes to yaks, comments, etc, without being forced to publish them unexpectedly. Let's go with that direction, and just accept the extra complexity, unless there's a strong counterargument.

---
▸ 2026-10-21T20:29:39Z [delta:herd-dirs]

Decision (Joel 2026-10-21 on-yak answers, recorded by delta:herd-dirs):

- **Command:** one `yaks sync` = commit what is dirty, fetch, rebase onto the remote farm branch, push. Half-syncs by flag: `--no-commit`, `--no-push` (pull only), `--no-pull` (commit + push).
- **Pull timing:** explicit only; the skills say when (session start, after landings, before a machine switch).
- **No auto-commit.** Changes stay local until `yaks commit` / `yaks sync`, so a human or agent can futz locally without publishing. Manual edits to farm files are swept in by commit (it already stages everything under the farm). Cost accepted: dirty-state affordances in the TUI (indicator + commit/sync keys) and in `yaks status`.
- **Merge:** rebase, and `sync` is built ON the merge driver (yaks-4a53), never without it.
- **Remote branch:** created automatically on first sync (`push -u`), same name as the local branch. A personal farm picks its branch name at creation (rename affordance later).
- **Private:** `git config yaks.private true` makes sync refuse to push (commit + pull only, if a remote exists at all); a pre-push guard keeps a manual push from leaking it.
- **Where:** the farm worktree, whatever the cwd (as `commit`).

Plan (baby yaks; 1-3 in parallel on cheaper workers, coordinator reviews and lands):
1. yaks-4a53 merge driver.
2. yaks status: farm-branch ahead/behind (+ yaks-d4a7 preflight fix).
3. yaks sync.
4. TUI: dirty/ahead indicator + commit/sync key (after 2 and 3).
5. Private-farm pre-push guard.
Skills guidance goes into yaks-b048.

---
▸ 2026-10-21T20:33:01Z [delta:herd-dirs]

moved: hairy -> shaving

---
▸ 2026-10-21T20:34:01Z [delta:herd-dirs]

Round 1 spawned (delta:herd-dirs, 2026-10-21), base 1e95cd3 (mergedriver stub prep commit), model anthropic/claude-sonnet-5-5 for all three:
- `driver-1` -> yaks-4a53 (merge driver), thread ksQQfIIp_WA2ToGZ4UwiEcPWKpLOACZFrpIBxBCA0aLJyrhJPreUvVwLftHG
- `status-1` -> yaks-bd1d + yaks-d4a7 (status sync state, preflight on a branch farm), thread ksQQfIIp_WA2ToGZ4UwiEcPWKpLOACZG4JIBxBCA0aLJyrhJPreUvVwLftHG
- `private-1` -> yaks-85bb (private guard), thread ksQQfIIp_WA2ToGZ4UwiEcPWKpLOACZHFZIBxBCA0aLJyrhJPreUvVwLftHG

Next, after landing: yaks-e3b2 (sync; needs 4a53 + 85bb's is_private), then yaks-ce5d (TUI; needs bd1d + e3b2).

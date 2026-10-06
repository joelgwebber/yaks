---
id: yaks-5f12
title: 'Decision: git-store vs Model 3 (coordinator-sole-writer on FS)'
type: task
priority: 1
created: '2026-08-31T20:47:23Z'
updated: '2026-10-06T03:58:55Z'
parent: yaks-4fe6
labels:
- git
---

Model 3 (pure FS): files authoritative, coordinator is sole herd writer on main; zero new engine; no merge/concurrency (single writer); provenance via file+commit (works); human-editable files; but limited worker autonomy, no live cross-worktree sharing, PR-cleanliness needs the coordinator to land herd separately. Git-store: refs authoritative + materialized cache; big new engine (ops, clocks, compile, sync); mergeless + live-shared-across-worktrees + PR-clean + intrinsic attribution + multi-writer-safe + isolation-compatible; but ref-sync friction, platform-invisibility, provenance rework, ongoing maintenance (git-bug itself is barely maintained, a signal of cost/thin adoption). Lean: Model 3 now (tiny effort, max reliability for small careful parallelism); git-store as the north-star architecture if multi-agent-native coordination justifies the engine, pursued via the incremental path so the file experience survives as the cache.

---
▸ 2026-09-01T23:31:59Z
Refined architectural judgment (path-independent). Retracting the 'git-store as north star' framing. Divorced from what we have built: FILE-BASED is the better TARGET for yaks' core identity (tasks as plain greppable/diffable/hand-editable files; status as directory; single simple binary). git-store trades that differentiator away for mergeless multi-writer concurrency, which careful parallelism does not need (disjoint assignment / single-writer removes contention with zero engine). Even WITH a materialized cache, git-store still loses platform visibility (GitHub/PR/git log), trivial hand-editing, and file-level git history. git-store wins ONLY in the private + distributed + offline + multi-writer/multi-machine quadrant (git-bug's niche). Not a general north star; a different product that wins in one quadrant.

---
▸ 2026-10-05T22:43:55Z [delta-lead]
Thoughts for future consideration (coordinator delta-lead, 2026-10-05), prompted by Joel's question about how we handle git commits in linear and parallel work (yaks-0a87, yaks-75cc, now shorn). No decision asked; a trigger to revisit is at the end.

## The pattern behind every incident
The farm is mutable shared state that lives in the same working tree, index and commit stream as the code, and it has three kinds of writer at once: the human (live, in the TUI or by hand), agents (concurrently), and git operations (merge, squash, reset, Delta's applied state). Git assumes one writer at a time on a tree.

Incidents we logged: a human's uncommitted answer vs a squash (0a87), a worker moving a yak file while the coordinator edited it (yaks-df61 O52), a human's uncommitted yaks blocking a landing (O41), Delta's applied tree reverting files (O31, O46), a missing pathspec silently staging nothing (0a87).

## What we do today (in-tree farm plus discipline and tools)
- Commit only what you own: `yaks commit` (farm only), explicit pathspecs.
- Land by SHA, never by working tree: `land.sh` (refuses unexplained changes, computes the merge, checks the result).
- Do not edit a yak a worker owns while it runs; record spawn facts on the umbrella yak.
- `yaks preflight` before committing or pushing.

Residual risk: two lanes (or a lane and the human) both append to the SAME yak; text appended at the end of a file conflicts. yaks-7149 (a move now appends an entry) widens this a little.

## Options if the residual risk starts to bite
1. Keep the farm in the tree and keep tightening (cheapest, where we are).
2. Conflict-free by construction: notes and moves as separate append-only files, so two lanes never touch the same file (a merge is a union). Costs the one-file-per-yak readability, which is a stated invariant.
3. Farm on its own ref and worktree (this yak's "git-store" family): code operations never touch it. Costs the provenance that comes free when the yak move rides in the code commit (yaks commits <id>), and PR-visible yak moves.
4. Farm outside git entirely: that is private mode today; loses PR visibility.

## Recommendation
Do not build 2 or 3 now. The incidents above all have shipped mitigations, and the last six landings produced no hand-resolved yak-file conflict. Revisit when ANY of these happens: a landing needs a hand-resolved conflict inside `.yaks/`, a yak edit is lost, or two lanes need to write the same yak routinely. Count them in the observation log (yaks-df61) so the trigger is evidence, not a feeling.

---
▸ 2026-10-06T03:58:55Z [delta-lead]
Trigger check (coordinator delta-lead, 2026-10-06): one hand-resolved conflict inside .yaks/ has now happened, an append/append on yaks-2c22 (the coordinator's ask and the human's answer appended to the same file) when merging the human's commit. It resolved by keeping both. One incident is not the pattern; the count stays in yaks-df61.

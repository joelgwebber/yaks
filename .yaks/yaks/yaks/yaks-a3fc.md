---
id: yaks-a3fc
title: 'Design: show a shed''s commits so far (pushed branches plus a little yak data)'
status: shaving
type: idea
priority: 2
created: '2026-10-31T15:18:16Z'
updated: '2026-10-31T16:54:08Z'
parent: yaks-039b
depends_on:
- yaks-bdf9
labels:
- git
- sheds
- design
---

Goal (Joel, 2026-10-31): from one machine, the TUI/CLI can show "this shed's commits so far" for work on any machine, on the assumption that we are willing to fetch from origin. Not the fine-grained live view Delta worktrees half-provide; only what has been pushed.

## Minimum data

- **Where the code is:** a remote and branch for the yak (or for the actor's current work), e.g. `origin` + `yaks/tex-1/mem-7aba`.
- **How far it got:** the head sha at the last checkpoint, so the view can say "the farm says abc123; origin has it / doesn't yet".
- **Base:** the branch it forked from (usually `main`), for `git log base..branch`.

## Conventions it needs

- Workers push their branch to a remote everyone can read. This changes current policy ("never push origin without the human's say-so") and needs a decision on the namespace (`yaks/<actor>/<id>`?), the remote, and cleanup after landing.
- Order: push the code first, then record the sha and `yaks sync`, so the farm never names a sha nobody can fetch.
- A helper, if the conventions keep being missed: `yaks checkpoint <id>` = push the branch, write a checkpoint note (sha, branch), sync.

## The affordance

- Sheds row or yak detail: "3 commits on `origin/yaks/tex-1/mem-7aba`, last abc123 (2h ago)", expandable to `git log --oneline`.
- Fetch only on demand (a key, `--fetch`), never automatically; say when the local copy is stale.
- After landing, show the landed sha instead (commit hashes design yak).

## Open questions

- A branch per yak, or per actor (one worker branch spanning several yaks)?
- Where it lives: frontmatter (`branch:`, beside the PRs field of `yaks-d695`) for current state, notes for checkpoint events?
- Delta: worker commits live in clones. Does pushing from a Delta worker clone work cleanly? The memcraft coordinator had to push from a separate full clone because of a stale shallow marker.

---
▸ 2026-10-31T15:56:38Z [delta:Cloud Runners]

Decision (Joel, 2026-10-31): workers push their branches to `origin`.

- **Naming:** we give guidance (e.g. `yaks/<actor>/<topic>`), but a harness's own pattern is fine; Indent and others name worker branches themselves. What matters is that the yak records the connection.
- **Granularity:** one branch per worker, not per yak. A worker often spans a parent and its children, so the view shows a worker's branch against each yak it records it on.
- **Commits in comments:** workers note their branch and the commit sha at each checkpoint, so the branch can be found from the yak. Until note metadata exists (`yaks-6a11`), this is a plain-text convention; afterwards it is a `checkpoint` note with `branch`/`commit` keys.
- **Tooling only when needed:** no `yaks checkpoint` helper yet. First run trials with workers on cheaper models following the convention by hand (`yaks-039b` child trial yak), and build the helper if they miss the push-then-record-then-sync order.

Rejected: a separate remote for work in progress; one branch per yak.

---
▸ 2026-10-31T16:54:08Z [yakherd]

moved: hairy -> shaving

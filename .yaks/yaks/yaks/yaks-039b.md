---
id: yaks-039b
title: Sheds from notes, remote work visibility, and structured comments
status: hairy
type: feature
priority: 2
created: '2026-10-31T15:16:48Z'
updated: '2026-10-31T15:16:48Z'
labels:
- sheds
- design
---

Umbrella for a redesign that started from a Delta cloud-runner trial (memcraft, 2026-10-29): the runner's worktrees never appeared as sheds on Joel's laptop, and yak changes reached the laptop only when the runner ran `yaks sync`.

## Findings that motivate it

- Sheds are machine-local by construction: discovery is git on this machine (`refs/delta` pins, alternates, `git worktree list`) and attribution is the per-machine `.activity/` log, which is never committed. A runner's checkouts are invisible from the laptop.
- The laptop did list one unnamed Delta clone (`xf7x0fg96q1h`): Delta's local mirror of the runner coordinator's worktree. Unnamed because `yaks.shed.title` lives in the runner clone's git config; no yaks because `.activity/` is local.
- The farm branch is the only cross-machine channel, and it already carries who/what/when: every note and move is stamped (`[delta:Set up remote yak workflow]`, `[tex-1]`).
- The runner was wiped twice; the farm worktree outside the Delta worktree went with it. Unsynced yak writes on a runner are lost, so sync is durability, not just visibility.
- The `delta` CLI (0.23) adds little: `thread tree --json` (names, parents), `worktree list --thread` (empty `directories` for runner threads), `thread mount` (side-effecting). No supported running/idle state or local-dir-to-thread map. The runner had no `delta` CLI at all.

## Direction (decided with Joel, 2026-10-31)

Drop git/Delta discovery from sheds; build them from note attribution. Keep the name `sheds` for now. Remove `yaks discover` and the activity log. Then explore, as design yaks, what a little explicit yak data would buy: remote commit visibility via origin, commit hashes in yaks (no longer circular with a branch farm), and structured comment metadata.

## Related existing yaks

- `yaks-d695` (pull-request field in frontmatter): the same "where does this yak's code live" question as the commit-visibility design here.
- Likely obsoleted by the rebuild: `yaks-6beb` (managed-repo sheds), `yaks-9b9f` (flaky sheds mtime test), `yaks-2177` if still open. `yaks-ded9` (thread naming) becomes an actor-name question.
- `yaks-f1e6` (sheds view like list/tree) and `yaks-19c3` (activity log view) fit the note-based model directly.
- `yaks-2610` decided against storing hashes because it was circular with the farm on `main`; that premise is gone.

---
id: yaks-039b
title: Sheds from notes, remote work visibility, and structured comments
status: shaving
type: feature
priority: 2
created: '2026-10-31T15:16:48Z'
updated: '2026-10-31T17:24:35Z'
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

---
▸ 2026-10-31T16:08:35Z [yakherd]

moved: hairy -> shaving

---
▸ 2026-10-31T16:19:03Z [yakherd]

Fan-out 2026-10-31 (yakherd: the "Cloud Runners" Delta thread). Base `7f78a28` (the prep commit: fence-aware note boundaries plus the note metadata core; `yaks-4cd7` shorn).

| Worker | Yaks | Model | Thread |
|---|---|---|---|
| `meta-1` | `yaks-b8cd` | LiteLLM `claude-sonnet-5-5` | `ksQQtfBVdv4DR2eA_txRWzEtg5LOADk87JIBxBCA0aLJyrhJPreUvVwLftHG` |
| `sheds-1` | `yaks-da23` then `yaks-d3ad` | LiteLLM `claude-sonnet-5-5` | `ksQQtfBVdv4DR2eA_txRWzEtg5LOADlCv5IBxBCA0aLJyrhJPreUvVwLftHG` |
| `commits-1` | `yaks-7a11` then `yaks-735a` | LiteLLM `claude-sonnet-5-5` | `ksQQtfBVdv4DR2eA_txRWzEtg5LOADlICZIBxBCA0aLJyrhJPreUvVwLftHG` |

The yakherd keeps the skills (`yaks-0071`, then `yaks-4c2a` after the metadata surface), landing, and review. Wave 2 after these land: `yaks-a3fc` (commits so far from pushed branches) and `yaks-d695` (PR field).

---
▸ 2026-10-31T16:55:04Z [yakherd]

Wave 1 landed on the yakherd's `main`:

- `7f78a28` prep (`yaks-4cd7`, `yaks-6a11` core)
- `71ca61e` `meta-1` (`yaks-b8cd`)
- `c4bde33`, `7e4a014` `commits-1` (`yaks-7a11`, `yaks-735a`)
- `7393e45`, `b2e4638` `sheds-1` (`yaks-da23`, `yaks-d3ad`)
- skills: `c94b182`, `45bfc8b`, `575f347`

The full gate is green after each landing. `yaks land` recorded its own landing note from `7393e45` on.

Wave 2, on LiteLLM `claude-sonnet-5-5` unless noted:

| Worker | Yak | Thread |
|---|---|---|
| `branches-1` | `yaks-4728` (under `yaks-a3fc`) | `ksQQtfBVdv4DR2eA_txRWzEtg5LOADmwaZIBxBCA0aLJyrhJPreUvVwLftHG` |
| `prs-1` | `yaks-d695` | `ksQQtfBVdv4DR2eA_txRWzEtg5LOADmzgZIBxBCA0aLJyrhJPreUvVwLftHG` |
| scout (LiteLLM `claude-haiku-5-5`) | cold read of the four changed skills, for `yaks-0071` | `ksQQtfBVdv4DR2eA_txRWzEtg5LOADm2eZIBxBCA0aLJyrhJPreUvVwLftHG` |

Trial observations for `yaks-7d57` (wave 1, all local, same machine):

- All three workers set `YAKS_ACTOR` correctly: one stable name each.
- Notes came at milestones, not every 30 minutes; `commits-1` said so itself. The runs were 20–30 minutes long, so the cadence rule was barely tested.
- Sync was not in play (same machine; the yakherd syncs).
- The yakherd itself wrote its first notes as `delta:Cloud Runners` before setting `YAKS_ACTOR`, which is the split the rule exists for.

---
▸ 2026-10-31T17:24:35Z [yakherd]

## Status 2026-10-31: everything built, rebased onto the human's `main`, not yet pushed there

The human's checkout had moved to `c2b640f` (`yaks-a99f` branch farms only, `yaks-e532` one skills install). The family's 17 commits are rebased on top of it; the pre-rebase tip is kept as branch `backup/pre-rebase-039b` in the yakherd's clone.

Resolving the rebase took:

- the new `sheds.rs`, with the farm-discovery helpers moved to `gitrepo.rs` and upstream's callers rewritten to use them;
- the new integration tests moved onto upstream's branch-farm fixture (`tests/common`);
- the skills changes re-applied by hand on upstream's rewritten skills (`931d4e4`).

Each yak has a fresh `kind: landed` note with its rebased sha. `cargo test --workspace` is green, `yaks doctor` is clean, and `yaks preflight --push-main` says ok. The push is a fast-forward of the human's `main`.

| Rebased | Was | Yaks |
|---|---|---|
| `1dfb324` | `7f78a28` | `yaks-4cd7`, `yaks-6a11` |
| `5bb0b68` | `c94b182` | `yaks-0071` |
| `920c436` | `71ca61e` | `yaks-b8cd` |
| `ceb6d57` | `45bfc8b` | `yaks-4c2a`, `yaks-0071` |
| `ffe8a66` | `c4bde33` | `yaks-7a11` |
| `b5abcb7` | `7e4a014` | `yaks-735a` |
| `0882751` | `7393e45` | `yaks-da23` |
| `ec29bc2` | `b2e4638` | `yaks-d3ad` |
| `9e37afc`, `c1f9b74` | `575f347`, `276c873` | `yaks-0071` |
| `d391058`, `7c74904` | `601fa07`, `6cad621` | `yaks-d695` |
| `8f1d145`, `a7bc953`, `365ab47` | `2a09f9e`, `4dda0fa`, `028a8e3` | `yaks-4728` |
| `3ca87ad`, `931d4e4` | (new) | rebase test fixes; skills re-applied |

Open in the family: `yaks-7d57` (a cloud-runner trial), `yaks-8003` and `yaks-ded9` (stable names), `yaks-59bf` (commits on an empty farm), and the TUI ideas `yaks-f1e6` and `yaks-19c3`. Outside it: `yaks-0cfb` (docshots drift).

One more data point for `yaks-8003`: `yaks land` run without `YAKS_ACTOR` stamped its landed notes `delta:Cloud Runners` (the thread title), not `yakherd`.

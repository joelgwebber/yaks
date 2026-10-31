---
id: yaks-7a11
title: yaks commits on a branch farm searches the farm branch and misses the code commits
status: shaving
type: bug
priority: 2
created: '2026-10-31T15:18:27Z'
updated: '2026-10-31T16:18:59Z'
parent: yaks-039b
labels:
- git
- cli
---

`Farm::commits` in `src/farm.rs` runs `git log --grep=<id>` and `git log --follow -- <yak file>` in the farm root. On a branch farm that is the farm branch's own worktree, so both halves search farm history only.

## Repro (this repo)

```sh
yaks commits yaks-e5fb              # four "yaks: ..." farm commits
git log --oneline --grep=yaks-e5fb main
# e99754d yaks-e5fb, yaks-3bcd: yaks land <sha> ...   <- missing from yaks commits
```

## Fix shape

- `--grep` over the code repository (the checkout yaks was run from, all branches or `HEAD`), not the farm.
- The `--follow` half is the yak's own edit history on the farm branch: keep it, but label it as farm history.
- Later, recorded shas come first (see `yaks-bdf9`).

Docs: `docs/cli.md` and the `--help` text describe it as "commits naming its id and those that touched its file".

---
▸ 2026-10-31T16:17:15Z [yakherd]

moved: hairy -> shaving

---
▸ 2026-10-31T16:17:16Z [yakherd]

Claim (yakherd). Owner: worker `commits-1`, then `yaks-735a` in the same worker. Owns `Farm::commits` and `render_commits`, `src/land.rs`, and the `Commits`/`Land` commands in `src/main.rs`. Evidence: a test with a branch farm plus a code repo where the code commit names the id; real-farm `yaks commits yaks-e5fb` shows `e99754d`. Judge: yakherd.

---
▸ 2026-10-31T16:18:59Z [commits-1]

Start (commits-1).

- `date -u`: Sat Oct 31 16:18:58 UTC 2026
- `yaks path yaks-7a11`: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-7a11.md

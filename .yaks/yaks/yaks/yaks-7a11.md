---
id: yaks-7a11
title: yaks commits on a branch farm searches the farm branch and misses the code commits
status: shorn
type: bug
priority: 2
created: '2026-10-31T15:18:27Z'
updated: '2026-10-31T16:52:20Z'
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

---
▸ 2026-10-31T16:30:47Z [commits-1]

Progress (commits-1): fix and test written, gate running next.

- `Farm::commits(id, cwd)` now greps the code repository (git top-level of the cwd, so `-C` works) over branches, remotes and tags except the farm branch, plus `HEAD`. Farm history (`--follow`) is kept and labelled.
- Outside a code checkout (not in git, or cwd is the farm worktree) it says so and lists nothing.
- New `tests/commits.rs` builds a temp repo with a branch farm. It failed first with `--all` (the farm commit leaked into the code list because the farm worktree's `HEAD` is added by `--all`), which is why the refs are named explicitly.

Decision: name refs (`--branches --remotes --tags HEAD` with `--exclude=<farm branch>`) instead of `--all` or `--all --exclude`. `--all` always adds every worktree's HEAD, including the farm worktree's, so the farm commits leaked (observed in the test).

Decision: the farm branch is excluded from the message grep and its commits are shown only in the farm-history section. Rejected: listing them in both. Pre-branch-farm farm commits that live on `main` (e.g. `c6d77e5` for `yaks-e5fb`) still appear in the code list, which is correct: they are reachable from code branches.

Decision: `--grep` uses `--fixed-strings` so ids with regex characters match literally. Not `-i`: ids are lowercase.

---
▸ 2026-10-31T16:34:42Z [commits-1]

verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-31T16:34:48Z [commits-1]

Summary (commits-1): `yaks commits` now greps the code repository, not the farm worktree.

- `Farm::commits(id, cwd)`: message grep in the git top-level of cwd (respects `-C`) over branches, remotes, tags and `HEAD`, minus the farm branch, newest first, `--fixed-strings`. Outside a code checkout it says so. `--follow` half kept, labelled "Farm history".
- `Commits` now holds `CommitRef { sha, short, subject }`, `code_repo`, `by_message`, `by_file`.
- Docs: `docs/cli.md` (table row + new `commits` section), `README.md`, clap `--help`.
- Test: `tests/commits.rs` (branch farm in a temp repo; commits on `main` and a side branch are listed, farm commits are not in the code list, subdir and `-C` work, running from the farm worktree says "not run from a code checkout"). It failed with `--all` before the fix.
- Real farm: `./target/debug/yaks commits yaks-e5fb` from this checkout lists `e99754d yaks-e5fb, yaks-3bcd: yaks land <sha> (port of land.sh) ...` and `c6d77e5` (an old farm commit on `main`), then "Farm history" with `f115c18 ... 412010e ...`.
- Gate: `cargo test --workspace` all green (761 unit + integration); `yaks verify` PASS.

---
▸ 2026-10-31T16:34:49Z [commits-1]

moved: shaving -> shorn

---
▸ 2026-10-31T16:52:20Z [yakherd]
kind: landed
commit: c4bde339f2ad7cda9531ca9d8c1d3cee8a13655b
branch: main
---

landed: c4bde33 yaks-7a11: yaks commits greps the code repository (all branches), labels farm history (recorded by hand: landed with a yaks binary from before `yaks-735a`)

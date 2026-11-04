---
id: yaks-bcd4
title: TUI reload spends ~130 ms in git to refresh the farm chip (10 spawns per reload)
status: shorn
type: bug
priority: 3
created: '2026-11-04T21:35:10Z'
updated: '2026-11-04T23:14:42Z'
parent: yaks-907a
tags:
- tui
- perf
---

Found while pinning down yaks-907a. Every TUI reload (`reload`, `reload_preserving_selection`) calls `refresh_farm_sync`, which runs `FarmSync::read`: `changes::read` (`ls-files` + `status`) plus `status::sync_state`. Timed with a `git` shim on a clone of this farm (880 yaks), release build:

- `refresh_farm_sync`: 126-138 ms per reload
- `h.list` (the yak files): 13-15 ms
- `reload_doks`: 0 ms; `term.draw`: 0-4 ms

So the redraw is not the cost: the git calls behind the chip are. One reload is ~10 git spawns, each 11-30 ms here, several of them repeats:

- `symbolic-ref --short -q HEAD`, `rev-parse --show-toplevel`, `rev-parse --git-common-dir`, `rev-parse --is-bare-repository`, `config --get yaks.branch` twice: all from `store::farm_branch`, called once by `changes::read` and again by `sync_state`
- `rev-parse --abbrev-ref yaks@{upstream}` and `rev-list --left-right --count`: the ahead/behind
- `ls-files -- .` and `status --porcelain`: the dirty count

Lever: one `git status --porcelain=v2 --branch -z --untracked-files=all -- .` returns the branch, its upstream and ahead/behind (`# branch.head`, `# branch.upstream`, `# branch.ab`) and the changes, in one spawn. Cache the farm's branch/toplevel for the process (they cannot change under a running TUI), or at least once per reload.

Checks: a headless timing of a no-commit save (`YAKS_NO_AUTOCOMMIT=1`) before and after; it is ~173 ms now against ~45 ms for a plain directory with no git.

---
▸ 2026-11-04T23:03:25Z [perf-907a]
kind: move
from: hairy
to: shaving
thread: ksQQstI5T-eCRACxAjKJdNo3oJLOAEQG4JIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-04T23:14:41Z [perf-907a]

Landed on local `main` as the commit after dd61f317 (not pushed). `FarmSync::read` (the TUI chip, run on every reload) now uses `changes::read_with_sync`: one `git status --porcelain -z --branch --ahead-behind` gives the dirty files and the branch standing, so a reload is 1 git spawn where it was 4 (the two `status`-ish calls plus `rev-parse @{upstream}` and `rev-list --count`); the discovery calls were already cached by yaks-7058.

Parity: `the_one_call_standing_matches_sync_state` compares it with `status::sync_state` in each state (never published, up to date, ahead, ahead and behind, dirty, upstream deleted on the remote); `a_branch_header_is_read_as_a_standing` covers the header forms (`[ahead 1, behind 12]`, `[gone]`, `No commits yet on x`, a detached HEAD, a branch with a slash).

`yaks status`, `sync` and `sheds` still call `sync_state` (the same answer, a few more calls): they are per-command, not per-keystroke.

Measured, TUI `:wq` save on a clone of this farm, first output: ~312 ms (700-760 ms before yaks-7058). The tail of the git log still shows two reloads back to back: that is yaks-1ebf.

---
▸ 2026-11-04T23:14:42Z [perf-907a]
kind: move
from: shaving
to: shorn
thread: ksQQstI5T-eCRACxAjKJdNo3oJLOAEQG4JIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn

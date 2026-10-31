---
id: yaks-d3ad
title: 'Remove git/Delta shed discovery: yaks discover, the activity log, yaks.shed.title'
status: hairy
type: task
priority: 2
created: '2026-10-31T15:18:14Z'
updated: '2026-10-31T15:18:16Z'
parent: yaks-039b
depends_on:
- yaks-da23
labels:
- sheds
- cli
---

Once sheds read notes, delete the machinery that existed only for checkout discovery (Joel, 2026-10-31: "chuck it").

- `yaks discover` (command, docs, `scripts/discover-fixture.sh`).
- `src/activity.rs` and its call in `store.rs` (`activity::record`), plus `.activity` in the farm's ignore handling.
- `sheds::record_thread` and the `yaks.shed.title` / `yaks.shed.thread` git config writes.
- The git-discovery half of `src/sheds.rs` (pins, `inspect`, `git_state`, ahead/behind, merged).

Keep the git helpers that farm discovery uses (`host`, `host_checkout`, `canonical`, `worktree_of_branch`, `common_dir`; callers in `store.rs`, `branchfarm.rs`, `init.rs`, `privatefarm.rs`, `tui/source.rs`). Move them to a small `git` module rather than leaving them in `sheds.rs`.

Then close the yaks this obsoletes (`yaks-6beb`, `yaks-9b9f`; check `yaks-2177`), with a note pointing here. Grep docs, skills, README and `--help` for `discover`, `activity` and `shed.title`.

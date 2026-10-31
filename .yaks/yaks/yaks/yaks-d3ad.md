---
id: yaks-d3ad
title: 'Remove git/Delta shed discovery: yaks discover, the activity log, yaks.shed.title'
status: shorn
type: task
priority: 2
created: '2026-10-31T15:18:14Z'
updated: '2026-10-31T16:50:11Z'
parent: yaks-039b
depends_on:
- yaks-da23
labels:
- sheds
- cli
verify: GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.fsmonitor GIT_CONFIG_VALUE_0=false cargo test --workspace
---

Once sheds read notes, delete the machinery that existed only for checkout discovery (Joel, 2026-10-31: "chuck it").

- `yaks discover` (command, docs, `scripts/discover-fixture.sh`).
- `src/activity.rs` and its call in `store.rs` (`activity::record`), plus `.activity` in the farm's ignore handling.
- `sheds::record_thread` and the `yaks.shed.title` / `yaks.shed.thread` git config writes.
- The git-discovery half of `src/sheds.rs` (pins, `inspect`, `git_state`, ahead/behind, merged).

Keep the git helpers that farm discovery uses (`host`, `host_checkout`, `canonical`, `worktree_of_branch`, `common_dir`; callers in `store.rs`, `branchfarm.rs`, `init.rs`, `privatefarm.rs`, `tui/source.rs`). Move them to a small `git` module rather than leaving them in `sheds.rs`.

Then close the yaks this obsoletes (`yaks-6beb`, `yaks-9b9f`; check `yaks-2177`), with a note pointing here. Grep docs, skills, README and `--help` for `discover`, `activity` and `shed.title`.

---
▸ 2026-10-31T16:17:15Z [yakherd]

moved: hairy -> shaving

---
▸ 2026-10-31T16:17:15Z [yakherd]

Claim (yakherd): done by `sheds-1` after `yaks-da23`, in the same worker.

---
▸ 2026-10-31T16:46:56Z [sheds-1]

Decision: `src/gitrepo.rs` is the small farm-discovery module, pruned to what discovery needs.

- Kept: `Host { git_dir, via_alternates, bare }`, `host`, `host_checkout`, `canonical`, `worktree_of_branch`, `common_dir`, `alternates`, plus the private `git`/`git_raw`/`worktree_entries` they use. Callers moved: `store`, `branchfarm`, `init`, `privatefarm`, `tui/source`.
- `host` now makes one probe less (`common_dir` instead of `--absolute-git-dir --git-common-dir`); the dropped `Host` fields (`own_git_dir`, `store`) had no caller outside discovery of sheds. The result for `git_dir`/`via_alternates`/`bare` is unchanged.
- The old module's tests were about sheds/pins; replaced by five focused tests (own host, Delta clone via alternates, bare managed host, `worktree_of_branch` from a clone, relative alternates).
- Rejected: leaving `Host.store`/`own_git_dir` "in case" (dead code, and they only existed for pin resolution).

---
▸ 2026-10-31T16:46:56Z [sheds-1]

Decision: tests `shed_activity.rs` and `shed_title.rs` removed; `tests/sheds.rs` (added in yaks-da23's commit) asserts the farm has no `.activity` directory after writes.

- `inspecting_a_shed_changes_nothing_in_it` (the yaks-9b9f fsmonitor flake) went with the old `sheds.rs` tests, as the brief said.
- `tests/dash_c.rs`: `init_and_discover_honor_c` is now `init_honors_c` (the `discover` half is gone).

---
▸ 2026-10-31T16:47:35Z [sheds-1]

verify: `GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.fsmonitor GIT_CONFIG_VALUE_0=false cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-10-31T16:47:44Z [sheds-1]

Summary (sheds-1): the git/Delta shed discovery is gone.

- Deleted: `src/discover.rs` and `yaks discover` (command, help, `docs/cli.md`, README), `scripts/discover-fixture.sh`, `src/activity.rs` with its `store::write::save` call and the `.activity` entry in `only_leftovers`, `record_thread` and the `yaks.shed.title`/`yaks.shed.thread` writes (and the call in `main`), and all the pin/inspect/git-state code.
- Kept, as `src/gitrepo.rs` (no behavior change): `host`, `Host`, `host_checkout`, `canonical`, `worktree_of_branch`, `common_dir`, `alternates`.
- Tests: the full workspace gate passes (the branch-farm and discovery tests are untouched); `yaks verify` PASS.
- Leftover grep over `docs/ README.md src/ tests/ scripts/` for `discover`, `.activity`, `activity log`, `YAKS_NO_ACTIVITY`, `shed.title`, `refs/delta`, `record_thread`, `--sheds`: only `yaks log`'s own "activity log" wording remains (a different thing).
- yaks-6beb and yaks-9b9f are marked obsolete in notes for the yakherd to slaughter. yaks-2177 not touched.

---
▸ 2026-10-31T16:47:44Z [sheds-1]

moved: shaving -> shorn

---
▸ 2026-10-31T16:50:11Z [delta:Cloud Runners]
kind: landed
commit: b2e4638ab5f8a8637551e01168be509ec833e4a3
source: a37186b0d7ebcf21b8056d2a772fe311c7cec634
branch: main
---

landed: b2e4638 yaks-d3ad: remove git/Delta shed discovery: yaks discover, the activity log, yaks.shed.title

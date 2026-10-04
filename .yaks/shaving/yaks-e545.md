---
id: yaks-e545
title: 'yaks lanes: CLI and data layer for sibling checkouts (git worktrees and Delta clones)'
type: feature
priority: 2
created: '2026-10-04T20:52:28Z'
updated: '2026-10-04T20:52:37Z'
parent: yaks-38dc
labels:
- cli
- delta
---

Read-only CLI + data layer for yaks-38dc (the TUI lanes view is a separate, later yak that calls this). Design agreed with Joel in yaks-99ec / yaks-df61 O50-O53: no Delta database, no Delta CLI; plain git plus the filesystem.

`yaks lanes [--json]` lists the OTHER checkouts of the current repo and what each one's farm says relative to this checkout's farm.

Discovery (new module src/lanes.rs; Farm gets a thin method, the CLI and later the TUI both call it):
1. Plain git worktrees: `git worktree list --porcelain` (skip the bare entry and prunable/missing paths).
2. Delta clones. `git worktree list` shows ONLY the current clone inside Delta (probe yaks-f49c, O51), so also scan sibling directories. A Delta checkout is `<root>/<dir>/<name>` in both layouts: laptop `<repo>/.delta/worktrees/<dir>/<name>`, managed machine `~/.local/share/delta/worktrees/<dir>/<name>`. With T = this git top-level and name = basename(T): candidates are `dirname(dirname(T))/*/<name>`, and, when T is itself a real checkout, `T/.delta/worktrees/*/<name>`. Accept a candidate only if it has its own `.git` (file or dir) and is the same repository: its git dir's `objects/info/alternates` equals ours, else the same `remote.origin.url`. Never accept T itself; dedupe by canonical path.
3. A lane found by both sources is one lane. Kind: self is not listed; show `worktree` or `delta`.

Per lane report: path; branch or detached; HEAD short sha; commits it has that this HEAD lacks (`git rev-list --count HEAD..<lane HEAD>`, 0 when the object is unknown); dirty file count; newest mtime under its `.yaks` (a liveness hint: finished threads' checkouts persist); and its FARM DELTA against this farm, computed by comparing the two `.yaks` directories on disk (no git needed): yaks only in the lane, yaks whose status directory differs (`hairy -> shaving`), yaks whose notes differ (count of new note entries), yaks with `needs:` set in the lane. Resolve the lane's farm with `store::discover_with(lane_path, None)` (NEVER the env var). If the lane has no farm of its own (private: discovery errors) say `no farm here (private? set YAKS_DIR)`; if it resolves to the SAME farm directory as this checkout say `shares this farm`.

Hard rules: strictly read-only in other checkouts. Use `git --no-optional-locks -C <lane> ...` for status so no index refresh happens; no fetch, no checkout, no file writes there. A lane that cannot be read is listed with its error, never fatal. Exits 0 with an empty list when there are no siblings.
Output: a compact human table by default; `--json` has the same fields. `yaks lanes` outside a git repo or farm fails like other commands.
Evidence: unit/integration tests building temp repos for: a plain `git worktree add` lane; a fake managed-layout sibling (`<root>/<dir>/<name>` with an alternates file); a fake laptop-layout one under `T/.delta/worktrees`; a same-named but different repository that must be rejected; lane with an added yak, a moved yak, an added note, a `needs: human`; a private-farm lane; an unreadable lane; and a test that nothing in the lane changed (file mtimes and `git status` identical before and after). Plus a real run: in a Delta checkout, `yaks lanes` shows the coordinator's checkout as a lane carrying its uncommitted yak changes. Docs parity (docs/cli.md, README, --help, `.agents/skills/yaks/SKILL.md` command list if it has one). Leave the coordination skills (`.agents/skills/yaks-coordinating*`) to the coordinator; report what in them is now out of date.
Out of scope: TUI, `yaks diff` (ref-generic, gated, yaks-70e5), pins/refs/delta, anything that talks to Delta, changes to discovery rules.

---
▸ 2026-10-04T20:52:37Z [delta-lead]
Claim (coordinator delta-lead). Owner: worker lanes-1. Scope: new src/lanes.rs, a thin Farm method, the 'lanes' subcommand in src/main.rs, docs/README/skill command list. Judge: coordinator re-runs 'cargo test -p yaks' and 'yaks lanes' in my checkout against the real Delta layout (this thread has sibling checkouts). Forbidden: other checkouts' files (read-only), TUI, Cargo/config, discovery rule changes, coordination skills.

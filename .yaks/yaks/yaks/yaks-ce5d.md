---
id: yaks-ce5d
title: 'TUI: farm dirty/ahead/behind indicator + commit and sync keys'
status: shorn
type: feature
priority: 2
created: '2026-10-21T20:31:47Z'
updated: '2026-10-21T21:44:08Z'
parent: yaks-472e
depends_on:
- yaks-e3b2
labels:
- tui
- git
---

The TUI side of the explicit sync lifecycle (yaks-472e: no auto-commit, so the human needs to SEE that the farm has unpublished state and act on it in place).

---
▸ 2026-10-21T21:11:20Z [delta:herd-dirs]

Claim (delta:herd-dirs, yakherd). Owner: `tui-sync-1`.

**Goal:** the TUI shows the farm's sync state at a glance and can commit/sync without leaving it.

**Scope (files/symbols):**
- State: on the `App`, a `farm_sync: Option<FarmSync>` holding (a) the uncommitted yak count from `status::collect(root)` (its `yaks.len()` + `other.len()`, or `clean()`), and (b) `status::sync_state(root)` (ahead/behind/upstream). Refresh it on startup, after every TUI mutation's existing reload path, on the file-watcher reload, and after commit/sync; it is cheap (local git, never fetches). An in-tree or private farm: show only the dirty count (no sync part). Put the struct + refresh in a focused spot (e.g. `src/tui/viewmodel.rs` or a new `src/tui/farmsync.rs` + `mod` line in `src/tui.rs`).
- Indicator: in the tab bar's right side (where `filter:` already renders, `render_tabs` / whatever draws that row in `src/tui/render.rs`), a compact chip: `● 3 uncommitted` when dirty, `↑2` ahead, `↓1` behind, `not published` when no upstream, nothing at all when clean and up to date. Dim, with a warning accent only for `behind`.
- Keys (pick free ones; `W` and `U` are free; verify against the handlers): one key commits the farm (`commit::run(root, None, false)`), one runs `sync::run(root, &Opts::default())`. Both run synchronously (sync touches the network: show `syncing…` in the notification line first, draw, then run), then reload, and put the result (`committed <sha> <message>` / the sync report's last line / the error, one line) in `app.notification`. No confirmation prompt (both are non-destructive), but nothing at all happens automatically.
- Help: the `?` overlay (`help_content` in `src/tui/render.rs`) gets both keys; the bottom bar (`help_hint`) does NOT need them (it is full), but the chip itself is the cue.
- Docs: `docs/tui.md` (keys + the chip), and one clause in `.agents/skills/yaks/SKILL.md`'s TUI paragraph.

**Out of scope:** background fetch, auto-anything, changing sync/commit behaviour, the Sheds tab.

**Evidence (judge: the yakherd, looking at frames):**
- insta snapshot tests in `src/tui/tests.rs` for: clean (no chip), dirty count, ahead/behind/not-published (inject a `FarmSync` value; do not depend on a real git remote in snapshot tests).
- one behaviour test: the commit key on a dirty team farm in a scratch git repo commits and the chip disappears.
- a headless frame (`yaks tui --headless --size 120x30`, see tests/cli.rs `run_headless` for the stdin protocol) of the chip on a real dirty farm, attached to this yak with `yaks attach`.
- `cargo test --release` green; the `?` overlay frame showing the two keys.

---
▸ 2026-10-21T21:11:20Z [delta:herd-dirs]

moved: hairy -> shaving

---
▸ 2026-10-21T21:11:51Z [tui-sync-1]

Start: Wed Oct 21 21:11:51 UTC 2026. Farm file: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-ce5d.md

---
▸ 2026-10-21T21:20:17Z [tui-sync-1]

Decision: keys `W` (commit) and `U` (sync), both in list AND detail focus (like `S`/`P`); not on the Sheds tab (it owns its keys; out of scope).
- Alternatives rejected: `c`/`C`/`s` taken or mnemonic clash; `Ctrl-` chords (not discoverable); a confirm prompt (spec: none, both non-destructive).
- Verified free: grep for `'W'`/`'U'` in `src/tui/*.rs` shows only the two new arms in `handlers.rs`.

Decision: where the chip sits: the far right of the tab row; the existing right slot (notification / `family:` / sheds label) now ends one column left of it (`slot_w` in `render_tabs`), so they coexist instead of fighting. With no chip the geometry is byte-identical (existing snapshots unchanged).
- Rejected: replacing the family label or the notification with it; a second row.

Decision: chip degrades `● 3 uncommitted` -> `●3`, `not published` -> `unpub` when the full text does not fit beside the tabs, and is hidden if even that does not fit.
- Why: found on a real farm-branch scratch farm at 120 columns: tabs (with Sheds) leave ~25 columns, and `● 1 uncommitted not published` (29) vanished. The compact form shows at 120.
- Rejected: leaving it to vanish (the main case, a never-published farm branch, would show nothing at the common width).

Decision: the dirty count is `changes::read` + `changes::classify` (`yaks.len() + other.len()`), the same numbers `status::collect` derives, NOT `status::collect` itself.
- Why: `collect` does a `git show HEAD:<file>` per dirty yak (`describe_yak`) plus `changes::outside`, run on every reload and file-watcher event; the count needs only `git status`. `status::sync_state` is called as-is. status.rs/commit.rs/sync.rs untouched.

Decision: refresh points: `with_farm` (startup), `reload()`, `reload_preserving_selection()` (which every mutation path and the file-watcher use), and via the reload after `W`/`U`. Not refreshed on a timer or in the background. Known limit (documented in docs/tui.md): a commit/push done by another process changes git state but not files under `.yaks/`, so the chip catches up at the next reload.
- A private farm (`changes::read` -> None) or a git error gives no chip.

Decision: `W`/`U` run in two steps: the key sets `pending_farm_op` + notification `committing…`/`syncing…`; the event loop (`runtime.rs`) runs it right after the next `term.draw` and redraws (no timer, no thread: sync blocks as the claim specified). The headless adapter's `settle()` runs it before each frame, so toque frames show the settled result.
- Rejected: running inside the key handler (the `syncing…` frame would never be drawn).

Decision: the result line: commit = `committed <sha> <first line of the message>` / `nothing to commit`; sync = the last non-empty line of `sync::Report::render()` (the `farm branch …` status line); any error = `error: <message>` with whitespace collapsed to one line. After the op the yaks are re-read (`reload_preserving_selection`), since a sync's rebase can rewrite yak files.

Decision: tests inject a `FarmSync` value for the snapshots (no real git remote); the behaviour test uses a scratch git repo.
- Note for reviewers: the first TUI write creates `.yaks/.gitignore`, so a one-yak change shows `● 2 uncommitted` in an in-tree farm that did not commit it yet; the test asserts 2 with a comment.

[chip-dirty-then-W](yaks-ce5d/chip-dirty-then-W.txt)

[help-overlay](yaks-ce5d/help-overlay.txt)

[farm-branch-W-then-U](yaks-ce5d/farm-branch-W-then-U.txt)

---
▸ 2026-10-21T21:22:40Z [tui-sync-1]

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-10-21T21:22:40Z [tui-sync-1]

Summary: farm chip on the tab row (`● N uncommitted`, `not published`, `↑N`, `↓N` in warning yellow; compact form when tight), `W` commit / `U` sync in list and detail focus, result in the notification; `?` help section, docs/tui.md, one skill clause.
- New `src/tui/farmsync.rs` (FarmSync, FarmOp, App refresh/request/run); hooks in tui.rs, viewmodel.rs (reloads), handlers.rs (keys), runtime.rs (run after draw), headless.rs (settle), render.rs (chip + help).
- Tests (tests.rs, 11 new): 7 insta snapshots (clean/no chip, dirty, ahead+behind, not published, beside a notification, compact at 120, yields at 30) + help overlay snapshot; behaviour test: dirty in-tree farm in a scratch git repo, `W` commits, chip disappears, 2 commits in the log; W-on-clean / U-on-in-tree error one-liners; no-farm no-ops.
- Evidence frames attached: chip-dirty-then-W.txt (in-tree scratch farm, 120x30: `● 3 uncommitted` then after W `committed d5530bb …`, chip gone), farm-branch-W-then-U.txt (scratch farm branch + bare remote: after W `not published`, after U `farm branch yaks: up to date with origin/yaks`, remote got the branch), help-overlay.txt (frame 55 shows W and U).
- Gate: cargo test --release all green (785 unit + integration suites).
- Not done / known: chip is not refreshed by commits/pushes made by another process until the next reload (documented).

---
▸ 2026-10-21T21:22:40Z [tui-sync-1]

moved: shaving -> shorn

[real-farm-chip](yaks-ce5d/real-farm-chip.txt)

---
▸ 2026-10-21T21:44:08Z [delta:herd-dirs]

Landed (delta:herd-dirs): 47120db (fast-forward, linear). Review: read the chip snapshots (ahead/behind, dirty), full gate 20 suites green, and a headless frame of THIS repo's real farm from Joel's checkout (attached real-farm-chip.txt): the tab row ends `family: ~remaining ● 2 uncommitted ↑13`. W/U not pressed on the real farm.

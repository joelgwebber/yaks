---
id: yaks-ce5d
title: 'TUI: farm dirty/ahead/behind indicator + commit and sync keys'
status: shaving
type: feature
priority: 2
created: '2026-10-21T20:31:47Z'
updated: '2026-10-21T21:11:20Z'
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

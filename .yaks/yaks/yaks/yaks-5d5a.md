---
id: yaks-5d5a
title: On-screen log spew when syncing
status: shorn
type: bug
priority: 2
created: '2026-10-22T12:29:53Z'
updated: '2026-10-22T19:18:30Z'
labels:
- tui
---

Sometimes, at least. Especially right after a migration:
```
rename .yaks/{ => canon/yaks}/canon-c398.md (100%)
rename .yaks/{ => canon/yaks}/canon-c4c3.md (100%)
rename .yaks/{ => canon/yaks}/canon-c67f.md (100%)
rename .yaks/{ => canon/yaks}/canon-c6ac.md (100%)
rename .yaks/{ => canon/yaks}/canon-c6cd.md (100%)
rename .yaks/{ => canon/yaks}/canon-c739.md (100%)
rename .yaks/{ => canon/yaks}/c...
```

---
▸ 2026-10-22T12:29:56Z [Joel Webber]

moved: hairy -> shaving

---
▸ 2026-10-22T18:45:11Z [delta:ksQQxR6dwDfzTEa0Hjn76t535ZLOACjjyZIBxBCA]

Cause: `commit::run` ran `git commit` with inherited stdio (`.status()`), so git's summary (`rename .yaks/{ => canon/yaks}/... (100%)`, one line per file after a migration) and any hook output went straight to the terminal, under the TUI's alternate screen when sync (`U`) or commit (`W`) ran. Every other git call in the sync path already captures output.

Fix: `src/commit.rs` now runs the commit through the captured `changes::git` helper; on failure stdout+stderr go into the error (so a failing hook is still visible, and the TUI shows errors in its status line). Test `hook_output_is_captured_and_reported_on_failure`; `cargo test --bin yaks commit::` passes (8). docs/cli.md note updated.

Not verified: a live TUI sync after a migration (no repro farm); the CLI `yaks commit` no longer shows hook output on success either.

---
▸ 2026-10-22T18:45:11Z [delta:ksQQxR6dwDfzTEa0Hjn76t535ZLOACjjyZIBxBCA]

moved: shaving -> shorn

---
▸ 2026-10-22T19:18:15Z [delta:Land commit output capture fix]

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-10-22T19:18:30Z [delta:Land commit output capture fix]

Landed on main as aec9c81 (local, not pushed). Gate: yaks verify yaks-5d5a PASS. Also added .agents/skills/land (Land Changes skill).

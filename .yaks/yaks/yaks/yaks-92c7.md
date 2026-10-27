---
id: yaks-92c7
title: Tidy requests add missing inline code and code fences
status: shorn
type: task
priority: 3
created: '2026-10-27T19:14:55Z'
updated: '2026-10-27T19:16:11Z'
labels:
- markdown
---

---
▸ 2026-10-27T19:14:55Z [delta:Check yaks-6beb comment list formatting]

moved: hairy -> shaving

---
▸ 2026-10-27T19:16:11Z [delta:Check yaks-6beb comment list formatting]

The canned tidy instruction now says to add missing code formatting: fence commands, code and pasted output with a language, and put inline commands, paths, flags, ids and symbols in backticks. That matches the skill's existing Shape rule, so the skill needed no change.

Evidence:

- `cargo test`: all suites pass; `tests/needs_loop.rs` now asserts the new sentence.
- With the built binary in a scratch farm, `yaks request <id> --kind tidy` writes the new instruction.
- Surfaces updated together: `src/store.rs` (instruction), `src/main.rs` (`--kind` help), `docs/cli.md`.

---
▸ 2026-10-27T19:16:11Z [delta:Check yaks-6beb comment list formatting]

moved: shaving -> shorn

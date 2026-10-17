---
id: yaks-ade2
title: Release 0.0.20
status: shaving
type: task
priority: 2
created: '2026-10-17T19:44:47Z'
updated: '2026-10-17T19:45:47Z'
labels:
- release
---

Cut 0.0.20: the readable-markdown arc (yaks-e9de), schema 5 + `yaks migrate` (yaks-a870), cross-shed attachments (yaks-31e2), the yaks-48d6 perf fix.

---
▸ 2026-10-17T19:44:47Z [delta-lead]

moved: hairy -> shaving

---
▸ 2026-10-17T19:45:47Z [delta-lead]

Version 0.0.20 (Cargo.toml, Cargo.lock, npm/yaks/package.json). The headline since v0.0.19:

- **Schema 5 + `yaks migrate`** (yaks-a870): comment headers get a blank line, non-image attachments link as `[x](…)`. Every command but `migrate` refuses a schema-4 farm. This repo's farm is migrated.
- **Readable markdown in the TUI** (yaks-e9de family): CommonMark rendering via pulldown-cmark (hidden markers, reflow, hanging indents, quotes holding nested blocks, tables, task lists, alerts, footnotes), syntax-coloured code with per-block horizontal scroll, source links to repo files (`#L` ranges, pinned commits) opened in `$EDITOR`, 📎/🖼 attachments, link destination in the status bar, one shared palette, comment headers with relative local dates.
- **Editing**: description and comments edit in place in the detail pane (`E` on a body row, `M` for a new comment); the `E` form keeps header fields only; editor markers dim/hide off the cursor line; lists and quotes auto-continue on Enter (edtui fork); no more black editor background.
- **Sheds**: attachments open from whichever shed has them; `yaks attachment <id> [<file>]` (yaks-31e2).
- **Perf**: no per-row view rebuild (yaks-48d6), memoized rows and detail lines (yaks-bc30).

Gate: full `cargo test --release` (18 suites green), `node npm/yaks/test-mapping.mjs` ok, `yaks doctor` clean, `yaks migrate --dry-run` nothing to migrate.

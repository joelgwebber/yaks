---
id: yaks-39f9
title: 'dok query: contains, compare, not-equal and missing'
status: shaving
type: feature
priority: 3
created: '2026-11-06T04:28:32Z'
updated: '2026-11-06T23:13:37Z'
parent: yaks-fcdc
tags:
- cli
- doks
---

`dok query --where` matches exactly; Hearth's `page_query` also has `~text` (contains), `<x` / `>=x` (compare; dates as YYYY-MM-DD), `!x` and `""` (missing). "Which renew next month" and "which notes are unreviewed" both need it. Already an open question on /doks/design/knowledge-store.md.

---
▸ 2026-11-06T23:12:44Z [herd-upkeep-2]
kind: move
from: hairy
to: shaving
thread: ksQQKUp3Ux8dTNqgb6cSSWpUMpLOAEX_fZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-06T23:13:37Z [herd-upkeep-2]

Claim (yakherd herd-upkeep-2, 2026-11-06): worker `query-1`, then yaks-6cdd in the same thread.

- **Scope:** `dok query --where` (and `--rows`) learns Hearth's operators: `key=~text` (contains, case-insensitive), `key=<x`, `<=x`, `>x`, `>=x` (numbers numerically, dates as YYYY-MM-DD strings), `key=!x` (not equal), `key=` (empty: missing or empty). A plain value stays an exact match, so nothing changes for existing queries. A list field matches when any item does.
- **Files:** `doks::matches` in `src/doks.rs` and the `--rows` filter (`schemas::table_rows` callers), `DokAction::Query` help in `src/main.rs`; docs: `docs/cli.md` `### dok`, the `yaks-research` skill's "Find before you write" and the typed-doks line that says "`dok query` matches values exactly", and the open question on `/doks/design/knowledge-store.md` (the yakherd ticks it at landing, not you).
- **Judge:** `cargo test -p yaks` with unit tests per operator and a CLI test for a date comparison.

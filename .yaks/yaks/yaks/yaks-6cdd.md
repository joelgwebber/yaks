---
id: yaks-6cdd
title: Write and delete table dok rows by key
status: shaving
type: feature
priority: 3
created: '2026-11-06T04:28:31Z'
updated: '2026-11-06T23:13:38Z'
parent: yaks-fcdc
tags:
- cli
- doks
---

Table doks can be read by row (`dok query --rows`) but not written by row: Hearth has `page_set path#key` (create or update one row's fields) and `page_delete_row`. Agents now hand-edit the markdown table, which is easy to break.

Add row writes addressed as `<dok>#<key>`, keeping the table's column order and sorting.

---
▸ 2026-11-06T23:12:45Z [herd-upkeep-2]
kind: move
from: hairy
to: shaving
thread: ksQQKUp3Ux8dTNqgb6cSSWpUMpLOAEX_fZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-06T23:13:38Z [herd-upkeep-2]

Claim (yakherd herd-upkeep-2, 2026-11-06): worker `query-1`, after yaks-39f9, in the same checkout.

- **Scope:** write rows of a table dok: `yaks dok row set <dok>#<key> --set col=value ...` creates or updates the row whose first column is `<key>`, keeping the table's column order (a new column is appended to the header and every row gets an empty cell), cell escaping (`|`), and the rows' order (a new row goes at the end, unless the rows are already sorted by key, then in place). `yaks dok row delete <dok>#<key>` removes one. Refused, with what to do, when the dok has no table, or two rows share the key. The rest of the body is untouched byte for byte.
- **Files:** a new `rows` function set beside `schemas::table_rows`, `Farm` methods under the farm lock, a `DokAction::Row` subcommand; docs: `docs/cli.md` `### dok`, the `yaks-research` skill's table-dok line.
- **Judge:** `cargo test -p yaks` with unit tests (append, update, new column, delete, sorted insert, body outside the table unchanged) and a CLI round-trip with `dok query --rows`.

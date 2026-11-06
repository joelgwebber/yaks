---
id: yaks-6cdd
title: Write and delete table dok rows by key
status: shaving
type: feature
priority: 3
created: '2026-11-06T04:28:31Z'
updated: '2026-11-06T23:41:58Z'
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

---
▸ 2026-11-06T23:40:09Z [query-2]

Started (query-2, 23:40Z): new src/rows.rs (table locate/set/delete, split_cells with \| escapes), Farm::dok_row_set/dok_row_delete, DokAction::Row. schemas::table_rows now delegates to rows::table_rows.

---
▸ 2026-11-06T23:41:58Z [query-2]

Decision: new src/rows.rs holds the table code (locate/set/delete/split_cells); `schemas::table_rows` delegates to it so `dok query --rows` and the writes share one definition of "the table". Rejected: a second parser in schemas.rs.

Decision: read side changed slightly with it: cells split on unescaped `|` only (`\|` reads back as `|`), and the separator line must start with `|` (before, a blank line after the header counted as a separator).

Decision: only lines that change are rewritten. An updated row is re-rendered canonically (`| a | b |`); a new column is appended to existing lines (header, separator, rows) so other text keeps its padding; a ragged row is written cell by cell. Rejected: re-rendering the whole table.

Decision: "already sorted" = keys non-decreasing ignoring case (one or zero rows counts as sorted); then the row goes before the first greater key. Otherwise at the end.

Decision: `--set <key column>=v` is refused unless v equals the key (no rename); empty value clears a cell; values and keys must be one line. `row set` with no `--set` creates a key-only row. Rejected: rename-by-set (not in the claim; delete + set does it).

Decision: no JSON output for row writes (not asked).

---
id: yaks-6cdd
title: Write and delete table dok rows by key
status: shaving
type: feature
priority: 3
created: '2026-11-06T04:28:31Z'
updated: '2026-11-06T23:12:45Z'
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

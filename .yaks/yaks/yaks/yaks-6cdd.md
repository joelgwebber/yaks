---
id: yaks-6cdd
title: Write and delete table dok rows by key
status: hairy
type: feature
priority: 3
created: '2026-11-06T04:28:31Z'
updated: '2026-11-06T04:28:31Z'
parent: yaks-fcdc
tags:
- cli
- doks
---

Table doks can be read by row (`dok query --rows`) but not written by row: Hearth has `page_set path#key` (create or update one row's fields) and `page_delete_row`. Agents now hand-edit the markdown table, which is easy to break.

Add row writes addressed as `<dok>#<key>`, keeping the table's column order and sorting.

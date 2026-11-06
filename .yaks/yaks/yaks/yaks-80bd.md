---
id: yaks-80bd
title: 'Small tool gaps: mark sources, write schemas, backdate imports'
status: hairy
type: task
priority: 3
created: '2026-11-06T04:28:32Z'
updated: '2026-11-06T04:28:32Z'
parent: yaks-fcdc
tags:
- cli
- doks
---

Smaller gaps against Hearth's tools, each a hand edit today:

- **Mark a source uncited on purpose** (`source_mark`: `superseded_by` or `uncited`). `yaks check` honours the fields in `source.md`, but nothing sets them and `yaks-research` doesn't say how.
- **Write a schema** (`page_define_type`): `schemas/<kind>.md` is written by hand; the housekeeping pass merges kinds and adds fields, so a command keeps it well formed.
- **Backdate an import** (`HEARTH_DATE` / `-date`): commits and saved sources dated to the day the imported notes describe.

Split any that grows.

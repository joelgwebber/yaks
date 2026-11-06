---
id: yaks-39f9
title: 'dok query: contains, compare, not-equal and missing'
status: hairy
type: feature
priority: 3
created: '2026-11-06T04:28:32Z'
updated: '2026-11-06T04:28:32Z'
parent: yaks-fcdc
tags:
- cli
- doks
---

`dok query --where` matches exactly; Hearth's `page_query` also has `~text` (contains), `<x` / `>=x` (compare; dates as YYYY-MM-DD), `!x` and `""` (missing). "Which renew next month" and "which notes are unreviewed" both need it. Already an open question on /doks/design/knowledge-store.md.

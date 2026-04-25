---
id: yak-bf54.3
title: Add last_synced frontmatter field (and holistic review of yak timestamp fields)
type: feature
priority: 2
created: '2026-04-25T15:47:28Z'
updated: '2026-04-25T15:47:28Z'
---

Add last_synced: <iso> to yak frontmatter, written by /yaks:sync after a successful merge. Used as the fast-path predicate: skip the diff if upstream.updated <= last_synced; sweep mode (bf54.2) batches a JQL filter against this field.

Mechanics:
- yaklib.model: accept and preserve last_synced through load/save.
- /yaks:update: --last-synced flag (or just let the sync skill rewrite the file directly — flag may be unnecessary if only the skill writes it).
- /yaks:show: display last_synced when present.
- TUI detail render: show last_synced on yaks that have it.
- Make sure created/updated/last_synced don't get trampled by routine field edits.

Holistic review (per user direction): existing 'created' and 'updated' fields earn varying amounts of keep — 'updated' is used by TUI sort and 'recent' filter; 'created' is decorative. With last_synced joining the family, audit whether all three are pulling weight, or whether one can be dropped/folded. Likely outcome: keep all three but document semantics clearly. Don't touch behavior unless audit surfaces something genuinely confusing.

---
id: yaks-1c89
title: 'Upkeep: session notes, session review and housekeeping (Hearth''s knowledge procedures in yaks)'
status: hairy
type: feature
priority: 2
created: '2026-11-06T04:27:55Z'
updated: '2026-11-06T04:27:55Z'
parent: yaks-44b3
tags:
- doks
- skills
---

Move the knowledge-*maintenance* half of Hearth into yaks. The storage half (doks, sources, schemas, `find`, `check`) landed under yaks-44b3; what Hearth's `hearth-housekeeping` and `hearth-session-notes` skills and its `session-review` job do has no yaks equivalent yet. Hearth will drop its own versions and keep only the outer agent loop, server and Hearth-specific tools.

Origin: assessment by `herd-upkeep` with Joel, 2026-11-05, comparing `.agents/skills/yaks-research` with Hearth's `internal/defaults/files/skills/` and its yaks (`hearth-e1c5`, `hearth-d740`, `hearth-6e7e`, `hearth-6e98`, `hearth-3285`).

## Rule for the split

As yaks-44b3 decided: what can be decided from the files alone is a command; judgement is a skill; the clock, model, network and trust policy stay in the harness.

## Gaps

- **Session notes:** `sessions/` is reserved, imported and searchable, but nothing writes a note and no skill asks for one.
- **Session review:** nothing finds unreviewed notes, promotes their facts into doks, or stamps `promoted:`.
- **Housekeeping:** `yaks check` finds loose ends; nothing says how to tidy them (Hearth's rules: merge look-alike kinds, fix only unambiguous formats, never choose a date or number, fix clearly renamed links, never delete or rewrite, give an account).
- **Jobs:** recurring yaks with `--wake agent` replace Hearth's job registry, but nothing registers the upkeep ones.
- **Tools an agent layer needs** on top of a farm (hash-checked edits, table row writes, ...).
- **Weekly reflection:** not built in Hearth either; decide the split.

Stays in Hearth: `core/` memory injection, protected paths and trust, fetching (web, Drive, mail), the clock, transcripts, the morning summary.

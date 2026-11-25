---
id: yaks-5bc9
title: 'Conditional writes on yaks: --if status=, --if-latest note, --if-hash'
status: hairy
type: idea
priority: 3
created: '2026-11-25T04:24:42Z'
updated: '2026-11-25T04:24:42Z'
tags:
- cli
- agent
---

Let an agent change a yak only if it is still in the state it read: conditional state moves and notes, as `dok set --if-hash` already does for doks.

Vercel Board has `reply --after <last-id>` (post only if nothing landed since), `label --if k=v` / `--if-unset k` and `edit --if-revision N`, each failing with a stable reason and changing nothing ([CLI reference](/sources/2026-11-25-476f2206-vercel-board-docs-cli-reference.md#edit-delete-react-and-label), [research](/doks/research/vercel-board.md#ideas-worth-taking)).

## Candidates

- `--if status=<s>` on the state moves and `update` (any frontmatter field, `--if field=` for missing).
- `--if-latest <note anchor>` on `update --note`: refuse if a note landed after the one the agent read (`show --json` already lists anchors).
- `--if-hash <h>` on `update` for a yak, the same hash `dok show` reports.

All under the farm lock; a refusal writes nothing and exits non-zero with a stable reason (see the error-reasons yak). Exclusive claims (filed beside this) may cover the most common case; decide whether both are needed.

---
id: yaks-ba2f
title: 'Plan files: create a family of yaks with dependencies from one file (create --file)'
status: hairy
type: feature
priority: 2
created: '2026-11-25T04:24:40Z'
updated: '2026-11-25T04:24:40Z'
tags:
- cli
- agent
---

Create a whole family of yaks, with dependencies between them, from one file in one locked write: `yaks create --file plan.yaml`.

Taken from Vercel Board's `tasks create --file plan.json` ([research](/doks/research/vercel-board.md#ideas-worth-taking), [Board's tasks docs](/sources/2026-11-25-b347410c-vercel-board-docs-tasks-claims-leases-needs.md#create-a-plan-of-tasks)).

## Shape

- Each entry: `title`, and optionally `key`, `description`, `type`, `priority`, `tags`, `parent` (a key in the plan or an existing id), `depends_on` (keys or ids), `verify`, `set`.
- `key` is local to the plan: it names an entry for the others and is not stored.
- Unknown entry keys are refused before anything is written (Board's point: a typo such as `need` must not create a yak that waits for nothing).
- One lock, one farm commit; `--json` prints the key-to-id map.
- `--dry-run` prints what it would create.

## Open

- [ ] Idempotency: an `--idempotency-key`, or a `source:`-like plan key per entry so re-running creates nothing (`create --key` already does this for one yak)?
- [ ] YAML only, or JSON too (stdin with `-`)?
- [ ] Does `yaks brief` want a matching "plan" view of a family?

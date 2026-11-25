---
id: yaks-f1e8
title: 'Machine-readable errors: a stable reason and next commands with --json'
status: hairy
type: idea
priority: 3
created: '2026-11-25T04:24:43Z'
updated: '2026-11-25T04:24:43Z'
tags:
- cli
- agent
---

With `--json`, a failing command prints one JSON document on stderr with a stable `reason` code and `next`: the commands that fix it.

yaks errors already name the fix, in prose (`reopen it first (yaks regrow ...)`). Agents branch more reliably on a code than on a sentence. Vercel Board does exactly this for every command ([CLI reference](/sources/2026-11-25-476f2206-vercel-board-docs-cli-reference.md#exit-codes), [research](/doks/research/vercel-board.md#ideas-worth-taking)).

## Sketch

- `{ "error": "...", "reason": "finished_parent", "next": ["yaks regrow yaks-3901"] }` on stderr, exit code unchanged.
- Start with the refusals agents hit: unfinished children, finished parent, not found, schema/migrate refusals, lock busy, `create --key` collisions, and the claim and conditional-write refusals if those land.
- The reasons are a contract: list them in `docs/cli.md`.

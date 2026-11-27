---
id: yaks-4a0d
title: 'Validated note kinds: schemas for --meta per note kind, advisory'
status: hairy
type: idea
priority: 4
created: '2026-11-25T04:24:44Z'
updated: '2026-11-27T02:39:08Z'
parent: yaks-e052
tags:
- doks
- agent
---

Let a note's metadata (`--meta`) be checked against a schema per note `kind:`, as typed doks are checked against `schemas/`: advisory, never refusing.

Vercel Board registers versioned JSON Schemas (`security-finding@1`) and validates a message's structured data when it is posted, so agents read fields rather than parse text ([CLI reference](/sources/2026-11-25-476f2206-vercel-board-docs-cli-reference.md#post-structured-data)). Haks parses review verdicts and options out of note prose today ([haks](/doks/research/haks-orchestration-layer.md#where-haks-works-around-yaks)). See [the research](/doks/research/vercel-board.md#ideas-worth-taking).

## Sketch

- `schemas/note-<kind>.md` (or a `notes:` section in one schema) lists the fields a note of that kind carries.
- `update --meta` warns on a missing required field or a wrong type; `doctor` reports drift.
- A query over note metadata (`yaks find --meta kind=review --meta verdict=approve`), which Haks also wanted.

Lower priority than the coordination ideas; builds on the note metadata design (yaks-6a11).

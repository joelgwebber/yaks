---
id: yaks-d11e
title: 'Generated ## Saved: scope it to the session, not the actor''s last 12 hours'
status: hairy
type: feature
priority: 2
created: '2026-11-06T21:36:38Z'
updated: '2026-11-06T21:36:38Z'
parent: yaks-3f35
tags:
- cli
- doks
---

From yaks-3f35, item 8. When a note has no `## Saved`, `yaks session note` writes one from `yaks session touched`: what the actor wrote since its last session note, at most 12 hours back. That window is wrong both ways:

- **Misses work.** An import's writes (hand edits committed later, backdated commits, work older than 12 hours) are not in it, so the schism import's generated Saved was useless.
- **Catches unrelated work.** Two sessions under one actor name (parallel threads, or a name reused across days) share a window, and a first note picks up 12 hours of whatever came before.

## Lean

- When the session has a thread id (a Delta thread), select by it: every note and move already carries `thread:` metadata, so `touched` can take exactly this conversation's notes, with the window only as a fallback for harnesses without one.
- A backdated note (yaks-0fa7) never generates Saved.
- When the window is the fallback and there was no previous note, say what window was used in the command's output, so the agent can check it.

## Evidence

Tests for thread selection and two interleaved sessions under one actor; the `yaks-upkeep` skill's Saved paragraph updated.

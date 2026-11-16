---
id: yaks-3fc6
title: session note records the Delta thread under another --harness
status: hairy
type: bug
priority: 2
created: '2026-11-06T21:36:37Z'
updated: '2026-11-06T21:36:37Z'
parent: yaks-3f35
tags:
- cli
---

From yaks-3f35, item 3. `yaks session note --harness claude-code`, run from a Delta thread (the schism import wrote its devlog's Claude Code sessions that way), still records that Delta thread as `thread:`. The note then claims a transcript that is not the session's.

Cause, in the `upkeep` work's `src/main.rs`: `Origin { harness: harness.or_else(harness_from_env), thread: actor::harness_thread(), .. }` takes the thread from the environment whatever `--harness` says.

## Fix

Record the environment's thread only when the note's harness is the environment's harness. Consider a `--thread <id>` for a note about a session from elsewhere.

## Evidence

A test with `--harness` naming another harness in a Delta environment: no `thread:`.

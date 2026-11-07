---
id: yaks-f4ea
title: 'YAKS_THREAD: a harness-neutral thread id for moves and session notes'
status: hairy
type: feature
priority: 3
created: '2026-11-07T16:57:32Z'
updated: '2026-11-07T16:59:19Z'
parent: yaks-44b3
tags:
- cli
- doks
---

yaks records `thread:` metadata on moves (and a `thread` field on session notes) only from Delta's `DELTA_CURRENT_THREAD_ID` (`src/actor.rs`). A harness that is not Delta, such as Hearth (hearth-1ed0 in the Hearth farm), cannot point its writes back at its own conversation.

Proposal: `YAKS_THREAD=<id>` wins over the Delta variable wherever yaks reads a thread id: move metadata, `session note`'s `thread:`, and the `whoami` report. Nothing else changes. Related: yaks-8003 (stable thread ids in actor stamps) and yaks-3fc6 (session note records the Delta thread under another `--harness`).

Small; one test per place it is read.

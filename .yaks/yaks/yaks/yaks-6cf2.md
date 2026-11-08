---
id: yaks-6cf2
title: 'check: flag doks whose cited code has changed since the pin or last verification'
status: hairy
type: feature
priority: 3
created: '2026-11-08T16:24:07Z'
updated: '2026-11-08T16:24:07Z'
parent: yaks-1c89
tags:
- cli
- doks
---

From [the agent-memory research](/doks/research/agent-memory-dreaming.md#3-let-evidence-not-only-the-clock-say-what-to-re-check) (yaks-e4f5): `yaks check` only knows clock staleness (`stale_after:`). A dok that describes code goes wrong when the code moves, and the farm already holds the evidence: `code:` links name the files, `@sha` pins what was read.

Add one model-free finding to `src/check.rs`: a dok that cites `code:<path>` where the path has commits in the code repo since the pinned sha (or, unpinned, since the dok's `verified:` or last commit). Report the dok, the path and the commit count. It never edits and decides nothing about whether the dok is wrong: an agent reads the dok against the code and sets `verified:` (the existing meaning of that field).

Design points to settle in the work: what counts for an unpinned link, whether a count threshold is useful, and what to do when no code checkout is reachable (skip, as `peek` refuses). Docs and the `yaks-research` skill's `yaks check` paragraph change with it.

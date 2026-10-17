---
id: yaks-7c76
title: Find-references affordance in CLI/TUI
status: hairy
type: feature
priority: 3
created: '2026-10-17T14:22:08Z'
updated: '2026-10-17T14:22:08Z'
labels:
- cli
- tui
- links
---

I don't recall whether this functionality is hiding in the core CLI already, but it would be really helpful to be able to find references *to* a given yak, from desc/comments. We only model forward-references formally, which is the right choice, so we don't have to maintain an external back-ref index (or make updates more complex).

But find-references can be a bit slower, and perhaps (eventually) leverage a hidden index if needed. We'd want the core functionality in the lib/cli, and a simple way in the tui to quickly find and filter them. I imagine we could leverage the existing fuzzy-search tui for filtering from the set of incoming references. Including parents/children/blockers/blocked-byj -- even though those are easy to find in the detail UI, it would still be convenient to quickly filter them alongside other back-refs.

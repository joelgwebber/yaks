---
id: yak-bf54.1
title: Investigate reliable last-modified timestamps across Jira/Linear/GitHub MCP
  tools
type: task
priority: 2
created: '2026-04-24T03:01:44Z'
updated: '2026-04-24T03:01:44Z'
---

Blocks a 'last_synced' field on yaks and any sweep/check mode. For each tracker's MCP: can we cheaply fetch (issue updated_at, per-comment updated_at, per-attachment created_at)? If the data isn't exposed uniformly enough, a local last_synced buys us nothing and we stick with full-diff sync. Deliverable: a short writeup appended here with what each MCP exposes and a recommendation.

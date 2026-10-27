---
id: yaks-d695
title: Add pull-request(s) field to frontmatter
status: hairy
type: feature
priority: 3
created: '2026-08-26T12:58:19Z'
updated: '2026-10-27T01:18:50Z'
labels:
- schema
---

Alongside 'source' for upstream issues, we need a place to track active PR(s) as well.

I think this needs to handle multiple PRs, as that's a pretty common pattern.

Source can be singular, because we want to discourage a single yak from representing multiple issues (that's what herds are for).

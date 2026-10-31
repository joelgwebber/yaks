---
id: yaks-d695
title: Add pull-request(s) field to frontmatter
status: hairy
type: feature
priority: 3
created: '2026-08-26T12:58:19Z'
updated: '2026-10-31T15:56:26Z'
parent: yaks-039b
labels:
- schema
---

Alongside 'source' for upstream issues, we need a place to track active PR(s) as well.

I think this needs to handle multiple PRs, as that's a pretty common pattern.

Source can be singular, because we want to discourage a single yak from representing multiple issues (that's what herds are for).

---
▸ 2026-10-31T15:56:26Z [delta:Cloud Runners]

Adopted into `yaks-039b` (2026-10-31) so the PRs field stays consistent with the other "where does this yak's code live" fields (`yaks-bdf9` landed shas, `yaks-a3fc` worker branches). Where it lives (frontmatter vs note metadata) follows `yaks-6a11`.

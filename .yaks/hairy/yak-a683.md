---
id: yak-a683
title: Search tokenization improvements
type: feature
priority: 2
created: '2026-08-01T17:34:50Z'
updated: '2026-08-15T05:10:56Z'
labels:
- search
---

It would be really helpful if yak searches could be a bit more contextually aware. Some examples:
- When I have a field like `source: ...github/issues/123`, it would be great if `123` matched this.
  We might also consider special forms like `source:123` that use tokenization rules to limit the scope of the search.

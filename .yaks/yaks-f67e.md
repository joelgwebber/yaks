---
id: yaks-f67e
title: 'Fix: markdown link URL target swallows the closing paren'
status: hairy
type: bug
priority: 2
created: '2026-10-17T04:24:29Z'
updated: '2026-10-17T04:24:29Z'
parent: yaks-ea13
labels:
- markdown
- tui
---

`[Links](https://example.com/)` in a body yields a `Target::Url("https://example.com/)")`: `scan_body_links` (src/tui/detail.rs) reads a bare URL up to the next whitespace. Fix the scan to stop at an unbalanced `)` (and trailing `.,;:!?` punctuation), the GFM autolink rule. A test that fails first: the URL target for `[x](https://e.com/)` and for `see https://e.com/a).` .

Standalone now; the renderer core (pulldown-cmark) later replaces the scan for markdown links, but bare-URL autolinking keeps this rule.

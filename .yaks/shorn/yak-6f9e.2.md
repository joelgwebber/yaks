---
id: yak-6f9e.2
title: Auto-detect repo from serving URL
type: feature
priority: 2
created: '2026-04-17T23:13:23Z'
updated: '2026-04-17T23:53:15Z'
commit: 46326de
---

When hosted on GitHub Pages (e.g. joelgwebber.github.io/yaks), the repo is implicit from the URL. Auto-populate the repo field from the hosting domain so users don't have to type it. The input should still be editable for pointing at other repos.

### 2026-04-17T23:47:43Z
Implemented in docs/index.html v2 rewrite.

### 2026-04-17T23:53:15Z
Verified working on GH Pages — joelgwebber.github.io/yaks auto-detects to joelgwebber/yaks and boots without needing a hash.

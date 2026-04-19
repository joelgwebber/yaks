---
id: yak-6f9e.14
title: 'Web UI: resolve artifact image paths to raw.githubusercontent.com'
type: bug
priority: 2
created: '2026-04-19T00:11:28Z'
updated: '2026-04-19T00:11:28Z'
---

Artifact images referenced in yak descriptions as relative paths (artifacts/yak-xxxx/foo.png) don't load in the web UI because the base URL is the GH Pages host, not raw.githubusercontent.com. Rewrite relative artifact paths to absolute raw.githubusercontent.com URLs at render time. The TUI handles this by parsing artifacts and opening them via the local filesystem.

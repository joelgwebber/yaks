---
id: yaks-9f5e
title: import-hearth skips dot-files inside a directory source
status: hairy
type: bug
priority: 2
created: '2026-11-02T03:01:25Z'
updated: '2026-11-02T03:01:25Z'
parent: yaks-44b3
tags:
- wiki
- cli
---

Running `yaks import-hearth` on `~/src/fs/joel` (2026-11-02) left out 4 files of source `raw/sources/2026-01-08-376fe3e0/files/`: `.claude-plugin/marketplace.json`, `.claude-plugin/plugin.json`, `.gitignore` and `.mcp.json`. The walk in `src/hearth_import.rs` skips names starting with `.` everywhere. A source is written once and must import byte for byte, so dot-files under `raw/sources/<id>/` must come along. Skipping at the repo top level (.git, .hearth) is fine. Found by checking every removed file was present in the farm before deleting it from main; copied by hand in joel farm commit b7374aa. Test: a fixture source with `files/.mcp.json`.

---
id: yak-a1bc.2
title: Add .codex-plugin/plugin.json Codex plugin manifest
type: task
priority: 2
created: '2026-05-31T16:26:07Z'
updated: '2026-05-31T16:26:59Z'
parent: yak-a1bc
---

Add a proper .codex-plugin/plugin.json manifest so Yaks is recognized as a first-class Codex plugin (in addition to the existing .claude-plugin/ legacy path Codex already reads). Minimal manifest pointing at skills/ directory. Codex plugin format: {name, version, description, skills: './skills/'}.

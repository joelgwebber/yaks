---
id: yak-c6c3.3
title: Drop commands/ directory and command wiring
type: task
priority: 2
created: '2026-07-25T20:48:26Z'
updated: '2026-07-25T21:19:42Z'
depends_on:
- yak-c6c3.2
---

Delete the 18 commands/*.md Claude slash-command files (obviated by the TUI + direct-CLI skill). Remove any command references from .claude-plugin/ and .codex-plugin manifests. Verify the Claude plugin still installs cleanly with skills-only. Depends on c6c3.2 so Claude users retain guidance via the rewritten skill before the commands go away.

---
▸ 2026-07-25T21:19:42Z
Done. Deleted commands/ (18 Claude slash-command .md files) — obviated by the TUI and the direct-CLI skill. No manifest changes needed: neither .claude-plugin nor .codex-plugin referenced commands/ (Claude auto-discovers the dir by convention), and skills/ is likewise auto-discovered, so the plugin is now cleanly skills-only.

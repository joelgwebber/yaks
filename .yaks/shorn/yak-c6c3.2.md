---
id: yak-c6c3.2
title: Rewrite yak skill to direct-CLI (uvx) invocation
type: task
priority: 1
created: '2026-07-25T20:48:21Z'
updated: '2026-07-25T21:18:55Z'
depends_on:
- yak-c6c3.1
---

Rewrite skills/yak/SKILL.md to the yak-tracker skill's style: teach direct CLI invocation (uvx yakshave <cmd>, or bare yaks after uv tool install). Drop the Claude-only framing entirely: remove the 'Always use the Skill tool to invoke these commands / Do not run yak.py directly via Bash' rule and all /yaks:* slash-command references. Keep all workflow/hard-rules/parent-child/format/filtering content. This is the high-leverage fix that makes yaks work under Codex and Zed (both load skills but have no slash commands). Depends on the distribution name chosen in c6c3.1.

---
▸ 2026-07-25T21:05:05Z
Use 'yakherder' as the distribution name (not the placeholder 'yakshave'). Skill leads with 'uvx yakherder <cmd>' for agents; mention 'uv tool install yakherder' -> bare 'yaks' as the local-install option.

---
▸ 2026-07-25T21:18:55Z
Done. Rewrote skills/yak/SKILL.md to direct-CLI style: added a 'Running yaks' section (prefer 'yaks' on PATH via 'uv tool install yakherder'; else 'uvx yakherder <cmd>'; git fallback until published). Removed the Claude-only 'Always use the Skill tool / do not run yak.py via Bash' mandate and converted every /yaks:* reference to bare 'yaks <cmd>'. Added 'yaks tui' to the command table. All workflow/hard-rules/parent-child/format/filtering content preserved. NOTE: the installed copy at ~/.agents/skills/yak/SKILL.md is the plugin output — user must reinstall/re-sync the plugin to pick this up in Zed/Codex.

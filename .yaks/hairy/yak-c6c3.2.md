---
id: yak-c6c3.2
title: Rewrite yak skill to direct-CLI (uvx) invocation
type: task
priority: 1
created: '2026-07-25T20:48:21Z'
updated: '2026-07-25T21:05:05Z'
depends_on:
- yak-c6c3.1
---

Rewrite skills/yak/SKILL.md to the yak-tracker skill's style: teach direct CLI invocation (uvx yakshave <cmd>, or bare yaks after uv tool install). Drop the Claude-only framing entirely: remove the 'Always use the Skill tool to invoke these commands / Do not run yak.py directly via Bash' rule and all /yaks:* slash-command references. Keep all workflow/hard-rules/parent-child/format/filtering content. This is the high-leverage fix that makes yaks work under Codex and Zed (both load skills but have no slash commands). Depends on the distribution name chosen in c6c3.1.

---
▸ 2026-07-25T21:05:05Z
Use 'yakherder' as the distribution name (not the placeholder 'yakshave'). Skill leads with 'uvx yakherder <cmd>' for agents; mention 'uv tool install yakherder' -> bare 'yaks' as the local-install option.

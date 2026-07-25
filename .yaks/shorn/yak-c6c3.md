---
id: yak-c6c3
title: Revisit commands/cli structure and skills
type: task
priority: 1
created: '2026-07-25T20:32:55Z'
updated: '2026-07-25T21:23:13Z'
---

I'm finding that agents often get confused about the CLI tools, perhaps because they don't all support "commands"?

I believe Claude might have worked better, but when I'm using Zed it's not picking up on the CLI without some struggle. The commands also reference the claude plugin root to execute the python. We should consider replacing this whole flow with a simpler "skill + uvx" approach, or something equally simple. I don't think we need MCP tools, because the CLI works fine and doesn't need to hold onto persistent process state.

---
▸ 2026-07-25T20:48:51Z
Plan agreed: go skill+uvx, no MCP. Root cause = instructions are Claude-shaped (slash commands + CLAUDE_PLUGIN_ROOT) but yaks runs under Claude/Codex/Zed; only Claude has slash commands, so Codex/Zed agents are told to use nonexistent commands. yak-tracker skill is already in the target (direct-CLI) style. Split into c6c3.1 packaging, .2 skill rewrite, .3 drop commands, .4 docs. PyPI 'yaks' is TAKEN (abandoned ADLINK/zenoh pkg) -> publish under distribution 'yakshave' (available), keep command 'yaks', add yakshave script alias for clean 'uvx yakshave'. Unify pyproject+plugin versions in lockstep. Actual PyPI upload is user-gated (trusted publishing).

---
▸ 2026-07-25T21:05:05Z
FINAL NAME: yakherder (distribution) / yaks (command). Supersedes the earlier yakshave placeholder. Register yaks + yakherder console-script aliases. Recommend 'uv tool install yakherder' for humans so 'yaks tui' stays on PATH.

---
▸ 2026-07-25T21:23:13Z
All four children shorn. Outcome: yaks now uses a single cross-harness model — direct CLI via the rewritten yak skill (no slash commands, no CLAUDE_PLUGIN_ROOT), packaged as 'yakherder' on PyPI (command stays 'yaks'), commands/ deleted, docs updated, versions unified in lockstep. Remaining hand-off (tracked separately, owner-gated): create the PyPI pending publisher for 'yakherder' and push a v0.1.78 tag to publish; users on Claude/Codex/Zed should reinstall/re-sync the plugin to pick up the new skill.

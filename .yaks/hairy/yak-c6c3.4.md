---
id: yak-c6c3.4
title: Update docs for new invocation + version unification
type: task
priority: 2
created: '2026-07-25T20:48:31Z'
updated: '2026-07-25T20:48:41Z'
depends_on:
- yak-c6c3.2
- yak-c6c3.3
---

Update CLAUDE.md (Running the script / subcommand list / invocation model), README, and the _inject_mandate() text in scripts/yaklib/commands.py to reflect: direct-CLI invocation via uvx yakshave / yaks, no slash commands, no CLAUDE_PLUGIN_ROOT. Document the unified version scheme (pyproject + marketplace.json + .codex-plugin move in lockstep). Depends on c6c3.2 and c6c3.3.

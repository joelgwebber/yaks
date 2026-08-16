---
id: yak-c6c3.4
title: Update docs for new invocation + version unification
type: task
priority: 2
created: '2026-07-25T20:48:31Z'
updated: '2026-07-25T21:23:06Z'
depends_on:
- yak-c6c3.2
- yak-c6c3.3
parent: yak-c6c3
---

Update CLAUDE.md (Running the script / subcommand list / invocation model), README, and the _inject_mandate() text in scripts/yaklib/commands.py to reflect: direct-CLI invocation via uvx yakshave / yaks, no slash commands, no CLAUDE_PLUGIN_ROOT. Document the unified version scheme (pyproject + marketplace.json + .codex-plugin move in lockstep). Depends on c6c3.2 and c6c3.3.

---
▸ 2026-07-25T21:23:06Z
Done. README: rewrote install (uv tool install yakherder / uvx yakherder + git fallback), quick start, commands table, filtering examples, and requirements to direct-CLI; removed all /yaks:* slash-command refs. CLAUDE.md: updated intro/architecture (removed stale commands/ + CLAUDE_PLUGIN_ROOT bullets, added yak-tracker/pyproject/publish.yml), Running section (dev vs installed vs uvx), Releasing (3-manifest lockstep incl pyproject + PyPI tag/trusted-publishing), and the self-mandate. Code: model.py 'no .yaks' hint and commands.py _YAKS_MANDATE now say 'yaks next'/'yaks init'; updated the mandate idempotency guard accordingly. Repo-wide scan clean; 117 tests pass.

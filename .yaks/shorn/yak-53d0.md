---
id: yak-53d0
title: Easily invokable CLI interfaces
type: feature
priority: 1
created: '2026-04-17T12:02:00Z'
updated: '2026-04-17T12:38:15Z'
commit: b6b5b4b
---

We need to be able to invoke all the yaks commands from a `yaks` CLI interface, that matches the skill interface used by agents.

We also need a way to invoke the `yaks-tui` from the command-line without having to know where the Cluade plugin's installed. Maybe there's a simple way to do this in the python/uv-verse?

### 2026-04-17T12:38:15Z
Added [project.scripts] entry point in pyproject.toml pointing to yaklib.cli:main. Created scripts/yaklib/cli.py as the installed entry point. Changed yak.py to import tui directly instead of execvp. Made sys.path hacks conditional in both yak.py and tui.py so both direct invocation (plugin) and installed package paths work. Users install with `uv tool install git+https://github.com/joelgwebber/yaks` to get `yaks` on PATH.

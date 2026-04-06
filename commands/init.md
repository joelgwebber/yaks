---
description: "Initialize task tracking in the current project"
argument-hint: "[--prefix PREFIX] [--agents]"
allowed-tools:
  - Bash
---

Run the following command to initialize task tracking:

```
python3 ${CLAUDE_PLUGIN_ROOT}/scripts/yak.py init $ARGUMENTS
```

This creates the `.yaks/` directory structure and appends a yaks workflow mandate to `CLAUDE.md` (or `AGENTS.md` if one exists). Use `--agents` to force writing to `AGENTS.md`. After running, confirm the directory was created and show the user the injected guidance.

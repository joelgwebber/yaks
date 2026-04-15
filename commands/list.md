---
description: "List tasks with optional filters"
argument-hint: "[--status S] [--type T] [--priority P] [--label L] [--search Q] [--ready] [--tangled] [--parent-of ID] [--json]"
allowed-tools:
  - Bash
---

Run the following command to list tasks:

```
python3 ${CLAUDE_PLUGIN_ROOT}/scripts/yak.py list $ARGUMENTS
```

Show the output directly to the user. If the user asks to filter, map their request to the appropriate flags.

Filter flags (AND across dimensions, OR within a repeatable one):
- `--status S` — status (repeatable): hairy, shaving, shorn, dead
- `--type T` — yak type (repeatable): bug, feature, task, idea
- `--priority P` — priority (repeatable): 1, 2, 3
- `--label L` — match any listed label (repeatable)
- `--search Q` — substring across title/description/id
- `--ready` — only tasks whose deps are all resolved
- `--tangled` — only tasks with at least one unresolved dep
- `--parent-of ID` — only descendants of ID

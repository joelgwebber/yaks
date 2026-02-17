---
description: "List tasks with optional filters"
argument-hint: "[--status open|closed] [--type TYPE] [--priority P] [--label L] [--json]"
allowed-tools:
  - Bash
---

Run the following command to list tasks:

```
python3 ${CLAUDE_PLUGIN_ROOT}/scripts/yak.py list $ARGUMENTS
```

Show the output directly to the user. If the user asks to filter, map their request to the appropriate flags.

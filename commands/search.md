---
description: "Search tasks by keyword"
argument-hint: "<query> [--status S] [--type T] [--priority P] [--label L] [--ready] [--tangled] [--parent-of ID] [--json]"
allowed-tools:
  - Bash
---

Run the following command to search tasks:

```
python3 ${CLAUDE_PLUGIN_ROOT}/scripts/yak.py search $ARGUMENTS
```

Show the output directly to the user.

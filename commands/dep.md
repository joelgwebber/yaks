---
description: "Manage task dependencies"
argument-hint: "add|remove TASK_ID DEP_ID"
allowed-tools:
  - Bash
---

Run the following command to manage dependencies:

```
python3 ${CLAUDE_PLUGIN_ROOT}/scripts/yak.py dep $ARGUMENTS
```

- `add TASK_ID DEP_ID` — TASK_ID now depends on DEP_ID (TASK_ID is blocked until DEP_ID is closed)
- `remove TASK_ID DEP_ID` — remove that dependency

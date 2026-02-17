---
description: "Reopen a closed task"
argument-hint: "TASK_ID"
allowed-tools:
  - Bash
---

Run the following command to reopen a task:

```
python3 ${CLAUDE_PLUGIN_ROOT}/scripts/yak.py reopen $ARGUMENTS
```

This moves the task YAML file from `closed/` back to `open/`.

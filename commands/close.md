---
description: "Close a completed task"
argument-hint: "TASK_ID"
allowed-tools:
  - Bash
---

Run the following command to close a task:

```
python3 ${CLAUDE_PLUGIN_ROOT}/scripts/yak.py close $ARGUMENTS
```

This moves the task YAML file from `open/` to `closed/`.

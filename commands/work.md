---
description: "Start working on a task"
argument-hint: "TASK_ID"
allowed-tools:
  - Bash
---

Run the following command to start working on a task:

```
python3 ${CLAUDE_PLUGIN_ROOT}/scripts/yak.py work $ARGUMENTS
```

This moves the task YAML file from `open/` to `working/`.

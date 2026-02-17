---
description: "Mark a yak as shorn"
argument-hint: "TASK_ID"
allowed-tools:
  - Bash
---

Run the following command to mark a yak as shorn:

```
python3 ${CLAUDE_PLUGIN_ROOT}/scripts/yak.py shorn $ARGUMENTS
```

This moves the task YAML file to `shorn/`.

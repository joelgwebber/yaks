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

This moves the task file to `shorn/`. When using git, prefer to stage the file move alongside the code changes that completed the yak and commit them together.

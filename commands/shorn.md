---
description: "Mark a yak as shorn"
argument-hint: "TASK_ID [--commit HASH]"
allowed-tools:
  - Bash
---

Run the following command to mark a yak as shorn:

```
python3 ${CLAUDE_PLUGIN_ROOT}/scripts/yak.py shorn $ARGUMENTS
```

This moves the task file to `shorn/` and records the current `git HEAD` as the `commit` field — this captures the work commit, not the commit that includes the yak file itself. Do not go back and update the hash after committing.

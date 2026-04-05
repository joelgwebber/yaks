---
description: "Revive a dead yak (move back to hairy)"
argument-hint: "TASK_ID"
allowed-tools:
  - Bash
---

Run the following command to revive a dead yak:

```
python3 ${CLAUDE_PLUGIN_ROOT}/scripts/yak.py revive $ARGUMENTS
```

This moves the task file from `.yaks/dead/` back to `.yaks/hairy/`. Use when a slaughtered idea turns out to be worth pursuing after all.

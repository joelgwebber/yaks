---
description: "Regrow a shorn yak"
argument-hint: "TASK_ID"
allowed-tools:
  - Bash
---

Run the following command to regrow a yak:

```
python3 ${CLAUDE_PLUGIN_ROOT}/scripts/yak.py regrow $ARGUMENTS
```

This moves the task YAML file back to `hairy/`.

---
description: "Show tasks ready to work on (all dependencies met)"
argument-hint: "[--json]"
allowed-tools:
  - Bash
---

Run the following command to show ready tasks:

```
python3 ${CLAUDE_PLUGIN_ROOT}/scripts/yak.py ready $ARGUMENTS
```

These are open tasks whose dependencies are all closed (or that have no dependencies).

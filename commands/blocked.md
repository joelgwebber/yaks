---
description: "Show tasks blocked by open dependencies"
argument-hint: "[--json]"
allowed-tools:
  - Bash
---

Run the following command to show blocked tasks:

```
python3 ${CLAUDE_PLUGIN_ROOT}/scripts/yak.py blocked $ARGUMENTS
```

These are open tasks that have at least one open (unresolved) dependency.

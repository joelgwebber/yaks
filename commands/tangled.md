---
description: "Show tangled yaks (blocked by unshorn dependencies)"
argument-hint: "[--type T] [--priority P] [--label L] [--search Q] [--parent-of ID] [--json]"
allowed-tools:
  - Bash
---

Run the following command to show tangled yaks:

```
python3 ${CLAUDE_PLUGIN_ROOT}/scripts/yak.py tangled $ARGUMENTS
```

These are hairy yaks that have at least one unshorn dependency.

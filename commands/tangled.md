---
description: "Show tangled yaks (blocked by unshorn dependencies)"
argument-hint: "[--json]"
allowed-tools:
  - Bash
---

Run the following command to show tangled yaks:

```
python3 ${CLAUDE_PLUGIN_ROOT}/scripts/yak.py tangled $ARGUMENTS
```

These are hairy yaks that have at least one unshorn dependency.

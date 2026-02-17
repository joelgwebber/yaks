---
description: "Initialize task tracking in the current project"
argument-hint: "[--prefix PREFIX]"
allowed-tools:
  - Bash
---

Run the following command to initialize task tracking:

```
python3 ${CLAUDE_PLUGIN_ROOT}/scripts/yak.py init $ARGUMENTS
```

After running, confirm the `.yaks/` directory was created and show the user the config.

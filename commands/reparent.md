---
description: "Move a task to a new parent or promote to top-level"
argument-hint: "TASK_ID (--parent PARENT_ID | --unparent)"
allowed-tools:
  - Bash
---

Run the following command to reparent a task:

```
python3 ${CLAUDE_PLUGIN_ROOT}/scripts/yak.py reparent $ARGUMENTS
```

Use `--parent TASK_ID` to move a task under a new parent (assigns the next available child number). Use `--unparent` to promote a child task to a top-level task (generates a new ID). All descendants are renamed recursively and dependency references across the entire `.yaks/` tree are updated.

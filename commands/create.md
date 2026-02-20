---
description: "Create a new task"
argument-hint: "--title TITLE [--type TYPE] [--priority P] [--description DESC] [--labels L ...] [--depends-on ID ...] [--parent ID]"
allowed-tools:
  - Bash
---

Run the following command to create a new task:

```
python3 ${CLAUDE_PLUGIN_ROOT}/scripts/yak.py create $ARGUMENTS
```

If the user provided a natural language request instead of flags, extract the appropriate flags from their message. At minimum `--title` is required. Use `--type` for bug/feature/task, `--priority` for 1-3 (1=highest), `--description` for longer details, and `--parent TASK_ID` to create a child task (the child ID will be `PARENT_ID.N`).

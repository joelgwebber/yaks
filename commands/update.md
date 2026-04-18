---
description: "Update a task's fields"
argument-hint: "TASK_ID [--title T] [--type T] [--priority P] [--description D] [--note TEXT] [--add-label L ...] [--remove-label L ...] [--source URL]"
allowed-tools:
  - Bash
---

Run the following command to update a task:

```
python3 ${CLAUDE_PLUGIN_ROOT}/scripts/yak.py update $ARGUMENTS
```

If the user provides a natural language update request, extract the appropriate flags. The task ID is required as the first positional argument.

Use `--note "text"` to append a timestamped progress note to the task's description body (rather than replacing it). This is the preferred way to log progress while working on a yak.

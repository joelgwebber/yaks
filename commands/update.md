---
description: "Update a task's fields"
argument-hint: "TASK_ID [--title T] [--type T] [--priority P] [--description D] [--add-label L ...] [--remove-label L ...]"
allowed-tools:
  - Bash
---

Run the following command to update a task:

```
python3 ${CLAUDE_PLUGIN_ROOT}/scripts/yak.py update $ARGUMENTS
```

If the user provides a natural language update request, extract the appropriate flags. The task ID is required as the first positional argument.

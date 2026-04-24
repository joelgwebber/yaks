---
description: "Sync a yak bidirectionally with its external issue tracker"
argument-hint: "TASK_ID"
allowed-tools:
  - Bash
  - Read
  - Edit
  - Write
---

Sync a single yak with its external issue (Jira / Linear / GitHub / etc.),
merging changes in both directions with user confirmation on anything
ambiguous.

First, load the yak so you can see what's there and whether it has an
external `source:`:

```
python3 ${CLAUDE_PLUGIN_ROOT}/scripts/yak.py show $ARGUMENTS
```

Then follow the **yak-sync** skill. It walks you through the full diff
workflow, tracker-specific hints, and the rules for handling notes,
attachments, and missing-upstream cases. Do not skip the prompts it
describes — bidirectional sync without confirmation turns into data loss
quickly.

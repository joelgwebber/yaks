---
description: "Slaughter a yak (move to hidden 'dead' state)"
argument-hint: "TASK_ID"
allowed-tools:
  - Bash
---

Run the following command to slaughter a yak:

```
python3 ${CLAUDE_PLUGIN_ROOT}/scripts/yak.py slaughter $ARGUMENTS
```

This moves the task file to `.yaks/dead/`. Dead yaks are hidden from every default query and don't appear in the TUI, but they remain on disk for history. Use this for ideas you won't pursue and tasks that have been obviated. Tasks that depended on a slaughtered yak are automatically unblocked (dead deps count as resolved).

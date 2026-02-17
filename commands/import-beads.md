---
description: "Import tasks from a beads issues.jsonl file"
argument-hint: "[--file PATH] [--dry-run]"
allowed-tools:
  - Bash
---

Run the following command to import tasks from a beads JSONL file:

```
python3 ${CLAUDE_PLUGIN_ROOT}/scripts/yak.py import-beads $ARGUMENTS
```

If the user doesn't specify `--file`, the command will auto-detect `.beads/issues.jsonl` by walking up from the current directory. Use `--dry-run` first to preview what would be imported. The import is idempotent — re-running skips tasks that already exist.

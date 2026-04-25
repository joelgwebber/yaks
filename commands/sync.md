---
description: "Plan or apply a bidirectional sync between a yak and its external issue tracker"
argument-hint: "TASK_ID"
allowed-tools:
  - Bash
  - Read
  - Edit
  - Write
---

Sync a single yak with its external issue (Jira / Linear / GitHub / etc.) using the **plan / apply** model: a sidecar at `.yaks/.sync-pending/<yak-id>.yaml` captures every proposed change, the user reviews/edits it, and apply re-fetches upstream + verifies the snapshot before touching anything.

First, check whether a sidecar already exists for this yak:

```
python3 ${CLAUDE_PLUGIN_ROOT}/scripts/yak.py sync show $ARGUMENTS 2>/dev/null
```

- **No sidecar** → run the **plan phase** from the yak-sync skill: load the yak, fetch upstream, compute the diff, write the sidecar. No mutations.
- **Sidecar exists** → run the **apply phase**: re-fetch upstream, verify the snapshot, apply each item per its resolution, stamp `last_synced`, clear the sidecar.

Either way: follow the yak-sync skill — it has the schema, per-field policies, tracker-specific hints, the snapshot-drift abort path, and the "suppress remaining drift?" prompt for partial-apply outcomes. Do not skip its safety guards; bidirectional sync without confirmation turns into data loss quickly.

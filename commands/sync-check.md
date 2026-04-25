---
description: "Sweep all source-linked yaks and report which ones might need syncing"
argument-hint: "[--tracker jira|linear|github]"
allowed-tools:
  - Bash
---

Identify which yaks may have drifted from their external issue trackers since the last sync. Read-only — no plans written, no upstream mutations.

First, enumerate the source-linked yaks:

```
python3 ${CLAUDE_PLUGIN_ROOT}/scripts/yak.py sync check --json $ARGUMENTS
```

Then follow the **yak-sync** skill's *Sweep / drift check* section: group the result by tracker, batch-fetch upstream `updated` timestamps via the tracker's MCP (one JQL call per Jira instance, one GraphQL call per Linear team, etc.), compare against each yak's `last_synced`, and report the drifted set as a concise table.

Drift categories worth reporting:

- **upstream-newer** — `upstream.updated > yak.last_synced`. Real change you haven't seen.
- **local-newer** — `yak.updated > yak.last_synced`. You edited locally and haven't pushed.
- **both** — both predicates fire.
- **pending sidecar** — already has a sidecar at `.yaks/.sync-pending/<id>.yaml`; needs apply or discard, not re-plan.

Do not auto-plan. The whole point of this command is to surface drift, not to act on it. The user picks specific yaks to sync next.

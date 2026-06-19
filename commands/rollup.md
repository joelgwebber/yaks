---
description: "Group yaks by the external issue they roll up to"
argument-hint: "[--label X] [--status S] [--parent-of ID] [--keys] [--json]"
allowed-tools:
  - Bash
---

Run the following command to roll yaks up to the external issues they point at:

```
python3 ${CLAUDE_PLUGIN_ROOT}/scripts/yak.py rollup $ARGUMENTS
```

This is the read-only projection view: yaks are grouped by their `source:` URL
(a yak with no `source:` inherits the nearest ancestor's), so you can see which
external issues (Jira / Linear / GitHub) a set of yaks maps to. Many yaks
typically roll up to few external issues.

- Filter the scope with the usual flags, e.g. `--label pr-369`, `--status shaving`,
  `--parent-of yak-abcd`.
- `--keys` prints just the distinct external keys (e.g. `SUBTEXT-369`) — paste
  these into a PR description so the forge links the PR to the issue natively.
- `--json` for machine-readable output.

Nothing is written to the external tracker; this only reads local task files.

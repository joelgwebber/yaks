---
activation:
  - multi-session work detected
  - .yaks/ directory exists in the project
---

# Yaks — Filesystem-native task tracker

You have access to a task tracker that stores tasks as plain YAML files in `.yaks/` within the project. Use it to track work across sessions, manage dependencies, and prioritize what to shave next.

## Available commands

| Command | What it does |
|---------|-------------|
| `/yaks:init` | Initialize `.yaks/` in the current project |
| `/yaks:create` | Create a new task |
| `/yaks:list` | List tasks with optional filters |
| `/yaks:show` | Show full details of a task |
| `/yaks:update` | Update a task's fields |
| `/yaks:shave` | Start shaving a yak (move to shaving) |
| `/yaks:shorn` | Mark a yak as shorn |
| `/yaks:regrow` | Regrow a shorn yak |
| `/yaks:next` | Show yaks ready to shave (all deps met) |
| `/yaks:tangled` | Show tangled yaks (unshorn dependencies) |
| `/yaks:dep` | Add/remove dependencies between tasks |
| `/yaks:stats` | Show task statistics |
| `/yaks:import-beads` | Import tasks from a beads JSONL export |

## Task format

Tasks are YAML files stored in `.yaks/hairy/`, `.yaks/shaving/`, or `.yaks/shorn/`. Status is implicit from the directory — no status field in the file.

```yaml
id: yak-a1b2
title: Fix the login crash
type: bug
priority: 2
created: "2026-02-16T10:00:00Z"
updated: "2026-02-16T10:30:00Z"
depends_on:
  - yak-c3d4
labels:
  - auth
commit: a1b2c3d          # added when shorn; git HEAD by default
description: |
  Details go here. Git tracks the history.
```

## Workflow tips

- Before starting work, run `/yaks:next` to see what's ready to shave
- When picking up a task, run `/yaks:shave TASK_ID` to mark it in progress
- After finishing, run `/yaks:shorn TASK_ID` — this auto-records the git commit hash (override with `--commit HASH`)
- Use `/yaks:dep add` to express "do X before Y" relationships
- Use `/yaks:tangled` to see what's waiting on other work

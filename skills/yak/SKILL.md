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
| `/yak:init` | Initialize `.yaks/` in the current project |
| `/yak:create` | Create a new task |
| `/yak:list` | List tasks with optional filters |
| `/yak:show` | Show full details of a task |
| `/yak:update` | Update a task's fields |
| `/yak:shave` | Start shaving a yak (move to shaving) |
| `/yak:shorn` | Mark a yak as shorn |
| `/yak:regrow` | Regrow a shorn yak |
| `/yak:next` | Show yaks ready to shave (all deps met) |
| `/yak:tangled` | Show tangled yaks (unshorn dependencies) |
| `/yak:dep` | Add/remove dependencies between tasks |
| `/yak:stats` | Show task statistics |
| `/yak:import-beads` | Import tasks from a beads JSONL export |

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
description: |
  Details go here. Git tracks the history.
```

## Workflow tips

- Before starting work, run `/yak:next` to see what's ready to shave
- When picking up a task, run `/yak:shave TASK_ID` to mark it in progress
- After finishing, run `/yak:shorn TASK_ID`
- Use `/yak:dep add` to express "do X before Y" relationships
- Use `/yak:tangled` to see what's waiting on other work

---
activation:
  - .yaks/ directory exists in the project
---

# Yaks — Task tracking workflow

This project tracks work with Yaks. Tasks are plain YAML files in `.yaks/`. You MUST follow this workflow to keep task state accurate.

## Required workflow

**At the start of every session**, run `/yaks:list` to see what exists and `/yaks:next` to see what's ready.

**Before starting any significant work:**

1. Check if a yak already exists for the work (`/yaks:list`).
2. If not, create one with `/yaks:create`.
3. Run `/yaks:shave TASK_ID` to mark it in progress.

**After completing work:**

1. Run `/yaks:shorn TASK_ID` — this auto-records the git commit hash.

Do not skip these steps. If you finish work without marking the yak shorn, or start work without shaving a yak, the tracker becomes stale and loses its value.

## Commands

| Command | What it does |
|---------|-------------|
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

## Task format

Tasks live in `.yaks/hairy/`, `.yaks/shaving/`, or `.yaks/shorn/`. Status is implicit from the directory.

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
  Details go here.
```

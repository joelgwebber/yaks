---
activation:
  - .yaks/ directory exists in the project
---

# Yaks — Task tracking workflow

This project tracks work with Yaks. Tasks are markdown files with YAML frontmatter in `.yaks/`. You MUST follow this workflow to keep task state accurate.

## Required workflow

Every piece of work MUST be bracketed by shave/shorn. No exceptions.

1. **Session start** — run `/yaks:list` and `/yaks:next` to see current state.
2. **Before writing code** — `/yaks:shave TASK_ID` (create the yak first if needed).
3. **After the commit** — `/yaks:shorn TASK_ID` immediately. Do this right after committing, not later.

If you are about to write code and haven't shaved a yak, stop and shave one first. If you just committed and haven't marked a yak shorn, do it now before moving on.

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
| `/yaks:search` | Search tasks by keyword |
| `/yaks:dep` | Add/remove dependencies between tasks |
| `/yaks:reparent` | Move task to new parent or top-level |
| `/yaks:stats` | Show task statistics |

## Task format

Tasks live in `.yaks/hairy/`, `.yaks/shaving/`, or `.yaks/shorn/` as `.md` files. Status is implicit from the directory. Metadata is YAML frontmatter; the markdown body is the description.

```markdown
---
id: yak-a1b2              # or yak-a1b2.1 for a child task
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
---

Details go here.
```

Child tasks use `--parent TASK_ID` on create. The hierarchy is implicit from the ID (dot-suffixed integers). `/yaks:show` displays parent and children automatically.

---
activation:
  - .yaks/ directory exists in the project
---

# Yaks — Task tracking workflow

This project tracks work with Yaks. Tasks are markdown files with YAML frontmatter in `.yaks/`. You MUST follow this workflow to keep task state accurate.

## Hard rules

**NEVER write code without an active shaving yak.** Before touching any code — even a one-line fix — you must have a yak in shaving state. If you don't, stop and `/yaks:shave` one first (create it if needed). No exceptions.

**ALWAYS shorn immediately after committing.** Run `/yaks:shorn TASK_ID` right after the commit, before doing anything else. The `commit` field auto-captures the current HEAD (the work commit). Do not amend or update it — the shorn yak file gets included in the next commit as-is.

## Workflow

1. **Session start** — run `/yaks:list` and `/yaks:next` to see current state.
2. **Before writing code** — `/yaks:shave TASK_ID` (create the yak first if needed).
3. **After the commit** — `/yaks:shorn TASK_ID` immediately.

## Parent/child state rules

A parent yak's state should reflect its children:
- When you shave a child, shave the parent too (if it's still hairy).
- When you shorn the last unshorn child, shorn the parent too.
- NEVER leave a hairy parent with shorn children — that means work was done but the parent doesn't reflect it.

## Commands

**Always use the Skill tool to invoke these commands.** Do not run `yak.py` directly via Bash — the slash commands handle tool permissions and argument passing automatically.

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

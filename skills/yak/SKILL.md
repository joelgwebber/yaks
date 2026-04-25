---
activation:
  - .yaks/ directory exists in the project
---

# Yaks — Task tracking workflow

This project tracks work with Yaks. Tasks are markdown files with YAML frontmatter in `.yaks/`. You MUST follow this workflow to keep task state accurate.

## Hard rules

**NEVER write code without an active shaving yak.** Before touching any code — even a one-line fix — you must have a yak in shaving state. If you don't, stop and `/yaks:shave` one first (create it if needed). No exceptions.

**ALWAYS shear when a yak's work is done.** Run `/yaks:shorn TASK_ID` as soon as the task is complete. When the project uses git, prefer to stage the shorn yak file alongside the code changes that completed it and commit them together rather than as a separate commit.

## Workflow

1. **Session start** — run `/yaks:list` and `/yaks:next` to see current state.
2. **Before writing code** — `/yaks:shave TASK_ID` (create the yak first if needed).
3. **While working** — append progress notes with `/yaks:update TASK_ID --note "what you found / decided / changed"`. This builds a running log in the markdown body so future sessions have context.
4. **When the work is done** — `/yaks:update TASK_ID --note "..."` with a brief shorn summary (what was done, what was learned, any yaks spawned), then `/yaks:shorn TASK_ID`. If using git, stage the shorn yak move together with the code changes and commit them in one commit whenever practical.

## Parent/child state rules

A parent yak's state should reflect its children:
- When you shave a child, shave the parent too (if it's still hairy).
- When you shear the last unshear child, shear the parent too.
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
| `/yaks:slaughter` | Slaughter a yak (hide in `.yaks/dead/`) — for ideas you won't pursue or tasks that have been obviated |
| `/yaks:revive` | Revive a dead yak back to hairy |
| `/yaks:next` | Shortcut for `list --status hairy --ready` |
| `/yaks:tangled` | Shortcut for `list --status hairy --tangled` |
| `/yaks:search` | Shortcut for `list --search QUERY` |
| `/yaks:dep` | Add/remove dependencies between tasks |
| `/yaks:reparent` | Move task to new parent or top-level |
| `/yaks:stats` | Show task statistics |

## Task format

Tasks live in `.yaks/hairy/`, `.yaks/shaving/`, or `.yaks/shorn/` as `.md` files. Slaughtered tasks live in `.yaks/dead/` and are excluded from every default query — you can still grep that directory or pass `--status dead` to `list` to find them. Status is implicit from the directory. Metadata is YAML frontmatter; the markdown body is the description.

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
source: https://jira.example.com/browse/PROJ-123  # optional external issue URL
last_synced: "2026-02-16T10:30:00Z"               # written by /yaks:sync after a successful merge
---

Details go here.
```

Child tasks use `--parent TASK_ID` on create. The hierarchy is implicit from the ID (dot-suffixed integers). `/yaks:show` displays parent and children automatically.

### External source linking

Use `--source URL` on create or update to link a yak to an external issue (Jira, GitHub Issues, Linear, etc.). The URL is stored in the `source` frontmatter field. When a yak has a source, the agent should check the external system for context when starting work, and update it when the yak is shorn.

The companion `/yaks:sync TASK_ID` command (see the **yak-sync** skill) performs bidirectional merge with the linked external issue, and stamps `last_synced` on the yak after a successful merge so future syncs can short-circuit when nothing has drifted.

## Filtering

Every query command (`list`, `search`, `next`, `tangled`) shares the same filter flags. AND across dimensions; within a repeatable flag, OR:

- `--status S` / `--type T` / `--priority P` / `--label L` (all repeatable)
- `--search Q` — substring match on title/description/id
- `--ready` / `--tangled` — dep-state filters
- `--parent-of ID` — only descendants of ID

Examples:
- `list --type bug --type feature --priority 1` — urgent bugs or features
- `list --label auth --search retry` — auth-labeled tasks mentioning "retry"
- `next --type bug` — ready bugs only

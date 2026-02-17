# Yaks

Filesystem-native task tracker for AI coding agents. Plain YAML files, no database, no daemon.

Yaks is a [Claude Code plugin](https://docs.anthropic.com/en/docs/claude-code/plugins) that gives your coding assistant persistent task tracking across sessions. Tasks are stored as YAML files in a `.yaks/` directory within your project — readable, diffable, and version-controlled alongside your code.

## Install

```
claude plugin add --from /path/to/yaks
```

Or, if published to a registry, follow the standard plugin installation instructions.

## Quick start

Once installed, initialize tracking in any project:

```
/yak:init
```

This creates a `.yaks/` directory with `open/`, `working/`, and `closed/` subdirectories, plus a `config.yaml`. From there, your coding assistant can create and manage tasks using slash commands:

```
/yak:create --title "Add retry logic to API client" --type feature --priority 1
/yak:list
/yak:ready
/yak:work yak-a1b2
/yak:close yak-a1b2
```

## How it works

- **Status is a directory.** A task in `.yaks/open/` is open. Move it to `.yaks/working/` and it's in progress. Move it to `.yaks/closed/` and it's done. No status field in the YAML — the filesystem is the source of truth.
- **Tasks are plain YAML.** Every task is a single `.yaml` file with an ID, title, type, priority, timestamps, optional dependencies, labels, and description.
- **Dependencies are first-class.** Tasks can depend on other tasks. `/yak:ready` shows only tasks whose dependencies are all closed. `/yak:blocked` shows what's stuck.
- **Git-friendly.** Task files are small, human-readable, and merge cleanly. Git history is your audit log.

## Commands

| Command | Description |
|---------|-------------|
| `/yak:init` | Initialize `.yaks/` in the current project |
| `/yak:create` | Create a new task |
| `/yak:list` | List tasks with optional filters |
| `/yak:show` | Show full details of a task |
| `/yak:update` | Update a task's fields |
| `/yak:work` | Move a task to working status |
| `/yak:close` | Close a completed task |
| `/yak:reopen` | Reopen a closed task |
| `/yak:ready` | Show tasks ready to work on (all deps met) |
| `/yak:blocked` | Show tasks blocked by open dependencies |
| `/yak:dep` | Add or remove dependencies between tasks |
| `/yak:stats` | Show task statistics |

## Task format

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
  Users see a crash on the login screen when
  submitting with an empty password field.
```

Fields:

- **id** — Auto-generated as `{prefix}-{4 hex chars}` (prefix is configurable)
- **title** — Short description of the task
- **type** — `bug`, `feature`, or `task`
- **priority** — `1` (highest) through `3` (lowest)
- **created** / **updated** — ISO 8601 timestamps
- **depends_on** — Optional list of task IDs that must be closed first
- **labels** — Optional list of string tags
- **description** — Optional longer description (block scalar)

## Configuring your AI assistant to use Yaks

Add the following to your project's `CLAUDE.md` (or `AGENTS.md` for other tools) so that your coding assistant knows to use Yaks for task management:

```markdown
## Task tracking

This project uses Yaks for task tracking. Tasks are stored as YAML files in `.yaks/`.

When working on multi-step or multi-session work:

- Run `/yak:list` at the start of a session to see current tasks.
- Run `/yak:ready` to find unblocked work.
- Before starting a task, run `/yak:work TASK_ID` to mark it in progress.
- After completing a task, run `/yak:close TASK_ID`.
- When planning work, use `/yak:create` to break it into trackable tasks with dependencies.
- Use `/yak:dep add TASK_ID DEP_ID` to express ordering constraints.
- Prefer checking `/yak:blocked` before starting new work to avoid picking up tasks with unmet dependencies.
```

### Custom prefix

If you want task IDs to match your project (e.g., `api-f3a1` instead of `yak-f3a1`), initialize with a custom prefix:

```
/yak:init --prefix api
```

## Requirements

- Python 3.10+
- PyYAML (`pyyaml>=6.0`)

The script uses [PEP 723](https://peps.python.org/pep-0723/) inline metadata, so package managers like `uv` can run it directly without manual dependency installation.

## License

MIT

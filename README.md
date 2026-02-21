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
/yaks:init
```

This creates a `.yaks/` directory with `hairy/`, `shaving/`, and `shorn/` subdirectories, plus a `config.yaml`. From there, your coding assistant can create and manage tasks using slash commands:

```
/yaks:create --title "Add retry logic to API client" --type feature --priority 1
/yaks:list
/yaks:next
/yaks:shave yak-a1b2
/yaks:shorn yak-a1b2
```

## How it works

- **Status is a directory.** A task in `.yaks/hairy/` needs shaving. Move it to `.yaks/shaving/` and it's in progress. Move it to `.yaks/shorn/` and it's done. No status field in the YAML — the filesystem is the source of truth.
- **Tasks are plain YAML.** Every task is a single `.yaml` file with an ID, title, type, priority, timestamps, optional dependencies, labels, and description.
- **Parent/child tasks.** Create subtasks with `--parent TASK_ID`. Children get dot-suffixed IDs (`yak-a1b2.1`, `yak-a1b2.2`). The relationship is implicit from the ID — no extra YAML field. `show` displays the hierarchy automatically.
- **Dependencies are first-class.** Tasks can depend on other tasks. `/yaks:next` shows only tasks whose dependencies are all shorn. `/yaks:tangled` shows what's stuck.
- **Git-friendly.** Task files are small, human-readable, and merge cleanly. Git history is your audit log.

## Commands

| Command | Description |
|---------|-------------|
| `/yaks:init` | Initialize `.yaks/` in the current project |
| `/yaks:create` | Create a new task |
| `/yaks:list` | List tasks with optional filters |
| `/yaks:show` | Show full details of a task |
| `/yaks:update` | Update a task's fields |
| `/yaks:shave` | Start shaving a yak |
| `/yaks:shorn` | Mark a yak as shorn |
| `/yaks:regrow` | Regrow a shorn yak |
| `/yaks:next` | Show yaks ready to shave (all deps met) |
| `/yaks:tangled` | Show tangled yaks (unshorn dependencies) |
| `/yaks:dep` | Add or remove dependencies between tasks |
| `/yaks:reparent` | Move a task to a new parent or promote to top-level |
| `/yaks:stats` | Show task statistics |
| `/yaks:import-beads` | Import tasks from a beads JSONL export |

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
commit: a1b2c3d
description: |
  Users see a crash on the login screen when
  submitting with an empty password field.
```

Fields:

- **id** — Auto-generated as `{prefix}-{4 hex chars}` (prefix defaults to directory name), or `{parent-id}.N` for child tasks
- **title** — Short description of the task
- **type** — `bug`, `feature`, or `task`
- **priority** — `1` (highest) through `3` (lowest)
- **created** / **updated** — ISO 8601 timestamps
- **depends_on** — Optional list of task IDs that must be shorn first
- **labels** — Optional list of string tags
- **commit** — Short git hash, auto-populated from HEAD when shorn (override with `--commit`)
- **description** — Optional longer description (block scalar)

## Configuring your AI assistant to use Yaks

Once `.yaks/` exists in a project, the Yaks plugin skill activates automatically and instructs Claude to follow the tracking workflow. No additional `CLAUDE.md` configuration is required.

If you want to reinforce the behavior, you can add a brief note to your project's `CLAUDE.md`:

```markdown
## Task tracking

This project uses Yaks to track work. The Yaks skill (activated by the `.yaks/` directory) defines the required workflow — follow it.
```

### Custom prefix

If you want task IDs to match your project (e.g., `api-f3a1` instead of the default), initialize with a custom prefix:

```
/yaks:init --prefix api
```

## Requirements

- Python 3.10+
- PyYAML (`pyyaml>=6.0`)

The script uses [PEP 723](https://peps.python.org/pep-0723/) inline metadata, so package managers like `uv` can run it directly without manual dependency installation.

## License

MIT

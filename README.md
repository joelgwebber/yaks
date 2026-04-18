# Yaks

Filesystem-native task tracker for AI coding agents. Markdown files with YAML frontmatter, no database, no daemon.

Yaks is a [Claude Code plugin](https://docs.anthropic.com/en/docs/claude-code/plugins) that gives your coding assistant persistent task tracking across sessions. Tasks are stored as markdown files (with YAML frontmatter for metadata) in a `.yaks/` directory within your project — readable, diffable, and version-controlled alongside your code.

## Install

### As a Claude Code plugin

```
claude plugin add --from /path/to/yaks
```

### As a CLI tool

```bash
# From GitHub
uv tool install git+https://github.com/joelgwebber/yaks

# Or from a local clone
uv tool install /path/to/yaks
```

This puts `yaks` on your `$PATH`. All subcommands work: `yaks list`, `yaks create`, `yaks tui`, etc. Upgrade with `uv tool upgrade yaks`.

## Quick start

Once installed, initialize tracking in any project:

```
/yaks:init
```

This creates a `.yaks/` directory with `hairy/`, `shaving/`, and `shorn/` subdirectories, plus a `config.yaml`. It also appends a workflow mandate to your project's `CLAUDE.md` (or `AGENTS.md` if one exists).

From there, your coding assistant can create and manage tasks using slash commands:

```
/yaks:create --title "Add retry logic to API client" --type feature --priority 1
/yaks:list
/yaks:next
/yaks:shave yak-a1b2
/yaks:shorn yak-a1b2
```

## How it works

- **Status is a directory.** A task in `.yaks/hairy/` needs shaving. Move it to `.yaks/shaving/` and it's in progress. Move it to `.yaks/shorn/` and it's done. No status field in the YAML — the filesystem is the source of truth.
- **Tasks are markdown with frontmatter.** Every task is a single `.md` file. Structured metadata (ID, title, type, priority, timestamps, dependencies, labels) lives in YAML frontmatter. The markdown body is the description.
- **Parent/child tasks.** Create subtasks with `--parent TASK_ID`. Children get dot-suffixed IDs (`yak-a1b2.1`, `yak-a1b2.2`). The relationship is implicit from the ID — no extra YAML field. `show` displays the hierarchy automatically.
- **Dependencies are first-class.** Tasks can depend on other tasks. `/yaks:next` shows only tasks whose dependencies are all shorn. `/yaks:tangled` shows what's stuck.
- **Artifacts.** Attach files or clipboard images to tasks with `attach`. They're stored in `.yaks/artifacts/{task-id}/` and linked from the task body.
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
| `/yaks:slaughter` | Slaughter a yak (move to hidden `dead/` state) |
| `/yaks:revive` | Revive a dead yak back to hairy |
| `/yaks:next` | Show yaks ready to shave (all deps met) |
| `/yaks:tangled` | Show tangled yaks (unshorn dependencies) |
| `/yaks:search` | Search tasks by keyword |
| `/yaks:dep` | Add or remove dependencies between tasks |
| `/yaks:reparent` | Move a task to a new parent or promote to top-level |
| `/yaks:stats` | Show task statistics |
| `/yaks:import-beads` | Import tasks from a beads JSONL export |

Additionally, `attach` and `detach` are available via the CLI (`yak.py attach`/`yak.py detach`) for managing file artifacts on tasks.

## Filtering

All query commands (`list`, `search`, `next`, `tangled`) share the same filter flags. Filters AND across dimensions; within a repeatable flag, values are OR'd:

- `--status S` — filter by status (repeatable): `hairy`, `shaving`, `shorn`, `dead`
- `--type T` — filter by type (repeatable): `bug`, `feature`, `task`, `idea`
- `--priority P` — filter by priority (repeatable): `1`, `2`, `3`
- `--label L` — match any listed label (repeatable)
- `--search Q` — substring match on title, description, or id
- `--ready` — only tasks whose dependencies are all resolved
- `--tangled` — only tasks with at least one unresolved dependency
- `--parent-of ID` — only descendants of a given task

Examples:

```
/yaks:list --type bug --type feature --priority 1
/yaks:list --label auth --search retry
/yaks:next --type bug
```

## TUI

Yaks includes a curses-based terminal UI for interactive task management:

```
yaks tui
```

The TUI provides:

- **Tab-based status views** — switch between hairy, shaving, and shorn tabs
- **Tree display** — parent/child tasks rendered as a collapsible tree
- **Detail pane** — full task view with rendered markdown, metadata, dependencies, and artifacts
- **Inline mutations** — change status, priority, type, title, labels, and dependencies without leaving the TUI
- **Filter drawer** — drop-down filter panel (press `/` for search, `f` for full filter) with live preview as you type
- **Help overlay** — press `?` for a full keybinding reference
- **Vim-style editing** — optional vim keybindings in all text inputs (see Configuration below)

## Task format

Tasks are `.md` files with YAML frontmatter for metadata. The markdown body is the description.

```markdown
---
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
source: https://jira.example.com/browse/PROJ-123
---

Users see a crash on the login screen when
submitting with an empty password field.
```

Frontmatter fields:

- **id** — Auto-generated as `{prefix}-{4 hex chars}` (prefix from config), or `{parent-id}.N` for child tasks
- **title** — Short description of the task
- **type** — `bug`, `feature`, `task`, or `idea`
- **priority** — `1` (highest) through `3` (lowest)
- **created** / **updated** — ISO 8601 timestamps
- **depends_on** — Optional list of task IDs that must be shorn first
- **labels** — Optional list of string tags
- **commit** — Short git hash, auto-populated from HEAD when shorn
- **source** — Optional URL linking to an external issue (Jira, GitHub Issues, Linear, etc.)

## Configuration

Yaks reads configuration from two levels, with per-project values overriding user-global ones:

1. **User-global** — `~/.config/yaks/config.yaml` (created by `yaks init` if it doesn't exist)
2. **Per-project** — `.yaks/config.yaml`

Available settings:

| Key | Default | Description |
|-----|---------|-------------|
| `prefix` | directory name | ID prefix for new tasks (e.g., `api` → `api-f3a1`) |
| `vim_mode` | `false` | Vim-style insert/normal mode editing in TUI text inputs |

## Configuring your AI assistant to use Yaks

`/yaks:init` automatically appends a workflow mandate to your project's `CLAUDE.md` (or `AGENTS.md` if one exists). Use `--agents` to force writing to `AGENTS.md`.

If you already have `.yaks/` set up and didn't get the guidance via init, add this block to your `CLAUDE.md` (or `AGENTS.md`) manually:

```markdown
## Task tracking

This project uses Yaks. The Yaks skill has the full workflow.

1. Never start coding without a shaving yak. No exceptions.
2. Shorn immediately after committing, before anything else.
3. Check existing yaks before creating new ones.
4. Append progress notes to yak descriptions as you work.
5. When unsure what's next, run `/yaks:next` — don't freelance.
```

The Yaks plugin skill activates automatically when `.yaks/` exists and carries the full workflow details, command reference, and task format documentation. The block above is the behavioral mandate that ensures your assistant actually follows it.

## Requirements

- Python 3.10+
- PyYAML (`pyyaml>=6.0`)

The script uses [PEP 723](https://peps.python.org/pep-0723/) inline metadata, so package managers like `uv` can run it directly without manual dependency installation.

## License

Apache 2.0

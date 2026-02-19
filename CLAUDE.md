# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What is this?

Yaks is a filesystem-native task tracker distributed as a Claude Code plugin. Tasks are plain YAML files stored in `.yaks/` directories within projects — no database, no daemon. Task status is implicit from the directory the file lives in (`hairy/`, `shaving/`, `shorn/`).

## Architecture

- **`scripts/yak.py`** — The entire tracker is a single Python script (PEP 723 inline metadata, requires `pyyaml>=6.0`). It uses argparse with subcommands that map 1:1 to the commands below.
- **`commands/*.md`** — Each file defines a slash command for the Claude Code plugin. Frontmatter specifies `description`, `argument-hint`, and `allowed-tools`. The body tells Claude how to invoke `yak.py` using `${CLAUDE_PLUGIN_ROOT}/scripts/yak.py`.
- **`skills/yak/SKILL.md`** — The skill definition that activates when `.yaks/` exists or multi-session work is detected. Documents the full command set and task YAML schema.
- **`.claude-plugin/plugin.json`** and **`marketplace.json`** — Plugin and marketplace metadata.
- **`.yaks/config.yaml`** — Per-project config (currently just `prefix` for task ID generation).

## Running the script

```
python3 scripts/yak.py <subcommand> [args]
```

Subcommands: `init`, `create`, `list`, `show`, `update`, `shave`, `shorn`, `regrow`, `next`, `tangled`, `dep`, `stats`, `import-beads`. Old names (`work`, `close`, `reopen`, `ready`, `blocked`) are accepted as aliases. All support `--json` where applicable.

## Task YAML schema

```yaml
id: prefix-hex4       # e.g. yak-a1b2
title: string
type: bug | feature | task
priority: 1-3         # 1=highest
created: ISO8601
updated: ISO8601
depends_on: [task-ids] # optional
labels: [strings]      # optional
commit: short-hash     # optional, auto-populated from git HEAD when shorn
description: |         # optional, block scalar
  multiline text
```

## Task tracking

This project uses Yaks to track its own work. All significant tasks should be reflected as yaks with their status updated appropriately:

- Before starting work, create a yak with `/yaks:create` (or check `/yaks:list` for an existing one).
- Run `/yaks:shave TASK_ID` when you begin working on a task.
- Run `/yaks:shorn TASK_ID` when the task is complete — this auto-records the commit hash.
- At the start of a session, run `/yaks:list` and `/yaks:next` to see what's in flight and what's ready.

## Key design decisions

- Status is never stored in the YAML file — it's determined by which directory (`hairy/`, `shaving/`, `shorn/`) the file is in. Moving a task between statuses means renaming the file to a different directory.
- Task IDs are `{prefix}-{4 hex chars}`, generated collision-free against existing files.
- The YAML dumper uses block scalars (`|`) for multiline strings via a custom `_BlockScalarDumper`.
- `next` checks that all `depends_on` IDs exist in `shorn/`; `tangled` shows tasks with at least one unshorn dependency.

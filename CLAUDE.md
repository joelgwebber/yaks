# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What is this?

Yaks is a filesystem-native task tracker distributed as a Claude Code plugin. Tasks are plain YAML files stored in `.yaks/` directories within projects — no database, no daemon. Task status is implicit from the directory the file lives in (`hairy/`, `shaving/`, `shorn/`).

## Architecture

- **`scripts/yak.py`** — The entire tracker is a single Python script (PEP 723 inline metadata, requires `pyyaml>=6.0`). It uses argparse with subcommands that map 1:1 to the commands below.
- **`commands/*.md`** — Each file defines a slash command for the Claude Code plugin. Frontmatter specifies `description`, `argument-hint`, and `allowed-tools`. The body tells Claude how to invoke `yak.py` using `${CLAUDE_PLUGIN_ROOT}/scripts/yak.py`.
- **`skills/yak/SKILL.md`** — The skill definition that activates when `.yaks/` exists. Carries the prescriptive workflow instructions that tell Claude when and how to track tasks.
- **`.claude-plugin/plugin.json`** and **`marketplace.json`** — Plugin and marketplace metadata.
- **`.yaks/config.yaml`** — Per-project config (currently just `prefix` for task ID generation).

## Running the script

```
python3 scripts/yak.py <subcommand> [args]
```

Subcommands: `init`, `create`, `list`, `show`, `update`, `shave`, `shorn`, `regrow`, `next`, `tangled`, `dep`, `stats`, `import-beads`. Old names (`work`, `close`, `reopen`, `ready`, `blocked`) are accepted as aliases. All support `--json` where applicable.

## Task YAML schema

```yaml
id: prefix-hex4       # e.g. yak-a1b2, or parent-id.N for children
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

## Key design decisions

- Status is never stored in the YAML file — it's determined by which directory (`hairy/`, `shaving/`, `shorn/`) the file is in. Moving a task between statuses means renaming the file to a different directory.
- Task IDs are `{prefix}-{4 hex chars}`, generated collision-free against existing files. Child tasks use `{parent-id}.N` (dot-suffixed integers, arbitrary depth). Prefixes must not contain dots.
- Parent/child relationships are implicit from IDs — no YAML field needed. `show` displays parent and children automatically.
- The YAML dumper uses block scalars (`|`) for multiline strings via a custom `_BlockScalarDumper`.
- `next` checks that all `depends_on` IDs exist in `shorn/`; `tangled` shows tasks with at least one unshorn dependency.

## Task tracking

This project uses Yaks to track its own work. The Yaks skill (activated by the `.yaks/` directory) defines the required workflow — follow it.

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What is this?

Yaks is a filesystem-native task tracker distributed as a Claude Code plugin. Tasks are markdown files with YAML frontmatter stored in `.yaks/` directories within projects — no database, no daemon. Task status is implicit from the directory the file lives in (`hairy/`, `shaving/`, `shorn/`).

## Architecture

Two entry scripts (both carry PEP 723 inline metadata, require `pyyaml>=6.0`) plus two supporting packages:

- **`scripts/yak.py`** — CLI entry. Thin shim that imports `COMMANDS` and `build_parser` from `yaklib` and dispatches. The plugin invokes it as `${CLAUDE_PLUGIN_ROOT}/scripts/yak.py`.
- **`scripts/tui.py`** — TUI entry. Holds the `TUI` class (state, main loop, scroll/nav helpers); the actual work delegates to `yaktui.*` modules.
- **`scripts/yaklib/`** — CLI-side library:
  - `model.py` — status constants, YAML I/O, `.yaks/` layout, task load/save, ID generation, parent/child arithmetic, `move_task`, git integration.
  - `commands.py` — every `cmd_*` function + `COMMANDS` dispatch table + mandate injection.
  - `parser.py` — `build_parser()`.
  - `deps.py` — shared dep resolution: `ready_tasks`, `tangled_tasks`, `compute_blocked`, `depends_on_transitively`.
  - `artifacts.py` — artifact link parsing + `artifacts_dir`.
  - `clipboard.py` — `copy_text` + `read_png` (macOS + Linux).
  - `format.py` — `humanize_date` + `status_char`.
  - `filter.py` — `FilterSpec` dataclass + `filter_tasks()`, shared across CLI and TUI.
  - `rollup.py` — one-way yak→external projection: `tracker_and_key()` URL classification, `effective_source()` ancestor inheritance, `build_rollup()` grouping. Read-only, no network.
  - `cli.py` — `main()` entry point for the installed `yaks` command (`uv tool install`).
- **`scripts/yaktui/`** — TUI-side library. All functions take the `App` instance (or just `stdscr`) as the first argument:
  - `colors.py`, `tree.py`, `detail.py`, `dialogs.py`, `mutate.py`, `render.py`, `keys_list.py`, `keys_detail.py`, `vim_edit.py`.
- **`commands/*.md`** — Slash commands for the Claude Code plugin. Each invokes `${CLAUDE_PLUGIN_ROOT}/scripts/yak.py`.
- **`skills/yak/SKILL.md`** — Skill definition that activates when `.yaks/` exists.
- **`tests/`** — pytest suite (subprocess CLI tests + unit tests for `deps` and the TUI pure functions). Run with `uv run pytest`.
- **`.claude-plugin/plugin.json`** and **`marketplace.json`** — Plugin / marketplace metadata.
- **`.yaks/config.yaml`** — Per-project config (`prefix`, `vim_mode`). User-global config at `~/.config/yaks/config.yaml` is merged under per-project values.

## Running the script

```
python3 scripts/yak.py <subcommand> [args]
```

Subcommands: `init`, `create`, `list`, `show`, `update`, `shave`, `shorn`, `regrow`, `slaughter`, `revive`, `next`, `tangled`, `dep`, `reparent`, `attach`, `detach`, `search`, `stats`, `rollup`, `tui`, `import-beads`. Old names (`work`, `close`, `reopen`, `ready`, `blocked`) are accepted as aliases. Most support `--json` where applicable.

## Task file format

Tasks are `.md` files with YAML frontmatter. The markdown body (after the closing `---`) is the description.

```markdown
---
id: prefix-hex4       # e.g. yak-a1b2, or parent-id.N for children
title: string
type: bug | feature | task | idea
priority: 1-5         # 1=urgent, 3=medium (default), 5=lowest
created: ISO8601
updated: ISO8601
depends_on: [task-ids]   # optional
labels: [strings]        # optional
source: URL              # optional, external issue URL
---

Optional description as markdown body.
```

## Key design decisions

- Status is never stored in the YAML file — it's determined by which directory (`hairy/`, `shaving/`, `shorn/`, or `dead/`) the file is in. Moving a task between statuses means renaming the file to a different directory.
- `dead/` is a hidden state for slaughtered yaks (ideas you won't pursue, obviated tasks). Dead yaks are excluded from every default query and from the TUI, but remain on disk. Dead deps count as "resolved" in `next`/`tangled`/blocked computations — slaughtering a dep unblocks its dependents.
- Task IDs are `{prefix}-{4 hex chars}`, generated collision-free against existing files. Child tasks use `{parent-id}.N` (dot-suffixed integers, arbitrary depth). Prefixes must not contain dots.
- Parent/child relationships are implicit from IDs — no YAML field needed. `show` displays parent and children automatically.
- Task files are `.md` with YAML frontmatter. The `description` field is not stored in frontmatter — the markdown body after the closing `---` is the description. Legacy `.yaml` task files are auto-migrated on first access.
- `next` checks that all `depends_on` IDs exist in `shorn/`; `tangled` shows tasks with at least one unshorn dependency.

## Releasing

Whenever you make changes that affect the plugin (commands, skills, `scripts/**`), bump the version. Without a bump, Claude Code plugin installs use a stale cached version.

Keep these two manifests in lockstep — bump **both** to the same new version in the same commit:

- `.claude-plugin/marketplace.json` — the `version` under `plugins[0]` (not the top-level `"version": "1.0.0"`, which is the marketplace schema version and stays put).
- `.codex-plugin/plugin.json` — the top-level `version`.

`.claude-plugin/plugin.json` deliberately carries no version field (the marketplace entry is authoritative for Claude), so there's nothing to bump there. Versions are plain `MAJOR.MINOR.PATCH`; increment the patch for ordinary changes.

## Task tracking

This project uses Yaks to track its own work. Every piece of work must be bracketed: `/yaks:shave` before coding, `/yaks:shorn` once the work is done — and when practical, land the shorn yak file in the same commit as the code that completed it. The Yaks skill has the full workflow — follow it.

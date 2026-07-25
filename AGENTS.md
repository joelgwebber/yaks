# AGENTS.md

This file provides guidance to AI coding agents (Claude Code, OpenAI Codex, Zed, and others) working in this repository. It replaces the former `CLAUDE.md`; Claude Code, Codex, and Zed all read `AGENTS.md` natively.

## What is this?

Yaks is a filesystem-native task tracker. It ships as a plugin for Claude Code and OpenAI Codex, as an installable skill for Zed, and as a standalone CLI (published on PyPI as `yakherder`, exposing the `yaks` command) for any other agent. Tasks are markdown files with YAML frontmatter stored in `.yaks/` directories within projects — no database, no daemon. Task status is implicit from the directory the file lives in (`hairy/`, `shaving/`, `shorn/`).

## Architecture

Two entry scripts (both carry PEP 723 inline metadata, require `pyyaml>=6.0`) plus two supporting packages:

- **`scripts/yak.py`** — CLI entry. Thin shim that imports `COMMANDS` and `build_parser` from `yaklib` and dispatches. Runnable directly (`python3 scripts/yak.py`) for development from a checkout; installed users get the same dispatch via the `yaks` console script (see `cli.py`).
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
  - `cli.py` — `main()` entry point for the installed `yaks`/`yakherder` console scripts (`uv tool install yakherder`, or `uvx yakherder`).
- **`scripts/yaktui/`** — TUI-side library. All functions take the `App` instance (or just `stdscr`) as the first argument:
  - `colors.py`, `tree.py`, `detail.py`, `dialogs.py`, `mutate.py`, `render.py`, `keys_list.py`, `keys_detail.py`, `vim_edit.py`.
- **`skills/yak/SKILL.md`** — Primary skill; activates when `.yaks/` exists and teaches the agent to drive the `yaks` CLI directly (via `uvx yakherder` or an installed `yaks`). No slash commands — those were removed so the workflow is identical across Claude Code, Codex, and Zed.
- **`skills/yak-tracker/SKILL.md`** — Skill for relating yaks to external issue trackers (Jira/Linear/GitHub) as a one-way projection.
- **`pyproject.toml`** — Packaging. Distribution name `yakherder`; console scripts `yaks` + `yakherder`; setuptools backend. Built/published via `uv build` / `uv publish`.
- **`.github/workflows/publish.yml`** — Publishes `yakherder` to PyPI via Trusted Publishing on a `v*` tag.
- **`tests/`** — pytest suite (subprocess CLI tests + unit tests for `deps` and the TUI pure functions). Run with `uv run pytest`.
- **`.claude-plugin/plugin.json`** and **`marketplace.json`** — Plugin / marketplace metadata.
- **`.yaks/config.yaml`** — Per-project config (`prefix`, `vim_mode`). User-global config at `~/.config/yaks/config.yaml` is merged under per-project values.

## Running the script

From a checkout during development:

```
python3 scripts/yak.py <subcommand> [args]
```

As an end user it's the `yaks` command (`uv tool install yakherder`), or zero-install via `uvx yakherder <subcommand>`.

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

## Distribution & releasing

Yaks ships through four surfaces, all built from this one repo:

| Surface | What ships | How users get updates |
|---------|-----------|-----------------------|
| PyPI package | `yakherder` distribution (exposes the `yaks`/`yakherder` commands) | `uv tool upgrade yakherder`, or `uvx yakherder` re-resolves |
| Claude Code plugin | `.claude-plugin/` + `skills/` | marketplace re-pulls when `plugins[0].version` bumps |
| Codex plugin | `.codex-plugin/` + `skills/` | re-pulls when the top-level `version` bumps |
| Zed skill | `skills/yak/`, `skills/yak-tracker/` | skills.sh, or a manual re-sync of `~/.agents/skills/` from the repo |

The `skills/` directory is shared by the plugin surfaces, so a skill edit is a change to all of them at once.

### Version bumps

Whenever you change anything that ships (skills, `scripts/**`, `pyproject.toml`), bump the version. Plugin marketplaces and `uv`'s cache key on the version string, so an unchanged number means users keep a stale copy. Pure-docs changes that don't touch the shipped payload (e.g. this file, top-level `README.md` is an exception — it's the packaged long description, so treat README edits as shipping) don't strictly need a bump.

Bump these three in lockstep — same value, same commit:

- `pyproject.toml` — the `[project] version` (the PyPI/`yakherder` package version).
- `.claude-plugin/marketplace.json` — the `version` under `plugins[0]` (NOT the top-level `"version": "1.0.0"`, which is the marketplace-schema version and stays put).
- `.codex-plugin/plugin.json` — the top-level `version`.

`.claude-plugin/plugin.json` deliberately carries no version field (the marketplace entry is authoritative for Claude), so there's nothing to bump there. Versions are plain `MAJOR.MINOR.PATCH`; increment the patch for ordinary changes.

### Publishing to PyPI

Publishing is automated by `.github/workflows/publish.yml` via PyPI Trusted Publishing (OIDC — no tokens or secrets). The pending publisher is configured on PyPI (project `yakherder`, repo `joelgwebber/yaks`, workflow `publish.yml`, environment `pypi`).

To cut a release:

1. Bump the three versions in lockstep and commit.
2. Tag that commit `vX.Y.Z`, matching `pyproject.toml` (e.g. `git tag v0.1.78`).
3. `git push origin v0.1.78`. The workflow builds the sdist + wheel and uploads to PyPI.

The plugin and Zed-skill surfaces do **not** need the tag — they update from the committed manifest versions. The `v*` tag exists solely to trigger the PyPI publish.

## Task tracking

This project uses Yaks to track its own work. Every piece of work must be bracketed: `yaks shave` before coding, `yaks shorn` once the work is done — and when practical, land the shorn yak file in the same commit as the code that completed it. The Yaks skill has the full workflow — follow it.

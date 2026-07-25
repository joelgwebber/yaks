---
name: yak
description: Yaks task tracking workflow. Use when a .yaks/ directory exists in the project. Provides commands and guidance for managing filesystem-native tasks stored as markdown files with YAML frontmatter.
activation:
  - .yaks/ directory exists in the project
---

# Yaks — Task tracking workflow

This project tracks work with Yaks. Tasks are markdown files with YAML frontmatter in `.yaks/`. You MUST follow this workflow to keep task state accurate.

## Running yaks

Yaks is a plain command-line tool. Run it directly from your shell — there are
**no slash commands**. Use the first invocation that works in your environment:

1. **`yaks <cmd>`** — if `yaks` is on `PATH` (installed via `uv tool install yakherder`). Prefer this.
2. **`uvx yakherder <cmd>`** — zero-install, runs in an isolated environment. Requires `uv`.
3. **`uvx --from git+https://github.com/joelgwebber/yaks yaks <cmd>`** — fallback if `yakherder` isn't on PyPI yet.

The distribution is published as `yakherder` (the name `yaks` was taken on PyPI);
the command you run is `yaks`. Every example below is written as `yaks <cmd>` —
substitute whichever invocation above works for you. The CLI is stateless, so
each call is independent; there's nothing to keep running.

Add `--json` to any query command for machine-readable output.

## Hard rules

**NEVER write code without an active shaving yak.** Before touching any code — even a one-line fix — you must have a yak in shaving state. If you don't, stop and `yaks shave TASK_ID` one first (create it if needed). No exceptions.

**ALWAYS shear when a yak's work is done.** Run `yaks shorn TASK_ID` as soon as the task is complete. In **team mode** (see below), prefer to stage the shorn yak file alongside the code changes that completed it and commit them together rather than as a separate commit. In **local-only mode**, never commit yak files at all.

## Two workflows: local or team

Yaks runs in one of two modes, and they call for different habits. **Figure out which mode you're in before you commit anything** — the signal is whether `.yaks/` is tracked by git:

- `.yaks/` is gitignored or otherwise untracked → **local-only** (a private scratchpad).
- `.yaks/` is committed alongside the code → **team** (a shared tracker).

To check: `git check-ignore .yaks` printing a path means local-only; `git ls-files .yaks` listing files means team. If a fresh checkout is genuinely ambiguous, default to local-only — it's the safer assumption.

**Local-only.** The yak files live only on this machine; they're your planning memory, not part of the project's shared history.
- Never `git add` yak files or include them in commits.
- Keep yaks invisible to everyone else: do **not** mention them — or their IDs — in commit messages, PR titles/descriptions, code comments, or external trackers. Describe the change in plain terms ("add retry logic"), not as "shorn yak-1234".
- The `commit:` field still gets stamped on shorn yaks; that's fine, it stays in the local file.

**Team.** The yak files are part of the repo — treat them like code.
- Commit the shorn yak move together with the code that completed it (hard rule 2).
- Referencing yak IDs in commit messages and PRs is welcome; everyone can resolve them.

## Workflow

1. **Session start** — run `yaks list` and `yaks next` to see current state.
2. **Before writing code** — `yaks shave TASK_ID` (create the yak first if needed).
3. **While working** — append progress notes with `yaks update TASK_ID --note "what you found / decided / changed"`. This builds a running log in the markdown body so future sessions have context.
4. **When the work is done** — `yaks update TASK_ID --note "..."` with a brief shorn summary (what was done, what was learned, any yaks spawned), then `yaks shorn TASK_ID`. If using git, stage the shorn yak move together with the code changes and commit them in one commit whenever practical.

## Parent/child state rules

A parent yak's state should reflect its children:
- When you shave a child, shave the parent too (if it's still hairy).
- When you shear the last unshear child, shear the parent too.
- NEVER leave a hairy parent with shorn children — that means work was done but the parent doesn't reflect it.

## Commands

Run these directly from the shell (see **Running yaks** above for the exact invocation).

| Command | What it does |
|---------|-------------|
| `yaks create` | Create a new task |
| `yaks list` | List tasks with optional filters |
| `yaks show` | Show full details of a task |
| `yaks update` | Update a task's fields |
| `yaks shave` | Start shaving a yak (move to shaving) |
| `yaks shorn` | Mark a yak as shorn |
| `yaks regrow` | Regrow a shorn yak |
| `yaks slaughter` | Slaughter a yak (hide in `.yaks/dead/`) — for ideas you won't pursue or tasks that have been obviated |
| `yaks revive` | Revive a dead yak back to hairy |
| `yaks next` | Shortcut for `list --status hairy --ready` |
| `yaks tangled` | Shortcut for `list --status hairy --tangled` |
| `yaks search` | Shortcut for `list --search QUERY` |
| `yaks dep` | Add/remove dependencies between tasks |
| `yaks reparent` | Move task to new parent or top-level |
| `yaks stats` | Show task statistics |
| `yaks rollup` | Group yaks by the external issue they roll up to |
| `yaks tui` | Open the interactive terminal UI |

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
---

Details go here.
```

Child tasks use `--parent TASK_ID` on create. The hierarchy is implicit from the ID (dot-suffixed integers). `yaks show` displays parent and children automatically.

### External source linking

Use `--source URL` on create or update to link a yak to an external issue (Jira, GitHub Issues, Linear, etc.). The URL is stored in the `source` frontmatter field. The relationship is a **one-way projection**: the yak points at the external issue, never the reverse, and the external tracker stays unaware of yaks.

Many yaks can roll up to one external issue. `yaks rollup` groups yaks by their source (a yak with no `source:` inherits its nearest ancestor's, so one stamp on an umbrella yak covers the subtree); `yaks rollup --keys` lists the external keys to paste into a PR body. For seeding a yak from an external issue or drafting a status update back to one, see the **yak-tracker** skill.

## Filtering

Every query command (`list`, `search`, `next`, `tangled`) shares the same filter flags. AND across dimensions; within a repeatable flag, OR:

- `--status S` / `--type T` / `--priority P` / `--label L` (all repeatable)
- `--search Q` — substring match on title/description/id
- `--ready` / `--tangled` — dep-state filters
- `--parent-of ID` — only descendants of ID

Examples:
- `yaks list --type bug --type feature --priority 1` — urgent bugs or features
- `yaks list --label auth --search retry` — auth-labeled tasks mentioning "retry"
- `yaks next --type bug` — ready bugs only

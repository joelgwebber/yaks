# yaks

A filesystem-native task tracker. Tasks are plain markdown files with YAML
frontmatter, kept in a `.yaks/` directory inside your project — no database, no
daemon, no server. A task's status is implicit in *which folder it lives in*, so
your task list is just files you can read, grep, edit, and commit alongside your
code.

yaks ships as a single self-contained binary. Startup is effectively instant,
which matters because the workflow is lots of small command invocations.

```text
.yaks/
  hairy/     todo          (a hairy yak, not yet shaved)
  shaving/   in progress   (you're shaving it)
  shorn/     done          (shorn)
  dead/      abandoned     (slaughtered; hidden from normal queries)
```

## Install

**npm** (prebuilt binary for your platform; the command is `yaks`):

```sh
npm i -g @j15r/yaks     # then: yaks --help
# or zero-install:
npx @j15r/yaks list
```

**From source** (needs a Rust toolchain):

```sh
git clone https://github.com/joelgwebber/yaks
cd yaks
cargo build --release
./target/release/yaks --help
```

## Quick start

A farm is just a `.yaks/` directory. `yaks init` creates one at your project
root (plain `yaks init` makes a committed farm and tells you how to add the agent
skills; `--mode private|pointer` and `--skills default` do the whole setup in one
idempotent step, see [docs/cli.md](docs/cli.md)). Then start tracking:

Within a farm, yaks group into **herds** by id prefix — a farm can hold several
(`yaks create --herd <herd>`), so one (typically private) farm can track
several projects at once — and a parent yak with its descendants forms a
**family**.

```sh
yaks init --skills default   # the farm + the agent skills (add --mode private to keep both out of git)
yaks create --title "Wire up the login form" --type feature --priority 2
yaks list
yaks shave <id>     # start work  (hairy -> shaving)
yaks shorn <id>     # finish      (shaving -> shorn)
```

That's the whole loop: **shave** a yak before you work on it, **shear** it
(`shorn`) when it's done.

## Concepts

**States are adjectives, verbs are transitions.** A yak is *hairy* (todo),
*shaving* (in progress), or *shorn* (done); *slaughtered* yaks go to a hidden
`dead/`. You **shave** (hairy→shaving), **shear** / mark **shorn**
(shaving→shorn), **regrow** (shorn→hairy), **slaughter**, and **revive**.

**Task file.** Frontmatter for metadata, the markdown body for the description:

```markdown
---
id: yak-a1b2
title: Wire up the login form
type: feature          # bug | feature | task | idea
priority: 2            # 1 urgent … 5 lowest (default 3)
created: "2026-02-16T10:00:00Z"
updated: "2026-02-16T10:30:00Z"
parent: yak-c3d4       # optional; only on child tasks
depends_on: [yak-e5f6] # optional
labels: [auth]         # optional
source: https://…      # optional external issue URL
---

Longer description goes here.
```

IDs are flat and stable (`{prefix}-{4hex}`). Hierarchy lives in the `parent:`
field, not in the ID. Dependencies are ids in `depends_on:`; a yak is *ready*
when all of them are shorn (or dead), and *tangled* otherwise.

## Commands

`yaks -C <path> <command>` (global, like `git -C`) runs any command as if started
in `<path>`. `yaks --shed <name> <command>` is `-C` by name: it runs the command
in another checkout of this repo (a shed from `yaks sheds`, picked like
`yaks changes <shed>`), full access, so use it only on a checkout you own.

| Command | What it does |
|---------|--------------|
| `yaks create` | Create a task; the title is positional (`yaks create "Fix login"`). Flags: `--type`, `--priority`, `--parent`, `--labels` (comma- or space-separated), `--depends-on`, `--source`, `--description` (`-` reads stdin; `--description-file PATH`), `--json` (emit the new id + file path) |
| `yaks list` | List tasks; filter by `--status/--type/--priority/--label/--search`, `--ready`, `--tangled`, `--parent-of`, `--all` |
| `yaks show <id>` | Full detail for one task, with parent + children; `--sheds` adds how each other shed (git worktree / Delta clone) has it where that differs: status, `needs:`, new notes |
| `yaks refs <id>` | List what a task points at (parent, deps, id mentions in its text), flagging any that dangle |
| `yaks commits <id>` | Show the git commits linked to a yak — those naming its id and those that touched its file across status moves |
| `yaks update <id>` | Change fields/labels, set `--description`, or append a `--note` (`-` reads stdin; `--note-file PATH`: multi-line markdown without shell quoting) |
| `yaks ask <id>` / `answer <id>` | Block a yak on a human (sets `needs: human`, drops it from `next`) / answer it (sets `needs: agent`, so the answer cannot get lost; `--done` clears instead), each recording a `--note`. `answer <id>@<shed>` answers in another shed's copy of the yak, left uncommitted there |
| `yaks pickup <id>` | An agent takes on an answered yak: clears `needs: agent` with an attributed note. The only thing that clears it |
| `yaks inbox` | The `needs` queue, two sections: awaiting a human (`replied` marks a plain-note reply that skipped `answer`) and answered, awaiting an agent; `--for human\|agent`, `--sheds` (also the open asks in other sheds, with their question text), `--json` |
| `yaks shave <id>` | hairy → shaving (alias: `work`) |
| `yaks shorn <id>` | shaving → shorn (alias: `close`) |
| `yaks regrow <id>` | shorn → hairy (alias: `reopen`) |
| `yaks slaughter <id>` / `revive <id>` | move to / from the hidden `dead/` (`slaughter --family` also takes live descendants) |
| `yaks next` / `tangled` | ready tasks / dependency-blocked tasks |
| `yaks path <id>…` / `path <filters>` | each yak's current absolute file path (for a precise `git add`) |
| `yaks search <q>` | substring search over id/title/description |
| `yaks sheds` | the other checkouts of this repo (git worktrees, Delta clones) and what each one's farm changed since that shed forked (from its own reflog, so a worker is not credited with what its coordinator did first, and a shed synced with your checkout since then counts only what came after the sync; `(vs merge-base)` marks the fallback when there is no usable reflog; not a diff against your checkout): new/moved yaks, new notes, `needs:`, plus commits ahead/behind (`+? -?`, JSON `null`, when there is no merge-base, e.g. shallow history; the table then says once to `git fetch --unshallow`), and a label per shed: who is working there (actors on its own new notes) and which yaks it has in `shaving`; read-only. Each shed's `name:` line (JSON `name`) is `main` for the primary checkout, else the slug of the Delta thread title (lowercase, runs of other characters one `-`, at most 24 characters, cut at a `-`; two clones with one slug get their dir id's first 4 characters appended, e.g. `fix-it-q7zz`), else its first `who` actor, else its Delta dir id, else its branch. The title is read from the clone's own git config `yaks.shed.title` (JSON `title`, also shown on the `name:` line), which ANY yaks command run inside a Delta clone with `DELTA_THREAD_TITLE` set records (with `yaks.shed.thread` from `DELTA_CURRENT_THREAD_ID`), only when the value changed, silently, never for `-C` or `--shed`, never in a plain git worktree; it is per-clone and never committed. `--json` |
| `yaks changes SHED` | What ONE shed (named by: the name `sheds` prints (the thread-title slug of a Delta clone), `main` for the primary checkout, a Delta dir id, a branch, an actor on its notes, a path, or a unique substring; the same rule as `answer <id>@<shed>`) created, moved and noted since it forked, per yak, with actors: new yaks (title, status), `from -> to` moves, each new note entry (timestamp, actor, first line) and `needs:` newly set. The per-yak expansion of the `farm:` line of `sheds`, with the same baseline (fork, sync, else `(vs merge-base)` / `(vs this checkout)`). `--json`: one object per changed yak (`id`, `title`, `status`, `added`, `moved`, `notes` with `ts`/`actor`/`text`, `needs`, `vs`, `base`). An unknown or ambiguous name lists the sheds and exits 1; a shed sharing this farm (or with none) says why there is nothing to show. Read-only |
| `yaks discover [PATH]` | diagnostic: how `yaks sheds` finds the other checkouts, step by step, from inside any checkout: `git worktree list`, the host repo this checkout's alternates name, every Delta clone pinned there (`refs/delta/<dir>/<name>/*`, live or gone), and the host checkout's git worktrees. Git data only; read-only, no farm needed. `--json` |
| `yaks log` | timestamped notes and status moves across a filtered set, oldest first (an activity log); `--since`/`--by` narrow it |
| `yaks dep` / `reparent` | edit dependencies / move under a new parent |
| `yaks bulk` | Apply one field edit (and/or reparent) to every yak matching a filter. **Dry-run by default** — pass `--commit` to apply. Requires a filter *and* a mutation flag |
| `yaks rollup` | group yaks by the external issue they roll up to (`--keys` for a PR body) |
| `yaks stats` | task statistics |
| `yaks doctor` | read-only farm-integrity check (duplicate-status ids, dangling parent/dep refs); exits non-zero on problems, so it's CI-usable. `--json` for machine output |
| `yaks doctor --strict` | also flags shorn yaks with no recorded note — a shear without evidence (the evidence-before-shear rule) |
| `yaks preflight [<id>...]` | read-only landing-readiness check for a team farm: nothing under `.yaks/` untracked or unstaged in git, the `verify:` of each shorn yak in the change last PASSed (`--all`: every shorn yak), no yak in two status dirs; `--push-main` also checks that `local` is the human's checkout (not bare, has `main`, not dirty on `main`) before `git push local <branch>:main`; exits non-zero with one line per failure, else `preflight: ok` |
| `yaks brief <id> --as <name>` | print the worker brief for a yak, read-only: the actor prefix, the gate (`verify:`), how to finish in a team or a private farm, how to ask and pick up an answer. The coordinator adds scope, the spawn title and landing |
| `yaks status [--check] [--json]` | what the farm has changed that git does not have yet, one line per yak (`created`, `moved hairy -> shaving`, `notes +2`, `edited`, `removed`, `artifacts`; `*` marks staged), ending with the message `yaks commit` would use; read-only; code elsewhere is only counted; `--check` exits 1 when dirty, `--json` for scripts |
| `yaks commit [-m <msg>] [--dry-run]` | commit the farm's own changes (everything under `.yaks/`) and nothing else, with a generated message like `yaks: shorn yaks-abc1; updated yaks-def2`; code staged elsewhere stays staged and out of the commit; runs your git hooks, never pushes; errors in a private farm, no-op when clean |
| `yaks scan-ids [file]` | flag real yak-ids in text (file and/or stdin) — a leak check for a pre-commit / PR gate; exits non-zero if any are found |
| `yaks tui` | open the interactive terminal UI |

Add `--json` to any query command for machine-readable output. The
state-transition verbs (`shave`, `shorn`, `regrow`, `slaughter`, `revive`) plus
`update` and `reparent` accept **multiple ids** and apply the same change to
each. Note-writing commands (`update`, `ask`, `answer`) and the state-transition
verbs take `--as <actor>` to attribute the note or move (falling back to
`$YAKS_ACTOR`, then the harness identity, then the git user). Every move appends
a `moved: <from> -> <to>` entry to the yak, stamped like a note.

`yaks bulk` is a filter-driven mass edit: the standard query filters
(`--status`, `--type`, `--priority`, `--label`, `--search`, `--ready`,
`--tangled`, `--needs`, `--parent-of`) *select* the set, and a mutation flag
(`--add-label`, `--remove-label`, `--set-priority`, `--set-type`, `--reparent`,
`--unparent`) *applies* the change to each. It is **dry-run by default**:
without `--commit` it just prints the matched set and the intended mutation,
changing nothing. It refuses to run without at least one filter flag (so it
never touches the whole farm) and without at least one mutation flag. It only
edits fields and reparents — no state transitions (use `shave`/`shorn`/etc. for
those). Example: `yaks bulk --label auth --set-priority 1 --commit`.

## Interactive TUI

`yaks tui` opens a full-screen browser over the farm — views by state, a detail
pane, inline create/edit, dependency and reparent pickers, search, and an
embedded modal editor (vim or emacs keybindings, per `.yaks/config.yaml`). It
auto-refreshes when the files change underneath it, so it stays in sync if you
(or an agent) edit yaks from elsewhere. It takes the mouse too (wheel scroll,
click to select / open / switch view). Mark rows with `m` for a bulk state
change over the selection; `a` asks or answers (answer hands the yak to an agent: `needs: agent`);
and an **Inbox** view lists every yak with a `needs` value, each flagged inline
with a ⏳ badge and a warning accent (awaiting a human) or a ✅ badge (answered,
awaiting an agent).

![The yaks TUI list view: a parent yak with children across statuses, labels, a dependency, and a ⏳ needs badge](docs/assets/tui-list.svg)

Open a yak (`l`) for its metadata, relations, description and attributed notes:

![The yaks TUI detail pane for a yak awaiting a human, with attributed notes](docs/assets/tui-detail.svg)

These screenshots are rendered headlessly, and the same machinery lets an agent
drive the TUI: `yaks tui --headless` reads one action per line on stdin and prints a
text snapshot after each, via [toque](https://github.com/joelgwebber/toque) (its
README walks through a short session). See [docs/tui.md](docs/tui.md) for the keys.

## Use with an AI coding agent

yaks includes an **agent skill** so assistants drive it correctly (shave before
coding, shear when done, keep task state honest). It's a plain skill — it just
shells out to the `yaks` CLI (or `npx`), so there's nothing to run as a plugin.

Install the skill straight from the binary — no clone required:

```sh
yaks skills install                          # -> ./.agents/skills in the git repo you're in (yaks + yaks-tracker)
yaks skills install --with coordination      # + the multi-agent skills (yaks-coordinating*, yaks-working)
yaks skills install --user                   # -> ~/.agents/skills (also the default outside a git repo)
yaks skills install --dir ~/.claude/skills   # any agent's skills dir; --force to overwrite
yaks skills status                           # current / stale / edited, same directory rules
```

Inside a git repo the default is project-local (`.agents/skills` in the repo's
top-level), so agents working in the project find the skills and you can commit
them; it prints where it wrote. Project-local installs are never updated behind
your back, but an ordinary `yaks` command tells you (one stderr line, once per
yaks release per checkout) when they are `stale`; `yaks skills status` and
`yaks doctor` say it any time, and you re-run `yaks skills install`.
`YAKS_SKILLS_AUTOSYNC=0` silences the notice.
`--user` / outside a repo, `~/.agents/skills` is the **cross-client convention**:
the [Agent Skills](https://agentskills.io) client-implementation guide tells
agents to scan it in addition to their own native directory, so one install
reaches every compliant client. (The spec defines the `SKILL.md` format, not
where skills live — there is no official installer.) **Claude Code is the
exception**: it reads `~/.claude/skills` and does *not* scan `.agents`, so
install there explicitly with `--dir`.

It activates when a `.yaks/` directory is present, and shells out to the `yaks`
binary (or `npx @j15r/yaks`), so make sure one of those is on the
agent's `PATH`.

Prefer a universal, multi-agent skills manager? The skills are plain
spec-compliant `SKILL.md` files, so [openskills](https://github.com/numman-ali/openskills)
installs them too (note it uses `.agent/` — singular — for its `--universal`
mode, which is its own convention rather than the `.agents/` one above):

```sh
npx openskills install joelgwebber/yaks
```

## Public and private farms

Because a farm is just a `.yaks/` directory, *you* decide whether it's shared or
private by choosing whether git tracks it.

**Public (team).** Commit `.yaks/` alongside the code. The task list travels with
the repo, shows up in PRs and `git log`, and yak moves merge with the change that
completed them. This project works this way — it tracks its own work in a
committed farm.

**Private (local-only).** Keep `.yaks/` out of the code repo and it becomes a
personal scratchpad no one else sees. `yaks init --mode private` does it for you
(the farm, a `/.yaks` line in `.git/info/exclude`, and the skills);
`yaks init --mode pointer --path <dir>` points the repo at a farm kept elsewhere.
By hand, hide it whichever way fits:

- a `.yaks/` line in the root `.gitignore` — simplest, but the rule is committed;
- a `.yaks/.gitignore` containing `*`, so the farm hides itself with no change to
  the repo root — for a plain, non-nested farm only;
- `.git/info/exclude`, which is per-repo and never committed;
- a global `core.excludesFile`, to ignore `.yaks/` across every project at once.

**Private across machines.** To carry a private farm between machines without
committing it to the code repo, give `.yaks/` its own git repo on a private
remote, nested inside the project:

```sh
cd .yaks
git init && git remote add origin <your-private-remote>
# work from inside .yaks/ for farm git ops; pull before, push after
```

Hide the nested repo from the outer repo with `.git/info/exclude` (not the `*`
trick, which would also blind the farm's own repo). yaks needs no configuration —
it discovers `.yaks/` exactly as before (from the repo, or any directory below it).

**Several repos, one farm.** To track several projects in a single out-of-tree
farm, put a `.yaks` *file* (not a directory) at each repo root pointing at the
shared farm:

```
path: ~/work/shared-farm/.yaks
herd: web
```

Discovery follows the pointer, so every command in that repo operates on the
shared farm and `yaks create` routes new yaks into that repo's herd (`herd:`)
with no flag. A `.yaks` symlink to the farm works too (but every repo then
shares one herd). Keep the pointer/symlink out of the shared repo with
`.git/info/exclude`, and use `yaks merge` to fold existing farms into the shared
one.

**Where discovery stops.** `yaks` walks up from the current directory but stops
at the git top-level (the first directory with a `.git`) unless that directory
has a `.yaks` entry. A checkout nested inside another repo's tree, such as a
Delta checkout or an in-tree `git worktree` of a private farm, therefore does
not find the outer repo's farm by accident; it fails with an error naming the
fixes: `yaks init`, a `.yaks` pointer file, or `YAKS_DIR=<farm>` (the `.yaks/`
dir, a directory containing it, or a pointer file). Outside a git repo the walk
runs to the filesystem root.

> **Heads up:** `git clean -fdx` in the outer repo will delete an ignored or
> excluded `.yaks/`, including a nested farm's history. Push a private farm
> often, and remember a gitignored `.yaks/` won't appear in fresh clones or other
> worktrees.

## Configuration

Optional per-project config lives in `.yaks/config.yaml`:

```yaml
herd: yak              # default herd / id prefix (`yaks init` writes it; `create` errors without one)
default_type: task     # default --type
default_priority: 3    # default --priority
vim_mode: true         # TUI editor keybindings: vim (true) or emacs (false)
```

A user-global `~/.config/yaks/config.yaml` is merged underneath per-project
values.

## License

Apache-2.0.

# Yaks

A filesystem-native task tracker, for humans and their agents to get shit done together. Yaks are
aggressively simple. Each is a plain markdown file with YAML frontmatter, kept in a `.yaks/`
directory inside your project -- no database, no daemon, no server. A task's status is implicit in
*which folder it lives in*, so your task list is just files you can read, grep, edit, and commit
alongside your code.

```text
.yaks/
  hairy/     todo          (a hairy yak)
  shaving/   in progress   (you're shaving it)
  shorn/     done          (shorn)
  dead/      abandoned     (slaughtered)
```

## Why Yaks?

Because [yak-shaving](). We all do it, for good or ill -- and coding assistants only make each hairy
yak more tempting. This tool aims to at least make the endless shearing manageable.

## Why Rust?

Overkill? Maybe. But I love a fast, self-contained binary. Like the gods and Ken Thompson intended.
It's also nice for CLI tools to be fast.

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

A farm is just a `.yaks/` directory. Create one at your project root and start tracking:

Within a farm, yaks group into **herds** by id prefix — a farm can hold several (`yaks create --herd
<herd>`), so one (typically private) farm can track several projects at once — and a parent yak with
its descendants forms a **family**.

```sh
yaks init --herd myherd
yaks create --title "Wire up the login form" --type feature --priority 2
yaks list
yaks shave <id>     # start work  (hairy -> shaving)
yaks shorn <id>     # finish      (shaving -> shorn)
```

## Yak files

Each task is a simple markdown file, with metadata in frontmatter.

**Task file.** Frontmatter for metadata, the markdown body for the description:

```markdown
---
id: myherd-a1b2
title: Wire up the login form
type: feature          # bug | feature | task | idea
priority: 2            # 1 urgent … 5 lowest (default 3)
created: "2026-02-16T10:00:00Z"
updated: "2026-02-16T10:30:00Z"
parent: myherd-c3d4       # optional; only on child tasks
depends_on: [myherd-e5f6] # optional
labels: [auth]         # optional
source: https://…      # optional external issue URL
---

Lots more to say here in the description.

They can also have comments. Frank will demonstrate:

---
▸ 2026-09-24T01:33:44Z [Frank Zappa]
Forget yaks. Gonna raise me up a crop of dental floss!
```

IDs are flat and stable (`{prefix}-{4hex}`). Yaks can have a parent and children, which of course makes them a family. Yaks are stolid, reliable creatures -- so they can depend upon one another. A yak that depends upon another yak is "tangled", and can't be shorn until the other is.

And while it's a sad day when we most occasionally slaughter a yak, sometimes we must for the good of the herd.
Slaughtered yaks are no longer seen in the herd, but we never forget that they're with us in spirit (and in the
`.yaks/dead` directory).

## The `yaks` CLI

Agents and scripts will use the CLI to do most everything. You, thankfully, do not, terminal-nerd
though you may be. Hopefully you'll be using it once for `yaks init`, then `yaks tui` (see below).

But if you're a glutton for punishment (or an agent), then the CLI is documented in some detail
[here](./docs/cli.md). Or you could just run `yaks --help` like a normal person.

## The Yaks TUI

Even hardened terminal junkies accept that sometimes a UI is more effective than invoking everything
from the CLI like an old unix greybeard. But no fancy pixels for us! No sir, if we're going to have
a fancy UI, it's going to be stuffed into a beautiful grid of perfect little boxes.

The `yaks tui` gives you everything you could possibly want for your yak-herding needs. You can even
point your mouse at it, but don't let your keyboard-wielding neighbors see you do that. If you're
lost, try hitting `?` to see what you can do. And if for some unfathomable reason you want to close
this amazing work of interactive beauty, you can just hit `Q`. See, already so much friendlier than
`vi`! You can of course read [these docs](./docs/tui.md) if you absolutely must.

**Behold your beautiful herd**
![The yaks TUI list view: a parent yak with children across statuses, labels, a dependency, and a ⏳ needs badge](docs/assets/tui-list.svg)

**Peer into a yak's soul**
![The yaks TUI detail pane for a yak awaiting a human, with attributed notes](docs/assets/tui-detail.svg)

## Hire an agent for your yaks

... todo ...

yaks includes an **agent skill** so assistants drive it correctly (shave before
coding, shear when done, keep task state honest). It's a plain skill — it just
shells out to the `yaks` CLI (or `npx`), so there's nothing to run as a plugin.

Install the skill straight from the binary — no clone required:

```sh
yaks skills install                          # -> ~/.agents/skills  (yaks + yaks-tracker)
yaks skills install --dir ~/.claude/skills   # any agent's skills dir; --force to overwrite
```

`~/.agents/skills` is the default because it's the **cross-client convention**:
the [Agent Skills](https://agentskills.io) client-implementation guide tells
agents to scan it in addition to their own native directory, so one install
reaches every compliant client. (The spec defines the `SKILL.md` format, not
where skills live — there is no official installer.) **Claude Code is the
exception**: it reads `~/.claude/skills` and does *not* scan `.agents`, so
install there explicitly with `--dir`.

It activates when a `.yaks/` directory is present, and shells out to the `yaks`
binary (or `npx @j15r/yaks`), so make sure one of those is on the
agent's `PATH`.

## Public and private farms

Because a farm is just a `.yaks/` directory, *you* decide whether it's shared or
private by choosing whether git tracks it.

**Public (team).** Commit `.yaks/` alongside the code. The task list travels with
the repo, shows up in PRs and `git log`, and yak moves merge with the change that
completed them. This project works this way — it tracks its own work in a
committed farm.

**Private (local-only).** Keep `.yaks/` out of the code repo and it becomes a
personal scratchpad no one else sees. Hide it whichever way fits:

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
it discovers `.yaks/` exactly as before.

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

> **Heads up:** `git clean -fdx` in the outer repo will delete an ignored or
> excluded `.yaks/`, including a nested farm's history. Push a private farm
> often, and remember a gitignored `.yaks/` won't appear in fresh clones or other
> worktrees.

## Configuration

Optional per-project config lives in `.yaks/config.yaml`:

```yaml
herd: yak              # default herd / id prefix (default "yak")
default_type: task     # default --type
default_priority: 3    # default --priority
vim_mode: true         # TUI editor keybindings: vim (true) or emacs (false)
```

A user-global `~/.config/yaks/config.yaml` is merged underneath per-project
values.

## License

Apache-2.0.

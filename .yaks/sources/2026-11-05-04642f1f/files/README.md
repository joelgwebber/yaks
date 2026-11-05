# YASS! Yet Another Spec System

**A sassy "yes" to building with coding agents, without losing the plot.**

YASS keeps the plan, progress and decisions for every change in plain files next to your code, so people and agents can plan and build together.

**Describe the change, not the whole system.** Your code, docs and READMEs are the source of truth for what exists. YASS doesn't write it down a second time: it specs only what needs to change, keeps that next to the code, in plain markdown that agents and humans both read. It's lightweight project management for people and coding agents.

**[Install it](#install)**, then [set up a repo](#set-up-a-repo), or see [the YASS page](https://donjaime.github.io/yass/) for the tour.

Each piece of work is a **change**: a folder that carries as much ceremony as its scope needs, from a one-file bug fix to a feature with a PRD, a plan and a dozen pull requests. Progress, decisions and notes live in the folder, so any person or agent, on any harness, can pick up where the last one stopped.

```
yass/
├── changes/                                active work
│   ├── 2026-10-02-fix-double-tap-save/
│   │   └── change.md                       small: goal, acceptance, steps, decisions, log
│   └── 2026-09-14-offline-sync/
│       ├── change.md                       large: a slimmer head file (goal, decisions, log)…
│       ├── prd.md                          …plus why and what
│       ├── plan.md                         …how, and acceptance criteria
│       ├── design.md                       …the hard-to-undo calls (optional)
│       ├── 2026-09-16-offline-queue/       …and PR-sized pieces, one level deep
│       └── 2026-09-24-sync-badge/
└── archive/2026/10/                        finished changes, moved here as they were, by the month they were archived
```

That's the whole system: no database, no server, and no config unless you want one ([`yass.yaml`](#keeping-plans-out-of-the-repo)). A small CLI (`init`, `new`, `status`, `archive`, `decisions`, `evict`, `deinit`), six playbooks your agent follows, and an optional git hook.

YASS doesn't commit for you. The CLI never commits (archiving only stages the move), and the playbooks only commit, branch or rebase when you or your harness have given the agent that latitude; otherwise they hand you the commands. The rules below are about what lands on your main branch, and the hook checks them.

## The rules

1. **YASS describes change, not the current system.** Your code and its docs say what exists today. YASS says what you're changing, why, and how it went. A project that uses YASS from day one ends up with an archive that tells the story of the whole product; a project that adopts it later just starts recording from there.
2. **A change is a folder, and ceremony scales with scope.** A bug fix is one `change.md`. A new feature, component or system adds `prd.md` and `plan.md`, maybe `design.md`, and splits into PR-sized pieces inside it.
3. **Progress travels with the code.** Marking boxes in progress or done and appending to `## Log` and `## Decisions` go in the same commits as the code they describe.
4. **Intent changes get their own commit.** The PRD, the design, the plan's text, and a change's title, Goal and Acceptance, including dropping an acceptance criterion. If the plan turns out wrong, change it on purpose, in a commit someone can review, not quietly next to the code. Mid-build, that means a stacked branch: the intent change underneath, the code rebased on top, so work keeps moving and decisions can still be reviewed apart from execution.
5. **Done means every box is checked.** Then `yass archive` moves the folder to `archive/<YYYY>/<MM>/`. Finishing is separate from merging: a large change spans many branches and pull requests, and feedback can reopen the plan before it's done.
6. **The archive is append-only.** Follow-up work is a new change with `follows: <archived folder>`. Only `yass evict` removes from it, moving the oldest months into git history once a yass folder holds more than its `keep:` setting ([big archives](docs/monorepo.md#big-archives)).

## Install

Like most developer tools, YASS takes two steps: install the `yass` binary, once per machine, then [set up each repo](#set-up-a-repo) you use it in. [docs/install.md](docs/install.md) has every way in, with the details.

```bash
curl -fsSL https://raw.githubusercontent.com/donjaime/yass/main/install.sh | bash
```

That downloads the latest release for your machine, checks its checksum, and puts `yass` in `~/.local/bin`; it does nothing else, and it's short enough to [read first](install.sh). Rather check each step yourself? Install [from a release](docs/install.md#from-a-release), with checksums and signed build provenance, or from source with Go 1.24 or later: `go install github.com/donjaime/yass/cmd/yass@latest`.

**Then put it on your PATH,** so you, your agents and the hook can run `yass`. `install.sh` says whether `~/.local/bin` is on it (macOS doesn't put it there by default) and, if not, prints the line that adds it for your shell; for zsh:

```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshenv
```

Open a new terminal, and `yass --version` should answer.

## Set up a repo

From inside the repo:

```bash
yass init
git add -A && git commit -m "chore: adopt YASS"
```

`yass init` writes the playbooks in `.agents/skills/`, a short section in `AGENTS.md`, an empty `yass/` folder (or a [`yass.yaml`](#keeping-plans-out-of-the-repo) pointing elsewhere, with `--path`) and the optional hook in `tools/yass/githooks/`, and lists them. Add `--claude` for Claude Code, `--hooks` to turn on the hook, and `--global` to put the playbooks in your user folder instead of the repo ([all the options](docs/install.md#set-up-a-repo)). Committing it means teammates get it: joining a repo that already uses YASS needs only the binary.

Rather your agent did both steps? Paste [the install prompt](docs/install.md#with-your-agent) into it. It downloads a release (or builds from source), verifies it, reads `install.sh`, asks you for options, and doesn't commit.

Then talk to your agent:

| You say | Playbook | What happens |
|---|---|---|
| "Save this plan as a change" | `yass-shape` or `yass-plan` | the plan from your harness's plan mode becomes a change folder |
| "Fix the double-tap crash" | `yass-work` | a small change: goal, acceptance, build, tick, log |
| "Keep going" | `yass-work` | the change this session is already on; `yass-work` never picks one by itself |
| "Let's build offline sync" | `yass-shape` | a conversation that ends in a PRD you approve |
| "Plan it" | `yass-plan` | approach, acceptance criteria, PR-sized pieces, without much back-and-forth; nothing gets built yet |
| "Work on offline-queue" | `yass-work` | builds that change, resuming from its Log's last **Next:**; a wrong plan gets fixed on a branch underneath the code |
| "Where are we?" | `yass-status` | progress, blockers, recent decisions, what's ready to archive; changes nothing |
| "Why did we pick SQLite?" | `yass-log` | the decision, who made it and when, from the folders and git, with the code that carried it out |
| "Why does sync.go merge like this?" | `yass-log` | from the file's commits back to the change and decision that cite them |
| "Update YASS" | `yass-update` | checks the binary against the repo and the latest release, gets a verified one (asking first), then upgrades the repo and hands you the diff |

## Upgrading

The same two steps: the binary, once per machine, then each repo.

```bash
yass update                                          # 1. a newer binary, verified
yass upgrade                                         # 2. in each repo: its YASS files, to the binary's version
git add -A && git commit -m "chore: upgrade YASS"    # 3. so teammates get them
```

1. **`yass update`** installs the version your repo's YASS files are at when the repo is ahead of your binary, so you match your team, else the latest release, and says when a newer release exists (`--latest` gets it). It checks the release's checksums, and its signed build provenance when the GitHub CLI is installed, before swapping the binary in. `--check` only looks. A binary from `go install` or a clone isn't replaced; it prints the command to update that way.
2. **`yass upgrade`** brings every YASS file in the repo (the playbooks wherever they are, each `AGENTS.md` section, the hook) and your user folder's playbooks up to the binary's version, then brings each yass folder's layout up to date, printing how to commit that part on its own. Your changes, `yass.yaml` and your own parts of `AGENTS.md` are never touched; hand edits to YASS's own files are replaced, so check the diff. It never downgrades: when the repo is newer than your binary, it says to run `yass update` first.
3. **Commit it.** Teammates run `yass update` when the repo moves ahead; `yass status` tells anyone whose binary and repo are out of step.

Or ask your agent to "update YASS": `yass-update` does all three, asking before it downloads. Read the [release notes](https://github.com/donjaime/yass/releases) first: before 1.0, a minor version can change the file format or the CLI. [docs/install.md](docs/install.md#upgrading) has the details.

## Removing YASS

Trying YASS is low risk. To take it back out of a repo:

```bash
yass deinit
git add -A && git commit -m "chore: remove YASS"
```

`yass deinit` removes the playbooks, the YASS section of each `AGENTS.md` and the hook, and lists them. Your plans stay: it ends by saying how to remove the `yass/` folder (`git rm -r yass`), or you can keep it as the record of what you changed and why. It never commits, so `git restore .` undoes it, and `--global` removes your user folder's playbooks too. [docs/install.md](docs/install.md#removing-yass) has the details.

## The CLI

```
yass init [dir]                 set up YASS: yass/, and for the repo itself the AGENTS.md section,
                                the playbooks and the hook script; for a folder (dir), only its planning
          [--agents]            …plus, for a folder, its own AGENTS.md section and playbooks (monorepos)
          [--path <folder>]     …or a yass.yaml pointing to a yass folder somewhere else
          [--claude]            …plus Claude Code's copies of the playbooks and the CLAUDE.md import
          [--global]            …with the playbooks in your user folder instead of the repo
          [--hooks]             …and turn the hook on for this clone
          [--no-agents]         …leaving AGENTS.md, the playbooks and the hook alone
yass deinit [--global]          take YASS out of the repo: the playbooks, AGENTS.md sections and hook;
                                your plans stay (--global: your user folder's playbooks too)
yass update                     get a newer yass binary, verified: the repo's version if the repo is
                                ahead, else the latest release
            [--check] [--latest] [--version vX.Y.Z] [--require-provenance]
yass upgrade                    bring every YASS file in the repo, and your user folder's playbooks,
                                up to this binary's version (after yass update)
yass new "<title>"              a small change
         [--large] [--design]   …with prd.md and plan.md, and design.md
         [--in <change>]        …as a piece of a large change
         [--source gh#41] [--follows <archived>] [--platforms "ios, android"] [--goal "…"]
yass status [<change>]          what's in flight, progress, next steps, what's blocked, not started or done,
                                and the unmerged branches working on each
            [--brief]           …or one change's progress in a line, for commits and pull requests
            [--archived] [--strict] [--since 14d|DATE] [--no-branches]
yass archive <change>           move a finished change to archive/<YYYY>/<MM>/ (refuses while boxes are open)
yass evict                      move the oldest archived months into git history, when past keep:
yass decisions                  past decisions, active and archived, newest first, 50 at a time
               [--since D] [--until D] [--about "words"] [--change <c>] [--cites <sha>] [--limit N] [--json]
yass root                       print the yass folder(s) this repo uses
yass paths [--only A...B]       the repo paths that are YASS's, as globs; --only exits 0 for a
                                plans-only commit range, so CI can skip builds and tests
yass template <name>            print a template (design, prd, plan, …), e.g. to add design.md later
yass hook [--range A..B]        the optional hook's check, for CI
```

`yass status` looks like this:

```
2026-10-02-fix-double-tap-save      1/4  next: disable Save while saving, then the unit test.
2026-09-14-offline-sync  (large)   5/16  next: offline-queue first; sync-badge needs it.
  ├ 2026-09-16-offline-queue        4/8  next: drain on `online` for the web, then batch the drain.
  └ 2026-09-24-sync-badge           0/5  BLOCKED: does the web get the badge too?
        ↳ origin/badge-web  Kim, 2 h ago  2/5  next: ask Sam about the web
branches: unmerged work since 2026-09-02; remote branches as of the last fetch, 2 h ago
```

Changes are listed oldest first, unless the yass folder has a [`queue.md`](#the-queue) saying what order to tackle them in.

Under each change or piece, `yass status` lists the branches working on it that haven't merged yet: local branches, and pushed ones as of your last fetch, with commits to that change's folder in the last 30 days. Each line has the branch, who touched it last and when, its progress there, and its latest **Next:**. So you see who's on what, and how far they've got, without anyone claiming work anywhere. It never fetches; the last line says how fresh the pushed branches are, so run `git fetch` first for an up-to-date view. `--since 14d` (or a date) changes the window, and `--no-branches` leaves the view out.

The CLI is one Go binary with no runtime dependencies. It reads the files and moves folders; it never commits, branches or merges.

## The files

Every change has a `change.md`:

```markdown
---
platforms: [ios, android]
source: gh#41
follows:
blocked:
created: 2026-10-02T09:14:07Z
---
# Fix double-tap save crash

## Goal
Tapping Save twice quickly saves one entry and never crashes.

## Acceptance
- [ ] Given the picker, when Save is tapped twice within 300ms, then one entry is saved — verify: unit SaveTests.doubleTap

## Steps
- [x] Reproduce on Android
- [ ] Disable Save while a save is in flight

## Decisions
- Disable the button rather than dedupe in the store - fixes the cause (claude)

## Log
### 2026-10-02 (claude)
- Did: reproduced; two inserts race on the same id.
- Next: disable Save while saving, then the unit test.
```

- **Frontmatter:** `platforms` is what "done" has to cover. `source` is the issue, alert or request it came from. `follows` names the archived change it follows up. `blocked` says why work can't continue: a reason for a human, or the [changes it waits on](#waiting-on-another-change). `created` is when `yass new` made it (UTC), so changes from the same day list in the order they were made; without it, a change sorts by its folder's date. `archived` is when `yass archive` moved it (UTC), and the month folder it's in. All optional.
- **Intent:** the title, `## Goal` and `## Acceptance`. They change only in commits without code.
- **Progress:** `## Steps` (your working checklist), `## Decisions` (`- <decision> - <why> (<who>, <YYYY-MM-DD>)`; `yass decisions` lists them across every change) and `## Log` (ending in `- Next:`, which `yass status` shows). These change with the code.

A large change has a slimmer `change.md` (goal, decisions, log) and adds:
- **`prd.md`:** why, for whom, measurable outcomes, requirements (`R1`), non-goals, milestones, open questions.
- **`plan.md`:** approach, acceptance criteria per milestone (`AC1 (R1) Given … — verify: …`), the order of the pieces, validation.
- **`design.md`** (optional): context, at least two options, the decision and who made it, rollback.
- **Pieces:** PR-sized changes in their own dated folders inside it. Each names the plan criteria it delivers (`- [ ] Delivers AC1, AC3–AC5 in plan.md`). A criterion counts as done in `yass status` once every piece that delivers it is done, and `yass archive` ticks it in the archived `plan.md`, so landing a piece needs no closing commit for the parent. A piece's branch changes only that piece's folder (and code): parallel branches on different pieces then never touch the same file, so they rebase onto each other cleanly, and the hook gives a heads-up when a commit with code reaches past one piece.

### The queue

A yass folder can hold a `queue.md`: the order you want to tackle its changes in, top first. It's optional, and it doesn't have to list everything.

```markdown
# Queue

1. 2026-10-02-fix-double-tap-save — a crash; before more sync work
2. 2026-09-14-offline-sync
```

- **Each list item names a change folder** (`1.` or `-`, plain, in backticks, or as a link). Anything after the name is a note. Headings and prose are ignored.
- **`yass status` follows it:** listed changes first, in order, then the rest, oldest first. It warns about a name that isn't an active change there, an archived change, a piece (a large change's `plan.md` orders its pieces), or a repeat.
- **Reordering is a decision,** so it's intent: commit it without code, and the hook checks that. Creating a queue alongside code is fine, like any new intent file.
- **`yass archive` takes the change off the list** and stages that with the move.
- **The queue informs whoever directs the work.** `yass-work` still works only on the change it's pointed at.

### Waiting on another change

When a change can't start until another is done, put that change's folder in `blocked:` (several, separated by commas):

```yaml
blocked: 2026-09-16-offline-queue
# blocked: services/payments/yass/changes/2026-09-10-saved-cards-api, 2026-09-14-offline-sync/2026-09-16-offline-queue
```

- **A name is a change's folder,** a path ending in one, or `<change>/<piece>`. It can be in another yass folder; a path only matters when two changes share a name.
- **It clears itself.** `yass status` shows `waiting on: <change>` until every change it names is done (every box checked) or archived, then nothing; `yass status <change>` suggests clearing it. `yass archive` refuses only while it's still waiting.
- **Anything else is a reason** for a human, shown as `BLOCKED:` until someone clears it, as before. That includes text that mentions a change (`waiting on 2026-09-16-offline-queue and legal`).
- `yass status` warns about a name that looks like a change folder but isn't one, and about changes that wait on each other.

Checkboxes: `[ ]` not started, `[/]` in progress, `[x]` done, `[-]` dropped (say why in Decisions). Progress counts are just these boxes: `[/]` isn't done yet, and `[-]` doesn't count. The same boxes give each change its state in `yass status`: `not started` (no Log entry, nothing in progress or done), in progress, or `done` (every box done or dropped, ready to archive). Archived means finished.

## The optional hook

`tools/yass/githooks/pre-commit` runs `yass hook`, which warns when a commit mixes code with edits to existing intent: `prd.md`, `design.md`, `plan.md` beyond ticking boxes, or a change's title, Goal or Acceptance. Dropping an acceptance criterion (`[-]`) counts as an intent edit. It also warns on any edit or addition to an already-archived change. Ticked or in-progress boxes, Log and Decisions next to code are expected and never flagged.

- **Turn it on for a clone:** `yass init --hooks`, which sets `git config core.hooksPath tools/yass/githooks`. That setting makes git run hooks from that folder only, so `--hooks` only uses it when the repo has no hooks setup of its own. If it has one (a `core.hooksPath` of its own, local or global, or hooks in `.git/hooks/`), `--hooks` leaves it alone and prints how to wire YASS in; see [Already have hooks?](#already-have-hooks). Without `yass` on the PATH, the hook skips itself with a note.
- **It only warns.** `YASS_STRICT=1` makes it refuse the commit; `YASS_HOOK=off` skips it.
- **In CI**, install `yass` (a release, or `go install github.com/donjaime/yass/cmd/yass@<version>`) and check a whole branch: `yass hook --range origin/main..HEAD --strict`. CI needs the history for that range (with GitHub Actions, `actions/checkout` with `fetch-depth: 0`); the hook fails loudly if it can't read it.

It's a heuristic over markdown diffs. It catches the common way goalposts move (reworded acceptance criteria next to the code that "meets" them). Review and CI do the rest.

### Already have hooks?

Run YASS's check from the setup you have instead of switching to its folder:

- **Plain git hooks** (`.git/hooks/`, or a folder `core.hooksPath` points to): add `tools/yass/githooks/pre-commit "$@"` as a line in that folder's `pre-commit` (create it, executable, if there isn't one).
- **husky:** add `yass hook` to `.husky/pre-commit`.
- **lefthook:** in `lefthook.yml`,
  ```yaml
  pre-commit:
    commands:
      yass:
        run: yass hook
  ```
- **The pre-commit framework:** in `.pre-commit-config.yaml`, under `repos:`,
  ```yaml
  - repo: local
    hooks:
      - id: yass
        name: yass hook
        entry: yass hook
        language: system
        pass_filenames: false
        always_run: true
  ```

## Keeping plans out of the repo

Not every project wants its work in progress in the repo; an open source project might keep it private, or somewhere else entirely. Put a `yass.yaml` where the `yass/` folder would go, pointing to the folder you want:

```yaml
path: ../my-project-plans        # relative to this file
# path: ~/plans/my-project
# path: ${YASS_HOME}/my-project  # an environment variable, with a path under it
# path: ${YASS_PLANS:-../my-project-plans}  # the variable if it's set, else a default
```

`yass init --path <folder>` writes it and creates the folder. From then on everything works as before: `yass new` puts changes there, `yass status` reads them, and `yass root` prints where it resolved. If a variable it uses isn't set (and has no `:-default`), `yass status` says so.

- **`yass/` is a link to it.** `yass init` and every `yass` command keep a `yass/` symlink in the checkout pointing to the plans folder, so you, your editor and your agents find them at `yass/` as usual. It's ignored through the clone's `.git/info/exclude`, so it never shows up in `git status` or in commits. Nothing depends on it: a real `yass/` already there is left alone (with a warning), and where symlinks can't be made (Windows without Developer Mode) YASS skips it with a note. Some tools don't follow links by default (`rg` needs `-L`).
- **Worktrees find the same plans.** A relative path like `../my-project-plans` means "next to the checkout", so from a linked worktree somewhere else YASS looks where the clone's other checkouts see it, and links `yass/` there too. A `yass.yaml` kept out of git is borrowed from the main checkout the same way. If the plans aren't found from any checkout, a worktree never offers to create them: run `yass init` from the main checkout.

- **Commit the `yass.yaml`** so everyone finds the plans the same way (each person sets the variable for their machine), or **gitignore it** to keep even the pointer private. `yass` still reads one at the top of the repo or in the folder you're in.
- **YASS says when plans are separate.** Plans outside this repo's git work tree (in another repo, or in none) are *separate*; plans inside it, a linked worktree's included, are *inline*. `yass status` ends with `plans: separate (<folder>)`, and notes when that folder's repo has uncommitted files or is behind its upstream as of its last fetch (it never fetches). `yass root -v` prints `inline` or `separate` after each folder.
- **The folder can be a git repo of its own,** private if you like. Then no commit can mix code and plans, so the hook has nothing to check; commit intent changes there on their own, as usual. Its history is where Decisions get their dates and authors.
- **Commits name their change.** With a change's plans elsewhere, each code commit for it carries a `yass:` trailer naming the change (`yass: 2026-09-14-offline-sync`), which the hooks add for you (`yass use <change>`, or a branch named after the change); CI can require it. [docs/separate-plans.md](docs/separate-plans.md) has the details and hook managers.
- **Boxes cite the code they rest on,** since progress can't ride in the same commit as the code: `- [x] Disable Save while saving — code: a1b2c3d`. A box is `[x]` once its code is merged, citing the commit on the main branch, and `[/]` until then. `yass status` warns about a cited commit this repo doesn't have, and about a done box whose code isn't merged. `branch:` in `yass.yaml` says which branch counts (default: `origin/HEAD`, then `main`, then `master`).
- **Agents follow a routine for it** (`yass-work`): pull the plans before starting, never `yass init` a missing plans folder, `yass use <change>` so commits carry the trailer, code first and plans second, the two linked, and git latitude asked for separately in each repo.
- **Run your agent from the code repo** and give it the plans folder as an extra directory (Claude Code: `claude --add-dir <folder>`, or `permissions.additionalDirectories`). The `yass/` link is for finding the plans, not for permissions: Claude Code resolves it, treats the files behind it as outside the project, and asks before reading them, even in auto mode. Granting the plans folder is what lets an agent work in it without asking.
- **A path inside the repo works too,** under any name (`path: planning`), and the hook recognizes it.
- **One team can plan privately in a shared repo.** `yass init private --path ~/plans/private --private` makes a team folder that git never sees: it's listed in the clone's `.git/info/exclude`, and named in the clone's git config (`yass.include`) so YASS still finds it, from the top of the repo and from every worktree. Nothing is written to the repo, so each clone sets it up for itself. `yass paths` leaves it out, since no commit can contain it. To stop: `git config --unset yass.include '^private$'`, then delete the folder.

`yass.yaml` is also where YASS settings go as they're added (`path`, `branch`, [`ignore`](#folders-that-arent-yours) and [`archive: keep:`](docs/monorepo.md#keeping-the-working-tree-small) so far). It doesn't have to point anywhere: without `path:`, the yass folder is the `yass/` next to it, so a `yass.yaml` beside your `yass/` folder just holds settings. A setting your version doesn't know is a warning, not an error.

## Monorepos

Any folder named `yass/` with `changes/` or `archive/` inside counts, and so does any `yass.yaml`, so teams opt in by running `yass init services/payments`. `yass status` shows every `yass/` folder in the repo, and `yass new` puts a change in the nearest one to where you're standing. Cross-team work is links, not copies: a root change's plan names the team changes that deliver each criterion, each team change names the criterion it delivers, and a team change that needs another team's work first names it in `blocked:`. Ownership and review rules belong in your CODEOWNERS. Team folders also keep each archive a manageable size: a yass folder archiving more than about 1,000 changes a month is a sign to split it. See [examples/monorepo](examples/monorepo).

Rolling it out across a large monorepo, with expensive CI and many teams? [docs/monorepo.md](docs/monorepo.md) shows how to keep plan-only commits from running builds and tests (including GitHub required checks, merge queues and Bazel), route each team's plans to its reviewers with CODEOWNERS, work in sparse checkouts, store mockups with Git LFS, and leave the plans out of `git log` and `git diff`.

### Folders that aren't yours

Some `yass/` folders belong to something else: examples, test fixtures, a vendored project. List them under `ignore:` in a `yass.yaml` next to your `yass/` folder:

```yaml
ignore:                    # folders or glob patterns, relative to this file
  - examples/*
  - third_party/some-lib
```

Their `yass/` folders and `yass.yaml` files are left out of `yass status`, `yass root` and `yass new`, and the hook treats their files as ordinary files. From inside one, it's a project of its own: `cd examples/solo-app && yass status` shows that example's changes, not yours. That's why `examples/*` beats `examples` when each folder is a separate project. An entry that matches no folder, or one outside the folder `yass.yaml` is in, is a warning. This repo uses it for its [examples](examples).

## Harnesses

The playbooks are [Agent Skills](https://agentskills.io) in `.agents/skills/`, which Codex, OpenCode and pi read natively. Claude Code reads `.claude/skills/`, so `yass init --claude` copies them there and imports `AGENTS.md` from `CLAUDE.md`. The rules themselves live in `AGENTS.md` and the files, so any agent that can read files and run a shell command can follow them.

### Plan mode

YASS is meant to sit next to your harness's plan mode (Claude Code's, or any other), not replace it. Plan mode is good at a session's worth of research and a plan the human approves; a YASS change is where that plan lives after the session ends, with progress and decisions next to it.

- **Plan first, then save it.** Use plan mode as usual. Once you approve the plan, ask the agent to save it as a change: `yass-shape` turns it into a `prd.md` for large work, `yass-plan` into a `plan.md` (or a small change's Goal, Acceptance and Steps). They start from what the plan already says and only ask about what's missing.
- **Plan mode can't write files**, in most harnesses. The playbooks draft there and write once you've approved.
- **Mid-build, plan mode is a good way to rethink.** Whatever it decides becomes an intent change, stacked under the code like any other.

## What YASS leaves to you

Guardrails, approvals, CI, verification gates, merge strategy, issue tracking and unattended agent loops differ from team to team, so YASS doesn't own them. It gives you places to record their results (acceptance criteria with `verify:`, Decisions with who made them, `source:` links to your tracker) and one optional hook for the rule it cares most about. See [docs/comparison.md](docs/comparison.md) for how YASS relates to OpenSpec, Spec Kit and others.

## Examples

- [examples/solo-app](examples/solo-app): a mobile app built by one developer and agents, using YASS from day one. One large change in progress with two pieces, a small bug fix, and an archive.
- [examples/monorepo](examples/monorepo): three teams, one cross-team change, and a team that adopted YASS late.

## Status

**v0.1, early.** The CLI and hook are covered by Go unit tests and an end-to-end test suite (`tests/e2e.sh`), run in CI on Linux and macOS; the Windows builds compile but aren't tested yet. The playbooks have been written for Claude Code, Codex, OpenCode and pi, but haven't been battle-tested in each. Feedback and war stories are very welcome.

## License

MIT. See [LICENSE](LICENSE).

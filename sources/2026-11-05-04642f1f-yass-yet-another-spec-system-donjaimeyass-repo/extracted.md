## AGENTS.md

<!-- yass:begin version=0.4.0 (managed by `yass init`; edit outside these markers) -->
## Changes (YASS)
Planned work lives in the yass folder's `changes/`, one dated folder per change: `yass/` in the repo, or wherever `yass.yaml` points (`yass root` prints it; see its README.md). CLI: `yass`, on your PATH. Playbooks: the `yass-*` skills, in `.agents/skills/` or your user skills folder.
- `yass status` shows what's in flight. If `yass root` is outside this repo, plans and code are committed separately: see "Plans in another folder" in `yass-work`. Work only on the change you're pointed at (unless you've been put in charge of choosing), and resume it from the last **Next:** in its Log. `not started` means planned but not built yet; its plan may still need approval.
- As you work, mark boxes `[/]` when you start them and `[x]` when they're verified, and append to `## Log` and `## Decisions` alongside the code (each decision dated: `- <decision> - <why> (<who>, <YYYY-MM-DD>)`).
- Intent (`prd.md`, `design.md`, plan text, a Goal, acceptance criteria, including dropping one with `[-]`) never changes in the same commit as code (with squash merges, the same pull request), so decisions can be reviewed apart from execution. If the plan is wrong mid-build, put the fix on its own branch underneath the code and rebase onto it (see `yass-work`), or stop and say so.
- Don't commit, branch, rebase, push or open a pull request unless you've been given that latitude; otherwise show the human the git commands.
- When `yass status` notes that the `yass` binary and this repo's YASS files are out of step, or someone asks to update YASS: `yass-update`.
- A plan from plan mode can become a change: `yass-shape` for large work, `yass-plan` or `yass-work` for small.
- Never edit the archive (`archive/` in the yass folder); only `yass evict` removes from it. Follow-up work is a new change with `follows: <archived folder>`.
- Finished means every box is checked; then `yass archive <change>`, in its own commit, apart from code (with squash merges, a pull request without code).
<!-- yass:end -->


## CHANGELOG.md

# Changelog

## 0.5.0 (2026-10-11)
Large changes and many teams in one repo, and plans kept outside it, with less ceremony: parent criteria close themselves, `yass status` shows who's working on what, and commits name a change kept elsewhere for you.
- **Delivered criteria close themselves.** A parent criterion counts as done once every piece that names it in a `Delivers` box is done, and `yass archive` ticks it in the archived `plan.md`. No closing commit per piece.
- **Pieces own their files.** A branch working on a piece changes only that piece's folder, so parallel branches never conflict over progress; the hook gives a heads-up when a commit with code reaches into the parent's files or a second piece.
- **The branch view:** under each change and piece, `yass status` lists the unmerged local and pushed branches with recent commits to it, who touched them last, their progress and latest **Next:**, as of your last fetch. It never fetches. `--since 14d` sets the window; `--no-branches` leaves it out.
- **`yass status <change> --brief`:** a change's progress in one line (a piece's after its parent's), for commit messages and pull requests. `yass-work` suggests it.
- **Plans kept outside the repo:** `yass root -v` says whether each yass folder is inline or separate, and `yass status` notes a separate plans repo with uncommitted files or behind its upstream. A code commit for a change kept outside carries a `yass:` trailer naming it (`yass: <change> [ac:ac1,ac2]`), added by a new `prepare-commit-msg` hook from `yass use <change>` (per worktree), the branch name or earlier commits, and checked by a new `commit-msg` hook. `yass hook --range [--require-trailers] [--strict]` checks them in CI, and `yass status <change>` lists a change's merged code and each criterion's from the trailers, squash merges included. [docs/separate-plans.md](docs/separate-plans.md) has the details and hook managers.
- **Many teams:** `yass status` in a sparse checkout notes a `blocked:` or `follows:` naming a change outside it instead of warning. [docs/monorepo.md](docs/monorepo.md) covers CODEOWNERS per yass folder, Git LFS for mockups and PDFs, and history views without the plans.
- **`yass deinit` takes YASS back out of a repo:** the playbooks, the YASS section of each `AGENTS.md` (and the `CLAUDE.md` import, when nothing else is left), the hook and `core.hooksPath`, listing what it removed. Your `yass/` folders and `yass.yaml` stay; it says how to remove them. `--global` removes your user folder's playbooks too. It never commits, so trying YASS is low risk.
- **Installing is one plain step:** `curl -fsSL …/install.sh | bash` puts the binary in `~/.local/bin` (`--bin-dir` picks another folder). The README, docs/install.md and the YASS page now show installing the binary and putting it on your PATH, setting up a repo, upgrading, and removing YASS as separate steps.
- **`yass archive` warns** about a done box citing a commit that's missing or not merged.
- Fixes: hooks in a linked worktree find a plans repo elsewhere (git calls no longer inherit the hook's `GIT_DIR`); `yass update --version` is the command's own option; the agent playbooks follow the routine for plans kept elsewhere (pull first, never `yass init` a missing plans folder, latitude per repo).

Migrating:
- **Run `yass upgrade`** to install the two new hooks beside `pre-commit` (and the updated playbooks). They do nothing in a repo whose plans are all inline, and skip themselves under an older `yass`, so teammates who haven't updated can still commit.
- **`install.sh` no longer needs `--bin-dir`,** and the documented one-liner no longer runs `yass init`: run it yourself in each repo.
- **`yass status` output** has a line per unmerged branch under its change, and a closing `branches:` line when there are any; `--no-branches` restores the old output for scripts.

## 0.4.0 (2026-10-08)
The archive scales: finished changes go into month folders, YASS reads only their names unless it needs more, and a yass folder can move its oldest months into git history. Past decisions are one command away for people and agents.
- **`yass update` gets a newer binary.** Inside a repo whose YASS files are newer than the binary, it installs exactly that version, so you match your team; otherwise the latest release. It says when a newer release exists (`--latest` gets it) and that `yass upgrade` and a commit follow; `--check` only looks. It verifies before installing: the release's `checksums.txt` always, and its signed build provenance when the GitHub CLI is installed (`--require-provenance` to insist), then checks the new binary runs before swapping it in. Binaries from `go install` or a clone aren't replaced; it prints the command for those.
- **A sixth playbook, `yass-update`:** ask your agent to "update YASS", and it checks both versions, asks before downloading, runs `yass update` and `yass upgrade`, and hands back the diff. `yass status`'s version note and the `AGENTS.md` section point to it.
- **Archives by month.** `yass archive` stamps `archived:` (UTC) in the change's frontmatter and moves it to `archive/<YYYY>/<MM>/<name>/`, so no folder grows without bound and GitHub can list every month. Every command reads both this layout and the old flat one.
- **`yass upgrade` migrates.** After writing YASS's files, it brings each yass folder's layout up to date, including folders `yass.yaml` points to outside the repo: the first step moves a flat `archive/` into months, dated by the commit that archived each change (else the date in its name), and prints how to commit that on its own. The hook accepts the move.
- **`yass decisions`:** every `## Decisions` entry across active and archived changes, in every yass folder, newest first, 50 at a time, with `--since`/`--until`, `--about "words"`, `--change <c>` (which follows the `follows:` chain both ways), `--cites <sha>` (the boxes that cite a commit) and `--json`. `yass-log` and `yass-shape` use it instead of reading `archive/`.
- **Dated decisions.** Entries now read `- <decision> - <why> (<who>, <YYYY-MM-DD>)`; the templates, the `AGENTS.md` section and `yass-work` ask for it. Older entries stay valid and take their change's dates.
- **`keep` and `yass evict`.** A yass folder keeps up to 2,000 archived changes in its working tree (`archive: { keep: N }` in its `yass.yaml`). Past that, `yass status` and `yass archive` say so, and `yass evict` moves the oldest whole months into git history, leaving a small `archive/<YYYY>/<MM>.evicted` file per month. Evicted changes still count for `follows:` and `blocked:`, and `yass decisions` reads them back from git when a query reaches them. [docs/monorepo.md](docs/monorepo.md#keeping-the-working-tree-small) has a scheduled CI recipe.
- **Faster with big archives:** `yass status` reads only archived changes' names; their files are read when a command shows them.
- **`yass init --hooks` leaves existing hooks alone:** with a hooks setup of its own (a `core.hooksPath`, husky, `.git/hooks/`), the repo keeps it, and YASS prints how to run its check from there.
- **Same-day changes list in creation order,** from a `created:` stamp `yass new` writes.
- Docs: what squash merges mean for YASS's rules (a pull request apart from code, where a commit used to be), and pushing and opening pull requests need the same latitude as committing.

Migrating:
- **From 0.3.0, update the binary by hand once** (a release download, `install.sh` or `go install`): `yass update` arrives in 0.4.0, and only release builds from 0.4.0 on can replace themselves.
- **Upgrade everyone's `yass` binary before committing the archive migration on a shared repo.** An older binary reads `archive/2026` as a change, its `yass status` says to upgrade the binary, and it still archives flat; the next `yass upgrade`, by anyone, moves those into months too.
- **To upgrade a repo,** run `yass upgrade` and commit in two steps, as it prints: the archive move on its own (`yass: archive-into-months`), then the rest (`chore: upgrade YASS`).
- **Big archives:** a yass folder already past 2,000 archived changes gets a note from `yass status`; run `yass evict` when you're ready, or set `keep:` higher.

## 0.3.0 (2026-10-05)
Installing and upgrading are now two plain steps. To install: get the `yass` binary, then run `yass init` in your repo. To upgrade: get the newer binary, then run `yass upgrade` in each repo.
- **The binary carries the playbooks and the hook.** `yass init` sets up a whole repo from the binary alone: the playbooks, the hook script, the `AGENTS.md` section and the yass folder, with `--claude`, `--global`, `--hooks` and `--path`. Nothing is downloaded after the binary. Running it again writes only what's missing, and it ends with the next steps (commit it, then ask your agent).
- **`yass upgrade`:** run anywhere in a repo, it finds every YASS file from the repo root down (playbooks in any `.agents/skills/` or `.claude/skills/`, each `AGENTS.md` section, the hook) and your user folder's playbooks, and brings them to the binary's version. It lists what it wrote, says when everything is already up to date, skips folders `yass.yaml` ignores, and never downgrades: if the repo is newer than your binary, it says to upgrade the binary.
- **Version stamps:** every file YASS installs records the version that wrote it: `metadata.yass-version` in a playbook's frontmatter, `version=` in the `AGENTS.md` marker, `# yass-version:` in the hook. `yass status` notes when your binary and a repo are out of step, and which to upgrade.
- **Monorepo folders with their own agent files:** `yass init <folder> --agents` puts the `AGENTS.md` section and the playbooks in that folder, and `yass upgrade` finds them there. The hook stays at the repo root.
- **`install.sh` only installs the binary,** into `--bin-dir` (now required), and says whether that folder is on your PATH, with the line to add it if not (for zsh, in `~/.zshenv`, which agents and git hooks read too). The one-line install runs `yass init` itself. Release archives no longer include `kit/`.
- **Worktrees find separate plans:** a relative `path:` in `yass.yaml` resolves from the clone's other checkouts, so linked worktrees, where agents usually run, find plans kept outside the repo. Paths accept `${VAR:-default}`, and with plans outside the repo, every `yass` command keeps an ignored `yass/` link to them.
- **Private team folders:** `yass init <folder> --path <plans> --private` sets up a folder git never sees, found through the clone's git config, from every worktree.
- **`yass paths`:** prints the repo paths that are YASS's; `--only <range>` exits 0 when a commit range touches nothing else, so CI can skip plan-only commits. [docs/monorepo.md](docs/monorepo.md) has recipes.
- Fixes: change names end on a whole word; paths that don't exist yet compare correctly through symlinks (macOS).
- Building from source needs Go 1.24 or later, so a build from a clone carries a version.

Migrating:
- **`install.sh` no longer sets up or upgrades repos.** An old-style call (a repo path, `--claude`, `--global`, `--hooks`, `--path` or `--upgrade`) installs nothing and prints the commands to use instead. Update scripts that call it: `install.sh --bin-dir <folder>`, then `yass init` or `yass upgrade`.
- **To upgrade a repo,** install the new binary, run `yass upgrade` in the repo, and commit the diff (`chore: upgrade YASS`). Files from 0.2 or earlier carry no version stamp; `yass upgrade` counts them as older and stamps them. As before, hand edits to YASS's own files are replaced, and the diff shows them.
- **`yass init <folder>`** (a team folder) now sets up only that folder's planning. It no longer adds the YASS section to the root `AGENTS.md`; pass `--agents` to give the folder its own agent files. And `yass init` no longer rewrites an existing `AGENTS.md` section; `yass upgrade` does.

## 0.2.0 (2026-10-03)
- **`queue.md`:** an optional file in a yass folder that ranks its changes, top first. `yass status` follows it (unlisted changes come after, oldest first) and warns about entries that aren't active changes there; `yass archive` takes the change off the list; the hook treats reordering alongside code as intent.
- **Dependencies through `blocked:`:** naming one or more changes (a folder name, a path ending in one, or `<change>/<piece>`, comma-separated) means waiting on them. `yass status` shows `waiting on:` until they're done or archived, then nothing; `yass archive` refuses only while it's still waiting. Warnings for names that look like a change but aren't, and for changes that wait on each other.
- **`ignore:` in `yass.yaml`:** folders or glob patterns (`examples/*`) whose `yass/` folders belong to something else. They're left out of `status`, `root`, `new` and the hook; from inside one, it's a project of its own. A `yass.yaml` without `path:` beside a `yass/` folder just holds settings.
- **Upgrades:** `install.sh --upgrade` refreshes the playbooks wherever they're installed (Claude's copies, or your user folder's when the repo has none) without the original flags, and ends with upgrade steps and a link to the release notes. docs/install.md covers upgrading, including the one-line form.
- The playbooks know about the queue and dependencies: `yass-status` answers "what's next?" from them, `yass-work` sets `blocked: <change>` when work has to wait, and `yass-plan` can use it between pieces. Examples: solo-app has a `queue.md`; the monorepo's web change waits on payments through `blocked:`.

Migrating: nothing to do. One behavior changes: a `blocked:` whose value is only change names now clears itself once they're done, where before it stayed until someone cleared it. Run `install.sh --upgrade` for the new playbooks.

## 0.1.0 (2026-10-03)
First public version.
- Changes as dated folders under `yass/changes/`, with ceremony that scales: `change.md` for small work; `prd.md`, `plan.md`, optional `design.md` and PR-sized pieces for large work.
- `yass/archive/` for finished changes; append-only, with `follows:` for follow-ups.
- CLI: one Go binary with no runtime dependencies. `init`, `new`, `status`, `archive`, `root`, and `hook` (the optional pre-commit check, also for CI). `status` marks changes `not started` or `done`.
- `yass.yaml`: keep the yass folder outside the repo (or under another name), by relative path, `~`, or environment variables. Also the home for future settings.
- Boxes can cite code commits (`— code: a1b2c3d`); `yass status` warns about commits the repo doesn't have, and about done boxes whose code isn't on the merged branch (`branch:` in `yass.yaml`).
- Releases for macOS, Linux and Windows (amd64, arm64) with checksums and build provenance, built by GoReleaser from tags.
- Checkboxes: `[ ]` not started, `[/]` in progress, `[x]` done, `[-]` dropped.
- An optional pre-commit hook that warns when code and edits to existing intent share a commit; `yass hook --range … --strict` for CI.
- Playbooks (Agent Skills): yass-shape, yass-plan, yass-work, yass-status, yass-log.
- `install.sh`, with `--bin-dir`, `--path`, `--global`, `--claude` and `--hooks`. Install from a release, from source, or with an agent prompt that verifies before it runs (docs/install.md).
- Examples: a solo mobile app and a three-team monorepo.


## CLAUDE.md

@AGENTS.md


## CONTRIBUTING.md

# Contributing

Thanks for helping. YASS tries to stay small. Before adding something, ask whether it belongs in the **README rules** (where things go and what done means), a **playbook** (how an agent does a job), the **CLI** (a view or a file operation that's tedious or error-prone by hand), or **nowhere**: most workflow choices (approvals, merging, CI, verification) belong to each team's repo, not to YASS.

## Layout
```
cmd/yass/                           the CLI's entry point
internal/yass/                      the CLI and the hook check (`yass hook`): Go, two dependencies (yaml.v3, x/mod/semver)
internal/yass/templates/            what `yass init` and `yass new` write, built into the binary
install.sh                          installs the yass binary (yass init and yass upgrade handle repos)
kit/                                what `yass init` and `yass upgrade` put in a repo, embedded in the binary (kit/embed.go)
kit/tools/yass/githooks/pre-commit  the optional hook: a shim that runs `yass hook`
kit/.agents/skills/                 the playbooks (Agent Skills format), unstamped; `yass.yaml` ignores kit/
.goreleaser.yaml                    release builds; .github/workflows/ runs CI and releases
docs/  examples/  tests/
```

## Principles
- **No state outside the files.** Everything the CLI knows comes from folders, checkboxes and, if there is one, `yass.yaml`. Add a setting there only when a team genuinely needs to choose; the defaults should be enough for most.
- **The CLI never commits, branches or merges.** It reads files and moves folders.
- **No runtime dependencies.** One static binary, so any harness can shell out to it. Keep build dependencies rare too.
- **Honest docs.** If something isn't tested against a real harness, say so.

## Tests
```bash
go test ./...                                  # unit tests
go build -o bin/yass ./cmd/yass
YASS_BIN=$PWD/bin/yass tests/e2e.sh            # installs into temp repos and walks the lifecycle, hook included
YASS_BIN=$PWD/bin/yass tests/examples.sh       # every example passes `yass status --strict`
YASS_BIN=$PWD/bin/yass tests/bench.sh          # times status and the hook on generated large repos (~1 min; not in CI)
```
Without `YASS_BIN`, the test scripts build the binary themselves. CI runs all of it on Linux and macOS, then cross-builds every release target with GoReleaser. Please add an e2e check for any new behavior. If a change could slow `yass status` or the hook down, run `tests/bench.sh`: it fails when a speed target is missed.

## Releases
Releases are cut from tags; nothing else publishes.

1. Add a `## X.Y.Z (YYYY-MM-DD)` section to `CHANGELOG.md` on main. It becomes the release notes, and the release fails without it.
2. Tag and push: `git tag vX.Y.Z && git push origin vX.Y.Z`. A tag with a suffix (`v0.2.0-rc.1`) is published as a pre-release.
3. The release workflow runs the tests, builds every platform with GoReleaser, publishes the GitHub Release with the archives and `checksums.txt`, and attests their provenance.

Versions follow semver. Before 1.0, a minor version may change the file format or the CLI; say how to migrate in the changelog. `go install github.com/donjaime/yass/cmd/yass@vX.Y.Z` works from any tag.


## README.md

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


## cmd/yass/main.go

// Command yass is the YASS CLI. See https://github.com/donjaime/yass.
package main

import (
	"os"
	"runtime/debug"

	"github.com/donjaime/yass/internal/yass"
)

// version is set by release builds: -ldflags "-X main.version=1.2.3". channel is "release" in
// release builds (-X main.channel=release), so yass update knows it may replace this binary.
var (
	version = "dev"
	channel = ""
)

func main() {
	if version == "dev" {
		if bi, ok := debug.ReadBuildInfo(); ok && bi.Main.Version != "" && bi.Main.Version != "(devel)" {
			version = bi.Main.Version // go install github.com/donjaime/yass/cmd/yass@v1.2.3
		}
	}
	os.Exit(yass.MainChannel(os.Args[1:], version, channel))
}


## docs/comparison.md

# How YASS compares

*As of October 2026. Corrections welcome.*

**Short version:** YASS is a thin, file-based project-management layer: change folders whose ceremony scales with scope, progress and decisions recorded next to the code, and an archive that tells the story of how the system evolved. It deliberately leaves guardrails, approvals and verification to your repo. Most spec tools are built around a fixed multi-phase process. Others overlap with the code and documentation and create opportunity for drift.

| | Unit of work | Large work | Current-state spec | Process | Footprint |
|---|---|---|---|---|---|
| **OpenSpec** | a change folder: proposal, delta specs, design, tasks | one change per proposal; no layer above it | yes: `specs/`, updated when a change is archived | propose → apply → archive | CLI (Node) + slash commands for 30+ tools |
| **GitHub Spec Kit** | a feature: spec → plan → tasks | per feature | mostly spec-first | constitution + phased commands | CLI + templates |
| **Kiro** | a spec: requirements (EARS) → design → tasks | per spec | per spec | IDE-guided | Kiro IDE / CLI |
| **GSD** | phases from a project roadmap | project roadmap + state file | — | phase commands with verification | slash commands |
| **BMAD** | PRD → architecture → stories, via agent personas | yes, heavy | documents | role-based | many agents and templates |
| **YASS** | a change folder: one `change.md` for small work; `prd.md`, `plan.md`, `design.md` and PR-sized pieces for large work | the same folder, with pieces inside | no: the code and its docs are the current state | none imposed; five optional playbooks | small CLI (one Go binary), plain markdown, optional hook |

## Against OpenSpec, specifically

OpenSpec is the closest neighbor and very good at what it does. The differences are deliberate:
- **No current-state spec.** OpenSpec merges each change's delta into living `specs/`. YASS describes evolution only and leaves "what exists today" to the code and its docs, so there's no second description to keep in sync. If you want per-capability behavior specs, use OpenSpec for those; the two can live side by side.
- **Ceremony scales inside one folder.** A bug fix is a single file; a new system gets a PRD, a plan, a design doc and pieces, without a separate roadmap or epic layer.
- **Progress lives in the change.** Ticked boxes, decisions with who made them, and a log ending in **Next:** let any agent on any harness resume from the files alone.
- **Finishing is separate from merging.** A change can span many pull requests and plan revisions before it's archived.

## When not to use YASS
- A prototype you'll throw away. Use plan mode and tests.
- You want a maintained, per-capability description of current behavior. Use OpenSpec or similar.
- You want the tool to enforce approvals, verification or merge rules. YASS records them; your CI and review enforce them.


## docs/install.md

# Installing YASS

Getting YASS takes two steps, like most developer tools:

1. **[Install](#install)** the `yass` binary and put it on your PATH, once per machine. It's a single Go binary with no runtime dependencies.
2. **[Set up a repo](#set-up-a-repo)** with `yass init`, once per repo, and commit what it writes, so everyone who clones the repo gets it. A repo that [already uses YASS](#joining-a-repo-that-already-uses-yass) needs no setup.

Later, [upgrading](#upgrading) is the same two steps (`yass update`, then `yass upgrade` in each repo), and [`yass deinit`](#removing-yass) takes YASS back out of a repo.

## Install

### With the script

```bash
curl -fsSL https://raw.githubusercontent.com/donjaime/yass/main/install.sh | bash
```

It downloads the latest release for your machine (macOS or Linux), checks it against the release's `checksums.txt`, and copies `yass` into `~/.local/bin`. That's all it does: it doesn't set up a repo or edit your shell files. `--bin-dir <folder>` picks another folder (`… | bash -s -- --bin-dir /usr/local/bin`), and `YASS_VERSION=v0.5.0` a version. The checksum protects against a corrupted download, not against the script itself, so [read it](../install.sh) first, or install from a release below.

### From a release

Each release has an archive per platform (`yass_<os>_<arch>`, with `os` one of `darwin`, `linux`, `windows`, and `arch` one of `amd64`, `arm64`), a `checksums.txt`, and signed build provenance.

```bash
os=darwin arch=arm64   # or linux/amd64, linux/arm64, darwin/amd64
curl -fsSLO "https://github.com/donjaime/yass/releases/latest/download/yass_${os}_${arch}.tar.gz"
curl -fsSLO https://github.com/donjaime/yass/releases/latest/download/checksums.txt
grep "yass_${os}_${arch}.tar.gz" checksums.txt | shasum -a 256 -c    # or sha256sum -c
gh attestation verify "yass_${os}_${arch}.tar.gz" --repo donjaime/yass   # optional; needs the GitHub CLI
tar -xzf "yass_${os}_${arch}.tar.gz"
"yass_${os}_${arch}/install.sh"
```

To pin a version, replace `latest/download` with `download/v0.5.0`. On Windows, download the `.zip` and run `install.sh` from Git Bash, or copy `yass.exe` into a folder on your PATH yourself.

### From source

With Go 1.24 or later (earlier versions build a `yass` with no version, which won't write YASS's files):

```bash
go install github.com/donjaime/yass/cmd/yass@latest
```

That puts it in `$(go env GOPATH)/bin`. Or build a clone, which `install.sh` copies into `~/.local/bin`:

```bash
git clone https://github.com/donjaime/yass && cd yass
git checkout v0.5.0                       # optional: a released version rather than main
go build -o bin/yass ./cmd/yass
./install.sh
```

### Put it on your PATH

You, your agents and the git hook all run `yass` by name, so the folder it's in has to be on your PATH. `install.sh` says whether it is. If it isn't, it prints the line that adds it for your shell; `~/.local/bin` isn't on PATH by default on macOS. For zsh:

```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshenv
```

`~/.zshenv`, not `~/.zshrc`: every zsh reads it, including the non-interactive shells agents and git hooks run. For bash, use `~/.bashrc` (`~/.bash_profile` on macOS); for fish, `fish_add_path ~/.local/bin`. Then open a new terminal and check it:

```bash
yass --version
```

## Set up a repo

From inside the repo:

```bash
yass init
```

It writes the playbooks in `.agents/skills/`, a short section in `AGENTS.md`, an empty `yass/` folder for your changes, and the optional hook in `tools/yass/githooks/`, and lists them. The binary carries these files, so nothing is downloaded. Then commit them, so teammates get them:

```bash
git add -A && git commit -m "chore: adopt YASS"
```

Its options:

| Option | What it does |
|---|---|
| `--claude` | also copies the playbooks to `.claude/skills/` and imports `AGENTS.md` from `CLAUDE.md`, for Claude Code |
| `--hooks` | turns on the optional hook for this clone (`git config core.hooksPath tools/yass/githooks`), unless the repo already has a hooks setup (its own `core.hooksPath`, local or global, or hooks in `.git/hooks/`): then it leaves that alone and prints how to run YASS's check from it ([README](../README.md#already-have-hooks)) |
| `--global` | puts the playbooks in your user folder (`~/.agents/skills/`, or `$YASS_SKILLS_DIR`; `~/.claude/skills/` with `--claude`) for every repo, instead of in this one. The default keeps them in the repo, so everyone working on it gets the same version. Check that your harness reads the user folder you pick |
| `--path P` | keeps the yass folder outside the repo: writes a `yass.yaml` pointing to `P`, creates it, and links `yass/` to it (ignored through `.git/info/exclude`; see [the README](../README.md#keeping-plans-out-of-the-repo)) |
| `<folder> [--agents]` | sets up a folder of a monorepo instead: its own yass folder, and with `--agents` its own `AGENTS.md` section and playbooks, for a part of the codebase that keeps its own agent files. The hook stays at the repo root |

Running `yass init` again writes only what's missing and keeps the rest; to bring YASS's files up to a newer version, use [`yass upgrade`](#upgrading). Each further repo is just `yass init` again.

### Joining a repo that already uses YASS

Its files are committed, so you need only the binary: [install](#install) it and run `yass status` in the repo. If the repo is on a newer YASS than your binary, `yass status` says so, and `yass update` gets that version.

### With your agent

Paste this into your coding agent from inside the repo you want to set up. It does both steps the way a careful person would: verifies the download, reads the script, and asks before running anything.

```text
Install YASS (https://github.com/donjaime/yass) and set it up in this repository. Go step by step,
show me what you find, and don't run anything that changes files until I say so.

Part 1, the binary:
1. Work out this machine's OS (darwin, linux or windows) and CPU (amd64 or arm64).
2. In a temporary folder, download yass_<os>_<arch>.tar.gz (.zip on Windows) and checksums.txt
   from https://github.com/donjaime/yass/releases/latest/download/.
   If I've said "from source" instead: clone https://github.com/donjaime/yass, check out the
   latest release tag, and build it with `go build -o bin/yass ./cmd/yass` (Go 1.24 or later);
   skip step 3.
3. Check the archive's SHA-256 against its line in checksums.txt. If the GitHub CLI is installed,
   also run `gh attestation verify <archive> --repo donjaime/yass`. Stop if either check fails.
4. Unpack it and read install.sh in full. Tell me what it will do: it should only copy the yass
   binary into a folder (~/.local/bin, or the one passed to --bin-dir) and say whether that folder
   is on my PATH. Point out anything that looks unexpected.
5. Ask me where to put the binary, then once I say go, run install.sh (with --bin-dir <folder>
   if I picked another). If the folder isn't on my PATH, show me the line that adds it
   (install.sh prints one); don't edit my shell files.

Part 2, this repo:
6. Ask me which options I want: Claude Code files (--claude), playbooks in my user folder rather
   than this repo (--global), the optional pre-commit hook (--hooks), or keeping the yass folder
   outside this repo (--path <folder>).
7. Run <folder>/yass init with my options, then yass status to check it works, and show me
   git status. Don't commit; I'll review it and commit it myself.
```

## Upgrading

Upgrading is the same two steps as getting YASS: the binary, once per machine, then each repo's files, once per repo.

```bash
yass update                                          # 1. the binary
yass upgrade                                         # 2. in each repo: its YASS files
git add -A && git commit -m "chore: upgrade YASS"    # 3. so teammates get them
```

Read the [release notes](https://github.com/donjaime/yass/releases) first: before 1.0, a minor version can change the file format or the CLI, and the notes say how to migrate. Or ask your agent to "update YASS": the `yass-update` playbook does all three, asking before it downloads.

### 1. The binary: `yass update`

`yass update --check` shows what it would do, changing nothing.

- **Which version:** inside a repo whose YASS files are newer than your binary, exactly that version, so you match your team; otherwise the latest release. When a newer release exists than the one it installs, it says so: `yass update --latest` gets it. `--version vX.Y.Z` picks one. It never downgrades.
- **What it checks:** it refuses unless the download matches the release's `checksums.txt`. With the [GitHub CLI](https://cli.github.com) installed and signed in, it also verifies the release's signed build provenance (`gh attestation verify`), and refuses if that fails; without it, it says provenance wasn't checked. `--require-provenance` refuses to install without that check. The new binary must run and report the version asked for before it replaces the old one, which happens by a rename in the same folder (on Windows, the old one is set aside as `yass.exe.old` and removed on the next run).
- **When it won't:** a binary built with `go install` or from a clone isn't replaced; it prints `go install github.com/donjaime/yass/cmd/yass@<version>`, or how to rebuild the clone. A binary in a folder you can't write to: rerun with permission to write there, or install into a folder of your own. From a release before 0.4.0, which has no `yass update`, [install](#install) again once.

`yass --version` shows what you have.

### 2. Each repo: `yass upgrade`

Run it from anywhere in the repo. It finds every YASS file from the repo root down, with `git ls-files`: playbooks in any folder's `.agents/skills/` or `.claude/skills/` (so monorepo folders set up with `yass init <folder> --agents` are included), every `AGENTS.md` with the YASS section, and the hook. It adds your user folder's playbooks, if you installed them with `--global`, and skips folders `yass.yaml` lists under `ignore:`.

What it does with each file depends on the version stamped in it:
- **Older, or no stamp** (files written by v0.2 or earlier): replaced with this binary's version. A skills folder also gets any playbook it's missing.
- **The same version:** left alone. When nothing needs doing, it says everything is already up to date.
- **Newer:** nothing is written, and it says to run `yass update` first. `yass upgrade` never downgrades.

Your changes, `yass.yaml` and anything outside the `yass:begin`/`yass:end` markers in `AGENTS.md` are left alone. Hand edits to YASS's own files are replaced; since they're committed, the diff shows them. Files outside version control (user-folder playbooks) are labeled as such in the list of what it wrote.

Then it brings each yass folder's layout up to date, including folders `yass.yaml` points to outside the repo and private team folders. Each step does only what's needed, says what it did, and prints how to commit that on its own, apart from the upgrade commit and from code:
- **archive-into-months:** moves changes sitting directly in `archive/` into `archive/<YYYY>/<MM>/`, by when they were archived (the commit that added them there, else the date in their name), and stamps `archived:` on each. Upgrade everyone's `yass` binary before committing it on a shared repo: older binaries don't know the month folders.

### 3. Commit it

Review the diff and commit it (`chore: upgrade YASS`). The binary is per machine and the files are per repo: upgrading the binary affects every repo on your machine, so upgrade each repo you use YASS in. Teammates run `yass update` when the repo moves ahead; `yass status` tells anyone whose binary and repo are out of step, and which to upgrade.

## Removing YASS

From anywhere in the repo:

```bash
yass deinit
```

It removes what `yass init` and `yass upgrade` wrote, from the repo root down, and lists it: every YASS playbook (`yass-*` in any `.agents/skills/` or `.claude/skills/`), the YASS section of every `AGENTS.md` (and the file, when nothing else is in it, with the `@AGENTS.md` import `--claude` added to `CLAUDE.md`), the hook scripts, and `core.hooksPath` when it points at them. It doesn't ask first: it never commits, and everything it removes is in git, so `git status` shows it and `git restore .` undoes it.

Your plans stay: the `yass/` folders and `yass.yaml` are the record of what was changed and why, and `yass deinit` ends by saying how to remove them (`git rm -r yass`) if you'd rather not keep them. It also leaves alone:
- **Your user folder's playbooks,** which serve every repo; `yass deinit --global` removes them too.
- **A `yass hook` line in husky, lefthook or pre-commit's config,** which it points out for you to remove.
- **The binary.** Delete it once you're done with YASS everywhere (`rm "$(command -v yass)"`).

Then commit it: `git add -A && git commit -m "chore: remove YASS"`. `yass init` sets the repo up again whenever you like.


## docs/monorepo.md

# YASS in a large monorepo

YASS works best with plans next to the code, in the same repo, so progress lands in the same commit as the code it describes. That holds in a monorepo with many teams and expensive CI too. This page shows how to keep plans cheap there. For the folder layout (a `yass/` per team, cross-team changes as links), see [Monorepos in the README](../README.md#monorepos).

## Plan-only commits run no CI

Most YASS edits ride along with code: a box ticked in the same commit as the change it describes. The commits that touch *only* plans are intent changes (a new PRD, a revised plan) and archives. They have nothing to build or test.

`yass paths` prints what's YASS's in this repo, as globs: each yass folder inside the repo, and each `yass.yaml`. Folders a `yass.yaml` ignores aren't listed, and neither is a plans folder outside the repo. With a range, it answers the question CI needs:

```bash
yass paths                       # e.g. yass/**, services/payments/yass/**, yass.yaml
yass paths --only "$BASE...$HEAD"
```

`--only` exits **0** when every file changed in the range is YASS's (skip the build), **1** when anything else changed, or nothing did (run it), and **2** when it can't read the range (usually a shallow clone: fetch history first). Use three dots, so a branch is compared with where it left its base, not with commits that landed there since.

Run the check in one small job at the start, and gate everything else on it. Don't skip whole pipelines with your CI's own path filters unless you've checked what that does to required checks (below).

### GitHub Actions

GitHub treats the two ways of skipping differently ([troubleshooting required status checks](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/collaborating-on-repositories-with-code-quality-features/troubleshooting-required-status-checks)):

- A **workflow** skipped by `paths:`, `branches:` or a commit message leaves its checks **Pending**, and a pull request that requires them can't merge.
- A **job** skipped by `if:` reports **Success**.

So filter with a job, not with `on: paths`, and make one always-running job the check you require:

```yaml
on:
  push:
    branches: [main]   # your default branch; a PR's commits are checked as the PR
  pull_request:
  merge_group:

# A new push to a PR supersedes its checks still running; the default branch and the queue finish theirs.
concurrency:
  group: ci-${{ github.event.pull_request.number || github.ref }}
  cancel-in-progress: ${{ github.event_name == 'pull_request' }}

jobs:
  changes:
    runs-on: ubuntu-latest
    outputs:
      code: ${{ steps.only.outputs.code }}
    steps:
      - uses: actions/checkout@v4
        with: { fetch-depth: 0 }
      - name: Install yass
        run: |
          v=vX.Y.Z   # pin a release that has `yass paths`
          curl -fsSLO "https://github.com/donjaime/yass/releases/download/$v/yass_linux_amd64.tar.gz"
          curl -fsSLO "https://github.com/donjaime/yass/releases/download/$v/checksums.txt"
          grep yass_linux_amd64.tar.gz checksums.txt | sha256sum -c
          tar -xzf yass_linux_amd64.tar.gz && echo "$PWD/yass_linux_amd64" >> "$GITHUB_PATH"
      - id: only
        env:
          BASE: ${{ github.event.pull_request.base.sha || github.event.merge_group.base_sha || github.event.before }}
        run: |
          # Plans only: skip. Anything else, or no usable base (a new branch): run everything.
          if [ -n "$BASE" ] && [ "$BASE" != 0000000000000000000000000000000000000000 ] &&
             yass paths --only "$BASE...$GITHUB_SHA"; then
            echo code=false >> "$GITHUB_OUTPUT"
          else
            echo code=true >> "$GITHUB_OUTPUT"
          fi

  test:
    needs: changes
    if: needs.changes.outputs.code == 'true'
    runs-on: ubuntu-latest
    steps:
      - run: echo "your build and tests"

  # Require this one in branch protection, not the jobs it gates.
  ci-ok:
    if: always()
    needs: [changes, test]
    runs-on: ubuntu-latest
    steps:
      - if: contains(needs.*.result, 'failure') || contains(needs.*.result, 'cancelled')
        run: exit 1
      - run: echo "ok"
```

Limit `push` to your default branch: on any other branch, a first push has no `before` commit to compare with, so the check falls back to running everything, and the PR's own run would check the same commit again. A job that `needs:` a skipped job is skipped too, so a whole chain of jobs behind `test` drops out on a plan-only change, and `ci-ok` still reports. This repo's own [ci.yml](../.github/workflows/ci.yml) works this way, building `yass` from source instead of downloading it.

If you'd rather not install `yass` in CI, paste the output of `yass paths` into a plain diff check, and update it when a team adopts YASS:

```bash
! git diff --name-only "$BASE...$GITHUB_SHA" | grep -qvE '^(yass/|services/payments/yass/|yass\.yaml$|services/[^/]+/yass\.yaml$)'
```

### Merge queues

> **Not tested on a real merge queue yet.** This section follows GitHub's documentation; if you run it, please [tell us how it went](https://github.com/donjaime/yass/issues).

A merge queue runs your checks again on the `merge_group` event. That event takes `branches` filters but no `paths` filters ([events that trigger workflows](https://docs.github.com/en/actions/writing-workflows/choosing-when-your-workflow-runs/events-that-trigger-workflows#merge_group)), so a workflow can't skip itself there by path. The gate job above handles it: on `merge_group` it compares the queue's base with the merge group's commit, so a plan-only pull request still passes through the queue, but without running the build. It still takes its turn in the queue; it just doesn't hold the queue up for long.

### Bazel

> **Not tested end to end yet.** If you try this on a Bazel repo, please [tell us how it went](https://github.com/donjaime/yass/issues).

YASS folders hold Markdown and images, never build files, so:

- **List every yass folder in `.bazelignore`** (the output of `yass paths`, without the `/**`), so `bazel build //...` and `bazel query` never look inside them.
- **Keep them out of targets.** A `glob(["**"])` in a root `BUILD` file (a docs bundle, say) would pull plans into a target, and every plan edit would invalidate it. Exclude them, or glob only what the target needs.
- **Affected-target tools** ([bazel-diff](https://github.com/Tinder/bazel-diff), [target-determinator](https://github.com/bazel-contrib/target-determinator)) then find no affected targets for a plan-only change, so nothing builds. You can also skip running them at all with `yass paths --only` first, the same way as the gate job above.

### Any other CI

Tried YASS with Buck, Pants, Nx, Buildkite, GitLab CI or something else? [Tell us](https://github.com/donjaime/yass/issues) what worked, and we'll list it here.

The same three lines work in Buildkite, Jenkins, GitLab CI or a shell script. Decide on a base (the target branch's merge base for a change under review, the previous commit for a push) and:

```bash
if yass paths --only "$BASE...$HEAD"; then
  echo "plans only: skipping build and tests"
  exit 0
fi
# …the build and tests
```

In Buildkite, run this as the step that uploads the rest of the pipeline, and upload nothing when it exits 0.

## Many teams in one repo

Each team gets its own yass folder (`yass init services/search`), so its changes, archive and `keep:` setting are its own. `yass status` shows them all; `yass new` puts a change in the one nearest where you're standing.

### Who reviews plans: CODEOWNERS

Route each yass folder's plans to the team that owns it, and the root folder (cross-team changes) to whoever owns those:

```
# .github/CODEOWNERS (or CODEOWNERS at the root; GitLab uses the same file)
/yass/                   @acme/architecture
/services/search/yass/   @acme/search
/services/web/yass/      @acme/web
/yass.yaml               @acme/platform
```

A pull request that changes a team's PRD, plan or progress then asks that team to review it, and a cross-team change's plan goes to the people who coordinate across teams. Keep these lines below any broader rule for the same paths: the last matching line wins.

### Seeing who's working on what

`yass status` lists, under each change and piece, the unmerged branches with recent commits to its folder ([README](../README.md#the-cli)). It reads what's already in your clone, in a few git commands whatever the number of branches: the branch refs, one `git log` over them limited to the yass folders, and the changed files at their tips. It never fetches, so it's as fresh as your last `git fetch`, which its last line says; a branch deleted on the server stays listed until `git fetch --prune`. In a blobless partial clone it never downloads a branch's files to read its progress: such branches show without it.

On a large repo, keep git recent (2.44 or later, so a partial clone is never asked to download) and let it keep a commit graph with changed-path filters, which makes walking many branches by path fast:

```bash
git config fetch.writeCommitGraph true             # refresh the commit graph on every fetch
git commit-graph write --reachable --changed-paths # or build it once now
```

With 1,000 recently active remote branches on a repo of 200 yass folders and 2,000 changes, `yass status` takes about 1.5s on a 2024 laptop; with 100, the view adds next to nothing. `--no-branches` leaves it out, for scripts.

### Sparse checkouts

In a sparse checkout, `yass status` reports on the yass folders you have checked out and says nothing about the rest. A change that names one outside your checkout, in `blocked:` or `follows:`, gets a note saying where it is and that it can't be checked from here, not a warning, so `yass status --strict` still passes. To see it, add its folder (`git sparse-checkout add services/search`).

### Images, PDFs and other binaries

A mockup, a screenshot or a spec PDF belongs in the change folder it's about (`yass/changes/<change>/assets/`), and moves with the change when it's archived, so the archive keeps a change's evidence with its decisions. Store them with [Git LFS](https://git-lfs.com), so clones and CI fetch them only when they're read:

```
# .gitattributes
**/yass/**/*.png  filter=lfs diff=lfs merge=lfs -text
**/yass/**/*.jpg  filter=lfs diff=lfs merge=lfs -text
**/yass/**/*.gif  filter=lfs diff=lfs merge=lfs -text
**/yass/**/*.pdf  filter=lfs diff=lfs merge=lfs -text
```

### History without the plans

Plans live next to the code, so `git log` and `git diff` show both. `yass paths` prints the paths that are YASS's; turned into exclusions, they leave the plans out:

```bash
yass paths | sed 's/^/:(exclude,glob)/' | xargs git log --oneline -- .        # commits that touch code
yass paths | sed 's/^/:(exclude,glob)/' | xargs git diff --stat main... -- .  # a branch's code changes
```

As an alias, so it's `git codelog` from then on (any `git log` options pass through):

```bash
git config alias.codelog '!f() { yass paths | sed "s/^/:(exclude,glob)/" | xargs git log "$@" -- .; }; f'
```

`git blame` needs nothing: a commit that touches only plans never changes a line of code, so it never shows up in a code file's blame. The other way around, `git log -- yass/` (or `yass decisions`) is the plans' own history.

## Squash merges

A squash merge folds a pull request into one commit on main. YASS's rules are about commits: intent (a PRD, a design, plan text, a change's Goal or Acceptance), an archive move and a `queue.md` reorder each land apart from code, and progress (boxes, Log, Decisions) lands with the code it describes. With squash merges, "apart from code" means a pull request apart from code:

- **A plan revision, an archive move or a queue reorder goes in a pull request with no code in it.** It can share that pull request with other intent, progress or an archive: a PRD revision with its plan, or a change's last progress with its archive move. These are the pull requests [the recipes above](#plan-only-commits-run-no-ci) let skip build and tests.
- **Progress goes in the pull request of the code it describes,** as it would in a single commit.

The hook checks commit by commit, so it can't see a pull request that will squash an intent commit and a code commit together; keeping them apart is up to whoever opens it. How you get there (separate branches, a stack of pull requests, a stacking tool) is your team's call.

## Big archives

Each yass folder's `archive/` keeps finished changes in `<YYYY>/<MM>/` folders, by the month `yass archive` moved them, so no one folder grows without bound and GitHub can still list them all (it shows up to 1,000 entries in a folder). A team archiving more than about 1,000 changes a month in one yass folder should split it into team folders (`yass init <team folder>`): each team's archive then stays small on its own, and the repo's scales with them. A month that occasionally passes 1,000 is fine.

### Keeping the working tree small

A yass folder keeps up to 2,000 archived changes in its working tree: each one adds a little to every `yass status` (git walks its folders, about +170ms at 2,000) and about 10KB to every checkout, and a monorepo's yass folders add up. That's about a year for a 10-person team. Past that, `yass status` and `yass archive` say so, and `yass evict` moves its oldest whole months out: it deletes each month's folder and leaves `archive/<YYYY>/<MM>.evicted`, a short file naming the month's changes and the commit that still has them. Commit that on its own. Nothing is lost: evicted changes still count for `follows:` and `blocked:`, `yass status --archived` lists how many were evicted and from which months, `yass decisions` reads their decisions back from git when a query reaches them (`--since`, `--change`, or `--cites <sha>`), and `git show <commit>:<path>/<name>/change.md` reads one by hand. A shallow clone lacks the commits; `yass decisions` says so, and `git fetch --unshallow` gets them. The limit is soft: going past it changes nothing until someone runs `yass evict`. Set it per yass folder in its `yass.yaml`:

```yaml
archive:
  keep: 1000   # archived changes to keep in the working tree (default 2000)
```

Eviction keeps the working tree, GitHub's folder views and YASS's own commands small. It doesn't shrink `.git`: every evicted file is still in history, which is what lets you read it back. If clone size matters, use a partial clone (`git clone --filter=blob:none`), which fetches old files only when they're read, or keep the plans in a repo of their own ([`yass.yaml`'s `path:`](../README.md#keeping-plans-out-of-the-repo)).

To never think about it, let CI evict on a schedule and open a pull request for someone to merge:

```yaml
name: yass evict
on:
  schedule: [{ cron: "0 6 * * 1" }]   # Mondays
  workflow_dispatch:
permissions: { contents: write, pull-requests: write }
jobs:
  evict:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with: { fetch-depth: 0 }   # eviction records the last commit that touched each month
      - name: Install yass
        run: |
          v=vX.Y.Z   # pin a release that has `yass evict`
          curl -fsSLO "https://github.com/donjaime/yass/releases/download/$v/yass_linux_amd64.tar.gz"
          curl -fsSLO "https://github.com/donjaime/yass/releases/download/$v/checksums.txt"
          grep yass_linux_amd64.tar.gz checksums.txt | sha256sum -c
          tar -xzf yass_linux_amd64.tar.gz && echo "$PWD/yass_linux_amd64" >> "$GITHUB_PATH"
      - name: Evict, and open a pull request if anything moved
        env: { GH_TOKEN: "${{ github.token }}" }
        run: |
          yass evict
          git diff --cached --quiet && exit 0
          git switch -c "yass-evict-$(date +%Y-%m-%d)"
          git -c user.name="yass evict" -c user.email="actions@users.noreply.github.com" commit -q -m "yass: evict archived months"
          git push -u origin HEAD
          gh pr create --fill
```

A pull request opened with `github.token` doesn't start other workflows; it needs only a plans-only check ([above](#plan-only-commits-run-no-ci)), or use a token of your own if your required checks must run on it.


## docs/separate-plans.md

# Plans in another repo: commits name their change

When a change's plans live outside the code repo (a `yass.yaml` whose `path:` points elsewhere, see the [README](../README.md#keeping-plans-out-of-the-repo)), no commit can carry its progress next to the code. So each code commit for it says which change it belongs to instead, in a git trailer at the end of its message:

```
feat: disable Save while saving

yass: 2026-09-14-offline-sync/2026-09-16-offline-queue
```

The trailer survives squash merges when the pull request's description or commit list keeps it, and git tools read it (`git log --format='%(trailers:key=yass)'`). Changes whose plans are in the repo's own `yass/` need none: their progress travels in the commits.

## The trailer

```
yass: <target> [ac:<criteria>]
```

- **target** is a change's folder name, `<change>/<piece>`, a piece's own name when only one change has a piece by that name, or `none` for a commit that belongs to no change (a dependency bump, a typo fix). Archived changes count too.
- **`ac:`** names the criteria the commit works toward, comma-separated: `yass: 2026-09-14-offline-sync ac:ac2,ac5`.
- **Case doesn't matter;** YASS writes lower case. A commit for two changes gets two `yass:` lines. A key this version of YASS doesn't know is a note, not an error, so newer trailers don't break older binaries.

## You don't type it

`yass init --hooks` installs three hooks in `tools/yass/githooks/`: `pre-commit` (YASS's usual check), `prepare-commit-msg`, which adds the trailer, and `commit-msg`, which checks it. The hook works out the change from, in order:

1. **`yass use <change>`**, which sets this worktree's change until you change or clear it (`yass use` shows it, `yass use --clear` clears it). It's kept in the worktree's own git folder, so it's never committed, and each worktree can work on a different change. `--ac ac2,ac5` adds criteria.
2. **The branch name**, when it's a change's or piece's folder name, on its own or after a prefix: `jaime/2026-09-14-offline-sync`.
3. **An earlier commit's trailer** on the branch, since it left the main branch.

It adds the trailer when that change lives outside the repo, and leaves alone a trailer you wrote and merge commits. The `commit-msg` hook points out a trailer that's malformed or names no change, with the closest change names; it never asks for a missing one and never refuses a commit.

## In CI

`yass hook --range <base>..<head>` checks the trailers of every non-merge commit in the range, alongside its usual check. `--require-trailers` also reports commits without one, and `--strict` fails the job on anything it reports. A squash commit whose message carries the trailers passes. Where CI doesn't check out the plans, it checks only the trailers' form and says the change names weren't checked.

## Finding a change's code

`yass status <change>`, run in the code repo, lists the commits on the main branch whose trailers name that change or one of its pieces, and, when they name criteria with `ac:`, which commits worked toward each one:

```
code on origin/main (from yass: trailers):
  9f2c1aa Cart totals (#7)
  4e1b2c3 feat: cart ui
criteria:
  AC1: 4e1b2c3
  AC3: 9f2c1aa, 4e1b2c3
```

Because it reads trailers, not commit hashes cited in the plans, it still works after a squash merge. Unmerged commits aren't listed, and an `ac:` naming a criterion the plan doesn't have is a warning. It shows ten commits per list and says how many more there are.

## With a hook manager

If your repo already runs hooks through a manager, call YASS from it rather than pointing `core.hooksPath` at YASS's folder.

**husky** (tested): add a line to each hook file.

```sh
# .husky/prepare-commit-msg
yass hook --prepare-msg "$@"
# .husky/commit-msg
yass hook --commit-msg "$1"
# .husky/pre-commit
yass hook
```

**lefthook**, in `lefthook.yml`:

```yaml
prepare-commit-msg:
  commands:
    yass:
      run: yass hook --prepare-msg {1} {2}
commit-msg:
  commands:
    yass:
      run: yass hook --commit-msg {1}
pre-commit:
  commands:
    yass:
      run: yass hook
```

**pre-commit**, in `.pre-commit-config.yaml` (it passes the message file to both message hooks):

```yaml
default_install_hook_types: [pre-commit, prepare-commit-msg, commit-msg]
repos:
  - repo: local
    hooks:
      - id: yass
        name: yass hook
        entry: yass hook
        language: system
        pass_filenames: false
        always_run: true
      - id: yass-trailer
        name: yass trailer
        entry: yass hook --prepare-msg
        language: system
        stages: [prepare-commit-msg]
      - id: yass-trailer-check
        name: yass trailer check
        entry: yass hook --commit-msg
        language: system
        stages: [commit-msg]
```


## docs/skills.md

# The playbooks

YASS ships six playbooks, as [Agent Skills](https://agentskills.io) in [`kit/.agents/skills/`](../kit/.agents/skills). Every `yass` binary carries them, and `yass init` writes them to `.agents/skills/` in your repo (project scope); `--claude` also copies them to `.claude/skills/`. With `--global`, they go in your user folder instead. `yass upgrade` brings them up to a newer binary's version. Agents pick one by matching your request against its `description`.

This page is an index for reviewing what each playbook lets an agent do. The `SKILL.md` files are the source of truth; if this page and a playbook disagree, the playbook wins and this page needs fixing.

| Playbook | Mode | Use it for | Writes files | Waits for a human |
|---|---|---|---|---|
| [`yass-shape`](../kit/.agents/skills/yass-shape/SKILL.md) | conversation | a new feature, component or system; a harness plan to save; revising a PRD | `prd.md`, `change.md` | to pick a scope, and to approve the PRD |
| [`yass-plan`](../kit/.agents/skills/yass-plan/SKILL.md) | mostly automated | approach, acceptance criteria, pieces; a harness plan to save; revising a plan | `plan.md`, `design.md`, pieces | no; asks only about real choices the PRD leaves open |
| [`yass-work`](../kit/.agents/skills/yass-work/SKILL.md) | attended or unattended | building the one change it's pointed at | code, progress sections, stacked intent fixes | when it isn't pointed at a change, or isn't confident a plan fix is right |
| [`yass-status`](../kit/.agents/skills/yass-status/SKILL.md) | report | "where are we?" | nothing | no; it lists what needs one |
| [`yass-log`](../kit/.agents/skills/yass-log/SKILL.md) | report | "why did we…?", "why does this code…?", release notes, retros | nothing | no |
| [`yass-update`](../kit/.agents/skills/yass-update/SKILL.md) | attended | "update YASS", or `yass status`'s version note | the `yass` binary; YASS's files in the repo | before downloading a binary, and before committing |

The usual flow is (plan mode →) shape → plan → work, with `yass-status` along the way and `yass-log` whenever someone asks why. Small changes skip straight to `yass-work`. Each step ends where the next begins: shaping never plans, planning never builds.

**Starting work.** `yass-work` only builds a change it's been pointed at, by name or because the session is already on it. It never picks one because it's in progress. Boxes are `[ ]` not started, `[/]` in progress, `[x]` done, `[-]` dropped. `yass status` marks changes with no Log entry and no box in progress or done as `not started`, and changes whose boxes are all done or dropped as `done`; shaping and planning write neither, so a freshly planned change stays `not started` until someone starts it. In a fully autonomous run, where the harness or human has put the agent in charge, the agent makes that choice and any approvals itself, and records them in Decisions.

**Git.** No playbook commits on its own initiative, and none pushes, merges or opens pull requests. Only `yass-work` touches git, and only to stack an intent fix under the code, when the human or the harness has given it that latitude; otherwise it hands the human the commands. The CLI never touches git. What a good commit looks like (intent apart from code) is in the README rules, the `AGENTS.md` section and the optional hook.

**Plan mode.** `yass-shape`, `yass-plan` and `yass-work` can each start from a plan the harness's plan mode produced, carrying over its decisions instead of asking again, and telling the human what they added beyond it.

## yass-shape

A conversation where a human decides what to build and why. Turns a problem, or an approved harness plan, into a large change with a PRD, and stops there. Also revises the PRD of an active change.

- **Reads:** a harness plan if there is one; the relevant code and docs; the archive (`yass status --archived`, `yass decisions --about …`) for earlier work; `yass status` for overlapping work.
- **Runs:** `yass new "<title>" --large [--source …] [--follows …]`.
- **Writes:** `prd.md` (why, users and outcomes, requirements `R1…`, non-goals, milestones, open questions); the Goal and `## Decisions` in `change.md`, crediting the human for choices made in their plan. A revision also records its reason in Decisions.
- **Human gates:** picks among 2–3 scoping options for non-trivial problems, unless the plan already picked; shows the PRD and waits for a go-ahead before planning, *unless your team has told it otherwise*, or a fully autonomous run has put it in charge of approvals (it records that in Decisions). From an approved plan, it points out only what it added. Tells a human about revisions.
- **Never:** mixes code into PRD edits; edits an archived change (new intent is a new change with `--follows`).

## yass-plan

Mostly automated: turns an approved PRD into the how (approach, acceptance criteria, design calls, PR-sized pieces) without a back-and-forth, and stops at a written plan. The change stays `not started` until someone points `yass-work` at it. Also revises a plan when reality disagrees.

- **Reads:** a harness plan if there is one (checked against the code and PRD, not redone); the change's `prd.md`, the code it touches, its docs. With no change folder yet, hands large work to `yass-shape` and creates small work itself.
- **Runs:** `yass new "<piece>" --in <change> [--goal …]`; `yass new … --design`, or `yass template design` for an existing change.
- **Writes:** `plan.md` (approach, `AC1 (R1) Given … — verify: …` per milestone, piece order, validation); `design.md` for hard-to-undo calls (at least two options, who decided); each piece's `change.md`, naming the criteria it delivers. Turns a harness plan's steps into real criteria rather than relabeling them.
- **Human gates:** asks only when the PRD leaves a real choice open. Tells a human about any revision, and calls out reworded, removed or dropped criteria as lowering the bar. Doesn't wait for approval of a new plan.
- **Never:** mixes code into plan edits. A revision during a build goes underneath the code (see `yass-work`).

## yass-work

Builds the one change it's pointed at and keeps its progress next to the code. Also runs unattended: one change (or piece) per run, stopping instead of asking.

- **Picks:** only the change named in the request, or the one the session is already on. Otherwise it shows `yass status` and asks (unattended: stops). When it's been put in charge of choosing, it picks, checks or gives the plan's approval, and records both in Decisions.
- **Reads:** the change's `change.md` from the last **Next:** in its Log; for a piece, the parent's `prd.md` and `plan.md`; the code and docs it touches.
- **Runs:** `yass status [<change>]`; `yass new "<title>" [--source …]` for a small, clear request (from a harness plan, if there is one); `yass archive <change>` when the change is `done`.
- **Writes:** code; boxes marked `[/]` when started and `[x]` when verified (Steps, Acceptance, and plan criteria the work delivers); `## Decisions`; `## Steps`; a `## Log` entry before stopping. For a new small change, its Goal and Acceptance.
- **When the plan is wrong:** reasons out the fix. Not confident: sets `blocked:` and stops. Confident: puts the intent edit on its own branch from the work's base, with the reason in Decisions, rebases the work on top, and keeps going. Lowering the bar is stacked the same way but flagged as needing a human before merge.
- **Plans in another folder** (`yass root -v` says `separate`): pulls the plans first, and stops rather than running `yass init` when they're missing. Names the change in every code commit (`yass use <change>`, so the hooks add the `yass:` trailer), commits code first, then the plans, citing the code commits a box rests on (`— code: a1b2c3d`) and marking it `[x]` only once that code is merged. Cross-links the code PR (with `yass status <change> --brief` and the trailer) and the plans commit. Latitude for git is per repo. Intent changes go in the plans' own repo, linked from the code PR.
- **Git:** only for that stack, and only with latitude from the human or harness (plain `git`, or `gh stack` and similar). Attended without latitude: shows the commands. Unattended without latitude: leaves the edits, writes the commands into the Log's **Next:**, and stops.
- **Never:** changes intent alongside code; builds out-of-scope work (it notes it in the Log or the repo's issue tracker); marks a box `[x]` without evidence from its `verify:`.

## yass-status

A one-screen report, then a list of what needs a human. Read-only.

- **Reads:** `yass status`, then `yass status <change>` for every change that's been started (for its `[/]` items, and blocked or nearly done detail); `git log --since=<date>` over each folder `yass root` prints, and Log entries since then (it asks since when, or uses a week); recent decisions, the way `yass-log` finds them.
- **Reports:** progress per change (in progress apart from `not started`), blocked changes and why, recent decisions with who made them, changes ready to archive, stale changes, and intent revisions stacked under work in progress.
- **Then lists:** each item that needs a human, with the playbook or command that handles it, and one suggested next move. It acts on none of them.

## yass-log

Reconstructs decisions from the files and git, and links them to code through `code:` citations. Read-only.

- **Reads:** `## Decisions` in every yass folder `yass root` prints (in the repo, team folders, or wherever `yass.yaml` points) via `yass decisions`, never by reading `archive/` directly; `git log` for when and who, `git log -S` for the commit that added a line, `git log -p` for how a PRD or plan changed; the commits boxes cite, from the code repo (date, author, subject, merged or not); Goal, PRD and Log for context. Can start from code instead: a file's or symbol's commits, looked up with `yass decisions --cites <sha>`.
- **Reports:** newest first, grouped by change, with the cited code under each decision, citing file paths and commit shas (and which repo each sha is from). Flags decisions with no author, and moved goalposts: intent edits in the same commit as code when the plans are in the repo; acceptance criteria reworded or dropped after their code's box was marked `[x]` when they're in another folder.
- **Plans in another folder:** runs git for the plans with `git -C <folder> … -- .` (the folder may share a repo) and for the code here. With no git history, dates decisions only approximately, from the Log headings around them, and says so.
- **Never:** infers a decision that isn't written down; it says when the record is silent.

## yass-update

Brings YASS up to date on two levels: the `yass` binary on this machine, and YASS's files in the repo.

- **Reads:** `yass update --check` (the binary's version and how it was built, the repo's YASS version, the latest release, what would be installed).
- **Runs:** `yass update` (with `--latest` only when the human wants the repo upgraded past the team's version), then `yass upgrade`; `git status` and a diff summary.
- **Writes:** the `yass` binary, through `yass update`, which verifies it first; YASS's files in the repo, through `yass upgrade`. Commits only with latitude: a layout migration in its own commit, then `chore: upgrade YASS`.
- **Human gates:** asks before downloading a binary, unless it has latitude to; never works around a failed checksum or provenance check, and never fetches a binary another way. A source build's update command (`go install`, a clone rebuild) runs only when the human says so.


## examples/monorepo/README.md

# Example: many teams, one monorepo

Acme runs a marketplace from one repository. Three folders have a `yass/` folder. `yass status` finds every `yass/` folder in the repo, with no config.

| Folder | Who | What's in it |
|---|---|---|
| `yass/` | product | a cross-team change: *three-tap checkout*, with the PRD and the end-to-end acceptance |
| `services/payments/yass/` | payments team | *saved cards API*, a change payments owns |
| `apps/web/yass/` | web team | *preselect a saved card*, a change web owns |

`services/search/` has no `yass/` folder. That's fine: teams opt in by running `yass init <their folder>`.

Things to notice:
- **Cross-team work is just links.** The root change's plan lists which team change delivers each criterion; each team change names the root criterion it delivers. Nobody copies anyone's plan.
- **Web waits on payments.** The web change's `blocked:` names the payments change, so `yass status` shows `waiting on: 2026-09-10-saved-cards-api` until payments ticks its last box. Nobody has to remember to unblock it.
- **Payments adopted YASS late.** Its `README.md` and `docs/api.md` describe what exists today. Its `yass/` folder only describes what it's changing.
- **Ownership is your repo's business.** Use CODEOWNERS on `*/yass/` folders if you want reviews from the owning team.

Try it: copy this folder into a git repo with YASS installed and run `yass status`.


## examples/monorepo/apps/web/yass/README.md

# yass/

Planned work for this part of the repo, managed with YASS (Yet Another Spec System). Code and its docs describe what exists today; this folder describes what's changing.

- `changes/`: active work, one dated folder per change. A small change is a `change.md`. A large one adds `prd.md` (why and what), `plan.md` (how and acceptance), an optional `design.md`, and PR-sized pieces as folders inside it.
- `archive/`: finished changes, moved here as they were. Never edited; follow-up work is a new change with `follows: <folder>`.
- `queue.md` (optional): the order to tackle changes in, top first, one folder name per list item. `yass status` follows it; changes it doesn't list come after, oldest first.

Rules:
1. **A change is a folder; ceremony scales with scope.**
2. **Progress travels with the code.** Ticking boxes and appending to Log and Decisions go in the same commits as the code.
3. **Intent changes get their own commit.** `prd.md`, `design.md`, `plan.md` beyond ticking boxes, and a change's title, Goal and Acceptance (including dropping a criterion).
4. **Done means every box is checked.** Then `yass archive <change>`, in its own commit.
5. **The archive is append-only.**

`yass status` shows what's in flight.


## examples/monorepo/apps/web/yass/changes/2026-09-12-preselect-saved-card/change.md

---
platforms: [web]
source: yass/changes/2026-09-01-three-tap-checkout (AC1, AC2)
follows:
blocked: services/payments/yass/changes/2026-09-10-saved-cards-api
---
# Preselect a saved card

## Goal
Checkout preselects the buyer's most recent saved card, and Buy goes straight to confirm. Delivers AC1 and AC2 of three-tap checkout in the web app.

## Acceptance
- [ ] Delivers AC1 and AC2 (root plan.md)
- [ ] Given a buyer with no saved card, when they reach checkout, then the card form shows as today — verify: e2e checkout/no_saved_card

## Steps
- [x] Checkout reads cards from the payments client (mocked)
- [ ] Switch to the real endpoint
- [ ] Buy → confirm in one tap when a card is preselected

## Decisions
- Preselect the most recent card, not a "default" flag - payments has no default flag (priya)

## Log
### 2026-09-20 (claude)
- Did: UI against a mocked client.
- Next: switch to the real endpoint once payments ships it.


## examples/monorepo/services/payments/README.md

# Payments service

Takes money for the marketplace: charges, refunds, payouts. Card data never touches our storage; the vault tokenizes it. The API is in `docs/api.md`.


## examples/monorepo/services/payments/docs/api.md

# Payments API (current)

- `POST /charges`: charge a buyer with a card token.
- `POST /refunds`: refund a charge, fully or partly.
- `POST /cards`: tokenize a card through the vault and attach it to a buyer.


## examples/monorepo/services/payments/yass/README.md

# yass/

Planned work for this part of the repo, managed with YASS (Yet Another Spec System). Code and its docs describe what exists today; this folder describes what's changing.

- `changes/`: active work, one dated folder per change. A small change is a `change.md`. A large one adds `prd.md` (why and what), `plan.md` (how and acceptance), an optional `design.md`, and PR-sized pieces as folders inside it.
- `archive/`: finished changes, moved here as they were. Never edited; follow-up work is a new change with `follows: <folder>`.
- `queue.md` (optional): the order to tackle changes in, top first, one folder name per list item. `yass status` follows it; changes it doesn't list come after, oldest first.

Rules:
1. **A change is a folder; ceremony scales with scope.**
2. **Progress travels with the code.** Ticking boxes and appending to Log and Decisions go in the same commits as the code.
3. **Intent changes get their own commit.** `prd.md`, `design.md`, `plan.md` beyond ticking boxes, and a change's title, Goal and Acceptance (including dropping a criterion).
4. **Done means every box is checked.** Then `yass archive <change>`, in its own commit.
5. **The archive is append-only.**

`yass status` shows what's in flight.


## examples/monorepo/services/payments/yass/changes/2026-09-10-saved-cards-api/change.md

---
platforms: [api]
source: yass/changes/2026-09-01-three-tap-checkout (AC1)
follows:
blocked:
---
# Saved cards API

## Goal
`GET /buyers/{id}/cards` returns token ids, brand and last4 for a buyer's saved cards, so checkout can offer them. Part of three-tap checkout (root change, AC1).

## Acceptance
- [x] Given a buyer with two saved cards, when calling the endpoint, then both come back, newest first, with brand and last4 only — verify: contract test cards_list
- [ ] Given another buyer's id, when calling the endpoint, then it returns 403 — verify: contract test cards_list_forbidden

## Steps
- [x] Handler and query
- [x] Contract test cards_list
- [ ] Authorization check and its test
- [ ] Add the endpoint to docs/api.md

## Decisions
- Return brand and last4 only - enough for the UI, nothing sensitive (codex, approved by @acme/payments)
- Cache per buyer for 60s - the vault rate-limits listing (codex)

## Log
### 2026-10-02 (codex)
- Did: handler, query, cards_list passes.
- Next: authorization check, then document the endpoint.


## examples/monorepo/services/search/README.md

# Search service
Not using YASS (yet).


## examples/monorepo/yass/README.md

# yass/

Planned work for this part of the repo, managed with YASS (Yet Another Spec System). Code and its docs describe what exists today; this folder describes what's changing.

- `changes/`: active work, one dated folder per change. A small change is a `change.md`. A large one adds `prd.md` (why and what), `plan.md` (how and acceptance), an optional `design.md`, and PR-sized pieces as folders inside it.
- `archive/`: finished changes, moved here as they were. Never edited; follow-up work is a new change with `follows: <folder>`.
- `queue.md` (optional): the order to tackle changes in, top first, one folder name per list item. `yass status` follows it; changes it doesn't list come after, oldest first.

Rules:
1. **A change is a folder; ceremony scales with scope.**
2. **Progress travels with the code.** Ticking boxes and appending to Log and Decisions go in the same commits as the code.
3. **Intent changes get their own commit.** `prd.md`, `design.md`, `plan.md` beyond ticking boxes, and a change's title, Goal and Acceptance (including dropping a criterion).
4. **Done means every box is checked.** Then `yass archive <change>`, in its own commit.
5. **The archive is append-only.**

`yass status` shows what's in flight.


## examples/monorepo/yass/changes/2026-09-01-three-tap-checkout/change.md

---
platforms: [web, ios, android, api]
source: Q4 company bet
follows:
blocked:
---
# Three-tap checkout

## Goal
A returning buyer can buy anything in three taps from any surface: open the item, tap Buy, confirm. Product owns the PRD and the end-to-end acceptance; payments and web each own a piece of the work in their own folders.

## Decisions
- Returning buyers first - they are 70% of checkouts (@acme/product)
- Saved cards live in payments, not in web - card data stays behind the vault (@acme/payments, @acme/web)

## Log
### 2026-09-12 (claude)
- Did: plan agreed; team changes created in payments and web.
- Next: end-to-end flow once both team changes land.


## examples/monorepo/yass/changes/2026-09-01-three-tap-checkout/plan.md

# Three-tap checkout: plan

## Approach
Payments exposes a buyer's saved cards; web preselects the most recent one at checkout. No new storage.

## Acceptance
### M1
- [ ] AC1 (R1) Given a buyer with a saved card, when they reach checkout, then the card is preselected — verify: e2e checkout/saved_card (web)
- [ ] AC2 (R2) Given a buyer with a saved card, when they buy from the item page, then it takes three taps — verify: e2e checkout/three_taps (web)

## Pieces
Team changes, in their own folders:
1. payments: `services/payments/yass/changes/2026-09-10-saved-cards-api`, delivers the API for AC1.
2. web: `apps/web/yass/changes/2026-09-12-preselect-saved-card`, delivers AC1 and AC2 in the UI. Needs 1.

## Validation
Both e2e flows on staging, plus a manual run with a real test card.


## examples/monorepo/yass/changes/2026-09-01-three-tap-checkout/prd.md

# Three-tap checkout: PRD

## Why
Small sellers lose a third of buyers at checkout. Returning buyers re-enter card details every time.

## Users and outcomes
Returning buyers on web and mobile.
- Checkout conversion for returning buyers from 61% to 75%.
- p50 checkout time under 20s.

## Requirements
- **R1** [M1] A returning buyer can pay with a saved card without re-entering it.
- **R2** [M1] Checkout takes three taps from the item page for a buyer with a saved card.

## Non-goals
- Guest checkout changes.
- New payment methods.

## Milestones
| Milestone | When it ships, a user can … |
|---|---|
| M1 | buy anything in three taps on the web |

## Open questions
- Mobile in M1 or M2? (product, by 2026-10-15)


## examples/solo-app/README.md

# Example: a solo mobile app

Moodlog is a small iOS/Android/web mood journal built by one developer (Sam) and coding agents. It adopted YASS on day one, so its archive is the story of the whole app.

```
yass/
├── changes/
│   ├── 2026-09-14-offline-sync/          large, in progress
│   │   ├── change.md  prd.md  plan.md  design.md
│   │   ├── 2026-09-16-offline-queue/     piece, in progress (read its Log to see how an agent resumes)
│   │   └── 2026-09-24-sync-badge/        piece, blocked on a question for Sam
│   └── 2026-10-02-fix-double-tap-save/   small, from a bug report
├── queue.md                              what to tackle first: the crash, then sync
└── archive/2026/09/                      finished in September 2026
    ├── 2026-08-03-mood-logging/          the first feature, finished
    └── 2026-09-08-fix-history-sort/      a finished bug fix
```

Things to notice:
- **Ceremony scales.** The bug fix is one file. Offline sync has a PRD, a plan, a design doc for its one hard call, and two PR-sized pieces.
- **Progress lives next to the work.** Ticked boxes, Decisions with who made them, and a Log that ends with **Next:**.
- **An order to work in.** `queue.md` puts the crash fix ahead of offline sync, though it's newer; `yass status` lists it first.
- **Intent vs. progress.** The PRD, plan and design were committed on their own. The pieces tick plan boxes as they deliver them.

Try it: copy `yass/` into a git repo with YASS installed and run `yass status`, then `yass status offline-queue`.


## examples/solo-app/yass/README.md

# yass/

Planned work for this part of the repo, managed with YASS (Yet Another Spec System). Code and its docs describe what exists today; this folder describes what's changing.

- `changes/`: active work, one dated folder per change. A small change is a `change.md`. A large one adds `prd.md` (why and what), `plan.md` (how and acceptance), an optional `design.md`, and PR-sized pieces as folders inside it.
- `archive/`: finished changes, moved here as they were. Never edited; follow-up work is a new change with `follows: <folder>`.
- `queue.md` (optional): the order to tackle changes in, top first, one folder name per list item. `yass status` follows it; changes it doesn't list come after, oldest first.

Rules:
1. **A change is a folder; ceremony scales with scope.**
2. **Progress travels with the code.** Ticking boxes and appending to Log and Decisions go in the same commits as the code.
3. **Intent changes get their own commit.** `prd.md`, `design.md`, `plan.md` beyond ticking boxes, and a change's title, Goal and Acceptance (including dropping a criterion).
4. **Done means every box is checked.** Then `yass archive <change>`, in its own commit.
5. **The archive is append-only.**

`yass status` shows what's in flight.


## examples/solo-app/yass/archive/2026/09/2026-08-03-mood-logging/2026-08-04-log-a-mood/change.md

---
platforms: [ios, android, web]
source:
follows:
blocked:
---
# Log a mood

## Goal
Opening the app lands on the mood picker; one tap saves an entry with an optional note. Delivers AC1.

## Acceptance
- [x] Delivers AC1 (plan.md)

## Steps
- [x] Picker as the launch screen
- [x] Save with an optimistic list update
- [x] Maestro flow log_mood on iOS and Android, Playwright on web

## Decisions
- Launch straight into the picker - saves ~4s versus a home screen (claude, approved by sam)

## Log
### 2026-08-11 (claude)
- Did: picker and save on all three platforms; log_mood passes everywhere (p50 2.8s).
- Next: done.


## examples/solo-app/yass/archive/2026/09/2026-08-03-mood-logging/2026-08-15-history/change.md

---
platforms: [ios, android, web]
source:
follows:
blocked:
---
# History

## Goal
A History screen shows the last 30 days of entries, newest first. Delivers AC2.

## Acceptance
- [x] Delivers AC2 (plan.md)
- [-] Infinite scroll past 30 days - dropped: not in the PRD; see Decisions

## Steps
- [x] Query + list
- [x] HistoryTests

## Decisions
- Dropped infinite scroll - out of scope for M1; nobody asked (sam)

## Log
### 2026-09-10 (claude)
- Did: History on all platforms; HistoryTests pass.
- Next: done.


## examples/solo-app/yass/archive/2026/09/2026-08-03-mood-logging/change.md

---
platforms: [ios, android, web]
source:
follows:
blocked:
archived: 2026-09-12T17:20:00Z
---
# Mood logging

## Goal
The first version of Moodlog: log a mood in under five seconds and look back over the last 30 days, on iOS, Android and the web.

## Decisions
- One TypeScript core shared by React Native and the web app - most logic is the same, the UI isn't (sam)
- Launch straight into the mood picker - saves ~4s versus a home screen (claude, approved by sam)
- Shipped as v0.1.0 on 2026-09-12; p50 time-to-log 2.8s against a 5s target (sam)

## Log
### 2026-09-12 (sam)
- Did: shipped v0.1.0 on all three platforms.
- Next: nothing; offline is a separate change.


## examples/solo-app/yass/archive/2026/09/2026-08-03-mood-logging/plan.md

# Mood logging: plan

## Approach
Shared TypeScript core (entries, storage interface) used by a React Native app and a web app, backed by a small HTTP API.

## Acceptance
### M1
- [x] AC1 (R1) Given a fresh launch, when the user taps a mood, then it is saved within 5s — verify: e2e flows/log_mood
- [x] AC2 (R2) Given 40 days of entries, when opening History, then the last 30 show newest first — verify: unit HistoryTests

## Pieces
1. `log-a-mood` (AC1), then 2. `history` (AC2).

## Validation
`tools/verify all e2e` on all three platforms.


## examples/solo-app/yass/archive/2026/09/2026-08-03-mood-logging/prd.md

# Mood logging: PRD

## Why
People forget how they felt last week, so they can't see what helps. Logging has to be fast enough to do every day.

## Users and outcomes
People who want a lightweight, private mood journal, on iOS, Android or the web.
- p50 time from app open to saved entry < 5s.
- 40% of new users log on 5+ days of their first 14.

## Requirements
- **R1** [M1] A user can log a mood (1–5 plus an optional note) in under 5 seconds from launch.
- **R2** [M1] A user can see their last 30 days of entries, newest first.

## Non-goals
- Social features or sharing. Moodlog is private.
- Medical claims or diagnosis.

## Milestones
| Milestone | When it ships, a user can … |
|---|---|
| M1 | log a mood and look back over the last 30 days |

## Open questions


## examples/solo-app/yass/archive/2026/09/2026-09-08-fix-history-sort/change.md

---
platforms: [web]
source: gh#29
follows: 2026-08-03-mood-logging
blocked:
archived: 2026-09-08T16:05:00Z
---
# Fix History sort on the web

## Goal
History on the web shows newest first, like mobile.

## Acceptance
- [x] Given entries from three days, when opening History on the web, then they're newest first — verify: unit HistoryTests.web

## Steps
- [x] Sort by `createdAt` descending in the web query

## Decisions
- Sort in the query, not the component - the core already pages by date (claude)

## Log
### 2026-09-08 (claude)
- Did: fixed and tested.
- Next: done.


## examples/solo-app/yass/changes/2026-09-14-offline-sync/2026-09-16-offline-queue/change.md

---
platforms: [ios, android, web]
source:
follows:
blocked:
---
# Offline queue

## Goal
Entries are written locally first and queued; a worker replays the queue in order when the network returns, and the queue survives the app being killed. Delivers AC1 and AC2.

## Acceptance
- [x] Delivers AC1 (plan.md)
- [ ] Delivers AC2 (plan.md)
- [ ] Given a 500-entry queue, when it drains, then the UI stays responsive (no frame > 50ms) — verify: perf flows/drain_500

## Steps
- [x] Queue table + IndexedDB store
- [x] Worker drains in order with backoff
- [x] WAL mode on Android
- [ ] Web: drain on the `online` event
- [ ] Drain in batches of 50

## Decisions
- Replay strictly in order - parallel replay reordered entries on Android (claude)
- Backoff caps at 60s - longer felt broken in testing (sam)

## Log
### 2026-09-30 (claude)
- Did: queue + worker; offline_sync passes on iOS and Android.
- Learned: Android kills the worker before the DB transaction commits; needs WAL mode.
- Next: enable WAL on Android, rerun flows/offline_kill.

### 2026-10-02 (codex)
- Did: WAL on Android; offline_kill passes on Android and iOS. Playwright offline.spec.ts added; AC1 passes on web.
- Learned: the web has no background worker; draining must hook the `online` event.
- Next: drain on `online` for the web, then batch the drain.


## examples/solo-app/yass/changes/2026-09-14-offline-sync/2026-09-24-sync-badge/change.md

---
platforms: [ios, android]
source:
follows:
blocked: does the web get the badge too? (open question in prd.md, for Sam)
---
# Sync badge

## Goal
History marks entries still waiting in the sync queue with a small "not synced" badge. Delivers AC3.

## Acceptance
- [ ] Delivers AC3 (plan.md)
- [ ] The badge disappears within 1s of the entry syncing — verify: e2e flows/sync_badge

## Steps
- [ ] Queue exposes `pending(entryId)`
- [ ] Badge component
- [ ] Wire into History

## Decisions

## Log
### 2026-09-24 (claude)
- Did: nothing yet; read the PRD.
- Next: needs Sam's answer on the web, then start with `pending(entryId)`.


## examples/solo-app/yass/changes/2026-09-14-offline-sync/change.md

---
platforms: [ios, android, web]
source: feedback (9 notes in Sept mention the subway)
follows: 2026-08-03-mood-logging
blocked:
---
# Offline sync

## Goal
Logging works with no network, nothing logged offline is ever lost, and users can see what hasn't synced yet. Details in prd.md and plan.md.

## Decisions
- Offline before export - 6 of 9 feedback notes are about the subway; nobody asked for CSV yet (sam)
- Local-first storage with a sync queue - see design.md (claude, approved by sam)
- Sync is last-write-wins per entry - entries are append-only, so conflicts are rare (sam)

## Log
### 2026-09-15 (claude)
- Did: PRD and plan with Sam; split into two pieces.
- Next: offline-queue first; sync-badge needs it.


## examples/solo-app/yass/changes/2026-09-14-offline-sync/design.md

# Offline sync: design

## Context
R1 and R2 need entries to survive having no network and the app being killed. Today entries go straight to the API.

## Options
| Option | For | Against |
|---|---|---|
| Local-first + queue | survives app kill; works everywhere | two storage implementations |
| Retry in memory | trivial | loses entries when the app is killed |
| Third-party sync SDK | less code | lock-in, cost, web support unclear |

## Decision
Local-first + queue, owned by the shared core; platform layers only provide storage adapters. Proposed by claude, approved by sam on 2026-09-15.

## Consequences and rollback
Two storage adapters to maintain. Rollback: flush the queue, then write through to the API again.


## examples/solo-app/yass/changes/2026-09-14-offline-sync/plan.md

# Offline sync: plan

## Approach
Write every entry to local storage first and append it to a sync queue in the shared TypeScript core; a worker drains the queue in order with backoff. Mobile uses SQLite (WAL mode), web uses IndexedDB. See design.md for why.

## Acceptance
### M1
- [x] AC1 (R1) Given no network, when the user logs 3 entries and reconnects, then all 3 reach the server in order — verify: e2e flows/offline_sync (ios, android), Playwright offline.spec.ts (web)
- [ ] AC2 (R2) Given queued entries, when the app is killed and reopened, then the queue is intact — verify: e2e flows/offline_kill
### M2
- [ ] AC3 (R3) [ios, android] Given unsynced entries, when viewing History, then each shows a "not synced" badge — verify: e2e flows/sync_badge

## Pieces
1. `offline-queue` (AC1, AC2): storage, queue and worker. Everything else depends on it.
2. `sync-badge` (AC3): UI only, once the queue exposes "pending".

## Validation
`tools/verify all e2e` on all three platforms before archiving, plus one manual subway-style run (airplane mode, kill the app, reconnect).


## examples/solo-app/yass/changes/2026-09-14-offline-sync/prd.md

# Offline sync: PRD

## Why
Moodlog's most common feedback in September: "it lost what I logged on the subway". Entries go straight to the API today and fail without a network. A journal that loses entries isn't trusted.

## Users and outcomes
Commuters and anyone with patchy coverage, on iOS, Android and the web.
- Zero lost entries (support tickets tagged "lost entry", currently ~4 a week).
- No change to p50 time-to-log (< 5s).

## Requirements
- **R1** [M1] Entries logged offline are never lost, and sync in order when the device reconnects.
- **R2** [M1] Queued entries survive the app being killed.
- **R3** [M2] [ios, android] The user can see which entries haven't synced yet.

## Non-goals
- Multi-device conflict resolution UI. Last write wins until we see real conflicts.
- Background sync while the app is closed (OS limits; revisit later).

## Milestones
| Milestone | When it ships, a user can … |
|---|---|
| M1 | log on the subway and never lose an entry |
| M2 | see what hasn't synced yet (mobile) |

## Open questions
- Does the web need the badge too? (Sam to decide; see the sync-badge piece.)


## examples/solo-app/yass/changes/2026-10-02-fix-double-tap-save/change.md

---
platforms: [all]
source: gh#41
follows:
blocked:
---
# Fix double-tap save crash

## Goal
Tapping Save twice quickly saves one entry and never crashes.

## Acceptance
- [ ] Given the mood picker, when Save is tapped twice within 300ms, then exactly one entry is saved and the app doesn't crash — verify: unit SaveButtonTests.doubleTap

## Steps
- [x] Reproduce on Android (crash in EntryStore.insert, duplicate key)
- [ ] Disable Save while a save is in flight
- [ ] Unit test

## Decisions
- Disable the button rather than dedupe in the store - fixes the cause, not the symptom (claude)

## Log
### 2026-10-02 (claude)
- Did: reproduced; root cause is two inserts racing on the same generated id.
- Next: disable Save while saving, then the unit test.


## examples/solo-app/yass/queue.md

# Queue

What Sam wants tackled, top first. `yass status` follows this order; changes not listed come after, oldest first.

1. 2026-10-02-fix-double-tap-save — a crash in the wild; fix it before more sync work
2. 2026-09-14-offline-sync


## install.sh

#!/usr/bin/env bash
# Install the yass binary. Setting up a repo is `yass init`; upgrading one is `yass upgrade`.
#
#   From the web:    curl -fsSL https://raw.githubusercontent.com/donjaime/yass/main/install.sh | bash
#                    (downloads the latest release for this machine and checks it against the release's checksums;
#                    YASS_VERSION=v0.3.0 picks a version)
#   From a release:  tar -xzf yass_<os>_<arch>.tar.gz && yass_<os>_<arch>/install.sh
#   From a clone:    go build -o bin/yass ./cmd/yass && ./install.sh
#
# Options:
#   --bin-dir DIR  copy the yass binary into DIR, which should be on your PATH (default: ~/.local/bin)
#
# Then, in a repo:  yass init [--claude] [--hooks] [--global] [--path P]   (yass init --help)
# Uses the yass binary from $YASS_BIN, next to this script (a release folder), or bin/ (a clone).
set -euo pipefail
REPO_SLUG="${YASS_REPO:-donjaime/yass}"
VERSION="${YASS_VERSION:-latest}"
die() { echo "yass: $*" >&2; exit 1; }

BIN_DIR="$HOME/.local/bin"; BIN_ARG=""; OLD=0; INIT=(); UPGRADE=0; REPO=""
while [ $# -gt 0 ]; do
  case "$1" in
    --bin-dir) BIN_DIR="${2:?--bin-dir needs a folder}"; BIN_ARG=" --bin-dir $2"; shift 2 ;;
    -h|--help) sed -n '2,14p' "$0" 2>/dev/null || true; exit 0 ;;
    # What earlier versions took for setting up a repo: now yass init's and yass upgrade's job.
    --claude|--hooks|--global) OLD=1; INIT+=("$1"); shift ;;
    --path) OLD=1; INIT+=(--path "${2:?--path needs a value}"); shift 2 ;;
    --upgrade) OLD=1; UPGRADE=1; shift ;;
    -*) die "unknown option: $1 (install.sh --help)" ;;
    *) OLD=1; REPO="$1"; shift ;;
  esac
done

# Old-style calls stop before anything is installed, so a script that expects a repo set up fails
# loudly, and says what to run instead.
if [ "$OLD" = 1 ]; then
  {
    echo "install.sh only installs the yass binary now; yass sets up and upgrades repos itself. Run:"
    echo "  install.sh$BIN_ARG"
    cd_part=""; [ -n "$REPO" ] && [ "$REPO" != "." ] && cd_part="cd $REPO && "
    if [ "$UPGRADE" = 1 ]; then echo "  ${cd_part}yass upgrade"
    else echo "  ${cd_part}yass init${INIT[*]+ ${INIT[*]}}"; fi
  } >&2
  exit 2
fi

sha256() { if command -v sha256sum >/dev/null; then sha256sum "$1"; else shasum -a 256 "$1"; fi | awk '{print $1}'; }

# Find the binary: $YASS_BIN, next to this script (a release folder, or a clone's bin/), or download a release.
BIN=""
HERE=""; [ -n "${BASH_SOURCE[0]:-}" ] && [ -f "${BASH_SOURCE[0]}" ] && HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
for c in "${YASS_BIN:-}" ${HERE:+"$HERE/yass" "$HERE/yass.exe" "$HERE/bin/yass" "$HERE/bin/yass.exe"}; do
  if [ -n "$c" ] && [ -f "$c" ] && [ -x "$c" ]; then BIN="$c"; break; fi
done
if [ -z "$BIN" ] && [ -n "$HERE" ]; then
  die "no yass binary next to this script (from a clone: go build -o bin/yass ./cmd/yass, with Go 1.24 or later)"
fi
if [ -z "$BIN" ]; then
  case "$(uname -s)" in Linux) os=linux ;; Darwin) os=darwin ;; *) die "download a release by hand on this OS: https://github.com/$REPO_SLUG/releases" ;; esac
  case "$(uname -m)" in x86_64|amd64) arch=amd64 ;; arm64|aarch64) arch=arm64 ;; *) die "no release for $(uname -m); build from source" ;; esac
  asset="yass_${os}_${arch}.tar.gz"
  if [ "$VERSION" = latest ]; then url="https://github.com/$REPO_SLUG/releases/latest/download"
  else url="https://github.com/$REPO_SLUG/releases/download/$VERSION"; fi
  TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
  echo "downloading $asset ($VERSION) from $REPO_SLUG …"
  curl -fsSL -o "$TMP/$asset" "$url/$asset"
  curl -fsSL -o "$TMP/checksums.txt" "$url/checksums.txt"
  want="$(awk -v f="$asset" '$2 == f {print $1}' "$TMP/checksums.txt")"
  got="$(sha256 "$TMP/$asset")"
  [ -n "$want" ] && [ "$want" = "$got" ] || die "checksum mismatch for $asset (expected ${want:-nothing}, got $got)"
  echo "checksum ok  $got"
  tar -xzf "$TMP/$asset" -C "$TMP"
  BIN="$TMP/yass_${os}_${arch}/yass"
fi

mkdir -p "$BIN_DIR"
BIN_DIR_ABS="$(cd "$BIN_DIR" && pwd)"
if [ "$(cd "$(dirname "$BIN")" && pwd)/$(basename "$BIN")" != "$BIN_DIR_ABS/$(basename "$BIN")" ]; then
  cp "$BIN" "$BIN_DIR_ABS/"
fi
BIN="$BIN_DIR_ABS/$(basename "$BIN")"; chmod +x "$BIN"
echo "installed $("$BIN" --version) to $BIN"

# You, the hook and your agents run `yass` from PATH, so say whether it finds this one, and how to fix it.
ON_PATH="$(command -v yass || true)"
home() { case "$1" in "$HOME"/*) echo "\$HOME${1#"$HOME"}" ;; *) echo "$1" ;; esac; }
if [ -n "$ON_PATH" ] && [ "$(cd "$(dirname "$ON_PATH")" && pwd)" = "$BIN_DIR_ABS" ]; then
  echo "it's on your PATH"
else
  d="$(home "$BIN_DIR_ABS")"
  case "$(basename "${SHELL:-}")" in
    # ~/.zshenv, not ~/.zshrc: agents and git hooks run non-interactive shells, which skip ~/.zshrc.
    zsh)  fix="echo 'export PATH=\"$d:\$PATH\"' >> ~/.zshenv" ;;
    bash) if [ "$(uname -s)" = Darwin ]; then rc="~/.bash_profile"; else rc="~/.bashrc"; fi
          fix="echo 'export PATH=\"$d:\$PATH\"' >> $rc" ;;
    fish) fix="fish_add_path $d" ;;
    *)    fix="echo 'export PATH=\"$d:\$PATH\"' >> ~/.profile" ;;
  esac
  echo
  if [ -z "$ON_PATH" ]; then echo "WARNING: $BIN_DIR_ABS isn't on your PATH, so you, the hook and your agents can't run yass."
  else echo "WARNING: \`yass\` on your PATH is $ON_PATH, not the one just installed in $BIN_DIR_ABS."; fi
  echo "  Fix it with:  $fix"
  echo "  then open a new terminal. Until then, run it as $BIN."
fi

cat <<'EOF'

Next, set up a repo:  cd your-repo && yass init   (yass init --help for --claude, --hooks and the rest)
A repo that already uses YASS needs nothing more. Later, `yass update` gets a newer binary.
EOF


## internal/yass/branches.go

package yass

import (
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"sort"
	"strconv"
	"strings"
	"time"

	"golang.org/x/mod/semver"
)

// The branch view (design §7): which unmerged local and remote-tracking branches with recent
// commits touch each change or piece, who touched it last, and how far that branch has it. It reads
// only what git already has (no fetch, no partial-clone download), in a fixed number of git
// processes: for-each-ref, one log over every branch, one cat-file for the files at their tips.

type branchRow struct {
	branch, author string
	when           time.Time
	done, total    int
	next           string
	unread         bool // the files at the branch tip aren't local (a partial clone): no progress
}

type branchView struct {
	rows  map[string][]branchRow // by change or piece folder, repo-relative
	fresh string                 // how old the remote-tracking refs are
}

var sinceDaysRE = regexp.MustCompile(`^(\d+)d$`)

// parseSince reads --since: Nd, or a date like 2026-10-01.
func parseSince(v string, now time.Time) (time.Time, error) {
	if v == "" {
		return now.AddDate(0, 0, -30), nil
	}
	if m := sinceDaysRE.FindStringSubmatch(v); m != nil {
		n, _ := strconv.Atoi(m[1])
		return now.AddDate(0, 0, -n), nil
	}
	if t, err := time.Parse("2006-01-02", v); err == nil {
		return t, nil
	}
	return time.Time{}, fmt.Errorf("--since takes days (14d) or a date (2026-10-01)")
}

func gitAtLeast(v string) bool {
	out, ok := git("", "version")
	if !ok {
		return false
	}
	f := strings.Fields(out) // git version 2.54.0 (Apple Git-157)
	if len(f) < 3 {
		return false
	}
	return semver.Compare(semverOf(f[2]), v) >= 0
}

// branches builds the view for the yass folders inside this repo. It returns nil when there's
// nothing unmerged to show.
func (r *Repo) branches(since time.Time) *branchView {
	var paths []string
	var root *Root
	for _, rt := range r.Roots {
		if within(rt.Dir, r.Top) {
			rel, _ := filepath.Rel(r.Top, filepath.Join(rt.Dir, "changes"))
			paths = append(paths, filepath.ToSlash(rel))
			if root == nil {
				root = rt
			}
		}
	}
	if root == nil {
		return nil
	}
	merged := r.mergedBranch(root)
	if merged == "" {
		return nil
	}
	local := strings.TrimPrefix(merged, "origin/") // the local copy of the merged branch is noise
	out, ok := git(r.Top, "for-each-ref", "--no-merged="+merged, "--format=%(refname:short)%09%(committerdate:unix)", "refs/heads", "refs/remotes")
	if !ok {
		return nil
	}
	var refs []string
	for _, l := range strings.Split(strings.TrimSpace(out), "\n") {
		name, ts, _ := strings.Cut(l, "\t")
		sec, _ := strconv.ParseInt(ts, 10, 64)
		if name == "" || name == merged || name == local || strings.HasSuffix(name, "/HEAD") || time.Unix(sec, 0).Before(since) {
			continue
		}
		refs = append(refs, name)
	}
	if len(refs) == 0 {
		return nil
	}
	// One log over every branch, limited to the yass folders' changes/: which branch reached each
	// commit (--source), who made it and when, and the files it touched.
	args := append([]string{"log", "--source", "--no-renames", "--format=%x01%S%x09%an%x09%ct", "--name-only",
		fmt.Sprintf("--since=@%d", since.Unix())}, refs...)
	args = append(append(append(args, "--not", merged), "--"), paths...)
	log, _ := git(r.Top, args...)
	type key struct{ branch, folder string }
	seen := map[key]*branchRow{}
	var order []key
	var ref, author string
	var when time.Time
	for _, l := range strings.Split(log, "\n") {
		if strings.HasPrefix(l, "\x01") {
			f := strings.SplitN(l[1:], "\t", 3)
			if len(f) == 3 {
				ref, author = f[0], f[1]
				sec, _ := strconv.ParseInt(f[2], 10, 64)
				when = time.Unix(sec, 0)
			}
			continue
		}
		if l == "" || ref == "" {
			continue
		}
		for _, folder := range changeFolders(l, paths) {
			k := key{ref, folder}
			if seen[k] == nil { // newest first: the first commit seen is the latest
				seen[k] = &branchRow{branch: ref, author: author, when: when}
				order = append(order, k)
			}
		}
	}
	if len(order) == 0 {
		return nil
	}
	// The files at each branch's tip, in one cat-file, never downloading what a partial clone lacks.
	partial, _ := git(r.Top, "config", "--get", "extensions.partialclone")
	canRead := strings.TrimSpace(partial) == "" || gitAtLeast("v2.44.0")
	var specs []string
	for _, k := range order {
		specs = append(specs, k.branch+":"+k.folder+"/change.md", k.branch+":"+k.folder+"/plan.md")
	}
	var blobs map[string]string
	if canRead {
		blobs = catFilesNoFetch(r.Top, specs)
	}
	v := &branchView{rows: map[string][]branchRow{}}
	for _, k := range order {
		row := seen[k]
		head, ok := blobs[k.branch+":"+k.folder+"/change.md"]
		if !ok {
			row.unread = true
		} else {
			for _, text := range []string{head, blobs[k.branch+":"+k.folder+"/plan.md"]} {
				for _, b := range boxes(normalize(text)) {
					if b.mark == markDropped {
						continue
					}
					row.total++
					if b.mark == markDone {
						row.done++
					}
				}
			}
			if m := nextRE.FindAllStringSubmatch(commentRE.ReplaceAllString(section(normalize(head), "Log"), ""), -1); len(m) > 0 {
				row.next = strings.TrimSpace(m[len(m)-1][1])
			}
		}
		v.rows[k.folder] = append(v.rows[k.folder], *row)
	}
	for f, rows := range v.rows {
		sort.SliceStable(rows, func(i, j int) bool { return rows[i].when.After(rows[j].when) })
		v.rows[f] = rows
	}
	v.fresh = r.fetchFreshness()
	return v
}

// changeFolders maps a file under a yass folder's changes/ to the change it's in and, when it's in
// a subfolder, that subfolder too (a piece, if it holds a change.md at the branch tip).
func changeFolders(file string, paths []string) []string {
	for _, p := range paths {
		if !strings.HasPrefix(file, p+"/") {
			continue
		}
		parts := strings.Split(strings.TrimPrefix(file, p+"/"), "/")
		if len(parts) < 2 {
			return nil
		}
		change := p + "/" + parts[0]
		if len(parts) >= 3 {
			return []string{change + "/" + parts[1]}
		}
		return []string{change}
	}
	return nil
}

// catFilesNoFetch is catFiles that never makes a partial clone download what it doesn't have.
func catFilesNoFetch(dir string, specs []string) map[string]string {
	return catFiles(dir, specs, "GIT_NO_LAZY_FETCH=1") // git 2.44+; older git is only asked outside partial clones
}

// fetchFreshness says how old the remote-tracking refs are: when FETCH_HEAD was last written.
func (r *Repo) fetchFreshness() string {
	out, _ := git(r.Top, "for-each-ref", "--count=1", "--format=%(refname)", "refs/remotes")
	if strings.TrimSpace(out) == "" {
		return "no remote branches here (nothing fetched); local branches only"
	}
	dir, _ := git(r.Top, "rev-parse", "--git-common-dir")
	d := strings.TrimSpace(dir)
	if !filepath.IsAbs(d) {
		d = filepath.Join(r.Top, d)
	}
	st, err := os.Stat(filepath.Join(d, "FETCH_HEAD"))
	if err != nil {
		return "remote branches as of the clone (never fetched since)"
	}
	return "remote branches as of the last fetch, " + ago(st.ModTime())
}

func ago(t time.Time) string {
	d := time.Since(t)
	switch {
	case d < time.Minute:
		return "just now"
	case d < time.Hour:
		return fmt.Sprintf("%d min ago", int(d.Minutes()))
	case d < 48*time.Hour:
		return fmt.Sprintf("%d h ago", int(d.Hours()))
	}
	return fmt.Sprintf("%d days ago", int(d.Hours()/24))
}

// branchRowMark marks a status row that's a branch line, printed as is rather than in columns.
const branchRowMark = "\x00branch"

// linesFor are a change's or piece's branch lines in yass status.
func (v *branchView) linesFor(r *Repo, c *Change, indent string) [][2]string {
	if v == nil {
		return nil
	}
	rel, err := filepath.Rel(r.Top, c.Path)
	if err != nil {
		return nil
	}
	var out [][2]string
	for _, l := range branchLines(v.rows[filepath.ToSlash(rel)], indent) {
		out = append(out, [2]string{branchRowMark, l})
	}
	return out
}

// branchLines are the lines for one change's or piece's branches.
func branchLines(rows []branchRow, indent string) []string {
	var out []string
	for _, b := range rows {
		line := fmt.Sprintf("%s↳ %s  %s, %s", indent, b.branch, b.author, ago(b.when))
		if b.unread {
			line += "  (its files aren't local)"
		} else if b.total > 0 {
			line += fmt.Sprintf("  %d/%d", b.done, b.total)
		}
		if b.next != "" {
			line += "  next: " + trunc(b.next, 60)
		}
		out = append(out, line)
	}
	return out
}


## internal/yass/commands.go

package yass

import (
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strconv"
	"strings"
	"time"
)

func cmdInit(a *args) error {
	if semverOf(binVersion) == "" {
		return noVersion()
	}
	cwd := getwd()
	top := repoTop(cwd)
	base := top
	if len(a.pos) > 0 {
		base = canon(a.pos[0])
	}
	cfg := filepath.Join(base, ConfigName)
	var made []string
	if a.b["private"] {
		if err := makePrivate(top, base, a.v["path"]); err != nil {
			return err
		}
	}
	if p := a.v["path"]; p != "" {
		if isFile(cfg) {
			c, _, err := loadConfig(cfg)
			if err != nil {
				return fmt.Errorf("%s: %v", cfg, err)
			}
			if strings.TrimSpace(c.Path) != p {
				return fmt.Errorf("%s already points to '%s'; edit it by hand to change it", cfg, c.Path)
			}
		} else {
			if err := write(cfg, configText(p)); err != nil {
				return err
			}
			made = append(made, cfg)
		}
	}
	y, configured := filepath.Join(base, "yass"), false
	if isFile(cfg) {
		dir, c, _, err := configDir(cfg)
		if err != nil {
			return fmt.Errorf("%s: %v", cfg, err)
		}
		y, configured = dir, strings.TrimSpace(c.Path) != ""
		if configured && !isDir(y) {
			found, _, linked := fromOtherCheckouts(cfg, strings.TrimSpace(c.Path))
			if found != "" {
				y = found
			} else if linked {
				return fmt.Errorf("this is a linked worktree, and %s points to %s, which no checkout of this clone has;"+
					" run `yass init` from the main checkout, so the plans land where every checkout looks", cfg, y)
			}
		}
	}
	for _, f := range []struct{ path, text string }{
		{filepath.Join(y, "changes", ".gitkeep"), ""},
		{filepath.Join(y, "archive", ".gitkeep"), ""},
		{filepath.Join(y, "README.md"), render("readme.md", nil)},
	} {
		if !exists(f.path) {
			if err := write(f.path, f.text); err != nil {
				return err
			}
			made = append(made, f.path)
		}
	}
	// Where the agent files go: the repo root when init sets up the repo itself, or the folder with
	// --agents (a monorepo part with its own AGENTS.md and skills). A folder without --agents gets
	// only its planning. The hook belongs to the repo root alone.
	agentDir := ""
	if len(a.pos) == 0 {
		agentDir = top
	} else if a.b["agents"] {
		agentDir = base
	}
	if a.b["no-agents"] {
		agentDir = ""
	}
	kw, freshAgents := &kitWriter{}, false
	if agentDir != "" {
		ap := filepath.Join(agentDir, "AGENTS.md")
		cur := read(ap)
		if strings.Contains(cur, "<!-- yass:begin") {
			kw.kept = append(kw.kept, ap) // init keeps the section; yass upgrade updates it
		} else {
			next := ""
			if strings.TrimSpace(cur) != "" {
				next = strings.TrimRight(cur, " \t\r\n") + "\n\n"
			}
			next += stampAgents(strings.TrimSpace(render("agents.md", nil)), stampVersion()) + "\n"
			if err := write(ap, next); err != nil {
				return err
			}
			made, freshAgents = append(made, ap), true
		}
		if err := kw.agentFiles(agentDir, a.b["claude"], a.b["global"]); err != nil {
			return err
		}
	}
	if len(a.pos) == 0 && !a.b["no-agents"] {
		if err := kw.hook(top); err != nil {
			return err
		}
	}
	made = append(made, kw.made...)
	r := &Repo{Top: top}
	for _, p := range made {
		fmt.Printf("wrote %s%s\n", r.disp(canon(p)), outsideNote(!within(canon(p), canon(top))))
	}
	if a.b["hooks"] {
		if err := turnOnHook(top); err != nil {
			return err
		}
	}
	if configured && !within(canon(y), top) {
		lr := findRepo()
		lr.link()
		if ln := filepath.Join(base, "yass"); isSymlinkTo(ln, canon(y)) {
			fmt.Printf("linked %s/ to %s (ignored through .git/info/exclude)\n", r.disp(ln), canon(y))
		}
		for _, n := range lr.Notes {
			fmt.Printf("note: %s\n", n)
		}
		fmt.Printf("note: %s is outside this repo, so give your agent access to it too"+
			" (Claude Code: claude --add-dir %s, or permissions.additionalDirectories in .claude/settings.local.json)\n", canon(y), canon(y))
	}
	if !configured {
		if _, ignored := git(top, "check-ignore", "-q", filepath.Join(y, "changes", ".gitkeep")); ignored {
			fmt.Printf("warning: %s is ignored by .gitignore, so yass won't see it; pick another folder or unignore it"+
				" (or keep it out of the repo on purpose with %s)\n", r.disp(y), ConfigName)
		}
	}
	if len(made) == 0 {
		fmt.Println("already set up")
	}
	if freshAgents && len(a.pos) == 0 {
		fmt.Print(`
YASS is set up. Next:
  1. Commit it:  git add -A && git commit -m "chore: adopt YASS"
  2. Ask your agent to start something:
       small:  "use yass-work to fix <bug>"
       large:  "use yass-shape to plan <feature>"
  3. ` + "`yass status`" + ` shows what's in flight.
`)
	}
	for _, p := range kw.kept {
		if olderThanBinary(readStamp(p)) {
			fmt.Printf("note: YASS's files here are older than this yass (%s); `yass upgrade` updates them\n", binVersion)
			break
		}
	}
	return nil
}

// makePrivate keeps a team folder out of git for this clone only: its own line in the clone's
// .git/info/exclude, and a yass.include entry in the clone's git config so YASS still finds it,
// from every worktree. Nothing is written to the repo.
func makePrivate(top, base, path string) error {
	rel, err := filepath.Rel(top, base)
	if err != nil || rel == "." || !within(base, top) {
		return fmt.Errorf("--private needs a folder inside the repo: yass init <folder> --path <plans> --private")
	}
	if path == "" && !isFile(filepath.Join(base, ConfigName)) {
		return fmt.Errorf("--private needs --path: the plans live outside the repo, and %s points to them", ConfigName)
	}
	if _, ok := git(top, "rev-parse", "--git-dir"); !ok {
		return fmt.Errorf("--private needs a git repo")
	}
	rel = filepath.ToSlash(rel)
	r := &Repo{Top: top}
	r.exclude(base)
	if out, _ := git(top, "config", "--get-all", "yass.include"); !contains(lines(strings.TrimSpace(out)), rel) {
		if _, ok := git(top, "config", "--add", "yass.include", rel); !ok {
			return fmt.Errorf("couldn't add %s to git config yass.include", rel)
		}
	}
	for _, n := range r.Notes {
		fmt.Printf("note: %s\n", n)
	}
	fmt.Printf("private: %s/ is kept out of git (.git/info/exclude) and found through git config yass.include, for this clone only\n", rel)
	return nil
}

func cmdNew(a *args) error {
	r := loadRepo()
	title := a.pos[0]
	var root *Root
	var base string
	if into := a.v["in"]; into != "" {
		if a.b["large"] {
			return fmt.Errorf("a piece can't be large; pieces nest one level deep")
		}
		parent, err := r.resolve(into, r.Active, false)
		if err != nil {
			return err
		}
		root, base = parent.Root, parent.Path
	} else {
		if root = r.rootForCwd(); root == nil {
			msg := r.noRoot("no yass/ folder here; run `yass init` first")
			for _, w := range r.Warnings {
				msg += "\n  " + w
			}
			return fmt.Errorf("%s", msg)
		}
		base = filepath.Join(root.Dir, "changes")
	}
	stem := time.Now().Format("2006-01-02") + "-" + slug(title, 40)
	name := stem
	for n := 2; exists(filepath.Join(base, name)); n++ {
		name = fmt.Sprintf("%s-%d", stem, n)
	}
	p := filepath.Join(base, name)
	platforms := a.v["platforms"]
	if platforms == "" {
		platforms = "all"
	}
	kw := map[string]string{"title": title, "platforms": platforms, "source": a.v["source"],
		"follows": a.v["follows"], "goal": a.v["goal"], "created": time.Now().UTC().Format(time.RFC3339)}
	head := "change.md"
	if a.b["large"] {
		head = "change-large.md"
	}
	files := map[string]string{"change.md": render(head, kw)}
	if a.b["large"] {
		files["prd.md"] = render("prd.md", map[string]string{"title": title})
		files["plan.md"] = render("plan.md", map[string]string{"title": title})
	}
	if a.b["design"] {
		files["design.md"] = render("design.md", map[string]string{"title": title})
	}
	for f, text := range files {
		if err := write(filepath.Join(p, f), text); err != nil {
			return err
		}
	}
	fmt.Println(r.disp(p))
	return nil
}

func statusLine(c *Change) string {
	done, total := c.progress()
	first := "-"
	if total > 0 {
		first = fmt.Sprintf("%d/%d", done, total)
	}
	var bits []string
	if s := c.state(); s != "in progress" {
		bits = append(bits, s)
	}
	if reason, waiting := c.blockedBy(); reason != "" {
		bits = append(bits, "BLOCKED: "+reason)
	} else if len(waiting) > 0 {
		bits = append(bits, "waiting on: "+labels(waiting))
	} else if n := c.nextStep(); n != "" {
		bits = append(bits, "next: "+trunc(n, 70))
	}
	return fmt.Sprintf("%5s  ", first) + strings.Join(bits, "  ")
}

// brief is a change's progress in one line, for commit messages and pull requests: a piece also
// gets its parent's, so a reader sees where the whole change stands.
func brief(c *Change) string {
	short := func(c *Change) string {
		if nameRE.MatchString(c.Name) {
			return c.Name[11:] // without the date
		}
		return c.Name
	}
	one := func(c *Change) string {
		done, total := c.progress()
		var bits []string
		if len(c.Pieces) > 0 { // a piece has none, so its part is just boxes
			n := 0
			for _, p := range c.Pieces {
				if p.state() == "done" {
					n++
				}
			}
			bits = append(bits, fmt.Sprintf("%d/%d pieces done", n, len(c.Pieces)))
		}
		bits = append(bits, fmt.Sprintf("%d/%d boxes", done, total))
		if c.state() == "done" {
			bits = append(bits, "done")
		}
		return short(c) + ": " + strings.Join(bits, ", ")
	}
	if c.Parent != nil {
		return one(c.Parent) + " · " + one(c)
	}
	return one(c)
}

func cmdStatus(a *args) (int, error) {
	r := loadRepo()
	if a.b["archived"] {
		r.readArchived()
	}
	r.pastKeep()
	if a.b["brief"] && len(a.pos) == 0 {
		return 0, fmt.Errorf("--brief needs a change: yass status <change> --brief")
	}
	if len(a.pos) > 0 {
		c, err := r.resolve(a.pos[0], nil, a.b["archived"])
		if err != nil {
			return 0, err
		}
		if a.b["brief"] {
			fmt.Println(brief(c))
			return 0, nil
		}
		showChange(r, c)
		return 0, nil
	}
	archived := a.b["archived"]
	since, err := parseSince(a.v["since"], time.Now())
	if err != nil {
		return 0, err
	}
	var view *branchView
	if !archived && !a.b["no-branches"] {
		view = r.branches(since)
	}
	multi := len(r.Roots) > 1
	pad := ""
	if multi {
		pad = "  "
	}
	for _, root := range r.Roots {
		pool := r.Active
		if archived {
			pool = r.Archived
		}
		var items []*Change
		var evicted []*Change
		for _, c := range pool {
			switch {
			case c.Root != root:
			case c.Evicted:
				evicted = append(evicted, c)
			default:
				items = append(items, c)
			}
		}
		if !archived {
			sort.SliceStable(items, func(i, j int) bool { return root.rank(items[i].Name) < root.rank(items[j].Name) })
		}
		if multi {
			fmt.Printf("%s/\n", r.disp(root.Dir))
		}
		if len(items) == 0 {
			if archived {
				fmt.Println(pad + "nothing archived yet")
			} else {
				fmt.Println(pad + "no active changes")
			}
		}
		var rows [][2]string
		for _, c := range items {
			if archived {
				t := ""
				if !strings.Contains(c.Name, slug(c.Title, 40)) {
					t = c.Title
				}
				rows = append(rows, [2]string{c.Name, t})
				continue
			}
			name := c.Name
			if c.Large {
				name += "  (large)"
			}
			rows = append(rows, [2]string{name, statusLine(c)})
			rows = append(rows, view.linesFor(r, c, pad+"      ")...)
			for i, p := range c.Pieces {
				branch := "├"
				if i == len(c.Pieces)-1 {
					branch = "└"
				}
				rows = append(rows, [2]string{"  " + branch + " " + p.Name, statusLine(p)})
				rows = append(rows, view.linesFor(r, p, pad+"        ")...)
			}
		}
		w := 0
		for _, row := range rows {
			if n := len([]rune(row[0])); n > w && row[0] != branchRowMark {
				w = n
			}
		}
		for _, row := range rows {
			if row[0] == branchRowMark {
				fmt.Println(row[1])
				continue
			}
			fmt.Println(strings.TrimRight(pad+padRight(row[0], w)+"  "+row[1], " "))
		}
		if len(evicted) > 0 {
			seen := map[string]bool{}
			for _, c := range evicted {
				m := filepath.ToSlash(filepath.Dir(c.Path))
				seen[m[len(m)-7:]] = true // <YYYY>/<MM>
			}
			months := sortedKeys(seen)
			fmt.Printf("%s… and %d evicted, from %s (in git history; see their month's .evicted file)\n", pad, len(evicted), strings.Join(months, ", "))
		}
	}
	if len(r.Roots) == 0 {
		fmt.Println(r.noRoot("no yass/ folder yet; run `yass init`"))
	}
	if view != nil && !archived {
		fmt.Printf("branches: unmerged work since %s; %s\n", since.Format("2006-01-02"), view.fresh)
	}
	if n := versionNote(r); n != "" {
		r.note("%s", n)
	}
	plans := r.plansNotes()
	for _, w := range r.Warnings {
		fmt.Printf("warning: %s\n", w)
	}
	for _, n := range r.Notes {
		fmt.Printf("note: %s\n", n)
	}
	for _, l := range plans {
		fmt.Println(l)
	}
	if a.b["strict"] && len(r.Warnings) > 0 {
		return 1, nil
	}
	return 0, nil
}

func showChange(r *Repo, c *Change) {
	fmt.Println(c.Title)
	where := r.disp(c.Path)
	if c.Parent != nil {
		where += "  (piece of " + c.Parent.Name + ")"
	}
	fmt.Println(where)
	for _, k := range []string{"platforms", "source", "follows", "blocked"} {
		if v := c.Meta[k]; v != "" {
			fmt.Printf("%s: %s\n", k, v)
		}
	}
	if c.Deps != nil {
		if _, waiting := c.blockedBy(); len(waiting) > 0 {
			fmt.Printf("waiting on: %s\n", labels(waiting))
		} else {
			fmt.Printf("note: everything blocked: names is done or archived; clear `blocked:`\n")
		}
	}
	done, total := c.progress()
	fmt.Printf("files: %s   progress: %d/%d  (%s)\n", strings.Join(mdFiles(c.Path), ", "), done, total, c.state())
	for _, p := range c.Pieces {
		d, t := p.progress()
		line := fmt.Sprintf("piece: %s  %d/%d  (%s)", p.Name, d, t, p.state())
		if reason, waiting := p.blockedBy(); reason != "" {
			line += "  BLOCKED: " + reason
		} else if len(waiting) > 0 {
			line += "  waiting on: " + labels(waiting)
		}
		fmt.Println(line)
	}
	if opened := c.allOpen(); len(opened) > 0 {
		fmt.Println("open:")
		for i, b := range opened {
			if i == 25 {
				break
			}
			fmt.Printf("  [%s] %s  (%s)\n", b.mark, trunc(b.text, 90), b.file)
		}
	}
	if n := c.nextStep(); n != "" {
		fmt.Printf("next: %s\n", n)
	}
	if commits, byAC, branch, warns := r.changeCode(c); len(commits) > 0 || len(warns) > 0 {
		if len(commits) > 0 {
			fmt.Printf("code on %s (from yass: trailers):\n", branch)
			for i, tc := range commits {
				if i == 10 {
					fmt.Printf("  … and %d more (git log %s -i --grep='^yass: %s')\n", len(commits)-10, branch, strings.ToLower(c.label()))
					break
				}
				fmt.Printf("  %s %s\n", tc.sha, trunc(tc.subject, 80))
			}
		}
		if len(byAC) > 0 {
			ids := make([]string, 0, len(byAC))
			for id := range byAC {
				ids = append(ids, id)
			}
			sort.Slice(ids, func(i, j int) bool {
				a, _ := strconv.Atoi(ids[i][2:])
				b, _ := strconv.Atoi(ids[j][2:])
				return a < b
			})
			fmt.Println("criteria:")
			for _, id := range ids {
				var shas []string
				for i, tc := range byAC[id] {
					if i == 10 {
						shas = append(shas, fmt.Sprintf("… %d more", len(byAC[id])-10))
						break
					}
					shas = append(shas, tc.sha)
				}
				fmt.Printf("  %s: %s\n", id, strings.Join(shas, ", "))
			}
		}
		for _, w := range warns {
			fmt.Printf("warning: %s\n", w)
		}
	}
	for _, w := range r.Warnings {
		if strings.HasPrefix(w, r.disp(c.Path)+string(filepath.Separator)) || strings.HasPrefix(w, r.disp(c.Path)+":") {
			fmt.Printf("warning: %s\n", w)
		}
	}
}

func cmdArchive(a *args) error {
	r := loadRepo()
	c, err := r.resolve(a.pos[0], nil, false)
	if err != nil {
		return err
	}
	if c.Parent != nil {
		return fmt.Errorf("%s is a piece of %s; pieces are archived with their change", c.Name, c.Parent.Name)
	}
	var problems []string
	if reason, waiting := c.blockedBy(); reason != "" {
		problems = append(problems, fmt.Sprintf("it's blocked: %s (clear `blocked:` when it isn't)", reason))
	} else if len(waiting) > 0 {
		problems = append(problems, fmt.Sprintf("it's waiting on %s, which isn't done yet", labels(waiting)))
	}
	if opened := c.allOpen(); len(opened) > 0 {
		msg := fmt.Sprintf("%d box(es) still open; finish them, or drop them with [-] and say why in Decisions:", len(opened))
		for i, b := range opened {
			if i == 10 {
				msg += "\n    …"
				break
			}
			msg += fmt.Sprintf("\n    [%s] %s  (%s)", b.mark, trunc(b.text, 80), b.file)
		}
		problems = append(problems, msg)
	} else if _, done, dropped := c.tally(); done+dropped == 0 {
		problems = append(problems, "it has no checkboxes at all, so nothing says it's done")
	}
	citeWarnings := r.archiveWarnings(c)
	if len(problems) > 0 && !a.b["force"] {
		return fmt.Errorf("%s isn't finished: %s\n  (--force archives it anyway)", c.Name, strings.Join(problems, "\n  "))
	}
	for _, o := range r.Archived {
		if o.Root == c.Root && o.Name == c.Name {
			return fmt.Errorf("%s is already archived, at %s; rename one of them", c.Name, r.disp(o.Path))
		}
	}
	delivered := c.delivered() // before the move, while the pieces are where they were read
	now := time.Now().UTC()
	dest := archiveDir(c.Root, c.Name, now)
	if exists(dest) {
		return fmt.Errorf("%s already exists", r.disp(dest))
	}
	if err := os.MkdirAll(filepath.Dir(dest), 0o755); err != nil {
		return err
	}
	if _, ok := git(c.Root.Dir, "mv", c.Path, dest); !ok {
		if err := os.Rename(c.Path, dest); err != nil {
			return err
		}
		git(c.Root.Dir, "add", "-A", "--", c.Path)
	}
	// The stamp is part of the move: an archived change is never edited after it lands.
	if head := filepath.Join(dest, "change.md"); isFile(head) {
		if err := write(head, setMeta(read(head), "archived", now.Format(time.RFC3339))); err != nil {
			return err
		}
	}
	// So is ticking the criteria the pieces delivered: the archive is the one closing commit.
	if plan := filepath.Join(dest, "plan.md"); isFile(plan) && len(delivered) > 0 {
		if err := write(plan, markDelivered(read(plan), delivered)); err != nil {
			return err
		}
	}
	git(c.Root.Dir, "add", "-A", "--", dest)
	fmt.Printf("archived %s\n", r.disp(dest))
	for _, w := range citeWarnings {
		fmt.Printf("warning: %s\n", w)
	}
	if n, k := r.kept(c.Root)+1, c.Root.keep(); n > k {
		fmt.Printf("note: %s holds %d archived changes, %d past its keep of %d; `yass evict` moves its oldest months to git history\n",
			r.disp(filepath.Join(c.Root.Dir, "archive")), n, n-k, k)
	}
	if q := filepath.Join(c.Root.Dir, QueueName); isFile(q) {
		cur := read(q)
		var keep []string
		for _, l := range strings.SplitAfter(cur, "\n") {
			if name, ok := queueEntry(strings.TrimRight(l, "\r\n")); !ok || name != c.Name {
				keep = append(keep, l)
			}
		}
		if next := strings.Join(keep, ""); next != cur {
			if err := write(q, next); err != nil {
				return err
			}
			git(c.Root.Dir, "add", "--", q)
			fmt.Printf("took it off %s\n", r.disp(q))
		}
	}
	commit := fmt.Sprintf("git commit -m \"yass: archive %s\"", c.Name)
	if out, ok := git(c.Root.Dir, "rev-parse", "--show-toplevel"); ok {
		if gitTop := canon(filepath.FromSlash(strings.TrimSpace(out))); gitTop != r.Top {
			commit = fmt.Sprintf("git -C %s commit -m \"yass: archive %s\"", gitTop, c.Name)
		}
		fmt.Printf("Commit it on its own: %s\n", commit)
	}
	return nil
}

func cmdRoot(a *args) error {
	r := loadRepo()
	if len(r.Roots) == 0 {
		msg := r.noRoot("no yass folder yet; run `yass init`")
		for _, w := range r.Warnings {
			msg += "\n  " + w
		}
		return fmt.Errorf("%s", msg)
	}
	for _, root := range r.Roots {
		if a.b["v"] {
			fmt.Printf("%s\t%s\n", root.Dir, r.kind(root))
		} else {
			fmt.Println(root.Dir)
		}
	}
	for _, n := range r.Notes { // stderr, so scripts reading the folders aren't disturbed
		fmt.Fprintf(os.Stderr, "note: %s\n", n)
	}
	return nil
}

func cmdTemplate(a *args) error {
	name := strings.TrimSuffix(a.pos[0], ".md") + ".md"
	if _, err := templates.ReadFile("templates/" + name); err != nil {
		return fmt.Errorf("no template '%s'; try change, change-large, prd, plan, design, readme or agents", a.pos[0])
	}
	fmt.Print(render(name, nil))
	return nil
}


## internal/yass/config.go

package yass

import (
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"

	"gopkg.in/yaml.v3"
)

// ConfigName is the settings file that can stand in for a yass/ folder and point to one elsewhere.
const ConfigName = "yass.yaml"

// Config is what a yass.yaml holds.
type Config struct {
	// Path is where the yass folder lives: absolute, relative to the yass.yaml, ~/…, or with
	// environment variables ($VAR, ${VAR}, or ${VAR:-default} for when it's unset), e.g.
	// ${YASS_HOME:-..}/my-project. Empty means yass/ next to it.
	Path string `yaml:"path"`
	// Branch is the code branch that counts as merged when boxes cite code commits.
	// Empty means origin/HEAD, else main, else master.
	Branch string `yaml:"branch"`
	// Ignore lists folders, relative to the yass.yaml, whose yass folders and yass.yaml files belong
	// to something else (examples, fixtures, vendored projects). From inside one, it's a project of its own.
	Ignore []string `yaml:"ignore"`
	// Archive holds the archive's settings.
	Archive struct {
		// Keep is how many archived changes (pieces count with their change) the yass folder's working
		// tree should keep: a soft limit. Past it, yass status says to run yass evict. 0 means the default.
		Keep int `yaml:"keep"`
	} `yaml:"archive"`
}

// DefaultKeep is how many archived changes a yass folder keeps when its yass.yaml doesn't say.
const DefaultKeep = 2000

var knownSettings = map[string]bool{"path": true, "branch": true, "ignore": true, "archive": true}

var knownArchiveSettings = map[string]bool{"keep": true}

// loadConfig reads a yass.yaml. It also returns any settings this version doesn't know.
func loadConfig(file string) (Config, []string, error) {
	var c Config
	data, err := os.ReadFile(file)
	if err != nil {
		return c, nil, err
	}
	var raw map[string]any
	if err := yaml.Unmarshal(data, &raw); err != nil {
		return c, nil, fmt.Errorf("not valid YAML: %v", err)
	}
	if err := yaml.Unmarshal(data, &c); err != nil {
		return c, nil, fmt.Errorf("not valid YAML: %v", err)
	}
	var unknown []string
	for k, v := range raw {
		if !knownSettings[k] {
			unknown = append(unknown, k)
		}
		if sub, ok := v.(map[string]any); ok && k == "archive" {
			for sk := range sub {
				if !knownArchiveSettings[sk] {
					unknown = append(unknown, k+"."+sk)
				}
			}
		}
	}
	sort.Strings(unknown)
	return c, unknown, nil
}

// resolvePath expands environment variables and ~ in p, and makes it absolute relative to base.
// ${VAR:-default} uses default when VAR is unset or empty, so side-by-side clones need no setup.
func resolvePath(p, base string) (string, error) {
	var missing []string
	s := os.Expand(p, func(k string) string {
		k, def, hasDef := strings.Cut(k, ":-")
		v := os.Getenv(k)
		if v == "" && hasDef {
			return def
		}
		if v == "" {
			missing = append(missing, k)
		}
		return v
	})
	if len(missing) > 0 {
		return "", fmt.Errorf("$%s isn't set", missing[0])
	}
	if s == "~" || strings.HasPrefix(s, "~/") {
		home, err := os.UserHomeDir()
		if err != nil {
			return "", err
		}
		s = home + s[1:]
	}
	if !filepath.IsAbs(s) {
		s = filepath.Join(base, s)
	}
	return canon(s), nil
}

// configDir is the yass folder a yass.yaml points to.
func configDir(file string) (string, Config, []string, error) {
	owner := filepath.Dir(file)
	c, unknown, err := loadConfig(file)
	if err != nil {
		return "", c, nil, err
	}
	if strings.TrimSpace(c.Path) == "" {
		return filepath.Join(owner, "yass"), c, unknown, nil
	}
	dir, err := resolvePath(strings.TrimSpace(c.Path), owner)
	return dir, c, unknown, err
}

// ignoredDirs resolves a yass.yaml's ignore list (folders, or glob patterns like examples/*) to
// folders inside the folder it sits in, and says what's wrong with any entry it can't use.
func ignoredDirs(file string, c Config) ([]string, []string) {
	owner := filepath.Dir(file)
	var dirs, bad []string
	for _, e := range c.Ignore {
		e = strings.TrimSpace(e)
		if e == "" {
			continue
		}
		if filepath.IsAbs(filepath.FromSlash(e)) || !within(filepath.Join(owner, filepath.FromSlash(e)), owner) {
			bad = append(bad, fmt.Sprintf("ignore: '%s' isn't inside the folder yass.yaml is in", e))
			continue
		}
		matches, err := filepath.Glob(filepath.Join(owner, filepath.FromSlash(e)))
		if err != nil {
			bad = append(bad, fmt.Sprintf("ignore: '%s' isn't a valid pattern", e))
			continue
		}
		n, self := 0, false
		for _, m := range matches {
			if d := canon(m); d == owner {
				self = true
			} else if isDir(d) {
				dirs = append(dirs, d)
				n++
			}
		}
		if self {
			bad = append(bad, fmt.Sprintf("ignore: '%s' is the folder yass.yaml is in", e))
		} else if n == 0 {
			bad = append(bad, fmt.Sprintf("ignore: '%s' doesn't match a folder under the one yass.yaml is in", e))
		}
	}
	return dirs, bad
}

func configText(path string) string {
	return `# YASS settings for this folder.
# path: where its yass folder lives. Absolute, relative to this file, ~/…, or with environment
# variables, e.g. ${YASS_HOME}/my-project or ${YASS_HOME:-..}/my-project (.. when it's unset).
# Without it, yass/ next to this file.
# ignore: a list of folders (or patterns like examples/*) whose yass folders belong to something else.
path: '` + strings.ReplaceAll(path, "'", "''") + "'\n"
}


## internal/yass/config_test.go

package yass

import (
	"os"
	"path/filepath"
	"testing"
)

func TestResolvePath(t *testing.T) {
	base := t.TempDir()
	t.Setenv("YASS_TEST_HOME", "/srv/plans")
	t.Setenv("YASS_TEST_EMPTY", "")
	home, _ := os.UserHomeDir()
	for in, want := range map[string]string{
		"../elsewhere":               filepath.Join(filepath.Dir(base), "elsewhere"),
		"planning":                   filepath.Join(base, "planning"),
		"/abs/yass":                  "/abs/yass",
		"$YASS_TEST_HOME":            "/srv/plans",
		"${YASS_TEST_HOME}/my-repo":  "/srv/plans/my-repo",
		"~/yass/my-repo":             filepath.Join(home, "yass", "my-repo"),
		"${YASS_TEST_HOME:-..}/web":  "/srv/plans/web",
		"${YASS_TEST_UNSET:-..}/web": filepath.Join(filepath.Dir(base), "web"),
		"${YASS_TEST_EMPTY:-~/p}":    filepath.Join(home, "p"),
		"${YASS_TEST_UNSET:-}":       base,
	} {
		got, err := resolvePath(in, base)
		if err != nil || got != canon(want) {
			t.Errorf("resolvePath(%q) = %q, %v; want %q", in, got, err, canon(want))
		}
	}
	if _, err := resolvePath("${YASS_TEST_UNSET}/x", base); err == nil || err.Error() != "$YASS_TEST_UNSET isn't set" {
		t.Errorf("unset variable: %v", err)
	}
}

func TestConfigTextRoundTrips(t *testing.T) {
	f := filepath.Join(t.TempDir(), ConfigName)
	for _, p := range []string{"../private", "${YASS_HOME}/it's mine", `C:\plans`} {
		os.WriteFile(f, []byte(configText(p)), 0o644)
		c, unknown, err := loadConfig(f)
		if err != nil || c.Path != p || len(unknown) != 0 {
			t.Errorf("round trip %q: got %q %v %v", p, c.Path, unknown, err)
		}
	}
	os.WriteFile(f, []byte("path: x\ncolor: blue\n"), 0o644)
	if _, unknown, _ := loadConfig(f); len(unknown) != 1 || unknown[0] != "color" {
		t.Errorf("unknown = %v", unknown)
	}
}

func TestCanonResolvesMissingPathsThroughSymlinks(t *testing.T) {
	real := t.TempDir()
	link := filepath.Join(t.TempDir(), "link")
	if err := os.Symlink(real, link); err != nil {
		t.Skip("no symlinks here")
	}
	want := filepath.Join(canon(real), "not", "there")
	if got := canon(filepath.Join(link, "not", "there")); got != want {
		t.Errorf("canon = %q, want %q", got, want)
	}
}

func TestArchiveKeep(t *testing.T) {
	dir := t.TempDir()
	f := filepath.Join(dir, ConfigName)
	os.WriteFile(f, []byte("archive:\n  keep: 3\n  kepp: 4\n"), 0o644)
	c, unknown, err := loadConfig(f)
	if err != nil || c.Archive.Keep != 3 || len(unknown) != 1 || unknown[0] != "archive.kepp" {
		t.Fatalf("loadConfig = %+v, %v, %v", c, unknown, err)
	}
	if k := (&Root{Config: f}).keep(); k != 3 {
		t.Fatalf("keep = %d, want 3", k)
	}
	if k := (&Root{}).keep(); k != DefaultKeep {
		t.Fatalf("keep without a yass.yaml = %d, want %d", k, DefaultKeep)
	}
}


## internal/yass/decisions.go

package yass

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"sort"
	"strconv"
	"strings"
	"time"
)

// yass decisions: the ## Decisions entries of every change, active and archived, in every yass
// folder, so an agent asking "why did we…" gets a bounded answer instead of grepping the archive.

var (
	entryRE  = regexp.MustCompile(`^[-*+]\s+(.*\S)\s*$`)
	creditRE = regexp.MustCompile(`\s*\(([^()]*)\)\s*$`)
	dayRE    = regexp.MustCompile(`^\d{4}-\d{2}-\d{2}$`)
)

type decision struct {
	Change   string  `json:"change"`
	Title    string  `json:"title"`
	Who      string  `json:"who"`
	Decision string  `json:"decision"`
	Date     *string `json:"date"`
	Created  *string `json:"created"`
	Archived *string `json:"archived"`
	when     string  // what filters and sorting go by: its own date, else archived, else created
	order    int
}

// parseEntry splits a Decisions line into the decision and its credit: the last parenthesis, whose
// text after the last comma is the date when it's a YYYY-MM-DD (design §6).
func parseEntry(line string) (text, who, date string) {
	text = line
	m := creditRE.FindStringSubmatchIndex(line)
	if m == nil {
		return text, "", ""
	}
	text, who = strings.TrimSpace(line[:m[0]]), strings.TrimSpace(line[m[2]:m[3]])
	if i := strings.LastIndex(who, ","); i >= 0 && dayRE.MatchString(strings.TrimSpace(who[i+1:])) {
		who, date = strings.TrimSpace(who[:i]), strings.TrimSpace(who[i+1:])
	}
	return text, who, date
}

func day(stamp string) string {
	if t, err := time.Parse(time.RFC3339, stamp); err == nil {
		return t.UTC().Format("2006-01-02")
	}
	if len(stamp) >= 10 && dayRE.MatchString(stamp[:10]) {
		return stamp[:10]
	}
	return ""
}

func ptr(s string) *string {
	if s == "" {
		return nil
	}
	return &s
}

// decisionsOf lists a change's own entries (its pieces have their own).
func decisionsOf(c *Change) []decision {
	top := c
	if c.Parent != nil {
		top = c.Parent
	}
	created, archived := day(c.created()), day(top.Meta["archived"])
	var out []decision
	for _, l := range lines(commentRE.ReplaceAllString(section(c.Head, "Decisions"), "")) {
		m := entryRE.FindStringSubmatch(l)
		if m == nil {
			continue
		}
		text, who, date := parseEntry(m[1])
		d := decision{Change: c.label(), Title: c.Title, Who: who, Decision: text,
			Date: ptr(date), Created: ptr(created), Archived: ptr(archived), order: len(out)}
		for _, w := range []string{date, archived, created} {
			if w != "" {
				d.when = w
				break
			}
		}
		out = append(out, d)
	}
	return out
}

// related is the change named and every change linked to it through follows:, both ways, with
// their pieces.
func related(all []*Change, start *Change) map[*Change]bool {
	top := func(c *Change) *Change {
		if c.Parent != nil {
			return c.Parent
		}
		return c
	}
	byName := map[string]*Change{}
	for _, c := range all {
		byName[c.Name] = c
	}
	links := map[*Change][]*Change{}
	for _, c := range all {
		if f := strings.TrimRight(c.Meta["follows"], "/"); f != "" {
			if o := byName[f[strings.LastIndex(f, "/")+1:]]; o != nil {
				a, b := top(c), top(o)
				links[a], links[b] = append(links[a], b), append(links[b], a)
			}
		}
	}
	seen := map[*Change]bool{}
	var walk func(*Change)
	walk = func(c *Change) {
		if seen[c] {
			return
		}
		seen[c] = true
		for _, p := range c.Pieces {
			seen[p] = true
		}
		for _, o := range links[c] {
			walk(o)
		}
	}
	if start.Parent != nil {
		seen[start] = true // a piece alone: itself, and what its change is linked to
		for _, o := range links[start.Parent] {
			walk(o)
		}
		return seen
	}
	walk(start)
	return seen
}

func cmdDecisions(a *args) (int, error) {
	r := loadRepo()
	if len(r.Roots) == 0 {
		fmt.Println(r.noRoot("no yass folder here; `yass init` sets one up"))
		return 0, nil
	}
	r.readArchived()
	all := append(r.everything(), flatten(r.Archived)...)
	for _, k := range []string{"since", "until"} {
		if v := a.v[k]; v != "" && !dayRE.MatchString(v) {
			return 0, fmt.Errorf("--%s takes a date like 2026-10-07", k)
		}
	}
	limit := 50
	if v := a.v["limit"]; v != "" {
		n, err := strconv.Atoi(v)
		if err != nil || n < 1 {
			return 0, fmt.Errorf("--limit takes a number above 0")
		}
		limit = n
	}

	// Evicted months are read from git only when the query reaches them.
	stubs := r.evictedStubs()
	done := map[*Root]map[string]bool{}
	var missing []string
	readMonths := func(pick func(root *Root, m string) bool) {
		want := map[*Root][]string{}
		for root, months := range stubs {
			for m := range months {
				if !done[root][m] && pick(root, m) {
					if done[root] == nil {
						done[root] = map[string]bool{}
					}
					done[root][m] = true
					want[root] = append(want[root], m)
				}
			}
		}
		for root := range want {
			sort.Strings(want[root]) // oldest month first, whatever order the map gave
		}
		if len(want) > 0 {
			missing = append(missing, r.readEvicted(want)...)
			all = append(r.everything(), flatten(r.Archived)...)
		}
	}
	monthOfStub := func(c *Change) string {
		if c.Parent != nil {
			c = c.Parent
		}
		m := filepath.ToSlash(filepath.Dir(c.Path))
		return m[len(m)-7:]
	}

	if sha := strings.ToLower(a.v["cites"]); sha != "" {
		if !shaRE.MatchString(sha) {
			return 0, fmt.Errorf("--cites takes a commit hash (7 to 40 hex digits)")
		}
		readMonths(func(*Root, string) bool { return true })
		return citations(a, all, sha, limit, missing)
	}

	var only map[*Change]bool
	if name := a.v["change"]; name != "" {
		c, err := r.resolve(name, all, false)
		if err != nil {
			return 0, err
		}
		// Follow the chain into evicted months, reading each one the chain reaches.
		for {
			only = related(all, c)
			reach := map[*Root]map[string]bool{}
			for x := range only {
				if x.Evicted {
					if reach[x.Root] == nil {
						reach[x.Root] = map[string]bool{}
					}
					reach[x.Root][monthOfStub(x)] = true
				}
			}
			before := len(missing) + countDone(done)
			readMonths(func(root *Root, m string) bool { return reach[root][m] })
			if len(missing)+countDone(done) == before {
				break
			}
			for _, x := range all { // the change named may have been a stub, now read
				if x.Name == c.Name && x.Root == c.Root {
					c = x
				}
			}
		}
	} else if since := a.v["since"]; since != "" {
		readMonths(func(_ *Root, m string) bool { return monthEnd(m) >= since })
	}

	words := strings.Fields(strings.ToLower(a.v["about"]))
	collect := func() []decision {
		var found []decision
		for _, c := range all {
			if only != nil && !only[c] {
				continue
			}
		entries:
			for _, d := range decisionsOf(c) {
				if (a.v["since"] != "" && d.when < a.v["since"]) || (a.v["until"] != "" && (d.when == "" || d.when > a.v["until"])) {
					continue
				}
				hay := strings.ToLower(d.Decision + " " + d.Title)
				for _, w := range words {
					if !strings.Contains(hay, w) {
						continue entries
					}
				}
				found = append(found, d)
			}
		}
		return found
	}
	found := collect()
	// Without dates or a change to go by, evicted months are older than everything in the tree:
	// read them only when what's here doesn't fill the answer.
	if only == nil && a.v["since"] == "" && len(found) < limit && len(stubs) > 0 {
		readMonths(func(*Root, string) bool { return true })
		found = collect()
	}
	sort.SliceStable(found, func(i, j int) bool {
		if found[i].when != found[j].when {
			return found[i].when > found[j].when
		}
		if found[i].Change != found[j].Change {
			return found[i].Change < found[j].Change
		}
		return found[i].order < found[j].order
	})
	left := 0
	if len(found) > limit {
		left, found = len(found)-limit, found[:limit]
	}
	if len(missing) > 0 && a.b["json"] {
		fmt.Fprintf(os.Stderr, "yass: %s\n", missingNote(missing))
	}
	for _, w := range r.Warnings { // only what bears on the answer; yass status shows the rest
		if strings.Contains(w, "can't read it") {
			fmt.Fprintf(os.Stderr, "warning: %s\n", w)
		}
	}
	if a.b["json"] {
		if found == nil {
			found = []decision{}
		}
		out, _ := json.MarshalIndent(found, "", "  ")
		fmt.Println(string(out))
		if left > 0 {
			fmt.Fprintf(os.Stderr, "yass: %d more not shown; narrow the query or raise --limit\n", left)
		}
		return 0, nil
	}
	if len(found) == 0 {
		fmt.Println("no decisions match")
		return 0, nil
	}
	for _, d := range found {
		date := "          "
		if d.Date != nil {
			date = *d.Date
		}
		line := fmt.Sprintf("%s  %s  %s", date, d.Change, d.Decision)
		if d.Who != "" {
			line += " (" + d.Who + ")"
		}
		var also []string
		if d.Created != nil {
			also = append(also, "created "+*d.Created)
		}
		if d.Archived != nil {
			also = append(also, "archived "+*d.Archived)
		}
		if len(also) > 0 {
			line += "  [" + strings.Join(also, ", ") + "]"
		}
		fmt.Println(line)
	}
	if left > 0 {
		fmt.Printf("… %d more not shown; narrow the query or raise --limit\n", left)
	}
	if len(missing) > 0 {
		fmt.Printf("note: %s\n", missingNote(missing))
	}
	return 0, nil
}

func countDone(done map[*Root]map[string]bool) int {
	n := 0
	for _, ms := range done {
		n += len(ms)
	}
	return n
}

type citation struct {
	Change string `json:"change"`
	Title  string `json:"title"`
	File   string `json:"file"`
	Mark   string `json:"mark"`
	Box    string `json:"box"`
}

// citations is yass decisions --cites: the boxes that cite a commit, by any prefix of its hash.
func citations(a *args, all []*Change, sha string, limit int, missing []string) (int, error) {
	var found []citation
	for _, c := range all {
		for _, b := range c.ownBoxes() {
			for _, ref := range codeRefs(b.text) {
				if strings.HasPrefix(ref, sha) || strings.HasPrefix(sha, ref) {
					found = append(found, citation{c.label(), c.Title, b.file, b.mark, b.text})
					break
				}
			}
		}
	}
	left := 0
	if len(found) > limit {
		left, found = len(found)-limit, found[:limit]
	}
	if a.b["json"] {
		if found == nil {
			found = []citation{}
		}
		out, _ := json.MarshalIndent(found, "", "  ")
		fmt.Println(string(out))
		if len(missing) > 0 {
			fmt.Fprintf(os.Stderr, "yass: %s\n", missingNote(missing))
		}
		return 0, nil
	}
	if len(found) == 0 {
		fmt.Printf("no box cites %s\n", sha)
	}
	for _, f := range found {
		fmt.Printf("%s  %s  [%s] %s\n", f.Change, f.File, f.Mark, f.Box)
	}
	if left > 0 {
		fmt.Printf("… %d more not shown; raise --limit\n", left)
	}
	if len(missing) > 0 {
		fmt.Printf("note: %s\n", missingNote(missing))
	}
	return 0, nil
}


## internal/yass/decisions_test.go

package yass

import "testing"

func TestParseEntry(t *testing.T) {
	for _, tc := range []struct{ line, text, who, date string }{
		{"X - Y (Jaime, 2026-10-07)", "X - Y", "Jaime", "2026-10-07"},
		{"X - Y (claude, agreed by Jaime)", "X - Y", "claude, agreed by Jaime", ""},
		{"X - Y (Jaime, months from claude, 2026-10-07)", "X - Y", "Jaime, months from claude", "2026-10-07"},
		{"X (see #41) - Y (sam)", "X (see #41) - Y", "sam", ""},
		{"No credit at all", "No credit at all", "", ""},
		{"Odd (2026-10-07)", "Odd", "2026-10-07", ""},
	} {
		text, who, date := parseEntry(tc.line)
		if text != tc.text || who != tc.who || date != tc.date {
			t.Errorf("parseEntry(%q) = %q, %q, %q; want %q, %q, %q", tc.line, text, who, date, tc.text, tc.who, tc.date)
		}
	}
}


## internal/yass/deinit.go

package yass

import (
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

// cmdDeinit takes YASS back out of a repo: what yass init and yass upgrade write (the playbooks,
// AGENTS.md sections, the CLAUDE.md import and the hook), from the repo root down. The yass folders
// and yass.yaml stay, since they're the repo's plans and history; it says how to remove them. With
// global, the user folder's playbooks go too. It never commits: everything it removes in the repo is
// in git, so the diff shows it and git restore undoes it.
func cmdDeinit(a *args) error {
	r := findRepo()
	var removed []string
	gone := func(p, what string) { removed = append(removed, what+r.disp(p)+outsideNote(!within(p, r.Top))) }

	out, _ := git(r.Top, "ls-files", "--cached", "--others", "--exclude-standard", "-z")
	skillDirs, agents, hookDir := map[string]bool{}, []string{}, ""
	for _, rel := range strings.Split(out, "\x00") {
		p := filepath.Join(r.Top, filepath.FromSlash(rel))
		if rel == "" || r.ignored(p) || !exists(p) {
			continue
		}
		switch {
		case repoSkillRE.MatchString(rel):
			skillDirs[filepath.Dir(p)] = true
		case repoAgentRE.MatchString(rel) && strings.Contains(read(p), "<!-- yass:begin"):
			agents = append(agents, p)
		case rel == HookPath+"/pre-commit":
			hookDir = filepath.Dir(p)
		}
	}
	userDirs := yassSkillDirs(userSkillsDir(), userClaudeSkillsDir())
	if a.b["global"] {
		for _, d := range userDirs {
			skillDirs[d] = true
		}
	}
	var dirs []string
	for d := range skillDirs {
		if yassPlaybook(filepath.Join(d, "SKILL.md")) {
			dirs = append(dirs, d)
		}
	}
	sort.Strings(dirs)
	for _, d := range dirs {
		if err := os.RemoveAll(d); err != nil {
			return err
		}
		pruneEmpty(filepath.Dir(d), r.Top)
		gone(d, "removed ")
	}

	sort.Strings(agents)
	for _, p := range agents {
		rest := strings.TrimSpace(agentsRE.ReplaceAllLiteralString(read(p), ""))
		if rest != "" {
			if err := write(p, rest+"\n"); err != nil {
				return err
			}
			gone(p, "removed the YASS section of ")
			continue
		}
		if err := os.Remove(p); err != nil {
			return err
		}
		gone(p, "removed ")
		// CLAUDE.md's @AGENTS.md import, which yass init --claude adds, now points at nothing.
		cm := filepath.Join(filepath.Dir(p), "CLAUDE.md")
		if cur := read(cm); strings.Contains(cur, "@AGENTS.md") {
			var keep []string
			for _, l := range lines(cur) {
				if strings.TrimSpace(l) != "@AGENTS.md" {
					keep = append(keep, l)
				}
			}
			if next := strings.TrimSpace(strings.Join(keep, "\n")); next == "" {
				if err := os.Remove(cm); err != nil {
					return err
				}
				gone(cm, "removed ")
			} else {
				if err := write(cm, next+"\n"); err != nil {
					return err
				}
				gone(cm, "removed the @AGENTS.md import from ")
			}
		}
	}

	if hookDir != "" {
		for _, name := range hookScripts {
			if p := filepath.Join(hookDir, name); exists(p) {
				if err := os.Remove(p); err != nil {
					return err
				}
				gone(p, "removed ")
			}
		}
		pruneEmpty(hookDir, r.Top)
	}
	if local, _ := git(r.Top, "config", "--local", "--get", "core.hooksPath"); strings.TrimSpace(local) == HookPath {
		git(r.Top, "config", "--local", "--unset", "core.hooksPath")
		removed = append(removed, "unset core.hooksPath (it pointed at "+HookPath+")")
	}
	if f := useFile(); f != "" {
		os.Remove(f) // yass use's choice for this worktree: never committed, nothing to list
	}

	for _, l := range removed {
		fmt.Println(l)
	}
	if len(removed) == 0 {
		fmt.Println("no YASS playbooks, AGENTS.md section or hook here to remove")
	}
	if !a.b["global"] && len(userDirs) > 0 {
		fmt.Printf("note: your user folder's playbooks stay (%s), since they serve every repo; `yass deinit --global` removes them too\n",
			strings.Join(userDirs, ", "))
	}
	for _, f := range hookCalls(r.Top) {
		fmt.Printf("note: %s still runs `yass hook`; remove that line by hand\n", f)
	}

	var kept, rm []string
	for _, root := range r.Roots {
		switch {
		case root.Config != "" && within(root.Dir, r.Top) && !root.Included:
			kept = append(kept, r.disp(root.Dir)+"/ and "+r.disp(root.Config))
			rm = append(rm, rmCmd(r, root.Dir, root.Config))
		case root.Config != "":
			kept = append(kept, r.disp(root.Config)+" (pointing to "+r.disp(root.Dir)+")")
			rm = append(rm, rmCmd(r, root.Config))
			if ln := filepath.Join(root.Owner, "yass"); isSymlinkTo(ln, root.Dir) {
				rm = append(rm, "rm "+r.disp(ln))
			}
		case within(root.Dir, r.Top) && !root.Included:
			kept = append(kept, r.disp(root.Dir)+"/")
			rm = append(rm, rmCmd(r, root.Dir))
		default:
			kept = append(kept, r.disp(root.Dir)+"/")
			rm = append(rm, "rm -r "+r.disp(root.Dir))
		}
	}
	if len(kept) > 0 {
		fmt.Printf("\nKept your plans: %s. They're the record of what was changed and why: keep them, or remove them:\n  %s\n",
			strings.Join(kept, ", "), strings.Join(rm, " && "))
	}
	if len(removed) > 0 {
		fmt.Println("\nReview it with `git status`, then commit it:  git add -A && git commit -m \"chore: remove YASS\"")
	}
	bin, _ := os.Executable()
	fmt.Printf("The yass binary stays (%s): delete it once you're done with YASS everywhere. `yass init` sets this repo up again.\n", bin)
	return nil
}

// rmCmd is the command that removes paths in the repo: git rm when git tracks them, else rm.
func rmCmd(r *Repo, paths ...string) string {
	args := []string{"ls-files", "--"}
	var shown []string
	for _, p := range paths {
		args, shown = append(args, p), append(shown, r.disp(p))
	}
	if out, _ := git(r.Top, args...); strings.TrimSpace(out) != "" {
		return "git rm -r -q " + strings.Join(shown, " ")
	}
	return "rm -r " + strings.Join(shown, " ")
}

// yassSkillDirs lists the YASS playbook folders (yass-*) in skills folders.
func yassSkillDirs(roots ...string) []string {
	var out []string
	for _, d := range roots {
		m, _ := filepath.Glob(filepath.Join(d, "yass-*", "SKILL.md"))
		for _, p := range m {
			if yassPlaybook(p) {
				out = append(out, filepath.Dir(p))
			}
		}
	}
	return out
}

// yassPlaybook says whether a SKILL.md is one of YASS's: a playbook this binary carries, or one
// stamped with a yass-version (a playbook an older or newer version carried).
func yassPlaybook(p string) bool {
	names, _ := kitSkills()
	return contains(names, filepath.Base(filepath.Dir(p))) || readStamp(p) != ""
}

// pruneEmpty removes dir and its parents while they're empty, stopping at top.
func pruneEmpty(dir, top string) {
	for within(dir, top) && dir != top {
		if os.Remove(dir) != nil {
			return
		}
		dir = filepath.Dir(dir)
	}
}

// hookCalls lists the hook-manager files at the repo root that still run `yass hook`.
func hookCalls(top string) []string {
	files, _ := filepath.Glob(filepath.Join(top, ".husky", "*"))
	files = append(files, filepath.Join(top, "lefthook.yml"), filepath.Join(top, ".lefthook.yml"), filepath.Join(top, ".pre-commit-config.yaml"))
	var out []string
	for _, f := range files {
		if isFile(f) && strings.Contains(read(f), "yass hook") {
			rel, _ := filepath.Rel(top, f)
			out = append(out, filepath.ToSlash(rel))
		}
	}
	return out
}


## internal/yass/evict.go

package yass

import (
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"time"

	"gopkg.in/yaml.v3"
)

// Eviction moves a yass folder's oldest archived months out of the working tree into git history
// when the folder is past its keep: setting. Each evicted month leaves archive/<YYYY>/<MM>.evicted,
// written once, naming its changes and the commit that still has them (design §3–4).

const evictedExt = ".evicted"

type evictedMonth struct {
	Month   string   `yaml:"-"`       // 2026/03
	Commit  string   `yaml:"commit"`  // the last commit that touched the month's folder
	Path    string   `yaml:"path"`    // the month's folder at that commit, from the top of its git repo
	Changes []string `yaml:"changes"` // top-level changes, sorted
}

// manifestText is a .evicted file. It's the same for the same month from the same history, so two
// branches that evict a month write identical files and merge cleanly.
func manifestText(m evictedMonth) string {
	var b strings.Builder
	b.WriteString("# Evicted by yass evict. Read a change back: git show <commit>:<path>/<name>/change.md\n")
	fmt.Fprintf(&b, "commit: %s\npath: %s\nchanges:\n", m.Commit, m.Path)
	for _, c := range m.Changes {
		fmt.Fprintf(&b, "  - %s\n", c)
	}
	return b.String()
}

// evictedMonths reads a yass folder's .evicted files, oldest month first.
func evictedMonths(root *Root) []evictedMonth {
	base := filepath.Join(root.Dir, "archive")
	var out []evictedMonth
	for _, y := range subdirs(base) {
		if !yearRE.MatchString(y) {
			continue
		}
		files, _ := filepath.Glob(filepath.Join(base, y, "*"+evictedExt))
		sort.Strings(files)
		for _, f := range files {
			mm := strings.TrimSuffix(filepath.Base(f), evictedExt)
			if !monthRE.MatchString(mm) {
				continue
			}
			var m evictedMonth
			if yaml.Unmarshal([]byte(read(f)), &m) != nil {
				continue
			}
			m.Month = y + "/" + mm
			out = append(out, m)
		}
	}
	return out
}

// archiveMonths lists the month folders in a yass folder's archive that hold changes, oldest
// first, with how many top-level changes each holds.
func archiveMonths(root *Root) ([]string, map[string]int) {
	base := filepath.Join(root.Dir, "archive")
	var months []string
	count := map[string]int{}
	for _, y := range subdirs(base) {
		if !yearRE.MatchString(y) {
			continue
		}
		for _, m := range subdirs(filepath.Join(base, y)) {
			if !monthRE.MatchString(m) {
				continue
			}
			if n := len(subdirs(filepath.Join(base, y, m))); n > 0 {
				months = append(months, y+"/"+m)
				count[y+"/"+m] = n
			}
		}
	}
	return months, count
}

// kept is how many archived changes a yass folder holds in its working tree.
func (r *Repo) kept(root *Root) int {
	n := 0
	for _, c := range r.Archived {
		if c.Root == root && !c.Evicted {
			n++
		}
	}
	return n
}

// pastKeep notes each yass folder holding more archived changes than its keep: setting.
func (r *Repo) pastKeep() {
	for _, root := range r.Roots {
		if n, k := r.kept(root), root.keep(); n > k {
			r.note("%s holds %d archived changes, %d past its keep of %d; `yass evict` moves its oldest months to git history",
				r.disp(filepath.Join(root.Dir, "archive")), n, n-k, k)
		}
	}
}

func cmdEvict(a *args) error {
	r := loadRepo()
	if len(r.Roots) == 0 {
		return fmt.Errorf("%s", r.noRoot("no yass folder here; `yass init` sets one up"))
	}
	current := time.Now().UTC().Format("2006/01")
	over := false
	for _, root := range r.Roots {
		n, k := r.kept(root), root.keep()
		where := r.disp(filepath.Join(root.Dir, "archive"))
		if n <= k {
			continue
		}
		over = true
		top, ok := git(root.Dir, "rev-parse", "--show-toplevel")
		if !ok {
			return fmt.Errorf("%s isn't in git, so evicting would delete changes for good; it holds %d, %d past its keep of %d", where, n, n-k, k)
		}
		gitTop := canon(filepath.FromSlash(strings.TrimSpace(top)))
		// Decide every month first, and check each, so a refusal changes nothing.
		months, count := archiveMonths(root)
		var plan []evictedMonth
		left := n
		for _, m := range months {
			if left <= k || m == current {
				break
			}
			dir := filepath.Join(root.Dir, "archive", filepath.FromSlash(m))
			if out, _ := git(root.Dir, "status", "--porcelain", "--", dir); strings.TrimSpace(out) != "" {
				return fmt.Errorf("%s has changes that aren't committed; commit them first, then evict", r.disp(dir))
			}
			sha, _ := git(root.Dir, "log", "-1", "--format=%H", "--", dir)
			if strings.TrimSpace(sha) == "" {
				return fmt.Errorf("%s has never been committed, so evicting it would delete it for good", r.disp(dir))
			}
			rel, _ := filepath.Rel(gitTop, canon(dir))
			plan = append(plan, evictedMonth{Month: m, Commit: strings.TrimSpace(sha), Path: filepath.ToSlash(rel), Changes: subdirs(dir)})
			left -= count[m]
		}
		if len(plan) == 0 {
			hint := "only this month's"
			if flatArchived(root) {
				hint = "the rest sit directly in archive/ (`yass upgrade` moves them into months)"
			}
			fmt.Printf("%s holds %d archived changes, %d past its keep of %d, but has no month to evict: %s\n", where, n, n-k, k, hint)
			continue
		}
		for _, m := range plan {
			dir := filepath.Join(root.Dir, "archive", filepath.FromSlash(m.Month))
			if _, ok := git(root.Dir, "rm", "-r", "-q", "--", dir); !ok {
				return fmt.Errorf("couldn't remove %s with git rm", r.disp(dir))
			}
			os.RemoveAll(dir)
			f := dir + evictedExt
			if err := write(f, manifestText(m)); err != nil {
				return err
			}
			git(root.Dir, "add", "--", f)
			fmt.Printf("evicted %s: %d archived change(s), still in git at %s\n", r.disp(dir), len(m.Changes), m.Commit[:12])
		}
		fmt.Printf("%s now holds %d archived changes (keep: %d)\n", where, left, k)
		cmd := fmt.Sprintf("git commit -m \"yass: evict archived months\" -- %s", r.disp(filepath.Join(root.Dir, "archive")))
		if gitTop != r.Top {
			cmd = fmt.Sprintf("git -C %s commit -m \"yass: evict archived months\" -- %s", gitTop, filepath.Join(root.Dir, "archive"))
		}
		fmt.Printf("Commit it on its own: %s\n", cmd)
	}
	if !over {
		fmt.Println("nothing to evict: every yass folder is within its keep")
	}
	return nil
}


## internal/yass/history.go

package yass

import (
	"bufio"
	"fmt"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"strconv"
	"strings"
)

// Evicted months are read back from git only when a query reaches them, using the commit and path
// in each month's .evicted file (design §3): one git ls-tree per month, then one git cat-file
// --batch for every file, so the number of git processes doesn't grow with the number of changes.

// evictedStubs groups the name-only evicted changes by yass folder and month (<YYYY>/<MM>).
func (r *Repo) evictedStubs() map[*Root]map[string][]*Change {
	out := map[*Root]map[string][]*Change{}
	for _, c := range r.Archived {
		if !c.Evicted {
			continue
		}
		m := filepath.ToSlash(filepath.Dir(c.Path))
		m = m[len(m)-7:]
		if out[c.Root] == nil {
			out[c.Root] = map[string][]*Change{}
		}
		out[c.Root][m] = append(out[c.Root][m], c)
	}
	return out
}

// readEvicted fills in the evicted changes of the months named (by yass folder) from git: their
// change.md, their other .md files, and their pieces. It returns the months it couldn't read
// because their commit isn't in this clone.
func (r *Repo) readEvicted(want map[*Root][]string) []string {
	stubs := r.evictedStubs()
	var missing []string
	for root, months := range want {
		manifests := map[string]evictedMonth{}
		for _, m := range evictedMonths(root) {
			manifests[m.Month] = m
		}
		top, ok := git(root.Dir, "rev-parse", "--show-toplevel")
		if !ok {
			for _, m := range months {
				missing = append(missing, r.disp(filepath.Join(root.Dir, "archive", filepath.FromSlash(m))))
			}
			continue
		}
		top = strings.TrimSpace(top)
		var specs []string // <commit>:<path>
		for _, m := range months {
			man, ok := manifests[m]
			if !ok {
				continue
			}
			if _, ok := git(top, "cat-file", "-e", man.Commit+"^{commit}"); !ok {
				missing = append(missing, r.disp(filepath.Join(root.Dir, "archive", filepath.FromSlash(m))))
				continue
			}
			out, _ := git(top, "ls-tree", "-r", "--name-only", man.Commit, "--", man.Path)
			for _, f := range strings.Split(strings.TrimSpace(out), "\n") {
				if strings.HasSuffix(f, ".md") {
					specs = append(specs, man.Commit+":"+f)
				}
			}
		}
		blobs := catFiles(top, specs)
		for _, m := range months {
			man, ok := manifests[m]
			if !ok {
				continue
			}
			byName := map[string]*Change{}
			for _, c := range stubs[root][m] {
				byName[c.Name] = c
			}
			pieces := map[string]*Change{}
			for _, spec := range specs {
				f := strings.TrimPrefix(spec, man.Commit+":")
				if !strings.HasPrefix(spec, man.Commit+":") || !strings.HasPrefix(f, man.Path+"/") {
					continue
				}
				parts := strings.Split(strings.TrimPrefix(f, man.Path+"/"), "/")
				c := byName[parts[0]]
				if c == nil || len(parts) < 2 || len(parts) > 3 {
					continue
				}
				if len(parts) == 3 { // a piece's file
					key := parts[0] + "/" + parts[1]
					if pieces[key] == nil {
						p := nameOnly(filepath.Join(c.Path, parts[1]), root, c)
						p.Archived, p.Evicted, p.files = true, true, map[string]string{}
						pieces[key] = p
						c.Pieces = append(c.Pieces, p)
					}
					c = pieces[key]
				}
				if c.files == nil {
					c.files = map[string]string{}
				}
				name := parts[len(parts)-1]
				c.files[name] = normalize(blobs[spec])
				if name == "change.md" {
					c.setHead(c.files[name])
				}
				if name == "prd.md" || name == "plan.md" {
					c.Large = true
				}
			}
			for _, c := range stubs[root][m] {
				byCreated(c.Pieces)
			}
		}
	}
	return missing
}

// catFiles reads many files from git in one process: `git cat-file --batch`, by <commit>:<path>.
func catFiles(dir string, specs []string, env ...string) map[string]string {
	out := map[string]string{}
	if len(specs) == 0 {
		return out
	}
	cmd := exec.Command("git", "cat-file", "--batch")
	cmd.Dir = dir
	if len(env) > 0 {
		cmd.Env = append(os.Environ(), env...)
	}
	cmd.Stdin = strings.NewReader(strings.Join(specs, "\n") + "\n")
	pipe, err := cmd.StdoutPipe()
	if err != nil || cmd.Start() != nil {
		return out
	}
	rd := bufio.NewReader(pipe)
	for _, spec := range specs {
		header, err := rd.ReadString('\n')
		if err != nil {
			break
		}
		fields := strings.Fields(header)
		if len(fields) != 3 { // "<spec> missing"
			continue
		}
		size, _ := strconv.Atoi(fields[2])
		buf := make([]byte, size+1) // the content, then a newline
		if _, err := io.ReadFull(rd, buf); err != nil {
			break
		}
		out[spec] = string(buf[:size])
	}
	cmd.Wait()
	return out
}

// monthEnd is the last day a month (<YYYY>/<MM>) could hold, as a date that sorts with others.
func monthEnd(m string) string { return strings.Replace(m, "/", "-", 1) + "-31" }

// missingNote says which evicted months a query couldn't read, and how to get them.
func missingNote(missing []string) string {
	return fmt.Sprintf("couldn't read %s: evicted, and their commit isn't in this clone (`git fetch --unshallow` gets it)",
		strings.Join(sortedCopy(missing), ", "))
}


## internal/yass/history_test.go

package yass

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"
)

// Reading evicted months takes a fixed number of git processes, however many changes they hold.
func TestReadEvictedGitProcesses(t *testing.T) {
	if _, err := exec.LookPath("git"); err != nil {
		t.Skip("no git")
	}
	dir := t.TempDir()
	run := func(args ...string) string {
		cmd := exec.Command("git", args...)
		cmd.Dir = dir
		cmd.Env = append(os.Environ(), "GIT_AUTHOR_NAME=t", "GIT_AUTHOR_EMAIL=t@t", "GIT_COMMITTER_NAME=t", "GIT_COMMITTER_EMAIL=t@t")
		out, err := cmd.CombinedOutput()
		if err != nil {
			t.Fatalf("git %v: %v\n%s", args, err, out)
		}
		return strings.TrimSpace(string(out))
	}
	run("init", "-q", "-b", "main")
	month := filepath.Join(dir, "yass", "archive", "2025", "01")
	const n = 1000
	var names []string
	for i := 0; i < n; i++ {
		name := fmt.Sprintf("2025-01-01-c%04d", i)
		names = append(names, name)
		os.MkdirAll(filepath.Join(month, name), 0o755)
		os.WriteFile(filepath.Join(month, name, "change.md"), []byte(fmt.Sprintf("# C%d\n\n## Decisions\n- Call %d (t, 2025-01-02)\n", i, i)), 0o644)
	}
	run("add", "-A")
	run("commit", "-q", "-m", "archive")
	sha := run("rev-parse", "HEAD")
	run("rm", "-r", "-q", "yass/archive/2025/01")
	os.MkdirAll(filepath.Dir(month), 0o755) // git rm takes the emptied folders with it
	os.WriteFile(month+evictedExt, []byte(manifestText(evictedMonth{Commit: sha, Path: "yass/archive/2025/01", Changes: names})), 0o644)

	root := &Root{Dir: canon(filepath.Join(dir, "yass"))}
	r := &Repo{Top: canon(dir), Roots: []*Root{root}}
	r.Archived = r.load(root, "archive", false)
	trace := filepath.Join(t.TempDir(), "trace")
	t.Setenv("GIT_TRACE", trace)
	if missing := r.readEvicted(map[*Root][]string{root: {"2025/01"}}); len(missing) > 0 {
		t.Fatalf("missing %v", missing)
	}
	t.Setenv("GIT_TRACE", "")
	text, _ := os.ReadFile(trace)
	procs := strings.Count(string(text), "trace: built-in: git ")
	if procs > 4 {
		t.Errorf("read %d evicted changes with %d git processes, want at most 4 (rev-parse, cat-file -e, ls-tree, cat-file --batch)", n, procs)
	}
	read := 0
	for _, c := range r.Archived {
		if len(decisionsOf(c)) == 1 {
			read++
		}
	}
	if read != n {
		t.Errorf("read decisions from %d of %d evicted changes", read, n)
	}
}


## internal/yass/hook.go

package yass

import (
	"fmt"
	"os"
	"os/exec"
	"path"
	"path/filepath"
	"regexp"
	"sort"
	"strings"
)

// The optional pre-commit check: progress travels with code; intent changes get their own commit.
//
// It warns when one commit changes code and also edits existing intent in a yass folder: prd.md,
// design.md, plan.md beyond marking boxes, a yass folder's queue.md, or a change's title, Goal or
// Acceptance. Writing new intent files is fine. Marking boxes and appending to Log and Decisions
// alongside code is expected and never flagged. Editing the archive is always flagged: it's append-only.

var (
	hookBoxRE = regexp.MustCompile(`^(\s*[-*+]\s+)\[[ xX/]\]`) // starting, ticking or unticking is progress; dropping ([-]) is not
	h1RE      = regexp.MustCompile(`(?m)^# .*$`)
)

type entry struct{ status, path, oldPath string }

type place struct {
	kind   string // "changes", "archive", "queue", "meta", or "" for code
	fname  string
	folder string // for the archive: the archived change's folder
}

type hook struct {
	top      string   // the repo's top, where git's paths start
	prefixes []string // yass folders inside the repo that yass.yaml files point to, repo-relative
	ignored  []string // folders a yass.yaml ignores, repo-relative: their files are ordinary files
}

func newHook() *hook {
	h := &hook{}
	// Only the yass folders, seen from the top (git's paths are relative to it); no changes are read.
	r := &Repo{}
	r.Top = repoTop(getwd())
	r.Cwd = r.Top
	h.top = r.Top
	r.findRoots()
	for _, d := range r.Ignored {
		rel, _ := filepath.Rel(r.Top, d)
		h.ignored = append(h.ignored, filepath.ToSlash(rel))
	}
	for _, root := range r.Roots {
		if root.Config != "" && within(root.Dir, r.Top) && root.Dir != r.Top {
			rel, _ := filepath.Rel(r.Top, root.Dir)
			h.prefixes = append(h.prefixes, filepath.ToSlash(rel))
		}
	}
	return h
}

func (h *hook) where(p string) place {
	for _, d := range h.ignored {
		if strings.HasPrefix(p, d+"/") {
			return place{}
		}
	}
	parts := strings.Split(p, "/")
	if path.Base(p) == ConfigName {
		return place{kind: "meta"}
	}
	for _, pre := range h.prefixes {
		if !strings.HasPrefix(p, pre+"/") {
			continue
		}
		rest := strings.Split(strings.TrimPrefix(p, pre+"/"), "/")
		if len(rest) == 1 && rest[0] == QueueName {
			return place{kind: "queue", fname: QueueName}
		}
		if len(rest) >= 2 && (rest[0] == "changes" || rest[0] == "archive") {
			return place{rest[0], parts[len(parts)-1], pre + "/" + changeFolder(rest)}
		}
		return place{kind: "meta"}
	}
	for i := 0; i < len(parts)-1; i++ {
		if parts[i] == "yass" && (parts[i+1] == "changes" || parts[i+1] == "archive") {
			return place{parts[i+1], parts[len(parts)-1], strings.Join(parts[:i+1], "/") + "/" + changeFolder(parts[i+1:])}
		}
	}
	if len(parts) >= 2 && parts[len(parts)-2] == "yass" && parts[len(parts)-1] == QueueName {
		return place{kind: "queue", fname: QueueName}
	}
	if len(parts) >= 2 && parts[len(parts)-2] == "yass" && parts[len(parts)-1] == "README.md" {
		return place{kind: "meta"}
	}
	return place{}
}

var archivedLineRE = regexp.MustCompile(`(?m)^archived:.*\n`)

// intoMonths is the migration of a flat archived change into archive/<YYYY>/<MM>/: the same
// change, moved from archive/<name>/ to a month folder, with at most an archived: stamp added.
// Git sees the move as a rename, or, when the stamp changes a small file too much, as a deletion
// here and an addition there; both count.
func (h *hook) intoMonths(e entry, pl place, entries []entry, oldRef, newRef string) bool {
	oldPath, newPath := e.oldPath, e.path
	switch {
	case strings.HasPrefix(e.status, "R"):
	case e.status == "D":
		oldPath, newPath = e.path, ""
		for _, a := range entries {
			if a.status == "A" && h.monthOf(h.where(e.path), h.where(a.path)) && path.Base(a.path) == path.Base(e.path) &&
				strings.TrimPrefix(a.path, h.where(a.path).folder) == strings.TrimPrefix(e.path, pl.folder) {
				newPath = a.path
			}
		}
		if newPath == "" {
			return false
		}
	case e.status == "A":
		return false // the addition side is new to the archive, which is fine on its own
	default:
		return false
	}
	if !h.monthOf(h.where(oldPath), h.where(newPath)) {
		return false
	}
	if e.status == "R100" {
		return true
	}
	if oldRef == "" {
		return false
	}
	old, cur := show(oldRef, oldPath), archivedLineRE.ReplaceAllString(show(newRef, newPath), "")
	// The stamp may have come with frontmatter of its own, when the change had none.
	return old == cur || old == strings.TrimPrefix(cur, "---\n---\n")
}

// evicting is yass evict's commit: an archived change deleted while the same commit adds its month's
// .evicted file naming it (evicting), or that new .evicted file itself (manifest).
func (h *hook) evicting(e entry, pl place, entries []entry, newRef string) (evicting, manifest bool) {
	if strings.HasSuffix(e.path, evictedExt) {
		return false, e.status == "A" && monthRE.MatchString(strings.TrimSuffix(path.Base(e.path), evictedExt))
	}
	if e.status != "D" {
		return false, false
	}
	month := path.Dir(pl.folder) // <archive>/<YYYY>/<MM>
	if !monthRE.MatchString(path.Base(month)) || !yearRE.MatchString(path.Base(path.Dir(month))) {
		return false, false
	}
	for _, a := range entries {
		if a.status == "A" && a.path == month+evictedExt {
			for _, l := range lines(show(newRef, a.path)) {
				if strings.TrimSpace(strings.TrimPrefix(strings.TrimSpace(l), "- ")) == path.Base(pl.folder) {
					return true, false
				}
			}
		}
	}
	return false, false
}

// monthOf says whether to is from's flat archived change, under archive/<YYYY>/<MM>/.
func (h *hook) monthOf(from, to place) bool {
	month := path.Dir(to.folder) // <archive>/<YYYY>/<MM>
	return from.kind == "archive" && to.kind == "archive" && path.Base(to.folder) == path.Base(from.folder) &&
		path.Dir(from.folder) == path.Dir(path.Dir(month)) && monthRE.MatchString(path.Base(month)) &&
		yearRE.MatchString(path.Base(path.Dir(month)))
}

// changeFolder is the change's folder in a path that starts at changes/ or archive/: the folder
// right under it, or, in the archive, the one under <YYYY>/<MM>/.
func changeFolder(rest []string) string {
	n := 2
	if rest[0] == "archive" && len(rest) >= 4 && yearRE.MatchString(rest[1]) && monthRE.MatchString(rest[2]) {
		n = 4
	}
	if n > len(rest) {
		n = len(rest)
	}
	return strings.Join(rest[:n], "/")
}

func norm(text string) []string {
	var out []string
	for _, l := range lines(commentRE.ReplaceAllString(text, "")) {
		l = strings.TrimRight(hookBoxRE.ReplaceAllString(l, "${1}[ ]"), " \t")
		if strings.TrimSpace(l) != "" {
			out = append(out, l)
		}
	}
	return out
}

func intentOfChangeMD(text string) string {
	parts := []string{}
	if m := h1RE.FindString(text); m != "" {
		parts = append(parts, m)
	}
	parts = append(parts, norm(section(text, "Goal"))...)
	parts = append(parts, "--")
	parts = append(parts, norm(section(text, "Acceptance"))...)
	return strings.Join(parts, "\n")
}

// show is a file at a commit; ref "" is the index (what's staged).
func show(ref, p string) string {
	out, _ := git("", "show", ref+":"+p)
	return normalize(out)
}

func (h *hook) check(entries []entry, oldRef, newRef string) []string {
	places := map[string]place{}
	hasCode := false
	for _, e := range entries {
		places[e.path] = h.where(e.path)
		if places[e.path].kind == "" {
			hasCode = true
		}
	}
	var out []string
	for _, e := range entries {
		pl := places[e.path]
		if pl.kind == "" || pl.kind == "meta" || pl.fname == ".gitkeep" {
			continue
		}
		fromChanges := strings.HasPrefix(e.status, "R") && h.where(e.oldPath).kind == "changes"
		if pl.kind == "queue" {
			if hasCode && e.status[0] != 'A' {
				out = append(out, e.path+": queue.md is intent; reorder in a commit without code")
			}
			continue
		}
		if pl.kind == "archive" {
			if h.intoMonths(e, pl, entries, oldRef, newRef) {
				if hasCode {
					out = append(out, e.path+": move the archive into months in its own commit, not alongside code")
				}
				continue
			}
			if evicting, manifest := h.evicting(e, pl, entries, newRef); evicting || manifest {
				if hasCode && manifest {
					out = append(out, e.path+": evict archived months in their own commit, not alongside code")
				}
				continue
			}
			addedToOld := false
			if e.status[0] == 'A' && oldRef != "" {
				_, addedToOld = git("", "cat-file", "-e", oldRef+":"+pl.folder)
			}
			if strings.HasPrefix(e.status, "M") || strings.HasPrefix(e.status, "D") || addedToOld ||
				(strings.HasPrefix(e.status, "R") && !fromChanges) {
				out = append(out, e.path+": the archive is append-only; follow-up work is a new change with `follows:`")
			} else if fromChanges && hasCode {
				out = append(out, e.path+": archive a change in its own commit, not alongside code")
			}
			continue
		}
		if !hasCode || e.status[0] == 'A' {
			continue // new intent isn't a moved goalpost
		}
		oldP := e.oldPath
		if oldP == "" {
			oldP = e.path
		}
		old, cur := "", ""
		if oldRef != "" {
			old = show(oldRef, oldP)
		}
		if e.status[0] != 'D' {
			cur = show(newRef, e.path)
		}
		switch pl.fname {
		case "prd.md", "design.md":
			out = append(out, fmt.Sprintf("%s: %s is intent; change it in a commit without code", e.path, pl.fname))
		case "plan.md":
			if e.status[0] == 'D' || strings.Join(norm(old), "\n") != strings.Join(norm(cur), "\n") {
				out = append(out, e.path+": with code, only mark boxes in plan.md; revise the plan in its own commit")
			}
		case "change.md":
			if e.status[0] == 'D' {
				out = append(out, e.path+": removing a change alongside code")
			} else if intentOfChangeMD(old) != intentOfChangeMD(cur) {
				out = append(out, e.path+": the title, Goal or Acceptance changed; that's intent, so give it its own commit")
			}
		}
	}
	if hasCode {
		out = append(out, h.outsideThePiece(entries, places)...)
	}
	return out
}

// outsideThePiece flags a commit with code that reaches past one piece of a large change: it edits
// the parent's own files, or files in two of its pieces. A piece's branch changes only its own
// folder, so parallel branches on different pieces never touch the same file. New files are fine.
func (h *hook) outsideThePiece(entries []entry, places map[string]place) []string {
	var out []string
	pieces := map[string]map[string]bool{} // parent folder → pieces touched
	for _, e := range entries {
		pl := places[e.path]
		if pl.kind != "changes" || e.status[0] == 'A' || !h.hasPieces(pl.folder) {
			continue
		}
		rel := strings.TrimPrefix(e.path, pl.folder+"/")
		piece, _, inside := strings.Cut(rel, "/")
		if !inside || !isFile(filepath.Join(h.top, filepath.FromSlash(pl.folder), piece, "change.md")) {
			// the parent's own files, its assets/ included: anything not in a piece
			out = append(out, e.path+": a large change's own files change in commits without code; a piece's branch changes only its own folder")
			continue
		}
		if pieces[pl.folder] == nil {
			pieces[pl.folder] = map[string]bool{}
		}
		pieces[pl.folder][piece] = true
	}
	for folder, ps := range pieces {
		if len(ps) > 1 {
			var names []string
			for p := range ps {
				names = append(names, p)
			}
			sort.Strings(names)
			out = append(out, fmt.Sprintf("%s: this commit with code changes pieces %s; each piece's branch changes only its own folder",
				folder, strings.Join(names, " and ")))
		}
	}
	return out
}

// hasPieces says whether a change folder holds pieces: subfolders with a change.md.
func (h *hook) hasPieces(folder string) bool {
	m, _ := filepath.Glob(filepath.Join(h.top, filepath.FromSlash(folder), "*", "change.md"))
	return len(m) > 0
}

// nameStatus lists what a diff changed. With -z, paths come raw (no quoting of non-ASCII names) and
// NUL-separated: "M\0path\0", or "R100\0old\0new\0" for renames and copies.
func nameStatus(args ...string) []entry {
	out, _ := git("", append([]string{"diff", "--name-status", "-M", "-z"}, args...)...)
	f := strings.Split(strings.TrimSuffix(out, "\x00"), "\x00")
	var rows []entry
	for i := 0; i+1 < len(f); {
		status := f[i]
		if (status[0] == 'R' || status[0] == 'C') && i+2 < len(f) {
			rows = append(rows, entry{status, f[i+2], f[i+1]})
			i += 3
		} else {
			rows = append(rows, entry{status, f[i+1], ""})
			i += 2
		}
	}
	return rows
}

func emptyTree() string {
	cmd := exec.Command("git", "hash-object", "-t", "tree", "--stdin")
	cmd.Stdin = strings.NewReader("")
	out, _ := cmd.Output()
	return strings.TrimSpace(string(out))
}

func cmdHook(a *args) int {
	switch strings.ToLower(os.Getenv("YASS_HOOK")) {
	case "off", "0", "false":
		return 0
	}
	strict := a.b["strict"] || (os.Getenv("YASS_STRICT") != "" && os.Getenv("YASS_STRICT") != "0")
	switch {
	case a.b["prepare-msg"]:
		if len(a.pos) > 0 {
			source := ""
			if len(a.pos) > 1 {
				source = a.pos[1]
			}
			prepareMsg(a.pos[0], source)
		}
		return 0
	case a.b["commit-msg"]:
		if len(a.pos) == 0 {
			return 0
		}
		return commitMsg(a.pos[0])
	}
	h := newHook()
	var found, trailers []string
	trailerFail, namesChecked := false, true
	if rng, ok := a.v["range"]; ok {
		shas, ok := git("", "rev-list", "--reverse", "--no-merges", rng)
		if rng == "" || !ok {
			fmt.Fprintf(os.Stderr, "yass: can't read the range '%s'. In CI, fetch history first (e.g. actions/checkout with fetch-depth: 0).\n", rng)
			return 2
		}
		for _, sha := range strings.Fields(shas) {
			parent, _ := git("", "rev-parse", "-q", "--verify", sha+"^")
			parent = strings.TrimSpace(parent)
			base := parent
			if base == "" {
				base = emptyTree()
			}
			if w := h.check(nameStatus(base, sha), parent, sha); len(w) > 0 {
				subj, _ := git("", "log", "-1", "--format=%h %s", sha)
				for _, x := range w {
					found = append(found, strings.TrimSpace(subj)+": "+x)
				}
			}
		}
		if r := loadRepo(); r.usesSeparatePlans() {
			trailers, namesChecked = r.rangeTrailers(strings.Fields(shas), a.b["require-trailers"])
			trailerFail = len(trailers) > 0 && strict
		}
	} else {
		if _, merging := git("", "rev-parse", "-q", "--verify", "MERGE_HEAD"); merging {
			return 0
		}
		head := ""
		if _, ok := git("", "rev-parse", "-q", "--verify", "HEAD"); ok {
			head = "HEAD"
		}
		base := head
		if base == "" {
			base = emptyTree()
		}
		found = h.check(nameStatus("--cached", base), head, "")
	}
	if len(trailers) > 0 {
		verb := "heads-up"
		if trailerFail {
			verb = "failing"
		}
		fmt.Fprintf(os.Stderr, "yass: %s: commits whose `yass:` trailers need a look (the plans are kept apart from the code)\n", verb)
		for _, t := range trailers {
			fmt.Fprintf(os.Stderr, "  - %s\n", t)
		}
	}
	if !namesChecked {
		fmt.Fprintln(os.Stderr, "yass: the plans folder isn't here, so trailers were checked for form only; change names weren't checked")
	}
	if len(found) == 0 {
		if trailerFail {
			return 1
		}
		return 0
	}
	verb := "heads-up"
	if strict {
		verb = "refusing"
	}
	fmt.Fprintf(os.Stderr, "yass: %s: code and intent are mixed (progress travels with code; intent gets its own commit)\n", verb)
	for _, f := range found {
		fmt.Fprintf(os.Stderr, "  - %s\n", f)
	}
	if !strict {
		fmt.Fprintln(os.Stderr, "  (a warning only; YASS_STRICT=1 makes it an error, YASS_HOOK=off skips it)")
		if trailerFail {
			return 1
		}
		return 0
	}
	return 1
}


## internal/yass/hooksetup.go

package yass

import (
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

// turnOnHook points core.hooksPath at YASS's hook folder, but only when the repo has no hooks setup
// of its own: setting core.hooksPath switches off every hook git would otherwise run (a local or
// global core.hooksPath, as husky uses, and hooks in .git/hooks). When there is one, it leaves it
// alone and prints how to run `yass hook` from it.
func turnOnHook(top string) error {
	effective, _ := git(top, "config", "--get", "core.hooksPath")
	local, _ := git(top, "config", "--local", "--get", "core.hooksPath")
	effective, local = strings.TrimSpace(effective), strings.TrimSpace(local)
	if effective == HookPath {
		fmt.Printf("hook  already on: core.hooksPath = %s\n", HookPath)
		return nil
	}
	switch {
	case local != "":
		fmt.Printf("hook  not turned on: this repo already has a hooks setup (core.hooksPath = %s), and pointing it at\n"+
			"      %s would switch those hooks off.%s\n", local, HookPath, wireIn(top, local))
		return nil
	case effective != "":
		fmt.Printf("hook  not turned on: your global core.hooksPath (%s) runs hooks for every repo, and setting one here\n"+
			"      would override it for this repo.%s\n"+
			"      Or, to use YASS's hook folder in this repo instead:  git config core.hooksPath %s\n", effective, wireIn(top, effective), HookPath)
		return nil
	}
	if found := activeHooks(top); len(found) > 0 {
		fmt.Printf("hook  not turned on: this clone already runs hooks from .git/hooks (%s), and pointing core.hooksPath\n"+
			"      at %s would switch them off.%s\n", strings.Join(found, ", "), HookPath, wireIn(top, ""))
		return nil
	}
	if _, ok := git(top, "config", "core.hooksPath", HookPath); !ok {
		return fmt.Errorf("couldn't set git config core.hooksPath (is this a git repo?)")
	}
	fmt.Printf("hook  core.hooksPath = %s (undo: git config --unset core.hooksPath)\n", HookPath)
	return nil
}

// activeHooks lists the hooks git would run from the clone's own hooks folder (.git/hooks, or the
// common one for a worktree): executable files that aren't git's *.sample examples.
func activeHooks(top string) []string {
	dir, ok := git(top, "rev-parse", "--git-path", "hooks")
	if !ok {
		return nil
	}
	dir = strings.TrimSpace(dir)
	if !filepath.IsAbs(dir) {
		dir = filepath.Join(top, dir)
	}
	entries, _ := os.ReadDir(dir)
	var found []string
	for _, e := range entries {
		info, err := e.Info()
		if err != nil || !info.Mode().IsRegular() || info.Mode()&0o111 == 0 || strings.HasSuffix(e.Name(), ".sample") {
			continue
		}
		found = append(found, e.Name())
	}
	sort.Strings(found)
	return found
}

// wireIn says how to run YASS's check from an existing hooks setup: a hook manager the repo uses,
// or else the pre-commit script in the hooks folder git runs (dir; "" for .git/hooks).
func wireIn(top, dir string) string {
	const indent = "\n      "
	switch {
	case strings.Contains(dir, ".husky") || isDir(filepath.Join(top, ".husky")):
		return indent + "To run YASS's check from husky, add this line to .husky/pre-commit:" + indent + "  yass hook"
	case exists(filepath.Join(top, "lefthook.yml")) || exists(filepath.Join(top, ".lefthook.yml")):
		return indent + "To run YASS's check from lefthook, add this to lefthook.yml:" + indent +
			"  pre-commit:" + indent + "    commands:" + indent + "      yass:" + indent + "        run: yass hook"
	case exists(filepath.Join(top, ".pre-commit-config.yaml")):
		return indent + "To run YASS's check from pre-commit, add this to .pre-commit-config.yaml under repos:" + indent +
			"  - repo: local" + indent + "    hooks:" + indent + "      - id: yass" + indent + "        name: yass hook" + indent +
			"        entry: yass hook" + indent + "        language: system" + indent + "        pass_filenames: false" + indent + "        always_run: true"
	}
	where := ".git/hooks/pre-commit"
	if dir != "" {
		where = filepath.ToSlash(filepath.Join(dir, "pre-commit"))
	}
	return indent + "To run YASS's check too, add this line to " + where + " (create it, executable, if it doesn't exist):" + indent +
		"  " + HookPath + "/pre-commit \"$@\""
}


## internal/yass/install.go

package yass

import (
	"archive/tar"
	"archive/zip"
	"compress/gzip"
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"io"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
	"time"
)

// Installing a release: fetch the platform's archive and checksums.txt into a temporary folder,
// verify them (design §3), check the new binary runs and is the version asked for, then swap it in
// with a rename in the binary's own folder (design §4). Nothing outside the temporary folder
// changes until every check has passed.

var downloadClient = &http.Client{Timeout: 5 * time.Minute} // follows redirects, as GitHub's downloads need

// exePath is the running binary, symlinks resolved, so the swap happens where the file really is.
func exePath() (string, error) {
	p, err := os.Executable()
	if err != nil {
		return "", err
	}
	return filepath.EvalSymlinks(p)
}

// writable says whether yass can create a file in dir, by creating one.
func writable(dir string) bool {
	f, err := os.CreateTemp(dir, ".yass-update-*")
	if err != nil {
		return false
	}
	f.Close()
	os.Remove(f.Name())
	return true
}

func fetch(url, to string) error {
	resp, err := downloadClient.Get(url)
	if err != nil {
		return fmt.Errorf("couldn't reach %s: %v", url, err)
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		return fmt.Errorf("%s: HTTP %d", url, resp.StatusCode)
	}
	f, err := os.Create(to)
	if err != nil {
		return err
	}
	_, err = io.Copy(f, resp.Body)
	if cerr := f.Close(); err == nil {
		err = cerr
	}
	return err
}

// releaseExists checks a release by its checksums.txt, before anything is downloaded.
func releaseExists(version string) error {
	url := fmt.Sprintf("%s/download/%s/checksums.txt", releasesURL(), version)
	resp, err := httpClient.Head(url)
	if err != nil {
		return fmt.Errorf("couldn't reach %s: %v", releasesURL(), err)
	}
	resp.Body.Close()
	if resp.StatusCode == http.StatusNotFound {
		return fmt.Errorf("there's no release %s (see %s)", version, releasesURL())
	}
	if resp.StatusCode/100 != 2 && resp.StatusCode/100 != 3 {
		return fmt.Errorf("couldn't check release %s: HTTP %d", version, resp.StatusCode)
	}
	return nil
}

// checksumOK checks a file against its line in checksums.txt ("<sha256>  <name>").
func checksumOK(file, sums, name string) error {
	want := ""
	for _, l := range strings.Split(sums, "\n") {
		if f := strings.Fields(l); len(f) == 2 && f[1] == name {
			want = strings.ToLower(f[0])
		}
	}
	if want == "" {
		return fmt.Errorf("%s isn't listed in the release's checksums.txt", name)
	}
	data, err := os.ReadFile(file)
	if err != nil {
		return err
	}
	sum := sha256.Sum256(data)
	if got := hex.EncodeToString(sum[:]); got != want {
		return fmt.Errorf("%s doesn't match the release's checksums.txt (expected %s, got %s)", name, want, got)
	}
	return nil
}

// provenance verifies the archive's signed build provenance with the GitHub CLI. checked says
// whether it ran; a failed verification is an error, and why it didn't run is in note.
func provenance(archive string) (checked bool, note string, err error) {
	gh, lerr := exec.LookPath("gh")
	if lerr != nil {
		return false, "the GitHub CLI isn't installed", nil
	}
	if exec.Command(gh, "auth", "status").Run() != nil {
		return false, "the GitHub CLI isn't signed in (gh auth login)", nil
	}
	out, verr := exec.Command(gh, "attestation", "verify", archive, "--repo", "donjaime/yass").CombinedOutput()
	if verr != nil {
		return true, "", fmt.Errorf("its build provenance didn't verify (gh attestation verify):\n%s", strings.TrimSpace(string(out)))
	}
	return true, "", nil
}

// unpack finds the yass binary in a release archive and returns its contents.
func unpack(archive string) ([]byte, error) {
	name := "yass"
	if runtime.GOOS == "windows" {
		name = "yass.exe"
	}
	if strings.HasSuffix(archive, ".zip") {
		zr, err := zip.OpenReader(archive)
		if err != nil {
			return nil, err
		}
		defer zr.Close()
		for _, f := range zr.File {
			if filepath.Base(f.Name) == name && !f.FileInfo().IsDir() {
				rc, err := f.Open()
				if err != nil {
					return nil, err
				}
				defer rc.Close()
				return io.ReadAll(rc)
			}
		}
		return nil, fmt.Errorf("%s has no %s", filepath.Base(archive), name)
	}
	f, err := os.Open(archive)
	if err != nil {
		return nil, err
	}
	defer f.Close()
	gz, err := gzip.NewReader(f)
	if err != nil {
		return nil, err
	}
	tr := tar.NewReader(gz)
	for {
		h, err := tr.Next()
		if err == io.EOF {
			return nil, fmt.Errorf("%s has no %s", filepath.Base(archive), name)
		}
		if err != nil {
			return nil, err
		}
		if filepath.Base(h.Name) == name && h.Typeflag == tar.TypeReg {
			return io.ReadAll(tr)
		}
	}
}

// swapBinary replaces exe with next, a file in the same folder. On Unix a rename is atomic, so the
// binary is never missing or half written. Windows can't overwrite a running .exe, so the old one
// is renamed aside to <exe>.old first, and removed on a later run (cleanOldBinary).
func swapBinary(exe, next, goos string) error {
	if goos == "windows" {
		old := exe + ".old"
		os.Remove(old)
		if err := os.Rename(exe, old); err != nil {
			return err
		}
		if err := os.Rename(next, exe); err != nil {
			os.Rename(old, exe) // put it back
			return err
		}
		return nil
	}
	return os.Rename(next, exe)
}

// cleanOldBinary removes what a Windows update set aside, once it's no longer running.
func cleanOldBinary() {
	if runtime.GOOS != "windows" {
		return
	}
	if exe, err := exePath(); err == nil {
		os.Remove(exe + ".old")
	}
}

// install fetches, verifies and swaps in version. It returns how it verified the download.
func install(version string, requireProvenance bool) (string, error) {
	exe, err := exePath()
	if err != nil {
		return "", fmt.Errorf("couldn't find this yass binary: %v", err)
	}
	dir := filepath.Dir(exe)
	if !writable(dir) {
		return "", fmt.Errorf("can't write to %s, where this yass is; rerun with permission to write there, or install yass into a folder of your own (see docs/install.md)", dir)
	}
	if err := releaseExists(version); err != nil {
		return "", err
	}
	tmp, err := os.MkdirTemp("", "yass-update-")
	if err != nil {
		return "", err
	}
	defer os.RemoveAll(tmp)
	asset := assetName(runtime.GOOS, runtime.GOARCH)
	base := fmt.Sprintf("%s/download/%s", releasesURL(), version)
	archive, sums := filepath.Join(tmp, asset), filepath.Join(tmp, "checksums.txt")
	if err := fetch(base+"/"+asset, archive); err != nil {
		return "", err
	}
	if err := fetch(base+"/checksums.txt", sums); err != nil {
		return "", err
	}
	if err := checksumOK(archive, read(sums), asset); err != nil {
		return "", fmt.Errorf("not installing %s: %v", plain(version), err)
	}
	checked, why, err := provenance(archive)
	if err != nil {
		return "", fmt.Errorf("not installing %s: %v", plain(version), err)
	}
	if !checked && requireProvenance {
		return "", fmt.Errorf("not installing %s: --require-provenance, and provenance can't be checked: %s", plain(version), why)
	}
	bin, err := unpack(archive)
	if err != nil {
		return "", fmt.Errorf("not installing %s: %v", plain(version), err)
	}
	next, err := os.CreateTemp(dir, ".yass-new-*")
	if err != nil {
		return "", err
	}
	nextPath := next.Name()
	defer os.Remove(nextPath) // gone after a successful swap; cleaned up otherwise
	_, err = next.Write(bin)
	if cerr := next.Close(); err == nil {
		err = cerr
	}
	if err == nil {
		err = os.Chmod(nextPath, 0o755)
	}
	if err != nil {
		return "", err
	}
	// The new binary has to run here and be the version asked for: a wrong platform or a broken
	// download stops before anything is replaced.
	out, rerr := exec.Command(nextPath, "version").Output()
	if got := strings.TrimSpace(string(out)); rerr != nil || got != "yass "+plain(version) {
		if rerr != nil {
			got = rerr.Error()
		}
		return "", fmt.Errorf("not installing %s: the downloaded yass doesn't run as %s here (it said: %s)", plain(version), plain(version), firstLine(got))
	}
	if err := swapBinary(exe, nextPath, runtime.GOOS); err != nil {
		return "", fmt.Errorf("couldn't replace %s: %v", exe, err)
	}
	how := "checksum verified"
	if checked {
		how += ", build provenance verified"
	} else {
		how += "; build provenance not checked: " + why
	}
	return how, nil
}

func firstLine(s string) string {
	line, _, _ := strings.Cut(s, "\n")
	return line
}


## internal/yass/install_test.go

package yass

import (
	"crypto/sha256"
	"encoding/hex"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestSwapBinary(t *testing.T) {
	for _, goos := range []string{"darwin", "windows"} {
		dir := t.TempDir()
		exe, next := filepath.Join(dir, "yass"), filepath.Join(dir, ".yass-new-1")
		os.WriteFile(exe, []byte("old"), 0o755)
		os.WriteFile(next, []byte("new"), 0o755)
		if err := swapBinary(exe, next, goos); err != nil {
			t.Fatalf("%s: %v", goos, err)
		}
		if got, _ := os.ReadFile(exe); string(got) != "new" {
			t.Errorf("%s: binary is %q, want the new one", goos, got)
		}
		if isFile(next) {
			t.Errorf("%s: the new file is still beside it", goos)
		}
		old, _ := os.ReadFile(exe + ".old")
		if goos == "windows" && string(old) != "old" {
			t.Errorf("windows: the old binary should be set aside as yass.old, got %q", old)
		}
		if goos != "windows" && isFile(exe+".old") {
			t.Errorf("%s: nothing should be set aside", goos)
		}
	}
	// A failed swap on Windows puts the old binary back.
	dir := t.TempDir()
	exe := filepath.Join(dir, "yass")
	os.WriteFile(exe, []byte("old"), 0o755)
	if err := swapBinary(exe, filepath.Join(dir, "missing"), "windows"); err == nil {
		t.Fatal("swapping in a missing file should fail")
	}
	if got, _ := os.ReadFile(exe); string(got) != "old" {
		t.Errorf("after a failed swap the binary is %q, want the old one back", got)
	}
}

func TestChecksumOK(t *testing.T) {
	f := filepath.Join(t.TempDir(), "yass_linux_amd64.tar.gz")
	os.WriteFile(f, []byte("archive"), 0o644)
	sum := sha256.Sum256([]byte("archive"))
	good := hex.EncodeToString(sum[:])
	if err := checksumOK(f, good+"  yass_linux_amd64.tar.gz\nabc  other.zip\n", "yass_linux_amd64.tar.gz"); err != nil {
		t.Errorf("matching checksum: %v", err)
	}
	if err := checksumOK(f, strings.Repeat("0", 64)+"  yass_linux_amd64.tar.gz\n", "yass_linux_amd64.tar.gz"); err == nil || !strings.Contains(err.Error(), "doesn't match") {
		t.Errorf("mismatch: %v", err)
	}
	if err := checksumOK(f, good+"  other.tar.gz\n", "yass_linux_amd64.tar.gz"); err == nil || !strings.Contains(err.Error(), "isn't listed") {
		t.Errorf("unlisted: %v", err)
	}
}


## internal/yass/kit.go

package yass

import (
	"fmt"
	"io/fs"
	"os"
	"path"
	"path/filepath"
	"regexp"
	"sort"
	"strings"

	"golang.org/x/mod/semver"

	"github.com/donjaime/yass/kit"
)

// binVersion is this binary's version, as Main receives it; the stamps written into a repo use it.
var binVersion = "dev"

// binChannel is "release" for binaries built by the release workflow, which yass update may replace.
var binChannel = ""

// HookPath is where the hook script lives in a repo, and what --hooks points core.hooksPath at.
const HookPath = "tools/yass/githooks"

var metadataRE = regexp.MustCompile(`(?m)^metadata:[ \t]*$`)

// stampVersion is the version as it's written into files: without a leading "v".
func stampVersion() string { return strings.TrimPrefix(binVersion, "v") }

// stampSkill adds metadata.yass-version to a playbook's frontmatter (the Agent Skills spec's
// extension point), replacing any yass-version already there.
func stampSkill(text, version string) string {
	m := fmRE.FindStringSubmatchIndex(text)
	if m == nil {
		return text
	}
	front := text[m[2]:m[3]]
	line := fmt.Sprintf("  yass-version: %q", version)
	var out []string
	for _, l := range lines(front) {
		if !strings.HasPrefix(strings.TrimSpace(l), "yass-version:") {
			out = append(out, l)
		}
	}
	front = strings.Join(out, "\n")
	if loc := metadataRE.FindStringIndex(front); loc != nil {
		front = front[:loc[1]] + "\n" + line + front[loc[1]:]
	} else {
		front += "\nmetadata:\n" + line
	}
	return text[:m[2]] + front + text[m[3]:]
}

// stampHook adds a "# yass-version:" line after the hook script's first line.
func stampHook(text, version string) string {
	first, rest, _ := strings.Cut(text, "\n")
	return first + "\n# yass-version: " + version + "\n" + rest
}

// stampAgents adds version=V to the AGENTS.md section's begin marker.
func stampAgents(block, version string) string {
	return strings.Replace(block, "<!-- yass:begin ", "<!-- yass:begin version="+version+" ", 1)
}

// kitSkills lists the playbooks the binary carries: name and unstamped text, sorted by name.
func kitSkills() (names []string, text map[string]string) {
	text = map[string]string{}
	matches, _ := fs.Glob(kit.FS, ".agents/skills/*/SKILL.md")
	for _, m := range matches {
		b, _ := kit.FS.ReadFile(m)
		name := path.Base(path.Dir(m))
		names, text[name] = append(names, name), string(b)
	}
	sort.Strings(names)
	return names, text
}

// kitFile reads one file the binary carries.
func kitFile(p string) (string, error) {
	b, err := kit.FS.ReadFile(p)
	return string(b), err
}

// userSkillsDir is where --global puts the playbooks: $YASS_SKILLS_DIR, or ~/.agents/skills.
func userSkillsDir() string {
	if d := os.Getenv("YASS_SKILLS_DIR"); d != "" {
		return d
	}
	home, _ := os.UserHomeDir()
	return filepath.Join(home, ".agents", "skills")
}

func userClaudeSkillsDir() string {
	home, _ := os.UserHomeDir()
	return filepath.Join(home, ".claude", "skills")
}

// kitWriter writes stamped kit files, keeping any that already exist; it remembers which it
// wrote and which it kept, so init can list the first and check the second's versions.
type kitWriter struct{ made, kept []string }

func (w *kitWriter) place(p, text string, mode os.FileMode) error {
	if exists(p) {
		w.kept = append(w.kept, p)
		return nil
	}
	if err := write(p, text); err != nil {
		return err
	}
	if mode != 0 {
		if err := os.Chmod(p, mode); err != nil {
			return err
		}
	}
	w.made = append(w.made, p)
	return nil
}

// agentFiles writes the playbooks into dir's .agents/skills (or the user folder, with global) and,
// with claude, Claude Code's copies and the CLAUDE.md import in dir. dir is the repo root, or a
// monorepo folder that keeps its own agent files (yass init <folder> --agents).
func (w *kitWriter) agentFiles(dir string, claude, global bool) error {
	v := stampVersion()
	skills, claudeSkills := filepath.Join(dir, ".agents", "skills"), filepath.Join(dir, ".claude", "skills")
	if global {
		skills, claudeSkills = userSkillsDir(), userClaudeSkillsDir()
	}
	names, text := kitSkills()
	for _, n := range names {
		stamped := stampSkill(text[n], v)
		if err := w.place(filepath.Join(skills, n, "SKILL.md"), stamped, 0); err != nil {
			return err
		}
		if claude {
			if err := w.place(filepath.Join(claudeSkills, n, "SKILL.md"), stamped, 0); err != nil {
				return err
			}
		}
	}
	if claude {
		cm := filepath.Join(dir, "CLAUDE.md")
		if cur := read(cm); !strings.Contains(cur, "@AGENTS.md") {
			next := "@AGENTS.md\n"
			if cur != "" {
				next += "\n" + cur
			}
			if err := write(cm, next); err != nil {
				return err
			}
			w.made = append(w.made, cm)
		}
	}
	return nil
}

// hookScripts are the git hooks YASS ships: the intent check, and the trailer hooks that only act
// when plans are kept apart from the code.
var hookScripts = []string{"pre-commit", "prepare-commit-msg", "commit-msg"}

// hook writes the hook scripts, which belong to the repo root: git hooks are per repo.
func (w *kitWriter) hook(top string) error {
	for _, name := range hookScripts {
		b, _ := kit.FS.ReadFile(HookPath + "/" + name)
		if err := w.place(filepath.Join(top, filepath.FromSlash(HookPath), name), stampHook(string(b), stampVersion()), 0o755); err != nil {
			return err
		}
	}
	return nil
}

var (
	skillStampRE  = regexp.MustCompile(`(?m)^[ \t]+yass-version:[ \t]*"?([^"\s]+)"?[ \t]*$`)
	agentsStampRE = regexp.MustCompile(`<!-- yass:begin version=(\S+)`)
	hookStampRE   = regexp.MustCompile(`(?m)^# yass-version:[ \t]*(\S+)[ \t]*$`)
)

// readStamp returns the version stamped in a YASS file (a playbook, an AGENTS.md or the hook), or ""
// for a file with none, as v0.1 and v0.2 wrote them.
func readStamp(p string) string {
	text := read(p)
	var m []string
	switch {
	case filepath.Base(p) == "SKILL.md":
		if f := fmRE.FindStringSubmatch(text); f != nil {
			m = skillStampRE.FindStringSubmatch(f[1])
		}
	case filepath.Base(p) == "AGENTS.md":
		m = agentsStampRE.FindStringSubmatch(text)
	default:
		m = hookStampRE.FindStringSubmatch(text)
	}
	if m == nil {
		return ""
	}
	return m[1]
}

// semverOf turns a stamp or the binary's version into SemVer ("0.3.0" -> "v0.3.0"); "" if it isn't one.
func semverOf(v string) string {
	if v == "" {
		return ""
	}
	if !strings.HasPrefix(v, "v") {
		v = "v" + v
	}
	if !semver.IsValid(v) {
		return ""
	}
	return v
}

// olderThanBinary says whether a stamp is older than this binary's version. An unstamped file is
// older than any stamped one (design §3); a binary without a version compares with nothing.
func olderThanBinary(stamp string) bool {
	bin := semverOf(binVersion)
	if bin == "" {
		return false
	}
	s := semverOf(stamp)
	return s == "" || semver.Compare(s, bin) < 0
}


## internal/yass/kit_test.go

package yass

import (
	"strings"
	"testing"

	"gopkg.in/yaml.v3"

	"github.com/donjaime/yass/kit"
)

func TestKitCarriesThePlaybooksAndHook(t *testing.T) {
	names, text := kitSkills()
	want := []string{"yass-log", "yass-plan", "yass-shape", "yass-status", "yass-update", "yass-work"}
	if strings.Join(names, ",") != strings.Join(want, ",") {
		t.Fatalf("kit playbooks = %v, want %v", names, want)
	}
	for _, n := range names {
		if !strings.Contains(text[n], "name: "+n) {
			t.Errorf("%s: no name in its frontmatter", n)
		}
	}
	if b, err := kit.FS.ReadFile(HookPath + "/pre-commit"); err != nil || !strings.HasPrefix(string(b), "#!") {
		t.Errorf("kit hook missing or not a script: %v", err)
	}
}

func parseFront(t *testing.T, text string) map[string]any {
	t.Helper()
	m := fmRE.FindStringSubmatch(text)
	if m == nil {
		t.Fatalf("no frontmatter in %q", text)
	}
	var out map[string]any
	if err := yaml.Unmarshal([]byte(m[1]), &out); err != nil {
		t.Fatalf("frontmatter isn't YAML: %v\n%s", err, m[1])
	}
	return out
}

func TestStampSkill(t *testing.T) {
	_, text := kitSkills()
	for name, src := range text {
		fm := parseFront(t, stampSkill(src, "0.3.0"))
		if fm["name"] != name || fm["description"] == nil {
			t.Errorf("%s: stamping lost name or description: %v", name, fm)
		}
		md, _ := fm["metadata"].(map[string]any)
		if md["yass-version"] != "0.3.0" {
			t.Errorf("%s: metadata = %v, want yass-version 0.3.0", name, fm["metadata"])
		}
	}
	cases := map[string]string{
		"existing metadata kept":    "---\nname: x\nmetadata:\n  owner: me\n---\nbody\n",
		"restamp replaces version":  "---\nname: x\nmetadata:\n  yass-version: \"0.1.0\"\n---\nbody\n",
		"pseudo-version with dirty": "---\nname: x\n---\nbody\n",
	}
	for label, src := range cases {
		v := "0.3.0"
		if strings.HasPrefix(label, "pseudo") {
			v = "0.3.1-0.20261005034705-35527c13ffaf+dirty"
		}
		out := stampSkill(src, v)
		md, _ := parseFront(t, out)["metadata"].(map[string]any)
		if md["yass-version"] != v {
			t.Errorf("%s: yass-version = %v, want %s\n%s", label, md["yass-version"], v, out)
		}
		if label == "existing metadata kept" && md["owner"] != "me" {
			t.Errorf("%s: lost owner: %v", label, md)
		}
		if strings.Count(out, "yass-version") != 1 || !strings.HasSuffix(out, "---\nbody\n") {
			t.Errorf("%s: bad stamp\n%s", label, out)
		}
	}
}

func TestStampHookAndAgents(t *testing.T) {
	h := stampHook("#!/bin/sh\n# YASS hook\nexec yass hook\n", "0.3.0")
	if h != "#!/bin/sh\n# yass-version: 0.3.0\n# YASS hook\nexec yass hook\n" {
		t.Errorf("stampHook = %q", h)
	}
	a := stampAgents(strings.TrimSpace(render("agents.md", nil)), "0.3.0")
	if !strings.HasPrefix(a, "<!-- yass:begin version=0.3.0 (managed by") || !agentsRE.MatchString(a) {
		t.Errorf("stampAgents marker = %q", strings.SplitN(a, "\n", 2)[0])
	}
}

func TestReadStampAndOlder(t *testing.T) {
	dir := t.TempDir()
	put := func(rel, text string) string {
		p := dir + "/" + rel
		if err := write(p, text); err != nil {
			t.Fatal(err)
		}
		return p
	}
	_, text := kitSkills()
	skill := put("a/.agents/skills/yass-work/SKILL.md", stampSkill(text["yass-work"], "0.3.0"))
	old := put("b/.agents/skills/yass-work/SKILL.md", text["yass-work"])
	agents := put("AGENTS.md", "# Ours\n\n"+stampAgents(strings.TrimSpace(render("agents.md", nil)), "0.3.1-0.20261005034705-35527c13ffaf+dirty")+"\n")
	hook := put("tools/yass/githooks/pre-commit", stampHook("#!/bin/sh\necho\n", "0.2.0"))
	for p, want := range map[string]string{skill: "0.3.0", old: "", agents: "0.3.1-0.20261005034705-35527c13ffaf+dirty", hook: "0.2.0"} {
		if got := readStamp(p); got != want {
			t.Errorf("readStamp(%s) = %q, want %q", p, got, want)
		}
	}
	defer func(v string) { binVersion = v }(binVersion)
	binVersion = "v0.3.0"
	for stamp, want := range map[string]bool{"": true, "0.2.0": true, "0.3.0": false, "0.3.1": false, "0.3.0-rc.1": true, "0.3.0+dirty": false} {
		if got := olderThanBinary(stamp); got != want {
			t.Errorf("olderThanBinary(%q) at v0.3.0 = %v, want %v", stamp, got, want)
		}
	}
	binVersion = "dev"
	if olderThanBinary("0.1.0") {
		t.Error("a binary without a version shouldn't call anything older")
	}
}

func TestUpgradeAction(t *testing.T) {
	cases := []struct{ stamp, bin, want string }{
		{"", "0.3.0", actWrite},                                     // unstamped (v0.1, v0.2): older than anything
		{"0.2.0", "0.3.0", actWrite},                                // older release
		{"0.3.0", "0.3.0", actKeep},                                 // same release: up to date
		{"0.4.0", "0.3.0", actNewer},                                // newer: upgrade the binary
		{"0.3.0", "v0.3.1-0.20261005034705-35527c13ffaf", actWrite}, // a clone build after a release upgrades it
		{"0.3.1-0.20261005034705-35527c13ffaf", "0.3.1", actWrite},  // a later release upgrades the clone build
		{"0.3.1", "v0.3.1-0.20261005034705-35527c13ffaf", actNewer}, // a clone build doesn't downgrade a release
		{"0.3.0", "v0.3.0+dirty", actWrite},                         // +dirty rewrites its own version
		{"0.3.0+dirty", "0.3.0", actKeep},                           // build metadata is ignored in comparing
		{"0.3.0-rc.1", "0.3.0", actWrite},                           // a release candidate is older than its release
	}
	for _, c := range cases {
		if got := upgradeAction(c.stamp, c.bin); got != c.want {
			t.Errorf("upgradeAction(%q, %q) = %s, want %s", c.stamp, c.bin, got, c.want)
		}
	}
}


## internal/yass/main.go

// Package yass is the YASS CLI: planned work as change folders.
//
// Planned work lives in yass/changes/<date>-<slug>/, finished work in yass/archive/. A small change
// is a change.md. A large one adds prd.md, plan.md, an optional design.md, and PR-sized pieces as
// folders inside it. Any folder named yass/ with changes/ or archive/ in it is picked up, so a
// monorepo can have several. A yass.yaml can stand in for a yass/ folder and point to one elsewhere.
package yass

import (
	"fmt"
	"os"
	"strings"
)

const usage = `yass: Yet Another Spec System. Planned work as change folders.

usage: yass <command> [options]

  init [dir] [--path P] [--agents]      set up YASS: the yass folder, and for the repo itself the
       [--claude] [--global] [--hooks]  AGENTS.md section, the playbooks and the hook script; for a
       [--no-agents] [--private]        folder (dir), only its planning, or with --agents its own
                                        AGENTS.md section and playbooks too (the hook stays at the root);
                                        --path writes a yass.yaml pointing to P;
                                        --claude adds Claude Code's copies and the CLAUDE.md import;
                                        --global puts the playbooks in your user folder; --hooks turns
                                        the hook on; --no-agents skips AGENTS.md, the playbooks and the
                                        hook; --private keeps dir out of git, for this clone only
  upgrade                               bring every YASS file in the repo (playbooks, AGENTS.md sections,
                                        the hook) and your user folder's playbooks up to this version
  deinit [--global]                     take YASS out of this repo: its playbooks, AGENTS.md sections
                                        and hook; the yass folders stay. --global: your user folder's
                                        playbooks too
  new "<title>" [--large] [--design]    a change: a dated folder from a template
      [--in CHANGE] [--goal G] [--platforms P] [--source S] [--follows ARCHIVED]
  status [<change>] [--brief] [--archived] [--strict] [--since 14d|DATE] [--no-branches]
                                        what's in flight, or one change in detail, with the
                                        unmerged branches working on each (the last 30 days)
                                        --brief: one change's progress in a line, for commits and PRs
  archive <change> [--force]            move a finished change to the archive
  update [--check] [--latest] [--version V] [--require-provenance]
                                        get a newer yass binary: the repo's version when the repo is
                                        ahead of this one, else the latest release; verified first
  evict                                 move the oldest archived months out of the working tree, into
                                        git history, where a yass folder is past its keep: setting
  decisions [--since D] [--until D] [--about TEXT] [--change C] [--cites SHA] [--limit N] [--json]
                                        past decisions, active and archived, newest first
  root [-v]                             print the yass folder(s) this repo uses; -v says whether
                                        each is inline (in this repo) or separate
  paths [--only RANGE]                  the repo paths that are YASS's, as globs (for CI filters);
                                        --only exits 0 if a commit range touches nothing else
  template <name>                       print a template: change, change-large, prd, plan, design
  hook [--range A..B] [--strict]        the optional pre-commit check (also for CI); with plans kept
                                        apart, also commits' yass: trailers (--prepare-msg, --commit-msg)
  use [<change>] [--ac ACs] [--clear]   the change this worktree's commits name in their yass: trailer
  version

With no command, yass runs status.
`

type spec struct {
	bools, values  []string
	minPos, maxPos int
	use            string
}

var specs = map[string]spec{
	"init":    {[]string{"no-agents", "private", "claude", "global", "hooks", "agents"}, []string{"path"}, 0, 1, "yass init [dir] [--path P] [--agents] [--claude] [--global] [--hooks] [--no-agents] [--private]"},
	"new":     {[]string{"large", "design"}, []string{"in", "goal", "platforms", "source", "follows"}, 1, 1, `yass new "<title>" [--large] [--design] [--in CHANGE] [--goal G] [--platforms P] [--source S] [--follows ARCHIVED]`},
	"status":  {[]string{"archived", "strict", "no-branches", "brief"}, []string{"since"}, 0, 1, "yass status [<change>] [--brief] [--archived] [--strict] [--since 14d|DATE] [--no-branches]"},
	"archive": {[]string{"force"}, nil, 1, 1, "yass archive <change> [--force]"},
	"decisions": {[]string{"json"}, []string{"since", "until", "about", "change", "limit", "cites"}, 0, 0,
		"yass decisions [--since YYYY-MM-DD] [--until YYYY-MM-DD] [--about TEXT] [--change C] [--cites SHA] [--limit N] [--json]"},
	"update":   {[]string{"check", "latest", "require-provenance"}, []string{"version"}, 0, 0, "yass update [--check] [--latest] [--version vX.Y.Z] [--require-provenance]"},
	"evict":    {nil, nil, 0, 0, "yass evict"},
	"root":     {[]string{"v"}, nil, 0, 0, "yass root [-v]"},
	"paths":    {nil, []string{"only"}, 0, 0, "yass paths [--only RANGE]"},
	"template": {nil, nil, 1, 1, "yass template <change|change-large|prd|plan|design|readme|agents>"},
	"hook":     {[]string{"strict", "prepare-msg", "commit-msg", "require-trailers"}, []string{"range"}, 0, 3, "yass hook [--range A..B [--require-trailers]] [--strict] | --prepare-msg <file> [source] | --commit-msg <file>"},
	"use":      {[]string{"clear"}, []string{"ac"}, 0, 1, "yass use [<change>] [--ac ac1,ac2] [--clear]"},
	"upgrade":  {nil, nil, 0, 0, "yass upgrade"},
	"deinit":   {[]string{"global"}, nil, 0, 0, "yass deinit [--global]"},
	"version":  {nil, nil, 0, 0, "yass version"},
}

type args struct {
	pos []string
	b   map[string]bool
	v   map[string]string
}

func contains(list []string, s string) bool {
	for _, x := range list {
		if x == s {
			return true
		}
	}
	return false
}

// parse reads options anywhere among the positionals: --flag, --name value, --name=value.
func parse(sp spec, argv []string) (*args, error) {
	a := &args{b: map[string]bool{}, v: map[string]string{}}
	for i := 0; i < len(argv); i++ {
		arg := argv[i]
		if arg == "--" {
			a.pos = append(a.pos, argv[i+1:]...)
			break
		}
		if len(arg) == 2 && arg[0] == '-' && contains(sp.bools, arg[1:]) { // a one-letter switch, like yass root -v
			a.b[arg[1:]] = true
			continue
		}
		if !strings.HasPrefix(arg, "--") {
			a.pos = append(a.pos, arg)
			continue
		}
		name, val, hasVal := strings.Cut(arg[2:], "=")
		switch {
		case contains(sp.bools, name) && !hasVal:
			a.b[name] = true
		case contains(sp.values, name):
			if !hasVal {
				if i+1 >= len(argv) {
					return nil, fmt.Errorf("--%s needs a value", name)
				}
				i++
				val = argv[i]
			}
			a.v[name] = val
		default:
			return nil, fmt.Errorf("unknown option: %s", arg)
		}
	}
	if len(a.pos) < sp.minPos || len(a.pos) > sp.maxPos {
		return nil, fmt.Errorf("wrong number of arguments")
	}
	return a, nil
}

// Main runs the CLI and returns its exit code.
func Main(argv []string, version string) int {
	binVersion = version
	return run(argv)
}

// MainChannel is Main for a binary that also knows how it was built: "release" for release builds.
func MainChannel(argv []string, version, channel string) int {
	binVersion, binChannel = version, channel
	return run(argv)
}

func run(argv []string) int {
	cleanOldBinary()
	cmd, explicit := "status", false
	if len(argv) > 0 && !strings.HasPrefix(argv[0], "-") {
		cmd, argv, explicit = argv[0], argv[1:], true
	}
	for _, x := range argv {
		switch x {
		case "-h", "--help":
			if sp, ok := specs[cmd]; ok && explicit {
				fmt.Println("usage: " + sp.use)
			} else {
				fmt.Print(usage)
			}
			return 0
		case "--version":
			if !explicit || !contains(specs[cmd].values, "version") { // yass update --version vX.Y.Z is its own
				fmt.Println("yass " + binVersion)
				return 0
			}
		}
	}
	sp, ok := specs[cmd]
	if !ok {
		fmt.Fprintf(os.Stderr, "%syass: unknown command '%s'\n", usage, cmd)
		return 2
	}
	a, err := parse(sp, argv)
	if err != nil {
		fmt.Fprintf(os.Stderr, "usage: %s\nyass: %v\n", sp.use, err)
		return 2
	}
	code := 0
	switch cmd {
	case "init":
		err = cmdInit(a)
	case "deinit":
		err = cmdDeinit(a)
	case "new":
		err = cmdNew(a)
	case "status":
		code, err = cmdStatus(a)
	case "archive":
		err = cmdArchive(a)
	case "update":
		code, err = cmdUpdate(a)
	case "use":
		err = cmdUse(a)
	case "evict":
		err = cmdEvict(a)
	case "decisions":
		code, err = cmdDecisions(a)
	case "root":
		err = cmdRoot(a)
	case "paths":
		return cmdPaths(a)
	case "template":
		err = cmdTemplate(a)
	case "hook":
		return cmdHook(a)
	case "upgrade":
		code, err = cmdUpgrade(a)
	case "version":
		fmt.Println("yass " + binVersion)
	}
	if err != nil {
		fmt.Fprintf(os.Stderr, "yass: %v\n", err)
		return 2
	}
	return code
}


## internal/yass/migrate.go

package yass

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"
)

// Migrations bring an older yass folder up to how this version lays it out. Each step tells from
// the files whether it's needed and does nothing when it isn't, so there's no format version to
// track, and a step can be deleted in a later release once nobody needs it (design §5).
type migration struct {
	name   string
	needed func(root *Root) bool
	// run changes the working tree and stages what it can, and returns what it did in a line.
	run func(r *Repo, root *Root) (string, error)
}

var migrations = []migration{
	{"archive-into-months", flatArchived, archiveIntoMonths},
}

// migrate runs every needed step for every yass folder and says how to commit each one on its own.
func migrate(r *Repo, steps []migration) error {
	for _, root := range r.Roots {
		for _, m := range steps {
			if !m.needed(root) {
				continue
			}
			did, err := m.run(r, root)
			if err != nil {
				return fmt.Errorf("%s in %s: %v", m.name, r.disp(root.Dir), err)
			}
			fmt.Printf("migrated %s (%s): %s\n", r.disp(root.Dir), m.name, did)
			if top, ok := git(root.Dir, "rev-parse", "--show-toplevel"); ok {
				rel := r.disp(root.Dir)
				cmd := fmt.Sprintf("git commit -m \"yass: %s\" -- %s", m.name, rel)
				if gitTop := canon(filepath.FromSlash(strings.TrimSpace(top))); gitTop != r.Top {
					cmd = fmt.Sprintf("git -C %s commit -m \"yass: %s\" -- %s", gitTop, m.name, root.Dir)
				}
				fmt.Printf("  commit it on its own: %s\n", cmd)
			}
		}
	}
	return nil
}

// flatArchived lists the changes sitting directly in archive/, from before archives went by month.
func flatArchived(root *Root) bool { return len(flatArchive(root)) > 0 }

func flatArchive(root *Root) []string {
	var out []string
	for _, name := range subdirs(filepath.Join(root.Dir, "archive")) {
		if !yearRE.MatchString(name) {
			out = append(out, name)
		}
	}
	return out
}

// archiveIntoMonths moves each flat archived change to archive/<YYYY>/<MM>/ and stamps archived:.
// The date is its archived: stamp if it has one, else the date of the latest commit that added
// files to it where it is, else the date in its name.
func archiveIntoMonths(r *Repo, root *Root) (string, error) {
	base := filepath.Join(root.Dir, "archive")
	added := archiveDates(base)
	n := 0
	for _, name := range flatArchive(root) {
		src := filepath.Join(base, name)
		head := filepath.Join(src, "change.md")
		meta, _ := frontmatter(readText(head))
		when, err := time.Parse(time.RFC3339, meta["archived"])
		if err != nil {
			if t, ok := added[name]; ok {
				when = t
			} else if t, err := time.Parse("2006-01-02", name[:min(len(name), 10)]); err == nil {
				when = t
			} else {
				return "", fmt.Errorf("%s: no archived: stamp, no commit that added it, and no date in its name; give it an archived: stamp", r.disp(src))
			}
		}
		dest := archiveDir(root, name, when)
		if exists(dest) {
			return "", fmt.Errorf("%s already exists", r.disp(dest))
		}
		if err := os.MkdirAll(filepath.Dir(dest), 0o755); err != nil {
			return "", err
		}
		if _, ok := git(base, "mv", src, dest); !ok {
			if err := os.Rename(src, dest); err != nil {
				return "", err
			}
			git(base, "add", "-A", "--", src)
		}
		if meta["archived"] == "" && isFile(filepath.Join(dest, "change.md")) {
			h := filepath.Join(dest, "change.md")
			if err := write(h, setMeta(read(h), "archived", when.UTC().Format(time.RFC3339))); err != nil {
				return "", err
			}
		}
		git(base, "add", "-A", "--", dest)
		n++
	}
	return fmt.Sprintf("moved %d archived change(s) into archive/<YYYY>/<MM>/", n), nil
}

// archiveDates is when git last added files to each flat archived change, by change name, from
// one git log over the archive. Outside git, or for changes git never saw, there's no entry.
func archiveDates(base string) map[string]time.Time {
	out := map[string]time.Time{}
	log, ok := git(base, "log", "--diff-filter=A", "--relative", "--name-only", "--format=@%cI", "--", ".")
	if !ok {
		return out
	}
	var when time.Time
	for _, l := range strings.Split(log, "\n") {
		if strings.HasPrefix(l, "@") {
			when, _ = time.Parse(time.RFC3339, l[1:])
			continue
		}
		parts := strings.Split(l, "/")
		if len(parts) < 2 || when.IsZero() {
			continue
		}
		if _, seen := out[parts[0]]; !seen { // newest first: the latest add wins
			out[parts[0]] = when
		}
	}
	return out
}


## internal/yass/migrate_test.go

package yass

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestMigrateRunsOnlyNeededSteps(t *testing.T) {
	root := &Root{Dir: t.TempDir()}
	r := &Repo{Top: root.Dir, Roots: []*Root{root}}
	ran := map[string]int{}
	step := func(name string, needed bool) migration {
		return migration{name, func(*Root) bool { return needed },
			func(*Repo, *Root) (string, error) { ran[name]++; return "did it", nil }}
	}
	if err := migrate(r, []migration{step("a", true), step("b", false)}); err != nil {
		t.Fatal(err)
	}
	if ran["a"] != 1 || ran["b"] != 0 {
		t.Fatalf("ran = %v, want a once and not b", ran)
	}
	// A list without the archive step (as after it's deleted) still runs clean.
	if err := migrate(r, nil); err != nil {
		t.Fatal(err)
	}
	for _, m := range migrations {
		if m.name == "" || m.needed == nil || m.run == nil {
			t.Fatalf("incomplete step %+v", m)
		}
	}
}

func TestArchiveIntoMonthsOutsideGit(t *testing.T) {
	root := &Root{Dir: t.TempDir()}
	r := &Repo{Top: root.Dir, Roots: []*Root{root}}
	mk := func(name, text string) {
		p := filepath.Join(root.Dir, "archive", name)
		os.MkdirAll(p, 0o755)
		os.WriteFile(filepath.Join(p, "change.md"), []byte(text), 0o644)
	}
	mk("2026-03-04-plain", "# Plain\n")
	mk("2026-03-05-stamped", "---\narchived: 2026-05-01T10:00:00Z\n---\n# Stamped\n")
	os.MkdirAll(filepath.Join(root.Dir, "archive", "2026", "02", "2026-02-01-already"), 0o755)
	if !flatArchived(root) {
		t.Fatal("flat archive not seen")
	}
	if _, err := archiveIntoMonths(r, root); err != nil {
		t.Fatal(err)
	}
	got := readText(filepath.Join(root.Dir, "archive", "2026", "03", "2026-03-04-plain", "change.md"))
	if !strings.HasPrefix(got, "---\narchived: 2026-03-04T00:00:00Z\n---\n# Plain") {
		t.Fatalf("plain = %q", got)
	}
	if !isFile(filepath.Join(root.Dir, "archive", "2026", "05", "2026-03-05-stamped", "change.md")) {
		t.Fatal("a change with archived: goes by its stamp")
	}
	if flatArchived(root) {
		t.Fatal("still flat after migrating")
	}
}


## internal/yass/paths.go

package yass

import (
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

// yassPaths are the repo-relative globs that are YASS's: each yass folder inside the repo, and each
// yass.yaml. A plans folder outside the repo has none, and folders a yass.yaml ignores aren't YASS's.
// The hook scripts in tools/yass/ aren't either: changing them is changing code. Nor are team
// folders included through git config: git never sees them, so no commit contains them.
func yassPaths(r *Repo) []string {
	var out []string
	for _, root := range r.Roots {
		if !within(root.Dir, r.Top) || root.Included { // git never sees an included folder
			continue
		}
		if root.Dir == r.Top { // a plans repo (path: .): everything in it is YASS's
			return []string{"**"}
		}
		if rel, err := filepath.Rel(r.Top, root.Dir); err == nil {
			out = append(out, filepath.ToSlash(rel)+"/**")
		}
	}
	for _, f := range r.Configs {
		if rel, err := filepath.Rel(r.Top, f); err == nil && within(f, r.Top) {
			out = append(out, filepath.ToSlash(rel))
		}
	}
	sort.Strings(out)
	return out
}

// isYassPath says whether a repo-relative path is under one of the globs yassPaths prints.
func isYassPath(p string, globs []string) bool {
	for _, g := range globs {
		if g == "**" {
			return true
		}
		if dir, ok := strings.CutSuffix(g, "/**"); ok {
			if strings.HasPrefix(p, dir+"/") {
				return true
			}
		} else if p == g {
			return true
		}
	}
	return false
}

// cmdPaths prints YASS's paths, or with --only, says whether a commit range changed nothing else:
// exit 0 when it's plans only (CI can skip builds and tests), 1 when anything else changed (or
// nothing did: when in doubt, run the checks), 2 when the range can't be read.
func cmdPaths(a *args) int {
	r := findRepo()
	globs := yassPaths(r)
	rng, only := a.v["only"]
	if !only {
		for _, g := range globs {
			fmt.Println(g)
		}
		return 0
	}
	out, ok := git(r.Top, "diff", "--name-only", "--no-renames", "-z", rng)
	if rng == "" || !ok {
		fmt.Fprintf(os.Stderr, "yass: can't read the range '%s'. In CI, fetch history first (e.g. actions/checkout with fetch-depth: 0).\n", rng)
		return 2
	}
	var other []string
	n := 0
	for _, f := range strings.Split(out, "\x00") {
		if f == "" {
			continue
		}
		n++
		if !isYassPath(f, globs) {
			other = append(other, f)
		}
	}
	switch {
	case n == 0:
		fmt.Println("nothing changed in the range")
		return 1
	case len(other) == 0:
		fmt.Printf("plans only: %d file(s), all YASS's\n", n)
		return 0
	}
	fmt.Printf("%d of %d changed file(s) aren't YASS's, e.g. %s\n", len(other), n, other[0])
	return 1
}


## internal/yass/repo.go

package yass

import (
	"errors"
	"fmt"
	"io/fs"
	"os"
	"os/exec"
	"path"
	"path/filepath"
	"regexp"
	"sort"
	"strconv"
	"strings"
	"time"
)

// skipDirs are only used outside git; inside git, .gitignore decides.
var skipDirs = map[string]bool{".git": true, "node_modules": true, "__pycache__": true}

var yassDirRE = regexp.MustCompile(`^(?:(.*)/)?yass/(?:changes|archive)/`)

func git(dir string, args ...string) (string, bool) {
	cmd := exec.Command("git", args...)
	cmd.Dir = dir
	if dir != "" {
		// Asked about a particular folder, git should find that folder's repo. Hooks in a linked
		// worktree run with GIT_DIR set, which would point every git call at the code worktree,
		// even one meant for a plans repo elsewhere. Calls with no folder keep it (and the
		// hook's GIT_INDEX_FILE).
		cmd.Env = gitEnvFree()
	}
	out, err := cmd.Output()
	return string(out), err == nil
}

// gitEnvFree is the environment without the variables that pin git to a repo.
func gitEnvFree() []string {
	var out []string
	for _, kv := range os.Environ() {
		switch k, _, _ := strings.Cut(kv, "="); k {
		case "GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_COMMON_DIR", "GIT_PREFIX":
			continue
		}
		out = append(out, kv)
	}
	return out
}

// canon makes a path absolute and resolves symlinks, so paths compare reliably. For a path that
// doesn't exist yet, it resolves the deepest folder that does and keeps the rest: on macOS, a new
// folder under /var/... must still compare as inside a repo under /private/var/....
func canon(p string) string {
	if a, err := filepath.Abs(p); err == nil {
		p = a
	}
	p = filepath.Clean(p)
	for dir, rest := p, ""; ; {
		if r, err := filepath.EvalSymlinks(dir); err == nil {
			return filepath.Join(r, rest)
		}
		parent := filepath.Dir(dir)
		if parent == dir {
			return p
		}
		rest = filepath.Join(filepath.Base(dir), rest)
		dir = parent
	}
}

func within(p, dir string) bool {
	return p == dir || strings.HasPrefix(p, dir+string(filepath.Separator))
}

func getwd() string {
	wd, _ := os.Getwd()
	return canon(wd)
}

// repoTop is the git repository root; outside git, the highest folder above here with a yass folder.
func repoTop(cwd string) string {
	if out, ok := git(cwd, "rev-parse", "--show-toplevel"); ok {
		return canon(filepath.FromSlash(strings.TrimSpace(out)))
	}
	here, best := cwd, cwd
	for {
		if isDir(filepath.Join(here, "yass", "changes")) || isDir(filepath.Join(here, "yass", "archive")) ||
			isFile(filepath.Join(here, ConfigName)) {
			best = here
		}
		parent := filepath.Dir(here)
		if parent == here {
			return best
		}
		here = parent
	}
}

// Root is one yass folder and the part of the repo it serves.
type Root struct {
	Dir    string // the yass folder: changes/, archive/, README.md
	Owner  string // the folder it serves: where yass/ or the yass.yaml sits
	Config string // the yass.yaml that points to Dir, if any
	Branch string // the code branch that counts as merged, from yass.yaml
	// Included: a team folder git never sees, named in the clone's git config (yass.include)
	Included bool
	Queue    []string // the changes queue.md ranks, top first, as written
}

// keep is how many archived changes the yass folder should keep in its working tree (a soft limit).
func (root *Root) keep() int {
	if root.Config != "" {
		if c, _, err := loadConfig(root.Config); err == nil && c.Archive.Keep > 0 {
			return c.Archive.Keep
		}
	}
	return DefaultKeep
}

// rank is a change's place in its yass folder's queue.md; changes it doesn't list come after all that it does.
func (root *Root) rank(name string) int {
	for i, n := range root.Queue {
		if n == name {
			return i
		}
	}
	return len(root.Queue)
}

// created is when a change was made, for ordering: its created: stamp, else its folder's date,
// which sorts before any stamp from the same day.
func (c *Change) created() string {
	if t, err := time.Parse(time.RFC3339, c.Meta["created"]); err == nil {
		return t.UTC().Format(time.RFC3339)
	}
	if nameRE.MatchString(c.Name) {
		return c.Name[:10]
	}
	return ""
}

// byCreated orders changes oldest first; ties keep their order (by name, as loaded).
func byCreated(cs []*Change) {
	sort.SliceStable(cs, func(i, j int) bool { return cs[i].created() < cs[j].created() })
}

type fileBox struct{ file, mark, text string }

// Change is one change folder (or a piece inside one).
type Change struct {
	Path, Name, Head, Title string
	Root                    *Root
	Parent                  *Change
	Meta                    map[string]string
	Pieces                  []*Change
	Large                   bool
	Archived                bool
	Evicted                 bool              // archived, then evicted: only its month's .evicted file names it
	Deps                    []*Change         // the changes blocked: names, when it names changes rather than a reason
	files                   map[string]string // an evicted change read back from git: its .md files by name
}

func newChange(p string, root *Root, parent *Change) *Change {
	c := nameOnly(p, root, parent)
	c.read()
	return c
}

// nameOnly is a change known by its folder alone, its files unread: how archived changes load, so
// commands that only need their names (follows:, blocked:, the queue) stay fast however big the
// archive gets. readArchived reads them when a command shows what's in them.
func nameOnly(p string, root *Root, parent *Change) *Change {
	name := filepath.Base(p)
	return &Change{Path: p, Root: root, Parent: parent, Name: name, Title: name, Meta: map[string]string{}}
}

func (c *Change) read() {
	p := c.Path
	c.setHead(readText(filepath.Join(p, "change.md")))
	c.Large = exists(filepath.Join(p, "prd.md")) || exists(filepath.Join(p, "plan.md"))
}

// setHead takes in a change's change.md: its frontmatter and title.
func (c *Change) setHead(text string) {
	c.Head = text
	var body string
	c.Meta, body = frontmatter(c.Head)
	c.Title = c.Name
	if m := titleRE.FindStringSubmatch(body); m != nil {
		c.Title = strings.TrimSpace(m[1])
	}
}

// readArchived reads the archived changes' files, for commands that show what's in them, and
// warns about any it can't read.
func (r *Repo) readArchived() {
	r.archivedPieces()
	for _, c := range flatten(r.Archived) {
		if c.Evicted {
			continue
		}
		head := filepath.Join(c.Path, "change.md")
		if _, err := os.ReadFile(head); err != nil && !os.IsNotExist(err) {
			r.warn("%s: can't read it: %v", r.disp(head), errors.Unwrap(err))
		}
		c.read()
		if t := c.Meta["archived"]; t != "" && c.Parent == nil {
			if _, err := time.Parse(time.RFC3339, t); err != nil {
				r.warn("%s: archived '%s' isn't a time like 2026-10-05T14:32:07Z", r.disp(c.Path), t)
			}
		}
	}
	for _, c := range r.Archived {
		byCreated(c.Pieces)
	}
	byCreated(r.Archived)
}

func mdFiles(dir string) []string {
	entries, _ := os.ReadDir(dir)
	var out []string
	for _, e := range entries {
		if !e.IsDir() && strings.HasSuffix(e.Name(), ".md") {
			out = append(out, e.Name())
		}
	}
	sort.Strings(out)
	return out
}

// ownBoxes are the boxes in this folder's own .md files (pieces count separately).
func (c *Change) ownBoxes() []fileBox {
	var out []fileBox
	if c.files != nil {
		names := make([]string, 0, len(c.files))
		for f := range c.files {
			names = append(names, f)
		}
		sort.Strings(names)
		for _, f := range names {
			for _, b := range boxes(c.files[f]) {
				out = append(out, fileBox{f, b.mark, b.text})
			}
		}
		return out
	}
	for _, f := range mdFiles(c.Path) {
		for _, b := range boxes(read(filepath.Join(c.Path, f))) {
			out = append(out, fileBox{f, b.mark, b.text})
		}
	}
	return out
}

// allOpen lists open and in-progress boxes in this folder's own files and its pieces': the same
// boxes tally and progress count, so status and archive always agree.
var (
	deliversRE  = regexp.MustCompile(`(?i)\bdelivers\b`)
	acRefRE     = regexp.MustCompile(`\bAC(\d+)(?:\s*[–-]\s*AC(\d+))?`)
	criterionRE = regexp.MustCompile(`^AC(\d+)\b`)
)

// deliveredIDs reads a piece's "Delivers AC1, AC3–AC5 in plan.md" box: the parent's criteria it
// names, as lists and ranges, ignoring the words around them ("and AC19's preview card" counts AC19).
func deliveredIDs(text string) []int {
	loc := deliversRE.FindStringIndex(text)
	if loc == nil {
		return nil
	}
	var out []int
	for _, m := range acRefRE.FindAllStringSubmatch(text[loc[1]:], -1) {
		from, _ := strconv.Atoi(m[1])
		to := from
		if m[2] != "" {
			to, _ = strconv.Atoi(m[2])
		}
		for n := from; n <= to && n-from < 1000; n++ {
			out = append(out, n)
		}
	}
	return out
}

// delivered is the parent's criteria (by number) that pieces deliver, and whether every piece
// delivering each one is done: a criterion is met once all the pieces that name it are.
func (c *Change) delivered() map[int]bool {
	if c.Parent != nil || len(c.Pieces) == 0 {
		return nil
	}
	by := map[int]map[*Change]bool{}
	for _, p := range c.Pieces {
		for _, b := range p.ownBoxes() {
			for _, n := range deliveredIDs(b.text) {
				if by[n] == nil {
					by[n] = map[*Change]bool{}
				}
				by[n][p] = true
			}
		}
	}
	out := map[int]bool{}
	for n, ps := range by {
		all := true
		for p := range ps {
			all = all && p.state() == "done"
		}
		out[n] = all
	}
	return out
}

// countedBoxes are a folder's own boxes as progress counts them: a parent's criterion in plan.md
// that its pieces have delivered counts as done, so no closing commit is needed to tick it.
func (c *Change) countedBoxes() []fileBox {
	bs := c.ownBoxes()
	d := c.delivered()
	if len(d) == 0 {
		return bs
	}
	for i, b := range bs {
		if b.file != "plan.md" || !isOpen(b.mark) {
			continue
		}
		if m := criterionRE.FindStringSubmatch(b.text); m != nil {
			if n, _ := strconv.Atoi(m[1]); d[n] {
				bs[i].mark = markDone
			}
		}
	}
	return bs
}

var boxLineRE = regexp.MustCompile(`^(\s*[-*+]\s+\[)([ /])(\]\s+)(AC(\d+)\b.*)$`)

// markDelivered ticks the open criteria in a plan.md that the pieces delivered, leaving every other
// line as it was, comments included.
func markDelivered(text string, delivered map[int]bool) string {
	ls := strings.SplitAfter(text, "\n")
	inComment := false
	for i, l := range ls {
		body := strings.TrimRight(l, "\r\n")
		wasComment := inComment
		if strings.Contains(body, "<!--") {
			inComment = true
		}
		if strings.Contains(body, "-->") {
			inComment = false
			wasComment = true
		}
		if wasComment || inComment {
			continue
		}
		if m := boxLineRE.FindStringSubmatch(body); m != nil {
			if n, _ := strconv.Atoi(m[5]); delivered[n] {
				ls[i] = m[1] + "x" + m[3] + m[4] + l[len(body):]
			}
		}
	}
	return strings.Join(ls, "")
}

func (c *Change) allOpen() []fileBox {
	var out []fileBox
	for _, b := range c.countedBoxes() {
		if isOpen(b.mark) {
			out = append(out, b)
		}
	}
	for _, p := range c.Pieces {
		for _, b := range p.allOpen() {
			out = append(out, fileBox{p.Name + "/" + b.file, b.mark, b.text})
		}
	}
	return out
}

// tally counts the boxes in this folder and its pieces: open (or in progress), done, and dropped.
func (c *Change) tally() (open, done, dropped int) {
	for _, b := range c.countedBoxes() {
		switch b.mark {
		case markDone:
			done++
		case markDropped:
			dropped++
		default:
			open++
		}
	}
	for _, p := range c.Pieces {
		o, d, x := p.tally()
		open, done, dropped = open+o, done+d, dropped+x
	}
	return
}

// progress is (done, total) over this folder and its pieces; dropped boxes don't count, in-progress ones aren't done.
func (c *Change) progress() (int, int) {
	open, done, _ := c.tally()
	return done, open + done
}

func (c *Change) blocked() string { return c.Meta["blocked"] }

// met: a change others wait on is done (every box checked) or archived.
func (c *Change) met() bool { return c.Archived || c.state() == "done" }

// blockedBy is why a change can't proceed: a reason for a human, or the changes it's still waiting on.
// A blocked: that names only changes that are all met blocks nothing.
func (c *Change) blockedBy() (reason string, waiting []*Change) {
	if c.Deps == nil {
		return c.blocked(), nil
	}
	for _, d := range c.Deps {
		if !d.met() {
			waiting = append(waiting, d)
		}
	}
	return "", waiting
}

func labels(cs []*Change) string {
	var out []string
	for _, c := range cs {
		out = append(out, c.label())
	}
	return strings.Join(out, ", ")
}

func (c *Change) log() string { return commentRE.ReplaceAllString(section(c.Head, "Log"), "") }

// started: a Log entry, or a box in progress or done, here or in a piece. Shaping and planning write none.
func (c *Change) started() bool {
	if logEntryRE.MatchString(c.log()) {
		return true
	}
	for _, b := range c.countedBoxes() {
		if b.mark == markDone || b.mark == markWorking {
			return true
		}
	}
	for _, p := range c.Pieces {
		if p.started() {
			return true
		}
	}
	return false
}

func (c *Change) nextStep() string {
	if m := nextRE.FindAllStringSubmatch(c.log(), -1); len(m) > 0 {
		return strings.TrimSpace(m[len(m)-1][1])
	}
	own := c.countedBoxes()
	for _, mark := range []string{markWorking, markOpen} {
		for _, b := range own {
			if b.mark == mark {
				return b.text
			}
		}
	}
	return ""
}

// state is "not started", "in progress", or "done" (every box done or dropped: ready to archive).
func (c *Change) state() string {
	if open, done, dropped := c.tally(); open == 0 && done+dropped > 0 {
		return "done"
	}
	if c.started() {
		return "in progress"
	}
	return "not started"
}

func (c *Change) label() string {
	if c.Parent == nil {
		return c.Name
	}
	return c.Parent.Name + "/" + c.Name
}

// Repo is every yass folder this repository uses, and the changes in them.
type Repo struct {
	Top, Cwd         string
	Roots            []*Root
	Ignored          []string // folders a yass.yaml ignores
	Configs          []string // the yass.yaml files that count, wherever they point
	Warnings         []string
	Notes            []string // worth knowing, but not a problem: they don't fail --strict
	Linked           bool     // a yass.yaml's folder wasn't found from this linked worktree
	Active, Archived []*Change
	piecesFound      bool              // whether archivedPieces has run
	absent           map[string]string // changes git has that aren't on disk (sparse checkouts), by name
	merged           map[*Root]string  // the merged branch per yass folder, once looked up
	codeDir          *string           // this code repo's git common dir, once looked up
}

// findRepo finds the repo and its yass folders, without reading any changes.
func findRepo() *Repo {
	r := &Repo{Cwd: getwd()}
	r.Top = repoTop(r.Cwd)
	r.findRoots()
	return r
}

func loadRepo() *Repo {
	r := findRepo()
	r.link()
	for _, root := range r.Roots {
		root.Queue = parseQueue(readText(filepath.Join(root.Dir, QueueName)))
		r.Active = append(r.Active, r.load(root, "changes", true)...)
		r.Archived = append(r.Archived, r.load(root, "archive", false)...)
	}
	r.check()
	return r
}

func (r *Repo) warn(format string, a ...any) {
	r.Warnings = append(r.Warnings, fmt.Sprintf(format, a...))
}

func (r *Repo) note(format string, a ...any) {
	r.Notes = append(r.Notes, fmt.Sprintf(format, a...))
}

// noRoot says there's no yass folder to use, and how to get one. From a linked worktree that's never
// `yass init`: it would make a stray, empty plans folder where only this worktree looks.
func (r *Repo) noRoot(msg string) string {
	if r.Linked {
		return "no yass folder found from this worktree; see the warning"
	}
	return msg
}

// disp shows a path relative to the repo when it's inside it, and in full when it isn't.
func (r *Repo) disp(p string) string {
	if within(p, r.Top) {
		rel, _ := filepath.Rel(r.Top, p)
		return rel
	}
	return p
}

// findRoots finds every folder named yass/ with changes/ or archive/ in it, and every yass.yaml.
// Inside git, .gitignore decides, except that a yass.yaml at the top or above the current folder
// always counts, so it can be kept out of the repo. Folders a yass.yaml ignores don't count, unless
// you're standing in one: then it's a project of its own, and its folder is the top.
func (r *Repo) findRoots() {
	warned := len(r.Warnings)
	dirs, configs := map[string]bool{}, map[string]bool{}
	out, ok := git(r.Top, "ls-files", "-z", "--cached", "--others", "--exclude-standard", "--",
		":(glob)**/yass/changes/**", ":(glob)**/yass/archive/**", ":(glob)**/"+ConfigName)
	if ok {
		for _, f := range strings.Split(out, "\x00") { // -z: raw paths, even with non-ASCII names
			if f == "" {
				continue
			}
			if path.Base(f) == ConfigName {
				configs[filepath.Join(r.Top, filepath.FromSlash(f))] = true
			} else if m := yassDirRE.FindStringSubmatch(f); m != nil {
				dirs[filepath.Join(r.Top, filepath.FromSlash(m[1]), "yass")] = true
			}
		}
	} else {
		filepath.WalkDir(r.Top, func(p string, d fs.DirEntry, err error) error {
			if err != nil {
				return nil
			}
			if d.IsDir() {
				name := d.Name()
				if p != r.Top && (skipDirs[name] || strings.HasPrefix(name, ".")) {
					return filepath.SkipDir
				}
				if name == "yass" && (isDir(filepath.Join(p, "changes")) || isDir(filepath.Join(p, "archive"))) {
					dirs[p] = true
					return filepath.SkipDir
				}
			} else if d.Name() == ConfigName {
				configs[p] = true
			}
			return nil
		})
	}
	for d := r.Cwd; within(d, r.Top); d = filepath.Dir(d) {
		if f := filepath.Join(d, ConfigName); isFile(f) {
			configs[f] = true
		}
		if d == r.Top || filepath.Dir(d) == d {
			break
		}
	}

	// Settle ignores shallowest first: a yass.yaml in an ignored folder doesn't count, nor do its ignores.
	files := sortedKeys(configs)
	sort.SliceStable(files, func(i, j int) bool { return depth(files[i]) < depth(files[j]) })
	r.Ignored = nil
	for _, f := range files {
		if r.ignored(f) {
			delete(configs, f)
			continue
		}
		c, _, err := loadConfig(f)
		if err != nil {
			continue // reported below
		}
		ign, bad := ignoredDirs(f, c)
		for _, b := range bad {
			r.warn("%s: %s", r.disp(f), b)
		}
		r.Ignored = append(r.Ignored, ign...)
	}
	var inside string
	for _, d := range r.Ignored {
		if within(r.Cwd, d) && len(d) > len(inside) {
			inside = d
		}
	}
	if inside != "" {
		r.Top, r.Roots, r.Ignored, r.Warnings = inside, nil, nil, r.Warnings[:warned]
		r.findRoots()
		return
	}
	for d := range dirs {
		if r.ignored(d) {
			delete(dirs, d)
		}
	}

	included, borrowed := r.includes(configs)
	seen, claimed := map[string]bool{}, map[string]bool{}
	for _, f := range sortedKeys(configs) {
		if !included[f] {
			r.Configs = append(r.Configs, f)
		}
	}
	for _, f := range sortedKeys(configs) {
		owner := filepath.Dir(f)
		claimed[owner] = true
		dir, cfg, unknown, err := configDir(f)
		for _, k := range unknown {
			r.warn("%s: unknown setting '%s' (a newer yass may know it)", r.disp(f), k)
		}
		if err != nil {
			r.warn("%s: %v", r.disp(f), err)
			continue
		}
		if !isDir(dir) && strings.TrimSpace(cfg.Path) != "" {
			found, looked, linked := fromOtherCheckouts(f, strings.TrimSpace(cfg.Path))
			switch {
			case found != "":
				dir = found
			case linked:
				r.Linked = true
				places := r.disp(dir)
				for _, l := range looked {
					if l != dir {
						places += ", " + r.disp(l)
					}
				}
				r.warn("%s: points to a folder that isn't there from this worktree or the clone's other checkouts (looked in %s); check `path:`, or that the plans are where the main checkout expects them",
					r.disp(f), places)
				continue
			}
		}
		if !isDir(dir) {
			r.warn("%s: points to %s, which doesn't exist yet (`yass init` creates it)", r.disp(f), r.disp(dir))
			continue
		}
		dir = canon(dir)
		// On disk too, not just in git's list: info/exclude may hide a yass/ that was once a link.
		if local := filepath.Join(owner, "yass"); (dirs[local] || isRealYassDir(local)) && local != dir {
			r.warn("%s/: ignored, because %s points to %s", r.disp(local), r.disp(f), r.disp(dir))
		}
		if !seen[dir] {
			seen[dir] = true
			r.Roots = append(r.Roots, &Root{Dir: dir, Owner: owner, Config: f, Branch: strings.TrimSpace(cfg.Branch), Included: included[f]})
		}
	}
	for _, b := range borrowed {
		if !seen[b.Dir] {
			seen[b.Dir] = true
			r.Roots = append(r.Roots, b)
		}
	}
	for _, d := range sortedKeys(dirs) {
		owner := filepath.Dir(d)
		if claimed[owner] || seen[d] || !isDir(d) {
			continue
		}
		seen[d] = true
		r.Roots = append(r.Roots, &Root{Dir: d, Owner: owner})
	}
	if len(r.Roots) == 0 && len(r.Warnings) == warned {
		r.borrowConfig()
	}
	sort.SliceStable(r.Roots, func(i, j int) bool {
		a, b := r.Roots[i].Owner, r.Roots[j].Owner
		if da, db := depth(a), depth(b); da != db {
			return da < db
		}
		return a < b
	})
}

// ignored says whether p is inside a folder a yass.yaml ignores.
func (r *Repo) ignored(p string) bool {
	for _, d := range r.Ignored {
		if within(p, d) {
			return true
		}
	}
	return false
}

func depth(p string) int { return strings.Count(p, string(filepath.Separator)) }

func sortedKeys(m map[string]bool) []string {
	out := make([]string, 0, len(m))
	for k := range m {
		out = append(out, k)
	}
	sort.Strings(out)
	return out
}

func subdirs(dir string) []string {
	entries, _ := os.ReadDir(dir)
	var out []string
	for _, e := range entries {
		if e.IsDir() {
			out = append(out, e.Name())
		}
	}
	sort.Strings(out)
	return out
}

var (
	yearRE  = regexp.MustCompile(`^\d{4}$`)
	monthRE = regexp.MustCompile(`^\d{2}$`)
)

// changeDirs lists the change folders in changes/ or archive/. The archive keeps them in
// <YYYY>/<MM>/ folders by the month they were archived, and older archives keep them flat;
// both count, so a half-migrated archive still works.
func changeDirs(base string, archive bool) []string {
	var out []string
	for _, name := range subdirs(base) {
		if !archive || !yearRE.MatchString(name) {
			out = append(out, filepath.Join(base, name))
			continue
		}
		for _, month := range subdirs(filepath.Join(base, name)) {
			if !monthRE.MatchString(month) {
				continue
			}
			for _, c := range subdirs(filepath.Join(base, name, month)) {
				out = append(out, filepath.Join(base, name, month, c))
			}
		}
	}
	return out
}

// archiveDir is where a change archived at t goes: archive/<YYYY>/<MM>/<name>, by UTC month.
func archiveDir(root *Root, name string, t time.Time) string {
	t = t.UTC()
	return filepath.Join(root.Dir, "archive", t.Format("2006"), t.Format("01"), name)
}

// load reads a yass folder's active changes and their pieces, or, for the archive, only the names
// of its changes: their pieces are found when a name needs them (archivedPieces).
func (r *Repo) load(root *Root, kind string, warn bool) []*Change {
	base := filepath.Join(root.Dir, kind)
	var out []*Change
	for _, p := range changeDirs(base, kind == "archive") {
		name := filepath.Base(p)
		if kind == "archive" {
			c := nameOnly(p, root, nil)
			c.Archived = true
			out = append(out, c)
			continue
		}
		c := newChange(p, root, nil)
		out = append(out, c)
		if warn && c.Head == "" {
			r.warn("%s: no change.md", r.disp(p))
		}
		if warn && !nameRE.MatchString(name) {
			r.warn("%s: name it <YYYY-MM-DD>-<slug>", r.disp(p))
		}
		for _, sub := range subdirs(p) {
			sp := filepath.Join(p, sub)
			if !isFile(filepath.Join(sp, "change.md")) {
				continue
			}
			c.Pieces = append(c.Pieces, newChange(sp, root, c))
			if !warn {
				continue
			}
			filepath.WalkDir(sp, func(d string, e fs.DirEntry, err error) error {
				if err == nil && e.IsDir() && d != sp && isFile(filepath.Join(d, "change.md")) {
					r.warn("%s: pieces nest one level deep; move it up into %s", r.disp(d), c.Name)
				}
				return nil
			})
		}
		byCreated(c.Pieces)
	}
	if kind == "archive" {
		for _, m := range evictedMonths(root) {
			for _, name := range m.Changes {
				c := nameOnly(filepath.Join(root.Dir, "archive", filepath.FromSlash(m.Month), name), root, nil)
				c.Archived, c.Evicted = true, true
				out = append(out, c)
			}
		}
	}
	byCreated(out)
	return out
}

// archivedPieces finds the pieces of archived changes, by name, the first time a name isn't an
// archived change itself: it opens every archived change's folder, which plain status never needs.
func (r *Repo) archivedPieces() {
	if r.piecesFound {
		return
	}
	r.piecesFound = true
	for _, c := range r.Archived {
		if c.Evicted {
			continue
		}
		for _, sub := range subdirs(c.Path) {
			if sp := filepath.Join(c.Path, sub); isFile(filepath.Join(sp, "change.md")) {
				piece := nameOnly(sp, c.Root, c)
				piece.Archived = true
				c.Pieces = append(c.Pieces, piece)
			}
		}
	}
}

func (r *Repo) check() {
	known := map[string]bool{}
	addKnown := func() {
		for _, c := range append(append([]*Change{}, r.Active...), r.Archived...) {
			known[c.Name] = true
			for _, p := range c.Pieces {
				known[p.Name] = true
			}
		}
	}
	addKnown()
	isKnown := func(name string) bool {
		if !known[name] && !r.piecesFound {
			r.archivedPieces()
			addKnown()
		}
		return known[name]
	}
	for _, c := range r.everything() {
		if f := c.Meta["follows"]; f != "" && !isKnown(path.Base(strings.TrimRight(f, "/"))) {
			if where := r.notCheckedOut(path.Base(strings.TrimRight(f, "/"))); where != "" {
				r.note("%s: follows '%s', which is in %s, outside this sparse checkout, so it can't be checked", r.disp(c.Path), f, where)
			} else {
				r.warn("%s: follows '%s', which isn't in changes/ or archive/", r.disp(c.Path), f)
			}
		}
		if t := c.Meta["created"]; t != "" {
			if _, err := time.Parse(time.RFC3339, t); err != nil {
				r.warn("%s: created '%s' isn't a time like 2026-10-05T14:32:07Z", r.disp(c.Path), t)
			}
		}
	}
	r.checkDeps()
	r.checkQueues()
	r.checkCode()
}

// checkDeps reads each blocked: that names changes (comma-separated single words: a folder name, a
// path ending in one, or <change>/<piece>) into Deps, and warns about names that look like a change
// but aren't one, and about changes that end up waiting on themselves.
func (r *Repo) checkDeps() {
	all := append(flatten(r.Active), flatten(r.Archived)...)
	find := func(item string) *Change {
		if !r.piecesFound && !r.hasName(all, path.Base(item)) {
			r.archivedPieces()
			all = append(flatten(r.Active), flatten(r.Archived)...)
		}
		// By folder name, so a path keeps working after its change is archived; the path only
		// picks between changes that share a name.
		var hits, exact []*Change
		for _, c := range all {
			if c.Name == path.Base(item) {
				hits = append(hits, c)
				if strings.HasSuffix(filepath.ToSlash(c.Path), "/"+item) {
					exact = append(exact, c)
				}
			}
		}
		if len(hits) == 1 {
			return hits[0]
		}
		if len(exact) == 1 {
			return exact[0]
		}
		return nil
	}
	for _, c := range r.everything() {
		b := c.blocked()
		if b == "" {
			continue
		}
		var deps []*Change
		names := true
		for _, item := range strings.Split(b, ",") {
			item = strings.TrimRight(strings.TrimSpace(item), "/")
			if item == "" || strings.ContainsAny(item, " \t") {
				names = false
				continue
			}
			if d := find(item); d != nil {
				deps = append(deps, d)
				continue
			}
			names = false
			if nameRE.MatchString(path.Base(item)) {
				if where := r.notCheckedOut(path.Base(item)); where != "" {
					r.note("%s: blocked: '%s' is in %s, outside this sparse checkout, so it can't be checked", r.disp(c.Path), item, where)
				} else {
					r.warn("%s: blocked: '%s' looks like a change, but there's no such change (or more than one; give its path)", r.disp(c.Path), item)
				}
			}
		}
		if names && len(deps) > 0 {
			c.Deps = deps
		}
	}
	reported := map[string]bool{}
	for _, c := range r.everything() {
		if loop := depLoop(c, c, nil, map[*Change]bool{}); loop != nil {
			var names []string
			for _, x := range loop {
				names = append(names, x.label())
			}
			key := strings.Join(sortedCopy(names), " ")
			if !reported[key] {
				reported[key] = true
				r.warn("%s: blocked: waits on itself: %s", r.disp(c.Path), strings.Join(append([]string{c.label()}, names...), " → "))
			}
		}
	}
}

// depLoop returns the path of dependencies from cur back to start, if there is one.
func depLoop(start, cur *Change, trail []*Change, seen map[*Change]bool) []*Change {
	for _, d := range cur.Deps {
		next := append(append([]*Change{}, trail...), d)
		if d == start {
			return next
		}
		if !seen[d] {
			seen[d] = true
			if loop := depLoop(start, d, next, seen); loop != nil {
				return loop
			}
		}
	}
	return nil
}

func sortedCopy(s []string) []string {
	out := append([]string{}, s...)
	sort.Strings(out)
	return out
}

// checkQueues warns about queue.md entries that aren't active changes in that yass folder, and repeats.
func (r *Repo) checkQueues() {
	for _, root := range r.Roots {
		where := r.disp(filepath.Join(root.Dir, QueueName))
		active, archived, pieces := map[string]bool{}, map[string]bool{}, map[string]string{}
		for _, c := range r.Active {
			if c.Root == root {
				active[c.Name] = true
				for _, p := range c.Pieces {
					pieces[p.Name] = c.Name
				}
			}
		}
		for _, c := range r.Archived {
			if c.Root == root {
				archived[c.Name] = true
			}
		}
		seen := map[string]bool{}
		for _, name := range root.Queue {
			switch {
			case seen[name]:
				r.warn("%s: '%s' is listed more than once", where, name)
			case active[name]:
			case archived[name]:
				r.warn("%s: '%s' is archived; take it off the list", where, name)
			case pieces[name] != "":
				r.warn("%s: '%s' is a piece of %s; its plan.md orders its pieces", where, name, pieces[name])
			default:
				r.warn("%s: '%s' isn't a change in %s", where, name, r.disp(filepath.Join(root.Dir, "changes")))
			}
			seen[name] = true
		}
	}
}

// checkCode checks the code commits that boxes cite ("— code: a1b2c3d"): each must be a commit in
// this repo, and a box marked done must cite commits that are on the merged branch. Plans kept in
// another folder can't be committed with the code, so these links are how the two stay honest.
func (r *Repo) checkCode() {
	if _, ok := git(r.Top, "rev-parse", "--git-dir"); !ok {
		return
	}
	for _, c := range r.everything() {
		for _, b := range c.ownBoxes() {
			where, what := r.disp(filepath.Join(c.Path, b.file)), trunc(strings.TrimSpace(codeRefRE.Split(b.text, 2)[0]), 50)
			for _, sha := range codeRefs(b.text) {
				switch state, branch := r.citation(c.Root, sha); state {
				case citeMissing:
					r.warn("%s: '%s' cites %s, which isn't a commit here (fetch, or fix the ref)", where, what, sha)
				case citeUnmerged:
					if b.mark == markDone {
						r.warn("%s: '%s' is marked done, but %s isn't on %s yet (mark it [/] until it's merged, or cite the merged commit)",
							where, what, sha, branch)
					}
				}
			}
		}
	}
}

// archiveWarnings are a change's done boxes, its pieces' included, citing a commit that's missing
// here or not merged: yass archive says so as it archives.
func (r *Repo) archiveWarnings(c *Change) []string {
	if _, ok := git(r.Top, "rev-parse", "--git-dir"); !ok {
		return nil
	}
	var out []string
	for _, x := range flatten([]*Change{c}) {
		for _, b := range x.ownBoxes() {
			if b.mark != markDone {
				continue
			}
			what := trunc(strings.TrimSpace(codeRefRE.Split(b.text, 2)[0]), 50)
			where := filepath.ToSlash(filepath.Join(x.label(), b.file))
			for _, sha := range codeRefs(b.text) {
				switch state, branch := r.citation(c.Root, sha); state {
				case citeMissing:
					out = append(out, fmt.Sprintf("'%s' (%s) cites %s, which isn't a commit here", what, where, sha))
				case citeUnmerged:
					out = append(out, fmt.Sprintf("'%s' (%s) cites %s, which isn't on %s yet", what, where, sha, branch))
				}
			}
		}
	}
	return out
}

// What checking one citation found.
const (
	citeMerged   = iota // a commit here, on the merged branch (or no merged branch to compare with)
	citeUnmerged        // a commit here, not merged yet
	citeMissing         // not a commit here
)

// citation checks one cited commit against this repo; branch is the merged branch it compared with.
func (r *Repo) citation(root *Root, sha string) (state int, branch string) {
	if _, ok := git(r.Top, "rev-parse", "-q", "--verify", sha+"^{commit}"); !ok {
		return citeMissing, ""
	}
	if r.merged == nil {
		r.merged = map[*Root]string{}
	}
	branch, seen := r.merged[root]
	if !seen {
		branch = r.mergedBranch(root)
		r.merged[root] = branch
	}
	if branch == "" {
		return citeMerged, ""
	}
	if _, ok := git(r.Top, "merge-base", "--is-ancestor", sha, branch); !ok {
		return citeUnmerged, branch
	}
	return citeMerged, branch
}

// mergedBranch is the code branch that counts as merged: yass.yaml's branch, else origin/HEAD, main or master.
func (r *Repo) mergedBranch(root *Root) string {
	if root.Branch != "" {
		if _, ok := git(r.Top, "rev-parse", "-q", "--verify", root.Branch+"^{commit}"); !ok {
			r.warn("%s: branch '%s' isn't in this repo (fetch it, or fix `branch`)", r.disp(root.Config), root.Branch)
			return ""
		}
		return root.Branch
	}
	if out, ok := git(r.Top, "rev-parse", "--abbrev-ref", "origin/HEAD"); ok && strings.TrimSpace(out) != "origin/HEAD" {
		return strings.TrimSpace(out)
	}
	for _, b := range []string{"main", "master"} {
		if _, ok := git(r.Top, "rev-parse", "-q", "--verify", "refs/heads/"+b); ok {
			return b
		}
	}
	return ""
}

// everything is the active changes and their pieces.
func (r *Repo) everything() []*Change { return flatten(r.Active) }

// notCheckedOut is the yass folder (repo-relative) holding a change by that name that git knows
// but this checkout doesn't have on disk, as in a sparse checkout; "" if there's none. It asks git
// only the first time a name doesn't resolve.
func (r *Repo) notCheckedOut(name string) string {
	if r.absent == nil {
		r.absent = map[string]string{}
		out, _ := git(r.Top, "ls-files", "--cached", "-z", "--",
			":(glob)**/yass/changes/**/change.md", ":(glob)**/yass/archive/**/change.md")
		for _, f := range strings.Split(out, "\x00") {
			if f == "" || exists(filepath.Join(r.Top, filepath.FromSlash(f))) {
				continue
			}
			if m := yassDirRE.FindStringSubmatch(f); m != nil {
				r.absent[path.Base(path.Dir(f))] = path.Join(m[1], "yass") + "/"
			}
		}
	}
	return r.absent[name]
}

func (r *Repo) hasName(cs []*Change, name string) bool {
	for _, c := range cs {
		if c.Name == name {
			return true
		}
	}
	return false
}

func flatten(changes []*Change) []*Change {
	var out []*Change
	for _, c := range changes {
		out = append(out, c)
		out = append(out, c.Pieces...)
	}
	return out
}

// resolve finds the one change a name (or a unique part of one) means, among active changes and
// their pieces, or among archived ones when archived is set.
func (r *Repo) resolve(token string, pool []*Change, archived bool) (*Change, error) {
	kind, hint := "active", "yass status"
	if pool == nil {
		pool = r.everything()
		if archived {
			r.archivedPieces()
			pool, kind, hint = flatten(r.Archived), "archived", "yass status --archived"
		}
	}
	t := strings.TrimRight(token, "/")
	ap := canon(t)
	var exact []*Change
	for _, c := range pool {
		if c.Path == ap || c.Name == filepath.Base(t) || c.label() == t {
			exact = append(exact, c)
		}
	}
	if len(exact) == 1 {
		return exact[0], nil
	}
	hits := exact
	if len(hits) == 0 {
		for _, c := range pool {
			if strings.Contains(strings.ToLower(c.Name), strings.ToLower(t)) {
				hits = append(hits, c)
			}
		}
	}
	switch len(hits) {
	case 1:
		return hits[0], nil
	case 0:
		return nil, fmt.Errorf("no %s change matches '%s' (see `%s`)", kind, token, hint)
	}
	var names []string
	for i, c := range hits {
		if i == 6 {
			break
		}
		names = append(names, c.label())
	}
	return nil, fmt.Errorf("'%s' matches %d changes: %s", token, len(hits), strings.Join(names, ", "))
}

// rootForCwd is the yass folder serving the current folder: the one whose owner is nearest.
func (r *Repo) rootForCwd() *Root {
	var best *Root
	for _, root := range r.Roots {
		if within(r.Cwd, root.Owner) && (best == nil || len(root.Owner) > len(best.Owner)) {
			best = root
		}
	}
	return best
}


## internal/yass/repo_test.go

package yass

import (
	"reflect"
	"strings"
	"testing"
)

func TestByCreated(t *testing.T) {
	mk := func(name, created string) *Change {
		return &Change{Name: name, Meta: map[string]string{"created": created}}
	}
	cs := []*Change{ // as loaded: by name
		mk("2026-10-04-zebra", ""),
		mk("2026-10-05-apple", "2026-10-05T15:00:00Z"),
		mk("2026-10-05-bad-stamp", "yesterday"),
		mk("2026-10-05-mango", ""),
		mk("2026-10-05-zebra", "2026-10-05T10:00:00-04:00"),
		mk("no-date", ""),
	}
	byCreated(cs)
	var got []string
	for _, c := range cs {
		got = append(got, c.Name)
	}
	want := []string{"no-date", "2026-10-04-zebra", "2026-10-05-bad-stamp", "2026-10-05-mango", "2026-10-05-zebra", "2026-10-05-apple"}
	if !reflect.DeepEqual(got, want) {
		t.Fatalf("byCreated = %q, want %q", got, want)
	}
}

func TestChangeFolder(t *testing.T) {
	for _, tc := range []struct{ path, want string }{
		{"archive/2026-10-01-a/change.md", "archive/2026-10-01-a"},
		{"archive/2026/10/2026-10-01-a/change.md", "archive/2026/10/2026-10-01-a"},
		{"archive/2026/10/2026-10-01-a/2026-10-02-p/change.md", "archive/2026/10/2026-10-01-a"},
		{"changes/2026/10/x/change.md", "changes/2026"},
	} {
		if got := changeFolder(strings.Split(tc.path, "/")); got != tc.want {
			t.Errorf("changeFolder(%q) = %q, want %q", tc.path, got, tc.want)
		}
	}
}

func TestDeliveredIDs(t *testing.T) {
	for text, want := range map[string][]int{
		"Delivers AC1, AC3 in [plan.md](../plan.md)": {1, 3},
		"Delivers AC2–AC5":                           {2, 3, 4, 5},
		"Delivers AC2-AC4, AC9":                      {2, 3, 4, 9},
		"Delivers AC1, AC10, and AC19's preview card, in [plan.md](../plan.md)":      {1, 10, 19},
		"Delivers AC1, AC2 and AC3 of the plan.":                                     {1, 2, 3},
		"Delivers AC9–AC19, AC34 and AC35 in plan.md; the live checks are elsewhere": {9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 34, 35},
		"Builds AC1 but doesn't say delivers":                                        nil,
		"Fix AC2 (no delivers here)":                                                 nil,
	} {
		if got := deliveredIDs(text); !reflect.DeepEqual(got, want) {
			t.Errorf("deliveredIDs(%q) = %v, want %v", text, got, want)
		}
	}
}

func TestMarkDelivered(t *testing.T) {
	in := "## Acceptance\n<!-- e.g.\n- [ ] AC1 (R1) Given … -->\n- [ ] AC1 (R1) real\n- [/] AC2 (R1) going\n- [ ] AC3 (R2) not delivered\n- [-] AC4 dropped\r\n- [ ] AC5 crlf\r\nprose AC1\n"
	want := "## Acceptance\n<!-- e.g.\n- [ ] AC1 (R1) Given … -->\n- [x] AC1 (R1) real\n- [x] AC2 (R1) going\n- [ ] AC3 (R2) not delivered\n- [-] AC4 dropped\r\n- [x] AC5 crlf\r\nprose AC1\n"
	if got := markDelivered(in, map[int]bool{1: true, 2: true, 3: false, 4: true, 5: true}); got != want {
		t.Errorf("markDelivered =\n%q\nwant\n%q", got, want)
	}
}


## internal/yass/separate.go

package yass

import (
	"fmt"
	"strconv"
	"strings"
)

// Plans are separate when their yass folder isn't in the code repo's git work tree: in another
// repo, or in no repo at all (design §1). A linked worktree of the same repo counts as the same
// repo, so plans in the main checkout are inline from every worktree.

// gitCommonDir is the shared .git folder of the repo dir is in, or "" outside git.
func gitCommonDir(dir string) string {
	out, ok := git(dir, "rev-parse", "--path-format=absolute", "--git-common-dir")
	if !ok {
		return ""
	}
	return canon(strings.TrimSpace(out))
}

// separate says whether a yass folder's plans live outside the code repo. A folder inside this
// repo's work tree is inline without asking git, so this stays cheap with hundreds of team folders;
// only folders elsewhere, or private ones git never sees, are looked up.
func (r *Repo) separate(root *Root) bool {
	if r.codeDir == nil {
		d := gitCommonDir(r.Cwd)
		r.codeDir = &d
	}
	if *r.codeDir == "" {
		return false // no code repo to be separate from
	}
	if within(root.Dir, r.Top) && !root.Included {
		return false
	}
	return gitCommonDir(root.Dir) != *r.codeDir
}

// plansNotes notes, for each separate plans folder in git, uncommitted files there and how far it
// is behind its upstream as of the last fetch. It never fetches.
func (r *Repo) plansNotes() []string {
	var lines []string
	for _, root := range r.Roots {
		if !r.separate(root) {
			continue
		}
		lines = append(lines, fmt.Sprintf("plans: separate (%s)", root.Dir))
		if gitCommonDir(root.Dir) == "" {
			continue
		}
		if out, ok := git(root.Dir, "status", "--porcelain", "--", "."); ok {
			if n := len(strings.Split(strings.TrimSpace(out), "\n")); strings.TrimSpace(out) != "" {
				r.note("%s has %d uncommitted file(s); commit them there, where the plans are kept", root.Dir, n)
			}
		}
		if up, ok := git(root.Dir, "rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"); ok {
			up = strings.TrimSpace(up)
			if out, ok := git(root.Dir, "rev-list", "--count", "HEAD..@{u}"); ok {
				if n, _ := strconv.Atoi(strings.TrimSpace(out)); n > 0 {
					r.note("%s is %d commit(s) behind %s as of its last fetch; pull there (git -C %s pull)", root.Dir, n, up, root.Dir)
				}
			}
		}
	}
	return lines
}

// kind is how yass root -v describes a yass folder.
func (r *Repo) kind(root *Root) string {
	if r.separate(root) {
		return "separate"
	}
	return "inline"
}


## internal/yass/templates/agents.md

<!-- yass:begin (managed by `yass init`; edit outside these markers) -->
## Changes (YASS)
Planned work lives in the yass folder's `changes/`, one dated folder per change: `yass/` in the repo, or wherever `yass.yaml` points (`yass root` prints it; see its README.md). CLI: `yass`, on your PATH. Playbooks: the `yass-*` skills, in `.agents/skills/` or your user skills folder.
- `yass status` shows what's in flight. If `yass root` is outside this repo, plans and code are committed separately: see "Plans in another folder" in `yass-work`. Work only on the change you're pointed at (unless you've been put in charge of choosing), and resume it from the last **Next:** in its Log. `not started` means planned but not built yet; its plan may still need approval.
- As you work, mark boxes `[/]` when you start them and `[x]` when they're verified, and append to `## Log` and `## Decisions` alongside the code (each decision dated: `- <decision> - <why> (<who>, <YYYY-MM-DD>)`).
- Intent (`prd.md`, `design.md`, plan text, a Goal, acceptance criteria, including dropping one with `[-]`) never changes in the same commit as code (with squash merges, the same pull request), so decisions can be reviewed apart from execution. If the plan is wrong mid-build, put the fix on its own branch underneath the code and rebase onto it (see `yass-work`), or stop and say so.
- Don't commit, branch, rebase, push or open a pull request unless you've been given that latitude; otherwise show the human the git commands.
- When `yass status` notes that the `yass` binary and this repo's YASS files are out of step, or someone asks to update YASS: `yass-update`.
- A plan from plan mode can become a change: `yass-shape` for large work, `yass-plan` or `yass-work` for small.
- Never edit the archive (`archive/` in the yass folder); only `yass evict` removes from it. Follow-up work is a new change with `follows: <archived folder>`.
- Finished means every box is checked; then `yass archive <change>`, in its own commit, apart from code (with squash merges, a pull request without code).
<!-- yass:end -->


## internal/yass/templates/change-large.md

---
platforms: [{{platforms}}]
source: {{source}}
follows: {{follows}}
blocked:
created: {{created}}
---
# {{title}}

## Goal
<!-- Intent. One paragraph; the detail is in prd.md (why and what) and plan.md (how and acceptance). -->
{{goal}}

## Decisions
<!-- Progress. "- <decision> - <why> (<who>, <YYYY-MM-DD>)" for calls that span the whole change. -->

## Log
<!-- Progress. Change-level notes; each piece keeps its own Log.
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->


## internal/yass/templates/change.md

---
platforms: [{{platforms}}]
source: {{source}}
follows: {{follows}}
blocked:
created: {{created}}
---
# {{title}}

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
{{goal}}

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [ ] 

## Decisions
<!-- Progress. "- <decision> - <why> (<who>, <YYYY-MM-DD>)", appended as you go. -->

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->


## internal/yass/templates/design.md

# {{title}}: design
<!-- Intent: the calls that are hard to undo. Edit only in commits without code. -->

## Context
<!-- What forces a decision now. -->

## Options
<!-- At least two real ones, with trade-offs. -->

## Decision
<!-- What we chose, why, and who decided. -->

## Consequences and rollback


## internal/yass/templates/plan.md

# {{title}}: plan
<!-- Intent: how, and how we'll know it's done. With code, only mark boxes here ([/] in progress, [x] done);
     anything else is a plan revision in its own commit. -->

## Approach
<!-- The shape of the solution. Hard-to-undo calls go in design.md. -->

## Acceptance
<!-- Per milestone, one observable behavior each:
### M1
- [ ] AC1 (R1) Given …, when …, then … — verify: <test, flow, or manual steps> -->

## Pieces
<!-- PR-sized pieces, each its own folder in here: `yass new "<title>" --in <this change>`.
     `yass status` lists them; say here what order they go in and why. -->

## Validation
<!-- How the whole thing is verified before it's called done: suites, platforms, manual passes. -->


## internal/yass/templates/prd.md

# {{title}}: PRD
<!-- Intent: what should change or exist, and why. Not a description of the current system;
     the code and its docs are that. Edit this file only in commits without code. -->

## Why
<!-- The problem, with evidence. -->

## Users and outcomes
<!-- Who it's for, and measurable targets ("p50 time to log < 5s"). -->

## Requirements
<!-- One line each: "- **R1** [M1] A user can …". IDs are never reused. -->

## Non-goals
<!-- What this change will not do. Agents treat these as walls. -->

## Milestones
| Milestone | When it ships, a user can … |
|---|---|

## Open questions


## internal/yass/templates/readme.md

# yass/

Planned work for this part of the repo, managed with YASS (Yet Another Spec System). Code and its docs describe what exists today; this folder describes what's changing.

- `changes/`: active work, one dated folder per change. A small change is a `change.md`. A large one adds `prd.md` (why and what), `plan.md` (how and acceptance), an optional `design.md`, and PR-sized pieces as folders inside it.
- `archive/`: finished changes, moved here as they were, in `<YYYY>/<MM>/` folders by the month they were archived (older archives may sit directly in `archive/`). Never edited; follow-up work is a new change with `follows: <folder>`.
- `queue.md` (optional): the order to tackle changes in, top first, one folder name per list item. `yass status` follows it; changes it doesn't list come after, oldest first.

Rules:
1. **A change is a folder; ceremony scales with scope.**
2. **Progress travels with the code.** Ticking boxes and appending to Log and Decisions go in the same commits as the code.
3. **Intent changes get their own commit.** `prd.md`, `design.md`, `plan.md` beyond ticking boxes, and a change's title, Goal and Acceptance (including dropping a criterion).
4. **Done means every box is checked.** Then `yass archive <change>`, in its own commit.
5. **The archive is append-only,** except that `yass evict` moves its oldest months to git history when the folder is past its `keep:` setting.

`yass status` shows what's in flight.


## internal/yass/text.go

package yass

import (
	"embed"
	"os"
	"path/filepath"
	"regexp"
	"strings"
	"unicode/utf8"
)

//go:embed templates/*.md
var templates embed.FS

var (
	nameRE      = regexp.MustCompile(`^\d{4}-\d{2}-\d{2}-[a-z0-9][a-z0-9-]*$`)
	boxRE       = regexp.MustCompile(`^\s*[-*+]\s+\[([ xX/-])\]\s+(.*\S)\s*$`)
	commentRE   = regexp.MustCompile(`(?s)<!--.*?-->`)
	fmRE        = regexp.MustCompile(`(?s)^---\n(.*?)\n---\n?`)
	fmCommentRE = regexp.MustCompile(`\s+#(?:\s.*)?$`) // "# a comment", but not "#41" in "gh issue #41"
	titleRE     = regexp.MustCompile(`(?m)^# (.+)$`)
	h2RE        = regexp.MustCompile(`(?m)^## `)
	logEntryRE  = regexp.MustCompile(`(?m)^###\s`)
	nextRE      = regexp.MustCompile(`(?m)^\s*[-*]\s+\**Next:\**\s*(.+)$`)
	slugRE      = regexp.MustCompile(`[^a-z0-9]+`)
	agentsRE    = regexp.MustCompile(`(?s)<!-- yass:begin.*?<!-- yass:end -->`)
	codeRefRE   = regexp.MustCompile(`\s*(?:—|-{1,2})?\s*\bcode:\s*`)
	shaRE       = regexp.MustCompile(`^[0-9a-fA-F]{7,40}$`)
	queueRE     = regexp.MustCompile(`^\s*(?:[-*+]|\d+[.)])\s+(?:\[([^\]]+)\]\([^)]*\)|(\S+))`)
)

// QueueName is the optional file in a yass folder that ranks its changes, top first.
const QueueName = "queue.md"

// queueEntry is the change a queue.md line names: the first word of a list item ("- name",
// "1. name"), in backticks or as a link's text if it is one. Anything after it is a note.
func queueEntry(line string) (string, bool) {
	m := queueRE.FindStringSubmatch(line)
	if m == nil {
		return "", false
	}
	name := m[1] + m[2]
	name = strings.TrimRight(strings.Trim(name, "`"), "/")
	return name, name != ""
}

// parseQueue lists the changes a queue.md names, in order, duplicates included.
func parseQueue(text string) []string {
	var out []string
	for _, l := range lines(commentRE.ReplaceAllString(text, "")) {
		if name, ok := queueEntry(l); ok {
			out = append(out, name)
		}
	}
	return out
}

// Box marks: ' ' not started, '/' in progress, 'x' done, '-' dropped.
const (
	markOpen    = " "
	markWorking = "/"
	markDone    = "x"
	markDropped = "-"
)

func isOpen(mark string) bool { return mark == markOpen || mark == markWorking }

func read(path string) string {
	b, err := os.ReadFile(path)
	if err != nil {
		return ""
	}
	return string(b)
}

// normalize turns Windows line endings into \n, so the patterns below see one kind of line.
func normalize(text string) string { return strings.ReplaceAll(text, "\r\n", "\n") }

// readText reads a markdown file for parsing, with normalized line endings.
func readText(path string) string { return normalize(read(path)) }

func write(path, text string) error {
	if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
		return err
	}
	return os.WriteFile(path, []byte(text), 0o644)
}

func exists(path string) bool { _, err := os.Stat(path); return err == nil }

func isDir(path string) bool { st, err := os.Stat(path); return err == nil && st.IsDir() }

func isFile(path string) bool { st, err := os.Stat(path); return err == nil && !st.IsDir() }

func render(name string, kw map[string]string) string {
	b, _ := templates.ReadFile("templates/" + name)
	text := string(b)
	for k, v := range kw {
		text = strings.ReplaceAll(text, "{{"+k+"}}", v)
	}
	return text
}

func lines(text string) []string {
	return strings.Split(normalize(text), "\n")
}

// frontmatter returns the `key: value` pairs between the leading --- lines, and the rest of the text.
func frontmatter(text string) (map[string]string, string) {
	meta := map[string]string{}
	loc := fmRE.FindStringSubmatchIndex(text)
	if loc == nil {
		return meta, text
	}
	for _, line := range lines(text[loc[2]:loc[3]]) {
		k, v, ok := strings.Cut(line, ":")
		if ok && strings.TrimSpace(k) != "" {
			v = fmCommentRE.ReplaceAllString(v, "") // allow trailing "# comments"
			meta[strings.TrimSpace(k)] = strings.TrimSpace(strings.Trim(strings.TrimSpace(v), "[]"))
		}
	}
	return meta, text[loc[1]:]
}

// setMeta sets one frontmatter key, replacing its line if it has one, adding it at the end of the
// frontmatter if not, and adding frontmatter if there's none. The rest of the text is untouched.
func setMeta(text, key, value string) string {
	eol := "\n"
	if strings.Contains(text, "\r\n") {
		eol = "\r\n"
		text = strings.ReplaceAll(text, "\r\n", "\n")
	}
	line := key + ": " + value
	loc := fmRE.FindStringSubmatchIndex(text)
	switch {
	case loc == nil && strings.HasPrefix(text, "---\n---"): // empty frontmatter
		text = "---\n" + line + "\n" + text[len("---\n"):]
	case loc == nil:
		text = "---\n" + line + "\n---\n" + text
	default:
		fm := strings.Split(text[loc[2]:loc[3]], "\n")
		found := false
		for i, l := range fm {
			if k, _, ok := strings.Cut(l, ":"); ok && strings.TrimSpace(k) == key {
				fm[i], found = line, true
				break
			}
		}
		if !found {
			fm = append(fm, line)
		}
		text = text[:loc[2]] + strings.Join(fm, "\n") + text[loc[3]:]
	}
	return strings.ReplaceAll(text, "\n", eol)
}

// section returns the body of `## name`, up to the next `## ` heading.
func section(text, name string) string {
	re := regexp.MustCompile(`(?m)^## ` + regexp.QuoteMeta(name) + `[ \t]*\n`)
	loc := re.FindStringIndex(text)
	if loc == nil {
		return ""
	}
	rest := text[loc[1]:]
	if m := h2RE.FindStringIndex(rest); m != nil {
		return rest[:m[0]]
	}
	return rest
}

type box struct{ mark, text string }

// boxes returns every checkbox outside HTML comments.
func boxes(text string) []box {
	var out []box
	for _, line := range lines(commentRE.ReplaceAllString(text, "")) {
		if m := boxRE.FindStringSubmatch(line); m != nil {
			out = append(out, box{strings.ToLower(m[1]), m[2]})
		}
	}
	return out
}

// codeRefs are the commits a box cites at its end: "… — code: a1b2c3d" or "code: a1b2c3d, e4f5a6b".
// A token naming a repo (web@a1b2c3d) isn't a citation YASS reads.
func codeRefs(text string) []string {
	parts := codeRefRE.Split(text, -1)
	if len(parts) < 2 {
		return nil
	}
	var out []string
	for _, f := range strings.FieldsFunc(parts[len(parts)-1], func(r rune) bool { return r == ',' || r == ' ' }) {
		if shaRE.MatchString(f) {
			out = append(out, strings.ToLower(f))
		}
	}
	return out
}

func slug(text string, n int) string {
	s := strings.Trim(slugRE.ReplaceAllString(strings.ToLower(text), "-"), "-")
	if len(s) > n { // end on a whole word: cut at the last hyphen that fits (one right after n counts)
		if i := strings.LastIndex(s[:n+1], "-"); i > 0 {
			s = s[:i]
		} else {
			s = s[:n]
		}
	}
	if s = strings.TrimRight(s, "-"); s == "" {
		return "change"
	}
	return s
}

// trunc shortens s to at most n characters.
func trunc(s string, n int) string {
	if utf8.RuneCountInString(s) <= n {
		return s
	}
	return string([]rune(s)[:n])
}

// padRight pads s with spaces to n characters.
func padRight(s string, n int) string {
	if c := utf8.RuneCountInString(s); c < n {
		return s + strings.Repeat(" ", n-c)
	}
	return s
}


## internal/yass/text_test.go

package yass

import (
	"reflect"
	"strings"
	"testing"
)

func TestParseQueue(t *testing.T) {
	got := parseQueue("# Queue\nTop first.\n\n1. 2026-10-02-a — a note\n- `2026-09-14-b`\n* [2026-09-01-c](changes/2026-09-01-c/) - linked\n" +
		"10) 2026-08-01-d/\n<!-- - 2026-01-01-hidden -->\nprose with 2026-07-01-e in it\n  + 2026-10-02-a\n")
	want := []string{"2026-10-02-a", "2026-09-14-b", "2026-09-01-c", "2026-08-01-d", "2026-10-02-a"}
	if !reflect.DeepEqual(got, want) {
		t.Fatalf("parseQueue = %q, want %q", got, want)
	}
}

func TestBoxes(t *testing.T) {
	got := boxes("- [ ] a\n* [x] b\n+ [X] c\n- [/] d\n- [-] e\n- [ ] \n<!-- - [ ] hidden -->\n- [?] f\n")
	want := []box{{" ", "a"}, {"x", "b"}, {"x", "c"}, {"/", "d"}, {"-", "e"}}
	if !reflect.DeepEqual(got, want) {
		t.Fatalf("boxes = %v, want %v", got, want)
	}
}

func TestFrontmatter(t *testing.T) {
	meta, body := frontmatter("---\nplatforms: [ios, android]\nsource: gh#41\nblocked:    # set when stuck\n---\n# Title\n")
	want := map[string]string{"platforms": "ios, android", "source": "gh#41", "blocked": ""}
	if !reflect.DeepEqual(meta, want) || body != "# Title\n" {
		t.Fatalf("frontmatter = %v %q", meta, body)
	}
	if meta, _ := frontmatter("---\nsource: gh issue #41\nfollows: x   # a note\nblocked: #\n---\n"); meta["source"] != "gh issue #41" || meta["follows"] != "x" || meta["blocked"] != "" {
		t.Fatalf("comments = %v", meta)
	}
	if meta, body := frontmatter("---\n---\n# t\n"); len(meta) != 0 || body != "---\n---\n# t\n" {
		t.Fatalf("empty frontmatter = %v %q", meta, body)
	}
}

func TestSection(t *testing.T) {
	text := "# T\n## Goal\nship it\n\n## Acceptance\n- [ ] a\n## Log\n### 2026-10-02 (c)\n- Next: go\n"
	if got := section(text, "Goal"); got != "ship it\n\n" {
		t.Fatalf("Goal = %q", got)
	}
	if got := section(text, "Log"); got != "### 2026-10-02 (c)\n- Next: go\n" {
		t.Fatalf("Log = %q", got)
	}
	if got := section(text, "Steps"); got != "" {
		t.Fatalf("Steps = %q", got)
	}
}

func TestSlug(t *testing.T) {
	for in, want := range map[string]string{
		"Fix double-tap save":     "fix-double-tap-save",
		"Offline sync, web badge": "offline-sync-web-badge",
		"!!!":                     "change",
		"A very long title that keeps going past forty characters": "a-very-long-title-that-keeps-going-past",
		"Paths that don't exist yet compare through symlinks":      "paths-that-don-t-exist-yet-compare",
		"Large monorepos first, separate plans repo done right":    "large-monorepos-first-separate-plans",
		"Exactly forty characters long title here x":               "exactly-forty-characters-long-title-here",
		strings.Repeat("a", 45):                                    strings.Repeat("a", 40),
	} {
		if got := slug(in, 40); got != want {
			t.Errorf("slug(%q) = %q, want %q", in, got, want)
		}
	}
}

func TestHookWhere(t *testing.T) {
	h := &hook{prefixes: []string{"planning"}}
	for p, want := range map[string]place{
		"yass/changes/2026-01-01-x/prd.md":        {"changes", "prd.md", "yass/changes/2026-01-01-x"},
		"svc/yass/archive/2026-01-01-x/change.md": {"archive", "change.md", "svc/yass/archive/2026-01-01-x"},
		"planning/changes/2026-01-01-x/plan.md":   {"changes", "plan.md", "planning/changes/2026-01-01-x"},
		"planning/README.md":                      {kind: "meta"},
		"yass/README.md":                          {kind: "meta"},
		"yass.yaml":                               {kind: "meta"},
		"src/app.go":                              {},
	} {
		if got := h.where(p); got != want {
			t.Errorf("where(%q) = %+v, want %+v", p, got, want)
		}
	}
}

func TestNormIgnoresProgressMarks(t *testing.T) {
	a := norm("- [ ] AC1 x\n- [/] AC2 y\n")
	b := norm("- [x] AC1 x\n- [X] AC2 y\n")
	if !reflect.DeepEqual(a, b) {
		t.Fatalf("marking boxes changed intent: %v vs %v", a, b)
	}
	if reflect.DeepEqual(norm("- [ ] AC1 x\n"), norm("- [-] AC1 x\n")) {
		t.Fatal("dropping a box should count as an intent change")
	}
}

func TestCodeRefs(t *testing.T) {
	for in, want := range map[string][]string{
		"Disable Save — code: a1b2c3d":                          {"a1b2c3d"},
		"Queue table - code: A1B2C3D, e4f5a6b7c":                {"a1b2c3d", "e4f5a6b7c"},
		"Given x, then y — verify: unit code: 1234567":          {"1234567"},
		"Explain the code: it's fine":                           nil,
		"No refs here":                                          nil,
		"A repo-qualified token isn't read — code: web@a1b2c3d": nil,
		"Mixed — code: a1b2c3d, web@1234567":                    {"a1b2c3d"},
	} {
		if got := codeRefs(in); !reflect.DeepEqual(got, want) {
			t.Errorf("codeRefs(%q) = %v, want %v", in, got, want)
		}
	}
}

func TestSetMeta(t *testing.T) {
	for _, tc := range []struct{ in, want string }{
		{"---\nsource: x\narchived: old\nblocked:\n---\n# T\n", "---\nsource: x\narchived: NOW\nblocked:\n---\n# T\n"},
		{"---\nsource: x\n---\n# T\n- [x] archived: no\n", "---\nsource: x\narchived: NOW\n---\n# T\n- [x] archived: no\n"},
		{"---\r\nsource: x\r\n---\r\n# T\r\n", "---\r\nsource: x\r\narchived: NOW\r\n---\r\n# T\r\n"},
		{"---\n---\n# T\n", "---\narchived: NOW\n---\n# T\n"},
		{"# T\n", "---\narchived: NOW\n---\n# T\n"},
	} {
		if got := setMeta(tc.in, "archived", "NOW"); got != tc.want {
			t.Errorf("setMeta(%q) = %q, want %q", tc.in, got, tc.want)
		}
	}
}


## internal/yass/trailers.go

package yass

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"sort"
	"strings"
)

// With plans kept outside the code repo, no commit can carry progress next to the code, so every
// code commit names its change in a `yass:` trailer instead (design §2):
//
//	yass: <target> [<key>:<value> ...]
//	target := none | <change> | <change>/<piece> | <piece>
//	key    := ac (the criteria the commit works toward, comma-separated)
//
// The prepare-commit-msg hook adds it, the commit-msg hook and `yass hook --range` check it.

// trailer is one parsed `yass:` trailer.
type trailer struct {
	raw, target string
	ac          []string
	unknown     []string // keys this version doesn't know: a warning, not an error
}

// parseTrailer reads a trailer's value. It's case-insensitive; YASS writes lower case.
func parseTrailer(value string) (trailer, error) {
	t := trailer{raw: strings.TrimSpace(value)}
	f := strings.Fields(strings.ToLower(t.raw))
	if len(f) == 0 {
		return t, fmt.Errorf("it names nothing")
	}
	t.target = f[0]
	if strings.Contains(t.target, ":") {
		return t, fmt.Errorf("it starts with %q, not a change", f[0])
	}
	for _, kv := range f[1:] {
		k, v, ok := strings.Cut(kv, ":")
		if !ok || k == "" || v == "" || strings.HasPrefix(v, ",") || strings.HasSuffix(v, ",") || strings.Contains(v, ",,") {
			return t, fmt.Errorf("%q isn't key:value", kv)
		}
		if k == "ac" {
			t.ac = append(t.ac, strings.Split(v, ",")...)
		} else {
			t.unknown = append(t.unknown, k)
		}
	}
	return t, nil
}

// usesSeparatePlans: a yass folder this repo uses is outside it, or its yass.yaml points outside
// it to a folder that isn't here (CI without the plans checked out). Only then can a commit's
// change live elsewhere, so only then do the trailer hooks do anything.
func (r *Repo) usesSeparatePlans() bool {
	if gitCommonDir(r.Top) == "" {
		return false
	}
	for _, root := range r.Roots {
		if r.separate(root) {
			return true
		}
	}
	c, _, err := loadConfig(filepath.Join(r.Top, ConfigName))
	if err != nil || strings.TrimSpace(c.Path) == "" {
		return false
	}
	dir, err := resolvePath(strings.TrimSpace(c.Path), r.Top)
	return err == nil && !within(dir, r.Top) && !isDir(dir)
}

// targetSeparate says whether a trailer's target (a label, maybe followed by keys) names a change
// kept outside this repo; none counts, since it's a deliberate trailer.
func (r *Repo) targetSeparate(target string) bool {
	label := strings.ToLower(strings.Fields(target + " x")[0])
	if label == "none" {
		return true
	}
	for _, c := range append(r.everything(), flatten(r.Archived)...) {
		if strings.ToLower(c.label()) == label {
			return r.separate(c.Root)
		}
	}
	return false
}

// targets maps every name a trailer can use, lower-cased, to the change it means: change folders
// (active, archived, evicted), <change>/<piece>, and a piece's own name when it's unambiguous.
func (r *Repo) targets() (named map[string]string, ambiguous map[string]bool) {
	named, ambiguous = map[string]string{}, map[string]bool{}
	r.archivedPieces()
	all := append(r.everything(), flatten(r.Archived)...)
	for _, c := range all {
		if c.Parent == nil {
			named[strings.ToLower(c.Name)] = strings.ToLower(c.Name)
			continue
		}
		label := strings.ToLower(c.label())
		named[label] = label
		n := strings.ToLower(c.Name)
		if prev, ok := named[n]; ok && prev != label {
			ambiguous[n] = true
		}
		named[n] = label
	}
	for n := range ambiguous {
		delete(named, n)
	}
	return named, ambiguous
}

// checkTarget says what's wrong with a trailer's target, if anything; names are nil when the
// plans aren't here to check against.
func checkTarget(t trailer, named map[string]string, ambiguous map[string]bool) string {
	if t.target == "none" || named == nil {
		return ""
	}
	if _, ok := named[t.target]; ok {
		return ""
	}
	if ambiguous[t.target] {
		return fmt.Sprintf("'%s' is the name of pieces in more than one change; give it as <change>/<piece>", t.target)
	}
	msg := fmt.Sprintf("'%s' isn't a change or piece", t.target)
	if near := closest(t.target, named, 3); len(near) > 0 {
		msg += "; did you mean " + strings.Join(near, ", ") + "?"
	}
	return msg
}

// closest are up to n names nearest to s by edit distance.
func closest(s string, named map[string]string, n int) []string {
	type cand struct {
		name string
		d    int
	}
	var cs []cand
	for k := range named {
		if d := editDistance(s, k); d <= max(3, len(s)/2) {
			cs = append(cs, cand{k, d})
		}
	}
	sort.Slice(cs, func(i, j int) bool { return cs[i].d < cs[j].d || cs[i].d == cs[j].d && cs[i].name < cs[j].name })
	var out []string
	for i := 0; i < len(cs) && i < n; i++ {
		out = append(out, cs[i].name)
	}
	return out
}

func editDistance(a, b string) int {
	prev := make([]int, len(b)+1)
	for j := range prev {
		prev[j] = j
	}
	for i := 1; i <= len(a); i++ {
		cur := make([]int, len(b)+1)
		cur[0] = i
		for j := 1; j <= len(b); j++ {
			cost := 1
			if a[i-1] == b[j-1] {
				cost = 0
			}
			cur[j] = min(prev[j]+1, cur[j-1]+1, prev[j-1]+cost)
		}
		prev = cur
	}
	return prev[len(b)]
}

// messageTrailers are the `yass:` trailers in a commit message, comment lines left out.
func messageTrailers(msg string) []string {
	var keep []string
	for _, l := range strings.Split(normalize(msg), "\n") {
		if !strings.HasPrefix(l, "#") {
			keep = append(keep, l)
		}
	}
	cmd := exec.Command("git", "interpret-trailers", "--parse")
	cmd.Stdin = strings.NewReader(strings.Join(keep, "\n"))
	out, _ := cmd.Output()
	var vals []string
	for _, l := range strings.Split(string(out), "\n") {
		if k, v, ok := strings.Cut(l, ":"); ok && strings.EqualFold(strings.TrimSpace(k), "yass") {
			vals = append(vals, strings.TrimSpace(v))
		}
	}
	return vals
}

// trailerProblems checks a message's trailers: malformed ones and ones naming no change, and, when
// asked (CI's --require-trailers), a message with none. Notes are only ever notes.
func trailerProblems(msg string, named map[string]string, ambiguous map[string]bool, requireOne bool) (problems, notes []string) {
	vals := messageTrailers(msg)
	if len(vals) == 0 {
		if requireOne {
			return []string{"no `yass:` trailer naming its change (`yass use <change>` adds one to every commit here; `yass: none` for a commit that belongs to no change)"}, nil
		}
		return nil, nil
	}
	for _, v := range vals {
		t, err := parseTrailer(v)
		if err != nil {
			problems = append(problems, fmt.Sprintf("`yass: %s` is malformed: %v", v, err))
			continue
		}
		if p := checkTarget(t, named, ambiguous); p != "" {
			problems = append(problems, fmt.Sprintf("`yass: %s`: %s", v, p))
		}
		for _, k := range t.unknown {
			notes = append(notes, fmt.Sprintf("`yass: %s`: this yass doesn't know the key '%s'", v, k))
		}
	}
	return problems, notes
}

// useFile is where `yass use` keeps this worktree's change: in its own git folder, never committed.
func useFile() string {
	out, ok := git("", "rev-parse", "--path-format=absolute", "--git-dir")
	if !ok {
		return ""
	}
	return filepath.Join(strings.TrimSpace(out), "yass-use")
}

// inferTarget works out a commit's trailer: `yass use`, else the branch named after a change or
// piece (exactly, or after a prefix like jaime/), else an earlier commit's trailer on the branch.
func (r *Repo) inferTarget(named map[string]string) string {
	if f := useFile(); f != "" {
		if v := strings.TrimSpace(read(f)); v != "" {
			return v
		}
	}
	if b, ok := git("", "symbolic-ref", "--short", "-q", "HEAD"); ok {
		b = strings.ToLower(strings.TrimSpace(b))
		for _, cand := range []string{b, b[strings.LastIndex(b, "/")+1:]} {
			if label, ok := named[cand]; ok {
				return label
			}
		}
	}
	root := &Root{}
	if len(r.Roots) > 0 {
		root = r.Roots[0]
	}
	if merged := r.mergedBranch(root); merged != "" {
		out, _ := git("", "log", "--format=%(trailers:key=yass,valueonly,separator=%x00)", merged+"..HEAD")
		for _, l := range strings.Split(out, "\n") {
			if v := strings.TrimSpace(strings.Split(l, "\x00")[0]); v != "" {
				return strings.ToLower(v)
			}
		}
	}
	return ""
}

// prepareMsg is the prepare-commit-msg hook: when the change this commit is for lives outside the
// repo, add its trailer, unless there's one already or it's a merge. It never fails a commit.
func prepareMsg(file, source string) {
	if source == "merge" {
		return
	}
	if _, merging := git("", "rev-parse", "-q", "--verify", "MERGE_HEAD"); merging {
		return
	}
	if !findRepo().usesSeparatePlans() || len(messageTrailers(read(file))) > 0 {
		return // cheap checks first: most repos never read their changes here
	}
	r := loadRepo()
	var named map[string]string
	if len(r.Roots) > 0 {
		named, _ = r.targets()
	}
	if t := r.inferTarget(named); t != "" && r.targetSeparate(t) {
		git("", "interpret-trailers", "--in-place", "--trailer", "yass: "+t, file)
	}
}

// commitMsg is the commit-msg hook: point out a trailer that's malformed or names no change. It
// never asks for a missing one and never refuses a commit; CI can require them (--require-trailers).
func commitMsg(file string) int {
	if _, merging := git("", "rev-parse", "-q", "--verify", "MERGE_HEAD"); merging {
		return 0
	}
	if !findRepo().usesSeparatePlans() || len(messageTrailers(read(file))) == 0 {
		return 0
	}
	r := loadRepo()
	var named map[string]string
	var ambiguous map[string]bool
	if len(r.Roots) > 0 {
		named, ambiguous = r.targets()
	}
	problems, notes := trailerProblems(read(file), named, ambiguous, false)
	for _, n := range notes {
		fmt.Fprintf(os.Stderr, "yass: %s\n", n)
	}
	if len(problems) > 0 {
		fmt.Fprintln(os.Stderr, "yass: heads-up: this commit's `yass:` trailer doesn't name a change")
		for _, p := range problems {
			fmt.Fprintf(os.Stderr, "  - %s\n", p)
		}
	}
	return 0
}

// rangeTrailers checks the trailers of every non-merge commit in a range, for CI. names is false
// when the plans aren't here, so only the grammar is checked.
func (r *Repo) rangeTrailers(shas []string, requireOne bool) (found []string, names bool) {
	var named map[string]string
	var ambiguous map[string]bool
	if len(r.Roots) > 0 {
		named, ambiguous = r.targets()
	}
	for _, sha := range shas {
		msg, _ := git("", "log", "-1", "--format=%B", sha)
		problems, _ := trailerProblems(msg, named, ambiguous, requireOne)
		if len(problems) == 0 {
			continue
		}
		subj, _ := git("", "log", "-1", "--format=%h %s", sha)
		for _, p := range problems {
			found = append(found, strings.TrimSpace(subj)+": "+p)
		}
	}
	return found, named != nil
}

// cmdUse sets, shows or clears the change this worktree's commits name.
func cmdUse(a *args) error {
	f := useFile()
	if f == "" {
		return fmt.Errorf("yass use works inside a git repo")
	}
	if a.b["clear"] {
		os.Remove(f)
		fmt.Println("yass use: cleared; commits here get their change from the branch name or earlier commits")
		return nil
	}
	if len(a.pos) == 0 {
		if v := strings.TrimSpace(read(f)); v != "" {
			fmt.Println(v)
		} else {
			fmt.Println("none set (yass use <change>)")
		}
		return nil
	}
	r := loadRepo()
	c, err := r.resolve(a.pos[0], nil, false)
	if err != nil {
		return err
	}
	v := strings.ToLower(c.label())
	if ac := strings.ToLower(strings.ReplaceAll(a.v["ac"], " ", "")); ac != "" {
		v += " ac:" + ac
	}
	if err := write(f, v+"\n"); err != nil {
		return err
	}
	fmt.Printf("yass use: commits in this worktree now carry `yass: %s`\n", v)
	if !r.separate(c.Root) {
		fmt.Printf("(%s's plans are in this repo, so its commits don't carry a trailer)\n", c.Name)
	}
	return nil
}

// trailerCommit is a merged commit whose trailer names a change.
type trailerCommit struct{ sha, subject string }

// changeCode is a change's merged code, found by trailers (R16, R21): the commits naming it or one
// of its pieces, those commits by the criteria their ac: names, and warnings for an ac: naming a
// criterion the plan doesn't have. One git log over the merged branch, filtered by git.
func (r *Repo) changeCode(c *Change) (commits []trailerCommit, byAC map[string][]trailerCommit, branch string, warnings []string) {
	if !r.separate(c.Root) {
		return nil, nil, "", nil
	}
	if branch = r.mergedBranch(c.Root); branch == "" {
		return nil, nil, "", nil
	}
	named, _ := r.targets()
	me := strings.ToLower(c.label())
	top := c
	if c.Parent != nil {
		top = c.Parent
	}
	criteria := map[string]bool{}
	for _, b := range top.ownBoxes() {
		if m := criterionRE.FindStringSubmatch(b.text); m != nil && b.file == "plan.md" {
			criteria["AC"+m[1]] = true
		}
	}
	out, _ := git(r.Top, "log", branch, "-i", "--grep=^yass:", "--format=%h%x1f%s%x1f%(trailers:key=yass,valueonly,separator=%x1e)%x00")
	byAC = map[string][]trailerCommit{}
	for _, rec := range strings.Split(out, "\x00") {
		f := strings.SplitN(strings.TrimLeft(rec, "\n"), "\x1f", 3)
		if len(f) < 3 {
			continue
		}
		tc := trailerCommit{f[0], f[1]}
		matched := false
		for _, v := range strings.Split(strings.TrimSpace(f[2]), "\x1e") {
			t, err := parseTrailer(v)
			if err != nil {
				continue
			}
			label := named[t.target]
			if label != me && !(c.Parent == nil && strings.HasPrefix(label, me+"/")) {
				continue
			}
			matched = true
			for _, ac := range t.ac {
				id := strings.ToUpper(ac)
				if len(criteria) > 0 && !criteria[id] {
					warnings = append(warnings, fmt.Sprintf("%s's trailer names %s, which %s's plan doesn't have", tc.sha, id, top.Name))
					continue
				}
				byAC[id] = append(byAC[id], tc)
			}
		}
		if matched {
			commits = append(commits, tc)
		}
	}
	return commits, byAC, branch, warnings
}


## internal/yass/trailers_test.go

package yass

import (
	"reflect"
	"strings"
	"testing"
)

func TestParseTrailer(t *testing.T) {
	for _, tc := range []struct {
		in, target  string
		ac, unknown []string
		bad         bool
	}{
		{in: "2026-02-01-login", target: "2026-02-01-login"},
		{in: "  2026-02-01-LOGIN/2026-02-02-Form ", target: "2026-02-01-login/2026-02-02-form"},
		{in: "none", target: "none"},
		{in: "login ac:AC1,ac3", target: "login", ac: []string{"ac1", "ac3"}},
		{in: "login ac:ac1 ac:ac2", target: "login", ac: []string{"ac1", "ac2"}},
		{in: "login foo:bar", target: "login", unknown: []string{"foo"}},
		{in: "", bad: true},
		{in: "ac:ac1", bad: true},
		{in: "login ac", bad: true},
		{in: "login ac:", bad: true},
		{in: "login ac:ac1,", bad: true},
		{in: "login ac:ac1,,ac2", bad: true},
	} {
		got, err := parseTrailer(tc.in)
		if (err != nil) != tc.bad {
			t.Errorf("parseTrailer(%q): err = %v, want bad=%v", tc.in, err, tc.bad)
			continue
		}
		if tc.bad {
			continue
		}
		if got.target != tc.target || !reflect.DeepEqual(got.ac, tc.ac) || !reflect.DeepEqual(got.unknown, tc.unknown) {
			t.Errorf("parseTrailer(%q) = %+v", tc.in, got)
		}
	}
}

func TestCheckTarget(t *testing.T) {
	named := map[string]string{"2026-02-01-login": "2026-02-01-login", "2026-02-01-login/2026-02-03-api": "2026-02-01-login/2026-02-03-api", "2026-02-03-api": "2026-02-01-login/2026-02-03-api"}
	amb := map[string]bool{"2026-02-02-form": true}
	for target, want := range map[string]string{
		"2026-02-01-login": "", "2026-02-03-api": "", "none": "",
		"2026-02-02-form": "more than one change",
		"2026-02-01-logn": "did you mean 2026-02-01-login",
	} {
		got := checkTarget(trailer{target: target}, named, amb)
		if want == "" && got != "" || want != "" && !strings.Contains(got, want) {
			t.Errorf("checkTarget(%q) = %q, want %q", target, got, want)
		}
	}
	if got := checkTarget(trailer{target: "anything"}, nil, nil); got != "" {
		t.Errorf("without the plans, names aren't checked; got %q", got)
	}
}


## internal/yass/update.go

package yass

import (
	"fmt"
	"net/http"
	"os"
	"path"
	"runtime"
	"strings"
	"time"

	"golang.org/x/mod/module"
	"golang.org/x/mod/semver"
)

// yass update: get a newer yass binary. It decides what to install from the binary's version, the
// version the repo's YASS files are stamped with, and the latest release (design §1–2).

// releasesURL is where releases come from. YASS_RELEASES_URL is a seam for tests (design §5), not
// a supported way to use mirrors.
func releasesURL() string {
	if u := os.Getenv("YASS_RELEASES_URL"); u != "" {
		return strings.TrimRight(u, "/")
	}
	return "https://github.com/donjaime/yass/releases"
}

// assetName is the release archive for a platform, as .goreleaser.yaml names it.
func assetName(goos, goarch string) string {
	if goos == "windows" {
		return fmt.Sprintf("yass_%s_%s.zip", goos, goarch)
	}
	return fmt.Sprintf("yass_%s_%s.tar.gz", goos, goarch)
}

var httpClient = &http.Client{
	Timeout:       15 * time.Second,
	CheckRedirect: func(*http.Request, []*http.Request) error { return http.ErrUseLastResponse },
}

// latestRelease reads the latest release's version from where <releases>/latest redirects to
// (…/releases/tag/vX.Y.Z), so it needs no API and no token (design §2).
func latestRelease() (string, error) {
	resp, err := httpClient.Head(releasesURL() + "/latest")
	if err != nil {
		return "", fmt.Errorf("couldn't reach %s: %v", releasesURL(), err)
	}
	resp.Body.Close()
	loc := resp.Header.Get("Location")
	if resp.StatusCode/100 != 3 || loc == "" {
		return "", fmt.Errorf("couldn't find the latest release at %s/latest (HTTP %d)", releasesURL(), resp.StatusCode)
	}
	v := semverOf(path.Base(loc))
	if v == "" {
		return "", fmt.Errorf("the latest release at %s isn't a version", loc)
	}
	return v, nil
}

// updatePlan is what yass update would do, and why.
type updatePlan struct {
	bin, repo, latest string // semver with a v; repo is "" outside a repo that uses YASS
	target            string // what it would install
	why               string // "to match this repo", "the latest release", …
}

func planUpdate(a *args, bin, repo, latest string) (updatePlan, error) {
	p := updatePlan{bin: bin, repo: repo, latest: latest}
	switch {
	case a.v["version"] != "":
		p.target, p.why = semverOf(a.v["version"]), "as asked"
		if p.target == "" {
			return p, fmt.Errorf("--version takes a release like v0.4.0")
		}
	case a.b["latest"]:
		p.target, p.why = latest, "the latest release"
	case repo != "" && (bin == "" || semver.Compare(repo, bin) > 0):
		p.target, p.why = repo, "to match this repo's YASS files"
	default:
		p.target, p.why = latest, "the latest release"
	}
	if bin != "" && semver.Compare(p.target, bin) < 0 {
		return p, fmt.Errorf("%s is older than this yass (%s); yass update never downgrades", p.target, bin)
	}
	return p, nil
}

func plain(v string) string { return strings.TrimPrefix(v, "v") }

// sourceBuild says how to update a binary that wasn't built by the release workflow: a version
// that's a release tag came from go install; anything else (a pseudo-version, +dirty, dev) from a clone.
func sourceBuild(target string) string {
	if v := semverOf(binVersion); v != "" && !module.IsPseudoVersion(v) && semver.Build(v) == "" {
		return fmt.Sprintf("it was built with go install; update it the same way: go install github.com/donjaime/yass/cmd/yass@%s", target)
	}
	return fmt.Sprintf("it was built from a clone; update it there: git pull (or git checkout %s), then go build -o bin/yass ./cmd/yass", target)
}

func cmdUpdate(a *args) (int, error) {
	bin := semverOf(binVersion)
	repo := repoVersion(findRepo())
	latest, err := latestRelease()
	if err != nil {
		return 0, err
	}
	p, err := planUpdate(a, bin, repo, latest)
	if err != nil {
		return 0, err
	}
	kind := "release build"
	if binChannel != "release" {
		kind = "built from source"
	}
	fmt.Printf("yass %s (%s, %s/%s)\n", binVersion, kind, runtime.GOOS, runtime.GOARCH)
	if repo != "" {
		fmt.Printf("this repo's YASS files: %s\n", plain(repo))
	}
	fmt.Printf("latest release: %s\n", plain(latest))
	upToDate := bin != "" && semver.Compare(p.target, bin) == 0
	installed := false
	switch {
	case upToDate:
		fmt.Printf("yass is up to date (%s)\n", plain(bin))
	case binChannel != "release":
		fmt.Printf("yass update won't replace this binary: %s\n", sourceBuild(p.target))
	case a.b["check"]:
		fmt.Printf("yass update would install %s (%s)\n", plain(p.target), p.why)
	default:
		fmt.Printf("installing %s (%s) …\n", plain(p.target), p.why)
		how, err := install(p.target, a.b["require-provenance"])
		if err != nil {
			return 0, err
		}
		fmt.Printf("installed yass %s (%s)\n", plain(p.target), how)
		installed = true
	}
	if semver.Compare(latest, p.target) > 0 && !a.b["latest"] && a.v["version"] == "" {
		fmt.Printf("%s is newer: `yass update --latest` gets it; then run `yass upgrade` in each repo and commit the diff (chore: upgrade YASS)\n", plain(latest))
	}
	if installed && repo != "" && semver.Compare(p.target, repo) > 0 {
		fmt.Printf("next: this repo's YASS files are older (%s); run `yass upgrade` here and commit the diff (chore: upgrade YASS), and teammates then need yass %s too (`yass update`)\n",
			plain(repo), plain(p.target))
	}
	return 0, nil
}


## internal/yass/update_test.go

package yass

import (
	"strings"
	"testing"
)

func TestAssetName(t *testing.T) {
	for _, tc := range []struct{ goos, goarch, want string }{
		{"darwin", "arm64", "yass_darwin_arm64.tar.gz"},
		{"darwin", "amd64", "yass_darwin_amd64.tar.gz"},
		{"linux", "amd64", "yass_linux_amd64.tar.gz"},
		{"linux", "arm64", "yass_linux_arm64.tar.gz"},
		{"windows", "amd64", "yass_windows_amd64.zip"},
		{"windows", "arm64", "yass_windows_arm64.zip"},
	} {
		if got := assetName(tc.goos, tc.goarch); got != tc.want {
			t.Errorf("assetName(%s, %s) = %s, want %s", tc.goos, tc.goarch, got, tc.want)
		}
	}
}

func TestPlanUpdate(t *testing.T) {
	mk := func(latest bool, version string) *args {
		return &args{b: map[string]bool{"latest": latest}, v: map[string]string{"version": version}}
	}
	for _, tc := range []struct {
		name, bin, repo, latest string
		a                       *args
		want, err               string
	}{
		{"repo ahead: match it", "v0.4.0", "v0.5.0", "v0.6.0", mk(false, ""), "v0.5.0", ""},
		{"repo behind: latest", "v0.5.0", "v0.4.0", "v0.6.0", mk(false, ""), "v0.6.0", ""},
		{"no repo: latest", "v0.4.0", "", "v0.6.0", mk(false, ""), "v0.6.0", ""},
		{"--latest over the repo", "v0.4.0", "v0.5.0", "v0.6.0", mk(true, ""), "v0.6.0", ""},
		{"--version", "v0.4.0", "v0.5.0", "v0.6.0", mk(false, "0.5.0"), "v0.5.0", ""},
		{"never downgrades", "v0.5.0", "", "v0.6.0", mk(false, "v0.4.0"), "", "never downgrades"},
		{"--version must be a version", "v0.4.0", "", "v0.6.0", mk(false, "newest"), "", "takes a release"},
	} {
		p, err := planUpdate(tc.a, tc.bin, tc.repo, tc.latest)
		switch {
		case tc.err != "" && (err == nil || !strings.Contains(err.Error(), tc.err)):
			t.Errorf("%s: err = %v, want %q", tc.name, err, tc.err)
		case tc.err == "" && (err != nil || p.target != tc.want):
			t.Errorf("%s: target = %s, %v; want %s", tc.name, p.target, err, tc.want)
		}
	}
}

func TestSourceBuild(t *testing.T) {
	defer func(v string) { binVersion = v }(binVersion)
	for v, want := range map[string]string{
		"v0.4.0":                               "go install github.com/donjaime/yass/cmd/yass@v0.5.0",
		"v0.3.1-0.20261008201306-a4857ed4ea9a": "built from a clone",
		"v0.3.1-0.20261008201306-a4857ed4ea9a+dirty": "built from a clone",
		"dev": "built from a clone",
	} {
		binVersion = v
		if got := sourceBuild("v0.5.0"); !strings.Contains(got, want) {
			t.Errorf("sourceBuild with %s = %q, want it to say %q", v, got, want)
		}
	}
}


## internal/yass/upgrade.go

package yass

import (
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"sort"
	"strings"

	"golang.org/x/mod/semver"
)

// What yass upgrade does with one file, given its stamp and the binary's version (design §1).
const (
	actWrite = "write" // older, unstamped, or the same version from a +dirty binary
	actKeep  = "keep"  // already at the binary's version
	actNewer = "newer" // written by a newer yass: upgrade the binary instead
)

func upgradeAction(stamp, bin string) string {
	b, s := semverOf(bin), semverOf(stamp)
	switch {
	case s == "":
		return actWrite
	case semver.Compare(s, b) < 0:
		return actWrite
	case semver.Compare(s, b) > 0:
		return actNewer
	case strings.HasSuffix(bin, "+dirty"):
		return actWrite
	}
	return actKeep
}

// noVersion explains why a binary without a version can't write stamped files.
func noVersion() error {
	return fmt.Errorf("this yass has no version (%s), so it can't stamp YASS's files; build it from a git clone"+
		" with Go 1.24 or later (go build -o bin/yass ./cmd/yass), or install a release", binVersion)
}

var (
	repoSkillRE = regexp.MustCompile(`(^|/)\.(agents|claude)/skills/yass-[^/]+/SKILL\.md$`)
	repoAgentRE = regexp.MustCompile(`(^|/)AGENTS\.md$`)
)

// upgradeTarget is one file yass upgrade looks after.
type upgradeTarget struct {
	path, kind string // kind: skill, agents or hook
	outside    bool   // not in the repo, so not under version control (user-folder playbooks)
}

// findYassFiles lists every YASS file in the repo, from the root down (tracked or not, minus what
// .gitignore and yass.yaml's ignore: leave out), plus the user folder's playbooks (design §6). A
// skills folder holding any yass-* playbook gets every playbook the binary carries, so an upgrade
// also adds playbooks a newer version introduces.
func findYassFiles(r *Repo) []upgradeTarget {
	out, _ := git(r.Top, "ls-files", "--cached", "--others", "--exclude-standard", "-z")
	skillDirs := map[string]bool{}
	var found []upgradeTarget
	for _, rel := range strings.Split(out, "\x00") {
		if rel == "" {
			continue
		}
		p := filepath.Join(r.Top, filepath.FromSlash(rel))
		if r.ignored(p) || !exists(p) {
			continue
		}
		switch {
		case repoSkillRE.MatchString(rel):
			skillDirs[filepath.Dir(filepath.Dir(p))] = false
		case repoAgentRE.MatchString(rel) && strings.Contains(read(p), "<!-- yass:begin"):
			found = append(found, upgradeTarget{path: p, kind: "agents"})
		case rel == HookPath+"/pre-commit":
			// the folder holds YASS's hooks: upgrade them all, adding any a newer version ships
			for _, name := range hookScripts {
				found = append(found, upgradeTarget{path: filepath.Join(filepath.Dir(p), name), kind: "hook"})
			}
		}
	}
	for _, d := range []string{userSkillsDir(), userClaudeSkillsDir()} {
		if m, _ := filepath.Glob(filepath.Join(d, "yass-*", "SKILL.md")); len(m) > 0 {
			skillDirs[d] = true
		}
	}
	names, _ := kitSkills()
	var dirs []string
	for d := range skillDirs {
		dirs = append(dirs, d)
	}
	sort.Strings(dirs)
	for _, d := range dirs {
		for _, n := range names {
			found = append(found, upgradeTarget{path: filepath.Join(d, n, "SKILL.md"), kind: "skill", outside: skillDirs[d]})
		}
	}
	sort.SliceStable(found, func(i, j int) bool { return found[i].path < found[j].path })
	return found
}

// newText is what yass upgrade writes for a target: the binary's own version of it, stamped.
func newText(t upgradeTarget) string {
	v := stampVersion()
	switch t.kind {
	case "agents":
		block := stampAgents(strings.TrimSpace(render("agents.md", nil)), v)
		return agentsRE.ReplaceAllLiteralString(read(t.path), block)
	case "hook":
		b, _ := kitFile(HookPath + "/" + filepath.Base(t.path))
		return stampHook(b, v)
	}
	_, text := kitSkills()
	return stampSkill(text[filepath.Base(filepath.Dir(t.path))], v)
}

// cmdUpgrade writes YASS's files at this binary's version, then brings each yass folder's
// layout up to date (migrate.go).
func cmdUpgrade(a *args) (int, error) {
	code, err := upgradeFiles()
	if err != nil || code != 0 {
		return code, err
	}
	r := findRepo()
	return 0, migrate(r, migrations)
}

func upgradeFiles() (int, error) {
	if semverOf(binVersion) == "" {
		return 0, noVersion()
	}
	r := findRepo()
	if _, ok := git(r.Cwd, "rev-parse", "--show-toplevel"); !ok {
		return 0, fmt.Errorf("yass upgrade works inside a git repo: it finds YASS's files from the repo root down")
	}
	targets := findYassFiles(r)
	if len(targets) == 0 {
		fmt.Println("no YASS files here to upgrade; `yass init` sets a repo up")
		return 0, nil
	}
	var write []upgradeTarget
	from := map[string]bool{}
	for _, t := range targets {
		stamp := ""
		if exists(t.path) {
			stamp = readStamp(t.path)
		}
		switch upgradeAction(stamp, binVersion) {
		case actNewer:
			fmt.Fprintf(os.Stderr, "yass: %s was written by yass %s, newer than this one (%s); `yass update` gets that version, then run yass upgrade\n",
				r.disp(t.path), stamp, binVersion)
			return 1, nil
		case actWrite:
			write = append(write, t)
			if !exists(t.path) {
				from["(new)"] = true
			} else if stamp == "" {
				from["unstamped (v0.2 or earlier)"] = true
			} else {
				from[stamp] = true
			}
		}
	}
	if len(write) == 0 {
		fmt.Printf("already up to date: YASS's files are at %s\n", stampVersion())
		return 0, nil
	}
	for _, t := range write {
		if err := write1(t); err != nil {
			return 0, err
		}
		fmt.Printf("wrote %s%s\n", r.disp(t.path), outsideNote(t.outside))
	}
	var olds []string
	for v := range from {
		if v != "(new)" {
			olds = append(olds, v)
		}
	}
	sort.Strings(olds)
	if len(olds) > 0 {
		fmt.Printf("upgraded YASS's files from %s to %s; review the diff and commit it (chore: upgrade YASS)\n", strings.Join(olds, ", "), stampVersion())
	} else {
		fmt.Printf("added YASS's files at %s; review the diff and commit it (chore: upgrade YASS)\n", stampVersion())
	}
	return 0, nil
}

func write1(t upgradeTarget) error {
	if err := write(t.path, newText(t)); err != nil {
		return err
	}
	if t.kind == "hook" {
		return os.Chmod(t.path, 0o755)
	}
	return nil
}

func outsideNote(outside bool) string {
	if outside {
		return " (outside version control)"
	}
	return ""
}

// stampedFiles are the YASS files at the repo root that carry a version stamp: the AGENTS.md
// section, the playbooks in .agents/ and .claude/, the hook; and, with user, the user folder's
// playbooks. Only the root, so it stays fast in large repos.
func stampedFiles(r *Repo, user bool) []string {
	var paths []string
	if ap := filepath.Join(r.Top, "AGENTS.md"); strings.Contains(read(ap), "<!-- yass:begin") {
		paths = append(paths, ap)
	}
	dirs := []string{filepath.Join(r.Top, ".agents", "skills"), filepath.Join(r.Top, ".claude", "skills")}
	if user {
		dirs = append(dirs, userSkillsDir(), userClaudeSkillsDir())
	}
	for _, d := range dirs {
		m, _ := filepath.Glob(filepath.Join(d, "yass-*", "SKILL.md"))
		paths = append(paths, m...)
	}
	if hp := filepath.Join(r.Top, filepath.FromSlash(HookPath), "pre-commit"); exists(hp) {
		paths = append(paths, hp)
	}
	return paths
}

// repoVersion is the newest version stamped in the repo's own YASS files, or "" outside a repo
// that uses YASS.
func repoVersion(r *Repo) string {
	newest := ""
	for _, p := range stampedFiles(r, false) {
		if v := semverOf(readStamp(p)); v != "" && (newest == "" || semver.Compare(v, newest) > 0) {
			newest = v
		}
	}
	return newest
}

// versionNote is yass status's one line about versions: whether this binary is older than the
// version that wrote the repo's YASS files (upgrade the binary), or newer (yass upgrade would
// upgrade them). It only reads the files at the repo root and the user folder's playbooks, so
// status stays fast in large repos; yass upgrade is what searches the whole repo.
func versionNote(r *Repo) string {
	if semverOf(binVersion) == "" {
		return ""
	}
	paths := stampedFiles(r, true)
	newest, older := "", false
	for _, p := range paths {
		stamp := readStamp(p)
		switch upgradeAction(stamp, binVersion) {
		case actNewer:
			if newest == "" || semver.Compare(semverOf(stamp), semverOf(newest)) > 0 {
				newest = stamp
			}
		case actWrite:
			if !strings.HasSuffix(binVersion, "+dirty") || semver.Compare(semverOf(stamp), semverOf(binVersion)) != 0 {
				older = true
			}
		}
	}
	switch {
	case newest != "":
		return fmt.Sprintf("YASS's files here were written by yass %s, newer than this one (%s); `yass update` gets that version", newest, binVersion)
	case older:
		return fmt.Sprintf("YASS's files here are older than this yass (%s); `yass upgrade` would upgrade them", binVersion)
	}
	return ""
}


## internal/yass/worktree.go

package yass

import (
	"os"
	"path/filepath"
	"regexp"
	"strings"
)

// worktrees lists the clone's checkouts, the main one first (a bare repo has none of its own), and
// says whether dir is in a linked worktree rather than the main checkout.
func worktrees(dir string) (all []string, linked bool) {
	out, ok := git(dir, "worktree", "list", "--porcelain")
	if !ok {
		return nil, false
	}
	for _, rec := range strings.Split(strings.TrimSpace(out), "\n\n") {
		ls := strings.Split(rec, "\n")
		p, ok := strings.CutPrefix(ls[0], "worktree ")
		if ok && !contains(ls[1:], "bare") {
			all = append(all, canon(filepath.FromSlash(p)))
		}
	}
	gd, ok1 := git(dir, "rev-parse", "--path-format=absolute", "--git-dir")
	cd, ok2 := git(dir, "rev-parse", "--path-format=absolute", "--git-common-dir")
	linked = ok1 && ok2 && canon(strings.TrimSpace(gd)) != canon(strings.TrimSpace(cd))
	return all, linked
}

// fromOtherCheckouts resolves a yass.yaml's path the way the clone's other checkouts see it. A
// relative path like ../plans means "next to the checkout", and a linked worktree usually lives
// somewhere else (~/worktrees/x, .claude/worktrees/x), so from there it points at nothing. It
// returns the first folder that exists, every place it looked, and whether this is a linked worktree.
func fromOtherCheckouts(file, path string) (found string, looked []string, linked bool) {
	owner := filepath.Dir(file)
	out, ok := git(owner, "rev-parse", "--show-toplevel")
	if !ok {
		return "", nil, false
	}
	top := canon(filepath.FromSlash(strings.TrimSpace(out)))
	rel, err := filepath.Rel(top, owner)
	if err != nil {
		return "", nil, false
	}
	all, linked := worktrees(owner)
	for _, t := range all {
		if t == top {
			continue
		}
		d, err := resolvePath(path, filepath.Join(t, rel))
		if err != nil || contains(looked, d) {
			continue
		}
		looked = append(looked, d)
		if isDir(d) {
			return d, looked, linked
		}
	}
	return "", looked, linked
}

// borrowConfig covers a yass.yaml kept out of git (a private pointer): a linked worktree doesn't
// get one, so it uses the one the clone's other checkouts have at the same place, at the top or
// above the current folder.
func (r *Repo) borrowConfig() {
	all, linked := worktrees(r.Top)
	if !linked {
		return
	}
	for d := r.Cwd; within(d, r.Top); d = filepath.Dir(d) {
		rel, _ := filepath.Rel(r.Top, d)
		for _, t := range all {
			f := filepath.Join(t, rel, ConfigName)
			if t == r.Top || !isFile(f) {
				continue
			}
			dir, cfg, _, err := configDir(f)
			if err != nil || !isDir(dir) {
				continue
			}
			r.Roots = append(r.Roots, &Root{Dir: canon(dir), Owner: d, Config: f, Branch: strings.TrimSpace(cfg.Branch)})
			return
		}
		if d == r.Top {
			break
		}
	}
}

// includes adds the yass.yaml of each team folder named in the clone's git config (yass.include):
// folders git never sees, like a private/ kept out of the repo, so git's file list can't find them.
// Git config is shared by every worktree of the clone; a worktree without the folder reads the
// yass.yaml from another checkout, and gets its own copy of the folder (for the link) later.
func (r *Repo) includes(configs map[string]bool) (included map[string]bool, borrowed []*Root) {
	included = map[string]bool{}
	out, ok := git(r.Top, "config", "--get-all", "yass.include")
	if !ok {
		return included, nil
	}
	var others []string
	for _, rel := range lines(strings.TrimSpace(out)) {
		rel = strings.Trim(filepath.Clean(filepath.FromSlash(strings.TrimSpace(rel))), string(filepath.Separator))
		if rel == "" || rel == "." {
			continue
		}
		owner := filepath.Join(r.Top, rel)
		if f := filepath.Join(owner, ConfigName); isFile(f) {
			configs[f], included[f] = true, true
			continue
		}
		if others == nil {
			others, _ = worktrees(r.Top)
		}
		found := false
		for _, t := range others {
			f := filepath.Join(t, rel, ConfigName)
			if t == r.Top || !isFile(f) {
				continue
			}
			dir, cfg, _, err := configDir(f)
			if err != nil || !isDir(dir) {
				continue
			}
			borrowed = append(borrowed, &Root{Dir: canon(dir), Owner: owner, Config: f, Branch: strings.TrimSpace(cfg.Branch), Included: true})
			found = true
			break
		}
		if !found {
			r.warn("git config yass.include: '%s' has no %s in any checkout of this clone (to stop including it: git config --unset yass.include '^%s$')",
				filepath.ToSlash(rel), ConfigName, regexp.QuoteMeta(filepath.ToSlash(rel)))
		}
	}
	return included, borrowed
}

// link keeps a yass/ symlink next to each yass.yaml whose folder is outside the repo, so people,
// editors and agents find the plans at yass/ as they would anywhere else. It's ignored through the
// clone's info/exclude, which every worktree shares and nobody commits. Nothing depends on it:
// yass.yaml stays the source of truth, and a real yass/ already there is left alone.
func (r *Repo) link() {
	for _, root := range r.Roots {
		ln := filepath.Join(root.Owner, "yass")
		if root.Config == "" || within(root.Dir, r.Top) {
			// Plans moved into the repo: a link left from before points somewhere else.
			if t, err := os.Readlink(ln); err == nil && root.Config != "" && t != root.Dir && !within(t, r.Top) {
				r.warn("%s/ is a link to %s, but %s points to %s; remove the link", r.disp(ln), t, r.disp(root.Config), r.disp(root.Dir))
			}
			continue
		}
		if fi, err := os.Lstat(ln); err == nil {
			if fi.Mode()&os.ModeSymlink == 0 {
				// A yass/ with changes in it is already reported as ignored; say it once.
				if !isRealYassDir(ln) {
					r.warn("%s/: already there, so it isn't a link to %s", r.disp(ln), root.Dir)
				}
				continue
			}
			if isSymlinkTo(ln, root.Dir) {
				r.exclude(ln)
				continue
			}
			if err := os.Remove(ln); err != nil {
				r.note("couldn't repoint %s/ to %s: %v", r.disp(ln), root.Dir, err)
				continue
			}
		}
		if root.Included { // a worktree's own copy of the team folder, which git never creates
			os.MkdirAll(root.Owner, 0o755)
		}
		if err := os.Symlink(root.Dir, ln); err != nil {
			r.note("couldn't link %s/ to %s (%v); `yass root` prints where the plans are", r.disp(ln), root.Dir, err)
			continue
		}
		r.exclude(ln)
	}
}

// isRealYassDir is a yass folder on disk that isn't a link.
func isRealYassDir(p string) bool {
	fi, err := os.Lstat(p)
	return err == nil && fi.IsDir() && (isDir(filepath.Join(p, "changes")) || isDir(filepath.Join(p, "archive")))
}

func isSymlinkTo(p, target string) bool {
	t, err := os.Readlink(p)
	return err == nil && t == target
}

// exclude adds a path to the clone's info/exclude, once.
func (r *Repo) exclude(p string) {
	out, ok := git(r.Top, "rev-parse", "--path-format=absolute", "--git-path", "info/exclude")
	if !ok {
		return
	}
	f := filepath.FromSlash(strings.TrimSpace(out))
	rel, err := filepath.Rel(r.Top, p)
	if err != nil {
		return
	}
	entry := "/" + filepath.ToSlash(rel)
	cur := readText(f)
	if contains(lines(cur), entry) {
		return
	}
	if cur != "" && !strings.HasSuffix(cur, "\n") {
		cur += "\n"
	}
	if err := write(f, cur+entry+"\n"); err != nil {
		r.note("couldn't add %s to %s: %v", entry, f, err)
	}
}


## internal/yass/worktree_test.go

package yass

import (
	"os"
	"path/filepath"
	"testing"
)

func TestLinkSkipsWhereSymlinksFail(t *testing.T) {
	if os.Geteuid() == 0 {
		t.Skip("root can write anywhere")
	}
	top, plans := t.TempDir(), t.TempDir()
	if err := os.Chmod(top, 0o555); err != nil {
		t.Fatal(err)
	}
	defer os.Chmod(top, 0o755)
	r := &Repo{Top: canon(top), Roots: []*Root{{Dir: canon(plans), Owner: canon(top), Config: filepath.Join(canon(top), ConfigName)}}}
	r.link()
	if _, err := os.Lstat(filepath.Join(top, "yass")); err == nil {
		t.Errorf("made a link in a read-only folder")
	}
	if len(r.Notes) != 1 || len(r.Warnings) != 0 {
		t.Errorf("notes %v, warnings %v; want one note and no warnings", r.Notes, r.Warnings)
	}
}


## kit/embed.go

// Package kit carries the files YASS installs into a repo: the playbooks and the hook script.
// Every yass binary embeds them, so `yass init` can set up a repo from the binary alone.
package kit

import "embed"

// FS holds .agents/skills/yass-*/SKILL.md and tools/yass/githooks/pre-commit, unstamped;
// `yass init` adds the version as it writes them.
//
//go:embed all:.agents tools
var FS embed.FS


## site/assets/README.md

# Landing page assets

Every image the page uses lives here, at a fixed path, so new graphics replace files without touching `index.html`. The emblem and icons come from the YASS logo (transparent background); the social card is the YASS brand graphic; `favicon.svg` is a simplified vector of the hex. A placeholder carries the marker `yass-placeholder` (an SVG comment, a PNG `tEXt` chunk, or WebP XMP metadata), and `tests/site.sh --deploy`, which the Pages workflow runs before it publishes, fails while any file still has it. Real graphics shouldn't contain the marker.

| File | Required | Spec | Where it shows |
|---|---|---|---|
| `emblem.webp` | yes | square, at least 460×460, ideally 600×600, transparent background; the hex emblem trimmed close to its edges (the page draws it at up to 240 px) | top of the hero, above the YASS! wordmark (which is HTML text, not an image) |
| `favicon.svg` | yes | square; reads at 16 px (the hex emblem alone, without the robot's detail, reads best) | browser tab |
| `favicon-32.png` | yes | 32×32 | browser tab, for browsers without SVG favicons |
| `apple-touch-icon.png` | yes | 180×180, opaque (no transparency) | home-screen icon on iOS |
| `og.png` | yes | 1200×630; keep text inside the central 1000×500, since some apps crop | the preview card when someone shares a link to the page |

`tests/site.sh` checks that the required files exist, that the page references them, and that the images have the sizes above. Everything the page loads, except `og.png`, has to stay under 150 KB in total, so prefer SVG and compress PNGs.


## tests/bench.sh

#!/usr/bin/env bash
# Speed at scale: generates large repos in a temporary folder, times `yass status` and the
# pre-commit hook against the targets in the plan, and cleans up. Exits 1 if a target is missed.
# Usage: tests/bench.sh [workdir]   (BENCH_KEEP=1 keeps the generated repos)
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
W="${1:-$(mktemp -d)}"; mkdir -p "$W"
[ -n "${BENCH_KEEP:-}" ] || trap 'rm -rf "$W/bench"' EXIT
rm -rf "$W/bench"; mkdir -p "$W/bench"
unset YASS_STRICT YASS_HOOK YASS_HOME
if [ -z "${YASS_BIN:-}" ]; then
  mkdir -p "$W/bin"; (cd "$ROOT" && go build -o "$W/bin/yass" ./cmd/yass) || { echo "build failed"; exit 1; }
  YASS_BIN="$W/bin/yass"
fi
FOLDERS="${BENCH_FOLDERS:-200}" CHANGES="${BENCH_CHANGES:-2000}" ARCHIVED="${BENCH_ARCHIVED:-10000}" BRANCHES="${BENCH_BRANCHES:-100}" REMOTES="${BENCH_REMOTES:-1000}" COMMITS="${BENCH_COMMITS:-50000}" FILES="${BENCH_FILES:-100000}" RUNS="${BENCH_RUNS:-5}"
fail=0

# median wall time of RUNS runs of a command, in milliseconds, after one untimed warm-up run
# (freshly generated files are slow to read at first: the OS is still caching and indexing them)
ms() { perl -MTime::HiRes=time -e '
  my $n = shift; my @t;
  open my $out, ">&", \*STDOUT; open STDOUT, ">", "/dev/null"; open STDERR, ">", "/dev/null";
  system(@ARGV);
  for (1..$n) {
    my $s = time; my $rc = system(@ARGV);   # exit 1 is a finding (a warning), not a failure
    if ($rc == -1 || ($rc >> 8) > 1) { print $out "failed: @ARGV\n"; exit 1 }
    push @t, (time - $s) * 1000;
  }
  @t = sort { $a <=> $b } @t; printf $out "%d\n", $t[int($n / 2)]' "$RUNS" "$@"; }
check() { # name, measured ms, target ms
  if [ "$2" -lt "$3" ]; then echo "  ok   $1: ${2}ms (target < ${3}ms)"; else echo "  MISS $1: ${2}ms (target < ${3}ms)"; fail=1; fi; }
newrepo() { mkdir -p "$1"; cd "$1"; git init -q -b main; git config user.email b@b; git config user.name Bench; }

echo "status: $FOLDERS yass folders, $CHANGES changes, $ARCHIVED archived"
newrepo "$W/bench/status"
FOLDERS=$FOLDERS CHANGES=$CHANGES perl -e '
  my ($f, $c) = ($ENV{FOLDERS}, $ENV{CHANGES}); my $per = int($c / $f) || 1; my $n = 0;
  for my $i (1..$f) {
    my $y = sprintf("teams/t%03d/yass", $i); system("mkdir", "-p", "$y/archive");
    for my $j (1..$per) {
      last if $n++ >= $c;
      my $d = sprintf("%s/changes/2026-%02d-%02d-change-%d", $y, ($j % 12) + 1, ($j % 28) + 1, $j);
      system("mkdir", "-p", $d); open my $o, ">", "$d/change.md" or die;
      print $o "---\nplatforms: [all]\nsource:\nfollows:\nblocked:\n---\n# Change $i.$j\n\n## Goal\nDo the thing.\n\n## Acceptance\n";
      print $o "- [", ($_ % 3 ? " " : "x"), "] Given $_, when $_, then $_ - verify: test $_\n" for 1..8;
      print $o "\n## Steps\n", map("- [/] step $_\n", 1..4), "\n## Decisions\n- chose a - because b (bench)\n\n## Log\n### 2026-10-01 (bench)\n- Did: things\n- Next: more things\n";
    }
  }'
# Archived changes, spread over the folders and over months, as yass archive files them; some
# active changes follow one, so names are resolved across the archive.
FOLDERS=$FOLDERS ARCHIVED=$ARCHIVED perl -MFile::Path=make_path -e '
  my ($f, $c) = ($ENV{FOLDERS}, $ENV{ARCHIVED}); my $per = int($c / $f) || 1; my $n = 0;
  for my $i (1..$f) {
    my $y = sprintf("teams/t%03d/yass", $i);
    for my $j (1..$per) {
      last if $n++ >= $c;
      my $m = ($j % 12) + 1;
      my $d = sprintf("%s/archive/2025/%02d/2025-%02d-%02d-done-%d", $y, $m, $m, ($j % 28) + 1, $j);
      make_path($d); open my $o, ">", "$d/change.md" or die;
      printf $o "---\nplatforms: [all]\nsource:\nfollows:\nblocked:\narchived: 2025-%02d-28T12:00:00Z\n---\n# Done %d.%d\n\n## Goal\nDid the thing.\n\n## Acceptance\n", $m, $i, $j;
      print $o "- [x] Given $_, when $_, then $_ - verify: test $_\n" for 1..8;
      print $o "\n## Decisions\n- chose a - because b (bench)\n\n## Log\n### 2025-01-01 (bench)\n- Did: things\n";
    }
    my $first = sprintf("%s/changes/2026-02-02-change-1/change.md", $y);
    if (-f $first) { local @ARGV = ($first); local $^I = ""; while (<>) { s/^follows:[ \t]*$/follows: 2025-02-02-done-1/; print } }
  }'
mkdir -p yass/changes yass/archive; touch yass/changes/.gitkeep
git add -A >/dev/null; git commit -q -m "bench: plans"
echo "  $(git ls-files | wc -l | tr -d ' ') files, $("$YASS_BIN" status | grep -c ' not started\| in progress\|  [0-9]*/[0-9]*') status lines"
# branches <prefix> <count>: that many branches off main, each with a commit to one change's
# change.md dated now, written by one git fast-import (so 1,000 take a second, not minutes)
branches() { PREFIX=$1 COUNT=$2 FOLDERS=$FOLDERS perl -e '
  my ($p, $n, $f, $t) = ($ENV{PREFIX}, $ENV{COUNT}, $ENV{FOLDERS}, time);
  for my $i (1..$n) {
    my $file = sprintf("teams/t%03d/yass/changes/2026-02-02-change-1/change.md", ($i % $f) + 1);
    my $body = "# Change\n\n- [x] Step one\n- [/] Step two\n\n## Log\n### 2026-10-01 (bench)\n- Next: branch $i\n";
    printf "commit %s%d\ncommitter Bench <b\@b> %d +0000\ndata 7\nbranch\nfrom refs/heads/main^0\nM 100644 inline %s\ndata %d\n%s\n", $p, $i, $t, $file, length($body), $body;
  }' | git fast-import --quiet; }
check "yass status" "$(ms "$YASS_BIN" status)" 1000
branches refs/heads/local- "$BRANCHES"
echo "  + $BRANCHES local branches with recent work"
check "yass status, with $BRANCHES branches" "$(ms "$YASS_BIN" status)" 1000
branches refs/remotes/origin/remote- "$REMOTES"
echo "  + $REMOTES remote branches with recent work"
check "yass status, with $REMOTES remote branches more" "$(ms "$YASS_BIN" status)" 2000

echo "trailers: a code repo with $COMMITS commits, its plans kept apart"
newrepo "$W/bench/trailers"; printf 'path: %s\n' "$W/bench/status/teams/t001/yass" > yass.yaml
git add -A; git commit -q -m "chore: adopt YASS" -m "yass: none"
COMMITS=$COMMITS perl -e '
  my ($n, $t) = ($ENV{COMMITS}, time - $ENV{COMMITS});
  for my $i (1..$n) {
    my $msg = "feat: change $i\n" . ($i % 10 == 0 ? "\nyass: 2026-02-02-change-1 ac:ac1\n" : "");
    printf "commit refs/heads/main\ncommitter Bench <b\@b> %d +0000\ndata %d\n%s", $t + $i, length($msg), $msg;
    print "from refs/heads/main^0\n" if $i == 1;
    printf "M 100644 inline src/f%d.txt\ndata 2\n%d\n", $i % 500, $i % 10;
  }' | git fast-import --quiet; git reset -q --hard
echo "  $(git rev-list --count HEAD) commits, $(git log --oneline -i --grep='^yass:' | wc -l | tr -d ' ') with trailers"
check "yass status" "$(ms "$YASS_BIN" status)" 1000
check "yass status <change> (its code from trailers)" "$(ms "$YASS_BIN" status 2026-02-02-change-1)" 1000

echo "hook: $FILES files, one-file commit"
newrepo "$W/bench/hook"
FILES=$FILES perl -e '
  my $n = $ENV{FILES}; my $per = 500;
  for my $i (0..int(($n - 1) / $per)) {
    my $d = sprintf("src/m%04d", $i); system("mkdir", "-p", $d);
    for my $j (1..$per) { last if $i * $per + $j > $n; open my $o, ">", "$d/f$j.go" or die; print $o "package m\n" }
  }'
for t in a b c; do mkdir -p "teams/$t/yass/changes/2026-10-01-x"; printf -- '# X\n\n- [ ] box\n' > "teams/$t/yass/changes/2026-10-01-x/change.md"; done
git add -A >/dev/null; git commit -q -m "bench: a big repo"
echo "edit" >> src/m0000/f1.go; echo "- [x] box" >> teams/a/yass/changes/2026-10-01-x/change.md; git add -A
echo "  $(git ls-files | wc -l | tr -d ' ') files"
check "yass hook (staged, one code file and one plan)" "$(ms "$YASS_BIN" hook)" 200

echo; [ "$fail" -eq 0 ] && echo "all targets met" || echo "some targets missed"
exit $fail


## tests/e2e.sh

#!/usr/bin/env bash
# End-to-end test: installs YASS into throwaway repos and walks the lifecycle, including the optional hook.
# Usage: tests/e2e.sh [workdir]
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
W="${1:-$(mktemp -d)}"; rm -rf "$W/e2e"; mkdir -p "$W/e2e"
unset YASS_STRICT YASS_HOOK YASS_HOME
# Build the CLI (or use $YASS_BIN) and put it first on PATH, where the hook and install.sh look for it.
if [ -z "${YASS_BIN:-}" ]; then
  mkdir -p "$W/bin"; (cd "$ROOT" && go build -o "$W/bin/yass" ./cmd/yass) || { echo "build failed"; exit 1; }
  YASS_BIN="$W/bin/yass"
fi
export YASS_BIN; export PATH="$(dirname "$YASS_BIN"):$PATH"
pass=0; fail=0
ok()  { echo "  ok   $1"; pass=$((pass+1)); }
bad() { echo "  FAIL $1"; fail=$((fail+1)); }
run_ok()   { local d="$1"; shift; local o; if o="$("$@" 2>&1)"; then ok "$d"; else bad "$d"; sed 's/^/       /' <<<"$o" | head -20; fi; }
run_fail() { local d="$1"; shift; local o; if o="$("$@" 2>&1)"; then bad "$d (should have failed)"; sed 's/^/       /' <<<"$o" | head -20; else ok "$d"; fi; }
has()   { local d="$1" p="$2"; shift 2; local o; o="$("$@" 2>&1)"; if grep -qE -- "$p" <<<"$o"; then ok "$d"; else bad "$d (no /$p/)"; sed 's/^/       /' <<<"$o" | head -25; fi; }
hasnt() { local d="$1" p="$2"; shift 2; local o; o="$("$@" 2>&1)"; if grep -qE -- "$p" <<<"$o"; then bad "$d (found /$p/)"; sed 's/^/       /' <<<"$o" | head -25; else ok "$d"; fi; }
y() { yass "$@"; }
# portable in-place regex edit (GNU and BSD): sub FILE PATTERN REPLACEMENT [COUNT]
sub() { P="$2" R="$3" N="${4:-0}" perl -0777 -i -pe '
  BEGIN { ($p, $r, $n) = ($ENV{P}, $ENV{R}, $ENV{N}); $r =~ s/\\n/\n/g }
  if ($n) { my $c = 0; s/$p/$c++ < $n ? $r : $&/gme } else { s/$p/$r/gm }' "$1"; }
newrepo() { rm -rf "$1"; mkdir -p "$1"; cd "$1"; git init -q -b main; git config user.email t@t; git config user.name Sam
  echo app > app.txt; git add -A; git commit -q -m "existing app"; }
TODAY=$(date +%F); MONTH=$(date -u +%Y/%m)   # archives go in archive/<YYYY>/<MM>/, by UTC month

newrepo "$W/e2e/solo"
INSTALL_OUT="$(y init)"

echo "1. install"
has   "a first install suggests the adopting commit" "chore: adopt YASS" echo "$INSTALL_OUT"
for f in tools/yass/githooks/pre-commit yass/README.md yass/changes/.gitkeep \
         yass/archive/.gitkeep AGENTS.md .agents/skills/yass-work/SKILL.md .agents/skills/yass-shape/SKILL.md \
         .agents/skills/yass-plan/SKILL.md .agents/skills/yass-status/SKILL.md .agents/skills/yass-log/SKILL.md \
         .agents/skills/yass-update/SKILL.md; do
  [ -e "$f" ] && ok "has $f" || bad "missing $f"
done
has "the yass folder README describes the monthly archive" "archive/.*<YYYY>/<MM>/" cat yass/README.md
[ ! -e .claude ] && ok "no Claude files without --claude" || bad ".claude written without --claude"
[ ! -e yass/config.yml ] && ok "no config file" || bad "a config file appeared"
has   "AGENTS.md has the YASS section" "yass:begin" cat AGENTS.md
hasnt "the hook is off by default" "githooks" git config core.hooksPath
has   "status on an empty repo" "no active changes" y
has   "init is idempotent" "already set up" y init
git add -A; run_ok "adopting commit" git commit -q -m "chore: adopt YASS"
echo "# My rules" > AGENTS.md; y init >/dev/null
has   "init keeps your AGENTS.md content" "# My rules" cat AGENTS.md
has   "…and adds its section once" "^1$" bash -c 'grep -c "yass:begin" AGENTS.md'
y init >/dev/null
has   "…even when run again" "^1$" bash -c 'grep -c "yass:begin" AGENTS.md'
git checkout -q AGENTS.md

echo "2. a small change"
P=$(y new "Fix double-tap save" --source gh#41 --platforms "ios, android" --goal "Tapping Save twice saves once.")
has "dated folder" "^yass/changes/$TODAY-fix-double-tap-save$" echo "$P"
has "source recorded" "^source: gh#41" cat "$P/change.md"
has "platforms recorded" "^platforms: \[ios, android\]" cat "$P/change.md"
has "goal written" "Tapping Save twice saves once." cat "$P/change.md"
has "title heading" "^# Fix double-tap save" cat "$P/change.md"
P2=$(y new "Fix double-tap save")
has "same title same day gets a suffix" "fix-double-tap-save-2$" echo "$P2"
rm -rf "$P2"
sub "$P/change.md" '^## Acceptance\n' '## Acceptance\n- [ ] Given Save tapped twice, when saving, then one entry — verify: unit SaveTests\n'
has   "a fresh change is not started" "fix-double-tap-save +0/1 +not started" y status
has   "…in the detail view too" "progress: 0/1  \(not started\)" y status double-tap
sub "$P/change.md" '^- \[ \] $' '- [x] Reproduce\n- [ ] Disable Save while saving'
hasnt "a ticked box starts it" "not started" y status
has "status shows progress and next" "fix-double-tap-save +1/3 +next: Given Save tapped twice" y status
printf '\n### %s (claude)\n- Did: reproduced\n- Next: disable the button\n' "$TODAY" >> "$P/change.md"
has "next comes from the Log when there is one" "next: disable the button" y status
has "detail view lists open boxes" "\[ \] Disable Save while saving" y status double-tap
has "detail view shows source" "source: gh#41" y status double-tap
git add -A; git commit -q -m "yass: new change"
sub "$P/change.md" '^- \[ \] Disable' '- [/] Disable'
has      "an in-progress box isn't done" "fix-double-tap-save +1/3" y status
has      "…and the detail view marks it" "\[/\] Disable Save while saving" y status double-tap
run_fail "archive refuses with open boxes" y archive double-tap
has      "…and says which, in progress included" "2 box\(es\) still open" y archive double-tap
sub "$P/change.md" '^- \[ \] Given' '- [x] Given'; sub "$P/change.md" '^- \[/\] Disable' '- [-] Disable'
has      "a dropped box doesn't count" "fix-double-tap-save +2/2" y status
has      "every box done or dropped: done" "fix-double-tap-save +2/2 +done" y status
has      "…in the detail view too" "progress: 2/2  \(done\)" y status double-tap
git commit -qam "progress"
run_ok   "archive once every box is done or dropped" y archive double-tap
[ -d "yass/archive/$MONTH/$TODAY-fix-double-tap-save" ] && ok "moved to the archive" || bad "not in the archive"
[ ! -e "$P" ] && ok "gone from changes" || bad "still in changes"
has      "the move is staged" "R.*fix-double-tap-save/change.md" git status --porcelain
has      "archive stamps archived: in UTC" "^archived: [0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9:]{8}Z$" cat "yass/archive/$MONTH/$TODAY-fix-double-tap-save/change.md"
has      "…and changes nothing else" "^\+archived: [^ ]+Z\|$" bash -c 'git diff --cached -M | grep -E "^[-+][^-+]" | paste -sd"|" - | sed "s/$/|/"'
hasnt    "…and leaves nothing unstaged" "." git diff --name-only
has      "archived listing" "$TODAY-fix-double-tap-save" y status --archived
has      "an archived change in detail" "progress: 2/2  \(done\)" y status --archived double-tap
run_fail "…but not without --archived" y status double-tap
git commit -qm "yass: archive fix-double-tap-save"
OLD=yass/changes/2025-01-05-old-thing; mkdir -p "$OLD"; printf '# Old thing\n\n## Steps\n- [x] Done\n' > "$OLD/change.md"
git add -A; git commit -qm "yass: old thing"
run_ok   "a change from an earlier month…" y archive old-thing
[ -d "yass/archive/$MONTH/2025-01-05-old-thing" ] && ok "…goes in the month it was archived" || bad "not in this month's folder"
has      "a change without frontmatter gets some" "^---$" head -1 "yass/archive/$MONTH/2025-01-05-old-thing/change.md"
git commit -qm "yass: archive old-thing"
DUP="yass/changes/$TODAY-fix-double-tap-save"; mkdir -p "$DUP"; printf '# Again\n\n## Steps\n- [x] Done\n' > "$DUP/change.md"
run_fail "archive refuses a name that's already archived" y archive "$DUP"
has      "…and says where the other one is" "already archived, at yass/archive/$MONTH/$TODAY-fix-double-tap-save" y archive "$DUP"
rm -rf "$DUP"

echo "3. a large change with pieces"
L=$(y new "Offline sync" --large --design --goal "Never lose an entry.")
for f in change.md prd.md plan.md design.md; do [ -f "$L/$f" ] && ok "large has $f" || bad "large lacks $f"; done
hasnt "large head has no Steps" "^## Steps" cat "$L/change.md"
Q=$(y new "Offline queue" --in offline-sync)
B=$(y new "Sync badge" --in offline)
has "pieces live inside" "^$L/$TODAY-offline-queue$" echo "$Q"
run_fail "pieces nest one level deep" y new "Too deep" --in offline-queue
run_fail "a piece can't be large" y new "Big piece" --large --in offline-sync
has   "template prints one" "^## Options" y template design
run_fail "…and refuses an unknown one" y template nope
has "status nests pieces" "├ $TODAY-offline-queue" y status
has "large is labelled" "offline-sync  \(large\)" y status
printf '### M1\n- [x] AC1 (R1) Given no network — verify: e2e\n- [ ] AC2 (R2) Given a kill — verify: e2e\n' >> "$L/plan.md"
sub "$Q/change.md" '^- \[ \] $' '- [x] Queue table\n- [ ] Worker'
sub "$B/change.md" '^- \[ \] $' '- [ ] Badge'
has "large progress rolls up its pieces" "offline-sync  \(large\) +2/5" y status
has "pieces show their own" "offline-queue +1/2" y status
has   "a piece with no work is not started" "sync-badge +0/1 +not started" y status
hasnt "…but its change is, once any piece is" "offline-sync  \(large\) .*not started" y status
sub "$B/change.md" '^- \[ \] Badge' '- [/] Badge'
hasnt "an in-progress box starts it" "sync-badge .*not started" y status
has   "…and becomes next when the Log has none" "sync-badge .*next: Badge" y status
run_fail "a piece can't be archived alone" y archive offline-queue
has      "…it goes with its change" "archived with their change" y archive offline-queue
run_fail "archive refuses open boxes in pieces" y archive offline-sync
has      "…and names the piece" "Worker  \($TODAY-offline-queue/change.md\)" y archive offline-sync
git add -A; git commit -q -m "yass: offline sync"
run_ok   "--force archives anyway" y archive offline-sync --force
[ -f "yass/archive/$MONTH/$TODAY-offline-sync/$TODAY-sync-badge/change.md" ] && ok "pieces moved along" || bad "pieces left behind"
git commit -qm "yass: archive offline-sync"

echo "4. blocked, follows, warnings"
F=$(y new "Offline sync, web badge" --follows "$TODAY-offline-sync")
hasnt "follows an archived change: no warning" "warning" y status
sub "$F/change.md" '^blocked:$' 'blocked: waiting on design review'
sub "$F/change.md" '^- \[ \] $' '- [x] Spike'
has "blocked shows in status" "BLOCKED: waiting on design review" y status
run_fail "archive refuses a blocked change" y archive web-badge
G=$(y new "Something else" --follows 2020-01-01-nope)
has "follows something missing: warning" "warning: .*follows '2020-01-01-nope'" y status
run_fail "--strict fails on warnings" y status --strict
rm -rf "$G"
mkdir -p yass/changes/random-folder && echo "x" > yass/changes/random-folder/notes.md
has "a folder with no change.md" "random-folder: no change.md" y status
has "an undated folder name" "random-folder: name it" y status
rm -rf yass/changes/random-folder
mkdir -p "$Q" 2>/dev/null; mkdir -p "yass/changes/$TODAY-x/$TODAY-y/$TODAY-z"
for d in "$TODAY-x" "$TODAY-x/$TODAY-y" "$TODAY-x/$TODAY-y/$TODAY-z"; do printf -- '---\n---\n# t\n' > "yass/changes/$d/change.md"; done
has "nesting deeper than one level" "pieces nest one level deep" y status
rm -rf "yass/changes/$TODAY-x" "$Q"; rmdir "$L" 2>/dev/null || true
run_fail "unknown change" y status nope-nothing
E=$(y new "Empty one")
run_fail "archive refuses a change with no boxes" y archive empty-one
sub "$E/change.md" '^blocked:$' 'blocked:    # set when stuck'
sub "$E/change.md" '^follows:.*$' "follows: $TODAY-sync-badge   # an archived piece"
printf '### %s (claude)\n- **Next:** bold next step\n' "$TODAY" >> "$E/change.md"
hasnt "frontmatter comments are ignored" "empty-one .*BLOCKED|^warning" y status
has   "follows can name an archived piece" "empty-one" y status
has   "a bold **Next:** is read" "next: bold next step" y status
rm -rf "$E"
y new "Web cache" >/dev/null; y new "Web fonts" >/dev/null
has "ambiguous name" "matches [0-9]+ changes" y status web
rm -rf yass/changes/*web-cache yass/changes/*web-fonts
git add -A; git commit -qm "yass: follow-up"

echo "5. monorepo: several yass/ folders, no config"
mkdir -p services/payments && echo "# Payments" > services/payments/README.md
run_ok "init a team folder" y init services/payments
[ -d services/payments/yass/changes ] && ok "team yass/ created" || bad "team yass/ missing"
has    "AGENTS.md section not duplicated" "^1$" bash -c 'grep -c "yass:begin" AGENTS.md'
has    "new goes to the nearest yass/" "^services/payments/yass/changes/$TODAY-saved-cards-api$" bash -c 'cd services/payments && yass new "Saved cards API"'
has    "…and from the repo root, to the root yass/" "^yass/changes/$TODAY-three-tap-checkout$" y new "Three-tap checkout"
has    "status shows every yass/ folder" "services/payments/yass/" y status
has    "…root first" "^yass/$" bash -c 'yass status | head -1'
has    "detail by name across folders" "Saved cards API" y status saved-cards
git add -A; git commit -qm "monorepo"

echo "6. the optional hook"
git config core.hooksPath tools/yass/githooks
H=$(y new "Hook demo" --large)
printf '### M1\n- [ ] AC1 (R1) Given x, when y, then z — verify: unit\n' >> "$H/plan.md"
sub "$H/change.md" '^## Goal\n' '## Goal\nDemo the hook.\n'
git add -A; run_ok "intent-only commit" git commit -q -m "yass: plan hook demo"
hasnt "…without a warning" "yass:" bash -c "echo x >> README2.md; git add -A; git commit -q -m readme 2>&1"
echo "code1" >> app.txt; sub "$H/plan.md" '^- \[ \] AC1' '- [x] AC1'
printf '\n## Log\n### %s (claude)\n- Did: x\n- Next: y\n' "$TODAY" >> "$H/change.md"
git add -A
hasnt "ticking a plan box + Log + code: silent" "heads-up" git commit -q -m "feat: thing"
echo "code2" >> app.txt; echo "- **R9** new" >> "$H/prd.md"; git add -A
has   "prd edit + code: warns" "prd.md is intent" git commit -q -m "feat: sneaky"
has   "…but the commit went through" "feat: sneaky" git log -1 --format=%s
echo "code3" >> app.txt; echo "more" >> "$H/prd.md"; git add -A
run_fail "YASS_STRICT=1 refuses it" env YASS_STRICT=1 git commit -q -m "feat: sneaky 2"
run_ok   "YASS_HOOK=off skips it" env YASS_HOOK=off git commit -q -m "feat: skip"
echo "code4" >> app.txt; sub "$H/plan.md" 'when y, then z' 'when y, then maybe z'; git add -A
has   "rewording an acceptance criterion + code: warns" "only mark boxes in plan.md" git commit -q -m "feat: reword"
echo "code5" >> app.txt; sub "$H/change.md" 'Demo the hook.' 'Demo the hook, roughly.'; git add -A
has   "editing a Goal + code: warns" "title, Goal or Acceptance changed" git commit -q -m "feat: goal"
S=$(y new "Small one"); git add -A; git commit -q -m "yass: small one"
echo "code6" >> app.txt; sub "$S/change.md" '^- \[ \] $' '- [x] Step one\n- [ ] Step two'
sub "$S/change.md" '^## Decisions\n' '## Decisions\n- Do it simply - because (claude)\n'; git add -A
hasnt "adding Steps and Decisions + code: silent" "heads-up" git commit -q -m "feat: small"
echo "code7" >> app.txt; N=$(y new "Brand new"); git add -A
hasnt "a new change + code: silent" "heads-up" git commit -q -m "feat: with new change"
git add -A; git commit -q -m "progress" >/dev/null 2>&1 || true
y archive hook-demo --force >/dev/null; echo "code8" >> app.txt; git add -A
has   "archiving + code: warns" "archive a change in its own commit" git commit -q -m "feat: archive and code"
y archive brand-new --force >/dev/null; git add -A
hasnt "archiving alone: silent" "heads-up" git commit -q -m "yass: archive brand-new"
echo "x" > "yass/archive/$MONTH/$TODAY-fix-double-tap-save/notes.md"; git add -A
has   "adding a file to an archived change warns" "the archive is append-only" git commit -q -m "yass: add notes"
echo "edit" >> "yass/archive/$MONTH/$TODAY-fix-double-tap-save/change.md"; git add -A
has   "editing the archive warns, even alone" "the archive is append-only" git commit -q -m "yass: fix typo"
BASE=$(git rev-list --max-parents=0 HEAD)
has      "CI range mode finds the mixed commits" "feat: sneaky: .*prd.md is intent" yass hook --range "$BASE..HEAD"
run_fail "CI range mode with --strict fails" yass hook --range "$BASE..HEAD" --strict
D=$(y new "Drop demo"); sub "$D/change.md" '^## Acceptance\n' '## Acceptance\n- [ ] Given a, when b, then c — verify: unit\n'
git add -A; git commit -q -m "yass: drop demo"
echo "code9a" >> app.txt; sub "$D/change.md" '^- \[ \] Given a' '- [/] Given a'; git add -A
hasnt "marking a criterion in progress + code: silent" "heads-up" git commit -q -m "feat: start it"
echo "code9" >> app.txt; sub "$D/change.md" '^- \[/\] Given a' '- [-] Given a'; git add -A
has   "dropping an acceptance criterion + code: warns" "title, Goal or Acceptance changed" git commit -q -m "feat: drop it"
run_fail "a range CI can't read fails loudly" yass hook --range origin/nope..HEAD --strict
CLEAN=$(git rev-parse HEAD~1)   # "yass: drop demo": a new change, no code
run_ok   "a clean range passes" yass hook --range "$CLEAN~1..$CLEAN" --strict
git config --unset core.hooksPath

echo "7. edge cases"
echo "build/" > .gitignore; mkdir -p build/x
has "init warns when the folder is gitignored" "ignored by .gitignore" y init build/x --no-agents
rm -rf build .gitignore
mkdir -p infra/target && y init infra/target --no-agents >/dev/null
has "folders git doesn't ignore are found, whatever their name" "^infra/target/yass/changes/" bash -c 'cd infra/target && yass new "Infra thing"'
rm -rf infra
NG="$W/e2e/nogit"; rm -rf "$NG"; mkdir -p "$NG/sub/deeper"
(cd "$NG" && yass init --no-agents >/dev/null && yass new "Outside git" >/dev/null)
has "outside git, status from a subfolder finds yass/" "outside-git" bash -c "cd '$NG/sub/deeper' && yass status"
cd "$W/e2e/solo"
R=$(y new "Windows endings")
printf -- '---\r\nblocked: waiting on QA\r\n---\r\n# Windows endings\r\n\r\n## Steps\r\n- [x] a\r\n- [ ] b\r\n\r\n## Log\r\n### 2026-10-03 (c)\r\n- Next: from the log\r\n' > "$R/change.md"
has   "CRLF files: frontmatter is read" "windows-endings .*BLOCKED: waiting on QA" y status
sub "$R/change.md" '^blocked: waiting on QA' 'blocked:'
has   "…and the Log" "windows-endings .*next: from the log" y status
rm -rf "$R"
N=$(y new "With notes" --source "gh issue #41"); sub "$N/change.md" '^- \[ \] $' '- [x] The work'
mkdir -p "$N/notes" && printf -- '- [ ] an idea for later\n' > "$N/notes/ideas.md"
has      "a # inside a value isn't a comment" "source: gh issue #41" y status with-notes
has      "boxes in a subfolder that isn't a piece don't count" "with-notes .*1/1  done" y status
run_ok   "…for archive either" y archive with-notes
rm -rf "yass/archive/$MONTH/$TODAY-with-notes"; git add -A >/dev/null
A=$(y new "All dropped"); sub "$A/change.md" '^- \[ \] $' '- [-] Not needed after all'
has      "every box dropped: done" "all-dropped .*done" y status
run_ok   "…and it can be archived" y archive all-dropped
rm -rf "yass/archive/$MONTH/$TODAY-all-dropped"; git add -A >/dev/null
mkdir -p "apps/café"; y init "apps/café" --no-agents >/dev/null
has   "non-ASCII folder names: new goes to the nearest yass/" "^apps/café/yass/changes/$TODAY-accented$" bash -c 'cd "apps/café" && yass new "Accented" --large'
has   "…and status lists it" "^apps/café/yass/$" y status
git add -A; git commit -qm "yass: accented" >/dev/null
echo "code" >> app.txt; echo "- **R9** more" >> "apps/café/yass/changes/$TODAY-accented/prd.md"; git add -A
has   "…and the hook sees intent edits there" "café/yass/changes/$TODAY-accented/prd.md: prd.md is intent" yass hook
git commit -qm "feat: mixed" >/dev/null 2>&1
rm -rf "apps"; git add -A; git commit -qm "chore: tidy" >/dev/null

echo "8. install.sh installs the binary"
newrepo "$W/e2e/claude"
y init --claude --hooks >/dev/null
[ -f .claude/skills/yass-work/SKILL.md ] && ok "--claude copies the playbooks" || bad "no .claude/skills"
has "--claude imports AGENTS.md" "^@AGENTS.md" cat CLAUDE.md
has "--hooks sets the hook path" "tools/yass/githooks" git config core.hooksPath
git add -A; hasnt "the adopting commit isn't flagged" "heads-up" git commit -q -m "chore: adopt YASS"
BD="$W/e2e/bindir"; rm -rf "$BD" "$W/e2e/bindir2" "$W/e2e/bindir3" "$W/e2e/bindir4" "$W/e2e/onebin" "$W/e2e/nobin"
IOUT="$(bash "$ROOT/install.sh" --bin-dir "$BD")"
has   "--bin-dir copies the binary" "installed yass .* to .*/bindir/yass" echo "$IOUT"
[ -x "$BD/yass" ] && ok "…and it runs" || bad "no binary in --bin-dir"
[ -z "$(git status --porcelain)" ] && ok "…and, run in a repo that uses YASS, changes nothing there (joining a repo)" || bad "install.sh changed the repo: $(git status --porcelain)"
has   "…and says yass init sets up a repo" "set up a repo: .*yass init" echo "$IOUT"
mkdir -p "$W/e2e/plain"; rm -rf "$W/e2e/plain/"*
run_ok "…run outside a repo too" bash -c 'cd "$1" && bash "$2" --bin-dir "$3"' _ "$W/e2e/plain" "$ROOT/install.sh" "$W/e2e/bindir3"
[ -z "$(ls -A "$W/e2e/plain")" ] && ok "…writing nothing where it's run" || bad "install.sh wrote into the folder it ran in"
has   "…and warns that another yass on PATH comes first" "is .*, not the one just installed" bash "$ROOT/install.sh" --bin-dir "$BD"
has   "…or that --bin-dir isn't on PATH" "bindir2 isn't on your PATH" \
      env PATH=/usr/bin:/bin SHELL=/bin/zsh bash "$ROOT/install.sh" --bin-dir "$W/e2e/bindir2"
has   "…with the fix for the shell" "Fix it with:  echo 'export PATH=\".*/bindir2:\\\$PATH\"' >> ~/.zshenv" \
      env PATH=/usr/bin:/bin SHELL=/bin/zsh bash "$ROOT/install.sh" --bin-dir "$W/e2e/bindir2"
hasnt "no warning when the installed yass is the one on PATH" "WARNING" env PATH="$BD:/usr/bin:/bin" bash "$ROOT/install.sh" --bin-dir "$BD"
has   "…and it says so" "it's on your PATH" env PATH="$BD:/usr/bin:/bin" bash "$ROOT/install.sh" --bin-dir "$BD"
run_fail "an old-style install (a repo and setup options) refuses" bash "$ROOT/install.sh" . --claude --bin-dir "$W/e2e/nobin"
[ ! -e "$W/e2e/nobin" ] && ok "…before installing anything" || bad "an old-style install installed the binary"
has   "…printing the commands to use, options carried over" "^  yass init --claude$" bash "$ROOT/install.sh" . --claude --bin-dir "$W/e2e/nobin"
has   "…a repo path becoming a cd" "^  cd apps/web && yass init --hooks --path \.\./p$" bash "$ROOT/install.sh" apps/web --hooks --path ../p
has   "…and --upgrade pointing at yass upgrade" "^  yass upgrade$" bash "$ROOT/install.sh" --upgrade
has   "…after the binary install" "^  install.sh$" bash "$ROOT/install.sh" --global
has   "…keeping a --bin-dir given" "^  install.sh --bin-dir /opt/bin$" bash "$ROOT/install.sh" --global --bin-dir /opt/bin
H="$W/e2e/home"; rm -rf "$H"; mkdir -p "$H"
has   "without --bin-dir it installs to ~/.local/bin" "installed yass .* to $H/.local/bin/yass" env HOME="$H" bash "$ROOT/install.sh"
rm -rf "$H/.local"
has   "…piped too, like the one-liner" "installed yass .* to $H/.local/bin/yass" \
      bash -c 'env HOME="$2" PATH=/usr/bin:/bin bash -s < "$1"' _ "$ROOT/install.sh" "$H"
U="$W/e2e/unpacked"; rm -rf "$U"; mkdir -p "$U"; cp "$ROOT/install.sh" "$U/"; cp "$YASS_BIN" "$U/yass"
has   "an unpacked release with no kit/ installs its binary" "installed yass .* to .*/bindir4/yass" \
      env -u YASS_BIN PATH=/usr/bin:/bin bash "$U/install.sh" --bin-dir "$W/e2e/bindir4"
K="$W/e2e/script-only"; rm -rf "$K"; mkdir -p "$K"; cp "$ROOT/install.sh" "$K/"
run_fail "no binary next to the script: refuses" env -u YASS_BIN PATH=/usr/bin:/bin bash "$K/install.sh" --bin-dir "$W/e2e/nobin"
has      "…and says how to get one" "no yass binary next to this script" env -u YASS_BIN PATH=/usr/bin:/bin bash "$K/install.sh" --bin-dir "$W/e2e/nobin"
newrepo "$W/e2e/oneliner"
run_ok "the one-liner's two commands set up a repo, with --bin-dir not on PATH" \
       env PATH=/usr/bin:/bin bash -c 'bash "$1" --bin-dir "$2" >/dev/null && "$2/yass" init >/dev/null' _ "$ROOT/install.sh" "$W/e2e/onebin"
[ -f .agents/skills/yass-work/SKILL.md ] && [ -x tools/yass/githooks/pre-commit ] && ok "…playbooks, hook and all" || bad "the one-liner didn't set up the repo"
newrepo "$W/e2e/bare"
run_fail "new before init explains itself" y new "x"
has      "…with the fix" "run .*yass init" y new "x"

echo "9. yass.yaml: the yass folder lives somewhere else"
newrepo "$W/e2e/ext"; rm -rf "$W/e2e/ext-yass" "$W/e2e/home-yass"
y init --path ../ext-yass >/dev/null
[ -f yass.yaml ] && ok "--path writes yass.yaml" || bad "no yass.yaml"
[ -L yass ] && [ -z "$(git status --porcelain -- yass)" ] && ok "…and yass/ in the repo is only an ignored link" || bad "yass/ isn't an ignored link"
[ -d "$W/e2e/ext-yass/changes" ] && ok "…and creates the folder it points to" || bad "no external folder"
has   "root prints it" "ext-yass$" y root
X=$(y new "Private thing")
has   "new puts changes there" "^/.*ext-yass/changes/$TODAY-private-thing$" echo "$X"
sub "$X/change.md" '^- \[ \] $' '- [x] Did it'
has   "status reads them" "private-thing +1/1 +done" y status
git add -A; git commit -q -m "chore: adopt YASS"
hasnt "none of it lands in the repo" "private-thing" git log --stat --format=
run_ok "archive works outside git" y archive private-thing
[ -d "$W/e2e/ext-yass/archive/$MONTH/$TODAY-private-thing" ] && ok "…and moves the folder" || bad "not archived"
MAIN_SHA=$(git rev-parse --short HEAD)
git switch -q -c feature; echo wip >> app.txt; git commit -qam "feat: wip"; FEAT_SHA=$(git rev-parse --short HEAD); git switch -q main
C=$(y new "Cited")
sub "$C/change.md" '^- \[ \] $' "- [x] Shipped — code: $MAIN_SHA\n- [/] Building — code: $FEAT_SHA"
hasnt "citing a merged commit when done, a branch commit in progress: fine" "warning" y status
sub "$C/change.md" '^- \[/\] Building' '- [x] Building'
has   "done, citing a commit that isn't merged: warns" "'Building' is marked done, but $FEAT_SHA isn't on main yet" y status
has   "…in the detail view too" "isn't on main yet" y status cited
printf "path: ../ext-yass\nbranch: feature\n" > yass.yaml
hasnt "branch: in yass.yaml sets what counts as merged" "isn't on" y status
printf "path: ../ext-yass\nbranch: nope\n" > yass.yaml
has   "…and warns when it doesn't exist" "branch 'nope' isn't in this repo" y status
printf "path: ../ext-yass\n" > yass.yaml
sub "$C/change.md" "code: $MAIN_SHA" 'code: deadbee'
has   "citing a commit that doesn't exist: warns" "cites deadbee, which isn't a commit here" y status
rm -rf "$C"
printf 'path: ${YASS_HOME}/proj\n' > yass.yaml
has   "an unset variable warns" "YASS_HOME isn't set" y status
run_fail "…and new refuses" y new "Nope"
run_ok "init creates the folder under the variable" env YASS_HOME="$W/e2e/home-yass" yass init --no-agents
has   "…and root resolves it" "home-yass/proj$" env YASS_HOME="$W/e2e/home-yass" yass root
printf 'path: proj\ncolor: blue\n' > yass.yaml; mkdir -p proj/changes
has   "unknown settings warn" "unknown setting 'color'" y status
rm -rf proj; printf 'path: planning\n' > yass.yaml; y init --no-agents >/dev/null
P=$(y new "Planned here" --large)
has   "a relative path is relative to yass.yaml" "^planning/changes/$TODAY-planned-here$" echo "$P"
git add -A; git commit -q -m "yass: shape planned here"
git config core.hooksPath tools/yass/githooks
echo code >> app.txt; echo "- **R9** more" >> "$P/prd.md"; git add -A
has   "the hook finds a yass folder through yass.yaml" "prd.md is intent" git commit -q -m "feat: sneaky"
echo code >> app.txt; git add -A
hasnt "…and treats yass.yaml as YASS, not code" "heads-up" git commit -q -m "feat: plain code"
git config --unset core.hooksPath
has   "a link left from before the plans moved into the repo warns" "yass/ is a link to .*, but yass.yaml points to planning; remove the link" y status
rm yass; mkdir -p yass/changes && touch yass/changes/.gitkeep
has   "a yass/ beside a yass.yaml that points elsewhere warns" "yass/: ignored, because yass.yaml points to" y status
rm -rf yass

echo "10. yass.yaml beside yass/: settings only, and ignore"
newrepo "$W/e2e/ign"
y init >/dev/null
mkdir -p examples/a/yass/archive/2026-01-01-old examples/b/svc/yass/changes/2026-01-02-thing vendor/x/yass/changes
printf '# Old\n\n## Goal\nOld.\n\n## Steps\n- [x] Done\n' > examples/a/yass/archive/2026-01-01-old/change.md
printf '# Thing\n\n## Goal\nA thing.\n\n## Steps\n- [ ] Do it\n' > examples/b/svc/yass/changes/2026-01-02-thing/change.md
touch vendor/x/yass/changes/.gitkeep
git add -A; git commit -q -m "chore: adopt YASS, with examples"
has   "without ignore, nested yass folders count" "examples/b/svc/yass" y root
printf 'ignore:\n  - examples/*\n  - vendor\n' > yass.yaml
has   "a yass.yaml without path: keeps yass/ next to it" "^$W/e2e/ign/yass$|/ign/yass$" y root
hasnt "…and ignored folders aren't roots" "examples|vendor" y root
hasnt "…or in status" "thing|examples" y status
hasnt "…and nothing warns" "warning|ignored, because" y status
X=$(y new "Top thing")
has   "new still goes to yass/" "^yass/changes/$TODAY-top-thing$" echo "$X"
has   "inside an ignored folder, it's a project of its own" "thing" bash -c 'cd examples/b && yass status'
hasnt "…without the outer yass/" "top-thing" bash -c 'cd examples/b && yass status'
has   "…and new goes to its nearest yass/" "^svc/yass/changes/$TODAY-inner$" bash -c 'cd examples/b/svc && yass new "Inner"'
rm -rf examples/b/svc/yass/changes/$TODAY-inner
git add -A; git commit -q -m "yass: top thing"
git config core.hooksPath tools/yass/githooks
echo code >> app.txt; echo "- [x] Rewritten" >> examples/a/yass/archive/2026-01-01-old/change.md
sub examples/b/svc/yass/changes/2026-01-02-thing/change.md '^A thing\.$' 'A different thing.'
git add -A
mv yass.yaml "$W/e2e/ign.yaml"
has   "without ignore, the hook flags those edits" "append-only" y hook
mv "$W/e2e/ign.yaml" yass.yaml
hasnt "the hook treats ignored folders as ordinary files" "heads-up" git commit -q -m "feat: code and example edits"
git config --unset core.hooksPath
printf 'ignore: [../elsewhere, ".", nope/*, missing]\n' > yass.yaml
has   "an ignore outside the folder warns" "'../elsewhere' isn't inside" y status
has   "…so does the folder itself" "'\.' is the folder yass.yaml is in" y status
has   "…and one that matches nothing" "'missing' doesn't match a folder" y status

echo "11. queue.md: the order to tackle changes in"
newrepo "$W/e2e/queue"
y init >/dev/null
mk() { mkdir -p "$1"; printf '# %s\n\n## Goal\nIt.\n\n## Steps\n- [%s] Do it\n' "$(basename "$1")" "${2:- }" > "$1/change.md"; }
mk yass/changes/2026-01-01-alpha; mk yass/changes/2026-02-01-bravo; mk yass/changes/2026-03-01-charlie; mk yass/changes/2026-04-01-delta x
mk yass/changes/2026-03-01-charlie/2026-03-02-piece; mk yass/archive/2025-12-01-old x
git add -A; git commit -q -m "chore: adopt YASS"
order() { yass status | grep -oE '^ *[0-9]{4}-[0-9]{2}-[0-9]{2}-[a-z]+' | tr -d ' ' | paste -sd, -; }
has   "without queue.md, changes are in date order" "^2026-01-01-alpha,2026-02-01-bravo,2026-03-01-charlie,2026-04-01-delta$" order
printf '# Queue\nTop first.\n\n1. 2026-03-01-charlie — first\n2. `2026-01-01-alpha`\n' > yass/queue.md
has   "queue.md order first, then the rest by date" "^2026-03-01-charlie,2026-01-01-alpha,2026-02-01-bravo,2026-04-01-delta$" order
hasnt "…and nothing warns" "warning" y status
git add -A; git commit -q -m "yass: queue"
printf '\n- 2026-09-09-nope\n- 2025-12-01-old\n- 2026-03-02-piece\n- 2026-01-01-alpha\n' >> yass/queue.md
has   "an unknown entry warns" "'2026-09-09-nope' isn't a change in yass/changes" y status
has   "…an archived one" "'2025-12-01-old' is archived" y status
has   "…a piece" "'2026-03-02-piece' is a piece of 2026-03-01-charlie" y status
has   "…and a repeat" "'2026-01-01-alpha' is listed more than once" y status
run_fail "…and --strict fails" y status --strict
git checkout -q yass/queue.md
mkdir -p svc && y init svc --no-agents >/dev/null; mk svc/yass/changes/2026-01-01-one; mk svc/yass/changes/2026-02-01-two
printf -- '- 2026-02-01-two\n' > svc/yass/queue.md
has   "each yass folder follows its own queue.md" "2026-02-01-two,2026-01-01-one" order
hasnt "…without warnings across folders" "warning" y status
rm -rf svc
printf '1. 2026-04-01-delta — done, ready\n2. 2026-03-01-charlie — first\n3. `2026-01-01-alpha`\n' > yass/queue.md
git add -A; git commit -q -m "yass: rank delta"
has   "archive takes a change off queue.md" "took it off yass/queue.md" y archive delta
has   "…keeping the other lines" "^2. 2026-03-01-charlie — first$" head -1 yass/queue.md
has   "…and stages the edit with the move" "^M  yass/queue.md$" git status --short
git commit -q -m "yass: archive delta"
git config core.hooksPath tools/yass/githooks
echo code >> app.txt; printf '1. `2026-01-01-alpha`\n2. 2026-03-01-charlie\n' > yass/queue.md; git add -A
has   "the hook flags reordering queue.md with code" "queue.md is intent" git commit -q -m "feat: sneaky reorder"
git rm -q yass/queue.md; git commit -q -m "yass: drop queue"
echo code >> app.txt; printf -- '- 2026-02-01-bravo\n' > yass/queue.md; git add -A
hasnt "…but not creating one" "heads-up" git commit -q -m "feat: code and a new queue"
git config --unset core.hooksPath

echo "12. blocked: naming changes is a dependency"
newrepo "$W/e2e/deps"
y init >/dev/null
mkb() { mkdir -p "$1"; printf -- '---\nblocked: %s\n---\n# %s\n\n## Goal\nIt.\n\n## Steps\n- [%s] Do it\n' "${3:-}" "$(basename "$1")" "${2:- }" > "$1/change.md"; }
mkb yass/changes/2026-01-01-alpha /; mkb yass/changes/2026-02-01-bravo x "2026-01-01-alpha"
mkb yass/changes/2026-03-01-big; mkb yass/changes/2026-03-01-big/2026-03-02-part
mkb yass/changes/2026-04-01-multi x "yass/changes/2026-01-01-alpha, 2026-03-01-big/2026-03-02-part"
git add -A; git commit -q -m "chore: adopt YASS"
has   "waiting on a change that isn't done" "2026-02-01-bravo .*waiting on: 2026-01-01-alpha" y status
run_fail "archive refuses while it waits" y archive bravo
has   "…and says why" "waiting on 2026-01-01-alpha" y archive bravo
has   "several, as a path and as change/piece" "2026-04-01-multi .*waiting on: 2026-01-01-alpha, 2026-03-01-big/2026-03-02-part" y status
sub yass/changes/2026-01-01-alpha/change.md '^- \[/\]' '- [x]'
hasnt "once it's done, nothing waits on it" "bravo .*(waiting|BLOCKED)" y status
has   "…the detail view suggests clearing it" "clear .blocked:" y status bravo
has   "…and still waiting on the rest" "2026-04-01-multi .*waiting on: 2026-03-01-big/2026-03-02-part$" y status
run_ok "archive works once it's met" y archive bravo
git commit -q -m "yass: archive bravo"
y archive alpha >/dev/null; git commit -q -m "yass: archive alpha"
hasnt "an archived dependency is met" "multi .*alpha" y status
mkb yass/archive/2025-12-01-old x   # an archive from before months: flat
[ -d "yass/archive/$MONTH/2026-01-01-alpha" ] && [ -d yass/archive/2025-12-01-old ] && ok "flat and monthly archives side by side" || bad "layouts not mixed"
mkb yass/changes/2026-05-02-mixed / "2025-12-01-old, 2026-01-01-alpha"
sub yass/changes/2026-05-02-mixed/change.md '^blocked:' 'follows: 2025-12-01-old\nblocked:'
mkb yass/changes/2026-05-03-mixed2 / ; sub yass/changes/2026-05-03-mixed2/change.md '^blocked:' 'follows: 2026-01-01-alpha\nblocked:'
hasnt "follows: and blocked: resolve in either layout" "warning" y status
hasnt "…and both count as met" "mixed .*waiting" y status
has   "status --archived lists both" "2025-12-01-old.*2026-01-01-alpha" bash -c 'yass status --archived | tr "\n" " "'
has   "status finds an archived change in a month" "progress: 1/1  \(done\)" y status --archived 2026-01-01-alpha
printf -- '- 2026-01-01-alpha\n' >> yass/queue.md
has   "queue.md knows a monthly archive is archived" "'2026-01-01-alpha' is archived" y status
rm -rf yass/queue.md yass/changes/2026-05-02-mixed yass/changes/2026-05-03-mixed2 yass/archive/2025-12-01-old
chmod 000 "yass/archive/$MONTH/2026-01-01-alpha/change.md"
mkb yass/changes/2026-05-04-after / "2026-01-01-alpha"; sub yass/changes/2026-05-04-after/change.md '^blocked:' 'follows: 2026-01-01-alpha\nblocked:'
hasnt "status reads no archived files: unreadable ones don't matter" "warning|error|denied" y status
hasnt "…and follows: and blocked: naming one still resolve" "after .*waiting" y status
has   "status --archived says which it can't read" "2026-01-01-alpha/change.md: can't read it: permission denied" y status --archived
chmod 644 "yass/archive/$MONTH/2026-01-01-alpha/change.md"; rm -rf yass/changes/2026-05-04-after
mkb yass/changes/2026-05-01-free x "waiting on 2026-03-01-big and legal sign-off"
has   "free text, even mentioning a change, is a reason" "2026-05-01-free .*BLOCKED: waiting on 2026-03-01-big and legal" y status
run_fail "…and archive refuses" y archive free
mkb yass/changes/2026-05-01-free x "2026-09-09-nope"
has   "a dated name that isn't a change warns" "blocked: '2026-09-09-nope' looks like a change" y status
has   "…and stays a reason" "free .*BLOCKED: 2026-09-09-nope" y status
rm -rf yass/changes/2026-05-01-free
mkb yass/changes/2026-06-01-x "" "2026-06-02-y"; mkb yass/changes/2026-06-02-y "" "2026-06-01-x"
has   "a cycle warns" "waits on itself: 2026-06-01-x → 2026-06-02-y → 2026-06-01-x" y status
has   "…once" "^1$" bash -c 'yass status | grep -c "waits on itself"'
rm -rf yass/changes/2026-06-0*
mkdir -p svc; y init svc --no-agents >/dev/null; mkb svc/yass/changes/2026-07-01-api /
mkb yass/changes/2026-07-02-ui "" "svc/yass/changes/2026-07-01-api"
has   "dependencies work across yass folders" "2026-07-02-ui .*waiting on: 2026-07-01-api" y status

echo "13. yass paths: what's YASS's, for CI"
newrepo "$W/e2e/paths"; rm -rf "$W/e2e/paths-priv"
y init --no-agents >/dev/null; printf 'ignore:\n  - examples/*\n' > yass.yaml
mkdir -p services/payments services/search services/priv examples/demo/yass/changes
y init services/payments --no-agents >/dev/null
printf 'path: planning\n' > services/search/yass.yaml; mkdir -p services/search/planning/changes
printf 'path: ../../../paths-priv\n' > services/priv/yass.yaml; mkdir -p "$W/e2e/paths-priv/changes"
EXP=$'services/payments/yass/**\nservices/priv/yass.yaml\nservices/search/planning/**\nservices/search/yass.yaml\nyass.yaml\nyass/**'
[ "$(y paths)" = "$EXP" ] && ok "paths lists the yass folders in the repo and every yass.yaml, and nothing ignored or outside" || bad "paths: $(y paths | tr '\n' ' ')"
git add -A; git commit -q -m "chore: adopt YASS"
B=$(git rev-parse HEAD); y new "Plan only" >/dev/null; printf 'x\n' > services/payments/yass/changes/note.md; git add -A; git commit -q -m "yass: plan"
has   "--only: a range with only plans exits 0" "plans only: 2 file" y paths --only "$B..HEAD"
run_ok "…with status 0" y paths --only "$B..HEAD"
B=$(git rev-parse HEAD); echo more >> app.txt; echo y >> services/payments/yass/changes/note.md; git commit -qam "feat: code and plans"
run_fail "a range with any other file exits 1" y paths --only "$B..HEAD"
has   "…and names one" "1 of 2 changed file.* e.g. app.txt" y paths --only "$B..HEAD"
run_fail "an empty range runs the checks (exits 1)" y paths --only HEAD..HEAD
has   "a range it can't read exits 2 and says to fetch" "fetch history first" y paths --only nope..HEAD
bash -c 'yass paths --only nope..HEAD' >/dev/null 2>&1; [ $? -eq 2 ] && ok "…with status 2" || bad "not exit 2"
B=$(git rev-parse HEAD); sub yass/changes/*-plan-only/change.md '^- \[ \] $' '- [x] Done'; git commit -qam "yass: done"; y archive plan-only >/dev/null; git commit -q -m "yass: archive"
run_ok "archiving (a move inside yass/) is plans only" y paths --only "$B..HEAD"
cd "$W/e2e/paths-priv"; git init -q -b main; printf 'path: .\n' > yass.yaml
has   "a plans repo (path: .) is YASS's throughout" "^\*\*$" y paths

echo "16. worktrees and the yass/ link"
newrepo "$W/e2e/wt"; rm -rf "$W/e2e/wt-plans" "$W/e2e/wt-other" "$W/e2e/wts" "$W/e2e/wt-priv"*
y init --path ../wt-plans --no-agents >/dev/null; git add yass.yaml; git commit -q -m "chore: adopt YASS"
[ "$(readlink yass)" = "$(cd "$W/e2e/wt-plans" && pwd -P)" ] && ok "init links yass/ to the plans" || bad "no link: $(readlink yass)"
y status >/dev/null; y status >/dev/null
has   "…ignored through info/exclude, once" "^1$" grep -c '^/yass$' .git/info/exclude
[ -z "$(git status --porcelain)" ] && [ ! -e .gitignore ] && ok "…so git status shows nothing, and .gitignore is untouched" || bad "git status: $(git status --porcelain)"
y new "Linked" >/dev/null
has   "a change is listed once" "^1$" bash -c 'yass status | grep -c linked'
hasnt "…with no warnings" "warning" y status
git worktree add -q "$W/e2e/wts/deep/feat" -b feat
cd "$W/e2e/wts/deep/feat"
has   "a worktree elsewhere finds the main checkout's plans" "wt-plans$" y root
has   "…and lists its changes" "linked" y status
[ -L yass ] && [ -z "$(git status --porcelain)" ] && ok "…and gets its own ignored link" || bad "worktree link: $(ls -la yass 2>&1) $(git status --porcelain)"
cd "$W/e2e/wt"; git worktree add -q .claude/worktrees/n1 -b n1
has   "a worktree nested in the repo finds them too" "wt-plans$" bash -c 'cd .claude/worktrees/n1 && yass root'
mv "$W/e2e/wt-plans" "$W/e2e/wt-gone"
cd "$W/e2e/wts/deep/feat"
has   "plans missing from a worktree: the warning says where it looked" "isn't there from this worktree or the clone's other checkouts \(looked in .*wt-plans" y status
hasnt "…and never suggests yass init" "yass init" y status
run_fail "…root fails" y root
has   "…and init refuses rather than make a stray folder" "run .yass init. from the main checkout" y init --no-agents
[ ! -e "$W/e2e/wts/deep/wt-plans" ] && [ ! -e "$W/e2e/wts/wt-plans" ] && ok "…and made nothing" || bad "stray plans folder"
cd "$W/e2e/wt"; mv "$W/e2e/wt-gone" "$W/e2e/wt-plans"
printf 'path: ${WT_PLANS:-../wt-plans}\n' > yass.yaml
has   "\${VAR:-default}: the default when it's unset" "wt-plans$" y root
mkdir -p "$W/e2e/wt-other/changes"
has   "…the variable when it's set" "wt-other$" env WT_PLANS="$W/e2e/wt-other" yass root
case "$(readlink yass)" in *wt-other) ok "…and the link follows it";; *) bad "link not repointed: $(readlink yass)";; esac
y root >/dev/null
case "$(readlink yass)" in *wt-plans) ok "…and back";; *) bad "link not repointed back: $(readlink yass)";; esac
git checkout -q yass.yaml
rm yass; mkdir yass
has   "a real yass/ is left alone, with one warning" "^1$" bash -c 'yass status | grep -c "yass/: already there, so it isn.t a link"'
[ -d yass ] && [ ! -L yass ] && ok "…and kept" || bad "real yass/ replaced"
rmdir yass
newrepo "$W/e2e/wt-priv"; y init --path ../wt-priv-plans --no-agents >/dev/null
echo yass.yaml > .gitignore; git add .gitignore; git commit -q -m "keep the pointer private"
git worktree add -q "$W/e2e/wts/priv" -b feat
has   "a gitignored yass.yaml: a worktree uses the main checkout's" "wt-priv-plans$" bash -c "cd '$W/e2e/wts/priv' && yass root"
[ -L "$W/e2e/wts/priv/yass" ] && ok "…and gets the link" || bad "no link in the private-pointer worktree"
cd "$W/e2e"; rm -rf bare.git bare-wts; git clone -q --bare "$W/e2e/wt" bare.git
git -C bare.git worktree add -q "$W/e2e/bare-wts/main" main 2>/dev/null
git -C bare.git worktree add -q "$W/e2e/bare-wts/deep/feat2" -b feat2 main
mkdir -p "$W/e2e/bare-wts/wt-plans/changes"
has   "a bare clone's worktrees find plans next to another worktree" "bare-wts/wt-plans$" bash -c "cd '$W/e2e/bare-wts/deep/feat2' && yass root"

echo "22. private team folders: kept out of git, found through git config"
newrepo "$W/e2e/pv"; rm -rf "$W/e2e/pv-plans" "$W/e2e/pv-wts"
y init --no-agents >/dev/null; git add -A; git commit -q -m "chore: adopt YASS"
run_ok "init --private sets up a team folder" y init private --path ../../pv-plans --private --no-agents
y init private --path ../../pv-plans --private --no-agents >/dev/null
[ -f private/yass.yaml ] && [ -d "$W/e2e/pv-plans/changes" ] && [ -L private/yass ] && ok "…writes yass.yaml, creates the plans and links private/yass/" || bad "private setup incomplete"
has   "…adds /private to info/exclude once" "^1$" grep -c '^/private$' .git/info/exclude
has   "…and private to git config yass.include once" "^private$" git config --get-all yass.include
[ -z "$(git status --porcelain)" ] && ok "…and git status stays clean" || bad "git status: $(git status --porcelain)"
has   "status from the top lists it" "pv-plans/$" y status
has   "…and root" "pv-plans$" y root
hasnt "paths leaves it out" "private" y paths
P=$(cd private && yass new "Secret")
case "$P" in */e2e/pv-plans/changes/*-secret) ok "new inside private/ lands in the private plans";; *) bad "new went to $P";; esac
echo code >> app.txt; git add app.txt
hasnt "the hook ignores it" "." env YASS_STRICT=1 yass hook
git commit -q -m "feat: code"
git worktree add -q "$W/e2e/pv-wts/deep/a" -b a
cd "$W/e2e/pv-wts/deep/a"
has   "a worktree elsewhere lists the private plans" "pv-plans$" y root
[ -L private/yass ] && [ -z "$(git status --porcelain)" ] && ok "…gets its own private/yass link, and stays clean" || bad "worktree: $(ls -la private 2>&1) $(git status --porcelain)"
P=$(cd private && yass new "From a worktree")
case "$P" in */e2e/pv-plans/changes/*-from-a-worktree) ok "…and new inside its private/ lands there too";; *) bad "new went to $P";; esac
cd "$W/e2e/pv"; git worktree add -q .claude/worktrees/n -b n
has   "a nested worktree lists them too" "pv-plans$" bash -c 'cd .claude/worktrees/n && yass root'
git config --add yass.include gone
has   "an include with no yass.yaml anywhere warns, and says how to stop" "yass.include: 'gone' has no yass.yaml in any checkout .*git config --unset yass.include '.gone.'\)$" y status
git config --unset yass.include '^gone$'
run_fail "--private outside a folder refuses" y init --private --path ../x --no-agents

echo "23. change names end on a whole word"
newrepo "$W/e2e/slug"; y init --no-agents >/dev/null
P=$(y new "Paths that don't exist yet compare through symlinks")
has   "a long title ends on its last whole word" "/$TODAY-paths-that-don-t-exist-yet-compare$" echo "$P"
mkdir -p yass/archive/2026-01-01-paths-that-don-t-exist-yet-compare-throu
printf -- '# Paths that don'"'"'t exist yet compare through symlinks\n\n- [x] done\n' > yass/archive/2026-01-01-paths-that-don-t-exist-yet-compare-throu/change.md
has   "an archived change cut the old way still lists by name alone" "^2026-01-01-paths-that-don-t-exist-yet-compare-throu$" y status --archived

echo "24. yass init sets up a repo from the binary alone"
B="$W/e2e/alone-bin"; rm -rf "$B"; mkdir -p "$B"; cp "$YASS_BIN" "$B/yass"
V="$("$B/yass" --version | sed 's/^yass v\{0,1\}//')"
VR="$(printf '%s' "$V" | sed 's/[.+]/\\&/g')"   # the version as a regex: pseudo-versions have . and +
newrepo "$W/e2e/alone"
INIT_OUT="$(env -u YASS_BIN PATH="$B:/usr/bin:/bin" yass init)"
for f in .agents/skills/yass-log/SKILL.md .agents/skills/yass-plan/SKILL.md .agents/skills/yass-shape/SKILL.md \
         .agents/skills/yass-status/SKILL.md .agents/skills/yass-work/SKILL.md AGENTS.md yass/changes/.gitkeep; do
  [ -e "$f" ] && ok "writes $f" || bad "missing $f"
done
[ -x tools/yass/githooks/pre-commit ] && ok "…and an executable hook script" || bad "no executable hook"
[ ! -e .claude ] && [ ! -e CLAUDE.md ] && ok "…and no Claude files without --claude" || bad "Claude files without --claude"
hasnt "…and leaves the hook off" "githooks" git config core.hooksPath
has   "it lists what it wrote" "wrote .agents/skills/yass-work/SKILL.md" echo "$INIT_OUT"
same=1; for f in "$ROOT"/kit/.agents/skills/*/SKILL.md; do
  n="$(basename "$(dirname "$f")")"
  diff -q <(grep -v -e '^metadata:$' -e '^  yass-version: ' ".agents/skills/$n/SKILL.md") "$f" >/dev/null || same=0
done
diff -q <(grep -v '^# yass-version: ' tools/yass/githooks/pre-commit) "$ROOT/kit/tools/yass/githooks/pre-commit" >/dev/null || same=0
[ "$same" = 1 ] && ok "…each matching its source in kit/ but for the version stamp" || bad "an installed file differs from kit/ beyond its stamp"
has   "playbooks carry the version in their frontmatter" "^  yass-version: \"$VR\"$" cat .agents/skills/yass-work/SKILL.md
has   "…the AGENTS.md marker too" "<!-- yass:begin version=$VR " cat AGENTS.md
has   "…and the hook" "^# yass-version: $VR$" cat tools/yass/githooks/pre-commit
echo "changed" > .agents/skills/yass-work/SKILL.md
has   "re-running keeps what's there" "already set up" yass init
has   "…edits included" "^changed$" cat .agents/skills/yass-work/SKILL.md
newrepo "$W/e2e/alone-claude"; printf '# Our rules\n' > CLAUDE.md
yass init --claude --hooks >/dev/null; yass init --claude >/dev/null
[ -f .claude/skills/yass-work/SKILL.md ] && ok "--claude copies the playbooks for Claude Code" || bad "no .claude/skills"
has   "…imports AGENTS.md at the top of CLAUDE.md" "^@AGENTS.md$" head -1 CLAUDE.md
has   "…once, keeping what was there" "^1 1$" bash -c 'echo "$(grep -c "@AGENTS.md" CLAUDE.md) $(grep -c "# Our rules" CLAUDE.md)"'
has   "--hooks turns the hook on" "^tools/yass/githooks$" git config core.hooksPath
newrepo "$W/e2e/alone-claude2"; yass init --claude >/dev/null
has   "…and writes a CLAUDE.md when there's none" "^@AGENTS.md$" cat CLAUDE.md
newrepo "$W/e2e/alone-global"; H="$W/e2e/alone-home"; rm -rf "$H"
GL_OUT="$(HOME="$H" yass init --global --claude)"
[ -f "$H/.agents/skills/yass-work/SKILL.md" ] && [ -f "$H/.claude/skills/yass-work/SKILL.md" ] && ok "--global puts the playbooks (and Claude's) in the user folder" || bad "no user-folder playbooks"
[ ! -e .agents ] && [ ! -e .claude ] && ok "…none in the repo" || bad "--global wrote playbooks into the repo"
[ -x tools/yass/githooks/pre-commit ] && ok "…while the hook still goes in the repo" || bad "no hook with --global"
newrepo "$W/e2e/alone-skillsdir"; SD="$W/e2e/alone-skills"; rm -rf "$SD"
YASS_SKILLS_DIR="$SD" HOME="$H" yass init --global >/dev/null
[ -f "$SD/yass-work/SKILL.md" ] && ok "…or in \$YASS_SKILLS_DIR" || bad "\$YASS_SKILLS_DIR ignored"
newrepo "$W/e2e/alone-team"; yass init >/dev/null; git add -A; git commit -q -m "chore: adopt YASS"
mkdir -p svc; yass init svc >/dev/null
[ ! -e svc/.agents ] && [ ! -e svc/tools ] && ok "a team folder's init writes no playbooks or hook" || bad "team init wrote kit files"
newrepo "$W/e2e/alone-noagents"; yass init --no-agents >/dev/null
[ ! -e .agents ] && [ ! -e tools ] && [ ! -e AGENTS.md ] && ok "--no-agents skips AGENTS.md, the playbooks and the hook" || bad "--no-agents wrote agent files"

# vbin VERSION: a yass binary that reports VERSION ("dev" for none), built once, for checks that
# depend on versions; CI's own build reports whatever its checkout gives it.
vbin() { local d="$W/vbin/$1"
  if [ ! -x "$d/yass" ]; then mkdir -p "$d"
    if [ "$1" = dev ]; then (cd "$ROOT" && go build -buildvcs=false -o "$d/yass" ./cmd/yass)
    else (cd "$ROOT" && go build -ldflags "-X main.version=$1" -o "$d/yass" ./cmd/yass); fi
  fi; echo "$d/yass"; }

echo "25. agent files per folder, for monorepos"
newrepo "$W/e2e/mono"; yass init >/dev/null; git add -A; git commit -q -m "chore: adopt YASS"
ROOT_AGENTS="$(cksum < AGENTS.md)"
mkdir -p apps/web services/api
yass init services/api >/dev/null
[ -d services/api/yass/changes ] && ok "a folder without --agents gets its planning" || bad "no services/api/yass"
[ ! -e services/api/AGENTS.md ] && [ ! -e services/api/.agents ] && ok "…and no agent files" || bad "folder init wrote agent files"
[ "$(cksum < AGENTS.md)" = "$ROOT_AGENTS" ] && ok "…and leaves the root AGENTS.md alone" || bad "folder init changed the root AGENTS.md"
mkdir -p services/ext; yass init services/ext --path ../../../mono-ext-plans >/dev/null
[ -f services/ext/yass.yaml ] && [ ! -e services/ext/AGENTS.md ] && ok "…or its yass.yaml, with --path" || bad "folder --path init wrote the wrong files"
printf '# Web rules\n' > apps/web/AGENTS.md
WEB_OUT="$(yass init apps/web --agents --claude)"
has   "--agents puts the YASS section in the folder's AGENTS.md" "yass:begin version=" cat apps/web/AGENTS.md
has   "…keeping what was there" "# Web rules" cat apps/web/AGENTS.md
[ -f apps/web/.agents/skills/yass-work/SKILL.md ] && ok "…and the playbooks under the folder" || bad "no apps/web/.agents/skills"
[ -f apps/web/.claude/skills/yass-work/SKILL.md ] && has "…with Claude's copies and import there, with --claude" "^@AGENTS.md$" head -1 apps/web/CLAUDE.md || bad "no Claude files in apps/web"
[ ! -e apps/web/tools ] && ok "…and no second hook" || bad "--agents wrote a hook into the folder"
[ -d apps/web/yass/changes ] && ok "…along with its planning" || bad "no apps/web/yass"
[ "$(cksum < AGENTS.md)" = "$ROOT_AGENTS" ] && ok "…and the root AGENTS.md is untouched" || bad "--agents changed the root AGENTS.md"
has   "it lists what it wrote" "wrote apps/web/.agents/skills/yass-work/SKILL.md" echo "$WEB_OUT"
echo "Ours" > .agents/skills/yass-plan/SKILL.md; rm .agents/skills/yass-log/SKILL.md
REINIT="$("$(vbin 0.5.0)" init)"
has   "re-running init writes what's missing" "wrote .agents/skills/yass-log/SKILL.md" echo "$REINIT"
has   "…and keeps what's there" "^Ours$" cat .agents/skills/yass-plan/SKILL.md
has   "…and says yass upgrade would update files older than the binary (an unstamped one here)" "yass upgrade. updates them" echo "$REINIT"
git checkout -q -- .agents; git clean -qfd .agents
hasnt "a repo at the binary's version gets no upgrade note" "yass upgrade" yass init
sub AGENTS.md 'yass:begin version=\S+' 'yass:begin version=0.0.1'
has   "…an AGENTS.md section stamped older gets one" "yass upgrade. updates them" "$(vbin 0.5.0)" init
has   "…and init leaves the section as it is" "version=0.0.1 " cat AGENTS.md

echo "26. yass upgrade"
Y4="$(vbin 0.4.0)"; Y5="$(vbin 0.5.0)"; Y5D="$(vbin 0.5.0+dirty)"; YDEV="$(vbin dev)"
stamps() { grep -rhoE 'yass-version: "?[^" ]+|yass:begin version=[^ ]+' . --include=SKILL.md --include=AGENTS.md --include=pre-commit 2>/dev/null | sed -E 's/.*(version: "?|version=)//' | sort | uniq -c | tr -s ' '; }
newrepo "$W/e2e/up"; "$Y4" init --claude --hooks >/dev/null
printf '# Ours, before\n\n%s\n\n# Ours, after\n' "$(cat AGENTS.md)" > AGENTS.md; printf '\n# Our Claude notes\n' >> CLAUDE.md
"$Y4" new "Keep me" >/dev/null; git add -A; git commit -q -m "chore: adopt YASS"
KEEP="$(cat yass/changes/*-keep-me/change.md | cksum) $(grep -c 'Ours' AGENTS.md) $(grep -c 'Our Claude notes' CLAUDE.md)"
echo "edited" > .agents/skills/yass-work/SKILL.md; git commit -qam "edit a playbook"
UP="$("$Y5" upgrade)"
has   "upgrade says what it upgraded, from and to" "upgraded YASS's files from 0.4.0.* to 0.5.0" echo "$UP"
has   "…and lists each file" "wrote .claude/skills/yass-work/SKILL.md" echo "$UP"
has   "every stamp is the new version" "^ ?[0-9]+ 0.5.0$" stamps
hasnt "…none left at the old one" "0.4.0" stamps
[ -x tools/yass/githooks/pre-commit ] && ok "…and the hook stays executable" || bad "hook lost its mode"
[ "$(cat yass/changes/*-keep-me/change.md | cksum) $(grep -c 'Ours' AGENTS.md) $(grep -c 'Our Claude notes' CLAUDE.md)" = "$KEEP" ] && ok "changes, text outside the AGENTS.md markers and CLAUDE.md are untouched" || bad "upgrade touched what isn't YASS's"
has   "a hand edit is replaced, and the diff shows it" "^-edited$" git diff -- .agents/skills/yass-work/SKILL.md
git add -A; git commit -qm "chore: upgrade YASS"
has   "at the binary's version, it's already up to date" "already up to date: YASS's files are at 0.5.0" "$Y5" upgrade
[ -z "$(git status --porcelain)" ] && ok "…and changes nothing" || bad "an up-to-date upgrade changed files"
run_fail "an older binary refuses" "$Y4" upgrade
has   "…naming the newer version and the fix" "written by yass 0.5.0, newer than this one \(0.4.0\); .yass update. gets that version" "$Y4" upgrade
[ -z "$(git status --porcelain)" ] && ok "…and writes nothing" || bad "a refused upgrade changed files"
has   "a +dirty build rewrites its own version" "wrote AGENTS.md" "$Y5D" upgrade
git checkout -q -- .
run_fail "a binary with no version refuses to upgrade" "$YDEV" upgrade
has   "…saying how to get a versioned build" "has no version .*Go 1.24" "$YDEV" upgrade
newrepo "$W/e2e/up-dev"; run_fail "…or to init" "$YDEV" init
[ ! -e AGENTS.md ] && ok "…writing nothing" || bad "an unversioned init wrote files"
newrepo "$W/e2e/up-global"; H="$W/e2e/up-home"; rm -rf "$H"
has   "init labels files outside version control" "wrote .*up-home/.agents/skills/yass-work/SKILL.md \(outside version control\)" env HOME="$H" "$Y4" init --global
GUP="$(HOME="$H" "$Y5" upgrade)"
has   "upgrade upgrades user-folder playbooks, labeled outside version control" "up-home/.agents/skills/yass-work/SKILL.md \(outside version control\)" echo "$GUP"
has   "…to the new version" "yass-version: \"0.5.0\"" cat "$H/.agents/skills/yass-work/SKILL.md"
[ ! -e .agents ] && ok "…without adding repo copies" || bad "upgrade added repo playbooks"
newrepo "$W/e2e/up-legacy"
for f in "$ROOT"/kit/.agents/skills/*/SKILL.md; do n="$(basename "$(dirname "$f")")"
  mkdir -p ".agents/skills/$n" ".claude/skills/$n"; cp "$f" ".agents/skills/$n/"; cp "$f" ".claude/skills/$n/"; done
mkdir -p tools/yass/githooks yass/changes yass/archive; cp "$ROOT/kit/tools/yass/githooks/pre-commit" tools/yass/githooks/
"$Y5" template agents > AGENTS.md; printf '@AGENTS.md\n' > CLAUDE.md
has   "a repo set up by v0.2 (no stamps) upgrades" "from unstamped \(v0.2 or earlier\) to 0.5.0" "$Y5" upgrade
newrepo "$W/e2e/up-fresh"; "$Y5" init --claude --hooks >/dev/null
same=1; for f in .agents .claude tools AGENTS.md CLAUDE.md; do diff -r -q "$W/e2e/up-legacy/$f" "$f" >/dev/null || same=0; done
[ "$same" = 1 ] && ok "…ending up exactly like a fresh init" || bad "upgraded v0.2 repo differs from a fresh init: $(diff -rq "$W/e2e/up-legacy/.agents" .agents; diff -q "$W/e2e/up-legacy/AGENTS.md" AGENTS.md)"
NG2="$W/e2e/up-nogit"; rm -rf "$NG2"; mkdir -p "$NG2"
run_fail "upgrade outside a git repo refuses" bash -c 'cd "$1" && "$2" upgrade' _ "$NG2" "$Y5"
has   "…and says why" "works inside a git repo" bash -c 'cd "$1" && "$2" upgrade' _ "$NG2" "$Y5"
newrepo "$W/e2e/up-mono"; "$Y4" init >/dev/null; mkdir -p apps/web services/api vendor/x
"$Y4" init apps/web --agents >/dev/null; "$Y4" init services/api --agents >/dev/null
mkdir -p vendor/x/.agents/skills/yass-work; cp apps/web/.agents/skills/yass-work/SKILL.md vendor/x/.agents/skills/yass-work/
printf 'ignore:\n  - vendor/*\n' > yass.yaml
rm apps/web/.agents/skills/yass-log/SKILL.md; git add -A; git commit -q -m "chore: adopt YASS"
MUP="$(cd services/api && "$Y5" upgrade)"
has   "upgrade from a monorepo folder upgrades every copy in the repo" "wrote apps/web/AGENTS.md" echo "$MUP"
hasnt "…leaving none at the old version" "0.4.0" bash -c 'grep -rh "yass-version\|yass:begin" --include=SKILL.md --include=AGENTS.md --include=pre-commit apps services tools .agents AGENTS.md'
[ -f apps/web/.agents/skills/yass-log/SKILL.md ] && ok "…and adds a playbook missing from a skills folder" || bad "missing playbook not added"
has   "…but leaves folders yass.yaml ignores alone" "0.4.0" cat vendor/x/.agents/skills/yass-work/SKILL.md

echo "27. yass status knows the versions"
Y4="$(vbin 0.4.0)"; Y5="$(vbin 0.5.0)"; Y6="$(vbin 0.6.0)"
newrepo "$W/e2e/st"; "$Y5" init >/dev/null; git add -A; git commit -q -m "chore: adopt YASS"
has   "an older binary hears the repo is newer, and to run yass update" "note: YASS's files here were written by yass 0.5.0, newer than this one \(0.4.0\); .yass update. gets that version" "$Y4" status
run_ok "…as a note, so --strict doesn't fail on it" "$Y4" status --strict
has   "a newer binary hears yass upgrade would upgrade the repo" "note: YASS's files here are older than this yass \(0.6.0\); .yass upgrade. would upgrade them" "$Y6" status
hasnt "the same version says nothing about versions" "YASS's files" "$Y5" status
newrepo "$W/e2e/st-none"; "$Y5" init --no-agents >/dev/null
hasnt "…nor does a repo with no YASS files" "YASS's files" "$Y6" status

echo "28. --hooks leaves existing hooks alone"
newrepo "$W/e2e/hk-none"
HO="$(yass init --hooks)"
has   "no hooks setup yet: --hooks turns the hook on" "^tools/yass/githooks$" git config core.hooksPath
has   "…and says so" "hook  core.hooksPath = tools/yass/githooks" echo "$HO"
has   "running it again: already on" "hook  already on" yass init --hooks
newrepo "$W/e2e/hk-husky"; mkdir -p .husky; git config core.hooksPath .husky
run_ok "a local core.hooksPath elsewhere: init still succeeds" yass init --hooks
has   "…and leaves core.hooksPath as it was" "^\.husky$" git config core.hooksPath
has   "…naming the setup" "already has a hooks setup \(core.hooksPath = \.husky\)" yass init --hooks
has   "…and how to run yass hook from husky" "add this line to \.husky/pre-commit:" yass init --hooks
[ -x tools/yass/githooks/pre-commit ] && [ -f .agents/skills/yass-work/SKILL.md ] && ok "…while writing everything else" || bad "init skipped files because of the hooks setup"
newrepo "$W/e2e/hk-custom"; git config core.hooksPath ci/hooks
has   "a custom hooks folder gets the line for its pre-commit" "add this line to ci/hooks/pre-commit .*tools/yass/githooks/pre-commit \"\\\$@\"" bash -c 'yass init --hooks | tr "\n" " "'
newrepo "$W/e2e/hk-global"; GC="$W/e2e/hk-gitconfig"; printf '[core]\n\thooksPath = %s\n' "$W/e2e/hk-global-hooks" > "$GC"
GO="$(GIT_CONFIG_GLOBAL="$GC" yass init --hooks)"
hasnt "a global core.hooksPath: no local one is added" "." git config --local --get core.hooksPath
has   "…and it says why, and how to switch anyway" "global core.hooksPath .*override it for this repo.*git config core.hooksPath tools/yass/githooks" bash -c 'tr "\n" " " <<<"$1"' _ "$GO"
newrepo "$W/e2e/hk-githooks"; printf '#!/bin/sh\nexit 0\n' > .git/hooks/pre-commit; cp .git/hooks/pre-commit .git/hooks/commit-msg; chmod +x .git/hooks/pre-commit .git/hooks/commit-msg
LO="$(yass init --hooks)"
hasnt "hooks in .git/hooks: core.hooksPath stays unset" "." git config --get core.hooksPath
has   "…naming them (not the samples)" "runs hooks from \.git/hooks \(commit-msg, pre-commit\)" echo "$LO"
has   "…with the line to add to .git/hooks/pre-commit" "^        tools/yass/githooks/pre-commit \"\\\$@\"$" echo "$LO"
newrepo "$W/e2e/hk-lefthook"; printf 'pre-push:\n  commands: {}\n' > lefthook.yml; printf '#!/bin/sh\n' > .git/hooks/pre-commit; chmod +x .git/hooks/pre-commit
has   "lefthook gets a lefthook.yml snippet" "run: yass hook" yass init --hooks
newrepo "$W/e2e/hk-precommit"; printf 'repos: []\n' > .pre-commit-config.yaml; printf '#!/bin/sh\n' > .git/hooks/pre-commit; chmod +x .git/hooks/pre-commit
has   "the pre-commit framework gets a local-hook snippet" "entry: yass hook" yass init --hooks
cd "$W/e2e/hk-husky"; git add -A; git commit -q -m "chore: adopt YASS" --no-verify
"$(vbin 0.5.0)" upgrade >/dev/null
has   "yass upgrade leaves core.hooksPath alone" "^\.husky$" git config core.hooksPath

echo "29. same-day changes list in creation order"
newrepo "$W/e2e/created"; y init --no-agents >/dev/null
PZ=$(y new "Zebra"); PA=$(y new "Apple"); PP=$(y new "Parent" --large); PC=$(y new "Piece zebra" --in Parent); PB=$(y new "Piece apple" --in Parent)
has   "yass new stamps created: in UTC" "^created: [0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z$" cat "$PZ/change.md"
has   "…and keeps the dated folder name" "^yass/changes/$TODAY-zebra$" echo "$PZ"
has   "…a large change too" "^created: .*Z$" cat "$PP/change.md"
has   "…and a piece" "^created: .*Z$" cat "$PC/change.md"
sub "$PZ/change.md" '^created: .*$' "created: ${TODAY}T00:00:01Z"; sub "$PA/change.md" '^created: .*$' "created: ${TODAY}T00:00:02Z"
sub "$PP/change.md" '^created: .*$' "created: ${TODAY}T00:00:03Z"
sub "$PC/change.md" '^created: .*$' "created: ${TODAY}T00:00:04Z"; sub "$PB/change.md" '^created: .*$' "created: ${TODAY}T00:00:05Z"
mkdir -p "yass/changes/$TODAY-mango"; printf -- '---\nblocked:\n---\n# Mango\n\n- [ ] a\n' > "yass/changes/$TODAY-mango/change.md"
has   "same-day changes list in creation order, unstamped first" "mango .*zebra .*apple .*parent .*piece-zebra .*piece-apple " bash -c 'yass status | tr "\n" " "'
has   "an unstamped change still resolves" "# Mango|Mango" y status mango
hasnt "…and draws no warning" "created" y status
sub "$PA/change.md" '^created: .*$' "created: yesterday"
has   "a created: that isn't a time warns" "apple: created 'yesterday' isn't a time" y status
has   "…and sorts as unstamped (by date, then name)" "apple .*mango .*zebra " bash -c 'yass status | tr "\n" " "'

echo "30. AGENTS.md says intent and archives land apart from code"
Y5="$(vbin 0.5.0)"
newrepo "$W/e2e/apart"; "$Y5" init >/dev/null
has   "the section says intent stays out of a squashed pull request with code" "never changes in the same commit as code \(with squash merges, the same pull request\)" cat AGENTS.md
has   "…that archives land apart from code" "in its own commit, apart from code" cat AGENTS.md
has   "…and that pushing and pull requests need latitude" "rebase, push or open a pull request unless" cat AGENTS.md
git add -A; git commit -q -m "chore: adopt YASS"
sub AGENTS.md 'yass:begin version=0\.5\.0' 'yass:begin version=0.4.0'
sub AGENTS.md ', apart from code \(with squash merges, a pull request without code\)' ''
"$Y5" upgrade >/dev/null
has   "yass upgrade brings an older section up to it" "in its own commit, apart from code" cat AGENTS.md

echo "31. yass upgrade moves a flat archive into months"
Y6="$(vbin 0.6.0)"
newrepo "$W/e2e/migrate"; "$Y6" init --hooks >/dev/null
flat() { mkdir -p "$1"; printf '# %s\n\n- [x] Done\n' "$(basename "$1")" > "$1/change.md"; }
flat yass/archive/2026-01-10-one; flat yass/archive/2026-02-20-two/2026-02-21-piece
git add -A; GIT_COMMITTER_DATE=2026-04-15T12:00:00Z git commit -q -m "yass: archive one and two"
flat yass/archive/2026-03-01-untracked
OUT="$("$Y6" upgrade)"
has   "upgrade names the step" "migrated yass \(archive-into-months\): moved 3 archived" echo "$OUT"
has   "…and says to commit it on its own" 'commit it on its own: git commit -m "yass: archive-into-months" -- yass$' echo "$OUT"
[ -f yass/archive/2026/04/2026-01-10-one/change.md ] && ok "a change goes in the month of the commit that archived it" || bad "not in 2026/04"
has   "…stamped with that commit's date" "^archived: 2026-04-15T12:00:00Z$" cat yass/archive/2026/04/2026-01-10-one/change.md
[ -f yass/archive/2026/04/2026-02-20-two/2026-02-21-piece/change.md ] && ok "…with its pieces" || bad "piece left behind"
[ -f yass/archive/2026/03/2026-03-01-untracked/change.md ] && ok "one git never saw goes by the date in its name" || bad "untracked not by name"
[ ! -e yass/archive/2026-01-10-one ] && ok "nothing left flat" || bad "flat folder left"
hasnt "status is clean after it" "warning" y status --strict
hasnt "the hook passes the move" "heads-up|append-only" git commit -q -m "yass: archive-into-months" -- yass
hasnt "…which committed" "." git status --porcelain yass
hasnt "a second upgrade does nothing" "migrated" "$Y6" upgrade
git mv yass/archive/2026/04/2026-01-10-one yass/archive/2026/05/2026-01-10-one 2>/dev/null || { mkdir -p yass/archive/2026/05; git mv yass/archive/2026/04/2026-01-10-one yass/archive/2026/05/; }
has   "moving between months is still flagged" "append-only" git commit -q -m "yass: shuffle"
git reset -q --hard HEAD~1
flat yass/archive/2026-06-01-late; git add -A; git commit -q -m "yass: archive late"
"$Y6" upgrade >/dev/null; echo "- [x] Sneaked in" >> yass/archive/2026/*/2026-06-01-late/change.md; git add -A
has   "a move that also edits the change is flagged" "append-only" git commit -q -m "yass: migrate and edit"
git reset -q --hard HEAD~1

newrepo "$W/e2e/migrate-plain"; P6="$W/e2e/plain-plans"; rm -rf "$P6"; flat "$P6/archive/2026-02-03-elsewhere"
printf 'path: %s\n' "$P6" > yass.yaml
OUT="$("$Y6" upgrade)"
has   "a yass folder outside git migrates too" "migrated .*plain-plans \(archive-into-months\)" echo "$OUT"
hasnt "…with nothing to commit" "commit it on its own" echo "$OUT"
has   "…dated by its name, at midnight UTC" "^archived: 2026-02-03T00:00:00Z$" cat "$P6/archive/2026/02/2026-02-03-elsewhere/change.md"

PR6="$W/e2e/plans-repo"; newrepo "$PR6"; flat archive/2026-01-05-planned; git add -A
GIT_COMMITTER_DATE=2026-03-09T08:30:00Z git commit -q -m "yass: archive planned"
newrepo "$W/e2e/migrate-code"; printf 'path: %s\n' "$PR6" > yass.yaml
OUT="$("$Y6" upgrade)"
has   "a plans repo's dates come from its own history" "^archived: 2026-03-09T08:30:00Z$" cat "$PR6/archive/2026/03/2026-01-05-planned/change.md"
has   "…and the commit runs there" "git -C .*plans-repo commit -m \"yass: archive-into-months\" -- .*plans-repo" echo "$OUT"

echo "32. yass decisions"
newrepo "$W/e2e/decisions"; y init >/dev/null; y init teams/a --no-agents >/dev/null
# mkd <dir> <frontmatter lines> <entries…>
mkd() { local d="$1" fm="$2"; shift 2; mkdir -p "$d"
  { printf -- '---\n%b---\n# %s\n\n## Steps\n- [x] Done\n\n## Decisions\n<!-- - not an entry (x, 2026-01-01) -->\n' "$fm" "$(basename "$d")"
    for e in "$@"; do printf -- '- %s\n' "$e"; done; printf '\n## Log\n- Did: a thing (not a decision)\n'; } > "$d/change.md"; }
mkd yass/archive/2026/02/2026-01-01-origin 'archived: 2026-02-15T09:00:00Z\n' "Origin call - old (ann)"
mkd yass/changes/2026-03-01-bee 'follows: 2026-01-01-origin\n' "Bee call (bo, 2026-03-05)"
mkd yass/changes/2026-03-01-bee/2026-03-02-bee-piece '' "Bee piece call (bp, 2026-03-03)"
mkd teams/a/yass/changes/2026-04-01-cee 'follows: 2026-03-01-bee\n' "Cee call about Squash Merge - why (cy, 2026-04-02)"
mkd teams/a/yass/changes/2026-04-10-dee 'created: 2026-04-10T08:00:00Z\n' "Dee call (dy, 2026-04-11)"
has   "an archived entry, with the change's dates and no date of its own" "^ {10}  2026-01-01-origin  Origin call - old \(ann\)  \[created 2026-01-01, archived 2026-02-15\]$" y decisions
has   "an active entry in another yass folder, with its own date" "^2026-04-02  2026-04-01-cee  Cee call about Squash Merge - why \(cy\)  \[created 2026-04-01\]$" y decisions
has   "a piece's entries, under the piece" "^2026-03-03  2026-03-01-bee/2026-03-02-bee-piece  Bee piece call \(bp\)" y decisions
has   "each entry once, newest first, comments and Log left out" "^dee,cee,bee,piece,origin$" bash -c 'yass decisions | sed -E "s/^.{12}[^ ]*-([a-z-]+)  .*/\1/" | paste -sd, -'
has   "--since and --until, by each entry's date (else archived, else created)" "^cee,bee$" bash -c 'yass decisions --since 2026-03-04 --until 2026-04-05 | sed -E "s/^.{12}[^ ]*-([a-z-]+)  .*/\1/" | paste -sd, -'
has   "…an undated entry goes by its change's archived date" "origin" y decisions --since 2026-02-15 --until 2026-02-15
has   "--about matches all its words, any case" "^2026-04-02  2026-04-01-cee" y decisions --about "squash MERGE"
hasnt "…and nothing else" "bee|dee|origin" y decisions --about "squash merge"
has   "--about also matches the change's title" "dee call" bash -c 'yass decisions --about "10-dee" | tr A-Z a-z'
has   "--change takes in what it follows and what follows it, with pieces" "^cee,bee,piece,origin$" bash -c 'yass decisions --change 2026-03-01-bee | sed -E "s/^.{12}[^ ]*-([a-z-]+)  .*/\1/" | paste -sd, -'
has   "--limit keeps the newest and says how many it left out" "^dee,cee,… 3 more not shown" bash -c 'yass decisions --limit 2 | sed -E "s/^.{12}[^ ]*-([a-z-]+)  .*/\1/" | paste -sd, -'
hasnt "…and under the limit, says nothing more" "more not shown" y decisions
has   "no match says so" "^no decisions match$" y decisions --about zzz
run_ok "…and exits 0" y decisions --about zzz
run_fail "a bad date is refused" y decisions --since yesterday
has   "--json is one array with every field" '"change": "2026-04-01-cee",' y decisions --json
has   "…and parses" "Bee piece call" bash -c 'yass decisions --json | python3 -c "import json,sys; d=json.load(sys.stdin); print([x[\"decision\"] for x in d]); assert set(d[0]) == {\"change\",\"title\",\"who\",\"decision\",\"date\",\"created\",\"archived\"}"'
has   "…with null for what's unknown" "^None None 2026-02-15$" bash -c 'yass decisions --json --change origin | python3 -c "import json,sys; o=[x for x in json.load(sys.stdin) if x[\"change\"].endswith(\"origin\")][0]; print(o[\"date\"], None if o[\"date\"] else None, o[\"archived\"])"'
has   "…the same filters apply" '^\[\]$' y decisions --json --about zzz
has   "new changes ask for dated entries" "\(<who>, <YYYY-MM-DD>\)" y template change
has   "…large ones too" "\(<who>, <YYYY-MM-DD>\)" y template change-large
has   "…and so does the AGENTS.md section" "\(<who>, <YYYY-MM-DD>\)" cat AGENTS.md
newrepo "$W/e2e/decisions-none"
has   "no yass folder says so" "no yass folder" y decisions
run_ok "…and exits 0" y decisions

echo "33. keep and yass evict"
NOWM=$(date -u +%Y/%m)
mka() { mkdir -p "$1"; printf -- '---\narchived: %s\n---\n# %s\n\n- [x] Done\n' "${2:-2025-01-15T00:00:00Z}" "$(basename "$1")" > "$1/change.md"; }
newrepo "$W/e2e/evict"; y init --hooks >/dev/null
for n in 3 1 2; do mka yass/archive/2025/01/2025-01-0$n-jan-$n; done
for n in 1 2 3 4; do mka yass/archive/2025/02/2025-02-0$n-feb-$n 2025-02-20T00:00:00Z; done
mka "yass/archive/$NOWM/$TODAY-now-1" "${TODAY}T00:00:00Z"
printf 'archive:\n  keep: 3\n  kepp: 4\n' > yass.yaml
y init teams/x --no-agents >/dev/null; mka teams/x/yass/archive/2025/01/2025-01-01-team-one
git add -A; git commit -q -m "yass: archives"; JANSHA=$(git log -1 --format=%H -- yass/archive/2025/01)
has   "an unknown setting under archive: warns" "unknown setting 'archive.kepp'" y status
has   "a folder past keep gets a note" "note: yass/archive holds 8 archived changes, 5 past its keep of 3; .yass evict." y status
hasnt "…another folder keeps the default" "teams/x/yass/archive holds" y status
run_ok "…and --strict still passes on it" bash -c 'yass status --strict | grep -v kepp; sed -i.bak "/kepp/d" yass.yaml; yass status --strict'
rm -f yass.yaml.bak; git add -A; git commit -q -m "yass: settings"
N=$(y new "One more"); sub "$N/change.md" '^- \[ \] $' '- [x] Done'; git add -A; git commit -q -m wip
has   "archive says so too" "note: yass/archive holds 9 archived changes, 6 past its keep of 3" y archive one-more
git commit -q -m "yass: archive one-more"
OUT="$(y evict)"
has   "evict removes the oldest whole months…" "evicted yass/archive/2025/01: 3 archived change" echo "$OUT"
has   "…until it's at or under keep" "evicted yass/archive/2025/02: 4 archived change" echo "$OUT"
has   "…says what's left" "now holds 2 archived changes \(keep: 3\)" echo "$OUT"
has   "…and how to commit it" 'Commit it on its own: git commit -m "yass: evict archived months" -- yass/archive' echo "$OUT"
[ ! -e yass/archive/2025/01 ] && [ ! -e yass/archive/2025/02 ] && ok "…the months are gone" || bad "months left"
[ -d "yass/archive/$NOWM/$TODAY-now-1" ] && ok "…never this month" || bad "current month touched"
[ -d teams/x/yass/archive/2025/01 ] && ok "…nor a folder within its keep" || bad "other folder evicted"
has   "the manifest names the commit, its path and the changes, sorted" "^commit: $JANSHA
path: yass/archive/2025/01
changes:
  - 2025-01-01-jan-1
  - 2025-01-02-jan-2
  - 2025-01-03-jan-3$" bash -c 'grep -v "^#" yass/archive/2025/01.evicted'
hasnt "the hook passes the eviction" "heads-up|append-only" git commit -q -m "yass: evict archived months" -- yass/archive
has   "…and it reads back from git" "# 2025-01-02-jan-2" git show "$JANSHA:yass/archive/2025/01/2025-01-02-jan-2/change.md"
has   "evict again: nothing to do" "nothing to evict" y evict
mkb yass/changes/2026-05-05-uses-old / "2025-02-03-feb-3"; sub yass/changes/2026-05-05-uses-old/change.md '^blocked:' 'follows: 2025-01-02-jan-2\nblocked:'
hasnt "evicted changes still resolve for follows: and blocked:" "warning" y status
hasnt "…and count as met" "uses-old .*waiting" y status
has   "status --archived summarizes what's evicted" "… and 7 evicted, from 2025/01, 2025/02" y status --archived
hasnt "…and lists only what's in the tree" "jan-2|feb-3" y status --archived
git add -A; git commit -q -m "uses old" >/dev/null 2>&1
SH="$W/e2e/evict-shallow"; rm -rf "$SH"; git clone -q --depth 1 "file://$PWD" "$SH"
run_fail "in a shallow clone the evicted months' commit is missing" git -C "$SH" cat-file -e "$JANSHA"
hasnt "…and evicted names still resolve there" "warning" bash -c "cd '$SH' && yass status"
git reset -q --hard HEAD~1
git rm -q -r "yass/archive/$NOWM/$TODAY-now-1"
has   "deleting from the archive without a manifest is still flagged" "append-only" git commit -q -m "yass: tidy"
git reset -q --hard HEAD~1
has   "init's yass README says evict is the one exception" "append-only,\*\* except that .yass evict." cat yass/README.md
has   "…and so does AGENTS.md" "only .yass evict. removes from it" cat AGENTS.md

newrepo "$W/e2e/evict-refuse"; y init >/dev/null; printf 'archive:\n  keep: 1\n' > yass.yaml
for n in 1 2; do mka yass/archive/2025/0$n/2025-0$n-01-m$n; done; git add -A; git commit -q -m arch
echo "- [x] late edit" >> yass/archive/2025/01/2025-01-01-m1/change.md
run_fail "a month with uncommitted changes: evict refuses" y evict
has   "…and says why" "has changes that aren't committed" y evict
[ -d yass/archive/2025/01 ] && ok "…and changes nothing" || bad "evicted anyway"
git checkout -q -- yass
P7="$W/e2e/plain-evict"; rm -rf "$P7"; for n in 1 2; do mka "$P7/archive/2025/0$n/2025-0$n-01-p$n"; done
newrepo "$W/e2e/evict-plain"; printf 'path: %s\narchive:\n  keep: 1\n' "$P7" > yass.yaml
run_fail "outside git, evict refuses" y evict
has   "…and says why" "isn't in git" y evict

newrepo "$W/e2e/evict-merge"; y init >/dev/null; printf 'archive:\n  keep: 1\n' > yass.yaml
for n in 1 2 3; do mka yass/archive/2025/0$n/2025-0$n-01-x$n; done; git add -A; git commit -q -m arch
git switch -q -c one; y evict >/dev/null; git commit -q -m "yass: evict" -- yass/archive
git switch -q main; git switch -q -c two; y evict >/dev/null; git commit -q -m "yass: evict" -- yass/archive
run_ok "two branches evicting the same months merge without conflict" git merge -q --no-edit one
git switch -q main; git switch -q -c three; y evict >/dev/null; git commit -q -m "yass: evict" -- yass/archive
git switch -q main; git merge -q --squash three >/dev/null; git commit -q -m "yass: evict (squashed)"
S1=$(sed -n 's/^commit: //p' yass/archive/2025/01.evicted)
run_ok "after a squash merge, the manifest's commit is still on main" git merge-base --is-ancestor "$S1" main

echo "34. decisions from evicted months, and --cites"
newrepo "$W/e2e/history"; y init >/dev/null; printf 'archive:\n  keep: 1\n' > yass.yaml
CODE=$(git rev-parse HEAD); C7=${CODE:0:7}
# mkh <dir> <archived> <follows> <decision> [box]
mkh() { mkdir -p "$1"; printf -- '---\narchived: %s\nfollows: %s\n---\n# %s\n\n## Steps\n- [x] %s\n\n## Decisions\n- %s\n' "$2" "$3" "$(basename "$1")" "${5:-Done}" "$4" > "$1/change.md"; }
mkh yass/archive/2025/01/2025-01-02-jan-root 2025-01-20T00:00:00Z "" "Jan call (ann, 2025-01-10)" "Built it — code: $C7"
mkh yass/archive/2025/01/2025-01-02-jan-root/2025-01-03-jan-piece 2025-01-20T00:00:00Z "" "Jan piece call (ap, 2025-01-11)"
mkh yass/archive/2025/02/2025-02-02-feb-next 2025-02-20T00:00:00Z 2025-01-02-jan-root "Feb call (bo, 2025-02-10)"
mkh "yass/archive/$NOWM/$TODAY-now-done" "${TODAY}T00:00:00Z" "" "Now call (cy, $TODAY)" "Cited too — code: $CODE"
git add -A; git commit -q -m "yass: archives"; y evict >/dev/null; git commit -q -m "yass: evict" -- yass/archive
mkh yass/changes/2026-06-01-active "" 2025-02-02-feb-next "Active call (dy, 2026-06-02)" "Cites it — code: $C7"
sub yass/changes/2026-06-01-active/change.md '^archived: \n' ''
git add -A; git commit -q -m "yass: active"
[ ! -e yass/archive/2025/01 ] && ok "(set up: January and February are evicted)" || bad "setup: not evicted"
has   "--since reaching evicted months reads them from git" "^2025-01-10  2025-01-02-jan-root  Jan call \(ann\)  \[created 2025-01-02, archived 2025-01-20\]$" y decisions --since 2025-01-01
has   "…pieces too" "^2025-01-11  2025-01-02-jan-root/2025-01-03-jan-piece  Jan piece call" y decisions --since 2025-01-01
has   "--change follows the chain into evicted months, with pieces" "^active,next,piece,root$" bash -c 'yass decisions --change 2026-06-01-active | sed -E "s/^.{12}[^ ]*-([a-z]+)  .*/\1/" | paste -sd, -'
has   "--change naming an evicted change works" "Feb call" y decisions --change 2025-02-02-feb-next
has   "a plain query that's filled by the tree doesn't need them" "^$TODAY  $TODAY-now-done  Now call" y decisions --limit 1
has   "…one that isn't, does" "Jan call" y decisions
has   "--cites finds boxes citing a commit: active, archived and evicted, by short or long hash" "^2025-01-02-jan-root,2026-06-01-active,${TODAY}-now-done$" bash -c "yass decisions --cites $C7 | cut -d' ' -f1 | sort | paste -sd, -"
has   "…the same by its full hash" "jan-root" y decisions --cites "$CODE"
has   "…and says which file and box" "^2026-06-01-active  change.md  \[x\] Cites it — code: $C7$" y decisions --cites "$C7"
has   "a commit nobody cites" "^no box cites 0000000$" y decisions --cites 0000000
run_fail "--cites wants a hash" y decisions --cites nothex
has   "--cites --json" '"file": "change.md",' y decisions --cites "$C7" --json
SH="$W/e2e/history-shallow"; rm -rf "$SH"; git clone -q --depth 1 "file://$PWD" "$SH"
hasnt "shallow clone: a query that doesn't reach evicted months reads no history" "couldn't read" bash -c "cd '$SH' && yass decisions --since 2026-01-01"
has   "…and gives the same answer" "Active call" bash -c "cd '$SH' && yass decisions --since 2026-01-01"
hasnt "shallow clone: a plain query the tree fills reads no history either" "couldn't read" bash -c "cd '$SH' && yass decisions --limit 1"
has   "shallow clone: one that reaches them prints the rest" "Active call" bash -c "cd '$SH' && yass decisions --since 2025-01-01"
has   "…and names what it couldn't read, and how to get it" "couldn't read yass/archive/2025/01, yass/archive/2025/02: .*git fetch --unshallow" bash -c "cd '$SH' && yass decisions --since 2025-01-01"
run_ok "…and exits 0" bash -c "cd '$SH' && yass decisions --since 2025-01-01"
has   "--cites in a shallow clone says what it couldn't read too" "couldn't read" bash -c "cd '$SH' && yass decisions --cites $C7"
has   "…JSON stays clean on stdout" '^\[' bash -c "cd '$SH' && yass decisions --since 2025-01-01 --json 2>/dev/null | head -1"

echo "35. yass update --check"
# rbin <version>: a release build (it carries the release marker), built once
rbin() { local d="$W/rbin/$1"
  [ -x "$d/yass" ] || { mkdir -p "$d"; (cd "$ROOT" && go build -ldflags "-X main.version=$1 -X main.channel=release" -o "$d/yass" ./cmd/yass); }
  echo "$d/yass"; }
# releases <latest>: a fake release server whose /releases/latest redirects to v<latest>, as GitHub's does
RELPID=""
releases() { [ -n "$RELPID" ] && { kill "$RELPID"; wait "$RELPID"; } 2>/dev/null; rm -f "$W/relport"
  python3 - "$1" "$W/relport" "$W/releases" <<'PY' &
import http.server, socketserver, sys, os
latest, portfile, root = sys.argv[1], sys.argv[2], sys.argv[3]
class H(http.server.BaseHTTPRequestHandler):
    def do_HEAD(self, body=False):
        if self.path == "/releases/latest":
            self.send_response(302); self.send_header("Location", "/releases/tag/v" + latest); self.end_headers(); return
        f = os.path.join(root, self.path.removeprefix("/releases/download/"))
        if self.path.startswith("/releases/download/") and os.path.isfile(f):
            data = open(f, "rb").read()
            self.send_response(200); self.send_header("Content-Length", str(len(data))); self.end_headers()
            if body: self.wfile.write(data)
            return
        self.send_response(404); self.end_headers()
    def do_GET(self): self.do_HEAD(body=True)
    def log_message(self, *a): pass
s = socketserver.TCPServer(("127.0.0.1", 0), H)
open(portfile, "w").write(str(s.server_address[1])); s.serve_forever()
PY
  RELPID=$!
  for _ in $(seq 50); do [ -s "$W/relport" ] && break; sleep 0.1; done
  export YASS_RELEASES_URL="http://127.0.0.1:$(cat "$W/relport")/releases"; }
R4="$(rbin 0.4.0)"; R6="$(rbin 0.6.0)"
newrepo "$W/e2e/update"; "$(vbin 0.5.0)" init >/dev/null
releases 0.6.0
OUT="$("$R4" update --check)"
has   "--check names the binary and how it was built" "^yass 0.4.0 \(release build, " echo "$OUT"
has   "…the repo's version" "^this repo's YASS files: 0.5.0$" echo "$OUT"
has   "…the latest release" "^latest release: 0.6.0$" echo "$OUT"
has   "…and that it would match the repo" "^yass update would install 0.5.0 \(to match this repo's YASS files\)$" echo "$OUT"
has   "…says a newer one exists, how to get it, and what follows" "^0.6.0 is newer: .yass update --latest. gets it; then run .yass upgrade. in each repo and commit the diff" echo "$OUT"
run_ok "…exits 0" "$R4" update --check
has   "…and changes nothing" "^yass 0.4.0$" "$R4" version
has   "--latest: it would install the latest" "would install 0.6.0 \(the latest release\)" "$R4" update --check --latest
hasnt "…and says no more about newer ones" "is newer" "$R4" update --check --latest
has   "the latest binary is up to date" "^yass is up to date \(0.6.0\)$" "$R6" update --check
run_ok "…and exits 0" "$R6" update --check
cd "$W"
has   "outside a repo: no repo line, and the latest" "would install 0.6.0 \(the latest release\)" "$R4" update --check
hasnt "…no repo line" "this repo" "$R4" update --check
cd "$W/e2e/update"
has   "a go install build isn't replaced, and gets its command" "won't replace this binary: it was built with go install; .*go install github.com/donjaime/yass/cmd/yass@v0.5.0" "$(vbin 0.4.0)" update --check
has   "a clone build gets how to rebuild" "built from a clone; update it there" "$(vbin dev)" update --check   # not y: on a tagged commit, a clone build carries the plain tag
run_fail "an older --version is refused" "$R6" update --check --version v0.4.0
has   "…and says why" "never downgrades" "$R6" update --check --version v0.4.0
{ kill "$RELPID"; wait "$RELPID"; } 2>/dev/null; RELPID=""
run_fail "unreachable releases: it says so" "$R4" update --check
has   "…naming where it looked" "couldn't reach http://127.0.0.1" "$R4" update --check
unset YASS_RELEASES_URL

echo "36. yass update installs"
GO_OS=$(cd "$ROOT" && go env GOOS); GO_ARCH=$(cd "$ROOT" && go env GOARCH); ASSET="yass_${GO_OS}_${GO_ARCH}.tar.gz"
rm -rf "$W/releases"
# mkrel <version> <binary> [ok|bad|unlisted]: a fake release, laid out as GitHub serves one
mkrel() { local d="$W/releases/v$1" sum; mkdir -p "$d/x/yass_${GO_OS}_${GO_ARCH}"
  cp "$2" "$d/x/yass_${GO_OS}_${GO_ARCH}/yass"; tar -czf "$d/$ASSET" -C "$d/x" "yass_${GO_OS}_${GO_ARCH}"; rm -rf "$d/x"
  sum=$( (sha256sum "$d/$ASSET" 2>/dev/null || shasum -a 256 "$d/$ASSET") | cut -d' ' -f1)
  case "${3:-ok}" in ok) echo "$sum  $ASSET";; bad) echo "0000000000000000000000000000000000000000000000000000000000000000  $ASSET";; unlisted) echo "$sum  other.tar.gz";; esac > "$d/checksums.txt"; }
mkrel 0.5.0 "$(rbin 0.5.0)"; mkrel 0.6.0 "$R6"
printf '#!/bin/sh\necho "yass 9.9.9"\n' > "$W/liar"; chmod +x "$W/liar"; mkrel 0.7.0 "$W/liar"
mkrel 0.8.0 "$(rbin 0.8.0)" bad; mkrel 0.9.0 "$(rbin 0.9.0)" unlisted
# PATHs with git and nothing else, and with a fake gh: auth status and attestation verify exit as told
mkpath() { local d="$W/path-$1"; rm -rf "$d"; mkdir -p "$d"; ln -s "$(command -v git)" "$d/git"
  [ -n "${2:-}" ] && { printf '#!/bin/sh\ncase "$1" in auth) exit %s;; attestation) [ %s = 0 ] && exit 0; echo "verification failed: no attestation"; exit 1;; esac\n' "$2" "$3" > "$d/gh"; chmod +x "$d/gh"; }
  echo "$d"; }
NOGH=$(mkpath nogh); GHOK=$(mkpath ghok 0 0); GHBAD=$(mkpath ghbad 0 1); GHOUT=$(mkpath ghout 1 0)
fresh() { rm -rf "$W/inst"; mkdir -p "$W/inst"; cp "${1:-$R4}" "$W/inst/yass"; echo "$W/inst/yass"; }
upd() { env PATH="$1" "$W/inst/yass" "${@:2}"; }
cd "$W/e2e/update"; releases 0.6.0
fresh >/dev/null; OUT="$(upd "$NOGH" update 2>&1)"
has   "in a repo ahead of it, update installs the repo's version" "^installed yass 0.5.0 " echo "$OUT"
has   "…which runs" "^yass 0.5.0$" "$W/inst/yass" version
has   "…after the checksum; provenance unchecked without gh, and why" "checksum verified; build provenance not checked: the GitHub CLI isn't installed" echo "$OUT"
has   "…and says a newer one exists, and what follows" "0.6.0 is newer: .yass update --latest. gets it; then run .yass upgrade. in each repo and commit the diff" echo "$OUT"
hasnt "…leaving nothing beside the binary" "yass-new|\.old" ls -A "$W/inst"
fresh >/dev/null; OUT="$(upd "$NOGH" update --latest 2>&1)"
has   "--latest installs the latest" "^installed yass 0.6.0 " echo "$OUT"
hasnt "…and says nothing about newer ones" "is newer" echo "$OUT"
fresh >/dev/null
has   "--version installs that one" "^installed yass 0.5.0 " upd "$NOGH" update --version v0.5.0
fresh "$R6" >/dev/null
run_fail "an older --version is refused" upd "$NOGH" update --version v0.5.0
has   "…and the binary is unchanged" "^yass 0.6.0$" "$W/inst/yass" version
cd "$W"; fresh >/dev/null
has   "outside a repo, update installs the latest" "^installed yass 0.6.0 " upd "$NOGH" update
cd "$W/e2e/update"; fresh >/dev/null
run_fail "a checksum that doesn't match: refused" upd "$NOGH" update --version v0.8.0
has   "…and says so" "doesn't match the release's checksums.txt" upd "$NOGH" update --version v0.8.0
has   "…the binary unchanged" "^yass 0.4.0$" "$W/inst/yass" version
has   "an archive checksums.txt doesn't list: refused" "isn't listed in the release's checksums.txt" upd "$NOGH" update --version v0.9.0
has   "with gh, provenance is verified" "checksum verified, build provenance verified" upd "$GHOK" update
fresh >/dev/null
run_fail "provenance that doesn't verify: refused" upd "$GHBAD" update
has   "…and says why" "not installing 0.5.0: its build provenance didn't verify" upd "$GHBAD" update
has   "…with what gh said" "verification failed: no attestation" upd "$GHBAD" update
has   "…the binary unchanged" "^yass 0.4.0$" "$W/inst/yass" version
has   "gh not signed in: installs, and says provenance wasn't checked" "not checked: the GitHub CLI isn't signed in" upd "$GHOUT" update
fresh >/dev/null
run_fail "--require-provenance without gh: refused" upd "$NOGH" update --require-provenance
has   "…and says why" "provenance can't be checked: the GitHub CLI isn't installed" upd "$NOGH" update --require-provenance
run_fail "a downloaded binary that isn't the version asked for: refused" upd "$NOGH" update --version v0.7.0
has   "…and says what it said" "doesn't run as 0.7.0 here \(it said: yass 9.9.9\)" upd "$NOGH" update --version v0.7.0
has   "…the binary unchanged" "^yass 0.4.0$" "$W/inst/yass" version
hasnt "…and nothing left beside it" "yass-new" ls -A "$W/inst"
chmod a-w "$W/inst"
run_fail "a folder it can't write: refused before downloading" upd "$NOGH" update
has   "…saying where, and what to do" "can't write to .*inst, where this yass is; rerun with permission" upd "$NOGH" update
chmod u+w "$W/inst"
run_fail "a release that doesn't exist: refused" upd "$NOGH" update --version v0.42.0
has   "…naming it" "there's no release v0.42.0" upd "$NOGH" update --version v0.42.0
fresh >/dev/null
has   "installing past the repo says to upgrade it next, and that teammates follow" "^next: this repo's YASS files are older \(0.5.0\); run .yass upgrade. here and commit the diff \(chore: upgrade YASS\), and teammates then need yass 0.6.0 too" upd "$NOGH" update --latest
fresh >/dev/null
hasnt "…matching the repo, there's nothing next" "^next:" upd "$NOGH" update
fresh >/dev/null
{ kill "$RELPID"; wait "$RELPID"; } 2>/dev/null; RELPID=""
run_fail "releases unreachable: refused" upd "$NOGH" update
has   "…the binary unchanged" "^yass 0.4.0$" "$W/inst/yass" version
unset YASS_RELEASES_URL

echo "37. yass-update, the playbook"
newrepo "$W/e2e/update-kit"; "$(vbin 0.5.0)" init --claude >/dev/null
[ -f .agents/skills/yass-update/SKILL.md ] && [ -f .claude/skills/yass-update/SKILL.md ] && ok "init installs yass-update with the others" || bad "no yass-update playbook"
has   "the AGENTS.md section points to it" "out of step, or someone asks to update YASS: .yass-update." cat AGENTS.md
has   "…and so does yass-status" "Versions out of step.*yass-update" cat .agents/skills/yass-status/SKILL.md
git add -A; git commit -q -m "chore: adopt YASS"; rm -rf .agents/skills/yass-update .claude/skills/yass-update
has   "upgrade adds it to a repo that doesn't have it" "wrote .agents/skills/yass-update/SKILL.md" "$(vbin 0.5.0)" upgrade
[ -f .claude/skills/yass-update/SKILL.md ] && ok "…in .claude/skills too" || bad "not added to .claude/skills"

echo "38. criteria the pieces deliver"
newrepo "$W/e2e/delivers"; y init --hooks >/dev/null; git add -A; git commit -q -m "chore: adopt YASS"; BASE=$(git rev-parse HEAD)
L=$(y new "Three pieces" --large)
printf '### M1\n- [ ] AC1 (R1) Given two pieces — verify: e2e\n- [ ] AC2 (R1) Given one — verify: e2e\n- [ ] AC3 (R2) Given none — verify: e2e\n- [ ] AC4 (R2) Given part — verify: e2e\n' >> "$L/plan.md"
P1=$(y new "Piece one" --in three-pieces); P2=$(y new "Piece two" --in three-pieces); P3=$(y new "Piece three" --in three-pieces)
sub "$P1/change.md" '^- \[ \] $' '- [ ] Delivers AC1 in [plan.md](../plan.md)\n- [ ] Work one'
sub "$P2/change.md" '^- \[ \] $' '- [ ] Delivers AC1–AC2 in [plan.md](../plan.md)\n- [ ] Work two'
sub "$P3/change.md" '^- \[ \] $' '- [ ] Delivers AC4'"'"'s other half\n- [ ] Work three'
git add -A; git commit -q -m "yass: plan three-pieces"
# land <piece dir> <n>: tick the piece's boxes with code on a branch, squash-merge it into main
land() { git switch -q -c "piece-$2"; sub "$1/change.md" '^- \[ \] ' '- [x] '; echo "code $2" >> app.txt; git add -A; git commit -q -m "feat: piece $2"
  git switch -q main; git merge -q --squash "piece-$2" >/dev/null; git commit -q -m "feat: piece $2 (#$2)"; }
land "$P1" 1
has   "one of two delivering pieces done: the criterion stays open" "AC1 \(R1\) Given two pieces" y status three-pieces
land "$P2" 2
hasnt "both pieces done: the criterion counts as done" "\[ \] AC1 \(R1\)" bash -c 'yass status three-pieces | sed -n "/^open:/,\$p"'
hasnt "…and isn't offered as next" "next: AC1" y status three-pieces
hasnt "a criterion delivered by one done piece counts too" "AC2 \(R1\) Given one" bash -c 'yass status three-pieces | sed -n "/^open:/,\$p"'
has   "a criterion no piece delivers counts by its own box" "AC3 \(R2\) Given none" bash -c 'yass status three-pieces | sed -n "/^open:/,\$p"'
has   "…and the parent's progress counts what's delivered" "three-pieces  \(large\) +6/10" y status
land "$P3" 3
sub "$L/plan.md" '^- \[ \] AC3 ' '- [x] AC3 '; git commit -q -am "yass: AC3 checked"
cp "$L/plan.md" "$W/plan.orig"
has   "with every piece done and AC3 ticked, the change is done" "three-pieces  \(large\) +10/10 +done" y status
run_ok "archive doesn't refuse for criteria its pieces delivered" y archive three-pieces
A="yass/archive/$MONTH/$TODAY-three-pieces/plan.md"
has   "…and ticks them in the archived plan.md" "^- \[x\] AC1 .*\n?" grep -E "^- \[x\] AC(1|2|4) " "$A"
has   "…and nothing else changes in it" "^AC1,AC1,AC2,AC2,AC4,AC4$" bash -c "diff '$W/plan.orig' '$A' | grep -E '^[<>]' | grep -oE 'AC[0-9]+' | sort | paste -sd, -"
hasnt "the archive commit passes the hook" "heads-up|append-only" git commit -q -m "yass: archive three-pieces"
hasnt "main's whole history passes the hook" "." yass hook --range "$BASE..HEAD"
[ "$(git log --oneline "$BASE..HEAD" | grep -c "close\|done")" -eq 0 ] && ok "no closing commit per piece" || bad "a closing commit snuck in"

echo "39. sparse checkouts and assets"
newrepo "$W/e2e/sparse-full"; y init >/dev/null; y init services/search --no-agents >/dev/null; y init services/web --no-agents >/dev/null
mkb services/search/yass/changes/2026-01-02-search-thing; mkb services/search/yass/archive/2025/12/2025-12-01-search-old x
mkb services/web/yass/changes/2026-01-01-web-thing / "2026-01-02-search-thing"
sub services/web/yass/changes/2026-01-01-web-thing/change.md '^blocked:' 'follows: 2025-12-01-search-old\nblocked:'
git add -A; git commit -q -m "chore: teams"
SP="$W/e2e/sparse"; rm -rf "$SP"; git clone -q --no-checkout "file://$PWD" "$SP"; cd "$SP"
git sparse-checkout set --cone services/web; git checkout -q main
[ ! -e services/search ] && ok "(set up: a cone-mode sparse checkout without services/search/)" || bad "setup: search is checked out"
has   "status lists the yass folder that's checked out" "2026-01-01-web-thing" y status
hasnt "…and says nothing of the one that isn't" "services/search/yass/ *$|search-thing +[0-9]" y status
run_ok "…and --strict passes" y status --strict
has   "a blocked: outside the checkout says so, not that there's no such change" "blocked: '2026-01-02-search-thing' is in services/search/yass/, outside this sparse checkout, so it can't be checked" y status
has   "…as does a follows:" "follows '2025-12-01-search-old', which is in services/search/yass/, outside this sparse checkout" y status
hasnt "…and neither is a warning" "warning" y status
cd "$W/e2e/sparse-full"
mkb services/web/yass/changes/2026-01-03-typo / "2026-09-09-nope"
has   "a name that really doesn't exist still warns" "warning: .*blocked: '2026-09-09-nope' looks like a change, but there's no such change" y status
rm -rf services/web/yass/changes/2026-01-03-typo
N=$(y new "With a mockup" --large); mkdir -p "$N/assets"; head -c 2048 /dev/urandom > "$N/assets/mockup.png"
sub "$N/plan.md" '^## Validation' '- [x] AC1 (R1) Given a mockup — verify: e2e\n\n## Validation'
git add -A; git commit -q -m "yass: with a mockup"; y archive with-a-mockup --force >/dev/null
[ -f "yass/archive/$MONTH/$TODAY-with-a-mockup/assets/mockup.png" ] && ok "an asset in a change folder moves with it into the archive" || bad "asset left behind"

echo "40. pieces own their files"
newrepo "$W/e2e/own"; y init --hooks >/dev/null; git add -A; git commit -q -m "chore: adopt YASS"
L=$(y new "Parallel" --large); printf '### M1\n- [ ] AC1 (R1) Given A — verify: e2e\n- [ ] AC2 (R1) Given B — verify: e2e\n' >> "$L/plan.md"
PA=$(y new "Piece a" --in parallel); PB=$(y new "Piece b" --in parallel)
sub "$PA/change.md" '^- \[ \] $' '- [ ] Delivers AC1\n- [ ] Work a'; sub "$PB/change.md" '^- \[ \] $' '- [ ] Delivers AC2\n- [ ] Work b'
git add -A; git commit -q -m "yass: plan parallel"
git switch -q -c a; sub "$PA/change.md" '^- \[ \] Work a' '- [x] Work a'; printf '### %s (sam)\n- Did: a\n' "$TODAY" >> "$PA/change.md"; echo a >> a.txt; git add -A
hasnt "code with progress in one piece: no heads-up" "heads-up" git commit -q -m "feat: a"
git switch -q main; git switch -q -c b; sub "$PB/change.md" '^- \[ \] Work b' '- [x] Work b'; printf '### %s (kim)\n- Did: b\n' "$TODAY" >> "$PB/change.md"; echo b >> b.txt; git add -A; git commit -q -m "feat: b"
run_ok "branches on different pieces rebase onto each other without conflicts" git rebase -q a
has   "…and both pieces' progress survives" "Work a.*Work b|Work b.*Work a" bash -c "grep -h '\[x\] Work' '$PA/change.md' '$PB/change.md' | tr '\n' ' '"
sub "$L/plan.md" '^- \[ \] AC1 ' '- [x] AC1 '; echo c >> c.txt; git add -A
has   "code plus a tick in the parent's plan.md: a heads-up naming it" "parallel/plan.md: a large change's own files change in commits without code" git commit -q -m "feat: c"
git reset -q --hard HEAD~1
printf '### %s (sam)\n- Did: parent note\n' "$TODAY" >> "$L/change.md"; echo d >> d.txt; git add -A
has   "code plus a Log entry in the parent's change.md: a heads-up" "parallel/change.md: a large change's own files" git commit -q -m "feat: d"
git reset -q --hard HEAD~1
echo "- [/] more" >> "$PA/change.md"; echo "- [/] more" >> "$PB/change.md"; echo e >> e.txt; git add -A
has   "code plus progress in two pieces: a heads-up naming both" "this commit with code changes pieces $TODAY-piece-a and $TODAY-piece-b" git commit -q -m "feat: e"
git reset -q --hard HEAD~1
sub "$L/plan.md" '^- \[ \] AC1 ' '- [x] AC1 '; printf '### %s (sam)\n- Did: closing note\n' "$TODAY" >> "$L/change.md"; git add -A
hasnt "the parent's boxes and Log in a commit without code: no heads-up" "heads-up" git commit -q -m "yass: parallel progress"
mkdir -p "$L/assets"; head -c 64 /dev/urandom > "$L/assets/sketch.png"; git add -A; git commit -q -m "yass: sketch"
head -c 64 /dev/urandom > "$L/assets/sketch.png"; echo f >> f.txt; git add -A
has   "a parent's subfolder that isn't a piece counts as the parent" "assets/sketch.png: a large change's own files" git commit -q -m "feat: f"
git reset -q --hard HEAD~1

echo "41. the branch view"
O="$W/e2e/bv-origin"; newrepo "$O"; git config uploadpack.allowfilter true; y init >/dev/null
L=$(y new "Shared" --large); A=$(y new "Piece a" --in shared); B=$(y new "Piece b" --in shared)
sub "$A/change.md" '^- \[ \] $' '- [ ] Delivers AC1\n- [ ] Work a'; sub "$B/change.md" '^- \[ \] $' '- [ ] Delivers AC2\n- [ ] Work b'
git add -A; git commit -q -m "yass: shared"
git switch -q -c feat-b; sub "$B/change.md" '^- \[ \] Work b' '- [x] Work b'; printf '### %s (kim)\n- Next: polish b\n' "$TODAY" >> "$B/change.md"
git -c user.name=Kim commit -qam "feat: b"; git switch -q main
C1="$W/e2e/bv"; rm -rf "$C1"; git clone -q "file://$O" "$C1"; cd "$C1"; git config user.email t@t; git config user.name Sam
git switch -q -c feat-a; sub "$A/change.md" '^- \[ \] Work a' '- [x] Work a'; printf '### %s (sam)\n- Next: test a\n' "$TODAY" >> "$A/change.md"
git commit -qam "feat: a"; git switch -q main
git switch -q -c code-only; echo x >> app.txt; git commit -qam "feat: code only"; git switch -q main
git switch -q -c merged-one; echo "- [/] merged" >> "$A/change.md"; git commit -qam "wip"; git switch -q main
git -C "$O" fetch -q "$C1" merged-one:merged-one; git -C "$O" merge -q --ff-only merged-one; git fetch -q   # merged on the origin's main
git switch -q -c old-one; echo "- [/] old" >> "$B/change.md"; git add -A
D40=$(date -v-40d +%Y-%m-%dT12:00:00 2>/dev/null || date -d '40 days ago' +%Y-%m-%dT12:00:00); GIT_AUTHOR_DATE="$D40" GIT_COMMITTER_DATE="$D40" git commit -q -m "old"; git switch -q main
git switch -q -c ten-days; echo "- [/] ten" >> "$L/change.md"; git add -A
D10=$(date -v-10d +%Y-%m-%dT12:00:00 2>/dev/null || date -d '10 days ago' +%Y-%m-%dT12:00:00); GIT_AUTHOR_DATE="$D10" GIT_COMMITTER_DATE="$D10" git commit -q -m "ten"; git switch -q main
OUT="$(y status)"
hasnt "a Next: inside the template's comment isn't taken for one" "next: … -->" echo "$OUT"
has   "a local branch shows under its piece, with author, progress and next" "↳ feat-a  Sam, .*  1/2  next: test a" echo "$OUT"
has   "a pushed branch too" "↳ origin/feat-b  Kim, .*  1/2  next: polish b" echo "$OUT"
hasnt "a branch that only changes code isn't listed" "code-only" echo "$OUT"
hasnt "nor one already merged" "merged-one" echo "$OUT"
hasnt "nor one with nothing in the last 30 days" "old-one" echo "$OUT"
has   "a branch on the parent shows under the parent" "↳ ten-days  Sam" echo "$OUT"
has   "it says how fresh the remote branches are" "^branches: unmerged work since .*; remote branches as of the last fetch" echo "$OUT"
hasnt "--since 7d leaves out a branch from 10 days ago" "ten-days" y status --since 7d
has   "…--since an earlier date takes it in" "ten-days" y status --since "$(date -v-20d +%F 2>/dev/null || date -d '20 days ago' +%F)"
hasnt "--no-branches shows none" "↳|^branches:" y status --no-branches
hasnt "…and looks none up" "for-each-ref" bash -c 'GIT_TRACE=1 yass status --no-branches 2>&1'
hasnt "the branch view never fetches" "fetch|remote-https|upload-pack" bash -c 'GIT_TRACE=1 yass status 2>&1 | grep "trace: \(built-in\|exec\|run_command\)"'
P2="$W/e2e/bv-partial"; rm -rf "$P2"; git clone -q --filter=blob:none "file://$O" "$P2"; cd "$P2"
has   "a blobless clone shows a branch whose files aren't local, without progress" "↳ origin/feat-b  Kim, .*\(its files aren't local\)" y status
hasnt "…and downloads nothing to do it" "fetch|upload-pack" bash -c 'GIT_TRACE=1 yass status 2>&1 | grep "trace: \(built-in\|exec\|run_command\)"'
NR="$W/e2e/bv-noremote"; newrepo "$NR"; y init >/dev/null; N=$(y new "Lonely"); git add -A; git commit -q -m "yass: lonely"
hasnt "no unmerged branches: status looks as it always did" "↳|^branches:" y status
git switch -q -c local-only; sub "$N/change.md" '^- \[ \] $' '- [/] Started'; git commit -qam "start"; git switch -q main
has   "with no remote, it says there are only local branches" "^branches: .*no remote branches here" y status

echo "42. inline or separate plans"
newrepo "$W/e2e/inline"; y init >/dev/null; git add -A; git commit -q -m "chore: adopt YASS"
hasnt "plans in the repo: status says nothing more" "^plans:" y status
has   "…and root -v says inline" "/yass	inline$" y root -v
git worktree add -q "$W/e2e/inline-wt" -b wt 2>/dev/null
has   "from a linked worktree too" "	inline$" bash -c "cd '$W/e2e/inline-wt' && yass root -v"
PO="$W/e2e/plans-origin"; newrepo "$PO"; mkdir -p archive changes; touch changes/.gitkeep; git add -A; git commit -q -m plans
PR="$W/e2e/plans-clone"; rm -rf "$PR"; git clone -q "file://$PO" "$PR"
newrepo "$W/e2e/sep"; printf 'path: %s\n' "$PR" > yass.yaml; git add -A; git commit -q -m "chore: adopt YASS"
has   "plans in another repo: status ends saying so" "^plans: separate \(.*plans-clone\)$" bash -c 'yass status | tail -1'
has   "…root -v says separate" "plans-clone	separate$" y root -v
has   "…and plain root prints the folder alone" "plans-clone$" y root
echo "draft" > "$PR/changes/notes.md"
has   "uncommitted files in the plans repo get a note" "plans-clone has 1 uncommitted file\(s\)" y status
run_ok "…and --strict still passes" y status --strict
rm "$PR/changes/notes.md"
(cd "$PO" && echo more > more.txt && git add -A && git commit -q -m more); git -C "$PR" fetch -q
has   "a plans repo behind its upstream gets a note, and how to catch up" "plans-clone is 1 commit\(s\) behind origin/main as of its last fetch; pull there \(git -C .*plans-clone pull\)" y status
hasnt "…without fetching" "fetch" bash -c 'GIT_TRACE=1 yass status 2>&1 | grep "trace: \(built-in\|exec\|run_command\)"'
PN="$W/e2e/plans-plain"; rm -rf "$PN"; mkdir -p "$PN/changes" "$PN/archive"
newrepo "$W/e2e/sep-plain"; printf 'path: %s\n' "$PN" > yass.yaml; git add -A; git commit -q -m "chore: adopt YASS"
has   "plans in no repo are separate too" "^plans: separate \(.*plans-plain\)$" bash -c 'yass status | tail -1'
hasnt "…with nothing to say about commits" "uncommitted|behind" y status

echo "43. citations: one repo, plain hashes"
newrepo "$W/e2e/cites"; y init >/dev/null; git add -A; git commit -q -m "chore: adopt YASS"
git switch -q -c feature; echo f >> app.txt; git commit -qam "feat: unmerged"; U7=$(git rev-parse --short=7 HEAD); git switch -q main
H7=$(git rev-parse --short=7 HEAD)
mkdir -p yass/changes/2026-01-02-cited; printf '# Cited\n\n## Steps\n- [x] Qualified — code: web@%s\n- [x] Thing — code: %s\n- [x] Gone — code: 0badc0d\n' "$H7" "$U7" > yass/changes/2026-01-02-cited/change.md
git add -A; git commit -q -m "yass: cited"
hasnt "a repo-qualified token isn't a citation: no check, no warning" "Qualified" y status
hasnt "…and --cites doesn't find it" "Qualified" y decisions --cites "$H7"
printf 'repo: web\nrepos:\n  web: ../web\n' > yass.yaml
has   "repo: is an unknown setting now" "unknown setting 'repo'" y status
has   "…and so is repos:" "unknown setting 'repos'" y status
rm yass.yaml
OUT="$(y archive cited 2>&1)"
has   "archive archives a change whose done boxes cite unmerged or missing commits" "^archived .*2026-01-02-cited" echo "$OUT"
has   "…warning about the unmerged one" "^warning: 'Thing' \(2026-01-02-cited/change.md\) cites $U7, which isn't on main yet" echo "$OUT"
has   "…and the missing one" "^warning: 'Gone' \(2026-01-02-cited/change.md\) cites 0badc0d, which isn't a commit here" echo "$OUT"
git commit -q -m "yass: archive cited"

echo "44. one line of progress, for commits and pull requests"
newrepo "$W/e2e/brief"; y init >/dev/null
L=$(y new "Offline sync" --large); printf '### M1\n- [ ] AC1 (R1) Given a — verify: e2e\n' >> "$L/plan.md"
Q=$(y new "Offline queue" --in offline-sync); B=$(y new "Sync badge" --in offline-sync)
sub "$Q/change.md" '^- \[ \] $' '- [x] Delivers AC1\n- [x] Queue table'; sub "$B/change.md" '^- \[ \] $' '- [/] Badge'
S=$(y new "Fix double tap"); sub "$S/change.md" '^- \[ \] $' '- [x] Disable Save\n- [ ] Unit test'
has   "a piece: its parent's pieces and boxes, then its own" "^offline-sync: 1/2 pieces done, 3/4 boxes · offline-queue: 2/2 boxes, done$" y status offline-queue --brief
has   "a large change: its pieces and boxes" "^offline-sync: 1/2 pieces done, 3/4 boxes$" y status offline-sync --brief
has   "a small change: its boxes" "^fix-double-tap: 1/2 boxes$" y status double-tap --brief
sub "$S/change.md" '^- \[ \] Unit test' '- [x] Unit test'
has   "…and done when it is" "^fix-double-tap: 2/2 boxes, done$" y status double-tap --brief
run_fail "--brief needs a change" y status --brief
has   "…and says so" "--brief needs a change" y status --brief

echo "45. yass: trailers, with plans kept apart"
PL="$W/e2e/trail-plans"; newrepo "$PL"; mkdir -p changes archive; touch changes/.gitkeep
mkt() { mkdir -p "$PL/$1"; printf '# %s\n\n- [ ] Do it\n' "$(basename "$1")" > "$PL/$1/change.md"; }
mkt changes/2026-02-01-login; mkt changes/2026-02-01-login/2026-02-02-form; mkt changes/2026-02-01-login/2026-02-03-api
mkt changes/2026-02-05-signup; mkt changes/2026-02-05-signup/2026-02-02-form; mkt archive/2026-01-01-old
git add -A; git commit -q -m plans
CODE="$W/e2e/trail-code"; newrepo "$CODE"; y init --path "$PL" --hooks >/dev/null; git add -A; git commit -q -m "chore: adopt YASS" 2>/dev/null
[ -x tools/yass/githooks/prepare-commit-msg ] && [ -x tools/yass/githooks/commit-msg ] && ok "init installs the trailer hooks beside pre-commit" || bad "trailer hooks missing"
trailer() { git log -1 --format='%(trailers:key=yass,valueonly)' | head -1; }
c() { echo "$RANDOM" >> code.txt; git add code.txt; git commit -q "$@" 2>&1; }
hasnt "no change to name: the commit goes through, and nothing asks for a trailer" "." c -m "feat: nameless"
[ -z "$(trailer)" ] && ok "…and nothing is added" || bad "a trailer was invented"
y use login >/dev/null
has   "yass use names this worktree's change" "^2026-02-01-login$" y use
hasnt "with it, commits go through quietly" "." c -m "feat: with use"
has   "…carrying the trailer, in lower case" "^2026-02-01-login$" trailer
git worktree add -q "$W/e2e/trail-wt" -b other 2>/dev/null
(cd "$W/e2e/trail-wt" && yass use SIGNUP >/dev/null && echo x >> wt.txt && git add wt.txt && git commit -q -m "feat: other worktree")
has   "another worktree uses its own change" "^2026-02-05-signup$" git -C "$W/e2e/trail-wt" log -1 --format='%(trailers:key=yass,valueonly)'
has   "…while this one keeps its" "^2026-02-01-login$" bash -c 'echo y >> code.txt; git add code.txt; git commit -q -m "feat: again"; git log -1 --format="%(trailers:key=yass,valueonly)" | head -1'
y use --clear >/dev/null; has "--clear clears it" "none set" y use
git switch -q -c jaime/2026-02-01-login
c -m "feat: on a branch" >/dev/null; has "a branch named after a change (after a prefix) gives the trailer" "^2026-02-01-login$" trailer
git switch -q main; git switch -q -c 2026-02-03-api
c -m "feat: piece branch" >/dev/null; has "…a piece's name gives <change>/<piece>" "^2026-02-01-login/2026-02-03-api$" trailer
git switch -q main; git switch -q -c feature-x
c -m "feat: first" -m "yass: 2026-02-05-signup" >/dev/null; c -m "feat: second" >/dev/null
has   "otherwise an earlier commit's trailer on the branch carries over" "^2026-02-05-signup$" trailer
c -m "feat: mine" -m "yass: none" >/dev/null; has "a trailer you wrote is left alone" "^none$" trailer
git switch -q main; git merge -q --no-ff feature-x -m "Merge feature-x" 2>/dev/null
[ -z "$(git log -1 --format='%(trailers:key=yass,valueonly)')" ] && ok "a merge commit gets none" || bad "a merge got a trailer"
for t in "2026-02-01-login" "2026-01-01-old" "2026-02-01-login/2026-02-02-form" "2026-02-03-api" "none" "2026-02-01-LOGIN ac:ac1,AC2"; do
  hasnt "\`yass: $t\` is accepted" "heads-up|refusing" c -m "feat: t" -m "yass: $t"; done
N0=$(git rev-list --count HEAD)
has   "a malformed trailer gets a heads-up" "is malformed" c -m "feat: bad" -m "yass: 2026-02-01-login ac"
[ "$(git rev-list --count HEAD)" -gt "$N0" ] && ok "…and the commit is made" || bad "a commit with a bad trailer was refused"
has   "a piece name in two changes is ambiguous" "pieces in more than one change; give it as <change>/<piece>" c -m "feat: amb" -m "yass: 2026-02-02-form"
has   "an unknown key is only a warning" "doesn't know the key 'foo'" c -m "feat: k" -m "yass: 2026-02-01-login foo:bar"
hasnt "…not a heads-up" "heads-up" c -m "feat: k2" -m "yass: 2026-02-01-login foo:bar"
has   "a name that isn't a change gets a heads-up, suggesting the closest" "'2026-02-01-logn' isn't a change or piece; did you mean 2026-02-01-login" c -m "feat: typo" -m "yass: 2026-02-01-logn"
printf 'path: %s\ntrailers: require\n' "$PL" > yass.yaml
has   "trailers: is an unknown setting now" "unknown setting 'trailers'" y status
git checkout -q yass.yaml
BASE=$(git rev-parse HEAD); c -m "feat: unnamed" >/dev/null; c -m "feat: named" -m "yass: 2026-02-01-login" >/dev/null
c -m "feat: wrongly named" -m "yass: 2026-09-09-nope" >/dev/null
hasnt "hook --range leaves a commit without a trailer alone" "feat: unnamed" yass hook --range "$BASE..HEAD"
has   "…but reports one naming no change" "feat: wrongly named: .*isn't a change or piece" yass hook --range "$BASE..HEAD"
has   "--require-trailers reports a commit without one" "feat: unnamed: no \`yass:\` trailer" yass hook --range "$BASE..HEAD" --require-trailers
run_ok "…as a warning" yass hook --range "$BASE..HEAD" --require-trailers
run_fail "…and fails with --strict" yass hook --range "$BASE..HEAD" --require-trailers --strict
B2=$(git rev-parse HEAD); c -m "Login form (#12)" -m "* feat: a" -m "yass: 2026-02-01-login ac:ac1" >/dev/null
run_ok "a squash commit carrying the trailers passes" yass hook --range "$B2..HEAD" --strict
CI="$W/e2e/trail-ci"; rm -rf "$CI"; git clone -q "file://$CODE" "$CI"; cd "$CI"; printf 'path: %s/nowhere\n' "$W" > yass.yaml
has   "in CI without the plans, only the form is checked, and it says so" "change names weren't checked" yass hook --range "$BASE..HEAD"
has   "…still reporting a missing trailer when asked" "feat: unnamed: no \`yass:\` trailer" yass hook --range "$BASE..HEAD" --require-trailers
MX="$W/e2e/trail-mixed"; newrepo "$MX"; y init --hooks >/dev/null; mkdir -p team; printf 'path: %s\n' "$PL" > team/yass.yaml
mkdir -p yass/changes/2026-06-01-local; printf '# Local\n\n- [ ] Do it\n' > yass/changes/2026-06-01-local/change.md
git add -A; git commit -q -m "chore: adopt YASS"
y use 2026-02-01-login >/dev/null
c -m "feat: for the separate plans" >/dev/null; has "a repo with both kinds of plans: a change kept outside gets its trailer" "^2026-02-01-login$" trailer
has   "yass use on a change in this repo says its commits carry no trailer" "2026-06-01-local's plans are in this repo, so its commits don't carry a trailer" y use 2026-06-01-local
c -m "feat: for the local plans" >/dev/null; [ -z "$(trailer)" ] && ok "…and none is added" || bad "an inline change got a trailer"
y use --clear >/dev/null
newrepo "$W/e2e/trail-inline"; y init --hooks >/dev/null; git add -A; git commit -q -m "chore: adopt YASS"
rm tools/yass/githooks/prepare-commit-msg tools/yass/githooks/commit-msg; git add -A; git commit -q -m "older install: pre-commit only"
sub tools/yass/githooks/pre-commit '^# yass-version: .*$' '# yass-version: 0.1.0'
has   "yass upgrade adds the trailer hooks to an older install" "wrote tools/yass/githooks/commit-msg" "$(vbin 0.9.0)" upgrade
[ -x tools/yass/githooks/prepare-commit-msg ] && ok "…executable" || bad "prepare-commit-msg missing or not executable"
git checkout -q -- . 2>/dev/null; git clean -qfd tools 2>/dev/null
y use --clear >/dev/null 2>&1
hasnt "plans in the repo: nothing about trailers" "." bash -c 'echo a >> a.txt; git add a.txt; git commit -q -m "feat: inline" 2>&1'
[ -z "$(git log -1 --format='%(trailers:key=yass,valueonly)')" ] && ok "…and no trailer" || bad "inline commit got a trailer"

echo "46. a change's code, from its trailers"
P3="$W/e2e/code-plans"; newrepo "$P3"; mkdir -p changes/2026-04-01-cart/2026-04-02-ui archive
printf '# Cart\n\n## Log\n' > changes/2026-04-01-cart/change.md
printf '# Cart: plan\n\n## Acceptance\n- [ ] AC1 (R1) Given a cart — verify: e2e\n- [ ] AC3 (R2) Given a total — verify: e2e\n' > changes/2026-04-01-cart/plan.md
printf '# UI\n\n- [ ] Delivers AC3\n' > changes/2026-04-01-cart/2026-04-02-ui/change.md; git add -A; git commit -q -m plans
newrepo "$W/e2e/code-repo"; printf 'path: %s\n' "$P3" > yass.yaml; git add -A; git commit -q -m "chore: adopt YASS" -m "yass: none"
cc() { echo "$RANDOM" >> c.txt; git add c.txt; git commit -q -m "$1" -m "yass: $2"; }
cc "feat: cart model" "2026-04-01-cart ac:ac1"; M1=$(git rev-parse --short=7 HEAD)
cc "feat: cart ui" "2026-04-02-ui ac:ac1,ac3"; M2=$(git rev-parse --short=7 HEAD)
cc "chore: bump deps" "none"; cc "feat: odd one" "2026-04-01-cart ac:ac9"; M4=$(git rev-parse --short=7 HEAD)
git switch -q -c wip; cc "feat: not merged" "2026-04-01-cart"; git switch -q main
git switch -q -c pr; echo s >> s.txt; git add s.txt; git commit -q -m "wip"; git switch -q main
git merge -q --squash pr >/dev/null; git commit -q -m "Cart totals (#7)" -m "yass: 2026-04-01-cart ac:ac3"; M5=$(git rev-parse --short=7 HEAD)
OUT="$(y status 2026-04-01-cart)"
has   "status <change> lists its merged code, the pieces' included" "code on main \(from yass: trailers\)" echo "$OUT"
for m in "$M1 feat: cart model" "$M2 feat: cart ui" "$M5 Cart totals \(#7\)"; do has "…$m" "^  $m$" echo "$OUT"; done
hasnt "…not the unmerged, nor others" "not merged|bump deps" echo "$OUT"
has   "each criterion lists its commits" "^  AC1: $M2, $M1$" echo "$OUT"
has   "…AC3 once each" "^  AC3: $M5, $M2$" echo "$OUT"
has   "an ac: the plan doesn't have warns" "warning: $M4's trailer names AC9, which 2026-04-01-cart's plan doesn't have" echo "$OUT"
has   "a piece lists only its own" "^  $M2 feat: cart ui$" y status 2026-04-02-ui
hasnt "…not its change's" "cart model" y status 2026-04-02-ui

echo "48. the trailer hooks and an older yass"
newrepo "$W/e2e/old-yass"; y init --hooks >/dev/null; git add -A; git commit -q -m "chore: adopt YASS"
OLD="$W/old-yass-bin"; rm -rf "$OLD"; mkdir -p "$OLD"; ln -s "$(command -v git)" "$OLD/git"
printf '#!/bin/sh\ncase "$*" in\n  "hook --help") echo "usage: yass hook [--range A..B] [--strict]";;\n  hook) exit 0;;\n  *) echo "yass: unknown option" >&2; exit 2;;\nesac\n' > "$OLD/yass"; chmod +x "$OLD/yass"
echo x >> a.txt; git add a.txt
run_ok "with a yass from before trailers on PATH, commits still go through" env PATH="$OLD:/usr/bin:/bin" git commit -q -m "feat: older yass"
hasnt "…without its errors" "unknown option" env PATH="$OLD:/usr/bin:/bin" bash -c 'echo y >> a.txt; git add a.txt; git commit -q -m "feat: again" 2>&1'

echo "49. yass deinit takes YASS back out of a repo"
newrepo "$W/e2e/deinit"; printf '# Mine\n\nOur own notes.\n' > AGENTS.md
y init --claude --hooks >/dev/null; y init apps/web --agents --claude >/dev/null
mkdir -p .agents/skills/yass-mine .husky; printf -- '---\nname: yass-mine\n---\nnot YASS'"'"'s\n' > .agents/skills/yass-mine/SKILL.md
printf 'yass hook\n' > .husky/pre-commit; git add -A; git commit -q -m "chore: adopt YASS"; HEAD0=$(git rev-parse HEAD)
H="$W/e2e/deinit-home"; US="$W/e2e/deinit-user"; rm -rf "$H" "$US"; mkdir -p "$H" "$US"; cp -R .agents/skills/yass-work "$US/"
OUT="$(env HOME="$H" YASS_SKILLS_DIR="$US" yass deinit)"
for p in .agents/skills/yass-work .claude/skills/yass-work apps/web/.agents/skills/yass-plan tools/yass/githooks/pre-commit tools/yass/githooks/commit-msg; do
  [ ! -e "$p" ] && ok "removes $p" || bad "$p is still there"
done
has   "…listing what it removed" "^removed \.agents/skills/yass-work$" echo "$OUT"
[ ! -e tools ] && [ ! -e .claude ] && ok "…and the folders it leaves empty" || bad "empty folders left: $(ls -d tools .claude 2>/dev/null)"
[ -f .agents/skills/yass-mine/SKILL.md ] && ok "a yass-* skill that isn't YASS's stays" || bad "removed a skill that isn't YASS's"
has   "the root AGENTS.md keeps its own text" "Our own notes" cat AGENTS.md
hasnt "…without the YASS section" "yass:begin" cat AGENTS.md
has   "…and says so" "^removed the YASS section of AGENTS.md$" echo "$OUT"
has   "…and CLAUDE.md keeps its import, AGENTS.md being there" "^@AGENTS.md" cat CLAUDE.md
[ ! -e apps/web/AGENTS.md ] && [ ! -e apps/web/CLAUDE.md ] && ok "an AGENTS.md that was only YASS's goes, with the CLAUDE.md that imported it" || bad "apps/web's AGENTS.md or CLAUDE.md is still there"
hasnt "core.hooksPath is unset" "githooks" git config --get core.hooksPath
has   "…and says so" "unset core.hooksPath" echo "$OUT"
[ -d yass/changes ] && [ -d apps/web/yass ] && ok "the yass folders stay" || bad "a yass folder was removed"
has   "…and it says how to remove them, or keep them" "Kept your plans: .*yass/.*keep them, or remove them" echo "$OUT"
has   "…with the commands" "git rm -r -q yass" echo "$OUT"
has   "it says how to commit it" 'chore: remove YASS' echo "$OUT"
[ "$(git rev-parse HEAD)" = "$HEAD0" ] && ok "…and commits nothing" || bad "yass deinit committed"
[ -f "$US/yass-work/SKILL.md" ] && ok "the user folder's playbooks stay" || bad "removed the user folder's playbooks without --global"
has   "…with a note on --global" "yass deinit --global" echo "$OUT"
has   "a husky hook still running yass hook gets a note" "\.husky/pre-commit still runs .yass hook." echo "$OUT"
run_ok "--global removes the user folder's playbooks too" env HOME="$H" YASS_SKILLS_DIR="$US" yass deinit --global
[ ! -e "$US/yass-work" ] && ok "…gone" || bad "the user folder's playbook is still there"
has   "run again, there's nothing left to remove" "no YASS playbooks, AGENTS.md section or hook here" env HOME="$H" YASS_SKILLS_DIR="$US" yass deinit
y init >/dev/null
[ -f .agents/skills/yass-work/SKILL.md ] && [ -x tools/yass/githooks/pre-commit ] && grep -q "yass:begin" AGENTS.md && ok "yass init sets it up again" || bad "yass init didn't set the repo up again"

echo; echo "passed: $pass  failed: $fail"
[ "$fail" -eq 0 ]


## tests/examples.sh

#!/usr/bin/env bash
# Every example must show a clean `yass status --strict` once dropped into a repo with YASS installed.
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
W="${1:-$(mktemp -d)}"; fail=0
if [ -z "${YASS_BIN:-}" ]; then
  mkdir -p "$W/bin"; (cd "$ROOT" && go build -o "$W/bin/yass" ./cmd/yass) || exit 1; YASS_BIN="$W/bin/yass"
fi
export YASS_BIN; export PATH="$(dirname "$YASS_BIN"):$PATH"
for ex in "$ROOT"/examples/*/; do
  name="$(basename "$ex")"; d="$W/example-$name"; rm -rf "$d"; mkdir -p "$d"; cd "$d"
  git init -q -b main && git config user.email t@t && git config user.name Tester
  yass init >/dev/null
  cp -R "$ex"/. .
  git add -A >/dev/null && git commit -q -m "chore: example"
  if out="$(yass status --strict 2>&1)"; then echo "ok   $name: status is clean"; else echo "FAIL $name"; echo "$out"; fail=1; fi
  yass status --archived >/dev/null 2>&1 && echo "ok   $name: archive lists" || { echo "FAIL $name: archive"; fail=1; }
  if out="$(yass hook --range "$(git rev-list --max-parents=0 HEAD)..HEAD" --strict 2>&1)"; then echo "ok   $name: hook clean"; else echo "FAIL $name: hook"; echo "$out"; fail=1; fi
done
exit $fail


## tests/site.sh

#!/usr/bin/env bash
# Checks the landing page in site/: install text matches the docs, links and anchors resolve,
# required sections and assets are there, nothing external loads, and the page stays small.
# With --deploy, also fails while any asset is still a placeholder (pages.yml runs it that way).
# Usage: tests/site.sh [--deploy] [--root DIR]    (DIR: a repo copy to check instead of this one)
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEPLOY=0; SELF=1
while [ $# -gt 0 ]; do
  case "$1" in
    --deploy) DEPLOY=1 ;;
    --root) ROOT="$(cd "$2" && pwd)"; SELF=0; shift ;;
    *) echo "usage: tests/site.sh [--deploy] [--root DIR]" >&2; exit 2 ;;
  esac
  shift
done
SITE="$ROOT/site"; PAGE="$SITE/index.html"
REPO=https://github.com/donjaime/yass; URL=https://donjaime.github.io/yass
pass=0; fail=0
ok()  { echo "  ok   $1"; pass=$((pass+1)); }
bad() { echo "  FAIL $1"; fail=$((fail+1)); }
[ -f "$PAGE" ] || { echo "  FAIL no $PAGE"; exit 1; }

# section ID: the HTML of <section id="ID">, up to its </section>
section() { ID="$1" perl -0777 -ne 'print $1 if /<section id="\Q$ENV{ID}\E"[^>]*>(.*?)<\/section>/s' "$PAGE"; }
# slug FILE: GitHub's heading anchors for a markdown file, one per line
slugs() { perl -ne 'if (/^#{1,6}\s+(.*?)\s*$/) { $h = lc $1; $h =~ s/[^\w\- ]//g; $h =~ s/ /-/g; print "$h\n" }' "$1"; }

echo "Sections"
for id in idea folder you-say install; do
  if grep -q "<section id=\"$id\"" "$PAGE"; then ok "#$id"; else bad "#$id is missing"; fi
done
if grep -q 'class="hero"' "$PAGE" && perl -0777 -ne 'exit !(/<header class="hero">.*?href="#install".*?<\/header>/s)' "$PAGE"; then
  ok "the hero links to #install"; else bad "the hero doesn't link to #install"; fi
folder="$(section folder)"
for f in change.md prd.md plan.md design.md archive/; do
  if grep -qF "$f" <<<"$folder"; then ok "#folder shows $f"; else bad "#folder doesn't show $f"; fi
done
install="$(section install)"
for spec in terminal:Terminal "source:From Source" "agent:Agent Prompt"; do
  id="install-${spec%%:*}"; name="${spec#*:}"
  if ID="$id" NAME="$name" perl -0777 -e '$_ = <STDIN>; exit !(/<div class="panel" id="\Q$ENV{ID}\E">\s*<h3>\Q$ENV{NAME}\E<\/h3>.*?<pre/s)' <<<"$install"; then
    ok "#$id has its heading and commands in the HTML"; else bad "#$id is missing, or has no <h3>$name</h3> and <pre>"; fi
  if grep -qF "aria-controls=\"$id\">$name</button>" <<<"$install"; then ok "the $name tab controls #$id"; else bad "no $name tab controlling #$id"; fi
done
if grep -q '<div class="tablist" hidden>' <<<"$install"; then ok "the tab bar is hidden without JavaScript"; else bad "the tab bar should start hidden, so the page works without JavaScript"; fi
yousay="$(section you-say)"
for s in yass-shape yass-plan yass-work yass-status yass-log yass-update; do
  if grep -qF "<code>$s</code>" <<<"$yousay"; then ok "#you-say has $s"; else bad "#you-say has no row for $s"; fi
done

echo "Category line and easter egg"
CAT="Lightweight project management for people and coding agents"
CLOSE="<strong><em>It's lightweight project management for people and coding agents.</em></strong></p>"
if grep -qF "$CLOSE" <<<"$(section idea)"; then ok "#idea's paragraph closes with the category line"; else bad "#idea's paragraph doesn't close with $CLOSE"; fi
if perl -0777 -ne 'exit !(/<footer.*\Q'"$CAT"'\E.*<\/footer>/s)' "$PAGE"; then ok "the footer has the category line"; else bad "the footer doesn't say $CAT"; fi
for m in 'name="description"' 'property="og:description"'; do
  if grep -qF "<meta $m content=\"$CAT" "$PAGE"; then ok "$m starts with the category line"; else bad "$m doesn't start with $CAT"; fi
done
EGG="A sassy “yes” to building with coding agents."
for el in '<img class="emblem"' '<span class="wordmark"'; do
  if grep -F "$el" "$PAGE" | grep -qF "title=\"$EGG\""; then ok "${el#<} has the easter egg"; else bad "${el#<} has no title=\"$EGG\""; fi
done
if [ "$(grep -oF "$EGG" "$PAGE" | wc -l | tr -d ' ')" = 2 ]; then ok "the easter egg shows only on hover"; else bad "the easter egg appears outside its two title attributes"; fi

echo "Teams"
teams="$(section teams)"
for l in "docs/comparison.md" "docs/monorepo.md" "README.md#monorepos" "README.md#keeping-plans-out-of-the-repo" "README.md#the-optional-hook"; do
  if grep -qF "blob/main/$l\"" <<<"$teams"; then ok "#teams links to $l"; else bad "#teams has no link to $l"; fi
done

echo "README lead-in"
lead="$(perl -0777 -ne 'print $1 if /\A(.*?)^## The rules/ms' "$ROOT/README.md")"
for l in "$URL/" "(#install)"; do
  if grep -qF "$l" <<<"$lead"; then ok "the lead-in links $l"; else bad "the README has no $l above ## The rules"; fi
done
lede="$(perl -0777 -ne 'print $1 if /<p class="lede">(.*?)<\/p>/s' "$PAGE")"
idea="$(perl -0777 -ne 'print $1 if /<section id="idea"[^>]*>\s*<h2>(.*?)<\/h2>/s' "$PAGE")"
for c in "$lede" "$idea"; do
  if [ -n "$c" ] && grep -qF "$c" <<<"$lead"; then ok "the lead-in says what the page does: ${c:0:50}…"; else bad "the README lead-in doesn't say: ${c:-?}"; fi
done

echo "Install text matches its source"
n=0
while IFS= read -r -d '' rec; do
  n=$((n+1)); src="${rec%%$'\t'*}"; text="${rec#*$'\t'}"
  if [ ! -f "$ROOT/$src" ]; then bad "block $n: data-source $src doesn't exist"; continue; fi
  if TEXT="$text" perl -0777 -ne 'exit !(index($_, $ENV{TEXT}) >= 0)' "$ROOT/$src"; then
    ok "block $n matches $src"
  else
    bad "block $n doesn't match $src word for word: $(head -1 <<<"$text" | cut -c1-70)…"
  fi
done < <(perl -0777 -ne '
  while (/<pre[^>]*data-source="([^"]+)"[^>]*>(.*?)<\/pre>/sg) {
    my ($s, $t) = ($1, $2);
    $t =~ s/<button.*?<\/button>//sg; $t =~ s/<[^>]+>//g;
    $t =~ s/&lt;/</g; $t =~ s/&gt;/>/g; $t =~ s/&quot;/"/g; $t =~ s/&#39;/\x27/g; $t =~ s/&amp;/&/g;
    print "$s\t$t\0";
  }' "$PAGE")
[ "$n" -ge 2 ] && ok "$n install blocks are checked" || bad "expected at least 2 data-source blocks, found $n"

echo "Links"
for a in $(perl -ne 'print "$1\n" while /href="#([^"]+)"/g' "$PAGE" | sort -u); do
  if grep -q "id=\"$a\"" "$PAGE"; then ok "#$a resolves"; else bad "#$a has no target"; fi
done
for l in "$REPO\"" "$REPO/blob/main/README.md" "$REPO/blob/main/LICENSE"; do
  if grep -qF "href=\"$l" "$PAGE"; then ok "links to ${l%\"}"; else bad "no link to ${l%\"}"; fi
done
for l in $(perl -ne 'print "$1\n" while m{href="https://github\.com/donjaime/yass/(?:blob|tree)/main/([^"]+)"}g' "$PAGE" | sort -u); do
  path="${l%%#*}"; anchor=""; [ "$path" != "$l" ] && anchor="${l#*#}"
  if [ ! -e "$ROOT/$path" ]; then bad "$path doesn't exist"; continue; fi
  if [ -n "$anchor" ]; then
    if slugs "$ROOT/$path" | grep -qxF "$anchor"; then ok "$path#$anchor resolves"; else bad "$path has no heading for #$anchor"; fi
  else ok "$path exists"; fi
done

echo "Assets"
# dims FILE: WxH of a PNG, or of a lossy WebP (VP8, or VP8X as cwebp writes with metadata or alpha)
dims() { perl -0777 -ne '
  if (substr($_, 1, 3) eq "PNG") { my ($w, $h) = unpack("NN", substr($_, 16, 8)); print "${w}x$h" }
  elsif (substr($_, 8, 8) eq "WEBPVP8X") { my @b = unpack("C6", substr($_, 24, 6));
    printf "%dx%d", 1 + $b[0] + ($b[1] << 8) + ($b[2] << 16), 1 + $b[3] + ($b[4] << 8) + ($b[5] << 16) }
  elsif (substr($_, 8, 8) eq "WEBPVP8 ") { my ($w, $h) = unpack("vv", substr($_, 26, 4)); printf "%dx%d", $w & 0x3fff, $h & 0x3fff }' "$1"; }
for spec in emblem.webp:square favicon.svg: favicon-32.png:32x32 apple-touch-icon.png:180x180 og.png:1200x630; do
  f="${spec%%:*}"; want="${spec#*:}"
  if [ ! -f "$SITE/assets/$f" ]; then bad "assets/$f is missing"; continue; fi
  if ! grep -qF "assets/$f" "$PAGE"; then bad "the page doesn't reference assets/$f"; continue; fi
  got="$(dims "$SITE/assets/$f")"
  if [ "$want" = square ]; then
    w="${got%x*}"
    if [ -z "$got" ] || [ "$got" != "${w}x$w" ] || [ "$w" -lt 460 ]; then bad "assets/$f is ${got:-unreadable}, not square and at least 460×460"; continue; fi
  elif [ -n "$want" ] && [ "$got" != "$want" ]; then bad "assets/$f is $(dims "$SITE/assets/$f"), not $want"; continue; fi
  ok "assets/$f"
done
for p in og:title og:description twitter:card; do
  if grep -qE "(property|name)=\"$p\" content=\"[^\"]+\"" "$PAGE"; then ok "$p"; else bad "no $p"; fi
done
if grep -qF "property=\"og:image\" content=\"$URL/assets/og.png\"" "$PAGE"; then ok "og:image is absolute"; else bad "og:image isn't $URL/assets/og.png"; fi

echo "Nothing external, and small"
ext="$(grep -nE 'src="(https?:)?//|<link[^>]+href="(https?:)?//' "$PAGE"; grep -nE '@import|url\((["'\'']?)(https?:)?//' "$SITE"/*.css)"
if [ -z "$ext" ]; then ok "no external scripts, styles, fonts or images"; else bad "loads something external:"; sed 's/^/       /' <<<"$ext"; fi
bytes=$(find "$SITE" -type f ! -path "$SITE/assets/og.png" ! -name README.md -exec cat {} + | wc -c | tr -d ' ')
if [ "$bytes" -lt 153600 ]; then ok "the page loads $bytes bytes (under 150 KB)"; else bad "the page loads $bytes bytes (150 KB at most)"; fi

placeholders="$(grep -rlF yass-placeholder "$SITE/assets" | grep -v '/README\.md$' | sed "s|^$SITE/||")"
if [ "$DEPLOY" = 1 ]; then
  echo "Ready to deploy"
  if [ -z "$placeholders" ]; then ok "no placeholder assets"; else
    for f in $placeholders; do bad "$f is still a placeholder (see site/assets/README.md)"; done; fi
elif [ -n "$placeholders" ]; then
  echo "  note $(wc -l <<<"$placeholders" | tr -d ' ') placeholder asset(s); --deploy fails until they're replaced"
fi

# Self-check, on a copy of this repo: the checks above must catch what they claim to.
if [ "$SELF" = 1 ]; then
  echo "Self-check"
  T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
  copy() { rm -rf "$T/r"; mkdir -p "$T/r"; (cd "$ROOT" && tar --exclude=./.git --exclude=./private --exclude=./bin -cf - .) | (cd "$T/r" && tar -xf -); }
  edit() { P="$2" R="$3" perl -0777 -i -pe 's/\Q$ENV{P}\E/$ENV{R}/' "$1"; }
  copy; edit "$T/r/docs/install.md" "Go step by step" "Go one step at a time"
  if o="$(bash "$0" --root "$T/r" 2>&1)"; then bad "an edited agent prompt in docs/install.md should fail"
  elif grep -q "doesn't match docs/install.md" <<<"$o"; then ok "an edited agent prompt in docs/install.md fails, naming the block"
  else bad "an edited agent prompt failed for another reason"; sed 's/^/       /' <<<"$(grep FAIL <<<"$o")"; fi
  copy; printf '\n<!-- yass-placeholder -->\n' >> "$T/r/site/assets/favicon.svg"
  if o="$(bash "$0" --root "$T/r" --deploy 2>&1)"; then bad "--deploy with a placeholder should fail"
  elif grep -q "assets/favicon.svg is still a placeholder" <<<"$o"; then ok "--deploy with a placeholder fails, naming it"
  else bad "--deploy with a placeholder failed for another reason"; sed 's/^/       /' <<<"$(grep FAIL <<<"$o")"; fi
  if bash "$0" --root "$T/r" >/dev/null 2>&1; then ok "without --deploy, placeholders pass"; else bad "without --deploy, placeholders should pass"; fi
  copy; edit "$T/r/site/index.html" 'docs/install.md#from-a-release' 'docs/install.md#no-such-heading'
  if o="$(bash "$0" --root "$T/r" 2>&1)"; then bad "a link to a missing heading should fail"
  elif grep -q "has no heading for #no-such-heading" <<<"$o"; then ok "a link to a missing heading fails"
  else bad "a link to a missing heading failed for another reason"; fi
fi

echo "$pass passed, $fail failed"
[ "$fail" = 0 ]


## yass.yaml

# YASS settings. With no path:, this project's yass folder is yass/ next to this file.
# Each example is a project of its own, so its yass/ folders aren't this project's work.
# kit/ holds the playbooks and hook the binary carries; they stay unstamped, so yass upgrade leaves them alone.
ignore:
  - examples/*
  - kit


## yass/README.md

# yass/

Planned work for this part of the repo, managed with YASS (Yet Another Spec System). Code and its docs describe what exists today; this folder describes what's changing.

- `changes/`: active work, one dated folder per change. A small change is a `change.md`. A large one adds `prd.md` (why and what), `plan.md` (how and acceptance), an optional `design.md`, and PR-sized pieces as folders inside it.
- `archive/`: finished changes, moved here as they were, in `<YYYY>/<MM>/` folders by the month they were archived (older archives may sit directly in `archive/`). Never edited; follow-up work is a new change with `follows: <folder>`.
- `queue.md` (optional): the order to tackle changes in, top first, one folder name per list item. `yass status` follows it; changes it doesn't list come after, oldest first.

Rules:
1. **A change is a folder; ceremony scales with scope.**
2. **Progress travels with the code.** Ticking boxes and appending to Log and Decisions go in the same commits as the code.
3. **Intent changes get their own commit.** `prd.md`, `design.md`, `plan.md` beyond ticking boxes, and a change's title, Goal and Acceptance (including dropping a criterion).
4. **Done means every box is checked.** Then `yass archive <change>`, in its own commit.
5. **The archive is append-only,** except that `yass evict` moves its oldest months to git history when the folder is past its `keep:` setting.

`yass status` shows what's in flight.


## yass/archive/2026/10/2026-10-03-ignore-folders-from-yass-yaml/change.md

---
platforms: [all]
source: 
follows: 
blocked:
archived: 2026-10-04T02:10:24Z
---
# Ignore folders from yass.yaml

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
A yass.yaml can list folders whose yass/ folders and yass.yaml files belong to something else (examples, fixtures, vendored projects), so they stay out of this project's status, new and hook.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Given a yass.yaml with `ignore:` listing folders or glob patterns (`examples/*`), when `yass status` or `yass root` runs from outside `examples/`, then no yass folder or yass.yaml under a matched folder is listed — verify: tests/e2e.sh
- [x] Given an ignored folder, when the hook checks a commit that edits a change or the archive inside it, then those files count as ordinary files, not YASS intent or archive — verify: tests/e2e.sh
- [x] Given the current folder is inside an ignored folder, when `yass status` or `yass new` runs, then that folder is treated as its own project: its yass folders are found and the outer ones aren't — verify: tests/e2e.sh
- [x] Given an `ignore:` entry outside the yass.yaml's folder, naming that folder itself, or naming a folder that doesn't exist, when `yass status` runs, then it warns — verify: tests/e2e.sh
- [x] Given a yass.yaml without `path:` next to a `yass/` folder, when `yass status` runs, then `yass/` is the root, the settings apply, and nothing warns — verify: tests/e2e.sh
- [x] Given the README, when someone looks up yass.yaml, then it documents `ignore:` and a yass.yaml that sits next to a `yass/` folder only to hold settings — verify: read README.md
- [x] Given this repo, when `yass status` runs from the top, then only this project's `yass/` shows, not the examples — verify: manual

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `ignore:` in `Config`, resolved by `ignoredDirs` (config.go)
- [x] `findRoots` drops ignored folders, settling nested yass.yaml files shallowest first, and treats an ignored folder you're standing in as the top (repo.go)
- [x] The hook sees the repo from the top and treats ignored files as ordinary files (hook.go)
- [x] e2e section 10
- [x] README: "Folders that aren't yours", and a yass.yaml that only holds settings
- [x] This repo's `yass.yaml` ignores `examples/*`

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- Standing inside an ignored folder makes it the top, a project of its own - examples stay browsable with `cd examples/solo-app && yass status`, and the outer project's changes don't leak in (claude)
- Entries are folders or glob patterns (`examples/*`) - with plain folders, `examples` would make all the examples one project from inside; a pattern makes each its own without listing them (claude)
- Only folders inside the yass.yaml's folder can be ignored; a yass.yaml inside an ignored folder doesn't count, and neither do its ignores - keeps ignores local and order-independent (claude)
- An entry that matches nothing warns - catches typos; ignoring a folder that might appear later isn't worth a silent typo (claude)
- The hook always discovers from the top - git's paths are relative to it, so running it from inside an ignored folder must not move the top (claude)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->

### 2026-10-03 (claude)
- Did: built `ignore:` (folders and glob patterns), the hook's handling, e2e section 10 (188 passing), README docs; this repo's yass.yaml ignores `examples/*`.
- Next: review, then archive.


## yass/archive/2026/10/2026-10-03-order-and-dependencies/2026-10-03-dependencies-through-blocked/change.md

---
platforms: [all]
source: 
follows: 
blocked:
---
# Dependencies through blocked:

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
blocked: can name the changes a change waits on; yass status shows what it's waiting on until they're done, and yass archive respects it.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [x] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC9, AC10, AC11, AC12, AC13, AC14 and AC15 of the plan.

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] Resolve `blocked:` items to changes across yass folders; all resolve means dependencies
- [x] `waiting on:` in `yass status`, and the clear-it hint in `yass status <change>`
- [x] `yass archive` refuses only on a reason or an unmet dependency
- [x] Warn about unresolved dated names and cycles
- [x] e2e checks
- [x] README, `yass-work`, `yass-plan`, `yass-status`, monorepo example

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- Names resolve by folder name; a path only breaks a tie between changes that share a name - a path written while a change was active (`yass/changes/…`) has to keep working after it's archived; the e2e caught this (claude)
- A dependency item is a single word - so free text that happens to contain a path ("waiting on legal and services/…/2026-…") stays a reason instead of clearing itself (claude)
- A dated-looking name that isn't a change warns and leaves the value a reason - a typo should keep the change blocked, not silently unblock it (claude)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->

### 2026-10-03 (claude)
- Did: `blocked:` naming changes is a dependency (`waiting on:` in status, clear-it note in the detail view, archive refuses only while waiting); warnings for unknown dated names and cycles; e2e section 12 (219 passing), examples pass; README "Waiting on another change", `yass-work`, `yass-plan`, `yass-status`, monorepo example.
- Next: review, then archive order-and-dependencies.


## yass/archive/2026/10/2026-10-03-order-and-dependencies/2026-10-03-order-changes-with-queue-md/change.md

---
platforms: [all]
source: 
follows: 
blocked:
---
# Order changes with queue.md

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
A yass folder's optional queue.md ranks its changes, yass status follows it, yass archive keeps it tidy, and the hook treats reordering as intent.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [x] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC1, AC2, AC3, AC4, AC5, AC6, AC7 and AC8 of the plan.

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] Parse `queue.md` per yass folder (list items only, notes ignored), with a unit test
- [x] Sort each folder's changes in `yass status` by queue position, unlisted after by date
- [x] Warn about unknown, archived, piece and duplicate entries
- [x] `yass archive` drops the change's line and stages it
- [x] Hook: modifying `queue.md` with code is intent
- [x] e2e checks
- [x] README, yass README template, `yass-status`, solo-app example `queue.md`

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- An entry can also be in backticks or a markdown link (`[name](changes/name/)`) - links make the queue clickable on GitHub at no cost to the parser (claude)
- `yass archive` removes the line without renumbering the rest - other lines stay byte-for-byte, and markdown renders ordered lists in sequence anyway (claude)
- No marker in `yass status` for where the ranked changes end - the order is the signal; the file says the rest (claude)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->

### 2026-10-03 (claude)
- Did: `queue.md` parsing (unit test), ordering in `yass status`, warnings, `yass archive` cleanup, hook rule; e2e section 11 (203 passing), examples pass; README "The queue", yass README template and copies, `yass-status`, solo-app example `queue.md`.
- Next: review; then the dependencies piece.


## yass/archive/2026/10/2026-10-03-order-and-dependencies/change.md

---
platforms: [all]
source: 
follows: 
blocked:
archived: 2026-10-04T02:28:40Z
---
# Order and dependencies

## Goal
<!-- Intent. One paragraph; the detail is in prd.md (why and what) and plan.md (how and acceptance). -->
A project can say what order it wants to tackle its changes in, and which changes have to wait for others, with one optional file and the existing `blocked:` key. `yass status` lists work in that order and says what each change is waiting on, so "what's next?" has an answer that lives in the repo.

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)" for calls that span the whole change. -->
- Order lives in one file per yass folder, not in each change - reordering is a one-file diff that reads as a decision, with no merge conflicts across folders (Jaime, claude)
- Dependencies reuse `blocked:` instead of a new `needs:` key - one key for "can't proceed"; a value naming changes clears itself, anything else stays a reason for a human (Jaime)
- The file is `queue.md`, not `next.md` - `next:` already means the Log's last Next line in `yass status`; `backlog.md` reads as a graveyard (claude)
- A dependency is met when the change it names is done, not only when it's archived - progress travels with the code, so done boxes on main mean the code is there; waiting for the archive commit adds nothing (claude)
- Entries name change folders exactly, not shortened names - a short name can become ambiguous when a later change is created; exact names stay stable (claude)

## Log
<!-- Progress. Change-level notes; each piece keeps its own Log.
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->


## yass/archive/2026/10/2026-10-03-order-and-dependencies/plan.md

# Order and dependencies: plan
<!-- Intent: how, and how we'll know it's done. With code, only mark boxes here ([/] in progress, [x] done);
     anything else is a plan revision in its own commit. -->

## Approach
**Order.** `Root` gains the parsed `queue.md`: the folder names in list order. A line counts when it's a list item (`-`, `*`, `+` or `1.`) whose first word is a folder name; everything after it (a ` — note`) and every other line is ignored. Reading happens in `loadRepo` next to loading `changes/`; the checks for R3 go in `Repo.check()` beside the `follows:` check. `cmdStatus` sorts each root's items by queue position, unlisted ones after, keeping today's date order among them (a stable sort). Nothing new is printed: the order itself is the signal.

`cmdArchive` rewrites `queue.md` without the archived change's line and `git add`s it with the move, so the archive commit carries both. The hook's `where()` gets a `queue` kind for `<yass folder>/queue.md`, and `check()` flags a modified (not added) `queue.md` in a commit that has code, the same way it treats `prd.md`.

**Dependencies.** A helper on `Change` splits `blocked:` on commas and resolves each item by its last path segment (or `<change>/<piece>`) against every active and archived change in the repo, across yass folders, the way `follows:` already resolves. If every item resolves, the change depends on them, and `waitingOn()` returns the ones that aren't done or archived; otherwise `blocked:` is a reason, exactly as today. `statusLine`, `showChange` and `cmdArchive` switch from `blocked()` to that distinction. `check()` warns about dated-looking items that don't resolve, and walks dependencies for cycles.

**Docs and playbooks** move with each milestone's code, so each piece leaves the docs true.

## Acceptance
### M1
- [x] AC1 (R1, R2) Given a `queue.md` listing two changes in the reverse of their date order, when `yass status` runs, then they're listed in queue order, followed by the unlisted changes in date order — verify: tests/e2e.sh
- [x] AC2 (R1) Given a `queue.md` with a heading, prose, ordered and unordered items, and ` — note` suffixes, when `yass status` runs, then only the items' folder names affect the order and nothing warns — verify: unit test for the parser
- [x] AC3 (R2) Given no `queue.md`, when `yass status` runs, then the output is the same as before this change — verify: tests/e2e.sh and tests/examples.sh pass unchanged
- [x] AC4 (R3) Given a `queue.md` naming an unknown folder, an archived change, a piece, and one change twice, when `yass status` runs, then each gets a warning — verify: tests/e2e.sh
- [x] AC5 (R2) Given a repo with two yass folders, each with its own `queue.md`, when `yass status` runs, then each folder's changes follow its own file — verify: tests/e2e.sh
- [x] AC6 (R4) Given a ranked change that's done, when `yass archive` runs, then its line is gone from `queue.md`, the other lines are untouched, and the edit is staged with the move — verify: tests/e2e.sh
- [x] AC7 (R5) Given a commit with code that reorders an existing `queue.md`, when the hook runs, then it flags `queue.md` as intent; a commit that creates `queue.md` with code isn't flagged — verify: tests/e2e.sh
- [x] AC8 (R10) Given the README, the yass README template, `yass-status` and the solo-app example, when someone reads them, then they describe `queue.md` and `yass-status` uses it to answer "what's next?" — verify: manual: read them; tests/examples.sh passes

### M2
- [x] AC9 (R6, R7) Given change B with `blocked: <A's folder>` and A in progress, when `yass status` runs, then B shows `waiting on: <A>` — verify: tests/e2e.sh
- [x] AC10 (R6, R7) Given A then becomes done, or is archived, when `yass status` runs, then B shows no block, and `yass status B` suggests clearing `blocked:` — verify: tests/e2e.sh
- [x] AC11 (R6) Given `blocked:` naming two changes, as a path ending in a folder and as `<change>/<piece>`, when only one is done, then B is still waiting on the other — verify: tests/e2e.sh
- [x] AC12 (R6) Given `blocked:` with free text, including text that mentions a change, when `yass status` runs, then it shows `BLOCKED: <text>` as today — verify: tests/e2e.sh
- [x] AC13 (R8) Given B's boxes all done, when `yass archive B` runs, then it refuses while A isn't done and succeeds once A is — verify: tests/e2e.sh
- [x] AC14 (R9) Given `blocked:` naming a dated folder that doesn't exist, or A and B blocked on each other, when `yass status` runs, then it warns — verify: tests/e2e.sh
- [x] AC15 (R10) Given the README, `yass-work`, `yass-plan`, `yass-status` and the monorepo example, when someone reads them, then they describe `blocked: <change>`, and the monorepo example uses it — verify: manual: read them; tests/examples.sh passes

## Pieces
Order first, then dependencies. They're independent, but both change how `yass status` builds its lines, and order is the simpler of the two, so it settles the shape of that code first. Each piece ships its own docs and playbook edits.

## Validation
`go vet`, `go test ./...`, `tests/e2e.sh` and `tests/examples.sh` on Linux and macOS (CI). Then this repo's own `yass status` with a real `queue.md`, ranking the changes that are open at the time.


## yass/archive/2026/10/2026-10-03-order-and-dependencies/prd.md

# Order and dependencies: PRD
<!-- Intent: what should change or exist, and why. Not a description of the current system;
     the code and its docs are that. Edit this file only in commits without code. -->

## Why
Once a project has more than a handful of changes, the question people ask most is "what's next?", and YASS can't answer it. `yass status` lists changes by creation date, which records when an idea arrived, not when anyone wants to work on it.

- **Migrating loses the order.** A repo moving to YASS often has a global todo list or "future work" sections that encode priority by position. Split into change folders, some small and some with full PRDs and plans, that order disappears and has nowhere to go.
- **Dependencies hide in prose.** The solo-app example's status line says "offline-queue first; sync-badge needs it", in a Log entry. The monorepo example sets `blocked: waiting on services/payments/yass/changes/2026-09-10-saved-cards-api`, which someone has to remember to clear when that change lands.
- **Inside a large change, order already works.** `plan.md` says under `## Pieces` what order they go in and why. Between top-level changes there's nothing equivalent.

Popular project management tools solve this with two separate ideas: a manually ranked backlog, and "blocked by" relations. Order is a preference about the whole set and shifts as priorities shift; a dependency is a fact about one change and stays true however you reprioritize. YASS should borrow both concepts.

## Users and outcomes
For anyone directing work in a YASS repo (a person, or an agent put in charge of choosing), and anyone asking where things stand.

- Ranking work, or reordering it, is one edit to one file, and its diff reads as the decision.
- `yass status` answers "what's next?" without opening any change: the first change in order that isn't done and isn't waiting on another.
- A change that waits on another stops showing as waiting once that change is done, with nobody clearing anything.
- A repo that doesn't use either feature sees no difference.

## Requirements
- **R1** [M1] A yass folder can hold an optional `queue.md`: a markdown list of change folder names, top first, each optionally followed by ` — <note>`. Other lines (headings, prose) are allowed and ignored.
- **R2** [M1] `yass status` lists each yass folder's active changes in `queue.md` order, then the ones it doesn't list, by date as today.
- **R3** [M1] `yass status` warns about a `queue.md` entry that isn't an active change in that yass folder (unknown, archived, or a piece), and about one listed twice.
- **R4** [M1] `yass archive` removes the archived change's line from `queue.md` and stages that edit with the move.
- **R5** [M1] The hook treats editing an existing `queue.md` in the same commit as code as an intent change. Creating one is fine, as with other intent files.
- **R6** [M2] When `blocked:` holds one or more change names (comma-separated; a name can be a path ending in the change's folder, or `<change>/<piece>`), the change is waiting on them. It's waiting until each is done (every box checked) or archived. Any other value is a reason for a human, as today.
- **R7** [M2] `yass status` shows `waiting on: <change>` for a dependency that isn't met yet, and nothing once it's met. `yass status <change>` suggests clearing a `blocked:` whose dependencies are all met.
- **R8** [M2] `yass archive` refuses a change that's blocked by a reason or by an unmet dependency, and doesn't refuse one whose dependencies are met.
- **R9** [M2] `yass status` warns when a `blocked:` entry looks like a change folder (`<YYYY-MM-DD>-<slug>`) but no such change exists, and when dependencies form a cycle.
- **R10** [M1, M2] The README, the yass folder README template, the playbooks and the examples cover both: `yass-status` answers "what's next?" from the order and dependencies; `yass-work` sets `blocked: <change>` when work has to wait on another change; `yass-plan` can use it between pieces; the solo-app example has a `queue.md`, and the monorepo example's dependency uses the new form.

## Non-goals
- **Priority levels** (urgent/high/medium/low). Redundant once there's an order, and they inflate until everything is high.
- **A rank stored in each change.** Reordering would touch many folders and conflict on merge.
- **Estimates, points, due dates, cycles or sprints, assignees.**
- **Relation types beyond "waits on"** (relates to, duplicates). A Log line or a Decision covers them.
- **Ordering across yass folders.** Each yass folder ranks its own changes; cross-team order is the root change's plan, as today.
- **A CLI command to reorder.** Editing `queue.md` is the interface.
- **Picking work automatically.** `yass-work` still works only on the change it's pointed at; the order informs whoever directs the work.
- **Ordering pieces with `queue.md`.** A large change's `plan.md` already orders its pieces.

## Milestones
| Milestone | When it ships, a user can … |
|---|---|
| M1 Order | rank changes in `queue.md` and see `yass status` follow it |
| M2 Dependencies | point `blocked:` at another change and see it clear itself when that change is done |

## Open questions
None. The calls made while shaping are in `change.md` `## Decisions`.


## yass/archive/2026/10/2026-10-03-upgrade-message-says-upgrade/change.md

---
platforms: [all]
source: 
follows: 
blocked:
archived: 2026-10-04T02:32:20Z
---
# Upgrade message says upgrade

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
Running install.sh --upgrade ends with next steps for an upgrade, not the first-time adoption steps.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Given a repo with YASS installed, when `install.sh --upgrade` runs, then it says YASS is upgraded and suggests a `chore: upgrade YASS` commit, not `chore: adopt YASS` — verify: tests/e2e.sh
- [x] Given a repo without YASS, when `install.sh` runs, then the first-time next steps are unchanged — verify: tests/e2e.sh

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] Branch the closing message in install.sh on --upgrade
- [x] e2e checks for both messages

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->

### 2026-10-03 (claude)
- Did: install.sh ends an upgrade with upgrade steps (`chore: upgrade YASS`); e2e checks both messages (222 passing).
- Next: archive.


## yass/archive/2026/10/2026-10-03-upgrades-refresh-installed/change.md

---
platforms: [all]
source: 
follows: 
blocked:
archived: 2026-10-04T02:42:02Z
---
# Upgrades refresh what's installed

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
Upgrading YASS updates every copy of the playbooks a repo already has, without having to remember the flags it was installed with, and tells people where to read what changed. The upgrade path, including the one-line form, is documented.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [x] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Given a repo installed with `--claude`, when `install.sh --upgrade` runs without `--claude`, then `.claude/skills/yass-*` are replaced too — verify: tests/e2e.sh
- [x] Given a repo without `.claude/skills/yass-*`, when `install.sh --upgrade` runs without `--claude`, then none are created — verify: tests/e2e.sh
- [x] Given playbooks installed with `--global` (none in the repo, some in the user folder), when `install.sh --upgrade` runs without `--global`, then it replaces the user folder's copies, says so, and writes none into the repo — verify: tests/e2e.sh
- [x] Given any upgrade, when it finishes, then its message points to the release notes for what changed and how to migrate — verify: tests/e2e.sh
- [x] Given docs/install.md, when someone looks up upgrading, then it says the flags are detected, shows the one-line upgrade, and says to read the release notes — verify: manual: read docs/install.md "Upgrading"

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `--upgrade` detects `.claude/skills/yass-*` in the repo and refreshes them
- [x] `--upgrade` with no playbooks in the repo but some in the user folder refreshes those
- [x] Release notes link in the upgrade message
- [x] e2e checks
- [x] docs/install.md "Upgrading", and the README's install paragraph

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- Detect what's installed instead of recording install flags in a file - the installed copies already say where the playbooks live; a record could drift from them (claude)
- Out of scope for now: stamping a version into the playbooks so `yass status` can warn when they don't match the binary - more machinery; revisit if mismatches bite (claude)
- Repo playbooks win over the user folder's when both exist - project scope is the default, and a repo that has its own copies chose them (claude)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->

### 2026-10-03 (claude)
- Did: `install.sh --upgrade` refreshes `.claude/skills` copies and, when the repo has no playbooks, the user folder's; the upgrade message links the release notes; e2e checks (229 passing); docs/install.md "Upgrading" and the README.
- Next: archive.


## yass/archive/2026/10/2026-10-04-change-names-end-on-a-whole-word/change.md

---
platforms: [all]
source: CI fixes on 2026-10-04: `yass new` cut "…compare through symlinks" to `…-compare-throu`, and a Log then quoted the wrong name
follows: 
blocked:
archived: 2026-10-04T21:32:30Z
---
# Change names end on a whole word

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
A long title gives a change folder name that ends on a whole word (cut at the last hyphen within 40 characters), so names stay readable and are easy to quote correctly in Logs and commit messages.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Given a title whose slug is longer than 40 characters, when you run `yass new`, then the folder name ends at the last whole word that fits, with no trailing hyphen — verify: go test TestSlug; e2e
- [x] Given a single word longer than 40 characters, then the slug is cut at 40, as before — verify: go test TestSlug
- [x] Given changes created with the old cut (`…-compare-throu`), then `yass status --archived` still lists them as before — verify: e2e

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] Cut `slug` at the last hyphen within the limit
- [x] Go test and e2e checks

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- A hyphen right after the limit counts as a whole-word cut - a 40-character slug followed by a hyphen keeps all 40 (claude)
- A single word longer than the limit is still cut at the limit - there's no whole word to keep (claude)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-04 (claude)
- Did: `slug` cuts at the last hyphen within 40 characters. Go tests for the two titles that were cut mid-word today, the exact-40 boundary and a long single word; e2e §23 (a new change, and an archived one named the old way still listing by name). Checks: go test, go vet, gofmt, e2e 281/281, examples.
- Next: archive.



## yass/archive/2026/10/2026-10-04-install-say-where-to-run-it-and-how-yass/change.md

---
platforms: [all]
source: Jaime, 2026-10-04: the curl install assumes ~/.local/bin is on PATH (it wasn't on Jaime's Mac) and doesn't say it must run inside your repo
follows: 
blocked:
archived: 2026-10-05T02:41:00Z
---
# Install: say where to run it and how yass gets on your PATH

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
Someone installing YASS knows two things before they start: the install runs from inside the repo they want to set up (or points at it), and the binary goes to `~/.local/bin`, which has to be on their PATH. If it isn't, `install.sh` ends with the line that adds it. Every install path says this: the README, `docs/install.md`, the agent prompt and the landing page. For zsh, that line goes in `~/.zshenv`, which every zsh reads, including the non-interactive shells agents and git hooks run, rather than `~/.zshrc`, which they skip. The docs and the page also make the two parts of an install plain: the `yass` binary goes on your PATH once per machine, and the repo setup (playbooks, the `AGENTS.md` section, the yass folder, the optional hook) happens once per repo and is committed, so teammates get it from git. With that model, the docs cover setting up another repo, joining a repo that already uses YASS (just the binary), and upgrading (the binary once, then each repo).

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Given the README, `docs/install.md` and the landing page, then each install method that passes `--bin-dir ~/.local/bin` says that folder has to be on your PATH, and that `install.sh` ends with the line to add it if it isn't — verify: manual: read each install method in all three
- [x] Given the one-line install in the README, `docs/install.md` and the page's Terminal tab, then it runs from inside your repo: `cd path/to/your-repo` comes first — verify: `tests/site.sh` (the page's block matches `docs/install.md` word for word); manual: the README
- [x] Given zsh, and a `--bin-dir` that isn't on PATH, when `install.sh` finishes, then the fix it prints appends to `~/.zshenv`, not `~/.zshrc` — verify: e2e (the PATH-warning checks)
- [x] Given `docs/install.md`, then it says why the line goes in `~/.zshenv`: agents and git hooks run shells that don't read `~/.zshrc` — verify: manual: review
- [x] Given the agent install prompt, then it has the agent check that the folder it installs to is on PATH and, if not, show the line to add, without editing shell files itself — verify: `tests/site.sh` (the page's copy matches `docs/install.md`); manual: review
- [x] Given the landing page's Terminal tab, then it says the command does two things: installs `yass` to `~/.local/bin`, once per machine, and sets up this repo, once per repo, committed — verify: manual: read the tab
- [x] Given `docs/install.md`, then it opens with the two parts: the binary, once per machine on your PATH, and the repo setup, once per repo and committed — verify: manual: review
- [x] Given `docs/install.md`, then it shows how to set up another repo when `yass` is already on your PATH: `install.sh` without `--bin-dir` — verify: manual: review; e2e (`install.sh` uses the `yass` on PATH)
- [x] Given `docs/install.md`, then it shows how to join a repo that already uses YASS: only the binary, from a release archive or `go install`, and no `install.sh` — verify: manual: review; run the release steps once by hand
- [x] Given the README, then its install section states the two parts in a sentence or two and links to `docs/install.md` for another repo and for joining one — verify: manual: review
- [x] Given the README, then it has an Upgrading section: upgrade the binary once per machine, run `install.sh --upgrade` in each repo and commit it, teammates upgrade their binary when the repo's playbooks move ahead, with a link to `docs/install.md#upgrading` — verify: manual: review; the link's heading exists
- [x] Given the change, then the existing e2e suite, `tests/examples.sh` and `tests/site.sh` pass — verify: CI on the branch

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `install.sh`: zsh fix goes to `~/.zshenv`; e2e expectation
- [x] `docs/install.md`: `cd` first in the one-liner, PATH notes, why `~/.zshenv`, agent prompt step
- [x] README: the same, briefly, in the install section
- [x] Landing page: Terminal tab and notes, agent prompt copy
- [x] `docs/install.md`: the two parts up front; another repo; joining a repo
- [x] README: the two parts; an Upgrading section

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- A separate small change, not part of the landing page change - that change rules out README edits beyond its lead-in and changes to how YASS works; this touches both (Jaime)
- The page's Terminal block now cites `docs/install.md`, whose one-liner has the `cd` line; the README keeps the one-liner inline and says "from inside your repo" instead - the README stays short, and the page still has a word-for-word source (claude)
- The agent prompt tells the agent to show the PATH line, not edit shell files - matches the prompt's ask-before-changing tone, and dotfiles are the user's (claude)
- Bash keeps `~/.bash_profile` (macOS) and `~/.bashrc`: non-interactive bash reads neither unless it's a login shell, and there's no single file that fixes that, so this change only fixes zsh, the macOS default (claude)
- Folded in after review: explain the two parts of an install (the binary per machine, the repo setup per repo), cover another repo and joining one, and give upgrading its own README section; the landing page doesn't cover upgrading. Docs only - actually splitting the two steps (the binary carrying the playbooks) is a future large change, noted separately (Jaime)
- "Another repo" leads with the one-liner without `--bin-dir`, the only form that uses the `yass` already on PATH; from a clone or an unpacked release, `install.sh` prefers the binary next to it (and warns about the one on PATH), so those keep `--bin-dir` - checked both by running them in throwaway repos (claude)
- Upgrading other repos: the same command again, or the one-liner without `--bin-dir` - `--upgrade` without `--bin-dir` from an unpacked release would hit the same spurious warning (claude)
- Clarify and fix the hint, don't change where the binary goes: `~/.local/bin` stays the documented default, and `install.sh` doesn't edit shell files itself - picking a folder that's already on PATH, or writing to dotfiles, is a bigger behavior change than this needs (claude)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->

### 2026-10-04 (claude)
- Did: `install.sh` suggests `~/.zshenv` for zsh (e2e updated); `docs/install.md` says up front to run it for the repo you're setting up and that the `--bin-dir` folder has to be on PATH (and why `~/.zshenv`), puts `cd path/to/your-repo` before the one-liner, and has the agent prompt show the PATH line; the README says the same, briefly; the landing page's Terminal and From Source tabs carry the `cd` line and PATH notes, and its agent prompt copy matches. Local: e2e 281 passed, examples pass, `tests/site.sh` 51 passed; checked the Terminal tab in the browser.
- Next: Jaime reviews the branch `install-path-hazards`; CI runs once it's pushed. After it merges, archive this change.

### 2026-10-04 (claude)
- Did: the folded-in scope (revision 1e5ba55, under the code). `docs/install.md` sharpens its two-part opening (`--bin-dir` is the binary half, the repo path the files half) and adds "Another repo" and "Joining a repo that already uses YASS"; its Upgrading section names the two parts and covers other repos. The README states the two parts, links to both new sections, and gets an Upgrading section. The page's install intro and Terminal tab say what happens where, with links to another repo and joining. Ran each flow in throwaway repos: the piped one-liner without `--bin-dir` sets up a repo from the `yass` on PATH, with no warning; `./install.sh` from a clone without `--bin-dir` uses the clone's binary and warns, hence the wording; joining (download the latest release, verify the checksum, copy the binary) works, and `yass status` runs. `tests/site.sh`: 53 passed.
- Next: Jaime reviews the branch `install-path-hazards`; CI runs once it's pushed. After it merges, archive this change.

### 2026-10-04 (claude)
- Did: merged to main and pushed (e5fe20a..8da5f0a); CI and `pages` passed on 8da5f0a, and the live page shows the new install copy.
- Next: done; archive it.


## yass/archive/2026/10/2026-10-04-monorepos-at-scale-and-plans-repos/2026-10-04-agent-guidance-for-separate-plans/change.md

---
platforms: [all]
source: 
follows: 
blocked: 2026-10-04-check-from-the-plans-repo
---
# Agent guidance for separate plans

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
The skills give agents the whole routine for separate plans: state checks at session start, commit order with trailers, cross-links, per-repo git latitude, worktrees.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC55, AC56 in [plan.md](../plan.md)
- [x] The README's separate-plans section and `docs/skills.md` match the skills

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] AC55: `yass-work`'s "Plans in another folder" is the whole routine: pull the plans first; stop, never `yass init`, when they're missing (worktrees already look in the clone's other checkouts); `yass use` so the hooks add the trailer; code first, then plans, citing (`repo@sha` too); `[x]` after the merge; cross-link with `--brief` and the trailer line; latitude per repo
- [x] `yass-log` and `yass-status` find a change's code by trailers (`yass status <change>`, `%(trailers:key=yass)`)
- [x] README bullet and `docs/skills.md` match; e2e (700 ok) still finds the installed skills
- [x] Jaime reviews the wording (AC55, README, docs/skills.md)
- [x] AC56: one fresh agent session in a linked worktree of a code repo with separate plans, told "keep going" on a change (run with Jaime, 2026-10-11; see Log)
- [x] Its findings: the plans repo with no upstream gets a sentence in the routine; the trailer hooks no longer block commits under an older `yass` (#74); the mixed inline-and-separate case is for Jaime (see Log)

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- AC56 ran on this repo, as Jaime proposed: it has an inline `yass/` and a separate private plans repo, so the run exercised the routine against a real separate plans repo; trailers are off in a mixed repo by design, so the agent added one by hand (claude, 2026-10-11)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-10 (claude)
- Did: AC55: the separate-plans routine in `yass-work`, trailers in `yass-log` and `yass-status`, README and `docs/skills.md`. `tests/e2e.sh` (700 ok), `tests/site.sh` pass.
- Next: Jaime reviews the wording, and runs AC56: a fresh agent session in a linked worktree of a code repo with separate plans, told "keep going"; it should pull the plans, build, commit code with the trailer, record progress in the plans repo, and never run `yass init`.
### 2026-10-11 (claude)
- Did: AC56 with Jaime. Setup: a small real change (`2026-10-11-test-brief-output`, a go test for `--brief`) committed in Jaime's private plans repo (no remote); a linked worktree of this repo on a throwaway local branch, its playbooks refreshed from a development build so the agent read this piece's routine; a fresh general-purpose subagent told only "Keep going on the change 2026-10-11-test-brief-output", with where the worktree and the development `yass` were and that it could commit but not push.
- The run: it read AGENTS.md and the worktree's `yass-work`; ran `yass root -v` (separate), tried `git -C <plans> pull --ff-only` (failed: no upstream; it carried on), `yass status`, `yass use 2026-10-11-test-brief-output`; wrote `TestBrief` and broke `brief()` on purpose twice to see the test catch it; committed the code first (98fc050, with `yass: 2026-10-11-test-brief-output`), then the plans (2e055d9: both criteria `[/]` citing `code: 98fc050`, a Decision, a Log entry, Next: merge, then `[x]` and archive). It never ran `yass init`. All checked against both repos.
- Findings: (1) in a repo with both inline and separate plans, trailers are off, so the hook added none and it wrote the trailer by hand - Jaime to decide whether a change in a separate folder should get one; (2) no upstream to pull - now a sentence in the routine; (3) the Skill tool served the main checkout's older `yass-work`; it noticed and read the worktree's; (4) setting up showed the trailer hooks blocking commits under an older `yass` - fixed in #74; (5) a dead guard in `brief()` - removed in #74.
- Next: Jaime reviews the wording; the mixed-repo trailer question.
### 2026-10-11 (claude)
- Did: Jaime reviewed the wording and merged it (#73); the commit with the no-upstream sentence and the AC56 record missed that merge and is carried over here. The piece is done, and with it every piece of the change.
- Next: archive `monorepos-at-scale-and-plans-repos` once #74 (which logs into piece 9's folder) has merged.


## yass/archive/2026/10/2026-10-04-monorepos-at-scale-and-plans-repos/2026-10-04-branches-view/change.md

---
platforms: [all]
source: 
follows: 
blocked: 2026-10-04-pieces-own-their-files
---
# Branches view

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
yass status shows by default who is working on which change or piece on unmerged local and pushed branches, and how far they are, as of the last fetch, without leaving git. --since sets the window and --no-branches turns it off.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC18–AC22 in [plan.md](../plan.md)
- [x] The README's `yass status` section documents the branch view, `--since` and `--no-branches`
- [x] `docs/monorepo.md` recommends `git commit-graph write --changed-paths` (or `fetch.writeCommitGraph`) and a recent git for large repos, and explains what the branch view reads and how fresh it is

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `branches.go`: recent unmerged branches (`for-each-ref --no-merged`, the window from `--since`), one `git log --source` over them limited to the yass folders' `changes/`, and the `change.md`/`plan.md` at each tip from one `cat-file --batch` with `GIT_NO_LAZY_FETCH` (or no read at all, in a partial clone with git before 2.44); the freshness line from `FETCH_HEAD`
- [x] `yass status` shows each change's and piece's branches under it, and the freshness line; `--since 14d|DATE`, `--no-branches`
- [x] e2e §41 against a remote that's a local folder (610 ok): local and pushed branches with author, progress and Next; code-only, merged and 40-day-old branches left out; `--since`; `--no-branches` running no lookup; no fetch, also in a blobless clone (shown without progress); no remote; no unmerged branches
- [x] `tests/bench.sh` with 100 local and 1,000 remote branches made by `git fast-import`: 849–929ms with 100 (target < 1s, AC16), 1,480–1,579ms with 1,000 more (target < 2s, AC22)
- [x] README and `docs/monorepo.md` (what it reads, freshness, commit-graph, partial clones, timings)
- [x] Jaime reviews the docs

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- The local copy of the merged branch (`main` when it's `origin/main`) is left out: its unpushed commits aren't someone's work in progress (claude, 2026-10-09)
- A branch's progress is the boxes in the folder's `change.md` and `plan.md` at its tip; other files there aren't read - two blobs per row in one `cat-file`, and they hold all of a change's boxes in practice (claude, 2026-10-09)
- A Next: inside the template's HTML comment isn't taken for one; the e2e caught `next: … -->` (claude, 2026-10-09)
- The benchmark's `follows:` line had been broken since `archive-names-only`: the Perl edit ate the newline (`follows: 2025-02-02-done-1blocked:`), so every run since resolved 200 unknown names, scanned the archive for pieces and printed 200 warnings. Fixed; the numbers above are with it fixed. The first timing after generating the repo can run high (1,045ms once) while the OS settles the new files; reruns gave 900–935ms. Headroom under 1s is about 70–100ms, most of it git's untracked-file walk (claude, 2026-10-09)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-09 (claude)
- Did: the branch view in `yass status`, `--since`, `--no-branches`; e2e; the bench with 100 and 1,000 branches; docs. `go test`, `tests/e2e.sh` (610 ok), `tests/examples.sh`, `tests/site.sh`, `tests/bench.sh` pass.
- Next: Jaime reviews the docs; then this piece is done.
### 2026-10-09 (claude)
- Did: Jaime reviewed the docs and merged it (#65). The piece is done.
- Next: none here.


## yass/archive/2026/10/2026-10-04-monorepos-at-scale-and-plans-repos/2026-10-04-check-from-the-plans-repo/change.md

---
platforms: [all]
source: 
follows: 
blocked: 2026-10-04-repo-qualified-citations, 2026-10-04-yass-trailers
---
# Check from the plans repo

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
A plans repo maps repo names to code checkouts, so yass status there, locally or in CI, checks every citation.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC53, AC54 in [plan.md](../plan.md)

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `repos:` in a plans repo's `yass.yaml`, resolved like `path:` (`${VAR:-default}` included); `checkCode` checks each citation against its repo's checkout and main branch (`citationIn`), an unqualified one against the only repo listed
- [x] Missing checkouts, unset variables and unplaceable unqualified citations are notes, so `--strict` passes
- [x] e2e §47 (700 ok); `docs/separate-plans.md` "Checking from the plans repo"; README settings

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- A code checkout's main branch is its `origin/HEAD`, else `main` or `master`, read in that checkout; the plans repo's own `branch:` doesn't apply to it (claude, 2026-10-10)
- R17 also mentions trailers; checking a code repo's trailers stays with that repo (`yass hook --range` there), since the plans repo has no commits of the code's to read - AC53 and AC54, the piece's criteria, are about citations (claude, 2026-10-10)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-10 (claude)
- Did: `repos:` and checking a plans repo's citations against each code checkout. `go test`, `tests/e2e.sh` (700 ok), `tests/examples.sh`, `tests/site.sh`, `tests/bench.sh` (status 811ms, all targets) pass. The piece is done.
- Next: none here; piece 11 (`agent-guidance-for-separate-plans`) can start.


## yass/archive/2026/10/2026-10-04-monorepos-at-scale-and-plans-repos/2026-10-04-pieces-own-their-files/change.md

---
platforms: [all]
source: 
follows: 
blocked:
---
# Pieces own their files

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
Parallel branches on different pieces of one change never touch the same YASS file: each piece's progress stays in its folder, the hook flags commits that reach outside it, and the parent needs no closing commit.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC10, AC11, AC14 in [plan.md](../plan.md)
- [x] The README's section on large changes explains the rule, and that the parent needs no closing commit

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] Hook: with code, a commit that changes a large change's own files (its `plan.md`, `change.md`, or anything outside its pieces, `assets/` included), or files in two of its pieces, gets a heads-up naming them; new files are fine, and commits without code are left alone
- [x] e2e §40: two branches on different pieces rebase onto each other without conflicts (AC10); each heads-up, and the commits that get none (AC11) (593 ok)
- [x] `yass-plan`, `yass-work` and the README say a piece's branch changes only that piece's folder and the parent needs no closing commit (AC14)
- [x] This piece's own progress follows the rule: only this folder changes; AC10, AC11 and AC14 count as delivered through the `Delivers` box
- [x] Jaime reviews the wording (AC14, README)

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- A parent's subfolder that isn't a piece (no `change.md`, like `assets/`) counts as the parent's own files - the e2e caught the first version letting it through silently (claude, 2026-10-09)
- New files are exempt, as everywhere else in the hook: creating a piece or a change alongside code isn't reaching into another piece's progress (claude, 2026-10-09)
- `--range` decides which folders have pieces from the working tree, so it doesn't flag history in changes since archived; the commit hook, which checks the commit being made, isn't affected. Over v0.3.0..main it flags one commit: #62, which ticked the parent's plan.md with code (claude, 2026-10-09)
- A large change without pieces still ticks its plan criteria alongside code; `yass-work` keeps saying so (claude, 2026-10-09)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-09 (claude)
- Did: the hook's heads-up for commits with code that reach past one piece; e2e for parallel pieces; the wording in `yass-plan`, `yass-work` and the README. `go test`, `tests/e2e.sh` (593 ok), `tests/examples.sh`, `tests/site.sh` pass.
- Next: Jaime reviews the wording (AC14, README); then this piece is done.
### 2026-10-09 (claude)
- Did: Jaime reviewed the wording and merged it (#64). The piece is done.
- Next: none here.


## yass/archive/2026/10/2026-10-04-monorepos-at-scale-and-plans-repos/2026-10-04-repo-qualified-citations/change.md

---
platforms: [all]
source: 
follows: 
blocked: 2026-10-04-separate-plans-mode
---
# Repo-qualified citations

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
Several code repos can share one plans repo without false warnings, citing code as repo@sha, and yass archive refuses changes whose done boxes cite missing or unmerged code.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC34–AC39 in [plan.md](../plan.md)

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `codeRefs` reads `repo@sha` as well as a bare sha (`codeRef`); `yass decisions --cites` matches either by sha
- [x] The code repo's name: `repo:` in its `yass.yaml`, else its `origin` URL's last segment, else its main worktree's folder (`repoName`, `repoFromURL`); `repo:` is a known setting
- [x] `checkCode` checks only this repo's citations and notes how many it left to each other repo; one `citation` check shared with `yass archive`
- [x] `yass archive` refuses done boxes citing a missing or unmerged commit here (`--force` overrides), and lists the citations it couldn't check, with where to
- [x] go test (`TestCodeRefs` with qualified forms, `TestRepoFromURL`); e2e §43 with two code repos cloned from bare origins sharing a plans repo (636 ok); the existing §9 single-repo checks unchanged (AC37); README

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- A citation that names another repo is a note per repo ("2 citation(s) name web, so they're checked from there"), not a warning: it's not wrong, just not ours to check (claude, 2026-10-10)
- The name from a linked worktree is the main worktree's folder, read through git's common dir, so every checkout of a clone agrees (claude, 2026-10-10)
- A citation token with an invalid repo name (`w*b@…`) is ignored rather than guessed at; `@sha` with no name is taken as unqualified (claude, 2026-10-10)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-10 (claude)
- Did: repo-qualified citations, the repo's name, per-repo checks, and `yass archive`'s citation check. `go test`, `tests/e2e.sh` (636 ok), `tests/examples.sh`, `tests/site.sh` pass. The piece is done.
- Next: none here; piece 9 (`yass-trailers`) can start.


## yass/archive/2026/10/2026-10-04-monorepos-at-scale-and-plans-repos/2026-10-04-separate-plans-mode/change.md

---
platforms: [all]
source: 
follows: 
blocked:
---
# Separate plans mode

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
YASS knows when plans live outside the code repo, says so, and notes when the plans repo has uncommitted changes or is behind.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC30–AC33 in [plan.md](../plan.md)

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `separate.go`: separate when the yass folder's git common dir isn't the code repo's (another repo, or none); a linked worktree counts as the same repo
- [x] `yass status` ends with `plans: separate (<folder>)` per separate folder, and notes uncommitted files there and how far it's behind its upstream as of its last fetch (no fetch); inline plans add nothing
- [x] `yass root -v` prints `inline` or `separate` after each folder; one-letter switches (`-v`) are accepted where a command has one
- [x] e2e §42 (622 ok): inline (and from a linked worktree), another repo, uncommitted files, behind upstream without fetching, no repo; README

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- "Same repo" compares git's common dir, so plans in the main checkout stay inline from a linked worktree, as design §1 says (claude, 2026-10-09)
- The `plans: separate` line comes after the notes, so it's the last line, as AC31 asks; the uncommitted and behind messages are notes, so `--strict` passes (claude, 2026-10-09)
- `-v` needed the parser to take one-letter switches; it does so only for commands that declare that letter, so nothing else changes (claude, 2026-10-09)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-09 (claude)
- Did: inline or separate plans in `yass status` and `yass root -v`, and the notes for a separate plans repo. `go test`, `tests/e2e.sh` (622 ok), `tests/examples.sh`, `tests/site.sh` pass. The piece is done.
- Next: none here; pieces 8 (`repo-qualified-citations`) and 9 (`yass-trailers`) can start.


## yass/archive/2026/10/2026-10-04-monorepos-at-scale-and-plans-repos/2026-10-04-speed-at-scale/change.md

---
platforms: [all]
source: 
follows: 
blocked:
---
# Speed at scale

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
yass status and the pre-commit hook meet the PRD's speed targets on generated large repos, held there by a benchmark script.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC16, AC17 in [plan.md](../plan.md)
- [x] `tests/bench.sh` generates its repos in a temporary folder and cleans up after itself

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `tests/bench.sh`: a repo with 200 yass folders and 2,000 changes, timed with `yass status` (AC16, without the branch view)
- [x] A repo of 100,000 files with a staged one-file change, timed with `yass hook` (AC17)
- [x] Median of 5 runs after an untimed warm-up; exits 1 on a missed target; `BENCH_KEEP=1` keeps the repos
- [x] CONTRIBUTING lists it
- [x] AC16 again once the branch view exists: 100 recently active branches in the status repo (piece `2026-10-04-branches-view` extends this script)

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- The hook is timed as `yass hook` on a staged change, not as a whole `git commit` - AC17 is about what the hook adds, and timing it alone keeps git's own commit cost out of the number (claude)
- Timings are the median of 5 runs after one untimed warm-up - the first run over freshly generated files measured 1,176ms against 380–410ms afterwards, which was the OS still caching and indexing new files, not yass (claude)
- Not run in CI - shared runners' timings vary too much for pass/fail targets; it's a local check, listed in CONTRIBUTING (claude)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-04 (claude)
- Did: `tests/bench.sh`. On an Apple M5 Mac, macOS 27.0.1: `yass status` with 200 yass folders and 2,000 changes, 384ms and 410ms on two runs (target < 1s); `yass hook` on a 100,000-file repo, 100ms and 102ms (target < 200ms). Generated repos are removed afterwards (checked). AC17 met.
- AC16 isn't finished: its target includes the branch view with 100 active branches, which doesn't exist yet. About 600ms of the budget is left for it. `yass status` spends about two thirds of its time in system calls (reading 2,000 files), so that's where to look if the branch view runs short.
- Next: the branch-view piece adds 100 recently active branches to the status repo here, and reruns; then mark AC16 and the Delivers box.
### 2026-10-09 (claude)
- Did: AC16 with the branch view: 100 recently active branches in the status repo, 849–929ms over three runs (target < 1s); AC17's hook 118–119ms (target < 200ms). The bench's `follows:` fix (see `branches-view`) means earlier status numbers here were measured with 200 unknown names, so ran slow. The piece is done.
- Next: none here.


## yass/archive/2026/10/2026-10-04-monorepos-at-scale-and-plans-repos/2026-10-04-team-scale-docs/change.md

---
platforms: [all]
source: 
follows: 
blocked: 2026-10-04-yass-paths-and-ci-recipes
---
# Team-scale docs

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
A platform owner can roll YASS out across many teams: CODEOWNERS per yass folder, LFS for assets, history filters, and yass status that behaves in sparse checkouts.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC6–AC9, AC15 in [plan.md](../plan.md)

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] Sparse checkouts: a `blocked:` or `follows:` naming a change git has but this checkout doesn't (`notCheckedOut`, from `git ls-files` only when a name doesn't resolve) gets a note naming its yass folder, not a warning
- [x] e2e §39: a cone-mode sparse checkout without `services/search/` (status, `--strict`, `blocked:`, `follows:`, a real typo still warning), and a binary asset archived with its change (585 ok)
- [x] `docs/monorepo.md` "Many teams in one repo": CODEOWNERS, sparse checkouts, LFS for assets, history without the plans (each command run in this repo, in zsh and bash); README points to it
- [x] Jaime reviews the docs (AC6, AC9)

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- A cone-mode sparse checkout already left other teams' yass folders out of `yass status` (AC7 held before any change); only names in `blocked:` and `follows:` needed a note (claude, 2026-10-09)
- The history filters pipe `yass paths` through `xargs`, not `$(…)` - zsh doesn't split an unquoted variable, so the `$(…)` form silently filtered nothing there; checked in zsh and bash (claude, 2026-10-09)
- `git blame` gets an explanation, not a command: a plans-only commit never changes a line of code, so it never appears in a code file's blame (claude, 2026-10-09)
- The box on `git commit-graph` and the branch view's freshness waits: the branch view is piece 6 and doesn't exist yet; proposed moving it there (claude, 2026-10-09)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-09 (claude)
- Did: sparse checkouts say where an unseen change is; e2e for that and for assets; the many-teams docs.
- Next: Jaime reviews the docs (AC6, AC9). The commit-graph and branch-view box should move to `branches-view` (a plan revision).
### 2026-10-09 (claude)
- Did: Jaime reviewed the docs and merged them (#62); the commit-graph box moved to `branches-view` in the revision. The piece is done.
- Next: none here.


## yass/archive/2026/10/2026-10-04-monorepos-at-scale-and-plans-repos/2026-10-04-worktrees-find-the-plans/change.md

---
platforms: [all]
source: 
follows: 
blocked:
---
# Worktrees find the plans

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
In any linked worktree, YASS finds the same plans folder as the main checkout and never suggests yass init there. Paths in yass.yaml accept ${VAR:-default}, and with separate plans each checkout gets a gitignored yass/ symlink to the plans folder.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC23–AC29 in [plan.md](../plan.md)
- [x] Resolving from the main worktree is covered for a bare-repo clone too, or the code says why it isn't

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `${VAR:-default}` in path expansion, with go tests (AC29)
- [x] Resolve a missing `path:` from the other worktrees of the clone (`git worktree list`), main first; covers bare-repo layouts (AC23)
- [x] Missing-folder messages in a linked worktree name the places looked and never suggest `yass init`; `yass init` there refuses rather than creating a stray folder (AC24)
- [x] The `yass/` link: create, repoint, leave real folders alone, skip where symlinks fail, `info/exclude` once, no double counting (AC25–AC27)
- [x] Notes that don't count toward `--strict`
- [x] e2e §16, and §9 updated for the link
- [x] A `yass.yaml` kept out of git: a linked worktree borrows the main checkout's
- [x] README and docs/install.md
- [x] Manual: Claude Code reading and editing through the link (AC28)

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- The `yass/` link is ignored through `.git/info/exclude`, not `.gitignore` - editing `.gitignore` would add a diff to every repo with separate plans (and reveal the pointer in repos that keep `yass.yaml` private), while `info/exclude` is per clone, shared by its worktrees, and exactly as long-lived as the link itself; AC25 reworded to match (claude)
- A linked worktree with no yass folder at all borrows the `yass.yaml` the clone's other checkouts have at the same place - the README supports keeping `yass.yaml` out of git as a private pointer, and then a worktree has none; without this, R11 failed for that setup (claude)
- The "yass/: ignored, because yass.yaml points to…" check also looks on disk, not only at git's file list - `/yass` stays in info/exclude after a link is replaced by a real folder, which would hide that folder from the check (claude)
- Link problems that don't affect the plans (a symlink that can't be made) are notes, printed by status and on stderr by root, and don't fail `--strict`; a real `yass/` in the link's place is a warning, since it shadows the plans (claude)
- A link left from before the plans moved into the repo is reported, never deleted - YASS can't tell its own old link from one the user made (claude)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-04 (claude)
- Did: `${VAR:-default}` in paths; a missing relative `path:` resolves from the clone's other checkouts (bare layouts too); no `yass init` advice from a linked worktree, and `init` there refuses; the ignored `yass/` link (create, repoint, leave real folders, skip on failure, `info/exclude` once); a private `yass.yaml` is borrowed in worktrees; README and install docs. Intent fix on its own commit below this branch: the link is ignored via info/exclude, not .gitignore. Checks: go test, go vet, gofmt, e2e 253/253 (new §16, §9 updated), examples.
- Did: AC28 manual check from this session: reading and editing a file through a link inside the project to a folder outside it both worked, and the edit landed in the target. This session runs in auto mode, so it says nothing about whether a default-permission session prompts for writes through the link; the README says to grant the folder to be safe.
- Next: a human checks AC28 in a default-permission Claude Code session (edit `yass/changes/...` through the link without `--add-dir`; note whether it prompts), then mark it and the Delivers box. Then merge the intent commit and this branch, and move on to piece 2 (`2026-10-04-yass-paths-and-ci-recipes`).
### 2026-10-04 (claude, with Jaime)
- Did: AC28, run by Jaime in a Claude Code terminal session at the repo root, in auto mode (Claude Code's default permission mode), with the plans folder not granted:
  - `yass new` run in `private/` printed the plans folder's real path, so the agent read and edited `~/dev/private/yass-private-plans/...`. The first read asked ("Read outside the working directories"); the edits after it were allowed by auto mode.
  - Reading `private/yass/changes/.../change.md` through the link asked the same question, showing the resolved path behind the link. Claude Code resolves the link, so the link doesn't make the plans count as inside the project.
- Did: the README now says so: the link is for finding the plans; granting the plans folder (`--add-dir`, `permissions.additionalDirectories`) is what lets an agent work there without asking. `yass init` already suggests it. Stricter permission modes weren't tried; they'd only ask more.
- Not changed: having `yass new` print link paths instead of real ones. Without a permission benefit, it isn't worth the code.
- Next: done. Its parent criteria (AC23–AC29) are ready to mark in a closing commit.



## yass/archive/2026/10/2026-10-04-monorepos-at-scale-and-plans-repos/2026-10-04-yass-paths-and-ci-recipes/change.md

---
platforms: [all]
source: 
follows: 
blocked:
---
# yass paths and CI recipes

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
CI can skip plan-only changes with one gate job built on yass paths, with tested recipes for GitHub Actions (including required checks and merge queues), Bazel and a generic CI, and this repo's own CI uses the GitHub one.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC1–AC3, AC57, AC58 in [plan.md](../plan.md)
- [x] `docs/monorepo.md` exists and the README's Monorepos section links to it

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `yass paths`: globs for each yass folder inside the repo and each `yass.yaml`; `**` for a plans repo (AC1)
- [x] `--only <range>`: 0 plans only, 1 anything else or nothing, 2 unreadable range (AC2)
- [x] e2e §13
- [x] `docs/monorepo.md`: why a job and not a path filter, GitHub gate job and `ci-ok`, a no-install diff check, merge queues, Bazel, other CI (AC4, AC5)
- [x] This repo's `ci.yml`: the gate job (`go run ./cmd/yass`), `test`/`build` behind it, `ci-ok` aggregating, `merge_group` added (AC3)
- [x] README: `yass paths` in the CLI list, link to docs/monorepo.md
- [x] AC3 on GitHub: a plan-only push skips `test` and `build` and `ci-ok` passes; a code push runs them
- [-] AC4: the Bazel recipe run against a real Bazel repo (moved to `2026-10-04-validate-the-ci-recipes-in-the-field`)
- [-] AC5: a plan-only pull request through a real merge queue (moved to `2026-10-04-validate-the-ci-recipes-in-the-field`)
- [x] Mark the Bazel and merge-queue sections of `docs/monorepo.md` as untested, with where to report results (AC57, AC58)

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- `yass paths` lists each whole yass folder (`yass/**`), not just the parts the hook classifies (changes/, archive/, queue.md, README.md) - a yass folder is YASS's by definition, and one glob per folder is what CI filters and `.bazelignore` want (claude)
- `--only` exits 1 for a range with no changed files - when in doubt, a gate should run the checks, not skip them (claude)
- Recipes compare `BASE...HEAD` (three dots) - a branch is compared with where it left its base, so commits that landed on the base meanwhile don't count as its changes (claude)
- This repo's CI builds the gate's yass with `go run ./cmd/yass` - no release has `yass paths` yet, and it dogfoods the branch's own version (claude)
- The GitHub facts in the docs come from GitHub's documentation, checked on 2026-10-04: a workflow skipped by a path filter leaves required checks Pending; a job skipped by `if:` reports Success; `merge_group` takes `branches` filters, no `paths` (claude)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-04 (claude)
- Did: `yass paths` and `--only`; e2e §13 (10 checks); `docs/monorepo.md` with the GitHub gate-job recipe (plus `ci-ok`, a no-install variant, merge queues), Bazel and generic recipes; this repo's `ci.yml` gated the same way; README updated. Checks: go test, go vet, gofmt, e2e 263/263, examples. The gate step simulated locally: plan-only commit c7a9f90 → skip, code commit e6bdcbc → run, new branch → run. Workflow YAML parses.
- Not yet verified (needs GitHub or tools not here): AC3 on a real push, AC4's Bazel recipe (no Bazel on this machine), AC5 through a real merge queue. The `merge_group.base_sha` field name comes from the webhook payload and isn't confirmed on GitHub's events page.
- Next: when this branch is pushed, check one plan-only push and one code push in Actions (record the run links here), and consider requiring `ci-ok` on main. Try the Bazel recipe on a repo that uses Bazel, and a merge queue on a scratch repo, or record why not. Then mark AC3–AC5 and the Delivers box.
### 2026-10-04 (claude)
- Did: AC3 verified on GitHub after Jaime had me push main. Plan-only push d5ee308: run https://github.com/donjaime/yass/actions/runs/37235914174, `changes` passed, `test` and `build` skipped, `ci-ok` passed (26s). Code pushes ran everything: https://github.com/donjaime/yass/actions/runs/37235844602 (which caught a macOS-only bug, fixed in `2026-10-04-paths-that-don-t-exist-yet-compare-throu`) and https://github.com/donjaime/yass/actions/runs/37236153589 (all passed).
- Did: AC4 and AC5 moved to `2026-10-04-validate-the-ci-recipes-in-the-field` (Jaime). In their place, AC57 and AC58: the docs now say the Bazel and merge-queue recipes are untested and where to report (8ad7aac); the GitHub recipe is the one these runs used, and the generic one is what e2e §13 checks.
- Next: done. Its parent criteria (AC1–AC3, AC57, AC58) are ready to mark in a closing commit. `ci-ok` could be made the required check on main (Jaime's call, in GitHub settings).



## yass/archive/2026/10/2026-10-04-monorepos-at-scale-and-plans-repos/2026-10-04-yass-trailers/change.md

---
platforms: [all]
source: 
follows: 
blocked: 2026-10-04-separate-plans-mode, 2026-10-04-repo-qualified-citations, 2026-10-04-the-binary-carries-the-playbooks
---
# yass: trailers

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
With separate plans, every code commit names its change in a `yass:` trailer that the hooks add automatically and check (a warning by default, enforced with `trailers: require`), and yass status lists each change's and each criterion's code from those trailers.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC40–AC52 in [plan.md](../plan.md)
- [x] `yass init` (with `--hooks` to turn them on) installs and upgrades the new `prepare-commit-msg` and `commit-msg` scripts beside `pre-commit`

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] 9a: `trailers.go`: the grammar (`parseTrailer`), targets (active, archived, evicted, `<change>/<piece>`, an unambiguous piece; the closest names suggested), when trailers apply (`plansSeparate`, `trailers:` warn, require, off)
- [x] 9a: `yass hook --prepare-msg` (from `yass use`, the branch name, an earlier commit's trailer; never over yours, never on merges), `--commit-msg` (heads-up, or refuse under `require`), `--range` (reported; fails with `--strict` or `require`; form only without the plans)
- [x] 9a: `yass use [<change>] [--ac] [--clear]`, kept in the worktree's own git folder
- [x] 9a: `prepare-commit-msg` and `commit-msg` scripts, installed by `yass init` and added by `yass upgrade`
- [x] 9a: `git` calls for a particular folder drop the hook's `GIT_DIR` (and `GIT_INDEX_FILE`), so a hook in a linked worktree finds the plans repo rather than the code worktree
- [x] 9a: go test (`TestParseTrailer`, `TestCheckTarget`); e2e §45 (681 ok); husky wired in a scratch repo; `docs/separate-plans.md`, README
- [x] 9a: Jaime reviews the docs (AC49)
- [x] 9b: `yass status <change>` lists its merged commits from trailers, its pieces' included (AC50), and each criterion's, warning about an `ac:` its plan doesn't have (AC51); ten per list, then a count; one `git log --grep` on the merged branch
- [x] 9b: e2e §46 (691 ok); `tests/bench.sh` with 50,000 commits, 5,001 with trailers: `yass status` 77ms, `yass status <change>` 229ms (AC52, < 1s); `docs/separate-plans.md`

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- The commit lists are only in `yass status <change>`, not in plain `yass status`, so the overview pays nothing for them (claude, 2026-10-10)
- 9b's bench caught a regression from piece 7 (#66): deciding inline or separate ran two git processes per yass folder, so `yass status` on the 200-folder bench went from ~0.9s to ~3.2s. A folder inside this repo's work tree (and not a private one) is now inline without asking git, and the code repo's own answer is cached: back to 858ms. Piece 7 and 8 weren't benchmarked when they landed; they should have been (claude, 2026-10-10)
- Lists cap at ten with a count of the rest: the bench's change had 5,001 commits, all printed (claude, 2026-10-10)
- Built in two pull requests: 9a, the trailer itself (AC40–AC49), and 9b, status listing code from trailers (AC50–AC52) - thirteen criteria and two new hooks are more than one reviewable pull request (claude, 2026-10-10)
- Trailers apply only when every yass folder the code repo uses is outside it (or its `yass.yaml` points outside to a folder that isn't checked out, as in CI); a repo with inline plans and a private separate folder, like this one, treats them as off (claude, 2026-10-10)
- The branch-name source matches the whole branch or the part after its last `/`, against change and piece folder names, not the slug without the date - names are what the trailer writes, and a loose match could pick the wrong change (claude, 2026-10-10)
- Found by e2e: hooks in a linked worktree run with `GIT_DIR` set, so every `git -C <plans repo>` call was answered by the code worktree; `git` now drops the repo-pinning variables when it's given a folder, which fixes it for every hook, not just trailers (claude, 2026-10-10)
- AC49's manual check used husky (installed with npm in a scratch repo): `yass use` gave the commit its trailer through `.husky/prepare-commit-msg`, and without it `.husky/commit-msg` gave the heads-up. lefthook and pre-commit weren't available; their configs follow those tools' documented formats, and the doc says husky is the one tested (claude, 2026-10-10)
- Waits for 2026-10-04-the-binary-carries-the-playbooks and ships its hook scripts through `yass init`, which installs and upgrades everything a repo gets once that change lands; this criterion said `install.sh` before (Jaime)
- The trailer hooks skip a `yass` that doesn't know them (`yass hook --help` without the option) - setting up AC56 showed that once `yass upgrade` installs them, anyone with an older `yass` on PATH couldn't commit: it rejected `--prepare-msg` and git aborted (claude, 2026-10-11)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-10 (claude)
- Did: 9a, the `yass:` trailer: grammar, hooks, `yass use`, range checks, scripts, docs. `go test`, `tests/e2e.sh` (681 ok), `tests/examples.sh`, `tests/site.sh` pass.
- Next: 9b: `yass status <change>` lists commits from trailers, per change and per criterion, and the 50,000-commit bench. Jaime reviews the docs (AC49).
### 2026-10-10 (claude)
- Did: 9b: code and criteria from trailers in `yass status <change>`, the 50,000-commit bench, docs; fixed the per-folder git lookup piece 7 added to every `yass status`. `go test`, `tests/e2e.sh` (691 ok), `tests/examples.sh`, `tests/bench.sh` (all targets) pass. Jaime reviewed 9a's docs (#70). The piece is done.
- Next: none here; piece 10 (`check-from-the-plans-repo`) can start.
### 2026-10-11 (claude)
- Did: fixed the trailer hooks blocking commits under an older `yass` (found setting up AC56); e2e §48 with a stand-in older binary, and a manual check under the real 0.4.0 release (702 ok).
- Next: none here.


## yass/archive/2026/10/2026-10-04-monorepos-at-scale-and-plans-repos/change.md

---
platforms: [all]
source: 
follows: 
blocked:
archived: 2026-10-11T20:57:58Z
---
# Large monorepos first, separate plans repo done right

## Goal
<!-- Intent. One paragraph; the detail is in prd.md (why and what) and plan.md (how and acceptance). -->
YASS works at a large company with one big monorepo, many teams and expensive CI. Plan-only commits cost no CI, each team owns its own plans, and the tools stay fast at that scale, so keeping plans next to the code is the clear recommendation. For projects that need private plans, a separate plans repo is a supported fallback. It works correctly in worktrees, with several code repos sharing it, and through archiving. Its link between code and plans is a `yass:` trailer that tooling adds automatically and checks, warning by default, and only in that setup.

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)" for calls that span the whole change. -->
- Monorepo with plans inline is the recommended path, including for large orgs; a separate plans repo is the fallback for privacy - CI cost and size are solvable inside the monorepo (path filters, affected-target builds, LFS), and inline keeps progress in the same commit as code (Jaime)
- No orphan branches, submodules or subtrees - they bring separate-repo semantics or submodule pain without solving privacy (Jaime)
- Trailers on code commits are checked only when plans are in a separate repo - they replace the same-commit guarantee that repo loses, and the monorepo setup stays as simple as it is (Jaime)
- Link code to plans with a trailer in the code commit, not only SHA citations in the plans - trailers can't be rewritten and survive squash merges, a lesson from Gerrit/Zuul `Depends-On:` (Jaime, claude)
- Assets (mocks, images) live in the change folder, with no tooling change - checked on 2026-10-04: archiving moves them as renames, status ignores them, and the archive's append-only rule covers them (claude)
- PRD approved, open questions to be settled during planning (Jaime, 2026-10-04)
- Plan written: 56 criteria in eleven pieces, worktree fix first (claude)
- Open questions settled: `code:` citations stay as optional detail; Bazel is the affected-targets recipe; the speed targets stay as written (Jaime, 2026-10-04)
- Trailers: one lower-case `yass:` trailer with a small DSL (`yass: <change>[/<piece>] [ac:ac1,ac2]`, `yass: none`), read case-insensitively. A warning by default, with `trailers: require` in the code repo's `yass.yaml` (which separate plans need anyway) to enforce it. Added automatically by a prepare-commit-msg hook from `yass use`, the branch name, or earlier commits - so trailers are never a burden to agents or people, and teams choose how strict to be (Jaime)
- Pieces own their files, instead of a merge driver; R7 reworded to match - the conflicts come from pieces on parallel branches sharing the parent's plan.md and Log, so the plan's shape is the fix. A driver would need setting up in every clone, wouldn't run in hosted merges, and could silently lose content. This lowers R7's bar: parallel archive commits can still conflict on adjacent queue.md lines (rare, trivial to fix), and queue reorders conflict on purpose because they're intent (Jaime)
- Progress stays in files rather than a tracker (Jira, Linear); a tracker's real advantage, a live view of who's working on what, comes from a git-native `yass status --branches` instead, added as R19 in M1. A one-way projection into a tracker may be a later change (Jaime)
- Remaining design calls settled: separate = `path:` resolves outside the repo, plus a gitignored `yass/` symlink as a convenience; optional `web@sha` with names inferred; `repos:` with `${VAR:-default}` paths, offline; `yass paths` as proposed; the branch view on by default, with `--since` and `--no-branches` (Jaime, 2026-10-04)
- Branch view stays on by default everywhere, after reviewing its cost (a local walk of unmerged branches limited to YASS paths, plus reading plan files at branch tips) - safeguards: no on-demand downloads in partial clones, commit-graph recommended in the docs, freshness shown (Jaime, 2026-10-04)
- AC4 and AC5 dropped (`[-]`) here, which lowers this change's bar: neither a Bazel repo nor a GitHub merge queue (it needs an organization-owned repo) is available yet. Testing both moves to `2026-10-04-validate-the-ci-recipes-in-the-field`, along with inviting reports for other CI systems. In their place, AC57 and AC58 hold the docs to saying plainly what's tested and what isn't; R2 and R3 are reworded to match (Jaime, 2026-10-04)

- AC12 ("ready to mark" in `yass status`) dropped here: `2026-10-04-stacked-prs-and-squash-merges` takes over counting delivered parent criteria (its R5, R6), so piece 5 keeps only the rule that pieces own their files and the hook's heads-up. design.md §5 notes it (Jaime, 2026-10-04)
- Closing commits are gone: since `stacked-prs-and-squash-merges` (#59), `yass status` counts the criteria pieces deliver and `yass archive` ticks them. AC13 (a closing commit marks the parent done) is dropped, which lowers the bar: #59's e2e already proves the behaviour without one. AC14, piece 5's Goal and README box, the Pieces note and design §'Pieces own their files' now say no closing commit is needed. The commit-graph and branch-view box moves from piece 4 to piece 6, where the branch view is built (Jaime, 2026-10-09)

## Log
<!-- Progress. Change-level notes; each piece keeps its own Log.
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-04 (claude)
- Did: closing commit for the two finished pieces: marked AC1–AC3, AC57, AC58 (`2026-10-04-yass-paths-and-ci-recipes`) and AC23–AC29 (`2026-10-04-worktrees-find-the-plans`) in plan.md, with no code.
- Next: piece 4 (`2026-10-04-team-scale-docs`) and piece 5 (`2026-10-04-pieces-own-their-files`) are unblocked; piece 3 waits on the branch view (piece 6).
### 2026-10-09 (claude)
- Did: piece 4 (`team-scale-docs`) done (#62). Revised the plan for #59: no closing commits (AC13 dropped), the commit-graph box moved to piece 6.
- Next: piece 5 (`2026-10-04-pieces-own-their-files`); then piece 6 (`branches-view`), which also finishes piece 3's last benchmark.
### 2026-10-09 (claude)
- Did: pieces 3 (`speed-at-scale`) and 5 (`pieces-own-their-files`) done; piece 6 (`branches-view`) built, waiting on Jaime's review of its docs.
- Next: piece 7 (`separate-plans-mode`), the start of the separate-plans pieces (8–11).


## yass/archive/2026/10/2026-10-04-monorepos-at-scale-and-plans-repos/design.md

# Large monorepos first, separate plans repo done right: design
<!-- Intent: the calls that are hard to undo. Edit only in commits without code. -->

## Context
This change adds public surface that people will script against and put into commits, CI configs and `yass.yaml` files: a commit trailer, two `yass.yaml` keys, a citation syntax, CLI commands and flags, and the rule that pieces own their files. Each is costly to rename once it's in use, so each is decided here. The PRD fixed the big calls: inline plans in a monorepo are recommended, a separate repo is the privacy fallback, and trailers are enforced only in that fallback (Jaime). Every call here was settled with Jaime on 2026-10-04.

## 1. When plans count as separate
**Options.**
- (a) Separate when the yass folder isn't inside the code repo's git work tree, whether it's in another repo or in no repo at all.
- (b) A `mode:` key in `yass.yaml`.
- (c) Separate whenever `path:` is set.

**Decision: (a)** (Jaime, 2026-10-04). Put simply: plans are separate when `yass.yaml`'s `path:` resolves outside this repo. It's derived from the setup, so it can't drift out of sync with it. (c) is wrong because `path: planning` inside the repo is still inline, and (b) adds a setting people could get wrong. A linked worktree counts as the same repo.

**A `yass/` symlink, as a convenience** (Jaime). When plans are separate, `yass init` and any `yass` command that finds it missing or stale create a `yass/` symlink in the checkout, or worktree, pointing to the resolved plans folder. They also add the link's path to the clone's `.git/info/exclude` once. That file is shared by every worktree of the clone and is never committed, so the link adds no diff, even in repos that keep `yass.yaml` itself private. Because the link is ignored and per machine, its target is absolute. Everyone, editors and agents included, sees `yass/changes/...` at the root. `yass.yaml` stays the source of truth, and nothing depends on the link:
- a real `yass/` folder or file already there is left alone, with a warning;
- where symlinks aren't allowed (Windows without Developer Mode), YASS skips it with a note;
- root discovery dedupes the link against the folder it points to.

Its known limits are documented: a new worktree has no link until a `yass` command runs there, `rg` needs `-L` to follow it, and an agent harness may treat files behind it as outside the project for permissions.

## 2. How code commits name their change (separate plans only)
**Options.**
- (a) A git trailer in the commit message.
- (b) A link in the PR body only.
- (c) SHA citations in the plans only, as today.

**Decision: (a), as one `yass:` trailer with a small DSL** (Jaime, 2026-10-04). Trailers sit in commit history and survive squash merges when the squash message keeps them. `git interpret-trailers` and `%(trailers)` already parse them, and they follow the precedent of `Change-Id:`, `Depends-On:` and `Bug:`. (b) can't be checked offline, and (c) breaks on squash merges.

```
yass: <target> [<key>:<value> ...]

target  := none | <change> | <change>/<piece> | <piece>
key     := ac            (more later)
value   := comma-separated, no spaces
```
- **Case:** YASS writes lower case, and reads the key and the value case-insensitively. Git matches trailer keys case-insensitively too.
- **Several changes:** one `yass:` line per change.
- **Pieces:** a piece can be named alone, matched by folder name as `blocked:` matches it, with the path needed only when the name is ambiguous.
- **`ac:`** names the criteria the commit works toward. That's the box-to-commit link from the code side, and a squash merge can't break it. `code:` citations on boxes stay, as optional detail (Jaime).
- **Unknown keys** are a warning, so newer trailers don't break older `yass` versions.
- **`yass: none`** marks a commit that belongs to no change (dependency bumps, typo fixes).

**Strictness** is set by `trailers:` in the code repo's `yass.yaml`: `warn` (the default), `require` or `off` (Jaime). `yass hook --range` follows it in CI, and `--strict` turns its warnings into failures, as today.

**Added automatically** (Jaime chose all three sources, in this order). A `prepare-commit-msg` hook works out the target from:
1. `yass use <change> [--ac ac3]`, stored per worktree in that worktree's git folder, so it's never committed;
2. the branch name, matched against change folder names;
3. a `yass:` trailer on an earlier commit since the branch left the merged branch.

The hook appends the trailer with `git interpret-trailers`. It never overwrites one the author wrote, and it skips merge commits. Unlike `commit-msg`, `prepare-commit-msg` isn't skipped by `--no-verify`. The `yass-work` skill runs `yass use` when it starts a change, and puts the trailer line in PR bodies so squash merges that use the PR description keep it. For orgs with a hook manager (husky, lefthook, pre-commit), the docs show how to call `yass hook --prepare-msg` and `yass hook --commit-msg` from it instead of using `core.hooksPath`.

## 3. How a code repo is named in citations
**Options.**
- (a) A `repo:` key in the code repo's `yass.yaml`. Without it, the name comes from the last path segment of the `origin` URL (minus `.git`), and failing that, from the main worktree's folder name.
- (b) Always the folder name.
- (c) Make `repo:` required.

**Decision: (a)** (Jaime, 2026-10-04). Since trailers carry the main link, this only matters for a shared plans repo whose boxes still carry the optional `code:` citations. Folder names differ between clones and worktrees, so (b) breaks for agents. Requiring `repo:` (c) taxes the common case, one code repo. Citations look like `code: web@a1b2c3d`. An unqualified `code: a1b2c3d` belongs to whichever repo is checking it, which is today's behavior.

## 4. How a plans repo finds the code
**Options.**
- (a) `repos:` in the plans repo's `yass.yaml`: a map from repo name to a local checkout path, resolved like `path:` (relative, `~`, `${VAR}`).
- (b) Clone remotes on demand.
- (c) Nothing: check only from the code repos.

**Decision: (a)** (Jaime, 2026-10-04). Paths accept `${VAR:-default}`, as `path:` does, so `web: ${CODE_HOME:-..}/web` works for side-by-side clones with no setup, and anyone can set `CODE_HOME` instead. The agent's project root stays the code repo, with the plans added as an extra directory; `repos:` is only for working in the plans repo itself and for its CI. It works offline and in CI, where the pipeline checks out the code repos next to the plans repo. It never touches the network. A name with no checkout is reported as unchecked, not as an error. (b) needs credentials and network access, and (c) leaves the plans repo unable to check itself.

## 5. How parallel branches stop conflicting over progress (R7)
**Options.**
- (a) A YASS merge driver (`yass merge`) that auto-resolves Log appends, box marks on different lines, and removals from `queue.md`.
- (b) `merge=union` in `.gitattributes`.
- (c) Guidance only.
- (d) Pieces own their files: parallel branches never share a YASS file, so there's nothing to merge.
- (e) Keep progress in a tracker (Jira, Linear) instead of in files.

**Decision: (d)** (Jaime, 2026-10-04, after discussing the trade-offs with trackers). The conflicts come from the plan's shape, with pieces on parallel branches ticking the same parent `plan.md` and appending to the parent's Log, so the fix is the shape, not git machinery.
- **The rule:** a branch working on a piece changes only that piece's folder (and code). The parent's `prd.md`, `plan.md`, `design.md` and `change.md` change only in intent commits, or in a commit with no code (its own progress, or the archive move).
- **Parent criteria:** a piece names what it delivers in a box like `Delivers AC1, AC3–AC5`. Superseded on 2026-10-04 by `2026-10-04-stacked-prs-and-squash-merges` (R5, R6): a delivered criterion counts as done without a closing commit per piece, and `yass archive` marks it. Until that ships, parent boxes are marked in a closing commit with no code, on main, so there's no parallel contention. It shipped on 2026-10-09 (#59): no closing commit is needed.
- **The hook** gives a heads-up when a commit with code touches the parent's files, or more than one piece's folder.
- **What still conflicts,** and should: two people on the same piece, reorders in `queue.md` (intent), and two archive commits that each remove an adjacent `queue.md` line (rare, and resolved in seconds).

Why not the others: (a) has to be configured in every clone, doesn't run in hosted merges or merge queues, and a bug in it silently loses content. (b) keeps both versions of a box marked differently on each side. (c) doesn't solve anything. (e) gives up progress travelling with code, and makes agents work through MCP instead of files.

## 6. `yass paths`
**Options.**
- (a) `yass paths` prints, one per line, the repo-relative globs YASS owns: each yass folder inside the repo and each `yass.yaml`. `yass paths --only <range>` exits 0 when every file changed in the range is one of them, 1 when not, and 2 on error.
- (b) Print globs only, and leave the comparison to each CI.
- (c) A separate `yass ci` command per CI vendor.

**Decision: (a)** (Jaime, 2026-10-04). One command lets any CI write a gate job, including when the path list changes as teams adopt YASS. (b) pushes fiddly diff logic into every pipeline. (c) is the per-vendor product the PRD rules out. The pre-commit hook script in `tools/yass/githooks/` isn't a YASS path: changing it is changing code.

## 7. Seeing unmerged work (R19)
**Options.**
- (a) A branch view in `yass status`: one `git log` over local and remote-tracking branches not yet on the merged branch, limited to YASS paths and to branches with commits in the last 30 days. For each change or piece a branch touches, it shows the branch, its last author and date, and its box progress and latest **Next:** as that branch has them. Results are as of the last fetch, and the output says when that was.
- (b) A claims file (`claimed-by:` in front matter) committed to main.
- (c) A tracker as the live view.

**Decision: (a), shown by default** (Jaime, 2026-10-04). It needs nothing new from anyone: pushing a branch already announces the work. (b) makes every start a commit to main, which brings back the contention §5 removes. (c) is a non-goal. It never fetches by itself, because the network is the caller's choice. `--since <date|Nd>` sets the window (default 30 days), and `--no-branches` leaves the view out, for scripts and huge repos. Because it's on by default, it counts toward `yass status`'s speed target. Jaime kept it on everywhere after reviewing what it costs (2026-10-04). To keep that safe on large repos:
- the branch walk uses the commit-graph and its changed-path filters when the repo has them;
- reading file versions at branch tips never triggers a partial clone's on-demand download (`--no-lazy-fetch`, git 2.44+). With older git, a branch whose files aren't local shows its name and author without progress;
- remote branches deleted on the server are listed until a `fetch --prune`, and the view's freshness line says how old the refs are.

## Consequences and rollback
- The `yass:` trailer grammar, the `trailers:` key, `yass use`, the `repo:` and `repos:` keys, `web@sha` citations, `yass paths`, the branch view's `--since` and `--no-branches`, the gitignored `yass/` symlink, `${VAR:-default}` in paths, and the `Delivers AC…` box format all become public API. Renaming any of them later needs a deprecation period. Until a release ships them, they can still change freely.
- "Pieces own their files" is a convention the hook only gives a heads-up about. Dropping it later loosens a rule; it doesn't break any file.
- For inline plans, the only changes are `yass paths`, the branch view, the *ready to mark* view and the pieces-own-their-files heads-up.


## yass/archive/2026/10/2026-10-04-monorepos-at-scale-and-plans-repos/plan.md

# Large monorepos first, separate plans repo done right: plan
<!-- Intent: how, and how we'll know it's done. With code, only mark boxes here ([/] in progress, [x] done);
     anything else is a plan revision in its own commit. -->

## Approach
The public surface (trailer, `repo:` and `repos:`, `web@sha`, `yass paths`, the branch view, the `yass/` symlink, pieces owning their files, and when plans count as separate) is decided in [design.md](design.md). Everything else builds on the existing pieces: root discovery and `yass.yaml` in `internal/yass/repo.go` and `config.go`, the citation check (`checkCode`, `mergedBranch`), the hook (`internal/yass/hook.go`, `kit/tools/yass/githooks/`), the installer (`install.sh --hooks`), and the end-to-end suite (`tests/e2e.sh`), which gets a numbered section per piece.

**M1, large monorepos.** Mostly documentation, in a new `docs/monorepo.md` linked from the README's Monorepos section, backed by small tooling where a recipe needs it:
- `yass paths` (and `--only <range>`), so any CI can skip plan-only changes with a single gate job.
- A sparse-aware `blocked:` check: `git ls-files` lists entries that aren't checked out, and root discovery already skips folders that aren't on disk.
- Pieces own their files (design.md §5). The hook gives a heads-up when a commit with code touches a parent's files or more than one piece. `yass status <change>` shows parent criteria as *ready to mark* once the pieces delivering them are done, parsed from `Delivers AC…` boxes. The `yass-work` and `yass-plan` skills teach the rule.
- A branch view in `yass status`, on by default (design.md §7): one `git log` over unmerged local and remote-tracking branches, limited to YASS paths and recent branches, showing who is working on which change or piece and how far they are.
- A benchmark script that holds `yass status` and the hook to the PRD's speed targets.

This repo's own `ci.yml` adopts the GitHub recipe, so the recipe is tested every time someone touches a plan here.

**M2, separate plans done correctly.** First, fix path resolution in linked worktrees: when a relative `path:` doesn't exist from the worktree, resolve it from the main worktree, found through `git rev-parse --git-common-dir`. Then add `${VAR:-default}` to path expansion in `config.go`, and the gitignored `yass/` symlink (design.md §1). Then add a notion of "separate", derived as design.md §1 says. It drives:
- a `plans:` line in status;
- notes on the plans repo's uncommitted or behind state (notes, not warnings, so `--strict` is unaffected);
- repo-qualified citations: each repo checks its own and lists the rest as unchecked;
- `yass archive` running the citation check before it moves anything.

**M3, automatic, checked trailers.**
- A `prepare-commit-msg` script adds the `yass:` trailer from `yass use`, the branch name or earlier commits (design.md §2).
- A `commit-msg` script runs `yass hook --commit-msg <file>`, which warns, or rejects under `trailers: require`. Both scripts sit beside `pre-commit` and do nothing when plans are inline.
- `yass hook --range` runs the same check in CI. Where the plans folder isn't available (typical in CI for private plans), it checks only the trailer's grammar and says so.
- `yass status <change>` lists a change's code commits, and each criterion's commits from `ac:`, by reading trailers once from the merged branch, starting at the date of the oldest active change.
- A plans repo's `repos:` map lets status run all of these checks from the plans side.
- Finally, the `yass-work`, `yass-log` and `yass-status` skills, and `docs/skills.md`, get the full separate-plans routine, including `yass use` and trailers in PR bodies.

## Acceptance
### M1
- [x] AC1 (R1) Given a monorepo with `yass/`, `services/payments/yass/` and a `yass.yaml` pointing to `planning/`, when you run `yass paths`, then it prints exactly those yass folders as globs plus each `yass.yaml`, one per line, and no paths for ignored folders or for a plans folder outside the repo — verify: e2e §13
- [x] AC2 (R1) Given a commit range that changes only files under YASS paths, when you run `yass paths --only <range>`, then it exits 0. With any other file in the range it exits 1, and with a range it can't read it exits 2 with a hint to fetch history — verify: e2e §13
- [x] AC3 (R2) Given this repo's `ci.yml` using the documented gate-job recipe, when a PR changes only files under `yass/`, then build and test jobs are skipped, and the required check reports success instead of waiting — verify: manual: open a plan-only PR here and a code PR; record both check runs in the Log
- [-] AC4 (R2) Given `docs/monorepo.md`, then it has a GitHub Actions recipe (gate job, required checks), a Bazel recipe (`.bazelignore` and affected targets, with the yass folders in no target), and a generic recipe using `yass paths --only`, and each recipe's commands are copied from a run that worked — verify: manual: review against the Log of the runs
- [-] AC5 (R3) Given a GitHub merge queue, when a plan-only PR enters it, then the documented `merge_group` setup passes without running the test suite. The docs say how path filters behave for `merge_group` — verify: manual: test on a scratch repo with a merge queue, or say in the Log why that wasn't possible and what the docs rest on
- [x] AC57 (R2) Given `docs/monorepo.md`, then its GitHub Actions and generic recipes are copied from runs that worked (this repo's CI, e2e §13), and its Bazel section says it hasn't been tested end to end yet and where to report results — verify: manual: review against the Log
- [x] AC58 (R3) Given `docs/monorepo.md`, then its merge-queue section says how `merge_group` differs (`branches` filters, no `paths`), cites GitHub's docs, and says it hasn't been tested on a real queue yet and where to report results — verify: manual: review
- [x] AC6 (R4) Given `docs/monorepo.md`, then it shows CODEOWNERS entries routing each team's yass folder to that team, and the root folder to the people who own cross-team changes — verify: manual: review
- [x] AC7 (R5) Given a cone-mode sparse checkout without `services/search/`, when you run `yass status`, then it lists the checked-out yass folders, says nothing about `services/search/yass/`, and exits 0 under `--strict` — verify: e2e §14
- [x] AC8 (R5) Given a checked-out change with `blocked: <a change in a folder that isn't checked out>`, when you run `yass status`, then it says that change is outside this sparse checkout and can't be checked, not that there's no such change, and `--strict` still exits 0 — verify: e2e §14
- [x] AC9 (R6) Given `docs/monorepo.md`, then it has `.gitattributes` LFS rules for images and PDFs under yass folders, and says assets live in the change folder and move with it when archived — verify: manual: review; e2e §14 archives a change with a binary asset and finds it under `archive/`
- [x] AC10 (R7) Given two branches each working on a different piece of the same change, ticking boxes and appending Log entries only in their own piece's folder, when one rebases onto the other, then it finishes without conflicts — verify: e2e §15
- [x] AC11 (R7) Given a commit with code that also changes the parent's `plan.md` or `change.md`, or files in two pieces, when it's committed with the hook on, then the hook gives a heads-up naming the files, and a closing commit with no code that marks parent boxes gets none — verify: e2e §15
- [-] AC12 (R7) Given a parent criterion whose delivering pieces (by their `Delivers AC1, AC3–AC5` boxes) are all done, when you run `yass status <change>`, then it lists that criterion as ready to mark. A criterion with any piece still open isn't listed, and one no piece delivers is left alone — verify: e2e §15; go test for parsing ID lists and ranges
- [-] AC13 (R7) Given a parent whose criteria are all marked done in a closing commit, after its pieces are done, when you run `yass status`, then the change is `done` and `yass archive` accepts it, as today — verify: e2e §15
- [x] AC14 (R7) Given the `yass-plan` and `yass-work` skills, then they say a branch working on a piece changes only that piece's folder, and that the parent needs no closing commit: `yass status` counts the criteria its pieces deliver, and `yass archive` ticks them — verify: manual: review
- [x] AC15 (R8) Given `docs/monorepo.md`, then it gives commands for `git log`, `blame` and diff that leave YASS paths out, built from `yass paths` — verify: manual: run each in this repo
- [x] AC16 (R9) Given a generated repo with 200 yass folders and 2,000 changes, with 100 recently active branches, when you run `yass status` (branch view included), then it finishes in under 1s on a 2024-or-later laptop — verify: `tests/bench.sh`; record the timing in the Log
- [x] AC17 (R9) Given a generated repo of 100,000 files, when you commit a one-file change with the hook on, then the hook adds under 200ms — verify: `tests/bench.sh`; record the timing in the Log

- [x] AC18 (R19) Given a local branch and a pushed branch, each ahead of main with commits under a piece's folder, when you run `yass status`, then each piece lists those branches, with last author, date, the branch's box progress and its latest **Next:** — verify: e2e §21
- [x] AC19 (R19) Given branches that only change code, branches already merged, and branches with no commits in the last 30 days, when you run `yass status`, then none of them are listed — verify: e2e §21
- [x] AC20 (R19) Given a branch last touched 10 days ago, when you run `yass status --since 7d`, then it isn't listed. With `--since <a date before it>`, it is. With `--no-branches`, no branch information is shown and no branch lookup runs — verify: e2e §21
- [x] AC21 (R19) Given any run, then the branch view never fetches, including in a blobless partial clone whose branch-tip files aren't local (it shows those branches without progress instead), and it says when the remote-tracking refs were last fetched, or that there are none. Plain `yass status` with no unmerged branches looks as it does today — verify: e2e §21 (no network: a remote that's a local folder)
- [x] AC22 (R19) Given the benchmark repo with 1,000 recently active remote branches, when you run `yass status`, then it finishes in under 2s — verify: `tests/bench.sh`; record the timing in the Log

### M2
- [x] AC23 (R11) Given a code repo whose `yass.yaml` says `path: ../plans`, when you run `yass root` and `yass status` from a linked worktree elsewhere on disk, or nested under `.claude/worktrees/`, then both resolve to the same plans folder as the main checkout — verify: e2e §16
- [x] AC24 (R11) Given a linked worktree where the plans folder can't be found by either route, when you run `yass status`, then the warning names both places it looked and doesn't suggest `yass init` — verify: e2e §16
- [x] AC25 (R22) Given a code repo whose `yass.yaml` points outside it, when you run `yass init` or `yass status` in the main checkout or a linked worktree, then `yass/` there is a symlink to the resolved plans folder, the clone's `.git/info/exclude` lists it (added once, shared by all worktrees), and `git status` shows nothing new, with `.gitignore` untouched — verify: e2e §16
- [x] AC26 (R22) Given that link, when you run `yass status`, then each change is listed once and there are no warnings about the link. When the plans folder moves and `yass.yaml` is updated, the next `yass` command repoints the link — verify: e2e §16
- [x] AC27 (R22) Given a real `yass/` folder or file already in place, when a `yass` command runs, then it's left alone and YASS warns once. Where symlinks can't be created, YASS skips the link with a note and everything else works — verify: e2e §16; go test with a failing symlink call
- [x] AC28 (R22) Given Claude Code with the code repo as project root, when an agent reads and edits `yass/changes/...` through the link, then the docs say what works and what permission it needs (`--add-dir` or not) — verify: manual: one session; record the result in the Log
- [x] AC29 (R23) Given `path: ${YASS_PLANS:-../plans}`, when `YASS_PLANS` is unset or empty, then `../plans` is used, and when it's set, its value is. `${VAR}` with no default and unset still fails with today's "isn't set" message. `repos:` entries resolve the same way — verify: go test (config_test.go); e2e §16
- [x] AC30 (R10) Given plans inside the repo, when you run `yass status`, then its output is unchanged, and `yass root -v` prints each folder followed by `inline` — verify: e2e §17
- [x] AC31 (R10) Given plans in another git repo or in no repo, when you run `yass status`, then it ends with `plans: separate (<folder>)`, and `yass root -v` prints `separate`. Plain `yass root` output is unchanged — verify: e2e §17
- [x] AC32 (R14) Given separate plans with uncommitted changes in the plans folder, when you run `yass status`, then it notes how many files are uncommitted there, and `--strict` still exits 0 — verify: e2e §17
- [x] AC33 (R14) Given a separate plans repo behind its upstream as of the last fetch, when you run `yass status`, then it notes how far behind it is and suggests pulling, without fetching itself — verify: e2e §17
- [x] AC34 (R12) Given two code repos named `api` and `web` sharing a plans repo, with boxes citing `api@<sha>`, `web@<sha>` and `web@deadbee`, when you run `yass status` in `api`, then it checks only the `api` citations, warns about none of the others, and says how many it left to `web` — verify: e2e §18
- [x] AC35 (R12) Given the same setup, when you run `yass status` in `web`, then it warns that `web@deadbee` isn't a commit there — verify: e2e §18
- [x] AC36 (R12) Given no `repo:` key, then a code repo's name comes from its `origin` URL, and failing that from its main worktree's folder name, the same from every linked worktree. `repo:` overrides both — verify: e2e §18; go test
- [x] AC37 (R12) Given a single code repo with unqualified citations, when you run `yass status`, then it behaves exactly as before, and the existing §9 assertions still pass — verify: e2e §9
- [x] AC38 (R13) Given a change whose `[x]` box cites an unmerged or missing commit in this repo, when you run `yass archive`, then it refuses and names the box and commit. `--force` archives it anyway — verify: e2e §18
- [x] AC39 (R13) Given a change whose citations belong to another repo, when you run `yass archive` here, then it archives it and lists the citations it couldn't check and where to check them — verify: e2e §18

### M3
- [x] AC40 (R15) Given separate plans with `trailers:` unset or `warn` and the hooks on, when a commit has no `yass:` trailer, then it goes through with a heads-up saying how to add one (`yass use <change>`, or `yass: none`) — verify: e2e §19
- [x] AC41 (R15) Given `trailers: require`, when a commit has no `yass:` trailer, or one whose target names no change, then it's rejected. The message suggests the closest change names — verify: e2e §19
- [x] AC42 (R15) Given separate plans, when a trailer names an active change, an archived change, `<change>/<piece>`, a piece alone, or `none`, in any letter case, then it's accepted. A malformed trailer, or one whose piece name is ambiguous, is treated as invalid. An unknown `key:value` gives a warning only — verify: e2e §19; go test for the grammar
- [x] AC43 (R15) Given inline plans, or `trailers: off`, with the hooks on, when you commit, then nothing about trailers is added, checked or printed — verify: e2e §19
- [x] AC44 (R15) Given separate plans and `yass hook --range A..B`, when a non-merge commit in the range lacks a valid trailer, then it's reported, and with `--strict` or `trailers: require` it exits 1. A squash commit whose message carries the trailers passes — verify: e2e §19
- [x] AC45 (R15) Given CI where the plans folder isn't available, when `yass hook --range` runs, then it checks only the trailers' grammar and says the change names weren't checked — verify: e2e §19
- [x] AC46 (R20) Given `yass use <change>` in one worktree, a different change in another, and the hooks on, when each commits with `git commit -m`, then each commit gets `yass: <its change>` in lower case. `yass use` with no argument prints the current change, and `--clear` clears it — verify: e2e §19
- [x] AC47 (R20) Given no `yass use`, when the branch is named after a change (exactly, or with a prefix like `jaime/`), then the trailer comes from the branch name. With neither, it comes from an earlier commit's trailer on the branch. With none of the three, nothing is added and the commit-msg check reports it — verify: e2e §19
- [x] AC48 (R20) Given a commit message that already has a `yass:` trailer, or a merge commit, when the prepare-commit-msg hook runs, then the message is left as it is — verify: e2e §19
- [x] AC49 (R20) Given `docs/` for separate plans, then they show the `trailers:` setting, `yass use`, and calling `yass hook --prepare-msg` and `--commit-msg` from husky, lefthook and pre-commit — verify: manual: wire one manager in a scratch repo, and record it in the Log
- [x] AC50 (R16) Given merged commits carrying `yass: <change>`, when you run `yass status <change>` in that code repo, then it lists them (short SHA, subject) under the change, including after a squash merge, and unmerged commits aren't listed — verify: e2e §20
- [x] AC51 (R21) Given merged commits whose trailers carry `ac:ac1` and `ac:ac1,ac3`, when you run `yass status <change>`, then AC1 lists both commits and AC3 lists one. An `ac:` naming a criterion the change doesn't have gives a warning — verify: e2e §20
- [x] AC52 (R16) Given the benchmark repo plus 50,000 commits, when you run `yass status`, then the trailer lookup keeps it under the AC16 target — verify: `tests/bench.sh`; record the timing in the Log
- [x] AC53 (R17) Given a plans repo whose `yass.yaml` maps `api` and `web` to local checkouts, when you run `yass status --strict` there, then every qualified citation is checked against its repo. An unqualified citation is checked when the map has exactly one repo, and listed as unchecked otherwise — verify: e2e §20
- [x] AC54 (R17) Given a `repos:` entry whose checkout is missing, or whose `${VAR}` has no default and is unset, when you run `yass status` in the plans repo, then it notes that repo's citations as unchecked and doesn't fail `--strict` — verify: e2e §20
- [x] AC55 (R18) Given the `yass-work` skill, then its separate-plans section covers the whole routine. At session start, pull the plans and check their state. Commit code first, with the trailer, then plans. Cross-link the PR and the change. Git latitude is per repo. Never run `yass init` for a missing plans folder. Worktrees. `yass-log` and `yass-status` use trailers to find code — verify: manual: review; e2e §1 still finds the installed skills
- [x] AC56 (R18) Given a fresh agent session in a linked worktree of a code repo with separate plans, when it's pointed at a change with "keep going", then it pulls the plans, builds, commits code with the trailer, and records progress in the plans repo, without running `yass init` — verify: manual: one agent run; summarize it in the Log

## Pieces
In order. Each leaves main green and is one PR. This plan follows its own rule: a piece's branch changes only its own folder. The boxes here count as delivered once their pieces are done (`stacked-prs-and-squash-merges`, #59); `yass archive` ticks them.
1. `2026-10-04-worktrees-find-the-plans` (R11, R22, R23): first, because it breaks agents in the desktop app's worktrees today, and every later piece is tested from worktrees.
2. `2026-10-04-yass-paths-and-ci-recipes` (R1–R3): the gate job, dogfooded in this repo's CI. It also starts `docs/monorepo.md`.
3. `2026-10-04-speed-at-scale` (R9): the benchmark comes early, so later pieces are measured against it rather than tuned afterwards.
4. `2026-10-04-team-scale-docs` (R4–R6, R8): sparse checkouts, CODEOWNERS, LFS and history filters. Needs the doc from piece 2.
5. `2026-10-04-pieces-own-their-files` (R7): before the branch view, so the view reports progress from piece folders only.
6. `2026-10-04-branches-view` (R19): needs piece 5. Benchmarked with piece 3's script.
7. `2026-10-04-separate-plans-mode` (R10, R14): the "separate" notion that pieces 8–10 rely on.
8. `2026-10-04-repo-qualified-citations` (R12, R13): needs piece 7.
9. `2026-10-04-yass-trailers` (R15, R16, R20, R21): needs piece 7. It reads repo names from piece 8.
10. `2026-10-04-check-from-the-plans-repo` (R17): needs pieces 8 and 9.
11. `2026-10-04-agent-guidance-for-separate-plans` (R18): last, so it describes what shipped.

Pieces 4, 6 and 8–11 set `blocked:` on the pieces they need, so `yass status` shows what each is waiting on.

## Validation
- `go vet ./...`, `go test ./...`, `tests/e2e.sh` and `tests/examples.sh` pass on macOS and Linux in CI, including the new sections §13–§21.
- `tests/bench.sh` meets the AC16, AC17, AC22 and AC52 targets, with the timings in the Log.
- This repo's own CI runs the GitHub recipe (AC3), so the recipe is checked on every plan-only PR from then on.
- One manual end-to-end pass of each setup before archiving:
  - a monorepo with three team folders, a sparse checkout, and two agents on different pieces of one change in parallel, watched with the branch view;
  - two code repos sharing a private plans repo, with worktrees, trailers, a squash merge and the plans repo's own `--strict` check (AC56).
- `install.sh --upgrade` on an existing install picks up the new `prepare-commit-msg` and `commit-msg` hooks only with `--hooks`, and changes nothing else.


## yass/archive/2026/10/2026-10-04-monorepos-at-scale-and-plans-repos/prd.md

# Large monorepos first, separate plans repo done right: PRD
<!-- Intent: what should change or exist, and why. Not a description of the current system;
     the code and its docs are that. Edit this file only in commits without code. -->

## Why
YASS has to work at a large company with one big monorepo, many teams, and expensive, complicated CI. That's where its main promise matters most: plans sit next to the code, and progress lands in the same commit. Teams there worry that planning files, which change often and can carry mocks and images, will add CI runs, slow the merge queue, bloat clones and clutter history. Most of that can be solved inside the monorepo, but YASS doesn't show how yet, so the easy wrong answer is to move the plans out.

Some projects really do need plans in a separate repo, usually for privacy (an open source project with private plans). That setup exists today ([README: Keeping plans out of the repo](../../../README.md#keeping-plans-out-of-the-repo)), but trying it on 2026-10-04 found it fragile:
- In a linked git worktree, which is where agents usually run, a relative `path:` resolves from the worktree and finds nothing. The warning then suggests `yass init`, which would create a stray, empty plans folder.
- When two code repos share one plans repo, each repo warns about the other's citations as missing commits, and a repo-qualified citation (`code: web@e1553ee`) isn't checked at all.
- `yass archive` archives a change whose `[x]` boxes cite unmerged code.
- `yass status` run from the plans repo flags every citation, so the plans repo can't check itself in CI.
- Nothing warns about uncommitted plans or a plans repo that's behind its remote.
- Plans cite code by SHA, and a squash merge invalidates those SHAs, so they have to be fixed by hand.

## Users and outcomes
- **Teams in a large monorepo** (the primary path), and the platform or CI owners who decide whether YASS is allowed in.
  - With the documented setup, a commit that touches only yass folders runs no build or test jobs, and doesn't block merging with a check stuck waiting.
  - `yass status` takes under 1s on a repo with 200 yass folders and 2,000 changes. The pre-commit hook adds under 200ms on a repo of 100,000 files.
  - A platform owner can roll YASS out from one page of docs, without writing their own glue.
- **Projects that keep plans in a separate repo for privacy** (the fallback), and the agents working in them.
  - Every merged code commit in a separate-repo setup names its change. Tooling adds the name automatically, and checks it: a warning by default, enforced where a team opts in.
  - `yass status` gives no false warnings when two code repos share one plans repo.
  - An agent in any linked worktree resolves the same plans folder as the main checkout.

## Requirements
- **R1** [M1] A CI owner can tell which paths are YASS's from `yass paths`, and use that list in pipeline filters instead of hard-coding folder layouts.
- **R2** [M1] The docs give recipes for skipping build and test jobs on plan-only commits: GitHub Actions (including required checks that would otherwise wait forever when a path filter skips them) and one generic CI, tested; and Bazel or another affected-targets build, documented here and tested in `2026-10-04-validate-the-ci-recipes-in-the-field`.
- **R3** [M1] The docs explain how plan-only PRs go through a merge queue without full test runs. Testing that on a real merge queue is part of `2026-10-04-validate-the-ci-recipes-in-the-field`.
- **R4** [M1] The docs show CODEOWNERS for each team's yass folder, so plan reviews go to the owning team.
- **R5** [M1] `yass status` in a sparse checkout reports on the yass folders that are checked out. A `blocked:` that names a change outside the checkout says it can't see that change, instead of calling it unknown.
- **R6** [M1] The docs give Git LFS rules for binary assets under yass folders, and say that assets live in the change folder and move with it when it's archived.
- **R7** [M1] When agents on parallel branches each work on a different piece of the same change, those branches rebase onto each other without conflicts in YASS files. Each piece's progress lives only in its own folder, and `yass status` shows when a parent's criteria have been delivered by its pieces.
- **R8** [M1] The docs give commands for leaving YASS paths out of `git log`, `blame` and diff views.
- **R9** [M1] `yass status` and the pre-commit hook meet the speed targets above.
- **R19** [M1] `yass status` shows, by default, work on changes that hasn't merged yet: which local and pushed branches with recent commits (the last 30 days, or what `--since` says) touch each change or piece, their progress there, and their latest **Next:**, as of the last fetch, without leaving git. `--no-branches` leaves it out.
- **R10** [M2] `yass status` and `yass root` say whether the plans are inline (in the same git repo as the code) or separate.
- **R11** [M2] In a linked worktree, a relative `path:` resolves to the same folder as in the main checkout. When the plans folder is missing, YASS never suggests `yass init` from a linked worktree.
- **R22** [M2] When `path:` points outside the repo, YASS keeps a gitignored `yass/` symlink to the plans folder in each checkout and worktree, created and repaired by `yass` commands. It's a convenience: nothing depends on it.
- **R23** [M2] Paths in `yass.yaml` (`path:` and `repos:`) accept `${VAR:-default}`, so side-by-side clones work with no setup and anyone can override them.
- **R12** [M2] Citations can name their repo (`code: web@a1b2c3d`). Each code repo checks its own citations and says which ones it didn't check, without warning about them. An unqualified citation keeps today's behavior.
- **R13** [M2] `yass archive` refuses (unless forced) a change whose done boxes cite commits that are missing or unmerged, and lists any citations it couldn't check from where it runs.
- **R14** [M2] With separate plans, `yass status` warns when the plans repo has uncommitted changes or is behind its upstream.
- **R15** [M3] With separate plans, every code commit carries a `yass:` trailer naming its change, or `yass: none`. A commit-msg check warns about a missing or invalid one, or rejects it when `yass.yaml` sets `trailers: require`. `yass hook --range` does the same in CI, squash merges included. With inline plans, nothing asks for trailers.
- **R20** [M3] The hooks add the `yass:` trailer automatically, working out the change from `yass use <change>` (per worktree), then the branch name, then earlier commits on the branch, so neither agents nor people type it.
- **R21** [M3] A `yass:` trailer can name the criteria a commit works toward (`ac:ac1,ac2`), and `yass status <change>` lists each criterion's merged commits.
- **R16** [M3] `yass status` lists each change's code commits from its trailers on the merged branch, so finding a change's code doesn't depend on SHAs that a squash merge rewrites.
- **R17** [M3] A plans repo can map repo names to code checkouts (`repos:` in its `yass.yaml`), so `yass status` run there, locally or in CI, checks every citation and trailer. A repo whose checkout isn't there is reported as unchecked, not as an error.
- **R18** [M3] The `yass-work` skill's separate-plans guidance covers the whole routine. At session start, pull the plans and check their state. Commit code first, then plans. Cross-link the code PR and the change. Treat git latitude separately for each repo. Never run `yass init` to fix a missing plans folder.

## Non-goals
- Orphan branches, git submodules, subtrees, or manifest tools (`repo`, `west`) as a home for plans.
- Version control other than git (Piper, Perforce, Sapling).
- Trailers, or any new rule, for inline plans. The monorepo setup stays as simple as it is.
- CI plugins or integrations to install. YASS ships recipes and `yass paths`, not per-CI products.
- Moving plans out of the monorepo to save CI cost. That's solved inside the repo.
- Automatically rewriting SHA citations after a squash merge.
- A git merge driver for YASS files. Pieces owning their own files avoids the conflicts instead.
- Integrating with trackers (Jira, Linear), or keeping progress in one. Files stay the source of truth. A one-way projection from main into a tracker may be a later change.

## Milestones
| Milestone | When it ships, a user can … |
|---|---|
| M1 | roll YASS out across a large monorepo with many teams, with plan-only commits costing no CI, owned per team, fast at scale, and parallel work visible and conflict-free |
| M2 | keep plans in a separate repo, with worktrees, shared plans repos and archiving all working correctly |
| M3 | rely on automatic, checked trailers to link code to plans in a separate-repo setup, and check everything from the plans repo, including in CI |

## Open questions
None. The four raised while shaping were settled on 2026-10-04 (see design.md and `change.md` Decisions).


## yass/archive/2026/10/2026-10-04-paths-that-don-t-exist-yet-compare-throu/change.md

---
platforms: [all]
source: CI run 37235844602 (macos-latest)
follows: 
blocked:
archived: 2026-10-04T21:29:48Z
---
# Paths that don't exist yet compare through symlinks

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
A folder that doesn't exist yet compares correctly with the repo it's inside, even when a parent folder is a symlink (macOS's /var → /private/var), so yass init --private and other checks work in any temp or home layout.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Given a path that doesn't exist yet under a symlinked folder, when YASS makes it canonical, then the symlink is resolved and the rest of the path kept — verify: go test TestCanonResolvesMissingPathsThroughSymlinks
- [x] Given the e2e suite run in macOS's default temp folder (under /var), then it passes, including `yass init private --private` (§22) — verify: tests/e2e.sh with no workdir, on macOS

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `canon` resolves the deepest existing folder of a missing path
- [x] §22's checks match the plans path by its tail, not the unresolved temp path

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-04 (claude)
- Did: the first push of the private-team-folders stack failed on GitHub's macOS runner (run 37235844602, §22: "--private needs a folder inside the repo"). `canon` only resolved symlinks for paths that exist, so the new `private/` stayed under /var while the repo top resolved to /private/var. Local runs used a scratch folder without symlinks and missed it. Reproduced with tests/e2e.sh in the default temp folder, fixed, and checked both ways: e2e 279/279 in each; go test, go vet, examples.
- Next: archive once the fix's CI run on main passes.


## yass/archive/2026/10/2026-10-04-stacked-prs-and-squash-merges/2026-10-05-delivered-criteria-count/change.md

---
platforms: [all]
source: plan.md
follows: 
blocked:
created: 2026-10-05T20:06:03Z
---
# Delivered criteria count

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
A parent criterion counts as done in yass status and yass archive once every piece delivering it is done, yass archive marks those criteria [x], and an e2e scenario lands a three-piece change one squash-merged branch at a time.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC7–AC13 in [plan.md](../plan.md)

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] Parse `Delivers` boxes (lists, ranges, trailing words); go test
- [x] Count delivered criteria in `status` progress and `next:`
- [x] `yass archive`: don't refuse for them; mark them `[x]` in the archived `plan.md`
- [x] e2e: each case, and the three-piece squash scenario

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- A criterion counts as delivered once every piece that names it in a `Delivers` box is done, whatever words surround the reference: "and AC19's preview card" in one piece and "AC19's tags" in another means AC19 is met when both are - that's how pieces in this repo already split a criterion (claude, 2026-10-09)
- Delivered criteria count in `yass status` without the file changing (`countedBoxes`); only `yass archive` writes the tick, as part of the move - the archive is the one closing commit (R6), and the hook already allows the archive move to tick boxes (claude, 2026-10-09)
- `markDelivered` skips HTML comments, so a plan's commented-out example `AC1` stays as it was - the same trap caught me twice when ticking criteria by script during archive-that-scales and yass-update (claude, 2026-10-09)
- README and `yass-plan` say so, so nobody plans a closing commit for the parent (claude, 2026-10-09)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-09 (claude)
- Did: built and tested delivered criteria: `deliveredIDs`, `delivered`, `countedBoxes` in status, and `markDelivered` in `yass archive`. `go test` (`TestDeliveredIDs`, `TestMarkDelivered`), `tests/e2e.sh` section 38 (576 ok, with three pieces landed one squash-merged branch at a time, the archive closing them, and main's history passing the hook), `tests/examples.sh`, `tests/site.sh`. The piece is done, and so is the change.
- Next: archive the change, in its own pull request.


## yass/archive/2026/10/2026-10-04-stacked-prs-and-squash-merges/2026-10-05-squash-notes/change.md

---
platforms: [all]
source: plan.md
follows: 
blocked:
created: 2026-10-05T20:06:03Z
---
# Squash notes

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
docs/monorepo.md and yass-work say what squash merges mean for YASS's rules (a commit that must stand alone needs its own PR), yass-work asks for latitude before pushing or opening a PR, and yass-log names a squash-merged commit's PR. No workflow guidance.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC1–AC6 in [plan.md](../plan.md)

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `docs/monorepo.md`: a short section on squash merges
- [x] `yass-work`: "apart from code" and the squash-merge sentence; push and PRs need latitude
- [x] `templates/agents.md`: "apart from code" on the intent and archive lines
- [x] `yass-log`: name a squash-merged commit's PR
- [x] Review the diff: no workflow guidance

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- The latitude line in the `AGENTS.md` section gets push and pull requests too, not only `yass-work` - the section repeats `yass-work`'s latitude rule, and they should match (claude)
- `yass-log` also says to follow an archived change's `change.md` across its move (`git log --follow`): trying AC5, a plain `git log -- <folder>` on an archived change showed only the archive commit (claude)
- This repo's own `AGENTS.md` and installed playbooks pick the new text up at the next release's `yass upgrade`; upgrading them with a development build would stamp a pseudo-version (claude)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->

### 2026-10-05 (claude)
- Did: `docs/monorepo.md` gets a "Squash merges" section (apart from code means a pull request apart from code; intent, progress and archive can share one; how to split and stack is the team's call; the hook checks per commit). `yass-work`'s git paragraph asks for latitude to push and open pull requests and says what "separately" means under squash merges; its Finish step says the same for the archive. The `AGENTS.md` template says it on the intent, latitude and archive lines. `yass-log` names the `(#N)` pull request of a squash-merged commit, and follows an archived change's file across its move. e2e §30 (4 checks: the section's text after init, and after `yass upgrade` of an older one). All 390 e2e checks, examples, `tests/site.sh` and `go test` pass. AC5 tried by hand: following the steps for the hooks change names #15 (the decision) and #16 (the code). No workflow guidance added: no stacking tools, merge styles or branching advice in the kit diff.
- Next: Jaime reviews the text (AC1, AC2, AC4, AC6); then piece 2, `delivered-criteria-count`, stacked on this branch.
### 2026-10-09 (claude)
- Did: Jaime reviewed the text and merged it (#31); AC1, AC2, AC4 and AC6 done. The piece is done.
- Next: none here.


## yass/archive/2026/10/2026-10-04-stacked-prs-and-squash-merges/change.md

---
platforms: [all]
source: design discussion with Jaime, 2026-10-04
follows: 
blocked:
archived: 2026-10-09T14:22:20Z
---
# Stacked PRs and squash merges

## Goal
<!-- Intent. One paragraph; the detail is in prd.md (why and what) and plan.md (how and acceptance). -->
YASS stays compatible with however people land work, and is clear about what squash merges mean for its own rules: a commit that has to stand alone (a plan revision, an archive move, a queue reorder) needs its own PR. `yass-work` asks for latitude before pushing or opening a PR, as it does before committing. A large change needs no closing commit per piece: parent criteria count as done once the pieces delivering them are, and the archive marks them. `yass-log` traces a squash-merged decision to its PR.

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)" for calls that span the whole change. -->
- Its own change, not part of `2026-10-04-monorepos-at-scale-and-plans-repos` - it cuts across inline and separate plans, and is mostly rules, skill guidance and the hook (Jaime)
- The basic rule: YASS-only PRs (PRDs, plans, archives) skip CI; code and its YASS progress land together in PRs that run CI; the large-monorepo docs say so (Jaime)
- Under squash merges, the unit YASS rules apply to is the PR, not the commit - a squash keeps a PR's content and drops its commit boundaries (Jaime, claude) - superseded below
- `merges: squash | commits` in `yass.yaml`, defaulting to `squash` - most orgs squash, and the per-PR rules are the stricter reading; repos that keep merge commits opt out to the per-commit rules (Jaime) - superseded below
- This change takes over counting delivered parent criteria as done (R5, R6) from `2026-10-04-monorepos-at-scale-and-plans-repos` piece 5, which keeps only "pieces own their files" (the rule and the hook's heads-up); its AC12 is dropped there in favour of R5 here (Jaime)
- PRD approved (Jaime, 2026-10-04)
- Drastically simplified (2026-10-05): YASS's rules stay per commit, and the playbooks stay workflow-agnostic. How to branch, stack, sync and merge is the team's workflow, which their harness learns from them (gh-stack ships its own agent skill). Kept: what squash merges mean for YASS's own rules (R2, R13), latitude before pushing or opening PRs (R14), `yass-log` naming a squash commit's PR (R9, now M1), delivered criteria with no closing commit per piece (R5, R6), and an e2e test (R10, now about those). Dropped: R1 (`docs/monorepo.md` already states the basic rule), R3 (checking a PR's range as one squash; per-commit is as strong a check as YASS needs), R4 (the branch look-back, which assumed a workflow and would warn wrongly under merge commits), R7 (branching from main), R8 (PR shapes), R11 (dogfooding), the `merges:` key, and the never-committed R12, R15, R16 (a landing routine, syncing from a worktree, detecting the landing workflow). This also supersedes the PR-as-the-unit and `merges:` decisions above (Jaime)
- Planned in two pieces: the squash notes (M1, text) and delivered criteria counting with its e2e (M2), independent of each other; no design.md, since nothing here is hard to undo (claude)
- "Their own commit" means apart from code, and under squash merges a PR apart from code; intent, progress and archive commits can share a PR. The squash notes say so in `docs/monorepo.md`, `yass-work` and the `AGENTS.md` section (R2, R13, AC1–AC3), so agents don't split a PRD from its plan, or a change's last progress from its archive, into separate PRs as happened here (Jaime)

## Log
<!-- Progress. Change-level notes; each piece keeps its own Log.
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->


## yass/archive/2026/10/2026-10-04-stacked-prs-and-squash-merges/plan.md

# Stacked PRs and squash merges: plan
<!-- Intent: how, and how we'll know it's done. With code, only mark boxes here ([/] in progress, [x] done);
     anything else is a plan revision in its own commit. -->

## Approach
<!-- The shape of the solution. Hard-to-undo calls go in design.md. -->
**M1 is a few sentences of text.** [`docs/monorepo.md`](../../../docs/monorepo.md) gets a short section on squash merges: a squash folds a PR into one commit, so the commits YASS keeps apart from code (a plan revision, an archive move, a `queue.md` reorder) need a PR apart from code, while intent, progress and archive commits can share one; how a team splits and stacks its branches is its own call. [`yass-work`](../../../kit/.agents/skills/yass-work/SKILL.md) and the `AGENTS.md` section ([`templates/agents.md`](../../../internal/yass/templates/agents.md)) say the same where they ask for intent and archives in their own commits ("apart from code"), and its "Git is up to whoever is driving" paragraph adds pushing a branch and opening or updating a PR to what needs latitude. [`yass-log`](../../../kit/.agents/skills/yass-log/SKILL.md), when it explains a decision from history, names the PR of a squash-merged commit from the `(#N)` in its subject. No workflow guidance is added anywhere.

**M2 counts delivered criteria.** A piece's `Delivers AC1, AC3–AC5 in [plan.md](../plan.md)` box already names the parent criteria it delivers. `yass status` and `yass archive` ([`internal/yass/repo.go`](../../../internal/yass/repo.go), [`commands.go`](../../../internal/yass/commands.go)) read those boxes: a parent criterion counts as done once every piece delivering it is done, in progress and `next:`, and `yass archive` doesn't refuse for it. The archive move marks those criteria `[x]` in the archived `plan.md`. An e2e scenario lands a three-piece change one squash-merged branch at a time and checks it.

## Acceptance
<!-- Per milestone, one observable behavior each:
### M1
- [ ] AC1 (R1) Given …, when …, then … — verify: <test, flow, or manual steps> -->
### M1
- [x] AC1 (R2) Given `docs/monorepo.md`, then it says that a squash merge folds a PR into one commit, so a plan revision, an archive move or a `queue.md` reorder needs a PR apart from code, that intent, progress and archive commits can share a PR, and that splitting and stacking branches is the team's call — verify: manual: review
- [x] AC2 (R13) Given `yass-work`, then where it asks for intent and archives in their own commits, it says that means apart from code, and under squash merges a PR apart from code — verify: manual: review
- [x] AC3 (R13) Given the `AGENTS.md` section `yass init` writes, then its intent and archive lines say "apart from code", and `yass upgrade` brings existing sections up to it — verify: e2e (the section's text); manual: review
- [x] AC4 (R14) Given `yass-work`'s git paragraph, then pushing a branch and opening or updating a PR need the same latitude as committing, branching and rebasing, and without it the playbook shows the commands — verify: manual: review
- [x] AC5 (R9) Given `yass-log` explaining a decision whose commit was squash-merged, then it names the PR from the `(#N)` in the commit's subject — verify: manual: ask it why `yass init --hooks` leaves existing hooks alone, and check it names #16
- [x] AC6 (non-goals) Given the kit after M1, then it adds no workflow guidance: nothing on how to branch, stack, sync, merge or which tools to use — verify: manual: review the diff

### M2
- [x] AC7 (R5) Given a parent criterion named by `Delivers` boxes in two pieces, when both pieces are done, then `yass status` counts the criterion done in the parent's progress and doesn't offer it as `next:` — verify: e2e
- [x] AC8 (R5) Given one of those pieces still open, then the criterion stays open — verify: e2e
- [x] AC9 (R5) Given a criterion no piece delivers, then it counts by its own box, as today — verify: e2e
- [x] AC10 (R5) Given a large change whose only open boxes are criteria its done pieces deliver, when you run `yass archive`, then it doesn't refuse — verify: e2e
- [x] AC11 (R6) Given that archive, then the archived `plan.md` has those criteria marked `[x]`, and no other box changed — verify: e2e
- [x] AC12 (R5, R6) Given `Delivers` boxes written as lists and ranges (`AC1, AC3`, `AC2–AC9`, `AC2-AC9`, with trailing words such as "and AC19's preview card" or "in plan.md"), then each names the right criteria — verify: go test
- [x] AC13 (R10) Given an e2e scenario that lands a three-piece change one squash-merged branch at a time (`git merge --squash`, rebasing the rest onto main), then `yass status` is right after each landing, no piece needs a closing commit, `yass archive` closes the change, and every commit on main passes the hook — verify: e2e

## Pieces
<!-- PR-sized pieces, each its own folder in here: `yass new "<title>" --in <this change>`.
     `yass status` lists them; say here what order they go in and why. -->
1. `squash-notes` (M1): the `docs/monorepo.md` section, the two `yass-work` sentences and the `yass-log` note. Text only, so it can land first.
2. `delivered-criteria-count` (M2): parsing `Delivers`, counting in `status` and `archive`, marking on archive, and the e2e scenario. Independent of piece 1.

## Validation
<!-- How the whole thing is verified before it's called done: suites, platforms, manual passes. -->
- CI on each piece: `go test`, `tests/e2e.sh` (with the new scenario), `tests/examples.sh`, `tests/site.sh`, on Ubuntu and macOS.
- Jaime reviews the M1 text, which is judged by reading.


## yass/archive/2026/10/2026-10-04-stacked-prs-and-squash-merges/prd.md

# Stacked PRs and squash merges: PRD
<!-- Intent: what should change or exist, and why. Not a description of the current system;
     the code and its docs are that. Edit this file only in commits without code. -->

## Why
<!-- The problem, with evidence. -->
People land YASS work in many ways: stacked pull requests squash-merged one at a time, branches merged with merge commits, or local merges with no code host at all. YASS's rules hold in all of them because they're about commits: intent edits, archive moves and `queue.md` reorders each get their own commit, and progress rides in the same commit as its code. How a branch is split, stacked and merged is the team's workflow, which their harness learns from them; YASS doesn't need to teach it.

Landing large changes in this repo with squash-merged stacks (2026-10-04 and 2026-10-05) showed four places where YASS itself falls short, whatever the workflow:
- **A squash folds a PR's commits into one,** so a commit that has to stay apart from code (an archive move, a plan revision, a queue reorder) needs a PR apart from code. Nothing in YASS says so, and "in its own commit" reads stricter than the hook's actual rule (apart from code): an agent split a change's last progress from its archive move, and a PRD revision from its plan, into separate PRs that could have shared one.
- **`yass-work` asks for latitude to commit, branch and rebase, but not to push or open PRs,** so an agent following it published two PRs before the human had reviewed them locally.
- **Closing commits multiply PRs.** With pieces owning their files, a parent's criteria are marked in a separate closing commit, so every finished piece costs an extra plan-only PR.
- **`yass-log` reads commits,** and after a squash merge the PR is what a decision maps to.

## Users and outcomes
<!-- Who it's for, and measurable targets ("p50 time to log < 5s"). -->
Anyone landing YASS work, through pull requests (squash-merged or not) or local merges.
- Someone who squash-merges learns from YASS's own docs and `yass-work` that a commit that must stand alone needs its own PR, before it bites.
- An agent following `yass-work` never pushes or opens a PR without the latitude to.
- A large change with N pieces lands with no closing commit per piece: its parent criteria count as done once the pieces delivering them are, and the archive is the one closing commit.
- `yass-log` names the PR behind a squash-merged decision.

## Requirements
<!-- One line each: "- **R1** [M1] A user can …". IDs are never reused. -->
- **R2** [M1] `docs/monorepo.md` says what squash merges mean for YASS's rules: a squash folds a PR into one commit, so a plan revision, an archive move or a `queue.md` reorder needs a PR apart from code, though intent, progress and archive commits can share one; how a team splits and stacks its branches is up to it.
- **R13** [M1] `yass-work` and the `AGENTS.md` section say the same where they ask for intent and archives in their own commits: "their own commit" means apart from code, and under squash merges that means a PR apart from code.
- **R14** [M1] `yass-work` treats pushing a branch and opening or updating a PR like committing, branching and rebasing: only with that latitude, otherwise it shows the human the commands.
- **R9** [M1] `yass-log` attributes a squash-merged commit to its PR, from the `(#N)` in its subject, when it explains a decision.
- **R5** [M2] A parent criterion counts as done in `yass status` and `yass archive` once every piece delivering it (by its `Delivers AC…` box) is done, so no closing commit is needed per piece.
- **R6** [M2] `yass archive` marks a large change's delivered parent criteria `[x]` as part of the archive move, so the archive is the one closing commit.
- **R10** [M2] An e2e scenario lands a three-piece change one squash-merged branch at a time: `yass status` is right after each landing, no piece needs a closing commit, the archive closes it, and main's history passes the hook.

## Non-goals
<!-- What this change will not do. Agents treat these as walls. -->
- Workflow guidance: how to branch, stack, sync, choose a merge style or land a stack, or which tools to use (`gh stack`, Graphite and the like teach their own). The playbooks stay compatible with however people work, and their harness learns the rest from them.
- Checking PRs or branches as a whole. The hook stays per commit: a commit that mixes intent and code is the line YASS draws, and a branch with separate intent and code commits can always be split into stacked branches by whoever needs it.
- Settings for any of this in `yass.yaml`.
- A stacking tool, or talking to a code host's API.
- Limits on PR size or stack depth.
- Changing the rule that progress rides in the same commit as the code it describes.
- The `yass:` trailer and SHA citations under squash merges: `2026-10-04-monorepos-at-scale-and-plans-repos` (M3).

## Milestones
| Milestone | When it ships, a user can … |
|---|---|
| M1 | squash-merge YASS work knowing which commits need their own PR, trust an agent not to publish without the latitude to, and trace a squash-merged decision to its PR |
| M2 | land a large change with no closing commit per piece, the archive closing it, as an e2e test shows |

## Open questions
None.


## yass/archive/2026/10/2026-10-04-the-binary-carries-the-playbooks/2026-10-04-init-writes-the-whole-repo/change.md

---
platforms: [all]
source: plan.md
follows: 
blocked:
---
# Init writes the whole repo

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
Every yass binary carries the kit (the playbooks and the hook script) for its version, and yass init writes all of a repo's YASS files from it, stamped with that version, with the --claude, --global and --hooks options install.sh has today. Files that already exist are kept, as install.sh does without --upgrade. install.sh is unchanged except that it passes --global on to yass init, so yass init doesn't add repo copies of playbooks that went to the user folder.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC1–AC8 in [plan.md](../plan.md)

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `kit/` as a Go package that embeds its files; go test lists them
- [x] Stamp writers: playbook frontmatter `metadata`, `AGENTS.md` marker, hook header
- [x] `yass init` writes playbooks, hook, Claude copies and `CLAUDE.md` import, `core.hooksPath`; `--claude`, `--global`, `--hooks`
- [x] README CLI reference: `yass init` options
- [x] e2e: `yass init` alone, each option, stamps

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- `install.sh` passes `--global` on to `yass init` - `yass init` now writes playbooks, and without it, an `install.sh --global` would get repo copies too; revised before building (claude)
- The kit is written only when `yass init` sets up the repo itself: not for a team folder (`yass init <dir>`), and not with `--no-agents`, which now means "leave AGENTS.md, the playbooks and the hook alone". Team and private folders stay as clean as before, and the many `--no-agents` setups keep working (claude)
- Stamps keep a version's build metadata (`+dirty`): piece 2 needs it to rewrite files a `+dirty` build wrote, and SemVer ignores it when comparing (claude)
- The source files in `kit/` stay unstamped; `kit/embed.go` embeds them with `all:` so the `.agents` folder is included (claude)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->

### 2026-10-04 (claude)
- Did: `kit/` is a Go package embedding the playbooks and the hook; `yass init` writes them stamped (`metadata.yass-version` in the frontmatter, `version=` in the `AGENTS.md` marker, `# yass-version:` in the hook) with `--claude`, `--global` and `--hooks`, keeping files that exist; `install.sh` passes `--global` on. Tests: `go test` (the kit's contents, stamping keeps the frontmatter valid YAML with name and description), e2e §24 (init from the binary alone, each option, files match `kit/` but for the stamp), all 309 e2e checks, examples, `tests/site.sh`. README's CLI reference lists the options.
- Next: Jaime reviews; then piece 2, `init-upgrades-by-version`.

### 2026-10-05 (claude)
- Did: rebased onto the plan revision (`yass upgrade`, agent files per folder); `go test` passes.
- Next: piece 2 is now `agent-files-per-folder`, which also moves the agent files' location away from the repo root's and stops `yass init <folder>` writing the root `AGENTS.md`.


## yass/archive/2026/10/2026-10-04-the-binary-carries-the-playbooks/2026-10-04-install-sh-installs-the-binary/change.md

---
platforms: [all]
source: plan.md
follows: 
blocked: 2026-10-04-the-binary-carries-the-playbooks/2026-10-05-yass-upgrade
---
# install.sh installs the binary

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
install.sh only installs the yass binary, and says where, whether it's on PATH, and that yass init sets up a repo; the documented one-liner chains yass init itself. Given old-style repo arguments, install.sh installs nothing and prints the two commands instead. Release archives drop the kit files. The README, docs/install.md and the landing page describe the new model.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC20–AC27 in [plan.md](../plan.md)

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `install.sh`: binary install and PATH report only; refuse old-style repo arguments, printing the commands to use (`yass upgrade` for `--upgrade`)
- [x] `.goreleaser.yaml`: archives without `kit/`
- [x] e2e: rework the `install.sh` sections; unpacked release without `kit/`
- [x] README, `docs/install.md`, landing page: the new model and joining
- [x] Release candidate: the one-liner in a fresh repo (AC24)

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- `install.sh` requires `--bin-dir`: installing the binary is all it does, so without a folder there's nothing to do (claude)
- The "YASS is set up; commit it, then ask your agent" steps `install.sh` used to print now come from `yass init`, when it sets up a repo for the first time (claude)
- The agent prompt now has the agent run `install.sh --bin-dir`, then `yass init` with your options; the page's copy changed with it (claude)
- `docs/install.md` is reorganized around the two steps: get the binary (a release, source, `go install`), then "Setting up a repo" with `yass init`'s options, which replaces the old `install.sh` options table (claude)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->

### 2026-10-05 (claude)
- Did: `install.sh` only installs the binary: from `$YASS_BIN`, the binary next to it (a release folder or a clone's `bin/`), or a downloaded release checked against `checksums.txt`; it reports PATH and points at `yass init`; old-style arguments (a repo path, `--claude`, `--global`, `--hooks`, `--path`, `--upgrade`) install nothing and print the commands to use. Release archives drop `kit/`. e2e §8 rewritten for it; every e2e and `tests/examples.sh` setup now uses `yass init`. README install section, `docs/install.md`, `CONTRIBUTING.md`, `docs/skills.md` and the landing page's install tabs describe the two steps. Checks: e2e 355, examples, `tests/site.sh` 60, `go test`; a GoReleaser snapshot build, whose archive's `install.sh` installed the binary and whose `yass init` set up a fresh repo.
- Next: AC24, the piped one-liner against a real release candidate (`v0.3.0-rc.1`), which needs the branches merged and a tag; then piece 5, `status-knows-the-versions`.

### 2026-10-05 (claude)
- Did: release-candidate checks against `v0.3.0-rc.1`, published as a GitHub pre-release (latest stays v0.2.0). The documented one-liner with `YASS_VERSION=v0.3.0-rc.1`, in a fresh repo with `--bin-dir` not on PATH: downloaded, checksum ok, installed, PATH warning with the `~/.zshenv` fix, and the chained `yass init` set up the repo stamped `0.3.0-rc.1` (AC24). The README's manual route: checksum ok, `gh attestation verify` passes (SLSA provenance signed by `release.yml` at the tag), the archive holds only the binary, `install.sh` and docs, and its `install.sh` installs the binary. A repo set up by v0.2.0's own `install.sh --claude --hooks` gets `yass status`'s note, and the release candidate's `yass upgrade` leaves it identical to a fresh `yass init --claude --hooks`, hook setting kept. `yass upgrade` on this repo: stamps only, `kit/` untouched.
- Next: done.


## yass/archive/2026/10/2026-10-04-the-binary-carries-the-playbooks/2026-10-04-status-knows-the-versions/change.md

---
platforms: [all]
source: plan.md
follows: 
blocked: 2026-10-04-the-binary-carries-the-playbooks/2026-10-05-yass-upgrade
---
# Status knows the versions

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
yass status adds one line when the binary and a repo's YASS files are out of step: upgrade your binary, or yass init would upgrade the repo.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC28–AC30 in [plan.md](../plan.md)

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `yass status`: the version line
- [x] e2e: older, newer, equal, none

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- `yass status` checks only the files at the repo root and the user folder's playbooks, not the whole repo: status runs constantly and has to stay fast in large monorepos, while `yass upgrade` does the full search (claude)
- The version line is a `note:`, not a `warning:`, so `yass status --strict` in CI doesn't fail when the runner's binary is a little behind the repo (claude)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->

### 2026-10-05 (claude)
- Did: `yass status` adds one note when the binary and the repo's YASS files are out of step: newer files say to upgrade the binary, older ones that `yass upgrade` would upgrade them; nothing at the same version or with no YASS files. e2e §27 (5 checks, including `--strict` staying green); all 360 e2e checks, examples, `tests/site.sh`, `go test`. README and `docs/install.md` mention it.
- Next: Jaime reviews the stack; then merge, tag a release candidate, and run AC24 (the piped one-liner) and the release-candidate checks in the plan's Validation.


## yass/archive/2026/10/2026-10-04-the-binary-carries-the-playbooks/2026-10-05-agent-files-per-folder/change.md

---
platforms: [all]
source: plan.md
follows: 
blocked:
---
# Agent files per folder

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
yass init <folder> --agents puts the AGENTS.md section and the playbooks (and Claude's copies with --claude) in that folder, for a monorepo part with its own agent files; the hook stays at the repo root. Without --agents, yass init <folder> sets up only the folder's planning and touches no AGENTS.md. yass init on a set-up repo or folder writes only what's missing and points at yass upgrade when files are older.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC31–AC33 in [plan.md](../plan.md)

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `yass init`: the agent files' folder separate from the repo root's hook; `--agents` for a folder
- [x] `yass init <folder>` without `--agents` leaves `AGENTS.md` alone (it used to write the root's)
- [x] `yass init` on a set-up repo or folder: keep what's there, point at `yass upgrade` when files are older
- [x] README CLI reference and usage: `--agents`
- [x] e2e: monorepo folder with and without `--agents`, `--claude` in a folder, re-running init

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- The version comparison (`golang.org/x/mod/semver`) and the stamp reader arrive here, not in piece 3: AC33's "files older than the binary" needs them. `x/mod` v0.27.0 is pinned because later versions require Go 1.26; it needs Go 1.23, under the 1.24 floor piece 3 sets (claude)
- `yass init` now keeps an existing YASS section in `AGENTS.md` instead of rewriting it on every run: setting up writes what's missing (R16), and updating is `yass upgrade`'s job (claude)
- Known gap until piece 3: `install.sh --upgrade` still copies the playbooks and the hook, but the `AGENTS.md` section it used to refresh through `yass init` now stays as it is. `yass upgrade` (piece 3) closes it, and piece 4 points `--upgrade` there; these branches merge together (claude)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->

### 2026-10-05 (claude)
- Did: `yass init <folder> --agents` puts the `AGENTS.md` section and playbooks (and Claude's copies and import with `--claude`) in the folder, with the hook only at the repo root; `yass init <folder>` alone is planning only and leaves every `AGENTS.md` alone; re-running `yass init` writes what's missing, keeps the rest, and notes `yass upgrade` when kept files are older or unstamped. Added `readStamp` and `olderThanBinary` (go test). e2e §25 (18 checks); all 327 e2e checks, examples and `tests/site.sh` pass. README CLI reference lists `--agents`.
- Next: Jaime reviews; then piece 3, `yass-upgrade` (its first step, the comparison, is already here).

### 2026-10-05 (claude)
- Did: fixed two e2e §25 checks that failed in CI on this PR (donjaime/yass#5). At this commit `go.mod` says Go 1.23, which builds `yass` without a version (`dev`), so "`yass upgrade` updates them" never appeared; one check also hard-coded a `0.0.1` stamp, which a CI build's `v0.0.0-…` sorts below. Both now use a binary built with a known version (`vbin`, the helper the `yass-upgrade` piece adds). e2e passes with an unversioned build (as CI's Go 1.23 makes) and a versioned one: 327 each.
- Next: rebase the stack onto this, and merge #5 once CI is green.


## yass/archive/2026/10/2026-10-04-the-binary-carries-the-playbooks/2026-10-05-yass-upgrade/change.md

---
platforms: [all]
source: plan.md
follows: 
blocked: 2026-10-04-the-binary-carries-the-playbooks/2026-10-05-agent-files-per-folder
---
# yass upgrade

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
`yass upgrade`, run anywhere in a repo, finds every YASS file from the repo root down (playbooks in any `.agents/skills/` or `.claude/skills/`, every `AGENTS.md` with the YASS marker, the hook), plus the user folder's playbooks, and compares each one's stamp with its own version: older or unstamped files are upgraded where they are, the same version is up to date (unless the binary is +dirty), and any newer file stops it with upgrade-your-binary. It says which it did, lists every file it wrote, and labels files outside version control. Builds from source need Go 1.24, so clone builds carry a version. The README and docs/install.md say how to upgrade.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC9–AC19, AC34 and AC35 in [plan.md](../plan.md)

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `golang.org/x/mod/semver`; the version comparison and its decision table (go test)
- [x] Discovery: `git ls-files` for nested playbooks, marked `AGENTS.md` files and the hook; the user folder's playbooks
- [x] `yass upgrade`: upgrade / up to date / refuse, with the messages and exit codes; unversioned binaries refuse (in `yass init` too)
- [x] File listing, outside-version-control label
- [x] `go 1.24` in `go.mod`; docs that say 1.22
- [x] README CLI reference and an Upgrading section in the README and `docs/install.md`
- [x] e2e: upgrade paths, nested monorepo copies, a v0.2.0 fixture, identical-to-fresh check
- [x] Dogfood: `yass upgrade` in this repo

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- `yass upgrade` skips folders `yass.yaml` lists under `ignore:`, as the rest of YASS does; this repo ignores `kit/`, whose playbooks and hook are the binary's unstamped sources. A dry run here first stamped them, which is how this showed up (claude)
- A skills folder that holds any `yass-*` playbook gets every playbook the binary carries, so an upgrade also adds playbooks a later version introduces (claude)
- A hand-edited playbook loses its stamp, so the upgrade message reports it among the "unstamped" versions it upgraded from; that's accurate, and the diff shows the edit (claude)
- AC23 (`yass upgrade` outside git) is done here with the command, though the plan lists it under piece 4 (claude)
- Version checks in e2e use binaries built with known versions (`vbin`): CI builds from a shallow checkout without tags, whose pseudo-version (`v0.0.0-…`) is older than any real stamp (claude)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->

### 2026-10-05 (claude)
- Did: `yass upgrade` (discovery with `git ls-files` from the repo root, plus the user folder; upgrade, up to date, refuse when anything is newer; unstamped counts as older; a `+dirty` build rewrites; outside git refuses), the refusal for unversioned binaries in `yass init` too, the outside-version-control label in both, `go 1.24` in `go.mod` and the docs, and the upgrade docs in the README and `docs/install.md`. Tests: `go test` (the decision table, AC14), e2e §26 (29 checks, including a v0.2-style unstamped repo that upgrades to exactly what a fresh init writes, and a monorepo upgraded from a subfolder), all 356 e2e checks, examples, `tests/site.sh`. Dogfood dry run in a temporary worktree of this repo: the diff is stamps only, across both sets of playbooks, `AGENTS.md` and the hook. The real stamps land here with a release candidate, not a pseudo-version.
- Next: Jaime reviews; then piece 4, `install-sh-installs-the-binary`. Until then `install.sh` still sets up a repo when run in one, and the upgrade docs only point at it for getting the binary.


## yass/archive/2026/10/2026-10-04-the-binary-carries-the-playbooks/change.md

---
platforms: [all]
source: Jaime, 2026-10-04: installing the binary and setting up a repo are two steps that install.sh does as one, which confuses; noted while fixing the install docs (follows-up 2026-10-04-install-say-where-to-run-it-and-how-yass)
follows: 
blocked:
archived: 2026-10-05T14:20:50Z
---
# The binary carries the playbooks

## Goal
<!-- Intent. One paragraph; the detail is in prd.md (why and what) and plan.md (how and acceptance). -->
Using YASS takes two steps that each say what they do: get the `yass` binary, once per machine, and run `yass init` in each repo. The binary carries the playbooks and the hook for its version, so `yass init` sets up a whole repo, or a monorepo folder with its own agent files, with no release download, `yass upgrade` brings every copy in the repo up to the binary's version (or says it's already up to date), and a teammate joining a repo needs only the binary. The one-line install runs both, `install.sh` and then `yass init`. Every file YASS installs records the version that wrote it, and `yass status` says when the binary and the repo are out of step.

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)" for calls that span the whole change. -->
- Noted as a future large change rather than built now - it changes the CLI, `install.sh` and the release; the install docs were fixed first, explaining the two steps instead of splitting them (Jaime)
- Scope: `yass init` does the whole repo setup from the binary, re-running it upgrades, the repo records its YASS version, and `install.sh` becomes a binary installer; Homebrew and self-update, and a minimal `install.sh`-modes version, were considered and left out (Jaime)
- The one-line install keeps working: it installs the binary, then runs `yass init` (with any setup flags) when run in or pointed at a repo, and says which step did what (Jaime) - superseded below
- Versions live in the files YASS installs: the binary carries its version, the playbooks have it in their frontmatter, the `AGENTS.md` section in its markers (and the hook script in a header), and `yass init` reads them to decide what to do. That also covers playbooks installed with `--global` (Jaime; the hook header added by claude)
- `yass init` is idempotent and does the upgrade itself: no `--upgrade` flag. It detects an upgrade and says so, and says "already up to date" when there's nothing to do (Jaime) - superseded below
- `yass init` never downgrades: newer files than the binary mean upgrade the binary (claude, for review)
- The monorepo change's `yass-trailers` piece waits for this change and ships its hook scripts through `yass init` (Jaime)
- Builds from a clone carry the release they follow plus a commit, and `+dirty`; a `+dirty` build rewrites files stamped with its own version, and a newer release upgrades over a clone build (Jaime). Versions compare as SemVer, with Go's pseudo-versions for clone builds (claude; design.md §1)
- No edit detection: upgrading replaces YASS's files, and since they're committed, the replacement shows up in `git diff` for review. Hashes, a prompt and `--force` were considered and dropped as more than the problem needs (Jaime). `yass init` lists what it writes and labels files outside version control (`--global` playbooks); unstamped files from v0.1 and v0.2 count as older and get upgraded (claude)
- PRD and design approved 2026-10-04 (Jaime)
- Planned in four pieces: `yass init` writes everything, then upgrades by version, then `install.sh` becomes a binary installer with the docs rewritten, and the `yass status` line (claude)
- `install.sh` runs `yass init` only when given a path (`.` for the repo you're in); without one it installs just the binary (claude, while planning) - superseded below
- Version comparison uses `golang.org/x/mod/semver` (design §4), and building from source needs Go 1.24, so clone builds carry a version (design §5) (claude, while planning; for review)
- `install.sh` only installs the binary; the documented one-liner runs `yass init` itself (`… | bash -s -- --bin-dir ~/.local/bin && ~/.local/bin/yass init`), so `install.sh` stays simple (Jaime). It calls the binary by its full path so it works before PATH is fixed, and `install.sh` given an old-style repo path or setup option installs nothing and prints the two commands instead, so a script expecting repo setup fails loudly rather than half-working (claude)
- An explicit `yass upgrade`: it finds every YASS file from the repo root down (and the user folder's playbooks) and upgrades each by version; `yass init` only sets up, keeping what's there. Upgrade docs live in the README and `docs/install.md`: reinstall the binary, then `yass upgrade` in each repo; not on the landing page (Jaime). Design §6 (claude)
- Agent files per folder: `yass init <folder> --agents` puts the `AGENTS.md` section and the playbooks in that folder for monorepo parts with their own agent setup; the hook stays at the repo root; `yass init <folder>` alone is planning only (Jaime, option 1 of three)
- Revised plan: a new piece 2 for agent files per folder, piece 3 becomes `yass-upgrade` (renamed from `init-upgrades-by-version`), and `install.sh` and status follow; AC23 now covers `yass upgrade` outside git, since `yass init` outside git is supported (claude)

## Log
<!-- Progress. Change-level notes; each piece keeps its own Log.
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->

### 2026-10-05 (claude)
- Did: all five pieces merged through the stacked PRs donjaime/yass#1–#8 (squash-merged bottom up with `gh stack`); `v0.3.0-rc.1` tagged on 5a67c7d and published as a pre-release; the release-candidate checks in the plan's Validation passed (see the `install-sh-installs-the-binary` piece's Log); this repo's own YASS files upgraded to `0.3.0-rc.1` with `yass upgrade`.
- Next: done. Archive it. The final `v0.3.0` release (dating the changelog entry) is a release step, not part of this change.


## yass/archive/2026/10/2026-10-04-the-binary-carries-the-playbooks/design.md

# The binary carries the playbooks: design
<!-- Intent: the calls that are hard to undo. Edit only in commits without code. -->

## Context
`yass upgrade` decides, per file, whether to upgrade, leave alone or refuse, so every file YASS installs has to say which version wrote it. Those stamps get committed into every repo that uses YASS, and future versions have to read them, so their format is public surface that's hard to change later. Two calls: how versions compare, and where the stamp goes. A third, noticing hand edits, was considered and dropped.

## 1. Versions and how they compare
**Options.**
- (a) **Semantic Versioning 2.0**, as the release tags already are (`v0.2.0`), with Go's pseudo-versions for builds from a clone (`v0.2.1-0.20261005022542-577e076bfe44`, plus `+dirty` for uncommitted changes), which Go stamps into the binary on its own.
- (b) Calendar versions (`2026.10.4`).
- (c) Commit SHAs only.

**Decision: (a)** (proposed; precedence rule from Jaime, 2026-10-04). It's what the tags, `go install` and `yass --version` already produce, so there's nothing new to learn. Precedence is SemVer's: a pre-release, which a pseudo-version is, sorts before its release, and build metadata (`+dirty`) is ignored when comparing. One YASS rule on top: when versions compare equal and the binary is `+dirty`, `yass upgrade` rewrites anyway, since the content may differ. In practice:
- A clone build after `v0.2.0` (`v0.2.1-0.…`) upgrades a repo stamped `v0.2.0`.
- A release `v0.2.1` upgrades a repo stamped with that clone build.
- A `+dirty` build of the same version rewrites; a clean one of the same version says "already up to date".
- A binary with no version at all (`dev`, built outside a git checkout) refuses to write, in `yass init` and `yass upgrade`, and says how to get a versioned build.

## 2. Where the stamp goes
**Options.**
- (a) **In each file,** using each format's own place for metadata:
  - Playbooks: the [Agent Skills](https://agentskills.io) frontmatter's `metadata` map, the spec's extension point for exactly this, so harnesses that read skills ignore it:
    ```yaml
    metadata:
      yass-version: "0.3.0"
    ```
  - The `AGENTS.md` section: attributes in its begin marker, as managed-block tools do (`<!-- yass:begin version=0.3.0 (managed by `yass init`; edit outside these markers) -->`).
  - The hook script: a header line (`# yass-version: 0.3.0`).
- (b) **A lockfile,** `tools/yass/yass.lock`, listing each installed file's path and version, as `go.sum` and `package-lock.json` do. The files stay as they are, and there's one place to look. But playbooks installed with `--global` live outside the repo and need a lockfile of their own, and a lockfile can drift from the files it describes.

**Decision: (a)** (Jaime, 2026-10-04: versions in the playbooks' frontmatter and the `AGENTS.md` section; claude, the hook header). Each file answers for itself wherever it lives, including the user folder, and nothing can drift apart. The plan checks the Agent Skills spec's `metadata` field (string keys and values) before building on it.

## 3. Noticing hand edits
**Options.**
- (a) A content hash stamped beside each version, checked on the next run, with a frozen table of v0.1 and v0.2 hashes for unstamped files; at a terminal, ask before overwriting an edited file.
- (b) **Don't detect edits.** Upgrading replaces YASS's files, as `install.sh --upgrade` does today.

**Decision: (b)** (Jaime, 2026-10-04). The playbooks, the hook and the `AGENTS.md` section are committed with the repo, so an upgrade that replaces an edited file shows up in `git diff` and review before it's committed; git already does the auditing. (a) added hashes, a frozen table, a prompt and its no-terminal fallback for little gain. Two cheap things stay (claude): `yass init` and `yass upgrade` list every file they write, and labels any outside version control (playbooks in the user folder, with `--global`), the one place an edit could vanish unseen. A file with no stamp, as v0.1 and v0.2 wrote them, counts as older than any stamped version, so `yass upgrade` upgrades it.

## 4. Comparing versions in code
**Options.**
- (a) **`golang.org/x/mod/semver`,** the Go team's SemVer package, which the `go` command uses for module versions, pseudo-versions included. A second dependency beside `yaml.v3`.
- (b) A small comparison written in YASS, with its own tests.

**Decision: (a)** (proposed, claude, while planning). It's the reference implementation of the exact rules design §1 picks, and it's maintained by the Go team. A home-grown comparison would have to get pre-release ordering and pseudo-versions right for no gain.

## 5. Versioned builds from a clone
Go stamps a pseudo-version into a binary built from a clone (`go build` in a git checkout) since Go 1.24; older Go builds report `(devel)`, which YASS reads as no version (design §1: refuses to write).

**Options.**
- (a) **Require Go 1.24 or later** to build from source (`go 1.24` in `go.mod`), and update the docs that say 1.22.
- (b) Keep Go 1.22, and have the docs' build command pass the version itself (`-ldflags "-X main.version=$(git describe --tags --dirty)"`).
- (c) Keep Go 1.22, and let unversioned builds write anyway.

**Decision: (a)** (proposed, claude, while planning). The build command stays one plain `go build`, and design §1's rules hold for every build. Go 1.24 shipped in February 2025, and the release binaries don't need Go at all. (c) would break the never-downgrade guarantee.

## 6. Setting up and upgrading: one command or two
**Options.**
- (a) `yass init` is idempotent and also upgrades whatever is installed.
- (b) **`yass init` sets up; `yass upgrade` upgrades.** `init` writes what's missing and keeps what's there. `upgrade` finds every YASS file from the repo root down, plus the user folder's playbooks, and brings each to the binary's version.

**Decision: (b)** (Jaime, 2026-10-05, replacing (a), chosen while shaping). Once agent files can live in several folders of a monorepo, upgrading is its own job: find every copy and decide by version. Setting up a folder isn't. An explicit verb also gives the upgrade docs one command to point at.

**Finding the files.** `yass upgrade` lists the repo's tracked and untracked files with `git ls-files` (respecting `.gitignore`) and picks out `*/.agents/skills/yass-*/SKILL.md`, `*/.claude/skills/yass-*/SKILL.md`, every `AGENTS.md` with a `yass:begin` marker, and `tools/yass/githooks/pre-commit`. It adds the user folder's `yass-*` playbooks (`$YASS_SKILLS_DIR` or `~/.agents/skills`, and `~/.claude/skills`). A `CLAUDE.md` import isn't versioned: it's one line, and `yass init --claude` keeps it in place.

## Consequences and rollback
- Every repo gets a few lines of metadata in each playbook and the `AGENTS.md` marker, and they show up in diffs on upgrade, which is the point.
- Reading stays lenient: a file without a stamp is older than any stamped one, never an error, and unknown `metadata` keys or marker attributes are ignored. So a later format change can add fields without breaking older repos.
- Hand edits to YASS's files are replaced on upgrade. That's documented, and visible in the diff.
- Changing the stamp format later means teaching `yass upgrade` both formats for a while.


## yass/archive/2026/10/2026-10-04-the-binary-carries-the-playbooks/plan.md

# The binary carries the playbooks: plan
<!-- Intent: how, and how we'll know it's done. With code, only mark boxes here ([/] in progress, [x] done);
     anything else is a plan revision in its own commit. -->

## Approach
<!-- The shape of the solution. Hard-to-undo calls go in design.md. -->
**The binary carries the kit.** `kit/` (the playbooks and the hook script) becomes a Go package with an `embed.go` (`//go:embed all:.agents tools`), so every `yass` binary, from a release, a clone or `go install`, carries the files for its own version. The files in `kit/` stay unstamped; the version is added as they're written.

**`yass init` writes everything.** [`cmdInit`](../../../internal/yass/commands.go) already writes the yass folder, `yass.yaml` and the `AGENTS.md` section. It takes over the rest of what [`install.sh`](../../../install.sh) does: the playbooks (`.agents/skills/`, or the user folder with `--global`), Claude Code's copies and the `CLAUDE.md` import (`--claude`), the hook script, and `core.hooksPath` (`--hooks`), with the same paths, so nothing moves in existing repos. Run for a folder (`yass init <folder>`), it sets up only that folder's planning, unless `--agents` asks for the `AGENTS.md` section and the playbooks (and Claude's copies with `--claude`) in that folder too; the hook and `core.hooksPath` always belong to the repo root. On a repo or folder that's already set up, it writes what's missing, keeps the rest, and says `yass upgrade` would update files older than the binary. Every file it writes is stamped as [design.md](design.md) §2 describes: `metadata.yass-version` in each playbook's frontmatter, `version=` in the `AGENTS.md` begin marker, and `# yass-version:` in the hook header.

**`yass upgrade` upgrades.** A new command, run anywhere in a repo. It finds every YASS file from the repo root down, as [design.md](design.md) §6 describes (`git ls-files` for playbooks in any `.agents/skills/` or `.claude/skills/`, every `AGENTS.md` with the YASS marker, and the hook), plus the user folder's playbooks. It reads each one's stamp and compares it with its own version (design §1). Older or unstamped files get upgraded, files at the same version are left alone (unless the binary is `+dirty`), and if any file is newer, nothing is written and it says to upgrade the binary. It prints whether it upgraded (from which version) or found everything up to date, and lists every file it wrote, labeling any outside version control. It takes no options: it upgrades what's there, where it is.

**`install.sh` becomes a binary installer, and only that.** It downloads and verifies a release (when piped), or uses the binary next to it (a release folder or a clone's `bin/`), copies it to `--bin-dir`, says where it went and whether that's on PATH, and says `yass init` sets up a repo. The documented one-liner chains the two explicitly: `curl -fsSL https://raw.githubusercontent.com/donjaime/yass/main/install.sh | bash -s -- --bin-dir ~/.local/bin && ~/.local/bin/yass init`. It calls the binary by its full path, so it works before PATH is fixed. Given a repo path or a setup option (`--claude`, `--global`, `--hooks`, `--path`, `--upgrade`) as earlier versions took them, `install.sh` installs nothing, exits non-zero and prints the commands to run instead: the binary install, then `yass init …` (or `yass upgrade`, for `--upgrade`). Release archives drop the kit files, since the binary carries them; `install.sh` stays in them as the way to put the binary on PATH.

**`yass status`** (M2) reads the same stamps and adds one line when the binary and the repo are out of step.

**Docs.** `yass init`'s options and `yass upgrade` go in the README's CLI reference with the code that adds them, along with a short "Upgrading" section in the README and `docs/install.md` once `yass upgrade` exists: reinstall the binary, then run `yass upgrade` in each repo and commit what it changed. The install model in the README, `docs/install.md` and the landing page is rewritten in the piece that changes `install.sh`, so the docs never describe a flow that doesn't exist. The landing page doesn't cover upgrading.

## Acceptance
<!-- Per milestone, one observable behavior each:
### M1
- [ ] AC1 (R1) Given …, when …, then … — verify: <test, flow, or manual steps> -->
### M1
- [x] AC1 (R1) Given a fresh git repo and a `yass` binary with no release files beside it, when you run `yass init`, then it writes the five playbooks under `.agents/skills/`, an executable `tools/yass/githooks/pre-commit`, the `AGENTS.md` section and the yass folder — verify: e2e (binary copied alone into an empty folder on PATH)
- [x] AC2 (R1) Given that repo, then each file matches its source in `kit/` (or `templates/agents.md`) except for its version stamp — verify: e2e (diff with the stamp lines removed)
- [x] AC3 (R1) Given a binary built by `go install` or a release, then it carries all five playbooks and the hook — verify: go test (the embedded files list)
- [x] AC4 (R2) Given `yass init --claude`, then it also writes the playbooks under `.claude/skills/` and puts `@AGENTS.md` at the top of `CLAUDE.md` once, keeping what was there — verify: e2e (with and without an existing `CLAUDE.md`, run twice)
- [x] AC5 (R2) Given `yass init --global`, then the playbooks go to `~/.agents/skills/` (or `$YASS_SKILLS_DIR`), and to `~/.claude/skills/` with `--claude`, and none to the repo — verify: e2e (with a temporary `HOME`)
- [x] AC6 (R2) Given `yass init --hooks`, then `core.hooksPath` is `tools/yass/githooks` for this clone — verify: e2e
- [x] AC7 (R2) Given `--path`, `--private` and `--no-agents`, then they behave as they do today — verify: e2e (the existing sections, run through `yass init`)
- [x] AC8 (R7) Given a fresh `yass init` by a binary at version V, then every playbook's frontmatter has `metadata.yass-version: "V"` and still has its `name` and `description`, the `AGENTS.md` begin marker has `version=V`, and the hook has `# yass-version: V` — verify: go test (the frontmatter parses as YAML); e2e (all three stamps)
- [x] AC9 (R3) Given a repo whose YASS files are stamped with an older version, when you run `yass upgrade`, then every one is rewritten at the binary's version, and it prints that it upgraded, from which version to which — verify: e2e
- [x] AC10 (R3) Given a repo set up with `--claude`, or with playbooks only in the user folder, when you run `yass upgrade`, then those copies are upgraded too, where they are, and no new ones are added — verify: e2e
- [x] AC11 (R3) Given changes, a `yass.yaml`, text outside the `AGENTS.md` markers and other content in `CLAUDE.md`, when `yass upgrade` runs, then none of them changes — verify: e2e (checksums before and after)
- [x] AC12 (R10) Given a repo already at the binary's version, when you run `yass upgrade`, then no file changes, `git status` is clean, and it says the repo is already up to date at that version — verify: e2e
- [x] AC13 (R11) Given a repo with any YASS file stamped newer than the binary, when you run `yass upgrade`, then no file changes, it names the newer version and says to upgrade the binary, and it exits non-zero — verify: e2e
- [x] AC14 (R12) Given versions to compare, then a clone build after a release (`v0.3.1-0.…`) is newer than that release, a later release is newer than the clone build, `+dirty` is ignored in comparing, and an equal version written by a `+dirty` binary is rewritten — verify: go test (a table of version pairs and the expected action)
- [x] AC15 (R12) Given a binary with no version (`dev`), when you run `yass init` or `yass upgrade`, then it writes nothing, says how to get a versioned build, and exits non-zero — verify: go test (the decision for an empty version); e2e (a binary built with `-buildvcs=false`)
- [x] AC16 (R13) Given `yass init` or `yass upgrade` writes files, then it prints each one, and marks those outside version control (user-folder playbooks) as such — verify: e2e
- [x] AC17 (R13) Given a hand-edited playbook in a repo at an older version, when `yass upgrade` runs, then the edit is replaced, and `git diff` shows it — verify: e2e
- [x] AC18 (R14) Given a repo set up by v0.2.0's `install.sh` (no stamps), when you run `yass upgrade`, then it upgrades every YASS file — verify: e2e (a fixture installed from v0.2.0's kit)
- [x] AC19 (R4) Given that upgraded repo and a fresh `yass init` of an empty repo with the same options and binary, then their YASS files are identical — verify: e2e (`diff -r` of the YASS files, with `--claude` and `--hooks` too)
- [x] AC20 (R5) Given `install.sh --bin-dir DIR`, run inside a repo or outside one, then it copies the binary to DIR, says where it put it and whether DIR is on PATH, says `yass init` sets up a repo, and writes nothing in the current folder — verify: e2e (both places; `git status` clean)
- [x] AC21 (R5) Given `install.sh` with a repo path or any of `--claude`, `--global`, `--hooks`, `--path`, `--upgrade`, then it installs nothing, exits non-zero, and prints the commands to run instead: `install.sh --bin-dir …`, then `yass init …` with the options carried over (or `yass upgrade`, for `--upgrade`) — verify: e2e (each old form)
- [x] AC22 (R5) Given the documented one-liner's two commands, run in a fresh repo with DIR not on PATH, then the binary lands in DIR and the repo is set up — verify: e2e (`install.sh` from the clone, then `DIR/yass init`, with PATH lacking DIR)
- [x] AC23 (R3) Given `yass upgrade` run outside a git repo, then it changes nothing and says it works inside a git repo — verify: e2e
- [x] AC24 (R5) Given the piped one-liner, then `install.sh` still downloads the latest release and checks it against `checksums.txt` before installing the binary, and the chained `yass init` sets up the repo — verify: manual: run the documented one-liner against a release candidate in a fresh repo
- [x] AC25 (R1, R5) Given a release archive, then it has the binary, `install.sh` and the docs, and no `kit/` files, and its `install.sh` installs the binary — verify: CI (GoReleaser snapshot build); e2e (an unpacked-release folder without `kit/`)
- [x] AC26 (R6) Given a repo that already uses YASS, when a teammate runs `install.sh` (or `go install`) without `yass init`, then the binary is installed and no file in the repo changes — verify: e2e (`git status` clean); manual: docs review
- [x] AC27 (R9) Given the README, `docs/install.md` and the landing page, then they describe the model: get the binary, run `yass init` in each repo, and joining needs only the binary; the README's CLI reference lists `yass init`'s options and `yass upgrade` — verify: manual: review; `tests/site.sh` (the page's commands match the docs)

- [x] AC31 (R15) Given a monorepo set up at the root and `yass init apps/web --agents`, then `apps/web/` gets its yass folder, an `AGENTS.md` with the YASS section and the playbooks under `apps/web/.agents/skills/`, and no second hook; with `--claude`, Claude's copies and the `CLAUDE.md` import go in `apps/web/` too — verify: e2e
- [x] AC32 (R15) Given `yass init apps/web` without `--agents`, then it writes only `apps/web/yass/` (or `apps/web/yass.yaml` with `--path`), and no `AGENTS.md`, here or at the root — verify: e2e
- [x] AC33 (R16) Given a repo already set up, with files stamped older than the binary, when you run `yass init`, then it writes only what's missing, keeps the rest, and says `yass upgrade` would update them — verify: e2e
- [x] AC34 (R3) Given a monorepo with agent files at the root and in `apps/web/` and `services/api/`, all stamped older, when you run `yass upgrade` from `services/api/`, then every copy in the repo is upgraded where it is — verify: e2e
- [x] AC35 (R9) Given the README and `docs/install.md`, then each says how to upgrade: reinstall the binary, then run `yass upgrade` in each repo and commit what changed; the landing page doesn't mention upgrading — verify: manual: review

### M2
- [x] AC28 (R8) Given a repo whose YASS files are stamped newer than the binary, when you run `yass status`, then it warns which version wrote them and says to upgrade the binary — verify: e2e
- [x] AC29 (R8) Given a repo whose YASS files are stamped older than the binary, when you run `yass status`, then it notes that `yass upgrade` would upgrade the repo — verify: e2e
- [x] AC30 (R8) Given a repo at the binary's version, or with no YASS files installed, then `yass status` says nothing about versions — verify: e2e

## Pieces
<!-- PR-sized pieces, each its own folder in here: `yass new "<title>" --in <this change>`.
     `yass status` lists them; say here what order they go in and why. -->
1. `init-writes-the-whole-repo` (M1, done): the kit embedded in the binary, the stamps, and `yass init` writing every file with the new options, keeping files that already exist.
2. `agent-files-per-folder` (M1): `yass init <folder> --agents`, the agent files' location separate from the repo root's hook, `yass init <folder>` without `--agents` leaving `AGENTS.md` alone, and `yass init` on a set-up repo pointing at `yass upgrade`. It goes before `yass upgrade` because that has to find the nested copies this creates.
3. `yass-upgrade` (M1): the new command, with discovery from the repo root, reading stamps and comparing versions, upgrading or saying up to date or refusing, unstamped files, the file listing, the Go 1.24 floor (design §5), and the upgrade docs in the README and `docs/install.md`. Blocked on piece 2.
4. `install-sh-installs-the-binary` (M1): `install.sh` as a binary installer only, its refusal of old-style arguments (pointing `--upgrade` at `yass upgrade`), release archives without the kit, the one-liner that chains `yass init`, and the install docs and landing page rewritten to match. Blocked on piece 3.
5. `status-knows-the-versions` (M2): the `yass status` line. Blocked on piece 3 for the stamp reading; it can go before or after piece 4.

Once piece 4 lands, the monorepo change's `yass-trailers` piece (which waits on this change) ships its hooks through `yass init` and `yass upgrade`.

## Validation
<!-- How the whole thing is verified before it's called done: suites, platforms, manual passes. -->
- CI on each piece: `go test`, `tests/e2e.sh`, `tests/examples.sh` and `tests/site.sh`, on Ubuntu and macOS, plus the GoReleaser snapshot build.
- Dogfood: after piece 3, `yass upgrade` in this repo upgrades its own playbooks, Claude copies and hook, and the examples' monorepo checks nested copies. The diff should be stamps only, and `install.sh` is no longer needed here.
- A release candidate (`v0.3.0-rc.1`) before release: the piped one-liner in a fresh repo (AC24), an unpacked archive on macOS and Linux, and upgrading a repo set up by v0.2.0.
- The release notes say how to upgrade: get the new binary, then run `yass upgrade` in each repo.


## yass/archive/2026/10/2026-10-04-the-binary-carries-the-playbooks/prd.md

# The binary carries the playbooks: PRD
<!-- Intent: what should change or exist, and why. Not a description of the current system;
     the code and its docs are that. Edit this file only in commits without code. -->

## Why
<!-- The problem, with evidence. -->
Using YASS takes two things: the `yass` binary on your PATH, once per machine, and a few files in each repo, set up once and committed. Today one script, [`install.sh`](../../../install.sh), does both, and the playbooks and the hook only reach a repo through a downloaded release folder. That causes four problems:
- **The two jobs blur.** People can't tell which part of the command does what. The binary half fails quietly when its folder isn't on PATH, which happened on a stock Mac and needed [a docs fix](../../archive/2026-10-04-install-say-where-to-run-it-and-how-yass/change.md) to explain.
- **Joining a repo has no real path.** A teammate who clones a repo that already uses YASS needs only the binary. The docs fall back to copying it out of a release archive by hand ([install.md](../../../docs/install.md#joining-a-repo-that-already-uses-yass)).
- **Upgrading every repo needs the release every time.** `install.sh --upgrade` has to run from a release folder (or the piped download) for each repo, even though the binary you just upgraded could do it.
- **Nothing knows which version wrote a repo's files.** A teammate with an older binary and a repo with newer playbooks get no warning that they're out of step.

The binary already does part of the repo setup: `yass init` writes the yass folder, the `AGENTS.md` section and `yass.yaml`, it embeds the change templates, and it knows its version.

## Users and outcomes
<!-- Who it's for, and measurable targets ("p50 time to log < 5s"). -->
- **Someone adopting YASS** in a repo. Outcome: two steps, each saying what it did: get the binary, then `yass init`. The one-line install runs both, visibly.
- **A teammate joining a repo that already uses YASS.** Outcome: they get the binary and they're done; nothing in the repo needs to change.
- **Someone who uses YASS in several repos.** Outcome: setting up another repo needs no download (`yass init`), and neither does upgrading one: reinstall the binary once, then `yass upgrade` in each repo.
- **A monorepo team** whose parts keep their own `AGENTS.md` and skills folders. Outcome: YASS's agent files go in the folder they choose, the hook stays at the repo root, and one `yass upgrade` finds and upgrades every copy.
- **Anyone whose binary and repo are out of step.** Outcome: `yass status` says so, and says which to upgrade.

Targets:
- Setting up a second repo, or upgrading any repo after the binary: 0 downloads, 1 command.
- Joining a repo: 1 step (get the binary), no `install.sh`.
- Every repo set up by today's `install.sh` upgrades with `yass upgrade`, with no manual cleanup.

## Requirements
<!-- One line each: "- **R1** [M1] A user can …". IDs are never reused. -->
- **R1** [M1] A user with `yass` on their PATH sets up a repo with `yass init`, run for the repo itself: the playbooks, the hook script, the `AGENTS.md` section and the yass folder (or `yass.yaml`), the same files `install.sh` writes today, from the binary alone.
- **R2** [M1] `yass init` takes the setup options `install.sh` has today: `--claude` (Claude Code's copies and the `CLAUDE.md` import), `--global` (playbooks in the user folder), `--hooks` (turn on the hook for this clone) and `--path`, alongside the options it already has.
- **R3** [M1] `yass upgrade`, run anywhere in a repo, finds every YASS file the repo has, from the repo root down (playbooks in any folder's `.agents/skills/` or `.claude/skills/`, the YASS section of any `AGENTS.md`, the hook script), plus playbooks in the user folder, and upgrades each to the binary's version where it is. It says it upgraded and from which version. It never touches changes, `yass.yaml` or anything outside the `AGENTS.md` markers.
- **R4** [M1] A repo set up by an earlier `install.sh` upgrades with `yass upgrade` and ends up exactly as a fresh `yass init` at the same version would leave it.
- **R5** [M1] `install.sh` only installs the binary: it says where it put it, whether that folder is on PATH, and that `yass init` sets up a repo. The documented one-line install runs it and then `yass init` explicitly, so it still sets up a repo in one go. Given a repo path or a setup option the way earlier versions took them, it installs nothing and prints the commands to use instead (`yass init …`, or `yass upgrade` for `--upgrade`).
- **R6** [M1] A teammate joining a repo that already uses YASS installs only the binary, with `install.sh` or `go install`, and doesn't run `yass init`.
- **R7** [M1] Every file YASS installs records the version that wrote it, committed with it: the playbooks in their frontmatter, the `AGENTS.md` section in its marker, the hook script in a header. `yass upgrade` decides what to do by comparing those versions with its own. `yass --version` reports the binary's.
- **R8** [M2] `yass status` warns when the binary is older than the version that wrote a repo's YASS files (upgrade your binary), and notes when it's newer (`yass upgrade` would upgrade the repo).
- **R9** [M1] The README, `docs/install.md` and the landing page describe the new model: get the binary, run `yass init` in each repo, and joining needs only the binary. The README and `docs/install.md` say how to upgrade: reinstall the binary, then run `yass upgrade` in each repo and commit what it changed. The landing page doesn't cover upgrading.
- **R10** [M1] When a repo's YASS files are already at the binary's version, `yass upgrade` changes nothing and says the repo is already up to date.
- **R11** [M1] `yass upgrade` never downgrades. When any of a repo's YASS files is newer than the binary, it changes nothing, says which version wrote it, and says to upgrade the binary.
- **R12** [M1] A build from a clone stamps its own version (the release it follows plus a commit, and `+dirty` with uncommitted changes) and upgrades a repo stamped with an older release; a newer release then upgrades over it; a `+dirty` build rewrites files stamped with its own version. A binary with no version refuses to write.
- **R13** [M1] Upgrading replaces YASS's files, hand edits included. `yass init` and `yass upgrade` list every file they write, so the change shows up for review in `git diff`, and label any written outside version control (playbooks in the user folder, with `--global`).
- **R14** [M1] A file without a version stamp, as written by v0.1 and v0.2, counts as older than any stamped version, so `yass upgrade` upgrades it.
- **R15** [M1] `yass init <folder> --agents` puts the `AGENTS.md` section and the playbooks in that folder (and Claude Code's copies there with `--claude`), for a part of a monorepo that keeps its own agent files. The hook script and `core.hooksPath` stay at the repo root. Without `--agents`, `yass init <folder>` sets up only the folder's planning (its yass folder, or `yass.yaml` with `--path`) and touches no `AGENTS.md`.
- **R16** [M1] `yass init` on a repo or folder that's already set up writes only what's missing and keeps what's there. When the files there are older than the binary, it says `yass upgrade` would update them.

## Non-goals
<!-- What this change will not do. Agents treat these as walls. -->
- No Homebrew tap, other package manager or `yass self-update`. Getting the binary stays a release download, the one-liner or `go install`.
- No editing of shell startup files. Installers say what to add to PATH; the user adds it.
- No change to the playbooks' content, the hook's checks or the change file formats; only how they get into a repo.
- No detecting or preserving hand edits to YASS's files. They're committed with the repo, so an upgrade that replaces one shows up in review; `install.sh --upgrade` behaves the same way today.
- No change to where files land in a repo. Same paths as today, so existing repos upgrade in place.
- No new installer for Windows. `yass init` and `yass upgrade` work there natively; getting the binary stays a release download or `go install`.
- No changes to the monorepo change's trailer hooks themselves. Its `yass-trailers` piece waits for this change and ships its hook scripts through `yass init` and `yass upgrade`.

## Milestones
| Milestone | When it ships, a user can … |
|---|---|
| M1 | set up a repo, or a monorepo folder with its own agent files, with `yass init`, and upgrade every copy with one `yass upgrade` (or see it's already up to date), with no download after the binary; join a repo with just the binary; and read docs that describe it that way, including how to upgrade. A one-line install still sets up a repo, by running `yass init` itself. |
| M2 | see from `yass status` when their binary and a repo's YASS files are out of step, and which to upgrade |

## Open questions
- None. The version format and comparison are in [design.md](design.md).


## yass/archive/2026/10/2026-10-05-same-day-changes-list-in-creation-order/change.md

---
platforms: [all]
source: 
follows: 
blocked:
archived: 2026-10-05T19:13:17Z
---
# Same-day changes list in creation order

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
Changes made on the same day list in the order they were created, so "what's next?" has a tie-break when `queue.md` doesn't settle it. `yass new` stamps `created:` (UTC, to the second) in a change's frontmatter, and listings order by it. Folder names don't change, and nothing is migrated: changes without the stamp (the archive, older or hand-made changes) fall back to their date prefix.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Given `yass new` (or `yass new --in`), then the new change.md has `created: <UTC RFC 3339 time>` in its frontmatter and the folder is still `<YYYY-MM-DD>-<slug>` — verify: e2e
- [x] Given two changes made the same day whose slugs sort the other way, and no `queue.md` entry for either, when you run `yass status`, then they list in creation order — verify: go test; e2e
- [x] Given pieces of a large change, or archived changes with stamps, then `yass status` lists them in creation order too — verify: go test
- [x] Given a change without `created:`, then it sorts by its date prefix, before stamped changes from the same day, and still lists and resolves as before — verify: go test; e2e
- [x] Given a `created:` that isn't an RFC 3339 time, then `yass status` warns and sorts that change as if it had no stamp — verify: e2e

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] Templates and `yass new` stamp `created:`
- [x] Load orders changes and pieces by (created, name); warn on a bad stamp
- [x] README documents `created:`
- [x] Go tests and e2e

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- Order by a `created:` stamp in frontmatter, not by renaming folders - a counter (SEP style) collides across parallel branches, worktrees and plans repos, and renumbering breaks `follows:`, `blocked:`, `queue.md` and Log references; a time in the name costs too many characters (Jaime, claude)
- No migration - the archive is append-only, and unstamped changes sort well enough by date; queue.md already says what order matters (Jaime, claude)
- Order once, in load, for changes and pieces alike - every listing (status, --archived, pieces) agrees without each sorting on its own (claude)
- The yass folder READMEs and CHANGELOG stay as they are - "oldest first" was the promise and is now true; recent changes leave the CHANGELOG to release time (claude)
- UTC to the second - teammates in different zones still order correctly, and agents can make several changes in a minute (claude)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-05 (claude)
- Did: `yass new` stamps `created:` (UTC RFC 3339) via the change templates; `load` orders changes and pieces by `created()` (the stamp, else the folder's date), and `check` warns on a stamp that isn't a time. TestByCreated; e2e §29; README frontmatter. Checks: go test, go vet, gofmt, e2e 386/386.
- Next: archive.


## yass/archive/2026/10/2026-10-05-tighten-ci-fewer-duplicate-runs-pinned/change.md

---
platforms: [all]
source: Jaime, 2026-10-05: sanity check of CI while reviewing the binary-carries-the-playbooks stack; fix after the stack lands
follows: 
blocked:
archived: 2026-10-05T19:11:56Z
---
# Tighten CI: fewer duplicate runs, pinned release actions, required checks

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
CI stays as safe for outside contributors as it is today (fork PRs get a read-only token and no secrets, first-time contributors need approval, and only `main` and `v*` tags reach anything that writes), and gets cheaper and harder to tamper with. Each commit on a PR is tested once, superseded runs stop when a branch is pushed again, the release cross-build runs only where it's worth its cost, the actions that can write releases are pinned to commit hashes, and `main` requires the `ci-ok` gate before a PR merges.


## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Given a push to a PR's branch, then `ci` runs once for that commit (for the PR), not also for the branch push; pushes to `main` and merge-queue runs still run it — verify: manual: push to a PR branch and list the runs for its commit (`gh run list --commit <sha>`)
- [x] Given a PR branch pushed again while its `ci` run is in progress, then the older run is cancelled; runs on `main` are never cancelled — verify: manual: push twice in a row to a PR branch and check the first run's status
- [x] Given a PR that touches none of `.goreleaser.yaml`, `go.mod`, `go.sum`, `cmd/`, `kit/` or `.github/workflows/`, then the GoReleaser snapshot build is skipped and `ci-ok` still passes; on `main`, and on a PR that touches any of them, it runs — verify: manual: one PR of each kind
- [x] Given `release.yml`, then every third-party action is pinned to a full commit hash (with the version in a comment), and the GoReleaser version is pinned exactly — verify: manual: review; `grep -E 'uses: .*@' .github/workflows/release.yml` shows only 40-character hashes
- [x] Given `ci.yml` and `pages.yml`, then their actions are pinned the same way — verify: manual: review
- [x] Given a PR into `main` whose `ci-ok` hasn't passed, then GitHub won't merge it — verify: manual: a ruleset on `main` requiring `ci-ok`; try merging a PR while CI runs
- [x] Given a new PR branch that changes only plans (for example, an archive move), when it's pushed and its PR opens, then no run tests or builds anything: only the `changes` gate and `ci-ok` run — verify: manual: open a plans-only PR from a new branch and list the jobs of every run for its commit
- [x] Given the change, then no workflow uses `pull_request_target` or `workflow_run`, workflow tokens default to read, and fork PRs from first-time contributors still need approval — verify: manual: `gh api repos/donjaime/yass/actions/permissions/workflow` and `…/fork-pr-contributor-approval`; review

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `ci.yml`: `push` on `main` only; `concurrency` per PR with `cancel-in-progress` (not on `main`)
- [x] `ci.yml`: the snapshot build only on `main`, the merge queue, or PRs touching release inputs (a paths check in the `changes` job); `ci-ok` treats a skipped build as passing
- [x] Pin actions to commit hashes in `release.yml`, `ci.yml` and `pages.yml`; GoReleaser version exact
- [x] Repo settings (Jaime): a ruleset on `main` requiring `ci-ok`; optionally a `v*` tag ruleset, and approval for all outside contributors rather than first-timers only
- [x] Note the required check in `docs/monorepo.md`'s CI recipe if it changes
- [x] Look at the gate's own cost (a runner, `setup-go`, building `yass` for `yass paths --only`, about 10 s plus queueing) and whether a cache or a prebuilt binary is worth it; it's also this repo's dogfood of the `docs/monorepo.md` recipe, so keep that working

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- A separate small change, after the binary-carries-the-playbooks stack lands - found during a CI sanity check while reviewing that stack; nothing in it is a security hole today, so it doesn't block the stack (Jaime)
- Findings it starts from (claude, 2026-10-05): no `pull_request_target` or `workflow_run`; `ci.yml` has `contents: read` and the repo default is read; fork-PR approval is `first_time_contributors`; `release.yml` runs only on `v*` tags and `pages.yml` only on `main`, whose environment allows only `main`; the one event value used in a script goes through `env:`. Costs and gaps: `ci` runs on both `push` (every branch) and `pull_request`, so each PR commit runs twice; no `concurrency`; the six-target snapshot build runs on every PR; actions are pinned by movable tags, which matters most in `release.yml` (`contents: write`, `id-token: write`, `attestations: write`); `main` has no branch protection or ruleset, so `ci-ok` isn't required
- Seen on donjaime/yass#10, an archive-only PR (Jaime, 2026-10-05): its `pull_request` run did the right thing (gate about 10 s, tests and build skipped, `ci-ok` 3 s), but the `push` run for the new branch ran the full tests and the release build, because a branch's first push has no `before` commit to compare with, so the gate falls back to running everything. `push` on `main` only fixes it; the new criterion checks it (claude)
- The test job also cross-compiles `cmd/yass` for windows/amd64, linux/arm64 and darwin/arm64 on Ubuntu: with the snapshot cross-build now limited to release inputs, an ordinary code PR could otherwise break a platform unnoticed until main; the check takes seconds (claude)
- Actions are pinned to the commit their major tag pointed at on 2026-10-05, with the full version in a comment (checkout v4.4.0, setup-go v5.6.0, upload-artifact v4.6.2, goreleaser-action v6.4.0, attest-build-provenance v2.4.0, configure-pages v5.0.0, upload-pages-artifact v3.0.1, deploy-pages v4.0.5), and GoReleaser at v2.18.2; moving to a new version is now a deliberate edit (claude)
- `docs/monorepo.md`'s GitHub recipe gets the same `push`-on-the-default-branch and `concurrency` settings, so teams copying it don't get the duplicate runs; the required check is still `ci-ok` (claude)
- The gate's cost stays as it is: `setup-go` already caches modules and the build, so building `yass` is a few seconds, and the rest is a runner and a full-history checkout, which `--only` needs for its base; a prebuilt release binary would stop this repo's gate from exercising its own `yass paths` (claude)
- Settings, applied in the GitHub UI: a branch ruleset "main: require ci-ok" on the default branch, active, requiring `ci-ok` from GitHub Actions (not "up to date"), with repository admins able to bypass always so direct pushes to main keep working; it also restricts deleting main and blocks force pushes to it (Jaime, 2026-10-05)
- Fork-PR approval stays at first-time contributors: once someone's first run is approved, their later runs go ahead, and Jaime approves with care (Jaime)
- Deferred: the Actions setting "Require actions to be pinned to a full-length commit SHA" and a `v*` tag ruleset, for a later change (Jaime)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->

### 2026-10-05 (claude)
- Did: `ci.yml` runs on pushes to `main` only (plus PRs and the merge queue), with a per-PR `concurrency` group that cancels superseded PR runs and never main's; the snapshot cross-build runs only on main, the queue, or PRs touching `.goreleaser.yaml`, `go.mod`/`go.sum`, `cmd/`, `kit/` or `.github/workflows/` (checked against #15, #16, #7 and #6: false, false, true, true); a cross-compile check in the test job; every action in the three workflows pinned to a commit, GoReleaser exact; `docs/monorepo.md`'s recipe updated to match. YAML parses; the cross-compile loop builds locally; `tests/site.sh` passes.
- Next: Jaime reviews locally. The behavior criteria can only be checked on GitHub once this is pushed (one run per PR commit, cancelled superseded runs, the build skip, a plans-only PR). The `ci-ok` ruleset and the optional settings are Jaime's to approve.

### 2026-10-05 (claude)
- Did: checked the ruleset through the API (id 24523865: active, `~DEFAULT_BRANCH`, `ci-ok` required, admin bypass "always", deletion and non-fast-forward rules). Early evidence on #20: its branch gets only `pull_request` runs, and the rebase's push cancelled the older run. #19's branch still gets `push` runs, since it carries the old `ci.yml` until #20 merges.
- Next: Jaime reviews #19 and #20. To finish, check on GitHub: a PR whose `ci-ok` is still running shows as blocked from merging; after #20 merges, a new plans-only PR runs only the gate; a PR without release inputs skips `build`. Then archive.

### 2026-10-05 (claude)
- Did: #19 and #20 merged (602d44a, 6b4b94e). On #20 before merging: its branch got only `pull_request` runs, never `push` ones, and the rebase's push cancelled the run for the older commit (one run per PR commit; superseded runs cancelled). On main after the merge, `ci` ran everything, `build` included, and `pages` redeployed because `pages.yml` itself changed. Still true: no `pull_request_target` or `workflow_run` in any workflow, workflow tokens default to read, fork-PR approval is `first_time_contributors`.
- Next: this progress PR is plans-only, so it should run only `changes` and `ci-ok`, and stay blocked from merging until `ci-ok` passes; check both on it. The build-skip criterion needs a code PR that touches no release inputs (the next one will do). Then archive.

### 2026-10-05 (claude)
- Did: on #22, this plans-only progress PR from a new branch: one `ci` run, from the PR only (no branch-push run), in which `changes` and `ci-ok` passed and `test` and `build` were skipped; and GitHub reported it `BLOCKED` while `ci-ok` ran, `CLEAN` after, so the ruleset enforces it.
- Next: the last open criterion, a code PR that touches no release inputs skipping `build`, is checked on the next such PR; then archive.

### 2026-10-05 (claude)
- Did: the last criterion, seen on donjaime/yass#24 (a code PR touching `internal/yass/`, `tests/` and docs, no release inputs): one `pull_request` run in which the tests ran on Ubuntu and macOS, cross-compile check included, and `build` was skipped; #23, plans-only, ran only `changes` and `ci-ok`. Every box is done.
- Next: done. Archive it.


## yass/archive/2026/10/2026-10-05-yass-init-hooks-leaves-existing-hooks/change.md

---
platforms: [all]
source: Jaime, 2026-10-05: does the hook setup blow away existing git hooks? Make it safe and additive, or say how to wire it in
follows: 
blocked:
archived: 2026-10-05T18:25:40Z
---
# yass init --hooks leaves existing hooks alone

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
Turning on the YASS hook never switches off hooks someone already has. `yass init --hooks` sets `core.hooksPath` only when nothing else is in place; when a repo already has a hooks setup (a `core.hooksPath` of its own, global or local, as husky and others use, or working hooks in `.git/hooks/`), it leaves it alone and prints how to run `yass hook` from that setup. The README's hook section says the same, with snippets for plain git hooks, husky, lefthook and the pre-commit framework.


## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Given a repo with no `core.hooksPath` and no working hooks in `.git/hooks/` (only `*.sample` files, or none), when you run `yass init --hooks`, then it sets `core.hooksPath` to `tools/yass/githooks`, as today — verify: e2e
- [x] Given a repo whose local `core.hooksPath` points elsewhere (for example `.husky`), when you run `yass init --hooks`, then `core.hooksPath` is unchanged, and it says the repo already has a hooks setup, names it, and prints how to run `yass hook` from it — verify: e2e
- [x] Given a global `core.hooksPath` and no local one, when you run `yass init --hooks`, then it doesn't set a local `core.hooksPath` (which would override the global one for this repo), and says why and what to do — verify: e2e (with a temporary global config)
- [x] Given an executable hook in `.git/hooks/` (any name, not `*.sample`), when you run `yass init --hooks`, then `core.hooksPath` stays unset, and it names the hooks it found and prints the line to add to `.git/hooks/pre-commit` (`tools/yass/githooks/pre-commit "$@"`) — verify: e2e
- [x] Given `core.hooksPath` is already `tools/yass/githooks`, when you run `yass init --hooks` again, then nothing changes and it says the hook is already on — verify: e2e
- [x] Given any of these cases, then `yass init` still exits 0 and writes the hook script and everything else it would; only turning the hook on is skipped — verify: e2e
- [x] Given `yass upgrade`, then it never reads or changes `core.hooksPath` or `.git/hooks/` — verify: e2e (an upgrade with a custom `core.hooksPath` leaves it as it was)
- [x] Given the README's hook section and `docs/install.md`'s `--hooks` row, then they say `--hooks` only turns the hook on when there's no other hooks setup, and show how to call `yass hook` from a plain `.git/hooks/pre-commit`, husky, lefthook and the pre-commit framework — verify: manual: review

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `yass init --hooks`: detect an existing setup (local or global `core.hooksPath`, executable non-sample files in `.git/hooks/`, via `git rev-parse --git-path hooks` so worktrees work)
- [x] The message: what was found, and the snippet for it (a guess at husky from `.husky`, lefthook from `lefthook.yml`, the pre-commit framework from `.pre-commit-config.yaml`; otherwise the plain-git line)
- [x] e2e: each case above
- [x] README hook section ("Already have hooks?") and `docs/install.md`'s `--hooks` row

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- Check first and hand over a snippet, rather than make `tools/yass/githooks` a dispatcher that also runs the hooks that were there before - a dispatcher has to know what "before" was and how each hook manager works, and grows with every hook YASS adds; leaving the setup alone and saying how to wire in can't break anyone's hooks (claude, for Jaime's review)
- Today nothing is deleted, but setting `core.hooksPath` silently switches off hooks in `.git/hooks/`, overrides a global `core.hooksPath`, and breaks husky (which sets its own); the README warns about it, `yass init --hooks` doesn't (claude, 2026-10-05)
- The monorepo change's `yass-trailers` piece (two more hooks, `prepare-commit-msg` and `commit-msg`) should follow the same rule; worth a note in that piece when it's picked up (claude)
- The messages guess the hook manager from what's in the repo (`.husky/` or a `core.hooksPath` containing `.husky`, `lefthook.yml`, `.pre-commit-config.yaml`) and fall back to the plain-git line for whatever folder git runs hooks from; for a global `core.hooksPath` they also give the one-line way to use YASS's folder in this repo anyway (claude)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->

### 2026-10-05 (claude)
- Did: `yass init --hooks` checks for a hooks setup first (`internal/yass/hooksetup.go`): a local `core.hooksPath` elsewhere, a global one, or executable non-sample hooks in the clone's hooks folder (found with `git rev-parse --git-path hooks`, so worktrees work). With one, it leaves `core.hooksPath` alone, still writes everything else, and prints how to wire `yass hook` in for husky, lefthook, the pre-commit framework or plain git; already on says so. README: the hook section explains it, with an "Already have hooks?" subsection of snippets; `docs/install.md`'s `--hooks` row too. e2e §28 (17 checks, including `yass upgrade` leaving a custom `core.hooksPath` alone); all 377 e2e checks and `tests/site.sh` pass.
- Next: Jaime reviews locally; when approved, push the plan and code branches as a two-PR stack, then archive after merge.


## yass/archive/2026/10/2026-10-07-archive-that-scales/2026-10-07-archive-into-months/change.md

---
platforms: [all]
source: 
follows: 
blocked:
created: 2026-10-07T17:34:55Z
---
# Archive into months

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
yass archive stamps archived: and files changes under archive/<YYYY>/<MM>/; every command reads both that and the flat layout, the hook accepts archiving into months, and the templates and docs describe it.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC1–AC6, AC14, AC15 in [plan.md](../plan.md)
- [x] The hook passes an archive into a month and still flags edits under `archive/` (AC12's archiving half)

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] Loader reads `archive/<YYYY>/<MM>/<name>/` and flat entries alike (`changeDirs`)
- [x] `yass archive` moves into the UTC month, stamps `archived:` (`setMeta`), stages both, refuses a name already archived
- [x] `archived:` that isn't a time warns, like `created:`
- [x] Hook: an archived change's folder is under `<YYYY>/<MM>/` (`changeFolder`)
- [x] e2e: archive paths by month; stamp, nothing else changed, earlier-month change, duplicate name, mixed layouts for `follows:`/`blocked:`/queue/status; unit tests for `setMeta`, `changeFolder`
- [x] Yass folder README template, this repo's `yass/README.md`, `README.md`, `docs/monorepo.md` (Big archives)
- [x] Jaime reviews the doc text (AC14, AC15)

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- The stamp is written after the move and staged with it, so git still sees one rename (with a one-line change) and the hook passes it as an archive - stamping before `git mv` would leave the edit unstaged (claude, 2026-10-07)
- A change with no frontmatter gets one holding only `archived:` - the date has to live somewhere, and frontmatter is where every other date is (claude, 2026-10-07)
- `site/index.html` is left as is - "archive/, finished changes, as they were" is still true, and the site deploys from main, which is Jaime's call (claude, 2026-10-07)
- The examples' flat archives stay flat in this piece - they show mixed layouts work, and `migrations-in-upgrade` can move them as a real run (claude, 2026-10-07)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-07 (claude)
- Did: built and tested the monthly layout: `yass archive` stamps and files by UTC month, every reader takes both layouts, the hook knows month folders; docs and templates describe it. `go test`, `tests/e2e.sh` (401 ok) and `tests/examples.sh` pass.
- Next: Jaime reviews the doc text for AC14 and AC15; then this piece is done. AC12 completes with `migrations-in-upgrade`.
### 2026-10-07 (claude)
- Did: Jaime reviewed and merged it (#33, #34), doc text included; AC14 and AC15 done. The piece is done.
- Next: none here; `migrations-in-upgrade`, `archive-names-only` and `yass-decisions` can start.


## yass/archive/2026/10/2026-10-07-archive-that-scales/2026-10-07-archive-names-only/change.md

---
platforms: [all]
source: 
follows: 
blocked: 2026-10-07-archive-into-months
created: 2026-10-07T17:34:55Z
---
# Archive names only

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
Commands that don't show archived contents read only archived changes' names, and yass status stays under 1s with 10,000 archived changes.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC16–AC18 in [plan.md](../plan.md)

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] Archived changes load by name only (`nameOnly`); `yass status --archived` and `status --archived <change>` read them (`readArchived`), warning about any it can't read; `archived:` is checked there
- [x] Archived pieces are found only when a name isn't an archived change (`archivedPieces`): `follows:`, `blocked:`, and `status --archived`
- [x] e2e: unreadable archived `change.md` files don't affect `yass status`, still resolve, and `--archived` names them
- [x] `tests/bench.sh`: 10,000 archived changes over 200 folders and 12 months, some followed by active changes: `yass status` 856–913ms (target < 1s); 391ms without the archive

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- Archived pieces load lazily too, not with their change - opening each archived change's folder cost ~220ms at 10,000; only a name that isn't a top-level archived change needs them (claude, 2026-10-07)
- The rest of the archive's cost (~450ms at 10,000) is left alone: it's `git ls-files --others` walking every folder in the working tree to find untracked yass folders, whatever the pathspec (an exclude pathspec and `core.untrackedCache` didn't help). It grows with folders in the tree, which eviction (`keep-and-evict`) bounds; the target holds without it (claude, 2026-10-07)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-07 (claude)
- Did: names-only loading of the archive, lazy archived pieces, e2e and bench. `go test`, `tests/e2e.sh` (426 ok), `tests/examples.sh`, `tests/bench.sh` pass. The piece is done.
- Next: none here.


## yass/archive/2026/10/2026-10-07-archive-that-scales/2026-10-07-decisions-from-history/change.md

---
platforms: [all]
source: 
follows: 
blocked: 2026-10-07-yass-decisions, 2026-10-07-keep-and-evict
created: 2026-10-07T17:34:55Z
---
# Decisions from history

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
yass decisions reads evicted months from git, only when a query reaches them, and says what it couldn't read in a shallow clone.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC43–AC47, AC49, AC50, AC29 in [plan.md](../plan.md)

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `history.go`: evicted months read from git by the commit and path in their `.evicted` file: one `ls-tree` per month, one `cat-file --batch` for every file; pieces and other `.md` files included; months whose commit is missing are reported
- [x] `yass decisions` reads them only when a query reaches them: `--since` up to the month's end, `--change` following the chain month by month, `--cites`, or a plain query the tree doesn't fill
- [x] `--cites <sha>`: boxes citing a commit by any prefix of its hash, in active, archived and evicted changes; text and `--json`
- [x] A missing month prints a note after the results (on stderr with `--json`), with `git fetch --unshallow`; exit 0
- [x] `yass-log` uses `--cites`; nothing in the kit reads `archive/` directly (AC29); `docs/skills.md`, `docs/monorepo.md`, `README.md`
- [x] go test (`TestReadEvictedGitProcesses`: 1,000 evicted changes, at most 4 git processes); e2e section 34 (503 ok), including shallow clones

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- A query with neither `--since` nor `--change` reads evicted months only when the tree's matches don't fill `--limit` - evicted months are older than everything left in the tree, so a full answer from the tree doesn't need them; an old dated entry in a long-lived change could in principle sort below an evicted one, which this accepts (claude, 2026-10-08)
- `--change` reads only the evicted months its chain reaches, one round at a time, rather than every evicted month - an evicted change can only follow changes archived before it, so the chain never leads back from evicted months into the tree (claude, 2026-10-08)
- `--cites` reads every evicted month - a commit can be cited anywhere, and "no box cites it" must be true when it says so (claude, 2026-10-08)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-08 (claude)
- Did: built and tested reading evicted months from git and `--cites`, and switched `yass-log` to it. `go test`, `tests/e2e.sh` (503 ok), `tests/examples.sh` pass. The piece is done, and so is the change.
- Next: `yass archive 2026-10-07-archive-that-scales`, in its own pull request.
### 2026-10-08 (claude)
- Did: fixed CI on Linux: the note naming unreadable months listed them in Go's random map order (2025/02 before 2025/01); months are now read and named oldest first.


## yass/archive/2026/10/2026-10-07-archive-that-scales/2026-10-07-keep-and-evict/change.md

---
platforms: [all]
source: 
follows: 
blocked: 2026-10-07-archive-names-only
created: 2026-10-07T17:34:55Z
---
# Keep and evict

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
A soft keep limit per yass folder, notes when it's passed, and yass evict removing oldest whole months with a write-once manifest each, so evicted names still resolve.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC31–AC42, AC48 in [plan.md](../plan.md)

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `yass.yaml` `archive: { keep: N }` (default 2,000, after the revision), unknown keys under it warn; `Root.keep()`
- [x] `.evicted` manifests read at load: evicted changes resolve by name (`Change.Evicted`), never read from disk
- [x] Notes past `keep` in `yass status` and `yass archive`; `status --archived` lists the tree and summarizes what's evicted
- [x] `yass evict` (`evict.go`): plans every month first and refuses before changing anything (outside git, uncommitted or never-committed months); oldest whole months until at or under `keep`, never this month; `git rm`, write the manifest, stage it, print the commit
- [x] Hook: a deletion passes only when the same commit adds its month's `.evicted` naming it; a new `.evicted` passes, not alongside code
- [x] Docs: `docs/monorepo.md` "Keeping the working tree small" (keep, evict, `.git` and partial clones, scheduled CI recipe); rule 5 in the templates, this repo's `yass/README.md`, `README.md`, `AGENTS.md` section
- [x] go test (`TestArchiveKeep`); e2e section 33 (482 ok): notes, eviction, manifest, hook both ways, shallow clone, refusals, two branches merging, a squash merge
- [x] `tests/bench.sh`: main 834ms, this branch 840ms on the same repo; the noise between runs is larger
- [x] Jaime reviews the docs (AC42)

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- Evicted changes join `r.Archived` as name-only entries marked `Evicted`, rather than a list of their own - every name lookup (follows, blocked, the queue, `yass archive`'s duplicate check, `status --archived <name>`) already walks the archive, so they resolve with no new code paths (claude, 2026-10-08)
- `yass evict` acts on every yass folder past its keep, each with its own commit command - one command for "make the repo fit", and each folder's repo commits its own (claude, 2026-10-08)
- It checks every month it means to evict before deleting any, so a refusal leaves the tree as it was (claude, 2026-10-08)
- Changes still sitting flat in `archive/` count toward `keep` but can't be evicted; `yass evict` says to run `yass upgrade` first when they're what's left (claude, 2026-10-08)
- The CI recipe opens a pull request with `gh` rather than a third-party action, and says that a PR opened with `github.token` doesn't start other workflows (claude, 2026-10-08)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-08 (claude)
- Did: built and tested `keep`, `yass evict` and its manifests, the hook's eviction rule, notes and docs. `go test`, `tests/e2e.sh` (482 ok), `tests/examples.sh`, `tests/bench.sh` pass.
- Did: fixed a marking slip from piece 1: AC1's `[x]` had landed on the commented-out example at the top of plan.md's Acceptance, not on AC1 itself.
- Next: Jaime reviews the docs (AC42); then this piece is done and `decisions-from-history` can start.
### 2026-10-08 (claude)
- Did: Jaime lowered the default `keep` to 2,000 after measuring (revision underneath this PR); constant and docs follow.
- Next: Jaime reviews the docs (AC42).
### 2026-10-08 (claude)
- Did: Jaime reviewed the docs and merged it (#44, #45). The piece is done.
- Next: none here.


## yass/archive/2026/10/2026-10-07-archive-that-scales/2026-10-07-migrations-in-upgrade/change.md

---
platforms: [all]
source: 
follows: 
blocked: 2026-10-07-archive-into-months
created: 2026-10-07T17:34:55Z
---
# Migrations in upgrade

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
yass upgrade runs small, self-checking migration steps for every yass folder; the first moves a flat archive into months, dated from git or the change's name.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC7–AC11, AC13 in [plan.md](../plan.md)
- [x] The hook passes a migration move and still flags other moves under `archive/` (AC12's migrating half)

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `migrate.go`: a list of steps, each with its own needed-check; `yass upgrade` runs them for every yass folder after writing YASS's files, also when there are none to write or they're up to date
- [x] Step `archive-into-months`: date from `archived:`, else the latest commit that added files to the change (one `git log` per folder), else its name; moves with `git mv` where tracked, stamps, stages; prints a pathspec commit for that folder's repo
- [x] Hook: the move passes as a rename or as a deletion paired with its addition, if the content differs only by the stamp (and the frontmatter that holds it); moves between months and edits are still flagged
- [x] go tests for the mechanism and an outside-git run; e2e for git dates, untracked changes, pieces, a plain outside folder, a separate plans repo, a second run, and the hook
- [x] `examples/solo-app`'s archive moved into months, stamped with the story's dates; its README tree updated
- [x] `README.md` and `docs/install.md` say what `yass upgrade` migrates and to upgrade binaries before committing it
- [x] AC13: `yass upgrade` built at 0.3.0 (the repo's stamp) wrote no files and migrated this repo (11 changes) and the private plans folder (2); each committed on its own, the hook passing; `yass status --strict` clean

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- The hook also accepts the move when git reports it as a deletion and an addition - on a small `change.md`, the added stamp drops git's rename similarity under 50% (it did in e2e) (claude, 2026-10-07)
- A change's date is the latest commit that added any of its files, not just `change.md` - a large change may have no top-level `change.md`, only pieces' (claude, 2026-10-07)
- Migrations run even when YASS's files need nothing - an up-to-date repo can still have a flat archive (claude, 2026-10-07)
- The example's archive was moved by the step, then re-stamped with the story's finish dates (September 2026) by hand - the step dated them by when the example files were committed here, which fits no story (claude, 2026-10-07)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-07 (claude)
- Did: built and tested migrations in `yass upgrade` and the archive-into-months step, with the hook accepting the move. `go test`, `tests/e2e.sh` (423 ok), `tests/examples.sh` pass.
- Did: a test run from `examples/solo-app` with a dev build also migrated Jaime's private plans folder (it's included through the clone's `yass.include`). Nothing else was uncommitted there; I reset it to its last commit, so AC13 still runs with the release. Worth knowing: from inside an ignored example folder, a clone's private team folders still count as yass folders.
- Next: after a release, AC13 (this repo and the private plans folder), then this piece is done.
### 2026-10-07 (claude)
- Did: AC13 with a local build, as Jaime revised it: this repo's archive in `yass: archive-into-months` (its own commit and PR), the private plans folder's in a local commit there (8e086e8, not pushed). Dates are the archive PRs' squash-merge times. The piece is done.
- Next: none here. Anyone on an older `yass` should rebuild from main once this lands: older binaries don't read month folders.


## yass/archive/2026/10/2026-10-07-archive-that-scales/2026-10-07-yass-decisions/change.md

---
platforms: [all]
source: 
follows: 
blocked: 2026-10-07-archive-into-months
created: 2026-10-07T17:34:55Z
---
# yass decisions

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
yass decisions lists dated decisions across active and archived changes, filtered and bounded, as text or JSON, and yass-log and yass-shape use it instead of reading archive/.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC19–AC28, AC30 in [plan.md](../plan.md)

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `decisions.go`: entries from every change's and piece's `## Decisions`, active and archived, every yass folder; who and date from the last parenthesis; filters, newest first, `--limit` with a count of what's left out; `--json`; read warnings to stderr
- [x] `--change` follows `follows:` both ways and takes in pieces; a piece alone brings what its change is linked to
- [x] Change templates, the `AGENTS.md` section and `yass-work` ask for `(<who>, <YYYY-MM-DD>)`; `README.md` shows it and lists the command
- [x] `yass-log` and `yass-shape` (and `docs/skills.md`) look decisions up with `yass decisions`
- [x] go test (`TestParseEntry`), e2e section 32 (two yass folders, a piece, an archived change, a follows chain, JSON, templates)
- [-] AC29: moved to `decisions-from-history` with a `--cites` search
- [x] AC30: in a scratch copy upgraded with a local 0.4.0 build, following the refreshed `yass-log`: `yass decisions --about "existing hooks"` finds the decisions, and `git log --follow` across the migration names #16

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- Decisions are newest first by their own date, else their change's archived date, else its created date; ties keep file order - "why did we…" is usually about recent calls, and an undated entry is placed as well as the record allows (claude, 2026-10-07)
- `yass decisions` prints only warnings about files it couldn't read, to stderr - its stdout may be JSON, and the repo's other warnings belong to `yass status` (claude, 2026-10-07)
- The AC29 step is dropped here: Jaime chose a `--cites` search, built with reading evicted months in `decisions-from-history`, and AC29 moved there (claude, 2026-10-07)
- Text lines have the entry's own date first (blank when it has none), then the change, the decision, who, and the change's dates in brackets - one line per entry, as AC19 asks, with the date where scanning eyes look first (claude, 2026-10-07)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-07 (claude)
- Did: built and tested `yass decisions`, dated entries in the templates and playbooks, and switched `yass-log` and `yass-shape` to it. `go test`, `tests/e2e.sh` (450 ok), `tests/examples.sh` pass.
- Open (AC29): `yass-log` still finds a commit's decision by grepping every yass folder for `code:.*<sha>`, archive included. It's a needle search, so its output stays small, but it does walk `archive/`. Options: accept it and narrow AC29 to decision lookups (a plan revision), or add a citation search to the CLI (e.g. `yass decisions --cites <sha>`, searching boxes too). Jaime to choose.
- Next: Jaime's call on AC29; AC30 once the installed playbooks are refreshed (release, or `yass upgrade` with a local build).
### 2026-10-07 (claude)
- Did: a release dry run with local builds. A repo set up and archived with v0.3.0, upgraded with 0.4.0: files restamped, its flat archive migrated, both committed without hook warnings, a new archive filed by month, `yass decisions` reads it. A teammate still on 0.3.0 gets the "upgrade your binary" note and archives flat, which the next `yass upgrade` moves. A scratch copy of this repo upgraded cleanly too (12 files, nothing left to migrate), which gave AC30.
- Next: AC29, which Jaime is deciding (a `--cites` search in `decisions-from-history`, or narrowing AC29).
### 2026-10-07 (claude)
- Did: Jaime chose `--cites` for AC29; the plan revision moves it to `decisions-from-history`. Everything this piece delivers is done.
- Next: none here.


## yass/archive/2026/10/2026-10-07-archive-that-scales/change.md

---
platforms: [all]
source: Jaime, 2026-10-07: conversation about the archive growing without bound on large teams
follows: 
blocked:
created: 2026-10-07T16:51:19Z
archived: 2026-10-08T22:15:40Z
---
# Archive that scales

## Goal
<!-- Intent. One paragraph; the detail is in prd.md (why and what) and plan.md (how and acceptance). -->
The archive stays manageable however big a team gets. Archived changes are dated and kept in month folders that GitHub can browse. The CLI reads only their names unless it needs more. Agents ask `yass decisions` for past decisions and get bounded answers instead of grepping everything. Each yass folder has a soft limit on archived changes in its working tree (2,000 by default); past it, YASS says so, and `yass evict` moves the oldest months out to git history, with a small manifest saying where to find them, so a query reaches into history only when it has to.


## Decisions
<!-- Progress. "- <decision> - <why> (<who>)" for calls that span the whole change. -->
- The archive is read mostly by YASS's tooling and by AI explaining decisions; the concern is bloat in clones and on GitHub - this sets what the change optimizes for (Jaime)
- Existing flat archives are migrated, this repo's included - migrating our own archive tests and validates the migration (Jaime)
- Scale target is solo to 100+ person teams, solved per yass folder so it nests and ladders up; past that, git and GitHub are the limit - YASS shouldn't build for the rare case where git itself breaks down (Jaime)
- Old archived changes may leave the working tree for git history, governed by optional `yass.yaml` settings whose defaults are high enough that most teams never set them; git history is read only when a query needs it - crawling history on every query would slow the CLI (Jaime)
- `yass archive` stamps `archived:` in the frontmatter, and the archive is organized by archive date, not by the date in the change's name - folders fill in order and old months never receive late arrivals, so they can be evicted safely; finding a change by name means listing month folders, which is cheap (Jaime, refined with claude)
- Eviction is by count (`keep`), not by age - it exists to control size, not freshness (Jaime)
- Eviction happens inside `yass archive`, in the archive commit, and removes whole months - evicting one change per archive would rewrite a manifest on every archive and conflict constantly on busy teams; whole months make eviction rare and the manifests write-once (Jaime, months from claude)
- Each evicted month's manifest records the commit that last touched the month's folder - the same on any branch, so concurrent evictions merge cleanly; and old enough to be on the main branch, so squash merges don't lose it (claude)
- `keep` counts top-level changes, default 10,000 - people can reason about a count of changes, not of files; 10,000 is about a year at a 100-person team's pace (claude, agreed by Jaime)
- A yass folder archiving more than about 1,000 changes a month should split into team folders; month folders aren't split further, and an occasional big month is fine (Jaime)
- Migration runs in `yass upgrade`, as the first step of a small, general migration mechanism whose steps can be dropped later, not something carried forever (Jaime)
- Decision entries carry their own date; `yass decisions` surfaces it alongside the change's `created:` and `archived:` dates (Jaime)
- `yass decisions` has `--json`, and `--change` follows the `follows:` chain both ways - JSON gives tooling a stable contract while the plain text stays free to change; "why is X like this" usually spans several changes (claude, at Jaime's request)
- Replaces the call to evict inside `yass archive`: `keep` is a soft limit, `status` and `archive` say when it's passed, and eviction is its own command, `yass evict`, in its own commit, still by whole months - archive commits stay single-purpose, and eviction becomes a deliberate act by one person rather than something any archive on any branch can trigger; whole months keep each manifest written once (Jaime, whole months kept by claude)
- PRD approved for planning, with a scheduled CI recipe for `yass evict` added to R20 (Jaime, 2026-10-07)
- Plan: six pieces, the month layout first; formats and hook rules pinned in design.md (archive date and nesting, `archived:` stamp, `.evicted` manifest, soft `keep` with `yass evict`, migration steps, dated entries as `(<who>, <YYYY-MM-DD>)`) (claude, 2026-10-07)
- Rule 5's eviction wording moves from AC14 (piece 1) to a new AC48 (piece 5) - piece 1 would otherwise tell users about `yass evict` releases before it exists; same bar, later piece (claude, 2026-10-07)
- AC13 runs the migration on this repo and the private plans folder with a local build of `yass upgrade`, at the version the files are stamped with, instead of waiting for a release - same code, checked sooner, and the build writes no stamp for a version that doesn't exist (Jaime, 2026-10-07)
- `yass-log` keeps consulting the archive, but through `yass`: a `--cites <sha>` search joins piece 6 (R25, AC49, AC50), and AC29 moves there with it - once months can be evicted, a grep of the working tree would miss evicted changes and report a commit as unlinked; one way into the archive keeps layout changes out of the playbooks (Jaime, 2026-10-07)
- The default `keep` drops from 10,000 to 2,000 - measured, each archived change adds about 45µs to every `yass status` (git walks its folders: +170ms at 2,000, +460ms at 10,000) and about 10KB to every checkout; `keep` is per yass folder, so a monorepo's folders add up. 2,000 keeps about a year for a 10-person team, and `yass decisions` reaches evicted months (piece 6) (Jaime, 2026-10-08)
- One large change with five milestones - each milestone helps on its own, so eviction can wait if nobody needs it yet (Jaime)

## Log
<!-- Progress. Change-level notes; each piece keeps its own Log.
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->


## yass/archive/2026/10/2026-10-07-archive-that-scales/design.md

# Archive that scales: design
<!-- Intent: the calls that are hard to undo. Edit only in commits without code. -->

## Context
This change sets formats that end up in every team's committed files and that tools and agents will read for years: where archived changes live, a new frontmatter field, a per-month eviction manifest, a dated decision entry, a new command, and how `yass upgrade` migrates folders. Once a team has archived, migrated or evicted under one of these, changing it means another migration. The PRD fixed the big calls with Jaime on 2026-10-07: archive by archive date, an `archived:` stamp, migrate existing archives, eviction by count as a soft limit with its own command, dated decision entries, a light migration mechanism in `yass upgrade`. Each is pinned down here.

## 1. Where archived changes live
**Options.**
- (a) `archive/<YYYY>/<MM>/<name>/`, by the `archived:` date.
- (b) The same layout, by the date in the change's name.
- (c) `archive/<YYYY-MM>/<name>/`, one level.
- (d) Flat, as today, with an index file.

**Decision: (a)** (Jaime chose the archive date, claude the nesting, 2026-10-07). By archive date, a month only receives changes while it's the current month, so a past month never changes again and can be evicted safely; (b) would let a change started in January land in January when it's archived in October, possibly after January was evicted. Nesting by year keeps `archive/` itself to a handful of entries, where (c) adds twelve a year. (d) leaves the folder unbounded on GitHub, and a committed index conflicts on busy teams (a PRD non-goal). The month comes from `archived:` in UTC. A year folder is four digits and a month folder two, so neither can be mistaken for a change, whose name starts `YYYY-MM-DD-`.

Finding an archived change by name means listing month folders' names, not reading them. A name is still unique per yass folder: `yass archive` refuses a name that's already archived in any month.

## 2. The `archived:` stamp
**Options.**
- (a) `archived: <RFC 3339 UTC>` in the top-level `change.md` frontmatter, written by `yass archive` in the archive commit.
- (b) No stamp; read the date from git history when needed.
- (c) No stamp; the path's month is enough.

**Decision: (a)** (Jaime, 2026-10-07). It's the same shape as `created:`, works outside git and in shallow clones, survives copying a folder, and gives `yass decisions` an exact date. (b) costs a git call per change, which is what M2 exists to avoid; (c) loses the day. It's written as part of the move, like `yass archive` already marks delivered criteria (stacked-prs-and-squash-merges), so the archived change is never edited after it lands. Pieces aren't stamped; they're archived with their change and share its date.

## 3. The eviction manifest
**Options.**
- (a) One file per evicted month, `archive/<YYYY>/<MM>.evicted`, written once.
- (b) One `archive/evicted.yaml` for the whole folder, appended to.
- (c) A stub file per evicted change.

**Decision: (a)** (claude, 2026-10-07, following Jaime's whole-month eviction). (b) is edited by every eviction and conflicts; (c) leaves one file per change behind, which is what eviction exists to remove. The file is YAML:

```yaml
# Evicted by yass evict. Read a change back: git show <commit>:<path>/<name>/change.md
commit: 3f9c2e1a7b…            # full hash of the last commit that touched the month's folder
path: yass/archive/2026/03     # the month's folder at that commit, relative to the top of its git repo
changes:                       # top-level changes, by name, sorted
  - 2026-03-02-some-change
  - 2026-03-04-another-change
```

`commit` is `git log -1 --format=%H -- <month folder>` on the branch being evicted, so two branches evicting the same month from the same history write byte-identical files and git merges them cleanly. A month is old enough to evict only after it has stopped changing, so its last commit is on the main branch and survives squash merges. `path` is recorded because a yass folder can move or live in another repo (`yass.yaml` `path:`). Pieces aren't listed; a piece resolves through its change.

## 4. Eviction: when and how much
**Options.**
- (a) A soft `keep` limit; `yass status` and `yass archive` say when it's passed; `yass evict` removes oldest whole months, in its own commit.
- (b) `yass archive` evicts as needed, in the archive commit.
- (c) A hard limit the hook enforces.

**Decision: (a)** (Jaime, 2026-10-07; whole months from claude). Archive commits stay single-purpose, and eviction is a deliberate act by one person instead of a side effect any archive on any branch can trigger. (b) rewrites a manifest on nearly every archive at the limit, and (c) blocks commits over housekeeping. `yass evict` removes the oldest months until the count is at or under `keep`, never the current month, and only months with no uncommitted changes. It needs git in the yass folder: outside git, eviction would destroy data, so it refuses. `keep` counts top-level changes in the working tree, default 2,000 (Jaime, 2026-10-08, after measuring: see change.md Decisions).

## 5. How `yass upgrade` migrates
**Options.**
- (a) A list of small steps, each deciding from the files whether it's needed, and doing nothing when it isn't.
- (b) A format version stamped in each yass folder, with migrations from version to version.

**Decision: (a)** (Jaime, 2026-10-07). A step can be deleted in a later release without touching the others or any stamp, which is the point: migrations shouldn't be carried forever (a PRD non-goal rules out (b)). A step names itself, says what it would change, makes the change in the working tree, and prints how to commit it on its own. Steps run per yass folder, including ones outside the repo, after `yass upgrade` writes YASS's files. The first step moves a flat archive into months: `archived:` comes from the date of the commit that added the change's `change.md` under `archive/` in that folder's git repo, else the date in its name.

## 6. Dated decision entries
**Options.**
- (a) The date joins the trailing credit: `- <decision> - <why> (<who>, <YYYY-MM-DD>)`.
- (b) A leading date: `- 2026-10-07: <decision> - <why> (<who>)`.
- (c) No date on entries; use the change's dates, or `git blame`.

**Decision: (a)** (Jaime chose dated entries, claude the form, 2026-10-07). It extends the format in use without moving anything, and reads the same with or without a date, so entries written before this stay valid. (b) reorders every existing entry's shape; (c) can't tell decisions made months apart in one long change, and blame is slow and wrong after edits. `yass decisions` reads the last parenthesis on the entry's first line: text before the last comma is who, a `YYYY-MM-DD` after it is the date. An entry without one has no date of its own.

## 7. What the hook allows in the archive
The archive stays append-only, with exactly three ways in or out, each in a commit without code (claude, 2026-10-07):
- **Archiving:** a change moved from `changes/` to `archive/<YYYY>/<MM>/<name>/`; its content may differ by the `archived:` stamp and boxes `yass archive` marks.
- **Migrating:** a change moved from `archive/<name>/` to `archive/<YYYY>/<MM>/<name>/`, differing only by the `archived:` stamp.
- **Evicting:** an archived change deleted, in a commit that adds its month's `.evicted` file listing it.

Anything else under `archive/` is flagged as today.

## Consequences and rollback
- **Older binaries.** A yass from before this change reads `archive/2026` as a change named `2026`, and its hook flags new archives as edits. The upgrade that migrates also restamps YASS's files, so older binaries print their existing "written by a newer yass; upgrade your binary" note. The docs say to upgrade everyone's binary before running `yass upgrade` on a shared repo.
- **Rolling back the layout** is another migration step moving months back to flat; the `archived:` stamp can stay, since it's harmless.
- **Rolling back an eviction** is `git revert` of its commit. Nothing is lost while history is.
- **A team that wants no eviction** sets a very high `keep`, or never runs `yass evict`: past `keep` is only a note.


## yass/archive/2026/10/2026-10-07-archive-that-scales/plan.md

# Archive that scales: plan
<!-- Intent: how, and how we'll know it's done. With code, only mark boxes here ([/] in progress, [x] done);
     anything else is a plan revision in its own commit. -->

## Approach
<!-- The shape of the solution. Hard-to-undo calls go in design.md. -->
**M1 changes where archives go.** `cmdArchive` ([`commands.go`](../../../internal/yass/commands.go)) writes `archived:` into the frontmatter and moves the change to `archive/<YYYY>/<MM>/<name>/` ([design §1–2](design.md)). The loader in [`repo.go`](../../../internal/yass/repo.go) (`load`, `resolve`, `check`, `checkDeps`, `checkQueues`) walks year and month folders as well as flat entries, so a half-migrated folder works. The hook ([`hook.go`](../../../internal/yass/hook.go)) learns that an archived change's folder is three levels under `archive/`, and accepts archiving into months ([design §7](design.md)). The yass folder README and `AGENTS.md` templates ([`templates/`](../../../internal/yass/templates/)), the top-level [`README.md`](../../../README.md) and [`docs/monorepo.md`](../../../docs/monorepo.md) describe the layout, the `archived:` field, the reworded rule 5, and splitting busy folders into team folders.

**M1 also adds migrations to `yass upgrade`.** A new `migrate.go` holds a list of steps ([design §5](design.md)); [`upgrade.go`](../../../internal/yass/upgrade.go) runs them for every yass folder after writing YASS's files, including folders outside the repo, using git in each folder's own repo. The first step moves a flat archive into months. The hook accepts that move. This repo's archive and Jaime's private plans folder are migrated with the released step, as its first real runs.

**M2 reads only names.** Archived changes load as names (and their pieces' names) without reading `change.md`; titles and frontmatter are read on first use, which only `status --archived`, `status <archived change>` and `yass decisions` need. [`tests/bench.sh`](../../../tests/bench.sh) adds 10,000 archived changes across its team folders and months, timed against the existing 1s `yass status` target. This touches the same loader as the in-flight `speed-at-scale` piece of `monorepos-at-scale-and-plans-repos`; both keep its bench passing.

**M3 adds `yass decisions`.** A new `decisions.go` reads the `## Decisions` section of each change's and piece's `change.md`, active and archived, in every yass folder, parsing who and date from each entry's last parenthesis ([design §6](design.md)). Each result carries the entry's date and the change's `created:` and `archived:`; filters use the entry's date, else `archived:`, else `created:`. Results are newest first, cut at `--limit`, with a line saying how many were left out. `--json` emits an array of objects with the same fields. The change templates, `yass-work` and the `AGENTS.md` section ask for dated entries; [`yass-log`](../../../kit/.agents/skills/yass-log/SKILL.md) and [`yass-shape`](../../../kit/.agents/skills/yass-shape/SKILL.md) use `yass decisions` (and `yass status --archived`) instead of grepping `archive/`. Playbook edits are made in `kit/` and reach this repo through `yass upgrade`.

**M4 adds `keep` and `yass evict`.** [`config.go`](../../../internal/yass/config.go) learns `archive: { keep: N }`. `yass status` and `yass archive` add a note (not a warning, so `--strict` still passes) when a folder is past `keep`. A new `yass evict` removes whole months and writes their manifests ([design §3–4](design.md)); the loader reads `.evicted` files so evicted names still resolve; `status --archived` summarizes them. The hook accepts evictions. The docs cover `keep`, eviction, clone size, and a scheduled CI recipe that runs `yass evict` and opens a PR.

**M5 reads evicted months from git, and finds citations.** `yass decisions --cites <sha>` searches boxes for `code:` citations of that commit, in the tree and, through the manifests, in evicted months; `yass-log`'s "starting from code" step uses it instead of grepping the yass folders. `yass decisions` reads only the manifests until a query's date range or `--change` reaches an evicted month; then it reads those months' `change.md` files from the manifest's commit with one `git cat-file --batch`. A missing commit (shallow clone) is reported with the months it covers and `git fetch --unshallow`; the other results still print.

## Acceptance
<!-- Per milestone, one observable behavior each:
### M1
- [ ] AC1 (R1) Given …, when …, then … — verify: <test, flow, or manual steps> -->
### M1
- [x] AC1 (R1) Given a finished change, when you run `yass archive` on it, then its `change.md` frontmatter has `archived:` with the current UTC time, and nothing else in the folder changed beyond what `yass archive` already marks — verify: e2e
- [x] AC2 (R2) Given that archive, then the change is at `archive/<YYYY>/<MM>/<name>/` for the `archived:` date, with its pieces inside it, and the move is staged — verify: e2e
- [x] AC3 (R2) Given a change whose name's date is in an earlier month than today, when it's archived, then it lands in today's month — verify: e2e
- [x] AC4 (R2) Given a change with the same name already archived in some month, when you archive it, then `yass archive` refuses and names where the other one is — verify: e2e
- [x] AC5 (R3) Given a yass folder with some archived changes flat and some in months, then `yass status --archived` lists both, and `follows:`, `blocked:` and `queue.md` resolve names in either without warnings — verify: e2e
- [x] AC6 (R3) Given a change folder under `archive/<YYYY>/<MM>/`, then `yass status <name>` finds it, as with a flat one — verify: e2e
- [x] AC7 (R4, R21) Given a yass folder in git with a flat archive, when you run `yass upgrade`, then each archived change moves to the month of the commit that added its `change.md` under `archive/`, its `archived:` is that commit's date, and the output names the step and says to commit the move on its own — verify: e2e
- [x] AC8 (R4) Given a flat archive in a yass folder outside git (`yass.yaml` `path:` to a plain folder), when you run `yass upgrade` from the repo, then each change moves to the month of the date in its name, with `archived:` that date at 00:00 UTC — verify: e2e
- [x] AC9 (R4) Given a yass folder in a different git repo from the code (`yass.yaml` `path:`), when you run `yass upgrade`, then the dates come from that repo's history and the commit command it prints runs in that repo — verify: e2e
- [x] AC10 (R21) Given a yass folder already migrated, when you run `yass upgrade` again, then the migration step does nothing and says nothing — verify: e2e
- [x] AC11 (R21) Given the migration steps in the source, then each is one entry in a list with its own needed-check, and removing the archive step leaves the other steps and `yass upgrade` working — verify: go test (a step list without it runs clean)
- [x] AC12 (R4, R1) Given a migration commit or an archive into a month, when it's committed with the hook on, then the hook passes it; and given an edit, deletion or other move under `archive/`, then the hook still flags it — verify: e2e
- [x] AC13 (R5) Given this repo's flat archive and Jaime's private plans folder, when `yass upgrade` built from this piece runs on them (at the version their files are stamped with, so it writes no files and only migrates), then every archived change is in its month with `archived:` set, `yass status --strict` passes, and each migration is committed on its own — verify: manual: run it on both, check `yass status --archived` and the commits
- [x] AC14 (R19) Given a new `yass init`, then the yass folder README describes `archive/<YYYY>/<MM>/` — verify: e2e (the text); manual: review
- [x] AC15 (R19, R24) Given `README.md` and `docs/monorepo.md`, then they show the monthly layout and the `archived:` field, and recommend that a yass folder archiving more than about 1,000 changes a month split into team folders — verify: manual: review

### M2
- [x] AC16 (R6) Given archived changes whose `change.md` is unreadable, when you run `yass status` (no `--archived`), then it succeeds without errors about them, and `follows:` and `blocked:` naming them still resolve — verify: e2e (`chmod 000` on archived `change.md` files)
- [x] AC17 (R6) Given the same setup, when you run `yass status --archived`, then it reports the unreadable ones — verify: e2e
- [x] AC18 (R7) Given `tests/bench.sh`'s repo with 200 yass folders, 2,000 active changes and 10,000 archived changes across months, then `yass status` stays under 1s — verify: `tests/bench.sh`

### M3
- [x] AC19 (R8) Given active and archived changes in two yass folders with `## Decisions` entries, when you run `yass decisions`, then each entry prints once on its own line with its change, who, the decision, its own date if it has one, and the change's `created:` and `archived:` dates — verify: e2e
- [x] AC20 (R8) Given a piece's `## Decisions`, then its entries are listed under the piece's name — verify: e2e
- [x] AC21 (R22) Given an entry `- X - Y (Jaime, 2026-10-07)`, then who is `Jaime` and its date is 2026-10-07; given `- X - Y (claude, agreed by Jaime)`, then who is the whole credit and it has no date of its own — verify: go test
- [x] AC22 (R9) Given entries across several months, when you pass `--since` and `--until`, then only entries whose date (their own, else the change's `archived:`, else `created:`) falls in the range print — verify: e2e
- [x] AC23 (R9) Given `--about "squash merge"`, then only entries containing those words, case-insensitively, in the decision or the change's title print — verify: e2e
- [x] AC24 (R9) Given `--change B` where B follows A and C follows B, then entries from A, B and C print, and none from unrelated changes — verify: e2e
- [x] AC25 (R9) Given more matching entries than `--limit` (50 by default), then the newest print and a last line says how many were left out; with fewer, no such line — verify: e2e
- [x] AC26 (R9) Given no yass folder, or no decisions matching, then `yass decisions` says so and exits 0 — verify: e2e
- [x] AC27 (R23) Given `--json`, then the output is one JSON array whose objects have `change`, `title`, `who`, `decision`, `date`, `created` and `archived` (null when unknown), and the same filters apply — verify: e2e (parsed with `python3 -m json.tool`)
- [x] AC28 (R22) Given the change templates, `yass-work` and the `AGENTS.md` section, then they ask for `(<who>, <YYYY-MM-DD>)` on new entries — verify: e2e (template text); manual: review
- [x] AC29 (R10, R25) Given `yass-log` and `yass-shape`, then they look up past decisions and earlier work with `yass decisions` and `yass status --archived`, and nothing in the kit greps or walks `archive/` — verify: manual: review; `grep -rn "archive" kit/` shows no direct reads
- [x] AC30 (R10) Given `yass-log` asked why this repo's hook leaves existing hooks alone, then it finds the decision through `yass decisions` — verify: manual: ask it

### M4
- [x] AC31 (R11) Given `archive: { keep: 3 }` in a yass folder's `yass.yaml`, then that folder's limit is 3 and other folders keep 2,000; an unknown key under `archive:` warns like other unknown settings — verify: go test (config); e2e
- [x] AC32 (R14) Given a folder past `keep`, when you run `yass status` and `yass archive`, then each prints a note naming the folder, how far past it is, and `yass evict`; `yass status --strict` still exits 0; under `keep`, no note — verify: e2e
- [x] AC33 (R12) Given a folder 5 past `keep` whose oldest month holds 3 and next holds 4, when you run `yass evict`, then both months are deleted, the output names them with their counts, says to commit it on its own, and the current month is never touched — verify: e2e
- [x] AC34 (R12) Given a folder at or under `keep`, then `yass evict` changes nothing and says so — verify: e2e
- [x] AC35 (R12) Given a month with uncommitted changes, or a yass folder outside git, then `yass evict` refuses and says why — verify: e2e
- [x] AC36 (R13) Given an evicted month, then `archive/<YYYY>/<MM>.evicted` has the commit that last touched the month's folder, its path, and its change names sorted, as in design §3 — verify: e2e
- [x] AC37 (R13) Given two branches from the same commit that each run `yass evict` on the same month and commit, when one is merged into the other, then the merge has no conflicts — verify: e2e
- [x] AC38 (R13) Given a month evicted on a branch that was then squash-merged, then the manifest's commit is still reachable from main — verify: e2e (`git merge --squash`, then `git cat-file -e`)
- [x] AC39 (R15) Given an active change with `follows:` or `blocked:` naming an evicted change, then `yass status` resolves it without warnings, and reads no git history — verify: e2e (works with the evicted commit missing)
- [x] AC40 (R16) Given evicted months, when you run `yass status --archived`, then it lists what's in the working tree and ends with one line per yass folder giving the evicted count and months — verify: e2e
- [x] AC41 (R12, R13) Given an eviction commit with the hook on, then the hook passes it; and given a deletion under `archive/` without a matching `.evicted` file, then it still flags it — verify: e2e
- [x] AC48 (R19) Given a new `yass init`, then the yass folder README's rule 5 says the archive is append-only except for eviction by `yass evict`, and the AGENTS.md section says the same; `yass upgrade` brings existing ones in line — verify: e2e (the text)
- [x] AC42 (R20) Given `docs/monorepo.md`, then it explains `keep`, `yass evict`, that eviction bounds the working tree but not `.git` (partial clones or a plans repo for that), and gives a scheduled CI recipe that runs `yass evict` and opens a PR — verify: manual: review

### M5
- [x] AC43 (R17) Given an evicted month and a query whose dates reach it, when you run `yass decisions`, then decisions from its changes print as if they were in the tree — verify: e2e
- [x] AC44 (R17) Given `--change` naming an evicted change, or one following it, then its decisions print — verify: e2e
- [x] AC45 (R17) Given a query whose dates don't reach any evicted month, then it reads no git history: it succeeds with the same results in a clone where the evicted commits are missing — verify: e2e (shallow clone)
- [x] AC46 (R18) Given a shallow clone missing an evicted month's commit and a query that reaches it, then the other results print, a line names the months it couldn't read and `git fetch --unshallow`, and it exits 0 — verify: e2e
- [x] AC49 (R25) Given boxes citing a commit (`code: a1b2c3d`) in an active change, an archived one and a piece, when you run `yass decisions --cites a1b2c3d` (or a longer prefix of the same commit), then each box prints with its change, and nothing else does — verify: e2e
- [x] AC50 (R25) Given a box citing a commit in an evicted change, then `--cites` finds it from git, the way decisions from evicted months are read; and with that month's commit missing, it says so as AC46 does — verify: e2e
- [x] AC47 (R17) Given 1,000 evicted changes in range, then `yass decisions` reads them with a constant number of git processes, not one per change — verify: go test (counts git invocations)

## Pieces
<!-- PR-sized pieces, each its own folder in here: `yass new "<title>" --in <this change>`.
     `yass status` lists them; say here what order they go in and why. -->
1. **`2026-10-07-archive-into-months`** (AC1–AC6, AC12's archiving half, AC14, AC15). The layout everything else builds on, so it goes first.
2. **`2026-10-07-migrations-in-upgrade`** (AC7–AC11, AC12's migrating half, AC13). Needs piece 1's layout. AC13 runs on a local build, before release, so the migration lands on main with the code.
3. **`2026-10-07-archive-names-only`** (AC16–AC18). Needs piece 1's loader; can go alongside piece 2.
4. **`2026-10-07-yass-decisions`** (AC19–AC28, AC30). Needs piece 1's `archived:` dates; can go alongside pieces 2 and 3.
5. **`2026-10-07-keep-and-evict`** (AC31–AC42, AC48). Needs piece 3's names-only loader, which it extends with manifests.
6. **`2026-10-07-decisions-from-history`** (AC43–AC47, AC49, AC50, and AC29 once `yass-log` uses `--cites`). Needs pieces 4 and 5.

`monorepos-at-scale-and-plans-repos` (R13, citation checks in `yass archive`) and `stacked-prs-and-squash-merges` (M2, marking delivered criteria on archive) also change `cmdArchive`. They don't depend on each other; whichever lands second rebases.

## Validation
<!-- How the whole thing is verified before it's called done: suites, platforms, manual passes. -->
- `go test ./...`, `tests/e2e.sh`, `tests/examples.sh` and `tests/bench.sh` pass, on macOS and Linux in CI.
- This repo and Jaime's private plans folder are migrated by `yass upgrade` built from source (AC13), and `yass status --strict` passes on both.
- A dry run of eviction on a generated folder with `keep: 100`: evict, commit, then ask `yass decisions` about an evicted month in a full clone and in a shallow one.
- `yass-log` answers a "why did we…" question in this repo through `yass decisions` (AC30).


## yass/archive/2026/10/2026-10-07-archive-that-scales/prd.md

# Archive that scales: PRD
<!-- Intent: what should change or exist, and why. Not a description of the current system;
     the code and its docs are that. Edit this file only in commits without code. -->

## Why
<!-- The problem, with evidence. -->
Active changes stay few because finished ones are archived, but the archive itself only grows. One person with agents archived 11 changes in this repo in 5 days, roughly 800 a year; a 100-person team could archive 10,000 or more a year. Nothing bounds that, and it costs in four places:

- **Agents.** The archive is read mostly by YASS's tooling and by AI explaining past decisions. `yass-log` greps every archived `## Decisions` section, and `yass-shape` tells agents to check the archive. At thousands of changes that floods an agent's context before it finds anything.
- **The CLI.** Every command parses every archived change, though most only need their names (for `follows:`, `blocked:` and the queue).
- **GitHub.** `archive/` is one flat folder, and GitHub's web UI shows only the first 1,000 entries of a folder.
- **Clones.** Every archived change stays in every checkout forever.

YASS should scale from a solo developer to teams of 100 or more, per yass folder, so each team's archive is handled on its own and the whole repo scales because each team does. Beyond that, git and GitHub themselves become the limit, which is out of scope.

## Users and outcomes
<!-- Who it's for, and measurable targets ("p50 time to log < 5s"). -->
- **Agents explaining decisions** (`yass-log`, `yass-shape`), and the people asking them "why did we…".
- **Teams of any size** sharing a repo, and whoever browses their plans on GitHub.

Targets:
- `yass status` stays under the existing 1s target (200 yass folders, 2,000 active changes) with 10,000 archived changes added, held there by `tests/bench.sh`.
- No folder under `archive/` holds more than 1,000 entries at 10,000 archives a year. A month that occasionally passes 1,000 is acceptable; a yass folder that does so regularly should split into team folders.
- With the default settings, a yass folder that evicts when YASS tells it to keeps about 2,000 archived changes in its working tree.
- A decisions query returns at most its limit (50 lines by default), whatever the archive's size, and says when it cut results off.
- Two branches that each evict the same month merge without conflict.
- A decisions query that doesn't reach an evicted month reads no git history.

## Requirements
<!-- One line each: "- **R1** [M1] A user can …". IDs are never reused. -->
- **R1** [M1] `yass archive` writes `archived: <UTC timestamp>` into the change's frontmatter, as part of the archive commit.
- **R2** [M1] `yass archive` moves a change to `archive/<YYYY>/<MM>/<name>/`, by its `archived:` date.
- **R3** [M1] Every command that reads the archive (`status --archived`, `follows:`, `blocked:`, the queue, resolving a name) finds changes in both the flat and the monthly layout.
- **R4** [M1] `yass upgrade` moves a flat archive into months. It stamps `archived:` with the date of the commit that put the change in the archive, or, outside git, the date in the change's folder name. It prints how to commit the move on its own.
- **R21** [M1] `yass upgrade` runs migrations as small, separate steps of one general mechanism. Each step tells from the files whether it's needed, does nothing when it isn't, and can be deleted in a later release without touching the others. The archive move (R4) is the first step.
- **R5** [M1] This repo's archive is migrated with it, as the first real check of R4.
- **R6** [M2] Commands that don't show archived contents read only the names of archived changes, not their files.
- **R7** [M2] `tests/bench.sh` adds 10,000 archived changes to its large repo, and `yass status` meets the 1s target there.
- **R8** [M3] `yass decisions` lists the `## Decisions` entries of active and archived changes across every yass folder, one per line, with the change, who decided, the decision, and each date it knows: the entry's own date, and the change's `created:` and `archived:` dates.
- **R22** [M3] A decision entry carries the date it was made: `- <decision> - <why> (<who>, <YYYY-MM-DD>)`. The templates and playbooks say so. Entries written before this have no date of their own, and `yass decisions` shows them with the change's dates only.
- **R23** [M3] `yass decisions --json` gives the same results as JSON, with each date in its own field.
- **R9** [M3] `yass decisions` filters by `--since` and `--until`, by text (`--about`), and by change (`--change`, which takes in the changes it follows and the changes that follow it), and stops at `--limit` (50 by default), saying how many it left out.
- **R10** [M3] `yass-log` and `yass-shape` look up past decisions with `yass decisions` rather than reading `archive/` directly.
- **R11** [M4] A yass folder's `yass.yaml` can set `archive: { keep: N }`, the number of top-level archived changes (pieces count with their change) its working tree should keep. It's a soft limit: going past it changes nothing until someone evicts. Unset, `keep` is 2,000.
- **R12** [M4] `yass evict` removes a yass folder's oldest whole months until it's at or under `keep`, says which months and how many changes it evicted, and prints how to commit that on its own. It never evicts the current month, and does nothing when the folder is under `keep`.
- **R13** [M4] Each evicted month leaves `archive/<YYYY>/<MM>.evicted`, listing its changes and the commit that last touched that month's folder. Two branches that evict the same month from the same history write identical files.
- **R14** [M4] When a yass folder is past `keep`, `yass status` and `yass archive` say so, by how much, and to run `yass evict`. Neither evicts anything itself.
- **R15** [M4] An evicted change still counts for `follows:`, `blocked:` and name resolution, from its month's `.evicted` file, without reading git history.
- **R16** [M4] `yass status --archived` says how many changes are evicted, and from which months, rather than listing them.
- **R17** [M5] `yass decisions` reads evicted changes from git, using the commit in their month's `.evicted` file, only when a query's dates or `--change` reach an evicted month.
- **R18** [M5] When an evicted month's commit is missing (a shallow clone), `yass decisions` returns what it could read, names the months it couldn't, and prints the git command that would fetch them.
- **R25** [M5] `yass decisions --cites <sha>` lists the boxes in active and archived changes, evicted ones included, that cite a commit (`code: <sha>`), with their change, so finding the decision behind a piece of code doesn't mean reading `archive/` directly.
- **R19** [M1] The yass folder README and AGENTS.md templates describe the monthly layout, and rule 5 says the archive is append-only except for eviction by `yass evict`.
- **R20** [M4] The docs explain `keep` and eviction, and say that eviction bounds the working tree but not `.git`: a team that needs smaller clones uses partial clones or keeps its plans in another repo. The CI recipes show a scheduled job that runs `yass evict` and opens a PR with the result, for teams that never want to think about it.
- **R24** [M1] The docs recommend that a yass folder archiving more than about 1,000 changes a month split into team folders. A month folder occasionally passing 1,000 entries is fine.

## Non-goals
<!-- What this change will not do. Agents treat these as walls. -->
- Shrinking `.git` or rewriting history. Eviction only deletes from the working tree.
- Evicting by age, freshness or relevance. Eviction controls size, so it is by count.
- Marking decisions as superseded, or filtering out stale ones.
- A committed index or digest of the archive that every archive commit edits.
- Splitting month folders further (by day or week).
- Migrations that must live forever, or a versioned schema for yass folders.
- Moving the archive to another repo, branch or storage service.
- Editing archived changes, other than the `archived:` stamp and the one-time move into months.
- Evicting active changes, or evicting pieces apart from their change.
- Scaling past what one git repo and GitHub handle.

## Milestones
| Milestone | When it ships, a user can … |
|---|---|
| M1 | browse any team's archive on GitHub month by month, with every archived change dated, existing archives migrated |
| M2 | run `yass status` as fast with 10,000 archived changes as with none |
| M3 | ask `yass decisions` (or an agent using it) why something was decided, and get a bounded answer however big the archive is |
| M4 | cap how many archived changes a yass folder keeps in its working tree, with a sensible default nobody needs to touch |
| M5 | still get answers about evicted changes, from git, without slowing down queries that don't need them, including which change cites a commit |

## Open questions
- None yet.


## yass/archive/2026/10/2026-10-08-yass-update-the-binary-updates-itself/2026-10-08-update-check/change.md

---
platforms: [all]
source: 
follows: 
blocked:
created: 2026-10-08T20:14:53Z
---
# Update check

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
yass update --check decides what to install: the binary's version and whether it's a release build, the repo's version, the latest release; source builds get the right command. Release builds carry the marker.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC1, AC2, AC13, AC16 in [plan.md](../plan.md)
- [x] `.goreleaser.yaml` sets `main.channel=release` beside `main.version`

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `cmd/yass` passes `main.channel` (set to `release` by `.goreleaser.yaml`) through `yass.MainChannel`
- [x] `update.go`: the latest release from the `releases/latest` redirect (no API); the repo's version from its stamped files (`repoVersion`, sharing `stampedFiles` with `versionNote`); the target and why (`planUpdate`); `--check`, `--latest`, `--version`
- [x] Source builds: a tagged version means `go install …@<target>`, anything else (pseudo-version, `+dirty`, `dev`) a clone rebuild
- [x] `--version` after a command that takes it is that command's option, not "print yass's version"
- [x] go test (`TestAssetName`, `TestPlanUpdate`, `TestSourceBuild`); e2e section 35 against a fake release server (522 ok)

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- `yass update` without `--check` says what it would install and that installing isn't built yet, with the release page; it stays out of `yass --help` and the README until `update-installs` lands, so a release in between doesn't advertise a half-built command (claude, 2026-10-08)
- A release server that can't be reached fails the command (exit 2) - without the latest version there's no answer to give; `--check` exits 0 whenever it can answer, update or not (claude, 2026-10-08)
- `yass update --version vX.Y.Z` is the command's own option: the top-level `--version` now applies only without a command, or to one that doesn't take it - the PRD names the option `--version` (claude, 2026-10-08)
- e2e serves fake releases with a few lines of Python (`releases` in tests/e2e.sh), redirecting `/releases/latest` as GitHub does; piece 2 adds the files (claude, 2026-10-08)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-08 (claude)
- Did: built and tested `yass update --check`. `go test`, `tests/e2e.sh` (522 ok), and a run against github.com (latest 0.3.0) pass. The piece is done.
- Next: `update-installs`.


## yass/archive/2026/10/2026-10-08-yass-update-the-binary-updates-itself/2026-10-08-update-for-agents/change.md

---
platforms: [all]
source: 
follows: 
blocked:
created: 2026-10-08T20:14:53Z
---
# Update for agents

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
yass status and yass upgrade name yass update, a yass-update playbook does both steps with permission, and the docs describe it.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC18–AC23 in [plan.md](../plan.md)

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `yass status`'s note and `yass upgrade`'s refusal for an older binary say "`yass update` gets that version"
- [x] After installing past the repo's version, `yass update` ends with `yass upgrade` here and a commit, and that teammates then need that version
- [x] `kit/.agents/skills/yass-update/SKILL.md`: check both, ask before downloading, `yass update`, then `yass upgrade` and the diff, commits only with latitude; never works around a failed check
- [x] The `AGENTS.md` section and `yass-status` point to it when versions are out of step
- [x] `README.md` (six playbooks, the "Update YASS" row, upgrading with `yass update`, the command list), `docs/install.md` (Upgrading), `docs/skills.md` (row and section), `site/index.html` and `tests/site.sh` (six playbooks, a row)
- [x] go test (the kit carries six playbooks); e2e (563 ok): the messages, the next step, init and upgrade installing the playbook, the pointers; `tests/site.sh` (75 ok)
- [x] Jaime reviews the playbook and docs (AC21, AC23)

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- Started before `update-installs` is done: its only open item is AC12's Windows run, which nothing here depends on; `blocked:` cleared (Jaime, 2026-10-08)
- The landing page lists the sixth playbook too; it deploys from main, so it goes live when this merges (claude, 2026-10-08)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-08 (claude)
- Did: built and tested the pointers, the next step after installing, the `yass-update` playbook and the docs. `go test`, `tests/e2e.sh` (563 ok), `tests/examples.sh`, `tests/site.sh` pass.
- Next: Jaime reviews the playbook and docs (AC21, AC23); then this piece is done.
### 2026-10-08 (claude)
- Did: Jaime reviewed and merged it (#51, #52, #54). The piece is done.
- Next: none here; the change waits on AC12 (Windows) and AC17 (the real release).


## yass/archive/2026/10/2026-10-08-yass-update-the-binary-updates-itself/2026-10-08-update-installs/change.md

---
platforms: [all]
source: 
follows: 
blocked: 2026-10-08-update-check
created: 2026-10-08T20:14:53Z
---
# Update installs

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
yass update fetches, verifies (checksum, then provenance through gh) and swaps the binary in place, on every platform, changing nothing when anything fails.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC3–AC12, AC14, AC15 in [plan.md](../plan.md)

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `install.go`: refuse before downloading if the binary's folder isn't writable; check the release exists (its `checksums.txt`); fetch the archive and `checksums.txt` into a temp folder; checksum; `gh attestation verify` when `gh` is installed and signed in; unpack (`.tar.gz`, `.zip`); write the new binary beside the old, run `<new> version` and require the version asked for; swap by rename (aside to `.old` on Windows, removed on the next run)
- [x] `yass update` installs; `--require-provenance`; listed in `yass --help`
- [x] go test (`TestSwapBinary` for both strategies and a failed Windows swap, `TestChecksumOK`); e2e section 36 with fake releases and fake `gh` on a PATH with nothing else (555 ok)
- [x] A real install from github.com in scratch: a release-marked 0.2.0 build ran `yass update --version v0.3.0`; the download, checksum, provenance (the real `gh`) and swap all passed
- [-] AC12's manual part: one update on Windows, which can't run here

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- The Windows step is dropped, not done: AC12 now asks for macOS only, and Windows moved to `2026-10-09-validate-yass-update-on-windows-and-linux` (Jaime, 2026-10-09)
- "Not signed in" is checked with `gh auth status` before verifying, so an unauthenticated `gh` reads as "provenance not checked", as the design says, rather than as a failed verification (claude, 2026-10-08)
- Downloads follow redirects (GitHub serves assets from another host) while the version lookup doesn't (it reads the redirect itself) - two HTTP clients (claude, 2026-10-08)
- e2e runs `yass update` with a PATH holding only `git` and, when wanted, a fake `gh`: the real `gh` on a developer machine or a CI runner would otherwise verify fake archives against GitHub (claude, 2026-10-08)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-08 (claude)
- Did: built and tested installing: fetch, verify, check the new binary, swap. `go test`, `tests/e2e.sh` (555 ok), and a real install from github.com pass.
- Next: AC12's Windows run (Jaime, or once a Windows machine is at hand); then this piece is done, and `update-for-agents` can start.
### 2026-10-08 (claude)
- Did: merged (#52).
- Next: AC12's Windows run (Jaime). AC17 runs against the real 0.4.0 release once it's tagged.
### 2026-10-09 (claude)
- Did: dropped the Windows step here: Jaime revised AC12 to macOS only, and a real update on Windows moved to `2026-10-09-validate-yass-update-on-windows-and-linux`. The piece is done.


## yass/archive/2026/10/2026-10-08-yass-update-the-binary-updates-itself/change.md

---
platforms: [all]
source: Jaime, 2026-10-08: users shouldn't have to download binaries by hand before yass upgrade; teach agents to check and update both
follows: 2026-10-04-the-binary-carries-the-playbooks
blocked:
created: 2026-10-08T12:48:58Z
archived: 2026-10-09T03:02:25Z
---
# yass update: the binary updates itself

## Goal
<!-- Intent. One paragraph; the detail is in prd.md (why and what) and plan.md (how and acceptance). -->
Getting a newer `yass` is one command, and agents can do the whole upgrade. `yass update` downloads and verifies a release (checksums always, signed provenance when the GitHub CLI is there) and replaces the binary in place. Inside a repo it matches the version the repo's YASS files were upgraded to, and says when a newer release exists, how to get it, and that `yass upgrade` and a commit follow. A `yass-update` playbook checks both versions, asks before downloading, and does both steps.


## Decisions
<!-- Progress. "- <decision> - <why> (<who>)" for calls that span the whole change. -->
- Follows `the-binary-carries-the-playbooks`, which left self-update out; this brings it in - getting the binary is where upgrades stall, and agents can't help with it today (Jaime, 2026-10-08)
- `yass update` for the binary, `yass upgrade` for the repo, each pointing at the other in its messages - the conventional pair; the cross-references make mixing them up self-correcting (Jaime, 2026-10-08)
- Verify against the release's `checksums.txt` always, and its signed provenance through `gh attestation verify` when the GitHub CLI is installed, saying which ran; `--require-provenance` makes the second mandatory - provenance inside the binary would pull a large dependency into a small CLI (Jaime, 2026-10-08)
- Inside a repo newer than the binary, install the repo's version, and say when a newer release exists, how to get it (`--latest`), and the `yass upgrade` and commit that follow; otherwise the latest - teammates match the repo instead of jumping ahead of it, and still learn what's out there (Jaime, 2026-10-08)
- `yass update --check` only reports: it exits 0 whenever the check succeeds, update available or not - a non-zero code reads as failure to agents and `set -e` scripts, and nothing needs scripts to act on it yet; an opt-in flag (`--exit-code`, or `--json`) can come later (Jaime, 2026-10-08)
- PRD approved (Jaime, 2026-10-08)
- AC12 and AC17 are verified by hand on macOS only, marked with an asterisk; a real update on Windows and on Linux moves to a follow-up change, `2026-10-09-validate-yass-update-on-windows-and-linux` - Jaime has neither to test on and won't block the change on it. This lowers the bar for both criteria: Windows' swap is covered only by a go test of its strategy, and Linux only by CI's e2e against fake releases (Jaime, 2026-10-09)
- Plan: three pieces (decide, install, point agents at it); release builds carry a `main.channel=release` marker; versions from the `releases/latest` redirect, not the API; swap by rename in the binary's folder (aside on Windows); `YASS_RELEASES_URL` as an undocumented test seam (claude, 2026-10-08)
- Medium scope: `--check`, the version and verification calls above, source builds left alone with the right command, Windows, a `yass-update` playbook and pointers from `yass status`; no mirrors, Homebrew or in-binary Sigstore (Jaime, 2026-10-08)

## Log
<!-- Progress. Change-level notes; each piece keeps its own Log.
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-09 (claude)
- Did: 0.4.0 released (the first tagged run failed in e2e on the tag itself; fixed in #56 and the tag moved, nothing had been published). AC17 on macOS against github.com: the published 0.4.0 binary's `yass update --check` in this repo reports itself up to date, exit 0; a release-marked 0.3.9 build ran `yass update`, installed the latest with checksum and provenance (the real `gh`) verified, and ended up byte-identical to the published binary. This repo upgraded to 0.4.0 with it.
- Next: AC17 on Linux, and AC12 on Windows (Jaime); then archive.
### 2026-10-09 (claude)
- Did: AC12 and AC17 ticked on macOS, per Jaime's revision; every box is done or dropped.
- Next: archive it; Windows and Linux by hand go in `2026-10-09-validate-yass-update-on-windows-and-linux`.


## yass/archive/2026/10/2026-10-08-yass-update-the-binary-updates-itself/design.md

# yass update: the binary updates itself: design
<!-- Intent: the calls that are hard to undo. Edit only in commits without code. -->

## Context
`yass update` replaces an executable on people's machines with one it downloaded, which makes it the most security-sensitive thing YASS does. It also fixes, for every later release, how a binary knows it came from a release, how versions are discovered, and how the swap happens on each platform. The PRD settled the product calls with Jaime on 2026-10-08: `update` beside `upgrade`, checksums always plus provenance through `gh` when present, match the repo's version by default and tell about newer ones, leave source builds alone, a `yass-update` playbook. These are the mechanics.

## 1. Knowing a binary came from a release
**Options.**
- (a) Release builds carry a marker set at build time: GoReleaser adds `-X main.channel=release` beside `main.version`.
- (b) Infer it from the version's shape: `0.3.0` (ldflags, no `v`) is a release, `v0.3.0` is `go install`, a pseudo-version is a clone.
- (c) Infer it from where the binary lives (`$(go env GOPATH)/bin` means `go install`).

**Decision: (a)** (claude, 2026-10-08). It's explicit and survives changes to how versions are written. (b) works today by accident of formatting and would break silently; (c) misses `GOBIN` and copies. Without the marker, `yass update` treats the binary as a source build (R8) and says how to update it: `go install github.com/donjaime/yass/cmd/yass@<version>` when the version is a plain tag, `git pull` and rebuild for a clone build. Release binaries from before this change (0.3.0) can't run `yass update` anyway; the docs say to update those by hand once.

## 2. Finding versions
**Options.**
- (a) The latest version from the redirect of `https://github.com/donjaime/yass/releases/latest` (to `…/releases/tag/vX.Y.Z`); a specific version by fetching its assets directly.
- (b) The GitHub REST API (`/repos/donjaime/yass/releases/latest`).

**Decision: (a)** (claude, 2026-10-08). The API allows 60 unauthenticated requests an hour per IP, which shared CI runners and office networks exhaust; the redirect has no such limit and needs no JSON. The repo's version is the newest stamp among the files `yass status` already reads for its version note (`versionNote` in [`upgrade.go`](../../../internal/yass/upgrade.go)). Pre-releases are never "latest" (GitHub excludes them); `--version v0.5.0-rc.1` installs one on purpose.

## 3. Verifying
Download the platform's archive and `checksums.txt` into a temporary folder; refuse unless the archive's SHA-256 matches its line. Then, when `gh` is on PATH, run `gh attestation verify <archive> --repo donjaime/yass`: the release workflow attests every archive in `checksums.txt`. A failed verification refuses. `gh` missing, or not signed in, isn't a failure: the output says provenance wasn't checked and why, and `--require-provenance` turns that into a refusal (Jaime chose this in the PRD). Only after both does anything outside the temporary folder change.

## 4. Replacing the binary
**Options.**
- (a) Write the new binary to a temporary file in the same folder as the running one (`os.Executable`, symlinks resolved), make it executable, run `<new> version` to check it starts and reports the expected version, then rename it over the old one. On Windows, which can't overwrite a running `.exe`, rename the old one aside to `yass.exe.old` first, move the new one in, and delete any `yass.exe.old` on the next run.
- (b) Download over the binary directly.
- (c) Hand off to `install.sh`.

**Decision: (a)** (claude, 2026-10-08). A rename within a folder is atomic on Unix, so an interrupted update leaves either the old binary or the new one, never half of one (R7). Checking that the new binary runs before the swap catches a wrong-platform download. (b) breaks the binary on any interruption; (c) needs bash, which Windows may not have. If the folder isn't writable, nothing is downloaded: `yass update` checks that first and says to rerun with permission to write there, or to install into a folder that's yours (R9).

## 5. A test seam for the release source
End-to-end tests can't depend on github.com. `YASS_RELEASES_URL` (unset: `https://github.com/donjaime/yass/releases`) points `yass update` at a local server holding fake releases, and `gh` is faked with a script on PATH. It's undocumented for users: supporting mirrors as a feature is a PRD non-goal, with what that implies (docs, compatibility, support for their layouts). (claude, 2026-10-08)

## 6. Teaching agents
**Options.**
- (a) A sixth playbook, `yass-update`, shipped and upgraded like the others.
- (b) Steps inside `yass-status` and `yass-work`.

**Decision: (a), with pointers from (b)** (Jaime chose the playbook in the PRD). Updating touches the machine (a download, a replaced binary) and the repo (an upgrade commit); it needs its own permission rules, which don't belong in a read-only status report or in building a change. `yass upgrade` already adds any playbook the binary carries to a skills folder that has the others, so existing repos get it on their next upgrade. `yass-status` and the `AGENTS.md` section point to it when versions are out of step.

## Consequences and rollback
- **The release workflow gains a line** (`-X main.channel=release`); a release built without it can't update itself, and `yass update` says so.
- **Rolling back an update** isn't `yass update`'s job: it never downgrades (R4). The docs say how to reinstall a specific version by hand (a release download, or `install.sh --bin-dir … --version`).
- **A compromised GitHub release** with matching checksums passes when `gh` isn't installed; that's the trade Jaime made against shipping Sigstore. `--require-provenance` is the answer for anyone who needs more, and the docs say so.


## yass/archive/2026/10/2026-10-08-yass-update-the-binary-updates-itself/plan.md

# yass update: the binary updates itself: plan
<!-- Intent: how, and how we'll know it's done. With code, only mark boxes here ([/] in progress, [x] done);
     anything else is a plan revision in its own commit. -->

## Approach
<!-- The shape of the solution. Hard-to-undo calls go in design.md. -->
**M1 adds `yass update`,** in a new `internal/yass/update.go`, registered in [`main.go`](../../../internal/yass/main.go). It works in three stages, each usable on its own:
1. **Decide.** Read the binary's version and whether it's a release build ([design §1](design.md): `main.channel`, set in [`.goreleaser.yaml`](../../../.goreleaser.yaml) beside `main.version`, and passed to `yass.Main` from [`cmd/yass/main.go`](../../../cmd/yass/main.go)); the repo's version, from the stamps `versionNote` in [`upgrade.go`](../../../internal/yass/upgrade.go) already reads (factored out so both use it); and the latest release, from the `releases/latest` redirect ([§2](design.md)). Pick the target: the repo's version when it's ahead of the binary, else the latest, or `--latest`/`--version`. `--check` stops here and reports.
2. **Fetch and verify** the platform's archive and `checksums.txt` into a temporary folder; the checksum, then `gh attestation verify` when `gh` is there ([§3](design.md)).
3. **Swap** the binary in place ([§4](design.md)), then say what's next (`yass upgrade` where the repo is now behind).

Source builds stop after stage 1 with the command for their install method. The release source is `YASS_RELEASES_URL` for tests only ([§5](design.md)); e2e serves fake releases (archives built from the test binary, a matching `checksums.txt`) from a local `python3 -m http.server`, with a fake `gh` on PATH.

**M2 points people and agents at it.** `versionNote` and `yass upgrade`'s refusal name `yass update`. A new [`kit/.agents/skills/yass-update/SKILL.md`](../../../kit/.agents/skills/) ships like the other playbooks ([§6](design.md)); [`templates/agents.md`](../../../internal/yass/templates/agents.md) and `yass-status` point to it. [`docs/install.md`](../../../docs/install.md), [`docs/skills.md`](../../../docs/skills.md) and [`README.md`](../../../README.md) describe the command and the sixth playbook.

## Acceptance
<!-- Per milestone, one observable behavior each:
### M1
- [ ] AC1 (R1) Given …, when …, then … — verify: <test, flow, or manual steps> -->
### M1
- [x] AC1 (R1) Given a release binary at 0.4.0 in a repo stamped 0.5.0, with 0.6.0 the latest release, when you run `yass update --check`, then it prints the three versions and that `yass update` would install 0.5.0, changes nothing, and exits 0 — verify: e2e (fake releases)
- [x] AC2 (R1) Given the binary already at the latest release, then `--check` says it's up to date and exits 0 — verify: e2e
- [x] AC3 (R2) Given that repo, when you run `yass update`, then the binary is replaced by 0.5.0 (`yass version` says so) — verify: e2e
- [x] AC4 (R3) Given that update, then the output says 0.6.0 exists, gives `yass update --latest`, and says `yass upgrade` in each repo and a `chore: upgrade YASS` commit come after; with no newer release, it says neither — verify: e2e
- [x] AC5 (R4) Given a release binary outside any repo, or in a repo at or behind it, when you run `yass update`, then it installs the latest release — verify: e2e
- [x] AC6 (R4) Given `--latest` in a repo stamped 0.5.0, then it installs 0.6.0; given `--version v0.5.0`, then 0.5.0 — verify: e2e
- [x] AC7 (R4) Given `--version` older than the binary, then it refuses and changes nothing — verify: e2e
- [x] AC8 (R5) Given a release whose archive doesn't match `checksums.txt`, or isn't listed in it, then `yass update` refuses, says so, and the binary is unchanged — verify: e2e
- [x] AC9 (R6) Given `gh` on PATH whose `attestation verify` succeeds, then the output says provenance was verified; given one that fails, it refuses and the binary is unchanged — verify: e2e (fake `gh`)
- [x] AC10 (R6) Given no `gh` on PATH, then it installs after the checksum and says provenance wasn't checked and why; with `--require-provenance`, it refuses — verify: e2e
- [x] AC11 (R7) Given a downloaded binary that doesn't run, or reports a different version, then nothing is replaced — verify: e2e (a release whose binary is a shell script printing the wrong version)
- [x] AC12 (R7) Given the swap, then it's a rename in the binary's folder: no moment leaves the binary missing or partly written; on Windows the old one moves to `yass.exe.old` and is removed on the next run — verify: go test (swap function on a temp folder, both strategies); manual: one real update on macOS. *Windows not run by hand yet: only the go test covers its strategy; a real update there is the follow-up change `2026-10-09-validate-yass-update-on-windows-and-linux`.
- [x] AC13 (R8) Given a binary without the release marker, then `yass update` replaces nothing and prints `go install github.com/donjaime/yass/cmd/yass@<version>` for a tagged build, or how to rebuild a clone build — verify: e2e
- [x] AC14 (R9) Given the binary's folder isn't writable, then it says so before downloading anything, with what to do — verify: e2e (`chmod -w`)
- [x] AC15 (R9) Given the release source unreachable, or `--version` naming a release that doesn't exist, then it says which and changes nothing — verify: e2e (server stopped; a missing version)
- [x] AC16 (R10) Given each platform releases are built for, then `yass update` picks that platform's archive (`.zip` on Windows) — verify: go test (asset name per GOOS/GOARCH)
- [x] AC17 (R1–R10) Given the real 0.4.0-or-later release on GitHub, when a release binary runs `yass update --check` and `yass update`, then both behave as above against github.com — verify: manual: after the next release, on macOS. *Linux not run by hand yet: CI's e2e covers it against fake releases, not github.com; that's the follow-up change `2026-10-09-validate-yass-update-on-windows-and-linux`.

### M2
- [x] AC18 (R11) Given a binary older than the repo, then `yass status`'s note and `yass upgrade`'s refusal name `yass update` — verify: e2e
- [x] AC19 (R12) Given an update that leaves the binary ahead of the current repo, then the output ends with `yass upgrade` for that repo — verify: e2e
- [x] AC20 (R13) Given `yass init`, then `yass-update` is installed with the other playbooks (and in `.claude/skills/` with `--claude`); given an existing repo, `yass upgrade` adds it — verify: e2e
- [x] AC21 (R13) Given the `yass-update` playbook, then it checks both versions with `yass update --check`, asks before downloading, runs `yass update` then `yass upgrade`, shows the diff, and commits only with latitude — verify: manual: review; ask an agent to "update YASS" in a scratch repo with fake releases
- [x] AC22 (R14) Given the `AGENTS.md` section and `yass-status`, then both point to `yass-update` when the binary and the repo are out of step — verify: e2e (section text); manual: review
- [x] AC23 (R15) Given `docs/install.md`, `docs/skills.md` and `README.md`, then they describe `yass update` (what it verifies, how it picks a version, when to use a download, `install.sh` or `go install` instead) and the sixth playbook — verify: manual: review

## Pieces
<!-- PR-sized pieces, each its own folder in here: `yass new "<title>" --in <this change>`.
     `yass status` lists them; say here what order they go in and why. -->
1. **`2026-10-08-update-check`** (AC1, AC2, AC13, AC16, and the release marker). Deciding what to install, with no download: the stage everything else builds on, and useful alone.
2. **`2026-10-08-update-installs`** (AC3–AC12, AC14, AC15). Fetching, verifying and swapping. Needs piece 1.
3. **`2026-10-08-update-for-agents`** (AC18–AC23). Messages, the playbook and docs. Needs piece 2, so nothing points at a command that can't install yet.

AC17 runs after the first release that carries the marker.

## Validation
<!-- How the whole thing is verified before it's called done: suites, platforms, manual passes. -->
- `go test ./...`, `tests/e2e.sh` and `tests/examples.sh` on macOS and Linux in CI.
- A local release dry run, as for archive-that-scales: build two "releases" with the marker, serve them, update from one to the other, then `yass upgrade` a repo.
- AC17 against github.com after the release, on macOS. Windows and Linux by hand are the follow-up change `2026-10-09-validate-yass-update-on-windows-and-linux`.


## yass/archive/2026/10/2026-10-08-yass-update-the-binary-updates-itself/prd.md

# yass update: the binary updates itself: PRD
<!-- Intent: what should change or exist, and why. Not a description of the current system;
     the code and its docs are that. Edit this file only in commits without code. -->

## Why
<!-- The problem, with evidence. -->
Upgrading YASS takes two steps on two levels: get the newer binary (per machine), then run `yass upgrade` (per repo) and commit the diff ([docs/install.md](../../../docs/install.md#upgrading)). The second step is one command; the first is a manual download, checksum check and copy, or a `go install`, or a rebuild from a clone. [`the-binary-carries-the-playbooks`](../../archive/2026/10/2026-10-04-the-binary-carries-the-playbooks/) left self-update out on purpose (a non-goal: "getting the binary stays a release download, the one-liner or `go install`").

That first step is where upgrades stall:
- `yass status` already notices when the binary and a repo are out of step and says which to upgrade, but "upgrade your binary" leaves the user to find the release page and repeat the install by hand.
- A teammate whose repo was upgraded gets that note on every `yass status` until they do. Archive-that-scales makes that sharper: an older binary misreads month folders (`archive/2026` shows as a change).
- Agents are the main users of YASS, but they can't help with the binary today: no playbook covers upgrading, and nothing tells them which version the repo expects or how to get it safely.

## Users and outcomes
<!-- Who it's for, and measurable targets ("p50 time to log < 5s"). -->
- **People using YASS** on their own machines, who installed it from a release.
- **Their agents,** asked to "update YASS" or acting on `yass status`'s version note.
- **Teams,** whose members' binaries need to keep up with what the repo was upgraded to.

Targets:
- From "the repo is newer than your binary" to a working binary at the right version is one command, with no browser and no manual checksum.
- An agent asked to "update YASS" checks both levels, updates what's behind, and hands back a reviewed `chore: upgrade YASS` diff, asking before it downloads anything.
- A binary that can't be verified is never installed.

## Requirements
<!-- One line each: "- **R1** [M1] A user can …". IDs are never reused. -->
- **R1** [M1] `yass update --check` says the binary's version, the version the repo's YASS files are stamped with (inside a repo), the latest release, and what `yass update` would install, and changes nothing. It exits 0 whenever the check itself succeeds, whether or not an update is available.
- **R2** [M1] Inside a repo whose YASS files are newer than the binary, `yass update` installs exactly the repo's version, so the binary matches the repo rather than jumping ahead of it.
- **R3** [M1] When a release newer than the one it installs exists, `yass update` (and `--check`) says so, with the command to get it (`yass update --latest`) and what comes after: `yass upgrade` in each repo, then committing the diff (`chore: upgrade YASS`).
- **R4** [M1] Outside a repo, or when the binary is already at or past the repo's version, `yass update` installs the latest release. `--latest` and `--version <vX.Y.Z>` choose otherwise. It never installs an older version than the binary's.
- **R5** [M1] `yass update` installs nothing that doesn't match its entry in the release's `checksums.txt`.
- **R6** [M1] When the GitHub CLI is installed, `yass update` also verifies the release's signed build provenance (`gh attestation verify`). It says which checks ran, and `--require-provenance` refuses to install without that one.
- **R7** [M1] `yass update` replaces the binary it's running as, in place; if it's interrupted or fails, the old binary still works. This holds on Windows.
- **R8** [M1] A binary built from source (`go install`, or a build from a clone) isn't replaced: `yass update` prints the command that updates it the way it was installed.
- **R9** [M1] When it can't write the binary's folder, can't reach the releases, or the requested version doesn't exist, `yass update` says what went wrong and what to do, and changes nothing.
- **R10** [M1] `yass update` works on every platform releases are built for (macOS and Linux on amd64 and arm64, Windows).
- **R11** [M2] `yass status`'s note for a binary older than the repo, and `yass upgrade`'s refusal for the same, name `yass update`.
- **R12** [M2] After installing, `yass update` says what's next: `yass upgrade` in each repo whose YASS files are now older than the binary.
- **R13** [M2] A `yass-update` playbook, installed and upgraded like the others: for "update YASS" or the version note, it checks both versions (`yass update --check`), asks before downloading anything, runs `yass update`, then `yass upgrade` in the repo, shows the diff, and commits only with the latitude to.
- **R14** [M2] The `AGENTS.md` section and `yass-status` point to `yass-update` when the binary and the repo are out of step.
- **R15** [M2] `docs/install.md` and `README.md` describe `yass update`: what it verifies, how it picks a version, and when to use the release download, `install.sh` or `go install` instead.

## Non-goals
<!-- What this change will not do. Agents treat these as walls. -->
- Updating automatically or in the background. YASS updates only when someone runs the command.
- Checking for new releases on every `yass status` by default (network calls in a command people and hooks run constantly).
- Downgrading.
- Mirrors or another release source (`YASS_RELEASES_URL`) for air-gapped networks; releases come from github.com/donjaime/yass.
- A Homebrew tap or other package managers, and updating binaries they installed.
- Verifying provenance inside the binary (Sigstore libraries); provenance is checked through the GitHub CLI when it's there.

## Milestones
| Milestone | When it ships, a user can … |
|---|---|
| M1 | run `yass update` to get a verified binary matching their repo (or the latest), and learn when a newer one exists and what to do after |
| M2 | ask their agent to "update YASS" and get both levels done, with YASS's messages and docs pointing the way |

## Open questions
- None.


## yass/archive/2026/10/2026-10-10-progress-in-commits-and-pull-requests/change.md

---
platforms: [all]
source: Jaime, 2026-10-10: PR descriptions, titles and commits should give a sense of a change's overall progress; light advice, no imposed process
follows: 
blocked:
created: 2026-10-10T12:09:18Z
archived: 2026-10-10T20:22:19Z
---
# Progress in commits and pull requests

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
Commits and pull requests can say where their change stands as a whole: yass status <change> --brief prints one line with the change's progress (and, for a piece, its parent's), and yass-work suggests it for commit bodies, pull request descriptions and, in short, titles, without imposing a process.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Given a piece, when you run `yass status <piece> --brief`, then it prints one line: its parent's name, pieces done of all, boxes done of all, then the piece's name and its own boxes — verify: e2e
- [x] Given a large change, or a small one, then `--brief` prints the same line without the piece part (a small change: just its boxes), and marks a change that's done — verify: e2e
- [x] Given `--brief` without a change, then it says it needs one — verify: e2e
- [x] Given `yass-work`, then it suggests the line for commit message bodies and pull request descriptions, and a short form for titles, as advice that doesn't impose a process — verify: manual: review
- [x] Given the README's CLI list, then it shows `--brief` — verify: manual: review
- [x] Given this repo's pull requests from this one on, then their descriptions start with the line — verify: manual: the next pull requests

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `yass status <change> --brief`: the change's name without its date, pieces done (for a large change), boxes, and `done`; a piece adds its own after its parent's
- [x] `yass-work`: "Say where the change stands", suggesting the line for commit bodies, pull request descriptions and, in short, titles, as a suggestion rather than a step
- [x] README CLI list; e2e §44 (642 ok)

## Decisions
<!-- Progress. "- <decision> - <why> (<who>, <YYYY-MM-DD>)", appended as you go. -->
- The line counts what YASS knows: pieces done and boxes done (delivered criteria included), not "piece N of M", which depends on the plan's order - so it stays right whatever order pieces land in (claude, 2026-10-10)
- Names drop their date prefix in the line, which keeps it short enough for a commit body; the full folder name is one `yass status <name>` away (claude, 2026-10-10)
- The advice lives in `yass-work`, next to its git paragraph, and says it's a suggestion: Jaime asked for light process advice that doesn't impose a process (Jaime, 2026-10-10)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-10 (claude)
- Did: `--brief`, the advice in `yass-work`, README, e2e. `go test`, `tests/e2e.sh` (642 ok), `tests/examples.sh`, `tests/site.sh` pass.
- Next: Jaime reviews the advice and README (boxes 4, 5); the line opens this pull request and the next ones (box 6).
### 2026-10-10 (claude)
- Did: Jaime reviewed and merged it (#68), whose description, title and commit body used the line; this pull request does too, and so will the next ones. Done.
- Next: archive it.


## yass/archive/2026/10/2026-10-11-install-setup-upgrade-remove/change.md

---
platforms: [all]
source: Jaime, 2026-10-11: before 0.5.0, the install instructions mix the binary with setting up a repo; separate them, spell out upgrading, and add a way to take YASS out of a repo
follows: 2026-10-04-the-binary-carries-the-playbooks
blocked:
created: 2026-10-12T02:51:45Z
archived: 2026-10-12T03:09:24Z
---
# Install, set up, upgrade and remove, each on its own

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
Getting YASS reads like other tools: **install** puts the `yass` binary on your PATH, once per machine, and nothing else; **set up** (`yass init`) adds YASS to a repo; **upgrade** is `yass update` for the binary, then `yass upgrade` in each repo; and **remove** (`yass deinit`) takes YASS's playbooks, `AGENTS.md` section and hook back out of a repo, keeping the `yass/` folder unless you delete it, so trying YASS is low risk. The README, docs/install.md and the landing page each show these as separate steps, in that order.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] AC1: `curl -fsSL …/install.sh | bash`, with no options, installs the binary to `~/.local/bin`, and `--bin-dir` still picks another folder — verify: e2e
- [x] AC2: When the folder isn't on PATH, `install.sh` prints the line that adds it for your shell and doesn't edit shell files; when it is, it says so and prints `yass --version`'s output — verify: e2e (existing checks, kept)
- [x] AC3: Install's one-liner no longer runs `yass init`: the README, docs/install.md and the landing page show install (binary and PATH) first, then "Set up a repo" with `yass init` and its options, as separate steps; the agent prompt names both parts — verify: read the three; tests/site.sh
- [x] AC4: Each of the three says how upgrading works in three steps: `yass update` (the binary), `yass upgrade` in each repo, commit — verify: read them
- [x] AC5: `yass deinit`, run anywhere in a repo, removes every YASS playbook (`yass-*` in any `.agents/skills/` or `.claude/skills/`), the YASS section of every `AGENTS.md` (the file too when nothing else is left in it), the `@AGENTS.md` import YASS added to `CLAUDE.md` when that `AGENTS.md` is gone, the hook scripts in `tools/yass/githooks/`, and `core.hooksPath` when it points there; it lists each thing it removed — verify: e2e
- [x] AC6: `yass deinit` leaves the `yass/` folders, `yass.yaml` and everything else alone, and ends by saying they were kept and how to remove them (`git rm -r yass`) or keep them as a record, then how to commit (`chore: remove YASS`); it never commits — verify: e2e
- [x] AC7: `yass deinit` leaves your user folder's playbooks alone, saying they're there, unless you pass `--global`, which removes them too — verify: e2e
- [x] AC8: `yass deinit` notes any `yass hook` line left in husky, lefthook or pre-commit config, which it doesn't edit — verify: e2e
- [x] AC9: After `yass deinit`, `yass init` sets the repo up again — verify: e2e
- [x] AC10: `yass --help`, the README's CLI list and the 0.5.0 changelog include `yass deinit` and the `install.sh` default — verify: read them

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `install.sh`: `--bin-dir` defaults to `~/.local/bin` (AC1, AC2)
- [x] `yass deinit` and its e2e checks (AC5–AC9)
- [x] README: Install, Set up a repo, Upgrading, Removing YASS; CLI list (AC3, AC4, AC10)
- [x] docs/install.md in the same four parts (AC3, AC4)
- [x] Landing page: install the binary, then set up a repo (AC3, AC4)
- [x] `yass --help`, CHANGELOG (AC10)

## Decisions
<!-- Progress. "- <decision> - <why> (<who>, <YYYY-MM-DD>)", appended as you go. -->
- Named `yass deinit`, the inverse of `yass init` (as in `git submodule deinit`): `uninstall` would suggest the binary, which is install's part, and `remove` could read as removing a change (claude, 2026-10-11)
- `install.sh` defaults `--bin-dir` to `~/.local/bin`, so the one-liner is `curl … | bash` like other tools' (uv puts its binary there too); it still doesn't edit shell files, but the docs make putting the folder on PATH part of the install step (claude, 2026-10-11)
- `yass deinit` doesn't ask before deleting: everything it removes in the repo is in git, so `git restore` undoes it, and the user folder's playbooks go only with `--global` (`yass init --global` writes them again) (claude, 2026-10-11)
- `yass deinit` covers the whole repo, monorepo folders' playbooks and `AGENTS.md` sections included, finding them as `yass upgrade` does, rather than taking a folder (claude, 2026-10-11)
- One small change, not a large one: the docs are a rewrite of what exists and `deinit` mirrors `init` (claude, 2026-10-11)
- `yass deinit` suggests `git rm` for kept plans git tracks and `rm` for the rest, and for a `yass.yaml` pointing outside the repo, the link `yass/` too; the plans folder elsewhere is left to you (claude, 2026-10-11)
- The landing page's install tabs are now ways to get the binary (the agent prompt does both steps), with "Set up a repo" below them and a note on upgrading and `yass deinit` (claude, 2026-10-11)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-11 (claude)
- Did: shaped the change from Jaime's request; goal and acceptance up for review.
- Next: once approved, build it on a branch above this one: install.sh, then `yass deinit`, then the docs.

### 2026-10-11 (claude)
- Did: built it. `install.sh` defaults to `~/.local/bin`; `yass deinit` (e2e section 49, and tried on a clone of this repo and on one with plans outside it); README, docs/install.md and the YASS page rewritten as install (binary and PATH), set up a repo, upgrading, removing YASS; `yass --help`, the CLI list and the 0.5.0 changelog. e2e, go test, examples and site checks pass.
- Next: Jaime reviews the docs and the page; then archive it, and 0.5.0 is ready to tag.

### 2026-10-12 (claude)
- Did: Jaime merged the intent (#82) and the build (#83). Done.


## yass/archive/2026/10/2026-10-11-simplify-before-0-5-0/change.md

---
platforms: [all]
source: Jaime, 2026-10-11: a critical look at the growing complexity before 0.5.0; drop multi-repo plans
follows: 2026-10-04-monorepos-at-scale-and-plans-repos
blocked:
created: 2026-10-11T15:55:12Z
archived: 2026-10-12T02:42:54Z
---
# Simplify before 0.5.0

## Goal
<!-- Intent. One paragraph; the detail is in prd.md (why and what) and plan.md (how and acceptance). -->
0.5.0 ships a simpler YASS than main holds today. Plans kept outside the code repo have one link to the code, `yass:` trailers, and one rule: a commit for a change kept outside its repo names it. Multi-repo plans (`repo@sha`, `repos:`, repo naming) and the trailer modes are gone, `code:` citations are back to 0.4.0's plain note, and the docs and playbooks say only what ships. Smaller by measure, then released.


## Decisions
<!-- Progress. "- <decision> - <why> (<who>, <YYYY-MM-DD>)" for calls that span the whole change. -->
- Simplify before releasing 0.5.0, adding no features until it ships - the separate-plans half of monorepos-at-scale outgrew its purpose as a privacy fallback, and everything since 0.4.0 is still unreleased (Jaime, 2026-10-11)
- Drop shared plans for several code repos - it was the design being thorough, not a need (Jaime, 2026-10-11)
- Trailers are the one link for plans kept elsewhere (written from the code side, they survive squash merges); `code:` citations go back to 0.4.0's plain note (Jaime agreed to claude's proposal, 2026-10-11)
- Trailers follow one rule tied to where the change lives, not a repo-wide mode, and only CI requires them (`--require-trailers`), so a repo with both kinds of plans needs no special case (Jaime agreed to claude's proposal, 2026-10-11)
- `yass archive` stops refusing a change over its citations, but warns about missing or unmerged ones as it archives (Jaime, 2026-10-11)
- PRD approved (Jaime, 2026-10-11)
- Plan: no pieces, three pull requests, one per milestone; no design.md, since removing unreleased surface is easy to undo from git (claude, 2026-10-11)
- Measure before and after, so "simpler" can be checked (claude, 2026-10-11)
- A change of its own, following monorepos-at-scale, rather than a revision of it: the archive keeps what was built, this records what was removed and why (claude, 2026-10-11)
- The trailer hooks check, cheaply, whether the repo uses any plans folder outside it before reading a single change, so repos with inline plans pay nothing per commit (claude, 2026-10-11)
- `yass: none` counts as kept outside for the prepare hook, so a `yass use none` still writes it - it's a deliberate trailer, not an inference (claude, 2026-10-11)
- The cut is modest by measure (non-test Go −3%, docs −2%, playbooks −0.2%) because most of what grew since 0.3.0 is earning its keep and stays; the gain is in concepts: three settings, a repo-wide mode, a second way to link code and a mixed-repo special case are gone (claude, 2026-10-11)

## Log
<!-- Progress. Change-level notes; each piece keeps its own Log.
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-11 (claude)
- Did: M1. `codeRefs` reads plain hashes only; `checkCode` checks this repo only, as in 0.4.0; repo naming, `repos:` and their checks are gone; `yass archive` warns about missing or unmerged citations instead of refusing; `repo` and `repos` settings removed. Tests: `TestCodeRefs` updated, `TestRepoFromURL` and e2e §47 removed, §43 rewritten (686 ok).
- Next: M2, the one rule for trailers.
### 2026-10-11 (claude)
- Did: M2. Trailers follow one rule: the prepare hook adds one when the change it works out lives in a folder outside the repo (`targetSeparate`), mixed repos included; the commit-msg hook only points out malformed trailers or ones naming no change, and never refuses; `yass hook --range` checks the trailers commits carry, `--require-trailers` adds missing ones, `--strict` fails; `trailerMode`, `plansSeparate` and the `trailers:` setting are gone (`usesSeparatePlans` is the cheap gate). e2e §45 rewritten, with a repo holding both kinds of plans (689 ok).
- Next: M3: docs, playbooks and help say only what ships; sizes; the 0.5.0 changelog.
### 2026-10-11 (claude)
- Did: M3. `docs/separate-plans.md` rewritten for one link and one rule (no `trailers:`, no `repos:`); README (trailer bullet, multi-repo bullet gone, settings list), `yass-work` and `docs/skills.md` lose `repo@sha`; a 0.5.0 section in `CHANGELOG.md`. Grep finds none of `repo@sha`, `repo:`, `repos:` (other than pre-commit's own config key), `trailers:` or refusing commits.
- Sizes, 2026-10-11 main → now (commands: `ls internal/yass/*.go | grep -v _test | xargs cat cmd/yass/main.go | wc -l`; `cat internal/yass/*_test.go | wc -l`; `cat kit/.agents/skills/*/SKILL.md | wc -w`; `cat docs/*.md | wc -w`; `wc -w README.md`; the `yaml:` tags in config.go): non-test Go 6,097 → 5,913; tests 785 → 766; playbook words 5,085 → 5,076; docs words 7,670 → 7,490; README words 4,932 → 4,822; settings 8 → 5 (path, branch, ignore, archive with keep).
- Checks: `go test`, `tests/e2e.sh` (689 ok), `tests/examples.sh`, `tests/site.sh`, `go vet`, `tests/bench.sh`: status 915ms, 832ms with 100 branches, 1,439ms with 1,000 more, trailers 76ms and 221ms, hook 109ms; every target met.
- Next: Jaime reviews the docs and the changelog (AC14, AC16); then archive this change and release 0.5.0.
### 2026-10-11 (claude)
- Did: Jaime reviewed the docs and changelog and merged them (#80). Done.
- Next: archive it; then 0.5.0.


## yass/archive/2026/10/2026-10-11-simplify-before-0-5-0/plan.md

# Simplify before 0.5.0: plan
<!-- Intent: how, and how we'll know it's done. With code, only mark boxes here ([/] in progress, [x] done);
     anything else is a plan revision in its own commit. -->

## Approach
<!-- The shape of the solution. Hard-to-undo calls go in design.md. -->
Mostly deletion, in the order the code depends on itself. No design.md: the calls are in the PRD and `change.md`, and removing unreleased surface is easy to undo from git if it's ever wanted back.

**M1, one link in one repo.** In [`text.go`](../../../internal/yass/text.go), `codeRefs` goes back to plain shas (a `repo@sha` token isn't read). In [`repo.go`](../../../internal/yass/repo.go), `checkCode` checks citations against this repo only, as in 0.4.0; `repoName`, `repoFromURL`, `reposOf`, `codeRepos`, `citationIn` and the "left to another repo" notes go. `archiveCitations` becomes warnings that `yass archive` prints as it archives. [`config.go`](../../../internal/yass/config.go) loses `repo` and `repos`. `yass decisions --cites` matches plain shas, as before #67.

**M2, one rule for trailers.** In [`trailers.go`](../../../internal/yass/trailers.go), `trailerMode` and `plansSeparate` go, and with them `trailers:`. The prepare hook adds a trailer when the change it works out lives in a yass folder `separate()` says is outside this repo. The commit-msg hook reports only malformed trailers or ones naming no change, and always exits 0. `yass hook --range` checks the trailers commits carry, adds `--require-trailers` for commits without one, and fails with `--strict`. `yass status <change>` lists code from trailers for a change in a separate folder. `yass use` says when a change's plans are in this repo, so its commits won't carry a trailer.

**M3, say only what ships.** README, [`docs/separate-plans.md`](../../../docs/separate-plans.md), `docs/monorepo.md`, `docs/skills.md`, `yass-work` and the help text drop what's gone and describe the one link and the one rule. The Log records sizes before and after. `CHANGELOG.md` gets a 0.5.0 section.

The e2e sections for multi-repo citations and `repos:` are removed; the trailer section is rewritten for the one rule, including a repo with both kinds of plans.

## Acceptance
<!-- Per milestone, one observable behavior each:
### M1
- [ ] AC1 (R1) Given …, when …, then … — verify: <test, flow, or manual steps> -->
### M1
- [x] AC1 (R1) Given a done box citing a commit that isn't in this repo, or isn't merged, when you run `yass status`, then it warns as 0.4.0 did; a merged citation draws nothing — verify: e2e (the existing citation checks)
- [x] AC2 (R2) Given a box ending `— code: web@a1b2c3d`, then YASS reads no citation from it: no check, no warning, and `yass decisions --cites a1b2c3d` doesn't list it — verify: go test (`codeRefs`); e2e
- [x] AC3 (R3) Given a `yass.yaml` with `repo:` or `repos:`, when you run `yass status`, then it warns that the setting is unknown, and nothing else uses it — verify: e2e
- [x] AC4 (R4) Given a finished change whose done box cites a missing or unmerged commit, when you run `yass archive`, then it archives it and warns, naming the box and the commit — verify: e2e

### M2
- [x] AC5 (R5) Given a repo with an inline `yass/` and a separate plans folder, and `yass use` naming a change in the separate one, when you commit, then the commit gets `yass: <change>` — verify: e2e
- [x] AC6 (R5) Given `yass use` naming a change in the inline `yass/`, when you commit, then no trailer is added, and `yass use` said so when it was set — verify: e2e
- [x] AC7 (R5) Given a branch named after a change in a separate folder, or an earlier commit's trailer on the branch, then the trailer comes from them, as now — verify: e2e
- [x] AC8 (R6) Given a commit without a trailer, then the commit-msg hook says nothing and the commit goes through, whatever the repo holds — verify: e2e
- [x] AC9 (R6) Given a malformed trailer, or one naming no change, then the commit goes through with a heads-up naming it and the closest change names — verify: e2e
- [x] AC10 (R7) Given `yass hook --range` over commits with a bad trailer and commits without one, then it reports the bad one; with `--require-trailers` it also reports the ones without; with `--strict` either exits 1 — verify: e2e
- [x] AC11 (R7) Given `--range` where the plans aren't checked out, then it checks only the trailers' form and says so, as now — verify: e2e
- [x] AC12 (R8) Given a `yass.yaml` with `trailers:`, then it warns that the setting is unknown — verify: e2e
- [x] AC13 (R9) Given commits naming a change in a separate folder with `ac:`, then `yass status <change>` lists them and each criterion's, as now — verify: e2e (the existing section)

### M3
- [x] AC14 (R10) Given the README, `docs/`, the playbooks and `yass --help`, then none mentions `repo@sha`, `repo:`, `repos:`, `trailers:` or refusing commits, and `docs/separate-plans.md` describes one link and one rule — verify: `grep` for each; manual: review
- [x] AC15 (R11) Given the change's Log, then it records non-test Go lines, test lines, playbook words, docs words and settings at 2026-10-11 main and after; non-test Go, playbook words and docs words are each smaller, and settings are 5 — verify: manual: the numbers, from the commands in the Log
- [x] AC16 (R12) Given `CHANGELOG.md`, then a 0.5.0 section describes what ships since 0.4.0 and says nothing of what was removed before release — verify: manual: review
- [x] AC17 (all) Given the change done, then `go test`, `tests/e2e.sh`, `tests/examples.sh`, `tests/site.sh` and `tests/bench.sh` pass, every bench target met — verify: run them; record the bench numbers in the Log

## Pieces
<!-- PR-sized pieces, each its own folder in here: `yass new "<title>" --in <this change>`.
     `yass status` lists them; say here what order they go in and why. -->
None: three pull requests, one per milestone, in order (M2 rewrites code M1 shrinks first; M3 describes both). Each is small and mostly removes code.

## Validation
<!-- How the whole thing is verified before it's called done: suites, platforms, manual passes. -->
- Every suite passes after each pull request; `tests/bench.sh` before M3 merges.
- The sizes before and after (AC15).
- A read of the docs and playbooks against the CLI's help, for anything removed that's still described.


## yass/archive/2026/10/2026-10-11-simplify-before-0-5-0/prd.md

# Simplify before 0.5.0: PRD
<!-- Intent: what should change or exist, and why. Not a description of the current system;
     the code and its docs are that. Edit this file only in commits without code. -->

## Why
<!-- The problem, with evidence. -->
YASS calls itself a thin, file-based layer with a small CLI. In the six days from 0.3.0 to 2026-10-11, its non-test Go went from 2,893 lines to 6,097, its playbooks from 3,751 words to 5,085, its docs from 4,684 words to 7,670, and `yass.yaml` grew to 8 settings. Most of that is earning its keep (archives by month, `yass decisions`, delivered criteria, the branch view), but the half of [`monorepos-at-scale-and-plans-repos`](../2026-10-04-monorepos-at-scale-and-plans-repos/) for plans kept outside the code repo has outgrown its purpose:

- **Its PRD called separate plans the privacy fallback,** yet it took 5 of 11 pieces and most of the new concepts: inline, separate and mixed repos, repo naming, a map of code checkouts, three trailer modes, per-worktree state.
- **There are two ways to link code to plans:** `code:` citations (now with `repo@sha`, `repos:`, merged checks and an archive refusal) and `yass:` trailers (with `ac:`, `yass use`, three ways to infer, two hooks, three modes, range checks). Each has its own settings, checks and edge cases.
- **The rules hang on a repo-level mode** (trailers apply only when every yass folder is separate), so a repo with both inline and separate plans gets no trailers even for a change kept outside it. The AC56 agent run hit exactly this.
- **Bugs now come from interactions, not features:** hooks in linked worktrees reading the wrong repo, new hooks blocking commits under an older `yass`, a per-folder git lookup costing 2.3s, a benchmark silently broken for six pull requests.

Everything after 0.4.0 is unreleased, so it can still be simplified without breaking anyone. After 0.5.0 ships, it can't.

## Users and outcomes
<!-- Who it's for, and measurable targets ("p50 time to log < 5s"). -->
- **People keeping plans outside their code repo** (the privacy case): one way to link code, one rule for when commits name their change.
- **Everyone else:** a smaller YASS to learn, read and trust, and fewer interactions to break.

Targets, against main on 2026-10-11:
- `yass.yaml` settings drop from 8 to 5 (`repo`, `repos` and `trailers` go).
- Non-test Go (6,097 lines), playbook words (5,085) and docs words (7,670) are each smaller.
- A code commit for a change kept outside its repo gets its trailer, whatever else that repo holds.
- `tests/e2e.sh`, `go test` and `tests/bench.sh` pass, with every bench target met.

## Requirements
<!-- One line each: "- **R1** [M1] A user can …". IDs are never reused. -->
- **R1** [M1] A `code: <sha>` citation is an optional note on a box, as in 0.4.0: `yass status` checks that the commit exists in the repo it runs in and, on a done box, that it's merged there. Nothing more.
- **R2** [M1] `repo@sha` isn't a citation form: YASS doesn't read it.
- **R3** [M1] The `repo:` and `repos:` settings are gone, and with them naming code repos and checking citations from a plans repo. A `yass.yaml` that still has them gets the usual unknown-setting warning.
- **R4** [M1] `yass archive` doesn't refuse a change over its citations; it warns about a done box citing a commit that's missing here or not merged, and archives it.
- **R5** [M2] The `prepare-commit-msg` hook adds `yass: <change>` whenever the change it works out (from `yass use`, the branch name, or an earlier commit) lives in a yass folder outside this repo, whether or not the repo also has inline plans.
- **R6** [M2] The `commit-msg` hook reports only a malformed trailer, or one naming no change. It never asks for a missing trailer and never refuses a commit.
- **R7** [M2] `yass hook --range` checks the trailers commits carry. `--require-trailers` also reports commits without one, and `--strict` makes any report fail.
- **R8** [M2] The `trailers:` setting and the repo-wide "every folder is separate" mode are gone. `yass root -v` and `yass status`'s `plans: separate` line stay.
- **R9** [M2] `yass use`, `ac:`, and `yass status <change>` listing a change's code and criteria from trailers work as they do now.
- **R10** [M3] `docs/separate-plans.md`, the README, `yass-work` and `docs/skills.md` describe one link and one rule, and nothing that's been removed.
- **R11** [M3] The change's Log records the sizes before and after: non-test Go, tests, playbook words, docs words, settings.
- **R12** [M3] `CHANGELOG.md` has a 0.5.0 section describing what ships, not what was built and removed in between.

## Non-goals
<!-- What this change will not do. Agents treat these as walls. -->
- New features, until 0.5.0 ships.
- Changing what 0.4.0 shipped: citations as they were, archives by month, `keep` and `yass evict`, `yass decisions`, `yass update`.
- Revisiting what's earning its keep: the branch view, delivered criteria, pieces owning their files, sparse-checkout notes, `--brief`.
- Shared plans for several code repos, in any form.
- Changing YASS's process rules (how intent, progress and archives are committed); that's a conversation of its own.
- Windows and Linux validation of `yass update` (its own change).

## Milestones
| Milestone | When it ships, a user can … |
|---|---|
| M1 | link code to plans one way in their own repo, with no repo naming or checkout maps to learn |
| M2 | keep plans outside the repo and have every commit for such a change name it, with one rule and no modes |
| M3 | read docs and playbooks that match, and install 0.5.0 |

## Open questions
- None.


## yass/changes/2026-10-04-validate-the-ci-recipes-in-the-field/change.md

---
platforms: [all]
source: yass/changes/2026-10-04-monorepos-at-scale-and-plans-repos (AC4, AC5)
follows: 
blocked:
---
# Validate the CI recipes in the field

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
The CI recipes in `docs/monorepo.md` that couldn't be tested when they were written have been run for real: the Bazel recipe on a Bazel repo, and the GitHub gate job through a real merge queue. The docs say which recipes are tested and where, and invite reports for other CI systems and build tools. Split off from `2026-10-04-monorepos-at-scale-and-plans-repos` (its AC4 and AC5), so that change can finish without waiting on a test setup.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [ ] Given a small Bazel repo with a yass folder set up as the docs say (`.bazelignore`, no `glob(["**"])` pulling plans into a target), when an affected-targets tool (bazel-diff or target-determinator) compares a plan-only commit, then it finds no affected targets, and for a code commit it finds the code's target — verify: a script or fixture that can be rerun (Bazelisk installed), with its output in the Log
- [ ] Given a GitHub repo owned by an organization, with a merge queue on its main branch and the documented gate job, when a plan-only pull request goes through the queue, then `ci-ok` passes without the test jobs running, and a code pull request runs them — verify: manual: record both runs' links in the Log, and confirm the `merge_group.base_sha` field the recipe uses
- [ ] Given those runs, then the Bazel and merge-queue sections of `docs/monorepo.md` no longer say they're untested, and any fixes they needed are in — verify: manual: review
- [ ] Given `docs/monorepo.md`, then it says where to report results for other CI systems and build tools (Buck, Pants, Nx, Buildkite, GitLab CI), and the recipes people report as working are listed with who tested them — verify: manual: review

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [ ] 

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->


## yass/changes/2026-10-09-validate-yass-update-on-windows-and-linux/change.md

---
platforms: [all]
source: Jaime, 2026-10-09: AC12 and AC17 of yass-update were verified on macOS only; neither Windows nor Linux was at hand
follows: 2026-10-08-yass-update-the-binary-updates-itself
blocked:
created: 2026-10-09T03:02:25Z
---
# Validate yass update on Windows and Linux

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
A real yass update runs on Windows and on Linux against the published GitHub releases, the way AC12 and AC17 of the archived yass-update change were verified on macOS, so all three platforms releases are built for have been checked by hand.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [ ] Given a published release of 0.4.0 or later installed from its `.zip` on Windows, and a newer release published, when you run `yass update`, then it installs the newer one with its checksum (and provenance, with `gh` signed in) verified, `yass version` reports it, the old binary sits beside it as `yass.exe.old`, and the next `yass` command removes that — verify: manual, on Windows (amd64 or arm64)
- [ ] Given a published release installed on Linux, when you run `yass update --check` and then `yass update` against github.com, then they behave as AC17 of the archived change did on macOS: the right versions, verification reported, the binary swapped and identical to the published one — verify: manual, or a CI job on a Linux runner

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [ ] Easiest once a release after 0.4.0 is out: install 0.4.0 from its release download, then run `yass update`

## Decisions
<!-- Progress. "- <decision> - <why> (<who>, <YYYY-MM-DD>)", appended as you go. -->

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->


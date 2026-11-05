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

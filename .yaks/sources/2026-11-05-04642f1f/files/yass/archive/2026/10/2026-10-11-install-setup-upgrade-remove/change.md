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

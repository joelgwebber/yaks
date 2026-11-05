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


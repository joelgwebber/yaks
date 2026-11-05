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

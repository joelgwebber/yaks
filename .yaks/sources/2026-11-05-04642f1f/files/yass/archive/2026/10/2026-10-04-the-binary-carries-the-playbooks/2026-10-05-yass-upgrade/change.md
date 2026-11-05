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

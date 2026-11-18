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

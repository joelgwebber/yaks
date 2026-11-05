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

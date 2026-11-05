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

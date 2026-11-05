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

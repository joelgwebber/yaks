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

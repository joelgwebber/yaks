---
platforms: [all]
source: Jaime, 2026-10-07: conversation about the archive growing without bound on large teams
follows: 
blocked:
created: 2026-10-07T16:51:19Z
archived: 2026-10-08T22:15:40Z
---
# Archive that scales

## Goal
<!-- Intent. One paragraph; the detail is in prd.md (why and what) and plan.md (how and acceptance). -->
The archive stays manageable however big a team gets. Archived changes are dated and kept in month folders that GitHub can browse. The CLI reads only their names unless it needs more. Agents ask `yass decisions` for past decisions and get bounded answers instead of grepping everything. Each yass folder has a soft limit on archived changes in its working tree (2,000 by default); past it, YASS says so, and `yass evict` moves the oldest months out to git history, with a small manifest saying where to find them, so a query reaches into history only when it has to.


## Decisions
<!-- Progress. "- <decision> - <why> (<who>)" for calls that span the whole change. -->
- The archive is read mostly by YASS's tooling and by AI explaining decisions; the concern is bloat in clones and on GitHub - this sets what the change optimizes for (Jaime)
- Existing flat archives are migrated, this repo's included - migrating our own archive tests and validates the migration (Jaime)
- Scale target is solo to 100+ person teams, solved per yass folder so it nests and ladders up; past that, git and GitHub are the limit - YASS shouldn't build for the rare case where git itself breaks down (Jaime)
- Old archived changes may leave the working tree for git history, governed by optional `yass.yaml` settings whose defaults are high enough that most teams never set them; git history is read only when a query needs it - crawling history on every query would slow the CLI (Jaime)
- `yass archive` stamps `archived:` in the frontmatter, and the archive is organized by archive date, not by the date in the change's name - folders fill in order and old months never receive late arrivals, so they can be evicted safely; finding a change by name means listing month folders, which is cheap (Jaime, refined with claude)
- Eviction is by count (`keep`), not by age - it exists to control size, not freshness (Jaime)
- Eviction happens inside `yass archive`, in the archive commit, and removes whole months - evicting one change per archive would rewrite a manifest on every archive and conflict constantly on busy teams; whole months make eviction rare and the manifests write-once (Jaime, months from claude)
- Each evicted month's manifest records the commit that last touched the month's folder - the same on any branch, so concurrent evictions merge cleanly; and old enough to be on the main branch, so squash merges don't lose it (claude)
- `keep` counts top-level changes, default 10,000 - people can reason about a count of changes, not of files; 10,000 is about a year at a 100-person team's pace (claude, agreed by Jaime)
- A yass folder archiving more than about 1,000 changes a month should split into team folders; month folders aren't split further, and an occasional big month is fine (Jaime)
- Migration runs in `yass upgrade`, as the first step of a small, general migration mechanism whose steps can be dropped later, not something carried forever (Jaime)
- Decision entries carry their own date; `yass decisions` surfaces it alongside the change's `created:` and `archived:` dates (Jaime)
- `yass decisions` has `--json`, and `--change` follows the `follows:` chain both ways - JSON gives tooling a stable contract while the plain text stays free to change; "why is X like this" usually spans several changes (claude, at Jaime's request)
- Replaces the call to evict inside `yass archive`: `keep` is a soft limit, `status` and `archive` say when it's passed, and eviction is its own command, `yass evict`, in its own commit, still by whole months - archive commits stay single-purpose, and eviction becomes a deliberate act by one person rather than something any archive on any branch can trigger; whole months keep each manifest written once (Jaime, whole months kept by claude)
- PRD approved for planning, with a scheduled CI recipe for `yass evict` added to R20 (Jaime, 2026-10-07)
- Plan: six pieces, the month layout first; formats and hook rules pinned in design.md (archive date and nesting, `archived:` stamp, `.evicted` manifest, soft `keep` with `yass evict`, migration steps, dated entries as `(<who>, <YYYY-MM-DD>)`) (claude, 2026-10-07)
- Rule 5's eviction wording moves from AC14 (piece 1) to a new AC48 (piece 5) - piece 1 would otherwise tell users about `yass evict` releases before it exists; same bar, later piece (claude, 2026-10-07)
- AC13 runs the migration on this repo and the private plans folder with a local build of `yass upgrade`, at the version the files are stamped with, instead of waiting for a release - same code, checked sooner, and the build writes no stamp for a version that doesn't exist (Jaime, 2026-10-07)
- `yass-log` keeps consulting the archive, but through `yass`: a `--cites <sha>` search joins piece 6 (R25, AC49, AC50), and AC29 moves there with it - once months can be evicted, a grep of the working tree would miss evicted changes and report a commit as unlinked; one way into the archive keeps layout changes out of the playbooks (Jaime, 2026-10-07)
- The default `keep` drops from 10,000 to 2,000 - measured, each archived change adds about 45µs to every `yass status` (git walks its folders: +170ms at 2,000, +460ms at 10,000) and about 10KB to every checkout; `keep` is per yass folder, so a monorepo's folders add up. 2,000 keeps about a year for a 10-person team, and `yass decisions` reaches evicted months (piece 6) (Jaime, 2026-10-08)
- One large change with five milestones - each milestone helps on its own, so eviction can wait if nobody needs it yet (Jaime)

## Log
<!-- Progress. Change-level notes; each piece keeps its own Log.
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->

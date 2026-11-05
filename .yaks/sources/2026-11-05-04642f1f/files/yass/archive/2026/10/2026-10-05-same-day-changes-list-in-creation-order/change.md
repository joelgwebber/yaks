---
platforms: [all]
source: 
follows: 
blocked:
archived: 2026-10-05T19:13:17Z
---
# Same-day changes list in creation order

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
Changes made on the same day list in the order they were created, so "what's next?" has a tie-break when `queue.md` doesn't settle it. `yass new` stamps `created:` (UTC, to the second) in a change's frontmatter, and listings order by it. Folder names don't change, and nothing is migrated: changes without the stamp (the archive, older or hand-made changes) fall back to their date prefix.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Given `yass new` (or `yass new --in`), then the new change.md has `created: <UTC RFC 3339 time>` in its frontmatter and the folder is still `<YYYY-MM-DD>-<slug>` — verify: e2e
- [x] Given two changes made the same day whose slugs sort the other way, and no `queue.md` entry for either, when you run `yass status`, then they list in creation order — verify: go test; e2e
- [x] Given pieces of a large change, or archived changes with stamps, then `yass status` lists them in creation order too — verify: go test
- [x] Given a change without `created:`, then it sorts by its date prefix, before stamped changes from the same day, and still lists and resolves as before — verify: go test; e2e
- [x] Given a `created:` that isn't an RFC 3339 time, then `yass status` warns and sorts that change as if it had no stamp — verify: e2e

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] Templates and `yass new` stamp `created:`
- [x] Load orders changes and pieces by (created, name); warn on a bad stamp
- [x] README documents `created:`
- [x] Go tests and e2e

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- Order by a `created:` stamp in frontmatter, not by renaming folders - a counter (SEP style) collides across parallel branches, worktrees and plans repos, and renumbering breaks `follows:`, `blocked:`, `queue.md` and Log references; a time in the name costs too many characters (Jaime, claude)
- No migration - the archive is append-only, and unstamped changes sort well enough by date; queue.md already says what order matters (Jaime, claude)
- Order once, in load, for changes and pieces alike - every listing (status, --archived, pieces) agrees without each sorting on its own (claude)
- The yass folder READMEs and CHANGELOG stay as they are - "oldest first" was the promise and is now true; recent changes leave the CHANGELOG to release time (claude)
- UTC to the second - teammates in different zones still order correctly, and agents can make several changes in a minute (claude)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-05 (claude)
- Did: `yass new` stamps `created:` (UTC RFC 3339) via the change templates; `load` orders changes and pieces by `created()` (the stamp, else the folder's date), and `check` warns on a stamp that isn't a time. TestByCreated; e2e §29; README frontmatter. Checks: go test, go vet, gofmt, e2e 386/386.
- Next: archive.

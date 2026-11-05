# Archive that scales: design
<!-- Intent: the calls that are hard to undo. Edit only in commits without code. -->

## Context
This change sets formats that end up in every team's committed files and that tools and agents will read for years: where archived changes live, a new frontmatter field, a per-month eviction manifest, a dated decision entry, a new command, and how `yass upgrade` migrates folders. Once a team has archived, migrated or evicted under one of these, changing it means another migration. The PRD fixed the big calls with Jaime on 2026-10-07: archive by archive date, an `archived:` stamp, migrate existing archives, eviction by count as a soft limit with its own command, dated decision entries, a light migration mechanism in `yass upgrade`. Each is pinned down here.

## 1. Where archived changes live
**Options.**
- (a) `archive/<YYYY>/<MM>/<name>/`, by the `archived:` date.
- (b) The same layout, by the date in the change's name.
- (c) `archive/<YYYY-MM>/<name>/`, one level.
- (d) Flat, as today, with an index file.

**Decision: (a)** (Jaime chose the archive date, claude the nesting, 2026-10-07). By archive date, a month only receives changes while it's the current month, so a past month never changes again and can be evicted safely; (b) would let a change started in January land in January when it's archived in October, possibly after January was evicted. Nesting by year keeps `archive/` itself to a handful of entries, where (c) adds twelve a year. (d) leaves the folder unbounded on GitHub, and a committed index conflicts on busy teams (a PRD non-goal). The month comes from `archived:` in UTC. A year folder is four digits and a month folder two, so neither can be mistaken for a change, whose name starts `YYYY-MM-DD-`.

Finding an archived change by name means listing month folders' names, not reading them. A name is still unique per yass folder: `yass archive` refuses a name that's already archived in any month.

## 2. The `archived:` stamp
**Options.**
- (a) `archived: <RFC 3339 UTC>` in the top-level `change.md` frontmatter, written by `yass archive` in the archive commit.
- (b) No stamp; read the date from git history when needed.
- (c) No stamp; the path's month is enough.

**Decision: (a)** (Jaime, 2026-10-07). It's the same shape as `created:`, works outside git and in shallow clones, survives copying a folder, and gives `yass decisions` an exact date. (b) costs a git call per change, which is what M2 exists to avoid; (c) loses the day. It's written as part of the move, like `yass archive` already marks delivered criteria (stacked-prs-and-squash-merges), so the archived change is never edited after it lands. Pieces aren't stamped; they're archived with their change and share its date.

## 3. The eviction manifest
**Options.**
- (a) One file per evicted month, `archive/<YYYY>/<MM>.evicted`, written once.
- (b) One `archive/evicted.yaml` for the whole folder, appended to.
- (c) A stub file per evicted change.

**Decision: (a)** (claude, 2026-10-07, following Jaime's whole-month eviction). (b) is edited by every eviction and conflicts; (c) leaves one file per change behind, which is what eviction exists to remove. The file is YAML:

```yaml
# Evicted by yass evict. Read a change back: git show <commit>:<path>/<name>/change.md
commit: 3f9c2e1a7b…            # full hash of the last commit that touched the month's folder
path: yass/archive/2026/03     # the month's folder at that commit, relative to the top of its git repo
changes:                       # top-level changes, by name, sorted
  - 2026-03-02-some-change
  - 2026-03-04-another-change
```

`commit` is `git log -1 --format=%H -- <month folder>` on the branch being evicted, so two branches evicting the same month from the same history write byte-identical files and git merges them cleanly. A month is old enough to evict only after it has stopped changing, so its last commit is on the main branch and survives squash merges. `path` is recorded because a yass folder can move or live in another repo (`yass.yaml` `path:`). Pieces aren't listed; a piece resolves through its change.

## 4. Eviction: when and how much
**Options.**
- (a) A soft `keep` limit; `yass status` and `yass archive` say when it's passed; `yass evict` removes oldest whole months, in its own commit.
- (b) `yass archive` evicts as needed, in the archive commit.
- (c) A hard limit the hook enforces.

**Decision: (a)** (Jaime, 2026-10-07; whole months from claude). Archive commits stay single-purpose, and eviction is a deliberate act by one person instead of a side effect any archive on any branch can trigger. (b) rewrites a manifest on nearly every archive at the limit, and (c) blocks commits over housekeeping. `yass evict` removes the oldest months until the count is at or under `keep`, never the current month, and only months with no uncommitted changes. It needs git in the yass folder: outside git, eviction would destroy data, so it refuses. `keep` counts top-level changes in the working tree, default 2,000 (Jaime, 2026-10-08, after measuring: see change.md Decisions).

## 5. How `yass upgrade` migrates
**Options.**
- (a) A list of small steps, each deciding from the files whether it's needed, and doing nothing when it isn't.
- (b) A format version stamped in each yass folder, with migrations from version to version.

**Decision: (a)** (Jaime, 2026-10-07). A step can be deleted in a later release without touching the others or any stamp, which is the point: migrations shouldn't be carried forever (a PRD non-goal rules out (b)). A step names itself, says what it would change, makes the change in the working tree, and prints how to commit it on its own. Steps run per yass folder, including ones outside the repo, after `yass upgrade` writes YASS's files. The first step moves a flat archive into months: `archived:` comes from the date of the commit that added the change's `change.md` under `archive/` in that folder's git repo, else the date in its name.

## 6. Dated decision entries
**Options.**
- (a) The date joins the trailing credit: `- <decision> - <why> (<who>, <YYYY-MM-DD>)`.
- (b) A leading date: `- 2026-10-07: <decision> - <why> (<who>)`.
- (c) No date on entries; use the change's dates, or `git blame`.

**Decision: (a)** (Jaime chose dated entries, claude the form, 2026-10-07). It extends the format in use without moving anything, and reads the same with or without a date, so entries written before this stay valid. (b) reorders every existing entry's shape; (c) can't tell decisions made months apart in one long change, and blame is slow and wrong after edits. `yass decisions` reads the last parenthesis on the entry's first line: text before the last comma is who, a `YYYY-MM-DD` after it is the date. An entry without one has no date of its own.

## 7. What the hook allows in the archive
The archive stays append-only, with exactly three ways in or out, each in a commit without code (claude, 2026-10-07):
- **Archiving:** a change moved from `changes/` to `archive/<YYYY>/<MM>/<name>/`; its content may differ by the `archived:` stamp and boxes `yass archive` marks.
- **Migrating:** a change moved from `archive/<name>/` to `archive/<YYYY>/<MM>/<name>/`, differing only by the `archived:` stamp.
- **Evicting:** an archived change deleted, in a commit that adds its month's `.evicted` file listing it.

Anything else under `archive/` is flagged as today.

## Consequences and rollback
- **Older binaries.** A yass from before this change reads `archive/2026` as a change named `2026`, and its hook flags new archives as edits. The upgrade that migrates also restamps YASS's files, so older binaries print their existing "written by a newer yass; upgrade your binary" note. The docs say to upgrade everyone's binary before running `yass upgrade` on a shared repo.
- **Rolling back the layout** is another migration step moving months back to flat; the `archived:` stamp can stay, since it's harmless.
- **Rolling back an eviction** is `git revert` of its commit. Nothing is lost while history is.
- **A team that wants no eviction** sets a very high `keep`, or never runs `yass evict`: past `keep` is only a note.

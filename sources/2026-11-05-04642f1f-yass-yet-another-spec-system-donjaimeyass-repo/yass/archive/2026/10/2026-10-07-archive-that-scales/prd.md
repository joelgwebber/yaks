# Archive that scales: PRD
<!-- Intent: what should change or exist, and why. Not a description of the current system;
     the code and its docs are that. Edit this file only in commits without code. -->

## Why
<!-- The problem, with evidence. -->
Active changes stay few because finished ones are archived, but the archive itself only grows. One person with agents archived 11 changes in this repo in 5 days, roughly 800 a year; a 100-person team could archive 10,000 or more a year. Nothing bounds that, and it costs in four places:

- **Agents.** The archive is read mostly by YASS's tooling and by AI explaining past decisions. `yass-log` greps every archived `## Decisions` section, and `yass-shape` tells agents to check the archive. At thousands of changes that floods an agent's context before it finds anything.
- **The CLI.** Every command parses every archived change, though most only need their names (for `follows:`, `blocked:` and the queue).
- **GitHub.** `archive/` is one flat folder, and GitHub's web UI shows only the first 1,000 entries of a folder.
- **Clones.** Every archived change stays in every checkout forever.

YASS should scale from a solo developer to teams of 100 or more, per yass folder, so each team's archive is handled on its own and the whole repo scales because each team does. Beyond that, git and GitHub themselves become the limit, which is out of scope.

## Users and outcomes
<!-- Who it's for, and measurable targets ("p50 time to log < 5s"). -->
- **Agents explaining decisions** (`yass-log`, `yass-shape`), and the people asking them "why did we…".
- **Teams of any size** sharing a repo, and whoever browses their plans on GitHub.

Targets:
- `yass status` stays under the existing 1s target (200 yass folders, 2,000 active changes) with 10,000 archived changes added, held there by `tests/bench.sh`.
- No folder under `archive/` holds more than 1,000 entries at 10,000 archives a year. A month that occasionally passes 1,000 is acceptable; a yass folder that does so regularly should split into team folders.
- With the default settings, a yass folder that evicts when YASS tells it to keeps about 2,000 archived changes in its working tree.
- A decisions query returns at most its limit (50 lines by default), whatever the archive's size, and says when it cut results off.
- Two branches that each evict the same month merge without conflict.
- A decisions query that doesn't reach an evicted month reads no git history.

## Requirements
<!-- One line each: "- **R1** [M1] A user can …". IDs are never reused. -->
- **R1** [M1] `yass archive` writes `archived: <UTC timestamp>` into the change's frontmatter, as part of the archive commit.
- **R2** [M1] `yass archive` moves a change to `archive/<YYYY>/<MM>/<name>/`, by its `archived:` date.
- **R3** [M1] Every command that reads the archive (`status --archived`, `follows:`, `blocked:`, the queue, resolving a name) finds changes in both the flat and the monthly layout.
- **R4** [M1] `yass upgrade` moves a flat archive into months. It stamps `archived:` with the date of the commit that put the change in the archive, or, outside git, the date in the change's folder name. It prints how to commit the move on its own.
- **R21** [M1] `yass upgrade` runs migrations as small, separate steps of one general mechanism. Each step tells from the files whether it's needed, does nothing when it isn't, and can be deleted in a later release without touching the others. The archive move (R4) is the first step.
- **R5** [M1] This repo's archive is migrated with it, as the first real check of R4.
- **R6** [M2] Commands that don't show archived contents read only the names of archived changes, not their files.
- **R7** [M2] `tests/bench.sh` adds 10,000 archived changes to its large repo, and `yass status` meets the 1s target there.
- **R8** [M3] `yass decisions` lists the `## Decisions` entries of active and archived changes across every yass folder, one per line, with the change, who decided, the decision, and each date it knows: the entry's own date, and the change's `created:` and `archived:` dates.
- **R22** [M3] A decision entry carries the date it was made: `- <decision> - <why> (<who>, <YYYY-MM-DD>)`. The templates and playbooks say so. Entries written before this have no date of their own, and `yass decisions` shows them with the change's dates only.
- **R23** [M3] `yass decisions --json` gives the same results as JSON, with each date in its own field.
- **R9** [M3] `yass decisions` filters by `--since` and `--until`, by text (`--about`), and by change (`--change`, which takes in the changes it follows and the changes that follow it), and stops at `--limit` (50 by default), saying how many it left out.
- **R10** [M3] `yass-log` and `yass-shape` look up past decisions with `yass decisions` rather than reading `archive/` directly.
- **R11** [M4] A yass folder's `yass.yaml` can set `archive: { keep: N }`, the number of top-level archived changes (pieces count with their change) its working tree should keep. It's a soft limit: going past it changes nothing until someone evicts. Unset, `keep` is 2,000.
- **R12** [M4] `yass evict` removes a yass folder's oldest whole months until it's at or under `keep`, says which months and how many changes it evicted, and prints how to commit that on its own. It never evicts the current month, and does nothing when the folder is under `keep`.
- **R13** [M4] Each evicted month leaves `archive/<YYYY>/<MM>.evicted`, listing its changes and the commit that last touched that month's folder. Two branches that evict the same month from the same history write identical files.
- **R14** [M4] When a yass folder is past `keep`, `yass status` and `yass archive` say so, by how much, and to run `yass evict`. Neither evicts anything itself.
- **R15** [M4] An evicted change still counts for `follows:`, `blocked:` and name resolution, from its month's `.evicted` file, without reading git history.
- **R16** [M4] `yass status --archived` says how many changes are evicted, and from which months, rather than listing them.
- **R17** [M5] `yass decisions` reads evicted changes from git, using the commit in their month's `.evicted` file, only when a query's dates or `--change` reach an evicted month.
- **R18** [M5] When an evicted month's commit is missing (a shallow clone), `yass decisions` returns what it could read, names the months it couldn't, and prints the git command that would fetch them.
- **R25** [M5] `yass decisions --cites <sha>` lists the boxes in active and archived changes, evicted ones included, that cite a commit (`code: <sha>`), with their change, so finding the decision behind a piece of code doesn't mean reading `archive/` directly.
- **R19** [M1] The yass folder README and AGENTS.md templates describe the monthly layout, and rule 5 says the archive is append-only except for eviction by `yass evict`.
- **R20** [M4] The docs explain `keep` and eviction, and say that eviction bounds the working tree but not `.git`: a team that needs smaller clones uses partial clones or keeps its plans in another repo. The CI recipes show a scheduled job that runs `yass evict` and opens a PR with the result, for teams that never want to think about it.
- **R24** [M1] The docs recommend that a yass folder archiving more than about 1,000 changes a month split into team folders. A month folder occasionally passing 1,000 entries is fine.

## Non-goals
<!-- What this change will not do. Agents treat these as walls. -->
- Shrinking `.git` or rewriting history. Eviction only deletes from the working tree.
- Evicting by age, freshness or relevance. Eviction controls size, so it is by count.
- Marking decisions as superseded, or filtering out stale ones.
- A committed index or digest of the archive that every archive commit edits.
- Splitting month folders further (by day or week).
- Migrations that must live forever, or a versioned schema for yass folders.
- Moving the archive to another repo, branch or storage service.
- Editing archived changes, other than the `archived:` stamp and the one-time move into months.
- Evicting active changes, or evicting pieces apart from their change.
- Scaling past what one git repo and GitHub handle.

## Milestones
| Milestone | When it ships, a user can … |
|---|---|
| M1 | browse any team's archive on GitHub month by month, with every archived change dated, existing archives migrated |
| M2 | run `yass status` as fast with 10,000 archived changes as with none |
| M3 | ask `yass decisions` (or an agent using it) why something was decided, and get a bounded answer however big the archive is |
| M4 | cap how many archived changes a yass folder keeps in its working tree, with a sensible default nobody needs to touch |
| M5 | still get answers about evicted changes, from git, without slowing down queries that don't need them, including which change cites a commit |

## Open questions
- None yet.

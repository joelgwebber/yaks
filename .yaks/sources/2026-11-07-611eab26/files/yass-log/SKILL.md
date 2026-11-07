---
name: yass-log
description: Reconstruct the decision log from YASS change folders and git history. Who decided what, when and why, across active and archived changes, for a time range, a feature, or a person; and which code carried a decision out, or which decision is behind a piece of code. Use for "why did we…", "what did we decide about…", "why does this code look like this", release notes, retros, or reviewing decisions made without a human.
---

# yass-log

There's no separate log file. The record is the `## Decisions` sections in the yass folders' `changes/` and `archive/`, plus git history. `yass root` prints each yass folder: `yass/` in the repo (team folders too), or wherever a `yass.yaml` points. Boxes may cite the code they rest on (`— code: a1b2c3d`); those citations link decisions to code.

1. **Scope the question:** a time range, a feature or area, a person or agent, a single "why", or a piece of code (a file, a function, a commit).
2. **Find the decisions:**
   - Ask `yass decisions`: it lists the `## Decisions` entries of active and archived changes in every yass folder, newest first, with each change's created and archived dates. Narrow it with `--since`/`--until YYYY-MM-DD`, `--about "<words>"`, and `--change <name>` (which takes in the changes it follows and that follow it); it shows 50 at a time (`--limit N`) and says how many it left out. `--json` gives the same as data. It reads evicted months back from git when a query reaches them, and says when a shallow clone is missing one (`git fetch --unshallow`). Don't read `archive/` directly: it can hold thousands of changes, and evicted ones are only in git.
   - Each entry reads `- <decision> - <why> (<who>, <YYYY-MM-DD>)`; older ones have no date of their own, so `yass decisions` shows the change's dates instead.
   - **Plans kept apart from the code?** Each code commit names its change in a `yass:` trailer (`git log -1 --format='%(trailers:key=yass)' <sha>`), and `yass status <change>`, run in the code repo, lists the change's merged commits and, by their `ac:`, each criterion's. That's the most direct link, and it survives squash merges.
   - **Starting from code instead?** Find its commits in the code repo (`git log --format=%h -- <file>`, or `git log -S'<symbol>' --format=%h`), then ask which boxes cite each: `yass decisions --cites <sha>` (any prefix of the hash; it searches active, archived and evicted changes). A hit names the change, file and box; its Decisions and Log are the why. No hit means the record doesn't link that commit, so say so rather than guess from dates.
3. **Date them and check who made them** with git, because the files say *what* and git says *when* and *who committed it*:
   - `git log --format='%h %ad %an %s' --date=short -- <change folder>`; for an archived change, which was moved, follow its `change.md` across the move: `git log --follow --format='%h %ad %an %s' --date=short -- <archive folder>/change.md`
   - `git log -S'<distinctive words>' --format='%h %ad %an' -- <file>` finds the commit that added a decision line.
   - For intent edits (PRD, plan, design), `git log -p -- <file>` shows how the promise changed over time.
   - **Squash-merged history:** a subject ending in `(#N)` is pull request N, squashed. Name it next to the commit (`#16`), since its description and review are where the decision was argued, and read them when the decision line leaves the why unclear (with a code host CLI, e.g. `gh pr view N`).
4. **Follow the citations to the code.** For a change's boxes with `code:`, show each cited commit from the code repo: `git log -1 --format='%h %ad %an %s' --date=short <sha>`, and whether it's merged (`git merge-base --is-ancestor <sha> <branch>`; `branch:` in `yass.yaml`, else `origin/HEAD`, `main` or `master`).
5. **Read the change's context** (Goal, PRD, Log) for the *why* behind anything the decision line leaves unclear.
6. **Report** newest first, grouped by change, with the code that carried each decision out when it's cited:
   ```
   2026-09-15  offline-sync  Sync is last-write-wins per entry - entries are append-only (sam)
               code: 4f2c1aa 2026-09-18 sam "feat: per-entry LWW merge" (merged)
   ```
   Flag decisions with no recorded author, and goalposts that moved:
   - **Plans in this repo:** intent edits that landed in the same commit as code.
   - **Plans in another folder:** that can't happen, so look for the cross-repo version instead: an acceptance criterion reworded or dropped (`[-]`) *after* a box citing its code was marked `[x]`. Compare when the criterion changed (`git log -p` in the plans) with when the box was marked done.
7. Cite file paths and commit shas so every line can be checked. Don't infer decisions that aren't written down; say when the record is silent.

## Plans in another folder
When `yass root` prints a folder outside this repo, the plans and the code have separate histories:
- **Run git for the plans in their folder,** limited to it, since it may share a repo with other projects' plans: `git -C <folder> log … -- .`. Commit shas in your report should say which repo they're from.
- **Run git for the code here,** in this repo, for the cited commits.
- **No git at all?** A plain folder has no commit dates. The only dates are the Log headings (`### YYYY-MM-DD`), so place a decision between the Log entries around it, and say the date is approximate.

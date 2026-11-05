# Large monorepos first, separate plans repo done right: design
<!-- Intent: the calls that are hard to undo. Edit only in commits without code. -->

## Context
This change adds public surface that people will script against and put into commits, CI configs and `yass.yaml` files: a commit trailer, two `yass.yaml` keys, a citation syntax, CLI commands and flags, and the rule that pieces own their files. Each is costly to rename once it's in use, so each is decided here. The PRD fixed the big calls: inline plans in a monorepo are recommended, a separate repo is the privacy fallback, and trailers are enforced only in that fallback (Jaime). Every call here was settled with Jaime on 2026-10-04.

## 1. When plans count as separate
**Options.**
- (a) Separate when the yass folder isn't inside the code repo's git work tree, whether it's in another repo or in no repo at all.
- (b) A `mode:` key in `yass.yaml`.
- (c) Separate whenever `path:` is set.

**Decision: (a)** (Jaime, 2026-10-04). Put simply: plans are separate when `yass.yaml`'s `path:` resolves outside this repo. It's derived from the setup, so it can't drift out of sync with it. (c) is wrong because `path: planning` inside the repo is still inline, and (b) adds a setting people could get wrong. A linked worktree counts as the same repo.

**A `yass/` symlink, as a convenience** (Jaime). When plans are separate, `yass init` and any `yass` command that finds it missing or stale create a `yass/` symlink in the checkout, or worktree, pointing to the resolved plans folder. They also add the link's path to the clone's `.git/info/exclude` once. That file is shared by every worktree of the clone and is never committed, so the link adds no diff, even in repos that keep `yass.yaml` itself private. Because the link is ignored and per machine, its target is absolute. Everyone, editors and agents included, sees `yass/changes/...` at the root. `yass.yaml` stays the source of truth, and nothing depends on the link:
- a real `yass/` folder or file already there is left alone, with a warning;
- where symlinks aren't allowed (Windows without Developer Mode), YASS skips it with a note;
- root discovery dedupes the link against the folder it points to.

Its known limits are documented: a new worktree has no link until a `yass` command runs there, `rg` needs `-L` to follow it, and an agent harness may treat files behind it as outside the project for permissions.

## 2. How code commits name their change (separate plans only)
**Options.**
- (a) A git trailer in the commit message.
- (b) A link in the PR body only.
- (c) SHA citations in the plans only, as today.

**Decision: (a), as one `yass:` trailer with a small DSL** (Jaime, 2026-10-04). Trailers sit in commit history and survive squash merges when the squash message keeps them. `git interpret-trailers` and `%(trailers)` already parse them, and they follow the precedent of `Change-Id:`, `Depends-On:` and `Bug:`. (b) can't be checked offline, and (c) breaks on squash merges.

```
yass: <target> [<key>:<value> ...]

target  := none | <change> | <change>/<piece> | <piece>
key     := ac            (more later)
value   := comma-separated, no spaces
```
- **Case:** YASS writes lower case, and reads the key and the value case-insensitively. Git matches trailer keys case-insensitively too.
- **Several changes:** one `yass:` line per change.
- **Pieces:** a piece can be named alone, matched by folder name as `blocked:` matches it, with the path needed only when the name is ambiguous.
- **`ac:`** names the criteria the commit works toward. That's the box-to-commit link from the code side, and a squash merge can't break it. `code:` citations on boxes stay, as optional detail (Jaime).
- **Unknown keys** are a warning, so newer trailers don't break older `yass` versions.
- **`yass: none`** marks a commit that belongs to no change (dependency bumps, typo fixes).

**Strictness** is set by `trailers:` in the code repo's `yass.yaml`: `warn` (the default), `require` or `off` (Jaime). `yass hook --range` follows it in CI, and `--strict` turns its warnings into failures, as today.

**Added automatically** (Jaime chose all three sources, in this order). A `prepare-commit-msg` hook works out the target from:
1. `yass use <change> [--ac ac3]`, stored per worktree in that worktree's git folder, so it's never committed;
2. the branch name, matched against change folder names;
3. a `yass:` trailer on an earlier commit since the branch left the merged branch.

The hook appends the trailer with `git interpret-trailers`. It never overwrites one the author wrote, and it skips merge commits. Unlike `commit-msg`, `prepare-commit-msg` isn't skipped by `--no-verify`. The `yass-work` skill runs `yass use` when it starts a change, and puts the trailer line in PR bodies so squash merges that use the PR description keep it. For orgs with a hook manager (husky, lefthook, pre-commit), the docs show how to call `yass hook --prepare-msg` and `yass hook --commit-msg` from it instead of using `core.hooksPath`.

## 3. How a code repo is named in citations
**Options.**
- (a) A `repo:` key in the code repo's `yass.yaml`. Without it, the name comes from the last path segment of the `origin` URL (minus `.git`), and failing that, from the main worktree's folder name.
- (b) Always the folder name.
- (c) Make `repo:` required.

**Decision: (a)** (Jaime, 2026-10-04). Since trailers carry the main link, this only matters for a shared plans repo whose boxes still carry the optional `code:` citations. Folder names differ between clones and worktrees, so (b) breaks for agents. Requiring `repo:` (c) taxes the common case, one code repo. Citations look like `code: web@a1b2c3d`. An unqualified `code: a1b2c3d` belongs to whichever repo is checking it, which is today's behavior.

## 4. How a plans repo finds the code
**Options.**
- (a) `repos:` in the plans repo's `yass.yaml`: a map from repo name to a local checkout path, resolved like `path:` (relative, `~`, `${VAR}`).
- (b) Clone remotes on demand.
- (c) Nothing: check only from the code repos.

**Decision: (a)** (Jaime, 2026-10-04). Paths accept `${VAR:-default}`, as `path:` does, so `web: ${CODE_HOME:-..}/web` works for side-by-side clones with no setup, and anyone can set `CODE_HOME` instead. The agent's project root stays the code repo, with the plans added as an extra directory; `repos:` is only for working in the plans repo itself and for its CI. It works offline and in CI, where the pipeline checks out the code repos next to the plans repo. It never touches the network. A name with no checkout is reported as unchecked, not as an error. (b) needs credentials and network access, and (c) leaves the plans repo unable to check itself.

## 5. How parallel branches stop conflicting over progress (R7)
**Options.**
- (a) A YASS merge driver (`yass merge`) that auto-resolves Log appends, box marks on different lines, and removals from `queue.md`.
- (b) `merge=union` in `.gitattributes`.
- (c) Guidance only.
- (d) Pieces own their files: parallel branches never share a YASS file, so there's nothing to merge.
- (e) Keep progress in a tracker (Jira, Linear) instead of in files.

**Decision: (d)** (Jaime, 2026-10-04, after discussing the trade-offs with trackers). The conflicts come from the plan's shape, with pieces on parallel branches ticking the same parent `plan.md` and appending to the parent's Log, so the fix is the shape, not git machinery.
- **The rule:** a branch working on a piece changes only that piece's folder (and code). The parent's `prd.md`, `plan.md`, `design.md` and `change.md` change only in intent commits, or in a commit with no code (its own progress, or the archive move).
- **Parent criteria:** a piece names what it delivers in a box like `Delivers AC1, AC3–AC5`. Superseded on 2026-10-04 by `2026-10-04-stacked-prs-and-squash-merges` (R5, R6): a delivered criterion counts as done without a closing commit per piece, and `yass archive` marks it. Until that ships, parent boxes are marked in a closing commit with no code, on main, so there's no parallel contention. It shipped on 2026-10-09 (#59): no closing commit is needed.
- **The hook** gives a heads-up when a commit with code touches the parent's files, or more than one piece's folder.
- **What still conflicts,** and should: two people on the same piece, reorders in `queue.md` (intent), and two archive commits that each remove an adjacent `queue.md` line (rare, and resolved in seconds).

Why not the others: (a) has to be configured in every clone, doesn't run in hosted merges or merge queues, and a bug in it silently loses content. (b) keeps both versions of a box marked differently on each side. (c) doesn't solve anything. (e) gives up progress travelling with code, and makes agents work through MCP instead of files.

## 6. `yass paths`
**Options.**
- (a) `yass paths` prints, one per line, the repo-relative globs YASS owns: each yass folder inside the repo and each `yass.yaml`. `yass paths --only <range>` exits 0 when every file changed in the range is one of them, 1 when not, and 2 on error.
- (b) Print globs only, and leave the comparison to each CI.
- (c) A separate `yass ci` command per CI vendor.

**Decision: (a)** (Jaime, 2026-10-04). One command lets any CI write a gate job, including when the path list changes as teams adopt YASS. (b) pushes fiddly diff logic into every pipeline. (c) is the per-vendor product the PRD rules out. The pre-commit hook script in `tools/yass/githooks/` isn't a YASS path: changing it is changing code.

## 7. Seeing unmerged work (R19)
**Options.**
- (a) A branch view in `yass status`: one `git log` over local and remote-tracking branches not yet on the merged branch, limited to YASS paths and to branches with commits in the last 30 days. For each change or piece a branch touches, it shows the branch, its last author and date, and its box progress and latest **Next:** as that branch has them. Results are as of the last fetch, and the output says when that was.
- (b) A claims file (`claimed-by:` in front matter) committed to main.
- (c) A tracker as the live view.

**Decision: (a), shown by default** (Jaime, 2026-10-04). It needs nothing new from anyone: pushing a branch already announces the work. (b) makes every start a commit to main, which brings back the contention §5 removes. (c) is a non-goal. It never fetches by itself, because the network is the caller's choice. `--since <date|Nd>` sets the window (default 30 days), and `--no-branches` leaves the view out, for scripts and huge repos. Because it's on by default, it counts toward `yass status`'s speed target. Jaime kept it on everywhere after reviewing what it costs (2026-10-04). To keep that safe on large repos:
- the branch walk uses the commit-graph and its changed-path filters when the repo has them;
- reading file versions at branch tips never triggers a partial clone's on-demand download (`--no-lazy-fetch`, git 2.44+). With older git, a branch whose files aren't local shows its name and author without progress;
- remote branches deleted on the server are listed until a `fetch --prune`, and the view's freshness line says how old the refs are.

## Consequences and rollback
- The `yass:` trailer grammar, the `trailers:` key, `yass use`, the `repo:` and `repos:` keys, `web@sha` citations, `yass paths`, the branch view's `--since` and `--no-branches`, the gitignored `yass/` symlink, `${VAR:-default}` in paths, and the `Delivers AC…` box format all become public API. Renaming any of them later needs a deprecation period. Until a release ships them, they can still change freely.
- "Pieces own their files" is a convention the hook only gives a heads-up about. Dropping it later loosens a rule; it doesn't break any file.
- For inline plans, the only changes are `yass paths`, the branch view, the *ready to mark* view and the pieces-own-their-files heads-up.

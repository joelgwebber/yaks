# Skills restructure plan (yaks-5c9f, feeds yaks-2f9b)

Status: PLAN, nothing written into `.agents/skills/` yet. Drafts live under this
yak's artifacts until accepted: Delta discovers `.agents/skills/` live (log O1), so
a half-finished skill there would appear in every agent's catalog at once.
Author: delta-lead. Evidence base: yaks-80d5 (matrix) and yaks-df61 (log O1-O35).

## 1. Principle
Farm mode and environment are independent (Joel, b5a0). So the coordination guidance
splits along those two axes, and every skill says in its first lines when to use it
and what to read next:
- MODE says what the farm IS: who sees a claim, a note, an ask, and when; what may
  appear in git; how provenance is recovered.
- ENVIRONMENT says how work is MOVED: how a worker gets a checkout, how its result
  comes back, what the harness does for you.
- CORE is what holds in every cell.
Followable alone = each skill is complete for its axis; a reader loads core + one
mode + one environment (three short files), not one 386-line file full of "(in
private mode the opposite holds)".

## 2. Target set (all repo-internal, not embedded; see fork 2)
| skill | axis | holds | est. lines |
|---|---|---|---|
| yaks-working | worker | one yak, start to shear: re-read, claim, notes, evidence, shear, commit; asks; drift is signal (75cc); git-add pathspec + gate-on-contents (0a87) | ~110 (today 114) |
| yaks-coordinating | core + router | design test; degradation; disjoint scoping (files, TYPES, function-level); pre-flight; attribution; evidence contract + judge; ask/answer routing rules; the shape claim > fan out > land > reconcile; "read next" router keyed on `git ls-files .yaks` and the harness | ~150 |
| yaks-coordinating-team | mode | `.yaks/` committed: per-branch farms reconcile at merge; claim commit; shorn move rides with the code; asks live on the lane until landed; id in commit messages; `yaks commits` provenance; PR-driven (team fork) | ~110 |
| yaks-coordinating-private | mode | `.yaks/` gitignored or out-of-tree: ONE live farm for every lane; claim live; no ids in commits/PRs; `scan-ids`; SHA stamp-back; PR-driven (private fork); out-of-tree pointer; same-yak writes lose notes (yaks-800d) | ~110 |
| yaks-coordinating-worktrees | environment | `git worktree`: `wt/` dirs, file-tool SOP, per-lane build, serial-arc run shape, human-driven lane, crash recovery | ~130 |
| yaks-coordinating-delta | environment | Delta: parent thread = coordinator; `spawn_subagent` + worker brief; landing mechanics; resume by message; attached extra repos; ask/answer by SHA | ~150 |
Eight is too many to load, three is not: core + mode + environment.

## 3. Where today's yaks-coordinating (386 lines) goes
| today's section | goes to |
|---|---|
| The design test; Harness degradation | core |
| Worktrees are per-branch farms: team paragraph | team |
| ...the private parenthesis | private |
| File-tool SOP (wt/, explicit paths, gitignore trap) | worktrees |
| Disjoint scoping (incl. TYPES, function-level) | core |
| Coordinator pre-flight | core (+ a mode/env line each) |
| Attribution | core (rewritten, yaks-7149) |
| Parallel run shape (claim > fan out > merge > reconcile) | core shape; "claim in one commit on main" to team, "claim live" to private |
| Evidence contract + judge | core |
| Serial-arc run shape | worktrees (team checkpoints noted in team) |
| Merge / integration (squash, id in message) | team; private gets "no ids" |
| Human-driven interactive lane | worktrees (kickoff, harness root) with the farm split pointing at team/private |
| PR-driven integration | one fork section in each of team and private; guardrails in core |
| Recovery | worktrees; Delta has its own |
| Human-in-the-loop | core principles; team (ask on the branch until landed), private (live), Delta (resume, answer by SHA) |

## 4. Contradictions and gaps this must close (from yaks-80d5 / log)
1 mode forks scattered over 4 sections: solved by the split.
2 out-of-tree farms absent from coordination: a section in private.
3 "gitignored .yaks is absent in other clones" false in Delta (O11): private states the walk-up as the mechanism; fix tracked in yaks-b4dc (explicit pointer).
4 binary resolution (PATH > npx > ./target) fails in Delta: environments say how to get a binary; never npx.
5 claim-commit rule: Delta snapshots the working tree (O3): delta says commit anyway for durability, not visibility.
6 HITL "hand back, don't wait": delta says ask, return, answer to the SAME worker (O23); coordinator may answer scope/mechanics, must route design forks to the human (O24, Joel).
7 file-tool SOP is Zed-specific: lives only in worktrees.
8 no brief template: the brief lives with delta (template v2, yaks-2f9b) and a short "brief contract" in core.
9 attribution opt-in and defaults to the human: core rewritten around inline `YAKS_ACTOR=`/`--as` per command; transitions unattributed until yaks-7149.
10 evidence reaches the parent only if tracked: team and delta say `git add` new artifact dirs.
11 "merge back" not a coordinator-controlled event in Delta: delta checklist items 1-8.
New from trials: land by SHA one at a time; check `git merge-tree --write-tree` before any `git add`; pathspec aborts (0a87); same-yak write races (800d); private-farm live visibility; ask in the farm and wake by message.

## 5. Folding in
- yaks-0a87 (checkpoint footgun): working (pathspec, gate on contents) + team (commit human drift before any squash-merge checkpoint; gate re-sync on success).
- yaks-75cc (drift is signal): working + core: unexpected yak changes mean the human is working too; do not investigate or report them unless they change your task.
- yaks-c061 (answered-question pickup) informs delta/core ask routing; not folded.

## 6. Guards (cargo test, no new dependency)
Over `.agents/skills/*/SKILL.md`: frontmatter `name` equals the directory; non-empty
`description` that says when to use it; every `yaks-*` skill named in a body exists;
no skill over its line budget; core's router names every mode and environment skill.
The BUNDLED list stays explicit (Joel): none of these is embedded.

## 7. Proof, not prose
After drafting, run trial 4: workers get a SHORT brief (yak, scope, evidence, name)
and must load the skills themselves (project skill discovery now works), on one team
cell and one private cell. Compare behaviour against trials 1-3 (landing, asks,
attribution, evidence). A cold-read check first: a scout with only the three skills
answers 6 scenario questions (e.g. "a worker returned with MERGE_HEAD pending; what
now?"); wrong answers are skill bugs.

## 8. Order
1. core + team + private (farm semantics; fork-independent).
2. worktrees (move, trim) and delta (from template v2).
3. working (fold 0a87/75cc).
4. guards; cold-read; trial 4; then replace the old yaks-coordinating in one commit.

## 9. Forks (asked on the yak with leans)
1. Layering: A (core + 2 modes + 2 environments, as above) or B (two standalone mode skills with the git-worktree mechanics folded in, Delta layered on top, as originally phrased). Lean A: B makes every Delta reader load worktree material that does not apply (finding 7) and duplicates the shared core in two files.
2. Shipping: embed any of these in the binary (the original 5c9f text) or keep them repo-internal (AGENTS.md today)? Lean: internal until trial 4 shows they work for someone who is not us.

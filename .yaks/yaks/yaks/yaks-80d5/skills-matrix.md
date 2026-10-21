# Coordination skills x environment x farm mode (yaks-80d5)

Sources: `skills/yaks`, `skills/dev/yaks-coordinating` (386 lines), `skills/dev/yaks-working`
(114 lines), the 3901 capstone, yaks-213b (PR-driven validation), yaks-77a5 (11-lane run),
yaks-5fe9 (first Delta run), and the Delta observation log (yaks-df61, O-numbers below).
Written 2026-10-03 by delta-lead.

Legend. Doc: Y documented, P partial/scattered, N absent. Run: R ran for real (evidence on a
yak), V validated narrowly, P predicted from code/observation only, - nothing.

## Matrix

| Environment | Team (public, `.yaks/` committed) | Private (gitignored/stealth) | Out-of-tree (pointer/symlink) |
|---|---|---|---|
| **1. Single agent** | Doc Y (yaks). Run R (this repo, daily). | Doc Y (yaks). Run V: the only private-mode evidence I found is yaks-213b (a real stealth repo, but a PR-landing check, not a day-to-day single-agent run). | Doc Y (yaks, AGENTS.md). Run - not evidenced by anything I read. |
| **2. Git worktrees + coordinator** (Zed/Claude Code) | Doc Y (coordinating: "Worktrees are per-branch farms", claim/fan-out/merge). Run R: 10 runs in 3901 with zero corruption; 11 lanes + 3 scouts in 77a5. | Doc P: mode content is split across "Worktrees are per-branch farms" (one parenthesis), "Human-driven interactive lane" and "PR-driven integration". Run V: 213b, one worktree in a real stealth repo, walk-up resolves the shared farm. No multi-lane run. | Doc N: coordinating never mentions pointers. Prediction (code): an in-tree `wt/` finds the pointer by walk-up like private; an out-of-tree worktree needs the symlink or a copy of the pointer. Run -. |
| **3. Delta subagents** (`spawn_subagent`, auto-merge on finish) | Doc N. Run R x1 (7fa0): landed, but only after a fix-up; see yaks-5fe9, O2-O6. | Doc N. Run P: O11 shows walk-up from the nested checkout reaches the user's LIVE farm, so all subagents would share one farm, nothing to merge. Not run. | Doc N. Run P: same as private (pointer at repo root found by walk-up, resolved relative to the pointer). Not run. |
| **4. Delta human subthread** (human creates a thread to work a lane) | Doc N. Run R x1 (7fa0 fix-up): worked; landing left the parent stale (5fe9 #5, O8). | Doc N. Run -. Same walk-up prediction as row 3. | Doc N. Run -. |
| **5. PR-driven landing** (coordinator opens id-free PRs) | Doc Y (coordinating "PR-driven integration", team fork). Run V: 213b team-mode cycle, local squash standing in for GitHub. | Doc Y (private fork + `scan-ids`, SHA stamp-back). Run V: 213b in a real stealth repo, no real `gh pr create`. | Doc N. Run -. |

Row 5 is an integration style layered on rows 2-4, not an environment of its own. It is the
one place where Delta needs a decision the git-worktree docs never faced: the coordinator's
"main" is a thin clone, so landing means `git push local|origin` (5fe9 #1); O12 is open on
how a push into the user's checked-out `main` behaves.

## Mixed axis: what each skill assumes

| Skill | Assumes about the environment | Assumes about the farm mode |
|---|---|---|
| `yaks` | A shell with `yaks` on PATH, else `npx`, else `./target/release/yaks`. | Names all three modes and how to detect them (`git ls-files .yaks`). Rule 2 and 3 branch on mode. |
| `yaks-working` | One agent in its own checkout; run `./target/release/yaks` "in this checkout". | Almost mode-agnostic; one bullet forks on "Team mode: stage the shorn yak with the code". |
| `yaks-coordinating` | Git worktrees (`wt/<name>`), Zed-style file tools that root at the main checkout, a long-lived coordinator that owns `main`. | Defaults to team; private behaviour appended per section ("(In private mode the opposite holds...)"). |
| (none) | Delta | (none) |

## Contradictions and gaps the matrix exposes

1. **Mode forks are inline, not structural.** coordinating describes team behaviour in
   "Worktrees are per-branch farms", then the opposite in a parenthesis, then again in the
   Human-driven lane (split "Farm behavior splits by mode") and again in PR-driven
   integration (bold fork headings). Four places, four phrasings; a reader must assemble
   their mode from fragments. Supports yaks-5c9f (split by mode).
2. **Out-of-tree farms are invisible to the coordination layer.** The `yaks` skill and
   AGENTS.md document pointer files; coordinating never says what a worker's worktree sees.
   Row 2 and 3 cells are predictions.
3. **The `yaks` skill's private-mode claim is false in Delta.** It says a gitignored `.yaks/`
   "is also not carried into fresh clones or other git worktrees, so it's simply absent
   there." In Delta the clone is fresh yet the nested walk-up finds the user's farm (O11).
   Either the skill or `discover` has to change; the walk-up is also a latent hazard for
   any thread whose checkout lacks `.yaks/`.
4. **Binary resolution differs and fails in Delta.** `yaks` says PATH, then `npx @j15r/yaks`,
   then `./target/release/yaks`; `yaks-working` says `./target/release/yaks` in a checkout.
   A Delta terminal has no `yaks` on PATH and no `target/` in a fresh worker checkout, and
   my first attempt fell into `npx`, which offered to download `@j15r/yaks@0.0.12`
   from the network (the prompt was auto-answered; I did not check what it ran). A Delta skill must say: build in your own checkout, or be given the path.
5. **Claim-commit rule vs. Delta snapshot.** coordinating requires the claim as a commit on
   `main` so worktrees cut from it see it, and has a gotcha that an uncommitted `answer`
   is invisible to a worktree cut afterward. O3 suggests a Delta subagent's worktree is a
   snapshot of the parent's working tree, including uncommitted changes. If a deliberate
   test confirms it, the gotcha disappears in Delta and the claim commit is only for
   durability.
6. **HITL "hand back, don't wait" assumes no messaging.** Written for harnesses where a
   worker cannot be resumed. Delta has `send_agent_message` and may allow resuming a
   finished subagent; that is yaks-e87e and changes the Human-in-the-loop section for Delta.
7. **The file-tool SOP is Zed-specific and inapplicable.** It exists because Zed's tools
   root at the main checkout and refuse gitignored paths. Delta addresses files by worktree
   id and fails loudly with several worktrees attached (O13). The SOP is noise for a Delta
   reader and belongs to the git-worktree environment skill.
8. **No worker brief template anywhere.** coordinating says what the claim note must hold
   (owner tag, file scope, evidence contract and judge) but a spawn prompt has no checklist;
   my own brief missed four of them (O5). yaks-working is written to the worker, not as
   something a coordinator pastes into a brief.
9. **Attribution is opt-in and defaults to the human.** `--as` / `$YAKS_ACTOR` are described
   as optional; in Delta every thread runs as the user's git identity, so unattributed
   notes are indistinguishable from the human's (O4).
10. **Evidence reaches the parent only if tracked.** yaks-working says `yaks attach` keeps
    artifacts under `.yaks/artifacts/` "committed in team mode"; in Delta a new artifact
    directory is not replicated until staged (O9). Workers must `git add` it.
11. **"Merge back" is not an event the coordinator controls.** Delta moves a finished
    worker's commits into the parent clone on completion (O2), so the coordinator reviews
    after landing. coordinating's "verify each lane's branch is disjoint before merging"
    has no pre-merge point unless the brief says to leave the work uncommitted.

## What it means for the restructure

- Environment-specific content (file-tool SOP, `wt/` paths, `git worktree add/remove`,
  Zed gotchas) is a git-worktree skill; Delta gets its own (yaks-2f9b). That matches
  Joel's direction: split by farm mode, layer environment on top.
- The shared core (claim, lanes, evidence, ask/answer, attribution) should hold the rules
  both environments obey, which today are scattered across coordinating and working.
- Untested cells are the work list for yaks-fe33. Highest value first: row 3 x team with a
  proper brief (the repeat of the first run), row 3 x private (O11, risk), then O3 and O9
  as direct tests.

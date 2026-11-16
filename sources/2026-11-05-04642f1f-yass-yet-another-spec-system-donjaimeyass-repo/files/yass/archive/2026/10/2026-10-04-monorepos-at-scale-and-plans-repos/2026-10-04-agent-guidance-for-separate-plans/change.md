---
platforms: [all]
source: 
follows: 
blocked: 2026-10-04-check-from-the-plans-repo
---
# Agent guidance for separate plans

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
The skills give agents the whole routine for separate plans: state checks at session start, commit order with trailers, cross-links, per-repo git latitude, worktrees.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC55, AC56 in [plan.md](../plan.md)
- [x] The README's separate-plans section and `docs/skills.md` match the skills

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] AC55: `yass-work`'s "Plans in another folder" is the whole routine: pull the plans first; stop, never `yass init`, when they're missing (worktrees already look in the clone's other checkouts); `yass use` so the hooks add the trailer; code first, then plans, citing (`repo@sha` too); `[x]` after the merge; cross-link with `--brief` and the trailer line; latitude per repo
- [x] `yass-log` and `yass-status` find a change's code by trailers (`yass status <change>`, `%(trailers:key=yass)`)
- [x] README bullet and `docs/skills.md` match; e2e (700 ok) still finds the installed skills
- [x] Jaime reviews the wording (AC55, README, docs/skills.md)
- [x] AC56: one fresh agent session in a linked worktree of a code repo with separate plans, told "keep going" on a change (run with Jaime, 2026-10-11; see Log)
- [x] Its findings: the plans repo with no upstream gets a sentence in the routine; the trailer hooks no longer block commits under an older `yass` (#74); the mixed inline-and-separate case is for Jaime (see Log)

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- AC56 ran on this repo, as Jaime proposed: it has an inline `yass/` and a separate private plans repo, so the run exercised the routine against a real separate plans repo; trailers are off in a mixed repo by design, so the agent added one by hand (claude, 2026-10-11)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-10 (claude)
- Did: AC55: the separate-plans routine in `yass-work`, trailers in `yass-log` and `yass-status`, README and `docs/skills.md`. `tests/e2e.sh` (700 ok), `tests/site.sh` pass.
- Next: Jaime reviews the wording, and runs AC56: a fresh agent session in a linked worktree of a code repo with separate plans, told "keep going"; it should pull the plans, build, commit code with the trailer, record progress in the plans repo, and never run `yass init`.
### 2026-10-11 (claude)
- Did: AC56 with Jaime. Setup: a small real change (`2026-10-11-test-brief-output`, a go test for `--brief`) committed in Jaime's private plans repo (no remote); a linked worktree of this repo on a throwaway local branch, its playbooks refreshed from a development build so the agent read this piece's routine; a fresh general-purpose subagent told only "Keep going on the change 2026-10-11-test-brief-output", with where the worktree and the development `yass` were and that it could commit but not push.
- The run: it read AGENTS.md and the worktree's `yass-work`; ran `yass root -v` (separate), tried `git -C <plans> pull --ff-only` (failed: no upstream; it carried on), `yass status`, `yass use 2026-10-11-test-brief-output`; wrote `TestBrief` and broke `brief()` on purpose twice to see the test catch it; committed the code first (98fc050, with `yass: 2026-10-11-test-brief-output`), then the plans (2e055d9: both criteria `[/]` citing `code: 98fc050`, a Decision, a Log entry, Next: merge, then `[x]` and archive). It never ran `yass init`. All checked against both repos.
- Findings: (1) in a repo with both inline and separate plans, trailers are off, so the hook added none and it wrote the trailer by hand - Jaime to decide whether a change in a separate folder should get one; (2) no upstream to pull - now a sentence in the routine; (3) the Skill tool served the main checkout's older `yass-work`; it noticed and read the worktree's; (4) setting up showed the trailer hooks blocking commits under an older `yass` - fixed in #74; (5) a dead guard in `brief()` - removed in #74.
- Next: Jaime reviews the wording; the mixed-repo trailer question.
### 2026-10-11 (claude)
- Did: Jaime reviewed the wording and merged it (#73); the commit with the no-upstream sentence and the AC56 record missed that merge and is carried over here. The piece is done, and with it every piece of the change.
- Next: archive `monorepos-at-scale-and-plans-repos` once #74 (which logs into piece 9's folder) has merged.

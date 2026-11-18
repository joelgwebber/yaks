# Stacked PRs and squash merges: plan
<!-- Intent: how, and how we'll know it's done. With code, only mark boxes here ([/] in progress, [x] done);
     anything else is a plan revision in its own commit. -->

## Approach
<!-- The shape of the solution. Hard-to-undo calls go in design.md. -->
**M1 is a few sentences of text.** [`docs/monorepo.md`](../../../docs/monorepo.md) gets a short section on squash merges: a squash folds a PR into one commit, so the commits YASS keeps apart from code (a plan revision, an archive move, a `queue.md` reorder) need a PR apart from code, while intent, progress and archive commits can share one; how a team splits and stacks its branches is its own call. [`yass-work`](../../../kit/.agents/skills/yass-work/SKILL.md) and the `AGENTS.md` section ([`templates/agents.md`](../../../internal/yass/templates/agents.md)) say the same where they ask for intent and archives in their own commits ("apart from code"), and its "Git is up to whoever is driving" paragraph adds pushing a branch and opening or updating a PR to what needs latitude. [`yass-log`](../../../kit/.agents/skills/yass-log/SKILL.md), when it explains a decision from history, names the PR of a squash-merged commit from the `(#N)` in its subject. No workflow guidance is added anywhere.

**M2 counts delivered criteria.** A piece's `Delivers AC1, AC3–AC5 in [plan.md](../plan.md)` box already names the parent criteria it delivers. `yass status` and `yass archive` ([`internal/yass/repo.go`](../../../internal/yass/repo.go), [`commands.go`](../../../internal/yass/commands.go)) read those boxes: a parent criterion counts as done once every piece delivering it is done, in progress and `next:`, and `yass archive` doesn't refuse for it. The archive move marks those criteria `[x]` in the archived `plan.md`. An e2e scenario lands a three-piece change one squash-merged branch at a time and checks it.

## Acceptance
<!-- Per milestone, one observable behavior each:
### M1
- [ ] AC1 (R1) Given …, when …, then … — verify: <test, flow, or manual steps> -->
### M1
- [x] AC1 (R2) Given `docs/monorepo.md`, then it says that a squash merge folds a PR into one commit, so a plan revision, an archive move or a `queue.md` reorder needs a PR apart from code, that intent, progress and archive commits can share a PR, and that splitting and stacking branches is the team's call — verify: manual: review
- [x] AC2 (R13) Given `yass-work`, then where it asks for intent and archives in their own commits, it says that means apart from code, and under squash merges a PR apart from code — verify: manual: review
- [x] AC3 (R13) Given the `AGENTS.md` section `yass init` writes, then its intent and archive lines say "apart from code", and `yass upgrade` brings existing sections up to it — verify: e2e (the section's text); manual: review
- [x] AC4 (R14) Given `yass-work`'s git paragraph, then pushing a branch and opening or updating a PR need the same latitude as committing, branching and rebasing, and without it the playbook shows the commands — verify: manual: review
- [x] AC5 (R9) Given `yass-log` explaining a decision whose commit was squash-merged, then it names the PR from the `(#N)` in the commit's subject — verify: manual: ask it why `yass init --hooks` leaves existing hooks alone, and check it names #16
- [x] AC6 (non-goals) Given the kit after M1, then it adds no workflow guidance: nothing on how to branch, stack, sync, merge or which tools to use — verify: manual: review the diff

### M2
- [x] AC7 (R5) Given a parent criterion named by `Delivers` boxes in two pieces, when both pieces are done, then `yass status` counts the criterion done in the parent's progress and doesn't offer it as `next:` — verify: e2e
- [x] AC8 (R5) Given one of those pieces still open, then the criterion stays open — verify: e2e
- [x] AC9 (R5) Given a criterion no piece delivers, then it counts by its own box, as today — verify: e2e
- [x] AC10 (R5) Given a large change whose only open boxes are criteria its done pieces deliver, when you run `yass archive`, then it doesn't refuse — verify: e2e
- [x] AC11 (R6) Given that archive, then the archived `plan.md` has those criteria marked `[x]`, and no other box changed — verify: e2e
- [x] AC12 (R5, R6) Given `Delivers` boxes written as lists and ranges (`AC1, AC3`, `AC2–AC9`, `AC2-AC9`, with trailing words such as "and AC19's preview card" or "in plan.md"), then each names the right criteria — verify: go test
- [x] AC13 (R10) Given an e2e scenario that lands a three-piece change one squash-merged branch at a time (`git merge --squash`, rebasing the rest onto main), then `yass status` is right after each landing, no piece needs a closing commit, `yass archive` closes the change, and every commit on main passes the hook — verify: e2e

## Pieces
<!-- PR-sized pieces, each its own folder in here: `yass new "<title>" --in <this change>`.
     `yass status` lists them; say here what order they go in and why. -->
1. `squash-notes` (M1): the `docs/monorepo.md` section, the two `yass-work` sentences and the `yass-log` note. Text only, so it can land first.
2. `delivered-criteria-count` (M2): parsing `Delivers`, counting in `status` and `archive`, marking on archive, and the e2e scenario. Independent of piece 1.

## Validation
<!-- How the whole thing is verified before it's called done: suites, platforms, manual passes. -->
- CI on each piece: `go test`, `tests/e2e.sh` (with the new scenario), `tests/examples.sh`, `tests/site.sh`, on Ubuntu and macOS.
- Jaime reviews the M1 text, which is judged by reading.

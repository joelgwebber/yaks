# Order and dependencies: PRD
<!-- Intent: what should change or exist, and why. Not a description of the current system;
     the code and its docs are that. Edit this file only in commits without code. -->

## Why
Once a project has more than a handful of changes, the question people ask most is "what's next?", and YASS can't answer it. `yass status` lists changes by creation date, which records when an idea arrived, not when anyone wants to work on it.

- **Migrating loses the order.** A repo moving to YASS often has a global todo list or "future work" sections that encode priority by position. Split into change folders, some small and some with full PRDs and plans, that order disappears and has nowhere to go.
- **Dependencies hide in prose.** The solo-app example's status line says "offline-queue first; sync-badge needs it", in a Log entry. The monorepo example sets `blocked: waiting on services/payments/yass/changes/2026-09-10-saved-cards-api`, which someone has to remember to clear when that change lands.
- **Inside a large change, order already works.** `plan.md` says under `## Pieces` what order they go in and why. Between top-level changes there's nothing equivalent.

Popular project management tools solve this with two separate ideas: a manually ranked backlog, and "blocked by" relations. Order is a preference about the whole set and shifts as priorities shift; a dependency is a fact about one change and stays true however you reprioritize. YASS should borrow both concepts.

## Users and outcomes
For anyone directing work in a YASS repo (a person, or an agent put in charge of choosing), and anyone asking where things stand.

- Ranking work, or reordering it, is one edit to one file, and its diff reads as the decision.
- `yass status` answers "what's next?" without opening any change: the first change in order that isn't done and isn't waiting on another.
- A change that waits on another stops showing as waiting once that change is done, with nobody clearing anything.
- A repo that doesn't use either feature sees no difference.

## Requirements
- **R1** [M1] A yass folder can hold an optional `queue.md`: a markdown list of change folder names, top first, each optionally followed by ` — <note>`. Other lines (headings, prose) are allowed and ignored.
- **R2** [M1] `yass status` lists each yass folder's active changes in `queue.md` order, then the ones it doesn't list, by date as today.
- **R3** [M1] `yass status` warns about a `queue.md` entry that isn't an active change in that yass folder (unknown, archived, or a piece), and about one listed twice.
- **R4** [M1] `yass archive` removes the archived change's line from `queue.md` and stages that edit with the move.
- **R5** [M1] The hook treats editing an existing `queue.md` in the same commit as code as an intent change. Creating one is fine, as with other intent files.
- **R6** [M2] When `blocked:` holds one or more change names (comma-separated; a name can be a path ending in the change's folder, or `<change>/<piece>`), the change is waiting on them. It's waiting until each is done (every box checked) or archived. Any other value is a reason for a human, as today.
- **R7** [M2] `yass status` shows `waiting on: <change>` for a dependency that isn't met yet, and nothing once it's met. `yass status <change>` suggests clearing a `blocked:` whose dependencies are all met.
- **R8** [M2] `yass archive` refuses a change that's blocked by a reason or by an unmet dependency, and doesn't refuse one whose dependencies are met.
- **R9** [M2] `yass status` warns when a `blocked:` entry looks like a change folder (`<YYYY-MM-DD>-<slug>`) but no such change exists, and when dependencies form a cycle.
- **R10** [M1, M2] The README, the yass folder README template, the playbooks and the examples cover both: `yass-status` answers "what's next?" from the order and dependencies; `yass-work` sets `blocked: <change>` when work has to wait on another change; `yass-plan` can use it between pieces; the solo-app example has a `queue.md`, and the monorepo example's dependency uses the new form.

## Non-goals
- **Priority levels** (urgent/high/medium/low). Redundant once there's an order, and they inflate until everything is high.
- **A rank stored in each change.** Reordering would touch many folders and conflict on merge.
- **Estimates, points, due dates, cycles or sprints, assignees.**
- **Relation types beyond "waits on"** (relates to, duplicates). A Log line or a Decision covers them.
- **Ordering across yass folders.** Each yass folder ranks its own changes; cross-team order is the root change's plan, as today.
- **A CLI command to reorder.** Editing `queue.md` is the interface.
- **Picking work automatically.** `yass-work` still works only on the change it's pointed at; the order informs whoever directs the work.
- **Ordering pieces with `queue.md`.** A large change's `plan.md` already orders its pieces.

## Milestones
| Milestone | When it ships, a user can … |
|---|---|
| M1 Order | rank changes in `queue.md` and see `yass status` follow it |
| M2 Dependencies | point `blocked:` at another change and see it clear itself when that change is done |

## Open questions
None. The calls made while shaping are in `change.md` `## Decisions`.

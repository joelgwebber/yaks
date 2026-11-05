# yass/

Planned work for this part of the repo, managed with YASS (Yet Another Spec System). Code and its docs describe what exists today; this folder describes what's changing.

- `changes/`: active work, one dated folder per change. A small change is a `change.md`. A large one adds `prd.md` (why and what), `plan.md` (how and acceptance), an optional `design.md`, and PR-sized pieces as folders inside it.
- `archive/`: finished changes, moved here as they were, in `<YYYY>/<MM>/` folders by the month they were archived (older archives may sit directly in `archive/`). Never edited; follow-up work is a new change with `follows: <folder>`.
- `queue.md` (optional): the order to tackle changes in, top first, one folder name per list item. `yass status` follows it; changes it doesn't list come after, oldest first.

Rules:
1. **A change is a folder; ceremony scales with scope.**
2. **Progress travels with the code.** Ticking boxes and appending to Log and Decisions go in the same commits as the code.
3. **Intent changes get their own commit.** `prd.md`, `design.md`, `plan.md` beyond ticking boxes, and a change's title, Goal and Acceptance (including dropping a criterion).
4. **Done means every box is checked.** Then `yass archive <change>`, in its own commit.
5. **The archive is append-only,** except that `yass evict` moves its oldest months to git history when the folder is past its `keep:` setting.

`yass status` shows what's in flight.

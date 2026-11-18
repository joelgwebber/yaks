# Order and dependencies: plan
<!-- Intent: how, and how we'll know it's done. With code, only mark boxes here ([/] in progress, [x] done);
     anything else is a plan revision in its own commit. -->

## Approach
**Order.** `Root` gains the parsed `queue.md`: the folder names in list order. A line counts when it's a list item (`-`, `*`, `+` or `1.`) whose first word is a folder name; everything after it (a ` — note`) and every other line is ignored. Reading happens in `loadRepo` next to loading `changes/`; the checks for R3 go in `Repo.check()` beside the `follows:` check. `cmdStatus` sorts each root's items by queue position, unlisted ones after, keeping today's date order among them (a stable sort). Nothing new is printed: the order itself is the signal.

`cmdArchive` rewrites `queue.md` without the archived change's line and `git add`s it with the move, so the archive commit carries both. The hook's `where()` gets a `queue` kind for `<yass folder>/queue.md`, and `check()` flags a modified (not added) `queue.md` in a commit that has code, the same way it treats `prd.md`.

**Dependencies.** A helper on `Change` splits `blocked:` on commas and resolves each item by its last path segment (or `<change>/<piece>`) against every active and archived change in the repo, across yass folders, the way `follows:` already resolves. If every item resolves, the change depends on them, and `waitingOn()` returns the ones that aren't done or archived; otherwise `blocked:` is a reason, exactly as today. `statusLine`, `showChange` and `cmdArchive` switch from `blocked()` to that distinction. `check()` warns about dated-looking items that don't resolve, and walks dependencies for cycles.

**Docs and playbooks** move with each milestone's code, so each piece leaves the docs true.

## Acceptance
### M1
- [x] AC1 (R1, R2) Given a `queue.md` listing two changes in the reverse of their date order, when `yass status` runs, then they're listed in queue order, followed by the unlisted changes in date order — verify: tests/e2e.sh
- [x] AC2 (R1) Given a `queue.md` with a heading, prose, ordered and unordered items, and ` — note` suffixes, when `yass status` runs, then only the items' folder names affect the order and nothing warns — verify: unit test for the parser
- [x] AC3 (R2) Given no `queue.md`, when `yass status` runs, then the output is the same as before this change — verify: tests/e2e.sh and tests/examples.sh pass unchanged
- [x] AC4 (R3) Given a `queue.md` naming an unknown folder, an archived change, a piece, and one change twice, when `yass status` runs, then each gets a warning — verify: tests/e2e.sh
- [x] AC5 (R2) Given a repo with two yass folders, each with its own `queue.md`, when `yass status` runs, then each folder's changes follow its own file — verify: tests/e2e.sh
- [x] AC6 (R4) Given a ranked change that's done, when `yass archive` runs, then its line is gone from `queue.md`, the other lines are untouched, and the edit is staged with the move — verify: tests/e2e.sh
- [x] AC7 (R5) Given a commit with code that reorders an existing `queue.md`, when the hook runs, then it flags `queue.md` as intent; a commit that creates `queue.md` with code isn't flagged — verify: tests/e2e.sh
- [x] AC8 (R10) Given the README, the yass README template, `yass-status` and the solo-app example, when someone reads them, then they describe `queue.md` and `yass-status` uses it to answer "what's next?" — verify: manual: read them; tests/examples.sh passes

### M2
- [x] AC9 (R6, R7) Given change B with `blocked: <A's folder>` and A in progress, when `yass status` runs, then B shows `waiting on: <A>` — verify: tests/e2e.sh
- [x] AC10 (R6, R7) Given A then becomes done, or is archived, when `yass status` runs, then B shows no block, and `yass status B` suggests clearing `blocked:` — verify: tests/e2e.sh
- [x] AC11 (R6) Given `blocked:` naming two changes, as a path ending in a folder and as `<change>/<piece>`, when only one is done, then B is still waiting on the other — verify: tests/e2e.sh
- [x] AC12 (R6) Given `blocked:` with free text, including text that mentions a change, when `yass status` runs, then it shows `BLOCKED: <text>` as today — verify: tests/e2e.sh
- [x] AC13 (R8) Given B's boxes all done, when `yass archive B` runs, then it refuses while A isn't done and succeeds once A is — verify: tests/e2e.sh
- [x] AC14 (R9) Given `blocked:` naming a dated folder that doesn't exist, or A and B blocked on each other, when `yass status` runs, then it warns — verify: tests/e2e.sh
- [x] AC15 (R10) Given the README, `yass-work`, `yass-plan`, `yass-status` and the monorepo example, when someone reads them, then they describe `blocked: <change>`, and the monorepo example uses it — verify: manual: read them; tests/examples.sh passes

## Pieces
Order first, then dependencies. They're independent, but both change how `yass status` builds its lines, and order is the simpler of the two, so it settles the shape of that code first. Each piece ships its own docs and playbook edits.

## Validation
`go vet`, `go test ./...`, `tests/e2e.sh` and `tests/examples.sh` on Linux and macOS (CI). Then this repo's own `yass status` with a real `queue.md`, ranking the changes that are open at the time.

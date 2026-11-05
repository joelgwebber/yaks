---
platforms: [all]
source: CI run 37235844602 (macos-latest)
follows: 
blocked:
archived: 2026-10-04T21:29:48Z
---
# Paths that don't exist yet compare through symlinks

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
A folder that doesn't exist yet compares correctly with the repo it's inside, even when a parent folder is a symlink (macOS's /var → /private/var), so yass init --private and other checks work in any temp or home layout.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Given a path that doesn't exist yet under a symlinked folder, when YASS makes it canonical, then the symlink is resolved and the rest of the path kept — verify: go test TestCanonResolvesMissingPathsThroughSymlinks
- [x] Given the e2e suite run in macOS's default temp folder (under /var), then it passes, including `yass init private --private` (§22) — verify: tests/e2e.sh with no workdir, on macOS

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `canon` resolves the deepest existing folder of a missing path
- [x] §22's checks match the plans path by its tail, not the unresolved temp path

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-04 (claude)
- Did: the first push of the private-team-folders stack failed on GitHub's macOS runner (run 37235844602, §22: "--private needs a folder inside the repo"). `canon` only resolved symlinks for paths that exist, so the new `private/` stayed under /var while the repo top resolved to /private/var. Local runs used a scratch folder without symlinks and missed it. Reproduced with tests/e2e.sh in the default temp folder, fixed, and checked both ways: e2e 279/279 in each; go test, go vet, examples.
- Next: archive once the fix's CI run on main passes.

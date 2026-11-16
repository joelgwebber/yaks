---
platforms: [all]
source: 
follows: 
blocked:
archived: 2026-10-04T02:10:24Z
---
# Ignore folders from yass.yaml

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
A yass.yaml can list folders whose yass/ folders and yass.yaml files belong to something else (examples, fixtures, vendored projects), so they stay out of this project's status, new and hook.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Given a yass.yaml with `ignore:` listing folders or glob patterns (`examples/*`), when `yass status` or `yass root` runs from outside `examples/`, then no yass folder or yass.yaml under a matched folder is listed — verify: tests/e2e.sh
- [x] Given an ignored folder, when the hook checks a commit that edits a change or the archive inside it, then those files count as ordinary files, not YASS intent or archive — verify: tests/e2e.sh
- [x] Given the current folder is inside an ignored folder, when `yass status` or `yass new` runs, then that folder is treated as its own project: its yass folders are found and the outer ones aren't — verify: tests/e2e.sh
- [x] Given an `ignore:` entry outside the yass.yaml's folder, naming that folder itself, or naming a folder that doesn't exist, when `yass status` runs, then it warns — verify: tests/e2e.sh
- [x] Given a yass.yaml without `path:` next to a `yass/` folder, when `yass status` runs, then `yass/` is the root, the settings apply, and nothing warns — verify: tests/e2e.sh
- [x] Given the README, when someone looks up yass.yaml, then it documents `ignore:` and a yass.yaml that sits next to a `yass/` folder only to hold settings — verify: read README.md
- [x] Given this repo, when `yass status` runs from the top, then only this project's `yass/` shows, not the examples — verify: manual

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `ignore:` in `Config`, resolved by `ignoredDirs` (config.go)
- [x] `findRoots` drops ignored folders, settling nested yass.yaml files shallowest first, and treats an ignored folder you're standing in as the top (repo.go)
- [x] The hook sees the repo from the top and treats ignored files as ordinary files (hook.go)
- [x] e2e section 10
- [x] README: "Folders that aren't yours", and a yass.yaml that only holds settings
- [x] This repo's `yass.yaml` ignores `examples/*`

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- Standing inside an ignored folder makes it the top, a project of its own - examples stay browsable with `cd examples/solo-app && yass status`, and the outer project's changes don't leak in (claude)
- Entries are folders or glob patterns (`examples/*`) - with plain folders, `examples` would make all the examples one project from inside; a pattern makes each its own without listing them (claude)
- Only folders inside the yass.yaml's folder can be ignored; a yass.yaml inside an ignored folder doesn't count, and neither do its ignores - keeps ignores local and order-independent (claude)
- An entry that matches nothing warns - catches typos; ignoring a folder that might appear later isn't worth a silent typo (claude)
- The hook always discovers from the top - git's paths are relative to it, so running it from inside an ignored folder must not move the top (claude)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->

### 2026-10-03 (claude)
- Did: built `ignore:` (folders and glob patterns), the hook's handling, e2e section 10 (188 passing), README docs; this repo's yass.yaml ignores `examples/*`.
- Next: review, then archive.

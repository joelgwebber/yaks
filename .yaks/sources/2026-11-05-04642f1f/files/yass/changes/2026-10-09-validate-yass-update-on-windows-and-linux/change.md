---
platforms: [all]
source: Jaime, 2026-10-09: AC12 and AC17 of yass-update were verified on macOS only; neither Windows nor Linux was at hand
follows: 2026-10-08-yass-update-the-binary-updates-itself
blocked:
created: 2026-10-09T03:02:25Z
---
# Validate yass update on Windows and Linux

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
A real yass update runs on Windows and on Linux against the published GitHub releases, the way AC12 and AC17 of the archived yass-update change were verified on macOS, so all three platforms releases are built for have been checked by hand.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [ ] Given a published release of 0.4.0 or later installed from its `.zip` on Windows, and a newer release published, when you run `yass update`, then it installs the newer one with its checksum (and provenance, with `gh` signed in) verified, `yass version` reports it, the old binary sits beside it as `yass.exe.old`, and the next `yass` command removes that — verify: manual, on Windows (amd64 or arm64)
- [ ] Given a published release installed on Linux, when you run `yass update --check` and then `yass update` against github.com, then they behave as AC17 of the archived change did on macOS: the right versions, verification reported, the binary swapped and identical to the published one — verify: manual, or a CI job on a Linux runner

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [ ] Easiest once a release after 0.4.0 is out: install 0.4.0 from its release download, then run `yass update`

## Decisions
<!-- Progress. "- <decision> - <why> (<who>, <YYYY-MM-DD>)", appended as you go. -->

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->

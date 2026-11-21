---
type: source
title: 'reno (OpenStack): design constraints and usage'
description: Saved copy of https://docs.openstack.org/reno/latest/user/design.html
media_type: text/markdown
size: 2189
sha256: 2cb3322df50ce6fe4ae199cf14c5f655b0653820685e7d214bf9ccf2d533497b
sources:
  - id: web
    resource: https://docs.openstack.org/reno/latest/user/design.html
    last_modified: 2026-11-21T21:15:25Z
    retrieved: 2026-11-21T21:15:48Z
generated: {by: herd-files, at: 2026-11-21T21:15:48Z}
---

# reno: design constraints and usage (OpenStack, excerpt)

## Design inputs

- Release notes should be part of the git history, so as fixes in master are back-ported to older branches the notes can go with the code change.
- Release notes may need to change over time, as typos are found, logical errors or confusing language needs to be fixed, or as more information becomes available (CVE numbers, etc.).
- Release notes should be peer-reviewed, as with other documentation and code changes.
- Notes are mutable in that a clone today vs a clone tomorrow might have different release notes about the same change.
- Notes are immutable in that for a given git hash/tag the release notes will be the same.
- We want to avoid merge issues when shepherding in a lot of release-note-worthy changes.
- We must not make things progressively slow down to a crawl over years of usage.
- Release note authors shouldn't need to know any special values for naming their notes files (no change id or SHA value that has special meaning).
- Release notes should be grouped by type: features, known issues, upgrade notes, security fixes, bug fixes, other.

## Assumptions

reno expects trunk-based development: fixes land on master and are cherry-picked to stable branches. It is branch-based, not release-based, and does not support git-flow (stable branches merged back into master).

## Usage

`reno new <slug>` writes `releasenotes/notes/<slug>-<16-hex uid>.yaml` with sections `prelude`, `features`, `issues`, `upgrade`, `deprecations`, `critical`, `security`, `fixes`, `other`. `reno report` scans the branch's git history (topological order) and assigns each note to the release whose tag first contains it; `--branch` reports another branch.

## Fixing release notes

To fix a note, edit the file in place and commit. Because reno tracks notes by their unique ID, it pulls the updated contents from the newest commit while retaining the note's association with the release where it was first introduced (since 2.6.0). A fix to a note in a past stable release goes to that branch. A note can be excluded with `ignore_notes`. `reno semver-next` computes the next version from the sections present.

---
type: source
title: 'Changesets: a detailed explanation, and adding a changeset (v2 docs)'
description: Saved copy of https://github.com/changesets/changesets/blob/maintenance/v2/docs/detailed-explanation.md
media_type: text/markdown
size: 1782
sha256: 435c0b836d8be8627c7190a1bd24bf5e82dced0c27221ac5399406a028c1b0e6
sources:
  - id: web
    resource: https://github.com/changesets/changesets/blob/maintenance/v2/docs/detailed-explanation.md
    last_modified: 2026-11-21T21:15:25Z
    retrieved: 2026-11-21T21:15:47Z
generated: {by: herd-files, at: 2026-11-21T21:15:47Z}
---

# Changesets: a detailed explanation (v2 docs, excerpt)

## The problem

When organising the release of packages, you may end up wanting to group several changes together written by different people and/or over a relatively large period of time. The best time to capture this information is when submitting a PR (when it is fresh in your mind), not when you eventually go to batch and release these changes.

Git is a bad place to store this information, as it discourages writing detailed change descriptions - you want to allow people to provide as much documentation for the change as they want.

## The solution

A changeset is an 'intent to change'. It carries versioning ('major' | 'minor' | 'patch') and changelog information, stored as a markdown snippet in `.changeset/UNIQUE_ID.md`: YAML front matter naming the packages and bump types, and a markdown body that is the change summary written to the changelog.

This breaks versioning into two steps:

1. Adding a changeset - can be done in a PR, by a contributor, while the change is fresh in their mind.
2. Versioning - combines all changesets, creates one version bump for each package based on the maximum version bump of each package, updates dependencies where needed, and writes changelogs. Can then be reviewed as an aggregate.

## The tooling that makes this worthwhile

1. CLI generation of new changesets.
2. Automated consumption of changesets to do versioning.
3. Detection + surfacing of changesets in PRs.

## Adding a changeset (from adding-a-changeset.md)

A good changeset says WHAT the change is, WHY the change was made, and HOW a consumer should update their code. You can write as much markdown as you want, which will all be added to the changelog on publish. Changesets stack: a PR may add several.

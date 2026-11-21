---
type: source
title: 'GitHub: automatically generated release notes'
description: Saved copy of https://docs.github.com/en/repositories/releasing-projects-on-github/automatically-generated-release-notes
media_type: text/markdown
size: 516
sha256: f66f0491c68e9bb0c6a41621c97b737673e98dfe6d1ba743ef686a3e73e84042
sources:
  - id: web
    resource: https://docs.github.com/en/repositories/releasing-projects-on-github/automatically-generated-release-notes
    last_modified: 2026-11-21T21:15:25Z
    retrieved: 2026-11-21T21:15:49Z
generated: {by: herd-files, at: 2026-11-21T21:15:49Z}
---

# GitHub: automatically generated release notes (docs, excerpt)

Automatically generated release notes include a list of merged pull requests, a list of contributors to the release, and a link to a full changelog. They are generated between the release's tag and a chosen previous tag. `.github/release.yml` groups pull requests into categories by label (`changelog.categories[*].labels`, `*` as a catch-all) and excludes labels or authors (`changelog.exclude`). Each entry is a PR title with its author and number.

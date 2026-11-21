---
type: source
title: Keep a Changelog 1.1.0
description: Saved copy of https://keepachangelog.com/en/1.1.0/
media_type: text/markdown
size: 2259
sha256: 9a1c601ce047a38261d3ac4c75ab6d67bc8733e4eda1a25cc1f8006c3b73cc43
sources:
  - id: web
    resource: https://keepachangelog.com/en/1.1.0/
    last_modified: 2026-11-21T21:15:25Z
    retrieved: 2026-11-21T21:15:47Z
generated: {by: herd-files, at: 2026-11-21T21:15:47Z}
---

# Keep a Changelog 1.1.0 (excerpt)

> Don't let your friends dump git logs into changelogs.

A changelog is a file which contains a curated, chronologically ordered list of notable changes for each version of a project.

## Guiding Principles

- Changelogs are _for humans_, not machines.
- There should be an entry for every single version.
- The same types of changes should be grouped.
- Versions and sections should be linkable.
- The latest version comes first.
- The release date of each version is displayed.
- Mention whether you follow Semantic Versioning.

## Types of changes

- `Added` for new features.
- `Changed` for changes in existing functionality.
- `Deprecated` for soon-to-be removed features.
- `Removed` for now removed features.
- `Fixed` for any bug fixes.
- `Security` in case of vulnerabilities.

## How can I reduce the effort required to maintain a changelog?

Keep an `Unreleased` section at the top to track upcoming changes. People can see what changes they might expect in upcoming releases; at release time, you can move the `Unreleased` section changes into a new release version section.

## Commit log diffs

Using commit log diffs as changelogs is a bad idea: they're full of noise. Things like merge commits, commits with obscure titles, documentation changes, etc. The purpose of a commit is to document a step in the evolution of the source code. The purpose of a changelog entry is to document the noteworthy difference, often across multiple commits, to communicate them clearly to end users.

## Inconsistent Changes

A changelog which only mentions some of the changes can be as dangerous as not having a changelog. [...] having a good changelog means having a consistently updated changelog.

## What about GitHub Releases?

GitHub Releases create a non-portable changelog that can only be displayed to users within the context of GitHub. [...] arguably not very discoverable by end-users, unlike the typical uppercase files (`README`, `CONTRIBUTING`, etc.).

## Should you ever rewrite a changelog?

Sure. There are always good reasons to improve a changelog. [...] you may discover that you forgot to address a breaking change in the notes for a version.

## Yanked releases

`## [0.0.5] - 2014-12-13 [YANKED]`

---
type: source
title: git-cliff docs and Conventional Commits 1.0.0
description: Saved copy of https://git-cliff.org/docs/
media_type: text/markdown
size: 1097
sha256: 2fffc71b511a3dbc99b8055c04eea70353d4b92c2b4c886e7a7dc829b23be9f4
sources:
  - id: web
    resource: https://git-cliff.org/docs/
    last_modified: 2026-11-21T21:15:25Z
    retrieved: 2026-11-21T21:15:49Z
generated: {by: herd-files, at: 2026-11-21T21:15:49Z}
---

# git-cliff and Conventional Commits (excerpt)

## git-cliff (docs, v2.14)

git-cliff generates changelog files from the Git history by utilizing conventional commits as well as regex-powered custom parsers; a `cliff.toml` template shapes the output (`git-cliff -o CHANGELOG.md`). It can also pull remote metadata (pull request titles, numbers, labels) from Git hosts. It recommends **squash merges** so that each PR is one coherent commit: other merge strategies "may fail to consistently associate PR-level context (e.g., title, labels, issue references) with a single commit".

## Conventional Commits 1.0.0

`<type>[optional scope]: <description>`, optional body, optional footers in git-trailer form. `fix:` correlates with a SemVer PATCH, `feat:` with MINOR, `!` or a `BREAKING CHANGE:` footer with MAJOR. Stated uses: automatically generating changelogs, determining a version bump, communicating the nature of changes. The FAQ's fix for a wrong type before release is `git rebase -i`; for contributors who don't follow it, a squash workflow where maintainers rewrite the message on merge.

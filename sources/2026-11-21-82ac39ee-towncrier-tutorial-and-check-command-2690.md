---
type: source
title: Towncrier tutorial and check command (26.9.0)
description: Saved copy of https://towncrier.readthedocs.io/en/stable/cli.html
media_type: text/markdown
size: 1030
sha256: 82ac39ee7e3786f1db8731b12babf3099ab05b0ae9b9b184efb6e666fe6895e4
sources:
  - id: web
    resource: https://towncrier.readthedocs.io/en/stable/cli.html
    last_modified: 2026-11-21T21:15:41Z
    retrieved: 2026-11-21T21:15:48Z
generated: {by: herd-files, at: 2026-11-21T21:15:48Z}
---

# Towncrier tutorial (26.9.0, excerpt)

`towncrier` builds a news file (changelog) from **news fragments** in a directory (default `newsfragments/`). A fragment's file name is the issue/ticket ID plus its type: `1234.bugfix`. Default types: `feature`, `bugfix`, `doc`, `removal`, `misc` (misc lists only the issue ids, not the text). Fragments with the same content and different ids are merged: `(#3456, #7890)`. An orphan fragment with no issue starts with `+`: `+anything.bugfix`.

`towncrier build --draft --version 1.0.2` previews. The real build **removes the fragments (`git rm`)**, appends the rendered release to the news file (below a `<!-- towncrier release notes start -->` marker in markdown) and stages both; committing is left to the user.

`towncrier check` checks that a feature branch adds at least one news fragment, comparing against `origin/main` (`--compare-with` another); it is skipped when the branch edits the news file itself, which signals a release branch. `--keep` builds without deleting fragments.

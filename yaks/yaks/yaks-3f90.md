---
id: yaks-3f90
title: 'Commits phase 2: commit links in the TUI and a commit view in the pane'
status: shaving
type: feature
priority: 2
created: '2026-11-18T03:03:09Z'
updated: '2026-11-18T14:40:32Z'
parent: yaks-3515
depends_on:
- yaks-274c
tags:
- tui
- git
---

Phase 2 of yaks-3515; needs phase 1 (yaks-274c).

- **Note fields as links:** a note's `commit:` and `source:` values render as short-hash links in its header, with the full hash kept in the link.
- **Commit view:** Enter on a commit link opens a new pane view (beside the text, image and directory views) showing the subject, body, author, date and changed files with `+/-` counts. Enter on a file opens it in the code viewer at that commit. `O` takes the commit out: `g` the host URL, `p` the hash. History works.
- **Squashed worker commits:** a worker hash that is the `source:` of a landed note says "landed as <sha>" and links to it.
- **Missing commits:** a commit not in the local repo says so, and offers the host link and `yaks commits --fetch`.
- Keys go through `BINDINGS`. Update `docs/tui.md` and the `?` help, with a docshot if one fits.

#### 2026-11-18T14:40:32Z

*sources-triage · kind: move · from: hairy · to: shaving · thread: ksQQgIRwKupoSJu4WL5a-Y3Ss5LOAGwQr5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

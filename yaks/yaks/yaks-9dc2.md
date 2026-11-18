---
id: yaks-9dc2
title: check flags a CommonMark email autolink (<a@b.com>) as a broken farm link
status: hairy
type: bug
priority: 3
created: '2026-11-18T14:42:36Z'
updated: '2026-11-18T14:42:36Z'
tags:
- cli
- doks
- links
source: 'feedback-to-yaks:cowpaths/joel#2026-11-12: `yaks check` flags an email autolink as a broken link'
---

From cowpaths/joel's feedback dok (2026-11-12 entry). `<nancy.ung@braze.com>` in a saved report is a valid CommonMark email autolink; `yaks check` reports it under "Broken link ... no such file in the farm", and every tidy round there lists it as left alone.

Reproduced on 0.0.38 in a scratch farm: a dok containing `<nancy@example.com>` gives `doks/e.md:L1: (nancy@example.com) no such file in the farm`.

Cause: `src/links.rs` treats a destination as external only when it contains `://` or starts with `mailto:`. CommonMark's email autolink (`<local@domain>`, no scheme) is a mail link too. Fix where autolinks are parsed, so the TUI's link following agrees.

Evidence: a `check` test with an email autolink and no finding; a `mailto:`-less address still flagged when it is written as `[x](a@b.com)`, if that is the rule chosen.

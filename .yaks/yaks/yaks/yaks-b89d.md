---
id: yaks-b89d
title: Format src/actor.rs and tests/commits.rs once so cargo fmt stops rewriting them
status: hairy
type: task
priority: 2
created: '2026-11-10T17:53:07Z'
updated: '2026-11-10T17:53:07Z'
tags:
- meta
---

Every session note so far reports `cargo fmt -- <files>` rewriting `src/actor.rs` and `tests/commits.rs`, and every worker reverts them by hand, though `AGENTS.md` warns about it: a lesson already in context that still recurs (yaks-2b40, point 4). The mechanism fix: format the two files once on `main`, and drop the warning from `AGENTS.md` and from briefs.

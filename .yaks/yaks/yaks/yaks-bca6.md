---
id: yaks-bca6
title: 'Design: session notes in a farm (narrative plus references to yak notes)'
status: hairy
type: task
priority: 2
created: '2026-11-06T04:28:29Z'
updated: '2026-11-06T04:28:29Z'
parent: yaks-1c89
tags:
- design
- doks
---

Settle the shape of a session note in a farm before building the command (yaks-1c89).

Hearth's note (`hearth-session-notes`) has four sections: Summary, Turns (one verdict per request: good, partial, failed, unclear), Saved, Lessons. It lives at `raw/sessions/YYYY/MM/local-<person>-<host>-<day>-<time>-<title>.review.md` with `type: session-review`, and a delegated sub-session writes none: it reports friction to its parent instead.

In a farm, much of a session's work is already recorded in yak notes and shear notes. The design here: a session note points at those rather than repeating them.

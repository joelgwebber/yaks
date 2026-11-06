---
id: yaks-323b
title: 'session list --since <date>: a reflection round''s notes, reviewed or not'
status: hairy
type: feature
priority: 3
created: '2026-11-06T23:46:33Z'
updated: '2026-11-06T23:46:33Z'
parent: yaks-1c89
depends_on:
- yaks-3f35
tags:
- cli
---

For the reflection round (yaks-7168, /doks/design/session-reflection.md): `yaks session list --since <date>` lists the session notes whose `day` is on or after the date, reviewed or not. Today an agent filters `session list --json` by `day` itself; the skill says so until this lands. Waits on yaks-3f35, whose children change `src/sessions.rs`.

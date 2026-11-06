---
id: yaks-2e00
title: 'refs --incoming on a yak: include doks and session notes'
status: hairy
type: feature
priority: 3
created: '2026-11-06T04:57:49Z'
updated: '2026-11-06T04:57:49Z'
parent: yaks-1c89
tags:
- cli
---

`yaks refs <id> --incoming` reads only yaks (`backrefs_in` in src/farm.rs). Doks and session notes link yaks too, now often to a single note, and a session-note review wants to see them. Today `yaks find <id> --kind sessions` stands in. Found while building yaks-17f5.

---
id: yaks-1b39
title: 'yaks lanes: ahead/behind are unknown (?) when there is no merge-base'
type: bug
priority: 3
created: '2026-10-05T19:48:22Z'
updated: '2026-10-05T19:48:22Z'
parent: yaks-38dc
labels:
- cli
- delta
---

Seen on Joel's laptop (2026-10-04): the primary checkout was a shallow repository, so git found no merge-base with a lane at 4fc959e and 'yaks lanes' printed +374 -3 (the size of the whole history) and fell back to '(vs this checkout)' for the farm. The fallback is right; the counts are misleading. When no merge-base is found (shallow history, unrelated histories, unknown lane commit) print '?' for AHEAD/BEHIND (JSON null) and keep the existing vs-checkout note. Test: a shallow clone (git clone --depth 1) as the primary with a lane whose HEAD is older than the boundary. Docs parity for the lanes row.

---
id: yaks-e985
title: Ignore stray root package.json/package-lock.json from a bare npm install
type: chore
priority: 3
created: '2026-09-27T19:11:55Z'
updated: '2026-09-27T19:12:03Z'
labels:
- npm
---

---
▸ 2026-09-27T19:12:03Z [Joel Webber]
Root package.json ({}) + package-lock.json (empty packages) dated 2026-09-23 22:41, ~11m after the v0.0.11 release commit; never modified since. Nothing in the repo runs npm at the root (release is CI; build-npm.mjs writes dist/npm/). Contents are the fingerprint of a one-off bare 'npm install' at the root. Deleted the stubs and gitignored /package.json + /package-lock.json (root-anchored, so npm/*/package.json stay tracked). Verified: a recreated root package.json shows as ignored (!!).

---
type: session-review
title: Close sources umbrella, release 0.0.38
source: session-note
actor: sources-triage
person: Joel Webber
host: joel
harness: delta
thread: ksQQgIRwKupoSJu4WL5a-Y3Ss5LOAGwQr5IBxBCA0aLJyrhJPreUvVwLftHG
day: 2026-11-18
written: '2026-11-18T01:00:26Z'
good: 2
---

## Summary

Joel had migrated all his farms to schema 10 cleanly. I closed the last child, [yaks-5f14](/yaks/yaks/yaks-5f14.md) (`source set --text`), and the umbrella yaks-4cfe, then landed `main` and tagged v0.0.38 for release.

## Turns

- good: wrap up yaks-5f14 and close yaks-4cfe. `source set` grew `--text` (body replaced, `text_updated` stamped, provenance kept), with a test, a real run and docs.
- good: land and cut a release. 15 commits pushed to `origin/main`, tag `v0.0.38`, release workflow started.

## Saved

- yaks-4cfe: Sources: one coherent layout for text, provenance, directories and browsing (shaving -> shorn)
  - [Every child is shorn (`sources-triage`): yaks-5f14 closed with `source set --...](/yaks/yaks/yaks-4cfe.md#2026-11-18t005424z)
- yaks-5f14: source: add or replace extracted text for an existing source (hairy -> shorn)
  - [Done (`sources-triage`): `yaks source set <id> --text <file|->`. It extends t...](/yaks/yaks/yaks-5f14.md#2026-11-18t005423z)

## Lessons

- Extending an existing command (`source set`) beat adding a new verb: one write path, one lock, one commit, and the docs say one thing.

---
id: yaks-86f4
title: 'Flat farm, phase 2: yaks migrate + doctor/preflight checks'
type: task
priority: 1
created: '2026-10-12T19:53:57Z'
updated: '2026-10-12T19:53:57Z'
parent: yaks-8b7a
depends_on:
- yaks-95be
labels:
- cli
---

Phase 2 of yaks-8b7a (plan note: D5, D8; runs after phase 1, yaks-95be). Sonnet worker.
Scope: a `yaks migrate [--dry-run]` command (src/main.rs: one new `Command` variant and its dispatch; src/farm.rs: `Farm::migrate`), the doctor and preflight integrity checks (an id in both layouts replaces "two status dirs"; an invalid `status:` value), and the docs for exactly these: docs/cli.md rows (`migrate`, `doctor`, `preflight`, `init`), README.md rows, and AGENTS.md's first invariant (status is now a frontmatter field; one file per yak at `.yaks/<id>.md`).
Migrate: under the farm lock; refuse, listing the ids, when any id exists in both layouts; move each legacy `<status>/<id>.md` to `<id>.md` with `status:` set; remove the emptied status dirs (never a non-empty one); write `schema` 4; print one summary line and the suggested commit. `--dry-run` writes nothing and prints what would move. Idempotent on a v4 farm ("already flat"). Works on a private or pointer farm (no git involved).
Out of scope: src/store.rs (phase 1, done), src/changes.rs/status.rs/commit.rs and src/sheds.rs (other workers), skills (coordinator).
Evidence (fail first): a v3 fixture migrates to flat with every status preserved and notes byte-identical; a collision refuses and changes nothing; dry-run changes nothing; doctor flags a hand-made both-layouts farm.

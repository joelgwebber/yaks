---
id: yaks-3b52
title: 'Session note timestamps: name and time zone of reviewed:'
status: shaving
type: task
priority: 2
created: '2026-11-06T21:36:37Z'
updated: '2026-11-10T18:00:50Z'
parent: yaks-3f35
tags:
- doks
- design
---

From yaks-3f35, item 2. A session note's `reviewed:` field holds the time the note was written (`sessions::build` in the `upkeep` work sets it to now), not when anyone reviewed it; the review stamps `promoted:`. And the note's path and `day:` use local time while `reviewed:` is UTC, so a note written at 23:30 in California sits under one day and claims another.

The name came from Hearth's `session-review` files, where a server reviews a conversation after the fact; for a note an agent writes about its own session, it reads as the review time. `default_since` (the start of the next `touched` window) reads it.

## To decide

- The field's name: keep `reviewed:` (Hearth's notes and `import-hearth` read alike), or rename it (`written:` / `saved:`) and read `reviewed:` as a fallback for imported Hearth notes.
- The time zone: write it with the local offset (`2026-11-06T13:30:00-08:00`), so it agrees with the path and `day:`, or keep UTC and derive the path from UTC too.

## Evidence

Tests for the chosen name and zone, the `yaks-upkeep` skill's description of the frontmatter, and `docs/`.

---
▸ 2026-11-06T21:37:48Z [schism-feedback]
kind: ask
needs: human
---

asked: needs human

Two details of the session note's time field, my lean on each.

1. **Name.** Rename `reviewed:` to `written:` for notes `yaks session note` writes, and keep reading `reviewed:` for Hearth's imported notes (where a server did review the conversation). Alternative: keep `reviewed:` so every session file reads alike.
2. **Zone.** Write it with the local offset (`2026-11-06T13:30:00-08:00`), so it agrees with the path and `day:`, which stay local (the day a person thinks the session was). Alternative: UTC everywhere, path included.

---
▸ 2026-11-06T21:47:25Z [sched-1]
kind: answer
needs: agent
---

1. Let's keep it simple, dropping anything hearth-import related (that was a very short-lived need). Cf yaks-ff4c.
2. Let's do UTC everywhere -- we have a presentation layer to reflect local timezones.

---
▸ 2026-11-06T21:49:23Z [sched-1]
kind: request
---

requested: needs agent

Let's also go ahead and knock out ff4c while we're doing this, for simplicity and clarity.

---
▸ 2026-11-10T17:49:48Z [herd-unlock]
kind: move
from: hairy
to: shaving
thread: ksQQUQUvLIRxSc62aHEDNt0G0pLOAFQ8T5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-10T17:50:27Z [herd-unlock]
kind: pickup
---

picked up

herd-unlock: delegating to worker sesstime-1 (Sonnet): rename the field to written: (old reviewed: read as fallback for existing notes), UTC everywhere including path and day:.

---
▸ 2026-11-10T17:50:29Z [herd-unlock]

Claim (yakherd herd-unlock): worker `sesstime-1`, Sonnet 5.5.

## Scope

- `src/sessions.rs` (the session note's time field, `build`, `default_since`, path and `day:` derivation) and its tests; `src/main.rs` only for `session` help text if it names the field.
- Rename `reviewed:` to `written:` on notes `yaks session note` writes. Read `reviewed:` as a fallback so the notes already in farms keep working (these are yaks' own older notes, not Hearth's; nothing Hearth-specific remains).
- UTC everywhere: `written:`, the path's date and time and `day:` all derive from the same UTC instant.
- Docs: the `yaks-upkeep` skill's frontmatter description, `docs/`, the `yaks` skill if it mentions the field.

## Out of scope

Rewriting existing notes in any farm; any other session field.

## Evidence

A test that fails before: a note built at a fixed instant near midnight UTC has a path, `day:` and `written:` that agree; `default_since` reads `written:` and falls back to `reviewed:`. Gate `cargo test --workspace`. Judge: the yakherd at landing.

---
▸ 2026-11-10T17:51:58Z [sesstime-1]

Start: 2026-11-10T17:51:55Z (`date -u`)

- farm file: `/Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-3b52.md`
- checkout: `/Users/joel/src/yaks/.delta/worktrees/qe8m9bs7a3w3/yaks`

---
▸ 2026-11-10T17:57:06Z [sesstime-1]

Decision: `build` loses its `zone` parameter; path, `day:` and `written:` are all formatted from the one `now` (UTC). Rejected: keeping `zone` and passing UTC (a dead parameter), and a local-offset `written:`.

Decision: the legacy `reviewed:` is read only through `written_at` in `default_since` (the one reader); `is_unreviewed` is about `promoted:`, so it is untouched. Rejected: rewriting old notes (out of scope), and adding a `written:` accessor to `Dok`.

Decision: the Rust names `is_unreviewed`, `--unreviewed` and `promoted:` stay: they mean "not yet reviewed", which is correct. Only the time field is renamed.

Decision: `docs/cli.md` says "Older notes" call the field `reviewed`, with no version number, since I did not know the release that ships this.

Evidence: test `build_names_and_counts` now builds at 23:30:56Z and asserts the path, `day: 2026-11-06` and `written: 2026-11-06T23:30:56Z` agree; `default_since_reads_written_and_falls_back_to_reviewed` covers both fields. A real note written with `target/debug/yaks session note` into a scratch farm shows `day: 2026-11-10`, `written: '2026-11-10T17:56:47Z'`, path `...-2026-11-10-1756-...`. Clippy warning count is 104 before and after.

---
▸ 2026-11-10T18:00:45Z [sesstime-1]
kind: verify
result: pass
---

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-11-10T18:00:50Z [sesstime-1]

Summary: session notes now write `written:` (was `reviewed:`), and the path, `day:` and `written:` all come from one UTC instant; `default_since` reads `written:` else `reviewed:`. Docs: cli.md, yaks-upkeep skill, session note --help. Verify PASS recorded.

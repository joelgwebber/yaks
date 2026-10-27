---
id: yaks-075c
title: Discourage inline enumerations in notes; let tidy requests make small connective edits
status: shorn
type: task
priority: 3
created: '2026-10-27T19:02:24Z'
updated: '2026-10-27T19:04:50Z'
labels:
- markdown
- skills
---

---
▸ 2026-10-27T19:02:28Z [delta:Check yaks-6beb comment list formatting]

From the yaks-6beb review: its Decision note is valid CommonMark, but it packs about six points into one paragraph, including an inline "X (option D), or Y (option A)" enumeration.

Two fixes:

- The skill's Shape rule and the brief's notes line name inline enumeration as a failure.
- The tidy request's instruction allows minimal connective edits, so a run-on enumeration can become a list.

---
▸ 2026-10-27T19:02:36Z [delta:Check yaks-6beb comment list formatting]

moved: hairy -> shaving

---
▸ 2026-10-27T19:04:46Z [delta:Check yaks-6beb comment list formatting]

Landed in the code commit above (not pushed): the skill's Shape rule and the brief's notes line now name inline enumeration as a failure, and the canned tidy instruction lets a tidy split an inline enumeration or a many-point paragraph into a lead-in and a list, changing only connecting words.

Evidence:

- `cargo test`: all suites pass (the tidy-text assertion in `tests/needs_loop.rs` updated).
- With the built binary in a scratch farm, `yaks request <id> --kind tidy` writes the new instruction, and `yaks brief` prints the new notes line.
- Surfaces updated together: `.agents/skills/yaks/SKILL.md`, `src/brief.rs`, `src/store.rs` (instruction text), `src/main.rs` (`--kind` help), `docs/cli.md`.

Existing queued tidy requests (yaks-8dbb's batch) still carry the old "never change a word" text; only new requests get the new one.

---
▸ 2026-10-27T19:04:50Z [delta:Check yaks-6beb comment list formatting]

moved: shaving -> shorn

---
id: yaks-f347
title: 'yaks answer <id>@<shed>: answer an ask in another shed''s copy (the one cross-shed write)'
type: feature
priority: 3
created: '2026-10-09T20:44:57Z'
updated: '2026-10-09T20:52:42Z'
parent: yaks-a3d2
labels:
- cli
- delta
---

---
▸ 2026-10-09T20:44:58Z [delta:Delta :Yaks (cont'd)]
moved: hairy -> shaving

---
▸ 2026-10-09T20:44:58Z [delta:Delta :Yaks (cont'd)]
Claim (coordinator). Owner: ans-1.
Goal: `yaks answer <id>@<shed> --note "<reply>"` (and `--done`, same as plain answer) records the answer in THAT shed's copy of the yak, not in this checkout's. Decided by Joel (yaks-ee0a, 2026-10-06): the answer is written into the asking shed's working tree and left UNCOMMITTED there (the worker commits it with its own yak file); only `answer` (and maybe later `ask`) may write into another checkout; enforce that in the shared library, not only the CLI.
Rules:
- `<shed>` resolves with `sheds::resolve` (already built; print its error and exit 1 on no/ambiguous match). `<id>` without `@` behaves exactly as today.
- The write goes through the SAME code as a local `answer` (the needs transition, the attributed note, `--as`/YAKS_ACTOR), applied to the shed's farm root, holding THAT farm's lock (`store::lock(<shed farm root>)`, per AGENTS.md: lock once, at the outermost method). Implement it as opening a `Farm` on the shed's farm root (Farm::open on the shed path, or a constructor taking a root) and calling the existing answer method, so there is one answer implementation. Add an explicit, narrow guard: a `Farm` opened for a cross-shed write exposes only answer (e.g. a separate small type or a method that takes the shed and does only this); record the design as a Decision: note.
- Refuse, with a clear message, when: the shed's farm is Shared (say: it is this farm, use plain `answer`), None, or unreadable; the yak is not in that shed; its `needs` there is not `human` (same check plain answer makes, unless plain answer allows it: match plain answer).
- Print one line saying which shed and file were written, and that it is left uncommitted there.
- Never commit, stage, or run any git command that writes in the other checkout.
Scope: src/main.rs (the Answer arm: parse `@`), src/farm.rs (the cross-shed entry point). Docs: docs/cli.md (answer row), README.md, `.agents/skills/yaks/SKILL.md` (the ask/answer paragraph and table), clap help.
Out of scope: `ask` across sheds; `--shed` flags; sheds.rs internals (do not edit src/sheds.rs: another worker is refactoring it).
Evidence: tests that FAIL before (tests/answer_shed.rs, real binary, git worktree shed): an ask made in the shed is answered from the main checkout via `<id>@<shed>`; the shed's file now has needs: agent and the attributed reply, `git -C <shed> status` shows it modified and uncommitted, the main farm's copy is untouched, and `inbox --sheds` from main now shows it answered (needs agent). Refusals: unknown shed, yak absent in the shed, shared farm. Judge: coordinator, at landing.

---
▸ 2026-10-09T20:46:29Z [ans-1]
Start: Fri Oct  9 20:46:25 UTC 2026. path: /Users/joel/src/yaks/.delta/worktrees/h4skc0fpebk8/yaks/.yaks/shaving/yaks-f347.md

---
▸ 2026-10-09T20:52:07Z [ans-1]
verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-09T20:52:41Z [ans-1]
Decision: the cross-shed write is one `Farm::answer_in_shed(&self, shed, id, ...)` method (src/farm.rs). The shed `Farm` is a temporary built inside it and never returned, so no other mutation can reach a shed. Rejected: a public `Farm::open_at(root)` or a separate `ShedFarm` type, since either hands callers a full-power handle or adds a second type for one method; the narrowness is enforced in the library as Joel decided.
Decision: the shed's farm root comes from `store::discover_with(&shed.path, None)`, ignoring $YAKS_DIR (which names OUR farm). Then I refuse when `shed.farm` is Shared/None/Unreadable, when the found root canonicalises to our own root, or when its schema is newer. Rejected: trusting `ShedFarm::Own` alone (it can go stale between discovery and write).
Decision: `answer_in_shed` calls the existing `Farm::answer` on the shed root, so the needs transition, attributed note and the shed farm's lock (once, outermost) are the one implementation. An id absent in the shed is an error ("task X not found in shed N"). Plain answer's rule is matched: on a yak with no `needs` it only records the note (not refused), and an already-`agent` yak keeps the state, because plain answer does that. I did not add a needs=human check, since plain answer has none.
Decision: `<id>@<shed>` is parsed in the Answer arm with `split_once('@')` (yak ids contain no `@`). Output is one line, "Wrote the answer into shed <name> at <file> (left uncommitted there; ...)", then the usual "Answered <id>: ..." line.
Evidence: tests/answer_shed.rs (7 tests, real binary, git worktree shed with the ask committed there first): answer lands in the shed with needs: agent and [joel] note, `git status` there shows only it modified, HEAD unchanged, nothing staged, main copy and main `git status` untouched, `inbox --sheds --json` shows needs agent; --done clears needs; plain answer unchanged; unknown shed, absent yak, shared (symlinked) farm and no-farm are refused. FAIL-before check: stashed src/farm.rs and src/main.rs with `git checkout --`, ran `cargo test --release --test answer_shed`: 6 FAILED, 1 passed (the plain-answer test, which passes by design); restored. Gate `cargo test --release`: all suites ok (493 unit, plus every integration suite incl. answer_shed 7 passed). `yaks verify` ran its configured `cargo test -p yaks` (debug): PASS.
Docs: docs/cli.md answer row, README.md ask/answer row, .agents/skills/yaks/SKILL.md (ask/answer paragraph + table row), clap `///` help on Command::Answer and the `id` arg. Left alone: rustfmt touched src/actor.rs, src/tui/docshots.rs, src/tui/tests.rs; I reverted those.

---
▸ 2026-10-09T20:52:42Z [ans-1]
moved: shaving -> shorn

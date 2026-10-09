---
id: yaks-f347
title: 'yaks answer <id>@<shed>: answer an ask in another shed''s copy (the one cross-shed write)'
type: feature
priority: 3
created: '2026-10-09T20:44:57Z'
updated: '2026-10-09T20:44:58Z'
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

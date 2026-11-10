---
id: yaks-b89d
title: Format src/actor.rs and tests/commits.rs once so cargo fmt stops rewriting them
status: shorn
type: task
priority: 2
created: '2026-11-10T17:53:07Z'
updated: '2026-11-10T17:53:57Z'
tags:
- meta
verify: cargo fmt --check
---

Every session note so far reports `cargo fmt -- <files>` rewriting `src/actor.rs` and `tests/commits.rs`, and every worker reverts them by hand, though `AGENTS.md` warns about it: a lesson already in context that still recurs (yaks-2b40, point 4). The mechanism fix: format the two files once on `main`, and drop the warning from `AGENTS.md` and from briefs.

---
▸ 2026-11-10T17:53:15Z [herd-unlock]
kind: move
from: hairy
to: shaving
thread: ksQQUQUvLIRxSc62aHEDNt0G0pLOAFQ8T5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-10T17:53:47Z [herd-unlock]
kind: verify
result: pass
---

verify: `cargo fmt --check` -> PASS (exit 0)

---
▸ 2026-11-10T17:53:56Z [herd-unlock]

Done in `53576880`: `cargo fmt` run once on `main` (it touched only `src/actor.rs` and `tests/commits.rs`; `src/tui/docshots.rs` and `src/tui/tests.rs` were already clean). `AGENTS.md`'s "Delegating" section and the repo's `land` skill now say `main` is fmt-clean and to run `cargo fmt --check` before landing, instead of reverting files by hand.

- `verify: cargo fmt --check` PASS.
- This is the first lesson reflection would have caught as "already in context and still recurring"; the next round should report it `recurred` or `gone`.

---
▸ 2026-11-10T17:53:57Z [herd-unlock]
kind: move
from: shaving
to: shorn
thread: ksQQUQUvLIRxSc62aHEDNt0G0pLOAFQ8T5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn

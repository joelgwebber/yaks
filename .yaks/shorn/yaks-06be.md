---
id: yaks-06be
title: 'yaks preflight: landing-readiness checks (evidence staged, verify passed, no duplicate status dirs)'
type: feature
priority: 2
created: '2026-10-04T04:49:19Z'
updated: '2026-10-04T04:53:22Z'
parent: yaks-2c22
labels:
- cli
- agent
verify: cargo test -p yaks
---

The 'preflight now' half of yaks-2c22, approved by Joel ('yaks brief' waits for the skills restructure). A read-only command a coordinator or worker runs before landing in a team farm: yaks preflight [<id>...] [--json]. v1 checks: (1) nothing under .yaks/ is untracked or modified in git (git status --porcelain -- .yaks): a new artifacts dir that was never added, or yak edits not committed, are the failures trials 1-4 hit; (2) every shorn yak in scope (the ids given, else all shorn yaks) with a verify: command (own or the label default) last passed, reusing the logic behind 'doctor --strict'; (3) no yak sits in two status dirs (reuse doctor). One actionable line per failure, exit non-zero if any, 'preflight: ok' otherwise. In a private farm (git ls-files .yaks prints nothing) skip (1) with a clear line and still run (2) and (3). Ask before adding history checks (the yak move and code in one commit, the id in the commit message): they need a defined range. Evidence: tests for each check passing and failing in a temp git repo, a transcript of the built binary on a scratch repo (untracked artifact dir fails with the dir named; adding it passes), docs/cli.md, --help, README table and the yaks skill command table updated.

---
▸ 2026-10-04T04:49:19Z [delta-lead]
TRIAL 5 (yaks-fe33): worker 't5-preflight', SHORT brief, loads yaks-working itself; model read from the spawn confirmation. Claim committed. Scope: a new command (src/main.rs Command and dispatch; check logic in src/farm.rs or a small new module), docs/cli.md, README.md table, the yaks skill command table. Deliberate overlap with t5-actor on docs/cli.md, README and the skill tables: expect an adjacent-edit merge at landing. Gate cargo test -p yaks. Judge: coordinator re-runs gate and transcript.

![preflight-transcript](artifacts/yaks-06be/preflight-transcript.txt)

---
▸ 2026-10-04T04:53:09Z [t5-preflight]
Implemented src/preflight.rs (3 checks; ids scope the verify check only, git+duplicate checks farm-wide; staged changes pass check 1 since staging precedes the landing commit; verify uses own verify: else config label default, same last-PASS rule as doctor --strict). 9 unit tests in a temp git repo (each check pass+fail, private-farm skip, scoping, config default). Gate: cargo test -p yaks -> 340+28 passed. Binary transcript attached. Note: an unknown id given on the command line fails; given ids that are not shorn are ignored. No history checks added (awaiting a defined range).

---
▸ 2026-10-04T04:53:22Z [t5-preflight]
verify: `cargo test -p yaks` -> PASS (exit 0)

---
id: yaks-d1bd
title: 'yaks preflight: check that ''local'' is the human''s checkout before a push to main'
type: task
priority: 2
created: '2026-10-04T21:06:09Z'
updated: '2026-10-05T19:54:10Z'
parent: yaks-99ec
labels:
- cli
- delta
---

From yaks-99ec decision 2 and yaks-df61 O51/O55. The Delta skill now tells the coordinator, in prose, to check before 'git push local <branch>:main' that 'local' is a real checkout: 'git ls-remote --heads local main' prints a line and 'git -C $(git remote get-url local) rev-parse --is-bare-repository' says false; otherwise push pr/<name> and give the human a 'git fetch <path> pr/<name>' line. Move it into code: a yaks preflight item (and/or land.sh) that reports the state, exits non-zero when local is bare, has no main, is missing, or its checkout is on main with uncommitted changes (the push would be refused), and prints the fallback command. Where it belongs relative to the existing preflight scope (landing checks, src/preflight.rs) is part of the job. Tests with temp repos for: real checkout, bare repo, no local remote, dirty checkout on main. Then the skill's prose points at the command.

---
▸ 2026-10-05T19:49:09Z [delta-lead]
moved: hairy -> shaving

---
▸ 2026-10-05T19:49:09Z [delta-lead]
Claim (coordinator delta-lead). Owner: worker pre-1. Scope: src/preflight.rs (the new check and its item in the preflight report), the Preflight help/dispatch in src/main.rs only as needed, docs/cli.md, README.md, .agents/skills/yaks/SKILL.md if it lists preflight checks. Judge: coordinator re-runs 'cargo test -p yaks' and runs 'yaks preflight' against this thread's real 'local' (a real checkout on the laptop) and against a bare managed-style local made in a temp repo. Forbidden: pushing anywhere (the check reads: ls-remote, rev-parse, status), coordination skills (coordinator updates the prose to point at the command), Cargo/config.

---
▸ 2026-10-05T19:49:55Z [pre-1]
pre-1 start: file .yaks/shaving/yaks-d1bd.md; date -u Mon Oct 5 19:49:28 UTC 2026

---
▸ 2026-10-05T19:54:04Z [pre-1]
Scope decision: the local check is OPT-IN, behind 'yaks preflight --push-main', not default. Why: it reads another checkout's state (the human's dirty tree) that a worker committing its own yak can neither fix nor should be failed by, every worker clone has a 'local' remote, and only the coordinator pushes main; default-on would repeat the yaks-df61 O45 noise. Without the flag a repo with no 'local' remote is unaffected (verified: preflight ok, exit 0). With the flag a missing 'local' IS a failure (asked-for check cannot pass). New Check::LocalCheckout, code 'local-checkout', same Failure/Report/exit-code path, no new structure. Read-only: remote get-url, ls-remote --heads local main, rev-parse --is-bare-repository, symbolic-ref, status --porcelain --untracked-files=no (untracked files do not block updateInstead; flag added deliberately). Real remote.local.receivepack here is 'git -c receive.denyCurrentBranch=updateInstead receive-pack' and 'local' is /Users/joel/src/yaks/.git (a git dir, not the work tree): the check maps a '.git' path to its parent for the dirty/branch test (status fails inside a bare .git dir). Not checked: a clean checkout on main whose receive.denyCurrentBranch is not updateInstead/ignore would also refuse; out of scope. Evidence: cargo test -p yaks green (391+8+28); 8 new tests fail with the check disabled (mutation: 'if false' in run); real runs in a temp farm for every case (no remote, clean, dirty on main, other branch, missing path, https, scp-style ssh, bare empty, no main) behaved as designed. Docs updated: docs/cli.md, README.md, .agents/skills/yaks/SKILL.md, clap help in src/main.rs.

---
▸ 2026-10-05T19:54:10Z [pre-1]
Shorn: yaks preflight --push-main implemented in src/preflight.rs (Check::LocalCheckout), docs and help updated, gate green, real run in own checkout: 'preflight: ok' (exit 0) against local=/Users/joel/src/yaks/.git.

---
▸ 2026-10-05T19:54:10Z [pre-1]
moved: shaving -> shorn

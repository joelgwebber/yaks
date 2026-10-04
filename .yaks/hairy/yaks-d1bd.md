---
id: yaks-d1bd
title: 'yaks preflight: check that ''local'' is the human''s checkout before a push to main'
type: task
priority: 2
created: '2026-10-04T21:06:09Z'
updated: '2026-10-04T21:06:09Z'
parent: yaks-99ec
labels:
- cli
- delta
---

From yaks-99ec decision 2 and yaks-df61 O51/O55. The Delta skill now tells the coordinator, in prose, to check before 'git push local <branch>:main' that 'local' is a real checkout: 'git ls-remote --heads local main' prints a line and 'git -C $(git remote get-url local) rev-parse --is-bare-repository' says false; otherwise push pr/<name> and give the human a 'git fetch <path> pr/<name>' line. Move it into code: a yaks preflight item (and/or land.sh) that reports the state, exits non-zero when local is bare, has no main, is missing, or its checkout is on main with uncommitted changes (the push would be refused), and prints the fallback command. Where it belongs relative to the existing preflight scope (landing checks, src/preflight.rs) is part of the job. Tests with temp repos for: real checkout, bare repo, no local remote, dirty checkout on main. Then the skill's prose points at the command.

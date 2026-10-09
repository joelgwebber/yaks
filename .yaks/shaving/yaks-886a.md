---
id: yaks-886a
title: Shed names from Delta thread titles, slugged like a git worktree name
type: feature
priority: 3
created: '2026-10-09T22:35:46Z'
updated: '2026-10-09T22:35:48Z'
parent: yaks-a3d2
labels:
- cli
- delta
---

---
▸ 2026-10-09T22:35:48Z [delta:Delta :Yaks (cont'd)]
moved: hairy -> shaving

---
▸ 2026-10-09T22:35:48Z [delta:Delta :Yaks (cont'd)]
Claim (coordinator). Owner: title-1. Decided by Joel on yaks-a3d2 (2026-10-09): yes to thread titles as shed names, "slugged closer to normal git worktree names".
Goal:
1. RECORD: any yaks command run inside a Delta clone, with `DELTA_THREAD_TITLE` set, records the title in that clone's OWN git config (`git config --file <absolute-git-dir>/config yaks.shed.title "<title>"`), plus `yaks.shed.thread` = `DELTA_CURRENT_THREAD_ID` when set. Best effort and silent on any failure; write only when the stored value differs (no write on every command). Never derived state that is committed (git config is per-clone, never committed: the AGENTS.md "derived, rebuildable" rule).
   Guards (record a Decision: for each): only in a Delta clone (the checkout's `objects/info/alternates` names a host, i.e. `sheds::host(top).via_alternates`); NOT when `-C` was given (the env then belongs to the caller's thread, not that checkout's: a coordinator running `yaks -C <worker clone>` must not stamp its own title there); not in a plain git worktree (its local config is shared with the primary).
2. READ: discovery reads `yaks.shed.title` from each shed's git dir (`git config --file <git dir>/config --get yaks.shed.title`); add it to `Shed` (e.g. `title: Option<String>`) and to `yaks sheds` text (on the `name:` line) and JSON (`title`).
3. SLUG: `Shed::name()` becomes: `main` for the primary, else the SLUG of the title, else the first `who`, else dir id, else branch, else the dir name. Slug rule: lowercase; runs of non-[a-z0-9] become one `-`; trim `-`; cut to at most 24 chars at a `-` boundary when possible. Examples to test: "cflag-1: yaks -C global flag" -> "cflag-1-yaks-c-global"; "Delta :Yaks (cont'd)" -> "delta-yaks-cont-d". Two sheds with the same slug: append the first 4 chars of the dir id (`<slug>-q7zz`).
4. RESOLVE: `sheds::resolve` also matches the slug exactly (right after the dir id) and the full title in the substring step.
Scope: src/sheds.rs (Shed, name, resolve, inspect/found: read the title), a new small function for recording (in src/sheds.rs or a new module), ONE call line in src/main.rs placed immediately AFTER the `-C` loop (gate it on `cli.chdir.is_empty()`). Docs: docs/cli.md (sheds row: names), README.md, `.agents/skills/yaks/SKILL.md` (sheds row), `.agents/skills/yaks-coordinating/SKILL.md` §3 (the "its name" sentence), `.agents/skills/yaks-coordinating-delta/SKILL.md` (the "named by its Delta dir id until its first note" line: now by the slug of its spawn title from its first yaks command), clap help.
Out of scope: --shed (another yak, after this lands); compare_farms (another worker owns it now: do not touch).
Evidence (tests that FAIL first): slug unit tests incl. the two examples and the collision suffix; a delta_clone fixture (see the sheds tests helpers) where running the binary in the clone with DELTA_THREAD_TITLE set records the title and `yaks sheds` from the primary names the clone by its slug; the same with `-C <clone>` from the primary records nothing; a plain git worktree records nothing. Judge: coordinator.

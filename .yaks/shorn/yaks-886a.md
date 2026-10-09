---
id: yaks-886a
title: Shed names from Delta thread titles, slugged like a git worktree name
type: feature
priority: 3
created: '2026-10-09T22:35:46Z'
updated: '2026-10-09T22:48:34Z'
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

---
▸ 2026-10-09T22:38:36Z [title-1]
Started: Fri Oct  9 22:38:33 UTC 2026. yaks path: /Users/joel/src/yaks/.delta/worktrees/606y05kk15w7/yaks/.yaks/shaving/yaks-886a.md

---
▸ 2026-10-09T22:46:00Z [title-1]
verify: `cargo test -p yaks` -> FAIL (exit 101)

---
▸ 2026-10-09T22:47:15Z [title-1]
verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-09T22:48:34Z [title-1]
Decision: guard "Delta clone" = `host(cwd).via_alternates` AND `git rev-parse --git-common-dir` == own git dir. Rejected: alternates alone (a git worktree *of* a clone would pass but shares the clone's config, so it would stamp the clone with a title that is not the clone's thread); rejected checking `core.worktree`.
Decision: `-C` guard is `cli.chdir.is_empty()` at the one call line after the `-C` loop (as specified), not a check inside record_thread: the function stays a plain "record for the cwd" and the CLI says when it applies.
Decision: record the title with whitespace collapsed (split_whitespace/join), nothing else. Rejected: reusing actor.rs's 40-char cut / bracket replacement (that exists for note stamps; the full title is wanted for `title:` display and the full-title substring match).
Decision: empty/blank title writes nothing, and `yaks.shed.thread` is only written together with a title. A thread id without a title records nothing (the slug needs the title; the id alone would add nothing readable).
Decision: write only when the stored value differs, compared via `git config --file <gitdir>/config --get` (the same helper the readers use, so user/system config never leaks in); the write is `git config --file`. Test checks the config file mtime is unchanged on a same-value call.
Decision: slug cut: if byte 24 is `-` keep the first 24; else cut at the last `-` inside the first 24 chars; no `-` there means a hard cut at 24. Non-ASCII characters (e.g. accented letters, emoji) are treated as separators (rule says [a-z0-9] only); a title with none left has no slug and name() falls through to who/dir id.
Decision: disambiguation lives in `discover` (after all sheds are inspected), as `Shed.slug`, with `Shed.title` kept raw; name() uses `slug`. Rejected: computing the suffix in name() (it has no view of the other sheds). The primary is excluded (it is `main`) and a clone with no dir id (can't happen for a titled Delta clone, but a worktree has no title) is left alone.
Decision: titles are only read for non-primary sheds (the primary's own git config is the human's; a title there would never be written by us, and must not rename `main`).
Decision: resolve order: path, dir id, slug (new, right after dir id as specified), main, branch, who, then substring over path/dir/slug/title/who. A bare slug shared by two sheds is only possible when disambiguation could not run, and then `names N sheds` lists them.
Decision: `yaks sheds` text shows `name: <slug> · title: <title>` on the one name line (middle dot, like the label line); JSON gets `title` (null when none). Rejected: separate `title:` line (spec said on the name: line).
Decision: record_thread uses current_dir() and silently returns on any error, so it never changes stdout/stderr/exit status of any command; it runs before the init/skills early returns (so `yaks init` inside a clone also records), because the spec says "any yaks command".

Evidence: new tests
- src/sheds.rs: slug_is_a_git_worktree_like_name (both spec examples, 24-char cut cases, hard cut, empty), a_titled_clone_is_named_by_its_slug_and_resolves_by_it, the_slug_beats_the_actor_and_the_dir_id_in_a_name, sheds_with_the_same_slug_get_their_dir_id_prefix (`fix-the-thing-q7zz`/`-k3mm`, bare slug ambiguous), record_thread_writes_only_in_a_delta_clone_and_only_a_changed_value (no leak to host, no rewrite on same value, primary and plain git worktree record nothing).
- tests/shed_title.rs (real binary, Delta-shaped clone): in-clone command records title+thread and `yaks sheds` / `--json` from the primary names the clone `cflag-1-yaks-c-global`, host config and git status untouched; `-C <clone>` from the primary records nothing; plain git worktree + primary record nothing.
- They fail without the change: deleting the call line in main.rs fails the record test; removing the `cli.chdir.is_empty()` guard fails the -C test (both observed). The unit tests do not compile/pass without Shed.title/slug/slug()/record_thread_in.
- Docs/help: docs/cli.md (sheds + changes rows), README.md, .agents/skills/yaks/SKILL.md, yaks-coordinating §3, yaks-coordinating-delta (spawn line), clap help for sheds and changes. Example slugs in the two coordination skills avoid a `yaks-<word>` token because skills::repo_skills_are_well_formed treats it as a skill reference (found by a failing verify).
Gate: `cargo test --release` all suites ok (499 unit + integration); `yaks verify` PASS recorded.
Surprise: in this clone `git config --get yaks.shed.title` = "title-1: shed names from Delta thread titles" (recorded by my own binary, as expected).

---
▸ 2026-10-09T22:48:34Z [title-1]
moved: shaving -> shorn

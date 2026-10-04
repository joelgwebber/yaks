---
id: yaks-2cd4
title: 'yaks commit: commit the farm''s own changes in one command (public mode)'
type: idea
priority: 3
created: '2026-10-03T23:32:45Z'
updated: '2026-10-04T05:16:45Z'
parent: yaks-b5a0
labels:
- cli
- agent
---

Joel (2026-10-03): leaving dirty yak edits in a team-mode checkout blocks agents from landing (a push to the checked-out branch is refused while the tree is dirty) and hides answers from the agent's clone; the habit to adopt is committing yaks aggressively. Idea: a 'yaks commit' that stages only .yaks/ changes (never code) and commits them with a generated message ('yaks: <summary of moves and notes>'), refusing if other files are staged, so the human's 'Yak herding.' commit is one command (and the TUI could offer it). Design test: a mutation over files plus git, in the tool's lane only if it stays generic; otherwise a skill habit. Open: message format, whether to include artifacts, interaction with pre-commit hooks, private farms (nothing to commit: error clearly).

---
▸ 2026-10-04T05:11:28Z [delta-lead]
TRIAL 6 (yaks-fe33): worker 't6-commit', SHORT brief, loads yaks-working itself. Claim committed. Spec from the idea: 'yaks commit [-m <message>] [--dry-run]' commits ONLY the farm's own changes (everything under the farm root: yak files, artifacts, .gitignore) and never code, with a generated message when -m is absent (for example 'yaks: shorn yaks-abc1; notes on yaks-def2, yaks-ghi3', built from the changed files). Private farm (nothing under the farm tracked by git): fail clearly. Real forks to ASK about, with a lean: (a) what to do if other files are staged (lean: refuse, naming them; or commit only the farm paths with git commit --only); (b) message format; (c) whether it runs hooks (lean: yes, normal git commit). Scope: a new command (src/main.rs plus a small module or farm.rs), docs/cli.md, README table, the yaks skill command table, and a pointer in yaks-coordinating-team where it says to commit human drift. Evidence: gate cargo test -p yaks; tests in temp git repos (changes committed, code untouched, other staged files, nothing to commit, private farm, --dry-run); a transcript of the built binary. Judge: coordinator re-runs gate and transcript. Overlap with t6-bound on the docs/README/skill tables is deliberate.

---
▸ 2026-10-04T05:16:18Z [t6-commit]
Decisions (workers calls, recorded; none needed a human): (a) other staged files: commit only the farm paths with 'git commit --only -- <farm>' after 'git add -A -- <farm>'; they stay staged and are named in the output (chosen over refusing: nothing is lost, the human's code staging survives, and the command stays one-shot). (b) message: 'yaks: created A; shaving B; shorn C; dead D; regrown E; updated F; removed G; artifacts for H; farm config', one verb per yak from git status (move = file gone from one status dir and present in another), ids capped at 5 per phrase ('and N more'); -m overrides. 'updated' not 'notes on': an in-place edit can be a note, answer, or field edit, and the file diff does not say which. (c) hooks: plain git commit, hooks run; a hook failure leaves the farm staged and errors. Also: clean farm exits 0 with 'nothing to commit' (idempotent for 'yaks commit && ...'); private farm or non-git exits 1 with a clear error; no farm lock taken (it writes no farm file, and a hook that runs a yaks command would self-deadlock the non-reentrant lock). Code: src/commit.rs + Command::Commit in src/main.rs; preflight::yak_id_of made pub(crate).

![commit-transcript](artifacts/yaks-2cd4/commit-transcript.txt)

---
▸ 2026-10-04T05:16:19Z [t6-commit]
transcript of the built binary on scratch repos: clean farm, dry-run, commit with another staged file + dirty code, private farm

---
▸ 2026-10-04T05:16:45Z [t6-commit]
Shorn: gate 'cargo test -p yaks' green (358 + 28, incl. skills guard); 6 unit tests in src/commit.rs cover commit-farm-only/code untouched, other staged files, nothing to commit, private farm, dry-run, id cap, rename-as-move; transcript attached. Docs: docs/cli.md, README, yaks skill table, yaks-coordinating-team pointer, clap --help.

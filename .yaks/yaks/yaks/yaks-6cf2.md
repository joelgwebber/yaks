---
id: yaks-6cf2
title: 'check: flag doks whose cited code has changed since the pin or last verification'
status: shorn
type: feature
priority: 3
created: '2026-11-08T16:24:07Z'
updated: '2026-11-14T22:48:07Z'
parent: yaks-1c89
depends_on:
- yaks-f505
tags:
- cli
- doks
---

From [the agent-memory research](/doks/research/agent-memory-dreaming.md#3-let-evidence-not-only-the-clock-say-what-to-re-check) (yaks-e4f5): `yaks check` only knows clock staleness (`stale_after:`). A dok that describes code goes wrong when the code moves, and the farm already holds the evidence: `code:` links name the files, `@sha` pins what was read.

Add one model-free finding to `src/check.rs`: a dok that cites `code:<path>` where the path has commits in the code repo since the pinned sha (or, unpinned, since the dok's `verified:` or last commit). Report the dok, the path and the commit count. It never edits and decides nothing about whether the dok is wrong: an agent reads the dok against the code and sets `verified:` (the existing meaning of that field).

Design points to settle in the work: what counts for an unpinned link, whether a count threshold is useful, and what to do when no code checkout is reachable (skip, as `peek` refuses). Docs and the `yaks-research` skill's `yaks check` paragraph change with it.

---
▸ 2026-11-14T11:19:34Z [linkplan-lead]
kind: ask
needs: human
---

asked: needs human

Three design points this yak lists; answers I would build with unless you say otherwise:

1. **Unpinned `code:` link:** count commits touching the path since the dok's `verified:` date, else since the dok's last farm commit (a hand-set `verified:` is the deliberate re-check).
2. **Threshold:** none, report any count above zero, with the number, so the reader decides. A threshold is easy to add later.
3. **No code checkout reachable:** skip the dok with one info line (as `peek` refuses), never a finding.

One new interaction: yaks-1151 will store pinned code links as host URLs (`https://github.com/o/r/blob/<sha>/path#L12`) that yaks parses back into code links. `check` must read both forms, so it should go through the shared resolver (yaks-1151's resolver yak) rather than its own parse. Lean: build it after that yak lands, not before.

Answer "go" to accept all three, or say what to change.

---
▸ 2026-11-14T17:42:32Z [sched-1]
kind: answer
needs: agent
---

Ooh, that's a great idea I hadn't even considered for updating doks. Makes perfect sense for unpinned code links, as they can drift (pinned ones _shouldn't_ in all normal cases that don't involve deep git surgery, right?).

I'm good with all those design choices. On the last bit with pinned code links, see my earlier comment. If we *do* need to handle that parsing case for some reason, then yes -- let's defer it and leave a comment on yaks-1151 or another appropriate yak about unifying the resolution.

---
▸ 2026-11-14T17:51:08Z [linkplan-lead]
kind: pickup
---

picked up

Decision (Joel): accept all three (since `verified:` else the dok's last farm commit; no threshold; no code checkout means skip with an info line). Scope: `code:` links only. A pinned link should not drift in normal use, so only unpinned links count as findings (a pinned one still reports when its path has commits since the pin AND the dok has no `verified:` after them, per the original yak; keep it as the yak says). Host-URL code links are not read here: unifying their resolution is recorded on yaks-1151, and this is built through the resolver once that lands.

---
▸ 2026-11-14T17:52:06Z [linkplan-lead]

Scope, restated from the yak (my pickup note above was muddled): a finding is a dok that cites `code:<path>` where the path has commits in the code repo since the pinned sha, or for an unpinned link since the dok's `verified:` date else its last farm commit. Joel's remark that pinned links "shouldn't drift" is about the link still meaning what it meant; the dok's claim can still be stale against the code today, which is what this finding is for. If that is not the intent, reply here.

Decisions, per Joel: no count threshold; with no code checkout, skip with one info line. Read links through the resolver (`locate.rs`, landed with yaks-f505) so a host-URL code link (coming with the schema-9 migration) is read by the same code later; do not parse host URLs here.

---
▸ 2026-11-14T17:53:15Z [linkplan-lead]
kind: move
from: hairy
to: shaving
thread: ksQQw3f_HRXkT-eF6oV8kx3Nn5LOAFi59ZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-14T17:53:19Z [linkplan-lead]

Claim: worker `check-1` (Sonnet tier). Owns `src/check.rs` (the new finding and its report/`--json` shape), its tests, the `yaks check` paragraph in `docs/cli.md` and in the `yaks-research` skill, and the clap help for `check` if it lists findings. Reads `code:` links through `locate.rs` / `links.rs` as landed (yaks-f505); do not edit those. Out of scope: host-URL code links, schemas, other check findings.

---
▸ 2026-11-14T17:54:27Z [check-1]

Start: Sat Nov 14 17:54:19 UTC 2026. Farm file: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-6cf2.md. Checkout: /Users/joel/src/yaks/.delta/worktrees/hrp40zefpg6c/yaks

---
▸ 2026-11-14T18:01:26Z [check-1]

Read check.rs/locate.rs/links.rs. Plan: new kinds `code-moved` and `code-gone`; links read via `links::extract`+`links::parse`+`Space::resolve` (Place::CodeFile) from dok bodies, plus `code:` items in frontmatter `sources:`/`code:` lists; code repo from `links::code_base(farm, cwd)`; own git helpers in check.rs. `run()` keeps its signature (tick uses it); a new `run_report()` also returns info lines, which `Command::Check` prints (stdout in text mode, stderr with --json).

Decision: a pinned link to a file deleted at HEAD is not `code-gone` (a pin legitimately names history): its deleting commit counts as a commit since the pin, cleared by `verified:`. `code-gone` is for unpinned links only.

Decision: a `verified:` DATE covers that whole day (end of day UTC); a commit made on the verified day or before counts as checked. Rejected: start of day, which would keep a dok flagged all day after `dok set verified=<today>` (the exact fix the hint prescribes). A verified timestamp is exact.

---
▸ 2026-11-14T22:38:26Z [check-1]

Resumed after the network drop (22:38Z). Working tree has: `src/check.rs` (new kinds `code-moved`/`code-gone`, `Report`, `code_changes`, 10 unit tests) and `src/main.rs` (`Command::Check` prints info lines, clap help). My base `dc8aaa92` already contains the landed resolver (298b91ac, 3c258a53); `local/main` in this clone is the older remote ref, nothing newer to rebase on. 12 of 13 unit tests pass; fixing one test assertion (path-extraction in the frontmatter test), then the integration test, docs, real run.

---
▸ 2026-11-14T22:42:07Z [check-1]

Evidence, real runs with my binary (`target/debug/yaks`, base dc8aaa92).

Against this farm as it is: `yaks check` reports no `code-moved`. `/doks/design/knowledge-store.md` cites `code:docs/README.md` unpinned and has `verified: 2026-11-10`; `git log --format='%h %cI' -- docs/README.md` shows its newest commit is 671320f9, 2026-11-06, so nothing moved since the verification: correct, no finding.

Against a scratch copy of the farm (copied to /tmp, `YAKS_DIR` pointing at it, never touching the real farm) with that dok's `verified:` set to 2026-11-01:

```
Code has changed since the dok was written or last verified (commits to a `code:` path it cites):
  doks/design/knowledge-store.md cites code:docs/README.md: 4 commits to it since it was verified 2026-11-01

To clear a code finding: read the dok against the code, fix what drifted, then `yaks dok set <path> --set verified=<date>`.
```

`--json` gives `{"kind":"code-moved","message":...,"ids":["doks/design/knowledge-store.md","code:docs/README.md"]}`.

Confirmed by hand: `git log --format='%h %cI' --since=2026-11-02T00:00:00Z -- docs/README.md` lists 671320f9, 6352ec29, 5cf76867 (all 2026-11-04..06) and de32523e at 2026-11-01T20:50-05:00 = 2026-11-02T01:50Z, which is after the 2026-11-02T00:00Z `--since` but before the end of the verified day 2026-11-01 UTC... so git's `--since` shows 4, while the check's rule (a verified DATE covers its whole day in UTC, commits strictly after 2026-11-01T23:59:59Z) counts commits after it: de32523e is 2026-11-02T01:50Z, so it counts: 4 total. Matches.

Tests: 13 `check::` unit tests with scratch code repos (pinned with later commits, pinned with none, pinned + verified before/after/exact, unpinned since verified, no verified and no farm commit skipped, frontmatter `sources:`/`code:` lists, no checkout = one note and no finding, removed file unpinned = gone and pinned = counted, unknown pin = note) and one `tests/check.rs` integration test (unpinned, falls back to the dok's last farm commit, `code-gone`, `--json` ids, exit 1, hint line, `verified:` clears to exit 0). `cargo fmt --check` clean, clippy has no warnings in check.rs.

---
▸ 2026-11-14T22:47:55Z [check-1]
kind: verify
result: pass
---

verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-11-14T22:48:06Z [check-1]

Summary. New `code-moved` and `code-gone` findings in `src/check.rs`; `Command::Check` in `src/main.rs` prints the info lines (stdout in text mode, stderr with `--json`, so the array is unchanged); docs in `docs/cli.md`, `README.md`, the yaks-research skill and the clap help.

- `code_refs` reads inline body links through `links::extract`/`links::parse`/`Space::resolve`, plus `code:` entries of frontmatter `sources:` and `code:` lists. The code repo is `links::code_base`.
- Counts come from `git log --format=%ct [<sha>..HEAD] -- path` in the code repo (own helper in check.rs; no edits to links/locate/gitrepo).
- Gate: `yaks verify yaks-6cf2` PASS (`cargo test -p yaks`); `cargo fmt --check` clean.

Correction to my evidence note's hand check, plainly: the scratch dok was `verified: 2026-11-01`, which covers up to 2026-11-01T23:59:59Z. `git log --format='%h %cI' -- docs/README.md` has four commits after that (de32523e at 2026-11-02T01:50Z, 5cf76867, 6352ec29, 671320f9), so the reported 4 is right.

---
▸ 2026-11-14T22:48:07Z [check-1]
kind: move
from: shaving
to: shorn
thread: ksQQw3f_HRXkT-eF6oV8kx3Nn5LOAGKpipIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn

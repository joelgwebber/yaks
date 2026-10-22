---
id: yaks-f800
title: 'Trim docs/cli.md: short table rows, detail in sections below'
status: shorn
type: task
priority: 2
created: '2026-10-22T00:37:59Z'
updated: '2026-10-22T00:59:43Z'
parent: yaks-b048
labels:
- docs
---

---
▸ 2026-10-22T00:37:59Z [delta:herd-dirs]

moved: hairy -> shaving

---
▸ 2026-10-22T00:37:59Z [delta:herd-dirs]

Claim (yakherd delta:herd-dirs). Owner: `clidoc-1` (Sonnet worker).

**Goal.** `docs/cli.md`'s command table becomes scannable: each row at most ~40 words (what the command does, its key flags). Where a row is longer today (`init`, `sheds`, `changes`, `answer`, `skills status`, `doctor`, `preflight`, `land`, `sync`, `status`, `migrate`, ... check every row), move the detail into a `### <command>` section below the table (reuse an existing section if one already covers that command; link to it from the row). Facts move, they are not dropped.

**Scope.** `docs/cli.md` only.

**Out of scope.** Every other file. Rewording beyond what the move needs. Do not change any fact; if one looks wrong, note it on the yak instead of fixing it.

**Evidence.** A note with the word count of the longest table row before and after (e.g. an awk one-liner over the `|` rows), and confirmation that every in-file anchor link still resolves (each `](#x)` has a matching heading). `cargo test --release` is not affected but run `cargo test --release skills` to be sure nothing reads cli.md.

**Judge.** The yakherd, at landing.

---
▸ 2026-10-22T00:38:34Z [clidoc-1]

Start: Thu Oct 22 00:38:29 UTC 2026
Farm file: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-f800.md

---
▸ 2026-10-22T00:42:20Z [clidoc-1]

Decision: sheds/changes/etc. details moved verbatim into new `### <command>` sections placed right after each table (split into paragraphs at sentence boundaries, no wording changed); the table row is a ≤40-word summary + `[Details](#cmd)` link. Rejected: leaving the long text in the row; rewriting/condensing the detail (out of scope: facts must not change).
Decision: existing `### init: modes and skills...` section reused for `init` (its row links to it); the init row's long prose (modes pointer, old-layout refusal, flags) was prepended to that section.
Decision: `migrate` and `migrate --to-branch` share one `### migrate` section, `skills install`/`skills status` one `### skills` section (each with the command signature as a code line), rather than four sections; both rows link to it.
Decision: the "Default verify commands live in config.yaml..." prose + yaml blocks that sat between the Verification table and State transitions moved under `### verify` (it is that command's detail; the row's "(see below)" still holds). Rejected: leaving it as free text after the table (would now sit after the last `###` section of the table).
Decision: `doctor`, `private`, `init` first paragraphs were merged into one paragraph where my split would have broken a sentence at a `;`/`(`.
Evidence:
- Longest table row before: 1945 words (`sheds`); after: 38 (`refs --incoming` / `sync`), by `python3` counting whitespace tokens of each `|` row.
- Every in-file `](#x)` anchor (show, path, ..., multi-line-text, init-modes-and-skills-in-one-idempotent-step) matches a heading slug (script checked: all True). Only external anchor to cli.md is docs/README.md `#which-farm-a-command-uses`, still present.
- Fact preservation: whitespace-token multiset diff old→new (after unescaping `\|`) loses nothing; only added tokens are the new short summaries, links and headings.
- No src/ code reads cli.md (grep `cli\.md` in src: no matches).
Suspected errors (not fixed): none found; `sheds` text still mentions "(formerly lane)", and the `commit` section says "(rule 4 above)" which still points at the farm-as-branch rule in "Which farm a command uses".

---
▸ 2026-10-22T00:47:07Z [clidoc-1]

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-10-22T00:47:20Z [clidoc-1]

Summary: docs/cli.md command tables trimmed. Every row is now ≤38 words with a `[Details](#cmd)` link; the removed prose moved verbatim into `### <command>` sections after each table (new: show, path, next, sheds, changes, discover, refs, create, update, merge, merge-driver, bulk, verify, attach, rename-attachment, attachment, ask, answer, pickup, inbox, migrate, private, skills, doctor, preflight, brief, status, sync, commit, land; init reuses its existing section). Longest row 1945 → 38 words. Gate `yaks verify yaks-f800` PASS (808 passed). Only token lost vs. original: `works;` → `works.` (a `;` became a `.` where the init/private paragraph was joined).

---
▸ 2026-10-22T00:47:20Z [clidoc-1]

moved: shaving -> shorn

---
▸ 2026-10-22T00:59:43Z [delta:herd-dirs]

Landed as 0739dcd (squash of efb20a7). Yakherd re-checked: longest docs/cli.md table row 41 words; every in-file anchor resolves; full gate green.

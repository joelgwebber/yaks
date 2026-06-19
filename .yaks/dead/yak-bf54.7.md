---
id: yak-bf54.7
title: Test sync against Linear
type: task
priority: 2
created: '2026-04-25T17:46:57Z'
updated: '2026-06-19T16:47:10Z'
---

## Idiosyncrasies + tradeoffs to decide

### 1. Silent markdown normalization on save (HIGH)

Linear rewrites every `description` and comment body on save:

- `_italics_` → `*italics*`
- `- bullet` → `* bullet`

Probably more (e.g. trailing whitespace, link punctuation). Without normalization, *every* sync against a yak whose description was authored locally will surface phantom description drift even when the content is semantically identical.

Options:

- **(a) Best-effort markdown normalizer in the skill.** Cheap pass: convert `-` bullets to `*`, `_x_` to `*x*`, strip trailing whitespace, collapse blank lines. Apply to *both* sides before diffing. Can't catch everything, but covers the cases we've actually seen.
- **(b) Always treat description as `direction: pending`.** Force the user to eyeball every diff. Annoying, but bulletproof.
- **(c) Just live with it.** Document in the skill, accept that Linear-sourced yaks will always show description drift unless authored on the Linear side first.

Lean: **(a)** with a "if this still shows drift, hit `e` to view side-by-side" out. (b) is what the user is doing already in option (a) — just less noisily.

### 2. Status mapping is *good* (NONE)

Linear gives us normalized `statusType`: backlog | unstarted | started | completed | canceled. Maps cleanly to yak's hairy / shaving / shorn / dead. **Use `statusType`, not the display name** — display names are workspace-specific.

Decision: take the cleaner mapping; drop Linear-specific status prompts. The skill currently treats status as `direction: pending` everywhere; Linear could safely auto-map. But for v1 we should keep the conservative default (status changes always pending) — a status flip is the most user-visible thing in a sync, and "auto" is risky even when the mapping is clean.

### 3. Priority shape mismatch (LOW)

Linear: 0–4 (5 levels, with 0=None special). Yak: 1–3.

Mapping: 1/2 → 1, 3/0 → 2, 4 → 3. Lossy on the way down (collapsing Linear "Urgent" + "High" into one yak bucket), but the existing rubric (`upstream wins, auto`) handles this fine because we don't push priority back.

Decision: bake the table into the skill's *Linear hints* section. Auto-resolve as upstream-wins, no prompt.

### 4. Attachments are first-class (NICE)

Unlike Atlassian (no upload, no content download) and GitHub (no public API), the Linear MCP supports `create_attachment` / `get_attachment` / `delete_attachment`. We can actually ferry both directions.

Decision: keep `attachments_*` as `pending` per the skill's general rule (binaries are uneven), but the skill's *Linear hints* should call out that ferry actually works here, so the user picks "approve" with confidence.

### 5. Relations / depends_on (DEFER)

`blocks` / `blockedBy` / `relatedTo` / `duplicateOf` are first-class on Linear. Yak only has `depends_on` (1:1 with `blockedBy`).

Decision: out of scope for v1. The mapping is asymmetric (Linear has more link types than we do) and fixing that is a separate yak.

## Linear MCP shape (raw probe results)

Identifier: `TEAM-N` (e.g. ROC-5). Same as URL path segment. `upstream_key_for` already handles this.

Description: markdown. See item 1.

Status: see item 2. Use `statusType`, not display name.

Priority: see item 3. 0=None, 1=Urgent, 2=High, 3=Normal, 4=Low.

Labels: workspace-scoped strings. Namespace as `linear-<name>` per existing rubric.

Comments: separate `list_comments` (paged). Body, author (`{name, id}`), createdAt/updatedAt, id.

Attachments: see item 4.

Relations: see item 5.

## Test fixtures (Rocket-surgery / ROC)

- ROC-5: simple body, Backlog, priority 3 (Normal)
- ROC-6: markdown body, Backlog, priority 2 (High), 2 comments
- ROC-7: completed (Done), priority 4 (Low)

Scratch dir: `/tmp/yaks-sync-linear/` (prefix `linear`).

---
▸ 2026-04-26T17:15:53Z
Linear native GitHub Issues sync (per [github-to-linear](https://linear.app/docs/github-to-linear)) drops priority from its sync field set entirely — fields synced are title, description, labels, projects, comments, sub-issue. Validates our 'never push priority' lean. Linear's docs are silent on markdown normalization, conflict resolution, and attachments — confirming the gaps we identified are real-world. Linear's own caveat: 'Bidirectional sync is generally recommended as a temporary transition tool rather than a permanent solution' — strong validation of our one-shot, plan-then-apply shape.

---
▸ 2026-04-26T18:34:33Z
Five-scenario validation against Linear ROC-5/6/7 passed: (1) no-op fast-path, (2) local edit + deny + suppress, (3) comment ferry auto-apply with provenance, (4) two-sided drift with mixed accept-upstream/deny-push (description rewritten from upstream while all 4 comment blocks preserved), (5) sourceless → create-upstream + deny (no new ROC issue created). Net upstream writes from the skill itself: 0. Setup writes (3) for fixture state are not skill writes. Markdown normalization didn't bite during scenarios because fixtures were carefully aligned, but the post-read normalizer remains required for real-world use.

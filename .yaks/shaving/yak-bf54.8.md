---
id: yak-bf54.8
title: Test sync against Github issues
type: task
priority: 2
created: '2026-04-25T17:47:09Z'
updated: '2026-04-26T17:15:56Z'
---

## Idiosyncrasies + tradeoffs to decide

### 1. No GitHub MCP — `gh` CLI shellout (HIGH)

There's no Anthropic-side GitHub MCP. The skill needs a transport escape hatch: detect `github.com` and shell out to `gh`.

Options:

- **(a) Allow shellout in the sync skill.** Add `gh` as the GitHub transport. Skill explicitly documents the commands it expects to run (read: `gh issue view --json ...`, `gh api .../comments`; write: `gh issue edit`, `gh issue comment`, `gh issue close`).
- **(b) Stop at GitHub for now.** Tell the user "GitHub MCP not installed" and refuse to plan. Wait for an MCP.

Lean: **(a)**. `gh` is universally installed for any GitHub user, the JSON output is well-defined, and writes are gh's whole job. The skill is already MCP-tool-agnostic ("use whatever MCP is connected") — extending that to "or `gh` for GitHub" is a small line edit.

### 2. No native priority field (MEDIUM — visibility decision)

GitHub Issues have title / body / state / labels / assignees. No priority. When source is GitHub, the yak's `priority` value can never round-trip.

Options:

- **(a) Silently keep local-only.** Skill never proposes ferrying priority either way. User gets to set whatever they want locally and it stays.
- **(b) Surface in plan output.** One-line note: `priority: github source — local-only, not diffed`. Same behavior as (a) but visible.
- **(c) Map onto labels.** Convention like `priority/p1` → priority 1. Brittle (depends on the user setting up labels) and the fan-out is a footgun. Reject.

Lean: **(b)**. Cheap; the skill is already verbose; user shouldn't be surprised when a priority bump silently does nothing upstream. Reject (c).

### 3. Binary status (MEDIUM)

GitHub's only status signal is OPEN / CLOSED. Yak has hairy / shaving / shorn / dead. Mapping:

- OPEN → hairy
- CLOSED → shorn

The lossy gap: `shaving` and `dead` have no upstream representation. A yak in `shaving` can't ferry that signal upstream; closing on GitHub *also* doesn't tell us whether it was completed (shorn) or won't-do (dead).

Options:

- **(a) Conservative binary mapping, treat shaving as OPEN.** Locally: hairy/shaving both map to OPEN; dead/shorn both map to CLOSED. Round-trip is asymmetric — pulling a CLOSED issue resolves to `shorn` regardless of original local intent.
- **(b) Use the `wontfix` label as a tiebreaker.** Closed + `wontfix` → dead, otherwise → shorn. Closed yaks that are dead → push `wontfix`. Adds one rule the user has to know.
- **(c) Encode as a `gh-state-*` label.** Same shape as (b) but with our namespace. More principled, but uglier issues.

Lean: **(a)** for v1. The "lossy + asymmetric" honesty is fine — most users will close a GitHub issue when it's done, and `dead` is a yak-internal concept anyway. Revisit later if it bites.

### 4. Repo-scoped label objects (LOW)

GitHub returns labels as `{name, color, description, ...}`, not bare strings (Linear/Jira give us strings). The skill must extract `.name` before namespacing as `gh-<name>`.

Decision: trivial fix in the GitHub-shaped extractor. Document as a one-liner.

### 5. No public attachment API (LOW)

Issue body images go through GitHub's undocumented IDP-protected upload endpoint. The REST API supports release-asset uploads but not issue attachments.

Decision: per the existing skill rule, attachments stay `pending`; for GitHub specifically, surface "attachments are manual on GitHub" in plan output and list local paths/URLs without trying to ferry.

## GitHub shape (raw probe results, via `gh` CLI)

Identifier: `owner/repo#N`. `upstream_key_for` already returns this shape.

Description: markdown native. No silent normalization observed yet.

Status: see item 3.

Priority: see item 2.

Labels: see item 4.

Comments: `gh api repos/o/r/issues/N/comments` returns `[{id, body, created_at, updated_at, user.login}, ...]`.

Attachments: see item 5.

## Test fixtures (joelgwebber/yaks)

- yaks#2: enhancement label, OPEN, simple body
- yaks#3: documentation label, OPEN, markdown body, 2 comments
- yaks#4: CLOSED, no label

Scratch dir: `/tmp/yaks-sync-gh/` (prefix `gh`).

## Cross-tracker shape comparison

| Aspect | Atlassian (Jira) | Linear | GitHub Issues |
|---|---|---|---|
| Identifier | issuekey `PROJ-N` | identifier `TEAM-N` | `owner/repo#N` |
| Transport | Atlassian MCP | Linear MCP | `gh` CLI shellout |
| Description format | ADF (needs flattener) | Markdown (silently normalized) | Markdown |
| Status field | display name only | display + normalized `statusType` | `OPEN` / `CLOSED` only |
| Priority | name + id | 0–4 native | none |
| Labels shape | strings | strings | objects (`{name, ...}`) |
| Attachments via tool | metadata read only | full read+write+delete | none (no public API) |
| Status auto-mappable? | no (workspace-specific) | yes (statusType is normalized) | trivial (binary) |

Linear is the closest to "easy bidirectional sync." GitHub is the simplest shape but has the biggest gaps. Jira is the messiest but the most expressive.

### 2026-04-26T17:15:56Z
Linear's own docs (per [github-to-linear](https://linear.app/docs/github-to-linear)) state the OPEN/CLOSED mapping verbatim: 'If an issue is unstarted or started in Linear, it's considered open in GitHub. If completed in Linear, it's considered closed in GitHub.' Validates our binary mapping lean (a) word-for-word. Priority is explicitly not in their sync field set — same as our lean. Their 'synced thread' UI segregates GitHub-originated comments visually rather than algorithmically — our '### iso author (from tracker:key)' header is the textual equivalent.

---
id: yak-bf54.1
title: Investigate reliable last-modified timestamps across Jira/Linear/GitHub MCP
  tools
type: task
priority: 2
created: '2026-04-24T03:01:44Z'
updated: '2026-04-25T15:47:17Z'
---

Blocks a 'last_synced' field on yaks and any sweep/check mode. For each tracker's MCP: can we cheaply fetch (issue updated_at, per-comment updated_at, per-attachment created_at)? If the data isn't exposed uniformly enough, a local last_synced buys us nothing and we stick with full-diff sync. Deliverable: a short writeup appended here with what each MCP exposes and a recommendation.

---
▸ 2026-04-25T15:40:58Z
Jira investigation (live MCP probe against fullstory.atlassian.net).

**Issue-level `updated`:** present, ISO8601 with TZ + ms (e.g. `2026-04-25T06:59:16.876-0400`). Returned by both `searchJiraIssuesUsingJql` and `getJiraIssue`.

**Per-comment timestamps:** present at *both* granularities — each comment carries `created` and `updated` as distinct ISO8601 strings (see SUBTEXT-301 comment 458719). If a comment is edited upstream, its `updated` advances.

**Per-attachment timestamps:** did not see one populated in the probe (issue had `attachment: []`). The field exists; per Atlassian REST docs each attachment carries `created` (no `updated`, since attachments are immutable). Not blocking for v1.

**Sweep query shape:** JQL batch is fine. `issuekey IN (KEY1,...,KEYN) AND updated > "2026-04-20T00:00:00.000Z"` with `fields:["updated"]` returns only the issues that drifted since the cutoff — perfect for "which yaks might need syncing?" Mixed-project key lists work (verified incidentally; the open-ended `-7d` query returned issues from 4 different projects). JQL has a documented limit around ~1000 keys per IN clause; if a project ever has more synced yaks than that we'd need to chunk, but it's a non-issue for realistic use.

**Recommendation:** add `last_synced: <iso>` to yak frontmatter when sync-skill writes back. That alone gives:
- Per-yak fast-path: skip the diff entirely if upstream `updated` <= `last_synced`.
- Sweep mode: one batched JQL call returns the subset of synced yaks that drifted. Cheap.

**Comment body format:** comes back as ADF (Atlassian Document Format JSON tree) by default. The MCP tool offers `responseContentFormat: "markdown"` which flattens it. Worth noting for the sync-skill, but tangential to the timestamp question.

**Out of scope here, deferred:** Linear / GitHub equivalents. Per directional call, Jira having what we need is sufficient to greenlight `last_synced` for v1; Linear research can land when we actually wire Linear up. If Linear's API turns out to be uncooperative we revisit, but the field design itself is tracker-agnostic so the risk is low.

---
▸ 2026-04-25T15:47:17Z
Shorn summary: Jira gives us issue-level updated, per-comment created/updated, per-attachment created, and a clean batched JQL for 'which of these drifted?'. Added last_synced field is greenlit for v1. Linear deferred until we actually need it (no yak filed yet — will create when work starts). Spawned bf54.3 (last_synced field + holistic timestamp audit) and bf54.4 (pending-sync review pipeline).

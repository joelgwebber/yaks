---
id: yak-bf54.10
title: Unified attachment sync strategy
type: idea
priority: 3
created: '2026-04-26T16:09:14Z'
updated: '2026-04-26T16:09:14Z'
---

## Problem

Attachment ferry is uneven across the three trackers we've probed:

- **Jira (Atlassian MCP):** read metadata only. No upload, no content download. Manual on both sides.
- **GitHub Issues:** no public API at all. Web UI uses an undocumented IDP-protected endpoint. Manual on both sides.
- **Linear:** full read + write + delete via MCP. Auto-ferry actually works.

The current skill treats attachments as `pending` everywhere and tells the user to do it manually when the tracker can't help. That works but feels like the skill is shrugging.

## What we should figure out

- Should `attachments_*` resolutions become tracker-aware? E.g. on Jira/GitHub the row is locked to "manual" with a clickable path/URL; on Linear the row offers approve/skip like fields do.
- For the can't-ferry trackers: is there a sane "mark as ferried by hand" affordance? Right now the user has to clear the sidecar and hope they remembered to upload.
- Do we want a `.yaks/artifacts/<id>/manifest.yaml` or similar that records "this file was uploaded to upstream as <url>" so future syncs know not to re-propose it?

## Why not now

The current sync v1 is intentionally narrow. Punting attachments to "list paths and stop" is fine for the first release. Revisit when:

- A user asks for it explicitly.
- We discover the manual-handoff path is producing real bugs (lost attachments, duplicate uploads).
- We decide to deepen Linear support (where ferry works) and want a consistent UX with the others.

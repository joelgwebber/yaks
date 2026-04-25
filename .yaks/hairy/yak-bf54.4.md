---
id: yak-bf54.4
title: 'Pending-sync review pipeline: sidecar + TUI approval before upstream writes'
type: feature
priority: 2
created: '2026-04-25T15:47:45Z'
updated: '2026-04-25T15:47:45Z'
depends_on:
- yak-bf54.3
---

Sync should never write to upstream without giving the user a chance to review the proposed changes. The current skill puts that on the agent's prompts, which is fine in interactive runs but fragile (and impossible in headless / sweep contexts).

Proposal: a 'pending sync' sidecar file per yak.

Layout:
  .yaks/.sync-pending/<yak-id>.json

Schema (rough):
  {
    'yak_id': 'yak-abcd',
    'source': 'https://...',
    'generated': '<iso>',
    'fields': [{name: 'status', local: '...', remote: '...', resolution: 'pending|local|remote|skip'}, ...],
    'comments_up': [{body: '...', resolution: 'pending|approved|rejected'}, ...],
    'comments_down': [{author, body, resolution}, ...],
    'attachments_up': [{filename, size, resolution}, ...],
    'attachments_down': [{filename, size, url, resolution}, ...]
  }

Flow:
1. /yaks:sync (or sweep mode) runs in 'plan' phase → fetches upstream, computes diff, writes sidecar. No mutations yet.
2. TUI gains a 'pending sync' badge on any yak with a sidecar. New dialog/view shows the proposed changes per-item; user accepts/rejects/edits.
3. /yaks:sync --apply (or button in TUI) processes the sidecar: applies approved local changes, posts approved external changes, deletes sidecar on success (or rewrites with remaining items if a partial failure).
4. /yaks:sync --discard or TUI 'X' deletes the sidecar.

Why sidecar (not frontmatter): keeps the yak file pure (still 'just a markdown file with metadata'), makes batch review natural (sweep produces N sidecars; user reviews them all in TUI), supports clean discard.

Open questions for when this gets shaved:
- Does the sidecar live in .yaks/ or somewhere ignored? Probably in .yaks/ but in a hidden subdir so it doesn't show up in normal listings.
- Should the sidecar carry a snapshot of the upstream state at plan time, so apply doesn't re-fetch? Probably yes for atomicity, but that doubles storage.
- How do we surface 'someone changed upstream between plan and apply'? Re-fetch on apply, compare against snapshot, abort with a 'remote changed, re-plan' message.

Depends on bf54.3 (last_synced) for atomic apply.

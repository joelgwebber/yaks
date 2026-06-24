---
id: yak-4fde
title: Sync plugin version across manifests + document the bump checklist
type: task
priority: 2
created: '2026-06-24T18:10:10Z'
updated: '2026-06-24T18:10:55Z'
---

The codex plugin.json version (0.1.72) drifted behind marketplace.json (0.1.75) and its longDescription still says yaks are 'version-controlled alongside your code' (pre local/team framing). CLAUDE.md's release rule only names marketplace.json, so the codex manifest gets forgotten. Sync codex plugin.json to the current version + fix its longDescription, and update the release guidance to list every version location.

---
▸ 2026-06-24T18:10:51Z
Synced .codex-plugin/plugin.json 0.1.72->0.1.75 (matching marketplace.json) and refreshed its longDescription to the local/team framing. Rewrote CLAUDE.md Releasing section to name BOTH version locations (marketplace.json plugins[0].version + codex plugin.json top-level version), call out that the marketplace top-level 1.0.0 is the schema version (leave it), and that .claude-plugin/plugin.json intentionally has no version. JSON validates.

---
id: yak-bf54.3
title: Add last_synced frontmatter field (and holistic review of yak timestamp fields)
type: feature
priority: 2
created: '2026-04-25T15:47:28Z'
updated: '2026-04-25T15:57:19Z'
---

Add last_synced: <iso> to yak frontmatter, written by /yaks:sync after a successful merge. Used as the fast-path predicate: skip the diff if upstream.updated <= last_synced; sweep mode (bf54.2) batches a JQL filter against this field.

Mechanics:
- yaklib.model: accept and preserve last_synced through load/save.
- /yaks:update: --last-synced flag (or just let the sync skill rewrite the file directly — flag may be unnecessary if only the skill writes it).
- /yaks:show: display last_synced when present.
- TUI detail render: show last_synced on yaks that have it.
- Make sure created/updated/last_synced don't get trampled by routine field edits.

Holistic review (per user direction): existing 'created' and 'updated' fields earn varying amounts of keep — 'updated' is used by TUI sort and 'recent' filter; 'created' is decorative. With last_synced joining the family, audit whether all three are pulling weight, or whether one can be dropped/folded. Likely outcome: keep all three but document semantics clearly. Don't touch behavior unless audit surfaces something genuinely confusing.

---
▸ 2026-04-25T15:57:19Z
Done. Changes:

- parser.py: added --last-synced flag to the `update` subcommand. Accepts an ISO8601 string or the literal 'now'.
- commands.py cmd_update: stamps task['last_synced'] from the flag value (now → now_iso()).
- detail.py: TUI shows 'Synced' row when last_synced is present.
- docs/index.html: web UI shows 'Synced' meta row when last_synced is present.
- skills/yak-sync/SKILL.md: workflow step 7 stamps last_synced via `yak.py update <id> --last-synced now` as the *final* step of a successful sync.
- skills/yak/SKILL.md, CLAUDE.md, README.md: frontmatter examples and field lists updated.
- 3 new tests in test_cli_basics.py (explicit ISO, 'now', preservation across routine edits). 94/94 pass.

Holistic-review findings (per request to consider date/time fields together):

- created — immutable, set on creation. Decorative; not used by sort/filter. Cheap to keep, useful as 'when this yak was born' context. Verdict: keep as-is.
- updated — bumped on any field mutation, status move, --note append, reparent. Used by TUI tree sort (recent-first) and detail display. Load-bearing. Verdict: keep as-is.
- last_synced — written only by /yaks:sync (or explicit `update --last-synced`). Drift predicate: upstream.updated > last_synced ⇒ upstream changed; local.updated > last_synced ⇒ local changed. Verdict: new field, distinct purpose, no overlap.

No folding warranted. The three serve genuinely distinct roles. Routine edits (--note, --add-label, status moves) preserve last_synced — confirmed by test_update_preserves_last_synced_across_routine_edits.

Plumbing notes:
- Reading is free (load_task already returns whatever's in the YAML).
- Writing is free (save_task writes whatever is in the dict).
- The CLI `show` displays last_synced for free since it dumps the full frontmatter.
- The bead importer does NOT set last_synced (correctly — beads aren't synced, just imported).

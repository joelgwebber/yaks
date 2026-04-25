---
id: yak-bf54
title: External issue tracker sync
type: feature
priority: 2
created: '2026-04-24T02:12:56Z'
updated: '2026-04-25T20:02:36Z'
---

We have a "source" slot that can be used for external issue trackers -- JIRA, Linear, etc. But there's no formal mechanism for syncing yaks with the source issues.

I don't think it would be wise to try and do this programmatically, as schema mapping and such is asymptotically correct at best. What we can do is provide an external sync skill that looks at a single issue and manually synchronizes bidirectional changes as best it can.

Let's make sure to think this through carefully -- we want to create as general a skill as we can without trying to be impossibly comprehensive. Mostly this requires trusting the LM to do its thing; but there are probably some simple things we can do to help the agent focus its efforts efficiently.

I'd also like to keep it generic across upstream issue trackers. E.g., instead of saying "here's how you do it with JIRA", just give general guidance, assume the agent has the necessary tools for whatever issue tracker is being used, and let it deal with ambiguities effectively. Maybe there could be some very narrow, specific guidance about how to most easily find recent modifications, though.

### 2026-04-24T03:02:57Z
Drafted skill + command. Decisions locked: single-yak only, merge-with-prompts, silent upstream (no yak provenance markers), implicit sync state (no last_synced for now), confirm-on-create for new upstream issues (user supplies where). Notes merge by normalized-body hash with conservative duplication; artifacts match on (filename, size). Followups bf54.1 (investigate MCP timestamp availability) and bf54.2 (sync check mode, blocked on .1). Next: dry-run against a live Jira issue.

### 2026-04-25T15:47:58Z
bf54.1 shorn: confirmed Jira exposes everything we need (issue + per-comment + per-attachment timestamps, batched JQL drift query). Locked in 'add last_synced to frontmatter, write it after successful sync'. Spawned bf54.3 (last_synced field + holistic timestamp audit) and bf54.4 (pending-sync sidecar + TUI review pipeline). bf54.2 now depends on both. Linear research deferred — no yak filed yet, will create when we actually wire it up. Sync skill itself remains as drafted; once bf54.3 and bf54.4 land, the skill's interactive-only confirmation loop gets replaced (or augmented) by the sidecar pipeline.

### 2026-04-25T17:14:28Z
Test scenario 1 (no-op sync) ran on jira-301 against /tmp/yaks-sync-test scratch repo. First pass surfaced three drifts caused by incomplete bootstrap (priority defaulted, comments not pulled, jira marker label proposed for upstream push). Three policy decisions baked into SKILL.md: 'when in doubt do not sync', namespaced labels (jira-* round-trips, bare = local-only), priority is upstream-wins (PMs/Engs re-tune frequently). bf54.5 filed for the policy implementation work. Bootstrap re-run with priority + comments + namespaced labels — second pass on jira-301 is a true no-op. Next: test scenario 2 (local edit → propose upstream → deny).

### 2026-04-25T17:18:14Z
Test scenario 2 (local edit → propose upstream → deny) ran on jira-302. Edited local title with ' [test edit]' suffix. Skill detected local drift (local.updated > last_synced) and one-sided title diff vs upstream. At the proposed-push prompt, denied. Re-fetch confirmed upstream summary + updated unchanged — no leak. Hard rule 'never touch the external tracker without confirming' held under denial. Subtle finding: skill currently leaves last_synced alone after denial (drift re-surfaces next sync, which is correct default behavior); a 'mark drift as intentional' opt-in could be a future affordance but not worth a yak yet. Local title reverted for hygiene.

### 2026-04-25T17:29:50Z
Test scenario 3 (external→yak comment ferry) ran on jira-301. Setup: rolled last_synced back to before Clint's comment + removed the ferried block from local body, simulating 'we synced before he commented; now he has.' Diff correctly identified the missing comment via hash-match (no local match for upstream's comment body). Prompt direction was external→yak (safe — no upstream write). Accept applied a '### <iso> @author (from jira:KEY)' block to body. Zero upstream-mutating MCP calls made. Implementation hygiene caught: stamping last_synced with an explicit older timestamp (rather than 'now') leaves local.updated > last_synced, causing spurious 'drift' on next sync — added a one-liner to skill step 7 clarifying always-use-'now'.

### 2026-04-25T17:47:03Z
Attachment finding (raised by user): Atlassian MCP has no upload tool; only writes are comment/worklog/issue-link/createJiraIssue/editJiraIssue/transitionJiraIssue. Read returns filename + URL but no MCP path to fetch bytes either. GitHub Issues image upload is UI-only via an undocumented endpoint — REST API only does release assets. Linear is the only common tracker with clean attachment mutations. Skill sharpened: attachments are now flagged as best-effort with explicit per-tracker reality, and the skill must enumerate manual files rather than silently skipping. Skipping attachment scenarios for the SUBTEXT scratch since the MCP can't ferry; revisit when Linear MCP is in play. Bootstrap.py also skips attachments — fine for now since none of the 50 SUBTEXT issues sampled had any.

### 2026-04-25T17:51:56Z
Test scenario 4 (two-sided drift on jira-301) ran. Setup: added local note via update --note, removed Clint's ferry from body, rolled last_synced to 05:00Z. Skill produced two distinct prompts as designed (one per direction). Accepted external→yak (Clint re-ferried), denied yak→external (local note stays unique). Stamped last_synced=now. Result: structurally clean two-bucket ferry; uncovered a real design hole around last_synced semantics on partial-deny outcomes — bf54.6 is now load-bearing, not polish. All three prior tests still pass; no upstream writes made in any scenario.

### 2026-04-25T17:54:37Z
Test scenario 5 (no source → create upstream → deny) ran on a fresh sourceless yak. Skill's 'Creating a new upstream issue' branch correctly prescribes asking user for tracker + project + issue type before showing payload, and gates the createJiraIssue call behind explicit confirmation. Denied at confirmation; zero Atlassian write-tool invocations across the entire 5-scenario test arc (verified by session call audit). The yak is correctly left sourceless on deny — no half-state.

Net of the dry-run pass:
- All 5 test scenarios pass structurally.
- 4/5 surfaced no design issues; scenario 4 surfaced the last_synced-on-partial-deny hole that promotes bf54.6 from polish to required.
- bf54.5 (namespaced labels + upstream-wins priority + per-field policy) and bf54.6 (suppress-residual-drift prompt) are the next concrete coding work.
- Attachments remain a known per-tracker limitation; skill text now reflects.
- Bootstrap scaffold (/tmp/yaks-sync-test/bootstrap.py) demonstrates a working external→yak import path; if 'yaks:import' becomes a real feature (bf54.x), this script is the seed.

Checkpointing here. Next session can take on bf54.5 and bf54.6 implementation.

### 2026-04-25T18:07:06Z
bf54.5 + bf54.6 shorn. SKILL.md step 3 now states per-field policies explicitly (title/desc/status: prompt; priority: upstream-wins silent; labels: namespaced with bare = local-only). Step 7 handles partial-deny correctly via the 'suppress remaining drift?' prompt. bf54.4 description updated to YAML-formatted sidecar (per project preference). Outstanding: bf54.2 (sweep) and bf54.4 (sidecar pipeline). v1 sync skill is now consistent and dry-run-validated; first real /yaks:sync against live Jira is credible whenever the user wants to attempt it.

### 2026-04-25T18:25:35Z
bf54.4 shorn (v1 sidecar pipeline). Plumbing landed: yaklib/sync.py, 'yak sync ls/show/clear' CLI, '~' marker in list views, .gitignore for sidecars, 15 new tests. Skill rewritten end-to-end with plan/apply/discard, sidecar YAML schema, snapshot-drift abort, suppress-residual-drift prompt. Deferred to bf54.9: TUI interactive review dialog. Outstanding: bf54.2 (sweep) and the user-side bf54.7/bf54.8.

### 2026-04-25T20:02:36Z
bf54.2 shorn (sweep mode). 'yak sync check' CLI + /yaks:sync-check skill flow validated against the SUBTEXT scratch (50 yaks, batched JQL, drift classification correct in both no-drift and demo-drift cases). Outstanding bf54 children: bf54.7 (Linear test, blocked on MCP), bf54.8 (GitHub test, blocked on MCP), bf54.9 (TUI interactive review, p3 polish). v1 sync feature is functionally complete: plan/apply/discard, namespaced labels + upstream-wins priority + per-field policy table, suppress-residual-drift prompt, sweep/check, sidecar-marker in list views. Ready to ship; further work is exercising on more trackers or adding TUI ergonomics.

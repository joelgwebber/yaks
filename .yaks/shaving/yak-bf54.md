---
id: yak-bf54
title: External issue tracker sync
type: feature
priority: 2
created: '2026-04-24T02:12:56Z'
updated: '2026-04-25T17:47:03Z'
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

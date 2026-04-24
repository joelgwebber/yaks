---
id: yak-bf54
title: External issue tracker sync
type: feature
priority: 2
created: '2026-04-24T02:12:56Z'
updated: '2026-04-24T03:02:57Z'
---

We have a "source" slot that can be used for external issue trackers -- JIRA, Linear, etc. But there's no formal mechanism for syncing yaks with the source issues.

I don't think it would be wise to try and do this programmatically, as schema mapping and such is asymptotically correct at best. What we can do is provide an external sync skill that looks at a single issue and manually synchronizes bidirectional changes as best it can.

Let's make sure to think this through carefully -- we want to create as general a skill as we can without trying to be impossibly comprehensive. Mostly this requires trusting the LM to do its thing; but there are probably some simple things we can do to help the agent focus its efforts efficiently.

I'd also like to keep it generic across upstream issue trackers. E.g., instead of saying "here's how you do it with JIRA", just give general guidance, assume the agent has the necessary tools for whatever issue tracker is being used, and let it deal with ambiguities effectively. Maybe there could be some very narrow, specific guidance about how to most easily find recent modifications, though.

### 2026-04-24T03:02:57Z
Drafted skill + command. Decisions locked: single-yak only, merge-with-prompts, silent upstream (no yak provenance markers), implicit sync state (no last_synced for now), confirm-on-create for new upstream issues (user supplies where). Notes merge by normalized-body hash with conservative duplication; artifacts match on (filename, size). Followups bf54.1 (investigate MCP timestamp availability) and bf54.2 (sync check mode, blocked on .1). Next: dry-run against a live Jira issue.

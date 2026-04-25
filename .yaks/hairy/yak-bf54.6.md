---
id: yak-bf54.6
title: After-denial 'ignore drift until upstream changes' affordance
type: feature
priority: 2
created: '2026-04-25T17:28:26Z'
updated: '2026-04-25T17:28:26Z'
---

Surfaced during test scenario 2 (deny path). When the user denies a proposed sync change, the divergence currently re-surfaces on every subsequent sync. Often the user's actual intent is 'leave it alone — my local version is what I want.' Re-prompting them every time is annoying.

Proposal: after a denial, skill offers a follow-up prompt: 'Suppress this drift until upstream changes? [y/n]' If yes:
- Stamp last_synced = max(local.updated, upstream.updated) at the moment of deny.
- Future fast-path checks see upstream.updated <= last_synced → short-circuit with 'no drift.'
- Critically: if upstream subsequently changes, upstream.updated > last_synced again, drift resurfaces (correct — there's something new to consider).
- Critically: the field mismatch itself is NOT recorded anywhere. We're just bumping the watermark. Anyone running a full-diff (e.g., explicit /yaks:sync --force or whatever the explicit-resync gesture turns out to be) sees the divergence again.

Tradeoffs:
- Simple: one watermark bump, no per-field 'ignore' state.
- Coarse: suppresses ALL field drifts, not just the one denied. If user has both title and priority diverged and only wants to ignore the title, this approach would also stop nagging about priority. We could refine later if it becomes a problem.
- Reversible: user can re-run with a force flag to see the drift again, or edit the file to lower last_synced.

Open question: should the prompt default to 'yes' (less nagging) or 'no' (safer / more transparent)? Lean 'no' — the user has to actively opt into 'ignore.'

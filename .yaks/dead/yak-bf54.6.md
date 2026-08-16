---
id: yak-bf54.6
title: After-denial 'ignore drift until upstream changes' affordance
type: feature
priority: 2
created: '2026-04-25T17:28:26Z'
updated: '2026-06-19T16:47:10Z'
parent: yak-bf54
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

---
▸ 2026-04-25T17:51:56Z
Scenario 4 findings (two-sided drift on jira-301): the design hole bf54.6 was filed for is real and important, not just polish. Specifically: skill step 7 ('stamp last_synced as final step of successful sync') is ambiguous under mixed accept/deny. Stamping silently hides denied drift; not stamping loops re-diffs of already-applied changes. The right answer is the explicit 'suppress remaining drift?' prompt this yak proposes — default 'yes, suppress' since the user just reviewed, with an explicit 'no, surface again' for re-prompt cases. Without this, the fast-path predicate cannot be trusted under any partial-sync outcome.

---
▸ 2026-04-25T18:06:59Z
Done. SKILL.md step 7 rewritten: stamp last_synced=now if all changes accepted (or none proposed); on any denial, prompt 'Suppress remaining drift until upstream changes? [Y/n]' (default Y). Y → stamp now, fast-path predicate short-circuits until upstream actually changes. N → leave last_synced alone, drift surfaces next sync. Resolves the design hole surfaced by scenario 4 — the predicate is now consistent under partial-deny outcomes.

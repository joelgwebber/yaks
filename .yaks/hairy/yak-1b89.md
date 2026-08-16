---
id: yak-1b89
title: Activate/edit/save view; Esc reverts to saved (one filter)
type: feature
priority: 2
created: '2026-08-16T23:15:40Z'
updated: '2026-08-16T23:16:33Z'
parent: yak-4473
labels:
- ui
depends_on:
- yak-5892
---

Tranche 3 of yak-4473 (deps yak-5892). Wire the single live filter to Views — no base+overlay stack.

Activating a View loads its spec into the one live filter (the thing f and / already edit). Editing forks an ephemeral 'modified' View, shown with a * on the chip; you may loosen OR tighten any criterion, including removing base criteria (status is just another removable axis). 'Save' persists: update the active View, or fork a new named one. Esc reverts the live filter to the active View's saved spec (change from today, where Esc clears outright); 'empty' is simply the spec of an All view.

Reuses the entire existing filter path (FilterSpec + build_tree); the only new state is 'which View is active' + 'is it modified'. Status widening forks a clearly-marked ephemeral cross-status View (the * + Esc-revert are the guardrail).

Done when: switching Views swaps the live filter; edit/save/Esc-revert behave as specified.

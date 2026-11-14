---
id: yaks-b2ed
title: Autocompletion UI infrastructure
status: shaving
type: task
priority: 3
created: '2026-08-26T13:27:01Z'
updated: '2026-11-14T18:03:58Z'
tags:
- tui
---

We could use this when typing in both single-line fields, and text blocks.

We'll need to work out the visual and triggering affordances in both contexts. Including space-constrained single-line completions, which could get a little tricky.

Two immediately obvious use-cases:
- Label completion (once we have predefined labels)
- Yak-id completion, in desc/comment blocks

---
▸ 2026-09-23T20:10:53Z [Joel Webber]

I *think* this one might already be done, with the inline yak tab-completion stuff?

---
▸ 2026-09-23T21:38:18Z [coordinator]

Partly done, not fully: yaks-5656 shipped Tab-triggered yak-id completion inside editor blocks (desc/comment), reusing the fuzzy picker as the popup. Not yet built: completion in single-line fields (e.g. labels), which is blocked on predefined labels (yaks-f04d). No inline/ghost-text popup infra exists; completion is modal via the picker. Options:
- a. shear b2ed as covered by yaks-5656 and file 'label completion' under yaks-f04d [my lean];
- b. keep b2ed open for single-line completion infra.

---
▸ 2026-10-03T20:31:56Z [Joel Webber]

(a) SGTM

---
▸ 2026-11-14T18:03:34Z [linkplan-lead]
kind: move
from: hairy
to: shaving
thread: ksQQw3f_HRXkT-eF6oV8kx3Nn5LOAFi59ZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-14T18:03:58Z [linkplan-lead]

Shearing as covered, on the option Joel approved on 2026-10-03 (a: shear as covered by yaks-5656, file label completion under yaks-f04d). What this yak asked for exists:

- **Yak-id completion in description and comment blocks:** [yaks-5656](yaks-5656) (Tab opens the fuzzy picker as the popup); `completion_context` in `src/tui/editor.rs`.
- **Tag (label) completion in single-line fields:** [yaks-8d6a](yaks-8d6a) put it in the create and edit form's tags field, known tags first; [yaks-bf1b](yaks-bf1b) and [yaks-1a16](yaks-1a16) (landed `dc8aaa92`) extended it to the `L` editor and the filter drawer's tags row, and made a pick leave the cursor past a separator so Tab moves on. `tag_completion` is the one function behind all three.
- **Predefined labels, the blocker named on 2026-09-23:** [yaks-f04d](yaks-f04d) is shorn: known tags with descriptions live in `schemas/yak.md`.

Not built, and not needed by anything open: inline or ghost-text completion. Completion is modal, through the picker, in every field. If that is wanted it is a new idea yak, not this one.

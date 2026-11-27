---
id: yaks-958d
title: 'One picker to open or link any document: # and completion over yaks, doks, sources and sessions'
status: shaving
type: feature
priority: 2
created: '2026-11-27T02:27:48Z'
updated: '2026-11-27T02:36:27Z'
parent: yaks-7e32
tags:
- tui
---

`#` opens a yak by id or title through a typeahead picker (`NavGoto`, `FuzzyAction::Goto`, `fuzzy_candidates`); there is no equivalent for doks, sources or sessions. Yak-ref completion in editors uses the same yak-only picker, and dok-link completion is a second picker (`dok_candidates`) whose prefix tier never matches (its seed drops `doks/`, the candidates keep it).

Joel, 2026-11-27: one implementation should open both, reusing the existing typeahead.

## Shape

- One picker over every kind (yaks, doks, sources, sessions, release doks), each row showing its kind; ranked by the shared matcher (exact id or path prefix first).
- `#` opens the pick in the pane through the one opener (a yak's detail, a dok in the viewer), from every surface it is bound on today.
- Editor completion inserts the pick's link (`[yaks-1a2b](/…/yaks-1a2b.md)`, `[title](/doks/…)`); add-dependency and reparent use the same picker filtered to yaks.
- Update the `#` help lines in `src/tui/keymap.rs` ("Go to any yak by id / title"), `docs/tui.md` and the skill.

Design: [section 10 of the dok](/doks/design/one-document-model.md#10-tui). Can land before the shared matcher, ranking by the current prefix/substring tiers, and adopt the matcher when it arrives.

#### 2026-11-27T02:36:27Z

*herd-unify · kind: move · from: hairy · to: shaving · thread: ksQQvf9tyN_TT6-lYi87QcE1GpLOAHx7AZIBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

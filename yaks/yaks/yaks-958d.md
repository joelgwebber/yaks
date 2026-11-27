---
id: yaks-958d
title: 'One picker to open or link any document: # and completion over yaks, doks, sources and sessions'
status: shaving
type: feature
priority: 2
created: '2026-11-27T02:27:48Z'
updated: '2026-11-27T02:59:01Z'
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

#### 2026-11-27T02:36:57Z

*herd-unify*

Claim (yakherd `herd-unify`). Owner: worker `pick-1` (Sonnet 5.5), base `4f09d2d2`.

Scope: one picker over yaks, doks, sources and sessions in `src/tui/fuzzy.rs` (replace the yak-only `fuzzy_candidates` and `dok_candidates` with one candidate list whose rows carry a kind and a link destination; fix the `doks/` seed bug), `FuzzyAction::Goto` opening a document through the existing opener used for following a link to a farm file (find it in `src/tui/follow.rs` / `history.rs` `App::jump`, so `o` goes back), editor completion (`src/tui/editor.rs` `CompleteKind`) inserting a markdown link for every kind, `AddDep`/`Reparent` filtered to yaks. Help lines in `src/tui/keymap.rs` (`NavGoto`), `docs/tui.md`, the `yaks` skill if it mentions `#`.

Ranking: keep today's tiers (exact id/path prefix, then substring of id/path, then title), applied across kinds. Do not build a new matcher.

Out of scope: the shared matcher/tokenizer, the CLI, anything outside `src/tui/` plus docs.

Evidence: unit tests for the candidate list (a dok found by title, the prefix tier matching `doks/...`, reparent showing yaks only); insta snapshots updated; a toque headless frame of the `#` picker showing a yak and a dok row, attached as SVG (`yaks render` to PNG to check it). Gate `cargo test -p yaks`. Judge: the yakherd, by looking at the frame.

#### 2026-11-27T02:38:03Z

*pick-1*

Start Fri Nov 27 02:38:02 UTC 2026. farm file: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/yaks/yaks/yaks-958d.md. checkout: /Users/joel/src/yaks/.delta/worktrees/xca7593y2jw6/yaks

![goto-picker](yaks-958d/goto-picker.svg "# picker typed 'layout': a dok row (path hit) and a yak row (title hit), each showing its kind")

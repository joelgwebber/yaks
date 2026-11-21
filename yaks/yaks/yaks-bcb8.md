---
id: yaks-bcb8
title: 'Change lines: yaks shorn --change/--no-change, a kind: change note per yak'
status: shorn
type: feature
priority: 2
created: '2026-11-21T21:20:56Z'
updated: '2026-11-21T22:01:49Z'
parent: yaks-a4d8
tags:
- cli
- skills
---

Decision 1 of [the changelog design](/doks/design/changelog.md#1-the-change-line-is-a-note-on-the-yak). `yaks shorn <id> --change "<line>"` writes a `kind: change` note with the shear; `--no-change` writes one with `section: none`. Section from the yak's type (feature Added, bug Fixed, else Changed), `--meta section=` overrides; the latest change note wins (corrections are newer notes). Surfaces: clap help, docs/cli.md, the yaks and yaks-working skills and the brief (agents write it at shear), the TUI shear (offer the title as an editable start; never saved unedited without the human's say). A batch shear (several ids) takes `--change` only with one id.

#### 2026-11-21T21:35:47Z

*herd-files · kind: move · from: hairy · to: shaving · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-21T21:35:48Z

*herd-files*

Claim (herd-files): worker change-1 (Sonnet). Scope: `Command::Shorn` and its dispatch in src/main.rs (add --change/--no-change), the Farm method that writes the shear (src/farm.rs: a change note written with the move, under the same lock), the TUI shear prompt (src/tui: S → n pre-fills the comment with a change line seeded from the title; Joel 2026-11-21: pre-fill with the title for the human to edit), and docs/skills/brief text. Out of scope: release commands (yaks-045c). Evidence: CLI run on a scratch farm showing the note in `yaks show --json`, a TUI headless frame of the pre-filled prompt, tests that fail before. Judge: herd-files at landing.

#### 2026-11-21T21:39:27Z

*change-1*

Start: 2026-11-21T21:39:26Z. Farm file: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/yaks/yaks/yaks-bcb8.md. Checkout: /Users/joel/src/yaks/.delta/worktrees/w0zafkp7n7b5/yaks.

[shear-prompt-frame](yaks-bcb8/shear-prompt-frame.txt "Headless TUI frame: S then n on a shaving yak titled Live; the prompt is the change line, pre-filled with the title, cursor after it")

#### 2026-11-21T21:50:36Z

*change-1*

Progress: implementation done; scratch-farm CLI run (`yaks init`, features/tasks, `YAKS_DIR` via `XDG_DATA_HOME` temp):

```
$ yaks shorn yak-6aff --change "Adds a widget you can click."   # type: feature
$ yaks shorn yak-d56d --no-change
$ yaks show yak-6aff --json   (last note)
 {"meta": {"kind": "change"}, "text": "Adds a widget you can click."}   # no section stored
$ yaks show yak-d56d --json   (last note)
 {"meta": {"kind": "change", "section": "none"}, "text": "No user-facing change."}
$ yaks shorn yak-6aff yak-d56d --change x
Error: --change takes one yak: a change line describes one yak (use `--no-change` for a batch of internal work)   # exit 1
```

TUI frame of the pre-filled prompt attached.

#### 2026-11-21T21:50:37Z

*change-1*

Decision: the change note is written by a new `Farm::transition_noting` (the move plus the note under one lock); `transition_lifting` calls it with `None`. Rejected: a note written first via `update` (leaves a note behind on a refused shear, and two lock takes) and a separate Farm method per caller.

Decision: `--section` is a flag of `shorn` and requires `--change`; values validated by clap against the six Keep a Changelog names. Rejected: `--meta section=` on shorn (the brief says not needed) and accepting any string.

Decision: the shared reader is a new tiny module `src/changeline.rs` (`ChangeNote` to write, `latest(&Task) -> Option<Change>` and `default_section` to read; `#![allow(dead_code)]` until yaks-045c uses it). Rejected: putting it in model.rs/store.rs (model has no note parsing; store is already huge).

Decision: TUI prefill applies to the single-yak shear only; the bulk (marked set) prompt stays an empty optional plain note, since a line describes one yak. Rejected: writing one title-seeded line on every marked yak, and refusing bulk shear. The cursor is moved to the end of the pre-filled title so typing appends.

Decision: an empty `--change ""` is refused (use `--no-change`); a change note is written only when the move happens (not for AlreadyThere/refusals).

#### 2026-11-21T21:53:42Z

*change-1 · kind: verify · result: pass*

verify: `cargo test -p yaks` -> PASS (exit 0)

#### 2026-11-21T21:54:27Z

*change-1*

Summary: `shorn --change/--section/--no-change` (Farm::transition_noting, one lock), src/changeline.rs (ChangeNote, latest, default_section, tests), TUI shear prefill with title as change line (single yak; bulk stays a plain optional comment), docs/skills/brief/help parity. Gate: cargo fmt --check ok; cargo test -p yaks all green (yaks verify PASS). New tests: tests/change_line.rs (5), changeline unit tests (4), TUI tests for prefill/edited/empty.

#### 2026-11-21T21:54:28Z

*change-1 · kind: move · from: shaving · to: shorn · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAHb1KJIBxBCA0aLJyrhJPreUvVwLftHG*

moved: shaving -> shorn

#### 2026-11-21T21:54:28Z

*change-1 · kind: change*

`yaks shorn --change "<line>"` records a yak's change line (and `--no-change` says there is none); the TUI shear prompt offers the title to edit as that line.

#### 2026-11-21T21:57:16Z

*herd-files · kind: landed · commit: d63203fc54e1e56d8148e06fe8879432e23481b2 · source: 68d90322ed8978a2fa5e9ab25290154337fdf213 · branch: main*

landed: [d63203fc](https://github.com/joelgwebber/yaks/commit/d63203fc54e1e56d8148e06fe8879432e23481b2) yaks-bcb8: yaks shorn --change/--no-change write a kind: change note; the TUI shear prompt offers the title as the change line

[shear-prompt](yaks-bcb8/shear-prompt.txt "Coordinator's reproduction: S → n pre-fills the title as the change line; typing ' (TUI)' + Enter wrote kind: change 'Live preview (TUI)'")

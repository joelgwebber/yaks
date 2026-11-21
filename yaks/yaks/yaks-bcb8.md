---
id: yaks-bcb8
title: 'Change lines: yaks shorn --change/--no-change, a kind: change note per yak'
status: shaving
type: feature
priority: 2
created: '2026-11-21T21:20:56Z'
updated: '2026-11-21T21:50:36Z'
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

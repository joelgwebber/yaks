---
id: yaks-2c10
title: update --description deletes the yak's notes
status: shaving
type: bug
priority: 1
created: '2026-11-27T02:09:08Z'
updated: '2026-11-27T02:36:26Z'
parent: yaks-7e32
tags:
- cli
---

`yaks update <id> --description <text>` replaces the whole body, notes included. Reproduced: create, `--note n1`, `--description d2` leaves only `d2`; `show --json` goes from one note to none.

The TUI already uses `store::with_description`, which replaces the description and keeps the note blocks byte for byte; the CLI passes the new text straight into `TaskEdit::description` ([main.rs](https://github.com/joelgwebber/yaks/blob/7c1452bd5712911a15ba5e244c9079af914a5c76/src/main.rs#L3137-L3155)). The help text "Replace the body (markdown)" is literally true, and yaks-139b's guidance warns about it, but nobody wants the notes gone.

Fix: `update --description` replaces the description only (through `with_description`), and the help says so; a whole-body replace, if anyone needs one, is an explicit flag.

#### 2026-11-27T02:36:26Z

*herd-unify · kind: move · from: hairy · to: shaving · thread: ksQQvf9tyN_TT6-lYi87QcE1GpLOAHx7AZIBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

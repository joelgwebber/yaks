---
id: yaks-15dc
title: Ensure all the internal link types work as expected
status: shorn
type: task
priority: 3
created: '2026-11-19T22:16:40Z'
updated: '2026-11-20T04:23:40Z'
tags:
- tui
---

Various types of links -- it looks like all work except for line numbers:
- [Link](code:src/tui/actions.rs) to source
- [Link](code:src/tui/actions.rs#L10-L20) to source:line
- [Link](code:src/tui/actions.rs@58eb9a6) to source@short-hash
- [Link](code:src/tui/actions.rs@58eb9a6248e509e2e4cca2e158e1e97ca2c9a864) @hash
- [Link](code:src/tui/actions.rs@58eb9a6248e509e2e4cca2e158e1e97ca2c9a864#L10) @hash:line

Though perhaps I have that syntax wrong?

#### 2026-11-19T23:26:57Z

*sched-1*

Let's also consider line ranges in links, like the Github UI supports. This would be helpful for selecting subsets of a file visually, in the peek UI, as well as providing narrowed context for humans and agents.

#### 2026-11-19T23:28:12Z

*sched-1*

Also, also -- Might be cool to have line-range links open a restricted view of a file, from which you can also navigate to the whole file.

#### 2026-11-19T23:29:35Z

*herd-files · kind: move · from: hairy · to: shaving · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-19T23:40:14Z

*herd-files*

Checked (herd-files), with `yaks peek` and the TUI:

- `code:src/tui/actions.rs`, `#L10`, `#L10-L14`, `@58eb9a6#L10` and a full-hash pin all work: the grammar's line form is GitHub's `#L10` / `#L10-L14`, after the `@<rev>`.
- `code:src/tui/actions.rs:10` and `...@<sha>:10` do not: `path:N` is not a form, so `:10` is read as part of the file name ("no such file").

Plan:

1. Accept the `path:N` and `path:N-M` (and `:LN`) shorthand as the same `#L` anchor, for every path link (farm, attachment, code), so the forms people paste from editors and compilers work. `links::dest` keeps writing the `#L` form, and `doctor` keeps checking them.
2. A line-range link opens a narrowed view: the viewer shows only the linked lines (with a header saying so), and a key widens it to the whole file at the same place. Peek already shows only the range.

#### 2026-11-19T23:57:32Z

*herd-files*

Decision (herd-files): how a line-range link opens narrowed.

- **Only a real range narrows** (`#L10-L14`, `:10-14`). A single line (`#L12`, `:12`) opens the whole file on that line, as GitHub does; a one-line view would hide the context the reader came for.
- **The narrowed view is the same viewer, its rows filtered** to the range (from the first row that touches it to the last). So find, yank, Tab between links, `O`/`e` at the line and peek all keep working without a second code path. The header says `lines 10–14 of 120 · w whole file`.
- **`w` toggles** whole file / range, keeping the cursor on the same file line. It is a look, so lowercase (`Cmd::DocWiden`, `doc.widen`, both key families). Rejected: `Esc` peeling the range (Esc already peels selection, then search, then closes; one more layer would make closing a range link take three presses), and a separate "open whole file" link row.
- **History keeps it**: a narrowed view's subject is its link with the range (`links::dest` keeps `#L10-L14`), so `o`/`i` come back narrowed; widened, it is the plain file.
- **Applies to every text file**: code, farm files, doks, attachments, archive members. A code file's range is still marked in the gutter.

#### 2026-11-20T00:07:09Z

*herd-files*

Done (herd-files), in `412882e4` with yaks-9f55.

- **Shorthand:** `path:12`, `:12-40`, `:L12-L40`, after `@<rev>` too, read as `#L12-L40` on every path link (`links::split_line_suffix`). Not when a `#` fragment is present, never on `commit:`, never `:0`. `links::dest` writes the `#L` form. `yaks doctor` output on this farm is unchanged.
- **One line rule:** every viewer row now records its file line (plain text rows included), so a line anchor, the range cut and `e`-at-the-line read one function, `AttachView::row_line`, for code, plain text, markdown and archive members. The code viewer's own landing (`land_on_line`) and its separate open path are gone: `open_code` goes through `open_place`.
- **Narrowed range view:** a range link opens narrowed (`read_place`, so history restores it narrowed; the kept destination carries the range). The header says `lines 10–16 · w whole file`; `w` (`doc.widen`, lowercase, a look) toggles, the cursor staying on its file line. A single line opens the whole file on it.

Evidence:

- `yaks peek code:src/tui/actions.rs:10`, `...@<full sha>:10`, `...:10-12` all print the lines (before: "no such file").
- Tests: `a_colon_line_suffix_is_a_line_anchor`; `a_line_range_opens_narrowed_in_every_kind_of_file` (attachment by `#L` and `:`, a dok with frontmatter, a code file; narrowed, history keeps the range, `w` widens on the same line; a single line is not narrowed); the code-link test now checks narrowing and `w`.
- Headless, release build: [range link narrowed, then `w`](yaks-15dc/coord-15dc-range-and-w.txt).
- Gate: `cargo test -p yaks` 1528 passed, 0 failed; fmt clean. Docs: `docs/cli.md` (shorthand), `docs/tui.md` (narrowed view, `w`, markdown code files), `yaks` skill Links line; `?` help from `BINDINGS`.

#### 2026-11-20T00:07:11Z

*herd-files · kind: move · from: shaving · to: shorn · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: shaving -> shorn

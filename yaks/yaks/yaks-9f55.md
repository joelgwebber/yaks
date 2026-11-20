---
id: yaks-9f55
title: Use markdown renderer for source files with of type .md
status: shaving
type: task
priority: 3
created: '2026-11-19T20:49:16Z'
updated: '2026-11-20T00:07:09Z'
tags:
- tui
---

We do this automatically for doks, but not for linked files in source and (I think?) attachments.

#### 2026-11-19T23:29:34Z

*herd-files · kind: move · from: hairy · to: shaving · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-19T23:40:13Z

*herd-files*

Checked (herd-files): a markdown attachment already renders as markdown (`yak-d170/n.md` shows `markdown`, rendered). What does not is a `code:` link to a `.md` file: it opens in the code viewer, line numbers and all. Plan: a `code:` file that is markdown renders as markdown (read-only, as a code file is), keeping `#L` anchors working by source line; the take-out menu is unchanged.

#### 2026-11-20T00:07:09Z

*herd-files*

Done (herd-files) in `412882e4`: a `code:` link to a `.md`/`.markdown` file renders as markdown (header kind `markdown`, no line gutter), read-only (`E` refuses as for any code file); a range narrows it by source lines. Markdown attachments, farm files and archive members already rendered. One markdown test (`links::is_markdown_name`) for all of them now. Test: `a_markdown_code_file_renders_as_markdown`; seen headless on `code:README.md`. Gate 1528/0.

---
id: yaks-329a
title: 'source add: .mdx gets no text; treat it (and other markdown dialects) as prose'
status: hairy
type: bug
priority: 3
created: '2026-11-18T14:42:37Z'
updated: '2026-11-18T14:42:37Z'
tags:
- cli
- doks
source: 'feedback-to-yaks:cowpaths/joel#2026-11-13: a `.mdx` source gets no `extracted.md`'
---

From cowpaths/joel's feedback dok (2026-11-13 entry): saving an `.mdx` file as a source gave no text copy; the agent linked the original.

Reproduced on 0.0.38: `yaks source add x.mdx` saves `media_type: application/octet-stream` and the body "yaks extracts no text from this type ... Convert it and save it again with `--text <file>`". The same file named `.md` is kept as its own text.

Cause: `is_text_name` and `is_prose_name` in `src/sources.rs` list `md`/`markdown` but not `mdx`; `media_type` likewise. MDX is markdown plus JSX, readable as text. Consider the same for `mdc` (Cursor rules) and `qmd`/`rmd`, which turn up in agent docs.

Evidence: a test that an `.mdx` source keeps its body as text with a markdown media type.

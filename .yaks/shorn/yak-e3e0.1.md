---
id: yak-e3e0.1
title: Inline rendering + [[yak-xxxx]] explicit syntax
type: task
priority: 2
created: '2026-04-14T21:36:47Z'
updated: '2026-04-15T00:25:06Z'
commit: eea0143
---

Rework of the References-section approach. Make yak-ID mentions navigable *in place* within description text, no dedicated section. Rationale: a 'References:' header implies dependency-like semantics the bare token doesn't carry; free-form mentions should render where the author wrote them.

Design:
- Extend DetailLine with links: list[(start_col, end_col, task_id)] spans.
- build_detail_lines: for each description line, strip [[yak-xxxx]] → yak-xxxx, then scan each wrapped chunk for bare yak-IDs. Resolve via find_task_file, skip self + unresolved + code-fence interiors. Attach spans to the DetailLine.
- Renderer paints spans with C_LINK attr, active span (matching detail_span_cursor) with C_LINK_SEL.
- App gains detail_span_cursor; j/k resets it to 0; Tab/[/] cycles through (line, span) pairs globally; Enter follows the active span if any, else falls back to whole-line task_id/open_path.
- Explicit [[yak-xxxx]] syntax: bracket form gets stripped for display. Syntax chosen after conflict check — safe in CommonMark, GFM, Hugo, Jekyll; aligns with Obsidian/Roam/Logseq wiki-links. Rejected #yak-N (GFM issue collision) and @yak-N (mention collision).

Removes the References subheader section added in yak-e3e0.

---
id: yak-5c0c.4
title: Fix TUI bugs and polish interactions
type: task
priority: 2
created: '2026-04-04T15:18:33Z'
updated: '2026-04-04T15:25:50Z'
---

Fix multiple TUI issues uncovered in initial testing:

1. Detail pane inaccessible for tasks without parent/children/deps.
   The focus switch was gated on having navigable links, which excluded
   all the old shorn yaks that have no relationships.

2. Ghost descendants not shown in parent-state tabs. When a parent is
   shaving but children are in other states, the children were invisible
   from the Shaving tab. Now they appear as dimmed ghosts.

3. Underline on parent/child links was visually noisy. Color alone is
   sufficient to indicate navigability.

4. Escape in the search prompt should cancel, not submit whatever's
   currently typed.

5. Help popup assumed a tall enough window. In wide/short terminals,
   sections should lay out horizontally in columns.

6. Unified detail navigation: a single line cursor that scrolls through
   all detail content and highlights when it lands on a link. Tab and
   Shift-Tab jump directly between links as a shortcut.

---
id: yaks-2cd4
title: 'yaks commit: commit the farm''s own changes in one command (public mode)'
type: idea
priority: 3
created: '2026-10-03T23:32:45Z'
updated: '2026-10-03T23:32:45Z'
parent: yaks-b5a0
labels:
- cli
- agent
---

Joel (2026-10-03): leaving dirty yak edits in a team-mode checkout blocks agents from landing (a push to the checked-out branch is refused while the tree is dirty) and hides answers from the agent's clone; the habit to adopt is committing yaks aggressively. Idea: a 'yaks commit' that stages only .yaks/ changes (never code) and commits them with a generated message ('yaks: <summary of moves and notes>'), refusing if other files are staged, so the human's 'Yak herding.' commit is one command (and the TUI could offer it). Design test: a mutation over files plus git, in the tool's lane only if it stays generic; otherwise a skill habit. Open: message format, whether to include artifacts, interaction with pre-commit hooks, private farms (nothing to commit: error clearly).

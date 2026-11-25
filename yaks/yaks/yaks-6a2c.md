---
id: yaks-6a2c
title: 'Skills: work that lands through a pull request stays shaving until merge, with the PR in prs: and linked notes'
status: hairy
type: task
priority: 2
created: '2026-11-25T04:22:18Z'
updated: '2026-11-25T04:22:18Z'
tags:
- skills
source: 'feedback-to-yaks:cowpaths/joel#2026-11-24: the skills don''t say when a yak whose code is in a PR is done, or to record the PR in `prs:`'
---

From cowpaths/joel's feedback dok (2026-11-24 entry). An agent sheared seven yaks (sightmap-cd4b's children) when their PRs opened, with bare "PR #564" notes; Joel expected them to stay shaving until merge with the PR recorded. The agent regrew, re-shaved, ran `update --pr` and linked each PR.

Checked against the current `yaks` skill: "Hard rules" 2 ("Shear as soon as the work is done") and "Evidence before you shear" don't say whether a PR in review is done. `prs:` appears only in the task-format example and the command table. "Links" doesn't cover external URLs.

## Proposed

- `yaks` skill, Workflow (or a short "Work that lands through a pull request"): on opening a PR, `yaks update <id> --pr <url>` plus a note linking it as `[owner/repo#N](url)`; keep the yak shaving; shear when it merges (note the merge commit); `--remove-pr` if it closes unmerged.
- "Links": an external link is a markdown link with the full URL; a bare `#564` is neither clickable nor unambiguous in a farm over several repos.
- Maybe: `yaks shorn` warns when `prs:` lists an open PR (needs a host lookup; a separate yak if wanted).

Related: yaks-213b (coordinator-owned PRs), `yaks-tracker` step 2.

---
id: yaks-76a3
title: Not-in-a-repo error should name -C
status: hairy
type: chore
priority: 3
created: '2026-11-07T15:58:17Z'
updated: '2026-11-18T14:43:00Z'
tags:
- cli
---

Found while ingesting sources in yaks-3610 (0.0.33).

Agents download into a scratch directory outside any git repo, then run `yaks source add` there. The error is accurate but only offers one of the two ways out:

```text
error: /tmp/.../in is not inside a git repository. A yaks farm lives on a branch of one: run yaks from a checkout, or set YAKS_DIR to a farm
```

`yaks -C <checkout> ...` is the usual answer and is not mentioned. Add it to the message. The skill's "Running yaks" section could say the same in one line (the `yaks-research` skill is where an agent hits it, saving downloads).

#### 2026-11-18T14:43:00Z

*feedback-triage*

Seen again in cowpaths/joel: its feedback dok's entry `2026-11-10: \`dok new\` and \`source add\` fail when run from outside a farm checkout` (key `feedback-to-yaks:cowpaths/joel#<that heading>`), hit in two sessions (11-10 from `/tmp`, 11-13 from the scratch dir). Same error, same missing `-C` hint. Two farms, two sessions: worth raising above p4.

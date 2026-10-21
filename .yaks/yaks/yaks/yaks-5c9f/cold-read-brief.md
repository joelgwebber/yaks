# Cold-read brief for a skill change (reusable)

Used for the skills passes of yaks-5c9f (2026-10-05). Spawn a `scout` (a weak, cheap model is a stricter reader), title it `cold-read-N: ...`, and paste the body below, editing only the list of files and the scenario questions.

## Rules for the reader
- Read ONLY the skill files you list (and `land.sh`'s header if the Delta skill changed). No AGENTS.md, README, docs, src, `.yaks/`.
- Do not run any command that changes anything, no network.
- If the text does not answer, say so; do not guess silently.

## Step 1: do the skills load?
Load each skill with the `skill` tool (pass the worktree). A description that fails to parse means the skill cannot be found at all; report per skill, loaded or the exact error.

## Step 2: scenario questions (2-6 lines each, quote the sentence relied on, name skill and section)
Write 6-8 questions that probe EXACTLY what you changed, plus two that probe what you did not (to catch collateral damage). Good shapes:
- "You are a worker and need X. What exact command do you run?"
- "You are the yakherd and Y happens. List, in order, what you do."
- "Which two skills disagree about Z?"
- "What does <new term> mean, and is it the same as <old term>?"

## Step 3: report
The load table; the answers; every contradiction, undefined term or step you could not carry out from the text alone (file and quoted text); and the three edits that would most reduce a cold reader's chance of getting something wrong, each with the passage quoted and your replacement.

## How to use the result
Fix what is a real gap, ignore suggestions that are wrong (a weak reader sometimes invents shell-escaping advice: check it), then re-run ONLY the questions it missed with a second scout before counting the gate as passed.

---
platforms: [all]
source: Jaime, 2026-10-10: PR descriptions, titles and commits should give a sense of a change's overall progress; light advice, no imposed process
follows: 
blocked:
created: 2026-10-10T12:09:18Z
archived: 2026-10-10T20:22:19Z
---
# Progress in commits and pull requests

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
Commits and pull requests can say where their change stands as a whole: yass status <change> --brief prints one line with the change's progress (and, for a piece, its parent's), and yass-work suggests it for commit bodies, pull request descriptions and, in short, titles, without imposing a process.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Given a piece, when you run `yass status <piece> --brief`, then it prints one line: its parent's name, pieces done of all, boxes done of all, then the piece's name and its own boxes — verify: e2e
- [x] Given a large change, or a small one, then `--brief` prints the same line without the piece part (a small change: just its boxes), and marks a change that's done — verify: e2e
- [x] Given `--brief` without a change, then it says it needs one — verify: e2e
- [x] Given `yass-work`, then it suggests the line for commit message bodies and pull request descriptions, and a short form for titles, as advice that doesn't impose a process — verify: manual: review
- [x] Given the README's CLI list, then it shows `--brief` — verify: manual: review
- [x] Given this repo's pull requests from this one on, then their descriptions start with the line — verify: manual: the next pull requests

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `yass status <change> --brief`: the change's name without its date, pieces done (for a large change), boxes, and `done`; a piece adds its own after its parent's
- [x] `yass-work`: "Say where the change stands", suggesting the line for commit bodies, pull request descriptions and, in short, titles, as a suggestion rather than a step
- [x] README CLI list; e2e §44 (642 ok)

## Decisions
<!-- Progress. "- <decision> - <why> (<who>, <YYYY-MM-DD>)", appended as you go. -->
- The line counts what YASS knows: pieces done and boxes done (delivered criteria included), not "piece N of M", which depends on the plan's order - so it stays right whatever order pieces land in (claude, 2026-10-10)
- Names drop their date prefix in the line, which keeps it short enough for a commit body; the full folder name is one `yass status <name>` away (claude, 2026-10-10)
- The advice lives in `yass-work`, next to its git paragraph, and says it's a suggestion: Jaime asked for light process advice that doesn't impose a process (Jaime, 2026-10-10)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-10 (claude)
- Did: `--brief`, the advice in `yass-work`, README, e2e. `go test`, `tests/e2e.sh` (642 ok), `tests/examples.sh`, `tests/site.sh` pass.
- Next: Jaime reviews the advice and README (boxes 4, 5); the line opens this pull request and the next ones (box 6).
### 2026-10-10 (claude)
- Did: Jaime reviewed and merged it (#68), whose description, title and commit body used the line; this pull request does too, and so will the next ones. Done.
- Next: archive it.

---
id: yaks-d695
title: Add pull-request(s) field to frontmatter
status: shaving
type: feature
priority: 3
created: '2026-08-26T12:58:19Z'
updated: '2026-10-31T16:54:51Z'
parent: yaks-039b
labels:
- schema
---

Alongside 'source' for upstream issues, we need a place to track active PR(s) as well.

I think this needs to handle multiple PRs, as that's a pretty common pattern.

Source can be singular, because we want to discourage a single yak from representing multiple issues (that's what herds are for).

---
▸ 2026-10-31T15:56:26Z [delta:Cloud Runners]

Adopted into `yaks-039b` (2026-10-31) so the PRs field stays consistent with the other "where does this yak's code live" fields (`yaks-bdf9` landed shas, `yaks-a3fc` worker branches). Where it lives (frontmatter vs note metadata) follows `yaks-6a11`.

---
▸ 2026-10-31T16:54:08Z [yakherd]

moved: hairy -> shaving

---
▸ 2026-10-31T16:54:09Z [yakherd]

Decision (yakherd, 2026-10-31, within the `yaks-039b` direction): the PR list is current state, so it lives in frontmatter as `prs:` (a list of URLs), beside `source:`. Checkpoint and landed events live in note metadata.

Claim: owner worker `prs-1`. Owns: `model::Task` (new `prs` field), `store` parse/render of `prs:`, the merge driver's list merge for it, `yaks update --pr <url>` / `--remove-pr <url>` (and `create --pr`), the `prs` lines in `yaks show` text and JSON, the TUI detail header, `--json` list output if `source` appears there. Evidence: round-trip and merge tests, CLI tests, a TUI frame. Judge: yakherd.

---
▸ 2026-10-31T16:54:51Z [prs-1]

Started 2026-10-31T16:54:47Z (date -u). Yak file: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-d695.md

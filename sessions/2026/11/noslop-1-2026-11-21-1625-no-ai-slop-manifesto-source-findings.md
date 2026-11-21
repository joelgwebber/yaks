---
type: session-review
title: 'No AI Slop manifesto: source, findings, proposals'
source: session-note
actor: noslop-1
person: Joel Webber
host: joel
harness: delta
thread: ksQQAaR4Et5XRd6SUAsLXtG1X5LOAHWQYJIBxBCA0aLJyrhJPreUvVwLftHG
day: 2026-11-21
written: '2026-11-21T16:25:39Z'
unclear: 1
---

## Summary

Joel asked to save the No AI Slop manifesto as a source and read yaks against it. Saved it, measured the farm (notes by actor, review traces, skill sizes), and wrote [the findings dok](/doks/research/no-ai-slop-manifesto.md) under yaks-2c0d. The proposals went to one idea yak, yaks-a1dc, with an ask for which to do and in what order.

## Turns

- unclear: save the manifesto, internalize it, and map it to yaks' design (Joel hasn't responded yet)

## Saved

- yaks-2c0d: Research: the No AI Slop manifesto and what it means for yaks (created, hairy -> shorn)
  - [Shorn: the manifesto is saved and read against yaks.](/yaks/yaks/yaks-2c0d.md#2026-11-21t162522z)
- yaks-a1dc: Act on the No AI Slop findings: review notes, recorded authorship, heartbeats, budgets (created)
  - [ask: Which of the six proposals should become yaks, and in what order?](/yaks/yaks/yaks-a1dc.md#2026-11-21t162521z)
- created [/sources/2026-11-21-7a2e5a46-no-ai-slop-a-manifesto-aie-code-summit-2026.md](/sources/2026-11-21-7a2e5a46-no-ai-slop-a-manifesto-aie-code-summit-2026.md): No AI Slop: A Manifesto (AIE Code Summit 2026)
- created [/doks/research/no-ai-slop-manifesto.md](/doks/research/no-ai-slop-manifesto.md): The No AI Slop manifesto, read against yaks

## Lessons

- `yaks status` listed the new dok as uncommitted right after `dok set` had committed it; the following `yaks commit` then failed with "nothing to commit". Looks like a stale status read straight after a per-change commit.
- `yaks source add` refuses to run from a scratch dir (not a git repo) even with an absolute file path; running it from the checkout works. The fetching howto already says this.

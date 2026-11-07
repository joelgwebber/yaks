---
type: topic
title: 'Agent harnesses: Amp, T3 Code, and two views of how people work with agents'
tags:
- research
- agent
stale_after: P3M
sources:
- sources/2026-11-05-a5b10867/source.md
- sources/2026-11-05-404a8e9d/source.md
- sources/2026-11-05-1fecf49b/source.md
- sources/2026-11-05-40f53e77/source.md
- sources/2026-11-05-8323537b/source.md
- sources/2026-11-05-0272f64e/source.md
- sources/2026-11-05-c1c81752/source.md
- sources/2026-11-05-c2c88d91/source.md
---

# Agent harnesses: Amp, T3 Code, and two views of how people work with agents

Read on 2026-11-07 for yaks-3610. Joel asked what is worth taking from two harnesses he keeps hearing about, and from two tweets. This page is a first read, not a verdict. What was read, and what was not, is under [Limits](#limits).

## In one paragraph

The two products sit at different layers. **T3 Code** is a thin control surface over harnesses you already pay for. **Amp** is its own harness plus a cloud workspace for every thread. What overlaps with yaks is not the product but a handful of working patterns, and most of those yaks already has or has filed: scheduled wake-ups, evidence as proof, threads tied to commits, and measuring whether agents can use a repo's instructions.

## T3 Code

A landing page, so a vendor's own account of itself ([source](/sources/2026-11-05-404a8e9d/source.md)).

- **What it is:** "the open-source control plane for coding agents". It runs Claude Code, Codex, Antigravity, OpenCode, Cursor and Grok from one surface, with the user's own subscriptions. It does not resell tokens.
- **Git is the unit:** every agent thread writes to its own branch; one button opens a PR with a generated title, body and changelog; inline diff review before pushing; draft, stacked and amended PRs.
- **Reach:** the page lists iOS and Android apps; the desktop app, a browser connection and a remote VM come from user quotes on the page, not from product copy.
- **Fork it:** open source and free; the page invites restyling, adding agents and shipping your own build.
- **Claims to discount:** "tolerated by over 500,000 devs" and the testimonial carousel are marketing.

What it says about the field: a harness can be a UI and a git workflow over other people's agents. That is the same layering as [Haks](/doks/research/haks-orchestration-layer.md) over yaks (read through a CLI, write through a CLI, own nothing).

## Amp

Also a vendor's page ([source](/sources/2026-11-05-a5b10867/source.md)), but it carries a 26-minute transcript of Thorsten Ball using it for a day, which is the most concrete evidence here.

- **Orbs:** a fresh remote machine for every thread, with the code, plugins and tools. They keep working after the laptop closes, so "no worktrees to juggle, no port conflicts". Self-hosted orb providers and macOS machines for iOS builds are announced.
- **Portals:** the dev server of an orb opened in the viewer, with comments on the page itself sent to the agent.
- **Automations:** an agent wakes itself ("next run in 12 minutes"), or on outside events (a Linear issue, CI failing on main). In the transcript one thread ships a performance fix and then checks the rollout for hours.
- **Access by grant:** an orb asks for production access; a human approves it from a terminal (`amp threads continue`); OIDC scopes it.
- **Threads as team memory and multiplayer:** several people steer one agent; threads link to git commits; "artifacts" are documents that belong to a thread.
- **SDLC plugins:** the team wrote tools for bug reports, emailing users, a system-health check and feature flags. The agent reproduces a bug, ships the fix, cuts the release, emails the reporter and closes the issue.
- **Other pieces:** `amp --no-tui` for servers and CI; a "Dial" of model settings that can run on a ChatGPT subscription; a recap feature for threads.
- **Claims to discount:** every testimonial, and all comparisons to other cloud agents.

## "How I work with Agents" (Thorsten Ball)

An X article, 25 October 2026, "as of October 2026" ([source](/sources/2026-11-05-1fecf49b/source.md)). He works at Amp and says so in the video, and the article refers to "Amp" for threads, so read it as the view from inside one product.

Points that bear on yaks, in his words, condensed:

- **Make the whole lifecycle legible to the agent.** "If the agent can't see something, it makes it up": production logs, deploys, rollouts, releases, bug reports.
- **Ask for irrefutable proof.** Walkthroughs, demo videos, before/after numbers. "Put 'this is not evidence' in your AGENTS.md."
- **Automation, schedules, wake-ups and heartbeats are the killer feature.** "Check the logs in 2hrs", "let me know once this is live".
- **Eval agentic knowledge.** Spawn blind agents, control against treatment, to learn whether AGENTS.md, skills and docs get used. The video's "Using Orbs to Eval Agent Friendliness" does exactly this with fresh orbs and no hints.
- **Threads are the company's memory,** connected to commits, so "why did we do this?" has an answer.
- **Review only code with high blast radius.** Fewer than five central files and migrations; style, naming and duplication he no longer reviews. His test: does it work, do I understand it at the arrows-in-a-diagram level, was it tested end to end, can I change it in a month.
- **Don't iterate with the agent; have it build the tool for iterating.** "Generate 20 variations and a little page that lets me combine and test them."
- **Stop managing context by hand;** one thread, one task.
- **Prompt the what, not the how.**

## The Proton manifesto

A leaked internal document, posted as three screenshots by Joseph Cox on 29 October 2026 ([tweet](/sources/2026-11-05-40f53e77/source.md); pages [1](/sources/2026-11-05-8323537b/source.md), [2](/sources/2026-11-05-0272f64e/source.md), [3](/sources/2026-11-05-c1c81752/source.md), my transcriptions beside each image). Cox says he obtained it; 404 Media's piece calls it the CTO's, and it is signed "-Bart".

What it argues:

- "Your job is to help the LLM write better code faster", not to write code. Every engineer is a team leader whose reports are models.
- **Repeated failures should trigger an investigation and a fix** that helps everyone: repository-level context files, shared skills and prompts.
- **Phase 1 is speeding up tasks inside existing workflows; phase 2 is rethinking the workflows,** since removing one bottleneck exposes the next (CI capacity, design, approvals, review). It predicts that "most human code review will end".
- The third obstacle is a **collective-action problem**: one engineer can use an LLM alone, but retooling CI for a company needs organization and money.

Read it as one company's mandate, not evidence about productivity. [404 Media's article](/sources/2026-11-05-c2c88d91/source.md), which is truncated by a paywall in what I saved, reports from nine employees a culture of fear around AI adoption, a CEO telling a security engineer that asking for a few days of pentesting was "loser thinking", and a plan to re-interview staff about AI use. The company's spokesperson did not dispute the reporting. I did not read the rest.

## What lines up with yaks

| Pattern | Where it appeared | Yaks today |
|---|---|---|
| Agents wake on a schedule or event | Amp automations; Thorsten | [Scheduling](/doks/design/scheduling.md): `due:`, `every:`, `yaks tick`; the human kicks it off, by design |
| Proof, not claims | Thorsten; Amp "ask for proof" | Evidence before shear; `verify:` and `yaks verify` |
| Threads tied to commits | Amp | `yaks land` records shas, `yaks commits` reads them; a farm, not a thread, is the memory |
| Measure whether agents can use the repo's instructions | Thorsten; Amp "agent friendliness" evals | Trials filed but not run: yaks-fe33, yaks-7d57; AGENTS.md's cold-read scout |
| A repeated failure becomes a shared fix | Proton manifesto | `yaks-upkeep` session notes and reflection ([design](/doks/design/session-reflection.md)) |
| One surface over many harnesses | T3 Code | [Haks](/doks/research/haks-orchestration-layer.md); yaks' own "portable across harnesses" tenet (yaks-3901) |
| Grant access per agent, revoke it | Amp, orbs with OIDC | Nothing. yaks has no notion of what an agent may touch |

## What does not line up

- **Both products want their own cloud machine per thread.** Yaks assumes the opposite: one farm on a branch, readable from any checkout, plain files, no server. The overlap is in the patterns above, not the infrastructure.
- **Thread-as-memory is a real alternative to a farm.** Thorsten's "threads are your company memory" and the farm's "files are authoritative" are two answers to "why did we do this". Amp's lives in a vendor's database; the farm's is greppable and diffable by anyone. That difference is worth keeping on purpose.
- **"Review only high-blast-radius code"** is a claim about one team's codebase. Nothing here tests it.

## Limits

- T3 Code and Amp: landing pages only. I did not use either, read their docs, or read any source code. Amp's linked pages (`/docs`, `/notes/how-to-build-an-agent`, `/news/*`) are not saved.
- Thorsten's article: the 266 replies are not saved. The video transcript is Amp's own marketing material, edited by them.
- Proton: the 404 Media article is cut off mid-sentence. The manifesto text is complete (three pages).
- The model names in the Amp transcript ("GPT-6", "Fable 5.1") are as transcribed from speech and I did not check them.

## Open questions

- [ ] Is "what an agent may touch" (access grants, per-agent scope) something a farm should model, or does it belong to the harness?
- [ ] Should the unrun trials (yaks-fe33, yaks-7d57) be run now, given two independent sources here treat "blind agents, control against treatment" as the way to test a repo's instructions?
- [ ] Would a human-readable thread of an agent's work, linked from a yak's notes, be worth saving as a source when a harness offers one?

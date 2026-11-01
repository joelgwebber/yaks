---
id: yaks-5d1c
title: 'Actor names that survive fresh shells: persistent per-checkout actor, title-prefix fallback, a naming convention'
status: shaving
type: feature
priority: 2
created: '2026-11-01T15:37:47Z'
updated: '2026-11-01T15:59:34Z'
parent: yaks-039b
labels:
- cli
- skills
---

Report from the memcraft cloud-runner yakherd (2026-11-01, about 8 workers on 0.0.27): one farm held `herd`, `door-1`/`tgt-1` (from `yaks brief --as`) and `delta:Web Deploy v0`. The last came from a single `yaks pickup` run in a fresh shell without `YAKS_ACTOR`, which fell back to the herd's thread title.

## Causes

- **The skill's advice can't work in agent harnesses.** "`export YAKS_ACTOR` at session start" doesn't survive a harness where every terminal call is a fresh shell (Delta, likely others).
- **The fallback is the whole thread title.** It's unstable (threads get renamed) and is not the worker's name, even though spawn titles already start with it (`door-2: …`, per `yaks-coordinating`).
- **The name shape is undefined.** No convention for a name, and none for whether it carries a harness prefix.

## Proposed

1. **Persistent per-checkout actor.** `git config yaks.actor <name>` in the checkout's own git config, read by `actor::resolve` after `$YAKS_ACTOR` and before the harness fallback, plus a way to set it (`yaks whoami --set <name>`, or a `brief`/`init` step). A Delta clone has its own git dir, so this sticks per thread; a plain git worktree needs `git config --worktree`. Fresh shells then inherit the name with no prefix on every command.
2. **A better harness fallback.** Take the title's leading `<name>:` token when there is one (`door-2: add doors` gives `delta:door-2`), else the title as today. Warn on stderr when a write's actor came from a fallback.
3. **Skills.** Replace "export at session start" with "set it once with the persistent setting (or prefix every command with `YAKS_ACTOR=`; shells may not persist)". The coordinator names itself too.
4. **Optional:** `doctor` or `sheds` flag actor names outside the convention.

## Open question

Whether names carry a harness prefix (`delta:herd`, `delta:door-2`) by convention, or only derived names do (asked on this yak). This also decides how `yaks-8003` (a stable thread id in stamps) fits.

---
▸ 2026-11-01T15:37:47Z [yakherd]
kind: ask
needs: human
---

asked: needs human

Should agent actor names carry a harness prefix by convention?

1. **Prefix everywhere** (the runner's proposal): `delta:herd`, `delta:door-2`, `indent:w-1`. Humans keep their git name. Every name says which harness it came from, but the names are longer and the convention has to be taught.
2. **Prefix only derived names** (lean): explicit names stay as given (`herd`, `door-2`); only the fallback adds `delta:`. Combined with the persistent `yaks.actor` setting and the title-prefix fallback, a forgotten name still comes out as `delta:door-2`, never the herd's title. Less to teach; harness origin is visible only when it was derived.

Either way: the persistent per-checkout actor, the title-prefix fallback, and the skills telling agents not to rely on `export`.

---
▸ 2026-11-01T15:54:38Z [Joel Webber]
kind: answer
needs: agent
---

Good catch on the stable thread id question fits into this. I'm not entirely certain I follow the fallback mechanism, but as long as we have something that works fairly consistently, I'm good with your lean. The main thing I care about is ensuring we don't get inscrutable overlap (eg, "herd") or otherwise ambiguous names. And that agents can use them to follow post facto behavior when trying to piece together past changes. Nice side-benefit -- the "sheds" UI gets more constrained, coherent swimlanes (apart from humans, who *should* get long-running yak-lists).

---
▸ 2026-11-01T15:59:34Z [yakherd]
kind: pickup
---

picked up

Taking Joel's answer: my lean (prefix only derived names), plus his two requirements.

- **No ambiguous overlap.** A generic `herd` reused by different coordinators reads as one actor. Seen live in memcraft: `herd` and `delta:herd` are one agent before and after renaming itself, and its old claims stay under `herd`.
- **Traceable after the fact.** Agents must be able to map a name back to the thread that wrote it.

Plan: a persistent per-checkout name (`yaks whoami --set`, stored as `git config yaks.actor`), the title-prefix fallback with a warning, `thread:` metadata on state moves when the harness exposes a thread id, `yaks brief --as` warning on a reused name, and skills that ask for a specific, unique name.

---
▸ 2026-11-01T15:59:34Z [yakherd]
kind: move
from: hairy
to: shaving
---

moved: hairy -> shaving

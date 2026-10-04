---
id: yaks-12a0
title: Resolve the actor from Delta's environment when neither --as nor YAKS_ACTOR is set
type: feature
priority: 2
created: '2026-10-04T04:49:19Z'
updated: '2026-10-04T04:51:13Z'
parent: yaks-7149
labels:
- cli
- agent
- delta
verify: cargo test -p yaks
---

Part (1) of yaks-7149, approved by Joel. In src/actor.rs the resolution is explicit (--as) > $YAKS_ACTOR > git user.name. Insert a harness-identity step before the git fallback: when neither explicit value is set, use Delta's terminal environment: DELTA_THREAD_TITLE if set and non-empty, else DELTA_CURRENT_THREAD_ID. Format (coordinator's call, Joel may veto): 'delta:<title>' with the title trimmed to 40 chars and whitespace collapsed, or 'delta:<thread id>'; the 'delta:' prefix marks a non-human actor in notes, '[delta:Trial 4A: add yaks path]' style. Keep it a small ordered list of (env var, prefix) pairs so another harness is one line. Explicit --as and YAKS_ACTOR always win; git user.name stays the last resort. Out of scope: recording the actor on state transitions (the other part of yaks-7149, needs the transition log yaks-8265). Evidence: unit tests for every resolution path and precedence; a transcript of the built binary showing a note stamped from DELTA_THREAD_TITLE, from DELTA_CURRENT_THREAD_ID alone, and still [explicit] when YAKS_ACTOR is set; docs/cli.md, the --as help, the yaks skill and the yaks-working/yaks-coordinating attribution text updated to say what is derived and that an explicit name is still preferred.

---
▸ 2026-10-04T04:49:19Z [delta-lead]
TRIAL 5 (yaks-fe33): worker 't5-actor', SHORT brief, loads yaks-working itself; model read from the spawn confirmation. Claim committed. Scope: src/actor.rs and the --as help in src/main.rs; docs/cli.md; the attribution lines in the yaks, yaks-working and yaks-coordinating skills. Gate cargo test -p yaks. Judge: coordinator re-runs gate and the transcript.

![yaks-12a0-actor-transcript](artifacts/yaks-12a0/yaks-12a0-actor-transcript.txt)

---
▸ 2026-10-04T04:51:04Z [t5-actor]
Transcript of the built binary in a scratch farm: note stamped [delta:Trial 5A: derive the actor from Delta's] from DELTA_THREAD_TITLE, [delta:abc123] from DELTA_CURRENT_THREAD_ID alone, [t5-actor] with YAKS_ACTOR set, [flagname] with --as, [Git Human] with no Delta vars.

---
▸ 2026-10-04T04:51:04Z [t5-actor]
Implemented src/actor.rs HARNESSES list + resolve order explicit > YAKS_ACTOR > harness > git; 5 unit tests (all paths, precedence, blank fallthrough, whitespace/40-char/multibyte). --as help, docs/cli.md, yaks, yaks-working, yaks-coordinating skills updated. cargo test -p yaks: 335+28 passed.

---
▸ 2026-10-04T04:51:13Z [t5-actor]
Shorn summary: harness identity step added (delta:<title, whitespace collapsed, 40 chars> else delta:<thread id>); explicit/YAKS_ACTOR still win; git last. Evidence: unit tests in src/actor.rs + attached binary transcript; cargo test -p yaks green.

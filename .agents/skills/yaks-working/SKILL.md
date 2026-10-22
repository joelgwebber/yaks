---
name: yaks-working
description: "Use when you are taking one yak from hairy to shorn, as a worker handed one by a yakherd or on your own: re-read it, claim it, keep notes, gather evidence, shear, commit. Minimal and harness-agnostic. Opt-in skill, installed by `yaks skills install --with coordination`."
---

# Working a yak

The habits that leave a trail a human or another agent can trust. **If a yakherd briefed you, follow the brief** (`yaks brief` prints the exact commands, gate and finish for this farm); this skill is the reasoning behind it, and the whole procedure when you work alone.

1. **Re-read the yak now** (`yaks show <id>`, latest note first): someone may have moved it, redirected it or answered a question since you last looked. Check that its dependencies are met.
2. **Claim it** (`yaks shave <id>`, and its parent if hairy) unless your brief says it is already shaving. Name yourself on every command (`YAKS_ACTOR=<name>` or `--as <name>`).
3. **Keep a log.** One event per note. A choice a reviewer could argue about gets its own `Decision:` note with the alternatives you rejected: a decision that lives only in the code is invisible.
4. **Ask, don't guess, on a real fork** (behavior a user sees, a data format, a dependency): `yaks ask <id> --note "<question and your lean>"`, leave your edits in place and return. Never answer your own ask; `yaks pickup <id>` takes on the answer when you resume.
5. **Touch only your yak.** Changes you didn't make (a yak that moved, a new note) are someone working beside you, not an error.
6. **Shear against evidence, not "it compiles":** a recorded `yaks verify <id>` PASS, or what you saw attached with `yaks attach`. A UI change also updates its in-app help, `--help` and docs. Don't self-shear a subjective call: attach it and ask. No way to verify it? Ask whether to build one.
7. **Finish:** a summary note (what changed, what you learned, the evidence, any yaks spawned), `yaks shorn <id>`, then one commit of your own files with explicit paths. On a branch farm never stage `.yaks/` (the yak id goes in the message); on a private farm keep yak ids and the word "yaks" out of the commit entirely.

Coordinating several workers is `yaks-coordinating`.

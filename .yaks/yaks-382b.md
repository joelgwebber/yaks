---
id: yaks-382b
title: Give explicit markdown formatting guidance in skills
status: hairy
type: bug
priority: 1
created: '2026-10-05T22:10:36Z'
updated: '2026-10-10T20:06:05Z'
labels:
- markdown
- skills
---

I keep seeing massive chunks of md pasted into a single-line description, making it nigh-unreadable. This may be a byproduct of agents working through the CLI. We should give explicit guidance on markdown formatting, and ensure that it follows the constructs we actually implement in markdown rendering.

---
▸ 2026-10-05T22:26:45Z [Joel Webber]
Possibly related: yaks-c53a

---
▸ 2026-10-10T20:06:05Z [delta-lead]
Concrete case for the guidance (2026-10-10, see the failure-mode note on yaks-c53a): the markdown-guidance work here should also say HOW to pass a body, not only how to format it. The one rule: never put a note or description body inside `$( )` or a double-quoted argument. Use `--note - <<'EOF'` (heredoc piped on stdin) or `--note-file PATH`. In a quoted argument or `$(cat <<EOF)`, backticks and `$` in the body can execute (a worker's note ran `top`, which hung its terminal for 10 minutes, and `yaks discover`, whose output ended up in the note).
Where it must appear: the yaks skill (Writing notes and descriptions) already says to pass multi-line text on stdin or from a file; add the explicit "never `$( )`" line there, in `yaks brief`'s output (src/brief.rs, the multi-line notes sentence), and in the coordinating skill's worker brief template. Today the brief says "piped on stdin (`--note - <<'EOF'` ... `EOF`), never as one long argument", which a worker followed in letter and still wrapped in `$(cat ...)`.

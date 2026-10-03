# .agents/skills — this repo's skills (the only copy)

These are the real `SKILL.md` files for this repo: no `skills/` directory, no
symlinks. Agent harnesses (Delta, and anything else that scans
`.agents/skills/<name>/SKILL.md`) load them from here, and the binary embeds the
shipped ones from here.

- **Shipped:** `yaks` and `yaks-tracker`. The explicit `BUNDLED` list in
  `src/skills.rs` — not the contents of this directory — decides what is
  embedded in the binary and installed by `yaks skills install`.
- **Repo-internal, not shipped:** `yaks-coordinating` and `yaks-working` —
  simplified, pstack-inspired workflow skills we use to develop yaks itself and
  to test the coordination concepts tracked under `yaks-3901`. Nothing here
  that isn't in `BUNDLED` is ever embedded or installed; the same goes for any
  other project-local skill dropped into this directory.

`yaks skills install` refuses to write into this directory (the source-tree
guard, yaks-d8e9), with or without `--force`.

Keep each skill minimal. The goal is the least guidance that measurably changes
behavior — judged by whether it cuts turns and yak-herding drudgery — not a
heavyweight framework. Promote a convention into the tool (a CLI flag, a lint, a
field) once it has earned its place; until then it lives here as prose.

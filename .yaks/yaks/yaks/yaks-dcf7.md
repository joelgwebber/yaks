---
id: yaks-dcf7
title: Release v0.0.24
status: shorn
type: task
priority: 2
created: '2026-10-27T12:58:55Z'
updated: '2026-10-27T13:10:39Z'
labels:
- release
---

---
▸ 2026-10-27T12:58:55Z [delta-lead]

moved: hairy -> shaving

---
▸ 2026-10-27T13:10:39Z [delta-lead]

Released v0.0.24 (delta-lead, at Joel's request: "land everything and cut a new release").

- No farm schema change since v0.0.23, so a plain patch bump: `Cargo.toml`, `Cargo.lock` and `npm/yaks/package.json` to 0.0.24 in `c815bed`.
- Gate before tagging: `cargo test`, 875 passed, 0 failed; `node npm/yaks/test-mapping.mjs` ok.
- `main` pushed to `origin` (`2b1f3ba..c815bed`, 16 commits) and to the human's checkout; tag `v0.0.24` pushed to both.
- Release workflow: https://github.com/joelgwebber/yaks/actions/runs/40508385024 succeeded (tests, five platform builds, publish).
- npm: `@j15r/yaks` latest is 0.0.24. A fresh `npm i @j15r/yaks@0.0.24` runs `yaks 0.0.24` with `request`, `wait` and `pickup --drop-request`.
- Local: `cargo install --path .` gives `yaks 0.0.24`.

In this release: ask/answer use the full editor (yaks-ce5e); `yaks request` plus answered/requested in the inbox, TUI `a`/`Q` (yaks-b75b); `yaks wait` (yaks-f313); `--run-on` (yaks-ef3b); `request --kind tidy|summarize` and `bulk --request` (yaks-f853); `pickup --drop-request` (yaks-092f); skills guidance on markdown, evidence and linking yaks (yaks-8aeb, yaks-df02, yaks-248d); local builds via `cargo install` (yaks-be32); plus the image work already on main (yaks-0bf0, yaks-4e9c, yaks-89fc).

---
▸ 2026-10-27T13:10:39Z [delta-lead]

moved: shaving -> shorn

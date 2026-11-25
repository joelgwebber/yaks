---
id: yaks-dc7c
title: YAKS_DIR at a schema stub (<farm>/.yaks) gives schema advice; say it is not a farm root and name the real one
status: hairy
type: bug
priority: 3
created: '2026-11-25T04:22:33Z'
updated: '2026-11-25T04:22:33Z'
tags:
- cli
- skills
source: 'feedback-to-yaks:fullstorydev/surveyor#2026-11-24: `YAKS_DIR` at the wrong directory gives a misleading schema refusal'
---

From fullstorydev/surveyor's feedback dok (2026-11-24 entry). An agent set `YAKS_DIR=<farm checkout>/.yaks` (guessed from old `.yaks/` paths); that directory is the schema 9 stub (`README.md`, `schema`). yaks said the farm was on schema 9 while `origin/yaks` was on 10 and to `pull --ff-only`; the pull did nothing and the refusal stayed.

Reproduced on 0.0.41 with a stub dir: "... is on schema 9, this yaks writes schema 10: run `yaks migrate`" (`src/store.rs` schema check). The schema check runs before anything asks whether the dir is a farm root.

## Fix

- Before the schema check: a dir that holds only the stub (`schema` + `README.md`, no `config.yaml`, no herds/doks) whose parent has a `schema` is not a farm root: say so and name the parent (and what `yaks path` would find).
- Skills: say where the farm root is (`yaks path`); grep `.agents/skills/` for leftover `.yaks/<herd>/...` paths (the feedback names "Task format" and "Doks").

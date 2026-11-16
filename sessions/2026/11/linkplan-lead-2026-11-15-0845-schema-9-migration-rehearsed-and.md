---
type: session-review
title: Schema-9 migration rehearsed and documented
source: session-note
actor: linkplan-lead
person: Joel Webber
host: joel
harness: delta
thread: ksQQw3f_HRXkT-eF6oV8kx3Nn5LOAFi59ZIBxBCA0aLJyrhJPreUvVwLftHG
day: 2026-11-15
written: '2026-11-15T08:45:54Z'
good: 2
partial: 1
---

## Summary

Built and rehearsed the schema-9 migration for [yaks-1151](/yaks/yaks/yaks-1151.md): the farm moves from `.yaks/` to the branch root, links become paths a markdown host follows, sources get slugs, and notes get the `#### <timestamp>` shape. The work landed on local `main` (not pushed) and was rehearsed on clones of five real farms, with the numbers and the checks on [yaks-acc0](/yaks/yaks/yaks-acc0.md). Docs, skills and clap help were swept in [yaks-df4e](/yaks/yaks/yaks-df4e.md). No real farm was migrated; that is the human's call.

## Turns

- good: build the migration myself and rehearse it on clones, then bring before and after numbers
- good: update the docs, skills and help for the new layout
- partial: leave a build the human can try. The tree is installed, but deliberately as the schema-8 build, so the real farms still open until they are migrated; the human has to install the schema-9 tree right before migrating.

## Lessons

- Rehearse on a clone with a bare clone as its `origin`, never a copy of the farm worktree; a `git mv` of 1,300 files shows as renames and `git log --follow` still reaches back, so the move is cheap to review.
- A migration that moves paths strands anything a late writer commits at the old path. The refusal when behind upstream is not enough alone: `yaks check` now reports `stray-layout`, found by simulating a late writer on a clone.
- Two printers of the same thing (`update --note` and `session touched`) both emitted the old note-link form until a test failed. When a form changes, grep for every emitter, not only the one in the task.
- Tests that build a temp directory from the process id alone collide when two tests in one process do; that made `doks_tab` fail one run in three.
- A worker's friction, recorded on yaks-1151: the brief names `/Users/joel/.local/bin/yaks` in one place and `~/.cargo/bin/yaks` in another; `YAKS_DIR` wants the directory holding `config.yaml` but the error says "not a .yaks/ farm"; a failing test's name scrolls out of `yaks verify` output.

## Saved

- The schema-9 migration, `stray-layout`, host-form note links and the doc sweep, as commits on local `main`.
- The farm dok `howto/migrate-a-farm.md`, with the schema-9 order of steps.

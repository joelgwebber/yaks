---
type: howto
title: Migrate a farm to a new schema
description: Rehearse on a clone, migrate, check, then publish
tags:
- wiki
---

# Migrate a farm to a new schema

When a new yaks writes a newer farm schema, every farm it opens must be migrated once, and every machine that writes the farm needs the new binary first. Done for this farm on 2026-11-02 (schema 6 → 7, [yaks-37da](yaks-37da)).

1. **Install the new build everywhere first.** An older binary refuses a newer farm, and the new one refuses an older farm. From the code checkout: `cargo install --path .`.
2. **Rehearse on a clone, never a copy.** A copied farm worktree carries a `.git` file pointing back at the real repository, so git commands in the copy act on the real farm. Clone the branch instead:

   ```sh
   git clone -q -b yaks "$(git rev-parse --git-common-dir)" /tmp/farm-rehearsal
   YAKS_DIR=/tmp/farm-rehearsal/.yaks YAKS_NO_AUTOCOMMIT=1 yaks migrate
   git -C /tmp/farm-rehearsal diff --stat
   YAKS_DIR=/tmp/farm-rehearsal/.yaks yaks doctor
   ```

3. **Migrate for real:** `yaks migrate --dry-run`, then `yaks migrate`. With per-change commits it commits itself; `yaks commit` says nothing is left.
4. **Check:** `cat "$(yaks path)/schema"`, `yaks doctor`, `yaks check`.
5. **Publish** with `yaks sync` when everyone who writes the farm has the new binary.

For schema 7 the migration renamed `labels:` to `tags:` in 562 yaks and added a blank line after 206 comment headers; doctor's one finding afterwards (a never-committed screenshot in yaks-8222) predates it.

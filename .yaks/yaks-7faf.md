---
id: yaks-7faf
title: Release v0.0.15
status: shorn
type: task
priority: 3
created: '2026-10-12T01:54:23Z'
updated: '2026-10-12T02:20:13Z'
labels:
- release
---

---
▸ 2026-10-12T01:54:35Z [delta-lead]
moved: hairy -> shaving

---
▸ 2026-10-12T01:54:35Z [delta-lead]
Version 0.0.15 (Cargo.toml, Cargo.lock, npm/yaks/package.json in lockstep). Plan: push origin main (done, a0f6489), bump + commit, manual workflow dispatch dry run first (yaks-3479's lesson: a dry run does not exercise the registry's per-package auth, but it does catch a bad build/version lockstep before tagging), then tag v0.0.15 and push the tag for the real publish.

---
▸ 2026-10-12T01:59:40Z [delta-lead]
Dry run (workflow_dispatch, run 38193818126) green: tests, all 5 platform builds, npm assembly all passed with version correctly resolved to 0.0.15, publish step ran --dry-run (nothing published, confirmed in the logs). Tagging v0.0.15 now for the real publish.

---
▸ 2026-10-12T02:10:39Z [delta-lead]
Partial publish, same failure family as v0.0.13 (yaks-3479) but a different symptom. Tag v0.0.15 pushed, workflow run 38194044421 (triggered by the tag) completed all green: tests, 5 builds, and the publish step itself showed no error for any of the 6 packages -- each printed a normal npm publish success sequence (tarball details, "Signed provenance statement", "Provenance statement published to transparency log", "Your package is being processed").
Real check (not just the log): @j15r/yaks, @j15r/yaks-darwin-arm64, @j15r/yaks-linux-arm64, @j15r/yaks-linux-x64, @j15r/yaks-win32-x64 all verified via `npm pack <pkg>@0.0.15` (npm's real download path, not a guessed URL) -- all 5 downloaded successfully. `npx -y @j15r/yaks@0.0.15 --version` also printed `yaks 0.0.15` end to end.
@j15r/yaks-darwin-x64@0.0.15 is broken: its packument (`registry.npmjs.org/@j15r/yaks-darwin-x64`) lists 0.0.15 in `versions` and as `dist-tags.latest`, but the tarball itself 404s (`npm pack @j15r/yaks-darwin-x64@0.0.15` fails with E404), confirmed three times over about 6 minutes (not propagation lag -- the other 5 packages, including ones that also lagged briefly, all resolved within that window). npm's own registry has recorded the metadata but never actually stored the blob.
Not yet decided: whether to retry publishing just that one package (npm normally refuses "cannot publish over an existing version", and the version IS already recorded, even though the artifact is missing), unpublish it (72h window, risky, and may not even be the registry's problem to fix that way), or follow yaks-3479's precedent and bump to 0.0.16 for a full clean re-publish. Asking Joel before doing any of those -- each is its own irreversible, user-facing action.

---
▸ 2026-10-12T02:10:40Z [delta-lead]
asked: needs human

@j15r/yaks-darwin-x64@0.0.15 is broken on npm (metadata says it exists, tarball 404s -- confirmed via npm pack, not just a curl guess). The other 5 packages are fine and verified working (npx @j15r/yaks@0.0.15 --version runs). How do you want to handle this: wait longer and recheck, retry publishing just that one package, unpublish it, or bump to 0.0.16 and do a full clean re-publish (the precedent from yaks-3479/v0.0.13)?

---
▸ 2026-10-12T02:19:57Z [delta-lead]
Confirmed: @j15r/yaks-darwin-x64@0.0.15 is fully live. Re-checked all 6 packages directly against the registry (npm pack + a raw tarball fetch for each, not just the packument): every tarball now returns 200 and downloads real content, and `npx -y @j15r/yaks@0.0.15 --version` prints `yaks 0.0.15`. It really was npm's own "may take a few minutes" propagation delay, just past the ~6 minutes I'd waited when I filed this -- not a repeat of v0.0.13's actual publish failure (that one never resolved; this one did). v0.0.15 is fully released: all 6 packages live, the GitHub tag and release workflow run are clean, origin/main has everything.

---
▸ 2026-10-12T02:19:57Z [delta-lead]
moved: shaving -> shorn

---
▸ 2026-10-12T02:20:13Z [delta-lead]
picked up

Confirmed with Joel directly (chat) and verified myself: v0.0.15 is fully released, all 6 packages live. Closing out.

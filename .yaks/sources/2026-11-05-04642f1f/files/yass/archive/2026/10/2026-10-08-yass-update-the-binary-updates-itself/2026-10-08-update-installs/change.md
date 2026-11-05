---
platforms: [all]
source: 
follows: 
blocked: 2026-10-08-update-check
created: 2026-10-08T20:14:53Z
---
# Update installs

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
yass update fetches, verifies (checksum, then provenance through gh) and swaps the binary in place, on every platform, changing nothing when anything fails.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC3–AC12, AC14, AC15 in [plan.md](../plan.md)

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `install.go`: refuse before downloading if the binary's folder isn't writable; check the release exists (its `checksums.txt`); fetch the archive and `checksums.txt` into a temp folder; checksum; `gh attestation verify` when `gh` is installed and signed in; unpack (`.tar.gz`, `.zip`); write the new binary beside the old, run `<new> version` and require the version asked for; swap by rename (aside to `.old` on Windows, removed on the next run)
- [x] `yass update` installs; `--require-provenance`; listed in `yass --help`
- [x] go test (`TestSwapBinary` for both strategies and a failed Windows swap, `TestChecksumOK`); e2e section 36 with fake releases and fake `gh` on a PATH with nothing else (555 ok)
- [x] A real install from github.com in scratch: a release-marked 0.2.0 build ran `yass update --version v0.3.0`; the download, checksum, provenance (the real `gh`) and swap all passed
- [-] AC12's manual part: one update on Windows, which can't run here

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- The Windows step is dropped, not done: AC12 now asks for macOS only, and Windows moved to `2026-10-09-validate-yass-update-on-windows-and-linux` (Jaime, 2026-10-09)
- "Not signed in" is checked with `gh auth status` before verifying, so an unauthenticated `gh` reads as "provenance not checked", as the design says, rather than as a failed verification (claude, 2026-10-08)
- Downloads follow redirects (GitHub serves assets from another host) while the version lookup doesn't (it reads the redirect itself) - two HTTP clients (claude, 2026-10-08)
- e2e runs `yass update` with a PATH holding only `git` and, when wanted, a fake `gh`: the real `gh` on a developer machine or a CI runner would otherwise verify fake archives against GitHub (claude, 2026-10-08)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-08 (claude)
- Did: built and tested installing: fetch, verify, check the new binary, swap. `go test`, `tests/e2e.sh` (555 ok), and a real install from github.com pass.
- Next: AC12's Windows run (Jaime, or once a Windows machine is at hand); then this piece is done, and `update-for-agents` can start.
### 2026-10-08 (claude)
- Did: merged (#52).
- Next: AC12's Windows run (Jaime). AC17 runs against the real 0.4.0 release once it's tagged.
### 2026-10-09 (claude)
- Did: dropped the Windows step here: Jaime revised AC12 to macOS only, and a real update on Windows moved to `2026-10-09-validate-yass-update-on-windows-and-linux`. The piece is done.

# yass update: the binary updates itself: design
<!-- Intent: the calls that are hard to undo. Edit only in commits without code. -->

## Context
`yass update` replaces an executable on people's machines with one it downloaded, which makes it the most security-sensitive thing YASS does. It also fixes, for every later release, how a binary knows it came from a release, how versions are discovered, and how the swap happens on each platform. The PRD settled the product calls with Jaime on 2026-10-08: `update` beside `upgrade`, checksums always plus provenance through `gh` when present, match the repo's version by default and tell about newer ones, leave source builds alone, a `yass-update` playbook. These are the mechanics.

## 1. Knowing a binary came from a release
**Options.**
- (a) Release builds carry a marker set at build time: GoReleaser adds `-X main.channel=release` beside `main.version`.
- (b) Infer it from the version's shape: `0.3.0` (ldflags, no `v`) is a release, `v0.3.0` is `go install`, a pseudo-version is a clone.
- (c) Infer it from where the binary lives (`$(go env GOPATH)/bin` means `go install`).

**Decision: (a)** (claude, 2026-10-08). It's explicit and survives changes to how versions are written. (b) works today by accident of formatting and would break silently; (c) misses `GOBIN` and copies. Without the marker, `yass update` treats the binary as a source build (R8) and says how to update it: `go install github.com/donjaime/yass/cmd/yass@<version>` when the version is a plain tag, `git pull` and rebuild for a clone build. Release binaries from before this change (0.3.0) can't run `yass update` anyway; the docs say to update those by hand once.

## 2. Finding versions
**Options.**
- (a) The latest version from the redirect of `https://github.com/donjaime/yass/releases/latest` (to `…/releases/tag/vX.Y.Z`); a specific version by fetching its assets directly.
- (b) The GitHub REST API (`/repos/donjaime/yass/releases/latest`).

**Decision: (a)** (claude, 2026-10-08). The API allows 60 unauthenticated requests an hour per IP, which shared CI runners and office networks exhaust; the redirect has no such limit and needs no JSON. The repo's version is the newest stamp among the files `yass status` already reads for its version note (`versionNote` in [`upgrade.go`](../../../internal/yass/upgrade.go)). Pre-releases are never "latest" (GitHub excludes them); `--version v0.5.0-rc.1` installs one on purpose.

## 3. Verifying
Download the platform's archive and `checksums.txt` into a temporary folder; refuse unless the archive's SHA-256 matches its line. Then, when `gh` is on PATH, run `gh attestation verify <archive> --repo donjaime/yass`: the release workflow attests every archive in `checksums.txt`. A failed verification refuses. `gh` missing, or not signed in, isn't a failure: the output says provenance wasn't checked and why, and `--require-provenance` turns that into a refusal (Jaime chose this in the PRD). Only after both does anything outside the temporary folder change.

## 4. Replacing the binary
**Options.**
- (a) Write the new binary to a temporary file in the same folder as the running one (`os.Executable`, symlinks resolved), make it executable, run `<new> version` to check it starts and reports the expected version, then rename it over the old one. On Windows, which can't overwrite a running `.exe`, rename the old one aside to `yass.exe.old` first, move the new one in, and delete any `yass.exe.old` on the next run.
- (b) Download over the binary directly.
- (c) Hand off to `install.sh`.

**Decision: (a)** (claude, 2026-10-08). A rename within a folder is atomic on Unix, so an interrupted update leaves either the old binary or the new one, never half of one (R7). Checking that the new binary runs before the swap catches a wrong-platform download. (b) breaks the binary on any interruption; (c) needs bash, which Windows may not have. If the folder isn't writable, nothing is downloaded: `yass update` checks that first and says to rerun with permission to write there, or to install into a folder that's yours (R9).

## 5. A test seam for the release source
End-to-end tests can't depend on github.com. `YASS_RELEASES_URL` (unset: `https://github.com/donjaime/yass/releases`) points `yass update` at a local server holding fake releases, and `gh` is faked with a script on PATH. It's undocumented for users: supporting mirrors as a feature is a PRD non-goal, with what that implies (docs, compatibility, support for their layouts). (claude, 2026-10-08)

## 6. Teaching agents
**Options.**
- (a) A sixth playbook, `yass-update`, shipped and upgraded like the others.
- (b) Steps inside `yass-status` and `yass-work`.

**Decision: (a), with pointers from (b)** (Jaime chose the playbook in the PRD). Updating touches the machine (a download, a replaced binary) and the repo (an upgrade commit); it needs its own permission rules, which don't belong in a read-only status report or in building a change. `yass upgrade` already adds any playbook the binary carries to a skills folder that has the others, so existing repos get it on their next upgrade. `yass-status` and the `AGENTS.md` section point to it when versions are out of step.

## Consequences and rollback
- **The release workflow gains a line** (`-X main.channel=release`); a release built without it can't update itself, and `yass update` says so.
- **Rolling back an update** isn't `yass update`'s job: it never downgrades (R4). The docs say how to reinstall a specific version by hand (a release download, or `install.sh --bin-dir … --version`).
- **A compromised GitHub release** with matching checksums passes when `gh` isn't installed; that's the trade Jaime made against shipping Sigstore. `--require-provenance` is the answer for anyone who needs more, and the docs say so.

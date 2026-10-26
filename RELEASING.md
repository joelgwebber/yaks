# Releasing yaks

`yaks` ships as a single static binary. Two distribution paths:

## npm (primary; esbuild/Biome-style)

A thin `@j15r/yaks` launcher package declares one prebuilt binary per
platform as `optionalDependencies` (each `@j15r/yaks-<os>-<arch>`); npm
installs only the one matching the host's os/cpu, and `bin/yaks.js` execs it.
Users get it via `npm i -g @j15r/yaks` (command stays `yaks`) or
`npx @j15r/yaks`.

### Before tagging: try the build locally

`cargo install --path .` installs the tree you are about to tag into
`~/.cargo/bin` (put it ahead of npm's bin directory on your `PATH` if you also
have the npm package). Run it against a real farm before you push the tag; the
manual workflow run below covers the other platforms.

### Automated (primary): the `release` workflow

`.github/workflows/release.yml` does the whole thing:

- **Push a tag `vX.Y.Z`** -> it runs the tests, matrix-builds all five target
  binaries, assembles `dist/npm/*` via `build-npm.mjs`, and publishes every
  package to npm.
- **Run it manually** (Actions -> release -> Run workflow) -> identical pipeline
  but ends in `npm publish --dry-run`: nothing is published, no auth needed. Use
  this to exercise the full cross-platform build before you tag.

The workflow enforces version lockstep: the tag (minus `v`) must equal both
`Cargo.toml`'s version and `npm/yaks/package.json`'s version, or it fails before
publishing. So the steady-state release ritual is: bump those two (below),
commit, then `git tag vX.Y.Z && git push --tags`.

### Auth: npm Trusted Publishing (OIDC), no stored token

The publish job requests an OIDC token (`permissions: id-token: write`) and lets
the npm CLI exchange it for a one-shot publish credential — so steady-state
releases need **no npm token at all**, and get provenance attestations for free
(public repo + public package). Requirements the workflow already meets: npm
>= 11.5.1 (we `npm i -g npm@latest`), Node >= 22 (we use 24), GitHub-hosted
runners, and each package's `repository.url` matching the GitHub repo.

Trusted publishing is configured **per package** on npmjs.com
(Package -> Settings -> Trusted Publisher -> GitHub Actions):

- Organization or user: `joelgwebber`
- Repository: `yaks`
- Workflow filename: `release.yml` (filename only, with the extension)
- Environment: leave blank

We publish six packages, so this is done six times — for `@j15r/yaks`
and each `@j15r/yaks-<os>-<arch>`.

### First release is a bootstrap (one-time token)

Trusted publishing can only be configured on a package that **already exists**,
and only CI can build all five platform binaries — so the very first publish
can't use OIDC. Bootstrap it once:

1. Create a granular npm token (publish rights to the `@j15r` scope;
   "bypass 2FA" is required for CI use). Add it as the **`NPM_TOKEN`** repo
   secret (Settings -> Secrets and variables -> Actions).
2. Tag `vX.Y.Z` and push it. OIDC isn't configured yet, so npm falls back to the
   token and publishes all six packages, creating them.
3. Configure a trusted publisher on each of the six packages (fields above).
4. **Delete the `NPM_TOKEN` secret** and revoke the token. From now on every
   tagged release publishes via OIDC with no stored secret.

(Optional hardening, once trusted publishers are set: on each package's Settings
-> Publishing access, choose "Require two-factor authentication and disallow
tokens" — OIDC keeps working.)

### Manual (fallback)

1. Build release binaries for each target into `artifacts/<rust-target-triple>/yaks[.exe]`
   (the targets are listed in `scripts/build-npm.mjs`).
2. `node scripts/build-npm.mjs <version>` — assembles `dist/npm/*` (launcher +
   per-platform packages, versions matched).
3. Publish: `for d in dist/npm/*/; do (cd "$d" && npm publish --access public); done`
   (dist dir names are flat — `yaks`, `yaks-<os>-<arch>` — so this glob works;
   the scoped names live inside each `package.json`, and `--access public` is
   required for scoped packages.)

Sanity test the launcher without publishing:
- `node npm/yaks/test-mapping.mjs` (host→package mapping + deps consistency)
- Place a host binary at `npm/yaks/node_modules/<host-pkg>/bin/yaks` and run
  `node npm/yaks/bin/yaks.js list`.

## cargo-dist (optional: GitHub Releases + shell/brew installers)

For tag-driven GitHub Releases with shell/powershell/homebrew installers, use
cargo-dist ("dist") as a separate, additive path:
1. `cargo install cargo-dist` (or `dist`).
2. `dist init` — pick targets + installers; this generates/validates
   `dist-workspace.toml` and its own release workflow.
3. Cut a release by pushing a `v<version>` tag.

Not set up yet; run `dist init` if/when you want those installers.

## A release that bumps the farm schema

A schema bump (4 -> 5: comment header blank line, non-image attachment links)
ships with a `yaks migrate` fixup list (`src/migrate.rs`), and every command
but `migrate` refuses a farm on the old schema. A migration commit touches
nearly every yak file, so land it on its own and in this order:

1. Land the code (the new binary reads BOTH shapes, so nothing breaks yet for
   a checkout still on the old schema).
2. In the primary checkout, with the new binary: `yaks migrate --dry-run`
   (per-fixup counts), `yaks migrate`, `yaks doctor`, a second `yaks migrate`
   (it must say `nothing to migrate`), then ONE commit of just `.yaks/`
   (`git add -A .yaks && git commit -m "yaks: migrate the farm to schema <N>"`).
   Quit every running `yaks tui` first.
3. Every other checkout (sheds, Delta clones) rebases or merges that commit,
   or runs `yaks migrate` itself (the fixups are deterministic and
   idempotent, so both routes give the same bytes). Until then the new binary
   refuses it, naming `yaks migrate`; an OLD binary reading a migrated farm
   refuses it as too new (e.g. schema 6 > 5), never misreads it.
4. Open sheds with in-flight work: migrate the shed BEFORE merging it back, or
   the merge carries old-shape comments into a migrated farm (harmless to read,
   fixed by the next `yaks migrate`).

## Version bump

Keep `Cargo.toml`, `npm/yaks/package.json`, and the tag in lockstep.

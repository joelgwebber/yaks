# Contributing

Thanks for helping. YASS tries to stay small. Before adding something, ask whether it belongs in the **README rules** (where things go and what done means), a **playbook** (how an agent does a job), the **CLI** (a view or a file operation that's tedious or error-prone by hand), or **nowhere**: most workflow choices (approvals, merging, CI, verification) belong to each team's repo, not to YASS.

## Layout
```
cmd/yass/                           the CLI's entry point
internal/yass/                      the CLI and the hook check (`yass hook`): Go, two dependencies (yaml.v3, x/mod/semver)
internal/yass/templates/            what `yass init` and `yass new` write, built into the binary
install.sh                          installs the yass binary (yass init and yass upgrade handle repos)
kit/                                what `yass init` and `yass upgrade` put in a repo, embedded in the binary (kit/embed.go)
kit/tools/yass/githooks/pre-commit  the optional hook: a shim that runs `yass hook`
kit/.agents/skills/                 the playbooks (Agent Skills format), unstamped; `yass.yaml` ignores kit/
.goreleaser.yaml                    release builds; .github/workflows/ runs CI and releases
docs/  examples/  tests/
```

## Principles
- **No state outside the files.** Everything the CLI knows comes from folders, checkboxes and, if there is one, `yass.yaml`. Add a setting there only when a team genuinely needs to choose; the defaults should be enough for most.
- **The CLI never commits, branches or merges.** It reads files and moves folders.
- **No runtime dependencies.** One static binary, so any harness can shell out to it. Keep build dependencies rare too.
- **Honest docs.** If something isn't tested against a real harness, say so.

## Tests
```bash
go test ./...                                  # unit tests
go build -o bin/yass ./cmd/yass
YASS_BIN=$PWD/bin/yass tests/e2e.sh            # installs into temp repos and walks the lifecycle, hook included
YASS_BIN=$PWD/bin/yass tests/examples.sh       # every example passes `yass status --strict`
YASS_BIN=$PWD/bin/yass tests/bench.sh          # times status and the hook on generated large repos (~1 min; not in CI)
```
Without `YASS_BIN`, the test scripts build the binary themselves. CI runs all of it on Linux and macOS, then cross-builds every release target with GoReleaser. Please add an e2e check for any new behavior. If a change could slow `yass status` or the hook down, run `tests/bench.sh`: it fails when a speed target is missed.

## Releases
Releases are cut from tags; nothing else publishes.

1. Add a `## X.Y.Z (YYYY-MM-DD)` section to `CHANGELOG.md` on main. It becomes the release notes, and the release fails without it.
2. Tag and push: `git tag vX.Y.Z && git push origin vX.Y.Z`. A tag with a suffix (`v0.2.0-rc.1`) is published as a pre-release.
3. The release workflow runs the tests, builds every platform with GoReleaser, publishes the GitHub Release with the archives and `checksums.txt`, and attests their provenance.

Versions follow semver. Before 1.0, a minor version may change the file format or the CLI; say how to migrate in the changelog. `go install github.com/donjaime/yass/cmd/yass@vX.Y.Z` works from any tag.

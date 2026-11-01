---
id: yaks-de7a
title: 'Linux release binaries require glibc 2.39: a warning on every command on older hosts (cloud runners)'
status: shorn
type: bug
priority: 1
created: '2026-11-01T01:20:45Z'
updated: '2026-11-01T16:13:32Z'
labels:
- release
---

On the memcraft cloud runner (an e2b image), every run of the npm-installed `yaks` 0.0.27 prints:

```text
.../@j15r/yaks-linux-x64/bin/yaks: /lib/x86_64-linux-gnu/libc.so.6: weak version `GLIBC_2.39' not found (required by .../bin/yaks)
```

It still runs, because the symbol is weak, but the agent pipes every yaks command through `grep -v GLIBC` to hide it. That noise also hides real stderr.

Cause: `.github/workflows/release.yml` builds `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu` on `ubuntu-latest` (24.04, glibc 2.39), so the binary asks for the newest glibc.

## Fix options

- Build the Linux targets on an older image (`ubuntu-22.04`, glibc 2.35), or pin a glibc floor with `cargo-zigbuild` (`--target x86_64-unknown-linux-gnu.2.28`).
- Ship `*-unknown-linux-musl` binaries instead: fully static, no glibc dependency. Check that `arboard`'s x11rb and `ratatui-image` build and work on musl.

Verify on a container with an older glibc (e.g. `debian:bookworm`, 2.36): no warning, and `yaks --version` runs.

---
▸ 2026-11-01T15:37:48Z [yakherd]

Confirmed by the memcraft runner yakherd (2026-11-01): every worker brief has to tell workers to pipe yaks through `grep -v GLIBC`. Raising the priority: this costs every agent on such hosts.

---
▸ 2026-11-01T16:08:26Z [yakherd]
kind: move
from: hairy
to: shaving
---

moved: hairy -> shaving

---
▸ 2026-11-01T16:13:31Z [yakherd]

Fixed in `8d9a1b1`: both Linux targets build on `ubuntu-22.04` (glibc 2.35) instead of `ubuntu-latest` (24.04, 2.39).

## Evidence

- **Before:** the highest glibc symbol version in the published 0.0.27 `@j15r/yaks-linux-x64` binary is `GLIBC_2.39`, the weak symbol the runner warned about. The next highest is `GLIBC_2.35`, so the runner's glibc is at least 2.35.
- **After:** dry-run dispatch run `41324791355` built all five targets. The downloaded `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu` artifacts both top out at `GLIBC_2.35` (`strings | grep GLIBC_`).
- **Not yet seen on the runner itself:** that needs a release. Check the next one there with `yaks --version`, which should print no warning.

## Notes

- **Dispatch dry runs and npm:** that dry run's publish step failed with `You cannot publish over the previously published versions: 0.0.27`. Newer npm checks the registry even with `--dry-run`, so a dispatch on an already-published version always ends there. It is harmless, since the log shows the dry-run branch, but it reads like a failure. Worth a line in `RELEASING.md`, or making the dry run skip publish for a published version.
- **Fallback:** if `ubuntu-22.04` images retire, switch to musl targets or `cargo-zigbuild` with a glibc floor (the options in this yak's description).

---
▸ 2026-11-01T16:13:32Z [yakherd]
kind: move
from: shaving
to: shorn
---

moved: shaving -> shorn

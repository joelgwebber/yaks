---
id: yaks-de7a
title: 'Linux release binaries require glibc 2.39: a warning on every command on older hosts (cloud runners)'
status: hairy
type: bug
priority: 1
created: '2026-11-01T01:20:45Z'
updated: '2026-11-01T15:37:48Z'
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

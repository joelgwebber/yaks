---
name: yass-update
description: Update YASS on this machine and in this repo. Check the yass binary's version against the repo's and the latest release, get a verified binary with `yass update` (asking first), then `yass upgrade` the repo and hand back the diff. Use for "update YASS", "upgrade yass", "is yass up to date?", or when `yass status` notes that the binary and the repo's YASS files are out of step.
---

# yass-update

`yass` is the YASS CLI, on your PATH. YASS updates on two levels: the `yass` binary, once per machine (`yass update`), and YASS's files in each repo, the playbooks, the `AGENTS.md` section and the hook (`yass upgrade`, then a commit everyone gets). Keep them apart: the first changes this machine, the second changes the repo for the whole team.

1. **Check both.** `yass update --check` prints the binary's version and how it was built, the repo's YASS version, the latest release, and what `yass update` would install. Tell the human in a line or two. If nothing is behind, say so and stop.
2. **The binary.** It downloads a program and replaces the one on this machine, so ask first, unless you've been given latitude for it. Then run `yass update`.
   - By default it installs the repo's version when the repo is ahead of the binary, else the latest release. `--latest` goes past the repo; do that only if the human also wants to upgrade the repo, which is a team decision.
   - It verifies the download (checksum, and signed provenance when the GitHub CLI is there). If it says provenance wasn't checked, pass that on. If a check fails, stop and report it: never work around it, and never fetch a binary some other way.
   - A binary built from source isn't replaced: `yass update` prints the command for how it was built (`go install …`, or a rebuild in a clone). Show it, and run it only if the human says so.
   - A folder it can't write to: relay what it says (permissions, or installing yass into a folder of your own).
3. **The repo.** If the binary is now ahead of the repo's YASS files and the human wants the repo upgraded: run `yass upgrade` and show what changed (`git status`, a short diff summary).
   - It may also migrate the yass folder's layout and print a commit for that; it goes in its own commit, before the rest.
   - Commit only with the latitude to (`chore: upgrade YASS` for the rest); otherwise give the human the commands. Pushing and opening a pull request need latitude too.
   - Say that teammates need a `yass` at least this new once it lands: `yass update` gets it for them.
4. **Report:** the versions before and after, how the binary was verified, and any commits or commands left for the human.

Never downgrade, and never edit YASS's files by hand to make versions match: `yass update` and `yass upgrade` are the only ways.

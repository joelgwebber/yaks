# Installing YASS

Getting YASS takes two steps, like most developer tools:

1. **[Install](#install)** the `yass` binary and put it on your PATH, once per machine. It's a single Go binary with no runtime dependencies.
2. **[Set up a repo](#set-up-a-repo)** with `yass init`, once per repo, and commit what it writes, so everyone who clones the repo gets it. A repo that [already uses YASS](#joining-a-repo-that-already-uses-yass) needs no setup.

Later, [upgrading](#upgrading) is the same two steps (`yass update`, then `yass upgrade` in each repo), and [`yass deinit`](#removing-yass) takes YASS back out of a repo.

## Install

### With the script

```bash
curl -fsSL https://raw.githubusercontent.com/donjaime/yass/main/install.sh | bash
```

It downloads the latest release for your machine (macOS or Linux), checks it against the release's `checksums.txt`, and copies `yass` into `~/.local/bin`. That's all it does: it doesn't set up a repo or edit your shell files. `--bin-dir <folder>` picks another folder (`… | bash -s -- --bin-dir /usr/local/bin`), and `YASS_VERSION=v0.5.0` a version. The checksum protects against a corrupted download, not against the script itself, so [read it](../install.sh) first, or install from a release below.

### From a release

Each release has an archive per platform (`yass_<os>_<arch>`, with `os` one of `darwin`, `linux`, `windows`, and `arch` one of `amd64`, `arm64`), a `checksums.txt`, and signed build provenance.

```bash
os=darwin arch=arm64   # or linux/amd64, linux/arm64, darwin/amd64
curl -fsSLO "https://github.com/donjaime/yass/releases/latest/download/yass_${os}_${arch}.tar.gz"
curl -fsSLO https://github.com/donjaime/yass/releases/latest/download/checksums.txt
grep "yass_${os}_${arch}.tar.gz" checksums.txt | shasum -a 256 -c    # or sha256sum -c
gh attestation verify "yass_${os}_${arch}.tar.gz" --repo donjaime/yass   # optional; needs the GitHub CLI
tar -xzf "yass_${os}_${arch}.tar.gz"
"yass_${os}_${arch}/install.sh"
```

To pin a version, replace `latest/download` with `download/v0.5.0`. On Windows, download the `.zip` and run `install.sh` from Git Bash, or copy `yass.exe` into a folder on your PATH yourself.

### From source

With Go 1.24 or later (earlier versions build a `yass` with no version, which won't write YASS's files):

```bash
go install github.com/donjaime/yass/cmd/yass@latest
```

That puts it in `$(go env GOPATH)/bin`. Or build a clone, which `install.sh` copies into `~/.local/bin`:

```bash
git clone https://github.com/donjaime/yass && cd yass
git checkout v0.5.0                       # optional: a released version rather than main
go build -o bin/yass ./cmd/yass
./install.sh
```

### Put it on your PATH

You, your agents and the git hook all run `yass` by name, so the folder it's in has to be on your PATH. `install.sh` says whether it is. If it isn't, it prints the line that adds it for your shell; `~/.local/bin` isn't on PATH by default on macOS. For zsh:

```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshenv
```

`~/.zshenv`, not `~/.zshrc`: every zsh reads it, including the non-interactive shells agents and git hooks run. For bash, use `~/.bashrc` (`~/.bash_profile` on macOS); for fish, `fish_add_path ~/.local/bin`. Then open a new terminal and check it:

```bash
yass --version
```

## Set up a repo

From inside the repo:

```bash
yass init
```

It writes the playbooks in `.agents/skills/`, a short section in `AGENTS.md`, an empty `yass/` folder for your changes, and the optional hook in `tools/yass/githooks/`, and lists them. The binary carries these files, so nothing is downloaded. Then commit them, so teammates get them:

```bash
git add -A && git commit -m "chore: adopt YASS"
```

Its options:

| Option | What it does |
|---|---|
| `--claude` | also copies the playbooks to `.claude/skills/` and imports `AGENTS.md` from `CLAUDE.md`, for Claude Code |
| `--hooks` | turns on the optional hook for this clone (`git config core.hooksPath tools/yass/githooks`), unless the repo already has a hooks setup (its own `core.hooksPath`, local or global, or hooks in `.git/hooks/`): then it leaves that alone and prints how to run YASS's check from it ([README](../README.md#already-have-hooks)) |
| `--global` | puts the playbooks in your user folder (`~/.agents/skills/`, or `$YASS_SKILLS_DIR`; `~/.claude/skills/` with `--claude`) for every repo, instead of in this one. The default keeps them in the repo, so everyone working on it gets the same version. Check that your harness reads the user folder you pick |
| `--path P` | keeps the yass folder outside the repo: writes a `yass.yaml` pointing to `P`, creates it, and links `yass/` to it (ignored through `.git/info/exclude`; see [the README](../README.md#keeping-plans-out-of-the-repo)) |
| `<folder> [--agents]` | sets up a folder of a monorepo instead: its own yass folder, and with `--agents` its own `AGENTS.md` section and playbooks, for a part of the codebase that keeps its own agent files. The hook stays at the repo root |

Running `yass init` again writes only what's missing and keeps the rest; to bring YASS's files up to a newer version, use [`yass upgrade`](#upgrading). Each further repo is just `yass init` again.

### Joining a repo that already uses YASS

Its files are committed, so you need only the binary: [install](#install) it and run `yass status` in the repo. If the repo is on a newer YASS than your binary, `yass status` says so, and `yass update` gets that version.

### With your agent

Paste this into your coding agent from inside the repo you want to set up. It does both steps the way a careful person would: verifies the download, reads the script, and asks before running anything.

```text
Install YASS (https://github.com/donjaime/yass) and set it up in this repository. Go step by step,
show me what you find, and don't run anything that changes files until I say so.

Part 1, the binary:
1. Work out this machine's OS (darwin, linux or windows) and CPU (amd64 or arm64).
2. In a temporary folder, download yass_<os>_<arch>.tar.gz (.zip on Windows) and checksums.txt
   from https://github.com/donjaime/yass/releases/latest/download/.
   If I've said "from source" instead: clone https://github.com/donjaime/yass, check out the
   latest release tag, and build it with `go build -o bin/yass ./cmd/yass` (Go 1.24 or later);
   skip step 3.
3. Check the archive's SHA-256 against its line in checksums.txt. If the GitHub CLI is installed,
   also run `gh attestation verify <archive> --repo donjaime/yass`. Stop if either check fails.
4. Unpack it and read install.sh in full. Tell me what it will do: it should only copy the yass
   binary into a folder (~/.local/bin, or the one passed to --bin-dir) and say whether that folder
   is on my PATH. Point out anything that looks unexpected.
5. Ask me where to put the binary, then once I say go, run install.sh (with --bin-dir <folder>
   if I picked another). If the folder isn't on my PATH, show me the line that adds it
   (install.sh prints one); don't edit my shell files.

Part 2, this repo:
6. Ask me which options I want: Claude Code files (--claude), playbooks in my user folder rather
   than this repo (--global), the optional pre-commit hook (--hooks), or keeping the yass folder
   outside this repo (--path <folder>).
7. Run <folder>/yass init with my options, then yass status to check it works, and show me
   git status. Don't commit; I'll review it and commit it myself.
```

## Upgrading

Upgrading is the same two steps as getting YASS: the binary, once per machine, then each repo's files, once per repo.

```bash
yass update                                          # 1. the binary
yass upgrade                                         # 2. in each repo: its YASS files
git add -A && git commit -m "chore: upgrade YASS"    # 3. so teammates get them
```

Read the [release notes](https://github.com/donjaime/yass/releases) first: before 1.0, a minor version can change the file format or the CLI, and the notes say how to migrate. Or ask your agent to "update YASS": the `yass-update` playbook does all three, asking before it downloads.

### 1. The binary: `yass update`

`yass update --check` shows what it would do, changing nothing.

- **Which version:** inside a repo whose YASS files are newer than your binary, exactly that version, so you match your team; otherwise the latest release. When a newer release exists than the one it installs, it says so: `yass update --latest` gets it. `--version vX.Y.Z` picks one. It never downgrades.
- **What it checks:** it refuses unless the download matches the release's `checksums.txt`. With the [GitHub CLI](https://cli.github.com) installed and signed in, it also verifies the release's signed build provenance (`gh attestation verify`), and refuses if that fails; without it, it says provenance wasn't checked. `--require-provenance` refuses to install without that check. The new binary must run and report the version asked for before it replaces the old one, which happens by a rename in the same folder (on Windows, the old one is set aside as `yass.exe.old` and removed on the next run).
- **When it won't:** a binary built with `go install` or from a clone isn't replaced; it prints `go install github.com/donjaime/yass/cmd/yass@<version>`, or how to rebuild the clone. A binary in a folder you can't write to: rerun with permission to write there, or install into a folder of your own. From a release before 0.4.0, which has no `yass update`, [install](#install) again once.

`yass --version` shows what you have.

### 2. Each repo: `yass upgrade`

Run it from anywhere in the repo. It finds every YASS file from the repo root down, with `git ls-files`: playbooks in any folder's `.agents/skills/` or `.claude/skills/` (so monorepo folders set up with `yass init <folder> --agents` are included), every `AGENTS.md` with the YASS section, and the hook. It adds your user folder's playbooks, if you installed them with `--global`, and skips folders `yass.yaml` lists under `ignore:`.

What it does with each file depends on the version stamped in it:
- **Older, or no stamp** (files written by v0.2 or earlier): replaced with this binary's version. A skills folder also gets any playbook it's missing.
- **The same version:** left alone. When nothing needs doing, it says everything is already up to date.
- **Newer:** nothing is written, and it says to run `yass update` first. `yass upgrade` never downgrades.

Your changes, `yass.yaml` and anything outside the `yass:begin`/`yass:end` markers in `AGENTS.md` are left alone. Hand edits to YASS's own files are replaced; since they're committed, the diff shows them. Files outside version control (user-folder playbooks) are labeled as such in the list of what it wrote.

Then it brings each yass folder's layout up to date, including folders `yass.yaml` points to outside the repo and private team folders. Each step does only what's needed, says what it did, and prints how to commit that on its own, apart from the upgrade commit and from code:
- **archive-into-months:** moves changes sitting directly in `archive/` into `archive/<YYYY>/<MM>/`, by when they were archived (the commit that added them there, else the date in their name), and stamps `archived:` on each. Upgrade everyone's `yass` binary before committing it on a shared repo: older binaries don't know the month folders.

### 3. Commit it

Review the diff and commit it (`chore: upgrade YASS`). The binary is per machine and the files are per repo: upgrading the binary affects every repo on your machine, so upgrade each repo you use YASS in. Teammates run `yass update` when the repo moves ahead; `yass status` tells anyone whose binary and repo are out of step, and which to upgrade.

## Removing YASS

From anywhere in the repo:

```bash
yass deinit
```

It removes what `yass init` and `yass upgrade` wrote, from the repo root down, and lists it: every YASS playbook (`yass-*` in any `.agents/skills/` or `.claude/skills/`), the YASS section of every `AGENTS.md` (and the file, when nothing else is in it, with the `@AGENTS.md` import `--claude` added to `CLAUDE.md`), the hook scripts, and `core.hooksPath` when it points at them. It doesn't ask first: it never commits, and everything it removes is in git, so `git status` shows it and `git restore .` undoes it.

Your plans stay: the `yass/` folders and `yass.yaml` are the record of what was changed and why, and `yass deinit` ends by saying how to remove them (`git rm -r yass`) if you'd rather not keep them. It also leaves alone:
- **Your user folder's playbooks,** which serve every repo; `yass deinit --global` removes them too.
- **A `yass hook` line in husky, lefthook or pre-commit's config,** which it points out for you to remove.
- **The binary.** Delete it once you're done with YASS everywhere (`rm "$(command -v yass)"`).

Then commit it: `git add -A && git commit -m "chore: remove YASS"`. `yass init` sets the repo up again whenever you like.

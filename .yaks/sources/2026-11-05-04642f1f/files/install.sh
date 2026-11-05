#!/usr/bin/env bash
# Install the yass binary. Setting up a repo is `yass init`; upgrading one is `yass upgrade`.
#
#   From the web:    curl -fsSL https://raw.githubusercontent.com/donjaime/yass/main/install.sh | bash
#                    (downloads the latest release for this machine and checks it against the release's checksums;
#                    YASS_VERSION=v0.3.0 picks a version)
#   From a release:  tar -xzf yass_<os>_<arch>.tar.gz && yass_<os>_<arch>/install.sh
#   From a clone:    go build -o bin/yass ./cmd/yass && ./install.sh
#
# Options:
#   --bin-dir DIR  copy the yass binary into DIR, which should be on your PATH (default: ~/.local/bin)
#
# Then, in a repo:  yass init [--claude] [--hooks] [--global] [--path P]   (yass init --help)
# Uses the yass binary from $YASS_BIN, next to this script (a release folder), or bin/ (a clone).
set -euo pipefail
REPO_SLUG="${YASS_REPO:-donjaime/yass}"
VERSION="${YASS_VERSION:-latest}"
die() { echo "yass: $*" >&2; exit 1; }

BIN_DIR="$HOME/.local/bin"; BIN_ARG=""; OLD=0; INIT=(); UPGRADE=0; REPO=""
while [ $# -gt 0 ]; do
  case "$1" in
    --bin-dir) BIN_DIR="${2:?--bin-dir needs a folder}"; BIN_ARG=" --bin-dir $2"; shift 2 ;;
    -h|--help) sed -n '2,14p' "$0" 2>/dev/null || true; exit 0 ;;
    # What earlier versions took for setting up a repo: now yass init's and yass upgrade's job.
    --claude|--hooks|--global) OLD=1; INIT+=("$1"); shift ;;
    --path) OLD=1; INIT+=(--path "${2:?--path needs a value}"); shift 2 ;;
    --upgrade) OLD=1; UPGRADE=1; shift ;;
    -*) die "unknown option: $1 (install.sh --help)" ;;
    *) OLD=1; REPO="$1"; shift ;;
  esac
done

# Old-style calls stop before anything is installed, so a script that expects a repo set up fails
# loudly, and says what to run instead.
if [ "$OLD" = 1 ]; then
  {
    echo "install.sh only installs the yass binary now; yass sets up and upgrades repos itself. Run:"
    echo "  install.sh$BIN_ARG"
    cd_part=""; [ -n "$REPO" ] && [ "$REPO" != "." ] && cd_part="cd $REPO && "
    if [ "$UPGRADE" = 1 ]; then echo "  ${cd_part}yass upgrade"
    else echo "  ${cd_part}yass init${INIT[*]+ ${INIT[*]}}"; fi
  } >&2
  exit 2
fi

sha256() { if command -v sha256sum >/dev/null; then sha256sum "$1"; else shasum -a 256 "$1"; fi | awk '{print $1}'; }

# Find the binary: $YASS_BIN, next to this script (a release folder, or a clone's bin/), or download a release.
BIN=""
HERE=""; [ -n "${BASH_SOURCE[0]:-}" ] && [ -f "${BASH_SOURCE[0]}" ] && HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
for c in "${YASS_BIN:-}" ${HERE:+"$HERE/yass" "$HERE/yass.exe" "$HERE/bin/yass" "$HERE/bin/yass.exe"}; do
  if [ -n "$c" ] && [ -f "$c" ] && [ -x "$c" ]; then BIN="$c"; break; fi
done
if [ -z "$BIN" ] && [ -n "$HERE" ]; then
  die "no yass binary next to this script (from a clone: go build -o bin/yass ./cmd/yass, with Go 1.24 or later)"
fi
if [ -z "$BIN" ]; then
  case "$(uname -s)" in Linux) os=linux ;; Darwin) os=darwin ;; *) die "download a release by hand on this OS: https://github.com/$REPO_SLUG/releases" ;; esac
  case "$(uname -m)" in x86_64|amd64) arch=amd64 ;; arm64|aarch64) arch=arm64 ;; *) die "no release for $(uname -m); build from source" ;; esac
  asset="yass_${os}_${arch}.tar.gz"
  if [ "$VERSION" = latest ]; then url="https://github.com/$REPO_SLUG/releases/latest/download"
  else url="https://github.com/$REPO_SLUG/releases/download/$VERSION"; fi
  TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
  echo "downloading $asset ($VERSION) from $REPO_SLUG …"
  curl -fsSL -o "$TMP/$asset" "$url/$asset"
  curl -fsSL -o "$TMP/checksums.txt" "$url/checksums.txt"
  want="$(awk -v f="$asset" '$2 == f {print $1}' "$TMP/checksums.txt")"
  got="$(sha256 "$TMP/$asset")"
  [ -n "$want" ] && [ "$want" = "$got" ] || die "checksum mismatch for $asset (expected ${want:-nothing}, got $got)"
  echo "checksum ok  $got"
  tar -xzf "$TMP/$asset" -C "$TMP"
  BIN="$TMP/yass_${os}_${arch}/yass"
fi

mkdir -p "$BIN_DIR"
BIN_DIR_ABS="$(cd "$BIN_DIR" && pwd)"
if [ "$(cd "$(dirname "$BIN")" && pwd)/$(basename "$BIN")" != "$BIN_DIR_ABS/$(basename "$BIN")" ]; then
  cp "$BIN" "$BIN_DIR_ABS/"
fi
BIN="$BIN_DIR_ABS/$(basename "$BIN")"; chmod +x "$BIN"
echo "installed $("$BIN" --version) to $BIN"

# You, the hook and your agents run `yass` from PATH, so say whether it finds this one, and how to fix it.
ON_PATH="$(command -v yass || true)"
home() { case "$1" in "$HOME"/*) echo "\$HOME${1#"$HOME"}" ;; *) echo "$1" ;; esac; }
if [ -n "$ON_PATH" ] && [ "$(cd "$(dirname "$ON_PATH")" && pwd)" = "$BIN_DIR_ABS" ]; then
  echo "it's on your PATH"
else
  d="$(home "$BIN_DIR_ABS")"
  case "$(basename "${SHELL:-}")" in
    # ~/.zshenv, not ~/.zshrc: agents and git hooks run non-interactive shells, which skip ~/.zshrc.
    zsh)  fix="echo 'export PATH=\"$d:\$PATH\"' >> ~/.zshenv" ;;
    bash) if [ "$(uname -s)" = Darwin ]; then rc="~/.bash_profile"; else rc="~/.bashrc"; fi
          fix="echo 'export PATH=\"$d:\$PATH\"' >> $rc" ;;
    fish) fix="fish_add_path $d" ;;
    *)    fix="echo 'export PATH=\"$d:\$PATH\"' >> ~/.profile" ;;
  esac
  echo
  if [ -z "$ON_PATH" ]; then echo "WARNING: $BIN_DIR_ABS isn't on your PATH, so you, the hook and your agents can't run yass."
  else echo "WARNING: \`yass\` on your PATH is $ON_PATH, not the one just installed in $BIN_DIR_ABS."; fi
  echo "  Fix it with:  $fix"
  echo "  then open a new terminal. Until then, run it as $BIN."
fi

cat <<'EOF'

Next, set up a repo:  cd your-repo && yass init   (yass init --help for --claude, --hooks and the rest)
A repo that already uses YASS needs nothing more. Later, `yass update` gets a newer binary.
EOF

#!/usr/bin/env bash
# End-to-end test: installs YASS into throwaway repos and walks the lifecycle, including the optional hook.
# Usage: tests/e2e.sh [workdir]
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
W="${1:-$(mktemp -d)}"; rm -rf "$W/e2e"; mkdir -p "$W/e2e"
unset YASS_STRICT YASS_HOOK YASS_HOME
# Build the CLI (or use $YASS_BIN) and put it first on PATH, where the hook and install.sh look for it.
if [ -z "${YASS_BIN:-}" ]; then
  mkdir -p "$W/bin"; (cd "$ROOT" && go build -o "$W/bin/yass" ./cmd/yass) || { echo "build failed"; exit 1; }
  YASS_BIN="$W/bin/yass"
fi
export YASS_BIN; export PATH="$(dirname "$YASS_BIN"):$PATH"
pass=0; fail=0
ok()  { echo "  ok   $1"; pass=$((pass+1)); }
bad() { echo "  FAIL $1"; fail=$((fail+1)); }
run_ok()   { local d="$1"; shift; local o; if o="$("$@" 2>&1)"; then ok "$d"; else bad "$d"; sed 's/^/       /' <<<"$o" | head -20; fi; }
run_fail() { local d="$1"; shift; local o; if o="$("$@" 2>&1)"; then bad "$d (should have failed)"; sed 's/^/       /' <<<"$o" | head -20; else ok "$d"; fi; }
has()   { local d="$1" p="$2"; shift 2; local o; o="$("$@" 2>&1)"; if grep -qE -- "$p" <<<"$o"; then ok "$d"; else bad "$d (no /$p/)"; sed 's/^/       /' <<<"$o" | head -25; fi; }
hasnt() { local d="$1" p="$2"; shift 2; local o; o="$("$@" 2>&1)"; if grep -qE -- "$p" <<<"$o"; then bad "$d (found /$p/)"; sed 's/^/       /' <<<"$o" | head -25; else ok "$d"; fi; }
y() { yass "$@"; }
# portable in-place regex edit (GNU and BSD): sub FILE PATTERN REPLACEMENT [COUNT]
sub() { P="$2" R="$3" N="${4:-0}" perl -0777 -i -pe '
  BEGIN { ($p, $r, $n) = ($ENV{P}, $ENV{R}, $ENV{N}); $r =~ s/\\n/\n/g }
  if ($n) { my $c = 0; s/$p/$c++ < $n ? $r : $&/gme } else { s/$p/$r/gm }' "$1"; }
newrepo() { rm -rf "$1"; mkdir -p "$1"; cd "$1"; git init -q -b main; git config user.email t@t; git config user.name Sam
  echo app > app.txt; git add -A; git commit -q -m "existing app"; }
TODAY=$(date +%F); MONTH=$(date -u +%Y/%m)   # archives go in archive/<YYYY>/<MM>/, by UTC month

newrepo "$W/e2e/solo"
INSTALL_OUT="$(y init)"

echo "1. install"
has   "a first install suggests the adopting commit" "chore: adopt YASS" echo "$INSTALL_OUT"
for f in tools/yass/githooks/pre-commit yass/README.md yass/changes/.gitkeep \
         yass/archive/.gitkeep AGENTS.md .agents/skills/yass-work/SKILL.md .agents/skills/yass-shape/SKILL.md \
         .agents/skills/yass-plan/SKILL.md .agents/skills/yass-status/SKILL.md .agents/skills/yass-log/SKILL.md \
         .agents/skills/yass-update/SKILL.md; do
  [ -e "$f" ] && ok "has $f" || bad "missing $f"
done
has "the yass folder README describes the monthly archive" "archive/.*<YYYY>/<MM>/" cat yass/README.md
[ ! -e .claude ] && ok "no Claude files without --claude" || bad ".claude written without --claude"
[ ! -e yass/config.yml ] && ok "no config file" || bad "a config file appeared"
has   "AGENTS.md has the YASS section" "yass:begin" cat AGENTS.md
hasnt "the hook is off by default" "githooks" git config core.hooksPath
has   "status on an empty repo" "no active changes" y
has   "init is idempotent" "already set up" y init
git add -A; run_ok "adopting commit" git commit -q -m "chore: adopt YASS"
echo "# My rules" > AGENTS.md; y init >/dev/null
has   "init keeps your AGENTS.md content" "# My rules" cat AGENTS.md
has   "…and adds its section once" "^1$" bash -c 'grep -c "yass:begin" AGENTS.md'
y init >/dev/null
has   "…even when run again" "^1$" bash -c 'grep -c "yass:begin" AGENTS.md'
git checkout -q AGENTS.md

echo "2. a small change"
P=$(y new "Fix double-tap save" --source gh#41 --platforms "ios, android" --goal "Tapping Save twice saves once.")
has "dated folder" "^yass/changes/$TODAY-fix-double-tap-save$" echo "$P"
has "source recorded" "^source: gh#41" cat "$P/change.md"
has "platforms recorded" "^platforms: \[ios, android\]" cat "$P/change.md"
has "goal written" "Tapping Save twice saves once." cat "$P/change.md"
has "title heading" "^# Fix double-tap save" cat "$P/change.md"
P2=$(y new "Fix double-tap save")
has "same title same day gets a suffix" "fix-double-tap-save-2$" echo "$P2"
rm -rf "$P2"
sub "$P/change.md" '^## Acceptance\n' '## Acceptance\n- [ ] Given Save tapped twice, when saving, then one entry — verify: unit SaveTests\n'
has   "a fresh change is not started" "fix-double-tap-save +0/1 +not started" y status
has   "…in the detail view too" "progress: 0/1  \(not started\)" y status double-tap
sub "$P/change.md" '^- \[ \] $' '- [x] Reproduce\n- [ ] Disable Save while saving'
hasnt "a ticked box starts it" "not started" y status
has "status shows progress and next" "fix-double-tap-save +1/3 +next: Given Save tapped twice" y status
printf '\n### %s (claude)\n- Did: reproduced\n- Next: disable the button\n' "$TODAY" >> "$P/change.md"
has "next comes from the Log when there is one" "next: disable the button" y status
has "detail view lists open boxes" "\[ \] Disable Save while saving" y status double-tap
has "detail view shows source" "source: gh#41" y status double-tap
git add -A; git commit -q -m "yass: new change"
sub "$P/change.md" '^- \[ \] Disable' '- [/] Disable'
has      "an in-progress box isn't done" "fix-double-tap-save +1/3" y status
has      "…and the detail view marks it" "\[/\] Disable Save while saving" y status double-tap
run_fail "archive refuses with open boxes" y archive double-tap
has      "…and says which, in progress included" "2 box\(es\) still open" y archive double-tap
sub "$P/change.md" '^- \[ \] Given' '- [x] Given'; sub "$P/change.md" '^- \[/\] Disable' '- [-] Disable'
has      "a dropped box doesn't count" "fix-double-tap-save +2/2" y status
has      "every box done or dropped: done" "fix-double-tap-save +2/2 +done" y status
has      "…in the detail view too" "progress: 2/2  \(done\)" y status double-tap
git commit -qam "progress"
run_ok   "archive once every box is done or dropped" y archive double-tap
[ -d "yass/archive/$MONTH/$TODAY-fix-double-tap-save" ] && ok "moved to the archive" || bad "not in the archive"
[ ! -e "$P" ] && ok "gone from changes" || bad "still in changes"
has      "the move is staged" "R.*fix-double-tap-save/change.md" git status --porcelain
has      "archive stamps archived: in UTC" "^archived: [0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9:]{8}Z$" cat "yass/archive/$MONTH/$TODAY-fix-double-tap-save/change.md"
has      "…and changes nothing else" "^\+archived: [^ ]+Z\|$" bash -c 'git diff --cached -M | grep -E "^[-+][^-+]" | paste -sd"|" - | sed "s/$/|/"'
hasnt    "…and leaves nothing unstaged" "." git diff --name-only
has      "archived listing" "$TODAY-fix-double-tap-save" y status --archived
has      "an archived change in detail" "progress: 2/2  \(done\)" y status --archived double-tap
run_fail "…but not without --archived" y status double-tap
git commit -qm "yass: archive fix-double-tap-save"
OLD=yass/changes/2025-01-05-old-thing; mkdir -p "$OLD"; printf '# Old thing\n\n## Steps\n- [x] Done\n' > "$OLD/change.md"
git add -A; git commit -qm "yass: old thing"
run_ok   "a change from an earlier month…" y archive old-thing
[ -d "yass/archive/$MONTH/2025-01-05-old-thing" ] && ok "…goes in the month it was archived" || bad "not in this month's folder"
has      "a change without frontmatter gets some" "^---$" head -1 "yass/archive/$MONTH/2025-01-05-old-thing/change.md"
git commit -qm "yass: archive old-thing"
DUP="yass/changes/$TODAY-fix-double-tap-save"; mkdir -p "$DUP"; printf '# Again\n\n## Steps\n- [x] Done\n' > "$DUP/change.md"
run_fail "archive refuses a name that's already archived" y archive "$DUP"
has      "…and says where the other one is" "already archived, at yass/archive/$MONTH/$TODAY-fix-double-tap-save" y archive "$DUP"
rm -rf "$DUP"

echo "3. a large change with pieces"
L=$(y new "Offline sync" --large --design --goal "Never lose an entry.")
for f in change.md prd.md plan.md design.md; do [ -f "$L/$f" ] && ok "large has $f" || bad "large lacks $f"; done
hasnt "large head has no Steps" "^## Steps" cat "$L/change.md"
Q=$(y new "Offline queue" --in offline-sync)
B=$(y new "Sync badge" --in offline)
has "pieces live inside" "^$L/$TODAY-offline-queue$" echo "$Q"
run_fail "pieces nest one level deep" y new "Too deep" --in offline-queue
run_fail "a piece can't be large" y new "Big piece" --large --in offline-sync
has   "template prints one" "^## Options" y template design
run_fail "…and refuses an unknown one" y template nope
has "status nests pieces" "├ $TODAY-offline-queue" y status
has "large is labelled" "offline-sync  \(large\)" y status
printf '### M1\n- [x] AC1 (R1) Given no network — verify: e2e\n- [ ] AC2 (R2) Given a kill — verify: e2e\n' >> "$L/plan.md"
sub "$Q/change.md" '^- \[ \] $' '- [x] Queue table\n- [ ] Worker'
sub "$B/change.md" '^- \[ \] $' '- [ ] Badge'
has "large progress rolls up its pieces" "offline-sync  \(large\) +2/5" y status
has "pieces show their own" "offline-queue +1/2" y status
has   "a piece with no work is not started" "sync-badge +0/1 +not started" y status
hasnt "…but its change is, once any piece is" "offline-sync  \(large\) .*not started" y status
sub "$B/change.md" '^- \[ \] Badge' '- [/] Badge'
hasnt "an in-progress box starts it" "sync-badge .*not started" y status
has   "…and becomes next when the Log has none" "sync-badge .*next: Badge" y status
run_fail "a piece can't be archived alone" y archive offline-queue
has      "…it goes with its change" "archived with their change" y archive offline-queue
run_fail "archive refuses open boxes in pieces" y archive offline-sync
has      "…and names the piece" "Worker  \($TODAY-offline-queue/change.md\)" y archive offline-sync
git add -A; git commit -q -m "yass: offline sync"
run_ok   "--force archives anyway" y archive offline-sync --force
[ -f "yass/archive/$MONTH/$TODAY-offline-sync/$TODAY-sync-badge/change.md" ] && ok "pieces moved along" || bad "pieces left behind"
git commit -qm "yass: archive offline-sync"

echo "4. blocked, follows, warnings"
F=$(y new "Offline sync, web badge" --follows "$TODAY-offline-sync")
hasnt "follows an archived change: no warning" "warning" y status
sub "$F/change.md" '^blocked:$' 'blocked: waiting on design review'
sub "$F/change.md" '^- \[ \] $' '- [x] Spike'
has "blocked shows in status" "BLOCKED: waiting on design review" y status
run_fail "archive refuses a blocked change" y archive web-badge
G=$(y new "Something else" --follows 2020-01-01-nope)
has "follows something missing: warning" "warning: .*follows '2020-01-01-nope'" y status
run_fail "--strict fails on warnings" y status --strict
rm -rf "$G"
mkdir -p yass/changes/random-folder && echo "x" > yass/changes/random-folder/notes.md
has "a folder with no change.md" "random-folder: no change.md" y status
has "an undated folder name" "random-folder: name it" y status
rm -rf yass/changes/random-folder
mkdir -p "$Q" 2>/dev/null; mkdir -p "yass/changes/$TODAY-x/$TODAY-y/$TODAY-z"
for d in "$TODAY-x" "$TODAY-x/$TODAY-y" "$TODAY-x/$TODAY-y/$TODAY-z"; do printf -- '---\n---\n# t\n' > "yass/changes/$d/change.md"; done
has "nesting deeper than one level" "pieces nest one level deep" y status
rm -rf "yass/changes/$TODAY-x" "$Q"; rmdir "$L" 2>/dev/null || true
run_fail "unknown change" y status nope-nothing
E=$(y new "Empty one")
run_fail "archive refuses a change with no boxes" y archive empty-one
sub "$E/change.md" '^blocked:$' 'blocked:    # set when stuck'
sub "$E/change.md" '^follows:.*$' "follows: $TODAY-sync-badge   # an archived piece"
printf '### %s (claude)\n- **Next:** bold next step\n' "$TODAY" >> "$E/change.md"
hasnt "frontmatter comments are ignored" "empty-one .*BLOCKED|^warning" y status
has   "follows can name an archived piece" "empty-one" y status
has   "a bold **Next:** is read" "next: bold next step" y status
rm -rf "$E"
y new "Web cache" >/dev/null; y new "Web fonts" >/dev/null
has "ambiguous name" "matches [0-9]+ changes" y status web
rm -rf yass/changes/*web-cache yass/changes/*web-fonts
git add -A; git commit -qm "yass: follow-up"

echo "5. monorepo: several yass/ folders, no config"
mkdir -p services/payments && echo "# Payments" > services/payments/README.md
run_ok "init a team folder" y init services/payments
[ -d services/payments/yass/changes ] && ok "team yass/ created" || bad "team yass/ missing"
has    "AGENTS.md section not duplicated" "^1$" bash -c 'grep -c "yass:begin" AGENTS.md'
has    "new goes to the nearest yass/" "^services/payments/yass/changes/$TODAY-saved-cards-api$" bash -c 'cd services/payments && yass new "Saved cards API"'
has    "…and from the repo root, to the root yass/" "^yass/changes/$TODAY-three-tap-checkout$" y new "Three-tap checkout"
has    "status shows every yass/ folder" "services/payments/yass/" y status
has    "…root first" "^yass/$" bash -c 'yass status | head -1'
has    "detail by name across folders" "Saved cards API" y status saved-cards
git add -A; git commit -qm "monorepo"

echo "6. the optional hook"
git config core.hooksPath tools/yass/githooks
H=$(y new "Hook demo" --large)
printf '### M1\n- [ ] AC1 (R1) Given x, when y, then z — verify: unit\n' >> "$H/plan.md"
sub "$H/change.md" '^## Goal\n' '## Goal\nDemo the hook.\n'
git add -A; run_ok "intent-only commit" git commit -q -m "yass: plan hook demo"
hasnt "…without a warning" "yass:" bash -c "echo x >> README2.md; git add -A; git commit -q -m readme 2>&1"
echo "code1" >> app.txt; sub "$H/plan.md" '^- \[ \] AC1' '- [x] AC1'
printf '\n## Log\n### %s (claude)\n- Did: x\n- Next: y\n' "$TODAY" >> "$H/change.md"
git add -A
hasnt "ticking a plan box + Log + code: silent" "heads-up" git commit -q -m "feat: thing"
echo "code2" >> app.txt; echo "- **R9** new" >> "$H/prd.md"; git add -A
has   "prd edit + code: warns" "prd.md is intent" git commit -q -m "feat: sneaky"
has   "…but the commit went through" "feat: sneaky" git log -1 --format=%s
echo "code3" >> app.txt; echo "more" >> "$H/prd.md"; git add -A
run_fail "YASS_STRICT=1 refuses it" env YASS_STRICT=1 git commit -q -m "feat: sneaky 2"
run_ok   "YASS_HOOK=off skips it" env YASS_HOOK=off git commit -q -m "feat: skip"
echo "code4" >> app.txt; sub "$H/plan.md" 'when y, then z' 'when y, then maybe z'; git add -A
has   "rewording an acceptance criterion + code: warns" "only mark boxes in plan.md" git commit -q -m "feat: reword"
echo "code5" >> app.txt; sub "$H/change.md" 'Demo the hook.' 'Demo the hook, roughly.'; git add -A
has   "editing a Goal + code: warns" "title, Goal or Acceptance changed" git commit -q -m "feat: goal"
S=$(y new "Small one"); git add -A; git commit -q -m "yass: small one"
echo "code6" >> app.txt; sub "$S/change.md" '^- \[ \] $' '- [x] Step one\n- [ ] Step two'
sub "$S/change.md" '^## Decisions\n' '## Decisions\n- Do it simply - because (claude)\n'; git add -A
hasnt "adding Steps and Decisions + code: silent" "heads-up" git commit -q -m "feat: small"
echo "code7" >> app.txt; N=$(y new "Brand new"); git add -A
hasnt "a new change + code: silent" "heads-up" git commit -q -m "feat: with new change"
git add -A; git commit -q -m "progress" >/dev/null 2>&1 || true
y archive hook-demo --force >/dev/null; echo "code8" >> app.txt; git add -A
has   "archiving + code: warns" "archive a change in its own commit" git commit -q -m "feat: archive and code"
y archive brand-new --force >/dev/null; git add -A
hasnt "archiving alone: silent" "heads-up" git commit -q -m "yass: archive brand-new"
echo "x" > "yass/archive/$MONTH/$TODAY-fix-double-tap-save/notes.md"; git add -A
has   "adding a file to an archived change warns" "the archive is append-only" git commit -q -m "yass: add notes"
echo "edit" >> "yass/archive/$MONTH/$TODAY-fix-double-tap-save/change.md"; git add -A
has   "editing the archive warns, even alone" "the archive is append-only" git commit -q -m "yass: fix typo"
BASE=$(git rev-list --max-parents=0 HEAD)
has      "CI range mode finds the mixed commits" "feat: sneaky: .*prd.md is intent" yass hook --range "$BASE..HEAD"
run_fail "CI range mode with --strict fails" yass hook --range "$BASE..HEAD" --strict
D=$(y new "Drop demo"); sub "$D/change.md" '^## Acceptance\n' '## Acceptance\n- [ ] Given a, when b, then c — verify: unit\n'
git add -A; git commit -q -m "yass: drop demo"
echo "code9a" >> app.txt; sub "$D/change.md" '^- \[ \] Given a' '- [/] Given a'; git add -A
hasnt "marking a criterion in progress + code: silent" "heads-up" git commit -q -m "feat: start it"
echo "code9" >> app.txt; sub "$D/change.md" '^- \[/\] Given a' '- [-] Given a'; git add -A
has   "dropping an acceptance criterion + code: warns" "title, Goal or Acceptance changed" git commit -q -m "feat: drop it"
run_fail "a range CI can't read fails loudly" yass hook --range origin/nope..HEAD --strict
CLEAN=$(git rev-parse HEAD~1)   # "yass: drop demo": a new change, no code
run_ok   "a clean range passes" yass hook --range "$CLEAN~1..$CLEAN" --strict
git config --unset core.hooksPath

echo "7. edge cases"
echo "build/" > .gitignore; mkdir -p build/x
has "init warns when the folder is gitignored" "ignored by .gitignore" y init build/x --no-agents
rm -rf build .gitignore
mkdir -p infra/target && y init infra/target --no-agents >/dev/null
has "folders git doesn't ignore are found, whatever their name" "^infra/target/yass/changes/" bash -c 'cd infra/target && yass new "Infra thing"'
rm -rf infra
NG="$W/e2e/nogit"; rm -rf "$NG"; mkdir -p "$NG/sub/deeper"
(cd "$NG" && yass init --no-agents >/dev/null && yass new "Outside git" >/dev/null)
has "outside git, status from a subfolder finds yass/" "outside-git" bash -c "cd '$NG/sub/deeper' && yass status"
cd "$W/e2e/solo"
R=$(y new "Windows endings")
printf -- '---\r\nblocked: waiting on QA\r\n---\r\n# Windows endings\r\n\r\n## Steps\r\n- [x] a\r\n- [ ] b\r\n\r\n## Log\r\n### 2026-10-03 (c)\r\n- Next: from the log\r\n' > "$R/change.md"
has   "CRLF files: frontmatter is read" "windows-endings .*BLOCKED: waiting on QA" y status
sub "$R/change.md" '^blocked: waiting on QA' 'blocked:'
has   "…and the Log" "windows-endings .*next: from the log" y status
rm -rf "$R"
N=$(y new "With notes" --source "gh issue #41"); sub "$N/change.md" '^- \[ \] $' '- [x] The work'
mkdir -p "$N/notes" && printf -- '- [ ] an idea for later\n' > "$N/notes/ideas.md"
has      "a # inside a value isn't a comment" "source: gh issue #41" y status with-notes
has      "boxes in a subfolder that isn't a piece don't count" "with-notes .*1/1  done" y status
run_ok   "…for archive either" y archive with-notes
rm -rf "yass/archive/$MONTH/$TODAY-with-notes"; git add -A >/dev/null
A=$(y new "All dropped"); sub "$A/change.md" '^- \[ \] $' '- [-] Not needed after all'
has      "every box dropped: done" "all-dropped .*done" y status
run_ok   "…and it can be archived" y archive all-dropped
rm -rf "yass/archive/$MONTH/$TODAY-all-dropped"; git add -A >/dev/null
mkdir -p "apps/café"; y init "apps/café" --no-agents >/dev/null
has   "non-ASCII folder names: new goes to the nearest yass/" "^apps/café/yass/changes/$TODAY-accented$" bash -c 'cd "apps/café" && yass new "Accented" --large'
has   "…and status lists it" "^apps/café/yass/$" y status
git add -A; git commit -qm "yass: accented" >/dev/null
echo "code" >> app.txt; echo "- **R9** more" >> "apps/café/yass/changes/$TODAY-accented/prd.md"; git add -A
has   "…and the hook sees intent edits there" "café/yass/changes/$TODAY-accented/prd.md: prd.md is intent" yass hook
git commit -qm "feat: mixed" >/dev/null 2>&1
rm -rf "apps"; git add -A; git commit -qm "chore: tidy" >/dev/null

echo "8. install.sh installs the binary"
newrepo "$W/e2e/claude"
y init --claude --hooks >/dev/null
[ -f .claude/skills/yass-work/SKILL.md ] && ok "--claude copies the playbooks" || bad "no .claude/skills"
has "--claude imports AGENTS.md" "^@AGENTS.md" cat CLAUDE.md
has "--hooks sets the hook path" "tools/yass/githooks" git config core.hooksPath
git add -A; hasnt "the adopting commit isn't flagged" "heads-up" git commit -q -m "chore: adopt YASS"
BD="$W/e2e/bindir"; rm -rf "$BD" "$W/e2e/bindir2" "$W/e2e/bindir3" "$W/e2e/bindir4" "$W/e2e/onebin" "$W/e2e/nobin"
IOUT="$(bash "$ROOT/install.sh" --bin-dir "$BD")"
has   "--bin-dir copies the binary" "installed yass .* to .*/bindir/yass" echo "$IOUT"
[ -x "$BD/yass" ] && ok "…and it runs" || bad "no binary in --bin-dir"
[ -z "$(git status --porcelain)" ] && ok "…and, run in a repo that uses YASS, changes nothing there (joining a repo)" || bad "install.sh changed the repo: $(git status --porcelain)"
has   "…and says yass init sets up a repo" "set up a repo: .*yass init" echo "$IOUT"
mkdir -p "$W/e2e/plain"; rm -rf "$W/e2e/plain/"*
run_ok "…run outside a repo too" bash -c 'cd "$1" && bash "$2" --bin-dir "$3"' _ "$W/e2e/plain" "$ROOT/install.sh" "$W/e2e/bindir3"
[ -z "$(ls -A "$W/e2e/plain")" ] && ok "…writing nothing where it's run" || bad "install.sh wrote into the folder it ran in"
has   "…and warns that another yass on PATH comes first" "is .*, not the one just installed" bash "$ROOT/install.sh" --bin-dir "$BD"
has   "…or that --bin-dir isn't on PATH" "bindir2 isn't on your PATH" \
      env PATH=/usr/bin:/bin SHELL=/bin/zsh bash "$ROOT/install.sh" --bin-dir "$W/e2e/bindir2"
has   "…with the fix for the shell" "Fix it with:  echo 'export PATH=\".*/bindir2:\\\$PATH\"' >> ~/.zshenv" \
      env PATH=/usr/bin:/bin SHELL=/bin/zsh bash "$ROOT/install.sh" --bin-dir "$W/e2e/bindir2"
hasnt "no warning when the installed yass is the one on PATH" "WARNING" env PATH="$BD:/usr/bin:/bin" bash "$ROOT/install.sh" --bin-dir "$BD"
has   "…and it says so" "it's on your PATH" env PATH="$BD:/usr/bin:/bin" bash "$ROOT/install.sh" --bin-dir "$BD"
run_fail "an old-style install (a repo and setup options) refuses" bash "$ROOT/install.sh" . --claude --bin-dir "$W/e2e/nobin"
[ ! -e "$W/e2e/nobin" ] && ok "…before installing anything" || bad "an old-style install installed the binary"
has   "…printing the commands to use, options carried over" "^  yass init --claude$" bash "$ROOT/install.sh" . --claude --bin-dir "$W/e2e/nobin"
has   "…a repo path becoming a cd" "^  cd apps/web && yass init --hooks --path \.\./p$" bash "$ROOT/install.sh" apps/web --hooks --path ../p
has   "…and --upgrade pointing at yass upgrade" "^  yass upgrade$" bash "$ROOT/install.sh" --upgrade
has   "…after the binary install" "^  install.sh$" bash "$ROOT/install.sh" --global
has   "…keeping a --bin-dir given" "^  install.sh --bin-dir /opt/bin$" bash "$ROOT/install.sh" --global --bin-dir /opt/bin
H="$W/e2e/home"; rm -rf "$H"; mkdir -p "$H"
has   "without --bin-dir it installs to ~/.local/bin" "installed yass .* to $H/.local/bin/yass" env HOME="$H" bash "$ROOT/install.sh"
rm -rf "$H/.local"
has   "…piped too, like the one-liner" "installed yass .* to $H/.local/bin/yass" \
      bash -c 'env HOME="$2" PATH=/usr/bin:/bin bash -s < "$1"' _ "$ROOT/install.sh" "$H"
U="$W/e2e/unpacked"; rm -rf "$U"; mkdir -p "$U"; cp "$ROOT/install.sh" "$U/"; cp "$YASS_BIN" "$U/yass"
has   "an unpacked release with no kit/ installs its binary" "installed yass .* to .*/bindir4/yass" \
      env -u YASS_BIN PATH=/usr/bin:/bin bash "$U/install.sh" --bin-dir "$W/e2e/bindir4"
K="$W/e2e/script-only"; rm -rf "$K"; mkdir -p "$K"; cp "$ROOT/install.sh" "$K/"
run_fail "no binary next to the script: refuses" env -u YASS_BIN PATH=/usr/bin:/bin bash "$K/install.sh" --bin-dir "$W/e2e/nobin"
has      "…and says how to get one" "no yass binary next to this script" env -u YASS_BIN PATH=/usr/bin:/bin bash "$K/install.sh" --bin-dir "$W/e2e/nobin"
newrepo "$W/e2e/oneliner"
run_ok "the one-liner's two commands set up a repo, with --bin-dir not on PATH" \
       env PATH=/usr/bin:/bin bash -c 'bash "$1" --bin-dir "$2" >/dev/null && "$2/yass" init >/dev/null' _ "$ROOT/install.sh" "$W/e2e/onebin"
[ -f .agents/skills/yass-work/SKILL.md ] && [ -x tools/yass/githooks/pre-commit ] && ok "…playbooks, hook and all" || bad "the one-liner didn't set up the repo"
newrepo "$W/e2e/bare"
run_fail "new before init explains itself" y new "x"
has      "…with the fix" "run .*yass init" y new "x"

echo "9. yass.yaml: the yass folder lives somewhere else"
newrepo "$W/e2e/ext"; rm -rf "$W/e2e/ext-yass" "$W/e2e/home-yass"
y init --path ../ext-yass >/dev/null
[ -f yass.yaml ] && ok "--path writes yass.yaml" || bad "no yass.yaml"
[ -L yass ] && [ -z "$(git status --porcelain -- yass)" ] && ok "…and yass/ in the repo is only an ignored link" || bad "yass/ isn't an ignored link"
[ -d "$W/e2e/ext-yass/changes" ] && ok "…and creates the folder it points to" || bad "no external folder"
has   "root prints it" "ext-yass$" y root
X=$(y new "Private thing")
has   "new puts changes there" "^/.*ext-yass/changes/$TODAY-private-thing$" echo "$X"
sub "$X/change.md" '^- \[ \] $' '- [x] Did it'
has   "status reads them" "private-thing +1/1 +done" y status
git add -A; git commit -q -m "chore: adopt YASS"
hasnt "none of it lands in the repo" "private-thing" git log --stat --format=
run_ok "archive works outside git" y archive private-thing
[ -d "$W/e2e/ext-yass/archive/$MONTH/$TODAY-private-thing" ] && ok "…and moves the folder" || bad "not archived"
MAIN_SHA=$(git rev-parse --short HEAD)
git switch -q -c feature; echo wip >> app.txt; git commit -qam "feat: wip"; FEAT_SHA=$(git rev-parse --short HEAD); git switch -q main
C=$(y new "Cited")
sub "$C/change.md" '^- \[ \] $' "- [x] Shipped — code: $MAIN_SHA\n- [/] Building — code: $FEAT_SHA"
hasnt "citing a merged commit when done, a branch commit in progress: fine" "warning" y status
sub "$C/change.md" '^- \[/\] Building' '- [x] Building'
has   "done, citing a commit that isn't merged: warns" "'Building' is marked done, but $FEAT_SHA isn't on main yet" y status
has   "…in the detail view too" "isn't on main yet" y status cited
printf "path: ../ext-yass\nbranch: feature\n" > yass.yaml
hasnt "branch: in yass.yaml sets what counts as merged" "isn't on" y status
printf "path: ../ext-yass\nbranch: nope\n" > yass.yaml
has   "…and warns when it doesn't exist" "branch 'nope' isn't in this repo" y status
printf "path: ../ext-yass\n" > yass.yaml
sub "$C/change.md" "code: $MAIN_SHA" 'code: deadbee'
has   "citing a commit that doesn't exist: warns" "cites deadbee, which isn't a commit here" y status
rm -rf "$C"
printf 'path: ${YASS_HOME}/proj\n' > yass.yaml
has   "an unset variable warns" "YASS_HOME isn't set" y status
run_fail "…and new refuses" y new "Nope"
run_ok "init creates the folder under the variable" env YASS_HOME="$W/e2e/home-yass" yass init --no-agents
has   "…and root resolves it" "home-yass/proj$" env YASS_HOME="$W/e2e/home-yass" yass root
printf 'path: proj\ncolor: blue\n' > yass.yaml; mkdir -p proj/changes
has   "unknown settings warn" "unknown setting 'color'" y status
rm -rf proj; printf 'path: planning\n' > yass.yaml; y init --no-agents >/dev/null
P=$(y new "Planned here" --large)
has   "a relative path is relative to yass.yaml" "^planning/changes/$TODAY-planned-here$" echo "$P"
git add -A; git commit -q -m "yass: shape planned here"
git config core.hooksPath tools/yass/githooks
echo code >> app.txt; echo "- **R9** more" >> "$P/prd.md"; git add -A
has   "the hook finds a yass folder through yass.yaml" "prd.md is intent" git commit -q -m "feat: sneaky"
echo code >> app.txt; git add -A
hasnt "…and treats yass.yaml as YASS, not code" "heads-up" git commit -q -m "feat: plain code"
git config --unset core.hooksPath
has   "a link left from before the plans moved into the repo warns" "yass/ is a link to .*, but yass.yaml points to planning; remove the link" y status
rm yass; mkdir -p yass/changes && touch yass/changes/.gitkeep
has   "a yass/ beside a yass.yaml that points elsewhere warns" "yass/: ignored, because yass.yaml points to" y status
rm -rf yass

echo "10. yass.yaml beside yass/: settings only, and ignore"
newrepo "$W/e2e/ign"
y init >/dev/null
mkdir -p examples/a/yass/archive/2026-01-01-old examples/b/svc/yass/changes/2026-01-02-thing vendor/x/yass/changes
printf '# Old\n\n## Goal\nOld.\n\n## Steps\n- [x] Done\n' > examples/a/yass/archive/2026-01-01-old/change.md
printf '# Thing\n\n## Goal\nA thing.\n\n## Steps\n- [ ] Do it\n' > examples/b/svc/yass/changes/2026-01-02-thing/change.md
touch vendor/x/yass/changes/.gitkeep
git add -A; git commit -q -m "chore: adopt YASS, with examples"
has   "without ignore, nested yass folders count" "examples/b/svc/yass" y root
printf 'ignore:\n  - examples/*\n  - vendor\n' > yass.yaml
has   "a yass.yaml without path: keeps yass/ next to it" "^$W/e2e/ign/yass$|/ign/yass$" y root
hasnt "…and ignored folders aren't roots" "examples|vendor" y root
hasnt "…or in status" "thing|examples" y status
hasnt "…and nothing warns" "warning|ignored, because" y status
X=$(y new "Top thing")
has   "new still goes to yass/" "^yass/changes/$TODAY-top-thing$" echo "$X"
has   "inside an ignored folder, it's a project of its own" "thing" bash -c 'cd examples/b && yass status'
hasnt "…without the outer yass/" "top-thing" bash -c 'cd examples/b && yass status'
has   "…and new goes to its nearest yass/" "^svc/yass/changes/$TODAY-inner$" bash -c 'cd examples/b/svc && yass new "Inner"'
rm -rf examples/b/svc/yass/changes/$TODAY-inner
git add -A; git commit -q -m "yass: top thing"
git config core.hooksPath tools/yass/githooks
echo code >> app.txt; echo "- [x] Rewritten" >> examples/a/yass/archive/2026-01-01-old/change.md
sub examples/b/svc/yass/changes/2026-01-02-thing/change.md '^A thing\.$' 'A different thing.'
git add -A
mv yass.yaml "$W/e2e/ign.yaml"
has   "without ignore, the hook flags those edits" "append-only" y hook
mv "$W/e2e/ign.yaml" yass.yaml
hasnt "the hook treats ignored folders as ordinary files" "heads-up" git commit -q -m "feat: code and example edits"
git config --unset core.hooksPath
printf 'ignore: [../elsewhere, ".", nope/*, missing]\n' > yass.yaml
has   "an ignore outside the folder warns" "'../elsewhere' isn't inside" y status
has   "…so does the folder itself" "'\.' is the folder yass.yaml is in" y status
has   "…and one that matches nothing" "'missing' doesn't match a folder" y status

echo "11. queue.md: the order to tackle changes in"
newrepo "$W/e2e/queue"
y init >/dev/null
mk() { mkdir -p "$1"; printf '# %s\n\n## Goal\nIt.\n\n## Steps\n- [%s] Do it\n' "$(basename "$1")" "${2:- }" > "$1/change.md"; }
mk yass/changes/2026-01-01-alpha; mk yass/changes/2026-02-01-bravo; mk yass/changes/2026-03-01-charlie; mk yass/changes/2026-04-01-delta x
mk yass/changes/2026-03-01-charlie/2026-03-02-piece; mk yass/archive/2025-12-01-old x
git add -A; git commit -q -m "chore: adopt YASS"
order() { yass status | grep -oE '^ *[0-9]{4}-[0-9]{2}-[0-9]{2}-[a-z]+' | tr -d ' ' | paste -sd, -; }
has   "without queue.md, changes are in date order" "^2026-01-01-alpha,2026-02-01-bravo,2026-03-01-charlie,2026-04-01-delta$" order
printf '# Queue\nTop first.\n\n1. 2026-03-01-charlie — first\n2. `2026-01-01-alpha`\n' > yass/queue.md
has   "queue.md order first, then the rest by date" "^2026-03-01-charlie,2026-01-01-alpha,2026-02-01-bravo,2026-04-01-delta$" order
hasnt "…and nothing warns" "warning" y status
git add -A; git commit -q -m "yass: queue"
printf '\n- 2026-09-09-nope\n- 2025-12-01-old\n- 2026-03-02-piece\n- 2026-01-01-alpha\n' >> yass/queue.md
has   "an unknown entry warns" "'2026-09-09-nope' isn't a change in yass/changes" y status
has   "…an archived one" "'2025-12-01-old' is archived" y status
has   "…a piece" "'2026-03-02-piece' is a piece of 2026-03-01-charlie" y status
has   "…and a repeat" "'2026-01-01-alpha' is listed more than once" y status
run_fail "…and --strict fails" y status --strict
git checkout -q yass/queue.md
mkdir -p svc && y init svc --no-agents >/dev/null; mk svc/yass/changes/2026-01-01-one; mk svc/yass/changes/2026-02-01-two
printf -- '- 2026-02-01-two\n' > svc/yass/queue.md
has   "each yass folder follows its own queue.md" "2026-02-01-two,2026-01-01-one" order
hasnt "…without warnings across folders" "warning" y status
rm -rf svc
printf '1. 2026-04-01-delta — done, ready\n2. 2026-03-01-charlie — first\n3. `2026-01-01-alpha`\n' > yass/queue.md
git add -A; git commit -q -m "yass: rank delta"
has   "archive takes a change off queue.md" "took it off yass/queue.md" y archive delta
has   "…keeping the other lines" "^2. 2026-03-01-charlie — first$" head -1 yass/queue.md
has   "…and stages the edit with the move" "^M  yass/queue.md$" git status --short
git commit -q -m "yass: archive delta"
git config core.hooksPath tools/yass/githooks
echo code >> app.txt; printf '1. `2026-01-01-alpha`\n2. 2026-03-01-charlie\n' > yass/queue.md; git add -A
has   "the hook flags reordering queue.md with code" "queue.md is intent" git commit -q -m "feat: sneaky reorder"
git rm -q yass/queue.md; git commit -q -m "yass: drop queue"
echo code >> app.txt; printf -- '- 2026-02-01-bravo\n' > yass/queue.md; git add -A
hasnt "…but not creating one" "heads-up" git commit -q -m "feat: code and a new queue"
git config --unset core.hooksPath

echo "12. blocked: naming changes is a dependency"
newrepo "$W/e2e/deps"
y init >/dev/null
mkb() { mkdir -p "$1"; printf -- '---\nblocked: %s\n---\n# %s\n\n## Goal\nIt.\n\n## Steps\n- [%s] Do it\n' "${3:-}" "$(basename "$1")" "${2:- }" > "$1/change.md"; }
mkb yass/changes/2026-01-01-alpha /; mkb yass/changes/2026-02-01-bravo x "2026-01-01-alpha"
mkb yass/changes/2026-03-01-big; mkb yass/changes/2026-03-01-big/2026-03-02-part
mkb yass/changes/2026-04-01-multi x "yass/changes/2026-01-01-alpha, 2026-03-01-big/2026-03-02-part"
git add -A; git commit -q -m "chore: adopt YASS"
has   "waiting on a change that isn't done" "2026-02-01-bravo .*waiting on: 2026-01-01-alpha" y status
run_fail "archive refuses while it waits" y archive bravo
has   "…and says why" "waiting on 2026-01-01-alpha" y archive bravo
has   "several, as a path and as change/piece" "2026-04-01-multi .*waiting on: 2026-01-01-alpha, 2026-03-01-big/2026-03-02-part" y status
sub yass/changes/2026-01-01-alpha/change.md '^- \[/\]' '- [x]'
hasnt "once it's done, nothing waits on it" "bravo .*(waiting|BLOCKED)" y status
has   "…the detail view suggests clearing it" "clear .blocked:" y status bravo
has   "…and still waiting on the rest" "2026-04-01-multi .*waiting on: 2026-03-01-big/2026-03-02-part$" y status
run_ok "archive works once it's met" y archive bravo
git commit -q -m "yass: archive bravo"
y archive alpha >/dev/null; git commit -q -m "yass: archive alpha"
hasnt "an archived dependency is met" "multi .*alpha" y status
mkb yass/archive/2025-12-01-old x   # an archive from before months: flat
[ -d "yass/archive/$MONTH/2026-01-01-alpha" ] && [ -d yass/archive/2025-12-01-old ] && ok "flat and monthly archives side by side" || bad "layouts not mixed"
mkb yass/changes/2026-05-02-mixed / "2025-12-01-old, 2026-01-01-alpha"
sub yass/changes/2026-05-02-mixed/change.md '^blocked:' 'follows: 2025-12-01-old\nblocked:'
mkb yass/changes/2026-05-03-mixed2 / ; sub yass/changes/2026-05-03-mixed2/change.md '^blocked:' 'follows: 2026-01-01-alpha\nblocked:'
hasnt "follows: and blocked: resolve in either layout" "warning" y status
hasnt "…and both count as met" "mixed .*waiting" y status
has   "status --archived lists both" "2025-12-01-old.*2026-01-01-alpha" bash -c 'yass status --archived | tr "\n" " "'
has   "status finds an archived change in a month" "progress: 1/1  \(done\)" y status --archived 2026-01-01-alpha
printf -- '- 2026-01-01-alpha\n' >> yass/queue.md
has   "queue.md knows a monthly archive is archived" "'2026-01-01-alpha' is archived" y status
rm -rf yass/queue.md yass/changes/2026-05-02-mixed yass/changes/2026-05-03-mixed2 yass/archive/2025-12-01-old
chmod 000 "yass/archive/$MONTH/2026-01-01-alpha/change.md"
mkb yass/changes/2026-05-04-after / "2026-01-01-alpha"; sub yass/changes/2026-05-04-after/change.md '^blocked:' 'follows: 2026-01-01-alpha\nblocked:'
hasnt "status reads no archived files: unreadable ones don't matter" "warning|error|denied" y status
hasnt "…and follows: and blocked: naming one still resolve" "after .*waiting" y status
has   "status --archived says which it can't read" "2026-01-01-alpha/change.md: can't read it: permission denied" y status --archived
chmod 644 "yass/archive/$MONTH/2026-01-01-alpha/change.md"; rm -rf yass/changes/2026-05-04-after
mkb yass/changes/2026-05-01-free x "waiting on 2026-03-01-big and legal sign-off"
has   "free text, even mentioning a change, is a reason" "2026-05-01-free .*BLOCKED: waiting on 2026-03-01-big and legal" y status
run_fail "…and archive refuses" y archive free
mkb yass/changes/2026-05-01-free x "2026-09-09-nope"
has   "a dated name that isn't a change warns" "blocked: '2026-09-09-nope' looks like a change" y status
has   "…and stays a reason" "free .*BLOCKED: 2026-09-09-nope" y status
rm -rf yass/changes/2026-05-01-free
mkb yass/changes/2026-06-01-x "" "2026-06-02-y"; mkb yass/changes/2026-06-02-y "" "2026-06-01-x"
has   "a cycle warns" "waits on itself: 2026-06-01-x → 2026-06-02-y → 2026-06-01-x" y status
has   "…once" "^1$" bash -c 'yass status | grep -c "waits on itself"'
rm -rf yass/changes/2026-06-0*
mkdir -p svc; y init svc --no-agents >/dev/null; mkb svc/yass/changes/2026-07-01-api /
mkb yass/changes/2026-07-02-ui "" "svc/yass/changes/2026-07-01-api"
has   "dependencies work across yass folders" "2026-07-02-ui .*waiting on: 2026-07-01-api" y status

echo "13. yass paths: what's YASS's, for CI"
newrepo "$W/e2e/paths"; rm -rf "$W/e2e/paths-priv"
y init --no-agents >/dev/null; printf 'ignore:\n  - examples/*\n' > yass.yaml
mkdir -p services/payments services/search services/priv examples/demo/yass/changes
y init services/payments --no-agents >/dev/null
printf 'path: planning\n' > services/search/yass.yaml; mkdir -p services/search/planning/changes
printf 'path: ../../../paths-priv\n' > services/priv/yass.yaml; mkdir -p "$W/e2e/paths-priv/changes"
EXP=$'services/payments/yass/**\nservices/priv/yass.yaml\nservices/search/planning/**\nservices/search/yass.yaml\nyass.yaml\nyass/**'
[ "$(y paths)" = "$EXP" ] && ok "paths lists the yass folders in the repo and every yass.yaml, and nothing ignored or outside" || bad "paths: $(y paths | tr '\n' ' ')"
git add -A; git commit -q -m "chore: adopt YASS"
B=$(git rev-parse HEAD); y new "Plan only" >/dev/null; printf 'x\n' > services/payments/yass/changes/note.md; git add -A; git commit -q -m "yass: plan"
has   "--only: a range with only plans exits 0" "plans only: 2 file" y paths --only "$B..HEAD"
run_ok "…with status 0" y paths --only "$B..HEAD"
B=$(git rev-parse HEAD); echo more >> app.txt; echo y >> services/payments/yass/changes/note.md; git commit -qam "feat: code and plans"
run_fail "a range with any other file exits 1" y paths --only "$B..HEAD"
has   "…and names one" "1 of 2 changed file.* e.g. app.txt" y paths --only "$B..HEAD"
run_fail "an empty range runs the checks (exits 1)" y paths --only HEAD..HEAD
has   "a range it can't read exits 2 and says to fetch" "fetch history first" y paths --only nope..HEAD
bash -c 'yass paths --only nope..HEAD' >/dev/null 2>&1; [ $? -eq 2 ] && ok "…with status 2" || bad "not exit 2"
B=$(git rev-parse HEAD); sub yass/changes/*-plan-only/change.md '^- \[ \] $' '- [x] Done'; git commit -qam "yass: done"; y archive plan-only >/dev/null; git commit -q -m "yass: archive"
run_ok "archiving (a move inside yass/) is plans only" y paths --only "$B..HEAD"
cd "$W/e2e/paths-priv"; git init -q -b main; printf 'path: .\n' > yass.yaml
has   "a plans repo (path: .) is YASS's throughout" "^\*\*$" y paths

echo "16. worktrees and the yass/ link"
newrepo "$W/e2e/wt"; rm -rf "$W/e2e/wt-plans" "$W/e2e/wt-other" "$W/e2e/wts" "$W/e2e/wt-priv"*
y init --path ../wt-plans --no-agents >/dev/null; git add yass.yaml; git commit -q -m "chore: adopt YASS"
[ "$(readlink yass)" = "$(cd "$W/e2e/wt-plans" && pwd -P)" ] && ok "init links yass/ to the plans" || bad "no link: $(readlink yass)"
y status >/dev/null; y status >/dev/null
has   "…ignored through info/exclude, once" "^1$" grep -c '^/yass$' .git/info/exclude
[ -z "$(git status --porcelain)" ] && [ ! -e .gitignore ] && ok "…so git status shows nothing, and .gitignore is untouched" || bad "git status: $(git status --porcelain)"
y new "Linked" >/dev/null
has   "a change is listed once" "^1$" bash -c 'yass status | grep -c linked'
hasnt "…with no warnings" "warning" y status
git worktree add -q "$W/e2e/wts/deep/feat" -b feat
cd "$W/e2e/wts/deep/feat"
has   "a worktree elsewhere finds the main checkout's plans" "wt-plans$" y root
has   "…and lists its changes" "linked" y status
[ -L yass ] && [ -z "$(git status --porcelain)" ] && ok "…and gets its own ignored link" || bad "worktree link: $(ls -la yass 2>&1) $(git status --porcelain)"
cd "$W/e2e/wt"; git worktree add -q .claude/worktrees/n1 -b n1
has   "a worktree nested in the repo finds them too" "wt-plans$" bash -c 'cd .claude/worktrees/n1 && yass root'
mv "$W/e2e/wt-plans" "$W/e2e/wt-gone"
cd "$W/e2e/wts/deep/feat"
has   "plans missing from a worktree: the warning says where it looked" "isn't there from this worktree or the clone's other checkouts \(looked in .*wt-plans" y status
hasnt "…and never suggests yass init" "yass init" y status
run_fail "…root fails" y root
has   "…and init refuses rather than make a stray folder" "run .yass init. from the main checkout" y init --no-agents
[ ! -e "$W/e2e/wts/deep/wt-plans" ] && [ ! -e "$W/e2e/wts/wt-plans" ] && ok "…and made nothing" || bad "stray plans folder"
cd "$W/e2e/wt"; mv "$W/e2e/wt-gone" "$W/e2e/wt-plans"
printf 'path: ${WT_PLANS:-../wt-plans}\n' > yass.yaml
has   "\${VAR:-default}: the default when it's unset" "wt-plans$" y root
mkdir -p "$W/e2e/wt-other/changes"
has   "…the variable when it's set" "wt-other$" env WT_PLANS="$W/e2e/wt-other" yass root
case "$(readlink yass)" in *wt-other) ok "…and the link follows it";; *) bad "link not repointed: $(readlink yass)";; esac
y root >/dev/null
case "$(readlink yass)" in *wt-plans) ok "…and back";; *) bad "link not repointed back: $(readlink yass)";; esac
git checkout -q yass.yaml
rm yass; mkdir yass
has   "a real yass/ is left alone, with one warning" "^1$" bash -c 'yass status | grep -c "yass/: already there, so it isn.t a link"'
[ -d yass ] && [ ! -L yass ] && ok "…and kept" || bad "real yass/ replaced"
rmdir yass
newrepo "$W/e2e/wt-priv"; y init --path ../wt-priv-plans --no-agents >/dev/null
echo yass.yaml > .gitignore; git add .gitignore; git commit -q -m "keep the pointer private"
git worktree add -q "$W/e2e/wts/priv" -b feat
has   "a gitignored yass.yaml: a worktree uses the main checkout's" "wt-priv-plans$" bash -c "cd '$W/e2e/wts/priv' && yass root"
[ -L "$W/e2e/wts/priv/yass" ] && ok "…and gets the link" || bad "no link in the private-pointer worktree"
cd "$W/e2e"; rm -rf bare.git bare-wts; git clone -q --bare "$W/e2e/wt" bare.git
git -C bare.git worktree add -q "$W/e2e/bare-wts/main" main 2>/dev/null
git -C bare.git worktree add -q "$W/e2e/bare-wts/deep/feat2" -b feat2 main
mkdir -p "$W/e2e/bare-wts/wt-plans/changes"
has   "a bare clone's worktrees find plans next to another worktree" "bare-wts/wt-plans$" bash -c "cd '$W/e2e/bare-wts/deep/feat2' && yass root"

echo "22. private team folders: kept out of git, found through git config"
newrepo "$W/e2e/pv"; rm -rf "$W/e2e/pv-plans" "$W/e2e/pv-wts"
y init --no-agents >/dev/null; git add -A; git commit -q -m "chore: adopt YASS"
run_ok "init --private sets up a team folder" y init private --path ../../pv-plans --private --no-agents
y init private --path ../../pv-plans --private --no-agents >/dev/null
[ -f private/yass.yaml ] && [ -d "$W/e2e/pv-plans/changes" ] && [ -L private/yass ] && ok "…writes yass.yaml, creates the plans and links private/yass/" || bad "private setup incomplete"
has   "…adds /private to info/exclude once" "^1$" grep -c '^/private$' .git/info/exclude
has   "…and private to git config yass.include once" "^private$" git config --get-all yass.include
[ -z "$(git status --porcelain)" ] && ok "…and git status stays clean" || bad "git status: $(git status --porcelain)"
has   "status from the top lists it" "pv-plans/$" y status
has   "…and root" "pv-plans$" y root
hasnt "paths leaves it out" "private" y paths
P=$(cd private && yass new "Secret")
case "$P" in */e2e/pv-plans/changes/*-secret) ok "new inside private/ lands in the private plans";; *) bad "new went to $P";; esac
echo code >> app.txt; git add app.txt
hasnt "the hook ignores it" "." env YASS_STRICT=1 yass hook
git commit -q -m "feat: code"
git worktree add -q "$W/e2e/pv-wts/deep/a" -b a
cd "$W/e2e/pv-wts/deep/a"
has   "a worktree elsewhere lists the private plans" "pv-plans$" y root
[ -L private/yass ] && [ -z "$(git status --porcelain)" ] && ok "…gets its own private/yass link, and stays clean" || bad "worktree: $(ls -la private 2>&1) $(git status --porcelain)"
P=$(cd private && yass new "From a worktree")
case "$P" in */e2e/pv-plans/changes/*-from-a-worktree) ok "…and new inside its private/ lands there too";; *) bad "new went to $P";; esac
cd "$W/e2e/pv"; git worktree add -q .claude/worktrees/n -b n
has   "a nested worktree lists them too" "pv-plans$" bash -c 'cd .claude/worktrees/n && yass root'
git config --add yass.include gone
has   "an include with no yass.yaml anywhere warns, and says how to stop" "yass.include: 'gone' has no yass.yaml in any checkout .*git config --unset yass.include '.gone.'\)$" y status
git config --unset yass.include '^gone$'
run_fail "--private outside a folder refuses" y init --private --path ../x --no-agents

echo "23. change names end on a whole word"
newrepo "$W/e2e/slug"; y init --no-agents >/dev/null
P=$(y new "Paths that don't exist yet compare through symlinks")
has   "a long title ends on its last whole word" "/$TODAY-paths-that-don-t-exist-yet-compare$" echo "$P"
mkdir -p yass/archive/2026-01-01-paths-that-don-t-exist-yet-compare-throu
printf -- '# Paths that don'"'"'t exist yet compare through symlinks\n\n- [x] done\n' > yass/archive/2026-01-01-paths-that-don-t-exist-yet-compare-throu/change.md
has   "an archived change cut the old way still lists by name alone" "^2026-01-01-paths-that-don-t-exist-yet-compare-throu$" y status --archived

echo "24. yass init sets up a repo from the binary alone"
B="$W/e2e/alone-bin"; rm -rf "$B"; mkdir -p "$B"; cp "$YASS_BIN" "$B/yass"
V="$("$B/yass" --version | sed 's/^yass v\{0,1\}//')"
VR="$(printf '%s' "$V" | sed 's/[.+]/\\&/g')"   # the version as a regex: pseudo-versions have . and +
newrepo "$W/e2e/alone"
INIT_OUT="$(env -u YASS_BIN PATH="$B:/usr/bin:/bin" yass init)"
for f in .agents/skills/yass-log/SKILL.md .agents/skills/yass-plan/SKILL.md .agents/skills/yass-shape/SKILL.md \
         .agents/skills/yass-status/SKILL.md .agents/skills/yass-work/SKILL.md AGENTS.md yass/changes/.gitkeep; do
  [ -e "$f" ] && ok "writes $f" || bad "missing $f"
done
[ -x tools/yass/githooks/pre-commit ] && ok "…and an executable hook script" || bad "no executable hook"
[ ! -e .claude ] && [ ! -e CLAUDE.md ] && ok "…and no Claude files without --claude" || bad "Claude files without --claude"
hasnt "…and leaves the hook off" "githooks" git config core.hooksPath
has   "it lists what it wrote" "wrote .agents/skills/yass-work/SKILL.md" echo "$INIT_OUT"
same=1; for f in "$ROOT"/kit/.agents/skills/*/SKILL.md; do
  n="$(basename "$(dirname "$f")")"
  diff -q <(grep -v -e '^metadata:$' -e '^  yass-version: ' ".agents/skills/$n/SKILL.md") "$f" >/dev/null || same=0
done
diff -q <(grep -v '^# yass-version: ' tools/yass/githooks/pre-commit) "$ROOT/kit/tools/yass/githooks/pre-commit" >/dev/null || same=0
[ "$same" = 1 ] && ok "…each matching its source in kit/ but for the version stamp" || bad "an installed file differs from kit/ beyond its stamp"
has   "playbooks carry the version in their frontmatter" "^  yass-version: \"$VR\"$" cat .agents/skills/yass-work/SKILL.md
has   "…the AGENTS.md marker too" "<!-- yass:begin version=$VR " cat AGENTS.md
has   "…and the hook" "^# yass-version: $VR$" cat tools/yass/githooks/pre-commit
echo "changed" > .agents/skills/yass-work/SKILL.md
has   "re-running keeps what's there" "already set up" yass init
has   "…edits included" "^changed$" cat .agents/skills/yass-work/SKILL.md
newrepo "$W/e2e/alone-claude"; printf '# Our rules\n' > CLAUDE.md
yass init --claude --hooks >/dev/null; yass init --claude >/dev/null
[ -f .claude/skills/yass-work/SKILL.md ] && ok "--claude copies the playbooks for Claude Code" || bad "no .claude/skills"
has   "…imports AGENTS.md at the top of CLAUDE.md" "^@AGENTS.md$" head -1 CLAUDE.md
has   "…once, keeping what was there" "^1 1$" bash -c 'echo "$(grep -c "@AGENTS.md" CLAUDE.md) $(grep -c "# Our rules" CLAUDE.md)"'
has   "--hooks turns the hook on" "^tools/yass/githooks$" git config core.hooksPath
newrepo "$W/e2e/alone-claude2"; yass init --claude >/dev/null
has   "…and writes a CLAUDE.md when there's none" "^@AGENTS.md$" cat CLAUDE.md
newrepo "$W/e2e/alone-global"; H="$W/e2e/alone-home"; rm -rf "$H"
GL_OUT="$(HOME="$H" yass init --global --claude)"
[ -f "$H/.agents/skills/yass-work/SKILL.md" ] && [ -f "$H/.claude/skills/yass-work/SKILL.md" ] && ok "--global puts the playbooks (and Claude's) in the user folder" || bad "no user-folder playbooks"
[ ! -e .agents ] && [ ! -e .claude ] && ok "…none in the repo" || bad "--global wrote playbooks into the repo"
[ -x tools/yass/githooks/pre-commit ] && ok "…while the hook still goes in the repo" || bad "no hook with --global"
newrepo "$W/e2e/alone-skillsdir"; SD="$W/e2e/alone-skills"; rm -rf "$SD"
YASS_SKILLS_DIR="$SD" HOME="$H" yass init --global >/dev/null
[ -f "$SD/yass-work/SKILL.md" ] && ok "…or in \$YASS_SKILLS_DIR" || bad "\$YASS_SKILLS_DIR ignored"
newrepo "$W/e2e/alone-team"; yass init >/dev/null; git add -A; git commit -q -m "chore: adopt YASS"
mkdir -p svc; yass init svc >/dev/null
[ ! -e svc/.agents ] && [ ! -e svc/tools ] && ok "a team folder's init writes no playbooks or hook" || bad "team init wrote kit files"
newrepo "$W/e2e/alone-noagents"; yass init --no-agents >/dev/null
[ ! -e .agents ] && [ ! -e tools ] && [ ! -e AGENTS.md ] && ok "--no-agents skips AGENTS.md, the playbooks and the hook" || bad "--no-agents wrote agent files"

# vbin VERSION: a yass binary that reports VERSION ("dev" for none), built once, for checks that
# depend on versions; CI's own build reports whatever its checkout gives it.
vbin() { local d="$W/vbin/$1"
  if [ ! -x "$d/yass" ]; then mkdir -p "$d"
    if [ "$1" = dev ]; then (cd "$ROOT" && go build -buildvcs=false -o "$d/yass" ./cmd/yass)
    else (cd "$ROOT" && go build -ldflags "-X main.version=$1" -o "$d/yass" ./cmd/yass); fi
  fi; echo "$d/yass"; }

echo "25. agent files per folder, for monorepos"
newrepo "$W/e2e/mono"; yass init >/dev/null; git add -A; git commit -q -m "chore: adopt YASS"
ROOT_AGENTS="$(cksum < AGENTS.md)"
mkdir -p apps/web services/api
yass init services/api >/dev/null
[ -d services/api/yass/changes ] && ok "a folder without --agents gets its planning" || bad "no services/api/yass"
[ ! -e services/api/AGENTS.md ] && [ ! -e services/api/.agents ] && ok "…and no agent files" || bad "folder init wrote agent files"
[ "$(cksum < AGENTS.md)" = "$ROOT_AGENTS" ] && ok "…and leaves the root AGENTS.md alone" || bad "folder init changed the root AGENTS.md"
mkdir -p services/ext; yass init services/ext --path ../../../mono-ext-plans >/dev/null
[ -f services/ext/yass.yaml ] && [ ! -e services/ext/AGENTS.md ] && ok "…or its yass.yaml, with --path" || bad "folder --path init wrote the wrong files"
printf '# Web rules\n' > apps/web/AGENTS.md
WEB_OUT="$(yass init apps/web --agents --claude)"
has   "--agents puts the YASS section in the folder's AGENTS.md" "yass:begin version=" cat apps/web/AGENTS.md
has   "…keeping what was there" "# Web rules" cat apps/web/AGENTS.md
[ -f apps/web/.agents/skills/yass-work/SKILL.md ] && ok "…and the playbooks under the folder" || bad "no apps/web/.agents/skills"
[ -f apps/web/.claude/skills/yass-work/SKILL.md ] && has "…with Claude's copies and import there, with --claude" "^@AGENTS.md$" head -1 apps/web/CLAUDE.md || bad "no Claude files in apps/web"
[ ! -e apps/web/tools ] && ok "…and no second hook" || bad "--agents wrote a hook into the folder"
[ -d apps/web/yass/changes ] && ok "…along with its planning" || bad "no apps/web/yass"
[ "$(cksum < AGENTS.md)" = "$ROOT_AGENTS" ] && ok "…and the root AGENTS.md is untouched" || bad "--agents changed the root AGENTS.md"
has   "it lists what it wrote" "wrote apps/web/.agents/skills/yass-work/SKILL.md" echo "$WEB_OUT"
echo "Ours" > .agents/skills/yass-plan/SKILL.md; rm .agents/skills/yass-log/SKILL.md
REINIT="$("$(vbin 0.5.0)" init)"
has   "re-running init writes what's missing" "wrote .agents/skills/yass-log/SKILL.md" echo "$REINIT"
has   "…and keeps what's there" "^Ours$" cat .agents/skills/yass-plan/SKILL.md
has   "…and says yass upgrade would update files older than the binary (an unstamped one here)" "yass upgrade. updates them" echo "$REINIT"
git checkout -q -- .agents; git clean -qfd .agents
hasnt "a repo at the binary's version gets no upgrade note" "yass upgrade" yass init
sub AGENTS.md 'yass:begin version=\S+' 'yass:begin version=0.0.1'
has   "…an AGENTS.md section stamped older gets one" "yass upgrade. updates them" "$(vbin 0.5.0)" init
has   "…and init leaves the section as it is" "version=0.0.1 " cat AGENTS.md

echo "26. yass upgrade"
Y4="$(vbin 0.4.0)"; Y5="$(vbin 0.5.0)"; Y5D="$(vbin 0.5.0+dirty)"; YDEV="$(vbin dev)"
stamps() { grep -rhoE 'yass-version: "?[^" ]+|yass:begin version=[^ ]+' . --include=SKILL.md --include=AGENTS.md --include=pre-commit 2>/dev/null | sed -E 's/.*(version: "?|version=)//' | sort | uniq -c | tr -s ' '; }
newrepo "$W/e2e/up"; "$Y4" init --claude --hooks >/dev/null
printf '# Ours, before\n\n%s\n\n# Ours, after\n' "$(cat AGENTS.md)" > AGENTS.md; printf '\n# Our Claude notes\n' >> CLAUDE.md
"$Y4" new "Keep me" >/dev/null; git add -A; git commit -q -m "chore: adopt YASS"
KEEP="$(cat yass/changes/*-keep-me/change.md | cksum) $(grep -c 'Ours' AGENTS.md) $(grep -c 'Our Claude notes' CLAUDE.md)"
echo "edited" > .agents/skills/yass-work/SKILL.md; git commit -qam "edit a playbook"
UP="$("$Y5" upgrade)"
has   "upgrade says what it upgraded, from and to" "upgraded YASS's files from 0.4.0.* to 0.5.0" echo "$UP"
has   "…and lists each file" "wrote .claude/skills/yass-work/SKILL.md" echo "$UP"
has   "every stamp is the new version" "^ ?[0-9]+ 0.5.0$" stamps
hasnt "…none left at the old one" "0.4.0" stamps
[ -x tools/yass/githooks/pre-commit ] && ok "…and the hook stays executable" || bad "hook lost its mode"
[ "$(cat yass/changes/*-keep-me/change.md | cksum) $(grep -c 'Ours' AGENTS.md) $(grep -c 'Our Claude notes' CLAUDE.md)" = "$KEEP" ] && ok "changes, text outside the AGENTS.md markers and CLAUDE.md are untouched" || bad "upgrade touched what isn't YASS's"
has   "a hand edit is replaced, and the diff shows it" "^-edited$" git diff -- .agents/skills/yass-work/SKILL.md
git add -A; git commit -qm "chore: upgrade YASS"
has   "at the binary's version, it's already up to date" "already up to date: YASS's files are at 0.5.0" "$Y5" upgrade
[ -z "$(git status --porcelain)" ] && ok "…and changes nothing" || bad "an up-to-date upgrade changed files"
run_fail "an older binary refuses" "$Y4" upgrade
has   "…naming the newer version and the fix" "written by yass 0.5.0, newer than this one \(0.4.0\); .yass update. gets that version" "$Y4" upgrade
[ -z "$(git status --porcelain)" ] && ok "…and writes nothing" || bad "a refused upgrade changed files"
has   "a +dirty build rewrites its own version" "wrote AGENTS.md" "$Y5D" upgrade
git checkout -q -- .
run_fail "a binary with no version refuses to upgrade" "$YDEV" upgrade
has   "…saying how to get a versioned build" "has no version .*Go 1.24" "$YDEV" upgrade
newrepo "$W/e2e/up-dev"; run_fail "…or to init" "$YDEV" init
[ ! -e AGENTS.md ] && ok "…writing nothing" || bad "an unversioned init wrote files"
newrepo "$W/e2e/up-global"; H="$W/e2e/up-home"; rm -rf "$H"
has   "init labels files outside version control" "wrote .*up-home/.agents/skills/yass-work/SKILL.md \(outside version control\)" env HOME="$H" "$Y4" init --global
GUP="$(HOME="$H" "$Y5" upgrade)"
has   "upgrade upgrades user-folder playbooks, labeled outside version control" "up-home/.agents/skills/yass-work/SKILL.md \(outside version control\)" echo "$GUP"
has   "…to the new version" "yass-version: \"0.5.0\"" cat "$H/.agents/skills/yass-work/SKILL.md"
[ ! -e .agents ] && ok "…without adding repo copies" || bad "upgrade added repo playbooks"
newrepo "$W/e2e/up-legacy"
for f in "$ROOT"/kit/.agents/skills/*/SKILL.md; do n="$(basename "$(dirname "$f")")"
  mkdir -p ".agents/skills/$n" ".claude/skills/$n"; cp "$f" ".agents/skills/$n/"; cp "$f" ".claude/skills/$n/"; done
mkdir -p tools/yass/githooks yass/changes yass/archive; cp "$ROOT/kit/tools/yass/githooks/pre-commit" tools/yass/githooks/
"$Y5" template agents > AGENTS.md; printf '@AGENTS.md\n' > CLAUDE.md
has   "a repo set up by v0.2 (no stamps) upgrades" "from unstamped \(v0.2 or earlier\) to 0.5.0" "$Y5" upgrade
newrepo "$W/e2e/up-fresh"; "$Y5" init --claude --hooks >/dev/null
same=1; for f in .agents .claude tools AGENTS.md CLAUDE.md; do diff -r -q "$W/e2e/up-legacy/$f" "$f" >/dev/null || same=0; done
[ "$same" = 1 ] && ok "…ending up exactly like a fresh init" || bad "upgraded v0.2 repo differs from a fresh init: $(diff -rq "$W/e2e/up-legacy/.agents" .agents; diff -q "$W/e2e/up-legacy/AGENTS.md" AGENTS.md)"
NG2="$W/e2e/up-nogit"; rm -rf "$NG2"; mkdir -p "$NG2"
run_fail "upgrade outside a git repo refuses" bash -c 'cd "$1" && "$2" upgrade' _ "$NG2" "$Y5"
has   "…and says why" "works inside a git repo" bash -c 'cd "$1" && "$2" upgrade' _ "$NG2" "$Y5"
newrepo "$W/e2e/up-mono"; "$Y4" init >/dev/null; mkdir -p apps/web services/api vendor/x
"$Y4" init apps/web --agents >/dev/null; "$Y4" init services/api --agents >/dev/null
mkdir -p vendor/x/.agents/skills/yass-work; cp apps/web/.agents/skills/yass-work/SKILL.md vendor/x/.agents/skills/yass-work/
printf 'ignore:\n  - vendor/*\n' > yass.yaml
rm apps/web/.agents/skills/yass-log/SKILL.md; git add -A; git commit -q -m "chore: adopt YASS"
MUP="$(cd services/api && "$Y5" upgrade)"
has   "upgrade from a monorepo folder upgrades every copy in the repo" "wrote apps/web/AGENTS.md" echo "$MUP"
hasnt "…leaving none at the old version" "0.4.0" bash -c 'grep -rh "yass-version\|yass:begin" --include=SKILL.md --include=AGENTS.md --include=pre-commit apps services tools .agents AGENTS.md'
[ -f apps/web/.agents/skills/yass-log/SKILL.md ] && ok "…and adds a playbook missing from a skills folder" || bad "missing playbook not added"
has   "…but leaves folders yass.yaml ignores alone" "0.4.0" cat vendor/x/.agents/skills/yass-work/SKILL.md

echo "27. yass status knows the versions"
Y4="$(vbin 0.4.0)"; Y5="$(vbin 0.5.0)"; Y6="$(vbin 0.6.0)"
newrepo "$W/e2e/st"; "$Y5" init >/dev/null; git add -A; git commit -q -m "chore: adopt YASS"
has   "an older binary hears the repo is newer, and to run yass update" "note: YASS's files here were written by yass 0.5.0, newer than this one \(0.4.0\); .yass update. gets that version" "$Y4" status
run_ok "…as a note, so --strict doesn't fail on it" "$Y4" status --strict
has   "a newer binary hears yass upgrade would upgrade the repo" "note: YASS's files here are older than this yass \(0.6.0\); .yass upgrade. would upgrade them" "$Y6" status
hasnt "the same version says nothing about versions" "YASS's files" "$Y5" status
newrepo "$W/e2e/st-none"; "$Y5" init --no-agents >/dev/null
hasnt "…nor does a repo with no YASS files" "YASS's files" "$Y6" status

echo "28. --hooks leaves existing hooks alone"
newrepo "$W/e2e/hk-none"
HO="$(yass init --hooks)"
has   "no hooks setup yet: --hooks turns the hook on" "^tools/yass/githooks$" git config core.hooksPath
has   "…and says so" "hook  core.hooksPath = tools/yass/githooks" echo "$HO"
has   "running it again: already on" "hook  already on" yass init --hooks
newrepo "$W/e2e/hk-husky"; mkdir -p .husky; git config core.hooksPath .husky
run_ok "a local core.hooksPath elsewhere: init still succeeds" yass init --hooks
has   "…and leaves core.hooksPath as it was" "^\.husky$" git config core.hooksPath
has   "…naming the setup" "already has a hooks setup \(core.hooksPath = \.husky\)" yass init --hooks
has   "…and how to run yass hook from husky" "add this line to \.husky/pre-commit:" yass init --hooks
[ -x tools/yass/githooks/pre-commit ] && [ -f .agents/skills/yass-work/SKILL.md ] && ok "…while writing everything else" || bad "init skipped files because of the hooks setup"
newrepo "$W/e2e/hk-custom"; git config core.hooksPath ci/hooks
has   "a custom hooks folder gets the line for its pre-commit" "add this line to ci/hooks/pre-commit .*tools/yass/githooks/pre-commit \"\\\$@\"" bash -c 'yass init --hooks | tr "\n" " "'
newrepo "$W/e2e/hk-global"; GC="$W/e2e/hk-gitconfig"; printf '[core]\n\thooksPath = %s\n' "$W/e2e/hk-global-hooks" > "$GC"
GO="$(GIT_CONFIG_GLOBAL="$GC" yass init --hooks)"
hasnt "a global core.hooksPath: no local one is added" "." git config --local --get core.hooksPath
has   "…and it says why, and how to switch anyway" "global core.hooksPath .*override it for this repo.*git config core.hooksPath tools/yass/githooks" bash -c 'tr "\n" " " <<<"$1"' _ "$GO"
newrepo "$W/e2e/hk-githooks"; printf '#!/bin/sh\nexit 0\n' > .git/hooks/pre-commit; cp .git/hooks/pre-commit .git/hooks/commit-msg; chmod +x .git/hooks/pre-commit .git/hooks/commit-msg
LO="$(yass init --hooks)"
hasnt "hooks in .git/hooks: core.hooksPath stays unset" "." git config --get core.hooksPath
has   "…naming them (not the samples)" "runs hooks from \.git/hooks \(commit-msg, pre-commit\)" echo "$LO"
has   "…with the line to add to .git/hooks/pre-commit" "^        tools/yass/githooks/pre-commit \"\\\$@\"$" echo "$LO"
newrepo "$W/e2e/hk-lefthook"; printf 'pre-push:\n  commands: {}\n' > lefthook.yml; printf '#!/bin/sh\n' > .git/hooks/pre-commit; chmod +x .git/hooks/pre-commit
has   "lefthook gets a lefthook.yml snippet" "run: yass hook" yass init --hooks
newrepo "$W/e2e/hk-precommit"; printf 'repos: []\n' > .pre-commit-config.yaml; printf '#!/bin/sh\n' > .git/hooks/pre-commit; chmod +x .git/hooks/pre-commit
has   "the pre-commit framework gets a local-hook snippet" "entry: yass hook" yass init --hooks
cd "$W/e2e/hk-husky"; git add -A; git commit -q -m "chore: adopt YASS" --no-verify
"$(vbin 0.5.0)" upgrade >/dev/null
has   "yass upgrade leaves core.hooksPath alone" "^\.husky$" git config core.hooksPath

echo "29. same-day changes list in creation order"
newrepo "$W/e2e/created"; y init --no-agents >/dev/null
PZ=$(y new "Zebra"); PA=$(y new "Apple"); PP=$(y new "Parent" --large); PC=$(y new "Piece zebra" --in Parent); PB=$(y new "Piece apple" --in Parent)
has   "yass new stamps created: in UTC" "^created: [0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z$" cat "$PZ/change.md"
has   "…and keeps the dated folder name" "^yass/changes/$TODAY-zebra$" echo "$PZ"
has   "…a large change too" "^created: .*Z$" cat "$PP/change.md"
has   "…and a piece" "^created: .*Z$" cat "$PC/change.md"
sub "$PZ/change.md" '^created: .*$' "created: ${TODAY}T00:00:01Z"; sub "$PA/change.md" '^created: .*$' "created: ${TODAY}T00:00:02Z"
sub "$PP/change.md" '^created: .*$' "created: ${TODAY}T00:00:03Z"
sub "$PC/change.md" '^created: .*$' "created: ${TODAY}T00:00:04Z"; sub "$PB/change.md" '^created: .*$' "created: ${TODAY}T00:00:05Z"
mkdir -p "yass/changes/$TODAY-mango"; printf -- '---\nblocked:\n---\n# Mango\n\n- [ ] a\n' > "yass/changes/$TODAY-mango/change.md"
has   "same-day changes list in creation order, unstamped first" "mango .*zebra .*apple .*parent .*piece-zebra .*piece-apple " bash -c 'yass status | tr "\n" " "'
has   "an unstamped change still resolves" "# Mango|Mango" y status mango
hasnt "…and draws no warning" "created" y status
sub "$PA/change.md" '^created: .*$' "created: yesterday"
has   "a created: that isn't a time warns" "apple: created 'yesterday' isn't a time" y status
has   "…and sorts as unstamped (by date, then name)" "apple .*mango .*zebra " bash -c 'yass status | tr "\n" " "'

echo "30. AGENTS.md says intent and archives land apart from code"
Y5="$(vbin 0.5.0)"
newrepo "$W/e2e/apart"; "$Y5" init >/dev/null
has   "the section says intent stays out of a squashed pull request with code" "never changes in the same commit as code \(with squash merges, the same pull request\)" cat AGENTS.md
has   "…that archives land apart from code" "in its own commit, apart from code" cat AGENTS.md
has   "…and that pushing and pull requests need latitude" "rebase, push or open a pull request unless" cat AGENTS.md
git add -A; git commit -q -m "chore: adopt YASS"
sub AGENTS.md 'yass:begin version=0\.5\.0' 'yass:begin version=0.4.0'
sub AGENTS.md ', apart from code \(with squash merges, a pull request without code\)' ''
"$Y5" upgrade >/dev/null
has   "yass upgrade brings an older section up to it" "in its own commit, apart from code" cat AGENTS.md

echo "31. yass upgrade moves a flat archive into months"
Y6="$(vbin 0.6.0)"
newrepo "$W/e2e/migrate"; "$Y6" init --hooks >/dev/null
flat() { mkdir -p "$1"; printf '# %s\n\n- [x] Done\n' "$(basename "$1")" > "$1/change.md"; }
flat yass/archive/2026-01-10-one; flat yass/archive/2026-02-20-two/2026-02-21-piece
git add -A; GIT_COMMITTER_DATE=2026-04-15T12:00:00Z git commit -q -m "yass: archive one and two"
flat yass/archive/2026-03-01-untracked
OUT="$("$Y6" upgrade)"
has   "upgrade names the step" "migrated yass \(archive-into-months\): moved 3 archived" echo "$OUT"
has   "…and says to commit it on its own" 'commit it on its own: git commit -m "yass: archive-into-months" -- yass$' echo "$OUT"
[ -f yass/archive/2026/04/2026-01-10-one/change.md ] && ok "a change goes in the month of the commit that archived it" || bad "not in 2026/04"
has   "…stamped with that commit's date" "^archived: 2026-04-15T12:00:00Z$" cat yass/archive/2026/04/2026-01-10-one/change.md
[ -f yass/archive/2026/04/2026-02-20-two/2026-02-21-piece/change.md ] && ok "…with its pieces" || bad "piece left behind"
[ -f yass/archive/2026/03/2026-03-01-untracked/change.md ] && ok "one git never saw goes by the date in its name" || bad "untracked not by name"
[ ! -e yass/archive/2026-01-10-one ] && ok "nothing left flat" || bad "flat folder left"
hasnt "status is clean after it" "warning" y status --strict
hasnt "the hook passes the move" "heads-up|append-only" git commit -q -m "yass: archive-into-months" -- yass
hasnt "…which committed" "." git status --porcelain yass
hasnt "a second upgrade does nothing" "migrated" "$Y6" upgrade
git mv yass/archive/2026/04/2026-01-10-one yass/archive/2026/05/2026-01-10-one 2>/dev/null || { mkdir -p yass/archive/2026/05; git mv yass/archive/2026/04/2026-01-10-one yass/archive/2026/05/; }
has   "moving between months is still flagged" "append-only" git commit -q -m "yass: shuffle"
git reset -q --hard HEAD~1
flat yass/archive/2026-06-01-late; git add -A; git commit -q -m "yass: archive late"
"$Y6" upgrade >/dev/null; echo "- [x] Sneaked in" >> yass/archive/2026/*/2026-06-01-late/change.md; git add -A
has   "a move that also edits the change is flagged" "append-only" git commit -q -m "yass: migrate and edit"
git reset -q --hard HEAD~1

newrepo "$W/e2e/migrate-plain"; P6="$W/e2e/plain-plans"; rm -rf "$P6"; flat "$P6/archive/2026-02-03-elsewhere"
printf 'path: %s\n' "$P6" > yass.yaml
OUT="$("$Y6" upgrade)"
has   "a yass folder outside git migrates too" "migrated .*plain-plans \(archive-into-months\)" echo "$OUT"
hasnt "…with nothing to commit" "commit it on its own" echo "$OUT"
has   "…dated by its name, at midnight UTC" "^archived: 2026-02-03T00:00:00Z$" cat "$P6/archive/2026/02/2026-02-03-elsewhere/change.md"

PR6="$W/e2e/plans-repo"; newrepo "$PR6"; flat archive/2026-01-05-planned; git add -A
GIT_COMMITTER_DATE=2026-03-09T08:30:00Z git commit -q -m "yass: archive planned"
newrepo "$W/e2e/migrate-code"; printf 'path: %s\n' "$PR6" > yass.yaml
OUT="$("$Y6" upgrade)"
has   "a plans repo's dates come from its own history" "^archived: 2026-03-09T08:30:00Z$" cat "$PR6/archive/2026/03/2026-01-05-planned/change.md"
has   "…and the commit runs there" "git -C .*plans-repo commit -m \"yass: archive-into-months\" -- .*plans-repo" echo "$OUT"

echo "32. yass decisions"
newrepo "$W/e2e/decisions"; y init >/dev/null; y init teams/a --no-agents >/dev/null
# mkd <dir> <frontmatter lines> <entries…>
mkd() { local d="$1" fm="$2"; shift 2; mkdir -p "$d"
  { printf -- '---\n%b---\n# %s\n\n## Steps\n- [x] Done\n\n## Decisions\n<!-- - not an entry (x, 2026-01-01) -->\n' "$fm" "$(basename "$d")"
    for e in "$@"; do printf -- '- %s\n' "$e"; done; printf '\n## Log\n- Did: a thing (not a decision)\n'; } > "$d/change.md"; }
mkd yass/archive/2026/02/2026-01-01-origin 'archived: 2026-02-15T09:00:00Z\n' "Origin call - old (ann)"
mkd yass/changes/2026-03-01-bee 'follows: 2026-01-01-origin\n' "Bee call (bo, 2026-03-05)"
mkd yass/changes/2026-03-01-bee/2026-03-02-bee-piece '' "Bee piece call (bp, 2026-03-03)"
mkd teams/a/yass/changes/2026-04-01-cee 'follows: 2026-03-01-bee\n' "Cee call about Squash Merge - why (cy, 2026-04-02)"
mkd teams/a/yass/changes/2026-04-10-dee 'created: 2026-04-10T08:00:00Z\n' "Dee call (dy, 2026-04-11)"
has   "an archived entry, with the change's dates and no date of its own" "^ {10}  2026-01-01-origin  Origin call - old \(ann\)  \[created 2026-01-01, archived 2026-02-15\]$" y decisions
has   "an active entry in another yass folder, with its own date" "^2026-04-02  2026-04-01-cee  Cee call about Squash Merge - why \(cy\)  \[created 2026-04-01\]$" y decisions
has   "a piece's entries, under the piece" "^2026-03-03  2026-03-01-bee/2026-03-02-bee-piece  Bee piece call \(bp\)" y decisions
has   "each entry once, newest first, comments and Log left out" "^dee,cee,bee,piece,origin$" bash -c 'yass decisions | sed -E "s/^.{12}[^ ]*-([a-z-]+)  .*/\1/" | paste -sd, -'
has   "--since and --until, by each entry's date (else archived, else created)" "^cee,bee$" bash -c 'yass decisions --since 2026-03-04 --until 2026-04-05 | sed -E "s/^.{12}[^ ]*-([a-z-]+)  .*/\1/" | paste -sd, -'
has   "…an undated entry goes by its change's archived date" "origin" y decisions --since 2026-02-15 --until 2026-02-15
has   "--about matches all its words, any case" "^2026-04-02  2026-04-01-cee" y decisions --about "squash MERGE"
hasnt "…and nothing else" "bee|dee|origin" y decisions --about "squash merge"
has   "--about also matches the change's title" "dee call" bash -c 'yass decisions --about "10-dee" | tr A-Z a-z'
has   "--change takes in what it follows and what follows it, with pieces" "^cee,bee,piece,origin$" bash -c 'yass decisions --change 2026-03-01-bee | sed -E "s/^.{12}[^ ]*-([a-z-]+)  .*/\1/" | paste -sd, -'
has   "--limit keeps the newest and says how many it left out" "^dee,cee,… 3 more not shown" bash -c 'yass decisions --limit 2 | sed -E "s/^.{12}[^ ]*-([a-z-]+)  .*/\1/" | paste -sd, -'
hasnt "…and under the limit, says nothing more" "more not shown" y decisions
has   "no match says so" "^no decisions match$" y decisions --about zzz
run_ok "…and exits 0" y decisions --about zzz
run_fail "a bad date is refused" y decisions --since yesterday
has   "--json is one array with every field" '"change": "2026-04-01-cee",' y decisions --json
has   "…and parses" "Bee piece call" bash -c 'yass decisions --json | python3 -c "import json,sys; d=json.load(sys.stdin); print([x[\"decision\"] for x in d]); assert set(d[0]) == {\"change\",\"title\",\"who\",\"decision\",\"date\",\"created\",\"archived\"}"'
has   "…with null for what's unknown" "^None None 2026-02-15$" bash -c 'yass decisions --json --change origin | python3 -c "import json,sys; o=[x for x in json.load(sys.stdin) if x[\"change\"].endswith(\"origin\")][0]; print(o[\"date\"], None if o[\"date\"] else None, o[\"archived\"])"'
has   "…the same filters apply" '^\[\]$' y decisions --json --about zzz
has   "new changes ask for dated entries" "\(<who>, <YYYY-MM-DD>\)" y template change
has   "…large ones too" "\(<who>, <YYYY-MM-DD>\)" y template change-large
has   "…and so does the AGENTS.md section" "\(<who>, <YYYY-MM-DD>\)" cat AGENTS.md
newrepo "$W/e2e/decisions-none"
has   "no yass folder says so" "no yass folder" y decisions
run_ok "…and exits 0" y decisions

echo "33. keep and yass evict"
NOWM=$(date -u +%Y/%m)
mka() { mkdir -p "$1"; printf -- '---\narchived: %s\n---\n# %s\n\n- [x] Done\n' "${2:-2025-01-15T00:00:00Z}" "$(basename "$1")" > "$1/change.md"; }
newrepo "$W/e2e/evict"; y init --hooks >/dev/null
for n in 3 1 2; do mka yass/archive/2025/01/2025-01-0$n-jan-$n; done
for n in 1 2 3 4; do mka yass/archive/2025/02/2025-02-0$n-feb-$n 2025-02-20T00:00:00Z; done
mka "yass/archive/$NOWM/$TODAY-now-1" "${TODAY}T00:00:00Z"
printf 'archive:\n  keep: 3\n  kepp: 4\n' > yass.yaml
y init teams/x --no-agents >/dev/null; mka teams/x/yass/archive/2025/01/2025-01-01-team-one
git add -A; git commit -q -m "yass: archives"; JANSHA=$(git log -1 --format=%H -- yass/archive/2025/01)
has   "an unknown setting under archive: warns" "unknown setting 'archive.kepp'" y status
has   "a folder past keep gets a note" "note: yass/archive holds 8 archived changes, 5 past its keep of 3; .yass evict." y status
hasnt "…another folder keeps the default" "teams/x/yass/archive holds" y status
run_ok "…and --strict still passes on it" bash -c 'yass status --strict | grep -v kepp; sed -i.bak "/kepp/d" yass.yaml; yass status --strict'
rm -f yass.yaml.bak; git add -A; git commit -q -m "yass: settings"
N=$(y new "One more"); sub "$N/change.md" '^- \[ \] $' '- [x] Done'; git add -A; git commit -q -m wip
has   "archive says so too" "note: yass/archive holds 9 archived changes, 6 past its keep of 3" y archive one-more
git commit -q -m "yass: archive one-more"
OUT="$(y evict)"
has   "evict removes the oldest whole months…" "evicted yass/archive/2025/01: 3 archived change" echo "$OUT"
has   "…until it's at or under keep" "evicted yass/archive/2025/02: 4 archived change" echo "$OUT"
has   "…says what's left" "now holds 2 archived changes \(keep: 3\)" echo "$OUT"
has   "…and how to commit it" 'Commit it on its own: git commit -m "yass: evict archived months" -- yass/archive' echo "$OUT"
[ ! -e yass/archive/2025/01 ] && [ ! -e yass/archive/2025/02 ] && ok "…the months are gone" || bad "months left"
[ -d "yass/archive/$NOWM/$TODAY-now-1" ] && ok "…never this month" || bad "current month touched"
[ -d teams/x/yass/archive/2025/01 ] && ok "…nor a folder within its keep" || bad "other folder evicted"
has   "the manifest names the commit, its path and the changes, sorted" "^commit: $JANSHA
path: yass/archive/2025/01
changes:
  - 2025-01-01-jan-1
  - 2025-01-02-jan-2
  - 2025-01-03-jan-3$" bash -c 'grep -v "^#" yass/archive/2025/01.evicted'
hasnt "the hook passes the eviction" "heads-up|append-only" git commit -q -m "yass: evict archived months" -- yass/archive
has   "…and it reads back from git" "# 2025-01-02-jan-2" git show "$JANSHA:yass/archive/2025/01/2025-01-02-jan-2/change.md"
has   "evict again: nothing to do" "nothing to evict" y evict
mkb yass/changes/2026-05-05-uses-old / "2025-02-03-feb-3"; sub yass/changes/2026-05-05-uses-old/change.md '^blocked:' 'follows: 2025-01-02-jan-2\nblocked:'
hasnt "evicted changes still resolve for follows: and blocked:" "warning" y status
hasnt "…and count as met" "uses-old .*waiting" y status
has   "status --archived summarizes what's evicted" "… and 7 evicted, from 2025/01, 2025/02" y status --archived
hasnt "…and lists only what's in the tree" "jan-2|feb-3" y status --archived
git add -A; git commit -q -m "uses old" >/dev/null 2>&1
SH="$W/e2e/evict-shallow"; rm -rf "$SH"; git clone -q --depth 1 "file://$PWD" "$SH"
run_fail "in a shallow clone the evicted months' commit is missing" git -C "$SH" cat-file -e "$JANSHA"
hasnt "…and evicted names still resolve there" "warning" bash -c "cd '$SH' && yass status"
git reset -q --hard HEAD~1
git rm -q -r "yass/archive/$NOWM/$TODAY-now-1"
has   "deleting from the archive without a manifest is still flagged" "append-only" git commit -q -m "yass: tidy"
git reset -q --hard HEAD~1
has   "init's yass README says evict is the one exception" "append-only,\*\* except that .yass evict." cat yass/README.md
has   "…and so does AGENTS.md" "only .yass evict. removes from it" cat AGENTS.md

newrepo "$W/e2e/evict-refuse"; y init >/dev/null; printf 'archive:\n  keep: 1\n' > yass.yaml
for n in 1 2; do mka yass/archive/2025/0$n/2025-0$n-01-m$n; done; git add -A; git commit -q -m arch
echo "- [x] late edit" >> yass/archive/2025/01/2025-01-01-m1/change.md
run_fail "a month with uncommitted changes: evict refuses" y evict
has   "…and says why" "has changes that aren't committed" y evict
[ -d yass/archive/2025/01 ] && ok "…and changes nothing" || bad "evicted anyway"
git checkout -q -- yass
P7="$W/e2e/plain-evict"; rm -rf "$P7"; for n in 1 2; do mka "$P7/archive/2025/0$n/2025-0$n-01-p$n"; done
newrepo "$W/e2e/evict-plain"; printf 'path: %s\narchive:\n  keep: 1\n' "$P7" > yass.yaml
run_fail "outside git, evict refuses" y evict
has   "…and says why" "isn't in git" y evict

newrepo "$W/e2e/evict-merge"; y init >/dev/null; printf 'archive:\n  keep: 1\n' > yass.yaml
for n in 1 2 3; do mka yass/archive/2025/0$n/2025-0$n-01-x$n; done; git add -A; git commit -q -m arch
git switch -q -c one; y evict >/dev/null; git commit -q -m "yass: evict" -- yass/archive
git switch -q main; git switch -q -c two; y evict >/dev/null; git commit -q -m "yass: evict" -- yass/archive
run_ok "two branches evicting the same months merge without conflict" git merge -q --no-edit one
git switch -q main; git switch -q -c three; y evict >/dev/null; git commit -q -m "yass: evict" -- yass/archive
git switch -q main; git merge -q --squash three >/dev/null; git commit -q -m "yass: evict (squashed)"
S1=$(sed -n 's/^commit: //p' yass/archive/2025/01.evicted)
run_ok "after a squash merge, the manifest's commit is still on main" git merge-base --is-ancestor "$S1" main

echo "34. decisions from evicted months, and --cites"
newrepo "$W/e2e/history"; y init >/dev/null; printf 'archive:\n  keep: 1\n' > yass.yaml
CODE=$(git rev-parse HEAD); C7=${CODE:0:7}
# mkh <dir> <archived> <follows> <decision> [box]
mkh() { mkdir -p "$1"; printf -- '---\narchived: %s\nfollows: %s\n---\n# %s\n\n## Steps\n- [x] %s\n\n## Decisions\n- %s\n' "$2" "$3" "$(basename "$1")" "${5:-Done}" "$4" > "$1/change.md"; }
mkh yass/archive/2025/01/2025-01-02-jan-root 2025-01-20T00:00:00Z "" "Jan call (ann, 2025-01-10)" "Built it — code: $C7"
mkh yass/archive/2025/01/2025-01-02-jan-root/2025-01-03-jan-piece 2025-01-20T00:00:00Z "" "Jan piece call (ap, 2025-01-11)"
mkh yass/archive/2025/02/2025-02-02-feb-next 2025-02-20T00:00:00Z 2025-01-02-jan-root "Feb call (bo, 2025-02-10)"
mkh "yass/archive/$NOWM/$TODAY-now-done" "${TODAY}T00:00:00Z" "" "Now call (cy, $TODAY)" "Cited too — code: $CODE"
git add -A; git commit -q -m "yass: archives"; y evict >/dev/null; git commit -q -m "yass: evict" -- yass/archive
mkh yass/changes/2026-06-01-active "" 2025-02-02-feb-next "Active call (dy, 2026-06-02)" "Cites it — code: $C7"
sub yass/changes/2026-06-01-active/change.md '^archived: \n' ''
git add -A; git commit -q -m "yass: active"
[ ! -e yass/archive/2025/01 ] && ok "(set up: January and February are evicted)" || bad "setup: not evicted"
has   "--since reaching evicted months reads them from git" "^2025-01-10  2025-01-02-jan-root  Jan call \(ann\)  \[created 2025-01-02, archived 2025-01-20\]$" y decisions --since 2025-01-01
has   "…pieces too" "^2025-01-11  2025-01-02-jan-root/2025-01-03-jan-piece  Jan piece call" y decisions --since 2025-01-01
has   "--change follows the chain into evicted months, with pieces" "^active,next,piece,root$" bash -c 'yass decisions --change 2026-06-01-active | sed -E "s/^.{12}[^ ]*-([a-z]+)  .*/\1/" | paste -sd, -'
has   "--change naming an evicted change works" "Feb call" y decisions --change 2025-02-02-feb-next
has   "a plain query that's filled by the tree doesn't need them" "^$TODAY  $TODAY-now-done  Now call" y decisions --limit 1
has   "…one that isn't, does" "Jan call" y decisions
has   "--cites finds boxes citing a commit: active, archived and evicted, by short or long hash" "^2025-01-02-jan-root,2026-06-01-active,${TODAY}-now-done$" bash -c "yass decisions --cites $C7 | cut -d' ' -f1 | sort | paste -sd, -"
has   "…the same by its full hash" "jan-root" y decisions --cites "$CODE"
has   "…and says which file and box" "^2026-06-01-active  change.md  \[x\] Cites it — code: $C7$" y decisions --cites "$C7"
has   "a commit nobody cites" "^no box cites 0000000$" y decisions --cites 0000000
run_fail "--cites wants a hash" y decisions --cites nothex
has   "--cites --json" '"file": "change.md",' y decisions --cites "$C7" --json
SH="$W/e2e/history-shallow"; rm -rf "$SH"; git clone -q --depth 1 "file://$PWD" "$SH"
hasnt "shallow clone: a query that doesn't reach evicted months reads no history" "couldn't read" bash -c "cd '$SH' && yass decisions --since 2026-01-01"
has   "…and gives the same answer" "Active call" bash -c "cd '$SH' && yass decisions --since 2026-01-01"
hasnt "shallow clone: a plain query the tree fills reads no history either" "couldn't read" bash -c "cd '$SH' && yass decisions --limit 1"
has   "shallow clone: one that reaches them prints the rest" "Active call" bash -c "cd '$SH' && yass decisions --since 2025-01-01"
has   "…and names what it couldn't read, and how to get it" "couldn't read yass/archive/2025/01, yass/archive/2025/02: .*git fetch --unshallow" bash -c "cd '$SH' && yass decisions --since 2025-01-01"
run_ok "…and exits 0" bash -c "cd '$SH' && yass decisions --since 2025-01-01"
has   "--cites in a shallow clone says what it couldn't read too" "couldn't read" bash -c "cd '$SH' && yass decisions --cites $C7"
has   "…JSON stays clean on stdout" '^\[' bash -c "cd '$SH' && yass decisions --since 2025-01-01 --json 2>/dev/null | head -1"

echo "35. yass update --check"
# rbin <version>: a release build (it carries the release marker), built once
rbin() { local d="$W/rbin/$1"
  [ -x "$d/yass" ] || { mkdir -p "$d"; (cd "$ROOT" && go build -ldflags "-X main.version=$1 -X main.channel=release" -o "$d/yass" ./cmd/yass); }
  echo "$d/yass"; }
# releases <latest>: a fake release server whose /releases/latest redirects to v<latest>, as GitHub's does
RELPID=""
releases() { [ -n "$RELPID" ] && { kill "$RELPID"; wait "$RELPID"; } 2>/dev/null; rm -f "$W/relport"
  python3 - "$1" "$W/relport" "$W/releases" <<'PY' &
import http.server, socketserver, sys, os
latest, portfile, root = sys.argv[1], sys.argv[2], sys.argv[3]
class H(http.server.BaseHTTPRequestHandler):
    def do_HEAD(self, body=False):
        if self.path == "/releases/latest":
            self.send_response(302); self.send_header("Location", "/releases/tag/v" + latest); self.end_headers(); return
        f = os.path.join(root, self.path.removeprefix("/releases/download/"))
        if self.path.startswith("/releases/download/") and os.path.isfile(f):
            data = open(f, "rb").read()
            self.send_response(200); self.send_header("Content-Length", str(len(data))); self.end_headers()
            if body: self.wfile.write(data)
            return
        self.send_response(404); self.end_headers()
    def do_GET(self): self.do_HEAD(body=True)
    def log_message(self, *a): pass
s = socketserver.TCPServer(("127.0.0.1", 0), H)
open(portfile, "w").write(str(s.server_address[1])); s.serve_forever()
PY
  RELPID=$!
  for _ in $(seq 50); do [ -s "$W/relport" ] && break; sleep 0.1; done
  export YASS_RELEASES_URL="http://127.0.0.1:$(cat "$W/relport")/releases"; }
R4="$(rbin 0.4.0)"; R6="$(rbin 0.6.0)"
newrepo "$W/e2e/update"; "$(vbin 0.5.0)" init >/dev/null
releases 0.6.0
OUT="$("$R4" update --check)"
has   "--check names the binary and how it was built" "^yass 0.4.0 \(release build, " echo "$OUT"
has   "…the repo's version" "^this repo's YASS files: 0.5.0$" echo "$OUT"
has   "…the latest release" "^latest release: 0.6.0$" echo "$OUT"
has   "…and that it would match the repo" "^yass update would install 0.5.0 \(to match this repo's YASS files\)$" echo "$OUT"
has   "…says a newer one exists, how to get it, and what follows" "^0.6.0 is newer: .yass update --latest. gets it; then run .yass upgrade. in each repo and commit the diff" echo "$OUT"
run_ok "…exits 0" "$R4" update --check
has   "…and changes nothing" "^yass 0.4.0$" "$R4" version
has   "--latest: it would install the latest" "would install 0.6.0 \(the latest release\)" "$R4" update --check --latest
hasnt "…and says no more about newer ones" "is newer" "$R4" update --check --latest
has   "the latest binary is up to date" "^yass is up to date \(0.6.0\)$" "$R6" update --check
run_ok "…and exits 0" "$R6" update --check
cd "$W"
has   "outside a repo: no repo line, and the latest" "would install 0.6.0 \(the latest release\)" "$R4" update --check
hasnt "…no repo line" "this repo" "$R4" update --check
cd "$W/e2e/update"
has   "a go install build isn't replaced, and gets its command" "won't replace this binary: it was built with go install; .*go install github.com/donjaime/yass/cmd/yass@v0.5.0" "$(vbin 0.4.0)" update --check
has   "a clone build gets how to rebuild" "built from a clone; update it there" "$(vbin dev)" update --check   # not y: on a tagged commit, a clone build carries the plain tag
run_fail "an older --version is refused" "$R6" update --check --version v0.4.0
has   "…and says why" "never downgrades" "$R6" update --check --version v0.4.0
{ kill "$RELPID"; wait "$RELPID"; } 2>/dev/null; RELPID=""
run_fail "unreachable releases: it says so" "$R4" update --check
has   "…naming where it looked" "couldn't reach http://127.0.0.1" "$R4" update --check
unset YASS_RELEASES_URL

echo "36. yass update installs"
GO_OS=$(cd "$ROOT" && go env GOOS); GO_ARCH=$(cd "$ROOT" && go env GOARCH); ASSET="yass_${GO_OS}_${GO_ARCH}.tar.gz"
rm -rf "$W/releases"
# mkrel <version> <binary> [ok|bad|unlisted]: a fake release, laid out as GitHub serves one
mkrel() { local d="$W/releases/v$1" sum; mkdir -p "$d/x/yass_${GO_OS}_${GO_ARCH}"
  cp "$2" "$d/x/yass_${GO_OS}_${GO_ARCH}/yass"; tar -czf "$d/$ASSET" -C "$d/x" "yass_${GO_OS}_${GO_ARCH}"; rm -rf "$d/x"
  sum=$( (sha256sum "$d/$ASSET" 2>/dev/null || shasum -a 256 "$d/$ASSET") | cut -d' ' -f1)
  case "${3:-ok}" in ok) echo "$sum  $ASSET";; bad) echo "0000000000000000000000000000000000000000000000000000000000000000  $ASSET";; unlisted) echo "$sum  other.tar.gz";; esac > "$d/checksums.txt"; }
mkrel 0.5.0 "$(rbin 0.5.0)"; mkrel 0.6.0 "$R6"
printf '#!/bin/sh\necho "yass 9.9.9"\n' > "$W/liar"; chmod +x "$W/liar"; mkrel 0.7.0 "$W/liar"
mkrel 0.8.0 "$(rbin 0.8.0)" bad; mkrel 0.9.0 "$(rbin 0.9.0)" unlisted
# PATHs with git and nothing else, and with a fake gh: auth status and attestation verify exit as told
mkpath() { local d="$W/path-$1"; rm -rf "$d"; mkdir -p "$d"; ln -s "$(command -v git)" "$d/git"
  [ -n "${2:-}" ] && { printf '#!/bin/sh\ncase "$1" in auth) exit %s;; attestation) [ %s = 0 ] && exit 0; echo "verification failed: no attestation"; exit 1;; esac\n' "$2" "$3" > "$d/gh"; chmod +x "$d/gh"; }
  echo "$d"; }
NOGH=$(mkpath nogh); GHOK=$(mkpath ghok 0 0); GHBAD=$(mkpath ghbad 0 1); GHOUT=$(mkpath ghout 1 0)
fresh() { rm -rf "$W/inst"; mkdir -p "$W/inst"; cp "${1:-$R4}" "$W/inst/yass"; echo "$W/inst/yass"; }
upd() { env PATH="$1" "$W/inst/yass" "${@:2}"; }
cd "$W/e2e/update"; releases 0.6.0
fresh >/dev/null; OUT="$(upd "$NOGH" update 2>&1)"
has   "in a repo ahead of it, update installs the repo's version" "^installed yass 0.5.0 " echo "$OUT"
has   "…which runs" "^yass 0.5.0$" "$W/inst/yass" version
has   "…after the checksum; provenance unchecked without gh, and why" "checksum verified; build provenance not checked: the GitHub CLI isn't installed" echo "$OUT"
has   "…and says a newer one exists, and what follows" "0.6.0 is newer: .yass update --latest. gets it; then run .yass upgrade. in each repo and commit the diff" echo "$OUT"
hasnt "…leaving nothing beside the binary" "yass-new|\.old" ls -A "$W/inst"
fresh >/dev/null; OUT="$(upd "$NOGH" update --latest 2>&1)"
has   "--latest installs the latest" "^installed yass 0.6.0 " echo "$OUT"
hasnt "…and says nothing about newer ones" "is newer" echo "$OUT"
fresh >/dev/null
has   "--version installs that one" "^installed yass 0.5.0 " upd "$NOGH" update --version v0.5.0
fresh "$R6" >/dev/null
run_fail "an older --version is refused" upd "$NOGH" update --version v0.5.0
has   "…and the binary is unchanged" "^yass 0.6.0$" "$W/inst/yass" version
cd "$W"; fresh >/dev/null
has   "outside a repo, update installs the latest" "^installed yass 0.6.0 " upd "$NOGH" update
cd "$W/e2e/update"; fresh >/dev/null
run_fail "a checksum that doesn't match: refused" upd "$NOGH" update --version v0.8.0
has   "…and says so" "doesn't match the release's checksums.txt" upd "$NOGH" update --version v0.8.0
has   "…the binary unchanged" "^yass 0.4.0$" "$W/inst/yass" version
has   "an archive checksums.txt doesn't list: refused" "isn't listed in the release's checksums.txt" upd "$NOGH" update --version v0.9.0
has   "with gh, provenance is verified" "checksum verified, build provenance verified" upd "$GHOK" update
fresh >/dev/null
run_fail "provenance that doesn't verify: refused" upd "$GHBAD" update
has   "…and says why" "not installing 0.5.0: its build provenance didn't verify" upd "$GHBAD" update
has   "…with what gh said" "verification failed: no attestation" upd "$GHBAD" update
has   "…the binary unchanged" "^yass 0.4.0$" "$W/inst/yass" version
has   "gh not signed in: installs, and says provenance wasn't checked" "not checked: the GitHub CLI isn't signed in" upd "$GHOUT" update
fresh >/dev/null
run_fail "--require-provenance without gh: refused" upd "$NOGH" update --require-provenance
has   "…and says why" "provenance can't be checked: the GitHub CLI isn't installed" upd "$NOGH" update --require-provenance
run_fail "a downloaded binary that isn't the version asked for: refused" upd "$NOGH" update --version v0.7.0
has   "…and says what it said" "doesn't run as 0.7.0 here \(it said: yass 9.9.9\)" upd "$NOGH" update --version v0.7.0
has   "…the binary unchanged" "^yass 0.4.0$" "$W/inst/yass" version
hasnt "…and nothing left beside it" "yass-new" ls -A "$W/inst"
chmod a-w "$W/inst"
run_fail "a folder it can't write: refused before downloading" upd "$NOGH" update
has   "…saying where, and what to do" "can't write to .*inst, where this yass is; rerun with permission" upd "$NOGH" update
chmod u+w "$W/inst"
run_fail "a release that doesn't exist: refused" upd "$NOGH" update --version v0.42.0
has   "…naming it" "there's no release v0.42.0" upd "$NOGH" update --version v0.42.0
fresh >/dev/null
has   "installing past the repo says to upgrade it next, and that teammates follow" "^next: this repo's YASS files are older \(0.5.0\); run .yass upgrade. here and commit the diff \(chore: upgrade YASS\), and teammates then need yass 0.6.0 too" upd "$NOGH" update --latest
fresh >/dev/null
hasnt "…matching the repo, there's nothing next" "^next:" upd "$NOGH" update
fresh >/dev/null
{ kill "$RELPID"; wait "$RELPID"; } 2>/dev/null; RELPID=""
run_fail "releases unreachable: refused" upd "$NOGH" update
has   "…the binary unchanged" "^yass 0.4.0$" "$W/inst/yass" version
unset YASS_RELEASES_URL

echo "37. yass-update, the playbook"
newrepo "$W/e2e/update-kit"; "$(vbin 0.5.0)" init --claude >/dev/null
[ -f .agents/skills/yass-update/SKILL.md ] && [ -f .claude/skills/yass-update/SKILL.md ] && ok "init installs yass-update with the others" || bad "no yass-update playbook"
has   "the AGENTS.md section points to it" "out of step, or someone asks to update YASS: .yass-update." cat AGENTS.md
has   "…and so does yass-status" "Versions out of step.*yass-update" cat .agents/skills/yass-status/SKILL.md
git add -A; git commit -q -m "chore: adopt YASS"; rm -rf .agents/skills/yass-update .claude/skills/yass-update
has   "upgrade adds it to a repo that doesn't have it" "wrote .agents/skills/yass-update/SKILL.md" "$(vbin 0.5.0)" upgrade
[ -f .claude/skills/yass-update/SKILL.md ] && ok "…in .claude/skills too" || bad "not added to .claude/skills"

echo "38. criteria the pieces deliver"
newrepo "$W/e2e/delivers"; y init --hooks >/dev/null; git add -A; git commit -q -m "chore: adopt YASS"; BASE=$(git rev-parse HEAD)
L=$(y new "Three pieces" --large)
printf '### M1\n- [ ] AC1 (R1) Given two pieces — verify: e2e\n- [ ] AC2 (R1) Given one — verify: e2e\n- [ ] AC3 (R2) Given none — verify: e2e\n- [ ] AC4 (R2) Given part — verify: e2e\n' >> "$L/plan.md"
P1=$(y new "Piece one" --in three-pieces); P2=$(y new "Piece two" --in three-pieces); P3=$(y new "Piece three" --in three-pieces)
sub "$P1/change.md" '^- \[ \] $' '- [ ] Delivers AC1 in [plan.md](../plan.md)\n- [ ] Work one'
sub "$P2/change.md" '^- \[ \] $' '- [ ] Delivers AC1–AC2 in [plan.md](../plan.md)\n- [ ] Work two'
sub "$P3/change.md" '^- \[ \] $' '- [ ] Delivers AC4'"'"'s other half\n- [ ] Work three'
git add -A; git commit -q -m "yass: plan three-pieces"
# land <piece dir> <n>: tick the piece's boxes with code on a branch, squash-merge it into main
land() { git switch -q -c "piece-$2"; sub "$1/change.md" '^- \[ \] ' '- [x] '; echo "code $2" >> app.txt; git add -A; git commit -q -m "feat: piece $2"
  git switch -q main; git merge -q --squash "piece-$2" >/dev/null; git commit -q -m "feat: piece $2 (#$2)"; }
land "$P1" 1
has   "one of two delivering pieces done: the criterion stays open" "AC1 \(R1\) Given two pieces" y status three-pieces
land "$P2" 2
hasnt "both pieces done: the criterion counts as done" "\[ \] AC1 \(R1\)" bash -c 'yass status three-pieces | sed -n "/^open:/,\$p"'
hasnt "…and isn't offered as next" "next: AC1" y status three-pieces
hasnt "a criterion delivered by one done piece counts too" "AC2 \(R1\) Given one" bash -c 'yass status three-pieces | sed -n "/^open:/,\$p"'
has   "a criterion no piece delivers counts by its own box" "AC3 \(R2\) Given none" bash -c 'yass status three-pieces | sed -n "/^open:/,\$p"'
has   "…and the parent's progress counts what's delivered" "three-pieces  \(large\) +6/10" y status
land "$P3" 3
sub "$L/plan.md" '^- \[ \] AC3 ' '- [x] AC3 '; git commit -q -am "yass: AC3 checked"
cp "$L/plan.md" "$W/plan.orig"
has   "with every piece done and AC3 ticked, the change is done" "three-pieces  \(large\) +10/10 +done" y status
run_ok "archive doesn't refuse for criteria its pieces delivered" y archive three-pieces
A="yass/archive/$MONTH/$TODAY-three-pieces/plan.md"
has   "…and ticks them in the archived plan.md" "^- \[x\] AC1 .*\n?" grep -E "^- \[x\] AC(1|2|4) " "$A"
has   "…and nothing else changes in it" "^AC1,AC1,AC2,AC2,AC4,AC4$" bash -c "diff '$W/plan.orig' '$A' | grep -E '^[<>]' | grep -oE 'AC[0-9]+' | sort | paste -sd, -"
hasnt "the archive commit passes the hook" "heads-up|append-only" git commit -q -m "yass: archive three-pieces"
hasnt "main's whole history passes the hook" "." yass hook --range "$BASE..HEAD"
[ "$(git log --oneline "$BASE..HEAD" | grep -c "close\|done")" -eq 0 ] && ok "no closing commit per piece" || bad "a closing commit snuck in"

echo "39. sparse checkouts and assets"
newrepo "$W/e2e/sparse-full"; y init >/dev/null; y init services/search --no-agents >/dev/null; y init services/web --no-agents >/dev/null
mkb services/search/yass/changes/2026-01-02-search-thing; mkb services/search/yass/archive/2025/12/2025-12-01-search-old x
mkb services/web/yass/changes/2026-01-01-web-thing / "2026-01-02-search-thing"
sub services/web/yass/changes/2026-01-01-web-thing/change.md '^blocked:' 'follows: 2025-12-01-search-old\nblocked:'
git add -A; git commit -q -m "chore: teams"
SP="$W/e2e/sparse"; rm -rf "$SP"; git clone -q --no-checkout "file://$PWD" "$SP"; cd "$SP"
git sparse-checkout set --cone services/web; git checkout -q main
[ ! -e services/search ] && ok "(set up: a cone-mode sparse checkout without services/search/)" || bad "setup: search is checked out"
has   "status lists the yass folder that's checked out" "2026-01-01-web-thing" y status
hasnt "…and says nothing of the one that isn't" "services/search/yass/ *$|search-thing +[0-9]" y status
run_ok "…and --strict passes" y status --strict
has   "a blocked: outside the checkout says so, not that there's no such change" "blocked: '2026-01-02-search-thing' is in services/search/yass/, outside this sparse checkout, so it can't be checked" y status
has   "…as does a follows:" "follows '2025-12-01-search-old', which is in services/search/yass/, outside this sparse checkout" y status
hasnt "…and neither is a warning" "warning" y status
cd "$W/e2e/sparse-full"
mkb services/web/yass/changes/2026-01-03-typo / "2026-09-09-nope"
has   "a name that really doesn't exist still warns" "warning: .*blocked: '2026-09-09-nope' looks like a change, but there's no such change" y status
rm -rf services/web/yass/changes/2026-01-03-typo
N=$(y new "With a mockup" --large); mkdir -p "$N/assets"; head -c 2048 /dev/urandom > "$N/assets/mockup.png"
sub "$N/plan.md" '^## Validation' '- [x] AC1 (R1) Given a mockup — verify: e2e\n\n## Validation'
git add -A; git commit -q -m "yass: with a mockup"; y archive with-a-mockup --force >/dev/null
[ -f "yass/archive/$MONTH/$TODAY-with-a-mockup/assets/mockup.png" ] && ok "an asset in a change folder moves with it into the archive" || bad "asset left behind"

echo "40. pieces own their files"
newrepo "$W/e2e/own"; y init --hooks >/dev/null; git add -A; git commit -q -m "chore: adopt YASS"
L=$(y new "Parallel" --large); printf '### M1\n- [ ] AC1 (R1) Given A — verify: e2e\n- [ ] AC2 (R1) Given B — verify: e2e\n' >> "$L/plan.md"
PA=$(y new "Piece a" --in parallel); PB=$(y new "Piece b" --in parallel)
sub "$PA/change.md" '^- \[ \] $' '- [ ] Delivers AC1\n- [ ] Work a'; sub "$PB/change.md" '^- \[ \] $' '- [ ] Delivers AC2\n- [ ] Work b'
git add -A; git commit -q -m "yass: plan parallel"
git switch -q -c a; sub "$PA/change.md" '^- \[ \] Work a' '- [x] Work a'; printf '### %s (sam)\n- Did: a\n' "$TODAY" >> "$PA/change.md"; echo a >> a.txt; git add -A
hasnt "code with progress in one piece: no heads-up" "heads-up" git commit -q -m "feat: a"
git switch -q main; git switch -q -c b; sub "$PB/change.md" '^- \[ \] Work b' '- [x] Work b'; printf '### %s (kim)\n- Did: b\n' "$TODAY" >> "$PB/change.md"; echo b >> b.txt; git add -A; git commit -q -m "feat: b"
run_ok "branches on different pieces rebase onto each other without conflicts" git rebase -q a
has   "…and both pieces' progress survives" "Work a.*Work b|Work b.*Work a" bash -c "grep -h '\[x\] Work' '$PA/change.md' '$PB/change.md' | tr '\n' ' '"
sub "$L/plan.md" '^- \[ \] AC1 ' '- [x] AC1 '; echo c >> c.txt; git add -A
has   "code plus a tick in the parent's plan.md: a heads-up naming it" "parallel/plan.md: a large change's own files change in commits without code" git commit -q -m "feat: c"
git reset -q --hard HEAD~1
printf '### %s (sam)\n- Did: parent note\n' "$TODAY" >> "$L/change.md"; echo d >> d.txt; git add -A
has   "code plus a Log entry in the parent's change.md: a heads-up" "parallel/change.md: a large change's own files" git commit -q -m "feat: d"
git reset -q --hard HEAD~1
echo "- [/] more" >> "$PA/change.md"; echo "- [/] more" >> "$PB/change.md"; echo e >> e.txt; git add -A
has   "code plus progress in two pieces: a heads-up naming both" "this commit with code changes pieces $TODAY-piece-a and $TODAY-piece-b" git commit -q -m "feat: e"
git reset -q --hard HEAD~1
sub "$L/plan.md" '^- \[ \] AC1 ' '- [x] AC1 '; printf '### %s (sam)\n- Did: closing note\n' "$TODAY" >> "$L/change.md"; git add -A
hasnt "the parent's boxes and Log in a commit without code: no heads-up" "heads-up" git commit -q -m "yass: parallel progress"
mkdir -p "$L/assets"; head -c 64 /dev/urandom > "$L/assets/sketch.png"; git add -A; git commit -q -m "yass: sketch"
head -c 64 /dev/urandom > "$L/assets/sketch.png"; echo f >> f.txt; git add -A
has   "a parent's subfolder that isn't a piece counts as the parent" "assets/sketch.png: a large change's own files" git commit -q -m "feat: f"
git reset -q --hard HEAD~1

echo "41. the branch view"
O="$W/e2e/bv-origin"; newrepo "$O"; git config uploadpack.allowfilter true; y init >/dev/null
L=$(y new "Shared" --large); A=$(y new "Piece a" --in shared); B=$(y new "Piece b" --in shared)
sub "$A/change.md" '^- \[ \] $' '- [ ] Delivers AC1\n- [ ] Work a'; sub "$B/change.md" '^- \[ \] $' '- [ ] Delivers AC2\n- [ ] Work b'
git add -A; git commit -q -m "yass: shared"
git switch -q -c feat-b; sub "$B/change.md" '^- \[ \] Work b' '- [x] Work b'; printf '### %s (kim)\n- Next: polish b\n' "$TODAY" >> "$B/change.md"
git -c user.name=Kim commit -qam "feat: b"; git switch -q main
C1="$W/e2e/bv"; rm -rf "$C1"; git clone -q "file://$O" "$C1"; cd "$C1"; git config user.email t@t; git config user.name Sam
git switch -q -c feat-a; sub "$A/change.md" '^- \[ \] Work a' '- [x] Work a'; printf '### %s (sam)\n- Next: test a\n' "$TODAY" >> "$A/change.md"
git commit -qam "feat: a"; git switch -q main
git switch -q -c code-only; echo x >> app.txt; git commit -qam "feat: code only"; git switch -q main
git switch -q -c merged-one; echo "- [/] merged" >> "$A/change.md"; git commit -qam "wip"; git switch -q main
git -C "$O" fetch -q "$C1" merged-one:merged-one; git -C "$O" merge -q --ff-only merged-one; git fetch -q   # merged on the origin's main
git switch -q -c old-one; echo "- [/] old" >> "$B/change.md"; git add -A
D40=$(date -v-40d +%Y-%m-%dT12:00:00 2>/dev/null || date -d '40 days ago' +%Y-%m-%dT12:00:00); GIT_AUTHOR_DATE="$D40" GIT_COMMITTER_DATE="$D40" git commit -q -m "old"; git switch -q main
git switch -q -c ten-days; echo "- [/] ten" >> "$L/change.md"; git add -A
D10=$(date -v-10d +%Y-%m-%dT12:00:00 2>/dev/null || date -d '10 days ago' +%Y-%m-%dT12:00:00); GIT_AUTHOR_DATE="$D10" GIT_COMMITTER_DATE="$D10" git commit -q -m "ten"; git switch -q main
OUT="$(y status)"
hasnt "a Next: inside the template's comment isn't taken for one" "next: … -->" echo "$OUT"
has   "a local branch shows under its piece, with author, progress and next" "↳ feat-a  Sam, .*  1/2  next: test a" echo "$OUT"
has   "a pushed branch too" "↳ origin/feat-b  Kim, .*  1/2  next: polish b" echo "$OUT"
hasnt "a branch that only changes code isn't listed" "code-only" echo "$OUT"
hasnt "nor one already merged" "merged-one" echo "$OUT"
hasnt "nor one with nothing in the last 30 days" "old-one" echo "$OUT"
has   "a branch on the parent shows under the parent" "↳ ten-days  Sam" echo "$OUT"
has   "it says how fresh the remote branches are" "^branches: unmerged work since .*; remote branches as of the last fetch" echo "$OUT"
hasnt "--since 7d leaves out a branch from 10 days ago" "ten-days" y status --since 7d
has   "…--since an earlier date takes it in" "ten-days" y status --since "$(date -v-20d +%F 2>/dev/null || date -d '20 days ago' +%F)"
hasnt "--no-branches shows none" "↳|^branches:" y status --no-branches
hasnt "…and looks none up" "for-each-ref" bash -c 'GIT_TRACE=1 yass status --no-branches 2>&1'
hasnt "the branch view never fetches" "fetch|remote-https|upload-pack" bash -c 'GIT_TRACE=1 yass status 2>&1 | grep "trace: \(built-in\|exec\|run_command\)"'
P2="$W/e2e/bv-partial"; rm -rf "$P2"; git clone -q --filter=blob:none "file://$O" "$P2"; cd "$P2"
has   "a blobless clone shows a branch whose files aren't local, without progress" "↳ origin/feat-b  Kim, .*\(its files aren't local\)" y status
hasnt "…and downloads nothing to do it" "fetch|upload-pack" bash -c 'GIT_TRACE=1 yass status 2>&1 | grep "trace: \(built-in\|exec\|run_command\)"'
NR="$W/e2e/bv-noremote"; newrepo "$NR"; y init >/dev/null; N=$(y new "Lonely"); git add -A; git commit -q -m "yass: lonely"
hasnt "no unmerged branches: status looks as it always did" "↳|^branches:" y status
git switch -q -c local-only; sub "$N/change.md" '^- \[ \] $' '- [/] Started'; git commit -qam "start"; git switch -q main
has   "with no remote, it says there are only local branches" "^branches: .*no remote branches here" y status

echo "42. inline or separate plans"
newrepo "$W/e2e/inline"; y init >/dev/null; git add -A; git commit -q -m "chore: adopt YASS"
hasnt "plans in the repo: status says nothing more" "^plans:" y status
has   "…and root -v says inline" "/yass	inline$" y root -v
git worktree add -q "$W/e2e/inline-wt" -b wt 2>/dev/null
has   "from a linked worktree too" "	inline$" bash -c "cd '$W/e2e/inline-wt' && yass root -v"
PO="$W/e2e/plans-origin"; newrepo "$PO"; mkdir -p archive changes; touch changes/.gitkeep; git add -A; git commit -q -m plans
PR="$W/e2e/plans-clone"; rm -rf "$PR"; git clone -q "file://$PO" "$PR"
newrepo "$W/e2e/sep"; printf 'path: %s\n' "$PR" > yass.yaml; git add -A; git commit -q -m "chore: adopt YASS"
has   "plans in another repo: status ends saying so" "^plans: separate \(.*plans-clone\)$" bash -c 'yass status | tail -1'
has   "…root -v says separate" "plans-clone	separate$" y root -v
has   "…and plain root prints the folder alone" "plans-clone$" y root
echo "draft" > "$PR/changes/notes.md"
has   "uncommitted files in the plans repo get a note" "plans-clone has 1 uncommitted file\(s\)" y status
run_ok "…and --strict still passes" y status --strict
rm "$PR/changes/notes.md"
(cd "$PO" && echo more > more.txt && git add -A && git commit -q -m more); git -C "$PR" fetch -q
has   "a plans repo behind its upstream gets a note, and how to catch up" "plans-clone is 1 commit\(s\) behind origin/main as of its last fetch; pull there \(git -C .*plans-clone pull\)" y status
hasnt "…without fetching" "fetch" bash -c 'GIT_TRACE=1 yass status 2>&1 | grep "trace: \(built-in\|exec\|run_command\)"'
PN="$W/e2e/plans-plain"; rm -rf "$PN"; mkdir -p "$PN/changes" "$PN/archive"
newrepo "$W/e2e/sep-plain"; printf 'path: %s\n' "$PN" > yass.yaml; git add -A; git commit -q -m "chore: adopt YASS"
has   "plans in no repo are separate too" "^plans: separate \(.*plans-plain\)$" bash -c 'yass status | tail -1'
hasnt "…with nothing to say about commits" "uncommitted|behind" y status

echo "43. citations: one repo, plain hashes"
newrepo "$W/e2e/cites"; y init >/dev/null; git add -A; git commit -q -m "chore: adopt YASS"
git switch -q -c feature; echo f >> app.txt; git commit -qam "feat: unmerged"; U7=$(git rev-parse --short=7 HEAD); git switch -q main
H7=$(git rev-parse --short=7 HEAD)
mkdir -p yass/changes/2026-01-02-cited; printf '# Cited\n\n## Steps\n- [x] Qualified — code: web@%s\n- [x] Thing — code: %s\n- [x] Gone — code: 0badc0d\n' "$H7" "$U7" > yass/changes/2026-01-02-cited/change.md
git add -A; git commit -q -m "yass: cited"
hasnt "a repo-qualified token isn't a citation: no check, no warning" "Qualified" y status
hasnt "…and --cites doesn't find it" "Qualified" y decisions --cites "$H7"
printf 'repo: web\nrepos:\n  web: ../web\n' > yass.yaml
has   "repo: is an unknown setting now" "unknown setting 'repo'" y status
has   "…and so is repos:" "unknown setting 'repos'" y status
rm yass.yaml
OUT="$(y archive cited 2>&1)"
has   "archive archives a change whose done boxes cite unmerged or missing commits" "^archived .*2026-01-02-cited" echo "$OUT"
has   "…warning about the unmerged one" "^warning: 'Thing' \(2026-01-02-cited/change.md\) cites $U7, which isn't on main yet" echo "$OUT"
has   "…and the missing one" "^warning: 'Gone' \(2026-01-02-cited/change.md\) cites 0badc0d, which isn't a commit here" echo "$OUT"
git commit -q -m "yass: archive cited"

echo "44. one line of progress, for commits and pull requests"
newrepo "$W/e2e/brief"; y init >/dev/null
L=$(y new "Offline sync" --large); printf '### M1\n- [ ] AC1 (R1) Given a — verify: e2e\n' >> "$L/plan.md"
Q=$(y new "Offline queue" --in offline-sync); B=$(y new "Sync badge" --in offline-sync)
sub "$Q/change.md" '^- \[ \] $' '- [x] Delivers AC1\n- [x] Queue table'; sub "$B/change.md" '^- \[ \] $' '- [/] Badge'
S=$(y new "Fix double tap"); sub "$S/change.md" '^- \[ \] $' '- [x] Disable Save\n- [ ] Unit test'
has   "a piece: its parent's pieces and boxes, then its own" "^offline-sync: 1/2 pieces done, 3/4 boxes · offline-queue: 2/2 boxes, done$" y status offline-queue --brief
has   "a large change: its pieces and boxes" "^offline-sync: 1/2 pieces done, 3/4 boxes$" y status offline-sync --brief
has   "a small change: its boxes" "^fix-double-tap: 1/2 boxes$" y status double-tap --brief
sub "$S/change.md" '^- \[ \] Unit test' '- [x] Unit test'
has   "…and done when it is" "^fix-double-tap: 2/2 boxes, done$" y status double-tap --brief
run_fail "--brief needs a change" y status --brief
has   "…and says so" "--brief needs a change" y status --brief

echo "45. yass: trailers, with plans kept apart"
PL="$W/e2e/trail-plans"; newrepo "$PL"; mkdir -p changes archive; touch changes/.gitkeep
mkt() { mkdir -p "$PL/$1"; printf '# %s\n\n- [ ] Do it\n' "$(basename "$1")" > "$PL/$1/change.md"; }
mkt changes/2026-02-01-login; mkt changes/2026-02-01-login/2026-02-02-form; mkt changes/2026-02-01-login/2026-02-03-api
mkt changes/2026-02-05-signup; mkt changes/2026-02-05-signup/2026-02-02-form; mkt archive/2026-01-01-old
git add -A; git commit -q -m plans
CODE="$W/e2e/trail-code"; newrepo "$CODE"; y init --path "$PL" --hooks >/dev/null; git add -A; git commit -q -m "chore: adopt YASS" 2>/dev/null
[ -x tools/yass/githooks/prepare-commit-msg ] && [ -x tools/yass/githooks/commit-msg ] && ok "init installs the trailer hooks beside pre-commit" || bad "trailer hooks missing"
trailer() { git log -1 --format='%(trailers:key=yass,valueonly)' | head -1; }
c() { echo "$RANDOM" >> code.txt; git add code.txt; git commit -q "$@" 2>&1; }
hasnt "no change to name: the commit goes through, and nothing asks for a trailer" "." c -m "feat: nameless"
[ -z "$(trailer)" ] && ok "…and nothing is added" || bad "a trailer was invented"
y use login >/dev/null
has   "yass use names this worktree's change" "^2026-02-01-login$" y use
hasnt "with it, commits go through quietly" "." c -m "feat: with use"
has   "…carrying the trailer, in lower case" "^2026-02-01-login$" trailer
git worktree add -q "$W/e2e/trail-wt" -b other 2>/dev/null
(cd "$W/e2e/trail-wt" && yass use SIGNUP >/dev/null && echo x >> wt.txt && git add wt.txt && git commit -q -m "feat: other worktree")
has   "another worktree uses its own change" "^2026-02-05-signup$" git -C "$W/e2e/trail-wt" log -1 --format='%(trailers:key=yass,valueonly)'
has   "…while this one keeps its" "^2026-02-01-login$" bash -c 'echo y >> code.txt; git add code.txt; git commit -q -m "feat: again"; git log -1 --format="%(trailers:key=yass,valueonly)" | head -1'
y use --clear >/dev/null; has "--clear clears it" "none set" y use
git switch -q -c jaime/2026-02-01-login
c -m "feat: on a branch" >/dev/null; has "a branch named after a change (after a prefix) gives the trailer" "^2026-02-01-login$" trailer
git switch -q main; git switch -q -c 2026-02-03-api
c -m "feat: piece branch" >/dev/null; has "…a piece's name gives <change>/<piece>" "^2026-02-01-login/2026-02-03-api$" trailer
git switch -q main; git switch -q -c feature-x
c -m "feat: first" -m "yass: 2026-02-05-signup" >/dev/null; c -m "feat: second" >/dev/null
has   "otherwise an earlier commit's trailer on the branch carries over" "^2026-02-05-signup$" trailer
c -m "feat: mine" -m "yass: none" >/dev/null; has "a trailer you wrote is left alone" "^none$" trailer
git switch -q main; git merge -q --no-ff feature-x -m "Merge feature-x" 2>/dev/null
[ -z "$(git log -1 --format='%(trailers:key=yass,valueonly)')" ] && ok "a merge commit gets none" || bad "a merge got a trailer"
for t in "2026-02-01-login" "2026-01-01-old" "2026-02-01-login/2026-02-02-form" "2026-02-03-api" "none" "2026-02-01-LOGIN ac:ac1,AC2"; do
  hasnt "\`yass: $t\` is accepted" "heads-up|refusing" c -m "feat: t" -m "yass: $t"; done
N0=$(git rev-list --count HEAD)
has   "a malformed trailer gets a heads-up" "is malformed" c -m "feat: bad" -m "yass: 2026-02-01-login ac"
[ "$(git rev-list --count HEAD)" -gt "$N0" ] && ok "…and the commit is made" || bad "a commit with a bad trailer was refused"
has   "a piece name in two changes is ambiguous" "pieces in more than one change; give it as <change>/<piece>" c -m "feat: amb" -m "yass: 2026-02-02-form"
has   "an unknown key is only a warning" "doesn't know the key 'foo'" c -m "feat: k" -m "yass: 2026-02-01-login foo:bar"
hasnt "…not a heads-up" "heads-up" c -m "feat: k2" -m "yass: 2026-02-01-login foo:bar"
has   "a name that isn't a change gets a heads-up, suggesting the closest" "'2026-02-01-logn' isn't a change or piece; did you mean 2026-02-01-login" c -m "feat: typo" -m "yass: 2026-02-01-logn"
printf 'path: %s\ntrailers: require\n' "$PL" > yass.yaml
has   "trailers: is an unknown setting now" "unknown setting 'trailers'" y status
git checkout -q yass.yaml
BASE=$(git rev-parse HEAD); c -m "feat: unnamed" >/dev/null; c -m "feat: named" -m "yass: 2026-02-01-login" >/dev/null
c -m "feat: wrongly named" -m "yass: 2026-09-09-nope" >/dev/null
hasnt "hook --range leaves a commit without a trailer alone" "feat: unnamed" yass hook --range "$BASE..HEAD"
has   "…but reports one naming no change" "feat: wrongly named: .*isn't a change or piece" yass hook --range "$BASE..HEAD"
has   "--require-trailers reports a commit without one" "feat: unnamed: no \`yass:\` trailer" yass hook --range "$BASE..HEAD" --require-trailers
run_ok "…as a warning" yass hook --range "$BASE..HEAD" --require-trailers
run_fail "…and fails with --strict" yass hook --range "$BASE..HEAD" --require-trailers --strict
B2=$(git rev-parse HEAD); c -m "Login form (#12)" -m "* feat: a" -m "yass: 2026-02-01-login ac:ac1" >/dev/null
run_ok "a squash commit carrying the trailers passes" yass hook --range "$B2..HEAD" --strict
CI="$W/e2e/trail-ci"; rm -rf "$CI"; git clone -q "file://$CODE" "$CI"; cd "$CI"; printf 'path: %s/nowhere\n' "$W" > yass.yaml
has   "in CI without the plans, only the form is checked, and it says so" "change names weren't checked" yass hook --range "$BASE..HEAD"
has   "…still reporting a missing trailer when asked" "feat: unnamed: no \`yass:\` trailer" yass hook --range "$BASE..HEAD" --require-trailers
MX="$W/e2e/trail-mixed"; newrepo "$MX"; y init --hooks >/dev/null; mkdir -p team; printf 'path: %s\n' "$PL" > team/yass.yaml
mkdir -p yass/changes/2026-06-01-local; printf '# Local\n\n- [ ] Do it\n' > yass/changes/2026-06-01-local/change.md
git add -A; git commit -q -m "chore: adopt YASS"
y use 2026-02-01-login >/dev/null
c -m "feat: for the separate plans" >/dev/null; has "a repo with both kinds of plans: a change kept outside gets its trailer" "^2026-02-01-login$" trailer
has   "yass use on a change in this repo says its commits carry no trailer" "2026-06-01-local's plans are in this repo, so its commits don't carry a trailer" y use 2026-06-01-local
c -m "feat: for the local plans" >/dev/null; [ -z "$(trailer)" ] && ok "…and none is added" || bad "an inline change got a trailer"
y use --clear >/dev/null
newrepo "$W/e2e/trail-inline"; y init --hooks >/dev/null; git add -A; git commit -q -m "chore: adopt YASS"
rm tools/yass/githooks/prepare-commit-msg tools/yass/githooks/commit-msg; git add -A; git commit -q -m "older install: pre-commit only"
sub tools/yass/githooks/pre-commit '^# yass-version: .*$' '# yass-version: 0.1.0'
has   "yass upgrade adds the trailer hooks to an older install" "wrote tools/yass/githooks/commit-msg" "$(vbin 0.9.0)" upgrade
[ -x tools/yass/githooks/prepare-commit-msg ] && ok "…executable" || bad "prepare-commit-msg missing or not executable"
git checkout -q -- . 2>/dev/null; git clean -qfd tools 2>/dev/null
y use --clear >/dev/null 2>&1
hasnt "plans in the repo: nothing about trailers" "." bash -c 'echo a >> a.txt; git add a.txt; git commit -q -m "feat: inline" 2>&1'
[ -z "$(git log -1 --format='%(trailers:key=yass,valueonly)')" ] && ok "…and no trailer" || bad "inline commit got a trailer"

echo "46. a change's code, from its trailers"
P3="$W/e2e/code-plans"; newrepo "$P3"; mkdir -p changes/2026-04-01-cart/2026-04-02-ui archive
printf '# Cart\n\n## Log\n' > changes/2026-04-01-cart/change.md
printf '# Cart: plan\n\n## Acceptance\n- [ ] AC1 (R1) Given a cart — verify: e2e\n- [ ] AC3 (R2) Given a total — verify: e2e\n' > changes/2026-04-01-cart/plan.md
printf '# UI\n\n- [ ] Delivers AC3\n' > changes/2026-04-01-cart/2026-04-02-ui/change.md; git add -A; git commit -q -m plans
newrepo "$W/e2e/code-repo"; printf 'path: %s\n' "$P3" > yass.yaml; git add -A; git commit -q -m "chore: adopt YASS" -m "yass: none"
cc() { echo "$RANDOM" >> c.txt; git add c.txt; git commit -q -m "$1" -m "yass: $2"; }
cc "feat: cart model" "2026-04-01-cart ac:ac1"; M1=$(git rev-parse --short=7 HEAD)
cc "feat: cart ui" "2026-04-02-ui ac:ac1,ac3"; M2=$(git rev-parse --short=7 HEAD)
cc "chore: bump deps" "none"; cc "feat: odd one" "2026-04-01-cart ac:ac9"; M4=$(git rev-parse --short=7 HEAD)
git switch -q -c wip; cc "feat: not merged" "2026-04-01-cart"; git switch -q main
git switch -q -c pr; echo s >> s.txt; git add s.txt; git commit -q -m "wip"; git switch -q main
git merge -q --squash pr >/dev/null; git commit -q -m "Cart totals (#7)" -m "yass: 2026-04-01-cart ac:ac3"; M5=$(git rev-parse --short=7 HEAD)
OUT="$(y status 2026-04-01-cart)"
has   "status <change> lists its merged code, the pieces' included" "code on main \(from yass: trailers\)" echo "$OUT"
for m in "$M1 feat: cart model" "$M2 feat: cart ui" "$M5 Cart totals \(#7\)"; do has "…$m" "^  $m$" echo "$OUT"; done
hasnt "…not the unmerged, nor others" "not merged|bump deps" echo "$OUT"
has   "each criterion lists its commits" "^  AC1: $M2, $M1$" echo "$OUT"
has   "…AC3 once each" "^  AC3: $M5, $M2$" echo "$OUT"
has   "an ac: the plan doesn't have warns" "warning: $M4's trailer names AC9, which 2026-04-01-cart's plan doesn't have" echo "$OUT"
has   "a piece lists only its own" "^  $M2 feat: cart ui$" y status 2026-04-02-ui
hasnt "…not its change's" "cart model" y status 2026-04-02-ui

echo "48. the trailer hooks and an older yass"
newrepo "$W/e2e/old-yass"; y init --hooks >/dev/null; git add -A; git commit -q -m "chore: adopt YASS"
OLD="$W/old-yass-bin"; rm -rf "$OLD"; mkdir -p "$OLD"; ln -s "$(command -v git)" "$OLD/git"
printf '#!/bin/sh\ncase "$*" in\n  "hook --help") echo "usage: yass hook [--range A..B] [--strict]";;\n  hook) exit 0;;\n  *) echo "yass: unknown option" >&2; exit 2;;\nesac\n' > "$OLD/yass"; chmod +x "$OLD/yass"
echo x >> a.txt; git add a.txt
run_ok "with a yass from before trailers on PATH, commits still go through" env PATH="$OLD:/usr/bin:/bin" git commit -q -m "feat: older yass"
hasnt "…without its errors" "unknown option" env PATH="$OLD:/usr/bin:/bin" bash -c 'echo y >> a.txt; git add a.txt; git commit -q -m "feat: again" 2>&1'

echo "49. yass deinit takes YASS back out of a repo"
newrepo "$W/e2e/deinit"; printf '# Mine\n\nOur own notes.\n' > AGENTS.md
y init --claude --hooks >/dev/null; y init apps/web --agents --claude >/dev/null
mkdir -p .agents/skills/yass-mine .husky; printf -- '---\nname: yass-mine\n---\nnot YASS'"'"'s\n' > .agents/skills/yass-mine/SKILL.md
printf 'yass hook\n' > .husky/pre-commit; git add -A; git commit -q -m "chore: adopt YASS"; HEAD0=$(git rev-parse HEAD)
H="$W/e2e/deinit-home"; US="$W/e2e/deinit-user"; rm -rf "$H" "$US"; mkdir -p "$H" "$US"; cp -R .agents/skills/yass-work "$US/"
OUT="$(env HOME="$H" YASS_SKILLS_DIR="$US" yass deinit)"
for p in .agents/skills/yass-work .claude/skills/yass-work apps/web/.agents/skills/yass-plan tools/yass/githooks/pre-commit tools/yass/githooks/commit-msg; do
  [ ! -e "$p" ] && ok "removes $p" || bad "$p is still there"
done
has   "…listing what it removed" "^removed \.agents/skills/yass-work$" echo "$OUT"
[ ! -e tools ] && [ ! -e .claude ] && ok "…and the folders it leaves empty" || bad "empty folders left: $(ls -d tools .claude 2>/dev/null)"
[ -f .agents/skills/yass-mine/SKILL.md ] && ok "a yass-* skill that isn't YASS's stays" || bad "removed a skill that isn't YASS's"
has   "the root AGENTS.md keeps its own text" "Our own notes" cat AGENTS.md
hasnt "…without the YASS section" "yass:begin" cat AGENTS.md
has   "…and says so" "^removed the YASS section of AGENTS.md$" echo "$OUT"
has   "…and CLAUDE.md keeps its import, AGENTS.md being there" "^@AGENTS.md" cat CLAUDE.md
[ ! -e apps/web/AGENTS.md ] && [ ! -e apps/web/CLAUDE.md ] && ok "an AGENTS.md that was only YASS's goes, with the CLAUDE.md that imported it" || bad "apps/web's AGENTS.md or CLAUDE.md is still there"
hasnt "core.hooksPath is unset" "githooks" git config --get core.hooksPath
has   "…and says so" "unset core.hooksPath" echo "$OUT"
[ -d yass/changes ] && [ -d apps/web/yass ] && ok "the yass folders stay" || bad "a yass folder was removed"
has   "…and it says how to remove them, or keep them" "Kept your plans: .*yass/.*keep them, or remove them" echo "$OUT"
has   "…with the commands" "git rm -r -q yass" echo "$OUT"
has   "it says how to commit it" 'chore: remove YASS' echo "$OUT"
[ "$(git rev-parse HEAD)" = "$HEAD0" ] && ok "…and commits nothing" || bad "yass deinit committed"
[ -f "$US/yass-work/SKILL.md" ] && ok "the user folder's playbooks stay" || bad "removed the user folder's playbooks without --global"
has   "…with a note on --global" "yass deinit --global" echo "$OUT"
has   "a husky hook still running yass hook gets a note" "\.husky/pre-commit still runs .yass hook." echo "$OUT"
run_ok "--global removes the user folder's playbooks too" env HOME="$H" YASS_SKILLS_DIR="$US" yass deinit --global
[ ! -e "$US/yass-work" ] && ok "…gone" || bad "the user folder's playbook is still there"
has   "run again, there's nothing left to remove" "no YASS playbooks, AGENTS.md section or hook here" env HOME="$H" YASS_SKILLS_DIR="$US" yass deinit
y init >/dev/null
[ -f .agents/skills/yass-work/SKILL.md ] && [ -x tools/yass/githooks/pre-commit ] && grep -q "yass:begin" AGENTS.md && ok "yass init sets it up again" || bad "yass init didn't set the repo up again"

echo; echo "passed: $pass  failed: $fail"
[ "$fail" -eq 0 ]

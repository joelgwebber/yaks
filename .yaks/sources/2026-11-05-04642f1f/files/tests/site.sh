#!/usr/bin/env bash
# Checks the landing page in site/: install text matches the docs, links and anchors resolve,
# required sections and assets are there, nothing external loads, and the page stays small.
# With --deploy, also fails while any asset is still a placeholder (pages.yml runs it that way).
# Usage: tests/site.sh [--deploy] [--root DIR]    (DIR: a repo copy to check instead of this one)
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEPLOY=0; SELF=1
while [ $# -gt 0 ]; do
  case "$1" in
    --deploy) DEPLOY=1 ;;
    --root) ROOT="$(cd "$2" && pwd)"; SELF=0; shift ;;
    *) echo "usage: tests/site.sh [--deploy] [--root DIR]" >&2; exit 2 ;;
  esac
  shift
done
SITE="$ROOT/site"; PAGE="$SITE/index.html"
REPO=https://github.com/donjaime/yass; URL=https://donjaime.github.io/yass
pass=0; fail=0
ok()  { echo "  ok   $1"; pass=$((pass+1)); }
bad() { echo "  FAIL $1"; fail=$((fail+1)); }
[ -f "$PAGE" ] || { echo "  FAIL no $PAGE"; exit 1; }

# section ID: the HTML of <section id="ID">, up to its </section>
section() { ID="$1" perl -0777 -ne 'print $1 if /<section id="\Q$ENV{ID}\E"[^>]*>(.*?)<\/section>/s' "$PAGE"; }
# slug FILE: GitHub's heading anchors for a markdown file, one per line
slugs() { perl -ne 'if (/^#{1,6}\s+(.*?)\s*$/) { $h = lc $1; $h =~ s/[^\w\- ]//g; $h =~ s/ /-/g; print "$h\n" }' "$1"; }

echo "Sections"
for id in idea folder you-say install; do
  if grep -q "<section id=\"$id\"" "$PAGE"; then ok "#$id"; else bad "#$id is missing"; fi
done
if grep -q 'class="hero"' "$PAGE" && perl -0777 -ne 'exit !(/<header class="hero">.*?href="#install".*?<\/header>/s)' "$PAGE"; then
  ok "the hero links to #install"; else bad "the hero doesn't link to #install"; fi
folder="$(section folder)"
for f in change.md prd.md plan.md design.md archive/; do
  if grep -qF "$f" <<<"$folder"; then ok "#folder shows $f"; else bad "#folder doesn't show $f"; fi
done
install="$(section install)"
for spec in terminal:Terminal "source:From Source" "agent:Agent Prompt"; do
  id="install-${spec%%:*}"; name="${spec#*:}"
  if ID="$id" NAME="$name" perl -0777 -e '$_ = <STDIN>; exit !(/<div class="panel" id="\Q$ENV{ID}\E">\s*<h3>\Q$ENV{NAME}\E<\/h3>.*?<pre/s)' <<<"$install"; then
    ok "#$id has its heading and commands in the HTML"; else bad "#$id is missing, or has no <h3>$name</h3> and <pre>"; fi
  if grep -qF "aria-controls=\"$id\">$name</button>" <<<"$install"; then ok "the $name tab controls #$id"; else bad "no $name tab controlling #$id"; fi
done
if grep -q '<div class="tablist" hidden>' <<<"$install"; then ok "the tab bar is hidden without JavaScript"; else bad "the tab bar should start hidden, so the page works without JavaScript"; fi
yousay="$(section you-say)"
for s in yass-shape yass-plan yass-work yass-status yass-log yass-update; do
  if grep -qF "<code>$s</code>" <<<"$yousay"; then ok "#you-say has $s"; else bad "#you-say has no row for $s"; fi
done

echo "Category line and easter egg"
CAT="Lightweight project management for people and coding agents"
CLOSE="<strong><em>It's lightweight project management for people and coding agents.</em></strong></p>"
if grep -qF "$CLOSE" <<<"$(section idea)"; then ok "#idea's paragraph closes with the category line"; else bad "#idea's paragraph doesn't close with $CLOSE"; fi
if perl -0777 -ne 'exit !(/<footer.*\Q'"$CAT"'\E.*<\/footer>/s)' "$PAGE"; then ok "the footer has the category line"; else bad "the footer doesn't say $CAT"; fi
for m in 'name="description"' 'property="og:description"'; do
  if grep -qF "<meta $m content=\"$CAT" "$PAGE"; then ok "$m starts with the category line"; else bad "$m doesn't start with $CAT"; fi
done
EGG="A sassy “yes” to building with coding agents."
for el in '<img class="emblem"' '<span class="wordmark"'; do
  if grep -F "$el" "$PAGE" | grep -qF "title=\"$EGG\""; then ok "${el#<} has the easter egg"; else bad "${el#<} has no title=\"$EGG\""; fi
done
if [ "$(grep -oF "$EGG" "$PAGE" | wc -l | tr -d ' ')" = 2 ]; then ok "the easter egg shows only on hover"; else bad "the easter egg appears outside its two title attributes"; fi

echo "Teams"
teams="$(section teams)"
for l in "docs/comparison.md" "docs/monorepo.md" "README.md#monorepos" "README.md#keeping-plans-out-of-the-repo" "README.md#the-optional-hook"; do
  if grep -qF "blob/main/$l\"" <<<"$teams"; then ok "#teams links to $l"; else bad "#teams has no link to $l"; fi
done

echo "README lead-in"
lead="$(perl -0777 -ne 'print $1 if /\A(.*?)^## The rules/ms' "$ROOT/README.md")"
for l in "$URL/" "(#install)"; do
  if grep -qF "$l" <<<"$lead"; then ok "the lead-in links $l"; else bad "the README has no $l above ## The rules"; fi
done
lede="$(perl -0777 -ne 'print $1 if /<p class="lede">(.*?)<\/p>/s' "$PAGE")"
idea="$(perl -0777 -ne 'print $1 if /<section id="idea"[^>]*>\s*<h2>(.*?)<\/h2>/s' "$PAGE")"
for c in "$lede" "$idea"; do
  if [ -n "$c" ] && grep -qF "$c" <<<"$lead"; then ok "the lead-in says what the page does: ${c:0:50}…"; else bad "the README lead-in doesn't say: ${c:-?}"; fi
done

echo "Install text matches its source"
n=0
while IFS= read -r -d '' rec; do
  n=$((n+1)); src="${rec%%$'\t'*}"; text="${rec#*$'\t'}"
  if [ ! -f "$ROOT/$src" ]; then bad "block $n: data-source $src doesn't exist"; continue; fi
  if TEXT="$text" perl -0777 -ne 'exit !(index($_, $ENV{TEXT}) >= 0)' "$ROOT/$src"; then
    ok "block $n matches $src"
  else
    bad "block $n doesn't match $src word for word: $(head -1 <<<"$text" | cut -c1-70)…"
  fi
done < <(perl -0777 -ne '
  while (/<pre[^>]*data-source="([^"]+)"[^>]*>(.*?)<\/pre>/sg) {
    my ($s, $t) = ($1, $2);
    $t =~ s/<button.*?<\/button>//sg; $t =~ s/<[^>]+>//g;
    $t =~ s/&lt;/</g; $t =~ s/&gt;/>/g; $t =~ s/&quot;/"/g; $t =~ s/&#39;/\x27/g; $t =~ s/&amp;/&/g;
    print "$s\t$t\0";
  }' "$PAGE")
[ "$n" -ge 2 ] && ok "$n install blocks are checked" || bad "expected at least 2 data-source blocks, found $n"

echo "Links"
for a in $(perl -ne 'print "$1\n" while /href="#([^"]+)"/g' "$PAGE" | sort -u); do
  if grep -q "id=\"$a\"" "$PAGE"; then ok "#$a resolves"; else bad "#$a has no target"; fi
done
for l in "$REPO\"" "$REPO/blob/main/README.md" "$REPO/blob/main/LICENSE"; do
  if grep -qF "href=\"$l" "$PAGE"; then ok "links to ${l%\"}"; else bad "no link to ${l%\"}"; fi
done
for l in $(perl -ne 'print "$1\n" while m{href="https://github\.com/donjaime/yass/(?:blob|tree)/main/([^"]+)"}g' "$PAGE" | sort -u); do
  path="${l%%#*}"; anchor=""; [ "$path" != "$l" ] && anchor="${l#*#}"
  if [ ! -e "$ROOT/$path" ]; then bad "$path doesn't exist"; continue; fi
  if [ -n "$anchor" ]; then
    if slugs "$ROOT/$path" | grep -qxF "$anchor"; then ok "$path#$anchor resolves"; else bad "$path has no heading for #$anchor"; fi
  else ok "$path exists"; fi
done

echo "Assets"
# dims FILE: WxH of a PNG, or of a lossy WebP (VP8, or VP8X as cwebp writes with metadata or alpha)
dims() { perl -0777 -ne '
  if (substr($_, 1, 3) eq "PNG") { my ($w, $h) = unpack("NN", substr($_, 16, 8)); print "${w}x$h" }
  elsif (substr($_, 8, 8) eq "WEBPVP8X") { my @b = unpack("C6", substr($_, 24, 6));
    printf "%dx%d", 1 + $b[0] + ($b[1] << 8) + ($b[2] << 16), 1 + $b[3] + ($b[4] << 8) + ($b[5] << 16) }
  elsif (substr($_, 8, 8) eq "WEBPVP8 ") { my ($w, $h) = unpack("vv", substr($_, 26, 4)); printf "%dx%d", $w & 0x3fff, $h & 0x3fff }' "$1"; }
for spec in emblem.webp:square favicon.svg: favicon-32.png:32x32 apple-touch-icon.png:180x180 og.png:1200x630; do
  f="${spec%%:*}"; want="${spec#*:}"
  if [ ! -f "$SITE/assets/$f" ]; then bad "assets/$f is missing"; continue; fi
  if ! grep -qF "assets/$f" "$PAGE"; then bad "the page doesn't reference assets/$f"; continue; fi
  got="$(dims "$SITE/assets/$f")"
  if [ "$want" = square ]; then
    w="${got%x*}"
    if [ -z "$got" ] || [ "$got" != "${w}x$w" ] || [ "$w" -lt 460 ]; then bad "assets/$f is ${got:-unreadable}, not square and at least 460×460"; continue; fi
  elif [ -n "$want" ] && [ "$got" != "$want" ]; then bad "assets/$f is $(dims "$SITE/assets/$f"), not $want"; continue; fi
  ok "assets/$f"
done
for p in og:title og:description twitter:card; do
  if grep -qE "(property|name)=\"$p\" content=\"[^\"]+\"" "$PAGE"; then ok "$p"; else bad "no $p"; fi
done
if grep -qF "property=\"og:image\" content=\"$URL/assets/og.png\"" "$PAGE"; then ok "og:image is absolute"; else bad "og:image isn't $URL/assets/og.png"; fi

echo "Nothing external, and small"
ext="$(grep -nE 'src="(https?:)?//|<link[^>]+href="(https?:)?//' "$PAGE"; grep -nE '@import|url\((["'\'']?)(https?:)?//' "$SITE"/*.css)"
if [ -z "$ext" ]; then ok "no external scripts, styles, fonts or images"; else bad "loads something external:"; sed 's/^/       /' <<<"$ext"; fi
bytes=$(find "$SITE" -type f ! -path "$SITE/assets/og.png" ! -name README.md -exec cat {} + | wc -c | tr -d ' ')
if [ "$bytes" -lt 153600 ]; then ok "the page loads $bytes bytes (under 150 KB)"; else bad "the page loads $bytes bytes (150 KB at most)"; fi

placeholders="$(grep -rlF yass-placeholder "$SITE/assets" | grep -v '/README\.md$' | sed "s|^$SITE/||")"
if [ "$DEPLOY" = 1 ]; then
  echo "Ready to deploy"
  if [ -z "$placeholders" ]; then ok "no placeholder assets"; else
    for f in $placeholders; do bad "$f is still a placeholder (see site/assets/README.md)"; done; fi
elif [ -n "$placeholders" ]; then
  echo "  note $(wc -l <<<"$placeholders" | tr -d ' ') placeholder asset(s); --deploy fails until they're replaced"
fi

# Self-check, on a copy of this repo: the checks above must catch what they claim to.
if [ "$SELF" = 1 ]; then
  echo "Self-check"
  T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
  copy() { rm -rf "$T/r"; mkdir -p "$T/r"; (cd "$ROOT" && tar --exclude=./.git --exclude=./private --exclude=./bin -cf - .) | (cd "$T/r" && tar -xf -); }
  edit() { P="$2" R="$3" perl -0777 -i -pe 's/\Q$ENV{P}\E/$ENV{R}/' "$1"; }
  copy; edit "$T/r/docs/install.md" "Go step by step" "Go one step at a time"
  if o="$(bash "$0" --root "$T/r" 2>&1)"; then bad "an edited agent prompt in docs/install.md should fail"
  elif grep -q "doesn't match docs/install.md" <<<"$o"; then ok "an edited agent prompt in docs/install.md fails, naming the block"
  else bad "an edited agent prompt failed for another reason"; sed 's/^/       /' <<<"$(grep FAIL <<<"$o")"; fi
  copy; printf '\n<!-- yass-placeholder -->\n' >> "$T/r/site/assets/favicon.svg"
  if o="$(bash "$0" --root "$T/r" --deploy 2>&1)"; then bad "--deploy with a placeholder should fail"
  elif grep -q "assets/favicon.svg is still a placeholder" <<<"$o"; then ok "--deploy with a placeholder fails, naming it"
  else bad "--deploy with a placeholder failed for another reason"; sed 's/^/       /' <<<"$(grep FAIL <<<"$o")"; fi
  if bash "$0" --root "$T/r" >/dev/null 2>&1; then ok "without --deploy, placeholders pass"; else bad "without --deploy, placeholders should pass"; fi
  copy; edit "$T/r/site/index.html" 'docs/install.md#from-a-release' 'docs/install.md#no-such-heading'
  if o="$(bash "$0" --root "$T/r" 2>&1)"; then bad "a link to a missing heading should fail"
  elif grep -q "has no heading for #no-such-heading" <<<"$o"; then ok "a link to a missing heading fails"
  else bad "a link to a missing heading failed for another reason"; fi
fi

echo "$pass passed, $fail failed"
[ "$fail" = 0 ]

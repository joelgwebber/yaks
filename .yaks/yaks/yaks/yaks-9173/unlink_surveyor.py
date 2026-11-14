#!/usr/bin/env python3
"""Remove a yak body's `![..](<id>/<file>)` image line when an evidence document
that the same yak links already embeds <file>. Files are kept. Dry run unless --write."""
import re, os, sys, glob
write = '--write' in sys.argv
root = sys.argv[1]
os.chdir(root)
img = re.compile(r'^!\[[^\]]*\]\((?P<id>[^/)\s]+)/(?P<f>[^)\s]+)(?:\s+"[^"]*")?\)\s*$')
changed = removed = 0
for md in sorted(glob.glob('*.md')):
    id = md[:-3]
    if not os.path.isdir(id):
        continue
    text = open(md).read()
    docs = [f for f in os.listdir(id) if f.endswith('evidence.md') and f'({id}/{f})' in text]
    if not docs:
        continue
    embedded = set()
    for d in docs:
        t = open(f'{id}/{d}').read()
        embedded |= {os.path.basename(x) for x in re.findall(r'!\[[^\]]*\]\(([^)\s]+)\)', t)}
    lines = text.split('\n')
    out, n, i = [], 0, 0
    fence = False
    while i < len(lines):
        l = lines[i]
        if l.lstrip().startswith('```'):
            fence = not fence
        m = None if fence else img.match(l)
        if m and m.group('id') == id and os.path.basename(m.group('f')) in embedded \
                and os.path.exists(f"{id}/{m.group('f')}"):
            n += 1
            # drop the blank line that followed, so no double gap is left
            if i + 1 < len(lines) and lines[i + 1].strip() == '' and (not out or out[-1].strip() == ''):
                i += 1
            i += 1
            continue
        out.append(l)
        i += 1
    if n:
        changed += 1
        removed += n
        print(f'{id}: {n} link(s) removed')
        if write:
            open(md, 'w').write('\n'.join(out))
print(f'{changed} yaks, {removed} links', '(written)' if write else '(dry run)')

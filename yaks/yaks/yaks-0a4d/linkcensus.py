#!/usr/bin/env python3
"""linkcensus: read-only census of link forms and layout in yaks farms (yaks-0a4d).

Usage: linkcensus.py <farm-dir>... > report.md
Each <farm-dir> is a farm worktree (contains .yaks/). Only reads files and runs
read-only `git remote -v`. Link extraction is a regex approximation of
CommonMark: fenced code blocks and inline code spans are stripped first; inline
links/images (one level of nested brackets in the text), reference definitions
and <autolinks> are found. Resolution mirrors src/links.rs parse()/exists().
"""
import os, re, sys, json, subprocess, collections, posixpath, urllib.parse

RESERVED = {"docs", "doks", "sources", "schemas", "sessions", "wiki"}
ID_RE = re.compile(r"^[a-z0-9]+-[0-9a-z]{3,4}(\.\d+)*$")
TS_RE = re.compile(r"^\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d(\.\d+)?(Z|[+-]\d\d:\d\d)(-\d+)?$")
REV_RE = re.compile(r"^[0-9a-fA-F]{7,40}$")
FENCE_RE = re.compile(r"^ {0,3}(`{3,}|~{3,})")
INLINE_RE = re.compile(r"(!?)\[((?:[^\[\]]|\[[^\[\]]*\])*)\]\(\s*(<[^>]*>|(?:[^()\s]|\([^()\s]*\))*)(?:\s+(?:\"[^\"]*\"|'[^']*'|\([^)]*\)))?\s*\)")
REFDEF_RE = re.compile(r"^ {0,3}\[([^\]]+)\]:\s*(<[^>]*>|\S+)", re.M)
AUTO_RE = re.compile(r"<((?:https?|mailto|ftp):[^>\s]+)>")
CODESPAN_RE = re.compile(r"(`+)(.+?)\1", re.S)

def strip_code(text):
    out, fence = [], None
    for line in text.split("\n"):
        m = FENCE_RE.match(line)
        if fence:
            if m and m.group(1)[0] == fence[0] and len(m.group(1)) >= len(fence) and not line.strip()[len(m.group(1)):].strip():
                fence = None
            out.append("")
            continue
        if m:
            fence = m.group(1); out.append(""); continue
        out.append(line)
    t = "\n".join(out)
    return CODESPAN_RE.sub(lambda m: " " * len(m.group(0)), t)

def frontmatter(text):
    if not text.startswith("---\n"): return None, text
    end = text.find("\n---\n", 4)
    if end < 0: return None, text
    return text[4:end], text[end + 5:]

def note_ts(body):
    """Note timestamps in file order (a `---` line followed by `▸ ` line)."""
    lines = body.split("\n"); out = []
    for i in range(len(lines) - 1):
        if lines[i].rstrip() == "---" and lines[i + 1].startswith("\u25b8 "):
            out.append(lines[i + 1][2:].split()[0] if lines[i + 1][2:].split() else "")
    return out

class Farm:
    def __init__(self, path):
        self.path = path
        self.root = os.path.join(path, ".yaks")
        self.files = []  # rel paths of all files
        self.dirs = collections.Counter()
        for dp, dn, fn in os.walk(self.root):
            dn[:] = [d for d in dn if not d.startswith(".")]
            rel = os.path.relpath(dp, self.root)
            rel = "" if rel == "." else rel
            self.dirs[rel] = len(dn) + len([f for f in fn if not f.startswith(".")])
            for f in fn:
                if f.startswith("."): continue
                self.files.append(posixpath.join(rel, f) if rel else f)
        self.fileset = set(self.files)
        # yaks: <herd>/yaks/<id>.md, or legacy flat <id>.md at the root
        self.yaks = {}
        for f in self.files:
            p = f.split("/")
            if len(p) == 3 and p[1] == "yaks" and p[2].endswith(".md") and p[0] not in RESERVED:
                self.yaks[p[2][:-3]] = f
            elif len(p) == 1 and f.endswith(".md") and ID_RE.match(f[:-3]):
                self.yaks[f[:-3]] = f
        self.herds = sorted({self.yaks[i].split("/")[0] for i in self.yaks if "/" in self.yaks[i]})
        self._notes = {}

    def kind(self, f):
        p = f.split("/")
        if f in self.yaks.values(): return "yak", None
        if len(p) == 4 and p[1] == "yaks" and p[2] in self.yaks: return "attachment", p[2]
        if p[0] == "sessions": return "session", None
        if p[0] == "sources": return ("source" if p[-1] in ("source.md", "extracted.md") else "source-other"), None
        if p[0] == "schemas": return "schema", None
        if p[0] in ("doks", "wiki") or (len(p) > 1 and p[1] in ("doks", "wiki")): return "dok", None
        return "other", None

    def notes(self, yid):
        if yid not in self._notes:
            try: self._notes[yid] = note_ts(open(os.path.join(self.root, self.yaks[yid]), encoding="utf-8", errors="replace").read())
            except Exception: self._notes[yid] = []
        return self._notes[yid]

    def att_path(self, yid, name):
        y = self.yaks.get(yid)
        if not y: return None
        return posixpath.join(posixpath.dirname(y), yid, name)

def clean(p):
    parts = []
    for s in p.split("/"):
        if s in ("", "."): continue
        if s == "..":
            if not parts: return None
            parts.pop()
        else: parts.append(s)
    return "/".join(parts) if parts else None

def looks_like_file(p):
    return not any(c in p for c in "{}<>") and "." in p.rsplit("/", 1)[-1]

def split_rev(s):
    if "@" in s:
        p, r = s.rsplit("@", 1)
        if REV_RE.match(r) and p: return p, r
    return s, None

def classify(farm, dest, kind, owner, docpath, remotes):
    """-> dict(form, sub, target(rel path or None), check(True/False/None), pin, lines)"""
    d = dest.strip()
    if d.startswith("<") and d.endswith(">"): d = d[1:-1]
    r = dict(form=None, sub="", target=None, ok=None, pin=False, lines=False, dest=d)
    body, frag = (d.split("#", 1) + [None])[:2] if "#" in d else (d, None)
    r["lines"] = bool(frag and re.match(r"^L\d+", frag))
    if "://" in d or d.startswith("mailto:"):
        r["form"] = "url"
        m = re.match(r"https?://github\.com/([^/]+)/([^/]+)/(blob|tree|raw)/([^/]+)/?(.*)", d)
        if m:
            repo = (m.group(1) + "/" + m.group(2)).lower().removesuffix(".git")
            ref = m.group(4)
            own = repo in remotes
            r["sub"] = ("gh-own-farm-branch" if own and ref == "yaks" else
                        "gh-own-code" if own else "gh-other-blob")
            r["pin"] = bool(REV_RE.match(ref))
        return r
    if not body:
        r["form"] = "fragment"; return r
    body = urllib.parse.unquote(body)
    if body.startswith("code:"):
        p, rev = split_rev(body[5:]); r.update(form="code", pin=bool(rev)); return r
    if body.startswith("../"):
        if kind in ("yak", "attachment"):
            rest = body[3:]
            m = re.match(r"blob/([0-9a-fA-F]{7,40})/(.*)", rest)
            if clean(m.group(2) if m else rest) is None:
                r.update(form="upward", ok=False); return r
            r.update(form="legacy-code", sub="blob-pinned" if m else "plain", pin=bool(m))
            return r
        r.update(form="upward", ok=False); return r
    path, rev = split_rev(body)
    r["pin"] = bool(rev)
    if not rev and "/" not in path and path in farm.yaks:
        if frag and TS_RE.match(frag):
            r["form"] = "yak-note"
            ts, n = frag, 0
            m = re.match(r"(.*)-(\d+)$", frag)
            if m and TS_RE.match(m.group(1)) and not m.group(1).endswith(("Z",)) is False:
                pass
            mm = re.match(r"^(.*?Z|.*?[+-]\d\d:\d\d)-(\d+)$", frag)
            if mm: ts, n = mm.group(1), int(mm.group(2))
            r["ok"] = farm.notes(path).count(ts) > n
        else:
            r["form"] = "yak-id"
        r["target"] = farm.yaks[path]; return r
    if path.startswith("/"):
        c = clean(path[1:])
        if c is None: r.update(form="upward", ok=False); return r
        first = c.split("/")[0]
        if c.startswith(("doks/sessions/", "doks/sources/")): r["sub"] = "doks/" + c.split("/")[1] + " (yaks-bede)"
        elif first in ("doks", "sessions", "sources", "schemas", "wiki"): r["sub"] = first
        elif first in farm.herds: r["sub"] = "herd:" + first
        else: r["sub"] = "OTHER:/" + first
        r.update(form="farm-abs", target=c)
        r["ok"] = (c in farm.fileset or os.path.isdir(os.path.join(farm.root, c))) if looks_like_file(c) else None
        return r
    if "/" in path:
        yid, name = path.split("/", 1)
        if yid in farm.yaks and name and "/" not in name:
            t = farm.att_path(yid, name)
            r.update(form="attachment", target=t, ok=(t in farm.fileset) if looks_like_file(name) else None)
            return r
    if kind == "attachment":
        n = clean(path)
        if n and not rev and "/" not in n:
            t = farm.att_path(owner, n)
            r.update(form="sibling", target=t, ok=(t in farm.fileset) if looks_like_file(n) else None)
            return r
    base = posixpath.dirname(docpath) if kind in ("dok", "session", "source", "schema", "other") else ""
    c = clean(posixpath.join(base, path) if base else path)
    if c is None: r.update(form="upward", ok=False); return r
    if ID_RE.match(path.split("/")[0]) and path.split("/")[0] not in farm.yaks and "." not in path.split("/")[0]:
        r["sub"] = "unknown-id-like"
    r.update(form="relative-down" if base else "relative-root", target=c)
    r["ok"] = (c in farm.fileset) if looks_like_file(c) else None
    return r

def gh_resolves(farm, docpath, kind, r, dropped):
    """Would GitHub's blob view of docpath resolve this dest to an existing file?
    dropped: whether the .yaks root is dropped (farm root == repo root)."""
    if r["form"] in ("url", "fragment"): return None
    d = r["dest"].split("#")[0]
    if not d or "://" in d or d.startswith(("code:", "mailto:")): return False
    d = urllib.parse.unquote(d)
    repo_doc = docpath if dropped else ".yaks/" + docpath
    if d.startswith("/"): t = clean(d[1:])
    else: t = clean(posixpath.join(posixpath.dirname(repo_doc), d))
    if t is None: return False
    if not dropped:
        if not t.startswith(".yaks/"): return False
        t = t[6:]
    return t in farm.fileset

def relpath_to(docpath, target):
    return posixpath.relpath(target, posixpath.dirname(docpath) or ".")

def scan(fpath):
    farm = Farm(fpath)
    try:
        rem = subprocess.run(["git", "-C", fpath, "remote", "-v"], capture_output=True, text=True).stdout
    except Exception: rem = ""
    remotes = set()
    for line in rem.splitlines():
        m = re.search(r"github\.com[:/]([^/\s]+)/([^\s]+?)(\.git)?\s", line + " ")
        if m: remotes.add((m.group(1) + "/" + m.group(2)).lower())
    rem_hosts = sorted({re.sub(r"^.*?(://|@)([^/:]+).*$", r"\2", l.split()[1]) for l in rem.splitlines() if l.strip()})
    S = dict(name=os.path.basename(fpath), remotes=sorted({l.split()[1] for l in rem.splitlines() if l.strip()}), hosts=rem_hosts)
    forms = collections.Counter(); subs = collections.Counter(); syn = collections.Counter()
    by_kind = collections.Counter(); broken = collections.Counter(); broken_ex = collections.defaultdict(list)
    unchecked = collections.Counter(); broken_kind = collections.Counter(); migf = collections.Counter(); anomalies = collections.Counter(); pins = 0; lines = 0
    mig = collections.Counter(); mig_ex = collections.defaultdict(list)
    fm_count = 0; fm_keys = collections.Counter(); notes = 0; md_files = 0; kinds = collections.Counter()
    for f in farm.files:
        if not f.endswith((".md", ".markdown")): continue
        kind, owner = farm.kind(f)
        if kind == "source-other": continue
        md_files += 1; kinds[kind] += 1
        try: text = open(os.path.join(farm.root, f), encoding="utf-8", errors="replace").read()
        except Exception: continue
        fm, body = frontmatter(text)
        if fm is not None:
            fm_count += 1
            for k in re.findall(r"^([A-Za-z_][\w-]*):", fm, re.M): fm_keys[k] += 1
        if kind == "yak": notes += len(note_ts(body))
        t = strip_code(body)
        found = [("image" if m.group(1) else "inline", m.group(3)) for m in INLINE_RE.finditer(t)]
        found += [("refdef", m.group(2)) for m in REFDEF_RE.finditer(t)]
        found += [("autolink", m.group(1)) for m in AUTO_RE.finditer(t)]
        for synt, dest in found:
            syn[synt] += 1
            r = classify(farm, dest, kind, owner, f, remotes)
            key = r["form"] + (":" + r["sub"] if r["sub"] else "")
            forms[r["form"]] += 1; subs[key] += 1; by_kind[(kind, r["form"])] += 1
            pins += r["pin"]; lines += r["lines"]
            if r["sub"].startswith("OTHER:"): anomalies[(kind, r["sub"][6:])] += 1
            if r["ok"] is False:
                broken[key] += 1; broken_kind[kind] += 1
                if len(broken_ex[key]) < 4: broken_ex[key].append(f"{f}: `{dest}`")
            elif r["ok"] is None and r["form"] in ("farm-abs", "relative-down", "relative-root", "attachment", "sibling"):
                unchecked[key] += 1
            # --- migration sizing (farm-internal links only)
            internal = r["form"] not in ("url", "fragment", "code", "legacy-code")
            if not internal: continue
            mig["internal"] += 1
            sc = "cap_" if kind == "source" else "au_"
            mig[sc + "internal"] += 1
            resolvable = r["ok"] is not False and r["target"] is not None
            # (i) document-relative: rewrite unless written dest already equals relpath to target (+ fragment)
            if resolvable:
                want = relpath_to(f, r["target"])
                have = r["dest"].split("#")[0]
                if urllib.parse.unquote(have) != want: mig["i_rewrite"] += 1; mig[sc + "i_rewrite"] += 1; migf[("i", r["form"])] += 1
            else:
                mig["i_manual"] += 1; mig[sc + "manual"] += 1
                if len(mig_ex["manual"]) < 6: mig_ex["manual"].append(f"{f}: `{dest}`")
            # (ii) drop .yaks root, forms unchanged: nothing rewritten; count GitHub resolution before/after
            migf[("all", r["form"])] += 1
            if gh_resolves(farm, f, kind, r, False): mig["gh_ok_today"] += 1; mig[sc + "gh0"] += 1; migf[("gh0", r["form"])] += 1
            if gh_resolves(farm, f, kind, r, True): mig["gh_ok_ii"] += 1; mig[sc + "gh2"] += 1; migf[("gh2", r["form"])] += 1
            if not resolvable: mig["ii_manual"] += 1
            # (iii) also /doks/sessions|sources -> /sessions|sources
            if r["sub"].startswith("doks/"):
                c = r["target"][5:]
                if c in farm.fileset or not looks_like_file(c): mig["iii_rewrite_mech"] += 1; mig[sc + "gh3_extra"] += (c in farm.fileset)
                else:
                    mig["iii_manual"] += 1
                    if len(mig_ex["iii"]) < 4: mig_ex["iii"].append(f"{f}: `{dest}`")
            # yak-id / note links: GitHub-dead under any root unless written as a path with .md
            if r["form"] in ("yak-id", "yak-note"): mig["yak_links"] += 1; mig[sc + "yak"] += 1
    # layout
    per_herd = collections.Counter(p.split("/")[0] if "/" in p else "(flat root)" for p in farm.yaks.values())
    att_dirs = sum(1 for d in farm.dirs if len(d.split("/")) == 3 and d.split("/")[1] == "yaks" and d.split("/")[2] in farm.yaks)
    total = sum(os.path.getsize(os.path.join(farm.root, f)) for f in farm.files)
    enc = [f for f in farm.files if re.search(r"[ ()%#?\[\]]|[^\x00-\x7f]", f)]
    longest = sorted(farm.files, key=len, reverse=True)[:3]
    maxdir = max(farm.dirs.items(), key=lambda kv: kv[1]) if farm.dirs else ("", 0)
    over1000 = [(d, n) for d, n in farm.dirs.items() if n > 1000]
    S.update(broken_kind=broken_kind, migf=migf, forms=forms, subs=subs, syn=syn, by_kind=by_kind, broken=broken, broken_ex=broken_ex,
             unchecked=unchecked, anomalies=anomalies, pins=pins, lines=lines, mig=mig, mig_ex=mig_ex,
             fm_count=fm_count, fm_keys=fm_keys, notes=notes, md_files=md_files, kinds=kinds,
             per_herd=per_herd, att_dirs=att_dirs, total=total, enc=enc, longest=longest, maxdir=maxdir,
             over1000=over1000, nyaks=len(farm.yaks), nfiles=len(farm.files),
             schema=open(os.path.join(farm.root, "schema")).read().strip() if os.path.exists(os.path.join(farm.root, "schema")) else "?")
    return S

def pct(a, b): return f"{a} ({100*a/b:.0f}%)" if b else str(a)

def report(all_s):
    o = ["# Link and layout census (yaks-0a4d)\n", "Generated by `linkcensus.py` (read-only). Numbers are per farm `.yaks/` tree.\n"]
    o.append("## Authored documents only (yaks, attachments, doks, sessions, schemas; captured sources excluded)\n\n| farm | internal links | GitHub resolves today (`.yaks/` root) | after (ii) drop root | after (iii) | of which yak-id/note (dead on GitHub in any form) | (i) doc-relative: rewrites | unresolvable today |\n|---|---|---|---|---|---|---|---|")
    for s in all_s:
        m = s["mig"]; i = m["au_internal"]
        o.append(f"| {s['name']} | {i} | {pct(m['au_gh0'], i)} | {pct(m['au_gh2'], i)} | {pct(m['au_gh2'] + m['au_gh3_extra'], i)} | {m['au_yak']} | {pct(m['au_i_rewrite'], i)} | {m['au_manual']} |")
    o.append("\nCaptured sources (`sources/*/source.md|extracted.md`): " + "; ".join(f"{s['name']} {s['mig']['cap_internal']} internal-looking, {s['mig']['cap_manual']} unresolvable" for s in all_s if s['mig']['cap_internal']) + ". Most are site-relative links copied from the captured page, not farm links.\n")
    o.append("## Summary (all documents)\n\n| farm | schema | md files | yaks | links | internal | broken | farm-abs OTHER | GitHub-resolves today | after (ii) | yak-id/note links | (i) rewrite | (iii) rewrite | manual |\n|---|---|---|---|---|---|---|---|---|---|---|---|---|---|")
    for s in all_s:
        n = sum(s["forms"].values()); m = s["mig"]; i = m["internal"]
        o.append(f"| {s['name']} | {s['schema']} | {s['md_files']} | {s['nyaks']} | {n} | {i} | {sum(s['broken'].values())} | {sum(s['anomalies'].values())} | {pct(m['gh_ok_today'], i)} | {pct(m['gh_ok_ii'], i)} | {m['yak_links']} | {pct(m['i_rewrite'], i)} | {m['iii_rewrite_mech']}+{m['iii_manual']} | {m['i_manual']} |")
    for s in all_s:
        o.append(f"\n## {s['name']}\n")
        o.append(f"- remotes: {', '.join(s['remotes']) or 'none'}; schema {s['schema']}; {s['nfiles']} files, {s['total']/1e6:.1f} MB; md by kind: {dict(s['kinds'])}")
        o.append(f"- yaks per herd: {dict(s['per_herd'])}; attachment dirs: {s['att_dirs']}")
        o.append(f"- largest dir: `{s['maxdir'][0] or '.'}` = {s['maxdir'][1]} entries; dirs > 1000 entries: {s['over1000'] or 'none'}")
        o.append(f"- filenames needing URL-encoding: {len(s['enc'])}" + (f" e.g. {s['enc'][:3]}" if s['enc'] else ""))
        o.append(f"- longest paths: " + ", ".join(f"`{p}` ({len(p)})" for p in s['longest']))
        o.append(f"- frontmatter: {s['fm_count']}/{s['md_files']} md files; top keys: {dict(s['fm_keys'].most_common(10))}")
        o.append(f"- notes in yak bodies: {s['notes']}")
        o.append(f"- syntax: {dict(s['syn'])}; `@sha` pins: {s['pins']}; `#L` anchors: {s['lines']}")
        o.append("\n| form | count | broken | unchecked (placeholder/no ext) |\n|---|---|---|---|")
        for k, v in sorted(s["subs"].items(), key=lambda kv: -kv[1]):
            o.append(f"| {k} | {v} | {s['broken'].get(k, 0)} | {s['unchecked'].get(k, 0)} |")
        kinds = sorted({k for k, _ in s["by_kind"]})
        o.append("\nforms by source document kind: " + "; ".join(f"**{k}** " + ", ".join(f"{f}={n}" for (kk, f), n in sorted(s['by_kind'].items()) if kk == k) for k in kinds))
        if s["anomalies"]:
            byk = collections.defaultdict(list)
            for (k, seg), n in s["anomalies"].most_common(): byk[k].append(f"/{seg}={n}")
            o.append("\nfarm-abs with other first segment (by source doc kind): " + "; ".join(f"**{k}**: " + ", ".join(v[:25]) + (" ..." if len(v) > 25 else "") for k, v in byk.items()))
        o.append(f"\nbroken by source doc kind: {dict(s['broken_kind'])}")
        mf = s["migf"]; fs = sorted({f for (_, f) in mf})
        o.append("\n| internal form | n | GitHub resolves today | after (ii) | (i) needs rewrite |\n|---|---|---|---|---|")
        for f in fs: o.append(f"| {f} | {mf[('all', f)]} | {mf[('gh0', f)]} | {mf[('gh2', f)]} | {mf[('i', f)]} |")
        if s["broken_ex"]:
            o.append("\nbroken examples:")
            for k, ex in s["broken_ex"].items():
                for e in ex: o.append(f"- [{k}] {e}")
        m = s["mig"]
        o.append(f"\nmigration: internal={m['internal']}, (i) rewrite={m['i_rewrite']}, unresolvable={m['i_manual']}, GitHub-resolves today={m['gh_ok_today']}, after (ii)={m['gh_ok_ii']}, (iii) /doks/sessions|sources fixes: mechanical={m['iii_rewrite_mech']} manual={m['iii_manual']}")
    return "\n".join(o) + "\n"

if __name__ == "__main__":
    res = [scan(p) for p in sys.argv[1:]]
    print(report(res))
    with open(os.environ.get("LC_JSON", "/dev/null"), "w") as fh:
        json.dump([{k: (dict(v) if isinstance(v, collections.Counter) else v) for k, v in s.items() if k not in ("by_kind", "anomalies", "migf")} for s in res], fh, default=str, indent=1)

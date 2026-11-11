# linkhost-1 raw evidence (2026-11-11)

## GitHub POST /markdown, context=joelgwebber/yaks
### input t.md
```markdown
[abs](/doks/x.md) [rel](x.md) [sub](sub/x.md) [up](../x.md) [upup](../../../../x.md)
[head](/doks/x.md#some-heading) [lines](x.md#L12-L40) [dir](/doks/) [pct](a%20b.md) [ext](yaks-1a2b) [note](yaks-1a2b#2026-11-04T22:13:03Z)
[branch](/joelgwebber/yaks/blob/main/src/links.rs#L12) [code](code:src/foo.rs) [full](https://github.com/joelgwebber/yaks/blob/abc1234/src/links.rs#L12-L40)
![img](shot.png) ![img2](/doks/a.png)

## ▸ 2026-11-04T22:13:03Z [actor]

<a id="2026-11-04T22:13:03Z"></a><a name="n1"></a> <span id="x">s</span>
```
gfm:
```html
<p dir="auto"><a href="/doks/x.md">abs</a> <a href="x.md">rel</a> <a href="sub/x.md">sub</a> <a href="../x.md">up</a> <a href="../../../../x.md">upup</a><br>
<a href="/doks/x.md#some-heading">head</a> <a href="x.md#L12-L40">lines</a> <a href="/doks/">dir</a> <a href="a%20b.md">pct</a> <a href="yaks-1a2b">ext</a> note<br>
<a href="/joelgwebber/yaks/blob/main/src/links.rs#L12">branch</a> code <a href="https://github.com/joelgwebber/yaks/blob/abc1234/src/links.rs#L12-L40">full</a><br>
<a target="_blank" rel="noopener noreferrer" href="shot.png"><img src="shot.png" alt="img" style="max-width: 100%;"></a> <a target="_blank" rel="noopener noreferrer" href="/doks/a.png"><img src="/doks/a.png" alt="img2" style="max-width: 100%;"></a></p>
<h2 dir="auto">▸ 2026-11-04T22:13:03Z [actor]</h2>
<p dir="auto"><a id="user-content-2026-11-04t22:13:03z"></a><a name="user-content-n1"></a> <span id="user-content-x">s</span></p>```
markdown (file-view) mode:
```html
<p><a href="/doks/x.md">abs</a> <a href="x.md">rel</a> <a href="sub/x.md">sub</a> <a href="../x.md">up</a> <a href="../../../../x.md">upup</a>
<a href="/doks/x.md#some-heading">head</a> <a href="x.md#L12-L40">lines</a> <a href="/doks/">dir</a> <a href="a%20b.md">pct</a> <a href="yaks-1a2b">ext</a> note
<a href="/joelgwebber/yaks/blob/main/src/links.rs#L12">branch</a> code <a href="https://github.com/joelgwebber/yaks/blob/abc1234/src/links.rs#L12-L40">full</a>
<a target="_blank" rel="noopener noreferrer" href="shot.png"><img src="shot.png" alt="img" style="max-width: 100%;"></a> <a target="_blank" rel="noopener noreferrer" href="/doks/a.png"><img src="/doks/a.png" alt="img2" style="max-width: 100%;"></a></p>
<div class="markdown-heading"><h2 class="heading-element">▸ 2026-11-04T22:13:03Z [actor]</h2><a id="user-content--2026-11-04t221303z-actor" class="anchor" aria-label="Permalink: ▸ 2026-11-04T22:13:03Z [actor]" href="#-2026-11-04t221303z-actor"><span aria-hidden="true" class="octicon octicon-link"></span></a></div>
<p><a id="user-content-2026-11-04T22:13:03Z"></a><a name="user-content-n1"></a> <span id="user-content-x">s</span></p>
```
### input t2.md
```markdown
[n1](yaks-1a2b#2026-11-04T22:13:03Z) [n2](yaks-1a2b#2026-11-04T22%3A13%3A03Z) [n3](./yaks-1a2b.md#x:y) [n4](yaks-1a2b.md#20261104t221303z) [n5](<a b.md>) [n6](a b.md) [n7](mailto:x@y) [n8](foo:bar) [n9](x.md#a:b)
```
gfm:
```html
<p dir="auto">n1 <a href="yaks-1a2b#2026-11-04T22%3A13%3A03Z">n2</a> <a href="./yaks-1a2b.md#x:y">n3</a> <a href="yaks-1a2b.md#20261104t221303z">n4</a> <a href="a%20b.md">n5</a> [n6](a b.md) <a href="mailto:x@y">n7</a> n8 n9</p>```
markdown (file-view) mode:
```html
<p>n1 <a href="yaks-1a2b#2026-11-04T22%3A13%3A03Z">n2</a> <a href="./yaks-1a2b.md#x:y">n3</a> <a href="yaks-1a2b.md#20261104t221303z">n4</a> <a href="a%20b.md">n5</a> [n6](a b.md) <a href="mailto:x@y">n7</a> n8 n9</p>
```
### input t4.md
```markdown
[a](x.md#a:b) [b](./x.md#a:b) [c](x#a:b) [d](/doks/x.md#a:b) [e](sub/x.md#a:b) [f](x.md#2026-11-04T22%3A13%3A03Z) [g](x.md?a:b) [h](yaks-1a2b.md#n-2026-11-04t221303z) [i](#a:b) [j](code:src/x.rs) [k](https:x)

---
▸ 2026-11-04T22:13:03Z [actor]

body

---

### 2026-11-04T22:13:03Z

<a id="n-2026-11-04T22:13:03Z"></a>
```
gfm:
```html
<p dir="auto">a <a href="./x.md#a:b">b</a> c <a href="/doks/x.md#a:b">d</a> <a href="sub/x.md#a:b">e</a> <a href="x.md#2026-11-04T22%3A13%3A03Z">f</a> g <a href="yaks-1a2b.md#n-2026-11-04t221303z">h</a> i j <a href="https:x">k</a></p>
<hr>
<p dir="auto">▸ 2026-11-04T22:13:03Z [actor]</p>
<p dir="auto">body</p>
<hr>
<h3 dir="auto">2026-11-04T22:13:03Z</h3>
<p dir="auto"><a id="user-content-n-2026-11-04t22:13:03z"></a></p>```
markdown (file-view) mode:
```html
<p>a <a href="./x.md#a:b">b</a> c <a href="/doks/x.md#a:b">d</a> <a href="sub/x.md#a:b">e</a> <a href="x.md#2026-11-04T22%3A13%3A03Z">f</a> g <a href="yaks-1a2b.md#n-2026-11-04t221303z">h</a> i j <a href="https:x">k</a></p>
<hr>
<p>▸ 2026-11-04T22:13:03Z [actor]</p>
<p>body</p>
<hr>
<div class="markdown-heading"><h3 class="heading-element">2026-11-04T22:13:03Z</h3><a id="user-content-2026-11-04t221303z" class="anchor" aria-label="Permalink: 2026-11-04T22:13:03Z" href="#2026-11-04t221303z"><span aria-hidden="true" class="octicon octicon-link"></span></a></div>
<p><a id="user-content-n-2026-11-04T22:13:03Z"></a></p>
```
### input t5.md
```markdown
[a](yaks-1a2b#n) [b](yaks-1a2b.md#2026-11-04T22:13:03Z) [c](./yaks-1a2b#2026-11-04T22:13:03Z) [d](/yaks/yaks/yaks-1a2b.md#x:y) [e](yaks-1a2b?x:y)
```
gfm:
```html
<p dir="auto"><a href="yaks-1a2b#n">a</a> b <a href="./yaks-1a2b#2026-11-04T22:13:03Z">c</a> <a href="/yaks/yaks/yaks-1a2b.md#x:y">d</a> e</p>```
markdown (file-view) mode:
```html
<p><a href="yaks-1a2b#n">a</a> b <a href="./yaks-1a2b#2026-11-04T22:13:03Z">c</a> <a href="/yaks/yaks/yaks-1a2b.md#x:y">d</a> e</p>
```
## Real blob views (richText hrefs)
```
yaks-0079 (/doks/design/scheduling.md):
href="/joelgwebber/yaks/blob/yaks/doks/design/scheduling.md"
yaks-1f34 (../src/fields.rs):
href="/joelgwebber/yaks/blob/yaks/.yaks/yaks/src/fields.rs"
yaks-84e0 images:
<img src="/joelgwebber/yaks/raw/yaks/.yaks/yaks/yaks/yaks-84e0/yaks-84e0-viewer-link.png"
<img src="/joelgwebber/yaks/raw/yaks/.yaks/yaks/yaks/yaks-84e0/yaks-84e0-viewer-link.svg"
yaks-0e15 attachment:
href="/joelgwebber/yaks/blob/yaks/.yaks/yaks/yaks/yaks-0e15/doclink-frames.txt"
CyberMonitor root README ../../blob/master/... -> /CyberMonitor/APT_CyberCriminal_Campagin_Collections/blob/master/2024/...
ther0n/UnnaturalScrollWheels /../main/X -> /ther0n/UnnaturalScrollWheels/blob/main/X (raw/main for img)
codeberg forgejo/docs docs/user/incoming.md ../../admin/config-cheat-sheet/ -> /forgejo/docs/src/branch/next/admin/config-cheat-sheet
```
## URL probes (curl status)
```
blob/yaks/.yaks/doks -> 301 https://github.com/joelgwebber/yaks/tree/yaks/.yaks/doks
blob/yaks/.yaks/doks/lessons -> 404 
blob/yaks/doks/lessons.md -> 404 
blob/yaks/.yaks/yaks/yaks/yaks-0e15 -> 301 https://github.com/joelgwebber/yaks/tree/yaks/.yaks/yaks/yaks/yaks-0e15
blob/yaks/.yaks/yaks/yaks/yaks-0079 -> 404 
blob/e4d0ff5/README.md -> 200 
blob/HEAD/README.md -> 200 
blob/yaks/.yaks/doks/lessons.md?plain=1#L3-L5 -> 200 
raw/yaks/.yaks/doks/lessons.md -> 302 https://raw.githubusercontent.com/joelgwebber/yaks/yaks/.yaks/doks/lessons.md
tree/yaks/.yaks/yaks/yaks: totalCount 881, 881 entries served
nodejs/node tree/main/test/parallel: 'Sorry, we had to truncate this directory to 1,000 files. 4,185 entries were omitted'; contents API returns 1000
gh search/code repo:joelgwebber/yaks "Links from documents to yaks failing" -> total_count 0 (non-default branch not indexed)
```

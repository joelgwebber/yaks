# Render evidence for note formats

Rendered with `gh api -X POST markdown -f mode=gfm -f context=joelgwebber/yaks`.

## Candidate a source

```markdown
Description text of the yak.

#### 2026-11-04T22:13:03Z · linkplan-lead

kind: move · from: hairy · to: shaving

moved: hairy -> shaving

#### 2026-11-04T22:13:03Z · linkplan-lead

A second note in the same second.

See [the first](/yaks/yaks/yaks-1a2b.md#20261104t221303z-linkplan-lead).
```

### HTML

```html
<p dir="auto">Description text of the yak.</p>
<h4 dir="auto">2026-11-04T22:13:03Z · linkplan-lead</h4>
<p dir="auto">kind: move · from: hairy · to: shaving</p>
<p dir="auto">moved: hairy -&gt; shaving</p>
<h4 dir="auto">2026-11-04T22:13:03Z · linkplan-lead</h4>
<p dir="auto">A second note in the same second.</p>
<p dir="auto">See <a href="/yaks/yaks/yaks-1a2b.md#20261104t221303z-linkplan-lead">the first</a>.</p>
```

## Candidate b source

```markdown
Description text of the yak.

---

<a id="n-20261104t221303z"></a>
**2026-11-04T22:13:03Z** · linkplan-lead

> kind: move · from: hairy · to: shaving

moved: hairy -> shaving
```

### HTML

```html
<p dir="auto">Description text of the yak.</p>
<hr>
<p dir="auto"><a id="user-content-n-20261104t221303z"></a><br>
<strong>2026-11-04T22:13:03Z</strong> · linkplan-lead</p>
<blockquote>
<p dir="auto">kind: move · from: hairy · to: shaving</p>
</blockquote>
<p dir="auto">moved: hairy -&gt; shaving</p>
```

## Candidate c source

```markdown
Description text of the yak.

### 2026-11-04T22:13:03Z · linkplan-lead

| kind | from | to |
|---|---|---|
| move | hairy | shaving |

moved: hairy -> shaving

### 2026-11-04T22:14:00Z · Joel Webber

Plain note, no metadata.
```

### HTML

```html
<p dir="auto">Description text of the yak.</p>
<h3 dir="auto">2026-11-04T22:13:03Z · linkplan-lead</h3>
<markdown-accessiblity-table><table role="table">
<thead>
<tr>
<th>kind</th>
<th>from</th>
<th>to</th>
</tr>
</thead>
<tbody>
<tr>
<td>move</td>
<td>hairy</td>
<td>shaving</td>
</tr>
</tbody>
</table></markdown-accessiblity-table>
<p dir="auto">moved: hairy -&gt; shaving</p>
<h3 dir="auto">2026-11-04T22:14:00Z · Joel Webber</h3>
<p dir="auto">Plain note, no metadata.</p>
```

## Today's notes on the live page

ids found on https://github.com/joelgwebber/yaks/blob/yaks/.yaks/yaks/yaks/yaks-1151.md :

```text
id="user-content--2026-11-11t211311z-sched-1kind-movefrom-hairyto-shaving"
id="user-content--2026-11-11t223930z-triage-1kind-askneeds-human"
id="user-content--2026-11-14t045104z-sched-1kind-answerneeds-agent"
```

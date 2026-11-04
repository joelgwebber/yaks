---
id: yaks-dedc
title: Weird, duplicative structures in wiki sources
status: hairy
type: task
priority: 3
created: '2026-11-03T23:50:22Z'
updated: '2026-11-04T00:18:12Z'
tags:
- wiki
---

---
▸ 2026-11-04T00:04:43Z [Joel Webber]

Example: In another repo, I imported a confluence doc, which the agent decided to fetch as HTML, then convert to markdown. The resulting sources came out like this:
```python
sources/[date-hash0]/  # "[name] (Confluence, draft v10)"
  source.md            # Markdown with frontmatter, ironically full of HTML garbage
  extracted.md         # *Only* HTML garbage; no frontmatter

sources/[date-hash0]/  # "[name] (Markdown conversion)"
  source.md            # Actual markdown with frontmatter, pointing back to the HTML.
  extracted.md         # The exact same markdown, without frontmatter
```

I'm not quite sure what in the tools and skills possessed the agent to do this, but it's massively redundant and pointless. In scenarios like this, it really should only keep a single source file with the markdown conversion and frontmatter with provenance information. The raw HTML is completely useless, as is the description "Snapshot of the local file /some/random/tempfile/location/...". It actually has a perfectly good title, but it decided to keep the useless temp-file provenance for some reason.

Let's look carefully at the tool/skill structures likely to have led to this behavior, and try to nudge them in the right direction. Consider the totality, rather than just trying to slap a "don't do this one weird thing" band-aid on it.

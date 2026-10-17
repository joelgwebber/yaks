---
id: yaks-ea13
title: Markdown rendering improvements
status: shorn
type: task
priority: 3
created: '2026-10-05T22:15:30Z'
updated: '2026-10-17T19:55:57Z'
parent: yaks-e9de
labels:
- markdown
- rendering
- tui
---

Our markdown rendering is often hard to read. We should aim for a more complete markdown->TUI (or console) implementation that takes into account all the common markdown idioms that apply to text rendering. A few examples:

We could remove **markers** _like_ *these* in the rendering output.
[Links](https://example.com/) could be rendered more simply.

It also appears that _long styled strings don't get properly styled
across lines_. Perhaps this can be simplified by formalizing our notion
of a paragraph to include lines with breaks.

It would be helpful if lists:
- Rendered like this when their text wraps
  on long lines.
  - As well as for nested bullets that also have
    long lines. Otherwise the text gets really hard
    to read.

1. And the same with bulleted lists that also have
   very long lines that would otherwise wrap to the
   first column.
2. Like this, which is very ugly and difficult to read
in large lists.

---
HRs should really render as actual horizontal rules and not just three hyphens.

> Much liks lists, it would be great if comments also rendered
  and wrapped in such a way that the indendentation was clear.

> # Another case:
> It would also be nice if comment blocks rendered properly with nested markup,
> so blocks like this don't turn into an unending sea of gray.
> ```python
> # with unformatted code
> ```
> And lists that look like
> - still
> - more
> - gray

# We also need different
## Treatment for each kind
### of header

It would also be nice if long lines rendered naturally as a paragraph when they wrap past the last column in the viewport.
But it would also be nice if paragraphs
that had been auto-wrapped by an editor
or an agent that got overzealous about
formatting everything fastidiously.

I believe it's the markdown norm to only consider paragraph breaks with a newline.
But that's also worth double-checking.

```java
// Some languages can be insanely verbose, so it would help if code didn't wrap within blocks.
public static final native verbose obscene repetitive sealed bespoke class WhyDoJavaAndCSharpProgrammersUseClassNamesLikeThis {
    @dependencyInjectionInsanity(DoubleBuckyDjangoRheinhardtTool.class)
    public static final sealed void dynamic withEvenMoreWeirdlyNamedMethods() {
        // ...
    }
}
```

| Markdown | Tables | Can | Be | Really | useful |
|---|---|---|---|---|---|
| But | My | God | They | Can | Be |
| Such | An | Absolute | Bitch | To |
| Read | And | Write | Reliably | Like | This |

We should at least _render_ tables cleanly, even if we don't have great editing tools yet.

---
▸ 2026-10-17T03:05:33Z [Joel Webber]

Any thing else you can think of in the markdown standard (such as it is) and common extensions?

---
▸ 2026-10-17T04:26:52Z [delta-lead]

Restructured 2026-10-17 under the umbrella yaks-e9de (its description holds the decisions). This yak is now the rendering sub-parent; its body stays the torture-test fixture. Children: yaks-649d (renderer core, pulldown-cmark), yaks-e30d (headings/rules/emphasis), yaks-c733 (code + tables, h-scroll), yaks-3b8d (links + attachments), yaks-1bbb (task lists, alerts, footnotes), yaks-4fa6 (exact-line `E`), yaks-f67e (link `)` bug), yaks-3020 (code-block syntax colors).

Answer to the question above, for the record: beyond the examples here, the plan covers setext headings, hard breaks, lazy continuation, tight/loose lists, ordered start numbers, nested quotes, indented code, escapes/entities, inline HTML (literal), autolinks, reference links, and GFM tables, task lists, strikethrough, alerts and footnotes. Math, emoji shortcodes and definition lists are out.

[before-headless](yaks-ea13/before-headless.txt)

---
▸ 2026-10-17T04:27:08Z [delta-lead]

Baseline: yaks-ea13 as the detail pane renders it today (pre-yaks-649d), headless 100x110 frame.

---
▸ 2026-10-17T04:31:30Z [delta-lead]

moved: hairy -> shaving

---
▸ 2026-10-17T19:55:43Z [delta-lead]

Arc complete: every child shorn (or moved out / slaughtered), released in 0.0.20 (yaks-ade2).

---
▸ 2026-10-17T19:55:57Z [delta-lead]

moved: shaving -> shorn

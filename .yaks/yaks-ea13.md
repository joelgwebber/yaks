---
id: yaks-ea13
title: Markdown rendering improvements
status: hairy
type: task
priority: 3
created: '2026-10-05T22:15:30Z'
updated: '2026-10-17T03:48:58Z'
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

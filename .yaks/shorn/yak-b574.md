---
id: yak-b574
title: Vim key support in text inputs
type: idea
priority: 2
created: '2026-04-15T19:16:39Z'
updated: '2026-04-16T22:45:27Z'
commit: 6139ef5
---

Even simple readline() style vim handling would be helpful, perhaps as an option controlled by config.yaml so it doesn't confuse everyone.
The trickiest part might be figuring out how best to handle cancelling input. Maybe double-esc when in vim mode?

### 2026-04-16T20:56:28Z
Research + design notes (not yet committed).

## Feasibility
Practical with Python curses, but the cleanest path is a minimal "vi-style line editing" subset — not a full vim emulation. Shape of the work:

- Shared `_vim_line_edit()` helper that wraps `_text_edit()` and holds mode state. Callsites are input_prompt, edit_prompt, fuzzy_pick_task, filter drawer text fields, inline search.
- Mode state lives per-input (entering a fresh prompt starts in insert). Global mode state is tempting but surprising when a new prompt opens.

## The Esc problem
Esc is overloaded today:
- input_prompt / edit_prompt: Esc cancels (returns "" / None).
- fuzzy_pick_task: Esc cancels.
- Filter drawer: Esc reverts and closes.
- Inline search: Esc reverts and closes.
- Main list/detail: Esc clears filter / exits detail.

In vim, Esc switches insert→normal. The collision is the main UX risk. Options:
- **Double-Esc to cancel** (as hinted in the description). In insert mode, first Esc enters normal; second Esc cancels. Works but adds latency; vim users conditioned to mash Esc would see Esc→"did nothing" then Esc→cancel, which feels reasonable.
- **Ctrl-C / Ctrl-G to cancel** (emacs-readline convention). Free up Esc entirely for mode switching. Cleaner, but changes the mental model for cancel across the whole TUI — not just vim-mode inputs.
- **Esc cancels in insert mode; normal mode has its own cancel (Q?)**. Fewer keystrokes, but now Esc means different things in different modes of the same input.

My read: double-Esc is least disruptive. Keeps Esc-to-cancel intuition for non-vim callsites and non-insert moments; vim users pay a cheap extra keystroke.

## Cursor shape indicator
Terminals that support DECSCUSR (xterm, iTerm2, Alacritty, kitty, wezterm, modern Terminal.app, gnome-terminal) accept `\x1b[<n> q`:
  1 = blinking block, 2 = steady block, 5 = blinking bar, 6 = steady bar.

curses doesn't expose this, so we'd emit the escape directly via sys.stdout.write. Older terminals silently ignore — graceful degradation is fine.

Fallbacks when cursor shape isn't reliable:
- Prompt-label badge: `[I]` vs `[N]` prefix on the prompt.
- Color shift: prompt/field border changes color in normal mode.

I'd do cursor shape AND a subtle badge. The badge is the authoritative indicator; cursor shape is the nicety.

## Minimal viable vim subset
A first pass that covers 90% of muscle memory:
- **Modes**: insert (default), normal.
- **Normal→Insert**: i, a, A, I, o (o not applicable to single-line though), R (replace).
- **Insert→Normal**: Esc (first of a double-Esc).
- **Normal motions**: h l (char), w b (word), 0 ^ $ (line endpoints).
- **Normal edits**: x (delete char), dd or D (delete line / clear), cw (change word), . (repeat — maybe skip v1).
- **Normal cancel**: second Esc.
- Skip: visual mode, registers, counts, ex commands.

Maybe 150–200 lines of logic, mostly table-driven.

## Scope: which inputs get vim mode?
Three options:
- (a) All text inputs across the TUI.
- (b) Only the "big" ones: filter drawer text rows + edit_prompt.
- (c) Only edit_prompt (where you're editing a long existing value).

(a) is most consistent. (b) avoids cluttering one-shot prompts (search, attach path) where vim-mode barely earns its keep. (c) minimizes surface area but annoys users who expect it everywhere.

My instinct: (a), with the config flag off by default. Users who enable it want it everywhere.

## Configuration
Two axes:
- Per-project (.yaks/config.yaml) vs user-global (~/.config/yaks/config.yaml).
- Key name: `vim_mode: true` or `keybindings: vim` (extensible).

User-global is the right default — this is a personal preference, not a project property. Add a loader that merges user-global → project → env override. For now, start with project-level (already supported) + env var YAKS_VIM=1; add user-global in a follow-up if needed.

## Open questions (for future me / user)
1. Does vim-mode apply in fuzzy_pick_task? The list nav (Ctrl-N/P, arrows) is already vim-ish; only the input line would change.
2. Filter drawer chip rows already support hjkl — should those change in vim mode? Probably not; they're consistent either way.
3. Should normal-mode navigation work on multi-line text (e.g., task descriptions)? None of the current prompts are multi-line, so N/A today.
4. Cursor-shape fallback detection: do we try to query the terminal first, or just always emit and trust that unsupported terminals ignore? The latter is standard practice.

## Recommendation
Proceed as a dedicated future yak, not something to squeeze into polish. Suggested phasing:
1. Phase 1: implement the shared `_vim_line_edit()` helper + cursor shape + double-Esc cancel. Gate behind `YAKS_VIM=1` env var. Apply only to filter drawer text rows + edit_prompt.
2. Phase 2: expand to all text inputs; add config.yaml support.
3. Phase 3 (optional): counts, `.`, more normal-mode verbs.

### 2026-04-16T22:06:41Z
I think double-esc is good for cancellation, especially if we have the cursor right.
Agreed on "all inputs" and per-project + per-user config (I see no need for the env var). Let's leave another yak for in-app configuration to follow up later.
Open questions:
1. I like using it in fuzzy_pick_task for consistency.
2. Maybe we still implement the label/search fields with vim mode?
3. It would be nice if we ever *do* need multiline inputs, but as you point out, not necessary yet.
4. Let's just go with always emitting the cursor-shape escapes. I truly don't care about super old
   terminals.

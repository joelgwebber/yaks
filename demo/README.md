# demo/ — the embedded terminal demo

Build tooling for the animated two-pane demo on the docs site
([`docs/demo.html`](../docs/demo.html)). **Not shipped** in the PyPI package —
this directory only exists to (re)generate a `.cast` file that the vendored
[asciinema-player](../docs/vendor/asciinema-player/) plays back.

## What it is

A coding agent (left pane) and the `yaks` TUI board (right pane) evolving in
lockstep, to show that the point of yaks is **communication and structure**, not
agent autonomy.

We *generate* the recording deterministically instead of capturing a real
terminal. That means:

- No tmux, no asciinema recorder, no flaky re-shoots — just `python3 demo/build_demo.py`.
- Explicit timing: long agent "thinking" compresses to a beat; important state
  changes hold on screen.
- Chapter markers, idealized/simplified agent turns, and a reproducible artifact
  that diffs cleanly in git.

## Files

| File | Role |
|------|------|
| `castkit.py` | Dependency-free asciicast v2 builder + a wide-char-aware virtual `Screen`. |
| `yakscreen.py` | Two-pane composition (agent transcript + yaks board). Imports the real status set + emoji from `scripts/yaklib` and mirrors the TUI's row/tab layout + colors to resist drift. |
| `build_demo.py` | The screenplay (a `Director` over `Cast` + `Board`). Writes `docs/demo.cast`. |

## Rebuild

```sh
python3 demo/build_demo.py          # writes docs/demo.cast
python3 demo/build_demo.py -o /tmp/preview.cast
```

Preview locally (the player fetches `demo.cast`, so it needs a real HTTP origin):

```sh
python3 -m http.server -d docs 8000
# open http://localhost:8000/demo.html
```

## Drift

The renderer pulls `status_emoji` and the status constants from the shipped
source, but the row/tab **layout** and **color mapping** are re-expressed here in
ANSI (the real ones are curses-bound). If you restyle the TUI in
`scripts/yaktui/render.py` or `colors.py`, glance at `yakscreen.py` to keep the
demo honest.

## Updating the vendored player

The player bundle is pinned under `docs/vendor/asciinema-player/`
(asciinema-player 3.6.3). To bump it, re-download `asciinema-player.min.js` and
`asciinema-player.css` from the matching release.

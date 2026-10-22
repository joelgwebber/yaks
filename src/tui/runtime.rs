//! Terminal lifecycle + the event loop: `run` (the public entry point),
//! `event_loop`, the `.yaks/` filesystem watcher, and raw-mode `setup`/`restore`.
//! Split out of `tui.rs` (yaks-b1cc / b1cc/8). `use super::*` inherits `tui.rs`'s
//! scope (App, handle_key, render, the crossterm/notify imports, …).

use super::*;
use ratatui::crossterm::event::{DisableMouseCapture, EnableMouseCapture};
pub fn run(mut app: App) -> Result<()> {
    let (mut term, kitty) = setup()?;
    // Ask the terminal which image protocol it speaks and its cell size. Needs
    // raw mode and must run before any event is read; a terminal that does not
    // answer keeps the half-block fallback (yaks-4305).
    if let Ok(picker) = ratatui_image::picker::Picker::from_query_stdio() {
        app.picker = picker;
    }
    let res = event_loop(&mut term, &mut app, kitty);
    let _ = restore(kitty);
    crate::clipboard::release();
    res
}

fn event_loop(
    term: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
    kitty: bool,
) -> Result<()> {
    // Best-effort filesystem watch on the farm's `.yaks/` tree so external edits
    // (including the user's own, from another process) refresh the UI. Kept
    // alive for the loop's duration; `_watcher` must not be dropped early.
    let watch_path = app.farm.as_ref().map(|h| h.root().to_path_buf());
    let (_watcher, rx) = setup_watcher(watch_path);
    let mut dirty = false;
    loop {
        // Apply external changes only while idle (no overlay), so we never yank
        // data out from under an open editor/picker mid-interaction.
        if dirty && matches!(app.overlay, Overlay::None) {
            app.reload_preserving_selection();
            dirty = false;
        }
        // The Sheds tab's background scan: pick up a finished one, start a
        // due one. Never blocks (yaks-98f1).
        app.settle_sheds(false);
        term.draw(|f| render(app, f))?;
        // A `W`/`U` commit or sync queued by the last key: its `committing…` /
        // `syncing…` frame is now on screen, so run it (blocking) and redraw.
        if app.pending_farm_op.is_some() {
            app.run_pending_farm_op();
            continue;
        }
        // Record the main-area height (minus tab + help lines) for paging.
        let h = term.size()?.height;
        app.page = h.saturating_sub(2).max(1);
        app.detail_page = h.saturating_sub(3).max(1);
        // Block for input, but wake periodically to service filesystem events.
        // Then drain whatever else is already queued before redrawing: one
        // frame per burst, not per key, so a paste arriving as a keystroke
        // stream isn't echoed at one char per full redraw (yaks-f2aa).
        if event::poll(Duration::from_millis(250))? {
            let first = event::read()?;
            pump_events(app, first, Duration::from_millis(50), || {
                if event::poll(Duration::ZERO)? {
                    event::read().map(Some)
                } else {
                    Ok(None)
                }
            })?;
        }
        // An editor queued by following a source link (yaks-da5e): run it on
        // the real terminal, then take the screen back.
        if let Some(cmd) = app.pending_editor.take() {
            run_suspended(term, kitty, &cmd)?;
        }
        // Coalesce any pending fs notifications into one deferred refresh.
        if let Some(rx) = &rx {
            while rx.try_recv().is_ok() {
                dirty = true;
            }
        }
        if app.quit {
            break;
        }
    }
    Ok(())
}

/// Set up a recursive watcher on `path`, returning the watcher (which the caller
/// must keep alive) and a receiver that yields once per content-changing event.
/// Best-effort: any failure yields `(None, None)` and the TUI just won't
/// auto-refresh. Access/metadata-only events are filtered out so that our own
/// reads can't trigger a reload feedback loop.
fn setup_watcher(path: Option<PathBuf>) -> (Option<RecommendedWatcher>, Option<Receiver<()>>) {
    let Some(path) = path else {
        return (None, None);
    };
    let (tx, rx) = mpsc::channel();
    let handler = move |res: notify::Result<notify::Event>| {
        if let Ok(ev) = res {
            let interesting = match ev.kind {
                EventKind::Create(_) | EventKind::Remove(_) => true,
                EventKind::Modify(ModifyKind::Metadata(_)) => false,
                EventKind::Modify(_) => true,
                _ => false, // Access / Any / Other
            };
            if interesting {
                let _ = tx.send(());
            }
        }
    };
    let mut watcher = match notify::recommended_watcher(handler) {
        Ok(w) => w,
        Err(_) => return (None, None),
    };
    if watcher.watch(&path, RecursiveMode::Recursive).is_err() {
        return (None, None);
    }
    (Some(watcher), Some(rx))
}

// -- terminal lifecycle ---------------------------------------------------

/// Hand the terminal to `cmd` (an editor) and take it back when it exits:
/// leave the alternate screen and raw mode, wait for it, re-enter, and force a
/// full repaint. A failure to start it is not fatal; the TUI carries on.
fn run_suspended(
    term: &mut Terminal<CrosstermBackend<Stdout>>,
    kitty: bool,
    cmd: &source::ExternalCmd,
) -> Result<()> {
    restore(kitty)?;
    let _ = std::process::Command::new(&cmd.program)
        .args(&cmd.args)
        .status();
    enter_modes()?;
    term.clear()?;
    Ok(())
}

fn setup() -> Result<(Terminal<CrosstermBackend<Stdout>>, bool)> {
    let kitty = enter_modes()?;
    let term = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = restore(kitty);
        prev(info);
    }));
    Ok((term, kitty))
}

/// Raw mode, alternate screen, mouse, paste and keyboard-enhancement flags;
/// returns whether the kitty keyboard protocol is on. [`restore`] undoes it.
fn enter_modes() -> Result<bool> {
    enable_raw_mode()?;
    let mut out = io::stdout();
    execute!(out, EnterAlternateScreen)?;
    // Mouse capture (yaks-97a2): wheel + clicks. Most terminals still do native
    // text selection with Shift held while capture is on.
    let _ = execute!(out, EnableMouseCapture);
    // Bracketed paste: a terminal paste arrives as one `Event::Paste` we insert
    // in bulk, instead of a keystroke per char (yaks-f2aa).
    let _ = execute!(out, EnableBracketedPaste);
    let kitty = supports_keyboard_enhancement().unwrap_or(false);
    if kitty {
        let _ = execute!(
            out,
            PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
        );
    }
    Ok(kitty)
}

fn restore(kitty: bool) -> Result<()> {
    let mut out = io::stdout();
    if kitty {
        let _ = execute!(out, PopKeyboardEnhancementFlags);
    }
    let _ = execute!(out, DisableMouseCapture);
    let _ = execute!(out, DisableBracketedPaste);
    execute!(out, LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

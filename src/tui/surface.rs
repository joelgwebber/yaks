//! Highlight surfaces: the subtle background bands behind a selected row, the
//! line cursor, and the pinned header (yaks-48e9).
//!
//! A band has to be a *background*, and a fixed grey is wrong on one of dark or
//! light terminals (`Indexed(237)` is a black slab on a light theme). So the band
//! is derived from the terminal's own colours: the real background blended a
//! little toward the real foreground, which is subtle on any theme, Solarized and
//! Gruvbox included. The palette in [`super::palette`] stays 16-ANSI-only because
//! it only sets *foregrounds*; backgrounds come from here.
//!
//! [`init`] runs once at startup (it queries the terminal, so it needs raw mode
//! and runs before any event is read). Until then, and whenever nothing can be
//! learned, the dark-terminal greys the TUI always used apply; that is also what
//! the test snapshots see.
//!
//! Resolution order: `YAKS_THEME=dark|light`, then the terminal's reported
//! colours (OSC 10/11), then `COLORFGBG`, then dark.

use std::sync::OnceLock;

use ratatui::style::Color;

/// How far the band moves from the background toward the foreground, in 1/100.
const SELECTED_PCT: u32 = 18;
const BAND_PCT: u32 = 12;

type Rgb = (u8, u8, u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Surfaces {
    /// Blend toward the foreground from a known background.
    Measured {
        bg: Rgb,
        fg: Rgb,
        truecolor: bool,
    },
    /// Only dark or light is known: fixed 256-colour greys.
    Dark,
    Light,
}

static SURFACES: OnceLock<Surfaces> = OnceLock::new();

fn current() -> Surfaces {
    #[cfg(test)]
    if let Some(s) = TEST_OVERRIDE.with(|o| o.get()) {
        return s;
    }
    SURFACES.get().copied().unwrap_or(Surfaces::Dark)
}

#[cfg(test)]
thread_local! {
    static TEST_OVERRIDE: std::cell::Cell<Option<Surfaces>> = const { std::cell::Cell::new(None) };
}

/// Run `f` as if the terminal reported this background and foreground (always
/// truecolor): lets a doc shot draw the real frame on a light terminal. Per
/// thread, so parallel tests never see it.
#[cfg(test)]
pub(crate) fn with_terminal<R>(bg: Rgb, fg: Rgb, f: impl FnOnce() -> R) -> R {
    let s = Surfaces::Measured {
        bg,
        fg,
        truecolor: true,
    };
    let prev = TEST_OVERRIDE.with(|o| o.replace(Some(s)));
    let r = f();
    TEST_OVERRIDE.with(|o| o.set(prev));
    r
}

/// The strong band: the focused list's selected row, the detail line cursor,
/// a selection, the focused form choice.
pub fn selected() -> Color {
    resolve(current(), SELECTED_PCT, 237, 253)
}

/// The quieter band: an unfocused list's selected row, the pinned header.
pub fn band() -> Color {
    resolve(current(), BAND_PCT, 236, 254)
}

fn resolve(s: Surfaces, pct: u32, dark: u8, light: u8) -> Color {
    match s {
        Surfaces::Dark => Color::Indexed(dark),
        Surfaces::Light => Color::Indexed(light),
        Surfaces::Measured { bg, fg, truecolor } => {
            let (r, g, b) = blend(bg, fg, pct);
            if truecolor {
                Color::Rgb(r, g, b)
            } else {
                Color::Indexed(nearest_256((r, g, b)))
            }
        }
    }
}

fn blend(bg: Rgb, fg: Rgb, pct: u32) -> Rgb {
    let mix = |a: u8, b: u8| {
        let (a, b) = (a as i32, b as i32);
        (a + (b - a) * pct as i32 / 100) as u8
    };
    (mix(bg.0, fg.0), mix(bg.1, fg.1), mix(bg.2, fg.2))
}

/// The xterm-256 index closest to `c`, over the 6x6x6 cube (16..=231) and the
/// grey ramp (232..=255); the 16 ANSI colours are the user's theme, not ours.
fn nearest_256(c: Rgb) -> u8 {
    const LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];
    let dist = |p: Rgb| {
        let d = |a: u8, b: u8| (a as i32 - b as i32).pow(2);
        d(p.0, c.0) + d(p.1, c.1) + d(p.2, c.2)
    };
    let mut best = (i32::MAX, 16u8);
    for (i, &r) in LEVELS.iter().enumerate() {
        for (j, &g) in LEVELS.iter().enumerate() {
            for (k, &b) in LEVELS.iter().enumerate() {
                best = best.min((dist((r, g, b)), 16 + (36 * i + 6 * j + k) as u8));
            }
        }
    }
    for n in 0..24u8 {
        let v = 8 + 10 * n;
        best = best.min((dist((v, v, v)), 232 + n));
    }
    best.1
}

/// Work out the terminal's surfaces and fix them for the process. Idempotent:
/// the first call wins.
pub fn init() {
    let _ = SURFACES.set(detect(
        std::env::var("YAKS_THEME").ok().as_deref(),
        std::env::var("COLORFGBG").ok().as_deref(),
        std::env::var("COLORTERM").ok().as_deref(),
        query_terminal,
    ));
}

/// The terminal's foreground and background, via OSC 10/11. `None` when it does
/// not answer (tmux without passthrough, a dumb terminal, a slow link).
fn query_terminal() -> Option<(Rgb, Rgb)> {
    let p =
        terminal_colorsaurus::color_palette(terminal_colorsaurus::QueryOptions::default()).ok()?;
    Some((p.foreground.scale_to_8bit(), p.background.scale_to_8bit()))
}

fn detect(
    theme: Option<&str>,
    colorfgbg: Option<&str>,
    colorterm: Option<&str>,
    query: impl FnOnce() -> Option<(Rgb, Rgb)>,
) -> Surfaces {
    match theme.map(str::to_ascii_lowercase).as_deref() {
        Some("dark") => return Surfaces::Dark,
        Some("light") => return Surfaces::Light,
        _ => {}
    }
    if let Some((fg, bg)) = query() {
        let truecolor = matches!(colorterm, Some("truecolor" | "24bit"));
        return Surfaces::Measured { bg, fg, truecolor };
    }
    // `COLORFGBG=fg;bg` (rxvt, Konsole, some others): ANSI index of the background.
    // 0-6 and 8 are dark; 7 and 9-15 are light.
    match colorfgbg
        .and_then(|v| v.rsplit(';').next())
        .and_then(|v| v.parse::<u8>().ok())
    {
        Some(7 | 9..=15) => Surfaces::Light,
        _ => Surfaces::Dark,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DARK_BG: Rgb = (0x1e, 0x1e, 0x1e);
    const DARK_FG: Rgb = (0xd4, 0xd4, 0xd4);
    const LIGHT_BG: Rgb = (0xff, 0xff, 0xff);
    const LIGHT_FG: Rgb = (0x33, 0x33, 0x33);

    fn lum(c: Rgb) -> u32 {
        c.0 as u32 + c.1 as u32 + c.2 as u32
    }

    #[test]
    fn unset_is_the_dark_greys_the_snapshots_rely_on() {
        assert_eq!(
            resolve(Surfaces::Dark, SELECTED_PCT, 237, 253),
            Color::Indexed(237)
        );
        assert_eq!(
            resolve(Surfaces::Dark, BAND_PCT, 236, 254),
            Color::Indexed(236)
        );
    }

    #[test]
    fn a_band_sits_between_background_and_foreground_on_both_themes() {
        for (bg, fg) in [(DARK_BG, DARK_FG), (LIGHT_BG, LIGHT_FG)] {
            let s = Surfaces::Measured {
                bg,
                fg,
                truecolor: true,
            };
            let (Color::Rgb(r, g, b), Color::Rgb(r2, g2, b2)) =
                (resolve(s, SELECTED_PCT, 0, 0), resolve(s, BAND_PCT, 0, 0))
            else {
                panic!("truecolor gives Rgb")
            };
            let (sel, band) = (lum((r, g, b)), lum((r2, g2, b2)));
            // Moves toward the foreground, the strong band further than the quiet one.
            let (lo, hi) = (lum(bg).min(lum(fg)), lum(bg).max(lum(fg)));
            assert!(sel > lo && sel < hi && band > lo && band < hi);
            assert!(lum(bg).abs_diff(sel) > lum(bg).abs_diff(band));
        }
        // On a light terminal the band is darker than the background, not a slab.
        let s = Surfaces::Measured {
            bg: LIGHT_BG,
            fg: LIGHT_FG,
            truecolor: true,
        };
        let Color::Rgb(r, g, b) = resolve(s, SELECTED_PCT, 0, 0) else {
            panic!()
        };
        assert!(
            lum((r, g, b)) > 600,
            "light-theme band stays light: {r},{g},{b}"
        );
    }

    #[test]
    fn without_truecolor_the_band_is_a_256_grey_or_cube_index() {
        let s = Surfaces::Measured {
            bg: LIGHT_BG,
            fg: LIGHT_FG,
            truecolor: false,
        };
        assert!(matches!(resolve(s, SELECTED_PCT, 0, 0), Color::Indexed(i) if i >= 16));
        assert_eq!(nearest_256((0x3a, 0x3a, 0x3a)), 237);
        assert_eq!(nearest_256((255, 255, 255)), 231);
        assert_eq!(nearest_256((255, 0, 0)), 196);
    }

    #[test]
    fn detection_order_is_override_then_terminal_then_colorfgbg_then_dark() {
        let answer = Some((DARK_FG, DARK_BG));
        // The override wins and the terminal is never asked.
        assert_eq!(
            detect(Some("LIGHT"), None, None, || panic!("must not query")),
            Surfaces::Light
        );
        assert_eq!(
            detect(Some("dark"), Some("0;15"), None, || answer),
            Surfaces::Dark
        );
        // The terminal beats COLORFGBG; truecolor follows COLORTERM.
        assert_eq!(
            detect(None, Some("0;15"), Some("truecolor"), || answer),
            Surfaces::Measured {
                bg: DARK_BG,
                fg: DARK_FG,
                truecolor: true
            }
        );
        assert_eq!(
            detect(None, None, None, || answer),
            Surfaces::Measured {
                bg: DARK_BG,
                fg: DARK_FG,
                truecolor: false
            }
        );
        // No answer: COLORFGBG, else dark.
        assert_eq!(detect(None, Some("0;15"), None, || None), Surfaces::Light);
        assert_eq!(detect(None, Some("15;0"), None, || None), Surfaces::Dark);
        assert_eq!(detect(None, Some("junk"), None, || None), Surfaces::Dark);
        assert_eq!(detect(None, None, None, || None), Surfaces::Dark);
    }
}

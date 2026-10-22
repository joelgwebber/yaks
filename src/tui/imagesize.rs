//! Image size syntax and the cell-size rule for markdown images (yaks-89fc).
//!
//! An author sizes an image with a trailing token in its alt text, the one
//! form that is valid CommonMark and degrades to harmless alt text elsewhere:
//! `![alt =50%](p)`, `![alt =300x200](p)`, `![alt =x120](p)`. [`split_alt`]
//! parses it; [`cells`] turns a size (or the lack of one) into a cell count.
//! Both are pure: nothing here reads a file or draws.

/// One dimension of a requested image size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dim {
    /// Pixels (a bare number, or with `px`): over the cell size in a terminal.
    Px(u32),
    /// Percent of the pane's content width. The grammar only produces it for a
    /// width; a `Percent` height (built by hand) is read as a percent of the
    /// image's natural height.
    Percent(u32),
}

/// A parsed size token. At least one dimension is set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageSize {
    pub width: Option<Dim>,
    pub height: Option<Dim>,
}

/// Split a trailing size token off an image's alt text. Returns the alt
/// without the token (and the whitespace before it) and the parsed size, or
/// the alt unchanged and `None` when its last word is not a size.
///
/// The token is the last whitespace-separated word: `=` then `W`, `xH` or
/// `WxH`. `W` is digits with an optional `%` or `px`; `H` is digits with an
/// optional `px`. Bare digits are pixels; zero is not a size.
pub fn split_alt(alt: &str) -> (&str, Option<ImageSize>) {
    let trimmed = alt.trim_end();
    let start = trimmed.rfind(char::is_whitespace).map_or(0, |i| {
        i + trimmed[i..].chars().next().map_or(1, char::len_utf8)
    });
    match parse_token(&trimmed[start..]) {
        Some(size) => (trimmed[..start].trim_end(), Some(size)),
        None => (alt, None),
    }
}

fn parse_token(word: &str) -> Option<ImageSize> {
    let rest = word.strip_prefix('=')?;
    let (width, rest) = match digits(rest) {
        Some((n, after)) => {
            let (dim, after) = if let Some(a) = after.strip_prefix('%') {
                (Dim::Percent(n), a)
            } else {
                (Dim::Px(n), after.strip_prefix("px").unwrap_or(after))
            };
            (Some(dim), after)
        }
        None => (None, rest),
    };
    let height = match rest.strip_prefix('x') {
        Some(h) => {
            let (n, after) = digits(h)?;
            let after = after.strip_prefix("px").unwrap_or(after);
            if !after.is_empty() {
                return None;
            }
            Some(Dim::Px(n))
        }
        None if rest.is_empty() => None,
        None => return None,
    };
    // `=` alone, or `=px`: nothing was asked for.
    (width.is_some() || height.is_some()).then_some(ImageSize { width, height })
}

/// A leading run of ASCII digits as a non-zero number, and what follows it.
/// `None` when there are no digits, the number is zero, or it overflows.
fn digits(s: &str) -> Option<(u32, &str)> {
    let end = s.bytes().take_while(u8::is_ascii_digit).count();
    let n: u32 = s[..end].parse().ok()?;
    (n > 0).then_some((n, &s[end..]))
}

/// How many cells an image takes, at least `(1, 1)`.
///
/// - No size: natural size (`natural_px` over `cell_px`), shrunk to fit
///   `pane_cols` and `max_rows`, never enlarged.
/// - Width only (or height only): that size, width clamped to the pane, the
///   other side from the aspect ratio. No row cap: an explicit size is the
///   author's call.
/// - Both: the largest size that fits inside the `W`x`H` box with the aspect
///   kept.
pub fn cells(
    size: Option<ImageSize>,
    natural_px: (u32, u32),
    cell_px: (u16, u16),
    pane_cols: u16,
    max_rows: u16,
) -> (u16, u16) {
    let (nw, nh) = (
        f64::from(natural_px.0.max(1)),
        f64::from(natural_px.1.max(1)),
    );
    let (cw, ch) = (f64::from(cell_px.0.max(1)), f64::from(cell_px.1.max(1)));
    let pane = pane_cols.max(1);
    // Cells for a width / height request, rounded up so nothing is cropped.
    let w_cells = |d: Dim| {
        match d {
            Dim::Px(px) => up(f64::from(px) / cw),
            Dim::Percent(p) => up(f64::from(pane) * f64::from(p) / 100.0),
        }
        .min(pane)
    };
    let h_cells = |d: Dim| match d {
        Dim::Px(px) => up(f64::from(px) / ch),
        Dim::Percent(p) => up(nh / ch * f64::from(p) / 100.0),
    };
    // Scale `nw`x`nh` pixels by `scale` and report it in cells, within a box.
    let scaled = |scale: f64, box_cols: u16, box_rows: u16| {
        (
            up(nw * scale / cw).min(box_cols),
            up(nh * scale / ch).min(box_rows),
        )
    };
    let size = size.filter(|s| s.width.is_some() || s.height.is_some());
    match size.map(|s| (s.width, s.height)) {
        None => {
            let scale = (f64::from(pane) * cw / nw)
                .min(f64::from(max_rows.max(1)) * ch / nh)
                .min(1.0);
            scaled(scale, pane, u16::MAX)
        }
        Some((Some(w), None)) => {
            let cols = w_cells(w);
            scaled(f64::from(cols) * cw / nw, cols, u16::MAX)
        }
        Some((None, Some(h))) => {
            let rows = h_cells(h);
            let scale = (f64::from(rows) * ch / nh).min(f64::from(pane) * cw / nw);
            scaled(scale, pane, rows)
        }
        Some((Some(w), Some(h))) => {
            let (cols, rows) = (w_cells(w), h_cells(h));
            let scale = (f64::from(cols) * cw / nw).min(f64::from(rows) * ch / nh);
            scaled(scale, cols, rows)
        }
        Some((None, None)) => unreachable!("filtered above"),
    }
}

/// Round a cell count up (a float that is a whole number plus noise stays
/// whole) into `1..=u16::MAX`.
fn up(x: f64) -> u16 {
    (x - 1e-6).ceil().clamp(1.0, f64::from(u16::MAX)) as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    fn px(w: u32, h: u32) -> ImageSize {
        ImageSize {
            width: Some(Dim::Px(w)),
            height: Some(Dim::Px(h)),
        }
    }
    fn w(d: Dim) -> Option<ImageSize> {
        Some(ImageSize {
            width: Some(d),
            height: None,
        })
    }
    fn h(d: Dim) -> Option<ImageSize> {
        Some(ImageSize {
            width: None,
            height: Some(d),
        })
    }

    #[test]
    fn split_alt_forms() {
        assert_eq!(split_alt("alt =50%"), ("alt", w(Dim::Percent(50))));
        assert_eq!(split_alt("alt =300x200"), ("alt", Some(px(300, 200))));
        assert_eq!(split_alt("alt =x120"), ("alt", h(Dim::Px(120))));
        assert_eq!(split_alt("alt =300"), ("alt", w(Dim::Px(300))));
        assert_eq!(split_alt("=50%"), ("", w(Dim::Percent(50))));
        assert_eq!(
            split_alt("two words =80%"),
            ("two words", w(Dim::Percent(80)))
        );
    }

    #[test]
    fn split_alt_units() {
        assert_eq!(split_alt("a =300px"), ("a", w(Dim::Px(300))));
        assert_eq!(split_alt("a =300pxx200px"), ("a", Some(px(300, 200))));
        assert_eq!(split_alt("a =x120px"), ("a", h(Dim::Px(120))));
        assert_eq!(
            split_alt("a =50%x120"),
            (
                "a",
                Some(ImageSize {
                    width: Some(Dim::Percent(50)),
                    height: Some(Dim::Px(120)),
                })
            )
        );
        // Over 100% is accepted; `cells` clamps to the pane.
        assert_eq!(split_alt("a =150%"), ("a", w(Dim::Percent(150))));
    }

    #[test]
    fn split_alt_zero_and_junk_are_not_sizes() {
        for alt in [
            "a =0",
            "a =0%",
            "a =0x10",
            "a =10x0",
            "a =x0",
            "a =",
            "a =x",
            "a =px",
            "a =300x",
            "a =10%%",
            "a =10%x",
            "a =x10%",
            "a =1e3",
            "a =-5",
            "a =12abc",
            "a =300x200x1",
            "a 300x200",
            "a =5 0%",
            "a =99999999999",
        ] {
            assert_eq!(split_alt(alt), (alt, None), "{alt:?}");
        }
        assert_eq!(split_alt(""), ("", None));
        assert_eq!(split_alt("   "), ("   ", None));
    }

    #[test]
    fn split_alt_token_must_be_last() {
        assert_eq!(split_alt("a =50% b"), ("a =50% b", None));
        assert_eq!(split_alt("=50% shot"), ("=50% shot", None));
    }

    #[test]
    fn split_alt_equals_in_ordinary_text_is_not_a_size() {
        assert_eq!(split_alt("a = b"), ("a = b", None));
        assert_eq!(split_alt("x=5"), ("x=5", None));
        assert_eq!(split_alt("a ="), ("a =", None));
    }

    #[test]
    fn split_alt_trailing_space_and_non_ascii() {
        assert_eq!(split_alt("café =50% "), ("café", w(Dim::Percent(50))));
        assert_eq!(split_alt("日本\u{3000}=50%"), ("日本", w(Dim::Percent(50))));
        assert_eq!(split_alt("a =５０"), ("a =５０", None));
    }

    // cells: 10x20 px cells, a 80-col pane (800 px wide), 25 rows max.
    const CELL: (u16, u16) = (10, 20);

    #[test]
    fn cells_default_shrinks_to_pane_width() {
        // 1600x400 px is 160x20 cells: half of it fits.
        assert_eq!(cells(None, (1600, 400), CELL, 80, 25), (80, 10));
    }

    #[test]
    fn cells_default_never_enlarges() {
        assert_eq!(cells(None, (100, 40), CELL, 80, 25), (10, 2));
        // A fractional natural size rounds up, never down.
        assert_eq!(cells(None, (105, 41), CELL, 80, 25), (11, 3));
    }

    #[test]
    fn cells_default_caps_the_height() {
        // 400x2000 px is 40x100 cells: a quarter fits 25 rows.
        assert_eq!(cells(None, (400, 2000), CELL, 80, 25), (10, 25));
    }

    #[test]
    fn cells_width_only() {
        // 200 px wide of a 2:1 image: 20 cols, 100 px tall.
        assert_eq!(cells(w(Dim::Px(200)), (400, 200), CELL, 80, 25), (20, 5));
        assert_eq!(
            cells(w(Dim::Percent(50)), (400, 200), CELL, 80, 25),
            (40, 10)
        );
        // Enlarges, and ignores the row cap.
        assert_eq!(cells(w(Dim::Px(800)), (100, 100), CELL, 80, 5), (80, 40));
    }

    #[test]
    fn cells_height_only() {
        // 100 px tall of a 2:1 image: 5 rows, 200 px wide.
        assert_eq!(cells(h(Dim::Px(100)), (400, 200), CELL, 80, 3), (20, 5));
    }

    #[test]
    fn cells_both_fit_inside_the_box_keeping_aspect() {
        // A 400x100 px box around a square image: the 100 px height decides.
        assert_eq!(cells(Some(px(400, 100)), (400, 400), CELL, 80, 25), (10, 5));
        // A 100x400 px box: the width decides.
        assert_eq!(cells(Some(px(100, 400)), (400, 400), CELL, 80, 25), (10, 5));
        // A box larger than the image enlarges it to the box.
        assert_eq!(
            cells(Some(px(400, 400)), (100, 100), CELL, 80, 25),
            (40, 20)
        );
    }

    #[test]
    fn cells_clamp_to_the_pane() {
        assert_eq!(cells(w(Dim::Px(2000)), (1000, 500), CELL, 80, 25), (80, 20));
        assert_eq!(
            cells(w(Dim::Percent(150)), (1000, 500), CELL, 80, 25),
            (80, 20)
        );
        // A height that would need more than the pane's width shrinks to it.
        assert_eq!(cells(h(Dim::Px(1000)), (1000, 500), CELL, 80, 25), (80, 20));
        assert_eq!(
            cells(Some(px(2000, 1000)), (1000, 500), CELL, 80, 25),
            (80, 20)
        );
    }

    #[test]
    fn cells_never_below_one_by_one() {
        assert_eq!(cells(None, (1, 1), CELL, 80, 25), (1, 1));
        assert_eq!(cells(None, (0, 0), CELL, 80, 25), (1, 1));
        assert_eq!(cells(None, (1000, 1000), CELL, 0, 0), (1, 1));
        assert_eq!(cells(w(Dim::Px(1)), (1000, 10), CELL, 80, 25), (1, 1));
        assert_eq!(cells(h(Dim::Px(1)), (10, 1000), CELL, 80, 25), (1, 1));
        // A zero cell size reads as 1x1 px: 100x100 cells, capped to 25 rows.
        assert_eq!(cells(None, (100, 100), (0, 0), 80, 25), (25, 25));
    }
}

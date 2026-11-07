//! The image viewer (`DocView::Image`, yaks-4305): an image attachment
//! decoded once on open and drawn in the right pane by `ratatui-image`, over
//! the terminal's own graphics protocol (kitty, sixel, iTerm2) when it speaks
//! one, else as half-block cells. Headless frames (toque, tests) always take
//! the half-block path: it is ordinary colored cells, so it lands in text and
//! SVG snapshots like anything else.
//!
//! The picture is zoomed and panned by this module, not by `ratatui-image`
//! (yaks-e5f9): each draw crops the visible window out of the decoded pixels,
//! scales it to the pane's pixel grid and hands the library a finished image,
//! so every protocol zooms the same way. The grid is the terminal's cell size
//! in pixels, except for half-blocks, whose "pixel" is one column by half a
//! row (so `100%` is one image pixel per half-block, not per 10x20 cell).

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui_image::picker::{Picker, ProtocolType};
use ratatui_image::protocol::Protocol;
use ratatui_image::sliced::{SignedPosition, SlicedImage, SlicedProtocol};
use ratatui_image::{FilterType, FontSize, Image, Resize};

use super::keymap::Cmd;
use super::{App, detail};

/// Largest image file the viewer decodes; a bigger one opens externally.
const IMAGE_MAX_BYTES: u64 = 32 << 20;

/// Largest svg the viewer rasterizes. Far below [`IMAGE_MAX_BYTES`]: an svg is
/// text that expands to pixels, and rasterizing blocks the UI.
const SVG_MAX_BYTES: u64 = 4 << 20;

/// The PNG of svg `svg`, rasterized by toque with its embedded fonts (so the
/// same on every machine), cached by content under `cache_base`. A miss
/// rasterizes and writes the cache best-effort: a read-only or missing cache
/// directory only costs the next open a re-render (a few ms).
pub(super) fn svg_png(svg: &[u8], cache_base: Option<&Path>) -> Option<Vec<u8>> {
    let cached = cache_base.map(|b| super::cache::svg_png_path_in(b, svg));
    if let Some(png) = cached.as_ref().and_then(|p| std::fs::read(p).ok()) {
        return Some(png);
    }
    let png = toque::svg_to_png(svg)?;
    if let Some(p) = &cached {
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(p, &png);
    }
    Some(png)
}

/// The text of the svg at `path`, or `None` for one over [`SVG_MAX_BYTES`].
pub(super) fn read_svg(path: &Path) -> Option<Vec<u8>> {
    if std::fs::metadata(path).ok()?.len() > SVG_MAX_BYTES {
        return None;
    }
    std::fs::read(path).ok()
}

/// Decode an image attachment to pixels: an svg through toque's rasterizer (and
/// the PNG cache), anything else through the `image` crate with the format
/// guessed from the content.
fn decode_attachment(path: &Path, name: &str) -> Option<image::DynamicImage> {
    let is_svg = Path::new(name)
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("svg"));
    if is_svg {
        let png = svg_png(&read_svg(path)?, super::cache::svg_cache_base().as_deref())?;
        return image::load_from_memory(&png).ok();
    }
    image::ImageReader::open(path)
        .ok()?
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()
}

/// How the picture is scaled: to the pane, to its width, or by a factor.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Zoom {
    /// Shrink to fit the pane, never enlarge (the default).
    Fit,
    /// Scale to the pane's width (enlarging a narrow image); a taller image
    /// then pans down.
    FitWidth,
    /// A fixed factor: `1.0` is one image pixel per pixel of the grid.
    Scale(f64),
}

/// The factors `+` and `-` step through (1.0 is actual size).
const ZOOM_STEPS: [f64; 19] = [
    0.05, 0.1, 0.125, 0.25, 0.33, 0.5, 0.67, 0.75, 1.0, 1.5, 2.0, 3.0, 4.0, 6.0, 8.0, 12.0, 16.0,
    24.0, 32.0,
];

/// The factor for `zoom` showing an `img`-pixel picture in a `pane`-pixel
/// window. An unmeasured pane (no frame drawn yet) scales nothing.
fn scale_for(zoom: Zoom, img: (u32, u32), pane: (u32, u32)) -> f64 {
    let (iw, ih) = (f64::from(img.0.max(1)), f64::from(img.1.max(1)));
    let (pw, ph) = (f64::from(pane.0), f64::from(pane.1));
    match zoom {
        Zoom::Scale(s) => s,
        _ if pw == 0.0 || ph == 0.0 => 1.0,
        Zoom::Fit => (pw / iw).min(ph / ih).min(1.0),
        Zoom::FitWidth => pw / iw,
    }
}

/// The picture's size in pixels at `scale`.
fn shown_size(img: (u32, u32), scale: f64) -> (u32, u32) {
    let side = |n: u32| ((f64::from(n) * scale).round() as u32).max(1);
    (side(img.0), side(img.1))
}

/// The top-left of the window, kept inside the picture: a picture no larger
/// than the pane on an axis has nothing to pan there.
fn clamp_offset(off: (u32, u32), shown: (u32, u32), pane: (u32, u32)) -> (u32, u32) {
    (
        off.0.min(shown.0.saturating_sub(pane.0)),
        off.1.min(shown.1.saturating_sub(pane.1)),
    )
}

/// The pixel grid a picker's cells are drawn on: the font size, but one
/// column by half a row for half-blocks (the library draws those two pixels
/// to a cell whatever the font).
fn grid_of(picker: &Picker) -> FontSize {
    match picker.protocol_type() {
        ProtocolType::Halfblocks => FontSize::new(1, 2),
        _ => picker.font_size(),
    }
}

/// What the cached [`Protocol`] was built for: the factor, the window's
/// offset and the pane's cells.
type DrawKey = (u64, (u32, u32), (u16, u16));

/// An image attachment open in the viewer.
pub(crate) struct ImageView {
    /// The link target as written (`<id>/<file>`), shown as the header.
    pub(crate) target: String,
    pub(crate) path: PathBuf,
    /// Pixel size of the decoded image, shown in the header.
    dims: (u32, u32),
    image: image::DynamicImage,
    /// The picker, its half-block grid corrected by [`grid_of`] (see
    /// [`ImageView::open`]).
    picker: Picker,
    grid: FontSize,
    zoom: Cell<Zoom>,
    /// The window's top-left in pixels of the zoomed picture. Clamped when
    /// drawn too, so a pane that grows pulls it back.
    offset: Cell<(u32, u32)>,
    /// The pane's cells at the last draw (0 before the first), which zoom
    /// and pan measure against; `render` takes `&App`, so these are `Cell`s.
    pane: Cell<(u16, u16)>,
    /// The window's finished protocol, rebuilt only when its [`DrawKey`]
    /// changes, since a sixel or kitty encode is not free.
    drawn: RefCell<Option<(DrawKey, Option<Protocol>)>>,
}

impl ImageView {
    /// The viewer for `found`, or `None` when it should open externally: not
    /// an image by name, over [`IMAGE_MAX_BYTES`], or one that cannot be
    /// decoded (a corrupt file, a malformed svg, a format built without).
    pub(crate) fn open(
        target: &str,
        found: &crate::farm::FoundAttachment,
        picker: &Picker,
    ) -> Option<Self> {
        if !crate::store::is_image_name(&found.name) {
            return None;
        }
        if std::fs::metadata(&found.path).ok()?.len() > IMAGE_MAX_BYTES {
            return None;
        }
        let img = decode_attachment(&found.path, &found.name)?;
        let mut picker = picker.clone();
        let grid = grid_of(&picker);
        if picker.protocol_type() == ProtocolType::Halfblocks {
            // The library sizes an image to cells by the picker's font; tell it
            // the half-block grid, so the pixels handed to it are not rescaled.
            #[allow(deprecated)]
            let mut p = Picker::from_fontsize(grid);
            p.set_protocol_type(ProtocolType::Halfblocks);
            picker = p;
        }
        Some(ImageView {
            target: target.to_string(),
            path: found.path.clone(),
            dims: (img.width(), img.height()),
            image: img,
            picker,
            grid,
            zoom: Cell::new(Zoom::Fit),
            offset: Cell::new((0, 0)),
            pane: Cell::new((0, 0)),
            drawn: RefCell::new(None),
        })
    }

    #[cfg(test)]
    pub(crate) fn zoom(&self) -> Zoom {
        self.zoom.get()
    }

    #[cfg(test)]
    pub(crate) fn offset(&self) -> (u32, u32) {
        self.offset.get()
    }

    /// The pane in pixels of the grid.
    pub(crate) fn pane_px(&self) -> (u32, u32) {
        let (c, r) = self.pane.get();
        (
            u32::from(c) * u32::from(self.grid.width),
            u32::from(r) * u32::from(self.grid.height),
        )
    }

    /// The factor the picture is drawn at now.
    pub(crate) fn scale(&self) -> f64 {
        scale_for(self.zoom.get(), self.dims, self.pane_px())
    }

    /// The zoom as the status line says it: `fit`, `fit width`, `200%`.
    pub(crate) fn zoom_label(&self) -> String {
        match self.zoom.get() {
            Zoom::Fit => "fit".into(),
            Zoom::FitWidth => "fit width".into(),
            Zoom::Scale(s) => format!("{}%", (s * 100.0).round() as u32),
        }
    }

    /// Whether the picture is larger than the pane on some axis, so `h`/`j`/
    /// `k`/`l` have something to pan.
    pub(crate) fn pannable(&self) -> bool {
        let shown = shown_size(self.dims, self.scale());
        let pane = self.pane_px();
        shown.0 > pane.0 || shown.1 > pane.1
    }

    /// Switch to `zoom`, keeping the middle of the window on the same part of
    /// the picture (a mode that fits the pane clamps the offset to 0).
    pub(crate) fn set_zoom(&self, zoom: Zoom) {
        let pane = self.pane_px();
        let (old, new) = (self.scale(), scale_for(zoom, self.dims, pane));
        let shown = shown_size(self.dims, old);
        let off = self.offset.get();
        let mid = |o: u32, shown: u32, pane: u32| f64::from(o) + f64::from(shown.min(pane)) / 2.0;
        let (cx, cy) = (
            mid(off.0, shown.0, pane.0) / old,
            mid(off.1, shown.1, pane.1) / old,
        );
        self.zoom.set(zoom);
        let shown = shown_size(self.dims, new);
        let at = |c: f64, shown: u32, pane: u32| {
            (c * new - f64::from(shown.min(pane)) / 2.0)
                .max(0.0)
                .round() as u32
        };
        self.offset.set(clamp_offset(
            (at(cx, shown.0, pane.0), at(cy, shown.1, pane.1)),
            shown,
            pane,
        ));
    }

    /// One step up (`+`) or down (`-`) the [`ZOOM_STEPS`] from the factor
    /// drawn now (so from `fit` it steps off whatever fit came to).
    pub(crate) fn zoom_step(&self, up: bool) {
        let cur = self.scale();
        let next = if up {
            ZOOM_STEPS.iter().copied().find(|&s| s > cur * 1.001)
        } else {
            ZOOM_STEPS.iter().rev().copied().find(|&s| s < cur / 1.001)
        };
        if let Some(s) = next {
            self.set_zoom(Zoom::Scale(s));
        }
    }

    /// Pan the window a quarter of the pane (at least a cell) toward `dx`/`dy`
    /// (each -1, 0 or 1), as far as the picture goes.
    pub(crate) fn pan(&self, dx: i32, dy: i32) {
        let (pane, shown) = (self.pane_px(), shown_size(self.dims, self.scale()));
        let step = |len: u32, cell: u16| i64::from((len / 4).max(u32::from(cell)));
        let off = self.offset.get();
        let moved = |o: u32, d: i32, len: u32, cell: u16| {
            (i64::from(o) + i64::from(d) * step(len, cell)).max(0) as u32
        };
        self.offset.set(clamp_offset(
            (
                moved(off.0, dx, pane.0, self.grid.width),
                moved(off.1, dy, pane.1, self.grid.height),
            ),
            shown,
            pane,
        ));
    }

    /// The picture's window for a pane of `cols`x`rows` cells: the part of the
    /// zoomed picture at the offset, cropped from the source pixels and
    /// scaled once to exactly its size, on a canvas of whole cells. Returns the
    /// cells it covers.
    fn window(&self, cols: u16, rows: u16) -> Option<((u16, u16), Option<Protocol>)> {
        let (gw, gh) = (u32::from(self.grid.width), u32::from(self.grid.height));
        let pane = (u32::from(cols) * gw, u32::from(rows) * gh);
        self.pane.set((cols, rows));
        let scale = self.scale();
        let shown = shown_size(self.dims, scale);
        let off = clamp_offset(self.offset.get(), shown, pane);
        self.offset.set(off);
        let vis = (shown.0.min(pane.0), shown.1.min(pane.1));
        let cells = (vis.0.div_ceil(gw) as u16, vis.1.div_ceil(gh) as u16);
        let key: DrawKey = (scale.to_bits(), off, (cols, rows));
        let mut drawn = self.drawn.borrow_mut();
        if drawn.as_ref().is_none_or(|(k, _)| *k != key) {
            let proto = self.encode(scale, off, vis, cells);
            *drawn = Some((key, proto));
        }
        drawn.as_ref().map(|(_, p)| (cells, p.clone()))
    }

    /// The protocol for the `vis`-pixel window at `off` of the picture at
    /// `scale`, padded to `cells`.
    fn encode(
        &self,
        scale: f64,
        off: (u32, u32),
        vis: (u32, u32),
        cells: (u16, u16),
    ) -> Option<Protocol> {
        let (iw, ih) = self.dims;
        // The source pixels under the window, widened to whole pixels.
        let src = |o: u32, len: u32, max: u32| {
            let a = ((f64::from(o) / scale).floor() as u32).min(max.saturating_sub(1));
            let b = (((f64::from(o + len)) / scale).ceil() as u32).clamp(a + 1, max);
            (a, b - a)
        };
        let (x, w) = src(off.0, vis.0, iw);
        let (y, h) = src(off.1, vis.1, ih);
        let part = self.image.crop_imm(x, y, w, h);
        let part = if (part.width(), part.height()) == vis {
            part
        } else {
            part.resize_exact(vis.0, vis.1, FilterType::Triangle)
        };
        let canvas = (
            u32::from(cells.0) * u32::from(self.grid.width),
            u32::from(cells.1) * u32::from(self.grid.height),
        );
        let img = if (canvas.0, canvas.1) == vis {
            part
        } else {
            let mut bg = image::RgbaImage::new(canvas.0, canvas.1);
            image::imageops::overlay(&mut bg, &part.to_rgba8(), 0, 0);
            image::DynamicImage::ImageRgba8(bg)
        };
        self.picker
            .new_protocol(
                img,
                ratatui::layout::Size::new(cells.0, cells.1),
                Resize::Fit(None),
            )
            .ok()
    }
}

/// A cyan header naming the file and its pixel size, then the window of the
/// image the zoom and pan show of the rest of the pane.
pub(crate) fn render_image(v: &ImageView, frame: &mut Frame, area: Rect) {
    let [head, body] = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(area);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                format!("🖼 {}", v.target),
                Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  image {}×{}", v.dims.0, v.dims.1),
                Style::new().fg(Color::DarkGray),
            ),
        ])),
        head,
    );
    if let Some(((w, h), Some(proto))) = v.window(body.width, body.height) {
        frame.render_widget(
            Image::new(&proto),
            Rect {
                width: w,
                height: h,
                ..body
            },
        );
    }
}

impl App {
    /// Run an image viewer command: `+`/`-`/`0`/`w`/`f` zoom and `h`/`j`/`k`/
    /// `l` pan (yaks-e5f9), `O` hands the file to the OS opener, `Esc` closes
    /// it, `o`/`i` go back / forward in the history.
    pub(super) fn run_image_cmd(&mut self, cmd: Cmd) {
        let Some(v) = self.image() else {
            return;
        };
        match cmd {
            Cmd::ImageZoomIn => v.zoom_step(true),
            Cmd::ImageZoomOut => v.zoom_step(false),
            Cmd::ImageZoomActual => v.set_zoom(Zoom::Scale(1.0)),
            Cmd::ImageFitWidth => v.set_zoom(Zoom::FitWidth),
            Cmd::ImageFit => v.set_zoom(Zoom::Fit),
            Cmd::ImagePanLeft => v.pan(-1, 0),
            Cmd::ImagePanRight => v.pan(1, 0),
            Cmd::ImagePanUp => v.pan(0, -1),
            Cmd::ImagePanDown => v.pan(0, 1),
            Cmd::PaneClose => self.close_doc(),
            Cmd::NavBack => self.history_back(),
            Cmd::NavForward => self.history_forward(),
            Cmd::FileOpenExternal => {
                let (target, arg) = (v.target.clone(), v.path.display().to_string());
                self.close_doc();
                self.os_open(&target, &arg);
            }
            _ => {}
        }
    }
}

// -- inline images in the detail pane (yaks-0bf0) ---------------------------

/// Most decoded inline images kept at once; past it the cache is emptied and
/// the visible ones are rebuilt (a handful covers any yak a person scrolls).
const INLINE_CACHE_MAX: usize = 8;

/// Inline images ready to draw, by (file, its mtime in ns, columns, rows). A
/// file that failed to decode is remembered as `None` so a broken image is not
/// retried every frame; a new mtime is a new key, so an edited file reloads.
#[derive(Default)]
pub(crate) struct InlineCache {
    ready: HashMap<(PathBuf, u128, u16, u16), Option<SlicedProtocol>>,
}

#[cfg(test)]
impl InlineCache {
    pub(crate) fn len(&self) -> usize {
        self.ready.len()
    }
}

fn mtime_ns(path: &Path) -> u128 {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_nanos())
}

/// Decode `path` and encode it for `cols`x`rows` cells. Done once per image
/// and size, at first draw, and only for an image on screen.
fn build_inline(app: &App, path: &Path, cols: u16, rows: u16) -> Option<SlicedProtocol> {
    let name = path.file_name()?.to_str()?;
    let img = decode_attachment(path, name)?;
    // Stretch to exactly the reserved cells (the layout already kept the aspect
    // to within a cell), so no padding strip shows where rounding left a
    // remainder, and an author's `=50%` of a small image is enlarged to fit.
    let font = app.picker.font_size();
    let img = img.resize_exact(
        u32::from(cols) * u32::from(font.width),
        u32::from(rows) * u32::from(font.height),
        FilterType::Triangle,
    );
    SlicedProtocol::new_with_resize(
        &app.picker,
        img,
        ratatui::layout::Size::new(cols, rows),
        Resize::Fit(None),
    )
    .ok()
}

/// Paint the inline images that intersect the visible rows of a pane (the
/// detail pane, or the document viewer) over the blank rows the layout
/// reserved for them. `area` is the content area (below any pinned header);
/// `lines` are the pane's rows, scrolled `scroll` rows. An image
/// partly scrolled out of view is clipped at the pane edge by `SlicedImage`
/// (kitty and half-blocks skip rows, sixel drops bands, iTerm2 draws row
/// slices). An image wholly out of view is neither decoded nor drawn.
pub(crate) fn render_inline_images(
    app: &App,
    lines: &[detail::DLine],
    scroll: u16,
    frame: &mut Frame,
    area: Rect,
) {
    let scroll = scroll as isize;
    let mut i = 0;
    while i < lines.len() {
        let Some(slot) = lines[i].image.as_ref().filter(|s| s.row == 0) else {
            i += 1;
            continue;
        };
        let top = i as isize - scroll;
        let n = slot.rows.max(1) as usize;
        i += n;
        if top + n as isize <= 0 || top >= area.height as isize {
            continue;
        }
        let key = (
            slot.path.clone(),
            mtime_ns(&slot.path),
            slot.cols,
            slot.rows,
        );
        let mut cache = app.inline_cache.borrow_mut();
        if cache.ready.len() >= INLINE_CACHE_MAX && !cache.ready.contains_key(&key) {
            cache.ready.clear();
        }
        let proto = cache
            .ready
            .entry(key)
            .or_insert_with(|| build_inline(app, &slot.path, slot.cols, slot.rows));
        if let Some(proto) = proto {
            frame.render_widget(
                SlicedImage::new(proto, SignedPosition::from((0, top as i16))),
                area,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_shrinks_but_never_enlarges_and_fit_width_may_enlarge() {
        let pane = (100, 44);
        assert_eq!(scale_for(Zoom::Fit, (200, 100), pane), 0.44);
        assert_eq!(scale_for(Zoom::Fit, (50, 20), pane), 1.0);
        assert_eq!(scale_for(Zoom::FitWidth, (50, 20), pane), 2.0);
        assert_eq!(scale_for(Zoom::FitWidth, (200, 100), pane), 0.5);
        assert_eq!(scale_for(Zoom::Scale(3.0), (200, 100), pane), 3.0);
        // No frame drawn yet: nothing to measure against.
        assert_eq!(scale_for(Zoom::Fit, (200, 100), (0, 0)), 1.0);
    }

    #[test]
    fn the_window_stays_inside_the_picture() {
        assert_eq!(clamp_offset((90, 90), (300, 200), (100, 44)), (90, 90));
        assert_eq!(clamp_offset((500, 500), (300, 200), (100, 44)), (200, 156));
        // A picture that fits an axis has nothing to pan on it.
        assert_eq!(clamp_offset((9, 9), (80, 200), (100, 44)), (0, 9));
        assert_eq!(shown_size((200, 100), 0.5), (100, 50));
        assert_eq!(shown_size((1, 1), 0.05), (1, 1));
    }
}

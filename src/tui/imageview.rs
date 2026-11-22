//! The image viewer (`DocView::Image`, yaks-4305): an image attachment
//! decoded once on open and drawn in the right pane by `ratatui-image`, over
//! the terminal's own graphics protocol (kitty, sixel, iTerm2) when it speaks
//! one, else as half-block cells. Headless frames (toque, tests) always take
//! the half-block path: it is ordinary colored cells, so it lands in text and
//! SVG snapshots like anything else.
//!
//! Zoom and pan (yaks-e5f9) are toque's image pane (`toque::image`, extracted
//! from here); this module decodes attachments, draws the header, and maps the
//! viewer's keys onto it.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui_image::picker::Picker;
use ratatui_image::sliced::{SignedPosition, SlicedImage, SlicedProtocol};
use ratatui_image::{FilterType, Resize};
pub(crate) use toque::image::Zoom;

use super::keymap::Cmd;
use super::{App, detail};

/// Largest image file the viewer decodes; a bigger one opens externally.
pub(super) const IMAGE_MAX_BYTES: u64 = 32 << 20;

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

/// Decode an image (`name` says whether it is an svg) to pixels: an svg
/// through toque's rasterizer (and the PNG cache), anything else through the
/// `image` crate with the format guessed from the content.
fn decode_bytes(bytes: &[u8], name: &str) -> Option<image::DynamicImage> {
    let is_svg = Path::new(name)
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("svg"));
    if is_svg {
        if bytes.len() as u64 > SVG_MAX_BYTES {
            return None;
        }
        let png = svg_png(bytes, super::cache::svg_cache_base().as_deref())?;
        return image::load_from_memory(&png).ok();
    }
    image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()
}

/// Decode the image file at `path` to pixels ([`decode_bytes`]).
fn decode_attachment(path: &Path, name: &str) -> Option<image::DynamicImage> {
    decode_bytes(&std::fs::read(path).ok()?, name)
}

/// An image attachment open in the viewer: toque's image pane (zoom, pan, the
/// protocol), plus which attachment it is. Derefs to [`toque::image::ImageView`].
pub(crate) struct ImageView {
    /// The link target as written (`<id>/<file>`), shown as the header.
    pub(crate) target: String,
    pub(crate) path: PathBuf,
    /// The farm commit this picture was read at, for a pinned link
    /// (`path@<rev>`); `None` for the file as it is now.
    pub(crate) rev: Option<String>,
    view: toque::image::ImageView,
}

impl std::ops::Deref for ImageView {
    type Target = toque::image::ImageView;
    fn deref(&self) -> &Self::Target {
        &self.view
    }
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
        Some(Self::new(target, found.path.clone(), img, picker))
    }

    /// The viewer for image `name` whose content is `bytes` (a member of an
    /// archive, yaks-93ee, or a file pinned to a commit), or `None` when it
    /// cannot be decoded. `path` is where the farm would have it, never read.
    pub(crate) fn from_bytes(
        target: &str,
        path: PathBuf,
        name: &str,
        bytes: &[u8],
        picker: &Picker,
    ) -> Option<Self> {
        if !crate::store::is_image_name(name) {
            return None;
        }
        let img = decode_bytes(bytes, name)?;
        Some(Self::new(target, path, img, picker))
    }

    fn new(target: &str, path: PathBuf, img: image::DynamicImage, picker: &Picker) -> Self {
        ImageView {
            target: target.to_string(),
            path,
            rev: None,
            view: toque::image::ImageView::new(img, picker),
        }
    }
}

/// A cyan header naming the file and its pixel size, then the window of the
/// image the zoom and pan show of the rest of the pane.
pub(crate) fn render_image(v: &ImageView, frame: &mut Frame, area: Rect) {
    let [head, body] = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(area);
    let (w, h) = v.dims();
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                format!("🖼 {}", v.target),
                Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("  image {w}×{h}"), Style::new().fg(Color::DarkGray)),
        ])),
        head,
    );
    v.render(frame, body);
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

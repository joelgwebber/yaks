//! The image viewer overlay (`Overlay::Image`, yaks-4305): an image attachment
//! decoded once on open and drawn in the right pane by `ratatui-image`, over
//! the terminal's own graphics protocol (kitty, sixel, iTerm2) when it speaks
//! one, else as half-block cells. Headless frames (toque, tests) always take
//! the half-block path: it is ordinary colored cells, so it lands in text and
//! SVG snapshots like anything else.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use ratatui::Frame;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;
use ratatui_image::sliced::{SignedPosition, SlicedImage, SlicedProtocol};
use ratatui_image::{FilterType, Resize, StatefulImage};

use super::{App, Overlay, detail};

/// Largest image file the viewer decodes; a bigger one opens externally.
const IMAGE_MAX_BYTES: u64 = 32 << 20;

/// An image attachment open in the viewer.
pub(crate) struct ImageView {
    /// The link target as written (`<id>/<file>`), shown as the header.
    pub(crate) target: String,
    pub(crate) path: PathBuf,
    /// Pixel size of the decoded image, shown in the header.
    dims: (u32, u32),
    /// Resized and encoded to the pane at render time, so it lives behind a
    /// `RefCell` (`render` takes `&App`).
    protocol: RefCell<StatefulProtocol>,
}

impl ImageView {
    /// The viewer for `found`, or `None` when it should open externally: not
    /// an image by name, over [`IMAGE_MAX_BYTES`], or one the `image` crate
    /// cannot decode (an svg, a corrupt file, a format it was built without).
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
        let img = image::ImageReader::open(&found.path)
            .ok()?
            .with_guessed_format()
            .ok()?
            .decode()
            .ok()?;
        Some(ImageView {
            target: target.to_string(),
            path: found.path.clone(),
            dims: (img.width(), img.height()),
            protocol: RefCell::new(picker.new_resize_protocol(img)),
        })
    }
}

/// A cyan header naming the file and its pixel size, then the image fitted
/// into the rest of the pane (shrunk to fit, never enlarged).
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
    frame.render_stateful_widget(
        StatefulImage::default().resize(Resize::Fit(Some(FilterType::Triangle))),
        body,
        &mut *v.protocol.borrow_mut(),
    );
}

impl App {
    /// Keys in the image viewer: `O` hands the file to the OS opener,
    /// `Esc`/`q`/`h`/`←` close it.
    pub(crate) fn handle_image_key(&mut self, k: KeyEvent) {
        let Overlay::Image(v) = &self.overlay else {
            return;
        };
        match k.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('h') | KeyCode::Left => {
                self.overlay = Overlay::None;
            }
            KeyCode::Char('O') => {
                let (target, arg) = (v.target.clone(), v.path.display().to_string());
                self.overlay = Overlay::None;
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
    let img = image::ImageReader::open(path)
        .ok()?
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()?;
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

/// Paint the inline images that intersect the visible rows of the detail pane
/// over the blank rows the layout reserved for them. `area` is the content
/// area (below any pinned header); `lines` are the pane's rows. An image
/// partly scrolled out of view is clipped at the pane edge by `SlicedImage`
/// (kitty and half-blocks skip rows, sixel drops bands, iTerm2 draws row
/// slices). An image wholly out of view is neither decoded nor drawn.
pub(crate) fn render_inline_images(
    app: &App,
    lines: &[detail::DLine],
    frame: &mut Frame,
    area: Rect,
) {
    let scroll = app.detail_scroll as isize;
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

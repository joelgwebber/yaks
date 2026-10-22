//! The image viewer overlay (`Overlay::Image`, yaks-4305): an image attachment
//! decoded once on open and drawn in the right pane by `ratatui-image`, over
//! the terminal's own graphics protocol (kitty, sixel, iTerm2) when it speaks
//! one, else as half-block cells. Headless frames (toque, tests) always take
//! the half-block path: it is ordinary colored cells, so it lands in text and
//! SVG snapshots like anything else.

use std::cell::RefCell;
use std::path::PathBuf;

use ratatui::Frame;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;
use ratatui_image::{FilterType, Resize, StatefulImage};

use super::{App, Overlay};

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

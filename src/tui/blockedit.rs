//! In-place editing of one content block in the detail pane (yaks-597d).
//!
//! `E` on a body row turns that block (the description or a comment) into an
//! edtui editor at the same place in the pane; `M` opens an empty one under a
//! provisional header after the last comment. The rest of the pane stays
//! rendered and scrolls as usual: the editor is just N more rows in the row
//! model (`detail::EditSlot` → `Kind::Editor` rows), so the cursor, scroll and
//! hit arithmetic need no special case, and `render_detail` draws the edtui view
//! over those rows. Save writes only that block (`content::assemble`, so the
//! others round-trip), cancel goes through the usual dirty-confirm.
//!
//! `a` (ask / answer) opens the same new-comment editor with an `Ask`/`Answer`
//! action (yaks-ce5e): its header names the verb and its hints say what saving
//! does; the save raises or answers the `needs` block with the text as the note.

use super::*;
use edtui::EditorView;
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;
use unicode_width::UnicodeWidthChar;

/// Columns the editor's left bar takes.
const GUTTER: usize = 1;
/// Rows of surrounding content kept visible when the editor is capped.
const CONTEXT_ROWS: usize = 4;
/// Fewest rows the editor gets, however short the pane.
const MIN_ROWS: usize = 3;
/// Header hints, longest first; the longest that fits is drawn.
const HINTS: [&str; 3] = [
    "editing · Ctrl-S save · Ctrl-C cancel",
    "editing · Ctrl-S save",
    "Ctrl-S save",
];
/// Header hints of the ask editor: what Ctrl-S does to the yak.
const ASK_HINTS: [&str; 3] = [
    "Ctrl-S ask (needs: human) · Ctrl-C cancel",
    "Ctrl-S ask (needs: human)",
    "Ctrl-S ask",
];
/// Header hints of the answer editor.
const ANSWER_HINTS: [&str; 3] = [
    "Ctrl-S answer (needs: agent) · Ctrl-C cancel",
    "Ctrl-S answer (needs: agent)",
    "Ctrl-S answer",
];

/// The header hints for an editor whose save is `action`.
fn hints_for(action: &EditAction) -> &'static [&'static str; 3] {
    match action {
        EditAction::Ask(_) => &ASK_HINTS,
        EditAction::Answer(_) => &ANSWER_HINTS,
        _ => &HINTS,
    }
}

/// The open in-place editor: the `Editor` (whose `EditAction` says where the
/// save goes) plus what the row model needs to place and compare it.
pub(crate) struct BlockEdit {
    pub(crate) editor: Editor,
    id: String,
    /// The block being edited; `None` = a new comment.
    block: Option<usize>,
    /// The block's text when the editor opened, to tell whether anything changed.
    original: String,
    /// The provisional header of a new comment (`you · now`).
    label: String,
}

impl BlockEdit {
    /// Whether closing now would lose typing (a new comment: any text at all).
    pub(crate) fn is_dirty(&self) -> bool {
        let text = self.editor.text();
        match self.block {
            Some(_) => text != self.original,
            None => !text.trim().is_empty(),
        }
    }

    /// The help bar while editing: the vim mode, then the keys.
    pub(crate) fn status_line(&self) -> Line<'static> {
        let mut spans = Vec::new();
        if self.editor.vim {
            let mode = self.editor.state.borrow().mode;
            spans.push(Span::styled(mode_tag(mode), mode_style(mode)));
            spans.push(Span::raw("  "));
        }
        let save = match self.editor.action {
            EditAction::Ask(_) => "Ctrl-S ask: needs human",
            EditAction::Answer(_) => "Ctrl-S answer: needs agent",
            _ => "Ctrl-S save",
        };
        spans.push(Span::styled(
            format!("{save} · Ctrl-C / Esc Esc cancel"),
            Style::new().fg(Color::DarkGray),
        ));
        Line::from(spans)
    }
}

/// Rows the editor's text takes at `width` display columns, soft-wrapped the
/// way edtui wraps it. edtui keeps its wrapper private (`LineWrapper::
/// wrap_ranges`: break after the last space that fits, hard-break a longer
/// word), so this mirrors it; a drift only costs the spare row.
fn wrapped_rows(text: &str, width: usize) -> usize {
    let cw = |c: char| if c == '\t' { 2 } else { c.width().unwrap_or(0) };
    text.split('\n')
        .map(|line| {
            let chars: Vec<char> = line.chars().collect();
            let n = chars.len();
            let (mut rows, mut start) = (1, 0);
            while width > 0 && start < n {
                let (mut w, mut end) = (0, start);
                while end < n {
                    if w + cw(chars[end]) > width && end > start {
                        break;
                    }
                    w += cw(chars[end]);
                    end += 1;
                }
                if end >= n {
                    break;
                }
                rows += 1;
                start = (start + 1..end)
                    .rev()
                    .find(|&k| chars[k] == ' ')
                    .map_or(end, |sp| sp + 1);
            }
            rows
        })
        .sum()
}

impl App {
    /// The in-place editor, while it is open or stashed behind its
    /// "discard changes?" prompt (the region must not vanish under the prompt).
    pub(crate) fn block_edit(&self) -> Option<&BlockEdit> {
        match &self.overlay {
            Overlay::BlockEdit(be) => Some(be),
            Overlay::Confirm {
                action: ConfirmAction::DiscardEdit,
                ..
            } => match &self.dirty_cancel {
                Some(Overlay::BlockEdit(be)) => Some(be),
                _ => None,
            },
            _ => None,
        }
    }

    /// What `detail::build_at` should reserve for the open editor: the block's
    /// source lines (as wrapped) + 1 spare row, capped so ~4 rows of context
    /// stay on screen (edtui scrolls inside beyond that).
    pub(crate) fn edit_slot(&self) -> Option<detail::EditSlot> {
        let be = self.block_edit()?;
        if self.selected().map(|t| t.id.as_str()) != Some(be.id.as_str()) {
            return None;
        }
        let width = match self.detail_width.get() as usize {
            0 => 80,
            w => w,
        };
        let text = be.editor.state.borrow().lines.to_string();
        // The blank row and the header sit above the editor.
        let cap = (self.detail_rows_shown(1).saturating_sub(2 + CONTEXT_ROWS)).max(MIN_ROWS);
        Some(detail::EditSlot {
            block: be.block,
            rows: (wrapped_rows(&text, width.saturating_sub(GUTTER)) + 1).min(cap),
            label: be.label.clone(),
            hints: hints_for(&be.editor.action)
                .iter()
                .map(|h| h.to_string())
                .collect(),
        })
    }

    /// `E` on a body row (`Some(block)`) or `M` (`None`): open the in-place
    /// editor. The editor's cursor lands on the source line behind the row the
    /// line cursor was on; the line cursor moves to the block's header.
    pub(crate) fn open_block_edit(&mut self, block: Option<usize>) {
        let Some(id) = self.selected_id() else { return };
        let Some(task) = self.task(&id) else { return };
        let (text, label, row, action) = match block {
            Some(b) => {
                let blocks = content::parse(&task.body);
                let Some(blk) = blocks.get(b) else { return };
                let row = self
                    .detail_dlines()
                    .get(self.detail_line)
                    .and_then(|l| l.src)
                    .filter(|s| s.block == b)
                    .and_then(|s| s.lines)
                    .map_or(0, |(first, _)| first);
                (
                    blk.text.clone(),
                    String::new(),
                    row,
                    EditAction::Block {
                        id: id.clone(),
                        block: b,
                    },
                )
            }
            None => (
                String::new(),
                format!("{} · now", self.note_author()),
                0,
                EditAction::Comment(id.clone()),
            ),
        };
        self.open_slot_editor(id, block, text, label, row, action);
    }

    /// `a`: open the new-comment editor whose save asks (`answer` false:
    /// `needs: human`) or answers (`needs: agent`) the selected yak, the typed
    /// text becoming the note. The header names the verb.
    pub(crate) fn open_needs_edit(&mut self, answer: bool) {
        let Some(id) = self.selected_id() else { return };
        let (verb, action) = if answer {
            ("Answer", EditAction::Answer(id.clone()))
        } else {
            ("Ask", EditAction::Ask(id.clone()))
        };
        let label = format!("{verb} {id} · {} · now", self.note_author());
        self.open_slot_editor(id, None, String::new(), label, 0, action);
    }

    /// Who a new note will be attributed to, for its provisional header.
    /// Resolved here (not in the read-only constructor's path) so a farm-less
    /// App shows `you` and runs no git subprocess.
    fn note_author(&self) -> String {
        let who = if self.farm.is_some() {
            crate::actor::resolve(None)
        } else {
            None
        };
        who.unwrap_or_else(|| "you".into())
    }

    /// Open the in-place editor on `block` of `id` (`None`: a new note after
    /// the last block) holding `text`, its cursor on source line `row`.
    fn open_slot_editor(
        &mut self,
        id: String,
        block: Option<usize>,
        text: String,
        label: String,
        row: usize,
        action: EditAction,
    ) {
        let editor = Editor::new(self.editor_vim, false, String::new(), &text, action);
        let last = text.split('\n').count() - 1;
        editor.state.borrow_mut().cursor = edtui::Index2::new(row.min(last), 0);
        if self.focus == Focus::List {
            self.focus = Focus::Detail;
            self.detail_scroll = 0;
            self.detail_anchor = None;
            self.detail_find = None;
            self.detail_match = 0;
        }
        self.overlay = Overlay::BlockEdit(BlockEdit {
            editor,
            id,
            block,
            original: text,
            label,
        });
        let first = self
            .detail_dlines()
            .iter()
            .position(|l| l.kind == detail::Kind::Editor);
        if let Some(first) = first {
            self.detail_line = first.saturating_sub(1);
        }
        self.show_block_editor();
    }

    /// Scroll just enough that the whole editor (header to last row) is on
    /// screen. It is capped to fit, so scrolling to its end and then to its
    /// header never pushes the end out again.
    pub(crate) fn show_block_editor(&mut self) {
        if self.block_edit().is_none() {
            return;
        }
        let lines = self.detail_dlines();
        let Some(first) = lines.iter().position(|l| l.kind == detail::Kind::Editor) else {
            return;
        };
        let n = lines[first..]
            .iter()
            .take_while(|l| l.kind == detail::Kind::Editor)
            .count();
        self.scroll_line_into_view((first + n - 1) as u16);
        self.scroll_line_into_view(first.saturating_sub(1) as u16);
    }

    /// After the editor closes: keep the scroll in range and the line cursor on
    /// screen (the block's rows are back to their rendered height).
    pub(crate) fn settle_detail_scroll(&mut self) {
        let n = self.detail_line_count();
        self.detail_line = self.detail_line.min(n.saturating_sub(1));
        self.detail_scroll = self.detail_scroll.min(self.detail_max_scroll());
        self.scroll_line_into_view(self.detail_line as u16);
    }

    /// Save `text` as block `block` of `id`, leaving every other block as it
    /// was (`content::assemble` rewrites them to the canonical shape), then put
    /// the line cursor on the row of source line `row` of the re-rendered block.
    pub(crate) fn commit_block(&mut self, id: &str, block: usize, text: String, row: usize) {
        let Some(task) = self.task(id) else { return };
        let mut blocks = content::parse(&task.body);
        let Some(b) = blocks.get_mut(block) else {
            self.notification = Some(format!("{id}: that block is gone"));
            return;
        };
        b.text = text;
        let body = content::assemble(&blocks);
        if body == task.body {
            self.notification = Some(format!("{id} unchanged"));
        } else {
            // `task.body` may be stale: the farm keeps entries appended since
            // and refuses anything else (yaks-9f5c).
            let base = task.body.clone();
            let text = blocks.swap_remove(block).text;
            let outcome = self.apply_edit(
                id,
                TaskEdit {
                    description: Some(body),
                    base: Some(base),
                    ..Default::default()
                },
                format!("{id} updated"),
            );
            if outcome == Some(UpdateOutcome::Conflict) {
                self.reopen_after_conflict(id, block, text, row);
                return;
            }
        }
        self.focus_source_line(block, Some(row));
    }

    /// A save that conflicted wrote nothing and `apply_edit` reloaded the yak:
    /// reopen the editor on the same block of the fresh body, holding the
    /// user's `text` (dirty, so a cancel still asks), to review and save
    /// again. A block that is gone leaves the text on the clipboard instead.
    fn reopen_after_conflict(&mut self, id: &str, block: usize, text: String, row: usize) {
        let fresh = (self.selected_id().as_deref() == Some(id))
            .then(|| self.task(id))
            .flatten()
            .and_then(|t| content::parse(&t.body).get(block).map(|b| b.text.clone()));
        let Some(original) = fresh else {
            let copied = crate::clipboard::copy_text(&text);
            self.notification = Some(format!(
                "{id} changed on disk; not saved{}",
                if copied {
                    ", your text is on the clipboard"
                } else {
                    ", clipboard unavailable"
                }
            ));
            return;
        };
        let editor = Editor::new(
            self.editor_vim,
            false,
            String::new(),
            &text,
            EditAction::Block {
                id: id.to_string(),
                block,
            },
        );
        let last = text.split('\n').count() - 1;
        editor.state.borrow_mut().cursor = edtui::Index2::new(row.min(last), 0);
        self.overlay = Overlay::BlockEdit(BlockEdit {
            editor,
            id: id.to_string(),
            block: Some(block),
            original,
            label: String::new(),
        });
        self.show_block_editor();
        self.notification = Some(format!(
            "{id} changed on disk; not saved: review and Ctrl-S again"
        ));
    }

    /// Put the line cursor on the row showing `line` (a 0-based source line) of
    /// content block `block`, or on its header (`None`); stays put when the
    /// block is gone.
    fn focus_source_line(&mut self, block: usize, line: Option<usize>) {
        let lines = self.detail_dlines();
        let rows = || {
            lines
                .iter()
                .enumerate()
                .filter_map(|(i, l)| Some((i, l.src?)))
        };
        let hit = match line {
            Some(n) => rows()
                .find(|(_, s)| s.block == block && s.lines.is_some_and(|(a, b)| a <= n && n <= b))
                // The line is gone (the block got shorter): its last row.
                .or_else(|| {
                    rows()
                        .filter(|(_, s)| s.block == block && s.lines.is_some())
                        .last()
                }),
            None => rows().find(|(_, s)| s.block == block && s.lines.is_none()),
        };
        if let Some((i, _)) = hit {
            self.detail_line = i;
        }
    }

    /// After a new comment is saved: the line cursor goes to its header.
    pub(crate) fn focus_new_comment(&mut self) {
        let Some(task) = self.selected() else { return };
        let last = content::parse(&task.body).len() - 1;
        self.focus_source_line(last, None);
    }
}

/// Draw the open editor over its `Kind::Editor` rows in the detail pane: a
/// bar down the left, the edtui view beside it. The view is drawn at full
/// size into a scratch buffer and the visible rows copied over, so an editor
/// the wheel/cursor has half scrolled off still draws correctly.
pub(crate) fn render_block_editor(
    be: &BlockEdit,
    lines: &[detail::DLine],
    scroll: u16,
    frame: &mut Frame,
    area: Rect,
) {
    let Some(first) = lines.iter().position(|l| l.kind == detail::Kind::Editor) else {
        return;
    };
    let n = lines[first..]
        .iter()
        .take_while(|l| l.kind == detail::Kind::Editor)
        .count();
    let w = area.width as usize;
    if w <= GUTTER || n == 0 {
        return;
    }
    let full = Rect::new(0, 0, area.width, n as u16);
    let mut buf = Buffer::empty(full);
    {
        let mut st = be.editor.state.borrow_mut();
        let mode = st.mode;
        set_md_highlights(&mut st);
        let area = Rect::new(GUTTER as u16, 0, (w - GUTTER) as u16, n as u16);
        // edtui scrolls to the cursor only once it knows how many rows the
        // previous frame showed, so a cursor that opens below the fold would
        // wait a frame. Two passes settle the viewport in this one.
        for _ in 0..2 {
            EditorView::new(&mut st)
                .theme(editor_theme(mode))
                .render(area, &mut buf);
        }
    }
    let bar = Style::new().fg(Color::Cyan);
    for r in 0..n as u16 {
        buf[(0, r)].set_symbol("\u{2503}").set_style(bar); // ┃
    }
    for r in 0..n {
        let Some(y) = (first + r).checked_sub(scroll as usize) else {
            continue;
        };
        if y >= area.height as usize {
            break;
        }
        for x in 0..area.width {
            frame.buffer_mut()[(area.x + x, area.y + y as u16)] = buf[(x, r as u16)].clone();
        }
    }
}

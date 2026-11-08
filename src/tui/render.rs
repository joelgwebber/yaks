//! The render layer: paints `App` state into ratatui widgets. Split out of
//! `tui.rs` (yaks-b1cc / b1cc/5). Overlay dispatch here calls into the
//! per-module renderers (drawer/create/fuzzy/editor); shared render helpers
//! (`disp_width`, `status_word`, `priority_style`, `render_chip_row`/
//! `render_text_row`) live here and are re-exported by `tui.rs`, so sibling
//! modules reach them via `super::`. The whole-frame snapshot/integration tests
//! stay central in `tui.rs` (yaks-2301 decision (a)).

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Padding, Paragraph};

use std::cell::RefCell;

use edtui::{EditorState, EditorView};

use super::blockedit;
use super::keymap::{
    PopupSubject, editor_groups, editor_title, help_content, help_hint, popup_lines, rows_groups,
};
use super::palette;
use super::{
    App, DocView, Focus, Overlay, completion_context, detail, editor_theme, render_create,
    render_drawer, render_editor_panel, render_fuzzy_results, render_line_editor,
    render_query_line, render_shed_detail, tree, view,
};
use crate::filter::FilterSpec;
use crate::model::{Status, Task};

pub(crate) fn render(app: &App, frame: &mut Frame) {
    // Tab row, a blank gap, the main area, then the help bar (Python layout).
    let [top, _gap, mid, bot] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(frame.area());
    // Fresh hit map each frame: only panes drawn below get recorded (yaks-97a2).
    app.hits.replace(Default::default());
    render_tabs(app, frame, top);
    // The list is full-width until the detail pane, a document or a right-pane
    // overlay is shown. We intentionally keep the filter drawer / fuzzy picker / view
    // picker on the RIGHT (diverging from Python's top drawer): in a wide, short
    // terminal a right-side drawer wastes no rows. Every right-pane surface gets
    // the same left divider (via right_divider) so it doesn't bleed into the list.
    let right_overlay = matches!(&app.overlay, Overlay::Edit(ed) if !ed.single_line)
        || matches!(
            app.overlay,
            Overlay::Fuzzy(_)
                | Overlay::Drawer(_)
                | Overlay::Create(_)
                | Overlay::ViewPicker(_)
                | Overlay::Help(_)
        );
    if app.focus == Focus::Detail || app.doc.is_some() || right_overlay {
        let (left, right) = main_split(mid);
        let modal = left.width == 0;
        if !modal {
            render_list(app, frame, left);
        }
        let inner = right_divider(frame, right, true);
        if modal {
            // Affordance: the pane covers the list, so tip its divider with a
            // back-chevron — the list is behind it, `Esc` returns to it.
            frame.render_widget(
                Span::styled("\u{25c2}", Style::new().fg(Color::Cyan)),
                Rect {
                    width: 1,
                    height: 1,
                    ..right
                },
            );
        }
        match &app.overlay {
            Overlay::Edit(ed) if !ed.single_line => render_editor_panel(ed, frame, inner),
            Overlay::Fuzzy(fp) => render_fuzzy_results(app, fp, frame, inner),
            Overlay::Drawer(d) => render_drawer(d, frame, inner),
            Overlay::Create(f) => render_create(f, frame, inner, &mut app.hits.borrow_mut()),
            Overlay::Help(scroll) => render_help(app, *scroll, frame, inner),
            Overlay::ViewPicker(sel) => render_view_picker(app, *sel, frame, inner),
            // The pane's subject: the open document, else the selected row's.
            _ if app.doc.is_some() => match app.doc.as_ref().map(|d| &d.view) {
                Some(DocView::Text(v)) => render_attachment(app, v, frame, inner),
                Some(DocView::Image(v)) => super::imageview::render_image(v, frame, inner),
                None => {}
            },
            _ if app.selected_shed().is_some() => render_shed_detail(app, frame, inner),
            _ => render_detail(app, frame, inner),
        }
    } else {
        render_list(app, frame, mid);
    }
    if app.keys_popup {
        render_keys_popup(app, frame, mid);
    }
    render_status(app, frame, bot);
}

/// The `?` popup (which-key): the current context's keys over the bottom of
/// the main area, from the keymap in effect. It leaves a few rows of the
/// screen behind it.
fn render_keys_popup(app: &App, frame: &mut Frame, mid: Rect) {
    let (ctx, kind) = (app.surface_ctx(), app.surface_kind());
    // Over an overlay, the popup lists that editor's keys, not the surface's.
    let (groups, title) = match app.popup_subject() {
        Some(PopupSubject::Editor(at)) => (
            editor_groups(app.keymap.family, at),
            format!(" Keys · {} ", editor_title(app.keymap.family, at)),
        ),
        Some(PopupSubject::Rows) => (
            rows_groups(app.keymap.family),
            " Keys · form rows ".to_string(),
        ),
        None => (app.keymap.popup_entries(ctx, kind), String::new()),
    };
    let width = mid.width.saturating_sub(2) as usize;
    let max_rows = (mid.height as usize).saturating_sub(5).max(1);
    let lines = popup_lines(&groups, width, max_rows);
    let height = (lines.len() as u16 + 1).min(mid.height);
    let area = Rect {
        y: mid.y + mid.height - height,
        height,
        ..mid
    };
    let rows = match (ctx, kind) {
        (super::keymap::Ctx::List | super::keymap::Ctx::Detail, Some(kind)) => match kind {
            tree::RowKind::Yak => " · yak rows",
            tree::RowKind::Doc => " · document rows",
            tree::RowKind::Shed => " · shed rows",
        },
        _ => "",
    };
    let block = Block::new()
        .borders(Borders::TOP)
        .border_style(Style::new().fg(Color::Cyan))
        .padding(Padding::horizontal(1))
        .title(Span::styled(
            if title.is_empty() {
                format!(" Keys · {}{rows} ", ctx.title())
            } else {
                title
            },
            Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(area);
    frame.render_widget(Clear, area);
    frame.render_widget(block, area);
    frame.render_widget(Paragraph::new(lines), inner);
}

/// Right-pane (detail / overlay) share of the main area before clamping —
/// the historical 34/66 split.
pub(crate) const DETAIL_PCT: u16 = 66;
/// Narrowest the right pane may get: the field rows (label column + chips),
/// a note header (`▸ <RFC3339> [author]` ≈ 36 cols) and a ~45-col prose line
/// all still fit without awkward wrapping.
pub(crate) const DETAIL_MIN_WIDTH: u16 = 48;
/// Widest the right pane may get: past ~120 cols (divider + padding + ~115
/// cols of text) body prose is uncomfortably long to read, so extra terminal
/// width goes to the list instead.
pub(crate) const DETAIL_MAX_WIDTH: u16 = 120;
/// Narrowest useful list beside the pane (an id plus a few title words). If
/// the clamped pane would leave less than this, the pane covers the whole
/// main area instead (effectively modal).
pub(crate) const LIST_MIN_WIDTH: u16 = 48;

/// Split the main area into (list, right pane) when a right pane is shown.
/// The pane takes `DETAIL_PCT` of the width clamped to
/// `[DETAIL_MIN_WIDTH, DETAIL_MAX_WIDTH]`; the list gets the rest. When that
/// would leave the list narrower than `LIST_MIN_WIDTH`, the pane takes the
/// full width and the returned list rect is zero-width (modal). The single
/// source of the split, so hit-testing can reuse it.
pub(crate) fn main_split(mid: Rect) -> (Rect, Rect) {
    let w = mid.width;
    // Round the list's 34% share like the old Percentage layout did, so the
    // unclamped range is column-for-column unchanged.
    let list_pct = 100 - DETAIL_PCT;
    let list_share = ((w as u32 * list_pct as u32 + 50) / 100) as u16;
    let detail = (w - list_share).clamp(DETAIL_MIN_WIDTH, DETAIL_MAX_WIDTH);
    if w < detail.saturating_add(LIST_MIN_WIDTH) {
        return (Rect { width: 0, ..mid }, mid);
    }
    let list_w = w - detail;
    (
        Rect {
            width: list_w,
            ..mid
        },
        Rect {
            x: mid.x + list_w,
            width: detail,
            ..mid
        },
    )
}

/// Draw the shared left-divider rule (as the detail pane has) on a right-pane
/// area and return the inset content region. Applied to detail AND every
/// right-pane overlay so drawers don't bleed into the list.
pub(crate) fn right_divider(frame: &mut Frame, area: Rect, focused: bool) -> Rect {
    let block = Block::new()
        .borders(Borders::LEFT)
        .border_style(if focused {
            Style::new().fg(Color::Cyan)
        } else {
            Style::new().fg(Color::DarkGray)
        })
        .padding(Padding::horizontal(1));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    inner
}

pub(crate) fn render_view_picker(app: &App, sel: usize, frame: &mut Frame, area: Rect) {
    let [head, body] = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(area);
    frame.render_widget(
        Paragraph::new(Span::styled(
            "Views",
            Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        head,
    );
    let items: Vec<ListItem> = app
        .views
        .iter()
        .enumerate()
        .map(|(i, v)| {
            // 📌 pinned, 🔒 builtin — emoji glyphs for the view manager.
            let active = if i == app.view { "▸" } else { " " };
            let pin = if v.pinned { "\u{1f4cc}" } else { "  " };
            let lock = if v.builtin { "  \u{1f512}" } else { "" };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{active} "), Style::new().fg(Color::Cyan)),
                Span::raw(format!("{pin} ")),
                Span::raw(v.name.clone()),
                Span::styled(
                    format!("  ({})", app.view_count(v)),
                    Style::new().fg(Color::DarkGray),
                ),
                Span::raw(lock.to_string()),
            ]))
        })
        .collect();
    let mut state = ListState::default();
    if !app.views.is_empty() {
        state.select(Some(sel.min(app.views.len() - 1)));
    }
    frame.render_stateful_widget(
        List::new(items).highlight_style(Style::new().bg(super::surface::selected())),
        body,
        &mut state,
    );
}

pub(crate) fn render_help(app: &App, scroll: u16, frame: &mut Frame, area: Rect) {
    let [head, body] = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(area);
    frame.render_widget(
        Paragraph::new(Span::styled(
            "Help — keys",
            Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        head,
    );
    frame.render_widget(
        Paragraph::new(help_content(&app.keymap)).scroll((scroll, 0)),
        body,
    );
}

/// The attachment viewer (yaks-699b): a cyan header naming the file, then its
/// rows from the scroll down, painted with the detail pane's cursor, link,
/// selection and find highlights ([`render_nav_rows`], yaks-84e0).
pub(crate) fn render_attachment(
    app: &App,
    v: &super::handlers::AttachView,
    frame: &mut Frame,
    area: Rect,
) {
    let [head, body] = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(area);
    let kind = v.kind_label();
    let icon = match v.doc {
        super::handlers::DocKind::Attachment => "📎",
        _ => "📖",
    };
    // A pinned farm file says which commit it is shown at (yaks-7395).
    let pin = v
        .rev
        .as_ref()
        .map(|r| format!("@{}", &r[..r.len().min(7)]))
        .unwrap_or_default();
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                format!("{icon} {}{pin}", v.target),
                Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("  {kind}"), Style::new().fg(Color::DarkGray)),
        ])),
        head,
    );
    v.width.set(body.width);
    let known: std::collections::HashSet<&str> = app.all.iter().map(|t| t.id.as_str()).collect();
    let rows = v.rows(body.width, &known, app.image_env().as_ref());
    // The peek pane under the page (yaks-7395): what the link under the
    // cursor names, read by `links::peek`.
    let body = if app.viewer_has_peek(&rows) && body.height > super::follow::PEEK_ROWS + 2 {
        let [page, peek] = Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(super::follow::PEEK_ROWS),
        ])
        .areas(body);
        let jump = v.nav.link_at(&rows);
        render_peek(app, jump, Some(v), frame, peek);
        page
    } else {
        body
    };
    let w = body.width as usize;
    let extents = detail::nowrap_extents(&rows);
    let hscroll = |dl: &detail::DLine| {
        dl.nowrap_key()
            .and_then(|k| extents.get(&k).map(|&e| v.nav.block_hscroll(k, e, w)))
            .unwrap_or(0)
    };
    let lines = render_nav_rows(&rows, &v.nav, app.register.as_deref(), w, hscroll);
    frame.render_widget(Paragraph::new(lines).scroll((v.nav.scroll, 0)), body);
    super::imageview::render_inline_images(app, &rows, v.nav.scroll, frame, body);
    app.hits.borrow_mut().doc = Some(body);
}

/// The peek pane, under the viewer's dok or a yak's detail: a dim rule naming
/// the link under the cursor, then the peek's lines (a section, a line range,
/// a dok head) cut to the pane, or why it can't be read. Blank but for the
/// rule when the cursor is on no peekable link.
///
/// The one peek renderer: `jump` is the link under the cursor of the viewer
/// (whose dok is `doc`, for an in-page anchor) or of a yak's detail pane
/// (`doc` is `None`).
fn render_peek(
    app: &App,
    jump: Option<detail::Jump>,
    doc: Option<&super::handlers::AttachView>,
    frame: &mut Frame,
    area: Rect,
) {
    let dim = Style::new().fg(Color::DarkGray);
    let w = area.width as usize;
    let peek = jump.and_then(|j| app.peek_target(&j.target, doc));
    let title = match &peek {
        Some((label, _)) => format!("\u{2500}\u{2500} peek: {label} "),
        None => "\u{2500}\u{2500} peek ".into(),
    };
    let rule_fill = w.saturating_sub(disp_width(&title));
    let mut lines = vec![Line::from(Span::styled(
        truncate_disp(&format!("{title}{}", "\u{2500}".repeat(rule_fill)), w),
        dim,
    ))];
    let room = (area.height as usize).saturating_sub(1);
    match peek {
        Some((_, Ok(text))) => {
            let mut body = super::follow::peek_lines(&text);
            if body.len() > room {
                body.truncate(room.saturating_sub(1));
                body.push("\u{2026}".into());
            }
            lines.extend(body.into_iter().map(|l| Line::from(truncate_disp(&l, w))));
        }
        Some((_, Err(e))) => lines.push(Line::from(Span::styled(
            truncate_disp(&e, w),
            Style::new().fg(Color::Yellow),
        ))),
        None => {}
    }
    frame.render_widget(Paragraph::new(lines), area);
}

/// Paint a page of rows with a [`LineNav`]'s state, the one painter the
/// detail pane and the text viewer share (yaks-84e0): the line cursor and
/// any visual selection on a subtle background, the link on the cursor's
/// row emphasised, find matches highlighted (the current one green), and
/// each no-wrap row scrolled `hscroll(row)` columns. `find` is the search
/// register. Rows are returned in full; the caller scrolls the paragraph.
pub(crate) fn render_nav_rows<'a>(
    rows: &'a [detail::DLine],
    nav: &super::linenav::LineNav,
    find: Option<&str>,
    width: usize,
    hscroll: impl Fn(&detail::DLine) -> usize,
) -> Vec<Line<'a>> {
    let cur = nav.link_at(rows);
    let sel = nav.selection();
    let matches = super::linenav::find_matches(rows, find);
    rows.iter()
        .enumerate()
        .map(|(i, dl)| {
            let lm: Vec<(usize, usize, bool)> = matches
                .iter()
                .enumerate()
                .filter(|(_, (ln, _, _))| *ln == i)
                .map(|(mi, (_, col, len))| (*col, *len, Some(mi) == nav.matched))
                .collect();
            let in_sel = sel.is_some_and(|(lo, hi)| i >= lo && i <= hi);
            let line_bg = (i == nav.line || in_sel).then(super::surface::selected);
            render_dline(dl, cur.as_ref(), i, &lm, line_bg, hscroll(dl), width)
        })
        .collect()
}

/// The create/edit task form: header + title / type / priority / labels meta
/// rows, a `─ description ─` separator, then a multi-line description content
/// zone filling the rest. Laid out like `render_drawer`, inset by `right_divider`.

/// Label-column widths (gutter marker excluded): the drawer's longest label is
/// `priority` (8); the create form's is `description` (11). Each leaves one
/// trailing space before the chips/field.
pub(crate) const DRAWER_LABEL_W: usize = 9;
pub(crate) const CREATE_LABEL_W: usize = 12;

/// Render one chip row: a `▸` gutter, a padded label, then space-separated
/// chips. `current_row` highlights this row; `cursor_idx` is the chip the
/// cursor sits on (drawn REVERSED); each choice's bool marks it selected
/// (green bold). For single-select forms the cursor and the selection coincide.
/// Returns each chip's screen rect, in order, for mouse hit-testing.
pub(crate) fn render_chip_row(
    current_row: bool,
    cursor_idx: usize,
    label_w: usize,
    label: &str,
    choices: &[(String, bool)],
    frame: &mut Frame,
    area: Rect,
) -> Vec<Rect> {
    let mut spans = vec![
        Span::styled(
            if current_row { "▸ " } else { "  " },
            Style::new().fg(Color::Cyan),
        ),
        Span::styled(
            format!("{label:<label_w$}"),
            Style::new().fg(Color::DarkGray),
        ),
    ];
    let mut rects = Vec::with_capacity(choices.len());
    let mut x = area.x.saturating_add(2 + label_w as u16);
    for (j, (disp, sel)) in choices.iter().enumerate() {
        x = x.saturating_add(1); // the space before each chip
        let w = disp_width(disp) as u16;
        let right = area.x.saturating_add(area.width);
        rects.push(Rect::new(
            x.min(right),
            area.y,
            w.min(right.saturating_sub(x)),
            1,
        ));
        x = x.saturating_add(w);
        let mut style = if *sel {
            Style::new().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(Color::DarkGray)
        };
        if current_row && cursor_idx == j {
            style = style
                .bg(super::surface::selected())
                .add_modifier(Modifier::BOLD);
        }
        spans.push(Span::raw(" "));
        spans.push(Span::styled(disp.clone(), style));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
    rects
}

/// Render one text-facet row: a `▸` gutter, a padded label, then either a live
/// edtui field (when `current`) or the dimmed current value / `placeholder`
/// (shown when the field is empty and unfocused).
pub(crate) fn render_text_row(
    current: bool,
    label_w: usize,
    label: &str,
    cell: &RefCell<EditorState>,
    placeholder: &str,
    frame: &mut Frame,
    area: Rect,
) {
    let [g, lab, fld] = Layout::horizontal([
        Constraint::Length(2),
        Constraint::Length(label_w as u16),
        Constraint::Min(0),
    ])
    .areas(area);
    frame.render_widget(
        Paragraph::new(Span::styled(
            if current { "▸ " } else { "  " },
            Style::new().fg(Color::Cyan),
        )),
        g,
    );
    frame.render_widget(
        Paragraph::new(Span::styled(
            label.to_string(),
            Style::new().fg(Color::DarkGray),
        )),
        lab,
    );
    if current {
        let mut st = cell.borrow_mut();
        let mode = st.mode;
        frame.render_widget(
            // No wrap: an overflowing single-line field scrolls horizontally to
            // keep the cursor visible, instead of wrapping onto a hidden second
            // row under the next field (yaks-4da6).
            EditorView::new(&mut st)
                .theme(editor_theme(mode))
                .single_line(true)
                .wrap(false),
            fld,
        );
    } else {
        let text = cell.borrow().lines.to_string();
        let shown = if text.is_empty() {
            placeholder.to_string()
        } else {
            text
        };
        frame.render_widget(
            Paragraph::new(Span::styled(shown, Style::new().fg(Color::DarkGray))),
            fld,
        );
    }
}

pub(crate) fn overlay_name(o: &Overlay) -> &'static str {
    match o {
        Overlay::None => "none",
        Overlay::Pick { .. } => "pick",
        Overlay::Confirm { .. } => "confirm",
        Overlay::Edit(_) => "edit",
        Overlay::BlockEdit(_) => "block-edit",
        Overlay::Fuzzy(_) => "fuzzy",
        Overlay::Search(_) => "search",
        Overlay::Drawer(_) => "drawer",
        Overlay::Create(_) => "create",
        Overlay::Help(_) => "help",
        Overlay::Find(_) => "find",
        Overlay::ViewPicker(_) => "view-picker",
    }
}

/// Displayed count, capped like Python's format_count (unbounded views).
pub(crate) fn format_count(n: usize) -> String {
    if n <= 999 {
        n.to_string()
    } else {
        "999+".into()
    }
}

/// Approximate display width: emoji (and other astral glyphs) are width 2.
pub(crate) fn disp_width(s: &str) -> usize {
    s.chars()
        .map(|c| {
            let cp = c as u32;
            if cp >= 0x1_F000
                || cp == 0x2b50
                || cp == 0x2764
                || cp == 0x23f3
                || cp == 0x23f0
                || cp == 0x2705
            {
                2
            } else {
                1
            }
        })
        .sum()
}

/// Per-priority colour, matching Python's `_PRIORITY_PAIRS`:
/// P1 urgent red+bold, P2 high magenta, P3 medium yellow, P4 low green,
/// P5 lowest blue+dim.
pub(crate) fn priority_style(p: u8) -> Style {
    match p {
        1 => Style::new().fg(Color::Red).add_modifier(Modifier::BOLD),
        2 => Style::new().fg(Color::Magenta),
        4 => Style::new().fg(Color::Green),
        5 => Style::new().fg(Color::Blue).add_modifier(Modifier::DIM),
        _ => Style::new().fg(Color::Yellow), // 3 (and any fallback)
    }
}

pub(crate) fn status_word(s: Status) -> &'static str {
    match s {
        Status::Hairy => "hairy",
        Status::Shaving => "shaving",
        Status::Shorn => "shorn",
        Status::Dead => "dead",
    }
}

pub(crate) fn render_tabs(app: &App, frame: &mut Frame, area: Rect) {
    // ` {emoji name}{*} ({count}) ` per pinned view, active black-on-white bold,
    // others dim; a trailing filter indicator when the live filter is forked.
    let mut spans: Vec<Span> = Vec::new();
    let mut tab_x = area.x;
    for &i in &app.pinned_indices() {
        let v = &app.views[i];
        let mark = if i == app.view && app.is_view_modified() {
            "*"
        } else {
            ""
        };
        let count = if v.key == view::SHEDS_KEY {
            app.sheds_count_label()
        } else {
            format_count(app.view_count(v))
        };
        let text = format!(" {}{} ({count}) ", v.name, mark);
        let style = if i == app.view {
            // The terminal's own fg/bg swapped: ANSI Black/White are remapped
            // by themes (a black slab on some light ones) (yaks-48e9).
            Style::new().add_modifier(Modifier::REVERSED | Modifier::BOLD)
        } else {
            Style::new().add_modifier(Modifier::DIM)
        };
        // Record the tab's on-screen rect for mouse clicks (clipped to the row).
        let w = (disp_width(&text) as u16).min((area.x + area.width).saturating_sub(tab_x));
        if w > 0 {
            let r = Rect {
                x: tab_x,
                y: area.y,
                width: w,
                height: 1,
            };
            app.hits.borrow_mut().tabs.push((r, i));
        }
        tab_x = tab_x.saturating_add(w + 1);
        spans.push(Span::styled(text, style));
        spans.push(Span::raw(" "));
    }
    if let Some(chip) = query_chip(app) {
        spans.push(Span::styled(chip, Style::new().fg(Color::Cyan)));
    }
    if app.is_view_modified() {
        spans.push(Span::styled(
            format!(" filter: {}", filter_summary(&app.filter)),
            Style::new().fg(Color::Yellow),
        ));
    }
    let tabs_w = disp_width(&spans.iter().map(|s| s.content.as_ref()).collect::<String>());
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
    // The farm's git state, pinned to the row's far right; it yields (is not
    // drawn) when the tabs leave no room. Whatever else uses the right slot
    // (notification, family, sheds label) ends just left of it.
    let mut slot_w = area.width;
    let room = (area.width as usize).saturating_sub(tabs_w + 1);
    if let Some((line, w)) = farm_sync_chip(app, room) {
        let w = w as u16;
        let rect = Rect {
            x: area.x + area.width - w,
            y: area.y,
            width: w,
            height: 1,
        };
        frame.render_widget(Paragraph::new(line), rect);
        slot_w = area.width - w - 1;
    }
    // Right-aligned slot on the tab row: a transient notification takes it when
    // present (Python placement); otherwise the persistent family-scope indicator
    // for tree views. `~` marks a view inheriting the global default (auto) vs
    // an explicit per-view override.
    if let Some(n) = &app.notification {
        // The ephemeral status message wants its usual right-aligned slot on the
        // tab row, but it must not scribble over the tabs. When it fits beside
        // them (the last tab already contributes a trailing space to `tabs_w`),
        // keep it inline; otherwise drop it to the blank gap row directly below
        // the tab strip (`area.y + 1`, guaranteed by the top-of-frame layout in
        // `render`), left-aligned on that otherwise-empty line. Pure render-time
        // geometry, so it re-evaluates on every resize.
        let style = Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD);
        let full = disp_width(n);
        let w = (full as u16).min(area.width);
        if w > 0 {
            let rect = if tabs_w + full <= slot_w as usize {
                Rect {
                    x: area.x + slot_w - w,
                    y: area.y,
                    width: w,
                    height: 1,
                }
            } else {
                Rect {
                    x: area.x,
                    y: area.y + 1,
                    width: w,
                    height: 1,
                }
            };
            frame.render_widget(Paragraph::new(Span::styled(n.clone(), style)), rect);
        }
    } else if app.on_sheds_tab() {
        // The Sheds tab's scan state in the same slot (`scanned 4s ago`).
        let label = app.sheds_status_label();
        if !label.is_empty() && tabs_w + disp_width(&label) < slot_w as usize {
            let w = disp_width(&label) as u16;
            let rect = Rect {
                x: area.x + slot_w - w,
                y: area.y,
                width: w,
                height: 1,
            };
            frame.render_widget(
                Paragraph::new(Span::styled(label, Style::new().fg(Color::DarkGray))),
                rect,
            );
        }
    } else {
        let av = app.active_view();
        if !(av.is_flat() || av.key == "working-set" || av.key == view::DOKS_KEY) {
            let (marker, val) = match app.family_scope.get(&av.key) {
                Some(s) => ("", s.as_str()),
                None => ("~", view::FamilyScope::DEFAULT.as_str()),
            };
            let label = format!("family: {marker}{val}");
            // The indicator yields rather than overwrite tabs when the strip is
            // too wide to fit it.
            if tabs_w + disp_width(&label) < slot_w as usize {
                let w = (disp_width(&label) as u16).min(slot_w);
                if w > 0 {
                    let rect = Rect {
                        x: area.x + slot_w - w,
                        y: area.y,
                        width: w,
                        height: 1,
                    };
                    frame.render_widget(
                        Paragraph::new(Span::styled(label, Style::new().fg(Color::DarkGray))),
                        rect,
                    );
                }
            }
        }
    }
}

/// The tab-row chip and its display width, for `room` free columns after the
/// tabs: the full text if it fits, else the compact one, else `None` (also
/// when there is nothing to say). It holds the overdue count (`⏰ 2 overdue`,
/// red, yaks-f561) and the farm's git state ([`FarmSync`]). The git segments
/// are dim, except `↓behind`, the one warning (a sync would bring changes in).
fn farm_sync_chip(app: &App, room: usize) -> Option<(Line<'static>, usize)> {
    let overdue = app.overdue_count();
    let farm_sync = app.farm_sync.as_ref();
    [false, true].into_iter().find_map(|compact| {
        // (text, style) per segment, overdue first: it is the one to act on.
        let mut segments: Vec<(String, Style)> = Vec::new();
        if overdue > 0 {
            let text = if compact {
                format!("\u{23f0}{overdue}")
            } else {
                format!("\u{23f0} {overdue} overdue")
            };
            segments.push((text, Style::new().fg(Color::Red)));
        }
        for (text, warn) in farm_sync.map(|f| f.segments(compact)).unwrap_or_default() {
            let color = if warn { Color::Yellow } else { Color::DarkGray };
            segments.push((text, Style::new().fg(color)));
        }
        if segments.is_empty() {
            return None;
        }
        let mut spans = Vec::new();
        for (i, (text, style)) in segments.into_iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw(" "));
            }
            spans.push(Span::styled(text, style));
        }
        let w = disp_width(&spans.iter().map(|s| s.content.as_ref()).collect::<String>());
        (w <= room).then(|| (Line::from(spans), w))
    })
}

pub(crate) fn render_list(app: &App, frame: &mut Frame, area: Rect) {
    let rows = app.rows();
    let notice = if app.on_doks_tab() {
        super::doks_tab::doks_notice(app, rows.is_empty())
    } else if app.on_sheds_tab() {
        super::sheds_tab::sheds_notice(app, rows.is_empty())
    } else {
        None
    };
    // The Sheds view's footer (the farm's freshness) sits under its rows.
    let area = if notice.is_none() && app.on_sheds_tab() {
        super::sheds_tab::render_farm_footer(app, frame, area)
    } else {
        area
    };
    if let Some(notice) = notice {
        frame.render_widget(notice, area);
        return;
    }
    if rows.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled(
                "No tasks.",
                Style::new().add_modifier(Modifier::DIM),
            )),
            area,
        );
        return;
    }
    // A document or shed row keeps its own colours under the cursor; a yak
    // row's are reset (below).
    let plain_row = matches!(
        rows.get(app.cursor).map(|r| r.item),
        Some(tree::Item::Doc(_) | tree::Item::Shed(_))
    );
    // The list reads as focused only while it has focus.
    let focused = app.focus == Focus::List;
    // Dynamic id column: widest `id + 2*depth` across visible yak rows (min 4).
    let max_id_len = rows
        .iter()
        .filter_map(|r| Some(r.yak()?.id.chars().count() + 2 * r.depth as usize))
        .max()
        .unwrap_or(4)
        .max(4);
    let id_field_w = max_id_len + 2;
    let blocked = app.blocked_ids();
    let doc_cols = super::doks_tab::DocCols::of(&rows, area.width);
    // The Sheds view: the part of its shed each yak row is in (shaving,
    // asking, recent) labels the row.
    let shed_now = app.sheds.snapshot.as_ref().map(|s| s.report.now);
    let shed_name_w = super::sheds_tab::name_width(&rows);
    let mut shed_kinds = Vec::new();
    let items: Vec<ListItem> = rows
        .iter()
        .map(|r| match r.item {
            tree::Item::Yak(t) => list_item(
                r,
                t,
                id_field_w,
                blocked.contains(&t.id),
                app.selected.contains(&t.id),
                t.awaiting_human(),
                t.awaiting_agent(),
                app.is_starred(&t.id),
                area.width,
                app.is_multi_herd(),
                due_chip(t, app.clock.now(), app.clock.zone()),
                shed_kinds
                    .iter()
                    .find(|(_, y): &&(crate::sheds::Kind, &crate::sheds::Touched)| y.id == t.id)
                    .map(|(k, _)| k.label()),
            ),
            tree::Item::Doc(e) => doc_cols.item(e, area.width),
            tree::Item::Shed(s) => {
                shed_kinds = s.rows();
                let now = shed_now.unwrap_or_else(chrono::Utc::now);
                super::sheds_tab::shed_item(r, s, now, shed_name_w, area.width)
            }
        })
        .collect();
    // Carry the viewport offset across frames (yaks-9009): ratatui's List only
    // scrolls as far as needed to keep the selection visible *relative to the
    // offset it's given*, so a fresh offset of 0 would pin the cursor to the
    // bottom edge whenever it moved up from below the first screenful.
    let mut state = ListState::default()
        .with_offset(app.list_offset.get().min(rows.len() - 1))
        .with_selected(Some(app.cursor.min(rows.len() - 1)));
    // Subtle selection (Python's C_SELECTED): a dark-gray background with the
    // foreground reset to the terminal default, rather than an obtrusive
    // black-on-cyan reverse. Resetting fg keeps the row legible on the dark bg
    // (per-field colours like blue/blue-dim would be low-contrast on 237). The
    // unfocused list uses a slightly darker bg so focus is still clear.
    let fg = if plain_row {
        Style::new()
    } else {
        Style::new().fg(Color::Reset)
    };
    let hl = if focused {
        fg.bg(super::surface::selected())
    } else {
        fg.bg(super::surface::band())
    };
    frame.render_stateful_widget(List::new(items).highlight_style(hl), area, &mut state);
    app.list_offset.set(state.offset());
    let mut hits = app.hits.borrow_mut();
    hits.list = Some(area);
    hits.list_offset = state.offset();
}

/// Truncate `s` to a maximum display width (emoji counted as 2).
pub(crate) fn truncate_disp(s: &str, max: usize) -> String {
    if disp_width(s) <= max {
        return s.to_string();
    }
    let mut out = String::new();
    let mut w = 0;
    for c in s.chars() {
        let cw = if (c as u32) >= 0x1_F000 || c as u32 == 0x2b50 {
            2
        } else {
            1
        };
        if w + cw > max {
            break;
        }
        out.push(c);
        w += cw;
    }
    out
}

/// A stable per-herd colour keyed on the id prefix (FNV-1a into a small
/// palette), used to tint the id so herds are scannable in a multi-herd farm.
pub(crate) fn herd_color(prefix: &str) -> Color {
    const PALETTE: [Color; 6] = [
        Color::Cyan,
        Color::Green,
        Color::Magenta,
        Color::Yellow,
        Color::Blue,
        Color::Red,
    ];
    let mut h: u32 = 0x811c_9dc5;
    for b in prefix.bytes() {
        h ^= b as u32;
        h = h.wrapping_mul(0x0100_0193);
    }
    PALETTE[(h as usize) % PALETTE.len()]
}

/// A hairy or shaving yak's due day and whether it is overdue, for the list
/// row; `None` for a yak with no readable `due:` or a finished one.
fn due_chip(
    t: &Task,
    now: chrono::DateTime<chrono::Utc>,
    zone: crate::zone::Zone,
) -> Option<(String, bool)> {
    let d = crate::recur::due(t).filter(|_| t.status.is_open())?;
    Some((d.day(zone), crate::recur::is_overdue(t, now, zone)))
}

pub(crate) fn list_item<'a>(
    r: &tree::Row<'a>,
    task: &Task,
    id_field_w: usize,
    blocked: bool,
    selected: bool,
    needs_blocked: bool,
    answered: bool,
    starred: bool,
    width: u16,
    multi_herd: bool,
    due: Option<(String, bool)>,
    tag: Option<&str>,
) -> ListItem<'a> {
    let dim = |st: Style| {
        if r.ghost {
            st.add_modifier(Modifier::DIM)
        } else {
            st
        }
    };
    let width = width as usize;
    let indent = "  ".repeat(r.depth as usize);
    // Multi-select gutter: a marked row shows a filled dot, taking precedence
    // over the blocked `*` (selection is the active, user-driven concern; the
    // `*` returns once the row is unmarked).
    let lead = if selected {
        "\u{25cf}"
    } else if blocked {
        "*"
    } else {
        " "
    };
    let body = format!("{indent}{}", task.id);
    let body_w = id_field_w.saturating_sub(1);
    let body_padded = format!("{body:<body_w$}");
    let pri_s = format!("p{} ", task.priority);
    let type_s = format!("{:8} ", task.kind);

    // Right side: right-aligned labels, a star, and a collapse badge.
    let max_lw = (width / 4).clamp(8, 30);
    let label_str = if task.tags.is_empty() {
        String::new()
    } else {
        truncate_disp(&format!("[{}]", task.tags.join(", ")), max_lw)
    };
    let badge = if r.collapsed && r.hidden > 0 {
        format!(" \u{25b6} {} ", r.hidden)
    } else {
        String::new()
    };
    // Right side, as separately-styled spans (labels magenta-dim like Python's
    // C_LABEL, star its own glyph, collapse badge dim).
    let mut right_spans: Vec<Span> = Vec::new();
    let mut right_plain = String::new();
    if !label_str.is_empty() {
        right_plain.push_str(&label_str);
        right_spans.push(Span::styled(
            label_str.clone(),
            dim(Style::new().fg(Color::Magenta).add_modifier(Modifier::DIM)),
        ));
    }
    // Due date of an open yak (yaks-2fc5): dim, and red once it is overdue.
    if let Some((day, overdue)) = &due {
        if !right_plain.is_empty() {
            right_plain.push(' ');
            right_spans.push(Span::raw(" "));
        }
        right_plain.push_str(day);
        let style = if *overdue {
            Style::new().fg(Color::Red).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(Color::DarkGray)
        };
        right_spans.push(Span::styled(day.clone(), dim(style)));
    }
    // Needs badge (⏳): this yak is awaiting a human (an open `needs` question). A
    // distinct axis from the dependency-blocked `*` lead, so it can co-occur.
    if needs_blocked {
        if !right_plain.is_empty() {
            right_plain.push(' ');
            right_spans.push(Span::raw(" "));
        }
        right_plain.push('\u{23f3}');
        right_spans.push(Span::styled(
            "\u{23f3}",
            dim(Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ));
    }
    // Answered badge (✅): `needs: agent`, answered and waiting for an agent to
    // pick it up. Not a blocker, so no warning accent, and never ⏳.
    if answered {
        if !right_plain.is_empty() {
            right_plain.push(' ');
            right_spans.push(Span::raw(" "));
        }
        right_plain.push('\u{2705}');
        right_spans.push(Span::raw("\u{2705}"));
    }
    if starred {
        if !right_plain.is_empty() {
            right_plain.push(' ');
            right_spans.push(Span::raw(" "));
        }
        right_plain.push('\u{2b50}');
        right_spans.push(Span::raw("\u{2b50}"));
    }
    if !badge.is_empty() {
        right_plain.push_str(&badge);
        right_spans.push(Span::styled(
            badge.clone(),
            dim(Style::new().fg(Color::DarkGray)),
        ));
    }
    let left_fixed = 1 + disp_width(&body_padded) + disp_width(&pri_s) + disp_width(&type_s);
    // A dim tag (a yak's part of its shed), only where the whole title still
    // fits beside it.
    let title_full = disp_width(&format!("{} {}", task.status.emoji(), task.title));
    if let Some(tag) = tag
        .filter(|t| left_fixed + title_full + disp_width(&right_plain) + disp_width(t) + 2 <= width)
    {
        let gap = if right_plain.is_empty() { "" } else { " " };
        right_plain.insert_str(0, &format!("{tag}{gap}"));
        right_spans.insert(
            0,
            Span::styled(
                format!("{tag}{gap}"),
                dim(Style::new().add_modifier(Modifier::DIM)),
            ),
        );
    }
    let rw = disp_width(&right_plain);
    let title_avail = width.saturating_sub(left_fixed + rw + 1);
    let title = truncate_disp(
        &format!("{} {}", task.status.emoji(), task.title),
        title_avail,
    );
    let used = left_fixed + disp_width(&title);
    let pad = width.saturating_sub(used + rw);

    let lead_style = if selected {
        Style::new().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else if blocked {
        Style::new().fg(Color::Magenta).add_modifier(Modifier::BOLD)
    } else {
        Style::new()
    };
    // In a multi-herd farm, tint the id's herd prefix so herds are scannable at
    // a glance (the tail stays blue); a single-herd farm renders the id plain.
    let id_spans = if multi_herd {
        let (prefix, rest) = match task.id.split_once('-') {
            Some((p, tail)) => (p.to_string(), format!("-{tail}")),
            None => (task.id.clone(), String::new()),
        };
        let plain_w = disp_width(&indent) + task.id.chars().count();
        let pad = " ".repeat(body_w.saturating_sub(plain_w));
        vec![
            Span::styled(indent.clone(), dim(Style::new().fg(Color::Blue))),
            Span::styled(prefix.clone(), dim(Style::new().fg(herd_color(&prefix)))),
            Span::styled(format!("{rest}{pad}"), dim(Style::new().fg(Color::Blue))),
        ]
    } else {
        vec![Span::styled(body_padded, dim(Style::new().fg(Color::Blue)))]
    };
    let mut spans = vec![Span::styled(lead.to_string(), lead_style)];
    spans.extend(id_spans);
    spans.push(Span::styled(pri_s, dim(priority_style(task.priority))));
    spans.push(Span::styled(type_s, dim(Style::new().fg(Color::Cyan))));
    spans.push(Span::styled(title, dim(Style::new())));
    if pad > 0 {
        spans.push(Span::raw(" ".repeat(pad)));
    }
    spans.extend(right_spans);
    ListItem::new(Line::from(spans))
}

/// A one-line sticky summary of the task — status emoji, id, type, priority,
/// title, star — pinned above the detail content when it has scrolled past the
/// frontmatter, so you never lose track of which yak you're reading. Filled with
/// a faint background band so it reads as a header rather than a content row.
pub(crate) fn pinned_header_line(task: &Task, starred: bool, width: u16) -> Line<'static> {
    let bg = super::surface::band();
    let w = width as usize;
    let emoji = task.status.emoji(); // display width 2
    let type_s = format!("{:8}", task.kind);
    let pri_s = format!("p{}", task.priority);
    let star = if starred { " \u{2b50}" } else { "" };
    let star_w = disp_width(star);

    // Left cluster: " 🪒 yaks-b517  feature  p2  "
    let left_plain = format!(" {emoji} {}  {type_s} {pri_s}  ", task.id);
    let left_w = disp_width(&left_plain);
    let title_avail = w.saturating_sub(left_w + star_w);
    let title = truncate_disp(&task.title, title_avail);
    let pad = w.saturating_sub(left_w + disp_width(&title) + star_w);

    let band = Style::new().bg(bg);
    let mut spans = vec![
        Span::styled(format!(" {emoji} "), band),
        Span::styled(task.id.clone(), band.fg(Color::Blue)),
        Span::styled("  ".to_string(), band),
        Span::styled(type_s, band.fg(Color::Cyan)),
        Span::styled(" ".to_string(), band),
        Span::styled(pri_s, priority_style(task.priority).bg(bg)),
        Span::styled("  ".to_string(), band),
        Span::styled(title, band.add_modifier(Modifier::BOLD)),
    ];
    if pad > 0 {
        spans.push(Span::styled(" ".repeat(pad), band));
    }
    if starred {
        spans.push(Span::styled(star.to_string(), band));
    }
    Line::from(spans)
}

/// Rows the pinned detail header occupies: the one-line summary plus the rule
/// under it. `App::detail_rows_shown` subtracts this from the viewport.
pub(crate) const PINNED_ROWS: u16 = 2;

pub(crate) fn render_detail(app: &App, frame: &mut Frame, area: Rect) {
    let focused = app.focus == Focus::Detail;
    // The left divider is drawn by render() via right_divider(); we render the
    // content into the already-inset area.
    let Some(task) = app.selected() else {
        let p = Paragraph::new(Span::styled("(no task)", Style::new().fg(Color::DarkGray)));
        frame.render_widget(p, area);
        return;
    };
    // Capture the content width so the row-indexed model (and event handlers)
    // wrap to exactly what's rendered here.
    app.detail_width.set(area.width);
    // When scrolled past the frontmatter, reserve the top two rows for a sticky
    // header and a dim rule under it, and push the content down by both
    // (nothing is hidden — the pin adds rows rather than overlaying them).
    // `App::detail_rows_shown` mirrors this arithmetic (PINNED_ROWS).
    let pinned = app.detail.scroll > 0 && area.height > PINNED_ROWS;
    let content_area = if pinned {
        Rect {
            x: area.x,
            y: area.y + PINNED_ROWS,
            width: area.width,
            height: area.height - PINNED_ROWS,
        }
    } else {
        area
    };
    let lines = app.detail_dlines();
    // The peek pane under the lines (yaks-a004), as under a dok: what the
    // link under the cursor names, read by `links::peek`.
    let (content_area, peek_area) = if app.detail_has_peek() {
        let [page, peek] = Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(super::follow::PEEK_ROWS),
        ])
        .areas(content_area);
        (page, Some(peek))
    } else {
        (content_area, None)
    };
    app.hits.borrow_mut().detail = Some(content_area);
    if let Some(peek) = peek_area {
        render_peek(app, app.detail_link(), None, frame, peek);
    }
    // Widest row of each no-wrap block, to clamp its sideways scroll.
    let extents = detail::nowrap_extents(&lines);
    let hscroll = |dl: &detail::DLine| {
        dl.nowrap_key()
            .and_then(|k| {
                extents.get(&k).map(|&e| {
                    app.detail
                        .block_hscroll(k, e, app.detail_width.get() as usize)
                })
            })
            .unwrap_or(0)
    };
    let width = area.width as usize;
    let mut rendered = if focused {
        render_nav_rows(&lines, &app.detail, app.register.as_deref(), width, hscroll)
    } else {
        // Unfocused: no cursor, selection, current link or find highlights.
        lines
            .iter()
            .enumerate()
            .map(|(i, dl)| render_dline(dl, None, i, &[], None, hscroll(dl), width))
            .collect()
    };
    // The in-place editor draws over its rows below; they stay blank.
    for (i, dl) in lines.iter().enumerate() {
        if dl.kind == detail::Kind::Editor {
            rendered[i] = Line::default();
        }
    }
    // A yak the pane shows from outside the list says so on its header row
    // (the pinned one, once the first has scrolled off).
    let not_listed = || {
        Span::styled(
            "  not in this list",
            Style::new().fg(Color::DarkGray).add_modifier(Modifier::DIM),
        )
    };
    if app.pane_yak_id().is_some() && !pinned {
        if let Some(first) = rendered.first_mut() {
            first.spans.push(not_listed());
        }
    }
    // No wrap: link/match highlight columns must stay valid. (Rows of a
    // no-wrap block are clipped to the pane by `render_dline`.)
    let p = Paragraph::new(rendered).scroll((app.detail.scroll, 0));
    frame.render_widget(p, content_area);
    super::imageview::render_inline_images(app, &lines, app.detail.scroll, frame, content_area);
    if let Some(be) = app.block_edit() {
        let at = blockedit::render_block_editor(be, &lines, app.detail.scroll, frame, content_area);
        app.hits.borrow_mut().block_editor = at;
    }
    if pinned {
        let mut header = pinned_header_line(task, app.is_starred(&task.id), area.width);
        if app.pane_yak_id().is_some() {
            header.spans.push(not_listed());
        }
        frame.render_widget(Paragraph::new(header), Rect { height: 1, ..area });
        let rule = Line::from(Span::styled(
            "\u{2500}".repeat(area.width as usize),
            Style::new().fg(Color::DarkGray),
        ));
        frame.render_widget(
            Paragraph::new(rule),
            Rect {
                y: area.y + 1,
                height: 1,
                ..area
            },
        );
    }
}

/// Shown at a clipped edge of a no-wrap row: more to the left / to the right.
const CLIP_LEFT: char = '\u{2039}'; // ‹
const CLIP_RIGHT: char = '\u{203a}'; // ›

/// Clip one styled no-wrap row to a `width`-column pane scrolled `off` columns
/// right. The first `gutter` display columns (a quote bar, a list marker) stay
/// put; the rest scrolls. A dim [`CLIP_LEFT`] / [`CLIP_RIGHT`] replaces the
/// edge cell where text is hidden. Returns the row unchanged when it fits.
fn clip_nowrap(
    cells: Vec<(char, Style)>,
    gutter: usize,
    off: usize,
    width: usize,
    marker: Style,
) -> Vec<(char, Style)> {
    use unicode_width::UnicodeWidthChar;
    let w = |c: char| c.width().unwrap_or(0);
    // Split the gutter (display columns) from the scrolling text.
    let mut split = 0;
    let mut gw = 0;
    while split < cells.len() && gw < gutter {
        gw += w(cells[split].0);
        split += 1;
    }
    let (head, text) = cells.split_at(split);
    let avail = width.saturating_sub(gw);
    let total: usize = text.iter().map(|(c, _)| w(*c)).sum();
    if off == 0 && total <= avail {
        return cells;
    }
    // Keep the cells wholly inside the window [off, off + avail).
    let mut out: Vec<(char, Style)> = head.to_vec();
    let mut col = 0;
    let mut kept: Vec<(char, Style)> = Vec::new();
    for &(c, st) in text {
        let cw = w(c);
        if col >= off && col + cw <= off + avail {
            kept.push((c, st));
        }
        col += cw;
    }
    let clipped_left = off > 0 && !text.is_empty();
    let clipped_right = total > off + avail;
    // A marker takes one column; a wide cell it replaces leaves a blank.
    let mark = |kept: &mut Vec<(char, Style)>, at_start: bool, ch: char| {
        if kept.is_empty() {
            return;
        }
        let i = if at_start { 0 } else { kept.len() - 1 };
        let pad = w(kept[i].0) > 1;
        let bg = kept[i].1.bg;
        let mut st = marker;
        st.bg = bg;
        kept[i] = (ch, st);
        if pad {
            kept.insert(i + 1, (' ', st));
        }
    };
    if clipped_right {
        mark(&mut kept, false, CLIP_RIGHT);
    }
    if clipped_left {
        if kept.is_empty() {
            kept.push((CLIP_LEFT, marker));
        } else {
            mark(&mut kept, true, CLIP_LEFT);
        }
    }
    out.extend(kept);
    out
}

/// Render one detail line by computing a per-char style (base -> link ->
/// find-match, each overriding the last) and coalescing equal runs into spans.
/// A no-wrap row (code, table) is clipped to the pane `width`, scrolled
/// `hscroll` columns (yaks-c733).
pub(crate) fn render_dline<'a>(
    dl: &'a detail::DLine,
    cur: Option<&detail::Jump>,
    line_idx: usize,
    matches: &[(usize, usize, bool)],
    line_bg: Option<Color>,
    hscroll: usize,
    width: usize,
) -> Line<'a> {
    let chars: Vec<char> = dl.text.chars().collect();
    let n = chars.len();
    if n == 0 {
        // Still paint the cursor/selection background across an empty line.
        if let Some(bg) = line_bg {
            return Line::from(Span::styled(" ", Style::new().bg(bg)));
        }
        return Line::from(String::new());
    }
    // Base per-char style: dim label prefix on plain fields; section headers cyan.
    // Only a field's first row carries the label: a soft-wrapped continuation
    // row (`cont`) is all value, so it must not inherit the label colour in the
    // columns where the label sat (yaks-2d17).
    let label_end = if dl.links.is_empty() && dl.kind == detail::Kind::Field && !dl.cont && n > 13 {
        13
    } else {
        0
    };
    let mut styles: Vec<Style> = (0..n)
        .map(|i| {
            let base = if i < label_end {
                Style::new().fg(Color::DarkGray)
            } else if dl.kind == detail::Kind::Section {
                Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else if dl.kind == detail::Kind::Warn {
                // Warning accent (the `needs` blocker): the whole line, label
                // included, reads yellow + bold so it stands out as a blocker.
                Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::new()
            };
            match line_bg {
                Some(bg) => base.bg(bg),
                None => base,
            }
        })
        .collect();
    // Markdown highlight layer (body lines): sits beneath links/find so a link
    // inside emphasis still wins. Keep the line's cursor/selection background.
    for sp in &dl.md {
        let end = (sp.start + sp.len).min(n);
        let st = match line_bg {
            Some(bg) => sp.style.bg(bg),
            None => sp.style,
        };
        for s in styles.iter_mut().take(end).skip(sp.start) {
            *s = st;
        }
    }
    for (col, len, _) in &dl.links {
        let is_current = cur.is_some_and(|j| j.line == line_idx && j.col == *col);
        let mut st = if is_current {
            // Python C_LINK_SEL: blue on the subtle 237 background, emphasised.
            Style::new()
                .fg(Color::Blue)
                .bg(super::surface::selected())
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            Style::new()
                .fg(Color::Blue)
                .add_modifier(Modifier::UNDERLINED)
        };
        // Preserve the line's cursor/selection background on non-current links.
        if !is_current {
            if let Some(bg) = line_bg {
                st = st.bg(bg);
            }
        }
        for s in styles.iter_mut().take((col + len).min(n)).skip(*col) {
            *s = st;
        }
    }
    for (col, len, is_current) in matches {
        let st = if *is_current {
            Style::new()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(Color::Black).bg(Color::Yellow)
        };
        for s in styles.iter_mut().take((col + len).min(n)).skip(*col) {
            *s = st;
        }
    }
    let mut cells: Vec<(char, Style)> = chars.into_iter().zip(styles).collect();
    if let Some(nw) = dl.nowrap {
        cells = clip_nowrap(cells, nw.gutter, hscroll, width, palette::marker());
    }
    let mut spans: Vec<Span> = Vec::new();
    let mut i = 0;
    while i < cells.len() {
        let st = cells[i].1;
        let mut run = String::new();
        while i < cells.len() && cells[i].1 == st {
            run.push(cells[i].0);
            i += 1;
        }
        spans.push(Span::styled(run, st));
    }
    Line::from(spans)
}

/// Strip the common leading-space prefix across non-empty lines (Python's
/// `dedent_block`), so a copied detail block isn't indented by the pane layout.
pub(crate) fn dedent(lines: &[String]) -> Vec<String> {
    let strip = lines
        .iter()
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.len() - s.trim_start_matches(' ').len())
        .min()
        .unwrap_or(0);
    if strip == 0 {
        return lines.to_vec();
    }
    lines
        .iter()
        .map(|s| {
            if s.len() >= strip {
                s[strip..].to_string()
            } else {
                s.clone()
            }
        })
        .collect()
}

/// (line, col, len) of every case-insensitive occurrence of `q` in the lines.
pub(crate) fn detail_scan(lines: &[detail::DLine], q: &str) -> Vec<(usize, usize, usize)> {
    let ql = q.to_lowercase();
    if ql.is_empty() {
        return vec![];
    }
    let qlen = ql.chars().count();
    let mut out = Vec::new();
    for (i, l) in lines.iter().enumerate() {
        let text = l.text.to_lowercase();
        let mut from = 0usize;
        while let Some(pos) = text[from..].find(&ql) {
            let byte = from + pos;
            let col = text[..byte].chars().count();
            out.push((i, col, qlen));
            from = byte + ql.len();
        }
    }
    out
}

pub(crate) fn render_status(app: &App, frame: &mut Frame, area: Rect) {
    // An active `:` command line owns the status line.
    if let Some(cmd) = &app.cmdline {
        frame.render_widget(
            Paragraph::new(Span::styled(
                format!(":{cmd}"),
                Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )),
            area,
        );
        return;
    }
    // The `?` popup's hint: the help key again (over the list or pane), a
    // listed key runs it or goes to the editor, any other key closes.
    if app.keys_popup {
        let text = match app.popup_subject() {
            None => {
                let help = app
                    .keymap
                    .first_label(app.surface_ctx(), super::keymap::Cmd::AppHelp);
                format!(
                    "a listed key runs it · {help} again: the full reference · any other key closes"
                )
            }
            Some(_) => "a listed key goes to the editor · any other key closes".to_string(),
        };
        frame.render_widget(
            Paragraph::new(Span::styled(text, Style::new().fg(Color::DarkGray))),
            area,
        );
        return;
    }
    // A single-line editor field owns the status line while active.
    if let Overlay::Edit(ed) = &app.overlay {
        if ed.single_line {
            render_line_editor(ed, frame, area);
            return;
        }
    }
    // The fuzzy picker's query line owns the status line too.
    if let Overlay::Fuzzy(fp) = &app.overlay {
        render_query_line("search: ", &fp.query, frame, area);
        return;
    }
    // Inline search field.
    if let Overlay::Search(sb) = &app.overlay {
        render_query_line("/", &sb.query, frame, area);
        return;
    }
    // The pane's find field (a yak's detail, or a dok in the viewer).
    if let Overlay::Find(sb) = &app.overlay {
        render_query_line("find: ", &sb.query, frame, area);
        return;
    }
    // While editing, a `<prefix>-…` ref context surfaces a Tab-to-complete hint
    // on the help bar (opt-in: nothing pops up until you press Tab).
    let ref_context = match &app.overlay {
        Overlay::Edit(ed) if !ed.single_line => {
            completion_context(&ed.state, &app.ref_prefix).is_some()
        }
        Overlay::BlockEdit(be) => completion_context(&be.editor.state, &app.ref_prefix).is_some(),
        Overlay::Create(f) => f
            .description_editor()
            .is_some_and(|e| completion_context(e, &app.ref_prefix).is_some()),
        _ => false,
    };
    if ref_context {
        frame.render_widget(
            Paragraph::new(Span::styled(
                "Tab → complete yak · Esc dismiss",
                Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            )),
            area,
        );
        return;
    }
    // View manager help hint.
    if matches!(&app.overlay, Overlay::ViewPicker(_)) {
        frame.render_widget(
            Paragraph::new(Span::styled(
                "Enter open · p pin · J/K move · r rename · d delete · Esc close",
                Style::new().fg(Color::DarkGray),
            )),
            area,
        );
        return;
    }
    // Drawer help hint. In vim a lone Esc on a text row drops it to Normal, so
    // cancel is a double-Esc or Ctrl-C; emacs cancels on a plain Esc.
    if matches!(&app.overlay, Overlay::Drawer(_)) {
        let cancel = if app.editor_vim() {
            "EscEsc/Ctrl-C cancel"
        } else {
            "Esc cancel"
        };
        frame.render_widget(
            Paragraph::new(Span::styled(
                format!("↑↓/Tab rows · ←→ chips · Space toggle · C clear · Enter apply · {cancel}"),
                Style::new().fg(Color::DarkGray),
            )),
            area,
        );
        return;
    }
    // Help reference hint.
    if matches!(&app.overlay, Overlay::Help(_)) {
        let scroll = if app.editor_vim() {
            "j/k scroll · d/u page · g/G top/bottom"
        } else {
            "↑/↓ scroll · PgUp/PgDn page · Home/End top/bottom"
        };
        frame.render_widget(
            Paragraph::new(Span::styled(
                format!("{scroll} · ?/q/Esc close"),
                Style::new().fg(Color::DarkGray),
            )),
            area,
        );
        return;
    }
    // Attachment viewer: the destination of the link under the cursor, else
    // the key hints.
    if app.attach().is_some() && matches!(app.overlay, Overlay::None) {
        let text = link_hint(app, area.width as usize).unwrap_or_else(|| {
            let mv = if app.editor_vim() { "j/k" } else { "↑/↓" };
            format!("{mv} · Tab:link Enter:follow · o/i back/fwd · p:peek · / find · v sel · E edit · O open externally · Esc close · q quit")
        });
        let text = with_match_chip(app, text);
        frame.render_widget(
            Paragraph::new(Span::styled(text, Style::new().fg(Color::DarkGray))),
            area,
        );
        return;
    }
    // Image viewer hint.
    if app.image().is_some() && matches!(app.overlay, Overlay::None) {
        let v = app.image().expect("checked above");
        let pan = match (v.pannable(), app.editor_vim()) {
            (false, _) => "",
            (true, true) => " · h/j/k/l pan",
            (true, false) => " · arrows pan",
        };
        frame.render_widget(
            Paragraph::new(Span::styled(
                format!(
                    "zoom {} · +/- · 0 actual · w width · f fit{pan} · O open externally · Esc close · q quit",
                    v.zoom_label()
                ),
                Style::new().fg(Color::DarkGray),
            )),
            area,
        );
        return;
    }
    // The in-place block editor: its mode (vim) and the save/cancel keys.
    if let Overlay::BlockEdit(be) = &app.overlay {
        frame.render_widget(Paragraph::new(be.status_line()), area);
        return;
    }
    // Create/edit-form help hint. The `(need title)` marker mirrors Python's guard.
    if let Overlay::Create(f) = &app.overlay {
        let commit = if f.title_text().is_empty() {
            "(need title)"
        } else if f.is_editing() {
            "Ctrl-S save"
        } else {
            "Ctrl-S create"
        };
        // In vim, Esc is modal (drops the field to Normal); cancel is a rapid
        // double-Esc or Ctrl-C. In emacs, Esc cancels directly.
        let cancel = if app.editor_vim() {
            "EscEsc/Ctrl-C cancel"
        } else {
            "Esc cancel"
        };
        frame.render_widget(
            Paragraph::new(Span::styled(
                format!("Tab/↑↓ rows · ←→ chips · {commit} · {cancel}"),
                Style::new().fg(Color::DarkGray),
            )),
            area,
        );
        return;
    }
    // Otherwise: an active modal prompt, else a transient notification, else the
    // context help hint. (A multi-line editor falls through to notification/help.)
    // Otherwise: a single-key modal prompt, else the context help bar.
    // (Notification + active-filter indicator now live on the tab row.)
    let (text, style) = match &app.overlay {
        Overlay::Pick { prompt, .. } | Overlay::Confirm { prompt, .. } => (
            prompt.clone(),
            Style::new()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        _ => (
            with_match_chip(
                app,
                link_hint(app, area.width as usize).unwrap_or_else(|| help_hint(app)),
            ),
            Style::new().fg(Color::DarkGray),
        ),
    };
    frame.render_widget(Paragraph::new(Span::styled(text, style)), area);
}

/// `match 2/5 "q"` while the search register is set and the pane shows a
/// text subject (a yak's detail, or a dok): which of its matches in this
/// subject is current (0 before `n` lands on one), and of how many.
fn match_chip(app: &App) -> Option<String> {
    let q = app.register.as_deref()?;
    let (current, total) = if let Some((rows, _)) = app.attachment_rows() {
        let nav = &app.attach()?.nav;
        (
            nav.matched,
            super::linenav::find_matches(&rows, Some(q)).len(),
        )
    } else if app.focus == Focus::Detail && app.selected().is_some() {
        let rows = app.detail_dlines();
        (
            app.detail.matched,
            super::linenav::find_matches(&rows, Some(q)).len(),
        )
    } else {
        return None;
    };
    Some(format!(
        "match {}/{total} \"{}\"",
        current.map_or(0, |i| i + 1),
        truncate_disp(q, 24)
    ))
}

/// `text`, the status bar's usual hint, behind the [`match_chip`] if any.
fn with_match_chip(app: &App, text: String) -> String {
    match match_chip(app) {
        Some(chip) => format!("{chip}  {text}"),
        None => text,
    }
}

/// Where the link Enter would follow on the detail cursor's row points, for
/// the status bar: a collapsed `[text](url)` shows only its text in the pane,
/// so its destination is shown here instead of the key hints. Only URLs and
/// attachments and source files (what opens externally); yak ids and footnotes speak for
/// themselves.
fn link_hint(app: &App, width: usize) -> Option<String> {
    let jump = if let Some((rows, _)) = app.attachment_rows() {
        let v = app.attach()?;
        v.nav.link_at(&rows)?
    } else {
        if app.doc.is_some() || app.focus != Focus::Detail || !matches!(app.overlay, Overlay::None)
        {
            return None;
        }
        app.detail_link()?
    };
    let dest = super::linenav::link_dest(jump.target)?;
    Some(truncate_disp(&format!("→ {dest}"), width))
}

/// The tab row's query chip, apart from the facets' `filter:` chip:
/// `/"widget" · 14 in All`, where 14 is what the All view holds that matches.
/// `None` without a query.
fn query_chip(app: &App) -> Option<String> {
    let q = app.query.as_deref()?;
    let shown = truncate_disp(q, 30);
    let in_all = app
        .views
        .iter()
        .find(|v| v.key == view::ALL_KEY)
        .map(|v| format!(" \u{b7} {} in All", format_count(app.view_count(v))))
        .unwrap_or_default();
    Some(format!(" /\"{shown}\"{in_all} "))
}

/// One-line description of the active content facets (for the status line).
pub(crate) fn filter_summary(f: &FilterSpec) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(s) = f.search.as_deref().filter(|s| !s.is_empty()) {
        parts.push(format!("\u{201c}{s}\u{201d}"));
    }
    if !f.types.is_empty() {
        parts.push(f.types.join("|"));
    }
    if !f.priorities.is_empty() {
        parts.push(
            f.priorities
                .iter()
                .map(|p| format!("p{p}"))
                .collect::<Vec<_>>()
                .join("|"),
        );
    }
    if !f.tags.is_empty() {
        parts.push(f.tags.join(","));
    }
    if f.ready_only {
        parts.push("ready".into());
    }
    if f.tangled_only {
        parts.push("tangled".into());
    }
    if f.needs_only {
        parts.push("inbox".into());
    }
    if parts.is_empty() {
        "(all)".into()
    } else {
        parts.join(" · ")
    }
}

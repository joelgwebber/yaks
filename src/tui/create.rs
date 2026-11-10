//! The create/edit task form and its rendering. Split out of `tui.rs`
//! (yaks-b1cc / b1cc/3). Shared form helpers (`text_field`, `multiline_field`,
//! `kind_index`, `pri_index`, the `*_CHOICES` consts, `render_chip_row`/
//! `render_text_row`) stay in `tui.rs` and are reached via `super::`, as the
//! filter drawer uses them too; the editor render helpers live in `editor.rs`.

use std::cell::RefCell;

use edtui::{EditorEventHandler, EditorState, EditorView};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

use super::drafts::DraftBadge;
use super::mouse::HitMap;
use super::palette;
use super::{
    CREATE_LABEL_W, HEADER_ROWS, PRI_CHOICES, TYPE_CHOICES, WAKE_CHOICES, disp_width, editor_theme,
    kind_index, make_handler, multiline_field, pri_index, render_chip_row, render_text_row,
    set_md_highlights, text_field,
};
use crate::fields::Value;
use crate::model::Task;
use crate::recur;

/// The create/edit task form: a right-pane form modeled on `Drawer`. Two chip
/// rows (type/priority) are **single-select** — the cursor *is* the value —
/// plus single-line title/labels/source (and herd) rows, then one multi-line
/// **description** editor below them. Create (`c`/`C`) and edit (`E`) have the
/// same shape (yaks-255d): an edit seeds the description from the yak and
/// leaves its comments alone; a single comment, or the description block in
/// place, is edited in the detail pane (`blockedit.rs`). `Ctrl-N/P`/Tab walk
/// every row. `Ctrl-S` commits (create or update), `Esc`/`Ctrl-C` cancels.
pub(crate) struct CreateForm {
    pub(crate) title: RefCell<EditorState>,
    pub(crate) tags: RefCell<EditorState>,
    /// The `source:` external-issue URL (yaks-b0b4); empty = no source.
    pub(crate) source: RefCell<EditorState>,
    /// `due:` as typed: a date, a datetime or `+2d` / `+1w`; empty = none.
    pub(crate) due: RefCell<EditorState>,
    /// `every:` as typed: an ISO 8601 duration such as `P1W`; empty = none.
    pub(crate) every: RefCell<EditorState>,
    /// Index into `WAKE_CHOICES` (single-select cursor==value).
    pub(crate) wake_idx: usize,
    /// The description editor, the last row. An edit seeds it from the yak's
    /// description (never its comments).
    pub(crate) description: RefCell<EditorState>,
    /// An edit: the yak's whole body when the form opened. It is the `base` a
    /// changed description is saved against (the farm keeps comments appended
    /// since, and refuses anything else), and what tells a changed description
    /// from an untouched one. Empty on a create.
    pub(crate) seed_body: String,
    /// Index into `TYPE_CHOICES` (single-select cursor==value).
    pub(crate) kind_idx: usize,
    /// Index into `PRI_CHOICES` (single-select cursor==value; default → p3).
    pub(crate) pri_idx: usize,
    pub(crate) row: usize,
    /// Create: the (optional) parent for the new task. Edit: unused (reparent
    /// is a separate action); shown in the header for context.
    pub(crate) parent: Option<String>,
    /// Multi-herd farm only: the herd (id prefix) choices and the selected
    /// index, rendered as a chip row (like type/priority). Empty in a
    /// single-herd farm (then `create` uses the config default). On create the
    /// selection is the new yak's prefix; on edit it's the yak's current herd,
    /// and changing it renames the id prefix on commit (yaks-71d1).
    pub(crate) herds: Vec<String>,
    pub(crate) herd_idx: usize,
    /// `Some(id)` when editing an existing task; `None` when creating.
    pub(crate) edit_id: Option<String>,
    /// An edit: the yak's `updated` when the form (or the draft it was
    /// restored from) began, to tell at the next restore whether the yak has
    /// changed since (yaks-683f).
    pub(crate) base_updated: Option<String>,
    /// Set when the form was restored from a saved draft (yaks-683f).
    pub(crate) draft: Option<DraftBadge>,
    pub(crate) handler: EditorEventHandler,
}

impl CreateForm {
    pub(crate) fn new(
        vim: bool,
        parent: Option<String>,
        herds: Vec<String>,
        herd_idx: usize,
    ) -> Self {
        CreateForm {
            title: text_field("", vim),
            tags: text_field("", vim),
            source: text_field("", vim),
            due: text_field("", vim),
            every: text_field("", vim),
            wake_idx: 0,
            description: multiline_field("", vim),
            seed_body: String::new(),
            kind_idx: 0,           // task
            pri_idx: pri_index(3), // p3
            row: 0,
            parent,
            herds,
            herd_idx,
            edit_id: None,
            base_updated: None,
            draft: None,
            handler: make_handler(vim),
        }
    }

    /// Seed the form from an existing task for editing: the header fields and
    /// the description; the comments are left untouched by this form.
    pub(crate) fn for_edit(vim: bool, task: &Task) -> Self {
        CreateForm {
            title: text_field(&task.title, vim),
            tags: text_field(&task.tags.join(", "), vim),
            source: text_field(task.source.as_deref().unwrap_or(""), vim),
            due: text_field(&extra_text(task, recur::DUE), vim),
            every: text_field(&extra_text(task, recur::EVERY), vim),
            wake_idx: WAKE_CHOICES
                .iter()
                .position(|w| *w == extra_text(task, recur::WAKE))
                .unwrap_or(0),
            description: multiline_field(&seed_description(&task.body), vim),
            seed_body: task.body.clone(),
            kind_idx: kind_index(&task.kind),
            pri_idx: pri_index(task.priority),
            row: 0,
            parent: task.parent.clone(),
            herds: Vec::new(),
            herd_idx: 0,
            edit_id: Some(task.id.clone()),
            base_updated: task.updated.clone(),
            draft: None,
            handler: make_handler(vim),
        }
    }

    pub(crate) fn is_editing(&self) -> bool {
        self.edit_id.is_some()
    }

    /// Whether the herd-picker chip row is shown: only when the farm spans more
    /// than one herd (id prefix). Shown for both create and edit — on edit,
    /// changing it renames the yak's id prefix on commit (yaks-71d1).
    pub(crate) fn has_herd_row(&self) -> bool {
        self.herds.len() > 1
    }

    /// Header rows (everything but the description), including the optional herd row.
    pub(crate) fn header_rows(&self) -> usize {
        HEADER_ROWS + self.has_herd_row() as usize
    }

    /// The chosen target herd (id prefix), or `None` in a single-herd farm.
    pub(crate) fn target_herd(&self) -> Option<String> {
        self.herds.get(self.herd_idx).cloned()
    }

    pub(crate) fn row_count(&self) -> usize {
        self.header_rows() + 1
    }

    pub(crate) fn is_content_row(&self) -> bool {
        self.description_editor().is_some()
    }

    /// The description editor, when the cursor is on it.
    pub(crate) fn description_editor(&self) -> Option<&RefCell<EditorState>> {
        (self.row == self.header_rows()).then_some(&self.description)
    }

    /// Single-line text rows (title, labels, source); the description is
    /// multi-line and handled separately.
    pub(crate) fn is_line_text_row(&self) -> bool {
        matches!(self.row, 0 | 3 | 4 | 5 | 6)
    }

    /// The editor backing the current single-line text row (title/labels/source).
    pub(crate) fn line_editor(&self) -> Option<&RefCell<EditorState>> {
        match self.row {
            0 => Some(&self.title),
            3 => Some(&self.tags),
            4 => Some(&self.source),
            5 => Some(&self.due),
            6 => Some(&self.every),
            _ => None,
        }
    }

    /// Move the single-select chip cursor on a chip row (wrapping).
    pub(crate) fn move_chip(&mut self, delta: i32) {
        match self.row {
            1 => {
                let n = TYPE_CHOICES.len() as i32;
                self.kind_idx = (self.kind_idx as i32 + delta).rem_euclid(n) as usize;
            }
            2 => {
                let n = PRI_CHOICES.len() as i32;
                self.pri_idx = (self.pri_idx as i32 + delta).rem_euclid(n) as usize;
            }
            7 => {
                let n = WAKE_CHOICES.len() as i32;
                self.wake_idx = (self.wake_idx as i32 + delta).rem_euclid(n) as usize;
            }
            r if r == HEADER_ROWS && self.has_herd_row() => {
                let n = self.herds.len() as i32;
                self.herd_idx = (self.herd_idx as i32 + delta).rem_euclid(n) as usize;
            }
            _ => {}
        }
    }

    /// Select chip `idx` on the current chip row (a click; the keys use
    /// [`CreateForm::move_chip`]). An index past the row's chips is ignored.
    pub(crate) fn set_chip(&mut self, idx: usize) {
        let (slot, n) = match self.row {
            1 => (&mut self.kind_idx, TYPE_CHOICES.len()),
            2 => (&mut self.pri_idx, PRI_CHOICES.len()),
            7 => (&mut self.wake_idx, WAKE_CHOICES.len()),
            r if r == HEADER_ROWS && self.has_herd_row() => (&mut self.herd_idx, self.herds.len()),
            _ => return,
        };
        if idx < n {
            *slot = idx;
        }
    }

    pub(crate) fn title_text(&self) -> String {
        self.title.borrow().lines.to_string().trim().to_string()
    }

    /// The trimmed source URL; empty means "no source".
    pub(crate) fn source_text(&self) -> String {
        self.source.borrow().lines.to_string().trim().to_string()
    }

    /// What the form says about each schedule field, as the farm should hold
    /// it: `(key, Some(value))` to set, `(key, None)` to clear. On a create
    /// only the fields that are set; on an edit only those that differ from
    /// `cur`, so an untouched field never rewrites the file. A due date is
    /// resolved (`+2d` becomes a date); text that does not read is passed
    /// through as typed (`problem` refuses it before anything is written).
    pub(crate) fn schedule_edits(
        &self,
        cur: Option<&Task>,
        today: chrono::NaiveDate,
    ) -> Vec<(&'static str, Option<String>)> {
        let typed =
            |cell: &RefCell<EditorState>| cell.borrow().lines.to_string().trim().to_string();
        let due = typed(&self.due);
        let every = typed(&self.every);
        let wake = WAKE_CHOICES[self.wake_idx];
        let wake = (wake != "none").then(|| wake.to_string());
        // (key, the value to store, what the row shows). A set `due` is stored
        // resolved; one that does not read goes through as typed, and
        // `problem` refuses it before anything is written.
        let stored_due = (!due.is_empty())
            .then(|| recur::resolve_due(&due, today).unwrap_or_else(|_| due.clone()));
        let stored_every = (!every.is_empty()).then(|| every.clone());
        let mut out = Vec::new();
        for (key, now, shown) in [
            (recur::DUE, stored_due, due),
            (recur::EVERY, stored_every, every),
            (recur::WAKE, wake.clone(), wake.clone().unwrap_or_default()),
        ] {
            let was = cur.map(|t| extra_text(t, key)).unwrap_or_default();
            // Unchanged text (what the form was seeded with) is never a change,
            // even when resolving it would rewrite it (a `+02:00` datetime).
            if shown == was || (cur.is_none() && now.is_none()) {
                continue;
            }
            out.push((key, now));
        }
        out
    }

    /// The reason the schedule fields cannot be saved, in the CLI's words.
    pub(crate) fn problem(&self, today: chrono::NaiveDate) -> Option<String> {
        let typed =
            |cell: &RefCell<EditorState>| cell.borrow().lines.to_string().trim().to_string();
        let due = typed(&self.due);
        let bad_due = if due.is_empty() {
            Ok(String::new())
        } else {
            recur::resolve_due_short(&due, today)
        };
        if let Err(why) = bad_due {
            return Some(format!("due: {why}"));
        }
        let every = typed(&self.every);
        let bad_every = !every.is_empty()
            && recur::field_problem(recur::EVERY, &Value::Plain(every.clone())).is_some();
        bad_every.then(|| format!("every: {every:?} is not a duration like P1D, P2W or P1M"))
    }

    /// The labels field, normalized (commas/whitespace separate; yaks-7cb3).
    pub(crate) fn tags_vec(&self) -> Vec<String> {
        crate::model::normalize_tags([self.tags.borrow().lines.to_string()])
    }

    /// The description of a *create* (empty → no body); trailing whitespace is
    /// trimmed like `content::assemble` does.
    pub(crate) fn body_opt(&self) -> Option<String> {
        let text = self.description_text();
        let text = text.trim_end();
        if text.trim().is_empty() {
            None
        } else {
            Some(text.to_string())
        }
    }

    /// The raw description text.
    pub(crate) fn description_text(&self) -> String {
        self.description.borrow().lines.to_string()
    }

    /// An edit: the body to save when the description was changed, else `None`
    /// (so a header-only edit never rewrites the body). The comments are the
    /// seed body's, byte for byte ([`crate::store::with_description`]).
    pub(crate) fn changed_body(&self) -> Option<String> {
        let text = self.description_text();
        let text = text.trim_end();
        (text != seed_description(&self.seed_body))
            .then(|| crate::store::with_description(&self.seed_body, text))
    }
}

/// The description an edit form is seeded with: the body before its first
/// comment, trailing whitespace trimmed (leading indentation is kept).
fn seed_description(body: &str) -> String {
    super::content::parse(body).swap_remove(0).text
}

/// A yak's extra field as text (`due`, `every`, `wake`); empty when absent.
fn extra_text(t: &Task, key: &str) -> String {
    t.extra
        .get(key)
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_default()
}

pub(crate) fn render_create(f: &CreateForm, frame: &mut Frame, area: Rect, hits: &mut HitMap) {
    let has_herd = f.has_herd_row();
    let mut constraints = vec![
        Constraint::Length(1), // header
        Constraint::Length(1), // title
        Constraint::Length(1), // type chips
        Constraint::Length(1), // priority chips
        Constraint::Length(1), // labels
        Constraint::Length(1), // source
        Constraint::Length(1), // due
        Constraint::Length(1), // every
        Constraint::Length(1), // wake chips
    ];
    if has_herd {
        constraints.push(Constraint::Length(1)); // herd chips
    }
    constraints.push(Constraint::Min(0)); // description
    let rects = Layout::vertical(constraints).split(area);
    let header_r = rects[0];
    let title_r = rects[1];
    let type_r = rects[2];
    let pri_r = rects[3];
    let labels_r = rects[4];
    let source_r = rects[5];
    let due_r = rects[6];
    let every_r = rects[7];
    let wake_r = rects[8];
    let (herd_r, body_r) = if has_herd {
        (Some(rects[9]), rects[10])
    } else {
        (None, rects[9])
    };
    let header = match (&f.edit_id, &f.parent) {
        (Some(id), _) => format!("Edit {id}"),
        (None, Some(p)) => format!("New task (child of {p})"),
        (None, None) => "New yak".to_string(),
    };
    let mut head = vec![Span::styled(
        header,
        Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )];
    // A form restored from a draft says so; yellow when the yak has changed
    // since the draft began (yaks-683f).
    if let Some(d) = &f.draft {
        let (text, color) = if d.stale {
            (
                format!("  draft ({}, yak changed since) · Ctrl-R original", d.saved),
                Color::Yellow,
            )
        } else {
            (
                format!("  draft ({}) · Ctrl-R original", d.saved),
                Color::Magenta,
            )
        };
        head.push(Span::styled(text, Style::new().fg(color)));
    }
    frame.render_widget(Paragraph::new(Line::from(head)), header_r);
    let types: Vec<(String, bool)> = TYPE_CHOICES
        .iter()
        .enumerate()
        .map(|(i, &k)| (k.to_string(), i == f.kind_idx))
        .collect();
    let pris: Vec<(String, bool)> = PRI_CHOICES
        .iter()
        .enumerate()
        .map(|(i, &p)| (format!("p{p}"), i == f.pri_idx))
        .collect();
    render_text_row(
        f.row == 0,
        CREATE_LABEL_W,
        "title",
        &f.title,
        "",
        frame,
        title_r,
    );
    let chips = render_chip_row(
        f.row == 1,
        f.kind_idx,
        CREATE_LABEL_W,
        "type",
        &types,
        frame,
        type_r,
    );
    hits.form_chips.extend(row_chips(1, chips));
    let chips = render_chip_row(
        f.row == 2,
        f.pri_idx,
        CREATE_LABEL_W,
        "priority",
        &pris,
        frame,
        pri_r,
    );
    hits.form_chips.extend(row_chips(2, chips));
    render_text_row(
        f.row == 3,
        CREATE_LABEL_W,
        "tags",
        &f.tags,
        "",
        frame,
        labels_r,
    );
    render_text_row(
        f.row == 4,
        CREATE_LABEL_W,
        "source",
        &f.source,
        "",
        frame,
        source_r,
    );
    render_text_row(
        f.row == 5,
        CREATE_LABEL_W,
        "due",
        &f.due,
        "+2d, +1w or 2026-11-15",
        frame,
        due_r,
    );
    render_text_row(
        f.row == 6,
        CREATE_LABEL_W,
        "every",
        &f.every,
        "P1W, P1D, P1M: makes it recur",
        frame,
        every_r,
    );
    let wakes: Vec<(String, bool)> = WAKE_CHOICES
        .iter()
        .enumerate()
        .map(|(i, w)| (w.to_string(), i == f.wake_idx))
        .collect();
    let chips = render_chip_row(
        f.row == 7,
        f.wake_idx,
        CREATE_LABEL_W,
        "wakes",
        &wakes,
        frame,
        wake_r,
    );
    hits.form_chips.extend(row_chips(7, chips));
    if let Some(hr) = herd_r {
        let herds: Vec<(String, bool)> = f
            .herds
            .iter()
            .enumerate()
            .map(|(i, h)| (h.clone(), i == f.herd_idx))
            .collect();
        let chips = render_chip_row(
            f.row == HEADER_ROWS,
            f.herd_idx,
            CREATE_LABEL_W,
            "herd",
            &herds,
            frame,
            hr,
        );
        hits.form_chips.extend(row_chips(HEADER_ROWS, chips));
    }
    render_description(&f.description, f.is_content_row(), frame, body_r);
    // Every row's rect, for click-to-focus (the description's is its separator
    // and editor together).
    let mut rows = vec![
        (title_r, 0),
        (type_r, 1),
        (pri_r, 2),
        (labels_r, 3),
        (source_r, 4),
        (due_r, 5),
        (every_r, 6),
        (wake_r, 7),
    ];
    rows.extend(herd_r.map(|r| (r, HEADER_ROWS)));
    rows.push((body_r, f.header_rows()));
    hits.form_rows = rows;
}

/// A chip row's chip rects tagged with the form row and the chip's index.
fn row_chips(row: usize, chips: Vec<Rect>) -> impl Iterator<Item = (Rect, usize, usize)> {
    chips.into_iter().enumerate().map(move |(i, r)| (r, row, i))
}

/// The form's description: a `description` separator, then a live
/// editor while focused, else a dimmed wrapped preview of what is typed so far.
fn render_description(desc: &RefCell<EditorState>, focused: bool, frame: &mut Frame, area: Rect) {
    let [sep, body] = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(area);
    let head = format!("{}description ", if focused { "\u{25b8} " } else { "  " });
    let dashes = (sep.width as usize).saturating_sub(disp_width(&head));
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(head, palette::comment_header(focused)),
            Span::styled(
                palette::RULE_CHAR.to_string().repeat(dashes),
                palette::rule(),
            ),
        ])),
        sep,
    );
    if focused {
        let mut st = desc.borrow_mut();
        let mode = st.mode;
        set_md_highlights(&mut st);
        frame.render_widget(EditorView::new(&mut st).theme(editor_theme(mode)), body);
    } else {
        let text = desc.borrow().lines.to_string();
        let shown = if text.trim().is_empty() {
            "(no description)".to_string()
        } else {
            text
        };
        frame.render_widget(
            Paragraph::new(shown)
                .style(Style::new().fg(Color::DarkGray))
                .wrap(Wrap { trim: false }),
            body,
        );
    }
}

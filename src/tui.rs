//! Interactive terminal UI (`yaks tui`) — a thin layer over the core.
//!
//! All drawing goes through the pure `render(&App, &mut Frame)`, so the same
//! painter can target a real terminal (crossterm) or an in-memory `TestBackend`
//! buffer (snapshot tests + the future demo-cast pipeline). Key handling only
//! mutates `App`; mutating keys route through the `Farm` facade and then reload.

mod actions;
mod blockedit;
mod cache;
mod content;
mod create;
mod detail;
mod detail_nav;
#[cfg(test)]
mod docshots;
mod doks_tab;
mod drawer;
mod editor;
mod farmsync;
mod follow;
mod fuzzy;
mod handlers;
mod headless;
mod history;
mod imagesize;
mod imageview;
mod keymap;
mod linenav;
mod markdown;
mod mdrender;
mod memo;
mod mouse;
mod palette;
mod render;
mod runtime;
mod sheds_tab;
mod source;
mod surface;
mod syntax;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
mod tree;
mod view;
mod viewmodel;
mod views_store;
mod when;

// `headless` holds the `toque::HeadlessApp` impl for `App`; nothing to re-export
// (the headless driver lives in the `toque` crate, invoked from `main`).

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::io::{self, Stdout};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

use anyhow::Result;

use edtui::{EditorMode, EditorState, Lines};
use notify::event::{EventKind, ModifyKind};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::{
    self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyCode, KeyEvent, KeyEventKind,
    KeyModifiers, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
    PushKeyboardEnhancementFlags,
};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
    supports_keyboard_enhancement,
};

use crate::farm::{
    AttachOutcome, CreateOutcome, DepOutcome, Farm, MoveOutcome, NewTask, RenameOutcome, Reparent,
    TaskEdit, UpdateOutcome,
};
use crate::filter::{self, FilterSpec};
use crate::model::{Status, Task};

use blockedit::BlockEdit;
use create::*;
use doks_tab::*;
use drawer::*;
use editor::*;
use farmsync::{AutoSync, FarmOp, FarmSync};
use fuzzy::*;
#[cfg(test)]
use handlers::{handle_event, handle_paste};
use handlers::{handle_key, pump_events};
use keymap::{Cmd, Keymap, help_content};
use render::*;
pub use runtime::run;
use sheds_tab::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Focus {
    List,
    Detail,
}

/// A modal prompt painted on the bottom line: `pick()` (single keypress) and
/// `confirm()` (y/N) dialogs. Kept as plain
/// data so `render` stays pure — the action to perform on commit rides along.
pub(crate) enum Overlay {
    None,
    /// Single-key picker: any char in `keys` resolves `action`; Esc cancels.
    Pick {
        prompt: String,
        keys: String,
        action: PickAction,
    },
    /// y/N confirmation; Enter and Esc both default to "no".
    Confirm {
        prompt: String,
        action: ConfirmAction,
    },
    /// An in-frame edtui editor (single-line field or multi-line body).
    Edit(Editor),
    /// One content block (the description, a comment, or a new comment)
    /// edited in place in the detail pane (yaks-597d).
    BlockEdit(BlockEdit),
    /// A filter-as-you-type task picker (dependencies, reparent).
    Fuzzy(FuzzyPick),
    /// Inline incremental search: edits `App.query` live.
    Search(SearchBox),
    /// The multi-row filter drawer (chips + text facets).
    Drawer(Drawer),
    /// The create-task form (title/type/priority/labels/description).
    Create(CreateForm),
    /// The pane's find (`/` in a yak's detail or a text viewer): edits
    /// `App.register` live, for whichever text subject the pane shows.
    Find(SearchBox),
    /// The view manager (`v`): carries the picker's selection index.
    ViewPicker(usize),
    /// The keys of the current context in a compact popup (`?`); a second `?`
    /// opens the full reference. The context is the surface's own, so it
    /// carries nothing.
    Keys,
    /// The keyboard-help reference (`??`): carries the scroll offset.
    Help(u16),
}

/// A document open in the right pane (yaks-0906): the pane's subject while it
/// is `Some`, else the selected row's. Not modal: the list stays on the left
/// and the global keys (`U`, `W`, `?`, `#`) work with a dok open.
pub(crate) struct DocPane {
    pub(crate) view: DocView,
    /// Where focus goes back to when the document closes: the list, or the yak
    /// detail it was opened from. Fixed when the first document opens; a link
    /// followed from one document to the next keeps it.
    pub(crate) from: Focus,
}

/// What a [`DocPane`] shows.
pub(crate) enum DocView {
    /// A text attachment, dok or source in the viewer (yaks-699b).
    Text(handlers::AttachView),
    /// An image attachment in the image viewer (yaks-4305).
    Image(imageview::ImageView),
}

/// What a resolved single-key pick should do (carries the target task id).
pub(crate) enum PickAction {
    State(String),
    Priority(String),
    Type(String),
    /// Apply a state transition to every id in the multi-select set (yaks-de85).
    BulkState(Vec<String>),
    /// Move a yak to another herd (id prefix): `(id, candidate herds)`. The
    /// picked digit indexes into the candidates; the rename keeps the id tail
    /// (yaks-71d1).
    Herd(String, Vec<String>),
}

/// What a confirmed y/N prompt should do.
pub(crate) enum ConfirmAction {
    Slaughter(String),
    /// Slaughter a yak and every open descendant (yaks-05da, yaks-27c1).
    SlaughterFamily(String),
    /// Discard a dirty edit/create form or comment (stashed in `App.dirty_cancel`).
    DiscardEdit,
}

// Filter-drawer layout: 7 rows, three of them free-text.
const DRAWER_ROWS: usize = 7;
const STATUS_CHOICES: [Status; 4] = [Status::Hairy, Status::Shaving, Status::Shorn, Status::Dead];
const TYPE_CHOICES: [&str; 4] = ["task", "bug", "feature", "idea"];
const PRI_CHOICES: [u8; 5] = [1, 2, 3, 4, 5];
const DEPS_CHOICES: [&str; 3] = ["ready", "tangled", "inbox"];

fn text_field(seed: &str, vim: bool) -> RefCell<EditorState> {
    let mut st = EditorState::new(Lines::from(seed));
    st.set_single_line(true);
    st.mode = EditorMode::Insert;
    let _ = vim;
    RefCell::new(st)
}

/// A multi-line edtui field (content zone for descriptions/comments). In vim,
/// a field seeded with existing content opens in Normal mode (the common vi
/// expectation when reviewing text); an empty one opens in Insert to type
/// straight away.
fn multiline_field(seed: &str, vim: bool) -> RefCell<EditorState> {
    let mut st = EditorState::new(Lines::from(seed));
    st.set_single_line(false);
    st.set_markdown_continuation(true);
    st.mode = if vim && !seed.is_empty() {
        EditorMode::Normal
    } else {
        EditorMode::Insert
    };
    RefCell::new(st)
}

fn toggle<T: PartialEq>(v: &mut Vec<T>, val: T) {
    if let Some(i) = v.iter().position(|x| *x == val) {
        v.remove(i);
    } else {
        v.push(val);
    }
}

/// Max gap between two `Esc` presses for them to count as a double-tap cancel.
const DOUBLE_ESC_MS: u64 = 300;

// Task-form fixed rows: title, type, priority, labels, source, due, every, wake.
// Content blocks (a description, plus one per comment when editing) follow as rows
// `HEADER_ROWS + block_index`, so the total row count is dynamic
// (`CreateForm::row_count`).
const HEADER_ROWS: usize = 8;
/// The create form's `wake` chips: none (the yak only reopens), then who a
/// regrown recurring yak goes to (`wake: agent|human`).
const WAKE_CHOICES: [&str; 3] = ["none", "agent", "human"];

fn kind_index(kind: &str) -> usize {
    TYPE_CHOICES.iter().position(|&k| k == kind).unwrap_or(0)
}

fn pri_index(p: u8) -> usize {
    PRI_CHOICES.iter().position(|&x| x == p).unwrap_or(2)
}

/// Sort key for a flat view field (ISO timestamps sort lexically; priority is
/// zero-padded so it orders numerically as a string).
fn sort_key(t: &Task, f: view::SortField) -> String {
    match f {
        view::SortField::Priority => format!("{:03}", t.priority),
        view::SortField::Title => t.title.to_lowercase(),
        view::SortField::Updated => t.updated.clone().unwrap_or_default(),
        view::SortField::Created => t.created.clone().unwrap_or_default(),
        view::SortField::Id => t.id.clone(),
    }
}

fn same_set<T: PartialEq>(a: &[T], b: &[T]) -> bool {
    a.len() == b.len() && a.iter().all(|x| b.contains(x)) && b.iter().all(|x| a.contains(x))
}

/// Order-insensitive FilterSpec equality (for the view-modified marker).
fn spec_eq(a: &FilterSpec, b: &FilterSpec) -> bool {
    same_set(&a.statuses, &b.statuses)
        && same_set(&a.types, &b.types)
        && same_set(&a.priorities, &b.priorities)
        && same_set(&a.tags, &b.tags)
        && a.search == b.search
        && a.ready_only == b.ready_only
        && a.tangled_only == b.tangled_only
        && a.parent == b.parent
}

/// Snapshot a spec (thin alias for `.clone()`; kept for call-site clarity).
fn clone_spec(f: &FilterSpec) -> FilterSpec {
    f.clone()
}

/// TUI state. Holds the loaded task set plus (in live use) a `Farm` handle so
/// mutations re-query through the core. Per-tab tree views are derived on demand.
pub struct App {
    /// `None` in read-only snapshot tests; `Some` in live use (`with_farm`).
    farm: Option<Farm>,
    all: Vec<Task>,
    /// Ordered views; pinned ones form the tab strip. Replaces fixed tabs.
    views: Vec<view::View>,
    /// Index into `views` of the active view.
    view: usize,
    /// Ordered starred ids backing the built-in Starred (working-set) view.
    working_set: Vec<String>,
    cursor: usize,
    focus: Focus,
    /// The detail pane's navigation state, the same [`linenav::LineNav`] the
    /// text viewer owns: line cursor (j/k move it, Tab snaps it to link
    /// lines, Enter follows a link on it), scroll, visual selection, find
    /// match (n/N cycle) and the sideways scroll of the no-wrap block under the
    /// cursor (dropped when the cursor leaves it or the yak changes,
    /// `App::drop_stale_hscroll`, yaks-c733).
    detail: linenav::LineNav,
    /// The yak the pane shows when it is not the selected row's: a link
    /// followed to a yak the current list does not have (another state, a
    /// facet or the query keeps it out; the Doks or Sheds view has no such
    /// rows). `None`: the pane shows the selected row's yak. It means
    /// something only with the pane focused ([`App::pane_yak_id`]), and goes
    /// when the cursor moves, the view changes or focus returns to the list.
    pane_yak: Option<String>,
    /// The one history of locations (`o` back, `i` forward): see
    /// [`history`].
    history: history::History,
    collapsed: HashSet<String>,
    /// Per-view family-scope overrides, keyed by `View::key` (persisted in the
    /// UI-state cache). A missing entry means the view inherits the global
    /// default ("auto"). See [`view::FamilyScope`].
    family_scope: HashMap<String, view::FamilyScope>,
    /// The live view filter applied by the tree (re-colors + prunes). The inbox
    /// (yaks awaiting a human) is just `FilterSpec::needs_only` on this filter
    /// — surfaced via the Inbox view and the drawer's `inbox` chip, not a mode.
    filter: FilterSpec,
    /// The global query `/` edits (design §4): what every view's rows and
    /// counts also match ([`tree::query_matches`]), kept across view switches
    /// and apart from the view's facets, so it never marks a tab modified.
    /// `Esc` on a list clears it before it reverts the facets. Never `Some("")`.
    query: Option<String>,
    /// The search register (vim's `@/`, design §4): what `/` in the pane
    /// edits, highlighted in every text subject (a yak's detail, a dok, a
    /// source, an attachment) and stepped by `n`/`N`. It outlives the subject
    /// (`J`/`K`, `o`/`i` and following a link keep it) and is apart from the
    /// list's `query`, so searching inside a yak never filters the list. Never
    /// `Some("")`.
    register: Option<String>,
    /// Approx. list viewport height, refreshed each loop for paging math.
    page: u16,
    /// Approx. detail viewport height (mid area = terminal height - 3),
    /// refreshed each loop; used to keep the active link scrolled into view.
    detail_page: u16,
    /// Detail-pane content width captured at render, used to soft-wrap the
    /// detail lines so the row-indexed model matches what's on screen.
    detail_width: Cell<u16>,
    /// "Now" and the display timezone for human-friendly timestamps (pinned
    /// under `cargo test`; see [`when::Clock::app_default`]).
    clock: when::Clock,
    /// The list/tree viewport's scroll offset (first visible row), carried
    /// across frames so moving the cursor scrolls only as far as needed to keep
    /// it visible, rather than re-deriving it from 0 each frame (which pinned
    /// the selection to the bottom edge when scrolling up — yaks-9009).
    list_offset: Cell<usize>,
    overlay: Overlay,
    /// The document open in the right pane, if any (see [`DocPane`]).
    doc: Option<DocPane>,
    /// How this terminal draws images (kitty/sixel/iTerm2, else half-block
    /// cells). Halfblocks until [`runtime::run`] queries the real terminal, so
    /// headless frames and tests draw an image as colored cells.
    picker: ratatui_image::picker::Picker,
    /// Draw the images of a top-level body paragraph in place
    /// (`inline_images` in the farm config, default on; yaks-0bf0, yaks-0370).
    inline_images: bool,
    /// Decoded inline images, filled while drawing (see [`imageview`]).
    inline_cache: RefCell<imageview::InlineCache>,
    /// Transient one-line status message shown until the next mutation.
    notification: Option<String>,
    /// An editor the event loop should run with the terminal suspended (set
    /// by following a source link; see [`source`]).
    pending_editor: Option<source::ExternalCmd>,
    /// Editor keybinding profile (vim vs emacs), from farm config.
    editor_vim: bool,
    /// The farm's configured id prefix (e.g. `yaks`), for autocomplete: typing
    /// `<ref_prefix>-` while editing opens the yak-ref picker.
    ref_prefix: String,
    /// The farm's declared herd set (`Config::known_herds`); unioned with the
    /// prefixes actually present to form the create-form herd picker's choices.
    config_herds: Vec<String>,
    /// Timestamp of the last `Esc` in an editor overlay, for detecting a rapid
    /// double-`Esc` (a Ctrl-C-equivalent cancel gesture). See [`App::register_double_esc`].
    last_esc: Option<std::time::Instant>,
    /// A dirty edit/create/comment overlay stashed behind a "discard changes?"
    /// confirmation; restored if the user declines. See [`App::request_cancel`].
    dirty_cancel: Option<Overlay>,
    /// The in-progress `:` command line (vim `:w`/`:q`/...), layered over the
    /// active editor overlay while `Some`. See [`App::handle_cmdline_key`].
    cmdline: Option<String>,
    /// Ids marked for multi-select bulk actions (`m` toggles the cursor's yak).
    /// A bulk state transition (`S` with a non-empty set) loops these ids
    /// through `farm.transition`, then clears the set. See [`App::toggle_selected`].
    selected: HashSet<String>,
    /// Pane/tab rects recorded by the last render, for mouse hit-testing
    /// (reset each frame). See [`mouse::HitMap`].
    hits: RefCell<mouse::HitMap>,
    /// The Sheds tab's snapshot, cursors and background scan (yaks-98f1).
    sheds: ShedsTab,
    /// The Doks tab's doks and sources and its cursor (yaks-ebb2).
    doks: DoksTab,
    /// Peeks read for the preview pane, by link (see [`follow::PeekCache`]).
    peeks: RefCell<follow::PeekCache>,
    /// Whether the viewer shows its peek pane (`p` toggles).
    peek_shown: bool,
    /// The farm's uncommitted count and sync standing, for the tab-row chip
    /// (`None`: no farm, a private farm, or git could not say). Re-read on
    /// every reload; see [`farmsync`].
    farm_sync: Option<FarmSync>,
    /// A `W`/`U` commit or sync waiting for the frame that announces it.
    pending_farm_op: Option<FarmOp>,
    /// The interval sync (`sync_interval`, yaks-f0db).
    auto_sync: AutoSync,
    quit: bool,
    /// The keys in effect: the table's, plus the user's overrides.
    keymap: Keymap,
    /// Bumped by [`App::touch`] whenever `all` or the sheds snapshot is
    /// replaced; the memoized rows and detail lines are keyed on it.
    data_gen: u64,
    rows_cache: RefCell<Option<memo::RowsCache>>,
    dlines_cache: RefCell<Option<memo::DlinesCache>>,
}

impl App {
    /// Read-only constructor: renders `all` with no farm behind it. Mutating
    /// keys become no-ops. Used by snapshot tests and any preview caller.
    pub fn new(all: Vec<Task>) -> Self {
        let views = view::default_views();
        let filter = clone_spec(&views[0].spec);
        App {
            farm: None,
            all,
            views,
            view: 0,
            working_set: Vec::new(),
            cursor: 0,
            focus: Focus::List,
            detail: linenav::LineNav::default(),
            pane_yak: None,
            history: history::History::default(),
            collapsed: HashSet::new(),
            family_scope: HashMap::new(),
            filter,
            query: None,
            register: None,
            page: 10,
            detail_page: 10,
            detail_width: Cell::new(0),
            clock: when::Clock::app_default(),
            list_offset: Cell::new(0),
            overlay: Overlay::None,
            doc: None,
            picker: ratatui_image::picker::Picker::halfblocks(),
            inline_images: true,
            inline_cache: RefCell::default(),
            notification: None,
            pending_editor: None,
            editor_vim: true,
            ref_prefix: "yak".to_string(),
            config_herds: Vec::new(),
            last_esc: None,
            dirty_cancel: None,
            cmdline: None,
            selected: HashSet::new(),
            hits: RefCell::default(),
            sheds: ShedsTab::default(),
            doks: DoksTab::default(),
            peeks: RefCell::default(),
            peek_shown: true,
            farm_sync: None,
            pending_farm_op: None,
            auto_sync: AutoSync::off(),
            quit: false,
            keymap: Keymap::default(),
            data_gen: 0,
            rows_cache: RefCell::new(None),
            dlines_cache: RefCell::new(None),
        }
    }

    /// Live constructor: loads the current farm view and keeps the handle so
    /// mutations can re-query after each change.
    pub fn with_farm(farm: Farm) -> Result<Self> {
        // Load every status incl. dead. The tree/flat views scope down to what
        // each shows, but keeping dead in the model lets the ancestor walk root
        // a live yak beneath a slaughtered parent, lets a Dead filter surface
        // slaughtered yaks, and treats a dep on a dead yak as resolved (fe00).
        // Recurring yaks whose time has come are hairy again before the first
        // frame (yaks-2fc5).
        let regrew = farm
            .regrow_due(
                chrono::Utc::now(),
                false,
                crate::actor::resolve(None).as_deref(),
            )
            .unwrap_or_default();
        let all = farm.list(FilterSpec::default(), true)?;
        let cfg = farm.config();
        let vim = cfg.vim_mode;
        let ui = cache::load(farm.root());
        let views = views_store::load_views(farm.root());
        let working_set = views_store::load_working_set(farm.root());
        let mut app = App::new(all);
        app.editor_vim = vim;
        app.inline_images = cfg.inline_images;
        app.config_herds = cfg.known_herds();
        app.ref_prefix = cfg.prefix;
        app.collapsed = ui.collapsed;
        app.family_scope = ui.family;
        app.filter = clone_spec(&views[0].spec);
        app.views = views;
        app.working_set = working_set;
        app.farm = Some(farm);
        if !regrew.regrew.is_empty() {
            app.notification = Some(farmsync::regrew_notice(&regrew));
        }
        if let Some(msg) = app
            .farm
            .as_ref()
            .and_then(|f| crate::store::min_version_notice(f.root()))
        {
            app.notification = Some(msg);
        }
        app.auto_sync = AutoSync::new(
            cfg.sync_interval,
            farmsync::autosync_disabled_by_env(),
            std::time::Instant::now(),
        );
        app.refresh_farm_sync();
        app.reload_doks();
        app.clamp_cursor();
        // The user's key overrides (`keys:` in their config.yaml). Read here,
        // not in `new`, so a snapshot test never depends on the machine.
        let (keymap, notice) = Keymap::load_user();
        app.keymap = keymap;
        if let Some(notice) = notice {
            app.notification = Some(notice);
        }
        // One scan at startup feeds the Sheds tab's count; after that it
        // scans only while shown (yaks-98f1).
        app.sheds.cwd = std::env::current_dir().ok();
        app.sheds.scan = true;
        app.request_sheds_scan();
        Ok(app)
    }
}

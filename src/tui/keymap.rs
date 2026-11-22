//! The command table (yaks-fbb0, phase 2 of /doks/design/tui-model.md#5-keys):
//! every key binding of the non-modal surfaces in one place. A [`Binding`]
//! names a [`Cmd`] (a stable dotted name), the [`Ctx`] it applies in, its
//! keys, a help line for the `?` overlay and a label for the bottom hint.
//!
//! Dispatch is "work out the context, [`Keymap::lookup`], [`App::run_cmd`]".
//! The `?` popup ([`Keymap::popup_entries`]), the full `?` reference
//! ([`help_content`]) and the bottom bar ([`help_hint`]) are generated from
//! the same table, so a key cannot be bound without being listed, and the
//! four cannot drift. The [`Keymap`] is the table with the user's `keys:`
//! overrides applied (`overrides`); every read of a key goes through it.
//!
//! The mechanism (keys and their labels, the families, the lookup, overrides,
//! the popup and reference layout, and the rule checks the tests call) is
//! `toque::keys`, shared with canon's TUI (yaks-2e7c); this module is yaks'
//! own commands, contexts, table, sections and extras.
//!
//! A [`Cmd`] is one intent, not one handler: `move.down` moves whichever
//! surface has focus, and `run_cmd` picks the target from the context. A key
//! binds by code, and by the modifiers it names (all of them must be held);
//! a key with extra modifiers still matches its plain binding, as the
//! hand-written `match`es did, and a binding that names modifiers is tried
//! first (so Ctrl-C is quit, not `c`).
//!
//! In the list and the detail pane a binding may name the [`RowKind`] it is
//! for (yaks-be65, yaks-c354): the lookup tries the selected row's kind first,
//! then a binding for any kind. A yak command is bound for yak rows only, so on
//! a document or shed row it finds nothing and does nothing, as on an empty
//! list; with no row selected the view's own kind applies (an empty Doks still
//! takes `c` and `Ctrl-R`, an empty Sheds view `Ctrl-R`).
//!
//! An open document (the text and image viewers) is part of the right pane,
//! not an overlay (yaks-0906): its keys are the `Viewer` and `Image` contexts,
//! and the global ones (`U`, `W`, `?`, `#`) are bound there too.
//!
//! One key family for the whole TUI (yaks-7c6e): [`Family::Vim`] or
//! [`Family::Standard`], chosen once in the user's config. A key that belongs
//! to only one family (toque's `VIM_ONLY`, `STANDARD_ONLY`) is off in the other:
//! the table lists every key, and [`Keymap::keys`] filters them, so the
//! lookup, the `?` popup, the reference and the hint all agree. The keys both
//! families share (arrows, `PgUp`/`PgDn`, `Home`/`End`, `Enter`, `Esc`, the
//! Shift tier) work in both; a test keeps every command reachable in each.
//!
//! The key rules, with the tests that enforce them (also in `AGENTS.md`):
//!
//! - **Case is a tier.** Lowercase looks and moves; Shift changes an existing
//!   yak; Ctrl is the app and the farm. `Cmd::writes` says which commands
//!   write, and `no_lowercase_key_changes_an_existing_yak` fails on one that
//!   sits on a lowercase key (bar the listed exceptions).
//! - **One key, one intent, in both families.** `every_command_stays_bound_in_standard_mode`.
//! - **Enter goes in, Esc comes out.** `h`/`l` never move between the list and
//!   the pane: in the pane they only scroll a wide block.
//!
//! Modal overlays (editors, pickers, the drawer, create, help, the command
//! line, the search boxes) and the mouse are not in the table. The editors'
//! own keys are edtui's, listed by hand for the `F1` popup ([`editor_groups`]).

use super::*;
use ratatui::text::Line;

// The mechanism is toque's (yaks-2e7c): keys, families, the lookup, overrides
// and the help layout. This module is yaks' commands, contexts and table.
pub(crate) use toque::keys::{Family, Key, PopupGroups, popup_lines};
use toque::keys::{c, ctrl, ctrl_k, k, shift};

/// What a binding adds to the bottom hint bar (toque's [`toque::keys::Hint`]).
pub(crate) type Hint = toque::keys::Hint;
/// One row of the table.
pub(crate) type Binding = toque::keys::Binding<Cmd>;
/// The keys in effect: the table, filtered by the key family, with the user's
/// `keys:` overrides applied.
pub(crate) type Keymap = toque::keys::Keymap<Cmd>;
type Section = toque::keys::Section<Cmd>;

/// The filter drawer's hint, which depends on whether a filter is active
/// (`f:filter  /:search`, or `f:filter  Esc:clear`).
const FILTER_HINT: Hint = Hint::Custom("filter");

/// What the `?` popup says about the keys it had no room for.
pub(crate) const POPUP_MORE: &str = "the full reference has them all";

mod editor;
mod overrides;

pub(crate) use editor::{EditorAt, editor_groups, editor_lists, editor_title, rows_groups};
pub(crate) use overrides::load_user;

/// Declares [`Cmd`] with each variant's dotted name, and keeps `ALL` and
/// `name` in step with it (a new variant cannot be left out of either).
macro_rules! commands {
    ($($variant:ident => $name:literal,)*) => {
        /// One action a key can run.
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub(crate) enum Cmd {
            $($variant,)*
        }

        impl Cmd {
            /// Every command.
            const LIST: &'static [Cmd] = &[$(Cmd::$variant,)*];

            /// The stable dotted name: what a `keys:` override in the user
            /// config names, and what the `?` popup prints.
            fn dotted(self) -> &'static str {
                match self {
                    $(Cmd::$variant => $name,)*
                }
            }
        }
    };
}

commands! {
    AppQuit => "app.quit",
    AppHelp => "app.help",
    MoveDown => "move.down",
    MoveUp => "move.up",
    MoveTop => "move.top",
    MoveBottom => "move.bottom",
    MoveHalfDown => "move.half_down",
    MoveHalfUp => "move.half_up",
    MovePageDown => "move.page_down",
    MovePageUp => "move.page_up",
    ItemNext => "item.next",
    ItemPrev => "item.prev",
    ViewNext => "view.next",
    ViewPrev => "view.prev",
    ViewPick => "view.pick",
    ViewSave => "view.save",
    ViewRevert => "view.revert",
    FamilyCycle => "family.cycle",
    TreeToggle => "tree.toggle",
    TreeLeft => "tree.left",
    TreeRight => "tree.right",
    PaneOpen => "pane.open",
    PaneEnter => "pane.enter",
    PanePeel => "pane.peel",
    PaneClose => "pane.close",
    PaneScrollLeft => "pane.scroll_left",
    PaneScrollRight => "pane.scroll_right",
    RowCopy => "row.copy",
    ListRefresh => "list.refresh",
    SearchOpen => "search.open",
    FilterDrawer => "filter.drawer",
    NavGoto => "nav.goto",
    NavBack => "nav.back",
    NavForward => "nav.forward",
    LinkNext => "link.next",
    LinkPrev => "link.prev",
    TakeOut => "take.out",
    ImageZoomIn => "image.zoom_in",
    ImageZoomOut => "image.zoom_out",
    ImageZoomActual => "image.zoom_actual",
    ImageFitWidth => "image.fit_width",
    ImageFit => "image.fit",
    ImagePanLeft => "image.pan_left",
    ImagePanRight => "image.pan_right",
    ImagePanUp => "image.pan_up",
    ImagePanDown => "image.pan_down",
    FindOpen => "find.open",
    FindNext => "find.next",
    FindPrev => "find.prev",
    SelectVisual => "select.visual",
    SelectExtendDown => "select.extend_down",
    SelectExtendUp => "select.extend_up",
    SelectYank => "select.yank",
    BlockNext => "block.next",
    BlockPrev => "block.prev",
    PeekToggle => "peek.toggle",
    DocEdit => "doc.edit",
    DocWiden => "doc.widen",
    FarmCommit => "farm.commit",
    FarmSync => "farm.sync",
    YakMark => "yak.mark",
    YakState => "yak.state",
    YakPriority => "yak.priority",
    YakType => "yak.type",
    YakTags => "yak.tags",
    YakSlaughter => "yak.slaughter",
    YakNew => "yak.new",
    YakNewChild => "yak.new_child",
    YakEdit => "yak.edit",
    YakDep => "yak.dep",
    YakReparent => "yak.reparent",
    YakHerd => "yak.herd",
    YakBackrefs => "yak.backrefs",
    YakStar => "yak.star",
    YakComment => "yak.comment",
    YakAttach => "yak.attach",
    YakAnswer => "yak.answer",
    YakAsk => "yak.ask",
    YakSnooze => "yak.snooze",
    AttachmentRename => "attachment.rename",
    DokNew => "dok.new",
    DokEdit => "dok.edit",
}

impl toque::keys::Command for Cmd {
    type Ctx = Ctx;
    type Kind = RowKind;
    const ALL: &'static [Cmd] = Cmd::LIST;
    const GROUPS: &'static [&'static str] = &[
        Group::Move.title(),
        Group::Open.title(),
        Group::Find.title(),
        Group::Change.title(),
        Group::View.title(),
        Group::Farm.title(),
        Group::App.title(),
    ];

    fn name(self) -> &'static str {
        self.dotted()
    }

    fn group_title(self) -> &'static str {
        self.group().title()
    }

    fn mutates(self) -> bool {
        self.writes()
    }

    fn bindings() -> &'static [Binding] {
        BINDINGS
    }
}

impl Cmd {
    /// The `?` popup's group for it. Exhaustive, so a new command must pick one.
    pub(crate) fn group(self) -> Group {
        match self {
            MoveDown | MoveUp | MoveTop | MoveBottom | MoveHalfDown | MoveHalfUp | MovePageDown
            | MovePageUp | ItemNext | ItemPrev | TreeToggle | TreeLeft | TreeRight
            | PaneScrollLeft | PaneScrollRight | LinkNext | LinkPrev | BlockNext | BlockPrev
            | DocWiden | ImagePanLeft | ImagePanRight | ImagePanUp | ImagePanDown
            | SelectVisual | SelectExtendDown | SelectExtendUp | SelectYank | RowCopy => {
                Group::Move
            }
            PaneOpen | PaneEnter | PanePeel | PaneClose | NavBack | NavForward | TakeOut
            | PeekToggle => Group::Open,
            SearchOpen | FindOpen | FindNext | FindPrev | NavGoto | YakBackrefs => Group::Find,
            YakMark | YakState | YakPriority | YakType | YakTags | YakSlaughter | YakNew
            | YakNewChild | YakEdit | YakDep | YakReparent | YakHerd | YakStar | YakComment
            | YakAttach | YakAnswer | YakAsk | YakSnooze | AttachmentRename | DokNew | DokEdit
            | DocEdit => Group::Change,
            ViewNext | ViewPrev | ViewPick | ViewSave | ViewRevert | FamilyCycle | FilterDrawer
            | ImageZoomIn | ImageZoomOut | ImageZoomActual | ImageFitWidth | ImageFit => {
                Group::View
            }
            FarmCommit | FarmSync | ListRefresh => Group::Farm,
            AppQuit | AppHelp => Group::App,
        }
    }

    /// Whether the command writes to the farm: it creates, marks or changes a
    /// yak, a dok or a view. The case rule (lowercase looks, Shift changes) is
    /// tested over these (only the test reads it). Exhaustive, so a new
    /// command must say.
    fn writes(self) -> bool {
        match self {
            YakMark | YakState | YakPriority | YakType | YakTags | YakSlaughter | YakNew
            | YakNewChild | YakEdit | YakDep | YakReparent | YakHerd | YakStar | YakComment
            | YakAttach | YakAnswer | YakAsk | YakSnooze | AttachmentRename | DokNew | DokEdit
            | DocEdit | ViewSave | FarmCommit | FarmSync => true,
            AppQuit | AppHelp | MoveDown | MoveUp | MoveTop | MoveBottom | MoveHalfDown
            | MoveHalfUp | MovePageDown | MovePageUp | ItemNext | ItemPrev | ViewNext
            | ViewPrev | ViewPick | ViewRevert | FamilyCycle | TreeToggle | TreeLeft
            | TreeRight | PaneOpen | PaneEnter | PanePeel | PaneClose | PaneScrollLeft
            | PaneScrollRight | RowCopy | ListRefresh | SearchOpen | FilterDrawer | NavGoto
            | NavBack | NavForward | LinkNext | LinkPrev | TakeOut | ImageZoomIn | ImageZoomOut
            | ImageZoomActual | ImageFitWidth | ImageFit | ImagePanLeft | ImagePanRight
            | ImagePanUp | ImagePanDown | FindOpen | FindNext | FindPrev | SelectVisual
            | SelectExtendDown | SelectExtendUp | SelectYank | BlockNext | BlockPrev
            | PeekToggle | YakBackrefs | DocWiden => false,
        }
    }
}

/// A group of the `?` popup: what a command is for.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Group {
    Move,
    Open,
    Find,
    Change,
    View,
    Farm,
    App,
}

impl Group {
    /// The popup's title, which is also its order in [`Command::GROUPS`].
    pub(crate) const fn title(self) -> &'static str {
        match self {
            Group::Move => "Move",
            Group::Open => "Open and follow",
            Group::Find => "Find",
            Group::Change => "Change the yak",
            Group::View => "View and filter",
            Group::Farm => "Farm",
            Group::App => "App",
        }
    }
}

/// A surface that takes keys when no modal overlay is up. These are today's
/// surfaces as they are; later phases fold them together.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Ctx {
    /// The yak list.
    List,
    /// The yak detail pane.
    Detail,
    /// The text viewer in the pane (attachments, doks, sources).
    Viewer,
    /// The image viewer in the pane.
    Image,
    /// The directory view in the pane (yaks-3329).
    Dir,
    /// The commit view in the pane (yaks-3f90).
    Commit,
}

impl toque::keys::Context for Ctx {
    const ALL: &'static [Ctx] = &[
        Ctx::List,
        Ctx::Detail,
        Ctx::Viewer,
        Ctx::Image,
        Ctx::Dir,
        Ctx::Commit,
    ];

    /// What a message calls the surface.
    fn title(self) -> &'static str {
        match self {
            Ctx::List => "list",
            Ctx::Detail => "detail pane",
            Ctx::Viewer => "text viewer",
            Ctx::Image => "image viewer",
            Ctx::Dir => "directory view",
            Ctx::Commit => "commit view",
        }
    }
}

/// The `?` overlay's sections, in order. Every binding is in exactly one.
const SECTIONS: [Section; 10] = [
    Section {
        ctx: Ctx::List,
        kinds: &[None, Some(RowKind::Yak)],
        title: "List pane",
    },
    Section {
        ctx: Ctx::Detail,
        kinds: &[None, Some(RowKind::Yak)],
        title: "Detail pane",
    },
    Section {
        ctx: Ctx::Viewer,
        kinds: &[None],
        title: "Text viewer (an attachment, dok or source)",
    },
    Section {
        ctx: Ctx::Image,
        kinds: &[None],
        title: "Image viewer",
    },
    Section {
        ctx: Ctx::Dir,
        kinds: &[None],
        title: "Directory view (a yak's attachments, a dok's files, a source's files/)",
    },
    Section {
        ctx: Ctx::Commit,
        kinds: &[None],
        title: "Commit view (a commit link, a note's commit: or source: hash)",
    },
    Section {
        ctx: Ctx::List,
        kinds: &[Some(RowKind::Doc)],
        title: "Document rows (the Doks view: the farm's doks and sources)",
    },
    Section {
        ctx: Ctx::List,
        kinds: &[Some(RowKind::Place)],
        title: "Starred places (the Starred view: a starred attachment, directory, code file or commit)",
    },
    Section {
        ctx: Ctx::List,
        kinds: &[Some(RowKind::Shed)],
        title: "Shed rows (the Sheds view: actors, with the yaks they touched below)",
    },
    Section {
        ctx: Ctx::Detail,
        kinds: &[Some(RowKind::Shed)],
        title: "Shed detail (a shed's pane; its yaks' panes are the yak detail)",
    },
];

const fn bind(ctx: Ctx, keys: &'static [Key], cmd: Cmd, help: &'static str, hint: Hint) -> Binding {
    toque::keys::bind(ctx, keys, cmd, help, hint)
}

/// A list binding for one kind of row.
const fn bind_row(
    kind: RowKind,
    keys: &'static [Key],
    cmd: Cmd,
    help: &'static str,
    hint: Hint,
) -> Binding {
    toque::keys::bind_kind(Ctx::List, kind, keys, cmd, help, hint)
}

/// A detail-pane binding for the pane of one kind of row (a yak's keys act on
/// its lines; a shed's pane has none of them).
const fn bind_pane(
    kind: RowKind,
    keys: &'static [Key],
    cmd: Cmd,
    help: &'static str,
    hint: Hint,
) -> Binding {
    toque::keys::bind_kind(Ctx::Detail, kind, keys, cmd, help, hint)
}

const fn hint(label: &'static str) -> Hint {
    Hint::Text(label)
}

/// A hint whose label names the family's keys.
const fn hint_family(vim: &'static str, standard: &'static str) -> Hint {
    Hint::Family(vim, standard)
}

use Cmd::*;
// Explicit, so it wins over the glob-imported `FarmSync` (the auto-sync state).
use Cmd::FarmSync;
use Ctx::*;
#[allow(non_upper_case_globals)]
const NoHint: Hint = Hint::None;
use KeyCode::{BackTab, Down, End, Enter, Esc, F, Home, Left, PageDown, PageUp, Right, Tab, Up};
use tree::RowKind::{self, Doc, Place, Shed, Yak};

/// The table. Within a context, the order is the order of the bottom hint
/// and of the `?` overlay, so the hinted bindings keep their hint's order.
#[rustfmt::skip]
static BINDINGS: &[Binding] = &[
    // -- the list: yak rows, and the keys that suit any row -------------------
    bind(List, &[k(Tab), c(']'), c('}')], ViewNext, "Next view", hint("Tab:view")),
    bind(List, &[k(BackTab), c('['), c('{')], ViewPrev, "Previous view", NoHint),
    bind(List, &[c('j'), k(Down)], MoveDown, "Move cursor down", hint_family("j/k:move", "↑/↓:move")),
    bind(List, &[c('k'), k(Up)], MoveUp, "Move cursor up", NoHint),
    bind(List, &[c('d'), ctrl('d')], MoveHalfDown, "Half-page down", NoHint),
    bind(List, &[c('u'), ctrl('u')], MoveHalfUp, "Half-page up", NoHint),
    bind(List, &[k(PageDown)], MovePageDown, "Full-page down", NoHint),
    bind(List, &[k(PageUp)], MovePageUp, "Full-page up", NoHint),
    bind(List, &[c('g'), k(Home)], MoveTop, "Top", NoHint),
    bind(List, &[c('G'), k(End)], MoveBottom, "Bottom", NoHint),
    bind_row(Yak, &[k(Enter)], PaneOpen, "Show detail pane", hint("Enter:detail")),
    bind_row(Yak, &[c('h'), k(Left)], TreeLeft, "Collapse the row, else go to its parent (tree views)", NoHint),
    bind_row(Yak, &[c('l'), k(Right)], TreeRight, "Expand the row, else go to its first child (tree views)", NoHint),
    bind_row(Yak, &[c('F')], FamilyCycle, "Cycle the family scope (tree views)", hint("F:family")),
    bind_row(Yak, &[c(' ')], TreeToggle, "Collapse / expand subtree", NoHint),
    bind_row(Yak, &[c('m')], YakMark, "Mark / unmark row (multi-select)", NoHint),
    bind_row(Yak, &[c('`')], ViewPick, "View picker", hint("`:views")),
    bind_row(Yak, &[c('V')], ViewSave, "Save filter as view", NoHint),
    bind(List, &[k(Esc)], ViewRevert, "Clear the search, then revert the filter to the active view", NoHint),
    bind_row(Yak, &[c('c')], YakNew, "New root yak", hint("c/C:new")),
    bind_row(Yak, &[c('C')], YakNewChild, "New child yak", NoHint),
    bind_row(Yak, &[c('E')], YakEdit, "Edit: the form (header fields and description)", hint("E:edit")),
    bind_row(Yak, &[c('X')], YakSlaughter, "Slaughter (confirm; offers whole family)", hint("X:del")),
    bind_row(Yak, &[c('S')], YakState, "State (bulk over the marked set when rows are marked)", hint("S:state")),
    bind_row(Yak, &[c('P')], YakPriority, "Priority", NoHint),
    bind_row(Yak, &[c('Y')], YakType, "Type", NoHint),
    bind_row(Yak, &[c('T')], YakTags, "Tags", NoHint),
    bind_row(Yak, &[c('D')], YakDep, "Add dependency", hint("D:dep")),
    bind_row(Yak, &[c('R')], YakReparent, "Reparent", NoHint),
    bind_row(Yak, &[c('H')], YakHerd, "Move to another herd (multi-herd farm)", NoHint),
    bind_row(Yak, &[c('M')], YakComment, "Add a comment (in place, after the last one)", NoHint),
    bind_row(Yak, &[c('a')], YakAnswer, "Answer (awaiting a human) / request agent work (else): needs agent", NoHint),
    bind_row(Yak, &[c('Q')], YakAsk, "Ask a question: needs human", NoHint),
    bind_row(Yak, &[c('Z')], YakSnooze, "Snooze: move the due date (+1d, +1w, a date); the marked set when rows are marked", NoHint),
    bind_row(Yak, &[c('A')], YakAttach, "Attach artifact", NoHint),
    bind_row(Yak, &[c('*')], YakStar, "Star / unstar (Starred view)", NoHint),
    bind_row(Yak, &[c('y')], RowCopy, "Copy yak id to clipboard", NoHint),
    bind_row(Yak, &[c('O')], TakeOut, TAKE_OUT_ROW, NoHint),
    bind_row(Yak, &[c('/'), ctrl('f')], SearchOpen, "Search every view; Esc clears", NoHint),
    bind(List, &[c('n'), k(F(3))], FindNext, "Search again: the pane's last find becomes the query (no query set)", NoHint),
    bind(List, &[c('N'), shift(F(3))], FindPrev, "Search again, as n", NoHint),
    bind(List, &[c('#')], NavGoto, "Go to any yak by id / title", NoHint),
    bind(List, &[c('i')], NavForward, "History forward: a jump you went back from", NoHint),
    bind(List, &[c('o')], NavBack, "History back: where the last jump left (a link, a view switch, #), position restored", NoHint),
    bind_row(Yak, &[c('B')], YakBackrefs, "Find references: yaks that point at this one", NoHint),
    bind_row(Yak, &[c('f')], FilterDrawer, "Filter drawer", FILTER_HINT),
    bind(List, &[c('W')], FarmCommit, "Commit the farm (yaks commit)", NoHint),
    bind(List, &[c('U')], FarmSync, "Sync the farm: commit, pull, push (yaks sync)", NoHint),
    bind(List, &[c('?'), k(F(1))], AppHelp, HELP_LINE, hint("?:help")),
    bind(List, &[c('q'), ctrl('c')], AppQuit, "Quit", NoHint),

    // -- the yak detail pane ---------------------------------------------------
    bind(Detail, &[k(Esc)], PanePeel, "Peel back: selection, then the search, then the list", hint("Esc:list")),
    bind_pane(Shed, &[c('y')], RowCopy, "Copy the shed's name", hint("y:copy name")),
    bind_pane(Shed, &[ctrl('r')], ListRefresh, "Rescan now (auto every 10 s)", NoHint),
    bind_pane(Yak, &[c('h'), k(Left)], PaneScrollLeft, "Scroll the code block / table under the cursor left", NoHint),
    bind_pane(Yak, &[c('l'), k(Right)], PaneScrollRight, "Scroll the code block / table under the cursor right", NoHint),
    bind_pane(Yak, &[c('j'), k(Down)], MoveDown, "Move the line cursor down", hint_family("j/k:move", "↑/↓:move")),
    bind_pane(Yak, &[c('k'), k(Up)], MoveUp, "Move the line cursor up", NoHint),
    bind_pane(Yak, &[c('d'), ctrl('d')], MoveHalfDown, "Half-page down", NoHint),
    bind_pane(Yak, &[c('u'), ctrl('u')], MoveHalfUp, "Half-page up", NoHint),
    bind_pane(Yak, &[k(PageDown)], MovePageDown, "Full-page down", NoHint),
    bind_pane(Yak, &[k(PageUp)], MovePageUp, "Full-page up", NoHint),
    bind_pane(Yak, &[c('g'), k(Home)], MoveTop, "Top", NoHint),
    bind_pane(Yak, &[c('G'), k(End)], MoveBottom, "Bottom", NoHint),
    bind_pane(Yak, &[k(Tab), c(']')], LinkNext, "Next link (each link on a row is a stop)", hint("Tab:link")),
    bind_pane(Yak, &[k(BackTab), c('[')], LinkPrev, "Previous link", NoHint),
    bind_pane(Yak, &[k(Enter)], PaneEnter, "Yank the selection, else follow the link / footnote / source file / farm dok (status bar: the destination; the peek pane shows what it names)", hint("Enter:follow")),
    bind_pane(Yak, &[c('p')], PeekToggle, "Show / hide the peek pane under the lines (what the farm link, source or code range under the cursor names)", hint("p:peek")),
    bind_pane(Yak, &[c('O')], TakeOut, TAKE_OUT_LINE, hint("O:take out")),
    bind(Detail, &[c('i')], NavForward, "History forward: a jump you went back from (any pane)", hint("i/o:fwd/back")),
    bind(Detail, &[c('o')], NavBack, "History back: where the last jump left (a link, a view switch, #), position restored", NoHint),
    bind(Detail, &[c('J')], ItemNext, "Next task (stay in detail)", NoHint),
    bind(Detail, &[c('K')], ItemPrev, "Previous task (stay in detail)", NoHint),
    bind(Detail, &[c('}')], ViewNext, VIEW_PANE_NEXT, NoHint),
    bind(Detail, &[c('{')], ViewPrev, VIEW_PANE_PREV, NoHint),
    bind_pane(Yak, &[ctrl('n'), ctrl_k(Down)], BlockNext, "Next content block", NoHint),
    bind_pane(Yak, &[ctrl('p'), ctrl_k(Up)], BlockPrev, "Previous content block", NoHint),
    bind_pane(Yak, &[c('v')], SelectVisual, "Visual select lines", NoHint),
    bind_pane(Yak, &[shift(Down)], SelectExtendDown, "Extend the selection down", NoHint),
    bind_pane(Yak, &[shift(Up)], SelectExtendUp, "Extend the selection up", NoHint),
    bind_pane(Yak, &[c('y')], SelectYank, "Copy the selection, else the yak id", NoHint),
    bind_pane(Yak, &[c('/'), ctrl('f')], FindOpen, "Find in the pane (sets the search for every dok and yak)", NoHint),
    bind_pane(Yak, &[c('n'), k(F(3))], FindNext, "Next match; past the last, the next matching yak", NoHint),
    bind_pane(Yak, &[c('N'), shift(F(3))], FindPrev, "Previous match; before the first, the previous yak", NoHint),
    bind(Detail, &[c('#')], NavGoto, "Go to any yak by id / title", NoHint),
    bind_pane(Yak, &[c('B')], YakBackrefs, "Find references: yaks that point at this one", NoHint),
    bind_pane(Yak, &[c('E')], YakEdit, "Edit under the cursor: body block in place; header row: form (with the description)", hint("E:edit")),
    bind_pane(Yak, &[c('M')], YakComment, "Add a comment (in place, after the last one)", hint("M:comment")),
    bind_pane(Yak, &[c('D')], YakDep, "Add dependency", hint("D:dep")),
    bind_pane(Yak, &[c('S')], YakState, "State (S → n shears, with a change line)", hint("S:state")),
    bind_pane(Yak, &[c('m')], YakMark, "Mark / unmark the yak (multi-select)", NoHint),
    bind_pane(Yak, &[c('P')], YakPriority, "Priority", NoHint),
    bind_pane(Yak, &[c('Y')], YakType, "Type", NoHint),
    bind_pane(Yak, &[c('T')], YakTags, "Tags", NoHint),
    bind_pane(Yak, &[c('X')], YakSlaughter, "Slaughter (confirm; offers whole family)", NoHint),
    bind_pane(Yak, &[c('R')], YakReparent, "Reparent", NoHint),
    bind_pane(Yak, &[c('H')], YakHerd, "Move to another herd (multi-herd farm)", NoHint),
    bind_pane(Yak, &[c('c')], YakNew, "New root yak", NoHint),
    bind_pane(Yak, &[c('C')], YakNewChild, "New child yak", NoHint),
    bind_pane(Yak, &[c('a')], YakAnswer, "Answer (awaiting a human) / request agent work (else): needs agent", NoHint),
    bind_pane(Yak, &[c('Q')], YakAsk, "Ask a question: needs human", NoHint),
    bind_pane(Yak, &[c('Z')], YakSnooze, "Snooze: move the due date (+1d, +1w, a date)", NoHint),
    bind_pane(Yak, &[c('A')], YakAttach, "Attach artifact", NoHint),
    bind_pane(Yak, &[c('r')], AttachmentRename, "Rename the attachment at the cursor", NoHint),
    bind_pane(Yak, &[c('*')], YakStar, "Star / unstar (Starred view)", NoHint),
    bind_pane(Yak, &[c('f')], FilterDrawer, "Filter drawer", FILTER_HINT),
    bind(Detail, &[c('W')], FarmCommit, "Commit the farm (yaks commit)", NoHint),
    bind(Detail, &[c('U')], FarmSync, "Sync the farm: commit, pull, push (yaks sync)", NoHint),
    bind(Detail, &[c('?'), k(F(1))], AppHelp, HELP_LINE, NoHint),
    bind(Detail, &[c('q'), ctrl('c')], AppQuit, "Quit", hint("q:quit")),

    // -- the text viewer -------------------------------------------------------
    bind(Viewer, &[k(Esc)], PanePeel, "Peel back: selection, then the search, then close", NoHint),
    bind(Viewer, &[c('q'), ctrl('c')], AppQuit, "Quit", NoHint),
    bind(Viewer, &[c('h'), k(Left)], PaneScrollLeft, "Scroll the code block / table under the cursor left", NoHint),
    bind(Viewer, &[c('l'), k(Right)], PaneScrollRight, "Scroll the code block / table under the cursor right", NoHint),
    bind(Viewer, &[c('j'), k(Down)], MoveDown, "Move the line cursor down", NoHint),
    bind(Viewer, &[c('k'), k(Up)], MoveUp, "Move the line cursor up", NoHint),
    bind(Viewer, &[c('d'), ctrl('d')], MoveHalfDown, "Half-page down", NoHint),
    bind(Viewer, &[c('u'), ctrl('u')], MoveHalfUp, "Half-page up", NoHint),
    bind(Viewer, &[k(PageDown)], MovePageDown, "Full-page down", NoHint),
    bind(Viewer, &[k(PageUp)], MovePageUp, "Full-page up", NoHint),
    bind(Viewer, &[c('g'), k(Home)], MoveTop, "Top", NoHint),
    bind(Viewer, &[c('G'), k(End)], MoveBottom, "Bottom", NoHint),
    bind(Viewer, &[k(Tab), c(']')], LinkNext, "Next link (each link on a row is a stop)", NoHint),
    bind(Viewer, &[k(BackTab), c('[')], LinkPrev, "Previous link", NoHint),
    bind(Viewer, &[k(Enter)], PaneEnter, "Yank the selection, else follow the link (o / i come back)", NoHint),
    bind(Viewer, &[c('o')], NavBack, "History back: over a footnote or dok jump, then to where the dok was opened from", NoHint),
    bind(Viewer, &[c('i')], NavForward, "History forward", NoHint),
    bind(Viewer, &[c('v')], SelectVisual, "Visual select lines", NoHint),
    bind(Viewer, &[shift(Down)], SelectExtendDown, "Extend the selection down", NoHint),
    bind(Viewer, &[shift(Up)], SelectExtendUp, "Extend the selection up", NoHint),
    bind(Viewer, &[c('y')], SelectYank, "Copy the selection (a .md file: its source), else the file's path", NoHint),
    bind(Viewer, &[c('/'), ctrl('f')], FindOpen, "Find in the file (the same search as the yak detail)", NoHint),
    bind(Viewer, &[c('n'), k(F(3))], FindNext, "Next match (wraps)", NoHint),
    bind(Viewer, &[c('N'), shift(F(3))], FindPrev, "Previous match (wraps)", NoHint),
    bind(Viewer, &[c('J')], ItemNext, "Next dok of the Doks list (opened in place)", NoHint),
    bind(Viewer, &[c('K')], ItemPrev, "Previous dok of the Doks list", NoHint),
    bind(Viewer, &[c('}')], ViewNext, VIEW_PANE_NEXT, NoHint),
    bind(Viewer, &[c('{')], ViewPrev, VIEW_PANE_PREV, NoHint),
    bind(Viewer, &[c('w')], DocWiden, "A range link's view: the whole file, and back to the range", NoHint),
    bind(Viewer, &[c('p')], PeekToggle, "Show / hide the peek pane under the dok (the same flag as the yak pane's)", NoHint),
    bind(Viewer, &[c('*')], YakStar, "Star / unstar the file (Starred view)", NoHint),
    bind(Viewer, &[c('E')], DocEdit, "Edit the file in place (Ctrl-S saves; a past version or a source is read-only)", NoHint),
    bind(Viewer, &[c('O')], TakeOut, TAKE_OUT_FILE, NoHint),
    bind(Viewer, &[c('#')], NavGoto, "Go to any yak by id / title (closes the dok)", NoHint),
    bind(Viewer, &[c('W')], FarmCommit, "Commit the farm (yaks commit)", NoHint),
    bind(Viewer, &[c('U')], FarmSync, "Sync the farm: commit, pull, push (yaks sync)", NoHint),
    bind(Viewer, &[c('?'), k(F(1))], AppHelp, HELP_LINE, NoHint),

    // -- the image viewer ------------------------------------------------------
    bind(Image, &[k(Esc)], PaneClose, "Close", NoHint),
    bind(Image, &[c('+'), c('=')], ImageZoomIn, "Zoom in", NoHint),
    bind(Image, &[c('-')], ImageZoomOut, "Zoom out", NoHint),
    bind(Image, &[c('0')], ImageZoomActual, "Actual size: one image pixel per pixel of the cell grid", NoHint),
    bind(Image, &[c('w')], ImageFitWidth, "Fit the pane's width (a taller image pans down)", NoHint),
    bind(Image, &[c('f')], ImageFit, "Fit the pane: shrink to fit, never enlarge (the default)", NoHint),
    bind(Image, &[c('h'), k(Left)], ImagePanLeft, "Pan left (an image larger than the pane)", NoHint),
    bind(Image, &[c('l'), k(Right)], ImagePanRight, "Pan right", NoHint),
    bind(Image, &[c('k'), k(Up)], ImagePanUp, "Pan up", NoHint),
    bind(Image, &[c('j'), k(Down)], ImagePanDown, "Pan down", NoHint),
    bind(Image, &[c('q'), ctrl('c')], AppQuit, "Quit", NoHint),
    bind(Image, &[c('o')], NavBack, "History back", NoHint),
    bind(Image, &[c('i')], NavForward, "History forward", NoHint),
    bind(Image, &[c('*')], YakStar, "Star / unstar the image (Starred view)", NoHint),
    bind(Image, &[c('O')], TakeOut, TAKE_OUT_FILE, NoHint),
    bind(Image, &[c('#')], NavGoto, "Go to any yak by id / title (closes the image)", NoHint),
    bind(Image, &[c('W')], FarmCommit, "Commit the farm (yaks commit)", NoHint),
    bind(Image, &[c('U')], FarmSync, "Sync the farm: commit, pull, push (yaks sync)", NoHint),
    bind(Image, &[c('}')], ViewNext, VIEW_PANE_NEXT, NoHint),
    bind(Image, &[c('{')], ViewPrev, VIEW_PANE_PREV, NoHint),
    bind(Image, &[c('?'), k(F(1))], AppHelp, HELP_LINE, NoHint),

    // -- the directory view -----------------------------------------------------
    bind(Dir, &[k(Esc)], PaneClose, "Close (o goes back to where it was opened from)", hint("Esc:close")),
    bind(Dir, &[c('j'), k(Down)], MoveDown, "Move the cursor down", hint_family("j/k:move", "↑/↓:move")),
    bind(Dir, &[c('k'), k(Up)], MoveUp, "Move the cursor up", NoHint),
    bind(Dir, &[c('d'), ctrl('d')], MoveHalfDown, "Half-page down", NoHint),
    bind(Dir, &[c('u'), ctrl('u')], MoveHalfUp, "Half-page up", NoHint),
    bind(Dir, &[k(PageDown)], MovePageDown, "Full-page down", NoHint),
    bind(Dir, &[k(PageUp)], MovePageUp, "Full-page up", NoHint),
    bind(Dir, &[c('g'), k(Home)], MoveTop, "Top", NoHint),
    bind(Dir, &[c('G'), k(End)], MoveBottom, "Bottom", NoHint),
    bind(Dir, &[k(Enter)], PaneEnter, "Open the file in the viewer (o / i come back); fold or unfold a directory or a .zip/.tar/.tgz archive; any other archive is never opened, O → p copies its path", hint("Enter:open")),
    bind(Dir, &[c(' ')], TreeToggle, "Fold / unfold the directory", NoHint),
    bind(Dir, &[c('l'), k(Right)], TreeRight, "Unfold the directory, else go to its first entry", NoHint),
    bind(Dir, &[c('h'), k(Left)], TreeLeft, "Fold the directory, else go to the directory that holds the row", NoHint),
    bind(Dir, &[c('y')], RowCopy, "Copy the row's farm path", hint("y:copy path")),
    bind(Dir, &[c('*')], YakStar, "Star / unstar the directory (Starred view)", NoHint),
    bind(Dir, &[c('O')], TakeOut, TAKE_OUT_FILE, hint("O:take out")),
    bind(Dir, &[c('o')], NavBack, "History back", NoHint),
    bind(Dir, &[c('i')], NavForward, "History forward", hint("i/o:fwd/back")),
    bind(Dir, &[c('#')], NavGoto, "Go to any yak by id / title (closes the view)", NoHint),
    bind(Dir, &[c('W')], FarmCommit, "Commit the farm (yaks commit)", NoHint),
    bind(Dir, &[c('U')], FarmSync, "Sync the farm: commit, pull, push (yaks sync)", NoHint),
    bind(Dir, &[c('}')], ViewNext, VIEW_PANE_NEXT, NoHint),
    bind(Dir, &[c('{')], ViewPrev, VIEW_PANE_PREV, NoHint),
    bind(Dir, &[c('?'), k(F(1))], AppHelp, HELP_LINE, NoHint),
    bind(Dir, &[c('q'), ctrl('c')], AppQuit, "Quit", hint("q:quit")),

    // -- the commit view ------------------------------------------------------
    bind(Commit, &[k(Esc)], PaneClose, "Close (o goes back to where it was opened from)", hint("Esc:close")),
    bind(Commit, &[c('j'), k(Down)], MoveDown, "Move the cursor down", hint_family("j/k:move", "↑/↓:move")),
    bind(Commit, &[c('k'), k(Up)], MoveUp, "Move the cursor up", NoHint),
    bind(Commit, &[c('d'), ctrl('d')], MoveHalfDown, "Half-page down", NoHint),
    bind(Commit, &[c('u'), ctrl('u')], MoveHalfUp, "Half-page up", NoHint),
    bind(Commit, &[k(PageDown)], MovePageDown, "Full-page down", NoHint),
    bind(Commit, &[k(PageUp)], MovePageUp, "Full-page up", NoHint),
    bind(Commit, &[c('g'), k(Home)], MoveTop, "Top", NoHint),
    bind(Commit, &[c('G'), k(End)], MoveBottom, "Bottom", NoHint),
    bind(Commit, &[k(Enter)], PaneEnter, "On a file: unfold / fold its diff under it. On a diff line: open the file as it was at this commit, at that line (o / i come back). On Landed as: open the landed commit", hint("Enter:diff")),
    bind(Commit, &[c(' ')], TreeToggle, "Unfold / fold the file's diff under it (from a diff line, its file's)", NoHint),
    bind(Commit, &[c('l'), k(Right)], TreeRight, "Unfold the file's diff", NoHint),
    bind(Commit, &[c('h'), k(Left)], TreeLeft, "Fold the diff, the cursor back on its file", NoHint),
    bind(Commit, &[c('y')], RowCopy, "Copy the commit hash", hint("y:copy hash")),
    bind(Commit, &[c('*')], YakStar, "Star / unstar the commit (Starred view)", NoHint),
    bind(Commit, &[c('O')], TakeOut, TAKE_OUT_COMMIT, hint("O:take out")),
    bind(Commit, &[c('o')], NavBack, "History back", NoHint),
    bind(Commit, &[c('i')], NavForward, "History forward", hint("i/o:fwd/back")),
    bind(Commit, &[c('#')], NavGoto, "Go to any yak by id / title (closes the view)", NoHint),
    bind(Commit, &[c('W')], FarmCommit, "Commit the farm (yaks commit)", NoHint),
    bind(Commit, &[c('U')], FarmSync, "Sync the farm: commit, pull, push (yaks sync)", NoHint),
    bind(Commit, &[c('}')], ViewNext, VIEW_PANE_NEXT, NoHint),
    bind(Commit, &[c('{')], ViewPrev, VIEW_PANE_PREV, NoHint),
    bind(Commit, &[c('?'), k(F(1))], AppHelp, HELP_LINE, NoHint),
    bind(Commit, &[c('q'), ctrl('c')], AppQuit, "Quit", hint("q:quit")),

    // -- the list: document rows (the Doks view). Its movement, view and farm
    // -- keys are the list's; `` ` ``, `?` and `q` repeat theirs only so that the
    // -- bottom hint is the Doks's own, not the yak list's.
    bind_row(Doc, &[k(Enter)], PaneOpen, "Open in the viewer: frontmatter above the body", hint("Enter:open")),
    bind_row(Doc, &[c('E')], DokEdit, "Edit the dok: frontmatter (YAML) and body; Ctrl-S saves", hint("E:edit")),
    bind_row(Doc, &[c('c')], DokNew, "New dok: type its path (doks/... or <herd>/doks/...), then its body", hint("c:new dok")),
    bind_row(Doc, &[c('/'), ctrl('f')], SearchOpen, "Search every view; Esc clears", hint("/:search")),
    bind_row(Doc, &[c('*')], YakStar, "Star / unstar the document (Starred view)", NoHint),
    bind_row(Doc, &[c('y')], RowCopy, "Copy the path", hint("y:copy path")),
    bind_row(Doc, &[c('O')], TakeOut, TAKE_OUT_ROW, hint("O:take out")),
    bind_row(Doc, &[ctrl('r')], ListRefresh, "Reread the doks (also on any farm change)", NoHint),
    bind_row(Doc, &[c('`')], ViewPick, "", NoHint),
    bind_row(Doc, &[c('?'), k(F(1))], AppHelp, "", hint("?:help")),
    bind_row(Doc, &[c('q'), ctrl('c')], AppQuit, "", hint("q:quit")),

    // -- the list: starred place rows (the Starred view: an attachment, a
    // -- directory, a code file or a commit). Its movement, view and farm keys
    // -- are the list's; `` ` ``, `?` and `q` repeat theirs only so that the
    // -- bottom hint is the place row's own.
    bind_row(Place, &[k(Enter)], PaneOpen, "Open it: a file, directory or commit, as a followed link does (o goes back)", hint("Enter:open")),
    bind_row(Place, &[c('/'), ctrl('f')], SearchOpen, "Search every view; Esc clears", hint("/:search")),
    bind_row(Place, &[c('*')], YakStar, "Unstar it", NoHint),
    bind_row(Place, &[c('O')], TakeOut, TAKE_OUT_ROW, hint("O:take out")),
    bind_row(Place, &[c('`')], ViewPick, "", NoHint),
    bind_row(Place, &[c('?'), k(F(1))], AppHelp, "", hint("?:help")),
    bind_row(Place, &[c('q'), ctrl('c')], AppQuit, "", hint("q:quit")),

    // -- the list: shed rows (the Sheds view). A shed's yaks below it are yak
    // -- rows. Its movement and farm keys are the list's; `` ` ``, `?` and `q`
    // -- repeat theirs only so that the bottom hint is the Sheds' own.
    bind_row(Shed, &[k(Enter)], PaneOpen, "Show the shed: where its code is, and a count of its yaks", hint("Enter:detail")),
    bind_row(Shed, &[c('h'), k(Left)], TreeLeft, "Collapse the shed's yaks", NoHint),
    bind_row(Shed, &[c('l'), k(Right)], TreeRight, "Expand the shed's yaks, else go to the first", NoHint),
    bind_row(Shed, &[c(' ')], TreeToggle, "Collapse / expand the shed's yaks (idle sheds start collapsed)", NoHint),
    bind_row(Shed, &[c('/'), ctrl('f')], SearchOpen, "Search every view; Esc clears", hint("/:search")),
    bind_row(Shed, &[c('y')], RowCopy, "Copy the shed's name", hint("y:copy name")),
    bind_row(Shed, &[ctrl('r')], ListRefresh, "Rescan now (auto every 10 s)", NoHint),
    bind_row(Shed, &[c('`')], ViewPick, "View picker", hint("`:views")),
    bind_row(Shed, &[c('?'), k(F(1))], AppHelp, "", hint("?:help")),
    bind_row(Shed, &[c('q'), ctrl('c')], AppQuit, "", hint("q:quit")),
];

/// The `?` lines of `view.next` / `view.prev` in the pane and the viewers.
const VIEW_PANE_NEXT: &str = "Next view; the pane stays open on the new view's first row";
const VIEW_PANE_PREV: &str = "Previous view; the pane stays open on the new view's first row";

/// The `?` lines of `take.out`: what the menu takes out, by surface. The
/// menu's own keys are in the [`EXTRAS`].
const TAKE_OUT_LINE: &str = "Take out the link on the line (or the yak): a menu to open it in your editor, with the system, or copy its path or host link";
const TAKE_OUT_FILE: &str = "Take out the file: a menu to open it in your editor at the cursor's line, with the system, or copy its path or host link";
const TAKE_OUT_COMMIT: &str =
    "Take out the commit: a menu to copy its host link (g) or its hash (p)";
const TAKE_OUT_ROW: &str = "Take out the row's file (the yak's file): a menu to open it in your editor, with the system, or copy its path or host link";

/// The `?` line of the help command: the popup first, the reference on a second press.
const HELP_LINE: &str = "Keys of this context in a popup; ? again opens this full reference";

/// The bottom bar's hints for `ctx` on a row of `kind`; `filter_active` is
/// whether a filter or query is set, which changes the drawer's label.
pub(crate) fn hint_for(
    km: &Keymap,
    ctx: Ctx,
    kind: Option<RowKind>,
    filter_active: bool,
) -> String {
    km.hint_for(ctx, kind, |_filter, km| {
        let (find, clear) = match ctx {
            Ctx::List => (SearchOpen, ViewRevert),
            _ => (FindOpen, PanePeel),
        };
        let then = if filter_active {
            format!("{}:clear", km.first_label(ctx, clear))
        } else {
            format!("{}:search", km.first_label(ctx, find))
        };
        Some(format!(
            "{}:filter  {then}",
            km.first_label(ctx, FilterDrawer)
        ))
    })
}

/// What the popup lists while an overlay is up (see
/// [`App::popup_subject`]).
#[derive(Clone, Copy, Debug)]
pub(crate) enum PopupSubject {
    /// The text editor that has the focus: its own keys, by family and mode.
    Editor(EditorAt),
    /// The chip rows of a form or the filter drawer.
    Rows,
}

pub(crate) fn help_hint(app: &App) -> String {
    let filter_active = app.filter.content_active() || app.query.is_some();
    hint_for(
        &app.keymap,
        app.surface_ctx(),
        app.surface_kind(),
        filter_active,
    )
}

/// The `?` overlay's rows that are not key bindings: the editors' own keys
/// (modal, so outside the table), what a file or link does, the farm chip.
/// Each section is a title and its (label, text) rows.
const EXTRAS: &[toque::keys::Extra] = &[
    (
        "Editing (the editors and pickers are modal; their keys are not in the table)",
        &[
            (
                "S → n",
                "Shear: prompts for the change line, the title to edit (Enter shears; empty for none)",
            ),
            (
                "F1",
                "In any editor or form: its keys in a popup (a listed key goes on to the editor); also ? in a vim editor's Normal mode",
            ),
            (
                "Ctrl-S / Ctrl-C",
                "Save / cancel an in-place edit (Esc Esc cancels too)",
            ),
            (
                ":w / :q / :wq",
                "Save / cancel / save+close (editor Normal)",
            ),
            (
                "Enter",
                "(editor Insert) Continue a list / quote; on an empty item, end it",
            ),
            (
                "k / Enter",
                "(Unsaved changes? prompt, on a form or new comment) Keep what you typed as a draft",
            ),
            (
                "d",
                "(Unsaved changes? prompt) Discard it and delete any draft; Esc goes back to editing",
            ),
            (
                "Ctrl-R",
                "(a form or comment restored from a draft) Revert to the yak's current content and delete the draft",
            ),
        ],
    ),
    (
        "Take out (O: the menu over the foot of the screen; none of it changes a yak; a commit's has only p and g)",
        &[
            (
                "e",
                "Open in $VISUAL / $EDITOR at the line, the terminal suspended while it runs",
            ),
            (
                "o",
                "Open with the system (the default application); never an archive file itself (p copies its path); a member of an archive opens as a read-only copy",
            ),
            (
                "p",
                "Copy the absolute path (a pinned code: file: the working file if unchanged since the pin, else a read-only copy of it)",
            ),
            ("r", "Copy the farm- or repo-relative path"),
            (
                "g",
                "Copy the host link: GitHub, GitLab, Gitea/Forgejo (a custom domain: git config yaks.host)",
            ),
            ("Esc / q", "Close the menu"),
        ],
    ),
    (
        "Files, links and images",
        &[
            (
                "(text file)",
                "Opens in a viewer (.md rendered); E there edits it in place (Ctrl-S saves); O takes it out",
            ),
            (
                "(code file)",
                "A code: link opens in the viewer, read-only: line numbers, coloured by extension, at its #L lines; a pin is read from git. O then e edits it",
            ),
            (
                "(farm links)",
                "/doks/x.md#h, dok-relative, /sources/..., @sha pins: open in the viewer at the anchor; o/i go back/fwd",
            ),
            (
                "(peek)",
                "A pane under the dok previews the link under the cursor (section, lines, dok head); p hides it",
            ),
            (
                "(directory)",
                "A link to a directory, or an Attachments: / Files: row, opens it as a tree in the pane: Enter opens a file or folds a directory or a .zip/.tar/.tgz archive; any other archive is never opened",
            ),
            (
                "(image)",
                "Shown in the pane (png/jpg/gif/webp/svg); O takes it out, Esc/q closes",
            ),
            (
                "(body images)",
                "A body paragraph's images are drawn under it, svg too, and their title is a caption (config inline_images)",
            ),
        ],
    ),
    (
        "Customising keys",
        &[
            (
                "keys:",
                "Rebind a command in ~/.config/yaks/config.yaml (view.next: [\"Ctrl-L\"]); the names are in the ? popup",
            ),
            (
                "keys: standard",
                "The same file picks the key family for the whole TUI: vim (default) or standard (arrows, PgUp/PgDn, Home/End, Ctrl-F find; emacs-style editors). vim_mode: false is its alias",
            ),
        ],
    ),
    (
        "Farm git state (the chip on the tab row)",
        &[
            (
                "auto",
                "Syncs itself every sync_interval (config.yaml, default 5m; off)",
            ),
            (
                "chip",
                "● N uncommitted, ↑ahead, ↓behind, not published; none when clean",
            ),
        ],
    ),
    (
        "Mouse",
        &[
            (
                "Wheel",
                "Scroll the pane under the pointer (the cursor follows at the edge)",
            ),
            (
                "Sideways wheel",
                "Scroll the code block or table under the pointer",
            ),
            (
                "Click row",
                "Select (again: open detail; detail stays open); closes an open dok",
            ),
            ("Click detail", "Move line cursor (again: follow link)"),
            ("Click tab", "Switch view"),
            (
                "Click form",
                "Focus a row; select a chip; in a focused field, move the text cursor",
            ),
            (
                "Click editor",
                "In-place editor: move the text cursor; drag selects",
            ),
            ("Shift-drag", "Native text selection (terminal)"),
        ],
    ),
];

/// The keyboard reference shown by `?`, as styled lines: cyan-bold section
/// headers, a yellow key column, then the description. A section per context
/// from [`BINDINGS`], then the [`EXTRAS`]. Rebuilt on demand; also used to
/// clamp help scroll.
pub(crate) fn help_content(km: &Keymap) -> Vec<Line<'static>> {
    toque::keys::reference(km, &SECTIONS, EXTRAS)
}

impl App {
    /// The surface that takes a key when no overlay is up: the open document
    /// (it is the pane's subject), else the list or its detail pane.
    pub(crate) fn surface_ctx(&self) -> Ctx {
        match (&self.doc, self.focus) {
            (
                Some(DocPane {
                    view: DocView::Text(_),
                    ..
                }),
                _,
            ) => Ctx::Viewer,
            (
                Some(DocPane {
                    view: DocView::Image(_),
                    ..
                }),
                _,
            ) => Ctx::Image,
            (
                Some(DocPane {
                    view: DocView::Dir(_),
                    ..
                }),
                _,
            ) => Ctx::Dir,
            (
                Some(DocPane {
                    view: DocView::Commit(_),
                    ..
                }),
                _,
            ) => Ctx::Commit,
            (None, Focus::List) => Ctx::List,
            (None, Focus::Detail) => Ctx::Detail,
        }
    }

    /// The kind of row the list's and the pane's keys act on: the selected
    /// row's, else the view's.
    pub(crate) fn surface_kind(&self) -> Option<RowKind> {
        Some(self.row_kind())
    }

    /// Run `cmd` as the key bound to it in `ctx`. A command is an intent, so
    /// where surfaces differ (what `move.down` moves, what Esc peels) the arm
    /// picks the target from `ctx`; the viewers run theirs themselves.
    pub(crate) fn run_cmd(&mut self, ctx: Ctx, cmd: Cmd) {
        // The global commands run as anywhere; the rest of a viewer's are its own.
        // `*` stars the pane's subject on every surface ([`App::toggle_star`]).
        let global = matches!(
            cmd,
            AppQuit
                | AppHelp
                | NavGoto
                | FarmCommit
                | FarmSync
                | ViewNext
                | ViewPrev
                | TakeOut
                | YakStar
        );
        match ctx {
            Ctx::Viewer if !global => return self.run_viewer_cmd(cmd),
            Ctx::Image if !global => return self.run_image_cmd(cmd),
            Ctx::Dir if !global => return self.run_dir_cmd(cmd),
            Ctx::Commit if !global => return self.run_commit_cmd(cmd),
            _ => {}
        }
        match cmd {
            AppQuit => self.quit = true,
            AppHelp => self.open_keys_popup(),
            MoveDown | MoveUp | MoveTop | MoveBottom | MoveHalfDown | MoveHalfUp | MovePageDown
            | MovePageUp => self.run_move(ctx, cmd),
            ItemNext | ItemPrev => self.detail_next_task(if cmd == ItemNext { 1 } else { -1 }),
            ViewNext => self.switch_tab(1),
            ViewPrev => self.switch_tab(-1),
            ViewPick => self.open_view_picker(),
            ViewSave => self.open_save_view(),
            ViewRevert => self.revert_list_view(),
            FamilyCycle => self.cycle_family_scope(),
            TreeToggle => self.toggle_collapse(),
            TreeLeft => self.tree_left(),
            TreeRight => self.tree_right(),
            PaneOpen => match ctx {
                Ctx::List if self.selected_doc().is_some() => self.open_doks_entry(),
                Ctx::List if self.selected_place().is_some() => self.open_selected_place(),
                _ => self.open_detail_pane(),
            },
            PaneEnter => self.detail_enter(),
            PanePeel => self.detail_peel(),
            PaneScrollLeft => {
                self.scroll_block(-1);
            }
            PaneScrollRight => {
                self.scroll_block(1);
            }
            RowCopy => match ctx {
                Ctx::List if self.selected_doc().is_some() => self.copy_doks_path(),
                _ if self.selected_shed().is_some() => self.copy_shed_name(),
                _ => self.copy_selected_id(),
            },
            ListRefresh => match self.row_kind() {
                RowKind::Shed => self.rescan_sheds(),
                _ => self.reread_doks(),
            },
            SearchOpen => self.open_search(),
            FilterDrawer => self.open_drawer(),
            NavGoto => self.open_goto_picker(),
            NavBack => self.history_back(),
            NavForward => self.history_forward(),
            LinkNext => self.jump_link(1),
            LinkPrev => self.jump_link(-1),
            TakeOut => self.open_take_out(ctx),
            FindOpen => self.open_find(),
            FindNext | FindPrev => {
                let delta = if cmd == FindNext { 1 } else { -1 };
                match ctx {
                    Ctx::List => self.list_find(),
                    _ => self.detail_find_jump(delta),
                }
            }
            SelectVisual => self.toggle_visual(),
            SelectExtendDown => self.extend_selection(1),
            SelectExtendUp => self.extend_selection(-1),
            SelectYank => self.yank_selection_or_copy_id(),
            BlockNext => self.jump_block(1),
            BlockPrev => self.jump_block(-1),
            FarmCommit => self.request_farm_op(FarmOp::Commit),
            FarmSync => self.request_farm_op(FarmOp::Sync),
            YakMark => self.toggle_selected(),
            YakState => match ctx {
                Ctx::List if !self.selected.is_empty() => self.open_bulk_state_picker(),
                _ => self.open_state_picker(),
            },
            YakPriority => self.open_priority_picker(),
            YakType => self.open_type_picker(),
            YakTags => self.open_tags(),
            YakSlaughter => self.open_slaughter_confirm(),
            YakNew => self.open_create(false),
            YakNewChild => self.open_create(true),
            YakEdit => match ctx {
                Ctx::Detail => self.open_edit_at_cursor(),
                _ => self.open_edit(),
            },
            YakDep => self.open_dep_picker(),
            YakReparent => self.open_reparent_picker(),
            YakHerd => self.open_herd_picker(),
            YakBackrefs => self.open_backrefs_picker(),
            YakStar => self.toggle_star(),
            YakComment => self.open_comment(),
            YakAttach => self.open_attach(),
            YakAnswer => self.open_answer_or_request(),
            YakAsk => self.open_ask(),
            YakSnooze => match ctx {
                Ctx::List if !self.selected.is_empty() => self.open_snooze_marked(),
                _ => self.open_snooze(),
            },
            AttachmentRename => self.open_rename_attachment(),
            DokNew => self.open_new_dok(),
            DokEdit => self.edit_doks_entry(),
            PeekToggle => self.toggle_peek(),
            // The viewers' own: `run_viewer_cmd` and `run_image_cmd`.
            PaneClose | DocEdit | DocWiden | ImageZoomIn | ImageZoomOut | ImageZoomActual
            | ImageFitWidth | ImageFit | ImagePanLeft | ImagePanRight | ImagePanUp
            | ImagePanDown => {}
        }
    }

    /// A movement command on the surface `ctx`: by lines, a half or a full
    /// dok, or to either end.
    fn run_move(&mut self, ctx: Ctx, cmd: Cmd) {
        let half = (self.page / 2).max(1) as i32;
        let full = self.page.max(1) as i32;
        let by = match cmd {
            MoveDown => 1,
            MoveUp => -1,
            MoveHalfDown => half,
            MoveHalfUp => -half,
            MovePageDown => full,
            MovePageUp => -full,
            MoveTop => i32::MIN / 4,
            _ => i32::MAX / 4,
        };
        match ctx {
            Ctx::List if cmd == MoveTop => self.cursor = 0,
            Ctx::List => self.move_cursor(by),
            Ctx::Detail if cmd == MoveTop => self.detail_line_to(false),
            Ctx::Detail if cmd == MoveBottom => self.detail_line_to(true),
            Ctx::Detail => self.move_detail_line(by),
            Ctx::Viewer | Ctx::Image | Ctx::Dir | Ctx::Commit => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use toque::keys::{Command, Context};

    /// The table's own keys: no overrides.
    fn lookup(ctx: Ctx, kind: Option<RowKind>, k: &KeyEvent) -> Option<Cmd> {
        Keymap::default().lookup(ctx, kind, k)
    }

    fn hint_for(ctx: Ctx, kind: Option<RowKind>, filter_active: bool) -> String {
        super::hint_for(&Keymap::default(), ctx, kind, filter_active)
    }

    fn text(lines: &[Line<'_>]) -> String {
        lines
            .iter()
            .map(|l| {
                l.spans
                    .iter()
                    .map(|s| s.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// The popup's entries without their groups.
    fn flat(groups: &PopupGroups) -> Vec<(String, &'static str)> {
        groups.iter().flat_map(|(_, e)| e.clone()).collect()
    }

    /// Every kind of row a binding can name, and `None` (any kind).
    const KINDS: &[Option<RowKind>] = &[None, Some(Yak), Some(Doc), Some(Shed), Some(Place)];

    #[test]
    fn no_two_bindings_in_a_context_and_kind_share_a_key() {
        let problems = toque::keys::rules::no_shared_keys::<Cmd>(KINDS);
        assert!(problems.is_empty(), "{problems:#?}");
    }

    #[test]
    fn every_command_is_bound_and_named_once() {
        let problems = toque::keys::rules::every_command_bound_and_named::<Cmd>();
        assert!(problems.is_empty(), "{problems:#?}");
    }

    #[test]
    fn every_helped_binding_is_in_the_help_overlay() {
        let all = text(&help_content(&Keymap::default()));
        for section in &SECTIONS {
            let sect = all
                .split("\n\n")
                .find(|s| s.starts_with(section.title))
                .unwrap_or_else(|| panic!("no section {}", section.title));
            for b in section.bindings().filter(|b| !b.help.is_empty()) {
                let row = format!("  {:<16}{}", Keymap::default().labels(b, " / "), b.help);
                assert!(
                    sect.lines().any(|l| l == row),
                    "{}: no row {row:?}",
                    section.title
                );
            }
        }
    }

    #[test]
    fn every_binding_is_in_a_help_section() {
        for b in BINDINGS {
            assert!(
                SECTIONS
                    .iter()
                    .any(|s| s.bindings().any(|x| std::ptr::eq(x, b))),
                "{} ({:?}, {:?}) is in no section",
                b.cmd.name(),
                b.ctx,
                b.kind
            );
        }
    }

    /// The bottom hint, per context, as it was written by hand before the
    /// table: the table must reproduce it byte for byte.
    #[test]
    fn each_contexts_hint_is_what_it_was() {
        let cases = [
            (
                Ctx::List,
                Some(RowKind::Yak),
                "Tab:view  j/k:move  Enter:detail  F:family  `:views  c/C:new  E:edit  X:del  S:state  D:dep  f:filter  /:search  ?:help",
            ),
            (
                Ctx::Detail,
                Some(RowKind::Yak),
                "Esc:list  j/k:move  Tab:link  Enter:follow  p:peek  O:take out  i/o:fwd/back  E:edit  M:comment  D:dep  S:state  f:filter  /:search  q:quit",
            ),
            (
                Ctx::List,
                Some(RowKind::Doc),
                "Tab:view  j/k:move  Enter:open  E:edit  c:new dok  /:search  y:copy path  O:take out  ?:help  q:quit",
            ),
            (
                Ctx::List,
                Some(RowKind::Shed),
                "Tab:view  j/k:move  Enter:detail  /:search  y:copy name  `:views  ?:help  q:quit",
            ),
            (
                Ctx::Detail,
                Some(RowKind::Shed),
                "Esc:list  y:copy name  i/o:fwd/back  q:quit",
            ),
            (Ctx::Viewer, None, ""),
            (Ctx::Image, None, ""),
        ];
        for (ctx, kind, want) in cases {
            assert_eq!(hint_for(ctx, kind, false), want, "{ctx:?} {kind:?}");
        }
        // With a filter active the drawer's hint changes, nothing else.
        assert_eq!(
            hint_for(Ctx::Detail, Some(RowKind::Yak), true),
            "Esc:list  j/k:move  Tab:link  Enter:follow  p:peek  O:take out  i/o:fwd/back  E:edit  M:comment  D:dep  S:state  f:filter  Esc:clear  q:quit"
        );
        assert_eq!(
            hint_for(Ctx::List, Some(RowKind::Yak), true),
            "Tab:view  j/k:move  Enter:detail  F:family  `:views  c/C:new  E:edit  X:del  S:state  D:dep  f:filter  Esc:clear  ?:help"
        );
    }

    #[test]
    fn lookup_prefers_a_modifier_binding_and_ignores_extra_modifiers() {
        let key = |code, mods| KeyEvent::new(code, mods);
        // Ctrl-C is quit, not `c` (new yak).
        assert_eq!(
            lookup(
                Ctx::List,
                Some(RowKind::Yak),
                &key(KeyCode::Char('c'), KeyModifiers::CONTROL)
            ),
            Some(AppQuit)
        );
        assert_eq!(
            lookup(
                Ctx::List,
                Some(RowKind::Yak),
                &key(KeyCode::Char('c'), KeyModifiers::NONE)
            ),
            Some(YakNew)
        );
        // An upper-case letter arrives with Shift held.
        assert_eq!(
            lookup(
                Ctx::List,
                Some(RowKind::Yak),
                &key(KeyCode::Char('S'), KeyModifiers::SHIFT)
            ),
            Some(YakState)
        );
        // Shift-↓ extends in the detail pane and is a plain ↓ in the list.
        assert_eq!(
            lookup(
                Ctx::Detail,
                Some(RowKind::Yak),
                &key(KeyCode::Down, KeyModifiers::SHIFT)
            ),
            Some(SelectExtendDown)
        );
        assert_eq!(
            lookup(
                Ctx::List,
                Some(RowKind::Yak),
                &key(KeyCode::Down, KeyModifiers::SHIFT)
            ),
            Some(MoveDown)
        );
        // Ctrl-N is a block jump in the detail pane only: elsewhere Ctrl does
        // not ride along on the plain `n`.
        assert_eq!(
            lookup(
                Ctx::Detail,
                Some(RowKind::Yak),
                &key(KeyCode::Char('n'), KeyModifiers::CONTROL)
            ),
            Some(BlockNext)
        );
        assert_eq!(
            lookup(
                Ctx::Viewer,
                None,
                &key(KeyCode::Char('n'), KeyModifiers::CONTROL)
            ),
            None
        );
    }

    #[test]
    fn the_viewers_take_the_global_keys_but_not_the_view_picker() {
        let key = |ch| KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE);
        for ctx in [Ctx::Viewer, Ctx::Image] {
            assert_eq!(lookup(ctx, None, &key('U')), Some(FarmSync), "{ctx:?}");
            assert_eq!(lookup(ctx, None, &key('W')), Some(FarmCommit), "{ctx:?}");
            assert_eq!(lookup(ctx, None, &key('?')), Some(AppHelp), "{ctx:?}");
            assert_eq!(lookup(ctx, None, &key('#')), Some(NavGoto), "{ctx:?}");
        }
        // `v` is visual select in the text viewer, not the view picker.
        assert_eq!(lookup(Ctx::Viewer, None, &key('v')), Some(SelectVisual));
        assert_eq!(lookup(Ctx::Image, None, &key('v')), None);
    }

    #[test]
    fn lookup_prefers_a_binding_for_the_rows_kind() {
        let key = |ch| KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE);
        let (yak, doc) = (Some(RowKind::Yak), Some(RowKind::Doc));
        // The same key, per kind: E edits a yak by its form, a dok in the editor.
        assert_eq!(lookup(Ctx::List, yak, &key('E')), Some(YakEdit));
        assert_eq!(lookup(Ctx::List, doc, &key('E')), Some(DokEdit));
        // A key bound for any kind reaches every kind.
        assert_eq!(lookup(Ctx::List, yak, &key('j')), Some(MoveDown));
        assert_eq!(lookup(Ctx::List, doc, &key('j')), Some(MoveDown));
        // A yak command is no document command.
        assert_eq!(lookup(Ctx::List, doc, &key('S')), None);
        assert_eq!(lookup(Ctx::List, doc, &key('C')), None);
        // Ctrl-R rescans on a shed row, rereads the doks on a document row.
        let shed = Some(RowKind::Shed);
        let ctrl_r = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL);
        assert_eq!(lookup(Ctx::List, shed, &ctrl_r), Some(ListRefresh));
        assert_eq!(lookup(Ctx::List, shed, &key('S')), None);
        assert_eq!(lookup(Ctx::List, shed, &key('U')), Some(FarmSync));
        // Ctrl-C still quits where a kind binds plain `c`.
        let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(lookup(Ctx::List, yak, &ctrl_c), Some(AppQuit));
        assert_eq!(lookup(Ctx::List, doc, &ctrl_c), Some(AppQuit));
    }

    /// The key pass (yaks-6919): `h`, `v`, `N`, `r` and `q` each mean one
    /// thing in every context that binds them.
    #[test]
    fn h_v_n_r_and_q_have_one_meaning_everywhere() {
        let key = |ch| KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE);
        let ctrl_r = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL);
        let (yak, doc, shed) = (Some(Yak), Some(Doc), Some(Shed));
        let l = Ctx::List;
        // `h` is back / left; the family scope is `F`.
        assert_eq!(lookup(l, yak, &key('h')), Some(TreeLeft));
        assert_eq!(lookup(l, shed, &key('h')), Some(TreeLeft));
        assert_eq!(lookup(l, doc, &key('h')), None);
        assert_eq!(lookup(Ctx::Detail, yak, &key('h')), Some(PaneScrollLeft));
        assert_eq!(lookup(Ctx::Viewer, None, &key('h')), Some(PaneScrollLeft));
        assert_eq!(lookup(Ctx::Image, None, &key('h')), Some(ImagePanLeft));
        assert_eq!(lookup(l, yak, &key('F')), Some(FamilyCycle));
        // `v` is visual select; the view picker is the backtick on every row.
        assert_eq!(lookup(Ctx::Detail, yak, &key('v')), Some(SelectVisual));
        assert_eq!(lookup(Ctx::Viewer, None, &key('v')), Some(SelectVisual));
        for kind in [yak, doc, shed] {
            assert_eq!(lookup(l, kind, &key('v')), None, "{kind:?}");
            assert_eq!(lookup(l, kind, &key('`')), Some(ViewPick), "{kind:?}");
        }
        // `N` is the previous match; a new dok is `c` on a document row.
        for (ctx, kind) in [
            (l, yak),
            (l, doc),
            (l, shed),
            (Ctx::Detail, yak),
            (Ctx::Viewer, None),
        ] {
            assert_eq!(
                lookup(ctx, kind, &key('N')),
                Some(FindPrev),
                "{ctx:?} {kind:?}"
            );
        }
        assert_eq!(lookup(l, doc, &key('c')), Some(DokNew));
        assert_eq!(lookup(l, doc, &key('C')), None);
        assert_eq!(lookup(l, yak, &key('c')), Some(YakNew));
        // `r` renames an attachment; refresh is Ctrl-R.
        assert_eq!(lookup(Ctx::Detail, yak, &key('r')), Some(AttachmentRename));
        for (ctx, kind) in [(l, doc), (l, shed), (Ctx::Detail, shed)] {
            assert_eq!(lookup(ctx, kind, &key('r')), None, "{ctx:?} {kind:?}");
            assert_eq!(
                lookup(ctx, kind, &ctrl_r),
                Some(ListRefresh),
                "{ctx:?} {kind:?}"
            );
        }
        // `q` always quits, in the viewers too.
        for (ctx, kind) in [
            (l, yak),
            (l, doc),
            (l, shed),
            (Ctx::Detail, yak),
            (Ctx::Viewer, None),
            (Ctx::Image, None),
        ] {
            assert_eq!(
                lookup(ctx, kind, &key('q')),
                Some(AppQuit),
                "{ctx:?} {kind:?}"
            );
        }
    }

    /// The popup lists what the table binds in the context, with the command
    /// names an override uses; nothing is listed that the lookup would not run.
    #[test]
    fn the_popup_lists_the_contexts_bound_keys() {
        let km = Keymap::default();
        let list = flat(&km.popup_entries(Ctx::List, Some(Yak)));
        assert!(list.contains(&("Tab/]/}".to_string(), "view.next")));
        assert!(list.contains(&("F".to_string(), "family.cycle")));
        assert!(!list.iter().any(|(_, name)| *name == "dok.new"));
        let docs = flat(&km.popup_entries(Ctx::List, Some(Doc)));
        assert!(docs.contains(&("c".to_string(), "dok.new")));
        // A document row's own `q` stands in for the list's: listed once.
        assert_eq!(docs.iter().filter(|(_, n)| *n == "app.quit").count(), 1);
        for &ctx in Ctx::ALL {
            for kind in [None, Some(Yak), Some(Doc), Some(Shed)] {
                for (keys, name) in flat(&km.popup_entries(ctx, kind)) {
                    let cmd = Cmd::from_name(name).unwrap();
                    let first = Key::parse(keys.split('/').next().unwrap_or(""));
                    // `/` itself splits to nothing: check the plain keys only.
                    if let Some(key) = first {
                        let ev = KeyEvent::new(key.code, key.mods);
                        let got = km.lookup(ctx, kind, &ev);
                        assert!(got.is_some(), "{ctx:?} {kind:?} {keys} {name}");
                        let _ = cmd;
                    }
                }
            }
        }
    }
    // -- the key rules (yaks-7c6e), as tests over the table -------------------

    fn standard() -> Keymap {
        Keymap::new(Family::Standard)
    }

    fn ev(code: KeyCode, mods: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, mods)
    }

    fn plain(code: KeyCode) -> KeyEvent {
        ev(code, KeyModifiers::NONE)
    }

    /// Lowercase looks and moves; Shift changes an existing yak; Ctrl is the
    /// app and the farm. A command that writes (see `Cmd::writes`) must not
    /// sit on an unshifted key, bar the documented exceptions, which add or
    /// mark rather than change (or sit behind a prompt).
    #[test]
    fn no_lowercase_key_changes_an_existing_yak() {
        // (command, why it may be on a lowercase key)
        let exceptions = [
            (YakNew, "`c` adds a root yak"),
            (DokNew, "`c` adds a dok"),
            (
                YakAnswer,
                "`a` answers: it adds a note and hands the yak on",
            ),
            (YakMark, "`m` marks for a bulk action; it writes nothing"),
            (YakStar, "`*` stars, a mark in the working set"),
            (AttachmentRename, "`r` renames behind its own prompt"),
        ];
        let problems = toque::keys::rules::lowercase_only_looks::<Cmd>(&exceptions);
        assert!(problems.is_empty(), "{problems:#?}");
    }

    /// Standard mode is the same commands on other keys: whatever a command
    /// binds in vim mode, it still binds with the vim-only keys off, and no
    /// key left on is one of them.
    #[test]
    fn every_command_stays_bound_in_standard_mode() {
        let problems = toque::keys::rules::every_command_bound_in_both_families::<Cmd>(KINDS);
        assert!(problems.is_empty(), "{problems:#?}");
    }

    #[test]
    fn standard_mode_turns_the_vim_keys_off_and_the_arrows_do_the_work() {
        let (vim, std) = (Keymap::default(), standard());
        let (yak, doc, shed) = (Some(Yak), Some(Doc), Some(Shed));
        let off = ['h', 'j', 'k', 'l', 'd', 'u', 'g', 'G'];
        for (ctx, kind) in [
            (Ctx::List, yak),
            (Ctx::List, doc),
            (Ctx::List, shed),
            (Ctx::Detail, yak),
            (Ctx::Viewer, None),
            (Ctx::Image, None),
        ] {
            for ch in off {
                let k = plain(KeyCode::Char(ch));
                assert_eq!(std.lookup(ctx, kind, &k), None, "{ctx:?} {kind:?} {ch}");
            }
            for ch in ['n', 'p'] {
                let k = ev(KeyCode::Char(ch), KeyModifiers::CONTROL);
                assert_eq!(std.lookup(ctx, kind, &k), None, "{ctx:?} Ctrl-{ch}");
            }
        }
        // In vim they are on, and the arrows are on in both.
        assert_eq!(
            vim.lookup(Ctx::List, yak, &plain(KeyCode::Char('j'))),
            Some(MoveDown)
        );
        for (code, cmd) in [
            (KeyCode::Down, MoveDown),
            (KeyCode::Up, MoveUp),
            (KeyCode::Home, MoveTop),
            (KeyCode::End, MoveBottom),
            (KeyCode::PageDown, MovePageDown),
            (KeyCode::Left, TreeLeft),
            (KeyCode::Right, TreeRight),
        ] {
            assert_eq!(
                std.lookup(Ctx::List, yak, &plain(code)),
                Some(cmd),
                "{code:?}"
            );
            assert_eq!(
                vim.lookup(Ctx::List, yak, &plain(code)),
                Some(cmd),
                "{code:?}"
            );
        }
        for (ctx, kind) in [(Ctx::Detail, yak), (Ctx::Viewer, None)] {
            for (code, cmd) in [
                (KeyCode::Home, MoveTop),
                (KeyCode::End, MoveBottom),
                (KeyCode::PageDown, MovePageDown),
                (KeyCode::Left, PaneScrollLeft),
                (KeyCode::Right, PaneScrollRight),
            ] {
                assert_eq!(
                    std.lookup(ctx, kind, &plain(code)),
                    Some(cmd),
                    "{ctx:?} {code:?}"
                );
            }
        }
        // Half a page and the blocks have a key that is in both families.
        let ctrl = |ch| ev(KeyCode::Char(ch), KeyModifiers::CONTROL);
        for km in [&vim, &std] {
            assert_eq!(km.lookup(Ctx::List, yak, &ctrl('d')), Some(MoveHalfDown));
            assert_eq!(km.lookup(Ctx::Detail, yak, &ctrl('u')), Some(MoveHalfUp));
        }
    }

    #[test]
    fn find_and_block_keys_in_each_family() {
        let (vim, std) = (Keymap::default(), standard());
        let (yak, f3) = (Some(Yak), KeyCode::F(3));
        let ctrl_f = ev(KeyCode::Char('f'), KeyModifiers::CONTROL);
        // Ctrl-F is find in standard mode only, in every context that finds;
        // it is never the `f` filter drawer.
        for (ctx, kind, cmd) in [
            (Ctx::List, yak, SearchOpen),
            (Ctx::List, Some(Doc), SearchOpen),
            (Ctx::List, Some(Shed), SearchOpen),
            (Ctx::Detail, yak, FindOpen),
            (Ctx::Viewer, None, FindOpen),
        ] {
            assert_eq!(
                std.lookup(ctx, kind, &ctrl_f),
                Some(cmd),
                "{ctx:?} {kind:?}"
            );
            assert_eq!(vim.lookup(ctx, kind, &ctrl_f), None, "{ctx:?} {kind:?}");
        }
        // F3 and Shift-F3 step the matches in both families.
        for km in [&vim, &std] {
            for (ctx, kind) in [
                (Ctx::List, yak),
                (Ctx::List, Some(Doc)),
                (Ctx::Detail, yak),
                (Ctx::Viewer, None),
            ] {
                assert_eq!(km.lookup(ctx, kind, &plain(f3)), Some(FindNext), "{ctx:?}");
                assert_eq!(
                    km.lookup(ctx, kind, &ev(f3, KeyModifiers::SHIFT)),
                    Some(FindPrev),
                    "{ctx:?}"
                );
            }
            // Ctrl-↓ / Ctrl-↑ are the content blocks, beside Ctrl-N / Ctrl-P.
            assert_eq!(
                km.lookup(Ctx::Detail, yak, &ev(KeyCode::Down, KeyModifiers::CONTROL)),
                Some(BlockNext)
            );
            assert_eq!(
                km.lookup(Ctx::Detail, yak, &ev(KeyCode::Up, KeyModifiers::CONTROL)),
                Some(BlockPrev)
            );
        }
        // Ctrl never rides along on a plain key: Ctrl-↓ is not ↓.
        assert_eq!(
            vim.lookup(Ctx::List, yak, &ev(KeyCode::Down, KeyModifiers::CONTROL)),
            None
        );
    }

    /// `{` and `}` change views from the list, the pane and both viewers.
    #[test]
    fn braces_change_views_everywhere() {
        for km in [Keymap::default(), standard()] {
            for (ctx, kind) in [
                (Ctx::List, Some(Yak)),
                (Ctx::List, Some(Doc)),
                (Ctx::List, Some(Shed)),
                (Ctx::Detail, Some(Yak)),
                (Ctx::Detail, Some(Shed)),
                (Ctx::Viewer, None),
                (Ctx::Image, None),
            ] {
                assert_eq!(
                    km.lookup(ctx, kind, &plain(KeyCode::Char('}'))),
                    Some(ViewNext),
                    "{ctx:?} {kind:?}"
                );
                assert_eq!(
                    km.lookup(ctx, kind, &plain(KeyCode::Char('{'))),
                    Some(ViewPrev),
                    "{ctx:?} {kind:?}"
                );
            }
        }
    }

    /// `h` and `l` never move between the list and the pane: in the pane and
    /// the viewer they only scroll, and in the list `l` walks the tree.
    #[test]
    fn h_and_l_never_open_or_close_the_pane() {
        for ch in ['h', 'l'] {
            for (ctx, kind) in [
                (Ctx::List, Some(Yak)),
                (Ctx::List, Some(Shed)),
                (Ctx::Detail, Some(Yak)),
                (Ctx::Viewer, None),
            ] {
                let cmd = lookup(ctx, kind, &plain(KeyCode::Char(ch)));
                assert!(
                    !matches!(cmd, Some(PaneOpen | PaneEnter | PanePeel | PaneClose)),
                    "{ctx:?} {ch}: {cmd:?}"
                );
            }
        }
        assert_eq!(
            lookup(Ctx::List, Some(Yak), &plain(KeyCode::Char('l'))),
            Some(TreeRight)
        );
        assert_eq!(
            lookup(Ctx::List, Some(Doc), &plain(KeyCode::Char('l'))),
            None
        );
        assert_eq!(lookup(Ctx::List, Some(Doc), &plain(KeyCode::Right)), None);
        for kind in [Some(Yak), Some(Doc), Some(Shed)] {
            assert_eq!(
                lookup(Ctx::List, kind, &plain(KeyCode::Enter)),
                Some(PaneOpen)
            );
        }
        // Esc leaves the pane and the viewer.
        assert_eq!(
            lookup(Ctx::Detail, Some(Yak), &plain(KeyCode::Esc)),
            Some(PanePeel)
        );
        assert_eq!(
            lookup(Ctx::Viewer, None, &plain(KeyCode::Esc)),
            Some(PanePeel)
        );
    }

    #[test]
    fn the_popup_is_grouped_by_what_a_command_is_for() {
        let km = Keymap::default();
        let titles: Vec<&str> = km
            .popup_entries(Ctx::List, Some(Yak))
            .iter()
            .map(|(t, _)| *t)
            .collect();
        assert_eq!(
            titles,
            [
                "Move",
                "Open and follow",
                "Find",
                "Change the yak",
                "View and filter",
                "Farm",
                "App"
            ]
        );
        for &ctx in Ctx::ALL {
            for kind in [None, Some(Yak), Some(Doc), Some(Shed)] {
                for (title, entries) in km.popup_entries(ctx, kind) {
                    for (_, name) in entries {
                        let cmd = Cmd::from_name(name).unwrap();
                        assert_eq!(cmd.group().title(), title, "{name}");
                    }
                }
            }
        }
        // The grouped lines carry each group's title once.
        let lines = text(&popup_lines(
            &km.popup_entries(Ctx::List, Some(Yak)),
            200,
            40,
            POPUP_MORE,
        ));
        for t in titles {
            assert!(lines.contains(t), "{t}:\n{lines}");
        }
    }

    #[test]
    fn the_standard_hint_names_the_arrows() {
        let hint = super::hint_for(&standard(), Ctx::List, Some(Yak), false);
        assert!(
            hint.starts_with("Tab:view  \u{2191}/\u{2193}:move  Enter:detail"),
            "{hint}"
        );
        assert!(!hint.contains("j/k"), "{hint}");
        let detail = super::hint_for(&standard(), Ctx::Detail, Some(Yak), false);
        assert!(
            detail.starts_with("Esc:list  \u{2191}/\u{2193}:move"),
            "{detail}"
        );
    }

    /// The editors' hand tables: both families list the same kinds of key,
    /// and every key they stand for parses (so a listed key can be matched).
    #[test]
    fn the_editor_tables_cover_both_families_and_every_key_parses() {
        use edtui::EditorMode::{Insert, Normal};
        for family in [Family::Vim, Family::Standard] {
            for (mode, single_line) in [
                (Normal, false),
                (Insert, false),
                (Normal, true),
                (Insert, true),
            ] {
                let at = EditorAt { mode, single_line };
                let groups = editor_groups(family, at);
                let titles: Vec<&str> = groups.iter().map(|(t, _)| *t).collect();
                for want in ["Move", "Save and leave"] {
                    assert!(titles.contains(&want), "{family:?} {at:?}: {titles:?}");
                }
                if !single_line {
                    assert!(titles.contains(&"Change the text"), "{family:?} {at:?}");
                }
                assert!(groups.iter().all(|(_, e)| !e.is_empty()));
            }
        }
        for table in [
            editor::VIM_NORMAL,
            editor::VIM_INSERT,
            editor::STANDARD,
            editor::LINE,
            editor::SAVE_VIM_MULTI,
            editor::SAVE_VIM_LINE,
            editor::SAVE_STANDARD_MULTI,
            editor::SAVE_STANDARD_LINE,
        ] {
            for (_, rows) in table {
                for r in *rows {
                    for p in r.press {
                        assert!(Key::parse(p).is_some(), "{p:?} in {:?}", r.show);
                    }
                }
            }
        }
    }
}

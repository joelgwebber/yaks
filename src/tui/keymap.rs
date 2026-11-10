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
//! overrides applied ([`overrides`]); every read of a key goes through it.
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
//! to only one family ([`VIM_ONLY`], [`STANDARD_ONLY`]) is off in the other:
//! the table lists every key, and [`Keymap::keys`] filters them, so the
//! lookup, the `?` popup, the reference and the hint all agree. The keys both
//! families share (arrows, `PgUp`/`PgDn`, `Home`/`End`, `Enter`, `Esc`, the
//! Shift tier) work in both; a test keeps every command reachable in each.
//!
//! The key rules, with the tests that enforce them (also in `AGENTS.md`):
//!
//! - **Case is a tier.** Lowercase looks and moves; Shift changes an existing
//!   yak; Ctrl is the app and the farm. [`Cmd::mutates`] says which commands
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
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

mod editor;
mod overrides;

pub(crate) use editor::{EditorAt, editor_groups, editor_lists, editor_title, rows_groups};

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
            pub(crate) const ALL: &'static [Cmd] = &[$(Cmd::$variant,)*];

            /// The stable dotted name: what a `keys:` override in the user
            /// config names, and what the `?` popup prints.
            pub(crate) fn name(self) -> &'static str {
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
    LinkOpenExternal => "link.open_external",
    FileOpenExternal => "file.open_external",
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

impl Cmd {
    /// The command a dotted name (`view.next`) stands for.
    pub(crate) fn from_name(name: &str) -> Option<Cmd> {
        Cmd::ALL.iter().copied().find(|c| c.name() == name)
    }

    /// The `?` popup's group for it. Exhaustive, so a new command must pick one.
    pub(crate) fn group(self) -> Group {
        match self {
            MoveDown | MoveUp | MoveTop | MoveBottom | MoveHalfDown | MoveHalfUp | MovePageDown
            | MovePageUp | ItemNext | ItemPrev | TreeToggle | TreeLeft | TreeRight
            | PaneScrollLeft | PaneScrollRight | LinkNext | LinkPrev | BlockNext | BlockPrev
            | ImagePanLeft | ImagePanRight | ImagePanUp | ImagePanDown | SelectVisual
            | SelectExtendDown | SelectExtendUp | SelectYank | RowCopy => Group::Move,
            PaneOpen | PaneEnter | PanePeel | PaneClose | NavBack | NavForward
            | LinkOpenExternal | FileOpenExternal | PeekToggle => Group::Open,
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
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn mutates(self) -> bool {
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
            | NavBack | NavForward | LinkNext | LinkPrev | LinkOpenExternal | FileOpenExternal
            | ImageZoomIn | ImageZoomOut | ImageZoomActual | ImageFitWidth | ImageFit
            | ImagePanLeft | ImagePanRight | ImagePanUp | ImagePanDown | FindOpen | FindNext
            | FindPrev | SelectVisual | SelectExtendDown | SelectExtendUp | SelectYank
            | BlockNext | BlockPrev | PeekToggle | YakBackrefs => false,
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
    /// The popup's order.
    pub(crate) const ALL: [Group; 7] = [
        Group::Move,
        Group::Open,
        Group::Find,
        Group::Change,
        Group::View,
        Group::Farm,
        Group::App,
    ];

    pub(crate) fn title(self) -> &'static str {
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

/// The key family (yaks-7c6e): `keys: vim` or `keys: standard` in the user's
/// config. It decides which of the table's keys are on.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) enum Family {
    /// `hjkl`, `d`/`u`, `g`/`G`, `Ctrl-N`/`Ctrl-P`; edtui's vim mode in editors.
    #[default]
    Vim,
    /// Arrows, `PgUp`/`PgDn`, `Home`/`End`, `Ctrl-F` find; edtui's emacs mode.
    Standard,
}

/// The keys only the vim family has: off outside the editor in standard.
const VIM_ONLY: &[Key] = &[
    c('h'),
    c('j'),
    c('k'),
    c('l'),
    c('d'),
    c('u'),
    c('g'),
    c('G'),
    ctrl('n'),
    ctrl('p'),
];

/// The keys only the standard family has: off in vim.
const STANDARD_ONLY: &[Key] = &[ctrl('f')];

impl Family {
    /// The family a config value names (`vim`, `standard`).
    pub(crate) fn from_name(name: &str) -> Option<Family> {
        match name.trim() {
            "vim" => Some(Family::Vim),
            "standard" => Some(Family::Standard),
            _ => None,
        }
    }

    /// The legacy `vim_mode: true|false` setting as a family.
    pub(crate) fn from_vim_mode(vim: bool) -> Family {
        if vim { Family::Vim } else { Family::Standard }
    }

    pub(crate) fn is_vim(self) -> bool {
        self == Family::Vim
    }

    /// Whether the table's key `key` is on in this family.
    fn allows(self, key: &Key) -> bool {
        match self {
            Family::Vim => !STANDARD_ONLY.contains(key),
            Family::Standard => !VIM_ONLY.contains(key),
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
}

impl Ctx {
    const ALL: [Ctx; 4] = [Ctx::List, Ctx::Detail, Ctx::Viewer, Ctx::Image];

    /// What a message calls the surface.
    pub(crate) fn title(self) -> &'static str {
        match self {
            Ctx::List => "list",
            Ctx::Detail => "detail pane",
            Ctx::Viewer => "text viewer",
            Ctx::Image => "image viewer",
        }
    }
}

/// A section of the `?` overlay: the bindings of `ctx` whose kind is in
/// `kinds` (`None`: bound for any kind of row).
struct Section {
    ctx: Ctx,
    kinds: &'static [Option<RowKind>],
    title: &'static str,
}

/// The `?` overlay's sections, in order. Every binding is in exactly one.
const SECTIONS: [Section; 7] = [
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
        ctx: Ctx::List,
        kinds: &[Some(RowKind::Doc)],
        title: "Document rows (the Doks view: the farm's doks and sources)",
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

impl Section {
    fn bindings(&self) -> impl Iterator<Item = &'static Binding> + '_ {
        BINDINGS
            .iter()
            .filter(|b| b.ctx == self.ctx && self.kinds.contains(&b.kind))
    }
}

/// A key: a code and the modifiers that must be held with it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Key {
    code: KeyCode,
    mods: KeyModifiers,
}

impl Key {
    fn matches(&self, k: &KeyEvent) -> bool {
        // Ctrl never rides along on a key that does not name it: `Ctrl-N` is
        // not `n`, so a key a family turns off (Ctrl-N, Ctrl-P) goes dead
        // rather than falling through to its plain letter. (Ctrl with Alt is
        // AltGr on some layouts, which types `[` and `{`: that one is plain.)
        let ctrl_only =
            k.modifiers.contains(KeyModifiers::CONTROL) && !k.modifiers.contains(KeyModifiers::ALT);
        if ctrl_only && !self.mods.contains(KeyModifiers::CONTROL) {
            return false;
        }
        self.code == k.code && k.modifiers.contains(self.mods)
    }

    /// How the `?` overlay writes it: `j`, `↓`, `Space`, `Ctrl-N`, `S-Tab`.
    fn label(&self) -> String {
        let ctrl = self.mods.contains(KeyModifiers::CONTROL);
        let alt = self.mods.contains(KeyModifiers::ALT);
        let name = match self.code {
            KeyCode::Char(' ') => "Space".to_string(),
            KeyCode::Char(c) if ctrl || alt => c.to_ascii_uppercase().to_string(),
            KeyCode::Char(c) => c.to_string(),
            KeyCode::Up => "↑".into(),
            KeyCode::Down => "↓".into(),
            KeyCode::Left => "←".into(),
            KeyCode::Right => "→".into(),
            KeyCode::Tab => "Tab".into(),
            KeyCode::BackTab => "S-Tab".into(),
            KeyCode::Enter => "Enter".into(),
            KeyCode::Esc => "Esc".into(),
            KeyCode::PageUp => "PgUp".into(),
            KeyCode::PageDown => "PgDn".into(),
            KeyCode::Home => "Home".into(),
            KeyCode::End => "End".into(),
            KeyCode::F(n) => format!("F{n}"),
            other => format!("{other:?}"),
        };
        let mut label = String::new();
        if ctrl {
            label.push_str("Ctrl-");
        }
        if alt {
            label.push_str("Alt-");
        }
        if self.mods.contains(KeyModifiers::SHIFT) {
            label.push_str("Shift-");
        }
        label + &name
    }

    /// The inverse of [`Key::label`] (what a `keys:` override writes), with
    /// `Up`/`Down`/`Left`/`Right` as ASCII spellings of the arrows. A `Ctrl-`
    /// or `Alt-` letter is read as the lower-case key, as the table spells it.
    pub(crate) fn parse(s: &str) -> Option<Key> {
        let mut rest = s;
        let mut mods = KeyModifiers::NONE;
        loop {
            let (r, m) = if let Some(r) = rest.strip_prefix("Ctrl-") {
                (r, KeyModifiers::CONTROL)
            } else if let Some(r) = rest.strip_prefix("Shift-") {
                (r, KeyModifiers::SHIFT)
            } else if let Some(r) = rest.strip_prefix("Alt-") {
                (r, KeyModifiers::ALT)
            } else {
                break;
            };
            if r.is_empty() {
                break;
            }
            mods |= m;
            rest = r;
        }
        let mut chars = rest.chars();
        let code = match rest {
            "Space" => KeyCode::Char(' '),
            "Tab" => KeyCode::Tab,
            "S-Tab" => KeyCode::BackTab,
            "Enter" => KeyCode::Enter,
            "Esc" => KeyCode::Esc,
            "\u{2191}" | "Up" => KeyCode::Up,
            "\u{2193}" | "Down" => KeyCode::Down,
            "\u{2190}" | "Left" => KeyCode::Left,
            "\u{2192}" | "Right" => KeyCode::Right,
            "PgUp" => KeyCode::PageUp,
            "PgDn" => KeyCode::PageDown,
            "Home" => KeyCode::Home,
            "End" => KeyCode::End,
            "Backspace" => KeyCode::Backspace,
            "Delete" => KeyCode::Delete,
            "Insert" => KeyCode::Insert,
            _ => match (chars.next(), chars.next()) {
                (Some(ch), None) if mods.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) => {
                    KeyCode::Char(ch.to_ascii_lowercase())
                }
                (Some(ch), None) => KeyCode::Char(ch),
                _ => {
                    // `F3`, or `F(3)` as older configs wrote it.
                    let n = match rest.strip_prefix("F(") {
                        Some(r) => r.strip_suffix(')')?,
                        None => rest.strip_prefix('F')?,
                    };
                    KeyCode::F(n.parse().ok()?)
                }
            },
        };
        Some(Key { code, mods })
    }
}

const fn c(ch: char) -> Key {
    Key {
        code: KeyCode::Char(ch),
        mods: KeyModifiers::NONE,
    }
}

const fn k(code: KeyCode) -> Key {
    Key {
        code,
        mods: KeyModifiers::NONE,
    }
}

const fn ctrl(ch: char) -> Key {
    Key {
        code: KeyCode::Char(ch),
        mods: KeyModifiers::CONTROL,
    }
}

const fn ctrl_k(code: KeyCode) -> Key {
    Key {
        code,
        mods: KeyModifiers::CONTROL,
    }
}

const fn shift(code: KeyCode) -> Key {
    Key {
        code,
        mods: KeyModifiers::SHIFT,
    }
}

/// What a binding adds to the bottom hint bar.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Hint {
    /// Nothing.
    None,
    /// This label (`Tab:view`), in the table's order.
    Text(&'static str),
    /// A label that differs by family: the vim one (`j/k:move`), then the
    /// standard one (`↑/↓:move`).
    Family(&'static str, &'static str),
    /// The filter drawer's label, which depends on whether a filter is
    /// active (`f:filter  /:search`, or `f:filter  Esc:clear`).
    Filter,
}

/// One row of the table: in `ctx`, any of `keys` runs `cmd`. A binding in the
/// list may name the [`RowKind`] it is for (a document row's `E` edits a dok,
/// a yak's edits the yak); one that names none is for every kind of row.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Binding {
    pub(crate) ctx: Ctx,
    pub(crate) kind: Option<RowKind>,
    pub(crate) keys: &'static [Key],
    pub(crate) cmd: Cmd,
    /// The `?` overlay's line for it; empty: not listed.
    pub(crate) help: &'static str,
    pub(crate) hint: Hint,
}

const fn bind(ctx: Ctx, keys: &'static [Key], cmd: Cmd, help: &'static str, hint: Hint) -> Binding {
    Binding {
        ctx,
        kind: None,
        keys,
        cmd,
        help,
        hint,
    }
}

/// A list binding for one kind of row.
const fn bind_row(
    kind: RowKind,
    keys: &'static [Key],
    cmd: Cmd,
    help: &'static str,
    hint: Hint,
) -> Binding {
    Binding {
        kind: Some(kind),
        ..bind(Ctx::List, keys, cmd, help, hint)
    }
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
    Binding {
        kind: Some(kind),
        ..bind(Ctx::Detail, keys, cmd, help, hint)
    }
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
use Hint::None as NoHint;
use KeyCode::{BackTab, Down, End, Enter, Esc, F, Home, Left, PageDown, PageUp, Right, Tab, Up};
use tree::RowKind::{self, Doc, Shed, Yak};

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
    bind_row(Yak, &[c('T')], YakType, "Type", NoHint),
    bind_row(Yak, &[c('L')], YakTags, "Tags", NoHint),
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
    bind_row(Yak, &[c('/'), ctrl('f')], SearchOpen, "Search every view; Esc clears", NoHint),
    bind(List, &[c('n'), k(F(3))], FindNext, "Search again: the pane's last find becomes the query (no query set)", NoHint),
    bind(List, &[c('N'), shift(F(3))], FindPrev, "Search again, as n", NoHint),
    bind(List, &[c('#')], NavGoto, "Go to any yak by id / title", NoHint),
    bind(List, &[c('i')], NavForward, "History forward: a jump you went back from", NoHint),
    bind(List, &[c('o')], NavBack, "History back: where the last jump left (a link, a view switch, #), position restored", NoHint),
    bind_row(Yak, &[c('B')], YakBackrefs, "Find references: yaks that point at this one", NoHint),
    bind_row(Yak, &[c('f')], FilterDrawer, "Filter drawer", Hint::Filter),
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
    bind_pane(Yak, &[c('S')], YakState, "State (S → n shears, with an optional comment)", hint("S:state")),
    bind_pane(Yak, &[c('m')], YakMark, "Mark / unmark the yak (multi-select)", NoHint),
    bind_pane(Yak, &[c('P')], YakPriority, "Priority", NoHint),
    bind_pane(Yak, &[c('T')], YakType, "Type", NoHint),
    bind_pane(Yak, &[c('L')], YakTags, "Tags", NoHint),
    bind_pane(Yak, &[c('X')], YakSlaughter, "Slaughter (confirm; offers whole family)", NoHint),
    bind_pane(Yak, &[c('R')], YakReparent, "Reparent", NoHint),
    bind_pane(Yak, &[c('H')], YakHerd, "Move to another herd (multi-herd farm)", NoHint),
    bind_pane(Yak, &[c('c')], YakNew, "New root yak", NoHint),
    bind_pane(Yak, &[c('C')], YakNewChild, "New child yak", NoHint),
    bind_pane(Yak, &[c('a')], YakAnswer, "Answer (awaiting a human) / request agent work (else): needs agent", NoHint),
    bind_pane(Yak, &[c('Q')], YakAsk, "Ask a question: needs human", NoHint),
    bind_pane(Yak, &[c('Z')], YakSnooze, "Snooze: move the due date (+1d, +1w, a date)", NoHint),
    bind_pane(Yak, &[c('A')], YakAttach, "Attach artifact", NoHint),
    bind_pane(Yak, &[c('O')], LinkOpenExternal, "Open the artifact / URL / source link on the line (a text file in the viewer)", NoHint),
    bind_pane(Yak, &[c('r')], AttachmentRename, "Rename the attachment at the cursor", NoHint),
    bind_pane(Yak, &[c('*')], YakStar, "Star / unstar (Starred view)", NoHint),
    bind_pane(Yak, &[c('f')], FilterDrawer, "Filter drawer", Hint::Filter),
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
    bind(Viewer, &[c('p')], PeekToggle, "Show / hide the peek pane under the dok (the same flag as the yak pane's)", NoHint),
    bind(Viewer, &[c('E')], DocEdit, "Edit the file in place (Ctrl-S saves; a past version or a source is read-only)", NoHint),
    bind(Viewer, &[c('O')], FileOpenExternal, "Open the file externally", NoHint),
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
    bind(Image, &[c('O')], FileOpenExternal, "Open the file externally", NoHint),
    bind(Image, &[c('#')], NavGoto, "Go to any yak by id / title (closes the image)", NoHint),
    bind(Image, &[c('W')], FarmCommit, "Commit the farm (yaks commit)", NoHint),
    bind(Image, &[c('U')], FarmSync, "Sync the farm: commit, pull, push (yaks sync)", NoHint),
    bind(Image, &[c('}')], ViewNext, VIEW_PANE_NEXT, NoHint),
    bind(Image, &[c('{')], ViewPrev, VIEW_PANE_PREV, NoHint),
    bind(Image, &[c('?'), k(F(1))], AppHelp, HELP_LINE, NoHint),

    // -- the list: document rows (the Doks view). Its movement, view and farm
    // -- keys are the list's; `` ` ``, `?` and `q` repeat theirs only so that the
    // -- bottom hint is the Doks's own, not the yak list's.
    bind_row(Doc, &[k(Enter)], PaneOpen, "Open in the viewer: frontmatter above the body; a source with its extracted text", hint("Enter:open")),
    bind_row(Doc, &[c('E')], DokEdit, "Edit the dok: frontmatter (YAML) and body; Ctrl-S saves", hint("E:edit")),
    bind_row(Doc, &[c('c')], DokNew, "New dok: type its path (doks/... or <herd>/doks/...), then its body", hint("c:new dok")),
    bind_row(Doc, &[c('/'), ctrl('f')], SearchOpen, "Search every view; Esc clears", hint("/:search")),
    bind_row(Doc, &[c('y')], RowCopy, "Copy the path", hint("y:copy path")),
    bind_row(Doc, &[ctrl('r')], ListRefresh, "Reread the doks (also on any farm change)", NoHint),
    bind_row(Doc, &[c('`')], ViewPick, "", NoHint),
    bind_row(Doc, &[c('?'), k(F(1))], AppHelp, "", hint("?:help")),
    bind_row(Doc, &[c('q'), ctrl('c')], AppQuit, "", hint("q:quit")),

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

/// The `?` line of the help command: the popup first, the reference on a second press.
const HELP_LINE: &str = "Keys of this context in a popup; ? again opens this full reference";

/// The keys in effect: the table's, filtered by the key [`Family`], with the
/// user's `keys:` overrides applied (see [`overrides`]). An override replaces
/// a command's keys in every context it is bound in (and is the user's word:
/// the family does not filter it). Everything that reads keys (the lookup, the
/// `?` popup and reference, the bottom hint) goes through it, so none can show
/// a key the lookup does not take.
#[derive(Clone, Default, Debug)]
pub(crate) struct Keymap {
    pub(crate) family: Family,
    overrides: HashMap<Cmd, Vec<Key>>,
}

/// The `?` popup's content: titled groups of (keys, name) entries.
pub(crate) type PopupGroups = Vec<(&'static str, Vec<(String, &'static str)>)>;

impl Keymap {
    /// The keys of `b` in effect.
    fn keys<'a>(&'a self, b: &'a Binding) -> impl Iterator<Item = &'a Key> + 'a {
        let (keys, own) = match self.overrides.get(&b.cmd) {
            Some(v) => (v.as_slice(), true),
            None => (b.keys, false),
        };
        keys.iter().filter(move |k| own || self.family.allows(k))
    }

    /// `b`'s keys as `sep` joins their labels.
    fn labels(&self, b: &Binding, sep: &str) -> String {
        self.keys(b).map(Key::label).collect::<Vec<_>>().join(sep)
    }

    /// The command `k` runs in `ctx` on a row of `kind` (`None` outside the
    /// list): a binding that names modifiers first (Ctrl-C before `c`), else a
    /// plain one, ignoring any modifiers the terminal adds (an upper-case
    /// letter arrives with Shift) but Ctrl ([`Key::matches`]). At each step a
    /// binding for the row's kind beats one for any kind.
    pub(crate) fn lookup(&self, ctx: Ctx, kind: Option<RowKind>, k: &KeyEvent) -> Option<Cmd> {
        let find = |modified: bool, kind: Option<RowKind>| {
            BINDINGS
                .iter()
                .filter(|b| b.ctx == ctx && b.kind == kind)
                .find(|b| {
                    self.keys(b)
                        .any(|key| key.mods.is_empty() != modified && key.matches(k))
                })
                .map(|b| b.cmd)
        };
        [true, false].into_iter().find_map(|modified| {
            kind.and_then(|kind| find(modified, Some(kind)))
                .or_else(|| find(modified, None))
        })
    }

    /// The bindings that act in `ctx` on a row of `kind`, in table order: those
    /// for this kind, and those for any kind except where one for this kind
    /// takes their keys (a document row's `q` stands in for the list's).
    fn visible(&self, ctx: Ctx, kind: Option<RowKind>) -> impl Iterator<Item = &'static Binding> {
        BINDINGS.iter().filter(move |b| {
            b.ctx == ctx
                && match (b.kind, kind) {
                    (None, None) => true,
                    (None, Some(kind)) => !BINDINGS.iter().any(|o| {
                        o.ctx == ctx
                            && o.kind == Some(kind)
                            && self.keys(o).any(|k| self.keys(b).any(|x| x == k))
                    }),
                    (b_kind, kind) => b_kind == kind,
                }
        })
    }

    /// The first key of `cmd` in `ctx`, as a hint writes it (`f`).
    pub(crate) fn first_label(&self, ctx: Ctx, cmd: Cmd) -> String {
        BINDINGS
            .iter()
            .find(|b| b.ctx == ctx && b.cmd == cmd)
            .and_then(|b| self.keys(b).next())
            .map_or_else(|| "?".into(), Key::label)
    }

    /// The bottom bar's key hints for `ctx` on a row of `kind`: each hinted
    /// binding's label, in table order, two spaces apart. A command whose keys
    /// are overridden shows them in place of the label's own (`Tab:view`
    /// becomes `Ctrl-N:view`). `filter_active` is whether a filter or query is
    /// set, which changes the drawer's label.
    fn hint_for(&self, ctx: Ctx, kind: Option<RowKind>, filter_active: bool) -> String {
        self.visible(ctx, kind)
            .filter_map(|b| {
                let label = match b.hint {
                    Hint::None => return None,
                    Hint::Text(t) => t,
                    Hint::Family(vim, standard) => {
                        if self.family.is_vim() {
                            vim
                        } else {
                            standard
                        }
                    }
                    Hint::Filter => {
                        let (find, clear) = match ctx {
                            Ctx::List => (SearchOpen, ViewRevert),
                            _ => (FindOpen, PanePeel),
                        };
                        let then = if filter_active {
                            format!("{}:clear", self.first_label(ctx, clear))
                        } else {
                            format!("{}:search", self.first_label(ctx, find))
                        };
                        return Some(format!(
                            "{}:filter  {then}",
                            self.first_label(ctx, FilterDrawer)
                        ));
                    }
                };
                if !self.overrides.contains_key(&b.cmd) {
                    return Some(label.to_string());
                }
                let what = label.split_once(':').map_or(label, |(_, what)| what);
                let keys = self.labels(b, "/");
                (!keys.is_empty()).then(|| format!("{keys}:{what}"))
            })
            .collect::<Vec<_>>()
            .join("  ")
    }

    /// The `?` popup's groups for `ctx` on a row of `kind`: each binding's
    /// keys and its command's name (the name an override uses), under its
    /// command's [`Group`], in the popup's order; a group with nothing in it
    /// is left out.
    pub(crate) fn popup_entries(&self, ctx: Ctx, kind: Option<RowKind>) -> PopupGroups {
        Group::ALL
            .into_iter()
            .filter_map(|g| {
                let entries: Vec<_> = self
                    .visible(ctx, kind)
                    .filter(|b| b.cmd.group() == g && self.keys(b).next().is_some())
                    .map(|b| (self.labels(b, "/"), b.cmd.name()))
                    .collect();
                (!entries.is_empty()).then_some((g.title(), entries))
            })
            .collect()
    }
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

/// One line of the popup before it is laid out in columns.
enum PopupRow<'a> {
    Head(&'static str),
    Entry(&'a str, &'static str),
}

/// The `?` popup's rows: `groups` (a title, then its keys and names) laid out
/// in columns of at most `max_rows` rows, as many as fit in `width`, filling
/// down each column in turn (a title never ends a column, away from its
/// entries). Entries that do not fit are counted on a last line.
pub(crate) fn popup_lines(
    groups: &PopupGroups,
    width: usize,
    max_rows: usize,
) -> Vec<Line<'static>> {
    const GAP: usize = 2;
    let len = |s: &str| s.chars().count();
    let items: Vec<PopupRow> = groups
        .iter()
        .flat_map(|(title, entries)| {
            std::iter::once(PopupRow::Head(title))
                .chain(entries.iter().map(|(k, n)| PopupRow::Entry(k, n)))
        })
        .collect();
    let total_entries = items
        .iter()
        .filter(|r| matches!(r, PopupRow::Entry(..)))
        .count();
    let columns = |rows: usize| -> Vec<Vec<&PopupRow>> {
        let mut cols: Vec<Vec<&PopupRow>> = vec![Vec::new()];
        for it in &items {
            let at = cols.last().map_or(0, Vec::len);
            let orphan = matches!(it, PopupRow::Head(_)) && at + 1 >= rows;
            if (at >= rows || orphan) && at > 0 {
                cols.push(Vec::new());
            }
            if let Some(col) = cols.last_mut() {
                col.push(it);
            }
        }
        cols
    };
    // A column's key width and its whole width.
    let widths = |col: &[&PopupRow]| {
        let (mut kw, mut nw, mut hw) = (0, 0, 0);
        for r in col {
            match r {
                PopupRow::Head(t) => hw = hw.max(len(t)),
                PopupRow::Entry(k, n) => {
                    kw = kw.max(len(k));
                    nw = nw.max(len(n));
                }
            }
        }
        (kw, (kw + 1 + nw).max(hw))
    };
    let total = |rows: usize| {
        let cols: Vec<_> = columns(rows).iter().map(|c| widths(c).1).collect();
        cols.iter().sum::<usize>() + GAP * cols.len().saturating_sub(1)
    };
    if items.is_empty() || max_rows == 0 {
        return Vec::new();
    }
    let rows = (1..=max_rows)
        .find(|&r| total(r) <= width)
        .unwrap_or(max_rows);
    let mut cols: Vec<Vec<&PopupRow>> = Vec::new();
    let mut used = 0;
    for col in columns(rows) {
        let w = widths(&col).1 + if cols.is_empty() { 0 } else { GAP };
        if !cols.is_empty() && used + w > width {
            break;
        }
        used += w;
        cols.push(col);
    }
    let shown: usize = cols
        .iter()
        .flat_map(|c| c.iter())
        .filter(|r| matches!(r, PopupRow::Entry(..)))
        .count();
    let height = cols.iter().map(Vec::len).max().unwrap_or(0);
    let mut lines: Vec<Line<'static>> = (0..height)
        .map(|r| {
            let mut spans = Vec::new();
            for (i, col) in cols.iter().enumerate() {
                let (kw, w) = widths(col);
                if i > 0 {
                    spans.push(Span::raw(" ".repeat(GAP)));
                }
                match col.get(r) {
                    Some(PopupRow::Head(title)) => spans.push(Span::styled(
                        format!("{title:<w$}"),
                        Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    )),
                    Some(PopupRow::Entry(keys, name)) => {
                        spans.push(Span::styled(
                            format!("{keys:<kw$}"),
                            Style::new().fg(Color::Yellow),
                        ));
                        spans.push(Span::raw(format!(" {name:<0$}", w - kw - 1)));
                    }
                    None => spans.push(Span::raw(" ".repeat(w))),
                }
            }
            Line::from(spans)
        })
        .collect();
    if shown < total_entries {
        lines.push(Line::from(Span::styled(
            format!(
                "… {} more: the full reference has them all",
                total_entries - shown
            ),
            Style::new().fg(Color::DarkGray),
        )));
    }
    lines
}

pub(crate) fn help_hint(app: &App) -> String {
    let filter_active = app.filter.content_active() || app.query.is_some();
    app.keymap
        .hint_for(app.surface_ctx(), app.surface_kind(), filter_active)
}

/// The `?` overlay's rows that are not key bindings: the editors' own keys
/// (modal, so outside the table), what a file or link does, the farm chip.
/// Each section is a title and its (label, text) rows.
const EXTRAS: &[(&str, &[(&str, &str)])] = &[
    (
        "Editing (the editors and pickers are modal; their keys are not in the table)",
        &[
            ("S → n", "Shear: prompts an optional comment (Enter shears)"),
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
        "Files, links and images",
        &[
            (
                "(text file)",
                "Opens in a viewer (.md rendered); E there edits it in place (Ctrl-S saves), O opens externally",
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
                "(image)",
                "Shown in the pane (png/jpg/gif/webp/svg); O opens externally, Esc/q closes",
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
    fn section(name: &str) -> Line<'static> {
        Line::from(Span::styled(
            name.to_string(),
            Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ))
    }
    fn entry(key: &str, desc: &str) -> Line<'static> {
        Line::from(vec![
            Span::styled(format!("  {key:<16}"), Style::new().fg(Color::Yellow)),
            Span::raw(desc.to_string()),
        ])
    }
    let mut lines = Vec::new();
    for sect in &SECTIONS {
        lines.push(section(sect.title));
        for b in sect.bindings().filter(|b| !b.help.is_empty()) {
            lines.push(entry(&km.labels(b, " / "), b.help));
        }
        lines.push(Line::from(String::new()));
    }
    for (title, rows) in EXTRAS {
        lines.push(section(title));
        lines.extend(rows.iter().map(|(key, desc)| entry(key, desc)));
        lines.push(Line::from(String::new()));
    }
    lines.pop();
    lines
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
        let global = matches!(
            cmd,
            AppQuit | AppHelp | NavGoto | FarmCommit | FarmSync | ViewNext | ViewPrev
        );
        match ctx {
            Ctx::Viewer if !global => return self.run_viewer_cmd(cmd),
            Ctx::Image if !global => return self.run_image_cmd(cmd),
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
            LinkOpenExternal => self.open_current_external(),
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
            PaneClose | DocEdit | FileOpenExternal | ImageZoomIn | ImageZoomOut
            | ImageZoomActual | ImageFitWidth | ImageFit | ImagePanLeft | ImagePanRight
            | ImagePanUp | ImagePanDown => {}
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
            Ctx::Viewer | Ctx::Image => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The table's own keys: no overrides.
    fn lookup(ctx: Ctx, kind: Option<RowKind>, k: &KeyEvent) -> Option<Cmd> {
        Keymap::default().lookup(ctx, kind, k)
    }

    fn hint_for(ctx: Ctx, kind: Option<RowKind>, filter_active: bool) -> String {
        Keymap::default().hint_for(ctx, kind, filter_active)
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

    #[test]
    fn no_two_bindings_in_a_context_and_kind_share_a_key() {
        let kinds = [
            None,
            Some(RowKind::Yak),
            Some(RowKind::Doc),
            Some(RowKind::Shed),
        ];
        for (ctx, kind) in Ctx::ALL.into_iter().flat_map(|c| kinds.map(|k| (c, k))) {
            let mut seen: Vec<(Key, Cmd)> = Vec::new();
            for b in BINDINGS.iter().filter(|b| b.ctx == ctx && b.kind == kind) {
                for key in b.keys {
                    if let Some((_, other)) = seen.iter().find(|(s, _)| s == key) {
                        panic!(
                            "{ctx:?} {kind:?}: {} is bound to {} and {}",
                            key.label(),
                            other.name(),
                            b.cmd.name()
                        );
                    }
                    seen.push((*key, b.cmd));
                }
            }
        }
    }

    #[test]
    fn every_command_is_bound_and_named_once() {
        for cmd in Cmd::ALL {
            assert!(
                BINDINGS.iter().any(|b| b.cmd == *cmd),
                "{} is bound nowhere",
                cmd.name()
            );
        }
        let mut names: Vec<&str> = Cmd::ALL.iter().map(|c| c.name()).collect();
        names.sort();
        let n = names.len();
        names.dedup();
        assert_eq!(names.len(), n, "two commands share a name");
        assert!(names.iter().all(|n| n.contains('.')), "{names:?}");
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
                "Esc:list  j/k:move  Tab:link  Enter:follow  p:peek  i/o:fwd/back  E:edit  M:comment  D:dep  S:state  f:filter  /:search  q:quit",
            ),
            (
                Ctx::List,
                Some(RowKind::Doc),
                "Tab:view  j/k:move  Enter:open  E:edit  c:new dok  /:search  y:copy path  ?:help  q:quit",
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
            "Esc:list  j/k:move  Tab:link  Enter:follow  p:peek  i/o:fwd/back  E:edit  M:comment  D:dep  S:state  f:filter  Esc:clear  q:quit"
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
        for ctx in Ctx::ALL {
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
        Keymap {
            family: Family::Standard,
            ..Keymap::default()
        }
    }

    fn ev(code: KeyCode, mods: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, mods)
    }

    fn plain(code: KeyCode) -> KeyEvent {
        ev(code, KeyModifiers::NONE)
    }

    /// Lowercase looks and moves; Shift changes an existing yak; Ctrl is the
    /// app and the farm. A command that writes (see [`Cmd::mutates`]) must not
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
        let lowercase = |k: &Key| {
            matches!(k.code, KeyCode::Char(ch) if !ch.is_uppercase())
                && !k
                    .mods
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::SHIFT)
        };
        for b in BINDINGS.iter().filter(|b| b.cmd.mutates()) {
            for key in b.keys.iter().filter(|k| lowercase(k)) {
                assert!(
                    exceptions.iter().any(|(cmd, _)| *cmd == b.cmd),
                    "{} ({:?}) is on the lowercase key {}: Shift changes things, lowercase only looks",
                    b.cmd.name(),
                    b.ctx,
                    key.label()
                );
            }
        }
        // An exception that no longer sits on a lowercase key is stale.
        for (cmd, why) in exceptions {
            assert!(
                BINDINGS
                    .iter()
                    .any(|b| b.cmd == cmd && b.keys.iter().any(lowercase)),
                "{} is no longer on a lowercase key: drop its exception ({why})",
                cmd.name()
            );
        }
    }

    /// Standard mode is the same commands on other keys: whatever a command
    /// binds in vim mode, it still binds with the vim-only keys off, and no
    /// key left on is one of them.
    #[test]
    fn every_command_stays_bound_in_standard_mode() {
        let (vim, std) = (Keymap::default(), standard());
        for b in BINDINGS {
            assert!(
                vim.keys(b).next().is_none() || std.keys(b).next().is_some(),
                "{} ({:?}, {:?}) has no key in standard mode (it has {})",
                b.cmd.name(),
                b.ctx,
                b.kind,
                vim.labels(b, "/")
            );
            for key in std.keys(b) {
                assert!(
                    !VIM_ONLY.contains(key),
                    "{} keeps the vim-only key {}",
                    b.cmd.name(),
                    key.label()
                );
            }
            for key in vim.keys(b) {
                assert!(!STANDARD_ONLY.contains(key), "{}", b.cmd.name());
            }
        }
        // And per context: every command a context lists in vim is listed in standard.
        for ctx in Ctx::ALL {
            for kind in [None, Some(Yak), Some(Doc), Some(Shed)] {
                let names = |km: &Keymap| -> Vec<&'static str> {
                    km.popup_entries(ctx, kind)
                        .into_iter()
                        .flat_map(|(_, e)| e.into_iter().map(|(_, n)| n))
                        .collect()
                };
                assert_eq!(names(&vim), names(&std), "{ctx:?} {kind:?}");
            }
        }
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
        for ctx in Ctx::ALL {
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
        ));
        for t in titles {
            assert!(lines.contains(t), "{t}:\n{lines}");
        }
    }

    #[test]
    fn the_standard_hint_names_the_arrows() {
        let hint = standard().hint_for(Ctx::List, Some(Yak), false);
        assert!(
            hint.starts_with("Tab:view  \u{2191}/\u{2193}:move  Enter:detail"),
            "{hint}"
        );
        assert!(!hint.contains("j/k"), "{hint}");
        let detail = standard().hint_for(Ctx::Detail, Some(Yak), false);
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

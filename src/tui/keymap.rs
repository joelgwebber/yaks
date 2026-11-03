//! The command table (yaks-fbb0, phase 2 of /wiki/design/tui-model.md#5-keys):
//! every key binding of the non-modal surfaces in one place. A [`Binding`]
//! names a [`Cmd`] (a stable dotted name), the [`Ctx`] it applies in, its
//! keys, a help line for the `?` overlay and a label for the bottom hint.
//!
//! Dispatch is "work out the context, [`lookup`], [`App::run_cmd`]". The
//! `?` overlay ([`help_content`]) and the bottom bar ([`help_hint`]) are
//! generated from the same table, so a key cannot be bound without being
//! listed, and the three cannot drift.
//!
//! A [`Cmd`] is one intent, not one handler: `move.down` moves whichever
//! surface has focus, and `run_cmd` picks the target from the context. A key
//! binds by code, and by the modifiers it names (all of them must be held);
//! a key with extra modifiers still matches its plain binding, as the
//! hand-written `match`es did, and a binding that names modifiers is tried
//! first (so Ctrl-C is quit, not `c`).
//!
//! In the list a binding may name the [`RowKind`] it is for (yaks-be65): the
//! lookup tries the selected row's kind first, then a binding for any kind. A
//! yak command is bound for yak rows only, so on a document row it finds
//! nothing and does nothing, as on an empty list; with no row selected the
//! view's own kind applies (an empty Wiki still takes `N` and `r`).
//!
//! Modal overlays (editors, pickers, the drawer, create, help, the command
//! line, the search boxes) and the mouse are not in the table.

use super::*;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

/// Declares [`Cmd`] with each variant's dotted name, and keeps `ALL` and
/// `name` in step with it (a new variant cannot be left out of either).
macro_rules! commands {
    ($($variant:ident => $name:literal,)*) => {
        /// One action a key can run.
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub(crate) enum Cmd {
            $($variant,)*
        }

        impl Cmd {
            /// Every command, for tests.
            #[cfg(test)]
            pub(crate) const ALL: &'static [Cmd] = &[$(Cmd::$variant,)*];

            /// The stable dotted name (what a future `keys:` override names).
            #[cfg(test)]
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
    PaneOpen => "pane.open",
    PaneEnter => "pane.enter",
    PaneBack => "pane.back",
    PanePeel => "pane.peel",
    PaneClose => "pane.close",
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
    AttachmentRename => "attachment.rename",
    WikiNew => "wiki.new",
    WikiEdit => "wiki.edit",
}

/// A surface that takes keys when no modal overlay is up. These are today's
/// surfaces as they are; later phases fold them together.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Ctx {
    /// The yak list.
    List,
    /// The yak detail pane.
    Detail,
    /// The text viewer overlay (attachments, wiki pages, sources).
    Viewer,
    /// The image viewer overlay.
    Image,
    /// The Sheds tab's list.
    Sheds,
    /// The Sheds tab's detail pane.
    ShedDetail,
}

impl Ctx {
    #[cfg(test)]
    const ALL: [Ctx; 6] = [
        Ctx::List,
        Ctx::Detail,
        Ctx::Viewer,
        Ctx::Image,
        Ctx::Sheds,
        Ctx::ShedDetail,
    ];
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
        kinds: &[None],
        title: "Detail pane",
    },
    Section {
        ctx: Ctx::Viewer,
        kinds: &[None],
        title: "Text viewer (an attachment, page or source)",
    },
    Section {
        ctx: Ctx::Image,
        kinds: &[None],
        title: "Image viewer",
    },
    Section {
        ctx: Ctx::List,
        kinds: &[Some(RowKind::Doc)],
        title: "Document rows (the Wiki view: the farm's pages and sources)",
    },
    Section {
        ctx: Ctx::Sheds,
        kinds: &[None],
        title: "Sheds tab (who is working on what: yaks sheds)",
    },
    Section {
        ctx: Ctx::ShedDetail,
        kinds: &[None],
        title: "Sheds tab, detail (l / Enter on a shed)",
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
        self.code == k.code && k.modifiers.contains(self.mods)
    }

    /// How the `?` overlay writes it: `j`, `↓`, `Space`, `Ctrl-N`, `S-Tab`.
    fn label(&self) -> String {
        let ctrl = self.mods.contains(KeyModifiers::CONTROL);
        let name = match self.code {
            KeyCode::Char(' ') => "Space".to_string(),
            KeyCode::Char(c) if ctrl => c.to_ascii_uppercase().to_string(),
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
            other => format!("{other:?}"),
        };
        let mut label = String::new();
        if ctrl {
            label.push_str("Ctrl-");
        }
        if self.mods.contains(KeyModifiers::SHIFT) {
            label.push_str("Shift-");
        }
        label + &name
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
    /// The filter drawer's label, which depends on whether a filter is
    /// active (`f:filter  /:search`, or `f:filter  Esc:clear`).
    Filter,
}

/// One row of the table: in `ctx`, any of `keys` runs `cmd`. A binding in the
/// list may name the [`RowKind`] it is for (a document row's `E` edits a page,
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

const fn hint(label: &'static str) -> Hint {
    Hint::Text(label)
}

use Cmd::*;
// Explicit, so it wins over the glob-imported `FarmSync` (the auto-sync state).
use Cmd::FarmSync;
use Ctx::*;
use Hint::None as NoHint;
use KeyCode::{BackTab, Down, End, Enter, Esc, Home, Left, PageDown, PageUp, Right, Tab, Up};
use tree::RowKind::{self, Doc, Yak};

/// The table. Within a context, the order is the order of the bottom hint
/// and of the `?` overlay, so the hinted bindings keep their hint's order.
#[rustfmt::skip]
static BINDINGS: &[Binding] = &[
    // -- the list: yak rows, and the keys that suit any row -------------------
    bind(List, &[k(Tab), c(']')], ViewNext, "Next view", hint("Tab:view")),
    bind(List, &[k(BackTab), c('[')], ViewPrev, "Previous view", NoHint),
    bind(List, &[c('j'), k(Down)], MoveDown, "Move cursor down", hint("j/k:move")),
    bind(List, &[c('k'), k(Up)], MoveUp, "Move cursor up", NoHint),
    bind(List, &[c('d')], MoveHalfDown, "Half-page down", NoHint),
    bind(List, &[c('u')], MoveHalfUp, "Half-page up", NoHint),
    bind(List, &[k(PageDown)], MovePageDown, "Full-page down", NoHint),
    bind(List, &[k(PageUp)], MovePageUp, "Full-page up", NoHint),
    bind(List, &[c('g'), k(Home)], MoveTop, "Top", NoHint),
    bind(List, &[c('G'), k(End)], MoveBottom, "Bottom", NoHint),
    bind_row(Yak, &[c('l'), k(Right), k(Enter)], PaneOpen, "Show detail pane", hint("l:detail")),
    bind_row(Yak, &[c('h')], FamilyCycle, "Cycle the family scope (tree views)", hint("h:family")),
    bind_row(Yak, &[c(' ')], TreeToggle, "Collapse / expand subtree", NoHint),
    bind_row(Yak, &[c('m')], YakMark, "Mark / unmark row (multi-select)", NoHint),
    bind_row(Yak, &[c('v')], ViewPick, "View picker", hint("v:views")),
    bind_row(Yak, &[c('V')], ViewSave, "Save filter as view", NoHint),
    bind(List, &[k(Esc)], ViewRevert, "Revert filter to the active view", NoHint),
    bind_row(Yak, &[c('c')], YakNew, "New root yak", hint("c/C:new")),
    bind_row(Yak, &[c('C')], YakNewChild, "New child yak", NoHint),
    bind_row(Yak, &[c('E')], YakEdit, "Edit: the form", hint("E:edit")),
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
    bind_row(Yak, &[c('A')], YakAttach, "Attach artifact", NoHint),
    bind_row(Yak, &[c('*')], YakStar, "Star / unstar (Starred view)", NoHint),
    bind_row(Yak, &[c('y')], RowCopy, "Copy yak id to clipboard", NoHint),
    bind_row(Yak, &[c('/')], SearchOpen, "Inline search", NoHint),
    bind(List, &[c('#')], NavGoto, "Go to any yak by id / title", NoHint),
    bind_row(Yak, &[c('B')], YakBackrefs, "Find references: yaks that point at this one", NoHint),
    bind_row(Yak, &[c('f')], FilterDrawer, "Filter drawer", Hint::Filter),
    bind(List, &[c('W')], FarmCommit, "Commit the farm (yaks commit)", NoHint),
    bind(List, &[c('U')], FarmSync, "Sync the farm: commit, pull, push (yaks sync)", NoHint),
    bind(List, &[c('?')], AppHelp, "Toggle this help", hint("?:help")),
    bind(List, &[c('q'), ctrl('c')], AppQuit, "Quit", NoHint),

    // -- the yak detail pane ---------------------------------------------------
    bind(Detail, &[c('h'), k(Left)], PaneBack, "Back to list (scrolls a code block or table left first)", hint("h:list")),
    bind(Detail, &[k(Esc)], PanePeel, "Peel back: selection, then find, then the list", NoHint),
    bind(Detail, &[c('l'), k(Right)], PaneScrollRight, "Scroll the code block / table under the cursor right", NoHint),
    bind(Detail, &[c('j'), k(Down)], MoveDown, "Move the line cursor down", hint("j/k:move")),
    bind(Detail, &[c('k'), k(Up)], MoveUp, "Move the line cursor up", NoHint),
    bind(Detail, &[c('d')], MoveHalfDown, "Half-page down", NoHint),
    bind(Detail, &[c('u')], MoveHalfUp, "Half-page up", NoHint),
    bind(Detail, &[c('g')], MoveTop, "Top", NoHint),
    bind(Detail, &[c('G')], MoveBottom, "Bottom", NoHint),
    bind(Detail, &[k(Tab), c(']')], LinkNext, "Next link", hint("Tab:link")),
    bind(Detail, &[k(BackTab), c('[')], LinkPrev, "Previous link", NoHint),
    bind(Detail, &[k(Enter)], PaneEnter, "Yank the selection, else follow the link / footnote / source file / farm page (status bar: URL, or a peek of the page)", hint("Enter:follow")),
    bind(Detail, &[c('i')], NavForward, "Nav forward (restores position)", hint("i/o:fwd/back")),
    bind(Detail, &[c('o')], NavBack, "Nav back (restores position)", NoHint),
    bind(Detail, &[c('J')], ItemNext, "Next task (stay in detail)", NoHint),
    bind(Detail, &[c('K')], ItemPrev, "Previous task (stay in detail)", NoHint),
    bind(Detail, &[ctrl('n')], BlockNext, "Next content block", NoHint),
    bind(Detail, &[ctrl('p')], BlockPrev, "Previous content block", NoHint),
    bind(Detail, &[c('v')], SelectVisual, "Visual select lines", NoHint),
    bind(Detail, &[shift(Down)], SelectExtendDown, "Extend the selection down", NoHint),
    bind(Detail, &[shift(Up)], SelectExtendUp, "Extend the selection up", NoHint),
    bind(Detail, &[c('y')], SelectYank, "Copy the selection, else the yak id", NoHint),
    bind(Detail, &[c('/')], FindOpen, "Find in the pane", NoHint),
    bind(Detail, &[c('n')], FindNext, "Next match", NoHint),
    bind(Detail, &[c('N')], FindPrev, "Previous match", NoHint),
    bind(Detail, &[c('#')], NavGoto, "Go to any yak by id / title", NoHint),
    bind(Detail, &[c('B')], YakBackrefs, "Find references: yaks that point at this one", NoHint),
    bind(Detail, &[c('E')], YakEdit, "Edit under the cursor: body block in place; header row: form", hint("E:edit")),
    bind(Detail, &[c('M')], YakComment, "Add a comment (in place, after the last one)", hint("M:comment")),
    bind(Detail, &[c('D')], YakDep, "Add dependency", hint("D:dep")),
    bind(Detail, &[c('S')], YakState, "State (S → n shears, with an optional comment)", hint("S:state")),
    bind(Detail, &[c('m')], YakMark, "Mark / unmark the yak (multi-select)", NoHint),
    bind(Detail, &[c('P')], YakPriority, "Priority", NoHint),
    bind(Detail, &[c('T')], YakType, "Type", NoHint),
    bind(Detail, &[c('L')], YakTags, "Tags", NoHint),
    bind(Detail, &[c('X')], YakSlaughter, "Slaughter (confirm; offers whole family)", NoHint),
    bind(Detail, &[c('R')], YakReparent, "Reparent", NoHint),
    bind(Detail, &[c('H')], YakHerd, "Move to another herd (multi-herd farm)", NoHint),
    bind(Detail, &[c('c')], YakNew, "New root yak", NoHint),
    bind(Detail, &[c('C')], YakNewChild, "New child yak", NoHint),
    bind(Detail, &[c('a')], YakAnswer, "Answer (awaiting a human) / request agent work (else): needs agent", NoHint),
    bind(Detail, &[c('Q')], YakAsk, "Ask a question: needs human", NoHint),
    bind(Detail, &[c('A')], YakAttach, "Attach artifact", NoHint),
    bind(Detail, &[c('O')], LinkOpenExternal, "Open the artifact / URL / source link on the line (a text file in the viewer)", NoHint),
    bind(Detail, &[c('r')], AttachmentRename, "Rename the attachment at the cursor", NoHint),
    bind(Detail, &[c('*')], YakStar, "Star / unstar (Starred view)", NoHint),
    bind(Detail, &[c('f')], FilterDrawer, "Filter drawer", Hint::Filter),
    bind(Detail, &[c('W')], FarmCommit, "Commit the farm (yaks commit)", NoHint),
    bind(Detail, &[c('U')], FarmSync, "Sync the farm: commit, pull, push (yaks sync)", NoHint),
    bind(Detail, &[c('?')], AppHelp, "Toggle this help", NoHint),
    bind(Detail, &[c('q'), ctrl('c')], AppQuit, "Quit", hint("q:quit")),

    // -- the text viewer -------------------------------------------------------
    bind(Viewer, &[k(Esc)], PanePeel, "Peel back: selection, then find, then close", NoHint),
    bind(Viewer, &[c('q')], PaneClose, "Close", NoHint),
    bind(Viewer, &[c('h'), k(Left)], PaneBack, "Close (scrolls a code block or table left first)", NoHint),
    bind(Viewer, &[c('l'), k(Right)], PaneScrollRight, "Scroll the code block / table under the cursor right", NoHint),
    bind(Viewer, &[c('j'), k(Down)], MoveDown, "Move the line cursor down", NoHint),
    bind(Viewer, &[c('k'), k(Up)], MoveUp, "Move the line cursor up", NoHint),
    bind(Viewer, &[c('d'), k(PageDown)], MoveHalfDown, "Half-page down", NoHint),
    bind(Viewer, &[c('u'), k(PageUp)], MoveHalfUp, "Half-page up", NoHint),
    bind(Viewer, &[c('g'), k(Home)], MoveTop, "Top", NoHint),
    bind(Viewer, &[c('G'), k(End)], MoveBottom, "Bottom", NoHint),
    bind(Viewer, &[k(Tab), c(']')], LinkNext, "Next link", NoHint),
    bind(Viewer, &[k(BackTab), c('[')], LinkPrev, "Previous link", NoHint),
    bind(Viewer, &[k(Enter)], PaneEnter, "Yank the selection, else follow the link (o / i come back)", NoHint),
    bind(Viewer, &[c('o')], NavBack, "Back over a footnote or document jump", NoHint),
    bind(Viewer, &[c('i')], NavForward, "Forward over a footnote or document jump", NoHint),
    bind(Viewer, &[c('v')], SelectVisual, "Visual select lines", NoHint),
    bind(Viewer, &[shift(Down)], SelectExtendDown, "Extend the selection down", NoHint),
    bind(Viewer, &[shift(Up)], SelectExtendUp, "Extend the selection up", NoHint),
    bind(Viewer, &[c('y')], SelectYank, "Copy the selection (a .md file: its source), else the line", NoHint),
    bind(Viewer, &[c('/')], FindOpen, "Find in the file", NoHint),
    bind(Viewer, &[c('n')], FindNext, "Next match", NoHint),
    bind(Viewer, &[c('N')], FindPrev, "Previous match", NoHint),
    bind(Viewer, &[c('p')], PeekToggle, "Show / hide the peek pane under the page", NoHint),
    bind(Viewer, &[c('E')], DocEdit, "Edit the file in place (Ctrl-S saves; a past version or a source is read-only)", NoHint),
    bind(Viewer, &[c('O')], FileOpenExternal, "Open the file externally", NoHint),

    // -- the image viewer ------------------------------------------------------
    bind(Image, &[k(Esc), c('q'), c('h'), k(Left)], PaneClose, "Close (back to the viewer it came from, if any)", NoHint),
    bind(Image, &[c('O')], FileOpenExternal, "Open the file externally", NoHint),

    // -- the list: document rows (the Wiki view). Its movement, view and farm
    // -- keys are the list's; `v`, `?` and `q` repeat theirs only so that the
    // -- bottom hint is the Wiki's own, not the yak list's.
    bind_row(Doc, &[c('l'), k(Right), k(Enter)], PaneOpen, "Open in the viewer: frontmatter above the body; a source with its extracted text", hint("Enter:open")),
    bind_row(Doc, &[c('E')], WikiEdit, "Edit the page: frontmatter (YAML) and body; Ctrl-S saves", hint("E:edit")),
    bind_row(Doc, &[c('N')], WikiNew, "New page: type its path (wiki/... or <herd>/wiki/...), then its body", hint("N:new page")),
    bind_row(Doc, &[c('/')], SearchOpen, "Search titles, paths, kinds and tags (Esc clears)", hint("/:search")),
    bind_row(Doc, &[c('y')], RowCopy, "Copy the path", hint("y:copy path")),
    bind_row(Doc, &[c('r')], ListRefresh, "Reread the pages (also on any farm change)", hint("r:reread")),
    bind_row(Doc, &[c('v')], ViewPick, "", NoHint),
    bind_row(Doc, &[c('?')], AppHelp, "", hint("?:help")),
    bind_row(Doc, &[c('q'), ctrl('c')], AppQuit, "", hint("q:quit")),

    // -- the Sheds tab ---------------------------------------------------------
    bind(Sheds, &[k(Tab), c(']')], ViewNext, "Next view", hint("Tab:view")),
    bind(Sheds, &[k(BackTab), c('[')], ViewPrev, "Previous view", NoHint),
    bind(Sheds, &[c('j'), k(Down)], MoveDown, "Move between sheds (actors; attention first)", hint("j/k:move")),
    bind(Sheds, &[c('k'), k(Up)], MoveUp, "Move up", NoHint),
    bind(Sheds, &[c('d'), k(PageDown)], MoveHalfDown, "Half-page down", NoHint),
    bind(Sheds, &[c('u'), k(PageUp)], MoveHalfUp, "Half-page up", NoHint),
    bind(Sheds, &[c('g')], MoveTop, "Top", NoHint),
    bind(Sheds, &[c('G')], MoveBottom, "Bottom", NoHint),
    bind(Sheds, &[c('l'), k(Right), k(Enter)], PaneOpen, "Show the shed: shaving, asking, recent yaks", hint("l:detail")),
    bind(Sheds, &[c('y')], RowCopy, "Copy the shed's name", hint("y:copy name")),
    bind(Sheds, &[c('r')], ListRefresh, "Rescan now (auto every 10 s)", hint("r:rescan")),
    bind(Sheds, &[c('v')], ViewPick, "View picker", hint("v:views")),
    bind(Sheds, &[c('#')], NavGoto, "Go to any yak by id / title", NoHint),
    bind(Sheds, &[c('?')], AppHelp, "Toggle this help", hint("?:help")),
    bind(Sheds, &[c('q'), ctrl('c')], AppQuit, "Quit", hint("q:quit")),

    // -- the Sheds tab's detail pane (its keys repeat the list's: no yak key
    // -- and no other pane's key reaches it) ------------------------------------
    bind(ShedDetail, &[c('h'), k(Left), k(Esc)], PaneBack, "Back to the sheds", hint("h:list")),
    bind(ShedDetail, &[c('j'), k(Down)], MoveDown, "Move between its yaks", hint("j/k:yak")),
    bind(ShedDetail, &[c('k'), k(Up)], MoveUp, "Move up", NoHint),
    bind(ShedDetail, &[c('d'), k(PageDown)], MoveHalfDown, "Half-page down", NoHint),
    bind(ShedDetail, &[c('u'), k(PageUp)], MoveHalfUp, "Half-page up", NoHint),
    bind(ShedDetail, &[c('g')], MoveTop, "Top", NoHint),
    bind(ShedDetail, &[c('G')], MoveBottom, "Bottom", NoHint),
    bind(ShedDetail, &[c('J')], ItemNext, "Next shed", hint("J/K:shed")),
    bind(ShedDetail, &[c('K')], ItemPrev, "Previous shed", NoHint),
    bind(ShedDetail, &[k(Enter)], PaneEnter, "Open the yak", hint("Enter:open yak")),
    bind(ShedDetail, &[c('y')], RowCopy, "Copy the yak id", hint("y:copy id")),
    bind(ShedDetail, &[c('r')], ListRefresh, "Rescan now (auto every 10 s)", hint("r:rescan")),
    bind(ShedDetail, &[c('?')], AppHelp, "", hint("?:help")),
    bind(ShedDetail, &[k(Tab), c(']')], ViewNext, "", NoHint),
    bind(ShedDetail, &[k(BackTab), c('[')], ViewPrev, "", NoHint),
    bind(ShedDetail, &[c('v')], ViewPick, "", NoHint),
    bind(ShedDetail, &[c('#')], NavGoto, "", NoHint),
    bind(ShedDetail, &[c('q'), ctrl('c')], AppQuit, "", NoHint),
];

/// The command `k` runs in `ctx` on a row of `kind` (`None` outside the list):
/// a binding that names modifiers first (Ctrl-C before `c`), else a plain one,
/// ignoring any modifiers the terminal adds (an upper-case letter arrives with
/// Shift). At each step a binding for the row's kind beats one for any kind.
pub(crate) fn lookup(ctx: Ctx, kind: Option<RowKind>, k: &KeyEvent) -> Option<Cmd> {
    let find = |modified: bool, kind: Option<RowKind>| {
        BINDINGS
            .iter()
            .filter(|b| b.ctx == ctx && b.kind == kind)
            .find(|b| {
                b.keys
                    .iter()
                    .any(|key| key.mods.is_empty() != modified && key.matches(k))
            })
            .map(|b| b.cmd)
    };
    [true, false].into_iter().find_map(|modified| {
        kind.and_then(|kind| find(modified, Some(kind)))
            .or_else(|| find(modified, None))
    })
}

/// The bottom bar's key hints for `ctx` on a row of `kind`: each hinted
/// binding's label, in table order, two spaces apart. A binding for any kind
/// is left out where one for this kind takes its keys. `filter_hint` stands in
/// for the drawer's.
fn hint_for(ctx: Ctx, kind: Option<RowKind>, filter_hint: &str) -> String {
    let shown = |b: &Binding| {
        b.ctx == ctx
            && match (b.kind, kind) {
                (None, None) => true,
                (None, Some(kind)) => !BINDINGS.iter().any(|o| {
                    o.ctx == ctx
                        && o.kind == Some(kind)
                        && o.keys.iter().any(|k| b.keys.contains(k))
                }),
                (b_kind, kind) => b_kind == kind,
            }
    };
    BINDINGS
        .iter()
        .filter(|b| shown(b))
        .filter_map(|b| match b.hint {
            Hint::None => None,
            Hint::Text(t) => Some(t),
            Hint::Filter => Some(filter_hint),
        })
        .collect::<Vec<_>>()
        .join("  ")
}

pub(crate) fn help_hint(app: &App) -> String {
    let filter_hint = if app.filter.content_active() {
        "f:filter  Esc:clear"
    } else {
        "f:filter  /:search"
    };
    hint_for(app.surface_ctx(), app.surface_kind(), filter_hint)
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
                "/wiki/x.md#h, page-relative, /sources/..., @sha pins: open in the viewer at the anchor; o/i hop back/fwd between documents",
            ),
            (
                "(peek)",
                "A pane under the page previews the link under the cursor (section, lines, page head); p hides it",
            ),
            (
                "(image)",
                "Shown in the pane (png/jpg/gif/webp/svg); O opens externally, Esc/q closes",
            ),
            (
                "(image alone)",
                "A body image on its own line is drawn in place (config inline_images)",
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
            ("Wheel", "Scroll the pane under the pointer"),
            (
                "Click row",
                "Select (again: open detail; detail stays open)",
            ),
            ("Click detail", "Move line cursor (again: follow link)"),
            ("Click tab", "Switch view"),
            ("Shift-drag", "Native text selection (terminal)"),
        ],
    ),
];

/// The keyboard reference shown by `?`, as styled lines: cyan-bold section
/// headers, a yellow key column, then the description. A section per context
/// from [`BINDINGS`], then the [`EXTRAS`]. Rebuilt on demand; also used to
/// clamp help scroll.
pub(crate) fn help_content() -> Vec<Line<'static>> {
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
            lines.push(entry(&b.key_label(), b.help));
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

impl Binding {
    /// Its keys as the `?` overlay writes them: `j / ↓`.
    fn key_label(&self) -> String {
        self.keys
            .iter()
            .map(Key::label)
            .collect::<Vec<_>>()
            .join(" / ")
    }
}

impl App {
    /// The surface that takes a key when no overlay is up: the Sheds tab, else
    /// the list or its detail pane. (The viewers are overlays; their handlers
    /// pass their own context.)
    pub(crate) fn surface_ctx(&self) -> Ctx {
        match (self.on_sheds_tab(), self.focus) {
            (true, Focus::List) => Ctx::Sheds,
            (true, Focus::Detail) => Ctx::ShedDetail,
            (false, Focus::List) => Ctx::List,
            (false, Focus::Detail) => Ctx::Detail,
        }
    }

    /// The kind of row the list's keys act on: the selected row's, else the
    /// view's. `None` on every other surface.
    pub(crate) fn surface_kind(&self) -> Option<RowKind> {
        (self.surface_ctx() == Ctx::List).then(|| self.row_kind())
    }

    /// Run `cmd` as the key bound to it in `ctx`. A command is an intent, so
    /// where surfaces differ (what `move.down` moves, what Esc peels) the arm
    /// picks the target from `ctx`; the viewers run theirs themselves.
    pub(crate) fn run_cmd(&mut self, ctx: Ctx, cmd: Cmd) {
        match ctx {
            Ctx::Viewer => return self.run_viewer_cmd(cmd),
            Ctx::Image => return self.run_image_cmd(cmd),
            _ => {}
        }
        match cmd {
            AppQuit => self.quit = true,
            AppHelp => self.open_help(),
            MoveDown | MoveUp | MoveTop | MoveBottom | MoveHalfDown | MoveHalfUp | MovePageDown
            | MovePageUp => self.run_move(ctx, cmd),
            ItemNext | ItemPrev => {
                let by = if cmd == ItemNext { 1 } else { -1 };
                match ctx {
                    Ctx::ShedDetail => self.move_shed(by),
                    _ => self.detail_next_task(by),
                }
            }
            ViewNext => self.switch_tab(1),
            ViewPrev => self.switch_tab(-1),
            ViewPick => self.open_view_picker(),
            ViewSave => self.open_save_view(),
            ViewRevert => self.revert_list_view(),
            FamilyCycle => self.cycle_family_scope(),
            TreeToggle => self.toggle_collapse(),
            PaneOpen => match ctx {
                Ctx::Sheds => self.open_shed(),
                Ctx::List if self.selected_doc().is_some() => self.open_wiki_entry(),
                _ => self.open_detail_pane(),
            },
            PaneEnter => match ctx {
                Ctx::ShedDetail => self.open_shed_yak(),
                _ => self.detail_enter(),
            },
            PaneBack => match ctx {
                Ctx::ShedDetail => self.focus = Focus::List,
                _ => self.detail_back(),
            },
            PanePeel => self.detail_peel(),
            PaneScrollRight => {
                self.scroll_block(1);
            }
            RowCopy => match ctx {
                Ctx::List if self.selected_doc().is_some() => self.copy_wiki_path(),
                Ctx::Sheds | Ctx::ShedDetail => self.copy_shed_thing(),
                _ => self.copy_selected_id(),
            },
            ListRefresh => match ctx {
                Ctx::List => self.reread_wiki(),
                _ => self.rescan_sheds(),
            },
            SearchOpen => self.open_search(),
            FilterDrawer => self.open_drawer(),
            NavGoto => self.open_goto_picker(),
            NavBack => self.nav_back(),
            NavForward => self.nav_forward(),
            LinkNext => self.jump_link(1),
            LinkPrev => self.jump_link(-1),
            LinkOpenExternal => self.open_current_external(),
            FindOpen => self.open_detail_find(),
            FindNext => self.detail_find_jump(1),
            FindPrev => self.detail_find_jump(-1),
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
            AttachmentRename => self.open_rename_attachment(),
            WikiNew => self.open_new_page(),
            WikiEdit => self.edit_wiki_entry(),
            // The viewers' own: `run_viewer_cmd` and `run_image_cmd`.
            PaneClose | PeekToggle | DocEdit | FileOpenExternal => {}
        }
    }

    /// A movement command on the surface `ctx`: by lines, a half or a full
    /// page, or to either end.
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
            Ctx::Sheds => self.move_shed(by),
            Ctx::ShedDetail => self.move_shed_yak(by),
            Ctx::Viewer | Ctx::Image => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn no_two_bindings_in_a_context_and_kind_share_a_key() {
        let kinds = [None, Some(RowKind::Yak), Some(RowKind::Doc)];
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
        let all = text(&help_content());
        for section in &SECTIONS {
            let sect = all
                .split("\n\n")
                .find(|s| s.starts_with(section.title))
                .unwrap_or_else(|| panic!("no section {}", section.title));
            for b in section.bindings().filter(|b| !b.help.is_empty()) {
                let row = format!("  {:<16}{}", b.key_label(), b.help);
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
        let filter = "f:filter  /:search";
        let cases = [
            (
                Ctx::List,
                Some(RowKind::Yak),
                "Tab:view  j/k:move  l:detail  h:family  v:views  c/C:new  E:edit  X:del  S:state  D:dep  f:filter  /:search  ?:help",
            ),
            (
                Ctx::Detail,
                None,
                "h:list  j/k:move  Tab:link  Enter:follow  i/o:fwd/back  E:edit  M:comment  D:dep  S:state  f:filter  /:search  q:quit",
            ),
            (
                Ctx::List,
                Some(RowKind::Doc),
                "Tab:view  j/k:move  Enter:open  E:edit  N:new page  /:search  y:copy path  r:reread  ?:help  q:quit",
            ),
            (
                Ctx::Sheds,
                None,
                "Tab:view  j/k:move  l:detail  y:copy name  r:rescan  v:views  ?:help  q:quit",
            ),
            (
                Ctx::ShedDetail,
                None,
                "h:list  j/k:yak  J/K:shed  Enter:open yak  y:copy id  r:rescan  ?:help",
            ),
            (Ctx::Viewer, None, ""),
            (Ctx::Image, None, ""),
        ];
        for (ctx, kind, want) in cases {
            assert_eq!(hint_for(ctx, kind, filter), want, "{ctx:?} {kind:?}");
        }
        // With a filter active the drawer's hint changes, nothing else.
        assert_eq!(
            hint_for(Ctx::Detail, None, "f:filter  Esc:clear"),
            "h:list  j/k:move  Tab:link  Enter:follow  i/o:fwd/back  E:edit  M:comment  D:dep  S:state  f:filter  Esc:clear  q:quit"
        );
        assert_eq!(
            hint_for(Ctx::List, Some(RowKind::Yak), "f:filter  Esc:clear"),
            "Tab:view  j/k:move  l:detail  h:family  v:views  c/C:new  E:edit  X:del  S:state  D:dep  f:filter  Esc:clear  ?:help"
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
            lookup(Ctx::Detail, None, &key(KeyCode::Down, KeyModifiers::SHIFT)),
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
        // Ctrl-N is a block jump in the detail pane only.
        assert_eq!(
            lookup(
                Ctx::Detail,
                None,
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
            Some(FindNext)
        );
    }

    #[test]
    fn lookup_prefers_a_binding_for_the_rows_kind() {
        let key = |ch| KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE);
        let (yak, doc) = (Some(RowKind::Yak), Some(RowKind::Doc));
        // The same key, per kind: E edits a yak by its form, a page in the editor.
        assert_eq!(lookup(Ctx::List, yak, &key('E')), Some(YakEdit));
        assert_eq!(lookup(Ctx::List, doc, &key('E')), Some(WikiEdit));
        // A key bound for any kind reaches every kind.
        assert_eq!(lookup(Ctx::List, yak, &key('j')), Some(MoveDown));
        assert_eq!(lookup(Ctx::List, doc, &key('j')), Some(MoveDown));
        // A yak command is no document command.
        assert_eq!(lookup(Ctx::List, doc, &key('S')), None);
        assert_eq!(lookup(Ctx::List, doc, &key('c')), None);
        // Ctrl-C still quits where a kind binds plain `c`.
        let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(lookup(Ctx::List, yak, &ctrl_c), Some(AppQuit));
        assert_eq!(lookup(Ctx::List, doc, &ctrl_c), Some(AppQuit));
    }
}

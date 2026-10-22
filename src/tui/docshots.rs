//! Doc "screenshots": drive the headless [`App`] to a handful of representative
//! states and write each to a self-contained **color SVG** under `docs/assets/`
//! for embedding in `docs/tui.md`.
//!
//! This is not a normal test — it is a code-gated asset generator. It lives in a
//! `#[cfg(test)]` child module of `tui` (like `headless`) so it can construct an
//! `App` and reach the same private drive path the snapshot tests use, without a
//! `lib` target (yaks is a bin crate, so an `examples/` file cannot `use` it).
//!
//! The buffer→SVG rendering itself lives in [`toque::render_to_svg`]; this module
//! only picks the states worth capturing.
//!
//! Regenerate the assets with:
//!
//! ```sh
//! cargo test -p yaks docshots -- --ignored
//! ```

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use toque::HeadlessApp;

use crate::model::{Status, Task};
use crate::tui::{App, handle_key};

// -- sample farm ----------------------------------------------------------

fn t(id: &str, title: &str, kind: &str, pri: u8, status: Status, parent: Option<&str>) -> Task {
    Task {
        id: id.into(),
        title: title.into(),
        kind: kind.into(),
        priority: pri,
        status,
        created: Some("2026-09-01T10:00:00Z".into()),
        updated: Some("2026-09-06T12:00:00Z".into()),
        parent: parent.map(String::from),
        labels: vec![],
        depends_on: vec![],
        source: None,
        needs: None,
        verify: None,
        extra: Vec::new(),
        body: String::new(),
    }
}

/// A small, realistic farm: an umbrella with children across statuses, labels,
/// a dependency, and one child awaiting a human (the ⏳ needs badge).
fn farm() -> Vec<Task> {
    let mut coord = t(
        "yaks-3901",
        "Coordination substrate for parallel yak-shaving",
        "task",
        1,
        Status::Shaving,
        None,
    );
    coord.labels = vec!["coord".into(), "git".into()];

    let mut inbox = t(
        "yaks-b517",
        "HITL: needs-human frontmatter + inbox surfacing",
        "feature",
        2,
        Status::Shaving,
        Some("yaks-3901"),
    );
    inbox.labels = vec!["cli".into(), "ui".into()];
    inbox.needs = Some("human".into());
    inbox.body = "Add a `needs:` block agents raise via `ask` and a human clears via \
                  `answer`; surface blocked yaks in an Inbox view.\n\n\
                  ---\n▸ 2026-09-06T11:40:00Z [wtA]\nProposal drafted — see the drawer \
                  `inbox` chip. Which glyph should mark a blocked row?\n\n\
                  ---\n▸ 2026-09-06T12:00:00Z [joel]\nGo with the hourglass; ship it."
        .into();

    let mut diff = t(
        "yaks-70e5",
        "yaks diff <refA> <refB>: ref-generic farm diff",
        "feature",
        3,
        Status::Hairy,
        Some("yaks-3901"),
    );
    diff.labels = vec!["cli".into()];
    diff.depends_on = vec!["yaks-b517".into()];

    let mut shot = t(
        "yaks-3677",
        "TUI doc screenshots via headless buffer->SVG",
        "feature",
        3,
        Status::Shaving,
        Some("yaks-3901"),
    );
    shot.labels = vec!["ui".into(), "docs".into()];
    shot.body = "Walk the ratatui Buffer's per-cell symbol+fg/bg into an SVG grid so \
                 docs render in color."
        .into();

    let done = t(
        "yaks-865d",
        "Normalize skill names to yaks[-*]",
        "task",
        2,
        Status::Shorn,
        Some("yaks-3901"),
    );

    let mut solo = t(
        "yaks-10fb",
        "Coordination follow-ups (post-3901)",
        "idea",
        3,
        Status::Hairy,
        None,
    );
    solo.labels = vec!["agent".into()];

    vec![coord, inbox, diff, shot, done, solo]
}

fn app_at(w: u16, h: u16) -> App {
    let mut app = App::new(farm());
    // A settled Sheds tab, so the tab strip shows a real count, not `(…)`.
    app.apply_sheds_snapshot(crate::tui::sheds_tab::fixture::snapshot());
    HeadlessApp::on_resize(&mut app, w, h);
    app
}

/// A two-herd farm (`web-*` + `api-*`) for the multi-herd chrome shots: the
/// per-herd id colour and the edit form's herd chip row (yaks-71d1).
fn multi_herd_farm() -> Vec<Task> {
    let mut api = t(
        "api-0001",
        "Token refresh drops the retry budget",
        "bug",
        2,
        Status::Hairy,
        None,
    );
    api.labels = vec!["backend".into()];
    let mut web = t(
        "web-0007",
        "Settings page: surface the active herd",
        "feature",
        3,
        Status::Hairy,
        None,
    );
    web.labels = vec!["ui".into()];
    vec![api, web]
}

fn multi_herd_app_at(w: u16, h: u16) -> App {
    let mut app = App::new(multi_herd_farm());
    app.apply_sheds_snapshot(crate::tui::sheds_tab::fixture::snapshot());
    HeadlessApp::on_resize(&mut app, w, h);
    app
}

fn press(app: &mut App, c: char) {
    handle_key(app, KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
}

fn press_key(app: &mut App, code: KeyCode) {
    handle_key(app, KeyEvent::new(code, KeyModifiers::NONE));
}

/// Switch to a built-in view by its stable key (e.g. `status:shaving`, `inbox`).
fn view_to(app: &mut App, key: &str) {
    let i = app
        .views
        .iter()
        .position(|v| v.key == key)
        .unwrap_or_else(|| panic!("view {key:?} present"));
    app.set_view(i);
}

fn write_svg(name: &str, app: &App, w: u16, h: u16) {
    let svg = toque::render_to_svg(app, w, h);
    let path = format!("docs/assets/{name}.svg");
    std::fs::create_dir_all("docs/assets").unwrap();
    std::fs::write(&path, svg).unwrap();
    eprintln!("wrote {path}");
}

/// Regenerate every SVG in `docs/assets/`. Ignored by default; run explicitly:
/// `cargo test -p yaks docshots -- --ignored`.
#[test]
#[ignore = "asset generator; run with --ignored to refresh docs/assets/*.svg"]
fn docshots() {
    let (w, h) = (96u16, 20u16);

    // 1. List view (tree): the Shaving tab shows the umbrella + children across
    //    statuses, labels, a dependency, and the ⏳ needs badge on the HITL yak.
    let mut app = app_at(w, h);
    view_to(&mut app, "status:shaving");
    write_svg("tui-list", &app, w, h);

    // 2. Detail pane: open the HITL yak (only member of the Inbox), showing the
    //    Needs: field and attributed [wtA]/[joel] notes. Taller so the
    //    description + both comments fit under the metadata block.
    let dh = 34u16;
    let mut app = app_at(w, dh);
    view_to(&mut app, "inbox");
    press_key(&mut app, KeyCode::Enter); // open detail on the sole inbox row
    write_svg("tui-detail", &app, w, dh);

    // 3. Inbox view: flat list of every yak awaiting a human, across statuses.
    let mut app = app_at(w, h);
    view_to(&mut app, "inbox");
    write_svg("tui-inbox", &app, w, h);

    // 4. Filter drawer: the chip facets (status/type/priority/deps incl. inbox).
    let mut app = app_at(w, h);
    press(&mut app, 'f');
    write_svg("tui-drawer", &app, w, h);

    // 5. Pinned detail header: open the HITL yak's detail in a short pane and
    //    scroll to the end (`G`) so the frontmatter is off-screen — the sticky
    //    one-line header keeps its id / state / type / title in view.
    let ph = 18u16;
    let mut app = app_at(w, ph);
    view_to(&mut app, "inbox");
    press_key(&mut app, KeyCode::Enter); // open detail on the sole inbox row
    press(&mut app, 'G'); // jump to the last line -> scrolls; the header pins
    write_svg("tui-detail-pinned", &app, w, ph);

    // 6. Ephemeral status line placement: at a narrow width the yellow
    //    notification can't fit beside the tabs, so instead of scribbling over
    //    them it drops to the blank row directly below the tab strip. (Set the
    //    field directly — a child module can reach `App`'s private fields — to
    //    freeze the transient message that a live mutation would otherwise set.)
    let sw = 90u16;
    let mut app = app_at(sw, 12);
    app.notification = Some("saved view: quarterly-planning".into());
    write_svg("tui-status-below-tabs", &app, sw, 12);

    // 7. Multi-herd edit form: on a two-herd farm (`web`/`api`), `E` opens the
    //    shared edit form, which grows a `herd` chip row (seeded to the yak's
    //    current herd) after `source` (the form is header fields only; body
    //    rows edit in place) — the editable herd
    //    affordance (yaks-71d1).
    let mut app = multi_herd_app_at(w, h);
    app.select_id("web-0007");
    press(&mut app, 'E');
    write_svg("tui-edit-herd", &app, w, h);

    // 8. Sheds tab (yaks-98f1, yaks-9eaa): every checkout, attention first
    //    (an open ask, active workers, then idle ones dimmed), from a fixed
    //    snapshot so the shot is stable; then a shed's detail: its git state
    //    and the yaks it wrote, newest first. 150 wide so the detail sits
    //    beside the list.
    let (sw, sh) = (150u16, 18u16);
    let mut app = app_at(sw, sh);
    view_to(&mut app, crate::tui::view::SHEDS_KEY);
    app.sheds.age = Some(4);
    write_svg("tui-sheds", &app, sw, sh);
    press(&mut app, 'j');
    press(&mut app, 'j');
    press(&mut app, 'l');
    write_svg("tui-sheds-detail", &app, sw, sh);
}

/// Regenerate the agent-session example used by toque's README: drive the real
/// protocol (`Session::step`), writing the text transcript plus an SVG of every
/// frame to `docs/assets/session*`. Ignored by default; run explicitly:
/// `cargo test -p yaks docshots_session -- --ignored`.
#[test]
#[ignore = "asset generator; run with --ignored to refresh docs/assets/session*"]
fn docshots_session() {
    use toque::{DriverOpts, Session};

    let (w, h) = (96u16, 32u16);
    let mut app = App::new(farm());
    app.apply_sheds_snapshot(crate::tui::sheds_tab::fixture::snapshot());
    let mut s = Session::new(
        app,
        DriverOpts {
            width: w,
            height: h,
            diff: false,
        },
    );
    let mut transcript = Vec::new();
    let shot = |s: &Session<App>, n: usize| {
        let svg = toque::render_to_svg(s.app(), w, h);
        std::fs::write(format!("docs/assets/session-{n}.svg"), svg).unwrap();
    };

    s.emit(&mut transcript).unwrap();
    shot(&s, 0);
    for (n, line) in ["key Tab", "key j", "key l"].into_iter().enumerate() {
        transcript.extend_from_slice(format!("> {line}\n").as_bytes());
        s.step(line, &mut transcript).unwrap();
        shot(&s, n + 1);
    }
    std::fs::write("docs/assets/session.txt", transcript).unwrap();
    eprintln!("wrote docs/assets/session-{{0..3}}.svg + session.txt");
}

// -- markdown palette swatch (yaks-bc53) ----------------------------------

/// A "terminal theme" for the swatch: what the 16 ANSI colours and the default
/// fg/bg resolve to. The SVG renderer hard-codes a dark theme; a light terminal
/// is simulated by re-colouring the buffer's cells (so both panels live in one
/// SVG and show the *same* palette calls).
struct Theme {
    bg: ratatui::style::Color,
    fg: ratatui::style::Color,
    ansi: fn(ratatui::style::Color) -> Option<(u8, u8, u8)>,
}

/// VS Code "Light+" terminal ANSI colours: a real, widely-used light theme (its
/// green and yellow are the weak spots, which is the point of looking).
fn light_ansi(c: ratatui::style::Color) -> Option<(u8, u8, u8)> {
    use ratatui::style::Color::*;
    Some(match c {
        Black => (0x00, 0x00, 0x00),
        Red => (0xcd, 0x31, 0x31),
        Green => (0x00, 0xbc, 0x00),
        Yellow => (0x94, 0x98, 0x00),
        Blue => (0x04, 0x51, 0xa5),
        Magenta => (0xbc, 0x05, 0xbc),
        Cyan => (0x05, 0x98, 0xbc),
        Gray => (0x55, 0x55, 0x55),
        DarkGray => (0x66, 0x66, 0x66),
        LightRed => (0xcd, 0x31, 0x31),
        LightGreen => (0x14, 0xce, 0x14),
        LightYellow => (0xb5, 0xba, 0x00),
        LightBlue => (0x04, 0x51, 0xa5),
        LightMagenta => (0xbc, 0x05, 0xbc),
        LightCyan => (0x05, 0x98, 0xbc),
        White => (0xa5, 0xa5, 0xa5),
        _ => return None,
    })
}

fn paint_theme(buf: &mut ratatui::buffer::Buffer, area: ratatui::layout::Rect, theme: &Theme) {
    use ratatui::style::Color;
    let map = |c: Color, default: Color| match c {
        Color::Reset => default,
        c => (theme.ansi)(c).map_or(c, |(r, g, b)| Color::Rgb(r, g, b)),
    };
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            let cell = &mut buf[(x, y)];
            cell.fg = map(cell.fg, theme.fg);
            cell.bg = map(cell.bg, theme.bg);
        }
    }
}

/// The editor path: run a source line through the real [`markdown::Highlighter`]
/// and turn its char spans into a styled `Line`.
fn editor_line(
    hl: &mut crate::tui::markdown::Highlighter,
    text: &str,
) -> ratatui::text::Line<'static> {
    use ratatui::style::Style;
    use ratatui::text::{Line, Span};
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut at = 0;
    for sp in hl.line(text) {
        if sp.start > at {
            out.push(Span::raw(chars[at..sp.start].iter().collect::<String>()));
        }
        out.push(Span::styled(
            chars[sp.start..sp.start + sp.len]
                .iter()
                .collect::<String>(),
            sp.style,
        ));
        at = sp.start + sp.len;
    }
    if at < chars.len() {
        out.push(Span::styled(
            chars[at..].iter().collect::<String>(),
            Style::new(),
        ));
    }
    Line::from(out)
}

/// One panel's worth of sample document: every style in the palette, hand-built
/// from palette calls (this is a swatch, not the renderer).
fn palette_swatch(width: usize) -> Vec<ratatui::text::Line<'static>> {
    use crate::tui::palette::{self as p, AlertKind};
    use ratatui::style::Style;
    use ratatui::text::{Line, Span};

    let s = |t: &str, st: Style| Span::styled(t.to_string(), st);
    let r = |t: &str| Span::raw(t.to_string());
    let section = |t: &str| Line::from(s(&format!("── {t} "), p::marker()));
    let mut out: Vec<Line<'static>> = Vec::new();

    out.push(section("pane labels, for comparison"));
    out.push(Line::from(vec![
        s("Children:", p::pane_label()),
        r("   "),
        s("Depends on:", p::pane_label()),
        r("   "),
        s("Needs:", p::pane_label()),
    ]));
    out.push(Line::raw(""));

    out.push(section("headings (read pane)"));
    for (lvl, t) in [
        (1, "Heading 1"),
        (2, "Heading 2"),
        (3, "Heading 3"),
        (4, "Heading 4"),
    ] {
        out.push(Line::from(s(t, p::heading(lvl))));
    }
    out.push(Line::raw(""));

    out.push(section("inline"));
    out.push(Line::from(vec![
        r("Plain, "),
        s("emphasis", p::emphasis()),
        r(", "),
        s("strong", p::strong()),
        r(", "),
        s("strong+em", p::strong().patch(p::emphasis())),
        r(", "),
        s("struck", p::strikethrough()),
        r(","),
    ]));
    out.push(Line::from(vec![
        r("and "),
        s("inline_code()", p::inline_code()),
        r(", a "),
        s("link text", p::link_text()),
        r(" "),
        s("(https://example.com/x)", p::link_url()),
        r("."),
    ]));
    out.push(Line::raw(""));

    out.push(section("code block"));
    out.push(Line::from(s("```rust", p::code_fence())));
    out.push(Line::from(s(
        "fn main() { println!(\"hi\"); }",
        p::code_block(),
    )));
    out.push(Line::from(s("```", p::code_fence())));
    out.push(Line::raw(""));

    out.push(section("lists, tasks, quote, rule"));
    for (depth, t) in [
        (0usize, "top level"),
        (1, "nested"),
        (2, "nested deeper"),
        (3, "cycles back"),
    ] {
        out.push(Line::from(vec![
            r(&"  ".repeat(depth)),
            s(p::bullet_glyph(depth), p::bullet()),
            r(&format!(" {t}")),
        ]));
    }
    out.push(Line::from(vec![s("1.", p::bullet()), r(" ordered item")]));
    for (done, t) in [(true, "a finished task"), (false, "an open task")] {
        out.push(Line::from(vec![
            s(p::task_glyph(done), p::task(done)),
            r(&format!(" {t}")),
        ]));
    }
    out.push(Line::from(vec![
        s(p::QUOTE_BAR, p::quote_bar()),
        r(" "),
        s(
            "A quote reads in italic; the bar is quiet.",
            p::quote_text(),
        ),
    ]));
    out.push(Line::from(vec![
        s(p::QUOTE_BAR, p::quote_bar()),
        r(" "),
        s(p::bullet_glyph(0), p::bullet()),
        r(" a list inside a quote keeps its "),
        s("colours", p::inline_code()),
    ]));
    out.push(Line::from(s(
        &p::RULE_CHAR.to_string().repeat(width),
        p::rule(),
    )));
    out.push(Line::raw(""));

    out.push(section("alerts"));
    for kind in AlertKind::ALL {
        out.push(Line::from(vec![
            s(p::QUOTE_BAR, p::alert(kind)),
            r(" "),
            s(
                &format!("{} {}", kind.glyph(), kind.label()),
                p::alert_title(kind),
            ),
        ]));
        out.push(Line::from(vec![
            s(p::QUOTE_BAR, p::alert(kind)),
            r(" body text stays the default colour"),
        ]));
    }
    out.push(Line::raw(""));

    out.push(section("comment headers"));
    for (focused, t) in [
        (false, "  comment · 2026-09-06 · wtA "),
        (true, "▸ comment · 2026-09-06 · joel "),
    ] {
        let dashes = width.saturating_sub(t.chars().count());
        out.push(Line::from(vec![
            s(t, p::comment_header(focused)),
            s(&"─".repeat(dashes), p::rule()),
        ]));
    }
    out.push(Line::raw(""));

    out.push(section("editor (markers dimmed, text unchanged)"));
    let mut hl = crate::tui::markdown::Highlighter::new();
    for src in [
        "# Heading 1",
        "## Heading 2",
        "### Heading 3",
        "Some **strong**, *em* and `code` text",
        "- a bullet with **bold**",
        "> a quoted line",
        "```rust",
        "let x = 1;",
        "```",
    ] {
        out.push(editor_line(&mut hl, src));
    }
    out
}

/// Render every markdown style against the default (dark) and a light terminal
/// palette into `docs/assets/tui-md-palette.svg` (yaks-bc53). Ignored by
/// default; run explicitly: `cargo test -p yaks docshots_md_palette -- --ignored`.
#[test]
#[ignore = "asset generator; run with --ignored to refresh docs/assets/tui-md-palette.svg"]
fn docshots_md_palette() {
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::style::Color;
    use ratatui::text::Line;
    use ratatui::widgets::{Paragraph, Widget};

    let panel = 56u16;
    let lines = palette_swatch(panel as usize - 2);
    let h = lines.len() as u16 + 2;
    let area = Rect::new(0, 0, panel * 2, h);
    let mut buf = Buffer::empty(area);
    let themes = [
        (
            "dark terminal (default)",
            Rect::new(0, 0, panel, h),
            None::<Theme>,
        ),
        (
            "light terminal (VS Code Light+ ANSI)",
            Rect::new(panel, 0, panel, h),
            Some(Theme {
                bg: Color::Rgb(0xff, 0xff, 0xff),
                fg: Color::Rgb(0x33, 0x33, 0x33),
                ansi: light_ansi,
            }),
        ),
    ];
    for (title, rect, theme) in themes {
        let inner = Rect::new(rect.x + 1, rect.y + 1, rect.width - 2, rect.height - 1);
        Paragraph::new(Line::from(title))
            .render(Rect::new(inner.x, rect.y, inner.width, 1), &mut buf);
        Paragraph::new(lines.clone()).render(inner, &mut buf);
        if let Some(theme) = theme {
            paint_theme(&mut buf, rect, &theme);
        }
    }
    let svg = toque::buffer_to_svg(&buf);
    std::fs::create_dir_all("docs/assets").unwrap();
    std::fs::write("docs/assets/tui-md-palette.svg", svg).unwrap();
    eprintln!("wrote docs/assets/tui-md-palette.svg");
}

/// The markdown torture test (yaks-ea13's body plus a comment) in the detail
/// pane, at the top and scrolled to its code/table/comment part, into
/// `docs/assets/tui-md-render*.svg` (yaks-649d). Ignored by default; run
/// explicitly: `cargo test -p yaks docshots_md_render -- --ignored`.
#[test]
#[ignore = "asset generator; run with --ignored to refresh docs/assets/tui-md-render*.svg"]
fn docshots_md_render() {
    let mut y = t(
        "yaks-ea13",
        "Markdown rendering improvements",
        "task",
        3,
        Status::Hairy,
        None,
    );
    y.body = crate::store::append_note(
        include_str!("testdata/ea13.md").trim_end(),
        "2026-09-06T11:00:00Z",
        Some("joel"),
        "Any thing else you can think of in the markdown\nstandard (such as it is) and common extensions?",
    );
    let (w, h) = (110u16, 64u16);
    let mut app = App::new(vec![y]);
    HeadlessApp::on_resize(&mut app, w, h);
    press_key(&mut app, KeyCode::Enter);
    let _ = toque::render_to_svg(&app, w, h); // measure the pane width
    write_svg("tui-md-render", &app, w, h);
    press(&mut app, 'G'); // the code, table and comment at the end
    write_svg("tui-md-render-2", &app, w, h);
    // yaks-c733: a narrow pane, the line cursor on a long code row, scrolled
    // sideways (the table below is a separate block and stays put).
    let (w, h) = (72u16, 56u16);
    let mut app = App::new(vec![app.all[0].clone()]);
    HeadlessApp::on_resize(&mut app, w, h);
    press_key(&mut app, KeyCode::Enter);
    let _ = toque::render_to_svg(&app, w, h);
    let row = app
        .detail_dlines()
        .iter()
        .position(|l| l.text.contains("insanely verbose"))
        .expect("the fixture's long code line");
    app.detail_line = row;
    app.scroll_line_into_view(row as u16);
    for _ in 0..6 {
        press(&mut app, 'l');
    }
    app.detail_scroll = row.saturating_sub(6) as u16;
    write_svg("tui-md-render-scrolled", &app, w, h);
}

/// Fenced-code syntax colouring (yaks-3020): the read pane and the editor on a
/// yak with rust, sh and json blocks, into `docs/assets/tui-md-syntax*.svg`.
#[test]
#[ignore = "asset generator; run with --ignored to refresh docs/assets/tui-md-syntax*.svg"]
fn docshots_md_syntax() {
    let mut y = t(
        "yaks-1234",
        "Syntax colours in fenced code",
        "task",
        3,
        Status::Hairy,
        None,
    );
    y.body = r#"Fenced blocks are coloured by the language on the fence.

```rust
/// A lifetime, a string, a number and a block comment.
fn first<'a>(items: &'a [String], n: usize) -> Option<&'a str> {
    /* not yet:
       cached */
    let label = "first"; // trailing
    items.get(n.min(42)).map(|s| s.as_str())
}
```

```sh
# build and test
export TARGET="$HOME/target"
if cargo test --release; then echo "ok ${TARGET}"; fi
```

```json
{ "name": "yaks", "schema": 5, "ok": true, "deps": [null, -1.5e3] }
```

```brainfuck
+++[>++<-] unknown languages stay one flat colour
```"#
        .to_string();
    let (w, h) = (96u16, 34u16);
    let mut app = App::new(vec![y]);
    HeadlessApp::on_resize(&mut app, w, h);
    press_key(&mut app, KeyCode::Enter);
    let _ = toque::render_to_svg(&app, w, h);
    write_svg("tui-md-syntax", &app, w, h);
    // The same source in the in-place editor (`E` on a body row), cursor on
    // the first line so every code line is coloured and the fences stay dim.
    let row = app
        .detail_dlines()
        .iter()
        .position(|l| l.text.contains("Fenced blocks"))
        .expect("description row");
    app.detail_line = row;
    press(&mut app, 'E');
    write_svg("tui-md-syntax-edit", &app, w, h);
}

/// In-place block editing (yaks-597d): `E` on a comment row turns that comment
/// into an editor inside the detail pane, and `M` opens an empty one after the
/// last comment, into `docs/assets/tui-edit-inplace*.svg`. Ignored by default;
/// run explicitly: `cargo test -p yaks docshots_edit_inplace -- --ignored`.
#[test]
#[ignore = "asset generator; run with --ignored to refresh docs/assets/tui-edit-inplace*.svg"]
fn docshots_edit_inplace() {
    let (w, h) = (96u16, 30u16);
    let mut y = t(
        "yaks-5e11",
        "Login crashes on an expired token",
        "bug",
        2,
        Status::Shaving,
        None,
    );
    y.body = "Logging in with an expired token crashes the app.\n\n## Repro\n\n\
              1. Log in, wait an hour.\n2. Click anything."
        .into();
    y.body = crate::store::append_note(
        &y.body,
        "2026-09-06T11:40:00Z",
        Some("joel"),
        "Seen on staging too. The refresh path never runs when the token is already expired.",
    );
    y.body = crate::store::append_note(
        &y.body,
        "2026-09-06T11:58:00Z",
        Some("agent-1"),
        "Root cause: `refresh()` checks `exp` after the request, not before.\n\n- fix: check before\n- add a test",
    );
    let open = |y: &Task| {
        let mut app = App::new(vec![y.clone()]);
        HeadlessApp::on_resize(&mut app, w, h);
        view_to(&mut app, "status:shaving");
        press_key(&mut app, KeyCode::Enter);
        let _ = toque::render_to_svg(&app, w, h); // measure the pane width
        app
    };

    // `E` with the line cursor on the second comment's `fix:` row: the editor
    // opens on that source line, the rest of the pane stays rendered.
    let mut app = open(&y);
    let row = app
        .detail_dlines()
        .iter()
        .position(|l| l.text.contains("add a test"))
        .expect("the comment's list row");
    app.detail_line = row;
    press(&mut app, 'E');
    write_svg("tui-edit-inplace", &app, w, h);

    // `M`: an empty editor under a provisional `you · now` header.
    let mut app = open(&y);
    press(&mut app, 'M');
    press(&mut app, 'T');
    press(&mut app, 'h');
    press(&mut app, 'a');
    press(&mut app, 'n');
    press(&mut app, 'k');
    press(&mut app, 's');
    write_svg("tui-edit-inplace-new", &app, w, h);
}

/// The image viewer (yaks-4305): `Enter` on an image attachment draws it in the
/// pane (as half-block cells here; a graphics-capable terminal gets the real
/// pixels) into `docs/assets/tui-image-viewer.svg`. Ignored by default; run
/// explicitly: `cargo test -p yaks docshots_image_viewer -- --ignored`.
#[test]
#[ignore = "asset generator; run with --ignored to refresh docs/assets/tui-image-viewer.svg"]
fn docshots_image_viewer() {
    use crate::tui::test_support::temp_farm;

    // A 240×140 test card: a diagonal gradient with a white-ringed disc.
    let (iw, ih) = (240u32, 140u32);
    let mut px = Vec::new();
    for y in 0..ih {
        for x in 0..iw {
            let (dx, dy) = (x as f32 - 120.0, y as f32 - 70.0);
            let d = (dx * dx + dy * dy).sqrt();
            px.extend_from_slice(&if (44.0..50.0).contains(&d) {
                [255, 255, 255]
            } else if d < 44.0 {
                [240, 180, 40]
            } else {
                [(60 + x / 3) as u8, (40 + y) as u8, (200 - x / 2) as u8]
            });
        }
    }
    let mut png_bytes = Vec::new();
    let mut enc = png::Encoder::new(&mut png_bytes, iw, ih);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(&px).unwrap();

    let (_dir, farm) = temp_farm(&[t(
        "yak-1a2b",
        "Login page looks wrong on mobile",
        "bug",
        2,
        Status::Shaving,
        None,
    )]);
    farm.attach("yak-1a2b", "mobile-login.png", &png_bytes)
        .unwrap();
    let (w, h) = (110u16, 30u16);
    let mut app = App::with_farm(farm).unwrap();
    HeadlessApp::on_resize(&mut app, w, h);
    view_to(&mut app, "status:shaving");
    press(&mut app, 'l'); // focus the detail pane
    let _ = toque::render_to_svg(&app, w, h); // measure the pane width
    press_key(&mut app, KeyCode::Tab); // cursor onto the attachment link
    press_key(&mut app, KeyCode::Enter); // open it
    write_svg("tui-image-viewer", &app, w, h);
}

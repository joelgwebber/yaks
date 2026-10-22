use super::*;
use crate::tui::test_support::*;
use ratatui::style::{Color, Modifier};
use toque::HeadlessApp;

#[test]
fn mark_toggles_selection_and_renders_gutter_dot() {
    let mut app = App::new(vec![
        task("t0", "first", Status::Hairy, 3, None),
        task("t1", "second", Status::Hairy, 3, None),
    ]);
    // Nothing marked yet: no selection dot on screen.
    assert!(!navpos_frame(&app).contains('\u{25cf}'));
    // `m` marks the cursor's yak; the gutter shows a filled dot.
    handle_key(&mut app, key('m'));
    assert!(app.selected.contains("t0"));
    assert!(navpos_frame(&app).contains('\u{25cf}'));
    // `m` again unmarks it, and the dot disappears.
    handle_key(&mut app, key('m'));
    assert!(!app.selected.contains("t0"));
    assert!(!navpos_frame(&app).contains('\u{25cf}'));
}

#[test]
fn tree_with_ghost_family() {
    // Hairy tab under `all` scope: Root A + A1 (focus), A2 (ghost, shorn)
    // pulled in as family.
    let mut app = sample();
    app.family_scope
        .insert("status:hairy".into(), view::FamilyScope::All);
    insta::assert_snapshot!(draw(&app, 72, 14));
}

#[test]
fn family_indicator_on_tab_row() {
    // Wide enough for the right-aligned farm indicator to sit past the tabs.
    // The default Hairy view inherits the global default: shown as `~remaining`.
    insta::assert_snapshot!(draw(&sample(), 100, 8));
}

#[test]
fn collapsed_root_hides_children() {
    let mut app = sample();
    app.collapsed.insert("a0".into());
    insta::assert_snapshot!(draw(&app, 72, 14));
}

#[test]
fn build_universe_pulls_ghost_descendants() {
    let mut app = sample();
    // `all` scope pulls in every descendant, including the completed a2.
    app.family_scope
        .insert("status:hairy".into(), view::FamilyScope::All);
    let rows = app.rows();
    let ids: Vec<&str> = rows.iter().map(|r| r.task.id.as_str()).collect();
    assert_eq!(ids, vec!["a0", "a1", "a2"]); // b0 (shaving) excluded from Hairy tab
    let a2 = rows.iter().find(|r| r.task.id == "a2").unwrap();
    assert!(a2.ghost, "shorn child should be a ghost in the Hairy tab");
    let a0 = rows.iter().find(|r| r.task.id == "a0").unwrap();
    assert!(a0.has_children && !a0.ghost);
}

#[test]
fn family_remaining_default_hides_completed_leaf() {
    // The default scope (remaining) drops the shorn leaf a2; open a1 stays.
    let app = sample();
    let ids: Vec<&str> = app.rows().iter().map(|r| r.task.id.as_str()).collect();
    assert_eq!(ids, vec!["a0", "a1"]);
}

// -- overlay rendering (farm-less; opening only needs `selected`) ------

#[test]
fn state_picker_overlay() {
    let mut app = sample();
    handle_key(&mut app, key('S'));
    insta::assert_snapshot!(draw(&app, 72, 14));
}

#[test]
fn priority_picker_overlay() {
    let mut app = sample();
    handle_key(&mut app, key('P'));
    insta::assert_snapshot!(draw(&app, 72, 14));
}

#[test]
fn type_picker_overlay() {
    let mut app = sample();
    handle_key(&mut app, key('T'));
    insta::assert_snapshot!(draw(&app, 72, 14));
}

#[test]
fn create_targets_the_reference_herd_in_a_multi_herd_farm() {
    // Two herds present -> a new yak inherits the selected row's herd (prefix).
    let mut app = App::new(vec![
        task("api-0001", "api", Status::Hairy, 3, None),
        task("web-0001", "web", Status::Hairy, 3, None),
    ]);
    app.open_create(false);
    match &app.overlay {
        Overlay::Create(f) => assert_eq!(f.target_herd().as_deref(), Some("api")),
        _ => panic!("expected create overlay"),
    }
}

#[test]
fn is_multi_herd_reflects_distinct_id_prefixes() {
    let single = App::new(vec![
        task("yak-0001", "a", Status::Hairy, 3, None),
        task("yak-0002", "b", Status::Hairy, 3, None),
    ]);
    assert!(!single.is_multi_herd());
    let multi = App::new(vec![
        task("web-0001", "a", Status::Hairy, 3, None),
        task("api-0001", "b", Status::Hairy, 3, None),
    ]);
    assert!(multi.is_multi_herd());
    // Exercise the per-herd id-colour render path (colours don't show in the
    // text snapshot, so just assert it renders the ids without panicking).
    let frame = draw(&multi, 80, 12);
    assert!(frame.contains("web-0001") && frame.contains("api-0001"));
}

#[test]
fn create_form_herd_picker_cycles_the_target_in_a_multi_herd_farm() {
    let mut app = App::new(vec![
        task("api-0001", "a", Status::Hairy, 3, None),
        task("web-0001", "w", Status::Hairy, 3, None),
    ]);
    app.open_create(false);
    {
        let Overlay::Create(f) = &mut app.overlay else {
            panic!("expected create overlay")
        };
        assert!(
            f.has_herd_row(),
            "multi-herd farm shows the herd picker row"
        );
        assert_eq!(
            f.target_herd().as_deref(),
            Some("api"),
            "seeded to the selected row's herd"
        );
        f.row = HEADER_ROWS; // move the cursor onto the herd chip row
        f.move_chip(1);
        assert_eq!(f.target_herd().as_deref(), Some("web"));
    }
    // The herd chip row renders (its label is visible text).
    let frame = draw(&app, 80, 16);
    assert!(
        frame.contains("herd"),
        "herd chip row label renders:\n{frame}"
    );
}

#[test]
fn create_has_no_herd_target_in_a_single_herd_farm() {
    // One herd -> no explicit target; create falls through to the config default.
    let mut app = App::new(vec![
        task("yak-0001", "a", Status::Hairy, 3, None),
        task("yak-0002", "b", Status::Hairy, 3, None),
    ]);
    app.open_create(false);
    match &app.overlay {
        Overlay::Create(f) => assert!(f.target_herd().is_none()),
        _ => panic!("expected create overlay"),
    }
}

#[test]
fn slaughter_confirm_overlay() {
    // Move to a childless leaf (Child A1) so the confirm actually opens.
    let mut app = sample();
    handle_key(&mut app, key('j'));
    handle_key(&mut app, key('X'));
    assert!(matches!(app.overlay, Overlay::Confirm { .. }));
    insta::assert_snapshot!(draw(&app, 72, 14));
}

#[test]
fn slaughter_family_confirm_overlay() {
    // Root A has a live child (A1; A2 is shorn, also live), so X offers to
    // slaughter the whole family, naming the count (yaks-05da).
    let mut app = sample();
    handle_key(&mut app, key('X'));
    assert!(matches!(
        app.overlay,
        Overlay::Confirm {
            action: ConfirmAction::SlaughterFamily(_),
            ..
        }
    ));
    insta::assert_snapshot!(draw(&app, 72, 14));
}

fn open_body_editor(app: &mut App) {
    // A multiline comment editor, empty -> opens in Insert mode.
    app.overlay = Overlay::Edit(Editor::new(
        app.editor_vim,
        false,
        "Comment".into(),
        "",
        EditAction::Comment("a0".into()),
    ));
}

fn tab(app: &mut App) {
    handle_key(app, KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
}

#[test]
fn tab_after_prefix_autocompletes_and_replaces_the_typed_token() {
    let mut app = sample(); // default ref_prefix "yak"
    open_body_editor(&mut app);
    // Type a partial ref, then Tab: the picker opens seeded by the tail.
    typ(&mut app, "yak-a1");
    tab(&mut app);
    assert!(matches!(app.overlay, Overlay::Fuzzy(_)));
    // Commit; the whole "yak-a1" is replaced by the picked id.
    handle_key(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    match &app.overlay {
        Overlay::Edit(ed) => assert_eq!(ed.text(), "a1"),
        _ => panic!("expected to return to the editor"),
    }
}

#[test]
fn tab_in_a_content_block_without_context_does_not_navigate() {
    // Tab inside a content block is a normal editor key (Ctrl-N/P navigate
    // fields); it must not jump to the next field or open the picker.
    let mut app = sample();
    app.open_create(false);
    for _ in 0..8 {
        if matches!(&app.overlay, Overlay::Create(f) if f.is_content_row()) {
            break;
        }
        tab(&mut app);
    }
    assert!(matches!(&app.overlay, Overlay::Create(f) if f.is_content_row()));
    typ(&mut app, "hello");
    tab(&mut app);
    assert!(
        matches!(&app.overlay, Overlay::Create(f) if f.is_content_row()),
        "Tab in a content block must stay put, not navigate fields"
    );
}

#[test]
fn typing_prefix_dash_autocompletes_in_the_create_form_description() {
    // Regression: the create/edit form (Overlay::Create) is a separate key
    // path from the comment editor; the trigger must fire there too.
    let mut app = sample(); // default ref_prefix "yak"
    app.open_create(false);
    // Tab to the description content block (empty -> Insert mode).
    for _ in 0..8 {
        if matches!(&app.overlay, Overlay::Create(f) if f.is_content_row()) {
            break;
        }
        handle_key(&mut app, KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
    }
    assert!(
        matches!(&app.overlay, Overlay::Create(f) if f.is_content_row()),
        "expected to reach the description content block"
    );
    typ(&mut app, "yak-");
    tab(&mut app);
    assert!(
        matches!(app.overlay, Overlay::Fuzzy(_)),
        "Tab after the prefix in the create form should open the ref picker"
    );
}

#[test]
fn single_esc_from_normal_closes_the_picker() {
    let mut app = sample(); // vim editor profile by default
    open_body_editor(&mut app);
    typ(&mut app, "yak-");
    tab(&mut app);
    assert!(matches!(app.overlay, Overlay::Fuzzy(_)));
    // First Esc: Insert -> Normal in the query; the picker stays open.
    handle_key(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert!(
        matches!(app.overlay, Overlay::Fuzzy(_)),
        "first Esc should only drop the query to Normal"
    );
    // A *non-rapid* second Esc (clear the double-Esc timer) still closes it.
    app.last_esc = None;
    handle_key(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    match &app.overlay {
        Overlay::Edit(ed) => assert_eq!(ed.text(), "yak-"),
        _ => panic!("a second Esc from Normal should close the picker"),
    }
}

#[test]
fn ref_picker_cancel_keeps_the_typed_prefix() {
    let mut app = sample();
    open_body_editor(&mut app);
    typ(&mut app, "yak-");
    tab(&mut app);
    assert!(matches!(app.overlay, Overlay::Fuzzy(_)));
    // Ctrl-C cancels the picker; we return to the editor with "yak-" intact.
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
    );
    match &app.overlay {
        Overlay::Edit(ed) => assert_eq!(ed.text(), "yak-"),
        _ => panic!("expected to return to the editor"),
    }
}

#[test]
fn esc_cancels_picker() {
    let mut app = sample();
    handle_key(&mut app, key('P'));
    assert!(!matches!(app.overlay, Overlay::None));
    handle_key(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert!(matches!(app.overlay, Overlay::None));
    assert_eq!(app.notification.as_deref(), Some("cancelled"));
}

// -- editor overlay rendering (farm-less; open + render only) ----------

#[test]
fn label_editor_field() {
    // L opens a single-line field on the status line, seeded with labels.
    let mut app = editable();
    handle_key(&mut app, key('L'));
    assert!(matches!(app.overlay, Overlay::Edit(_)));
    insta::assert_snapshot!(draw(&app, 72, 14));
}

fn editor_state(app: &App) -> (EditorMode, String) {
    let ed = match &app.overlay {
        Overlay::Edit(ed) => ed,
        Overlay::BlockEdit(be) => &be.editor,
        _ => panic!("no editor overlay open"),
    };
    let st = ed.state.borrow();
    (st.mode, st.lines.to_string())
}

#[test]
fn x_and_shift_x_delete_chars() {
    // x deletes under the cursor, X deletes the previous char. (The yank to
    // the system clipboard is best-effort and not asserted here.)
    let mut app = editable();
    app.open_comment();
    typ(&mut app, "abcd");
    handle_key(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)); // Normal
    typ(&mut app, "0"); // col 0
    typ(&mut app, "x"); // remove 'a'
    assert_eq!(editor_state(&app).1, "bcd");
    typ(&mut app, "l"); // -> col 1 ('c')
    // X (Shift+X) removes the previous char; terminals send Shift with it.
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('X'), KeyModifiers::SHIFT),
    );
    assert_eq!(editor_state(&app).1, "cd");
}

#[test]
fn join_lines_vim_adds_space() {
    // Vim's J joins the next line onto this one with a single space, dropping
    // the next line's indent (yaks-7fa0).
    let enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    let mut app = editable();
    app.open_comment();
    typ(&mut app, "hello");
    handle_key(&mut app, enter);
    typ(&mut app, "    world");
    handle_key(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)); // Normal
    typ(&mut app, "gg"); // first line
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('J'), KeyModifiers::SHIFT),
    );
    assert_eq!(editor_state(&app).1, "hello world");
}

#[test]
fn enter_continues_lists_and_quotes_in_the_comment_editor() {
    // Insert-mode Enter continues `- `/`> ` prefixes and a second Enter on an
    // empty item drops the marker (edtui fork's markdown continuation,
    // yaks-1f84).
    let enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    let mut app = editable();
    app.open_comment();
    typ(&mut app, "- a");
    handle_key(&mut app, enter);
    typ(&mut app, "b");
    handle_key(&mut app, enter);
    assert_eq!(editor_state(&app).1, "- a\n- b\n- ");
    handle_key(&mut app, enter); // empty item: ends the list
    assert_eq!(editor_state(&app).1, "- a\n- b\n");
    typ(&mut app, "> q");
    handle_key(&mut app, enter);
    typ(&mut app, "r");
    assert_eq!(editor_state(&app).1, "- a\n- b\n> q\n> r");
}

#[test]
fn enter_does_not_continue_in_a_single_line_field() {
    let enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    let mut app = editable();
    handle_key(&mut app, key('L')); // single-line labels editor
    typ(&mut app, "- a");
    handle_key(&mut app, enter); // saves and closes: no continuation
    assert!(matches!(app.overlay, Overlay::None));
}

#[test]
fn single_line_vim_reaches_normal_mode() {
    // In vim mode a single-line field is modal: Esc switches to Normal and
    // never cancels, so the normal-mode keymap is usable; cancel is Ctrl-C
    // (or a rapid double-Esc).
    let mut app = editable();
    assert!(app.editor_vim);
    handle_key(&mut app, key('L')); // single-line labels editor (starts Insert)
    typ(&mut app, "aaa bbb");
    handle_key(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    // First Esc: still editing, now in Normal mode.
    let (mode, _) = editor_state(&app);
    assert_eq!(mode, EditorMode::Normal, "first Esc should enter Normal");
    // Normal-mode editing works: `dd` deletes the line.
    typ(&mut app, "dd");
    let (_, text) = editor_state(&app);
    assert_eq!(text, "", "normal-mode dd should clear the line");
    // Esc is modal in vim and never cancels on its own; Ctrl-C cancels.
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
    );
    assert!(matches!(app.overlay, Overlay::None), "Ctrl-C cancels");
}

#[test]
fn change_line_routes_through_editor() {
    // cc (change whole line) reaches edtui via the fork.
    let mut app = editable();
    app.open_comment();
    typ(&mut app, "hello");
    handle_key(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)); // Normal
    typ(&mut app, "cc"); // clear the line, enter Insert
    typ(&mut app, "bye");
    assert_eq!(editor_state(&app).1, "bye");
}

#[test]
fn substitute_routes_through_editor() {
    // s (substitute char) reaches edtui via the fork.
    let mut app = editable();
    app.open_comment();
    typ(&mut app, "abc");
    handle_key(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)); // Normal
    typ(&mut app, "0s"); // start of line, substitute 'a'
    typ(&mut app, "X");
    assert_eq!(editor_state(&app).1, "Xbc");
}

#[test]
fn tilde_x_and_r_route_through_editor() {
    let mut app = editable();
    app.open_comment();
    typ(&mut app, "hello");
    handle_key(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)); // Normal
    typ(&mut app, "0"); // to col 0
    typ(&mut app, "~"); // Hello, cursor -> col 1
    assert_eq!(editor_state(&app).1, "Hello");
    // Cursor is on col 1 after ~; X (Shift+X) deletes the previous char 'H'.
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('X'), KeyModifiers::SHIFT),
    );
    assert_eq!(editor_state(&app).1, "ello");
    typ(&mut app, "r"); // replace-char prefix
    typ(&mut app, "Y"); // 'e' -> 'Y'
    assert_eq!(editor_state(&app).1, "Yllo");
}

#[test]
fn editor_renders_markdown_highlights() {
    // Our hand-rolled highlighter runs at render for the editor; it must not
    // panic and must still show the text.
    let mut app = editable();
    app.open_comment();
    typ(&mut app, "# Heading");
    let out = draw(&app, 72, 14);
    assert!(out.contains("Heading"));
}

fn body_with_comments() -> Task {
    let mut t = task("c0", "Commented", Status::Hairy, 3, None);
    let b = crate::store::append_note(
        "The description.",
        "2026-01-01T00:00:00Z",
        None,
        "first note",
    );
    t.body = crate::store::append_note(&b, "2026-01-02T00:00:00Z", None, "second note");
    t
}

#[test]
fn vim_seeded_content_opens_in_normal_mode() {
    // Editing existing description/comment content lands in Normal (the vi
    // expectation when reviewing text).
    let mut app = App::new(vec![body_with_comments()]);
    assert!(app.editor_vim);
    app.open_block_edit(Some(0));
    assert_eq!(editor_state(&app).0, EditorMode::Normal);
    app.overlay = Overlay::None;
    app.open_block_edit(Some(1));
    assert_eq!(editor_state(&app).0, EditorMode::Normal);
}

#[test]
fn vim_new_content_opens_in_insert_mode() {
    // Fresh, empty content (new yak description, new comment) opens in Insert
    // so you can type immediately.
    let mut app = App::new(vec![task("t0", "t", Status::Hairy, 3, None)]);
    app.open_create(false);
    match &app.overlay {
        Overlay::Create(f) => assert_eq!(
            f.description.as_ref().unwrap().borrow().mode,
            EditorMode::Insert
        ),
        _ => panic!("expected create form"),
    }
    app.open_comment();
    assert_eq!(editor_state(&app).0, EditorMode::Insert);
}

#[test]
fn e_on_title_line_opens_form_on_the_title_row() {
    let mut app = App::new(vec![body_with_comments()]);
    handle_key(&mut app, key('l'));
    let lines = app.detail_dlines();
    let ln = lines
        .iter()
        .position(|l| l.text.starts_with("Title:"))
        .unwrap();
    app.detail_line = ln;
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('E'), KeyModifiers::NONE),
    );
    match &app.overlay {
        Overlay::Create(f) => assert_eq!(f.row, 0),
        _ => panic!("E should open the edit form"),
    }
}

#[test]
fn e_on_source_line_opens_form_on_the_source_row() {
    // yaks-b0b4: `E` on the detail pane's `Source:` line focuses the form's
    // source row, seeded with the current URL.
    let mut t = body_with_comments();
    t.source = Some("https://example.com/PROJ-9".into());
    let mut app = App::new(vec![t]);
    handle_key(&mut app, key('l'));
    let lines = app.detail_dlines();
    let ln = lines
        .iter()
        .position(|l| l.text.starts_with("Source:"))
        .unwrap();
    app.detail_line = ln;
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('E'), KeyModifiers::NONE),
    );
    match &app.overlay {
        Overlay::Create(f) => {
            assert_eq!(f.row, 4);
            assert_eq!(f.source_text(), "https://example.com/PROJ-9");
        }
        _ => panic!("E should open the edit form"),
    }
}

#[test]
fn ctrl_n_p_navigate_content_blocks_in_detail() {
    let mut app = App::new(vec![body_with_comments()]);
    handle_key(&mut app, key('l')); // detail; line cursor at top
    let starts = app.block_starts();
    let cn = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::CONTROL);
    let cp = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL);
    handle_key(&mut app, cn);
    assert_eq!(app.detail_line, starts[1]); // first comment
    handle_key(&mut app, cn);
    assert_eq!(app.detail_line, starts[2]); // second comment
    handle_key(&mut app, cp);
    assert_eq!(app.detail_line, starts[1]); // back to first
}

#[test]
fn unfocused_description_preview_wraps() {
    // The create form opens on the title row, so the description shows as the
    // dimmed preview. A long logical line must soft-wrap (like the focused
    // editor and the detail view) rather than truncate at the pane edge, so
    // its tail word stays visible.
    let mut app = App::new(vec![task("e0", "Editable", Status::Hairy, 3, None)]);
    app.open_create(false);
    let vim = app.editor_vim;
    match &mut app.overlay {
        Overlay::Create(f) => {
            f.description = Some(multiline_field(
                "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda",
                vim,
            ))
        }
        _ => panic!("create form should open"),
    }
    let out = draw(&app, 40, 20);
    assert!(
        out.contains("lambda"),
        "wrapped tail word should be visible:\n{out}"
    );
}

#[test]
fn editor_header_shows_mode_tag() {
    let mut app = editable();
    app.open_comment(); // multiline editor, starts Insert
    let insert_frame = draw(&app, 72, 14);
    assert!(insert_frame.contains("INSERT"), "insert mode tag in header");
    handle_key(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    let normal_frame = draw(&app, 72, 14);
    assert!(normal_frame.contains("NORMAL"), "normal mode tag after Esc");
}

#[test]
fn single_line_emacs_esc_cancels_immediately() {
    // With the emacs profile there is no Normal mode, so Esc must still
    // cancel a single-line field on the first press.
    let mut app = editable();
    app.editor_vim = false;
    handle_key(&mut app, key('L'));
    assert!(matches!(app.overlay, Overlay::Edit(_)));
    handle_key(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert!(matches!(app.overlay, Overlay::None), "emacs Esc cancels");
}

#[test]
fn create_form() {
    // Open the create form, type a title, then move to the priority chip row.
    let mut app = editable();
    handle_key(&mut app, key('c'));
    typ(&mut app, "new idea");
    handle_key(&mut app, KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)); // -> type
    handle_key(&mut app, KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)); // -> priority
    insta::assert_snapshot!(draw(&app, 72, 14));
}

#[test]
fn create_form_long_title_scrolls_horizontally() {
    // yaks-4da6: an overflowing single-line field scrolls horizontally (cursor
    // stays visible at the end) instead of wrapping onto a hidden second row
    // under the next field.
    let mut app = editable();
    handle_key(&mut app, key('c'));
    typ(
        &mut app,
        "a very long title that keeps going well past the edge of the form END",
    );
    let frame = draw(&app, 72, 14);
    let title = frame.lines().find(|l| l.contains("title")).unwrap();
    assert!(title.contains("END"), "tail stays visible:\n{frame}");
    assert!(
        !title.contains("a very long"),
        "head scrolled off:\n{frame}"
    );
    let ty = frame.lines().find(|l| l.contains("type")).unwrap();
    assert!(ty.contains("task"), "next row not overdrawn:\n{frame}");
    insta::assert_snapshot!(frame);
}

#[test]
fn create_form_source_field() {
    // yaks-b0b4: a single-line Source row sits under labels.
    let mut app = editable();
    handle_key(&mut app, key('c'));
    typ(&mut app, "linked");
    for _ in 0..4 {
        tab_key(&mut app); // -> type, priority, labels, source
    }
    typ(&mut app, "https://example.com/PROJ-1");
    match &app.overlay {
        Overlay::Create(f) => assert_eq!(f.source_text(), "https://example.com/PROJ-1"),
        _ => panic!("expected create form"),
    }
    insta::assert_snapshot!(draw(&app, 72, 14));
}

#[test]
fn help_overlay() {
    let mut app = sample();
    handle_key(&mut app, key('?'));
    insta::assert_snapshot!(draw(&app, 72, 16));
}

#[test]
fn priority_palette_matches_python() {
    // P1 red+bold, P2 magenta, P3 yellow, P4 green, P5 blue+dim (yaksrs-bce4).
    assert_eq!(priority_style(1).fg, Some(Color::Red));
    assert!(priority_style(1).add_modifier.contains(Modifier::BOLD));
    assert_eq!(priority_style(2).fg, Some(Color::Magenta));
    assert_eq!(priority_style(3).fg, Some(Color::Yellow));
    assert_eq!(priority_style(4).fg, Some(Color::Green));
    assert_eq!(priority_style(5).fg, Some(Color::Blue));
    assert!(priority_style(5).add_modifier.contains(Modifier::DIM));
}

#[test]
fn help_opens_and_closes() {
    let mut app = sample();
    handle_key(&mut app, key('?'));
    assert!(matches!(app.overlay, Overlay::Help(_)));
    // Scroll down, then Esc closes.
    handle_key(&mut app, key('j'));
    assert!(matches!(app.overlay, Overlay::Help(s) if s == 1));
    esc_key(&mut app);
    assert!(matches!(app.overlay, Overlay::None));
}

#[test]
fn edit_form_panel() {
    // E on the list opens the shared form seeded from the task: header fields
    // only (the body is edited in place in the detail pane), focus on title.
    let mut app = editable();
    handle_key(&mut app, key('E'));
    match &app.overlay {
        Overlay::Create(f) => assert!(f.is_editing()),
        _ => panic!("expected edit form"),
    }
    insta::assert_snapshot!(draw(&app, 72, 14));
}

fn typ(app: &mut App, s: &str) {
    for c in s.chars() {
        handle_key(app, key(c));
    }
}

#[test]
fn inline_search_field_and_recolor() {
    // '/' then "child" focuses the two matching children; Root A dims as
    // their ancestor, Root B is pruned.
    let mut app = sample();
    handle_key(&mut app, key('/'));
    typ(&mut app, "child");
    assert!(matches!(app.overlay, Overlay::Search(_)));
    insta::assert_snapshot!(draw(&app, 72, 14));
}

#[test]
fn inline_search_updates_filter_live() {
    let mut app = sample();
    handle_key(&mut app, key('/'));
    typ(&mut app, "child");
    assert_eq!(app.filter.search.as_deref(), Some("child"));
    let ids: Vec<&str> = app.rows().iter().map(|r| r.task.id.as_str()).collect();
    assert_eq!(ids, vec!["a0", "a1", "a2"]); // b0 pruned; a0 dimmed ancestor
    // Enter keeps the filter.
    handle_key(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert!(matches!(app.overlay, Overlay::None));
    assert_eq!(app.filter.search.as_deref(), Some("child"));
    // Esc in the list clears the active filter.
    handle_key(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert!(!app.filter.content_active());
}

#[test]
fn inline_search_ctrl_c_restores_previous() {
    // In vim a lone Esc enters Normal, so cancel-and-restore is Ctrl-C.
    let mut app = sample();
    handle_key(&mut app, key('/'));
    typ(&mut app, "zzz");
    assert_eq!(app.filter.search.as_deref(), Some("zzz"));
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
    );
    assert!(matches!(app.overlay, Overlay::None));
    assert!(app.filter.search.is_none());
}

#[test]
fn inline_search_esc_enters_normal_in_vim() {
    let mut app = sample();
    handle_key(&mut app, key('/'));
    typ(&mut app, "zz");
    handle_key(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    // Still open, now modal: the live filter is untouched.
    assert!(matches!(app.overlay, Overlay::Search(_)));
    if let Overlay::Search(sb) = &app.overlay {
        assert_eq!(sb.query.borrow().mode, EditorMode::Normal);
    }
    assert_eq!(app.filter.search.as_deref(), Some("zz"));
}

#[test]
fn drawer_text_row_esc_enters_normal_in_vim() {
    let mut app = sample();
    handle_key(&mut app, key('f')); // open filter drawer
    for _ in 0..3 {
        handle_key(&mut app, KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
    }
    typ(&mut app, "ui"); // into the labels text row
    handle_key(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert!(
        matches!(app.overlay, Overlay::Drawer(_)),
        "a lone Esc on a text row stays in the drawer"
    );
    if let Overlay::Drawer(d) = &app.overlay {
        assert_eq!(d.labels.borrow().mode, EditorMode::Normal);
    }
}

#[test]
fn drawer_normal_jk_navigates_and_carries_mode() {
    let mut app = sample();
    handle_key(&mut app, key('f'));
    for _ in 0..3 {
        handle_key(&mut app, KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
    }
    handle_key(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)); // labels -> Normal
    handle_key(&mut app, key('j')); // next text row, carrying Normal
    if let Overlay::Drawer(d) = &app.overlay {
        assert_eq!(d.row, 4);
        assert_eq!(d.search.borrow().mode, EditorMode::Normal);
    } else {
        panic!("drawer closed");
    }
    handle_key(&mut app, key('k')); // back up to labels, still Normal
    if let Overlay::Drawer(d) = &app.overlay {
        assert_eq!(d.row, 3);
        assert_eq!(d.labels.borrow().mode, EditorMode::Normal);
    }
}

#[test]
fn create_form_normal_j_navigates_rows() {
    let mut app = sample();
    handle_key(&mut app, key('c')); // create form, title row (Insert)
    handle_key(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)); // title -> Normal
    handle_key(&mut app, key('j')); // moves off the title row
    if let Overlay::Create(f) = &app.overlay {
        assert_eq!(f.row, 1);
    } else {
        panic!("form closed");
    }
    // In Insert, j types instead of navigating.
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE),
    ); // back to title (chip k-nav)
    handle_key(&mut app, key('i')); // ensure Insert on title
    handle_key(&mut app, key('j'));
    if let Overlay::Create(f) = &app.overlay {
        assert_eq!(f.row, 0, "j types in Insert, does not navigate");
        assert!(f.title_text().contains('j'));
    }
}

#[test]
fn filter_drawer_overlay() {
    // f opens the drawer; toggle the first status chip to show selection.
    let mut app = sample();
    handle_key(&mut app, key('f'));
    handle_key(&mut app, key(' ')); // toggle status=hairy (row 0, chip 0)
    assert!(matches!(app.overlay, Overlay::Drawer(_)));
    insta::assert_snapshot!(draw(&app, 72, 16));
}

#[test]
fn drawer_toggle_applies_live_and_commits() {
    let mut app = sample();
    handle_key(&mut app, key('f')); // row 0 (status)
    handle_key(&mut app, key('j')); // -> row 1 (type)
    handle_key(&mut app, key(' ')); // toggle type=task (chip 0)
    assert_eq!(app.filter.types, vec!["task".to_string()]); // live preview
    enter_key(&mut app); // apply
    assert!(matches!(app.overlay, Overlay::None));
    assert_eq!(app.filter.types, vec!["task".to_string()]);
}

#[test]
fn drawer_cancel_restores_saved() {
    let mut app = sample();
    handle_key(&mut app, key('f'));
    handle_key(&mut app, key('j')); // row 1 (type)
    handle_key(&mut app, key(' ')); // toggle task (live)
    assert!(app.filter.content_active());
    esc_key(&mut app); // cancel reverts
    assert!(matches!(app.overlay, Overlay::None));
    assert!(!app.filter.content_active());
}

#[test]
fn drawer_clear_empties_filter() {
    let mut app = sample();
    app.filter.types = vec!["bug".into()];
    handle_key(&mut app, key('f')); // seeded from the active filter
    handle_key(&mut app, key('C')); // clear all
    assert!(!app.filter.content_active());
}

#[test]
fn drawer_text_row_typing_sets_search() {
    let mut app = sample();
    handle_key(&mut app, key('f'));
    // Navigate to the search text row (row 4) with Down (works on all rows).
    for _ in 0..4 {
        down_key(&mut app);
    }
    typ(&mut app, "root");
    assert_eq!(app.filter.search.as_deref(), Some("root"));
    enter_key(&mut app);
    assert_eq!(app.filter.search.as_deref(), Some("root"));
}

#[test]
fn detail_shows_children_and_links() {
    let mut app = linked();
    enter_key(&mut app); // list Enter -> focus detail
    assert_eq!(app.focus, Focus::Detail);
    insta::assert_snapshot!(draw(&app, 72, 16));
}

#[test]
fn detail_jumplist_follows_to_task() {
    let mut app = linked();
    enter_key(&mut app); // enter detail
    assert_eq!(app.focus, Focus::Detail);
    // Jumplist order for a0: child a1, child a2, body a1, body a2.
    assert!(app.detail_jumps().len() >= 2);
    tab_key(&mut app); // line cursor -> first link line (a1)
    enter_key(&mut app); // follow it -> a1
    // We stay in the detail pane, now showing the followed task (yaksrs-3f19).
    assert_eq!(app.focus, Focus::Detail);
    assert_eq!(app.selected_id().as_deref(), Some("a1"));
    assert_eq!(app.notification.as_deref(), Some("→ a1"));
}

#[test]
fn follow_link_reveals_collapsed_target() {
    // a1 is hidden under a collapsed a0; following the link must expand a0,
    // select a1, and stay in the detail pane (yaksrs-3f19).
    let mut app = linked();
    app.collapsed.insert("a0".to_string());
    assert!(app.rows().iter().all(|r| r.task.id != "a1"), "a1 hidden");
    enter_key(&mut app); // focus detail on a0
    tab_key(&mut app); // -> first link line (a1)
    enter_key(&mut app); // follow it -> a1
    assert_eq!(app.focus, Focus::Detail);
    assert_eq!(app.selected_id().as_deref(), Some("a1"));
    assert!(!app.collapsed.contains("a0"), "ancestor expanded");
}

#[test]
fn follow_link_to_yak_outside_any_view_says_so() {
    // yaks-cc52: d0 is Dead and no pinned view shows Dead, so the cursor can't
    // land on it. Following the link must not leave the detail pane silently
    // showing a0 under a "→ d0" notice: goto_task says d0 isn't shown, and
    // records no nav history for the failed jump.
    let mut a0 = task("a0", "Root A", Status::Hairy, 2, None);
    a0.body = "see d0 for the old plan".into();
    let mut app = App::new(vec![a0, task("d0", "Dead plan", Status::Dead, 3, None)]);
    assert!(
        app.views.iter().all(|v| v.status != Some(Status::Dead)),
        "no pinned view for Dead"
    );
    enter_key(&mut app); // focus detail on a0
    tab_key(&mut app); // -> the d0 link line
    enter_key(&mut app); // follow it
    let frame = draw(&app, 72, 16);
    assert_eq!(
        app.notification.as_deref(),
        Some("d0 isn't shown in any view")
    );
    assert!(frame.contains("d0 isn't shown in any view"), "{frame}");
    assert!(!frame.contains("→ d0"), "{frame}");
    assert!(app.nav_back.is_empty(), "failed jump records no history");
}

#[test]
fn nav_history_back_and_forward() {
    // o/i retrace the link-follow chain (yaksrs-5d63).
    let mut app = linked();
    enter_key(&mut app); // detail on a0
    tab_key(&mut app); // -> first link line (a1)
    enter_key(&mut app); // follow it -> a1
    assert_eq!(app.selected_id().as_deref(), Some("a1"));
    handle_key(&mut app, key('o')); // back -> a0
    assert_eq!(app.selected_id().as_deref(), Some("a0"));
    assert_eq!(app.focus, Focus::Detail);
    handle_key(&mut app, key('i')); // forward -> a1
    assert_eq!(app.selected_id().as_deref(), Some("a1"));
    handle_key(&mut app, key('i')); // nothing further
    assert_eq!(app.notification.as_deref(), Some("no later yak"));
}

/// a0 and a1 are unrelated roots (so a0's only link is in its body) with long
/// bodies, so the detail pane scrolls; a0's link to a1 sits near its bottom.
fn linked_long() -> App {
    let filler: String = (1..=30).map(|i| format!("filler line {i}\n\n")).collect();
    let mut a0 = task("a0", "Root A", Status::Hairy, 2, None);
    a0.body = format!("{filler}now follow a1 please");
    let mut a1 = task("a1", "Other A1", Status::Hairy, 3, None);
    a1.body = filler;
    let mut app = App::new(vec![a0, a1]);
    app.detail_page = 16 - 3; // as runtime sets it for an 80x16 terminal
    app
}

/// Render at 80x16, blanking the transient notification row (it differs by
/// design: "→ a1" vs "← a0") so frames compare on the detail content alone.
fn navpos_frame(app: &App) -> String {
    let f = draw(app, 80, 16);
    let mut rows: Vec<&str> = f.lines().collect();
    rows[1] = "";
    rows.join("\n")
}

#[test]
fn nav_history_restores_cursor_and_scroll() {
    // o/i return to the detail cursor + scroll each yak was left at, not the
    // top (yaks-28b4).
    let mut app = linked_long();
    enter_key(&mut app); // detail on a0
    tab_key(&mut app); // -> the a1 link line, deep in the body (scrolls)
    let (a0_line, a0_scroll) = (app.detail_line, app.detail_scroll);
    assert!(a0_scroll > 0, "a0 scrolled down to its link");
    let frame_a0 = navpos_frame(&app);
    enter_key(&mut app); // follow -> a1, at the top
    assert_eq!(app.selected_id().as_deref(), Some("a1"));
    assert_eq!((app.detail_line, app.detail_scroll), (0, 0));
    press(&mut app, &"j".repeat(20)); // move a1's cursor down (scrolls)
    let (a1_line, a1_scroll) = (app.detail_line, app.detail_scroll);
    assert!(a1_scroll > 0, "a1 scrolled");
    let frame_a1 = navpos_frame(&app);
    handle_key(&mut app, key('o')); // back -> a0 where we left it
    assert_eq!(app.selected_id().as_deref(), Some("a0"));
    assert_eq!((app.detail_line, app.detail_scroll), (a0_line, a0_scroll));
    let frame_back = navpos_frame(&app);
    assert_eq!(frame_back, frame_a0, "back renders a0 exactly as left");
    handle_key(&mut app, key('i')); // forward -> a1 where we left it
    assert_eq!(app.selected_id().as_deref(), Some("a1"));
    assert_eq!((app.detail_line, app.detail_scroll), (a1_line, a1_scroll));
    assert_eq!(navpos_frame(&app), frame_a1, "forward renders a1 as left");
    if let Ok(dir) = std::env::var("YAKS_NAVPOS_FRAMES") {
        let out = format!(
            "== a0, scrolled to its a1 link ==\n{frame_a0}\n\n== a1 after following + j x20 ==\n{frame_a1}\n\n== o (back): a0 restored ==\n{frame_back}\n"
        );
        std::fs::write(std::path::Path::new(&dir).join("navpos-frames.txt"), out).unwrap();
    }
}

#[test]
fn nav_back_on_empty_history_is_noop() {
    let mut app = linked();
    enter_key(&mut app); // detail on a0, no history yet
    handle_key(&mut app, key('o'));
    assert_eq!(app.selected_id().as_deref(), Some("a0"));
    assert_eq!(app.notification.as_deref(), Some("no earlier yak"));
}

#[test]
fn detail_tab_cycles_link_lines() {
    // Tab snaps the line cursor onto successive link lines; Shift-Tab back.
    let mut app = linked();
    enter_key(&mut app);
    assert_eq!(app.detail_line, 0);
    tab_key(&mut app);
    let first = app.detail_line;
    assert!(first > 0, "moved onto a link line");
    tab_key(&mut app);
    assert!(app.detail_line > first, "advanced to the next link line");
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::BackTab, KeyModifiers::NONE),
    );
    assert_eq!(app.detail_line, first, "back to the first link line");
}

#[test]
fn detail_visual_selection_and_esc_clears() {
    let mut app = linked();
    enter_key(&mut app);
    assert_eq!(app.detail_anchor, None);
    handle_key(&mut app, key('v')); // anchor at the current line (0)
    assert_eq!(app.detail_anchor, Some(0));
    handle_key(&mut app, key('j')); // cursor -> 1
    handle_key(&mut app, key('j')); // cursor -> 2
    assert_eq!(app.selection_range(), Some((0, 2)));
    esc_key(&mut app); // Esc peels back the selection first
    assert_eq!(app.detail_anchor, None);
    assert_eq!(
        app.focus,
        Focus::Detail,
        "still in detail after clearing sel"
    );
}

/// A single yak whose body makes a long detail, entered with the detail focused.
fn long_detail_app(body_lines: usize) -> App {
    let mut t = task("d0", "Long", Status::Hairy, 3, None);
    t.body = (0..body_lines)
        .map(|i| format!("line {i}"))
        .collect::<Vec<_>>()
        // Hard breaks (trailing backslash), so each source line stays its own
        // row under CommonMark soft-break rendering (yaks-649d).
        .join("\\\n");
    let mut app = App::new(vec![t]);
    enter_key(&mut app);
    app
}

#[test]
fn scroll_into_view_is_stable_and_minimal() {
    let mut app = long_detail_app(60);
    draw(&app, 80, 13); // capture the detail width
    app.detail_page = 10; // viewport shows 10 rows (8 once the header + rule are pinned)
    app.detail_scroll = 5; // currently rows 5..14
    app.scroll_line_into_view(8); // already visible -> unchanged
    assert_eq!(app.detail_scroll, 5);
    app.scroll_line_into_view(2); // above -> scroll up to it
    assert_eq!(app.detail_scroll, 2);
    app.detail_scroll = 5;
    // Below -> land it on the last row that is actually visible: the pinned
    // header and its rule cost two rows, so only 8 content rows show
    // (yaks-15c7, yaks-66a3).
    app.scroll_line_into_view(20);
    assert_eq!(app.detail_scroll, 20 - (8 - 1));
}

#[test]
fn detail_last_line_is_visible_with_a_blank_row_below() {
    // yaks-15c7: scrolling to the end must show the last line, and leave a
    // blank row beneath it rather than abutting the viewport edge.
    let mut app = long_detail_app(40);
    let (w, h) = (80, 16);
    app.detail_page = h - 3; // as runtime sets it
    draw(&app, w, h);
    handle_key(&mut app, key('G'));
    let frame = draw(&app, w, h);
    println!("--- after G (yaks-15c7) ---\n{frame}");
    let rows: Vec<&str> = frame.lines().collect();
    // Rows: tabs, gap, detail area (h - 3 rows), help bar.
    let last = rows
        .iter()
        .position(|l| l.contains("line 39"))
        .unwrap_or_else(|| panic!("last line not on screen:\n{frame}"));
    let area_bottom = 2 + (h as usize - 3); // exclusive
    assert!(
        last + 1 < area_bottom,
        "last line abuts the bottom edge (row {last} of area ending {area_bottom}):\n{frame}"
    );
    assert!(
        rows[last + 1]
            .trim_start_matches(['\u{2502}', ' '])
            .is_empty(),
        "expected a blank row under the last line:\n{frame}"
    );
    // Stepping down from the top one line at a time also never loses the cursor.
    let mut app = long_detail_app(40);
    app.detail_page = h - 3;
    draw(&app, w, h);
    for _ in 0..80 {
        handle_key(&mut app, key('j'));
        let f = draw(&app, w, h);
        let cursor_row = app.detail_line as isize - app.detail_scroll as isize;
        let shown = app.detail_rows_shown(app.detail_scroll) as isize;
        assert!(
            (0..shown).contains(&cursor_row),
            "cursor off-screen at line {} scroll {}:\n{f}",
            app.detail_line,
            app.detail_scroll
        );
    }
    assert_eq!(app.detail_scroll, app.detail_max_scroll());
}

#[test]
fn pinned_header_has_a_rule_under_it_and_pushes_content_down_two_rows() {
    // yaks-66a3: once scrolled, the one-line header gets a dim `─` rule row.
    let mut app = long_detail_app(40);
    let (w, h) = (80, 16);
    app.detail_page = h - 3;
    draw(&app, w, h);
    handle_key(&mut app, key('G'));
    let frame = draw(&app, w, h);
    let rows: Vec<&str> = frame.lines().collect();
    let hdr = rows
        .iter()
        .position(|l| l.contains("d0") && l.contains("Long") && l.contains("p3"))
        .unwrap_or_else(|| panic!("no pinned header:\n{frame}"));
    assert!(
        rows[hdr + 1].contains(&"\u{2500}".repeat(20)),
        "no rule under the pinned header:\n{frame}"
    );
    assert!(
        rows[hdr + 2].contains("line "),
        "content should start right under the rule:\n{frame}"
    );
    assert_eq!(
        app.detail_rows_shown(app.detail_scroll),
        (h - 3 - 2) as usize
    );
    // The unpinned frame has no rule.
    app.detail_scroll = 0;
    let top = draw(&app, w, h);
    assert!(!top.contains(&"\u{2500}".repeat(20)), "{top}");
}

#[test]
fn full_detail_header_shows_the_status_emoji() {
    // yaks-66a3: the Status field carries the same icon as the pinned header.
    let app = long_detail_app(1);
    let frame = draw(&app, 80, 16);
    let row = frame
        .lines()
        .find(|l| l.contains("Status:"))
        .unwrap_or_else(|| panic!("no Status row:\n{frame}"));
    // (the text frame pads a wide glyph, so check the glyph and word apart)
    assert!(
        row.contains(Status::Hairy.emoji()) && row.contains("hairy"),
        "{row}"
    );
}

// -- view substrate (6b-i) --------------------------------------------

#[test]
fn starred_marker_and_tab_bar() {
    let mut app = sample();
    handle_key(&mut app, key('*')); // star a0
    insta::assert_snapshot!(draw(&app, 72, 14));
}

#[test]
fn default_views_and_tab_cycling() {
    let mut app = sample();
    assert_eq!(app.views.len(), 7); // 3 status + Recent + Inbox + Starred + Sheds
    assert_eq!(app.view, 0);
    assert_eq!(app.filter.statuses, vec![Status::Hairy]);
    tab_key(&mut app); // -> Shaving
    assert_eq!(app.view, 1);
    assert_eq!(app.filter.statuses, vec![Status::Shaving]);
}

#[test]
fn recent_view_is_flat_over_all_tasks() {
    let mut app = sample();
    // Recent is index 3 (after the 3 status views).
    app.set_view(3);
    assert_eq!(app.active_view().key, "recent");
    assert!(app.active_view().is_flat());
    let rows = app.rows();
    assert_eq!(rows.len(), 4); // all sample tasks, flat
    assert!(rows.iter().all(|r| r.depth == 0 && !r.ghost));
}

#[test]
fn star_toggles_and_starred_view_lists_it() {
    let mut app = sample(); // cursor on a0
    handle_key(&mut app, key('*'));
    assert!(app.is_starred("a0"));
    // Starred view (index 5) lists the starred task.
    app.set_view(5);
    assert_eq!(app.active_view().key, "working-set");
    let ids: Vec<&str> = app.rows().iter().map(|r| r.task.id.as_str()).collect();
    assert_eq!(ids, vec!["a0"]);
    // Unstar removes it.
    app.set_view(0);
    handle_key(&mut app, key('*'));
    assert!(!app.is_starred("a0"));
}

#[test]
fn save_view_creates_and_activates_custom_view() {
    let mut app = sample();
    handle_key(&mut app, key('/')); // inline search
    typ(&mut app, "child");
    enter_key(&mut app); // keep filter (search=child)
    handle_key(&mut app, key('V')); // save view
    typ(&mut app, "kids");
    enter_key(&mut app); // commit name
    assert_eq!(app.views.len(), 8);
    assert_eq!(app.view, 7);
    let v = app.active_view();
    assert_eq!(v.name, "kids");
    assert_eq!(v.spec.search.as_deref(), Some("child"));
    assert!(!v.builtin && v.pinned);
}

// -- view picker (6b-ii) ----------------------------------------------

#[test]
fn view_picker_overlay() {
    let mut app = sample();
    handle_key(&mut app, key('v'));
    assert!(matches!(app.overlay, Overlay::ViewPicker(_)));
    insta::assert_snapshot!(draw(&app, 72, 16));
}

#[test]
fn picker_activates_selected_view() {
    let mut app = sample();
    handle_key(&mut app, key('v'));
    for _ in 0..3 {
        handle_key(&mut app, key('j'));
    }
    enter_key(&mut app);
    assert!(matches!(app.overlay, Overlay::None));
    assert_eq!(app.active_view().key, "recent");
}

#[test]
fn picker_unpin_removes_from_tab_bar() {
    let mut app = sample();
    handle_key(&mut app, key('v'));
    handle_key(&mut app, key('j')); // sel 1 = Shaving
    handle_key(&mut app, key('p')); // unpin
    assert!(!app.views[1].pinned);
    assert!(!app.pinned_indices().contains(&1));
}

#[test]
fn picker_move_keeps_active_view() {
    let mut app = sample(); // active = Hairy (0)
    handle_key(&mut app, key('v'));
    handle_key(&mut app, key('J')); // move Hairy down
    assert_eq!(app.views[0].status, Some(Status::Shaving));
    assert_eq!(app.views[1].status, Some(Status::Hairy));
    assert_eq!(app.active_view().status, Some(Status::Hairy)); // followed the move
}

#[test]
fn picker_rename_returns_to_picker() {
    let mut app = sample();
    handle_key(&mut app, key('v'));
    handle_key(&mut app, key('r')); // seeded with the current name
    typ(&mut app, "Fuzz");
    enter_key(&mut app);
    assert!(app.views[0].name.contains("Fuzz") && app.views[0].name != "Hairy");
    assert!(matches!(app.overlay, Overlay::ViewPicker(0)));
}

#[test]
fn picker_deletes_custom_but_not_builtin() {
    let mut app = sample();
    // Create a custom view first.
    handle_key(&mut app, key('/'));
    typ(&mut app, "x");
    enter_key(&mut app);
    handle_key(&mut app, key('V'));
    typ(&mut app, "mine");
    enter_key(&mut app);
    assert_eq!(app.views.len(), 8);
    // Delete it via the picker (active view is the custom one, last index).
    handle_key(&mut app, key('v'));
    handle_key(&mut app, key('d'));
    assert_eq!(app.views.len(), 7);
    // Deleting a built-in is refused.
    handle_key(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    handle_key(&mut app, key('v'));
    // selection is clamped to a built-in row; delete refuses.
    handle_key(&mut app, key('g')); // no-op; ensure sel 0 path
    let before = app.views.len();
    // move selection to top then delete
    handle_key(&mut app, key('k'));
    handle_key(&mut app, key('k'));
    handle_key(&mut app, key('k'));
    handle_key(&mut app, key('k'));
    handle_key(&mut app, key('k'));
    handle_key(&mut app, key('d'));
    assert_eq!(app.views.len(), before);
    assert_eq!(
        app.notification.as_deref(),
        Some("can't delete a built-in view")
    );
}

#[test]
fn esc_reverts_modified_filter_to_view() {
    let mut app = sample();
    handle_key(&mut app, key('/'));
    typ(&mut app, "zzz");
    enter_key(&mut app);
    assert!(app.is_view_modified());
    handle_key(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert!(!app.is_view_modified());
    assert_eq!(app.filter.statuses, vec![Status::Hairy]); // back to Hairy view spec
}

#[test]
fn detail_find_matches_and_cycles() {
    let mut app = linked();
    enter_key(&mut app); // focus detail
    handle_key(&mut app, key('/')); // open detail find
    typ(&mut app, "child");
    assert_eq!(app.detail_find.as_deref(), Some("child"));
    assert!(app.detail_find_matches().len() >= 2); // both child lines
    enter_key(&mut app); // keep the find
    assert!(matches!(app.overlay, Overlay::None));
    assert_eq!(app.focus, Focus::Detail);
    assert_eq!(app.detail_match, 0);
    handle_key(&mut app, key('n'));
    assert_eq!(app.detail_match, 1);
    handle_key(&mut app, key('N'));
    assert_eq!(app.detail_match, 0);
}

#[test]
fn detail_body_wraps_at_narrow_width() {
    let mut t = task("a0", "Root A", Status::Hairy, 2, None);
    t.body = "alpha beta gamma delta epsilon zeta eta theta iota kappa".into();
    let mut app = App::new(vec![t]);
    enter_key(&mut app); // focus detail
    // Wide: the body fits on a single row.
    let _ = draw(&app, 200, 24);
    let wide = app.detail_line_count();
    // Narrow: the body must wrap into extra rows, and the trailing word
    // (off the right edge if it ran off unwrapped) stays visible.
    let out = draw(&app, 44, 24);
    let narrow = app.detail_line_count();
    assert!(
        narrow > wide,
        "narrow pane should wrap the body into more rows"
    );
    assert!(
        out.contains("kappa"),
        "the last body word should still be visible"
    );
}

#[test]
fn detail_find_ctrl_c_restores() {
    let mut app = linked();
    enter_key(&mut app);
    handle_key(&mut app, key('/'));
    typ(&mut app, "zzz");
    assert_eq!(app.detail_find.as_deref(), Some("zzz"));
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
    );
    assert!(matches!(app.overlay, Overlay::None));
    assert!(app.detail_find.is_none());
}

#[test]
fn detail_find_esc_enters_normal_in_vim() {
    let mut app = linked();
    enter_key(&mut app);
    handle_key(&mut app, key('/'));
    typ(&mut app, "zz");
    handle_key(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert!(matches!(app.overlay, Overlay::DetailFind(_)));
    if let Overlay::DetailFind(sb) = &app.overlay {
        assert_eq!(sb.query.borrow().mode, EditorMode::Normal);
    }
}

#[test]
fn detail_find_overlay() {
    let mut app = linked();
    enter_key(&mut app);
    handle_key(&mut app, key('/'));
    typ(&mut app, "child");
    insta::assert_snapshot!(draw(&app, 72, 16));
}

#[test]
fn detail_find_enter_lands_cursor_on_match() {
    // Enter commits the find on the current match (vi-like): the detail
    // cursor moves ONTO the match line and the scroll stays put, rather
    // than snapping back to where the cursor was before the search.
    let mut app = linked();
    enter_key(&mut app); // focus detail; cursor + scroll at the top
    assert_eq!(app.detail_line, 0);
    handle_key(&mut app, key('/'));
    typ(&mut app, "child"); // matches "Child A1" (line 2), "Child A2" (line 3)
    let m = app.detail_find_matches();
    assert!(m.len() >= 2);
    let match_line = m[app.detail_match].0;
    assert!(match_line > 0, "match should be below the pre-find cursor");
    assert_eq!(app.detail_scroll as usize, match_line); // find scrolled here
    enter_key(&mut app); // commit
    assert!(matches!(app.overlay, Overlay::None));
    assert_eq!(app.focus, Focus::Detail);
    // Cursor sits ON the current match; the find's scroll is preserved.
    assert_eq!(app.detail_line, match_line);
    assert_eq!(app.detail_scroll as usize, match_line);
}

#[test]
fn detail_find_enter_commits_the_cycled_match() {
    // Cycling with n before committing lands the cursor on that later
    // match, not always the first one.
    let mut app = linked();
    enter_key(&mut app);
    handle_key(&mut app, key('/'));
    typ(&mut app, "child");
    enter_key(&mut app); // commit on match 0
    handle_key(&mut app, key('n')); // advance to match 1
    assert_eq!(app.detail_match, 1);
    let second = app.detail_find_matches()[1].0;
    handle_key(&mut app, key('/')); // reopen (query + match index persist)
    enter_key(&mut app); // commit on the current match (1)
    assert!(matches!(app.overlay, Overlay::None));
    assert_eq!(app.detail_match, 1);
    assert_eq!(app.detail_line, second);
}

#[test]
fn detail_find_esc_restores_pre_find_position() {
    // Esc abandons the find and returns the cursor + scroll to where they
    // were before it opened (vi restores the origin on cancel).
    let mut app = linked();
    enter_key(&mut app); // focus detail
    handle_key(&mut app, key('j')); // move the cursor off the top
    let origin_line = app.detail_line;
    let origin_scroll = app.detail_scroll;
    assert!(origin_line > 0);
    handle_key(&mut app, key('/'));
    typ(&mut app, "child"); // scrolls the pane to the first match
    assert!(app.detail_find_matches().len() >= 2);
    assert_ne!(
        app.detail_scroll, origin_scroll,
        "find should move the view"
    );
    // Vim single-line field: first Esc drops to Normal, the rapid second
    // Esc is the cancel gesture.
    handle_key(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    handle_key(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert!(matches!(app.overlay, Overlay::None));
    assert!(app.detail_find.is_none());
    assert_eq!(app.detail_line, origin_line);
    assert_eq!(app.detail_scroll, origin_scroll);
}

#[test]
fn dep_picker_overlay() {
    // D on Root A: results (self excluded) in the right pane, query at foot.
    let mut app = sample();
    handle_key(&mut app, key('D'));
    assert!(matches!(app.overlay, Overlay::Fuzzy(_)));
    insta::assert_snapshot!(draw(&app, 72, 14));
}

#[test]
fn reparent_picker_overlay() {
    // R on Child A1 (has a parent) shows the clear-parent row first.
    let mut app = sample();
    handle_key(&mut app, key('j')); // move to a1
    handle_key(&mut app, key('R'));
    assert!(matches!(app.overlay, Overlay::Fuzzy(_)));
    insta::assert_snapshot!(draw(&app, 72, 14));
}

#[test]
fn needs_badge_renders_on_a_blocked_row() {
    // A yak carrying a `needs` block gets the ⏳ badge; an unblocked one
    // does not (the badge is per-row, on the right).
    let mut blocked = task("t0", "waiting on a human", Status::Hairy, 3, None);
    blocked.needs = Some("human".into());
    let app = App::new(vec![blocked, task("t1", "free", Status::Hairy, 3, None)]);
    let out = draw(&app, 80, 8);
    assert!(out.contains('\u{23f3}'), "hourglass badge present:\n{out}");
}

/// `needs: agent` (answered) is not awaiting a human: no hourglass, no
/// warning accent in the detail pane, and `a` asks (follow-up) instead of
/// answering; the row carries the answered badge instead.
#[test]
fn answered_yak_is_not_rendered_as_awaiting_a_human() {
    let mut answered = task("t0", "answered already", Status::Hairy, 3, None);
    answered.needs = Some("agent".into());
    let mut app = App::new(vec![answered]);
    let out = draw(&app, 80, 8);
    assert!(!out.contains('\u{23f3}'), "no hourglass:\n{out}");
    assert!(out.contains('\u{2705}'), "answered badge:\n{out}");
    app.open_ask_or_answer();
    match &app.overlay {
        Overlay::Edit(ed) => assert!(matches!(ed.action, EditAction::Ask(_))),
        _ => panic!("expected the ask prompt"),
    }
}

#[test]
fn inbox_view_lists_needs_blocked_across_statuses() {
    // The Inbox built-in view is a flat list of every yak awaiting a human,
    // regardless of status (a shorn-but-blocked yak still surfaces). It's
    // driven by the shared `needs_only` predicate, replacing the old modal
    // `i` toggle.
    let mut hairy = task("t0", "ask me", Status::Hairy, 3, None);
    hairy.needs = Some("human".into());
    let mut shorn = task("t1", "still blocked", Status::Shorn, 3, None);
    shorn.needs = Some("human".into());
    let free = task("t2", "unblocked", Status::Hairy, 3, None);
    let mut app = App::new(vec![hairy, shorn, free]);
    let idx = app
        .views
        .iter()
        .position(|v| v.key == "inbox")
        .expect("inbox view present in defaults");
    app.set_view(idx);
    assert!(app.active_view().is_flat());
    let mut ids: Vec<String> = app.rows().iter().map(|r| r.task.id.clone()).collect();
    ids.sort();
    assert_eq!(ids, vec!["t0".to_string(), "t1".to_string()]);
    // The free (unblocked) yak never appears in the inbox.
    assert!(!ids.contains(&"t2".to_string()));
    // The tab/picker count reflects the awaiting-a-human set, not the farm.
    assert_eq!(app.view_count(app.active_view()), 2);
    // Switching to the Hairy view drops the shorn yak again — inbox is a
    // view/filter, not a persistent override.
    app.set_view(0);
    let ids: Vec<String> = app.rows().iter().map(|r| r.task.id.clone()).collect();
    assert!(ids.contains(&"t0".to_string()) && ids.contains(&"t2".to_string()));
    assert!(
        !ids.contains(&"t1".to_string()),
        "shorn hidden in Hairy view"
    );
}

#[test]
fn drawer_inbox_chip_composes_with_status_scope() {
    // The inbox facet is the third chip in the drawer's `deps` row: toggling
    // it sets `needs_only` on the built spec without disturbing the rest of
    // the filter, so the inbox composes with any status scope.
    let base = FilterSpec {
        statuses: vec![Status::Hairy],
        ..Default::default()
    };
    let mut d = Drawer::from_filter(false, &base, Vec::new());
    d.row = 6; // deps row
    d.chip_idx = 2; // inbox chip
    d.toggle_chip();
    let spec = d.build_spec();
    assert!(spec.needs_only);
    assert_eq!(spec.statuses, vec![Status::Hairy], "status scope preserved");
    // Toggling it back off round-trips cleanly.
    let mut d2 = Drawer::from_filter(false, &spec, Vec::new());
    d2.row = 6;
    d2.chip_idx = 2;
    d2.toggle_chip();
    assert!(!d2.build_spec().needs_only);
}

#[test]
fn drawer_herd_chip_scopes_the_filter_in_a_multi_herd_farm() {
    // On a multi-herd farm the drawer grows a `herd` chip row (mirroring the
    // status row): toggling a chip sets FilterSpec.herds without disturbing the
    // rest of the filter. A single-herd farm keeps the fixed 8-row layout.
    let mut app = App::new(vec![
        task("api-0001", "a", Status::Hairy, 3, None),
        task("web-0001", "w", Status::Hairy, 3, None),
    ]);
    app.open_drawer();
    let Overlay::Drawer(d) = &mut app.overlay else {
        panic!("expected drawer overlay")
    };
    // The gated herd row is the last navigable row on a multi-herd farm, and it
    // is populated from `App::herd_choices()` (prefixes present, sorted).
    assert_eq!(d.row_count(), 8, "multi-herd farm adds the herd row");
    assert_eq!(d.herd_choices, vec!["api".to_string(), "web".to_string()]);
    d.row = d.row_count() - 1; // herd row
    d.chip_idx = 1; // "web"
    d.toggle_chip();
    let spec = d.build_spec();
    assert_eq!(spec.herds, vec!["web".to_string()]);
    // Toggling the same chip back off round-trips to an unconstrained scope.
    d.toggle_chip();
    assert!(d.build_spec().herds.is_empty());

    // A single-herd farm has no herd row (no layout / snapshot change).
    let mut single = App::new(vec![
        task("yak-0001", "a", Status::Hairy, 3, None),
        task("yak-0002", "b", Status::Hairy, 3, None),
    ]);
    single.open_drawer();
    let Overlay::Drawer(d) = &single.overlay else {
        panic!("expected drawer overlay")
    };
    assert!(!d.multi_herd());
    assert_eq!(d.row_count(), 7, "single-herd farm keeps the fixed rows");
}

// -- live mutation through a temp farm --------------------------------

mod live {
    use super::*;
    use std::fs;

    fn enter(app: &mut App) {
        handle_key(app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    }

    fn tab(app: &mut App) {
        handle_key(app, KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
    }

    fn back_tab(app: &mut App) {
        handle_key(app, KeyEvent::new(KeyCode::BackTab, KeyModifiers::NONE));
    }

    fn arrow_right(app: &mut App) {
        handle_key(app, KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
    }

    fn arrow_left(app: &mut App) {
        handle_key(app, KeyEvent::new(KeyCode::Left, KeyModifiers::NONE));
    }

    fn ctrl_s(app: &mut App) {
        handle_key(
            app,
            KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL),
        );
    }

    fn esc(app: &mut App) {
        handle_key(app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    }

    #[test]
    fn state_pick_transitions_and_reloads() {
        let (_dir, farm) = temp_farm(&[task("t0", "solo", Status::Hairy, 3, None)]);
        let mut app = App::with_farm(farm).unwrap();
        // S then n → the optional shear comment; Enter (empty) → shorn.
        press(&mut app, "Sn");
        enter(&mut app);
        assert!(matches!(app.overlay, Overlay::None));
        let t = app.task("t0").unwrap();
        assert_eq!(t.status, Status::Shorn);
        assert_eq!(app.notification.as_deref(), Some("t0 → shorn"));
    }

    #[test]
    fn state_pick_shear_records_the_optional_comment() {
        // yaks-27c6: shearing via the picker prompts for a comment; a typed
        // one lands as a note on the (now shorn) yak.
        let (_dir, farm) = temp_farm(&[task("t0", "solo", Status::Hairy, 3, None)]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "Sn");
        match &app.overlay {
            Overlay::Edit(ed) => assert!(matches!(ed.action, EditAction::Shear(_))),
            _ => panic!("expected the shear-comment prompt"),
        }
        // The prompt alone shears nothing.
        assert_eq!(app.task("t0").unwrap().status, Status::Hairy);
        press(&mut app, "done via tui");
        enter(&mut app);
        assert!(matches!(app.overlay, Overlay::None));
        let t = app.task("t0").unwrap();
        assert_eq!(t.status, Status::Shorn);
        assert!(t.body.contains("done via tui"), "{}", t.body);
        assert_eq!(app.notification.as_deref(), Some("t0 → shorn"));
    }

    #[test]
    fn state_pick_shear_with_empty_comment_adds_no_note() {
        let (_dir, farm) = temp_farm(&[task("t0", "solo", Status::Hairy, 3, None)]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "Sn");
        press(&mut app, "   "); // whitespace only counts as empty
        enter(&mut app);
        let t = app.task("t0").unwrap();
        assert_eq!(t.status, Status::Shorn);
        // Only the transition's own `moved:` note, no comment.
        assert_eq!(t.body.matches("▸ ").count(), 1, "{}", t.body);
    }

    #[test]
    fn state_pick_shear_cancel_leaves_the_yak() {
        let (_dir, farm) = temp_farm(&[task("t0", "solo", Status::Hairy, 3, None)]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "Sn");
        handle_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
        );
        assert!(matches!(app.overlay, Overlay::None));
        assert_eq!(app.task("t0").unwrap().status, Status::Hairy);
    }

    #[test]
    fn bulk_shear_comment_goes_on_every_moved_yak() {
        let (_dir, farm) = temp_farm(&[
            task("t0", "one", Status::Hairy, 3, None),
            task("t1", "two", Status::Shorn, 3, None),
            task("t2", "three", Status::Hairy, 3, None),
        ]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "mjm"); // mark the two hairy rows (t1 is hidden)
        app.selected.insert("t1".to_string()); // and the already-shorn one
        assert_eq!(app.selected.len(), 3);
        press(&mut app, "Sn");
        press(&mut app, "batch done");
        enter(&mut app);
        for id in ["t0", "t2"] {
            let t = app.task(id).unwrap();
            assert_eq!(t.status, Status::Shorn);
            assert!(t.body.contains("batch done"), "{id}: {}", t.body);
        }
        // t1 was already shorn: no move, so no note.
        assert!(!app.task("t1").unwrap().body.contains("batch done"));
        assert_eq!(app.notification.as_deref(), Some("2/3 → shorn"));
    }

    #[test]
    fn bulk_state_transition_shorns_the_marked_set() {
        let (_dir, farm) = temp_farm(&[
            task("t0", "one", Status::Hairy, 3, None),
            task("t1", "two", Status::Hairy, 3, None),
            task("t2", "three", Status::Hairy, 3, None),
        ]);
        let mut app = App::with_farm(farm).unwrap();
        // Mark all three rows (m, j, m, j, m) -- order-independent.
        press(&mut app, "mjmjm");
        assert_eq!(app.selected.len(), 3);
        // `S` with marks opens the bulk picker; `n` + Enter (no comment)
        // shorns the whole set.
        press(&mut app, "Sn");
        enter(&mut app);
        assert!(matches!(app.overlay, Overlay::None));
        for id in ["t0", "t1", "t2"] {
            assert_eq!(app.task(id).unwrap().status, Status::Shorn);
        }
        // The selection is cleared and the count is reported.
        assert!(app.selected.is_empty());
        assert_eq!(app.notification.as_deref(), Some("3/3 → shorn"));
    }

    #[test]
    fn bulk_slaughter_skips_yaks_with_children() {
        // p0 has a live child (c0); t0 is childless. Marking both and bulk-
        // slaughtering must skip the parent (would orphan c0) and only slay
        // the childless one -- mirroring the single-path guard (yaks-5c51).
        let (_dir, farm) = temp_farm(&[
            task("p0", "parent", Status::Hairy, 3, None),
            task("c0", "child", Status::Hairy, 3, Some("p0")),
            task("t0", "solo", Status::Hairy, 3, None),
        ]);
        let mut app = App::with_farm(farm).unwrap();
        // Mark p0 and t0 (skip over the child row c0 between them).
        press(&mut app, "mjjm");
        assert_eq!(app.selected.len(), 2);
        // `S` opens the bulk picker; `x` slaughters the set.
        press(&mut app, "Sx");
        assert!(matches!(app.overlay, Overlay::None));
        // The parent survives (skipped); the childless yak is dead.
        assert_eq!(app.task("p0").unwrap().status, Status::Hairy);
        assert_eq!(app.task("c0").unwrap().status, Status::Hairy);
        assert_eq!(app.task("t0").unwrap().status, Status::Dead);
        assert!(app.selected.is_empty());
        assert_eq!(
            app.notification.as_deref(),
            Some("1/2 → dead · 1 skipped: have children")
        );
    }

    #[test]
    fn detail_pane_mirrors_multi_select_mark() {
        // `m` marks the cursor's yak from the detail pane too (mirrored from
        // the list pane; yaks-5c51). The mark set is shared with the list.
        let (_dir, farm) = temp_farm(&[task("t0", "solo", Status::Hairy, 3, None)]);
        let mut app = App::with_farm(farm).unwrap();
        // `l` focuses the detail pane, then `m` marks t0.
        press(&mut app, "lm");
        assert!(matches!(app.focus, Focus::Detail));
        assert!(app.selected.contains("t0"));
        // `m` again unmarks it.
        press(&mut app, "m");
        assert!(app.selected.is_empty());
    }

    #[test]
    fn priority_pick_updates_and_reloads() {
        let (_dir, farm) = temp_farm(&[task("t0", "solo", Status::Hairy, 3, None)]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "P1");
        assert_eq!(app.task("t0").unwrap().priority, 1);
        assert_eq!(app.notification.as_deref(), Some("t0 → p1"));
    }

    #[test]
    fn type_pick_updates_and_reloads() {
        let (_dir, farm) = temp_farm(&[task("t0", "solo", Status::Hairy, 3, None)]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "Tb");
        assert_eq!(app.task("t0").unwrap().kind, "bug");
        assert_eq!(app.notification.as_deref(), Some("t0 → bug"));
    }

    #[test]
    fn herd_pick_moves_yak_to_another_herd() {
        // A multi-herd farm: two distinct id prefixes (`web`, `api`). Moving
        // `web-0001` to the `api` herd renames it `api-0001`, keeping the tail
        // (yaks-71d1). Distinct tails avoid a collision with the seeded api yak.
        let (_dir, farm) = temp_farm(&[
            task("web-0001", "web one", Status::Hairy, 3, None),
            task("api-0002", "api two", Status::Hairy, 3, None),
        ]);
        let mut app = App::with_farm(farm).unwrap();
        assert!(app.is_multi_herd());
        app.select_id("web-0001");
        // `H` opens the herd picker; candidates are herd_choices() minus the
        // current herd -> just ["api"], so `1` picks it.
        press(&mut app, "H1");
        assert!(matches!(app.overlay, Overlay::None));
        // The yak now lives in the `api` herd, tail preserved, and still exists.
        assert!(app.task("web-0001").is_none(), "old id is gone");
        assert!(app.task("api-0001").is_some(), "renamed into the api herd");
        assert_eq!(app.notification.as_deref(), Some("moved to api"));
    }

    #[test]
    fn herd_pick_works_from_the_detail_pane() {
        // The detail pane is the "full edit UI"; H must move the herd there too,
        // not only from the list (yaks-71d1). Enter detail with `l`, then `H1`.
        let (_dir, farm) = temp_farm(&[
            task("web-0001", "web one", Status::Hairy, 3, None),
            task("api-0002", "api two", Status::Hairy, 3, None),
        ]);
        let mut app = App::with_farm(farm).unwrap();
        app.select_id("web-0001");
        press(&mut app, "lH1");
        assert!(
            app.task("api-0001").is_some(),
            "moved to the api herd from detail"
        );
        assert_eq!(app.notification.as_deref(), Some("moved to api"));
    }

    #[test]
    fn herd_pick_single_herd_is_a_hint_noop() {
        // Herds only exist in a multi-herd farm; a single-herd farm no-ops with
        // a hint rather than opening an empty picker.
        let (_dir, farm) = temp_farm(&[task("t0", "solo", Status::Hairy, 3, None)]);
        let mut app = App::with_farm(farm).unwrap();
        assert!(!app.is_multi_herd());
        press(&mut app, "H");
        assert!(matches!(app.overlay, Overlay::None));
        assert_eq!(
            app.notification.as_deref(),
            Some("herds need a multi-herd farm")
        );
    }

    #[test]
    fn slaughter_confirm_moves_to_dead() {
        let (_dir, farm) = temp_farm(&[task("t0", "solo", Status::Hairy, 3, None)]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "Xy");
        // Dead stays in the model (so deps/ancestors still resolve) but is
        // hidden from the default Hairy view.
        assert_eq!(app.task("t0").unwrap().status, Status::Dead);
        assert!(app.rows().iter().all(|r| r.task.id != "t0"));
        assert_eq!(app.notification.as_deref(), Some("slaughtered t0"));
    }

    #[test]
    fn dead_is_loaded_but_hidden_until_filtered() {
        let (_dir, farm) = temp_farm(&[
            task("t0", "alive", Status::Hairy, 3, None),
            task("t1", "slain", Status::Dead, 3, None),
        ]);
        let mut app = App::with_farm(farm).unwrap();
        // Loaded into the model, but not shown on the default Hairy view.
        assert_eq!(app.task("t1").unwrap().status, Status::Dead);
        assert!(app.rows().iter().all(|r| r.task.id != "t1"));
        // Filtering to Dead surfaces it in the tree.
        app.filter.statuses = vec![Status::Dead];
        let ids: Vec<&str> = app.rows().iter().map(|r| r.task.id.as_str()).collect();
        assert_eq!(ids, vec!["t1"]);
        // Recent (flat) also hides dead by default.
        app.set_view(3);
        assert_eq!(app.active_view().key, "recent");
        assert!(app.rows().iter().all(|r| r.task.id != "t1"));
    }

    #[test]
    fn fuzzy_picker_esc_enters_normal_then_ctrl_c_cancels() {
        let (_dir, farm) = temp_farm(&[
            task("t0", "solo", Status::Hairy, 3, None),
            task("t1", "other", Status::Hairy, 3, None),
        ]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "D"); // dependency picker on the selected task
        assert!(matches!(app.overlay, Overlay::Fuzzy(_)));
        esc(&mut app); // a lone Esc drops the query to Normal, not cancel
        assert!(matches!(app.overlay, Overlay::Fuzzy(_)));
        if let Overlay::Fuzzy(fp) = &app.overlay {
            assert_eq!(fp.query.borrow().mode, EditorMode::Normal);
        }
        handle_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
        );
        assert!(matches!(app.overlay, Overlay::None));
    }

    #[test]
    fn fuzzy_normal_jk_moves_selection() {
        let (_dir, farm) = temp_farm(&[
            task("t0", "solo", Status::Hairy, 3, None),
            task("t1", "aaa", Status::Hairy, 3, None),
            task("t2", "bbb", Status::Hairy, 3, None),
        ]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "D"); // dependency picker on t0
        esc(&mut app); // Normal
        let sel0 = match &app.overlay {
            Overlay::Fuzzy(fp) => fp.sel,
            _ => panic!("picker closed"),
        };
        press(&mut app, "j");
        let sel1 = match &app.overlay {
            Overlay::Fuzzy(fp) => fp.sel,
            _ => panic!("picker closed"),
        };
        assert_eq!(sel1, sel0 + 1, "j moves the selection down");
        press(&mut app, "k");
        let sel2 = match &app.overlay {
            Overlay::Fuzzy(fp) => fp.sel,
            _ => panic!("picker closed"),
        };
        assert_eq!(sel2, sel0, "k moves it back up");
    }

    #[test]
    fn slaughter_family_confirm_moves_family_to_dead() {
        // A parent with a live child and grandchild: X raises the family
        // confirm; `y` slaughters all three, deepest first (yaks-05da).
        let (_dir, farm) = temp_farm(&[
            task("t0", "parent", Status::Hairy, 3, None),
            task("t1", "child", Status::Shaving, 3, Some("t0")),
            task("t2", "grandchild", Status::Hairy, 3, Some("t1")),
            task("t3", "bystander", Status::Hairy, 3, None),
        ]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "X");
        match &app.overlay {
            Overlay::Confirm { prompt, .. } => {
                assert!(prompt.contains("AND its 2 live descendants"), "{prompt}")
            }
            _ => panic!("expected the family confirm"),
        }
        press(&mut app, "y");
        for id in ["t0", "t1", "t2"] {
            assert_eq!(app.task(id).unwrap().status, Status::Dead, "{id}");
        }
        assert_eq!(app.task("t3").unwrap().status, Status::Hairy);
        assert_eq!(
            app.notification.as_deref(),
            Some("slaughtered t0 + family (3 yaks)")
        );
    }

    #[test]
    fn slaughter_family_declined_keeps_family() {
        let (_dir, farm) = temp_farm(&[
            task("t0", "parent", Status::Hairy, 3, None),
            task("t1", "child", Status::Hairy, 3, Some("t0")),
        ]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "Xn");
        assert_eq!(app.task("t0").unwrap().status, Status::Hairy);
        assert_eq!(app.task("t1").unwrap().status, Status::Hairy);
    }

    #[test]
    fn slaughter_declined_keeps_task() {
        let (_dir, farm) = temp_farm(&[task("t0", "solo", Status::Hairy, 3, None)]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "Xn");
        assert_eq!(app.task("t0").unwrap().status, Status::Hairy);
        assert_eq!(app.notification.as_deref(), Some("cancelled"));
    }

    #[test]
    fn state_pick_same_status_is_noop() {
        let (_dir, farm) = temp_farm(&[task("t0", "solo", Status::Hairy, 3, None)]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "Sh");
        assert_eq!(app.task("t0").unwrap().status, Status::Hairy);
        assert_eq!(app.notification.as_deref(), Some("t0 already hairy"));
    }

    #[test]
    fn create_root_via_form() {
        let (_dir, farm) = temp_farm(&[]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "c"); // open the create form (title row)
        press(&mut app, "foo"); // type the title
        tab(&mut app); // -> type row
        arrow_right(&mut app); // task -> bug (single-select cursor)
        ctrl_s(&mut app); // create
        assert!(matches!(app.overlay, Overlay::None));
        let created = app.all.iter().find(|t| t.title == "foo").expect("created");
        assert_eq!(created.kind, "bug");
        assert_eq!(created.priority, 3); // default p3
        assert_eq!(created.status, Status::Hairy);
        // The cursor lands on the new task.
        assert_eq!(app.selected().map(|t| t.title.as_str()), Some("foo"));
    }

    #[test]
    fn create_form_sets_priority_and_labels() {
        let (_dir, farm) = temp_farm(&[]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "c"); // title row
        press(&mut app, "tuned");
        tab(&mut app); // -> type
        tab(&mut app); // -> priority (default p3 == idx 2)
        arrow_left(&mut app); // p3 -> p2
        tab(&mut app); // -> labels
        press(&mut app, "rust, tui");
        ctrl_s(&mut app); // create
        let created = app
            .all
            .iter()
            .find(|t| t.title == "tuned")
            .expect("created");
        assert_eq!(created.priority, 2);
        assert_eq!(created.labels, vec!["rust".to_string(), "tui".to_string()]);
    }

    #[test]
    fn reload_preserving_selection_picks_up_external_add() {
        let (_dir, farm) = temp_farm(&[
            task("t0", "first", Status::Hairy, 3, None),
            task("t1", "second", Status::Hairy, 3, None),
        ]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "j"); // cursor -> t1
        assert_eq!(app.selected_id().as_deref(), Some("t1"));
        // Simulate an external write to the farm (as the file watcher would
        // observe), then refresh from disk.
        app.farm
            .as_ref()
            .unwrap()
            .create(NewTask {
                title: "third".into(),
                prefix: None,
                kind: Some("task".into()),
                priority: Some(3),
                parent: None,
                labels: vec![],
                depends_on: vec![],
                source: None,
                description: None,
                verify: None,
            })
            .unwrap();
        app.reload_preserving_selection();
        assert!(app.all.iter().any(|t| t.title == "third"), "picked up add");
        assert_eq!(app.selected_id().as_deref(), Some("t1"), "cursor kept");
    }

    #[test]
    fn priority_change_follows_selection_to_new_slot() {
        // yaks-f207: bumping the open yak's priority resorts the list; the
        // index-based cursor must follow it to its new slot rather than
        // point at whatever yak now occupies the old one.
        let (_dir, farm) = temp_farm(&[
            task("t0", "first", Status::Hairy, 1, None),
            task("t1", "second", Status::Hairy, 2, None),
            task("t2", "third", Status::Hairy, 3, None),
        ]);
        let mut app = App::with_farm(farm).unwrap();
        // Roots sort by priority ascending: t0, t1, t2. Cursor is on t0.
        assert_eq!(app.selected_id().as_deref(), Some("t0"));
        press(&mut app, "l"); // open t0 in the detail pane
        assert!(matches!(app.focus, Focus::Detail));
        press(&mut app, "P5"); // t0 -> p5, resorting it last: t1, t2, t0
        assert_eq!(app.task("t0").unwrap().priority, 5);
        // The cursor followed t0 to its new (last) slot.
        assert_eq!(
            app.selected_id().as_deref(),
            Some("t0"),
            "cursor follows the edited yak"
        );
        assert_eq!(app.cursor, 2, "t0 moved to the last slot");
        // Still in the list, so the detail pane stays open.
        assert!(matches!(app.focus, Focus::Detail));
    }

    #[test]
    fn priority_change_out_of_view_closes_detail() {
        // yaks-f207: when a priority change filters the open yak out of the
        // current view, the detail pane -- which tracks a list slot -- must
        // close (human-confirmed drop-out behavior (a): close, not keep).
        let (_dir, farm) = temp_farm(&[
            task("t0", "leaving", Status::Hairy, 1, None),
            task("t1", "survivor", Status::Hairy, 1, None),
        ]);
        let mut app = App::with_farm(farm).unwrap();
        // Constrain the live view to p1; both tasks match, cursor on t0.
        app.filter.priorities = vec![1];
        assert_eq!(app.selected_id().as_deref(), Some("t0"));
        press(&mut app, "l"); // open t0 in the detail pane
        assert!(matches!(app.focus, Focus::Detail));
        press(&mut app, "P5"); // t0 -> p5, dropping it out of the p1 view
        assert_eq!(app.task("t0").unwrap().priority, 5);
        assert!(
            app.rows().iter().all(|r| r.task.id != "t0"),
            "t0 filtered out of the view"
        );
        // Detail closed: focus is back on the list.
        assert!(
            matches!(app.focus, Focus::List),
            "detail closes when the yak drops out"
        );
    }

    #[test]
    fn create_child_sets_parent() {
        let (_dir, farm) = temp_farm(&[task("p0", "parent", Status::Hairy, 3, None)]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "C"); // create child of the selected task
        press(&mut app, "kid");
        ctrl_s(&mut app); // create (defaults: task, p3)
        let created = app.all.iter().find(|t| t.title == "kid").expect("child");
        assert_eq!(created.parent.as_deref(), Some("p0"));
        assert_eq!(created.kind, "task");
    }

    #[test]
    fn create_form_backtab_from_description_focuses_previous_field() {
        // Regression (yaks-f433): shift-tab (BackTab) while editing the
        // description content block used to forward the key straight to
        // edtui, whose `KeyCode::from` is `unimplemented!` for BackTab and
        // panicked the whole TUI. It must instead focus the previous field.
        let (_dir, farm) = temp_farm(&[]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "c"); // open the create form (title row)
        tab(&mut app); // -> type
        tab(&mut app); // -> priority
        tab(&mut app); // -> labels
        tab(&mut app); // -> source
        tab(&mut app); // -> description (content block)
        match &app.overlay {
            Overlay::Create(f) => assert!(f.is_content_row(), "on the description block"),
            _ => panic!("create form open"),
        }
        // Must not panic, and focus moves back to the labels row (3).
        back_tab(&mut app);
        match &app.overlay {
            Overlay::Create(f) => {
                assert_eq!(f.row, 4, "focus moved back a field");
                assert!(f.is_line_text_row(), "back on the source row");
            }
            _ => panic!("create form still open"),
        }
    }

    #[test]
    fn create_form_drops_unconvertible_editor_key_without_panic() {
        // Belt-and-suspenders: any key edtui can't convert (here Insert)
        // must be dropped rather than forwarded, on both the content and
        // single-line rows. Reaching edtui with it would panic.
        let (_dir, farm) = temp_farm(&[]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "c"); // title row (single-line text)
        handle_key(&mut app, KeyEvent::new(KeyCode::Insert, KeyModifiers::NONE));
        tab(&mut app); // -> type
        tab(&mut app); // -> priority
        tab(&mut app); // -> labels
        tab(&mut app); // -> source
        tab(&mut app); // -> description (content block)
        handle_key(&mut app, KeyEvent::new(KeyCode::Insert, KeyModifiers::NONE));
        match &app.overlay {
            Overlay::Create(f) => assert!(f.is_content_row(), "still on the description"),
            _ => panic!("create form still open"),
        }
    }

    #[test]
    fn create_empty_title_ctrl_s_is_noop() {
        // Ctrl-S with an empty title keeps the form open (`(need title)`).
        let (_dir, farm) = temp_farm(&[]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "c");
        ctrl_s(&mut app);
        assert!(matches!(app.overlay, Overlay::Create(_)));
        assert!(app.all.is_empty());
    }

    #[test]
    fn create_cancelled_with_esc() {
        let (_dir, farm) = temp_farm(&[]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "c");
        press(&mut app, "foo");
        esc(&mut app); // vim: a lone Esc drops the title field to Normal
        assert!(
            matches!(app.overlay, Overlay::Create(_)),
            "lone Esc is modal"
        );
        esc(&mut app); // rapid double-Esc requests cancel
        // Dirty (typed "foo"), so a discard confirmation appears first.
        assert!(matches!(app.overlay, Overlay::Confirm { .. }));
        press(&mut app, "y"); // confirm discard
        assert!(matches!(app.overlay, Overlay::None));
        assert!(app.all.is_empty());
        assert_eq!(app.notification.as_deref(), Some("changes discarded"));
    }

    #[test]
    fn labels_edit_commits() {
        let (_dir, farm) = temp_farm(&[task("t0", "solo", Status::Hairy, 3, None)]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "L"); // empty labels field
        press(&mut app, "x, y");
        enter(&mut app);
        assert_eq!(
            app.task("t0").unwrap().labels,
            vec!["x".to_string(), "y".to_string()]
        );
    }

    /// Label input is normalized (yaks-7cb3): commas and/or spaces separate
    /// labels, duplicates collapse, and a legacy `ui,docs` label is re-split.
    #[test]
    fn labels_edit_normalizes_commas_and_spaces() {
        let mut legacy = task("t0", "legacy", Status::Hairy, 3, None);
        legacy.labels = vec!["ui,docs".into()];
        let (_dir, farm) = temp_farm(&[legacy, task("t1", "plain", Status::Hairy, 3, None)]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "L"); // seeded "ui,docs"; accepting it re-splits
        enter(&mut app);
        assert_eq!(app.task("t0").unwrap().labels, ["ui", "docs"]);
        press(&mut app, "j"); // -> t1 (no labels)
        press(&mut app, "L");
        press(&mut app, "rust tui, rust,,cli");
        enter(&mut app);
        assert_eq!(app.task("t1").unwrap().labels, ["rust", "tui", "cli"]);
        // The create form's labels field splits on spaces too.
        press(&mut app, "c");
        press(&mut app, "fresh");
        tab(&mut app); // -> type
        tab(&mut app); // -> priority
        tab(&mut app); // -> labels
        press(&mut app, "a b, c");
        ctrl_s(&mut app);
        let created = app.all.iter().find(|t| t.title == "fresh").unwrap();
        assert_eq!(created.labels, ["a", "b", "c"]);
    }

    #[test]
    fn create_form_saves_its_description() {
        let (_dir, farm) = temp_farm(&[task("t0", "solo", Status::Hairy, 3, None)]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "c"); // create form, focus on title
        press(&mut app, "new yak");
        for _ in 0..HEADER_ROWS {
            tab(&mut app); // -> description block
        }
        press(&mut app, "hello");
        ctrl_s(&mut app);
        assert!(matches!(app.overlay, Overlay::None));
        let id = app.selected_id().unwrap();
        assert_eq!(app.task(&id).unwrap().title, "new yak");
        assert_eq!(app.task(&id).unwrap().body, "hello");
    }

    #[test]
    fn comment_appends_timestamped_note() {
        let (_dir, farm) = temp_farm(&[task("t0", "solo", Status::Hairy, 3, None)]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "M"); // open the multi-line comment editor
        press(&mut app, "a helpful note");
        ctrl_s(&mut app);
        assert!(matches!(app.overlay, Overlay::None));
        let body = &app.task("t0").unwrap().body;
        assert!(body.contains("a helpful note"), "note text present");
        assert!(body.contains('\u{25b8}'), "timestamp sigil present");
    }

    #[test]
    fn ask_raises_needs_and_records_the_question() {
        let (_dir, farm) = temp_farm(&[task("t0", "solo", Status::Hairy, 3, None)]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "a"); // no needs yet -> ask prompt
        press(&mut app, "why is this blocked");
        enter(&mut app); // single-line commit
        assert!(matches!(app.overlay, Overlay::None));
        let t = app.task("t0").unwrap();
        assert_eq!(t.needs.as_deref(), Some("human"));
        assert!(
            t.body.contains("why is this blocked"),
            "question recorded as a note: {}",
            t.body
        );
    }

    #[test]
    fn answer_hands_the_yak_to_an_agent_and_records_the_reply() {
        let mut seed = task("t0", "solo", Status::Hairy, 3, None);
        seed.needs = Some("human".into());
        let (_dir, farm) = temp_farm(&[seed]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "a"); // needs is set -> answer prompt
        press(&mut app, "resolved offline");
        enter(&mut app);
        let t = app.task("t0").unwrap();
        assert_eq!(t.needs.as_deref(), Some("agent"), "handed to an agent");
        assert!(
            t.body.contains("resolved offline"),
            "reply recorded as a note: {}",
            t.body
        );
    }

    #[test]
    fn attach_file_writes_artifact_and_links_body() {
        let (proj, farm) = temp_farm(&[task("t0", "solo", Status::Hairy, 3, None)]);
        let src = proj.join("shot.png");
        fs::write(&src, b"not-really-a-png").unwrap();
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "A"); // attach prompt
        press(&mut app, src.to_str().unwrap());
        enter(&mut app); // single-line commit -> attach
        let body = &app.task("t0").unwrap().body;
        assert!(
            body.contains("![shot](t0/shot.png)"),
            "body links artifact: {body}"
        );
        assert!(
            proj.join(".yaks/t0/yaks/t0/shot.png").is_file(),
            "artifact copied into the farm"
        );
    }

    /// r on an attachment's image link (detail pane) renames the file and
    /// rewrites the link; the extension is kept when the new name omits it.
    #[test]
    fn rename_attachment_from_detail_link() {
        let (proj, farm) = temp_farm(&[task("t0", "solo", Status::Hairy, 3, None)]);
        let _ = farm
            .attach("t0", "paste-20260921-234927.png", b"png")
            .unwrap();
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "l"); // detail pane
        press(&mut app, "r"); // cursor not on the link yet
        assert!(
            matches!(app.overlay, Overlay::None),
            "no link under cursor -> no prompt"
        );
        tab(&mut app); // cursor -> the image link
        press(&mut app, "r");
        assert!(draw(&app, 100, 20).contains("Rename paste-20260921-234927.png to"));
        press(&mut app, "login page");
        enter(&mut app);
        let body = &app.task("t0").unwrap().body;
        assert!(
            body.contains("![login-page](t0/login-page.png)"),
            "link rewritten: {body}"
        );
        let dir = proj.join(".yaks/t0/yaks/t0");
        assert!(dir.join("login-page.png").is_file());
        assert!(!dir.join("paste-20260921-234927.png").exists());
        assert_eq!(
            app.notification.as_deref(),
            Some("renamed paste-20260921-234927.png → login-page.png")
        );
    }

    /// The clipboard-PNG attach asks for a name (empty keeps paste-<ts>.png).
    #[test]
    fn paste_attach_prompts_for_a_name() {
        let (proj, farm) = temp_farm(&[task("t0", "solo", Status::Hairy, 3, None)]);
        let mut app = App::with_farm(farm).unwrap();
        app.open_paste_name("t0".into(), b"png".to_vec());
        assert!(draw(&app, 100, 20).contains("Name pasted PNG (empty = paste-"));
        press(&mut app, "dashboard");
        enter(&mut app);
        let body = &app.task("t0").unwrap().body;
        assert!(body.contains("![dashboard](t0/dashboard.png)"), "{body}");
        assert!(proj.join(".yaks/t0/yaks/t0/dashboard.png").is_file());
    }

    #[test]
    fn edit_form_changes_type_and_priority() {
        let (_dir, farm) = temp_farm(&[task("t0", "solo", Status::Hairy, 3, None)]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "E");
        tab(&mut app); // -> type
        arrow_right(&mut app); // task -> bug
        tab(&mut app); // -> priority (p3 == idx 2)
        arrow_left(&mut app); // p3 -> p2
        arrow_left(&mut app); // p2 -> p1
        ctrl_s(&mut app);
        let t = app.task("t0").unwrap();
        assert_eq!(t.kind, "bug");
        assert_eq!(t.priority, 1);
    }

    #[test]
    fn edit_form_shows_herd_row_seeded_to_current_herd() {
        // The edit affordance carries a herd chip row on a multi-herd farm,
        // seeded to the yak's current herd (yaks-71d1).
        let (_dir, farm) = temp_farm(&[
            task("web-0001", "web one", Status::Hairy, 3, None),
            task("api-0002", "api two", Status::Hairy, 3, None),
        ]);
        let mut app = App::with_farm(farm).unwrap();
        app.select_id("web-0001");
        press(&mut app, "E");
        match &app.overlay {
            Overlay::Create(f) => {
                assert!(f.is_editing());
                assert!(f.has_herd_row(), "edit form shows the herd row");
                assert_eq!(f.target_herd().as_deref(), Some("web"));
            }
            _ => panic!("expected edit form"),
        }
    }

    #[test]
    fn edit_form_herd_change_renames_on_commit() {
        // Changing the herd chip and saving renames the id prefix (tail kept),
        // the form's fold-in of the H move (yaks-71d1).
        let (_dir, farm) = temp_farm(&[
            task("web-0001", "web one", Status::Hairy, 3, None),
            task("api-0002", "api two", Status::Hairy, 3, None),
        ]);
        let mut app = App::with_farm(farm).unwrap();
        app.select_id("web-0001");
        press(&mut app, "E");
        // title -> type -> priority -> labels -> herd (HEADER_ROWS tabs).
        for _ in 0..HEADER_ROWS {
            tab(&mut app);
        }
        // Herds are sorted ["api", "web"]; current is web (idx 1) -> left = api.
        arrow_left(&mut app);
        ctrl_s(&mut app);
        assert!(app.task("web-0001").is_none(), "old id is gone");
        assert!(app.task("api-0001").is_some(), "renamed into the api herd");
    }

    #[test]
    fn edit_form_cancel_leaves_task_unchanged() {
        let (_dir, farm) = temp_farm(&[task("t0", "solo", Status::Hairy, 3, None)]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "E");
        press(&mut app, "zzz"); // edits the title field in place
        esc(&mut app); // vim: drops the title field to Normal (no cancel)
        esc(&mut app); // rapid double-Esc requests cancel
        assert!(
            matches!(app.overlay, Overlay::Confirm { .. }),
            "dirty -> confirm"
        );
        press(&mut app, "y"); // confirm discard
        assert!(matches!(app.overlay, Overlay::None));
        assert_eq!(app.task("t0").unwrap().title, "solo");
        assert_eq!(app.notification.as_deref(), Some("changes discarded"));
    }

    /// A yak whose body `content::assemble` would normalize (stray blank
    /// lines, trailing spaces): any rebuild of the body shows up as a diff.
    fn untidy_body_app() -> (std::path::PathBuf, App, String) {
        let mut t = task("t0", "solo", Status::Hairy, 3, None);
        t.body = "Desc  \n\n\n---\n\u{25b8} 2026-01-01T00:00:00Z [joel]\n\nnote   \n\n\n".into();
        let (dir, farm) = temp_farm(&[t]);
        let app = App::with_farm(farm).unwrap();
        let before = app.task("t0").unwrap().body.clone();
        (dir, app, before)
    }

    #[test]
    fn edit_form_has_header_rows_only() {
        let (_d, mut app, _) = untidy_body_app();
        app.focus = Focus::List;
        press(&mut app, "E");
        let Overlay::Create(f) = &app.overlay else {
            panic!("expected the edit form")
        };
        assert!(f.description.is_none());
        assert_eq!(f.row_count(), f.header_rows());
        assert_eq!(f.row, 0, "focus starts at the title");
        let out = draw(&app, 90, 24);
        assert!(out.contains("Edit t0") && out.contains("title"), "{out}");
        assert!(!out.contains("description"), "no content rows: {out}");
        assert!(!out.contains("───"), "no content separator: {out}");
        // Ctrl-N walks only the header rows and wraps back to the title.
        let n = f.row_count();
        for _ in 0..n {
            handle_key(
                &mut app,
                KeyEvent::new(KeyCode::Char('n'), KeyModifiers::CONTROL),
            );
        }
        assert!(matches!(&app.overlay, Overlay::Create(f) if f.row == 0));
    }

    #[test]
    fn edit_form_title_edit_leaves_the_body_byte_identical() {
        let (_d, mut app, before) = untidy_body_app();
        app.focus = Focus::List;
        press(&mut app, "E");
        press(&mut app, "Z"); // title field: insert at the cursor
        ctrl_s(&mut app);
        let t = app.task("t0").unwrap();
        assert!(t.title.contains('Z'), "title edited: {:?}", t.title);
        assert_eq!(t.body, before);
    }

    #[test]
    fn edit_form_without_edits_is_not_dirty_and_keeps_the_body() {
        let (_d, mut app, before) = untidy_body_app();
        app.focus = Focus::List;
        press(&mut app, "E");
        ctrl_s(&mut app);
        assert_eq!(app.task("t0").unwrap().body, before);
    }

    #[test]
    fn create_form_still_has_its_description_block() {
        let mut app = App::new(vec![task("t0", "t", Status::Hairy, 3, None)]);
        app.open_create(false);
        let Overlay::Create(f) = &app.overlay else {
            panic!("expected the create form")
        };
        assert!(f.description.is_some());
        assert_eq!(f.row_count(), f.header_rows() + 1);
        let out = draw(&app, 90, 24);
        assert!(out.contains("description"), "{out}");
    }

    #[test]
    fn block_edit_deletes_an_emptied_comment() {
        let (_dir, farm) = temp_farm(&[task("t0", "solo", Status::Hairy, 3, None)]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "M");
        press(&mut app, "note to remove");
        ctrl_s(&mut app);
        assert!(app.task("t0").unwrap().body.contains('\u{25b8}'));
        app.open_block_edit(Some(1)); // the comment, in place
        press(&mut app, "dd"); // Normal mode: delete its only line
        ctrl_s(&mut app);
        let body = &app.task("t0").unwrap().body;
        assert!(!body.contains('\u{25b8}'), "comment deleted: {body:?}");
        assert!(
            body.trim().is_empty(),
            "body empty after deletion: {body:?}"
        );
    }

    #[test]
    fn lone_esc_stays_in_the_comment_editor() {
        let (_dir, farm) = temp_farm(&[task("t0", "solo", Status::Hairy, 3, None)]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "M");
        press(&mut app, "draft");
        esc(&mut app); // multiline: Esc drops to Normal, does NOT cancel
        assert!(
            matches!(app.overlay, Overlay::BlockEdit(_)),
            "editor stays open"
        );
    }

    #[test]
    fn double_esc_cancels_the_comment_editor() {
        let (_dir, farm) = temp_farm(&[task("t0", "solo", Status::Hairy, 3, None)]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "M");
        press(&mut app, "draft");
        esc(&mut app); // -> Normal
        esc(&mut app); // rapid second Esc == Ctrl-C -> request cancel
        // The draft is dirty, so a discard confirmation appears first.
        assert!(matches!(app.overlay, Overlay::Confirm { .. }));
        press(&mut app, "y"); // confirm discard
        assert!(matches!(app.overlay, Overlay::None));
        assert_eq!(app.notification.as_deref(), Some("changes discarded"));
        assert!(app.task("t0").unwrap().body.is_empty(), "comment not saved");
    }

    #[test]
    fn double_esc_cancels_the_edit_form_from_a_content_row() {
        let (_dir, farm) = temp_farm(&[task("t0", "solo", Status::Hairy, 3, None)]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "E");
        for _ in 0..4 {
            tab(&mut app); // title -> ... -> description (a content row)
        }
        esc(&mut app); // content row: Esc drops to Normal, form stays
        assert!(matches!(app.overlay, Overlay::Create(_)), "form stays open");
        esc(&mut app); // rapid second Esc cancels the whole edit
        assert!(matches!(app.overlay, Overlay::None));
        assert_eq!(app.notification.as_deref(), Some("edit cancelled"));
    }

    #[test]
    fn declining_the_discard_confirm_returns_to_the_editor() {
        let (_dir, farm) = temp_farm(&[task("t0", "solo", Status::Hairy, 3, None)]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "c");
        press(&mut app, "foo");
        esc(&mut app);
        esc(&mut app); // dirty -> discard confirm
        assert!(matches!(app.overlay, Overlay::Confirm { .. }));
        press(&mut app, "n"); // decline -> back to the form, work intact
        match &app.overlay {
            Overlay::Create(f) => assert_eq!(f.title_text(), "foo", "draft preserved"),
            _ => panic!("expected the form to be restored"),
        }
    }

    #[test]
    fn clean_cancel_skips_the_confirm() {
        let (_dir, farm) = temp_farm(&[task("t0", "solo", Status::Hairy, 3, None)]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "E"); // open edit, make no changes
        handle_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
        );
        assert!(
            matches!(app.overlay, Overlay::None),
            "clean cancel is immediate"
        );
        assert_eq!(app.notification.as_deref(), Some("edit cancelled"));
    }

    #[test]
    fn colon_wq_commits_the_comment() {
        let (_dir, farm) = temp_farm(&[task("t0", "solo", Status::Hairy, 3, None)]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "M");
        press(&mut app, "a note");
        esc(&mut app); // Normal
        press(&mut app, ":wq"); // open cmdline + type
        enter(&mut app); // run
        assert!(matches!(app.overlay, Overlay::None));
        assert!(app.task("t0").unwrap().body.contains("a note"));
    }

    #[test]
    fn colon_q_bang_discards_the_comment() {
        let (_dir, farm) = temp_farm(&[task("t0", "solo", Status::Hairy, 3, None)]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "M");
        press(&mut app, "draft");
        esc(&mut app); // Normal
        press(&mut app, ":q!"); // force-quit, no discard confirm
        enter(&mut app);
        assert!(matches!(app.overlay, Overlay::None));
        assert!(app.task("t0").unwrap().body.is_empty());
    }

    #[test]
    fn colon_w_saves_the_create_form() {
        let (_dir, farm) = temp_farm(&[task("t0", "solo", Status::Hairy, 3, None)]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "c");
        press(&mut app, "new yak");
        for _ in 0..HEADER_ROWS {
            tab(&mut app); // -> description block (empty -> Insert)
        }
        press(&mut app, "hello");
        esc(&mut app); // Normal
        press(&mut app, ":w");
        enter(&mut app);
        assert!(matches!(app.overlay, Overlay::None));
        let id = app.selected_id().unwrap();
        assert_eq!(app.task(&id).unwrap().body, "hello");
    }

    #[test]
    fn colon_unknown_command_notifies_and_stays_open() {
        let (_dir, farm) = temp_farm(&[task("t0", "solo", Status::Hairy, 3, None)]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "M");
        esc(&mut app); // empty comment -> Normal
        press(&mut app, ":nope");
        enter(&mut app);
        assert!(
            matches!(app.overlay, Overlay::BlockEdit(_)),
            "editor stays open"
        );
        assert_eq!(app.notification.as_deref(), Some("unknown command: :nope"));
    }

    fn with_deps(id: &str, status: Status, deps: &[&str]) -> Task {
        let mut t = task(id, id, status, 3, None);
        t.depends_on = deps.iter().map(|s| s.to_string()).collect();
        t
    }

    #[test]
    fn dep_add_via_picker() {
        let (_dir, farm) = temp_farm(&[
            task("t0", "t0", Status::Hairy, 3, None),
            task("t1", "t1", Status::Hairy, 3, None),
        ]);
        let mut app = App::with_farm(farm).unwrap();
        assert_eq!(app.selected_id().as_deref(), Some("t0"));
        press(&mut app, "D"); // open picker (t0 excluded)
        enter(&mut app); // pick first candidate (t1)
        assert!(matches!(app.overlay, Overlay::None));
        assert_eq!(app.task("t0").unwrap().depends_on, vec!["t1".to_string()]);
        assert_eq!(app.notification.as_deref(), Some("t0 depends on t1"));
    }

    #[test]
    fn dep_cycle_target_is_excluded() {
        // t1 already depends on t0, so t0 -> t1 would cycle: t1 is not offered.
        let (_dir, farm) = temp_farm(&[
            task("t0", "t0", Status::Hairy, 3, None),
            with_deps("t1", Status::Hairy, &["t0"]),
        ]);
        let mut app = App::with_farm(farm).unwrap();
        assert_eq!(app.selected_id().as_deref(), Some("t0"));
        press(&mut app, "D");
        enter(&mut app); // no candidates -> nothing selected
        assert!(app.task("t0").unwrap().depends_on.is_empty());
        assert_eq!(app.notification.as_deref(), Some("nothing selected"));
    }

    #[test]
    fn reparent_via_picker() {
        let (_dir, farm) = temp_farm(&[
            task("p0", "p0", Status::Hairy, 3, None),
            task("t0", "t0", Status::Hairy, 3, None),
        ]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "j"); // move to t0
        assert_eq!(app.selected_id().as_deref(), Some("t0"));
        press(&mut app, "R");
        enter(&mut app); // pick first candidate (p0)
        assert_eq!(app.task("t0").unwrap().parent.as_deref(), Some("p0"));
        assert_eq!(app.notification.as_deref(), Some("t0 reparented under p0"));
    }

    #[test]
    fn reparent_clear_to_root() {
        let (_dir, farm) = temp_farm(&[
            task("p0", "p0", Status::Hairy, 3, None),
            task("c0", "c0", Status::Hairy, 3, Some("p0")),
        ]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "j"); // move to c0 (child of p0)
        assert_eq!(app.selected_id().as_deref(), Some("c0"));
        press(&mut app, "R"); // p0 excluded (current parent); only clear-parent row
        enter(&mut app);
        assert!(app.task("c0").unwrap().parent.is_none());
        assert_eq!(app.notification.as_deref(), Some("c0 moved to top level"));
    }

    #[test]
    fn create_form_source_is_saved() {
        // yaks-b0b4: the Source row lands in the new yak's `source:`.
        let (_dir, farm) = temp_farm(&[task("t0", "solo", Status::Hairy, 3, None)]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "c");
        press(&mut app, "linked");
        for _ in 0..4 {
            tab(&mut app);
        }
        press(&mut app, "https://example.com/X-1");
        ctrl_s(&mut app);
        let id = app.selected_id().unwrap();
        let t = app.task(&id).unwrap();
        assert_eq!(t.title, "linked");
        assert_eq!(t.source.as_deref(), Some("https://example.com/X-1"));
    }

    #[test]
    fn edit_form_source_set_and_clear() {
        // yaks-b0b4: E seeds the Source row from `source:`; editing replaces it
        // and emptying it clears the field.
        let mut t0 = task("t0", "solo", Status::Hairy, 3, None);
        t0.source = Some("https://old.example/1".into());
        let (_dir, farm) = temp_farm(&[t0]);
        let mut app = App::with_farm(farm).unwrap();
        press(&mut app, "E");
        for _ in 0..4 {
            tab(&mut app);
        }
        match &app.overlay {
            Overlay::Create(f) => assert_eq!(f.source_text(), "https://old.example/1"),
            _ => panic!("expected edit form"),
        }
        handle_key(&mut app, KeyEvent::new(KeyCode::End, KeyModifiers::NONE));
        for _ in 0.."https://old.example/1".len() {
            handle_key(
                &mut app,
                KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
            );
        }
        press(&mut app, "https://new.example/2");
        ctrl_s(&mut app);
        assert_eq!(
            app.task("t0").unwrap().source.as_deref(),
            Some("https://new.example/2")
        );
        // Clear it.
        press(&mut app, "E");
        for _ in 0..4 {
            tab(&mut app);
        }
        handle_key(&mut app, KeyEvent::new(KeyCode::End, KeyModifiers::NONE));
        for _ in 0.."https://new.example/2".len() {
            handle_key(
                &mut app,
                KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
            );
        }
        ctrl_s(&mut app);
        assert!(app.task("t0").unwrap().source.is_none());
    }
}

// -- ephemeral status line placement ----------------------------------

#[test]
fn status_notification_drops_below_tabs_when_it_would_overlap() {
    // The ephemeral yellow status line lives right-aligned on the tab row,
    // but must not scribble over the tabs. At a narrow width it can't fit
    // beside them, so it drops to the blank gap row directly below the tabs;
    // at a wide width it stays inline on the tab row. This is a pure
    // render-time decision, so the same App re-lays-out on width alone.
    const NOTE: &str = "quarterly-planning";
    let mut app = sample();
    app.notification = Some("saved view: quarterly-planning".into());

    // Narrow: no room beside the tabs -> dropped to the gap row (row 1).
    let narrow = draw(&app, 90, 8);
    let nlines: Vec<&str> = narrow.lines().collect();
    assert!(
        !nlines[0].contains(NOTE),
        "narrow: the tab row must not carry the notification\n{narrow}"
    );
    assert!(
        nlines[1].contains(NOTE),
        "narrow: the notification drops to the blank row below the tabs\n{narrow}"
    );
    insta::assert_snapshot!("status_notification_below_tabs_narrow", narrow);

    // Wide: fits beside the tabs -> stays inline on the tab row (row 0),
    // leaving the gap row blank.
    let wide = draw(&app, 160, 8);
    let wlines: Vec<&str> = wide.lines().collect();
    assert!(
        wlines[0].contains(NOTE),
        "wide: the notification stays inline on the tab row\n{wide}"
    );
    assert!(
        !wlines[1].contains(NOTE),
        "wide: the gap row below the tabs stays blank\n{wide}"
    );
    insta::assert_snapshot!("status_notification_inline_wide", wide);
}

#[test]
fn list_scroll_up_moves_viewport_minimally() {
    // yaks-9009: scrolling up from the bottom must not re-pin the cursor to
    // the bottom edge each frame; the viewport stays put until the cursor
    // reaches its top edge, then scrolls one row at a time.
    let tasks: Vec<Task> = (0..30)
        .map(|i| {
            task(
                &format!("t{i:02}"),
                &format!("task {i:02}"),
                Status::Hairy,
                3,
                None,
            )
        })
        .collect();
    let mut app = App::new(tasks);
    let (w, h) = (60, 16);
    // Like the live loop, render after every key so the offset carries over.
    handle_key(&mut app, key('G'));
    let bottom = draw(&app, w, h);
    assert!(bottom.contains("task 29"));
    let top_of_viewport = app.list_offset.get();
    handle_key(&mut app, key('k'));
    draw(&app, w, h);
    handle_key(&mut app, key('k'));
    let after = draw(&app, w, h);
    // Moving up within the viewport doesn't scroll it at all.
    assert_eq!(app.list_offset.get(), top_of_viewport);
    assert!(
        after.contains("task 29"),
        "bottom row still visible:\n{after}"
    );
    insta::assert_snapshot!(after);
    // Walk up past the viewport's top edge: it scrolls by exactly one row per
    // step, keeping the cursor on the top row (not the bottom).
    for _ in 0..(app.cursor - top_of_viewport + 1) {
        handle_key(&mut app, key('k'));
        draw(&app, w, h);
    }
    assert_eq!(app.list_offset.get(), top_of_viewport - 1);
    assert_eq!(app.cursor, app.list_offset.get());
}

#[test]
fn hash_opens_goto_picker_and_enter_jumps_to_detail() {
    // `#` opens the fuzzy picker over every yak; typing an id and Enter jumps
    // to its detail — across views — and records o/i history (yaks-63f3).
    let mut app = sample(); // Hairy view, cursor on a0; b0 is Shaving
    handle_key(&mut app, key('#'));
    assert!(matches!(&app.overlay, Overlay::Fuzzy(fp) if fp.label == "Go to yak"));
    press(&mut app, "b0");
    insta::assert_snapshot!("goto_picker_with_id_typed", draw(&app, 72, 14));
    enter_key(&mut app);
    assert!(matches!(app.overlay, Overlay::None));
    assert_eq!(app.focus, Focus::Detail);
    assert_eq!(app.selected_id().as_deref(), Some("b0"));
    assert_eq!(app.notification.as_deref(), Some("→ b0"));
    insta::assert_snapshot!("goto_jumped_to_detail", draw(&app, 72, 14));
    // `o` comes back to where we jumped from.
    handle_key(&mut app, key('o'));
    assert_eq!(app.selected_id().as_deref(), Some("a0"));
}

#[test]
fn hash_from_detail_pane_matches_title_too() {
    let mut app = sample();
    enter_key(&mut app); // detail on a0
    handle_key(&mut app, key('#'));
    press(&mut app, "child a1");
    enter_key(&mut app);
    assert_eq!(app.focus, Focus::Detail);
    assert_eq!(app.selected_id().as_deref(), Some("a1"));
}

#[test]
fn b_opens_references_picker_over_incoming_yaks_and_enter_goes() {
    // `B` lists only the yaks that point at the selected one, each tagged with
    // how: a0's children a1 and a2 (a2 also depends on it); a1 is mentioned in
    // a0's body and a note of b0's.
    let mut a0 = task("a0", "Root A", Status::Hairy, 2, None);
    a0.body = "follow a1 then a2".into();
    let mut a2 = task("a2", "Child A2", Status::Hairy, 3, Some("a0"));
    a2.depends_on = vec!["a0".into()];
    let mut b0 = task("b0", "Unrelated B", Status::Hairy, 3, None);
    b0.body = "desc\n\n---\n▸ 2026-01-02T00:00:00Z [ann]\n\nsee a1, not a10".into();
    let mut app = App::new(vec![
        a0,
        task("a1", "Child A1", Status::Hairy, 3, Some("a0")),
        a2,
        b0,
    ]);
    handle_key(&mut app, key('B')); // cursor on a0
    let Overlay::Fuzzy(fp) = &app.overlay else {
        panic!("B must open the fuzzy picker")
    };
    assert_eq!(fp.label, "References to a0");
    let rows: Vec<(String, String)> = fuzzy_candidates(&app.all, fp)
        .iter()
        .map(|t| (t.id.clone(), fp.only.as_ref().unwrap()[&t.id].clone()))
        .collect();
    let s = |a: &str, b: &str| (a.to_string(), b.to_string());
    assert_eq!(rows, vec![s("a1", "child"), s("a2", "child, depends")]);
    esc_key(&mut app); // query to Normal
    esc_key(&mut app); // close
    assert!(matches!(app.overlay, Overlay::None));

    // From a1: mentioned by a0's description and by b0's note.
    app.goto_task("a1");
    handle_key(&mut app, key('B'));
    insta::assert_snapshot!("backrefs_picker", draw(&app, 72, 14));
    let Overlay::Fuzzy(fp) = &app.overlay else {
        panic!("B must open the fuzzy picker from detail")
    };
    let ids: Vec<&str> = fuzzy_candidates(&app.all, fp)
        .iter()
        .map(|t| t.id.as_str())
        .collect();
    assert_eq!(ids, vec!["a0", "b0"]);
    press(&mut app, "b0");
    enter_key(&mut app);
    assert_eq!(app.focus, Focus::Detail);
    assert_eq!(app.selected_id().as_deref(), Some("b0"));
}

#[test]
fn b_with_no_incoming_references_says_so() {
    let mut app = sample();
    handle_key(&mut app, key('j')); // a1: nothing points at it
    handle_key(&mut app, key('B'));
    assert!(matches!(app.overlay, Overlay::None));
    assert_eq!(app.notification.as_deref(), Some("no yak refers to a1"));
}

#[test]
fn goto_lands_even_when_an_ad_hoc_search_hides_the_target() {
    // The jump switches to the target's status view (resetting the live
    // filter to that view's spec), so a search that hid it doesn't block it.
    let mut app = sample();
    app.filter.search = Some("no such yak".into());
    handle_key(&mut app, key('#'));
    press(&mut app, "a1");
    enter_key(&mut app);
    assert_eq!(app.focus, Focus::Detail);
    assert_eq!(app.selected_id().as_deref(), Some("a1"));
}

#[test]
fn main_split_clamps_detail_width_and_goes_modal_when_narrow() {
    let at = |w: u16| {
        let (l, r) = main_split(ratatui::layout::Rect::new(0, 2, w, 10));
        assert_eq!(l.width + r.width, w, "split must cover the area at {w}");
        assert_eq!(r.x, l.width, "pane starts where the list ends at {w}");
        (l.width, r.width)
    };
    // Narrowest split: the list just reaches LIST_MIN_WIDTH beside a 66% pane.
    assert_eq!(at(140), (LIST_MIN_WIDTH, 92));
    // Clamped at the max: extra width goes to the list.
    assert_eq!(at(200), (200 - DETAIL_MAX_WIDTH, DETAIL_MAX_WIDTH));
    assert_eq!(at(220), (220 - DETAIL_MAX_WIDTH, DETAIL_MAX_WIDTH));
    // Below that the list would be starved: the pane covers everything (modal).
    assert_eq!(at(139), (0, 139));
    assert_eq!(at(80), (0, 80));
    assert_eq!(at(40), (0, 40));
}

#[test]
fn modal_detail_shows_back_affordance_only_when_covering_the_list() {
    let mut app = sample();
    handle_key(&mut app, key('l'));
    assert_eq!(app.focus, Focus::Detail);
    let wide = draw(&app, 160, 12);
    assert!(
        !wide.contains('\u{25c2}'),
        "split view has no chevron:\n{wide}"
    );
    let narrow = draw(&app, 60, 12);
    let row = narrow.lines().nth(2).unwrap();
    assert!(
        row.starts_with('\u{25c2}'),
        "modal pane tips its divider with a chevron:\n{narrow}"
    );
    // At 220 cols the pane stops at DETAIL_MAX_WIDTH; its divider sits at the
    // list/pane boundary.
    let huge = draw(&app, 220, 12);
    let row = huge.lines().nth(3).unwrap();
    let col = row.chars().position(|c| c == '\u{2502}').unwrap();
    assert_eq!(
        col as u16,
        220 - DETAIL_MAX_WIDTH,
        "divider column:\n{huge}"
    );
}

/// yaks-027a: the single-line ask/answer prompts end in a `: ` separator, so
/// the typed text doesn't butt straight up against the key hints
/// (`…Ctrl-C cancelStrong agreement…`).
#[test]
fn ask_and_answer_prompts_separate_label_from_typed_text() {
    let mut app = sample();
    app.open_ask();
    press(&mut app, "hello");
    let out = draw(&app, 120, 14);
    assert!(
        out.contains("Ask a0 — blocks on a human (Enter save · Ctrl-C cancel): hello"),
        "ask prompt should separate label from text\n{out}"
    );
    let mut app = sample();
    app.open_answer();
    press(&mut app, "hello");
    let out = draw(&app, 120, 14);
    assert!(
        out.contains("Answer a0 — hands it to an agent (Enter save · Ctrl-C cancel): hello"),
        "answer prompt should separate label from text\n{out}"
    );
}

/// yaks-2d17: a header field that soft-wraps in the detail pane must style its
/// continuation row as value text, not with the field-label colour in the
/// columns where the label sat on the first row.
#[test]
fn wrapped_detail_field_continuation_is_not_label_coloured() {
    let long = "E1: Jev-as-grouper probe per-candidate role Choice composition evaluate \
                on Burrito drops standalone Stage-A kind Gemini deferred";
    let mut app = App::new(vec![task("t0", long, Status::Hairy, 3, None)]);
    enter_key(&mut app); // focus the detail pane
    let mut term = ratatui::Terminal::new(ratatui::backend::TestBackend::new(100, 20)).unwrap();
    // Draw twice: the first frame records the pane width the wrap uses.
    term.draw(|f| render(&app, f)).unwrap();
    term.draw(|f| render(&app, f)).unwrap();
    let buf = term.backend().buffer();
    let text = buffer_to_string(buf);
    // Locate "Title:" by cell position (emoji rows make string offsets lie).
    let (title_x, title_y) = (0..buf.area.height)
        .flat_map(|y| (0..buf.area.width.saturating_sub(6)).map(move |x| (x, y)))
        .find(|&(x, y)| {
            (0..6u16)
                .map(|i| buf[(x + i, y)].symbol().to_string())
                .collect::<String>()
                == "Title:"
        })
        .unwrap_or_else(|| panic!("title row\n{text}"));
    // Sanity: the label on the first row is the dim label colour.
    assert_eq!(buf[(title_x, title_y)].fg, Color::DarkGray);
    // The continuation row: same column range, all value text (default fg).
    let cont_y = title_y + 1;
    let cont: String = (title_x..title_x + 13)
        .map(|x| buf[(x, cont_y)].symbol().to_string())
        .collect();
    assert!(
        !cont.trim().is_empty() && !cont.contains("Status:"),
        "expected the title to wrap onto the next row\n{text}"
    );
    for x in title_x..title_x + 13 {
        assert_ne!(
            buf[(x, cont_y)].fg,
            Color::DarkGray,
            "continuation cell {x} took the label colour\n{text}"
        );
    }
}

// -- paste (yaks-f2aa) ----------------------------------------------------

/// Open the create form on `app` and focus its description block.
fn focus_description(app: &mut App) {
    app.open_create(false);
    for _ in 0..5 {
        tab(app); // title -> type -> priority -> labels -> source -> description
    }
    assert!(matches!(&app.overlay, Overlay::Create(f) if f.description_editor().is_some()));
}

fn description_text(app: &App) -> String {
    match &app.overlay {
        Overlay::Create(f) => f.description_text(),
        _ => panic!("expected the create form"),
    }
}

#[test]
fn bracketed_paste_bulk_inserts_into_the_description() {
    let mut app = App::new(vec![task("t0", "t", Status::Hairy, 3, None)]);
    focus_description(&mut app);
    // CRLF normalizes; a `yaks-` + Tab doesn't pop the ref picker mid-paste.
    handle_event(
        &mut app,
        Event::Paste("line one\r\nsee yaks-\tthen **two**".into()),
    );
    assert!(
        matches!(app.overlay, Overlay::Create(_)),
        "no picker opened"
    );
    assert_eq!(description_text(&app), "line one\nsee yaks-\tthen **two**");
}

#[test]
fn vim_normal_mode_paste_inserts_literally_not_as_commands() {
    // In Normal mode pasting `dd` must insert text, not delete a line (it
    // would, replayed as keys).
    let mut app = App::new(vec![task("t0", "t", Status::Hairy, 3, None)]);
    focus_description(&mut app);
    handle_paste(&mut app, "First line.\nSecond line.");
    match &app.overlay {
        Overlay::Create(f) => {
            f.description.as_ref().unwrap().borrow_mut().mode = EditorMode::Normal
        }
        _ => panic!("expected the create form"),
    }
    handle_paste(&mut app, "dd");
    let text = description_text(&app);
    assert!(text.contains("dd"), "pasted literally: {text:?}");
    assert!(text.contains("First line.") && text.contains("Second line"));
}

#[test]
fn paste_into_a_single_line_field_flattens_newlines() {
    let mut app = App::new(vec![task("t0", "t", Status::Hairy, 3, None)]);
    app.open_edit(); // focus starts on the title row
    handle_paste(&mut app, "a\nb");
    match &app.overlay {
        Overlay::Create(f) => assert_eq!(f.title_text(), "a bt"), // at the cursor (col 0)
        _ => panic!("expected the edit form"),
    }
}

#[test]
fn paste_into_search_updates_the_live_filter() {
    let mut app = sample();
    handle_key(&mut app, key('/'));
    handle_paste(&mut app, "child\n");
    assert_eq!(app.filter.search.as_deref(), Some("child "));
}

#[test]
fn paste_outside_a_text_field_is_not_a_command_stream() {
    let mut app = sample();
    handle_paste(&mut app, "qXc");
    assert!(!app.quit);
    assert!(matches!(app.overlay, Overlay::None));
}

#[test]
fn pump_drains_a_keystroke_burst_before_the_next_redraw() {
    // A terminal without bracketed paste sends a paste as keys; the loop must
    // handle the whole queued burst in one pump (one frame), not one per key.
    let mut app = App::new(vec![task("t0", "t", Status::Hairy, 3, None)]);
    focus_description(&mut app);
    let text = "hello world ".repeat(40);
    let mut queue = text
        .chars()
        .map(|c| Event::Key(key(c)))
        .collect::<std::collections::VecDeque<_>>();
    let first = queue.pop_front().unwrap();
    let n = pump_events(&mut app, first, Duration::from_secs(5), || {
        Ok(queue.pop_front())
    })
    .unwrap();
    assert_eq!(n, text.chars().count());
    assert_eq!(description_text(&app), text);
}

#[test]
fn pump_stops_draining_once_the_app_quits() {
    let mut app = sample();
    let mut queue: std::collections::VecDeque<_> =
        [Event::Key(key('j')), Event::Key(key('j'))].into();
    let n = pump_events(
        &mut app,
        Event::Key(key('q')),
        Duration::from_secs(5),
        || Ok(queue.pop_front()),
    )
    .unwrap();
    assert_eq!(n, 1);
    assert!(app.quit);
}

/// Timing evidence for yaks-f2aa: the old loop redrew after every key, so a
/// keystroke-stream paste cost one full frame per char; now a burst (or a
/// bracketed paste) costs one frame. `cargo test -p yaks --release paste_bench
/// -- --ignored --nocapture`.
#[test]
#[ignore]
fn paste_bench() {
    use std::time::Instant;
    let tasks: Vec<_> = (0..300)
        .map(|i| {
            task(
                &format!("t{i}"),
                &format!("task number {i}"),
                Status::Hairy,
                3,
                None,
            )
        })
        .collect();
    let text = "The quick brown fox jumps over the lazy dog, **bold** and `code`.\n".repeat(30);
    let n = text.chars().count();
    let keys = || {
        text.chars().map(|c| {
            Event::Key(if c == '\n' {
                KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)
            } else {
                key(c)
            })
        })
    };
    let mut term = Terminal::new(ratatui::backend::TestBackend::new(160, 50)).unwrap();
    // Before: one full redraw per key event.
    let mut app = App::new(tasks.clone());
    focus_description(&mut app);
    let t0 = Instant::now();
    for ev in keys() {
        handle_event(&mut app, ev);
        term.draw(|f| render(&app, f)).unwrap();
    }
    let before = t0.elapsed();
    // After (no bracketed paste): the burst drains in one pump, one redraw.
    let mut app = App::new(tasks.clone());
    focus_description(&mut app);
    let t0 = Instant::now();
    let mut q: std::collections::VecDeque<_> = keys().collect();
    let first = q.pop_front().unwrap();
    pump_events(&mut app, first, Duration::from_millis(50), || {
        Ok(q.pop_front())
    })
    .unwrap();
    term.draw(|f| render(&app, f)).unwrap();
    let drained = t0.elapsed();
    // After (bracketed paste): one Paste event, one redraw.
    let mut app = App::new(tasks);
    focus_description(&mut app);
    let t0 = Instant::now();
    handle_event(&mut app, Event::Paste(text.clone()));
    term.draw(|f| render(&app, f)).unwrap();
    let pasted = t0.elapsed();
    eprintln!("PASTE_BENCH {n} chars @160x50, 300 yaks");
    eprintln!(
        "PASTE_BENCH before  (redraw per key):      {before:?} ({:?}/char)",
        before / n as u32
    );
    eprintln!("PASTE_BENCH after   (drained key burst):   {drained:?}");
    eprintln!("PASTE_BENCH after   (bracketed Paste):     {pasted:?}");
    // The resulting frame, for eyeballing: a markdown paste into a fresh form.
    let mut app = App::new(vec![task("t0", "paste target", Status::Hairy, 3, None)]);
    focus_description(&mut app);
    handle_event(
        &mut app,
        Event::Paste(
            "# Pasted\r\n\r\nA **bold** line and `code`, see yaks-\tx.\r\n- one\r\n- two".into(),
        ),
    );
    eprintln!("PASTE_FRAME\n{}", draw(&app, 90, 18));
}

#[test]
fn rendered_body_reflows_and_yank_copies_the_source_markdown() {
    // yaks-649d: the body is rendered (markers hidden, soft breaks joined,
    // list items hanging), and yanking rendered rows copies the source.
    let mut t = task("m0", "Md", Status::Hairy, 3, None);
    t.body = "Some **bold**\ntext here.\n\n- item one that is long enough to wrap around".into();
    let mut app = App::new(vec![t]);
    enter_key(&mut app);
    let frame = draw(&app, 40, 20);
    assert!(frame.contains("Some bold text here."), "{frame}");
    assert!(!frame.contains("**"), "{frame}");
    let lines = app.detail_dlines();
    let first = lines
        .iter()
        .position(|l| l.text == "Some bold text here.")
        .unwrap();
    let last = lines.len() - 1;
    assert!(
        lines[last].text.starts_with("  "),
        "hanging indent: {:?}",
        lines[last].text
    );
    assert_eq!(
        app.selection_text(first, last).unwrap(),
        "Some **bold**\ntext here.\n\n- item one that is long enough to wrap around"
    );
}

// -- yaks-c733: no-wrap blocks scroll sideways, per block -------------------

/// A yak whose body holds a long code block, a prose line and a wide table.
fn hscroll_app() -> App {
    let mut t = task("h0", "Scroll", Status::Hairy, 3, None);
    t.body = "Intro.\n\n```java\nclass Long { void aVeryLongMethodNameThatOverflows() {} }\nshort\n```\n\nOutro.\n\n| a | b |\n|---|---|\n| first column text | second column text that is wide |\n"
        .into();
    let mut app = App::new(vec![t]);
    enter_key(&mut app);
    let _ = draw(&app, 30, 30); // measure the pane width
    app
}

fn press_char(app: &mut App, c: char) {
    handle_key(app, key(c));
}

/// The frame's detail-pane rows, without the pane's left divider.
fn pane_rows(frame: &str) -> Vec<String> {
    frame
        .lines()
        .map(|l| l.strip_prefix("│ ").unwrap_or(l).to_string())
        .collect()
}

/// The detail line index of the first row whose text contains `needle`.
fn dline_of(app: &App, needle: &str) -> usize {
    app.detail_dlines()
        .iter()
        .position(|l| l.text.contains(needle))
        .unwrap_or_else(|| panic!("no detail row contains {needle:?}"))
}

#[test]
fn code_block_shows_language_label_and_clips_with_a_marker() {
    let app = hscroll_app();
    let frame = draw(&app, 30, 30);
    assert!(!frame.contains("```"), "fences are hidden:\n{frame}");
    assert!(
        pane_rows(&frame).iter().any(|l| l.trim() == "java"),
        "label row:\n{frame}"
    );
    let long = pane_rows(&frame)
        .into_iter()
        .find(|l| l.contains("class Long"))
        .unwrap_or_else(|| panic!("no code row:\n{frame}"));
    assert!(
        long.trim_end().ends_with('›'),
        "right edge marker: {long:?}"
    );
    assert!(
        !long.contains('‹'),
        "nothing hidden on the left yet: {long:?}"
    );
}

#[test]
fn l_scrolls_only_the_block_under_the_cursor_and_h_returns() {
    let mut app = hscroll_app();
    let code = dline_of(&app, "class Long");
    let table = dline_of(&app, "first column");
    app.detail_line = code;
    let before = draw(&app, 30, 30);
    press_char(&mut app, 'l');
    let after = draw(&app, 30, 30);
    let row = |f: &str, needle: &str| pane_rows(f).into_iter().find(|l| l.contains(needle));
    assert_ne!(row(&before, "class Long"), row(&after, "class Long"));
    assert!(
        pane_rows(&after).iter().any(|l| l.starts_with('‹')),
        "left marker once scrolled:\n{after}"
    );
    // The label, the prose and the table did not move.
    assert_eq!(row(&before, "Intro."), row(&after, "Intro."));
    assert_eq!(row(&before, "first column"), row(&after, "first column"));
    assert_eq!(row(&before, "java"), row(&after, "java"));
    // `l` on prose scrolls nothing (and does not leave the pane).
    app.detail_line = dline_of(&app, "Intro.");
    press_char(&mut app, 'l');
    assert_eq!(app.focus, Focus::Detail);
    // ...and scrolling is per block: the table starts at 0.
    app.detail_line = table;
    let t = draw(&app, 30, 30);
    assert!(
        !pane_rows(&t).iter().any(|l| l.starts_with('‹')),
        "the table was not scrolled:\n{t}"
    );
}

#[test]
fn h_scrolls_back_then_at_zero_returns_to_the_list() {
    let mut app = hscroll_app();
    app.detail_line = dline_of(&app, "class Long");
    press_char(&mut app, 'l');
    press_char(&mut app, 'l');
    press_char(&mut app, 'h');
    assert_eq!(app.focus, Focus::Detail, "still scrolled: h scrolls back");
    press_char(&mut app, 'h');
    assert_eq!(app.focus, Focus::Detail, "now at 0, but that h scrolled");
    press_char(&mut app, 'h');
    assert_eq!(
        app.focus,
        Focus::List,
        "at offset 0 h goes back to the list"
    );
}

#[test]
fn h_on_prose_returns_to_the_list_and_esc_always_does() {
    let mut app = hscroll_app();
    app.detail_line = dline_of(&app, "Intro.");
    press_char(&mut app, 'h');
    assert_eq!(app.focus, Focus::List);
    let mut app = hscroll_app();
    app.detail_line = dline_of(&app, "class Long");
    press_char(&mut app, 'l');
    esc_key(&mut app);
    assert_eq!(app.focus, Focus::List, "Esc leaves even a scrolled block");
}

#[test]
fn scroll_clamps_at_the_widest_row_and_resets_when_the_cursor_leaves() {
    let mut app = hscroll_app();
    app.detail_line = dline_of(&app, "class Long");
    for _ in 0..40 {
        press_char(&mut app, 'l');
    }
    let frame = draw(&app, 30, 30);
    let row = pane_rows(&frame)
        .into_iter()
        .find(|l| l.contains("Overflows"))
        .unwrap_or_else(|| panic!("end of the line is visible:\n{frame}"));
    assert!(
        !row.trim_end().ends_with('›'),
        "no right marker at the end: {row:?}"
    );
    // Leaving the block (j onto the next row is still the block; go to prose).
    app.detail_line = dline_of(&app, "Outro.");
    press_char(&mut app, 'j');
    app.detail_line = dline_of(&app, "class Long");
    let back = draw(&app, 30, 30);
    assert!(
        back.lines().any(|l| l.contains("class Long")),
        "the block starts at its left edge again:\n{back}"
    );
}

#[test]
fn yank_of_a_scrolled_code_row_copies_the_full_source_line() {
    let mut app = hscroll_app();
    let code = dline_of(&app, "class Long");
    app.detail_line = code;
    press_char(&mut app, 'l');
    assert_eq!(
        app.selection_text(code, code).unwrap(),
        "class Long { void aVeryLongMethodNameThatOverflows() {} }"
    );
}

/// A yak whose body has a footnote, an attachment link and a URL link.
fn linked_footnote() -> App {
    let mut a0 = task("a0", "Root A", Status::Hairy, 2, None);
    a0.body = "Claim[^1].\n\nSee [the docs](https://example.com/docs) and [log](a0/log.txt).\n\n[^1]: Because.\n"
        .into();
    App::new(vec![a0])
}

#[test]
fn following_a_footnote_reference_jumps_to_its_definition_and_o_returns() {
    // yaks-1bbb
    let mut app = linked_footnote();
    enter_key(&mut app);
    draw(&app, 80, 20); // measure the pane
    tab_key(&mut app);
    let from = app.detail_line;
    assert!(app.detail_dlines()[from].text.starts_with("Claim[1]"));
    enter_key(&mut app);
    let to = app.detail_line;
    assert!(to > from);
    assert_eq!(app.detail_dlines()[to].text, "[1] Because.");
    press(&mut app, "o");
    assert_eq!(app.detail_line, from);
}

#[test]
fn status_bar_shows_the_destination_of_the_link_under_the_cursor() {
    // yaks-3b8d
    let mut app = linked_footnote();
    enter_key(&mut app);
    draw(&app, 80, 20);
    tab_key(&mut app); // the footnote row: nothing to show
    tab_key(&mut app); // the row holding the URL and the attachment
    let frame = draw(&app, 80, 20);
    let status = frame.lines().last().unwrap();
    assert_eq!(status, "→ https://example.com/docs", "{frame}");
    // A row with no external link goes back to the key hints.
    press(&mut app, "g");
    assert!(
        draw(&app, 80, 20)
            .lines()
            .last()
            .unwrap()
            .contains("Enter:follow")
    );
}

#[test]
fn status_bar_shows_path_lines_and_pin_of_a_source_link() {
    // yaks-da5e
    let mut a0 = task("a0", "Root A", Status::Hairy, 2, None);
    a0.body = "See [the code](../src/foo.rs#L2-L4) and [then](../blob/abc1234def/src/foo.rs#L9).\n"
        .into();
    let mut app = App::new(vec![a0]);
    enter_key(&mut app);
    draw(&app, 80, 20);
    tab_key(&mut app);
    let frame = draw(&app, 80, 20);
    assert!(frame.contains("See the code and then."), "{frame}");
    let status = frame.lines().last().unwrap();
    assert_eq!(status, "→ src/foo.rs:L2-L4", "{frame}");
}

/// A farm whose only yak `t0` has attachment `name` holding `data`, opened
/// with the detail cursor on its link.
fn app_on_attachment(name: &str, data: &[u8]) -> (PathBuf, App) {
    let (dir, farm) = temp_farm(&[task("t0", "solo", Status::Hairy, 3, None)]);
    farm.attach("t0", name, data).unwrap();
    let mut app = App::with_farm(farm).unwrap();
    press(&mut app, "l");
    draw(&app, 100, 24); // measure the pane
    tab_key(&mut app);
    (dir, app)
}

#[test]
fn enter_on_a_markdown_attachment_opens_the_viewer_rendered() {
    // yaks-699b: a text attachment opens in the TUI, markdown rendered.
    let (_dir, mut app) = app_on_attachment(
        "notes.md",
        b"# Findings\n\nSome **bold** text\nthat reflows.\n\n- one\n- two\n",
    );
    enter_key(&mut app);
    assert!(matches!(app.overlay, Overlay::Attachment(_)));
    assert_eq!(app.notification.as_deref(), Some("viewing t0/notes.md"));
    let frame = draw(&app, 100, 24);
    assert!(frame.contains("t0/notes.md  markdown"), "{frame}");
    // Markers hidden, soft break joined, bullets drawn.
    assert!(
        frame.contains("Findings") && !frame.contains("# Findings"),
        "{frame}"
    );
    assert!(frame.contains("Some bold text that reflows."), "{frame}");
    assert!(frame.contains("• one"), "{frame}");
    assert!(frame.contains("O open externally"), "{frame}");
    press(&mut app, "q");
    assert!(matches!(app.overlay, Overlay::None));
}

#[test]
fn a_plain_text_attachment_is_shown_verbatim_and_scrolls() {
    let text: String = (1..=60)
        .map(|i| format!("line {i} **not bold**\n"))
        .collect();
    let (_dir, mut app) = app_on_attachment("run.log", text.as_bytes());
    press(&mut app, "O");
    let frame = draw(&app, 100, 24);
    assert!(frame.contains("t0/run.log  text"), "{frame}");
    assert!(frame.contains("line 1 **not bold**"), "{frame}");
    press(&mut app, "G");
    let frame = draw(&app, 100, 24);
    assert!(frame.contains("line 60 **not bold**"), "{frame}");
    assert!(!frame.contains("line 1 **"), "{frame}");
    press(&mut app, "g");
    assert!(draw(&app, 100, 24).contains("line 1 **not bold**"));
}

#[test]
fn an_image_or_binary_attachment_still_opens_externally() {
    // The OS opener is stubbed under `cargo test`; the overlay must stay shut.
    let (_dir, mut app) = app_on_attachment("shot.png", b"plain text but an image name");
    enter_key(&mut app);
    assert!(matches!(app.overlay, Overlay::None));
    assert_eq!(app.notification.as_deref(), Some("opened t0/shot.png"));
    let (_dir, mut app) = app_on_attachment("blob.bin", &[0xff, 0xfe, 0x00, 0x01]);
    enter_key(&mut app);
    assert!(matches!(app.overlay, Overlay::None));
    assert_eq!(app.notification.as_deref(), Some("opened t0/blob.bin"));
}

/// A `w`×`h` RGB PNG, red on the top half and blue on the bottom.
fn two_tone_png(w: u32, h: u32) -> Vec<u8> {
    let mut out = Vec::new();
    let mut enc = png::Encoder::new(&mut out, w, h);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    let data: Vec<u8> = (0..h)
        .flat_map(|y| (0..w).flat_map(move |_| if y < h / 2 { [255, 0, 0] } else { [0, 0, 255] }))
        .collect();
    enc.write_header().unwrap().write_image_data(&data).unwrap();
    out
}

#[test]
fn enter_on_an_image_attachment_shows_it_in_the_pane() {
    // yaks-4305: a decodable image opens in the TUI, drawn as half-block cells
    // when the terminal speaks no graphics protocol (as under test).
    let (_dir, mut app) = app_on_attachment("shot.png", &two_tone_png(64, 64));
    enter_key(&mut app);
    assert!(matches!(app.overlay, Overlay::Image(_)));
    assert_eq!(app.notification.as_deref(), Some("viewing t0/shot.png"));
    let mut term = Terminal::new(ratatui::backend::TestBackend::new(100, 24)).unwrap();
    term.draw(|f| render(&app, f)).unwrap();
    let buf = term.backend().buffer();
    let text = buffer_to_string(buf);
    assert!(text.contains("t0/shot.png  image 64×64"), "{text}");
    assert!(text.contains("O open externally"), "{text}");
    // Red over blue: pixels were drawn as colored cells, not a placeholder.
    let has = |rgb: Color| buf.content.iter().any(|c| c.fg == rgb || c.bg == rgb);
    assert!(has(Color::Rgb(255, 0, 0)), "no red cells:\n{text}");
    assert!(has(Color::Rgb(0, 0, 255)), "no blue cells:\n{text}");
    press(&mut app, "q");
    assert!(matches!(app.overlay, Overlay::None));
}

#[test]
fn an_undecodable_image_attachment_falls_back_to_the_os_opener() {
    // svg is an image by name but not decodable by the `image` crate; so is a
    // corrupt png. Neither opens the viewer.
    let (_dir, mut app) =
        app_on_attachment("logo.svg", b"<svg xmlns='http://www.w3.org/2000/svg'/>");
    enter_key(&mut app);
    assert!(matches!(app.overlay, Overlay::None));
    assert_eq!(app.notification.as_deref(), Some("opened t0/logo.svg"));
}

#[test]
fn o_in_the_image_viewer_opens_externally_and_closes_it() {
    let (_dir, mut app) = app_on_attachment("shot.png", &two_tone_png(8, 8));
    enter_key(&mut app);
    press(&mut app, "O");
    assert!(matches!(app.overlay, Overlay::None));
    assert_eq!(app.notification.as_deref(), Some("opened t0/shot.png"));
}

fn put_attachment(farm: &crate::farm::Farm, id: &str, name: &str, data: &[u8]) {
    let dir = crate::store::artifact_dir(farm.root(), id);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(name), data).unwrap();
}

/// A farm whose yak `t0` has body `body` and a 120x120 two-tone PNG attachment
/// `shot.png`, opened in the detail pane at 80 columns.
fn app_with_inline_image(body: &str, inline: bool) -> (PathBuf, App) {
    let mut t0 = task("t0", "solo", Status::Hairy, 3, None);
    t0.body = body.to_string();
    let (dir, farm) = temp_farm(&[t0]);
    // Written straight into the attachment directory: `Farm::attach` would
    // also append its own image link to the body, hiding what each test sets.
    put_attachment(&farm, "t0", "shot.png", &two_tone_png(120, 120));
    let mut app = App::with_farm(farm).unwrap();
    app.inline_images = inline;
    HeadlessApp::on_resize(&mut app, 100, 40);
    press(&mut app, "l");
    draw(&app, 100, 40); // measure the pane
    (dir, app)
}

fn image_rows(app: &App) -> Vec<usize> {
    app.detail_dlines()
        .iter()
        .enumerate()
        .filter(|(_, l)| l.image.is_some())
        .map(|(i, _)| i)
        .collect()
}

#[test]
fn an_image_alone_in_a_paragraph_reserves_rows_in_the_detail_pane() {
    // yaks-0bf0: 120x120 px at the half-block picker's 10x20 px cells is 12x6
    // cells, under the pane width and the row cap.
    let (_dir, app) = app_with_inline_image("Before.\n\n![shot](t0/shot.png)\n\nAfter.", true);
    let rows = image_rows(&app);
    assert_eq!(rows.len(), 6, "{rows:?}");
    assert!(rows.windows(2).all(|w| w[1] == w[0] + 1), "contiguous");
    let lines = app.detail_dlines();
    let slot = |i: usize| lines[i].image.as_ref().unwrap();
    assert_eq!((slot(rows[0]).cols, slot(rows[0]).rows), (12, 6));
    assert_eq!(
        rows.iter().map(|&i| slot(i).row).collect::<Vec<_>>(),
        vec![0, 1, 2, 3, 4, 5]
    );
    // The rows follow the `🖼 shot` link row, which stays followable.
    let link = rows[0] - 1;
    assert!(
        lines[link].text.contains("🖼 shot"),
        "{:?}",
        lines[link].text
    );
    assert!(!lines[link].links.is_empty());
    // And they belong to the paragraph: same source block as the link row.
    assert_eq!(
        lines[rows[0]].src.unwrap().block,
        lines[link].src.unwrap().block
    );
}

#[test]
fn an_image_in_text_a_list_or_with_inline_images_off_stays_a_link_row() {
    for body in [
        "see ![shot](t0/shot.png) here",
        "- ![shot](t0/shot.png)",
        "> ![shot](t0/shot.png)",
    ] {
        let (_dir, app) = app_with_inline_image(body, true);
        assert!(image_rows(&app).is_empty(), "{body:?}");
    }
    let (_dir, app) = app_with_inline_image("![shot](t0/shot.png)", false);
    assert!(image_rows(&app).is_empty(), "inline_images off");
    // A remote image or a missing file has no rows to reserve either.
    for body in ["![x](https://example.com/a.png)", "![x](t0/missing.png)"] {
        let (_dir, app) = app_with_inline_image(body, true);
        assert!(image_rows(&app).is_empty(), "{body:?}");
    }
}

#[test]
fn an_explicit_size_sets_the_reserved_rows() {
    // `=x60` is 60 px tall: 3 rows of 20 px, width from the aspect (6 cols).
    let (_dir, app) = app_with_inline_image("![shot =x60](t0/shot.png)", true);
    let lines = app.detail_dlines();
    let rows = image_rows(&app);
    assert_eq!(rows.len(), 3);
    let s = lines[rows[0]].image.as_ref().unwrap();
    assert_eq!((s.cols, s.rows), (6, 3));
}

/// Screen rows of a `w`x`h` frame that carry image pixels. The test image is
/// pure red over pure blue; the half-block encoder paints a uniform cell as a
/// background color on a space, so look at colors, not glyphs.
fn image_pixel_rows(app: &App, w: u16, h: u16) -> usize {
    let mut term = Terminal::new(ratatui::backend::TestBackend::new(w, h)).unwrap();
    term.draw(|f| render(app, f)).unwrap();
    let buf = term.backend().buffer();
    let is_img = |c: Color| c == Color::Rgb(255, 0, 0) || c == Color::Rgb(0, 0, 255);
    (0..h)
        .filter(|&y| {
            (0..w).any(|x| {
                let c = &buf[(x, y)];
                is_img(c.fg) || is_img(c.bg)
            })
        })
        .count()
}

#[test]
fn an_inline_image_is_drawn_clipped_at_the_pane_edge_and_scrolls() {
    // A short pane so the image straddles the bottom edge, then the top.
    let body = format!(
        "{}\n\n![shot](t0/shot.png)\n\nEnd.",
        "Line.\n\nMore.\n\n".repeat(3)
    );
    let (_dir, mut app) = app_with_inline_image(&body, true);
    HeadlessApp::on_resize(&mut app, 100, 14);
    draw(&app, 100, 14);
    let first = image_rows(&app)[0];
    // Scroll so the image starts two rows above the bottom edge of the content
    // area (once scrolled, a pinned header takes two rows of the pane): two of
    // its six rows are visible, clipped, not hidden.
    let shown = app.detail_rows_shown(1);
    app.detail_scroll = (first + 2 - shown) as u16;
    let n = image_pixel_rows(&app, 100, 14);
    assert_eq!(n, 2, "two rows visible, clipped at the pane edge");
    // Scrolled so the whole image is in view: every row drawn.
    app.detail_scroll = first as u16 - 1;
    assert_eq!(image_pixel_rows(&app, 100, 14), 6, "whole image drawn");
    // Scrolled until it is off the top: nothing drawn.
    app.detail_scroll = (image_rows(&app)[5] + 1) as u16;
    assert_eq!(image_pixel_rows(&app, 100, 14), 0, "off the top: not drawn");
}

#[test]
fn a_broken_inline_image_is_not_retried_every_frame_and_draws_nothing() {
    let mut t0 = task("t0", "solo", Status::Hairy, 3, None);
    t0.body = "![shot](t0/shot.png)".into();
    let (_dir, farm) = temp_farm(&[t0]);
    // A PNG header that `into_dimensions` accepts, then garbage: layout
    // reserves rows, decode fails at draw time.
    let mut png = two_tone_png(40, 40);
    png.truncate(png.len() / 2);
    put_attachment(&farm, "t0", "shot.png", &png);
    let mut app = App::with_farm(farm).unwrap();
    HeadlessApp::on_resize(&mut app, 100, 30);
    press(&mut app, "l");
    for _ in 0..3 {
        assert_eq!(image_pixel_rows(&app, 100, 30), 0);
    }
    assert_eq!(app.inline_cache.borrow().len(), 1, "one remembered failure");
}

/// The attachment viewer's `E` (yaks-56c1): open the editor over the file.
fn edit_attachment(app: &mut App) -> &Editor {
    press(app, "E");
    match &app.overlay {
        Overlay::Edit(ed) => {
            assert!(matches!(ed.action, EditAction::EditAttachment(_)));
            ed
        }
        _ => panic!("E in the viewer should open the editor"),
    }
}

fn ctrl_key(app: &mut App, c: char) {
    handle_key(app, KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL));
}

#[test]
fn e_in_the_viewer_edits_the_attachment_and_saves_it_back() {
    let (dir, mut app) = app_on_attachment("notes.md", b"- one\n- two");
    let path = dir.join(".yaks/t0/yaks/t0/notes.md");
    enter_key(&mut app);
    assert_eq!(edit_attachment(&mut app).text(), "- one\n- two");
    let frame = draw(&app, 100, 24);
    assert!(frame.contains("Edit t0/notes.md"), "{frame}");
    // Vim Normal on existing text: `i` inserts at the cursor (the start).
    press(&mut app, "i# Findings");
    enter_key(&mut app);
    press(&mut app, "**new** ");
    esc_key(&mut app);
    ctrl_key(&mut app, 's');
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "# Findings\n**new** - one\n- two"
    );
    assert_eq!(app.notification.as_deref(), Some("saved t0/notes.md"));
    // Back in the viewer, the new text rendered as markdown.
    assert!(matches!(app.overlay, Overlay::Attachment(_)));
    let frame = draw(&app, 100, 24);
    assert!(
        frame.contains("Findings") && !frame.contains("# Findings"),
        "{frame}"
    );
    assert!(
        frame.contains("new - one") && !frame.contains("**new**"),
        "{frame}"
    );
    assert!(frame.contains("E edit"), "{frame}");
}

#[test]
fn cancelling_an_attachment_edit_leaves_the_file_unchanged() {
    let (dir, mut app) = app_on_attachment("run.log", b"line 1\n");
    let path = dir.join(".yaks/t0/yaks/t0/run.log");
    enter_key(&mut app);
    edit_attachment(&mut app);
    press(&mut app, "iextra");
    // Ctrl-C on typed text asks first; declining keeps the edit.
    ctrl_key(&mut app, 'c');
    assert!(matches!(app.overlay, Overlay::Confirm { .. }));
    press(&mut app, "n");
    assert!(matches!(app.overlay, Overlay::Edit(_)));
    ctrl_key(&mut app, 'c');
    press(&mut app, "y");
    assert!(
        matches!(app.overlay, Overlay::Attachment(_)),
        "back to the viewer"
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "line 1\n");
    // An untouched edit cancels straight back to the viewer.
    edit_attachment(&mut app);
    ctrl_key(&mut app, 'c');
    assert!(matches!(app.overlay, Overlay::Attachment(_)));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "line 1\n");
}

#[test]
fn an_attachment_changed_on_disk_meanwhile_is_not_overwritten() {
    let (dir, mut app) = app_on_attachment("run.log", b"line 1\n");
    let path = dir.join(".yaks/t0/yaks/t0/run.log");
    enter_key(&mut app);
    edit_attachment(&mut app);
    press(&mut app, "imine");
    std::fs::write(&path, "theirs\n").unwrap();
    ctrl_key(&mut app, 's');
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "theirs\n");
    match &app.overlay {
        Overlay::Edit(ed) => assert_eq!(ed.text(), "mineline 1\n"),
        _ => panic!("the editor stays open with the typed text"),
    }
    // A second save is a deliberate overwrite.
    ctrl_key(&mut app, 's');
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "mineline 1\n");
    assert!(matches!(app.overlay, Overlay::Attachment(_)));
}

#[test]
fn an_image_attachment_offers_no_edit() {
    // An image never reaches the viewer, so there is no `E` to press.
    let (dir, mut app) = app_on_attachment("shot.png", b"plain text but an image name");
    enter_key(&mut app);
    assert!(matches!(app.overlay, Overlay::None));
    press(&mut app, "E");
    assert!(!matches!(
        &app.overlay,
        Overlay::Edit(Editor {
            action: EditAction::EditAttachment(_),
            ..
        })
    ));
    assert_eq!(
        std::fs::read(dir.join(".yaks/t0/yaks/t0/shot.png")).unwrap(),
        b"plain text but an image name"
    );
}

#[test]
fn rendering_a_code_block_does_not_rebuild_rows_per_row() {
    // yaks-48d6: render_detail asked `block_hscroll` (and so `selected()`,
    // and so a full `rows()` rebuild) once per row of every no-wrap block, so
    // a long code block made each frame cost one view rebuild per code line.
    let mut t = task("c0", "Code", Status::Hairy, 3, None);
    let code: Vec<String> = (0..40).map(|i| format!("let x{i} = {i};")).collect();
    t.body = format!("Intro.\n\n```rust\n{}\n```", code.join("\n"));
    let mut app = App::new(vec![t]);
    enter_key(&mut app);
    draw(&app, 100, 60); // capture the pane width
    let before = super::viewmodel::ROWS_CALLS.with(|c| c.get());
    draw(&app, 100, 60);
    let per_frame = super::viewmodel::ROWS_CALLS.with(|c| c.get()) - before;
    assert!(
        per_frame <= 10,
        "one frame rebuilt the rows {per_frame} times (40 code rows on screen)"
    );
}

/// In-place block editing (yaks-597d): `E` on a body row and `M` edit one
/// content block inside the detail pane.
mod block_edit {
    use super::*;

    const DESC: &str = "The description.\n\n## Plan\n\n- one\n- two";

    /// A farm-backed yak: a description and two comments (the second a list,
    /// one item per source line), with the detail pane open and measured.
    fn app_with_comments() -> (std::path::PathBuf, App) {
        let mut t = task("t0", "solo", Status::Hairy, 3, None);
        let b = crate::store::append_note(DESC, "2026-01-01T00:00:00Z", Some("joel"), "first note");
        t.body = crate::store::append_note(
            &b,
            "2026-01-02T00:00:00Z",
            Some("agent-1"),
            "- alpha\n- beta\n- gamma",
        );
        let (dir, farm) = temp_farm(&[t]);
        let mut app = App::with_farm(farm).unwrap();
        app.detail_page = 27;
        handle_key(&mut app, key('l'));
        let _ = draw(&app, 90, 30); // measure the pane width
        (dir, app)
    }

    fn ctrl(app: &mut App, c: char) {
        handle_key(app, KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL));
    }

    /// Append `text` to the cursor's line from Normal mode (`A`, which edtui
    /// binds with SHIFT, then the text).
    fn append(app: &mut App, text: &str) {
        handle_key(app, KeyEvent::new(KeyCode::Char('A'), KeyModifiers::SHIFT));
        typ(app, text);
    }

    fn body(app: &App) -> String {
        app.task("t0").unwrap().body.clone()
    }

    fn block_edit(app: &App) -> &BlockEdit {
        match &app.overlay {
            Overlay::BlockEdit(be) => be,
            _ => panic!("expected the in-place editor"),
        }
    }

    #[test]
    fn e_on_a_comment_row_opens_the_editor_on_the_source_line_under_the_cursor() {
        let (_d, mut app) = app_with_comments();
        app.detail_line = dline_of(&app, "beta");
        handle_key(&mut app, key('E'));
        let be = block_edit(&app);
        assert!(matches!(
            &be.editor.action,
            EditAction::Block { block: 2, .. }
        ));
        assert_eq!(be.editor.text(), "- alpha\n- beta\n- gamma");
        assert_eq!(
            be.editor.state.borrow().cursor.row,
            1,
            "source line of `beta`"
        );
        // Shows raw source in place, under its own header, bar in the gutter.
        let frame = draw(&app, 90, 30);
        assert!(frame.contains("┃- beta"), "{frame}");
        assert!(frame.contains("agent-1 · "), "{frame}");
        assert!(
            frame.contains("editing · Ctrl-S save · Ctrl-C cancel"),
            "{frame}"
        );
        // ...and the other blocks stay rendered around it.
        assert!(frame.contains("The description."), "{frame}");
        assert!(frame.contains("first note"), "{frame}");
    }

    #[test]
    fn e_on_the_description_edits_block_zero() {
        let (_d, mut app) = app_with_comments();
        app.detail_line = dline_of(&app, "The description.");
        handle_key(&mut app, key('E'));
        assert!(matches!(
            &block_edit(&app).editor.action,
            EditAction::Block { block: 0, .. }
        ));
        assert_eq!(block_edit(&app).editor.state.borrow().cursor.row, 0);
    }

    #[test]
    fn e_on_a_header_row_still_opens_the_form() {
        let (_d, mut app) = app_with_comments();
        app.detail_line = dline_of(&app, "Title:");
        handle_key(&mut app, key('E'));
        assert!(matches!(app.overlay, Overlay::Create(_)));
    }

    #[test]
    fn rows_above_the_editor_stay_put() {
        let (_d, mut app) = app_with_comments();
        let before = pane_rows(&draw(&app, 90, 30));
        app.detail_line = dline_of(&app, "beta");
        handle_key(&mut app, key('E'));
        let after = pane_rows(&draw(&app, 90, 30));
        let header = after
            .iter()
            .position(|l| l.contains("agent-1 · "))
            .expect("the edited block's header");
        assert_eq!(app.detail_scroll, 0, "no scroll needed: the editor fits");
        assert_eq!(&before[..header - 1], &after[..header - 1]);
    }

    #[test]
    fn ctrl_s_saves_only_that_block() {
        let (_d, mut app) = app_with_comments();
        let before = body(&app);
        app.detail_line = dline_of(&app, "beta");
        handle_key(&mut app, key('E'));
        // Normal mode, cursor on `- beta`: append to the line.
        append(&mut app, "!");
        ctrl(&mut app, 's');
        assert!(matches!(app.overlay, Overlay::None));
        assert_eq!(body(&app), before.replace("- beta", "- beta!"));
        // The description and the first comment are byte-identical.
        let a = content::parse(&before);
        let b = content::parse(&body(&app));
        assert_eq!(a[0], b[0]);
        assert_eq!(a[1], b[1]);
        assert_ne!(a[2], b[2]);
    }

    #[test]
    fn ctrl_s_without_edits_changes_nothing() {
        let (_d, mut app) = app_with_comments();
        let before = body(&app);
        app.detail_line = dline_of(&app, "first note");
        handle_key(&mut app, key('E'));
        ctrl(&mut app, 's');
        assert_eq!(body(&app), before);
        assert_eq!(app.notification.as_deref(), Some("t0 unchanged"));
    }

    #[test]
    fn saving_the_description_leaves_the_comments_alone() {
        let (_d, mut app) = app_with_comments();
        let before = body(&app);
        app.detail_line = dline_of(&app, "The description.");
        handle_key(&mut app, key('E'));
        append(&mut app, " edited");
        ctrl(&mut app, 's');
        assert_eq!(
            body(&app),
            before.replacen("The description.", "The description. edited", 1)
        );
    }

    /// yaks-9f5c: a note another writer appends while the editor is open
    /// survives the save.
    #[test]
    fn saving_keeps_a_note_appended_meanwhile() {
        let (_d, mut app) = app_with_comments();
        app.detail_line = dline_of(&app, "The description.");
        handle_key(&mut app, key('E'));
        let other = crate::farm::TaskEdit {
            note: Some("written by another shed".into()),
            ..Default::default()
        };
        app.farm.as_ref().unwrap().update("t0", other).unwrap();
        append(&mut app, " edited");
        ctrl(&mut app, 's');
        assert!(matches!(app.overlay, Overlay::None));
        let now = body(&app);
        assert!(now.starts_with("The description. edited\n"), "{now}");
        assert!(now.contains("- gamma"), "{now}");
        assert!(now.ends_with("\n\nwritten by another shed"), "{now}");
        assert_eq!(app.notification.as_deref(), Some("t0 updated"));
    }

    /// yaks-9f5c: a concurrent rewrite is a conflict: nothing is written and
    /// the editor reopens on the fresh yak holding the user's text.
    #[test]
    fn a_conflicting_save_writes_nothing_and_keeps_the_text() {
        let (_d, mut app) = app_with_comments();
        app.detail_line = dline_of(&app, "beta");
        handle_key(&mut app, key('E'));
        let theirs = body(&app).replace("first note", "their rewrite");
        let other = crate::farm::TaskEdit {
            description: Some(theirs.clone()),
            ..Default::default()
        };
        app.farm.as_ref().unwrap().update("t0", other).unwrap();
        append(&mut app, "!");
        ctrl(&mut app, 's');
        assert_eq!(body(&app), theirs, "nothing written; the App reloaded");
        let be = block_edit(&app);
        assert_eq!(be.editor.text(), "- alpha\n- beta!\n- gamma");
        assert!(be.is_dirty(), "a cancel still asks");
        assert_eq!(
            app.notification.as_deref(),
            Some("t0 changed on disk; not saved: review and Ctrl-S again")
        );
        let frame = draw(&app, 90, 30);
        assert!(frame.contains("their rewrite"), "{frame}");
        assert!(frame.contains("┃- beta!"), "{frame}");
        // Saving again goes against the fresh body and keeps their rewrite.
        ctrl(&mut app, 's');
        assert!(matches!(app.overlay, Overlay::None));
        assert_eq!(body(&app), theirs.replace("- beta", "- beta!"));
    }

    #[test]
    fn after_saving_the_line_cursor_stays_on_the_same_source_line() {
        let (_d, mut app) = app_with_comments();
        app.detail_line = dline_of(&app, "beta");
        handle_key(&mut app, key('E'));
        append(&mut app, "!");
        ctrl(&mut app, 's');
        assert_eq!(app.detail_line, dline_of(&app, "beta!"));
    }

    #[test]
    fn m_adds_a_comment_at_the_end() {
        let (_d, mut app) = app_with_comments();
        let before = body(&app);
        handle_key(&mut app, key('M'));
        assert!(matches!(
            &block_edit(&app).editor.action,
            EditAction::Comment(_)
        ));
        let frame = draw(&app, 90, 30);
        assert!(frame.contains("· now"), "provisional header:\n{frame}");
        assert!(
            frame.contains("• gamma"),
            "the comments stay visible:\n{frame}"
        );
        typ(&mut app, "a fresh note");
        ctrl(&mut app, 's');
        assert!(matches!(app.overlay, Overlay::None));
        let now = body(&app);
        assert!(now.starts_with(&before), "existing blocks untouched");
        assert!(now.ends_with("\n\na fresh note"), "{now:?}");
        let blocks = content::parse(&now);
        assert_eq!(blocks.len(), 4);
        // The line cursor sits on the new comment's header.
        assert_eq!(
            app.detail_dlines()[app.detail_line].src.map(|s| s.block),
            Some(3)
        );
    }

    #[test]
    fn an_empty_new_comment_is_not_saved() {
        let (_d, mut app) = app_with_comments();
        let before = body(&app);
        handle_key(&mut app, key('M'));
        ctrl(&mut app, 's');
        assert_eq!(body(&app), before);
        assert_eq!(app.notification.as_deref(), Some("comment cancelled"));
    }

    #[test]
    fn esc_esc_with_edits_asks_to_confirm_and_n_returns_to_the_editor() {
        let (_d, mut app) = app_with_comments();
        let before = body(&app);
        app.detail_line = dline_of(&app, "beta");
        handle_key(&mut app, key('E'));
        append(&mut app, "!");
        esc_key(&mut app);
        esc_key(&mut app); // rapid double-Esc = cancel
        assert!(matches!(app.overlay, Overlay::Confirm { .. }));
        // The editor region stays drawn behind the prompt.
        assert!(draw(&app, 90, 30).contains("┃- beta!"));
        press(&mut app, "n");
        assert_eq!(block_edit(&app).editor.text(), "- alpha\n- beta!\n- gamma");
        esc_key(&mut app);
        esc_key(&mut app);
        press(&mut app, "y");
        assert!(matches!(app.overlay, Overlay::None));
        assert_eq!(body(&app), before, "nothing saved");
        assert_eq!(app.notification.as_deref(), Some("changes discarded"));
    }

    #[test]
    fn ctrl_c_without_edits_cancels_at_once() {
        let (_d, mut app) = app_with_comments();
        app.detail_line = dline_of(&app, "beta");
        handle_key(&mut app, key('E'));
        ctrl(&mut app, 'c');
        assert!(matches!(app.overlay, Overlay::None));
        assert_eq!(app.notification.as_deref(), Some("cancelled"));
    }

    #[test]
    fn a_tall_editor_is_capped_and_scrolls_inside() {
        let mut t = task("t0", "solo", Status::Hairy, 3, None);
        let long: Vec<String> = (0..60).map(|i| format!("- item {i}")).collect();
        t.body = format!("Intro.\n\n{}", long.join("\n"));
        let (_d, farm) = temp_farm(&[t]);
        let mut app = App::with_farm(farm).unwrap();
        app.detail_page = 27;
        handle_key(&mut app, key('l'));
        let _ = draw(&app, 90, 30);
        app.detail_line = dline_of(&app, "item 40");
        handle_key(&mut app, key('E'));
        let lines = app.detail_dlines();
        let rows = lines
            .iter()
            .filter(|l| l.kind == detail::Kind::Editor)
            .count();
        assert!(rows < 27, "{rows} editor rows must leave context");
        assert_eq!(app.edit_slot().unwrap().rows, rows);
        // The whole editor is on screen, and the cursor's item is in it.
        let frame = draw(&app, 90, 30);
        assert!(frame.contains("┃- item 40"), "{frame}");
        let first = lines
            .iter()
            .position(|l| l.kind == detail::Kind::Editor)
            .unwrap();
        let (top, shown) = (
            app.detail_scroll as usize,
            app.detail_rows_shown(app.detail_scroll),
        );
        assert!(
            first > top && first + rows <= top + shown,
            "editor within the viewport"
        );
    }

    #[test]
    fn typing_grows_the_editor_and_keeps_it_on_screen() {
        let (_d, mut app) = app_with_comments();
        handle_key(&mut app, key('M'));
        let rows = |app: &App| {
            app.detail_dlines()
                .iter()
                .filter(|l| l.kind == detail::Kind::Editor)
                .count()
        };
        let before = rows(&app);
        for _ in 0..5 {
            handle_key(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        }
        assert!(rows(&app) > before);
        assert!(draw(&app, 90, 30).contains("now ─"));
    }
}

/// Memoized rows and detail lines (yaks-bc30): `ROWS_CALLS` / `DLINES_CALLS`
/// count the actual rebuilds, and every cache hit is checked against a fresh
/// build under `cfg(test)`, so a stale cache cannot pass silently.
mod memo {
    use super::*;
    use crate::tui::{memo::DLINES_CALLS, viewmodel::ROWS_CALLS};

    fn counts() -> (usize, usize) {
        (ROWS_CALLS.with(|c| c.get()), DLINES_CALLS.with(|c| c.get()))
    }

    /// Rebuilds `(rows, detail lines)` done while running `f`.
    fn builds(f: impl FnOnce()) -> (usize, usize) {
        let (r, d) = counts();
        f();
        let (r2, d2) = counts();
        (r2 - r, d2 - d)
    }

    /// A farm-ish app: `n` hairy yaks, the first with a long code block, the
    /// detail pane focused and measured.
    fn app_in_detail() -> App {
        let mut tasks: Vec<Task> = (0..30)
            .map(|i| {
                task(
                    &format!("t{i:02}"),
                    &format!("yak {i}"),
                    Status::Hairy,
                    3,
                    None,
                )
            })
            .collect();
        let code: Vec<String> = (0..30).map(|i| format!("let x{i} = {i};")).collect();
        tasks[0].body = format!("Intro.\n\n```rust\n{}\n```\n\nTail.", code.join("\n"));
        let mut app = App::new(tasks);
        let _ = draw(&app, 100, 40);
        enter_key(&mut app);
        let _ = draw(&app, 100, 40);
        app
    }

    #[test]
    fn a_detail_keypress_rebuilds_the_rows_and_lines_at_most_once() {
        let mut app = app_in_detail();
        for k in ['j', 'j', 'k', 'l', 'h', 'l'] {
            let b = builds(|| {
                handle_key(&mut app, key(k));
                let _ = draw(&app, 100, 40);
            });
            assert!(b.0 <= 1 && b.1 <= 1, "{k:?} rebuilt rows/lines {b:?} times");
        }
    }

    #[test]
    fn a_plain_redraw_and_a_cursor_move_inside_the_pane_rebuild_nothing() {
        let mut app = app_in_detail();
        let _ = draw(&app, 100, 40); // warm
        let b = builds(|| {
            let _ = draw(&app, 100, 40);
            let _ = draw(&app, 100, 40);
            handle_key(&mut app, key('j')); // same selection: no change of input
            let _ = draw(&app, 100, 40);
        });
        assert_eq!(b, (0, 0));
    }

    #[test]
    fn changing_the_selection_rebuilds_the_lines_once_and_not_the_rows() {
        let mut app = App::new(
            (0..5)
                .map(|i| task(&format!("t{i}"), "yak", Status::Hairy, 3, None))
                .collect(),
        );
        let _ = app.detail_dlines();
        let b = builds(|| {
            handle_key(&mut app, key('j'));
            let _ = draw(&app, 100, 20);
            let _ = app.detail_dlines();
            let _ = app.detail_dlines();
        });
        assert_eq!(b, (0, 1));
    }

    #[test]
    fn every_input_change_shows_up_in_the_next_read() {
        // Each step changes one input; the next reads must equal a fresh
        // build (the hit path asserts that) and reflect the change.
        let mut app = App::new(vec![
            task("a0", "Root", Status::Hairy, 2, None),
            task("a1", "Child", Status::Hairy, 2, Some("a0")),
            task("b0", "Other", Status::Hairy, 1, None),
        ]);
        let _ = draw(&app, 80, 20);
        assert_eq!(app.row_count(), 3);
        app.collapsed.insert("a0".into());
        assert_eq!(app.row_count(), 2);
        app.collapsed.clear();
        assert_eq!(app.row_count(), 3);
        app.filter.search = Some("other".into());
        assert_eq!(app.row_count(), 1, "only the match is left");
        assert_eq!(app.selected_id().as_deref(), Some("b0"));
        app.filter.types = vec!["nope".into()];
        assert_eq!(app.row_count(), 0);
        app.filter = FilterSpec::default();
        app.family_scope
            .insert("status:hairy".into(), view::FamilyScope::Lone);
        let _ = app.rows();
        app.family_scope.clear();
        app.set_view(1); // Shaving
        assert_eq!(app.row_count(), 0);
        app.set_view(0);
        // Farm data replaced (with `touch`): the new title is shown.
        let sel = app.selected_id().unwrap();
        let at = app.all.iter().position(|t| t.id == sel).unwrap();
        app.all[at].title = "Renamed".into();
        app.touch();
        assert_eq!(app.selected().unwrap().title, "Renamed");
        // Detail lines follow the width, the selection and the data.
        let w0 = app.detail_dlines().len();
        app.all[at].body = "one\n\ntwo\n\nthree".into();
        app.touch();
        assert!(app.detail_dlines().len() > w0);
        app.detail_width.set(20);
        let narrow = app.detail_dlines().len();
        app.detail_width.set(120);
        assert!(app.detail_dlines().len() <= narrow);
    }

    #[test]
    fn the_clock_minute_invalidates_relative_dates() {
        let mut t = task("a0", "Root", Status::Hairy, 2, None);
        t.updated = Some("2026-09-10T11:55:00Z".into());
        let mut app = App::new(vec![t]);
        let _ = draw(&app, 80, 20);
        let text = |app: &App| {
            app.detail_dlines()
                .iter()
                .map(|l| l.text.clone())
                .collect::<Vec<_>>()
                .join("\n")
        };
        assert!(text(&app).contains("5m ago"));
        let later = chrono::DateTime::parse_from_rfc3339("2026-09-10T12:10:00Z")
            .unwrap()
            .to_utc();
        app.clock = when::Clock::Fixed(later, chrono::FixedOffset::east_opt(0).unwrap());
        assert!(text(&app).contains("15m ago"));
    }

    #[test]
    #[should_panic(expected = "stale rows cache")]
    fn a_missed_touch_is_caught_by_the_test_assertion() {
        let mut app = App::new(vec![task("a0", "Root", Status::Hairy, 2, None)]);
        let _ = app.rows(); // fill the cache
        app.all.push(task("a1", "New", Status::Hairy, 2, None)); // no touch()
        let _ = app.rows();
    }

    // -- farm git-state chip + W/U keys (yaks-ce5d) --------------------------

    fn with_chip(dirty: usize, sync: Option<(Option<&str>, usize, usize)>) -> App {
        let mut app = sample();
        app.farm_sync = Some(FarmSync {
            dirty,
            sync: sync.map(|(upstream, ahead, behind)| crate::status::SyncState {
                branch: "yaks".into(),
                upstream: upstream.map(String::from),
                ahead,
                behind,
            }),
        });
        app
    }

    #[test]
    fn chip_is_absent_when_the_farm_is_clean_and_up_to_date() {
        let app = with_chip(0, Some((Some("origin/yaks"), 0, 0)));
        insta::assert_snapshot!(draw(&app, 140, 8));
    }

    #[test]
    fn chip_shows_the_uncommitted_count() {
        insta::assert_snapshot!(draw(&with_chip(3, None), 140, 8));
    }

    #[test]
    fn chip_shows_ahead_and_behind() {
        let app = with_chip(2, Some((Some("origin/yaks"), 2, 1)));
        insta::assert_snapshot!(draw(&app, 140, 8));
    }

    #[test]
    fn chip_says_not_published_without_an_upstream() {
        insta::assert_snapshot!(draw(&with_chip(0, Some((None, 0, 0))), 140, 8));
    }

    #[test]
    fn chip_sits_right_of_a_notification() {
        let mut app = with_chip(1, None);
        app.notification = Some("syncing…".into());
        insta::assert_snapshot!(draw(&app, 140, 8));
    }

    #[test]
    fn chip_goes_compact_when_the_tabs_leave_little_room() {
        // 120 columns: `● 1 uncommitted not published` does not fit, `●1 unpub` does.
        insta::assert_snapshot!(draw(&with_chip(1, Some((None, 0, 0))), 120, 8));
    }

    #[test]
    fn chip_yields_when_the_tabs_leave_no_room() {
        insta::assert_snapshot!(draw(&with_chip(3, None), 30, 6));
    }

    #[test]
    fn only_behind_is_a_warning() {
        let segs = FarmSync {
            dirty: 1,
            sync: Some(crate::status::SyncState {
                branch: "yaks".into(),
                upstream: Some("origin/yaks".into()),
                ahead: 2,
                behind: 1,
            }),
        }
        .segments(false);
        assert_eq!(
            segs,
            vec![
                ("● 1 uncommitted".to_string(), false),
                ("↑2".to_string(), false),
                ("↓1".to_string(), true),
            ]
        );
    }

    #[test]
    fn help_lists_the_commit_and_sync_keys() {
        let text: String = help_content()
            .iter()
            .map(|l| {
                l.spans
                    .iter()
                    .map(|s| s.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(text.contains("Commit the farm"), "{text}");
        assert!(text.contains("Sync the farm"), "{text}");
    }

    #[test]
    fn help_overlay_shows_the_farm_keys() {
        let mut app = sample();
        app.overlay = Overlay::Help(54);
        insta::assert_snapshot!(draw(&app, 100, 20));
    }

    fn git_in(dir: &std::path::Path, args: &[&str]) {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_INDEX_FILE")
            .output()
            .unwrap();
        assert!(out.status.success(), "git {args:?}: {out:?}");
    }

    #[test]
    fn w_commits_a_dirty_team_farm_and_the_chip_goes() {
        let (proj, farm) = temp_farm(&[task("t0", "solo", Status::Hairy, 3, None)]);
        git_in(&proj, &["init", "-q"]);
        git_in(&proj, &["config", "user.name", "t"]);
        git_in(&proj, &["config", "user.email", "t@t"]);
        git_in(&proj, &["add", "."]);
        git_in(&proj, &["commit", "-q", "-m", "init"]);
        let mut app = App::with_farm(farm).unwrap();
        assert_eq!(app.farm_sync.as_ref().map(|f| f.dirty), Some(0));
        assert!(!draw(&app, 140, 8).contains("uncommitted"));

        // A TUI mutation dirties the farm; the chip appears via the reload.
        press(&mut app, "Sn");
        enter_key(&mut app);
        // The yak's file, plus `.yaks/.gitignore` that the first write creates.
        assert_eq!(app.farm_sync.as_ref().map(|f| f.dirty), Some(2));
        assert!(draw(&app, 140, 8).contains("● 2 uncommitted"));

        // W queues the commit and says so; nothing is committed until it runs.
        handle_key(&mut app, key('W'));
        assert_eq!(app.notification.as_deref(), Some("committing…"));
        assert_eq!(app.farm_sync.as_ref().map(|f| f.dirty), Some(2));
        app.run_pending_farm_op();
        let note = app.notification.clone().unwrap();
        assert!(note.starts_with("committed "), "{note}");
        assert!(!note.contains('\n'));
        assert_eq!(app.farm_sync.as_ref().map(|f| f.dirty), Some(0));
        assert!(!draw(&app, 140, 8).contains("uncommitted"));
        let log = std::process::Command::new("git")
            .arg("-C")
            .arg(&proj)
            .args(["log", "--oneline"])
            .output()
            .unwrap();
        assert_eq!(String::from_utf8_lossy(&log.stdout).lines().count(), 2);
    }

    #[test]
    fn w_on_a_clean_farm_says_so_and_u_on_a_non_branch_farm_errors_in_one_line() {
        let (proj, farm) = temp_farm(&[task("t0", "solo", Status::Hairy, 3, None)]);
        git_in(&proj, &["init", "-q"]);
        git_in(&proj, &["add", "."]);
        git_in(
            &proj,
            &[
                "-c",
                "user.name=t",
                "-c",
                "user.email=t@t",
                "commit",
                "-q",
                "-m",
                "i",
            ],
        );
        let mut app = App::with_farm(farm).unwrap();
        handle_key(&mut app, key('W'));
        app.run_pending_farm_op();
        assert_eq!(app.notification.as_deref(), Some("nothing to commit"));
        handle_key(&mut app, key('U'));
        assert_eq!(app.notification.as_deref(), Some("syncing…"));
        app.run_pending_farm_op();
        let note = app.notification.clone().unwrap();
        assert!(note.starts_with("error: "), "{note}");
        assert!(!note.contains('\n'));
    }

    #[test]
    fn w_and_u_do_nothing_without_a_farm() {
        let mut app = sample();
        handle_key(&mut app, key('W'));
        handle_key(&mut app, key('U'));
        assert!(app.notification.is_none() && app.pending_farm_op.is_none());
    }
}

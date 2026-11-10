//! Drafts of unfinished edits (yaks-683f): what you typed into the create
//! form, the edit form or a new comment survives dismissing it.
//!
//! A draft is one JSON file per surface and yak, `drafts/<key>.json` under the
//! farm's per-user config dir (beside `views.json`, never under `.yaks/`: a
//! draft is the user's intent, not the farm's data). Keys are `edit:<id>`,
//! `comment:<id>` and `create:<parent|root>`; a newer draft replaces the older.
//!
//! When a draft is saved: the dirty-cancel prompt's `keep` (the default), and
//! the app exiting with an editor open ([`App::flush_open_draft`]). Phase 1:
//! no autosave, so a killed terminal still loses the text. When it goes: a
//! successful commit, `discard`, `:q!`, a clean cancel, and the startup sweep
//! ([`Store::sweep`]). Reads never panic; writes are best-effort, like
//! `views_store`.

use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Duration, SecondsFormat, Utc};
use serde_json::{Value, json};

use super::*;

const VERSION: u64 = 1;
/// A draft older than this is dropped by the startup sweep.
const MAX_AGE_DAYS: i64 = 30;

/// The prompt that replaces 275f's `Discard changes? (y/N)` for a surface that
/// can keep a draft.
pub(crate) const KEEP_PROMPT: &str =
    "Unsaved changes: k keep draft (Enter) · d discard · Esc back to editing ";

/// Which edit a draft belongs to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DraftKey {
    Edit(String),
    Comment(String),
    /// A new yak under this parent (`None`: at the root).
    Create(Option<String>),
}

impl DraftKey {
    /// The file name: the key with anything but `[A-Za-z0-9_-]` made safe.
    fn file_name(&self) -> String {
        let safe = |s: &str| {
            s.chars()
                .map(|c| {
                    if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                        c
                    } else {
                        '_'
                    }
                })
                .collect::<String>()
        };
        match self {
            DraftKey::Edit(id) => format!("edit-{}.json", safe(id)),
            DraftKey::Comment(id) => format!("comment-{}.json", safe(id)),
            DraftKey::Create(parent) => {
                format!(
                    "create-{}.json",
                    parent.as_deref().map_or("root".into(), safe)
                )
            }
        }
    }

    fn id(&self) -> Option<&str> {
        match self {
            DraftKey::Edit(id) | DraftKey::Comment(id) => Some(id),
            DraftKey::Create(_) => None,
        }
    }
}

/// The create/edit form's fields as typed (the chips by value, not index).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FormFields {
    pub(crate) title: String,
    pub(crate) tags: String,
    pub(crate) source: String,
    pub(crate) due: String,
    pub(crate) every: String,
    pub(crate) wake: String,
    pub(crate) kind: String,
    pub(crate) priority: u8,
    pub(crate) herd: Option<String>,
    pub(crate) description: String,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Body {
    Form(FormFields),
    Comment(String),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Draft {
    pub(crate) key: DraftKey,
    /// RFC 3339, when it was saved.
    pub(crate) saved_at: String,
    /// The yak's `updated` when the edit began (an edit form only): what says
    /// the yak has changed since.
    pub(crate) base_updated: Option<String>,
    pub(crate) body: Body,
}

impl Draft {
    fn to_json(&self) -> Value {
        let mut v = json!({
            "version": VERSION,
            "saved_at": self.saved_at,
            "base_updated": self.base_updated,
        });
        match &self.key {
            DraftKey::Edit(id) => {
                v["surface"] = json!("edit");
                v["id"] = json!(id);
            }
            DraftKey::Comment(id) => {
                v["surface"] = json!("comment");
                v["id"] = json!(id);
            }
            DraftKey::Create(parent) => {
                v["surface"] = json!("create");
                v["parent"] = json!(parent);
            }
        }
        match &self.body {
            Body::Comment(text) => v["text"] = json!(text),
            Body::Form(f) => {
                v["fields"] = json!({
                    "title": f.title, "tags": f.tags, "source": f.source,
                    "due": f.due, "every": f.every, "wake": f.wake,
                    "kind": f.kind, "priority": f.priority, "herd": f.herd,
                    "description": f.description,
                })
            }
        }
        v
    }

    fn from_json(v: &Value) -> Option<Draft> {
        if v.get("version")?.as_u64()? != VERSION {
            return None;
        }
        let s = |v: &Value, k: &str| v.get(k).and_then(Value::as_str).map(String::from);
        let key = match v.get("surface")?.as_str()? {
            "edit" => DraftKey::Edit(s(v, "id")?),
            "comment" => DraftKey::Comment(s(v, "id")?),
            "create" => DraftKey::Create(s(v, "parent")),
            _ => return None,
        };
        let body = match &key {
            DraftKey::Comment(_) => Body::Comment(s(v, "text")?),
            _ => {
                let f = v.get("fields")?;
                Body::Form(FormFields {
                    title: s(f, "title")?,
                    tags: s(f, "tags")?,
                    source: s(f, "source")?,
                    due: s(f, "due")?,
                    every: s(f, "every")?,
                    wake: s(f, "wake")?,
                    kind: s(f, "kind")?,
                    priority: f.get("priority")?.as_u64()? as u8,
                    herd: s(f, "herd"),
                    description: s(f, "description")?,
                })
            }
        };
        Some(Draft {
            key,
            saved_at: s(v, "saved_at")?,
            base_updated: s(v, "base_updated"),
            body,
        })
    }
}

/// The drafts directory of one farm.
pub(crate) struct Store {
    dir: PathBuf,
}

impl Store {
    /// The farm at `root`'s drafts. Under `cargo test` it is a per-process
    /// directory in the OS temp dir, so no test touches the real config.
    pub(crate) fn for_root(root: &Path) -> Store {
        #[cfg(not(test))]
        let base = views_store::config_dir(root);
        #[cfg(test)]
        let base = std::env::temp_dir()
            .join(format!("yaks-test-config-{}", std::process::id()))
            .join(
                views_store::config_dir(root)
                    .file_name()
                    .unwrap_or_default(),
            );
        Store {
            dir: base.join("drafts"),
        }
    }

    #[cfg(test)]
    pub(crate) fn at(dir: PathBuf) -> Store {
        Store { dir }
    }

    fn path(&self, key: &DraftKey) -> PathBuf {
        self.dir.join(key.file_name())
    }

    /// Write `d`, replacing any draft of its key: to a temp file, then renamed
    /// into place, so a crash leaves the old draft or the new, never half.
    pub(crate) fn save(&self, d: &Draft) -> bool {
        let path = self.path(&d.key);
        let tmp = path.with_extension("json.tmp");
        let text = serde_json::to_string_pretty(&d.to_json()).unwrap_or_default();
        fs::create_dir_all(&self.dir).is_ok()
            && fs::write(&tmp, text).is_ok()
            && fs::rename(&tmp, &path).is_ok()
    }

    pub(crate) fn load(&self, key: &DraftKey) -> Option<Draft> {
        let text = fs::read_to_string(self.path(key)).ok()?;
        Draft::from_json(&serde_json::from_str(&text).ok()?)
    }

    pub(crate) fn delete(&self, key: &DraftKey) {
        let _ = fs::remove_file(self.path(key));
    }

    /// The startup sweep: drop drafts older than 30 days, and those of a yak
    /// that is gone or dead (`alive(id)` says which ids are neither).
    pub(crate) fn sweep(&self, now: DateTime<Utc>, alive: impl Fn(&str) -> bool) {
        let Ok(rd) = fs::read_dir(&self.dir) else {
            return;
        };
        for entry in rd.flatten() {
            let path = entry.path();
            if path.extension().is_none_or(|e| e != "json") {
                continue;
            }
            let Some(d) = fs::read_to_string(&path)
                .ok()
                .and_then(|t| serde_json::from_str(&t).ok())
                .and_then(|v| Draft::from_json(&v))
            else {
                continue;
            };
            let old = DateTime::parse_from_rfc3339(&d.saved_at)
                .is_ok_and(|t| now.signed_duration_since(t) > Duration::days(MAX_AGE_DAYS));
            if old || d.key.id().is_some_and(|id| !alive(id)) {
                let _ = fs::remove_file(path);
            }
        }
    }
}

/// How a form restored from a draft says so in its header.
#[derive(Clone, Debug)]
pub(crate) struct DraftBadge {
    /// `saved 2h ago`.
    pub(crate) saved: String,
    /// The yak changed after the draft began: the badge warns.
    pub(crate) stale: bool,
}

impl CreateForm {
    /// The key a draft of this form is kept under.
    pub(crate) fn draft_key(&self) -> DraftKey {
        match &self.edit_id {
            Some(id) => DraftKey::Edit(id.clone()),
            None => DraftKey::Create(self.parent.clone()),
        }
    }

    pub(crate) fn fields(&self) -> FormFields {
        let text = |c: &RefCell<EditorState>| c.borrow().lines.to_string();
        FormFields {
            title: text(&self.title),
            tags: text(&self.tags),
            source: text(&self.source),
            due: text(&self.due),
            every: text(&self.every),
            wake: WAKE_CHOICES[self.wake_idx].to_string(),
            kind: TYPE_CHOICES[self.kind_idx].to_string(),
            priority: PRI_CHOICES[self.pri_idx],
            herd: self.target_herd(),
            description: text(&self.description),
        }
    }

    /// Put a draft's fields into this (freshly seeded) form.
    pub(crate) fn apply_fields(&mut self, f: &FormFields, vim: bool) {
        self.title = text_field(&f.title, vim);
        self.tags = text_field(&f.tags, vim);
        self.source = text_field(&f.source, vim);
        self.due = text_field(&f.due, vim);
        self.every = text_field(&f.every, vim);
        self.description = multiline_field(&f.description, vim);
        self.wake_idx = WAKE_CHOICES.iter().position(|w| *w == f.wake).unwrap_or(0);
        self.kind_idx = kind_index(&f.kind);
        self.pri_idx = pri_index(f.priority);
        if let Some(i) = f
            .herd
            .as_ref()
            .and_then(|h| self.herds.iter().position(|c| c == h))
        {
            self.herd_idx = i;
        }
    }
}

impl App {
    /// The drafts of the open farm; `None` without one (the read-only
    /// constructor keeps nothing).
    fn draft_store(&self) -> Option<Store> {
        self.farm.as_ref().map(|h| Store::for_root(h.root()))
    }

    /// The key and body a draft of overlay `ov` would have, if it is a surface
    /// that keeps drafts (the forms and a new comment) and the farm is open.
    pub(crate) fn draft_of(&self, ov: &Overlay) -> Option<Draft> {
        self.farm.as_ref()?;
        let saved_at = self.clock.now().to_rfc3339_opts(SecondsFormat::Secs, true);
        match ov {
            Overlay::Create(f) => Some(Draft {
                key: f.draft_key(),
                saved_at,
                base_updated: f.base_updated.clone(),
                body: Body::Form(f.fields()),
            }),
            Overlay::BlockEdit(be) => match &be.editor.action {
                EditAction::Comment(id) => Some(Draft {
                    key: DraftKey::Comment(id.clone()),
                    saved_at,
                    base_updated: None,
                    body: Body::Comment(be.editor.text()),
                }),
                _ => None,
            },
            _ => None,
        }
    }

    /// Whether dismissing `ov` should offer to keep a draft.
    pub(crate) fn keeps_drafts(&self, ov: &Overlay) -> bool {
        self.draft_of(ov).is_some()
    }

    /// Save `ov` as a draft. Returns whether it was written.
    pub(crate) fn save_draft_of(&self, ov: &Overlay) -> bool {
        match (self.draft_of(ov), self.draft_store()) {
            (Some(d), Some(store)) => store.save(&d),
            _ => false,
        }
    }

    /// Delete the draft `ov` would have (a discard, or a commit).
    pub(crate) fn delete_draft_of(&self, ov: &Overlay) {
        if let (Some(d), Some(store)) = (self.draft_of(ov), self.draft_store()) {
            store.delete(&d.key);
        }
    }

    pub(crate) fn delete_draft(&self, key: &DraftKey) {
        if let Some(store) = self.draft_store() {
            store.delete(key);
        }
    }

    pub(crate) fn save_draft(&self, d: &Draft) {
        if let Some(store) = self.draft_store() {
            store.save(d);
        }
    }

    /// Save a comment's text as its yak's comment draft (a save that failed).
    pub(crate) fn save_comment_draft(&self, id: &str, text: &str) {
        self.save_draft(&Draft {
            key: DraftKey::Comment(id.to_string()),
            saved_at: self.clock.now().to_rfc3339_opts(SecondsFormat::Secs, true),
            base_updated: None,
            body: Body::Comment(text.to_string()),
        });
    }

    /// Opening a form: put its saved draft, if any, into it, with a badge, and
    /// say how to get the original back.
    pub(crate) fn restore_form_draft(&mut self, form: &mut CreateForm) {
        let Some(d) = self.draft_store().and_then(|s| s.load(&form.draft_key())) else {
            return;
        };
        let Body::Form(fields) = &d.body else { return };
        form.apply_fields(fields, self.editor_vim());
        form.base_updated = d.base_updated.clone();
        let stale = form
            .edit_id
            .as_deref()
            .is_some_and(|id| self.task(id).is_some_and(|t| t.updated != d.base_updated));
        form.draft = Some(DraftBadge {
            saved: format!("saved {}", self.clock.format(&d.saved_at)),
            stale,
        });
        self.notification = Some("restored draft — Ctrl-R for original".into());
    }

    /// Opening a new-comment editor on `id`: its saved draft, if any, as
    /// `(text, badge)`.
    pub(crate) fn comment_draft(&self, id: &str) -> Option<(String, String)> {
        let d = self
            .draft_store()?
            .load(&DraftKey::Comment(id.to_string()))?;
        let Body::Comment(text) = d.body else {
            return None;
        };
        Some((
            text,
            format!("draft · saved {}", self.clock.format(&d.saved_at)),
        ))
    }

    /// `Ctrl-R` in a form with a restored draft: delete the draft and reopen
    /// the form from the yak as it is now (or a blank create), on the same row.
    pub(crate) fn revert_form_draft(&mut self) {
        let Overlay::Create(f) = &self.overlay else {
            return;
        };
        self.delete_draft(&f.draft_key());
        let row = f.row;
        let fresh = match f.edit_id.clone() {
            Some(id) => self.edit_form_for(&id),
            None => Some(self.create_form_for(f.parent.clone())),
        };
        let Some(mut fresh) = fresh else { return };
        fresh.row = row.min(fresh.row_count() - 1);
        self.overlay = Overlay::Create(fresh);
        self.notification = Some("draft reverted".into());
    }

    /// `Ctrl-R` in a new-comment editor with a restored draft: delete it and
    /// empty the editor.
    pub(crate) fn revert_comment_draft(&mut self) {
        let Overlay::BlockEdit(be) = &mut self.overlay else {
            return;
        };
        let EditAction::Comment(id) = &be.editor.action else {
            return;
        };
        let id = id.clone();
        be.editor = Editor::new(
            be.editor.vim,
            false,
            String::new(),
            "",
            EditAction::Comment(id.clone()),
        );
        be.draft = None;
        self.delete_draft(&DraftKey::Comment(id));
        self.notification = Some("draft reverted".into());
    }

    /// Whether `k` is the revert key and the open form or comment has a draft
    /// to revert. (It shadows vim's redo only while a draft is restored.)
    pub(crate) fn is_draft_revert(&self, k: &KeyEvent) -> bool {
        let ctrl_r = k.modifiers.contains(KeyModifiers::CONTROL) && k.code == KeyCode::Char('r');
        ctrl_r
            && match &self.overlay {
                Overlay::Create(f) => f.draft.is_some(),
                Overlay::BlockEdit(be) => be.draft.is_some(),
                _ => false,
            }
    }

    /// A key at the keep/discard prompt for the editor stashed in
    /// `dirty_cancel`.
    pub(crate) fn keep_draft_key(&mut self, prompt: String, code: KeyCode) {
        match code {
            KeyCode::Enter | KeyCode::Char('k') | KeyCode::Char('K') => {
                if let Some(ov) = self.dirty_cancel.take() {
                    let saved = self.save_draft_of(&ov);
                    self.drop_editor(ov);
                    self.notification = Some(if saved {
                        "draft saved".into()
                    } else {
                        "draft could not be saved; changes lost".into()
                    });
                }
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                if let Some(ov) = self.dirty_cancel.take() {
                    self.delete_draft_of(&ov);
                    self.drop_editor(ov);
                }
                self.notification = Some("changes discarded".into());
            }
            KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                if let Some(ov) = self.dirty_cancel.take() {
                    self.overlay = ov;
                }
            }
            _ => {
                self.overlay = Overlay::Confirm {
                    prompt,
                    action: ConfirmAction::KeepDraft,
                }
            }
        }
    }

    /// The app is exiting: keep what an open (or stashed) editor holds, so
    /// quitting does not lose it.
    pub(crate) fn flush_open_draft(&self) {
        let ov = match (&self.overlay, &self.dirty_cancel) {
            (Overlay::Confirm { .. }, Some(stashed)) => stashed,
            (open, _) => open,
        };
        let dirty = match ov {
            Overlay::Create(f) => self.form_is_dirty(f),
            Overlay::BlockEdit(be) => be.is_dirty(),
            _ => false,
        };
        if dirty {
            self.save_draft_of(ov);
        }
    }

    /// The startup sweep of this farm's drafts.
    pub(crate) fn sweep_drafts(&self) {
        if let Some(store) = self.draft_store() {
            store.sweep(self.clock.now(), |id| {
                self.task(id).is_some_and(|t| t.status != Status::Dead)
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp() -> PathBuf {
        let d = std::env::temp_dir().join(format!(
            "yaks-drafts-unit-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = fs::remove_dir_all(&d);
        d
    }

    fn form_draft(key: DraftKey, saved_at: &str) -> Draft {
        Draft {
            key,
            saved_at: saved_at.into(),
            base_updated: Some("2026-01-01T00:00:00Z".into()),
            body: Body::Form(FormFields {
                title: "t".into(),
                tags: "a, b".into(),
                source: String::new(),
                due: "+1d".into(),
                every: String::new(),
                wake: "agent".into(),
                kind: "bug".into(),
                priority: 2,
                herd: Some("api".into()),
                description: "line 1\n\nline 2".into(),
            }),
        }
    }

    #[test]
    fn a_draft_round_trips_through_its_file() {
        let store = Store::at(temp());
        let d = form_draft(DraftKey::Edit("yak-1".into()), "2026-09-10T12:00:00Z");
        assert!(store.save(&d));
        assert_eq!(store.load(&d.key), Some(d.clone()));
        let c = Draft {
            key: DraftKey::Comment("yak-1".into()),
            saved_at: "2026-09-10T12:00:00Z".into(),
            base_updated: None,
            body: Body::Comment("a note\n".into()),
        };
        assert!(store.save(&c));
        assert_eq!(store.load(&c.key), Some(c));
        store.delete(&d.key);
        assert_eq!(store.load(&d.key), None);
    }

    #[test]
    fn keys_do_not_share_a_file() {
        let names: Vec<String> = [
            DraftKey::Edit("a-1".into()),
            DraftKey::Comment("a-1".into()),
            DraftKey::Create(None),
            DraftKey::Create(Some("a-1".into())),
        ]
        .iter()
        .map(DraftKey::file_name)
        .collect();
        let mut sorted = names.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), names.len(), "{names:?}");
        assert_eq!(DraftKey::Edit("../x".into()).file_name(), "edit-___x.json");
    }

    #[test]
    fn an_unreadable_or_foreign_version_file_is_no_draft() {
        let dir = temp();
        let store = Store::at(dir.clone());
        let key = DraftKey::Edit("yak-1".into());
        fs::create_dir_all(&dir).unwrap();
        fs::write(store.path(&key), "{not json").unwrap();
        assert_eq!(store.load(&key), None);
        fs::write(store.path(&key), r#"{"version": 99}"#).unwrap();
        assert_eq!(store.load(&key), None);
    }

    #[test]
    fn the_sweep_drops_old_drafts_and_those_of_gone_yaks() {
        let store = Store::at(temp());
        let now = DateTime::parse_from_rfc3339("2026-09-10T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let fresh = form_draft(DraftKey::Edit("yak-1".into()), "2026-09-09T12:00:00Z");
        let old = form_draft(DraftKey::Edit("yak-2".into()), "2026-07-01T12:00:00Z");
        let gone = form_draft(DraftKey::Edit("yak-3".into()), "2026-09-09T12:00:00Z");
        let create = form_draft(DraftKey::Create(None), "2026-09-09T12:00:00Z");
        for d in [&fresh, &old, &gone, &create] {
            store.save(d);
        }
        store.sweep(now, |id| id != "yak-3");
        assert!(store.load(&fresh.key).is_some());
        assert!(
            store.load(&create.key).is_some(),
            "a create has no yak to lose"
        );
        assert!(store.load(&old.key).is_none(), "older than 30 days");
        assert!(store.load(&gone.key).is_none(), "its yak is gone");
    }
}

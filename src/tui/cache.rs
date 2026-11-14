//! Per-user, per-farm UI-state cache. This is *rebuildable* state (the set of
//! collapsed tree ids, plus per-view family-scope overrides), so it lives under
//! `$XDG_CACHE_HOME` (default `~/.cache`) at `yaks/<slug>.json`, never in the
//! farm and never committed.
//!
//! The slug is a stable hash of the absolute farm root. A slug change merely
//! resets the cache (collapsed rows re-expand, family overrides revert to auto),
//! which is harmless for rebuildable state.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::tui::view::FamilyScope;

/// The full rebuildable UI state persisted per farm. Fields are independent;
/// both are written together so neither clobbers the other.
#[derive(Default, Clone, PartialEq, Debug)]
pub struct UiState {
    /// Ids of collapsed tree parents.
    pub collapsed: HashSet<String>,
    /// Per-view family-scope overrides, keyed by `View::key`. A missing entry
    /// means the view inherits [`FamilyScope::DEFAULT`] ("auto").
    pub family: HashMap<String, FamilyScope>,
}

/// The farm root the per-user state was keyed by before the farm moved to the
/// branch root (yaks-6ffd): `<root>/.yaks`. `None` when `root` is itself a
/// `.yaks/` directory, whose key never changed.
pub(super) fn legacy_root(root: &Path) -> Option<PathBuf> {
    if root.file_name().is_some_and(|n| n == ".yaks") {
        return None;
    }
    Some(
        fs::canonicalize(root)
            .unwrap_or_else(|_| root.to_path_buf())
            .join(".yaks"),
    )
}

/// FNV-1a 64-bit over the absolute root path → 12 hex chars.
pub(super) fn slug(root: &Path) -> String {
    let abs = fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in abs.to_string_lossy().bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{h:016x}")[..12].to_string()
}

fn cache_home() -> PathBuf {
    if let Ok(x) = std::env::var("XDG_CACHE_HOME") {
        if !x.is_empty() {
            return PathBuf::from(x);
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".cache");
    }
    std::env::temp_dir()
}

/// Bumped whenever the svg rasterizer's output changes for the same input (a
/// toque bump that widens the embedded font subset, say), so a cached PNG from
/// the old renderer is not served.
const SVG_RENDER_GEN: u32 = 2; // 2: yaks-4f4d widened the fonts

/// Where the PNG of svg `bytes`, rasterized at its own size, is cached under
/// `base`: `<base>/yaks/svg/<gen>-<hash of content>.png`. Content-addressed, so
/// an edited svg is a new file and the old one is simply never read again.
pub fn svg_png_path_in(base: &Path, bytes: &[u8]) -> PathBuf {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    base.join("yaks")
        .join("svg")
        .join(format!("{SVG_RENDER_GEN}-{h:016x}.png"))
}

/// The user's cache directory for rebuildable files, or `None` under test, so
/// a test never reads or writes the real cache.
pub fn svg_cache_base() -> Option<PathBuf> {
    if cfg!(test) {
        return None;
    }
    Some(cache_home())
}

/// Absolute path to this farm's UI-state cache file.
pub fn ui_state_path(root: &Path) -> PathBuf {
    cache_home()
        .join("yaks")
        .join(format!("{}.json", slug(root)))
}

pub fn load(root: &Path) -> UiState {
    load_from(&ui_state_path(root))
}

/// Copy the state kept under the pre-move key (`<root>/.yaks`) to this root's
/// key, when this root has none yet. Never overwrites, never deletes: the
/// old file stays for an older yaks still reading it.
pub fn adopt_legacy(root: &Path) {
    if let Some(old) = legacy_root(root) {
        adopt_file(&ui_state_path(&old), &ui_state_path(root));
    }
}

fn adopt_file(old: &Path, new: &Path) {
    if old == new || new.exists() || !old.is_file() {
        return;
    }
    if let Some(dir) = new.parent() {
        let _ = fs::create_dir_all(dir);
    }
    let _ = fs::copy(old, new);
}

pub fn save(root: &Path, state: &UiState) {
    save_to(&ui_state_path(root), state);
}

fn load_from(path: &Path) -> UiState {
    let Ok(text) = fs::read_to_string(path) else {
        return UiState::default();
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else {
        return UiState::default();
    };
    let collapsed = v
        .get("collapsed")
        .and_then(|c| c.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let family = v
        .get("family")
        .and_then(|h| h.as_object())
        .map(|obj| {
            obj.iter()
                .filter_map(|(k, val)| {
                    val.as_str()
                        .and_then(FamilyScope::parse)
                        .map(|s| (k.clone(), s))
                })
                .collect()
        })
        .unwrap_or_default();
    UiState { collapsed, family }
}

fn save_to(path: &Path, state: &UiState) {
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    let mut ids: Vec<&String> = state.collapsed.iter().collect();
    ids.sort();
    let mut keys: Vec<&String> = state.family.keys().collect();
    keys.sort();
    let mut family = serde_json::Map::new();
    for k in keys {
        family.insert(k.clone(), serde_json::Value::from(state.family[k].as_str()));
    }
    let payload = serde_json::json!({ "collapsed": ids, "family": family });
    let _ = fs::write(path, payload.to_string());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static SEQ: AtomicU64 = AtomicU64::new(0);

    fn temp_file() -> PathBuf {
        let mut p = std::env::temp_dir();
        let n = SEQ.fetch_add(1, Ordering::Relaxed);
        p.push(format!("yaksrs-cache-{}-{}.json", std::process::id(), n));
        p
    }

    #[test]
    fn round_trip_collapsed() {
        let path = temp_file();
        let mut st = UiState::default();
        st.collapsed.insert("yak-0001".to_string());
        st.collapsed.insert("yak-0002".to_string());
        save_to(&path, &st);
        assert_eq!(load_from(&path), st);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn round_trip_family_overrides() {
        let path = temp_file();
        let mut st = UiState::default();
        st.family
            .insert("status:hairy".to_string(), FamilyScope::All);
        st.family
            .insert("status:shaving".to_string(), FamilyScope::Lone);
        save_to(&path, &st);
        assert_eq!(load_from(&path).family, st.family);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn missing_file_is_empty() {
        let st = load_from(Path::new("/nonexistent/yaksrs/cache.json"));
        assert!(st.collapsed.is_empty() && st.family.is_empty());
    }

    #[test]
    fn slug_is_stable_and_short() {
        let a = slug(Path::new("/tmp/some/farm"));
        let b = slug(Path::new("/tmp/some/farm"));
        assert_eq!(a, b);
        assert_eq!(a.len(), 12);
    }

    /// The farm root changed from `<wt>/.yaks` to `<wt>` (yaks-6ffd): state kept
    /// under the old key is copied to the new, never overwriting what the new
    /// key already has, and the old files stay.
    #[test]
    fn state_under_the_pre_move_key_is_adopted_once() {
        let base = std::env::temp_dir().join(format!("yaks-adopt-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        let wt = base.join("farm");
        fs::create_dir_all(wt.join(".yaks")).unwrap();
        let new_root = fs::canonicalize(&wt).unwrap();
        assert_eq!(
            legacy_root(&new_root),
            Some(new_root.join(".yaks")),
            "a branch-root farm's old key is its `.yaks/`"
        );
        assert_eq!(legacy_root(&new_root.join(".yaks")), None);
        // A cache file and a view dir for the old key.
        let old_cache = base.join("cache-old.json");
        let new_cache = base.join("cache-new.json");
        let mut st = UiState::default();
        st.collapsed.insert("yak-1".to_string());
        save_to(&old_cache, &st);
        adopt_file(&old_cache, &new_cache);
        assert_eq!(load_from(&new_cache).collapsed, st.collapsed);
        // An existing new file wins.
        let mut other = UiState::default();
        other.collapsed.insert("yak-2".to_string());
        save_to(&new_cache, &other);
        adopt_file(&old_cache, &new_cache);
        assert_eq!(load_from(&new_cache).collapsed, other.collapsed);
        assert!(
            old_cache.is_file(),
            "the old file is left for an older yaks"
        );
        let _ = fs::remove_dir_all(&base);
    }
}

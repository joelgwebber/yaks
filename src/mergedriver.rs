//! The farm's git merge driver (yaks-4a53): merges two versions of one yak
//! file so the result is always a valid yak, never conflict markers.
//!
//! Prep stub (yaks-472e plan): the interface `yaks sync` builds on. The driver
//! itself (`merge`, the `yaks merge-driver` command) lands under yaks-4a53.

use anyhow::Result;
use std::path::Path;

/// Make sure the farm worktree at `worktree` (the git top-level holding
/// `.yaks/`) merges yak files with the yaks driver: the `.gitattributes` rule
/// on the farm branch and the repository's `merge.yaks.driver` config.
/// Idempotent; changes nothing that is already in place.
#[allow(dead_code)] // used by `yaks sync` (yaks-472e)
pub fn ensure_registered(_worktree: &Path) -> Result<()> {
    Ok(())
}

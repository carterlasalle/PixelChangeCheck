//! Remembered relays.
//!
//! A sharer needs three things from a relay — its address, its certificate
//! fingerprint, and its token — and cannot guess any of them. Passing them
//! by hand meant copying a line out of another terminal and pasting it
//! back, every session, which was the most tedious part of using a relay.
//! They are remembered here under a short name instead:
//!
//! ```text
//! pcc relay --listen 0.0.0.0:5900 --web --remember hetzner
//! pcc share --use hetzner
//! ```
//!
//! This is the `~/.ssh/config` + `known_hosts` pattern, not a certificate
//! authority: the fingerprint is pinned when the relay is saved and is
//! still re-checked against the live certificate on every connection, so a
//! relay that changes identity is refused rather than quietly trusted. The
//! token is stored in a `0600` file, the same way `docker` and `gh` keep
//! their credentials.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// One remembered relay, keyed in the store by a name the user chose.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedRelay {
    /// `host:port`, as the relay printed it.
    pub addr: String,
    /// SHA-256 of the relay's certificate, pinned at save time.
    pub pin: String,
    /// The relay's token. A secret; the file is `0600`.
    pub token: String,
}

/// Where the store lives.
///
/// `PCC_RELAYS` overrides it, which is also what makes the tests hermetic:
/// they pass a path instead of touching a real user's config.
pub fn store_path() -> Result<PathBuf> {
    if let Some(explicit) = std::env::var_os("PCC_RELAYS") {
        return Ok(PathBuf::from(explicit));
    }
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .context("no HOME to put the relay store under; set PCC_RELAYS to a path")?;
    Ok(base.join("pcc").join("relays.json"))
}

/// Read the store. A missing file is an empty store, not an error; a
/// corrupt one is an error, because silently discarding it would lose the
/// pinned fingerprints it exists to keep.
pub fn load_from(path: &Path) -> Result<BTreeMap<String, SavedRelay>> {
    match std::fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text)
            .with_context(|| format!("{} is not valid relay store JSON", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(BTreeMap::new()),
        Err(e) => Err(e).with_context(|| format!("could not read {}", path.display())),
    }
}

/// Write the store, creating the directory if needed and restricting the
/// file to the owner: it holds a token.
pub fn save_to(path: &Path, relays: &BTreeMap<String, SavedRelay>) -> Result<()> {
    if let Some(dir) = path.parent() {
        // Only tighten a directory we are creating. An existing one is the
        // user's — it may be a shared temp dir, a config root they own the
        // policy for — and failing the save because its mode could not be
        // changed would break every custom PCC_RELAYS path.
        let created = !dir.as_os_str().is_empty() && !dir.exists();
        std::fs::create_dir_all(dir)
            .with_context(|| format!("could not create {}", dir.display()))?;
        if created {
            restrict_to_owner(dir, 0o700)?;
        }
    }
    let text = serde_json::to_string_pretty(relays)?;
    std::fs::write(path, text).with_context(|| format!("could not write {}", path.display()))?;
    restrict_to_owner(path, 0o600)
}

#[cfg(unix)]
fn restrict_to_owner(path: &Path, mode: u32) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
        .with_context(|| format!("could not restrict {}", path.display()))
}

/// On platforms without POSIX modes there is nothing to tighten; the file
/// inherits the user profile's own protection.
#[cfg(not(unix))]
fn restrict_to_owner(_path: &Path, _mode: u32) -> Result<()> {
    Ok(())
}

/// The store as the CLI sees it, at the default path.
pub fn load() -> Result<BTreeMap<String, SavedRelay>> {
    load_from(&store_path()?)
}

/// Save (or replace) a relay under `name`. Returns where it was written.
pub fn remember(name: &str, addr: &str, pin: &str, token: &str) -> Result<PathBuf> {
    let path = store_path()?;
    let mut relays = load_from(&path)?;
    relays.insert(
        name.to_string(),
        SavedRelay {
            addr: addr.to_string(),
            pin: pin.to_string(),
            token: token.to_string(),
        },
    );
    save_to(&path, &relays)?;
    Ok(path)
}

/// Forget a relay. `Ok(false)` when there was nothing under that name.
pub fn forget(name: &str) -> Result<bool> {
    let path = store_path()?;
    let mut relays = load_from(&path)?;
    let removed = relays.remove(name).is_some();
    save_to(&path, &relays)?;
    Ok(removed)
}

/// Look one up by name.
pub fn get(name: &str) -> Result<Option<SavedRelay>> {
    Ok(load()?.remove(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(tag: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("pcc-relays-test-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_file(&p);
        p
    }

    /// The round trip that makes the pasting unnecessary.
    #[test]
    fn a_saved_relay_comes_back_identical() {
        let path = temp_path("round");
        let mut store = BTreeMap::new();
        store.insert(
            "hetzner".to_string(),
            SavedRelay {
                addr: "203.0.113.5:5900".into(),
                pin: "ab".repeat(32),
                token: "TOKEN1234".into(),
            },
        );
        save_to(&path, &store).unwrap();
        assert_eq!(load_from(&path).unwrap(), store);
        let _ = std::fs::remove_file(&path);
    }

    /// A missing store is an empty store. Erroring here would make a first
    /// run fail for the one user who has no relays yet.
    #[test]
    fn a_missing_store_is_empty_rather_than_an_error() {
        let path = temp_path("missing");
        assert!(load_from(&path).unwrap().is_empty());
    }

    /// A corrupt store must not be silently replaced: the pinned
    /// fingerprints in it are the only record of which relay is which.
    #[test]
    fn a_corrupt_store_is_reported_rather_than_discarded() {
        let path = temp_path("corrupt");
        std::fs::write(&path, "{ not json").unwrap();
        let err = load_from(&path).unwrap_err().to_string();
        assert!(err.contains("not valid relay store JSON"), "got: {err}");
        assert!(path.exists(), "the original file must be left alone");
        let _ = std::fs::remove_file(&path);
    }

    /// The file holds a token, so it must not be world-readable.
    #[cfg(unix)]
    #[test]
    fn the_store_is_owner_only() {
        use std::os::unix::fs::PermissionsExt;
        let path = temp_path("perms");
        save_to(&path, &BTreeMap::new()).unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "store must be 0600, got {mode:o}");
        let _ = std::fs::remove_file(&path);
    }
}

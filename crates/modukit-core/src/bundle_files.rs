//! File mechanics for the M7 bundle protocol — the `bundle` helper:
//! hash-manifest parse, member verification (streamed, never RAM-whole),
//! and the content-addressed object store (D31/D55).

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::bundle::BundleError;
use crate::sha256;

/// D31/D55 content-addressing threshold: files > 1 MiB become objects
/// (the ">1MB" ledger line read as 1 MiB = 1<<20; at the boundary a file
/// stays a regular bundle member).
pub(crate) const OBJECT_THRESHOLD: u64 = 1 << 20;

/// One hash-manifest entry: bundle-relative path, lowercase hex sha256,
/// optional declared size (checked when present). Lives in the manifest's
/// own TOML envelope — same D7 file, no second truth source (D31/D28).
#[derive(Deserialize)]
pub(crate) struct HashEntry {
    pub path: String,
    pub sha256: String,
    #[serde(default)]
    pub size: Option<u64>,
}

/// The two checklist arrays (`[[files]]` general members + `[[resources]]`
/// per the 05 §5 envelope) read as one verification list.
#[derive(Deserialize)]
pub(crate) struct RawHashManifest {
    #[serde(default)]
    pub files: Vec<HashEntry>,
    #[serde(default)]
    pub resources: Vec<HashEntry>,
}

/// Bundle-relative member path rule: non-empty, no leading '/', no
/// backslashes, no empty or `..` segments. Trust boundary — manifest
/// paths must stay inside the bundle they describe.
pub(crate) fn validate_rel(path: &str) -> Result<(), BundleError> {
    let bad = path.is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || path.split('/').any(|seg| seg.is_empty() || seg == "..");
    if bad {
        return Err(BundleError::ManifestField(format!(
            "path {path:?} is not a bundle-relative member path"
        )));
    }
    Ok(())
}

/// Parse + sanity the checklist: every entry path and every digest shape
/// is validated BEFORE any file is touched.
pub(crate) fn parse(toml_text: &str) -> Result<Vec<HashEntry>, BundleError> {
    let raw: RawHashManifest = toml::from_str(toml_text)
        .map_err(|e| BundleError::ManifestField(format!("hash manifest: {e}")))?;
    let mut entries = raw.files;
    entries.extend(raw.resources);
    for entry in &entries {
        validate_rel(&entry.path)?;
        let hex_ok = entry.sha256.len() == 64
            && entry.sha256.bytes().all(|b| b.is_ascii_hexdigit())
            && !entry.sha256.bytes().any(|b| b.is_ascii_uppercase());
        if !hex_ok {
            return Err(BundleError::ManifestField(format!(
                "sha256 for {:?} must be 64 lowercase hex chars",
                entry.path
            )));
        }
    }
    Ok(entries)
}

/// Verify one staged member against its checklist entry; answers the real
/// file size (the object-store split needs it after the check, not before).
///
/// # Errors
///
/// `MissingFile` / `SizeMismatch` / `HashMismatch` / `Io`.
pub(crate) fn verify_member(staged_dir: &Path, entry: &HashEntry) -> Result<u64, BundleError> {
    let path = staged_dir.join(&entry.path);
    let meta = std::fs::metadata(&path).map_err(|_| BundleError::MissingFile {
        path: path.display().to_string(),
    })?;
    if !meta.is_file() {
        return Err(BundleError::MissingFile {
            path: path.display().to_string(),
        });
    }
    if let Some(expected) = entry.size
        && meta.len() != expected
    {
        return Err(BundleError::SizeMismatch {
            path: entry.path.clone(),
            expected,
            found: meta.len(),
        });
    }
    let found = sha256::hash_file_hex(&path).map_err(|e| io_err(&path, e))?;
    if found != entry.sha256 {
        return Err(BundleError::HashMismatch {
            path: entry.path.clone(),
            expected: entry.sha256.clone(),
            found,
        });
    }
    Ok(meta.len())
}

/// Commit one verified member to the object store (D31/D55): > 1 MiB files
/// live at `<objects-root>/sha256/<hex>`; the bundle path becomes a
/// HARDLINK to the object (logical privacy, physical sharing); the object
/// gets read-only bits after its first commit. Objects outlive bundles —
/// uninstall removes links, the filesystem refcount retires the object.
///
/// # Errors
///
/// `Io` on any filesystem refusal.
pub(crate) fn commit_object(
    objects_root: &Path,
    staged_dir: &Path,
    entry: &HashEntry,
) -> Result<(), BundleError> {
    let dir = objects_root.join("sha256");
    std::fs::create_dir_all(&dir).map_err(|e| io_err(&dir, e))?;
    let object = dir.join(&entry.sha256);
    let member = staged_dir.join(&entry.path);
    if object.exists() {
        // shortcut: the hash-named object's existence is its content proof;
        // a store-wide scrub is the object store's own later concern.
        std::fs::remove_file(&member).map_err(|e| io_err(&member, e))?;
    } else {
        std::fs::rename(&member, &object).map_err(|e| io_err(&member, e))?;
        set_read_only(&object)?;
    }
    std::fs::hard_link(&object, &member).map_err(|e| io_err(&member, e))?;
    Ok(())
}

/// Read-only permission bits for committed objects (D55 "444").
/// Non-unix v0: bits are a unix story, files stay as-created (documented).
fn set_read_only(path: &Path) -> Result<(), BundleError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o444))
            .map_err(|e| io_err(path, e))
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Ok(())
    }
}

/// Re-hash one installed member for the load-time gate.
pub(crate) fn rehash(path: &Path) -> Result<String, BundleError> {
    sha256::hash_file_hex(path).map_err(|e| io_err(path, e))
}

pub(crate) fn io_err(path: &Path, reason: impl std::fmt::Display) -> BundleError {
    BundleError::Io {
        path: path.display().to_string(),
        reason: reason.to_string(),
    }
}

/// The one writable data boundary per plugin (D26): `<data-root>/<id>/`,
/// not configurable; the id is grammar-checked before it names a path
/// (never let an unvalidated string reach join — same discipline as
/// `validate_rel`). The installer removes it on uninstall.
pub(crate) fn data_slot(data_root: &Path, id: &str) -> Result<PathBuf, BundleError> {
    crate::identity::validate_reverse_dns(id).map_err(|reason| BundleError::Grammar {
        value: id.to_string(),
        reason,
    })?;
    Ok(data_root.join(id))
}

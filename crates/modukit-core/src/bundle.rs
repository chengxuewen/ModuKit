//! M7 logical-bundle install protocol (D28; modules/05 §4 layout, §6
//! install wording; D31/D55 object store; D7 cross-check; D34 duplicate-id;
//! D26 data boundary). Physical form is NOT defined (D28): this consumes
//! the DIRECTORY form — a staging dir holding `modukit.toml` (the D7
//! manifest plus its `[[files]]`/`[[resources]]` hash checklist), the
//! artifact at a bundle-relative path, and optional lib/ vendor/
//! resources/. An archive form would expand into this same pipeline (zero
//! resolve-logic fork); zip unpack is DEFERRED with the D28 CLI trigger.
//!
//! Protocol: (a) staging/install/data/objects paths from the deploy config
//! `[install]` fields with sane defaults, cross-checked at open like every
//! field (values validated; the same-filesystem assumption behind the
//! atomic rename(2) is verified on unix device ids); (b) EVERY file listed
//! in the bundle's hash manifest is re-hashed and size-checked BEFORE
//! anything touches the install root; (c) the staged bundle is renamed
//! atomically into <bundles>/<id>/<version>; (d) a second bundle
//! declaring an installed manifest id is rejected at staging (D34: unique
//! per host, not per version; supersession is UPDATE, D8); (e) uninstall
//! removes the install dir and the data root <data>/<id>/ (D26; objects
//! stay — the link count retires them, D55).
//!
//! D35 red line honored: the id comes from the manifest ONLY — staging
//! directory names carry zero identity.

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::bundle_files::{self, HashEntry};
use crate::executor::{DlopenExecutor, Executor};
use crate::identity::{PluginId, validate_reverse_dns};
use crate::manifest::{Manifest, PlacementKind};

// The standalone failure type lives in `bundle_error` (file-size discipline).
pub use crate::bundle_error::BundleError;

/// The resolved host-tree paths (deploy tier, D37 layout: one tree per
/// host). Year-1 defaults, overridable by `[install]` string fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallConfig {
    pub root: PathBuf,
    pub staging: PathBuf,
    pub bundles: PathBuf,
    pub data: PathBuf,
    pub objects: PathBuf,
}

#[derive(Deserialize, Default)]
struct RawDeploy {
    #[serde(default)]
    install: Option<RawInstall>,
}

#[derive(Deserialize, Default)]
struct RawInstall {
    #[serde(default)]
    staging: Option<String>,
    #[serde(default)]
    bundles: Option<String>,
    #[serde(default)]
    data: Option<String>,
    #[serde(default)]
    objects: Option<String>,
}

/// `[install]` field cross-check: non-empty, host-tree-relative, no `..`
/// traversal (absolute paths are refused — the tree root is the boundary).
fn check_config_field(key: &str, value: &str) -> Result<(), BundleError> {
    let traversal = value.split(['/', '\\']).any(|seg| seg == "..");
    if value.is_empty() || value.starts_with('/') || value.contains('\\') || traversal {
        return Err(BundleError::Config(format!(
            "[install] {key} = {value:?} is not a host-tree-relative path"
        )));
    }
    Ok(())
}

impl InstallConfig {
    /// Resolve the `[install]` fields over the deploy TOML (None = all
    /// defaults). Every field takes the cross-check like the rest of the
    /// deploy tier (D7/D29/D37 family).
    ///
    /// # Errors
    ///
    /// `BundleError::Config` for TOML shape or field-grammar violations.
    pub fn from_deploy(root: &Path, deploy_toml: Option<&str>) -> Result<Self, BundleError> {
        let raw: RawDeploy = match deploy_toml {
            Some(text) => toml::from_str(text)
                .map_err(|e| BundleError::Config(format!("deploy TOML: {e}")))?,
            None => RawDeploy::default(),
        };
        let install = raw.install.unwrap_or_default();
        let take =
            |name: &str, given: Option<String>, default: &str| -> Result<PathBuf, BundleError> {
                match given {
                    Some(value) => {
                        check_config_field(name, &value)?;
                        Ok(root.join(value))
                    }
                    None => Ok(root.join(default)),
                }
            };
        Ok(Self {
            root: root.to_path_buf(),
            staging: take("staging", install.staging, "staging")?,
            bundles: take("bundles", install.bundles, "bundles")?,
            data: take("data", install.data, "data")?,
            objects: take("objects", install.objects, "objects")?,
        })
    }
}

/// The installed-bundle answer of `BundleInstaller::install`: everything a
/// consumer needs to boot the plugin through the untouched kernel doors and
/// to re-verify the object-backed files before any dlopen (D31 load gate).
#[derive(Debug, Clone)]
pub struct InstalledBundle {
    pub id: PluginId,
    pub version: String,
    pub dir: PathBuf,
    /// The manifest with `entry.artifact` rewritten to the INSTALLED
    /// absolute path — ready for `Kernel::install`.
    pub manifest: Manifest,
    /// (bundle-relative path, hex) for every > 1 MiB object-backed member.
    big_files: Vec<(String, String)>,
}

impl InstalledBundle {
    /// D31's "relocation cheap, load re-verifies" half: before handing the
    /// bundle to a dlopen, re-hash every > 1 MiB object-backed file against
    /// the checklist. Cheap boot, honest gate.
    ///
    /// # Errors
    ///
    /// `HashMismatch` / `Io` naming the first betrayed file.
    pub fn verify_for_load(&self) -> Result<(), BundleError> {
        for (rel, expected) in &self.big_files {
            let path = self.dir.join(rel);
            let found = bundle_files::rehash(&path)?;
            if found != *expected {
                return Err(BundleError::HashMismatch {
                    path: rel.clone(),
                    expected: expected.clone(),
                    found,
                });
            }
        }
        Ok(())
    }
}

/// The D28 install-protocol actor over one host tree. Holds resolved paths
/// only — the filesystem is the state (D37 file-as-truth posture).
#[derive(Debug, Clone)]
pub struct BundleInstaller {
    config: InstallConfig,
}

impl BundleInstaller {
    /// Resolve the deploy config, create the four host-tree directories,
    /// and cross-check the same-filesystem assumption behind the atomic
    /// rename (unix device ids; other platforms rely on the documented
    /// same-volume deployment rule).
    ///
    /// # Errors
    ///
    /// `BundleError::Config` / `Io`.
    pub fn open(root: &Path, deploy_toml: Option<&str>) -> Result<Self, BundleError> {
        let config = InstallConfig::from_deploy(root, deploy_toml)?;
        for dir in [
            &config.staging,
            &config.bundles,
            &config.data,
            &config.objects,
        ] {
            std::fs::create_dir_all(dir).map_err(|e| bundle_files::io_err(dir, e))?;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let staged = std::fs::metadata(&config.staging)
                .map_err(|e| bundle_files::io_err(&config.staging, e))?;
            let live = std::fs::metadata(&config.bundles)
                .map_err(|e| bundle_files::io_err(&config.bundles, e))?;
            if staged.dev() != live.dev() {
                return Err(BundleError::Config(
                    "staging and install root must share a filesystem (atomic rename)".to_string(),
                ));
            }
        }
        Ok(Self { config })
    }

    #[must_use]
    pub fn staging_dir(&self) -> &Path {
        &self.config.staging
    }

    #[must_use]
    pub fn bundles_dir(&self) -> &Path {
        &self.config.bundles
    }

    #[must_use]
    pub fn data_dir(&self) -> &Path {
        &self.config.data
    }

    #[must_use]
    pub fn objects_dir(&self) -> &Path {
        &self.config.objects
    }

    /// Run the protocol over the staging entry `staged_name`. Identity is
    /// read from the manifest, never the path (D35).
    ///
    /// # Errors
    ///
    /// The `BundleError` family; on any failure the install root is
    /// untouched (verify-before-rename, D28 step order).
    pub fn install(&self, staged_name: &str) -> Result<InstalledBundle, BundleError> {
        bundle_files::validate_rel(staged_name)?;
        let staged = self.config.staging.join(staged_name);
        let toml_path = staged.join("modukit.toml");
        let text = std::fs::read_to_string(&toml_path).map_err(|_| BundleError::MissingFile {
            path: toml_path.display().to_string(),
        })?;
        let manifest = Manifest::from_toml_str(&text).map_err(BundleError::Manifest)?;
        self.check_grammar(&manifest)?;
        validate_version(&manifest.version)?;

        // (b) the hash checklist — fully verified before the first write
        // anywhere under the install root.
        let entries = bundle_files::parse(&text)?;
        let artifact = &manifest.entry.artifact;
        bundle_files::validate_rel(artifact)?;
        if !entries.iter().any(|e| e.path == *artifact) {
            return Err(BundleError::ArtifactNotListed {
                path: artifact.clone(),
            });
        }
        let mut big: Vec<&HashEntry> = Vec::new();
        for entry in &entries {
            let size = bundle_files::verify_member(&staged, entry)?;
            if size > bundle_files::OBJECT_THRESHOLD {
                big.push(entry);
            }
        }

        // (d) duplicate identity against the installed set (D34).
        let id_dir = self.config.bundles.join(manifest.id.as_str());
        if id_dir.exists() {
            return Err(BundleError::DuplicateId {
                id: manifest.id.to_string(),
            });
        }

        // D7 cross-check: the staged artifact's self-report must agree
        // with the deploy tier BEFORE the rename and before any call.
        cross_check(&manifest, &staged)?;

        // D31 object store: > 1 MiB members become hardlinks to shared
        // objects (dedup), everything else stays a plain bundle member.
        for entry in &big {
            bundle_files::commit_object(&self.config.objects, &staged, entry)?;
        }

        // (c) atomic rename staging -> install root.
        let target = id_dir.join(&manifest.version);
        std::fs::create_dir_all(&id_dir).map_err(|e| bundle_files::io_err(&id_dir, e))?;
        std::fs::rename(&staged, &target).map_err(|e| bundle_files::io_err(&staged, e))?;

        let mut manifest = manifest;
        manifest.entry.artifact = target
            .join(&manifest.entry.artifact)
            .to_string_lossy()
            .into_owned();
        Ok(InstalledBundle {
            id: manifest.id.clone(),
            version: manifest.version.clone(),
            dir: target,
            manifest,
            big_files: big
                .iter()
                .map(|e| (e.path.clone(), e.sha256.clone()))
                .collect(),
        })
    }

    /// (e) Uninstall: remove the install dir and the plugin's data root
    /// (D26). Object-store entries stay — links die, the refcount retires
    /// the object (D55).
    ///
    /// # Errors
    ///
    /// `NotInstalled` / `Io`.
    pub fn uninstall(&self, id: &PluginId) -> Result<(), BundleError> {
        let id_dir = bundle_files::data_slot(&self.config.bundles, id.as_str())?;
        if !id_dir.is_dir() {
            return Err(BundleError::NotInstalled { id: id.to_string() });
        }
        std::fs::remove_dir_all(&id_dir).map_err(|e| bundle_files::io_err(&id_dir, e))?;
        let data = bundle_files::data_slot(&self.config.data, id.as_str())?;
        if data.is_dir() {
            std::fs::remove_dir_all(&data).map_err(|e| bundle_files::io_err(&data, e))?;
        }
        Ok(())
    }

    /// The M3 grammar deferral, ended here for new call sites: manifest id
    /// plus every declared requires/provides service id go through the one
    /// shared reverse-DNS validator.
    fn check_grammar(&self, manifest: &Manifest) -> Result<(), BundleError> {
        let check = |value: &str| -> Result<(), BundleError> {
            validate_reverse_dns(value).map_err(|reason| BundleError::Grammar {
                value: value.to_string(),
                reason,
            })
        };
        check(manifest.id.as_str())?;
        for req in &manifest.requires {
            check(req.service.as_str())?;
        }
        for prov in &manifest.provides {
            check(prov.service.as_str())?;
        }
        Ok(())
    }
}

/// The D7 deploy <-> self-report agreement, executed through the executor
/// door: dlopen reads the descriptor (no plugin code runs — 04 §3), the
/// reported id/version must equal the manifest's, ABI mismatch arrives as
/// a typed `KernelError` (the bad-plugin path). A silent descriptor (no
/// id/version strings) is disagreement: the strict v0 side of the liar
/// family.
fn cross_check(manifest: &Manifest, staged: &Path) -> Result<(), BundleError> {
    let mut probe = manifest.clone();
    probe.entry.artifact = staged
        .join(&manifest.entry.artifact)
        .to_string_lossy()
        .into_owned();
    // The probe door is dlopen (descriptor read, no plugin code runs —
    // 04 §3), so the probe rides the dl kind whatever the bundle declares:
    // `DlopenExecutor` polices its own placement kind, and an exe bundle's
    // spawn-time identity is cross-checked again at HELLO (executor_proc).
    probe.entry.kind = PlacementKind::Dl;
    let handle = DlopenExecutor::new()
        .load(&probe)
        .map_err(BundleError::Kernel)?;
    let (id, version) = handle.self_report();
    drop(handle); // release the staged mapping before anything moves it
    match id {
        Some(report) if report == manifest.id.as_str() => {}
        Some(report) => {
            return Err(BundleError::CrossCheck {
                what: format!(
                    "manifest id {:?}, artifact self-reports {report:?}",
                    manifest.id.as_str()
                ),
            });
        }
        None => {
            return Err(BundleError::CrossCheck {
                what: "artifact descriptor reports no id".to_string(),
            });
        }
    }
    match version {
        Some(report) if report == manifest.version => {}
        Some(report) => {
            return Err(BundleError::CrossCheck {
                what: format!(
                    "manifest version {:?}, artifact self-reports {report:?}",
                    manifest.version
                ),
            });
        }
        None => {
            return Err(BundleError::CrossCheck {
                what: "artifact descriptor reports no version".to_string(),
            });
        }
    }
    Ok(())
}

/// The version string names a path component under the install root: keep
/// it to a plain segment (year-1 semver-shaped data, no traversal).
fn validate_version(version: &str) -> Result<(), BundleError> {
    let ok = !version.is_empty()
        && version != "."
        && version != ".."
        && !version.contains('/')
        && !version.contains('\\');
    if !ok {
        return Err(BundleError::ManifestField(format!(
            "version {version:?} cannot name an install directory"
        )));
    }
    Ok(())
}

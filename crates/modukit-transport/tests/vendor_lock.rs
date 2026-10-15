//! The D73 / 07 §7 vendor-linkage floor, PROVEN (an unfalsifiable check is
//! decoration per C0). This test reads the workspace-root `Cargo.lock` and
//! fails if M5's crates ever link a data-plane vendor or add a crates.io / git
//! dependency outside the frozen approved set. It is the gate that makes
//! "iceoryx2 arrival is Stage-2, vendor-linkage is forbidden" a runnable fact,
//! not a comment.

use std::collections::BTreeSet;
use std::path::PathBuf;

/// The frozen crates.io package set the workspace approves (the transitive
/// closure of `libloading` + `serde` + `toml`, the only direct deps at M5). Any
/// new crates.io package — an "innocent" helper or a vendor — shows up here as
/// a diff and must go through its own decision, not slip into a milestone.
const APPROVED_CRATES_IO: &[&str] = &[
    "cfg-if",
    "equivalent",
    "hashbrown",
    "indexmap",
    "libloading",
    "proc-macro2",
    "quote",
    "serde",
    "serde_core",
    "serde_derive",
    "serde_spanned",
    "syn",
    "toml",
    "toml_datetime",
    "toml_parser",
    "toml_writer",
    "unicode-ident",
    "windows-link",
    "winnow",
];

/// Named data-plane / runtime vendors that are forbidden to appear in year 1
/// (07 §7; iceoryx2 and zenoh are the Stage-2 pool/carrier decision — D25).
const FORBIDDEN_VENDORS: &[&str] = &["iceoryx2", "iceoryx", "zenoh", "tokio", "nix"];

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("crate lives under <repo>/crates/")
        .to_path_buf()
}

/// Collect `(name, source)` per `[[package]]` block from `Cargo.lock`.
fn lockfile_packages() -> Vec<(String, Option<String>)> {
    let text = std::fs::read_to_string(workspace_root().join("Cargo.lock"))
        .expect("workspace-root Cargo.lock is committed (constraints.md)");
    let mut out = Vec::new();
    let mut name: Option<String> = None;
    let mut source: Option<String> = None;
    for line in text.lines() {
        let line = line.trim();
        if line == "[[package]]" {
            if let Some(n) = name.take() {
                out.push((n, source.take()));
            }
            source = None;
        } else if let Some(rest) = line.strip_prefix("name = ") {
            name = Some(rest.trim_matches('"').to_string());
        } else if let Some(rest) = line.strip_prefix("source = ") {
            source = Some(rest.trim_matches('"').to_string());
        }
    }
    if let Some(n) = name {
        out.push((n, source));
    }
    out
}

#[test]
fn the_crates_io_set_is_exactly_the_frozen_approved_baseline() {
    let registry: BTreeSet<String> = lockfile_packages()
        .into_iter()
        .filter_map(|(name, source)| {
            let src = source?;
            src.contains("registry+").then_some(name)
        })
        .collect();
    let approved: BTreeSet<String> = APPROVED_CRATES_IO
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    assert_eq!(
        registry, approved,
        "the crates.io closure changed — a new dependency slipped in past the D73 gate"
    );
}

#[test]
fn no_git_or_path_vendor_dependency_is_linked() {
    for (name, source) in lockfile_packages() {
        if let Some(src) = source {
            assert!(
                !src.contains("git+"),
                "package {name} is linked from a git source; year 1 forbids it"
            );
            assert!(
                src.contains("registry+") || src.contains("crates.io"),
                "package {name} has a non-crates.io, non-local source {src:?}"
            );
        }
        for vendor in FORBIDDEN_VENDORS {
            assert_ne!(
                name, *vendor,
                "vendor {vendor} is linked — the first pool backend is the Stage-2 \
                 iceoryx2 decision (D25/D73), never a silent year-1 dependency"
            );
        }
    }
}

#[test]
fn modukit_transport_itself_has_no_external_dependencies() {
    // The seam crate is std-only: its Cargo.lock block must list no dependencies.
    let text =
        std::fs::read_to_string(workspace_root().join("Cargo.lock")).expect("Cargo.lock present");
    let block = text
        .split("[[package]]")
        .find(|blk| blk.contains("name = \"modukit-transport\""))
        .expect("modukit-transport is a workspace member present in Cargo.lock");
    assert!(
        !block.contains("dependencies = [") && !block.contains("[[package.dependencies]]"),
        "modukit-transport must carry zero dependencies (std-only seam)"
    );
}

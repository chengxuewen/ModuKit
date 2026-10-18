//! The D73/D76 vendor-linkage floor, PROVEN (an unfalsifiable check is
//! decoration per C0). Amended at S2-M4 per D76: the blanket zero-crates.io
//! rule became a SPLIT proof. (1) The DEFAULT feature set must still link
//! nothing outside the frozen approved baseline — proven from each crate's
//! Cargo.toml shape (an optional dependency never enters the default build,
//! and Cargo.lock alone cannot show that because it records the union over
//! feature sets). (2) The one admitted vendor enters ONLY as an optional,
//! non-default, exact-pinned (D25) dependency of `modukit-transport`, and
//! whatever it adds to `Cargo.lock` must be exactly its own transitive tree —
//! no third-party package may ride in on any other pretext. (3) Every other
//! workspace member (kernel core first) must not reach the vendor at all.

use std::collections::{BTreeSet, VecDeque};
use std::path::PathBuf;

/// The frozen crates.io package set the workspace approves (the transitive
/// closure of `libloading` + `serde` + `toml`, the only direct default-path
/// deps). Any crates.io package outside this set AND outside the admitted
/// vendor tree — an "innocent" helper or a second vendor — shows up here as a
/// diff and must go through its own decision, not slip into a milestone.
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

/// The single admitted vendor (D25: first pool backend, exact-pinned at S2-M4
/// via the non-default `vendor-shm` feature, confined to one module per D76).
const ADMITTED_VENDOR: &str = "iceoryx2";
/// The exact pin D25 requires for the admitted vendor (D76 amended the
/// blanket ban; the pin itself is what makes the admission auditable).
const ADMITTED_VENDOR_VERSION: &str = "0.10.0";

/// Named data-plane / runtime vendors still forbidden everywhere (07 §7).
/// `iceoryx2` left this list under the D76 amendment (S2-M4); the legacy C++
/// `iceoryx` and every future vendor are gate material for their own hearing,
/// never a silent milestone entry.
const FORBIDDEN_VENDORS: &[&str] = &["iceoryx", "zenoh", "tokio", "nix"];

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("crate lives under <repo>/crates/")
        .to_path_buf()
}

struct LockedPackage {
    name: String,
    version: String,
    source: Option<String>,
    deps: Vec<String>,
}

/// Collect `[[package]]` blocks (name, version, source, direct dep names)
/// from `Cargo.lock`. Dependency entries look like `"name"`, `"name version"`
/// or `"name version source..."`; the first token is always the name.
fn lockfile_packages() -> Vec<LockedPackage> {
    let text = std::fs::read_to_string(workspace_root().join("Cargo.lock"))
        .expect("workspace-root Cargo.lock is committed (constraints.md)");
    let mut out: Vec<LockedPackage> = Vec::new();
    let mut cur: Option<LockedPackage> = None;
    let mut in_deps = false;
    for line in text.lines() {
        let line = line.trim();
        if line == "[[package]]" {
            if let Some(p) = cur.take() {
                out.push(p);
            }
            cur = Some(LockedPackage {
                name: String::new(),
                version: String::new(),
                source: None,
                deps: Vec::new(),
            });
            in_deps = false;
            continue;
        }
        let Some(pkg) = cur.as_mut() else { continue };
        if in_deps {
            if line == "]" {
                in_deps = false;
            } else if let Some(entry) = line.strip_prefix('"') {
                let entry = entry.trim_end_matches([',', '"']);
                let name = entry.split_whitespace().next().unwrap_or_default();
                if !name.is_empty() {
                    pkg.deps.push(name.to_string());
                }
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("name = ") {
            pkg.name = rest.trim_matches('"').to_string();
        } else if let Some(rest) = line.strip_prefix("version = ") {
            pkg.version = rest.trim_matches('"').to_string();
        } else if let Some(rest) = line.strip_prefix("source = ") {
            pkg.source = Some(rest.trim_matches('"').to_string());
        } else if line == "dependencies = [" {
            in_deps = true;
        }
    }
    if let Some(p) = cur {
        out.push(p);
    }
    out
}

/// Transitive closure of direct-dependency edges from `root` over the lock
/// graph (BFS over package names).
fn closure_of(packages: &[LockedPackage], root: &str) -> BTreeSet<String> {
    let mut seen = BTreeSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(root.to_string());
    while let Some(name) = queue.pop_front() {
        if !seen.insert(name.clone()) {
            continue;
        }
        for pkg in packages.iter().filter(|q| q.name == name) {
            for dep in &pkg.deps {
                queue.push_back(dep.clone());
            }
        }
    }
    seen
}

#[test]
fn the_crates_io_set_is_the_frozen_baseline_plus_exactly_the_admitted_vendor_tree() {
    let packages = lockfile_packages();
    let registry: BTreeSet<String> = packages
        .iter()
        .filter(|p| p.source.as_deref().is_some_and(|s| s.contains("registry+")))
        .map(|p| p.name.clone())
        .collect();
    let approved: BTreeSet<String> = APPROVED_CRATES_IO
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    let vendor_tree = closure_of(&packages, ADMITTED_VENDOR);
    assert!(
        vendor_tree.contains(ADMITTED_VENDOR),
        "the admitted vendor must exist in the lock graph"
    );
    let admitted: BTreeSet<String> = approved.union(&vendor_tree).cloned().collect();
    let slipped: Vec<&String> = registry.difference(&admitted).collect();
    assert!(
        slipped.is_empty(),
        "crates.io packages appeared that are neither the frozen baseline nor \
         part of the admitted vendor's transitive tree: {slipped:?}"
    );
    let vendor_pkg = packages
        .iter()
        .find(|p| p.name == ADMITTED_VENDOR)
        .expect("admitted vendor is locked");
    assert_eq!(
        vendor_pkg.version, ADMITTED_VENDOR_VERSION,
        "the admitted vendor must sit at its exact D25 pin"
    );
}

#[test]
fn no_git_or_path_vendor_dependency_is_linked() {
    for pkg in lockfile_packages() {
        if let Some(src) = pkg.source {
            assert!(
                !src.contains("git+"),
                "package {} is linked from a git source; only registry pins are approved",
                pkg.name
            );
            assert!(
                src.contains("registry+") || src.contains("crates.io"),
                "package {} has a non-crates.io, non-local source {src:?}",
                pkg.name
            );
        }
        for vendor in FORBIDDEN_VENDORS {
            assert_ne!(
                pkg.name, *vendor,
                "vendor {vendor} is linked — only the single admitted vendor enters, \
                 and only behind the non-default feature (D76); every other vendor \
                 is its own hearing, never a silent dependency"
            );
        }
    }
}

#[test]
fn the_vendor_enters_only_as_one_optional_non_default_exact_pinned_dependency() {
    // Default-purity is a Cargo.toml property (Cargo.lock is feature-blind):
    // the seam crate's only dependency is the admitted vendor, it is
    // `optional = true`, the only feature is the neutral `vendor-shm`
    // gate carrying the `dep:` link, and NO `default` key exists — so the
    // default feature set (empty) links zero crates.io packages.
    let toml =
        std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
            .expect("modukit-transport Cargo.toml");
    let deps: Vec<&str> = toml
        .lines()
        .skip_while(|l| !l.trim().starts_with("[dependencies]"))
        .skip(1)
        .take_while(|l| !l.trim_start().starts_with('['))
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();
    assert_eq!(
        deps.len(),
        1,
        "modukit-transport must carry exactly one dependency entry, the admitted vendor"
    );
    let vendor_dep = deps[0];
    assert!(
        vendor_dep.starts_with(&format!("{ADMITTED_VENDOR} = "))
            && vendor_dep.contains("optional = true")
            && vendor_dep.contains(&format!("\"={ADMITTED_VENDOR_VERSION}\"")),
        "the one dependency must be the vendor, optional and exact-pinned per D25: {vendor_dep}"
    );
    assert!(
        toml.lines()
            .any(|l| l.trim() == format!("vendor-shm = [\"dep:{ADMITTED_VENDOR}\"]")),
        "the neutral D76 feature must gate exactly the dep link"
    );
    assert!(
        !toml
            .lines()
            .any(|l| l.trim_start().starts_with("default =")),
        "no `default` feature may exist while the vendor is the only dependency: \
         a default set naming vendor-shm would link the vendor by default"
    );
    // The lock graph must agree: modukit-transport's sole direct edge is the vendor.
    let packages = lockfile_packages();
    let transport = packages
        .iter()
        .find(|p| p.name == "modukit-transport")
        .expect("transport is in the lock");
    assert_eq!(
        transport.deps,
        vec![ADMITTED_VENDOR.to_string()],
        "the seam crate's only lock edge is the admitted vendor"
    );
}

#[test]
fn no_other_workspace_member_reaches_the_admitted_vendor() {
    // D76: "Kernel core stays std-only, period." Walk every LOCAL (path)
    // package in the lock graph except the transport seam: none of their
    // transitive closures may contain the vendor.
    let packages = lockfile_packages();
    let local_members: Vec<&LockedPackage> = packages
        .iter()
        .filter(|p| p.source.is_none() && p.name != ADMITTED_VENDOR)
        .filter(|p| p.name != "modukit-transport")
        .collect();
    assert!(
        !local_members.is_empty(),
        "the lock must carry the workspace members for this proof to mean anything"
    );
    for member in local_members {
        let reach = closure_of(&packages, &member.name);
        assert!(
            !reach.contains(ADMITTED_VENDOR),
            "workspace member {} reaches the admitted vendor — only the seam crate \
             may, and only behind the non-default feature",
            member.name
        );
    }
}

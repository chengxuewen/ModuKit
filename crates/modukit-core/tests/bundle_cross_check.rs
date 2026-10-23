//! M7 D7 cross-check + reverse-DNS grammar (the M3 deferral closes here).
//! First-red area: cross-check/grammar — the bad-plugin ABI rejection now
//! happens at INSTALL (before the atomic rename), and a manifest that lies
//! about the artifact's self-report never reaches the install root.

// D104 T2.4: test-scaffolding file; the clippy.toml allow-*-in-tests valve
// exempts #[test] fn bodies only, not plain helper fns in a tests/ target
// (clippy 0.1.99), so the exemption lands here. Production stays denied.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::{Path, PathBuf};
use std::process::Command;

use modukit_core::bundle::BundleInstaller;
use modukit_core::identity::{IdGrammar, PluginId, ServiceId, TopicId, validate_reverse_dns};

const EXAMPLE_ID: &str = "com.example.modukit.demo";
const BAD_ID: &str = "com.example.modukit.bad";

fn build_fixture(crate_name: &str, lib_stem: &str) -> Option<PathBuf> {
    if std::env::var_os("MODUKIT_SKIP_DLOPEN").is_some() {
        eprintln!("SKIPPING: MODUKIT_SKIP_DLOPEN is set");
        return None;
    }
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).parent()?.parent()?;
    let target_dir = repo.join("out").join("dlopen-test-target");
    let status = Command::new(&cargo)
        .current_dir(repo)
        .args(["build", "-p", crate_name, "--target-dir"])
        .arg(&target_dir)
        .env("CARGO_TARGET_DIR", &target_dir)
        .status();
    match status {
        Ok(s) if s.success() => {}
        Ok(s) => {
            eprintln!("SKIPPING: fixture build failed ({crate_name}): {s}");
            return None;
        }
        Err(e) => {
            eprintln!("SKIPPING: cannot spawn cargo: {e}");
            return None;
        }
    }
    let so = target_dir.join("debug").join(format!("lib{lib_stem}.so"));
    so.exists().then_some(so)
}

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("modukit-m7x-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("staging")).expect("scratch");
    dir
}

/// Stage a bundle whose manifest may lie; returns the staging entry name.
/// `files_hash_ok = false` still writes correct hashes (the lies tested
/// here are about the SELF-REPORT, not the content hash).
fn stage(
    staging: &Path,
    name: &str,
    id: &str,
    version: &str,
    artifact_src: &Path,
    provides: Option<&str>,
) {
    use modukit_core::sha256::{hash_file, to_hex};
    let dir = staging.join(name);
    std::fs::create_dir_all(&dir).expect("bundle dir");
    std::fs::copy(artifact_src, dir.join("plugin.so")).expect("copy so");
    let hash = to_hex(&hash_file(&dir.join("plugin.so")).expect("hash so"));
    let size = std::fs::metadata(dir.join("plugin.so"))
        .expect("size")
        .len();
    let mut toml = format!(
        "id = \"{id}\"\nversion = \"{version}\"\n[entry]\nkind = \"dl\"\nartifact = \"plugin.so\"\n\
         [[files]]\npath = \"plugin.so\"\nsha256 = \"{hash}\"\nsize = {size}\n"
    );
    if let Some(service) = provides {
        toml.push_str(&format!("\n[[provides]]\nservice = \"{service}\"\n"));
    }
    std::fs::write(dir.join("modukit.toml"), toml).expect("write toml");
}

#[test]
fn bad_plugin_abi_mismatch_is_refused_at_install_before_the_rename() {
    let Some(so) = build_fixture("bad-plugin", "bad_plugin") else {
        return;
    };
    let root = scratch("abi");
    stage(&root.join("staging"), "evil", BAD_ID, "0.1.0", &so, None);
    let inst = BundleInstaller::open(&root, None).expect("installer");
    let err = inst
        .install("evil")
        .expect_err("ABI v999 must never reach the install root");
    assert_eq!(
        err.code(),
        "ABI_MISMATCH",
        "the plan-row gate: bad-plugin proves ABI-mismatch rejection at install"
    );
    assert!(
        !root.join("bundles").join(BAD_ID).exists(),
        "install root untouched"
    );
    assert!(
        root.join("staging/evil").is_dir(),
        "staging kept for inspection"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn liar_manifest_version_fails_cross_check_before_any_call() {
    let Some(so) = build_fixture("example-plugin", "example_plugin") else {
        return;
    };
    let root = scratch("liar");
    // Descriptor reports 0.1.0; the manifest claims 9.9.9 (D7 liar family).
    stage(
        &root.join("staging"),
        "liar",
        EXAMPLE_ID,
        "9.9.9",
        &so,
        None,
    );
    let inst = BundleInstaller::open(&root, None).expect("installer");
    let err = inst
        .install("liar")
        .expect_err("deploy<->self-report disagreement = rejection");
    assert_eq!(
        err.code(),
        "CONTRACT_VIOLATION",
        "D51: CONTRACT_VIOLATION IS the cross-check bucket"
    );
    assert!(!root.join("bundles").join(EXAMPLE_ID).exists());
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn liar_manifest_id_fails_cross_check() {
    let Some(so) = build_fixture("example-plugin", "example_plugin") else {
        return;
    };
    let root = scratch("liarid");
    stage(
        &root.join("staging"),
        "stealer",
        "com.example.modukit.imposter",
        "0.1.0",
        &so,
        None,
    );
    let inst = BundleInstaller::open(&root, None).expect("installer");
    let err = inst
        .install("stealer")
        .expect_err("id is claimed by the artifact, not by the path or the file name");
    assert_eq!(err.code(), "CONTRACT_VIOLATION");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn reverse_dns_grammar_matrix() {
    // Accepted: canonical reverse-DNS (D9/CLAP precedent).
    for ok in [
        "com.example.modukit.demo",
        "org.modukit.snapshot",
        "fleet.telemetry",
        "a.b",
        "com.example.foo-bar_baz",
    ] {
        validate_reverse_dns(ok).unwrap_or_else(|e| panic!("{ok} must pass grammar: {e}"));
        PluginId::try_new(ok.to_string()).expect("PluginId::try_new");
        ServiceId::try_new(ok.to_string()).expect("ServiceId::try_new");
        TopicId::try_new(ok.to_string()).expect("TopicId::try_new");
    }
    // Rejected: the documented rule set (see identity.rs for the grammar).
    let rejects: &[(&str, IdGrammar)] = &[
        ("", IdGrammar::Empty),
        ("com", IdGrammar::SingleLabel),
        ("Com.example", IdGrammar::LeadingUppercase),
        ("com.Example.foo", IdGrammar::LeadingUppercase),
        ("com..example", IdGrammar::EmptyLabel),
        ("com.example.", IdGrammar::EmptyLabel),
        ("1com.example", IdGrammar::LeadingDigit),
        ("com.1x.foo", IdGrammar::LeadingDigit),
        ("com.exAmple.foo", IdGrammar::IllegalChar),
        ("com.exam ple.x", IdGrammar::IllegalChar),
        ("com/example.x", IdGrammar::IllegalChar),
        ("_under.score", IdGrammar::IllegalChar),
    ];
    for (value, want) in rejects {
        let err = validate_reverse_dns(value).expect_err("{value} must be refused");
        assert_eq!(&err, want, "grammar verdict for {value:?}");
    }
    // The infallible v0 constructors stay for kernel-authored ids (M6
    // platform services); grammar policing is the new try_new door.
    assert!(ServiceId::try_new("org.modukit.snapshot".to_string()).is_ok());
    assert!(ServiceId::try_new("Bad.Service".to_string()).is_err());
}

#[test]
fn install_refuses_grammar_violations_before_anything_else() {
    let Some(so) = build_fixture("example-plugin", "example_plugin") else {
        return;
    };
    let root = scratch("grammar");
    stage(
        &root.join("staging"),
        "upper",
        "Com.Example.Foo",
        "0.1.0",
        &so,
        None,
    );
    let inst = BundleInstaller::open(&root, None).expect("installer");
    let err = inst
        .install("upper")
        .expect_err("uppercase-leading id never installs");
    assert_eq!(err.code(), "MANIFEST_INVALID", "the id is a manifest field");
    assert!(
        root.join("staging/upper").is_dir(),
        "refused before any dlopen/rename"
    );

    // Service ids declared by the manifest ride the same validator.
    stage(
        &root.join("staging"),
        "badprov",
        EXAMPLE_ID,
        "0.1.0",
        &so,
        Some("Bad.Service"),
    );
    let err = inst
        .install("badprov")
        .expect_err("uppercase-leading service id never installs");
    assert_eq!(err.code(), "MANIFEST_INVALID");
    let _ = std::fs::remove_dir_all(&root);
}

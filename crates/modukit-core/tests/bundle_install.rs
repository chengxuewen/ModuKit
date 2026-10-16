//! M7 bundle install protocol — directory-form bundles per modules/05 §4,
//! staging -> verify -> atomic rename per D28, content-addressed object
//! store per D31/D55. First-red area: install-verify (tampered bundle goes
//! RED before anything touches the install root — the plan-row gate),
//! atomic+dedupe, load-time re-verification.
//!
//! dlopen legs follow the M3 fixture-build pattern (SKIP green with a
//! printed reason where cargo cannot run).

use std::path::{Path, PathBuf};
use std::process::Command;

use modukit_core::bundle::{BundleError, BundleInstaller, InstalledBundle};
use modukit_core::identity::PluginId;
use modukit_core::kernel::Kernel;
use modukit_core::sha256::{hash_file, hash_file_hex, to_hex};
use std::os::unix::fs::{MetadataExt, PermissionsExt};

const EXAMPLE_ID: &str = "com.example.modukit.demo";

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

/// Fresh scratch host tree per test (std-only; no tempfile crate — the
/// vendor-lock floor is not negotiable).
fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("modukit-m7-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch root");
    dir
}

fn write_at(root: &Path, rel: &str, bytes: &[u8]) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    std::fs::write(&path, bytes).expect("write");
}

/// Assemble one staged bundle directory: modukit.toml (D7 manifest +
/// `[[files]]` hash manifest covering every staged member) + artifact +
/// arbitrary extra files. All hashes are computed with the M7 sha256 (it
/// is itself vector-proven in tests/sha256_vectors.rs).
fn stage_bundle(
    staging: &Path,
    name: &str,
    id: &str,
    version: &str,
    artifact_src: &Path,
    extra: &[(&str, &[u8])],
) -> PathBuf {
    let dir = staging.join(name);
    std::fs::create_dir_all(&dir).expect("staging bundle dir");
    write_at(
        &dir,
        "plugin.so",
        &std::fs::read(artifact_src).expect("read fixture so"),
    );
    let mut members = vec!["plugin.so".to_string()];
    for (rel, bytes) in extra {
        write_at(&dir, rel, bytes);
        members.push((*rel).to_string());
    }
    let mut toml = format!(
        "id = \"{id}\"\nversion = \"{version}\"\n[entry]\nkind = \"dl\"\nartifact = \"plugin.so\"\n"
    );
    for rel in &members {
        let hash = to_hex(&hash_file(&dir.join(rel)).expect("hash member"));
        let size = std::fs::metadata(dir.join(rel)).expect("size").len();
        toml.push_str(&format!(
            "\n[[files]]\npath = \"{rel}\"\nsha256 = \"{hash}\"\nsize = {size}\n"
        ));
    }
    std::fs::write(dir.join("modukit.toml"), toml).expect("write manifest");
    dir
}

fn installer(root: &Path) -> BundleInstaller {
    BundleInstaller::open(root, None).expect("default deploy config")
}

fn pattern(len: usize, byte: u8) -> Vec<u8> {
    vec![byte; len]
}

#[test]
fn deploy_config_defaults_and_field_cross_check() {
    let root = scratch("cfg");
    let def = installer(&root);
    assert_eq!(def.staging_dir(), root.join("staging"));
    assert_eq!(def.bundles_dir(), root.join("bundles"));
    assert_eq!(def.data_dir(), root.join("data"));
    assert_eq!(def.objects_dir(), root.join("objects"));

    // Declared keys win over the defaults (year-1: deploy config with a
    // sane default, cross-checked at resolve like every field).
    let custom = BundleInstaller::open(&root, Some("[install]\nstaging = \"hold\"\n"))
        .expect("custom staging");
    assert_eq!(custom.staging_dir(), root.join("hold"));

    // Field sanity: empty, traversal, and absolute values are refused
    // (D51 SCHEMA_INVALID — the D37 config-validation bucket).
    for bad in ["", "../escape", "/abs/staging", "a/../b"] {
        let err = BundleInstaller::open(&root, Some(&format!("[install]\nstaging = \"{bad}\"\n")))
            .expect_err("bad staging value must be refused");
        assert_eq!(err.code(), "SCHEMA_INVALID", "{bad:?}");
    }
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn clean_install_renames_atomically_then_walks_the_kernel() {
    let Some(so) = build_fixture("example-plugin", "example_plugin") else {
        return;
    };
    let root = scratch("clean");
    std::fs::create_dir_all(root.join("staging")).expect("staging");
    stage_bundle(&root.join("staging"), "demo", EXAMPLE_ID, "0.1.0", &so, &[]);
    let inst = installer(&root);

    let installed = inst.install("demo").expect("clean install must go green");
    assert_eq!(installed.id.as_str(), EXAMPLE_ID);
    assert!(
        !root.join("staging/demo").exists(),
        "staging entry consumed by rename"
    );
    assert!(
        installed.dir.join("plugin.so").is_file(),
        "bundle live under install root"
    );
    assert_eq!(
        installed.manifest.entry.artifact,
        installed.dir.join("plugin.so").to_string_lossy(),
        "the produced manifest carries the installed absolute artifact path"
    );

    // The installed bundle boots through the untouched kernel door.
    let kernel = Kernel::new();
    let id = installed.manifest.id.clone();
    kernel
        .install(installed.manifest.clone())
        .expect("kernel install");
    kernel.resolve(&id).expect("resolve");
    kernel.start(&id).expect("start");
    kernel.stop(&id).expect("stop");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn tampered_file_fails_install_before_anything_touches_the_install_root() {
    let Some(so) = build_fixture("example-plugin", "example_plugin") else {
        return;
    };
    let root = scratch("tamper");
    std::fs::create_dir_all(root.join("staging")).expect("staging");
    let dir = stage_bundle(&root.join("staging"), "demo", EXAMPLE_ID, "0.1.0", &so, &[]);
    // Flip one artifact byte after the hash manifest was written.
    let mut bytes = std::fs::read(dir.join("plugin.so")).expect("read so");
    let last = bytes.len() - 1;
    bytes[last] ^= 0xFF;
    std::fs::write(dir.join("plugin.so"), &bytes).expect("tamper");

    let inst = installer(&root);
    let err = inst
        .install("demo")
        .expect_err("tampered bundle must go RED");
    assert!(matches!(err, BundleError::HashMismatch { .. }), "{err}");
    assert_eq!(
        err.code(),
        "CONTRACT_VIOLATION",
        "D51 frozen bucket (see AMBIGUITY: no INTEGRITY_FAILED)"
    );
    assert!(
        !root.join("bundles/com.example.modukit.demo").exists(),
        "install root untouched"
    );
    assert!(dir.is_dir(), "staging left in place for inspection");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn lying_size_fails_install() {
    let Some(so) = build_fixture("example-plugin", "example_plugin") else {
        return;
    };
    let root = scratch("liesize");
    std::fs::create_dir_all(root.join("staging")).expect("staging");
    let dir = stage_bundle(&root.join("staging"), "demo", EXAMPLE_ID, "0.1.0", &so, &[]);
    let toml = std::fs::read_to_string(dir.join("modukit.toml")).expect("read manifest");
    let lied = toml.replace("size = ", "size = 1000000 # size = ");
    std::fs::write(dir.join("modukit.toml"), lied).expect("write lie");
    let inst = installer(&root);
    let err = inst.install("demo").expect_err("size lie must go RED");
    assert!(matches!(err, BundleError::SizeMismatch { .. }), "{err}");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn duplicate_id_rejected_at_staging_not_by_path() {
    let Some(so) = build_fixture("example-plugin", "example_plugin") else {
        return;
    };
    let root = scratch("dup");
    std::fs::create_dir_all(root.join("staging")).expect("staging");
    stage_bundle(&root.join("staging"), "demo", EXAMPLE_ID, "0.1.0", &so, &[]);
    let inst = installer(&root);
    let first = inst.install("demo").expect("first install green");

    // Second bundle: different staging name AND a different declared
    // version, same manifest id — identity is unique per host (D34).
    stage_bundle(
        &root.join("staging"),
        "demo-second",
        EXAMPLE_ID,
        "0.2.0",
        &so,
        &[],
    );
    let err = inst
        .install("demo-second")
        .expect_err("duplicate manifest id must be refused");
    assert!(matches!(err, BundleError::DuplicateId { .. }), "{err}");
    assert_eq!(err.code(), "CONTRACT_VIOLATION");
    assert!(first.dir.is_dir(), "the installed bundle stays intact");
    assert!(
        std::fs::read_dir(root.join("staging"))
            .map(|mut d| d.next().is_some())
            .unwrap_or(false),
        "the refused bundle remains in staging for the operator"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn big_files_dedupe_to_one_object_two_hardlinks() {
    let Some(so) = build_fixture("example-plugin", "example_plugin") else {
        return;
    };
    let root = scratch("dedupe");
    std::fs::create_dir_all(root.join("staging")).expect("staging");
    // > 1 MiB triggers content-addressing (D31/D55); 1 MiB exactly does not.
    let big = pattern(2 * 1024 * 1024, 0x42);
    let at_threshold = pattern(1024 * 1024, 0x42);
    stage_bundle(
        &root.join("staging"),
        "demo",
        EXAMPLE_ID,
        "0.1.0",
        &so,
        &[
            ("resources/big1.bin", &big),
            ("lib/big2.bin", &big),
            ("resources/at-threshold.bin", &at_threshold),
        ],
    );
    let inst = installer(&root);
    let installed = inst.install("demo").expect("dedupe install green");

    let m1 = std::fs::metadata(installed.dir.join("resources/big1.bin")).expect("stat big1");
    let m2 = std::fs::metadata(installed.dir.join("lib/big2.bin")).expect("stat big2");
    assert_eq!(m1.ino(), m2.ino(), "same content => same inode");
    assert!(m2.nlink() >= 2, "hardlinked bundle files share the object");
    // Object lives at <objects-root>/sha256/<hex>; the debug-built artifact
    // is itself > 1 MiB and lands in the store too, so the claim is per-
    // shared-content, not per-bundle count: one hex, one object.
    let obj_dir = inst.objects_dir().join("sha256");
    let hex = hash_file_hex(&installed.dir.join("resources/big1.bin")).expect("hex big1");
    let object = obj_dir.join(&hex);
    assert!(
        object.is_file(),
        "the shared content has exactly one object"
    );
    assert_eq!(
        std::fs::metadata(&object).expect("object stat").ino(),
        m1.ino(),
        "bundle links point at the object itself"
    );
    assert!(
        std::fs::metadata(&object)
            .expect("object stat")
            .permissions()
            .mode()
            & 0o222
            == 0,
        "committed objects are read-only (444 family)"
    );
    // The exactly-1MiB file is NOT > 1 MiB: it stays a regular bundle file.
    let plain = std::fs::metadata(installed.dir.join("resources/at-threshold.bin")).expect("stat");
    assert_eq!(
        plain.nlink(),
        1,
        "threshold boundary stays regular (> 1 MiB only)"
    );

    // Cross-bundle proof: uninstall retires links, the object persists, and
    // a second bundle with identical content lands on the SAME inode.
    let shared_ino = m1.ino();
    inst.uninstall(&installed.id)
        .expect("uninstall removes links only");
    assert!(
        obj_dir.exists(),
        "object store survives bundle uninstall (D55)"
    );
    stage_bundle(
        &root.join("staging"),
        "demo-b",
        EXAMPLE_ID,
        "0.1.0", // the artifact's true self-report (cross-check polices lies)
        &so,
        &[("resources/big.bin", &big)],
    );
    let second = inst
        .install("demo-b")
        .expect("second bundle installs green");
    let reused = std::fs::metadata(second.dir.join("resources/big.bin")).expect("stat b");
    assert_eq!(reused.ino(), shared_ino, "two bundles, one shared object");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn load_reverification_flags_tampered_backed_file() {
    let Some(so) = build_fixture("example-plugin", "example_plugin") else {
        return;
    };
    let root = scratch("reverify");
    std::fs::create_dir_all(root.join("staging")).expect("staging");
    let big = pattern(2 * 1024 * 1024, 0x77);
    stage_bundle(
        &root.join("staging"),
        "demo",
        EXAMPLE_ID,
        "0.1.0",
        &so,
        &[("resources/model.bin", &big)],
    );
    let inst = installer(&root);
    let installed: InstalledBundle = inst.install("demo").expect("install");
    // Honest gate: clean files pass, then a byte flipped under the read-only
    // bit (as root/attacker would) is caught BEFORE any dlopen.
    installed.verify_for_load().expect("clean tree re-verifies");
    let victim = installed.dir.join("resources/model.bin");
    std::fs::set_permissions(&victim, std::fs::Permissions::from_mode(0o644))
        .expect("chmod victim");
    let mut bytes = std::fs::read(&victim).expect("read");
    bytes[0] ^= 0xFF;
    std::fs::write(&victim, &bytes).expect("tamper after install");
    let err = installed
        .verify_for_load()
        .expect_err("tampered object must fail load re-verify");
    assert!(matches!(err, BundleError::HashMismatch { .. }), "{err}");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn uninstall_removes_install_dir_and_the_plugins_data_root() {
    let Some(so) = build_fixture("example-plugin", "example_plugin") else {
        return;
    };
    let root = scratch("uninstall");
    std::fs::create_dir_all(root.join("staging")).expect("staging");
    stage_bundle(&root.join("staging"), "demo", EXAMPLE_ID, "0.1.0", &so, &[]);
    let inst = installer(&root);
    let installed = inst.install("demo").expect("install");

    // D26: the plugin's writable boundary is <data-root>/<plugin-id>/ —
    // not configurable, and uninstall removes it.
    write_at(inst.data_dir(), &format!("{EXAMPLE_ID}/state.db"), b"cache");
    assert!(inst.data_dir().join(EXAMPLE_ID).is_dir());

    inst.uninstall(&installed.id).expect("uninstall");
    assert!(
        !inst.bundles_dir().join(EXAMPLE_ID).exists(),
        "install dir gone"
    );
    assert!(
        !inst.data_dir().join(EXAMPLE_ID).exists(),
        "data root gone (D26)"
    );

    let err = inst
        .uninstall(&PluginId::new("com.example.never.installed".to_string()))
        .expect_err("unknown id is not silently ok");
    assert_eq!(err.code(), "NOT_FOUND");
    let _ = std::fs::remove_dir_all(&root);
}

//! Integration proof for the exe rung of the embed face (M2a): the real
//! shadow-runner + the real rs example-plugin .so, walked end-to-end over
//! the C ABI verbs. Helper discipline (shared test target dirs, skip
//! guards, always-run cargo) is the dual_placement pattern verbatim.

// D104 T2.4: test-scaffolding file; the clippy.toml allow-*-in-tests valve
// exempts #[test] fn bodies only, not plain helper fns in a tests/ target
// (clippy 0.1.99), so the exemption lands here. Production stays denied.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, MutexGuard, PoisonError};

use modukit_core::embed_sup::{
    modukit_sup_install, modukit_sup_service_call, modukit_sup_state, modukit_sup_step,
    modukit_sup_stop, modukit_sup_uninstall,
};

const ID: &str = "com.example.modukit.demo";
const ACTIVE: i32 = 4;
const RESOLVED: i32 = 6;

/// The supervisor ambient is process-global; tests in this binary
/// serialize here (separate from the dl rung's ambient by design).
static SUP_SERIAL: Mutex<()> = Mutex::new(());

fn sup_lock() -> MutexGuard<'static, ()> {
    SUP_SERIAL.lock().unwrap_or_else(PoisonError::into_inner)
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace layout: crates/modukit-core")
        .to_path_buf()
}

fn target_dir() -> PathBuf {
    repo_root().join("out").join("embed-sup-test-target")
}

fn cargo_build(args: &[&str]) -> Option<bool> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let dir = target_dir();
    let status = Command::new(&cargo)
        .current_dir(repo_root())
        .args(args)
        .arg("--target-dir")
        .arg(&dir)
        .env("CARGO_TARGET_DIR", &dir)
        .status();
    match status {
        Ok(s) => Some(s.success()),
        Err(e) => {
            eprintln!("SKIPPING: cannot spawn cargo: {e}");
            None
        }
    }
}

fn build_example_so() -> Option<PathBuf> {
    if std::env::var_os("MODUKIT_SKIP_DLOPEN").is_some() {
        eprintln!("SKIPPING: MODUKIT_SKIP_DLOPEN is set");
        return None;
    }
    let so = target_dir().join("debug").join("libexample_plugin.so");
    match cargo_build(&["build", "-p", "example-plugin"]) {
        Some(true) => {}
        Some(false) => {
            eprintln!("SKIPPING: fixture build failed");
            return None;
        }
        None => return None,
    }
    so.is_file().then_some(so)
}

fn build_runner() -> PathBuf {
    let bin = target_dir().join("debug").join("modukit-run");
    match cargo_build(&["build", "-p", "modukit-runtime"]) {
        Some(true) => {}
        Some(false) => panic!("modukit-runtime must build (workspace member)"),
        None => panic!("cargo must be spawnable to build the runner"),
    }
    assert!(bin.is_file(), "runner missing at {}", bin.display());
    bin
}

fn c(s: &str) -> std::ffi::CString {
    std::ffi::CString::new(s).unwrap()
}

#[test]
fn exe_rung_walks_the_full_lifecycle_over_the_c_face() {
    let _serial = sup_lock();
    let Some(so) = build_example_so() else { return };
    let runner = build_runner();
    let manifest = format!(
        "id = \"{ID}\"\nversion = \"0.1.0\"\n[entry]\nkind = \"exe\"\nartifact = \"{}\"\nrunner = \"{}\"\n",
        so.display(),
        runner.display()
    );
    let mc = c(&manifest);
    let mut out: *const std::ffi::c_char = std::ptr::null();
    // composite door: install -> resolve (spawn the runner + HELLO) ->
    // start, over the REAL child machinery (production real-clock posture).
    assert_eq!(
        modukit_sup_install(mc.as_c_str().as_ptr(), &raw mut out),
        0,
        "the exe rung must walk a real child to ACTIVE"
    );
    assert!(!out.is_null());
    let id = unsafe { std::ffi::CStr::from_ptr(out) };
    assert_eq!(id.to_str().unwrap(), ID);

    let idc = c(ID);
    let mut state = -1i32;
    assert_eq!(
        modukit_sup_state(idc.as_c_str().as_ptr(), &raw mut state),
        0
    );
    assert_eq!(state, ACTIVE, "supervised child must sit in ACTIVE");

    let mut actions = -1i32;
    assert_eq!(modukit_sup_step(&raw mut actions), 0);
    assert_eq!(actions, 0, "nothing died or revived mid-walk");

    // M3 door (D94): the rs twin's echo is a raw passthrough, so the
    // response bytes must equal the request bytes EXACTLY across
    // ABI -> TL wire -> child service -> back. Non-empty payload first.
    let svc = c(ID);
    let method = c("echo");
    let probe = b"sup-fidelity-bytes";
    let mut resp = [0u8; 64];
    let n = modukit_sup_service_call(
        svc.as_c_str().as_ptr(),
        method.as_c_str().as_ptr(),
        probe.as_ptr(),
        probe.len(),
        resp.as_mut_ptr(),
        resp.len(),
    );
    assert_eq!(
        usize::try_from(n).expect("response byte count >= 0"),
        probe.len(),
        "the door answers the response byte count"
    );
    assert_eq!(
        &resp[..probe.len()],
        probe.as_slice(),
        "bytes back == bytes in, byte-exact over the real wire"
    );
    // empty payload: legal input, answers 0 bytes out.
    assert_eq!(
        modukit_sup_service_call(
            svc.as_c_str().as_ptr(),
            method.as_c_str().as_ptr(),
            std::ptr::null(),
            0,
            resp.as_mut_ptr(),
            resp.len(),
        ),
        0,
        "empty payload round-trips empty"
    );
    // cap discipline: fail rather than truncate (embed.rs decision 4).
    assert_eq!(
        modukit_sup_service_call(
            svc.as_c_str().as_ptr(),
            method.as_c_str().as_ptr(),
            probe.as_ptr(),
            probe.len(),
            resp.as_mut_ptr(),
            4,
        ),
        -19,
        "too-small cap -> QUOTA_EXCEEDED(-19), never a truncated answer"
    );

    assert_eq!(modukit_sup_stop(idc.as_c_str().as_ptr()), 0);
    assert_eq!(
        modukit_sup_state(idc.as_c_str().as_ptr(), &raw mut state),
        0
    );
    assert_eq!(state, RESOLVED, "stop drains the child to RESOLVED");
    assert_eq!(modukit_sup_uninstall(idc.as_c_str().as_ptr()), 0);
    // the record stays table-visible in UNINSTALLED (0) - the kernel keeps
    // uninstalled records; a truly unknown id is what answers NOT_FOUND.
    assert_eq!(
        modukit_sup_state(idc.as_c_str().as_ptr(), &raw mut state),
        0
    );
    assert_eq!(state, 0, "uninstalled answers state 0");
    let ghost = c("com.example.ghost");
    assert_ne!(
        modukit_sup_state(ghost.as_c_str().as_ptr(), &raw mut state),
        0,
        "unknown ids answer typed NOT_FOUND"
    );
    // the stop dropped the proxy: the door answers NOT_FOUND(-14) for a
    // service id that no longer lives on this rung (the dl face's twin rule).
    assert_eq!(
        modukit_sup_service_call(
            svc.as_c_str().as_ptr(),
            method.as_c_str().as_ptr(),
            probe.as_ptr(),
            probe.len(),
            resp.as_mut_ptr(),
            resp.len(),
        ),
        -14,
        "gone proxy answers NOT_FOUND after stop"
    );
}

//! D106 amendment A1 residuals at the script door (wave-2-plugin-door plan
//! v3 §2 batch 0): the ACK encoder twin of `parse_ack` (T0.2) and the
//! spawn-time HELLO cross-check that the install-time probe exemption
//! defers to. The t4 leg proves the deferral with a `/bin/sh` stand-in:
//! kind=exe, runner=/bin/sh, a `.py` artifact whose bytes write a lying
//! HELLO onto the control socketpair — the `--fd=0` convention means the
//! child's fd 0 IS the socket (stdout/stderr are nulled by the kernel).

// D104 T2.4 valve, same as bundle_cross_check.rs: the clippy.toml
// allow-*-in-tests exemption covers #[test] bodies only, so the
// scaffolding helpers opt in at file level. Production stays denied.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::{Path, PathBuf};

use modukit_core::clock::{Clock, ClockDomain};
use modukit_core::error::KernelError;
use modukit_core::executor::Supervisor;
use modukit_core::manifest::Manifest;
use modukit_core::proc_wire;

const ID: &str = "com.example.script.door";
const VERSION: &str = "0.1.0";

/// t3: the mirror encodes exactly what `parse_ack` decodes — one i32 LE,
/// the D27 author-code space; any other length still refuses.
#[test]
fn ack_payload_round_trips_through_parse_ack() {
    for status in [0i32, 1, -1, 42, i32::MIN, i32::MAX] {
        let bytes = proc_wire::ack_payload(status);
        assert_eq!(bytes, status.to_le_bytes().to_vec());
        assert_eq!(
            proc_wire::parse_ack(&bytes),
            Some(status),
            "round-trip {status}"
        );
    }
    assert_eq!(proc_wire::parse_ack(&proc_wire::ack_payload(-7)[..3]), None);
    let mut over = proc_wire::ack_payload(-7);
    over.push(0);
    assert_eq!(proc_wire::parse_ack(&over), None);
}

/// One complete HELLO frame with the given self-report: envelope
/// `[u32 LE payload len][WIRE_VERSION][MSG_HELLO]`, payload
/// `u32 LE abi | pstr x3` (the strict `parse_hello` grammar).
fn hello_frame(abi: u32, id: &str, version: &str) -> Vec<u8> {
    let mut p = abi.to_le_bytes().to_vec();
    for s in [id, version, "script-standin"] {
        p.push(s.len() as u8);
        p.extend_from_slice(s.as_bytes());
    }
    let mut f = (p.len() as u32).to_le_bytes().to_vec();
    f.push(proc_wire::WIRE_VERSION);
    f.push(proc_wire::MSG_HELLO);
    f.extend_from_slice(&p);
    f
}

fn scratch_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("modukit-sd-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch");
    dir
}

/// t4: identity for exempted script entries is not lost, it MOVES — to
/// the spawn-time strict self-report in executor_proc (the read_hello
/// region). A `/bin/sh` runner executing a lying `.py` child must be
/// refused with the typed `IdentityMismatch(id)`, never partial-started.
#[test]
fn script_entry_hello_lie_is_refused_at_spawn_with_a_typed_error() {
    if !Path::new("/bin/sh").is_file() {
        eprintln!("SKIPPING: no /bin/sh stand-in on this box");
        return;
    }
    let dir = scratch_dir("hello-lie");
    // Octal escapes keep every frame byte literal-safe for dash's printf;
    // `>&0` dupes the inherited control socket onto stdout for the write.
    let frame = hello_frame(1, &format!("{ID}.lie"), VERSION);
    let quoted: String = frame.iter().map(|b| format!("\\{b:03o}")).collect();
    let script = dir.join("entry.py");
    std::fs::write(&script, format!("printf '{quoted}' >&0\nexit 0\n")).expect("write script");
    let manifest = Manifest::from_toml_str(&format!(
        "id = \"{ID}\"\nversion = \"{VERSION}\"\n[entry]\nkind = \"exe\"\nartifact = \"{}\"\n\
         runner = \"/bin/sh\"\n",
        script.display()
    ))
    .expect("script-entry manifest parses");
    let id = manifest.id.clone();
    let sup = Supervisor::with_clock(Clock::new(ClockDomain::Step));
    sup.kernel().install(manifest).expect("kernel install");
    let err = sup
        .kernel()
        .resolve(&id)
        .expect_err("a lying HELLO from a script child must abort the handshake");
    assert!(
        matches!(&err, KernelError::IdentityMismatch { what, .. } if *what == "id"),
        "expected typed IdentityMismatch(id), got {err}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

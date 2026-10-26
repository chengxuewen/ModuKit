//! Instrumented reproduction test for the `wire-golden` bin (D106 batch 1,
//! wave-2-plugin-door-plan.md T1.2, delta-review F5): a fresh run of the
//! bin must be byte-identical to the checked-in goldens under
//! `contracts/golden/wire-v1/`. The shell parity leg alone produces no
//! profraw - this test is the coverage path for the bin in the codegen
//! region (coverage-boundary.sh's src/[a-z0-9_]+ glob).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn checked_in() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/golden/wire-v1")
}

#[test]
fn wire_golden_bin_reproduces_the_checked_in_frames() {
    let gold = checked_in();
    let out =
        std::env::temp_dir().join(format!("modukit-wire-golden-repro-{}", std::process::id()));
    let _ = fs::remove_dir_all(&out);

    let status = Command::new(env!("CARGO_BIN_EXE_wire-golden"))
        .args(["--out"])
        .arg(&out)
        .status()
        .expect("wire-golden bin runs");
    assert!(status.success(), "wire-golden exited {status}");

    // Exactly 14 frames + index.txt, byte-for-byte against the shelf.
    let mut names: Vec<String> = fs::read_dir(&out)
        .expect("bin output dir readable")
        .map(|e| {
            e.expect("dir entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.sort();
    assert_eq!(
        names.len(),
        15,
        "bin emitted {names:?}, expected 14 frames + index"
    );
    assert!(
        names.iter().any(|n| n == "index.txt"),
        "no index.txt emitted"
    );
    for name in &names {
        let fresh = fs::read(out.join(name)).expect("fresh golden readable");
        let checked = fs::read(gold.join(name))
            .unwrap_or_else(|e| panic!("checked-in golden {name} unreadable: {e}"));
        assert_eq!(fresh, checked, "{name} drifted from the checked-in golden");
    }
    assert_eq!(names.iter().filter(|n| n.ends_with(".frame")).count(), 14);

    let _ = fs::remove_dir_all(&out);
}

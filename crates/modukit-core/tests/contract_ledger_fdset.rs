//! M7 acceptance for the M6 deferral "LedgerDump = module ledger (fdset
//! equality rides M7/M8)" (plan row M6). The Kernel grows a `set_contract_ledger`
//! boot seam; the checked-in contract ledger bytes (protoc fdset, D12) must
//! read back from `LedgerDump` BYTE-EQUAL when the host is booted with them.
//! First-red area: fdset-equality — written before the seam existed.

// D104 T2.4: test-scaffolding file; the clippy.toml allow-*-in-tests valve
// exempts #[test] fn bodies only, not plain helper fns in a tests/ target
// (clippy 0.1.99), so the exemption lands here. Production stays denied.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::{Path, PathBuf};

use modukit_core::identity::ServiceId;
use modukit_core::introspect::LedgerDump;
use modukit_core::kernel::Kernel;
use modukit_core::testhost::TestHost;

fn checked_in_ledger() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("repo root")
        .join("contracts/org.modukit.runtime.proto.fdset")
}

fn acquire_ledger(kernel: &Kernel) -> LedgerDump {
    let r = kernel
        .registry()
        .acquire::<LedgerDump>()
        .expect("org.modukit.ledger_dump is a registered service");
    let inner = r.clone_inner();
    kernel.registry().release(&r).expect("acquire/release pair");
    let _ = ServiceId::new("org.modukit.ledger_dump".to_string());
    inner
}

#[test]
fn fdset_bytes_booted_into_the_kernel_read_back_byte_equal() {
    let bytes =
        std::fs::read(checked_in_ledger()).expect("checked-in fdset must exist (M4 ledger)");
    assert!(!bytes.is_empty(), "the contract ledger is real bytes");

    let host = TestHost::new();
    // Un-booted honesty: no bytes claimed.
    assert!(
        acquire_ledger(host.kernel()).contract_ledger().is_empty(),
        "a kernel booted without the seam reports an empty contract ledger"
    );

    host.kernel().set_contract_ledger(bytes.clone());
    let back = acquire_ledger(host.kernel()).contract_ledger();
    assert_eq!(
        back, bytes,
        "LedgerDump reads the contract ledger byte-equal"
    );

    // The M6 module-ledger half of the dump keeps its frozen shape: a
    // testhost that booted no plugins still answers zero loaded modules.
    let dump = acquire_ledger(host.kernel()).dump();
    assert!(
        dump.iter().all(|e| !e.loaded),
        "existing dump() claims preserved alongside the fdset seam"
    );
}

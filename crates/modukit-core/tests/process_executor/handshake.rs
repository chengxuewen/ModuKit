//! Handshake aborts at the door (ABI lie, id lie, foreign wire version) and
//! the frame-codec version stamping behind them.

use super::*;

#[test]
fn handshake_abort_on_abi_lie_lands_failed_edge_no_panic() {
    // Arrange
    let Some(dir) = build_standins("abi-lie") else {
        return;
    };
    let sup = step_supervisor();
    let manifest = parse_manifest(&dir.join("standin-abi-liar"), "");
    let id = manifest.id.clone();
    sup.kernel().install(manifest).expect("install");

    // Act: resolve spawns the liar; the ABI v1 recheck must abort it.
    let err = sup
        .kernel()
        .resolve(&id)
        .expect_err("abi 999 must never be accepted silently");

    // Assert: typed failure, FAILED edge, the bad-plugin doorway counted.
    assert!(
        matches!(err, KernelError::AbiMismatch { found, .. } if found == 999),
        "expected typed AbiMismatch(found=999), got {err}"
    );
    assert_eq!(
        sup.kernel().state(&id).expect("still tracked"),
        PluginState::Failed
    );
    assert_eq!(sup.kernel().counters().load_fail(), 1);
    assert_eq!(
        sup.poll(),
        Vec::<String>::new(),
        "the liar was reaped at load, no phantom"
    );
    assert_no_zombie_children();
    // Reaching here without a crash IS the second half of the assertion.
}

#[test]
fn handshake_abort_on_id_lie_is_rejected_by_strict_self_report() {
    // Arrange
    let Some(dir) = build_standins("id-lie") else {
        return;
    };
    let sup = step_supervisor();
    let manifest = parse_manifest(&dir.join("standin-id-liar"), "");
    let id = manifest.id.clone();
    sup.kernel().install(manifest).expect("install");

    // Act
    let err = sup
        .kernel()
        .resolve(&id)
        .expect_err("a lying self-report must abort the handshake (D7 family)");

    // Assert
    assert!(
        matches!(&err, KernelError::IdentityMismatch { what, .. } if *what == "id"),
        "expected typed IdentityMismatch(id), got {err}"
    );
    assert_eq!(
        sup.kernel().state(&id).expect("still tracked"),
        PluginState::Failed
    );
}

/// S2-M3 frame versioning, codec doorways: the writer stamps the version
/// byte, the reader reports a foreign version as data for the caller to
/// type (parse, don't validate — the kernel doors answer with
/// `KernelError::WireVersionMismatch`, the codec itself stays io-level).
// spec: WIRE-01
#[test]
fn frame_codec_stamps_the_wire_version_and_surfaces_a_foreign_one() {
    use modukit_core::proc_wire;
    use std::os::unix::net::UnixStream;
    let (mut a, mut b) = UnixStream::pair().expect("socketpair");

    // One honest round trip: the raw header is len(4) | ver=1 | kind.
    proc_wire::write_frame(&mut a, proc_wire::MSG_ACK, b"xy").expect("write");
    let mut raw = [0u8; 8];
    std::io::Read::read_exact(&mut b, &mut raw).expect("raw header+payload");
    assert_eq!(
        &raw,
        &[
            2,
            0,
            0,
            0,
            proc_wire::WIRE_VERSION,
            proc_wire::MSG_ACK,
            b'x',
            b'y'
        ],
        "envelope = len | ver | type | payload"
    );

    // A hand-framed foreign ver arrives intact: read_frame reports it,
    // it does not reinterpret or silently drop (the caller types it).
    let foreign = [4u8, 0, 0, 0, 9, 1, 0, 0, 0, 0]; // len 4, ver 9, kind 1
    std::io::Write::write_all(&mut b, &foreign).expect("write foreign");
    let (ver, kind, payload) = proc_wire::read_frame(&mut a).expect("read foreign");
    assert_eq!((ver, kind, payload.as_slice()), (9, 1, &[0u8, 0, 0, 0][..]));
}

/// The parent refuses a foreign envelope version on the HELLO with the
/// typed protocol error — never a decode, never a supervised run.
#[test]
fn handshake_abort_on_foreign_wire_version_is_typed_version_mismatch() {
    let Some(dir) = build_standins("ver-lie") else {
        return;
    };
    let sup = step_supervisor();
    let manifest = parse_manifest(&dir.join("standin-ver-liar"), "");
    let id = manifest.id.clone();
    sup.kernel().install(manifest).expect("install");

    let err = sup
        .kernel()
        .resolve(&id)
        .expect_err("a v9 frame must never be decoded as v1");
    assert!(
        matches!(
            err,
            KernelError::WireVersionMismatch {
                expected: 1,
                found: 9
            }
        ),
        "{err}"
    );
    assert_eq!(
        sup.kernel().state(&id).expect("still tracked"),
        PluginState::Failed
    );
    assert_eq!(sup.kernel().counters().load_fail(), 1);
    assert_eq!(
        sup.poll(),
        Vec::<String>::new(),
        "the mismatched child was reaped at load, no phantom"
    );
    assert_no_zombie_children();
}

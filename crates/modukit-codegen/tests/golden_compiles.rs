//! The golden corpus must be *compilable* Rust, not just stable text:
//! each generated file is include!d here (a broken emitter shows up as a
//! compile error of this test target, before the byte-compare even runs).

// Generated artifacts - formatting is the generator's business, not
// rustfmt's (byte-compare is the golden gate).
#[rustfmt::skip]
#[allow(dead_code)]
#[path = "../testdata/golden/org_modukit_runtime.rs"]
mod org_modukit_runtime;

// Generated artifacts - formatting is the generator's business, not
// rustfmt's (byte-compare is the golden gate).
#[rustfmt::skip]
#[allow(dead_code)]
#[path = "../testdata/golden/com_modukit_demo.rs"]
mod com_modukit_demo;

#[rustfmt::skip]
#[allow(dead_code)]
#[path = "../testdata/golden/com_modukit_demo.abi.rs"]
mod com_modukit_demo_abi;

#[test]
fn runtime_ledger_types_are_usable() {
    use org_modukit_runtime::{Error, Hint, KernelCode, KeyValue};

    let err = Error {
        code: "ABI_MISMATCH".to_string(),
        domain: "kernel".to_string(),
        hint: Hint::from_i32(2),
        kv: vec![KeyValue {
            key: "expected".to_string(),
            value: b"1".to_vec(),
        }],
        ..Default::default()
    };
    assert_eq!(err.hint, Hint::RETRY_AFTER);
    assert_eq!(
        KernelCode::from_i32(999),
        KernelCode::KERNEL_CODE_UNSPECIFIED
    );
    assert_eq!(KernelCode::from_i32(9), KernelCode::ABI_MISMATCH);
    assert_eq!(KernelCode::ABI_MISMATCH as i32, 9);
}

#[test]
fn demo_dispatch_and_trait_are_usable() {
    use com_modukit_demo::{
        DemoServiceAuthor, DemoServiceDispatch, EchoReply, EchoRequest, MetaReply, MetaRequest,
        SERVICE_ID,
    };

    struct Hand; // the impl side is by hand - D13 doctrine 2
    impl DemoServiceAuthor for Hand {
        fn echo(&self, request: EchoRequest) -> Result<EchoReply, com_modukit_demo::Error> {
            Ok(EchoReply {
                payload: request.payload,
                ..Default::default()
            })
        }
        fn describe(&self, _request: MetaRequest) -> Result<MetaReply, com_modukit_demo::Error> {
            Ok(MetaReply {
                service_id: SERVICE_ID.to_string(),
                method_count: 2,
                ..Default::default()
            })
        }
    }

    assert_eq!(SERVICE_ID, "com.modukit.demo.DemoService");
    let call = DemoServiceDispatch::Echo(EchoRequest {
        payload: b"ping".to_vec(),
    });
    assert_eq!(call.method_name(), "echo");
    let reply = Hand
        .echo(EchoRequest {
            payload: b"ping".to_vec(),
        })
        .expect("hand impl cannot fail here");
    assert_eq!(reply.payload, b"ping");
    assert_eq!(
        Hand.describe(MetaRequest::default())
            .expect("ok")
            .service_id,
        "com.modukit.demo.DemoService"
    );
}

#[test]
fn abi_shade_keeps_the_handshake_guard_shape() {
    use com_modukit_demo_abi::DemoServiceVtable;
    use std::mem::size_of;

    assert_eq!(
        DemoServiceVtable::expected_size() as usize,
        size_of::<DemoServiceVtable>()
    );
    assert_eq!(DemoServiceVtable::ABI_VERSION, 1);
    // Two methods + the two guard fields: 4 pointers on 64-bit after the
    // u32 pair; the guard fields leading the table is the 04 §3 shape rule.
    let table = DemoServiceVtable {
        abi_version: DemoServiceVtable::ABI_VERSION,
        size: DemoServiceVtable::expected_size(),
        echo: None,
        describe: None,
    };
    assert_eq!(table.abi_version, 1);
}

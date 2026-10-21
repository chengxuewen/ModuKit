use std::time::Duration;

use modukit_core::clock::{ClockDomain, DueJob, JobToken};
use modukit_core::proc_wire::{
    Call, DIAL_STATUS_REFUSED_REAL, DueTurn, FRAME_MAX, Hello, MSG_ACK, MSG_CALL, MSG_DIAL,
    MSG_DUE, MSG_HELLO, MSG_REGISTER, MSG_RESULT, MSG_START, MSG_STOP, ServiceRegistration,
    ServiceResult, WIRE_VERSION, call_payload, dial_payload, due_payload, hello_payload, parse_ack,
    parse_call, parse_dial, parse_due, parse_hello, parse_register, parse_result, register_payload,
    result_payload,
};

// ---------------------------------------------------------------------------
// proc_wire — frame-kind constants and envelope version (the frozen grammar).
// ---------------------------------------------------------------------------

#[test]
fn wire_constants_are_the_frozen_frame_grammar() {
    assert_eq!(WIRE_VERSION, 1);
    assert_eq!(MSG_HELLO, 1);
    assert_eq!(MSG_START, 2);
    assert_eq!(MSG_STOP, 3);
    assert_eq!(MSG_ACK, 4);
    assert_eq!(MSG_REGISTER, 5);
    assert_eq!(MSG_CALL, 6);
    assert_eq!(MSG_RESULT, 7);
    assert_eq!(MSG_DIAL, 8);
    assert_eq!(MSG_DUE, 9);
    assert_eq!(DIAL_STATUS_REFUSED_REAL, -2);
    assert_eq!(FRAME_MAX, 64 * 1024);
}

// ---------------------------------------------------------------------------
// proc_wire — HELLO codec: roundtrip + every refusal arm reachable without a
// socket (truncated header, truncated pstr, trailing bytes, non-UTF-8).
// ---------------------------------------------------------------------------

#[test]
fn hello_roundtrip_preserves_all_fields() {
    let hello = Hello {
        abi: 1,
        id: "com.example.plugin".into(),
        version: "1.2.3".into(),
        name: "Example".into(),
        caps: 0,
    };
    let parsed = parse_hello(&hello_payload(&hello)).expect("roundtrip must parse");
    assert_eq!(parsed, hello);
}

#[test]
fn hello_parse_refuses_truncated_and_lying_frames() {
    let hello = Hello {
        abi: 1,
        id: "com.example.plugin".into(),
        version: "1.2.3".into(),
        name: "Example".into(),
        caps: 0,
    };
    let full = hello_payload(&hello);

    // Shorter than the u32 abi header.
    assert_eq!(parse_hello(&full[..3]), None);
    // The last pstr declares more bytes than the frame carries.
    assert_eq!(parse_hello(&full[..full.len() - 1]), None);
    // Trailing bytes after the declared name: a lie, refused (strict tail).
    let mut trailing = full.clone();
    trailing.push(0);
    assert_eq!(parse_hello(&trailing), None);
    // Non-UTF-8 id bytes refuse the frame (silence and lies both disagree).
    let mut lying = full.clone();
    assert_eq!(lying[4] as usize, "com.example.plugin".len());
    lying[5] = 0xFF;
    assert_eq!(parse_hello(&lying), None);
}

// ---------------------------------------------------------------------------
// proc_wire — ACK codec: exactly one i32 LE or nothing.
// ---------------------------------------------------------------------------

#[test]
fn ack_codec_takes_exactly_one_i32_le() {
    assert_eq!(parse_ack(&(-7i32).to_le_bytes()), Some(-7));
    assert_eq!(parse_ack(&0i32.to_le_bytes()), Some(0));
    assert_eq!(parse_ack(&[0, 0, 0]), None);
    assert_eq!(parse_ack(&[0, 0, 0, 0, 0]), None);
}

// ---------------------------------------------------------------------------
// proc_wire — REGISTER codec.
// ---------------------------------------------------------------------------

#[test]
fn register_roundtrip_preserves_service_and_method_list() {
    let reg = ServiceRegistration {
        service_id: "com.example.svc".into(),
        methods: vec!["paint".into(), "resize".into(), "".into()],
    };
    let parsed = parse_register(&register_payload(&reg)).expect("roundtrip must parse");
    assert_eq!(parsed, reg);
}

#[test]
fn register_parse_refuses_malformed_frames() {
    // Empty payload: no service id at all.
    assert_eq!(parse_register(&[]), None);
    // Service id pstr declares more bytes than present.
    assert_eq!(parse_register(&[9, b'a']), None);
    // Count byte promises two methods but the frame carries none.
    let mut short = register_payload(&ServiceRegistration {
        service_id: "s".into(),
        methods: vec![],
    });
    short[1 + "s".len()] = 2;
    assert_eq!(parse_register(&short), None);
    // Trailing garbage after the last method is refused.
    let mut trailing = register_payload(&ServiceRegistration {
        service_id: "s".into(),
        methods: vec!["m".into()],
    });
    trailing.push(0xAB);
    assert_eq!(parse_register(&trailing), None);
    // Non-UTF-8 method bytes refuse.
    let mut lying = register_payload(&ServiceRegistration {
        service_id: "s".into(),
        methods: vec!["m".into()],
    });
    let m_pos = 1 + "s".len() + 1;
    lying[m_pos + 1] = 0xFF;
    assert_eq!(parse_register(&lying), None);
}

// ---------------------------------------------------------------------------
// proc_wire — CALL codec.
// ---------------------------------------------------------------------------

#[test]
fn call_roundtrip_preserves_seq_method_and_input() {
    let call = call_payload(9, "paint", &[1, 2, 3]);
    let parsed = parse_call(&call).expect("roundtrip must parse");
    assert_eq!(
        parsed,
        Call {
            seq: 9,
            method: "paint".into(),
            input: vec![1, 2, 3],
        }
    );
    // Empty input is a legal shape.
    let empty = parse_call(&call_payload(0, "m", &[])).expect("empty input parses");
    assert!(empty.input.is_empty());
}

#[test]
fn call_parse_refuses_malformed_frames() {
    // Shorter than the seq header.
    assert_eq!(parse_call(&[1, 2, 3]), None);
    // Method pstr truncated.
    assert_eq!(parse_call(&[0, 0, 0, 0, 5, b'a']), None);
    // Input-length prefix truncated (not even 4 bytes after the method).
    let mut p = call_payload(1, "m", &[7, 8]);
    let n = p.len();
    p.truncate(n - 2);
    assert_eq!(parse_call(&p), None);
    // Declared input length longer than the carried bytes (len lives at
    // offset 6: 4 seq + 1+1 method pstr).
    let mut p = call_payload(1, "m", &[7]);
    p[6] = 2;
    assert_eq!(parse_call(&p), None);
    // Declared input length shorter than the carried bytes.
    let mut p = call_payload(1, "m", &[7, 8]);
    p[6] = 0;
    assert_eq!(parse_call(&p), None);
}

// ---------------------------------------------------------------------------
// proc_wire — RESULT codec.
// ---------------------------------------------------------------------------

#[test]
fn result_roundtrip_preserves_seq_status_and_output() {
    for (seq, status, out) in [(1u32, 0i32, vec![9u8, 8]), (u32::MAX, i32::MIN, vec![])] {
        let parsed = parse_result(&result_payload(seq, status, &out)).expect("roundtrip");
        assert_eq!(
            parsed,
            ServiceResult {
                seq,
                status,
                output: out,
            }
        );
    }
}

#[test]
fn result_parse_refuses_malformed_frames() {
    // Shorter than seq+status.
    assert_eq!(parse_result(&[0; 7]), None);
    // Length prefix truncated.
    let mut p = result_payload(1, 0, &[5]);
    p.truncate(p.len() - 1);
    assert_eq!(parse_result(&p), None);
    // Trailing bytes past the declared output length are refused.
    let mut p = result_payload(1, 0, &[5]);
    p.push(0);
    assert_eq!(parse_result(&p), None);
    // Declared length disagrees downward (len lives at offset 8: 4 seq +
    // 4 status).
    let mut p = result_payload(1, 0, &[5, 6]);
    p[8] = 0;
    assert_eq!(parse_result(&p), None);
    // Negative status passes through verbatim (D27 author code space).
    assert_eq!(
        parse_result(&result_payload(3, -42, &[])).map(|r| r.status),
        Some(-42)
    );
}

// ---------------------------------------------------------------------------
// proc_wire — DIAL codec: u64 LE micros with named saturation.
// ---------------------------------------------------------------------------

#[test]
fn dial_roundtrip_and_saturation_at_u64_max() {
    assert_eq!(
        parse_dial(&dial_payload(Duration::from_micros(1500))),
        Some(Duration::from_micros(1500))
    );
    // Named saturation: a dial longer than u64 micros clamps, never wraps.
    assert_eq!(
        parse_dial(&dial_payload(Duration::from_secs(u64::MAX))),
        Some(Duration::from_micros(u64::MAX))
    );
    assert_eq!(parse_dial(&[0; 7]), None);
    assert_eq!(parse_dial(&[0; 9]), None);
}

// ---------------------------------------------------------------------------
// proc_wire — DUE codec: zero- and multi-job turns + every refusal arm.
// ---------------------------------------------------------------------------

#[test]
fn due_roundtrip_preserves_status_stamp_and_job_list() {
    let turn = DueTurn {
        status: 0,
        fired_at_ns: 1_234_567_890,
        jobs: vec![
            DueJob {
                token: JobToken::at(7),
                job: "tick".into(),
            },
            DueJob {
                token: JobToken::at(u64::MAX),
                job: "".into(),
            },
        ],
    };
    let parsed = parse_due(&due_payload(&turn)).expect("roundtrip must parse");
    assert_eq!(parsed, turn);
    let empty = DueTurn {
        status: -2,
        fired_at_ns: 0,
        jobs: vec![],
    };
    assert_eq!(parse_due(&due_payload(&empty)), Some(empty));
}

#[test]
fn due_parse_refuses_malformed_frames() {
    // Below the fixed 14-byte header.
    assert_eq!(parse_due(&[0; 13]), None);
    // Count promises one job but the token bytes are missing.
    let mut p = due_payload(&DueTurn {
        status: 0,
        fired_at_ns: 0,
        jobs: vec![],
    });
    p[12] = 1;
    assert_eq!(parse_due(&p), None);

    let one_job = || {
        due_payload(&DueTurn {
            status: 0,
            fired_at_ns: 0,
            jobs: vec![DueJob {
                token: JobToken::at(1),
                job: "job".into(),
            }],
        })
    };
    // Job pstr declares more bytes than remain (length byte at offset
    // 14 header + 8 token = 22).
    let mut p = one_job();
    p[22] = 9;
    assert_eq!(parse_due(&p), None);
    // Non-UTF-8 job label refuses the whole frame.
    let mut p = one_job();
    p[14 + 8 + 1] = 0xFF;
    assert_eq!(parse_due(&p), None);
    // Trailing garbage after the last job is refused (strict tail).
    let mut p = one_job();
    p.push(0);
    assert_eq!(parse_due(&p), None);
    // D74 domain discipline the due mapping rides on.
    assert_ne!(ClockDomain::Real, ClockDomain::Step);
}

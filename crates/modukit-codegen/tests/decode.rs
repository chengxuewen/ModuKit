//! Subset-decoder behavior, tested through the public entry point:
//! decode-and-skip on unknown fields (D12 rule 2) and the loud failures on
//! missing option stamps / unnumbered fields / missing sentinels.

use modukit_codegen::descriptor::{EXT_SERVICE_ID, decode_file_descriptor_set};
use modukit_codegen::{CodegenError, generate};
use std::path::PathBuf;

fn tag(number: u32, wire: u32) -> Vec<u8> {
    let mut v = Vec::new();
    let mut t = (number << 3) | wire;
    loop {
        let byte = (t & 0x7f) as u8;
        t >>= 7;
        v.push(byte | if t == 0 { 0 } else { 0x80 });
        if t == 0 {
            break;
        }
    }
    v
}

fn ld(number: u32, payload: &[u8]) -> Vec<u8> {
    let mut v = tag(number, 2);
    let mut n = payload.len();
    loop {
        let byte = (n & 0x7f) as u8;
        n >>= 7;
        v.push(byte | if n == 0 { 0 } else { 0x80 });
        if n == 0 {
            break;
        }
    }
    v.extend_from_slice(payload);
    v
}

fn varint_field(number: u32, value: u64) -> Vec<u8> {
    let mut v = tag(number, 0);
    let mut n = value;
    loop {
        let byte = (n & 0x7f) as u8;
        n >>= 7;
        v.push(byte | if n == 0 { 0 } else { 0x80 });
        if n == 0 {
            break;
        }
    }
    v
}

#[test]
fn decodes_minimal_file_with_service_and_option() {
    // FileDescriptorSet { file: FileDescriptorProto { name=1, package=2,
    // service=6 { name=1, options=3 { 50000: "a.b.C" } } } }
    let mut svc = ld(1, b"Greeter");
    svc.extend(ld(3, &ld(EXT_SERVICE_ID, b"com.acme.greeter")));
    let mut file = ld(1, b"x.proto");
    file.extend(ld(2, b"com.acme"));
    file.extend(ld(6, &svc));
    let set = ld(1, &file);
    let files = decode_file_descriptor_set(&set).expect("decode");
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].services[0].service_id, "com.acme.greeter");
}

#[test]
fn unknown_fields_decode_and_skip() {
    // D12 rule 2: an unknown field number (here 4242 in the set, 77 in
    // the service, plus an unknown fixed64) must decode and be ignored.
    let mut svc = ld(1, b"Greeter");
    svc.extend(varint_field(77, 9));
    svc.extend(ld(3, &ld(EXT_SERVICE_ID, b"com.acme.greeter")));
    let mut file = ld(1, b"x.proto");
    file.extend(ld(6, &svc));
    let mut set = ld(1, &file);
    set.extend(ld(4242, b"junk nobody models"));
    let files = decode_file_descriptor_set(&set).expect("unknown fields skipped");
    assert_eq!(files[0].services[0].name, "Greeter");
}

#[test]
fn missing_service_id_option_is_a_typed_error() {
    let mut svc = ld(1, b"Greeter");
    svc.extend(ld(3, &ld(1234, b"irrelevant")));
    let mut file = ld(2, b"com.acme");
    file.extend(ld(6, &svc));
    let set = ld(1, &file);
    let err = decode_file_descriptor_set(&set).expect_err("must fail loudly");
    assert!(
        matches!(err, CodegenError::ServiceIdOption(_)),
        "wrong error shape: {err}"
    );
}

#[test]
fn unnumbered_field_is_a_typed_error() {
    // FieldDescriptorProto with name but no number tag (hand-built).
    let mut field = ld(1, b"payload");
    field.extend(varint_field(4, 1)); // label optional
    field.extend(varint_field(5, 12)); // type bytes
    let mut msg = ld(1, b"Req");
    msg.extend(ld(2, &field));
    let mut file = ld(2, b"com.acme");
    file.extend(ld(4, &msg));
    let set = ld(1, &file);
    let err = decode_file_descriptor_set(&set).expect_err("must fail loudly");
    assert!(matches!(err, CodegenError::FieldRule(_)), "got: {err}");
}

#[test]
fn enum_without_sentinel_is_a_typed_error() {
    let mut e = ld(1, b"Status");
    let mut v = ld(1, b"OK");
    v.extend(varint_field(2, 1));
    e.extend(ld(2, &v));
    let mut file = ld(2, b"com.acme");
    file.extend(ld(5, &e));
    let set = ld(1, &file);
    let err = decode_file_descriptor_set(&set).expect_err("must fail loudly");
    assert!(
        matches!(err, CodegenError::MissingSentinel(_)),
        "got: {err}"
    );
}

// ---------------------------------------------------------------------------
// D84 coverage-debt lane (2026-10-19): the decoder's error arms. Every test
// drives REAL decode behavior through the public door and asserts the typed
// failure shape - the same charter as the tests above, on the arms the
// M5a/M3S-G runs showed uncovered.
// ---------------------------------------------------------------------------

/// A field whose tag promises varint (wire 0) but the buffer ends mid-value:
/// `read_varint` must report the truncation, not loop or default.
#[test]
fn truncated_varint_payload_is_a_wire_error() {
    // FileDescriptorSet { file: FileDescriptorProto { name: <varint tag with no bytes> } }
    let mut file = ld(1, b"x.proto");
    file.extend(tag(2, 0)); // package field, wire type varint, then EOF
    let set = ld(1, &file);
    let err = decode_file_descriptor_set(&set).expect_err("truncated varint must fail");
    assert!(matches!(err, CodegenError::Wire(_)), "got: {err}");
}

/// A length-delimited field whose declared length runs past the buffer end:
/// the reader must refuse instead of panicking or silently short-reading.
#[test]
fn length_delimited_field_declaring_more_bytes_than_exist_is_a_wire_error() {
    let mut file = ld(1, b"x.proto");
    file.extend(tag(2, 2)); // package field, wire type len ...
    file.push(200); // ... declaring 200 bytes, delivering 0
    let set = ld(1, &file);
    let err = decode_file_descriptor_set(&set).expect_err("overlong length must fail");
    assert!(matches!(err, CodegenError::Wire(_)), "got: {err}");
}

/// A group wire type (3) anywhere in the walk is deprecated and must be
/// rejected with the typed `GroupUnsupported` carrying the field number.
#[test]
fn group_wire_type_is_rejected_with_the_field_number() {
    let mut file = ld(1, b"x.proto");
    file.extend(ld(2, b"com.acme"));
    file.extend(tag(9, 3)); // a group-opening tag at field 9
    let set = ld(1, &file);
    let err = decode_file_descriptor_set(&set).expect_err("groups must fail loudly");
    assert!(
        matches!(err, CodegenError::GroupUnsupported(9)),
        "got: {err}"
    );
}

/// Field number 0 is illegal on the protobuf wire; the cursor must reject
/// the tag before any value is consumed.
#[test]
fn field_number_zero_is_a_wire_error() {
    let set = ld(1, b"junk");
    // Rewrite: a top-level field-0 tag (number 0 << 3 | wire 2 = tag byte 2)
    let mut evil = tag(0, 2);
    evil.extend_from_slice(b"x");
    let mut set2 = set;
    set2.extend(evil);
    let err = decode_file_descriptor_set(&set2).expect_err("field 0 must fail");
    assert!(matches!(err, CodegenError::Wire(_)), "got: {err}");
}

/// A field whose label varint is neither optional (1), required (2) nor
/// repeated (3) must fail with the typed unknown-label wire error.
#[test]
fn unknown_field_label_is_a_wire_error() {
    let mut field = ld(1, b"payload");
    field.extend(varint_field(3, 7)); // number 7
    field.extend(varint_field(4, 9)); // label 9: unknown
    let mut msg = ld(1, b"Req");
    msg.extend(ld(2, &field));
    let mut file = ld(2, b"com.acme");
    file.extend(ld(4, &msg));
    let set = ld(1, &file);
    let err = decode_file_descriptor_set(&set).expect_err("label 9 must fail");
    assert!(matches!(err, CodegenError::Wire(_)), "got: {err}");
}

/// A required label is refused with the FieldRule error naming the field.
#[test]
fn required_label_is_refused_naming_the_field() {
    let mut field = ld(1, b"payload");
    field.extend(varint_field(3, 7));
    field.extend(varint_field(4, 2)); // label required
    let mut msg = ld(1, b"Req");
    msg.extend(ld(2, &field));
    let mut file = ld(2, b"com.acme");
    file.extend(ld(4, &msg));
    let set = ld(1, &file);
    let err = decode_file_descriptor_set(&set).expect_err("required must be refused");
    match err {
        CodegenError::FieldRule(m) => assert!(m.contains("payload"), "must name the field: {m}"),
        other => panic!("wrong error shape: {other}"),
    }
}

/// A field declaring a oneof (FieldDescriptorProto.oneof_index = 9) is
/// refused at v0 (D14 deferral), with the field name in the message.
#[test]
fn oneof_field_is_refused_at_v0() {
    let mut field = ld(1, b"choice");
    field.extend(varint_field(3, 1));
    field.extend(varint_field(4, 1));
    field.extend(varint_field(5, 5)); // type int32
    field.extend(varint_field(9, 0)); // oneof_index present
    let mut msg = ld(1, b"Req");
    msg.extend(ld(2, &field));
    let mut file = ld(2, b"com.acme");
    file.extend(ld(4, &msg));
    let set = ld(1, &file);
    let err = decode_file_descriptor_set(&set).expect_err("oneof must be refused");
    match err {
        CodegenError::Wire(m) => assert!(m.contains("choice"), "must name the field: {m}"),
        other => panic!("wrong error shape: {other}"),
    }
}

/// A field number that overflows u32 (DescriptorProto.field.number) must
/// fail with the overflow wire error, not wrap.
#[test]
fn field_number_over_u32_is_a_wire_error() {
    // varint 0x1_0000_0000 (2^32) cannot fit u32.
    let mut field = ld(1, b"big");
    field.extend(varint_field(3, 1 << 32));
    let mut msg = ld(1, b"Req");
    msg.extend(ld(2, &field));
    let mut file = ld(2, b"com.acme");
    file.extend(ld(4, &msg));
    let set = ld(1, &file);
    let err = decode_file_descriptor_set(&set).expect_err("2^32 must overflow u32");
    match err {
        CodegenError::Wire(m) => assert!(m.contains("overflows u32"), "got: {m}"),
        other => panic!("wrong error shape: {other}"),
    }
}

/// An enum value number that overflows i32 must fail loudly, not wrap
/// around into a legal sentinel.
#[test]
fn enum_number_over_i32_is_a_wire_error() {
    let mut e = ld(1, b"Status");
    let mut ok = ld(1, b"OK");
    ok.extend(varint_field(2, 0)); // legal sentinel
    let mut over = ld(1, b"BIG");
    over.extend(varint_field(2, (1i64 << 31) as u64)); // 2^31: over i32
    e.extend(ld(2, &ok));
    e.extend(ld(2, &over));
    let mut file = ld(2, b"com.acme");
    file.extend(ld(5, &e));
    let set = ld(1, &file);
    let err = decode_file_descriptor_set(&set).expect_err("2^31 must overflow i32");
    match err {
        CodegenError::Wire(m) => assert!(m.contains("overflows i32"), "got: {m}"),
        other => panic!("wrong error shape: {other}"),
    }
}

/// A message declaring a nested type (DescriptorProto.nested_type = 3) is
/// outside the flat v0 subset and must be rejected, not half-decoded.
#[test]
fn nested_message_type_is_outside_the_v0_subset() {
    let mut inner = ld(1, b"Inner");
    inner.extend(varint_field(3, 1)); // dummy payload so inner is non-empty
    let mut msg = ld(1, b"Outer");
    msg.extend(ld(3, &inner)); // nested_type = 3
    let mut file = ld(2, b"com.acme");
    file.extend(ld(4, &msg));
    let set = ld(1, &file);
    let err = decode_file_descriptor_set(&set).expect_err("nested types must be rejected");
    match err {
        CodegenError::Wire(m) => {
            assert!(m.contains("Outer") && m.contains("nested"), "got: {m}")
        }
        other => panic!("wrong error shape: {other}"),
    }
}

/// A method declaring client or server streaming (fields 5/6 non-zero)
/// crosses the D6 sync boundary and must be refused, naming the method.
#[test]
fn streaming_method_is_refused_at_the_d6_boundary() {
    let mut m = ld(1, b"Watch");
    m.extend(ld(2, b".com.acme.Req"));
    m.extend(ld(3, b".com.acme.Resp"));
    m.extend(varint_field(5, 1)); // client_streaming
    let mut svc = ld(1, b"Greeter");
    svc.extend(ld(2, &m));
    svc.extend(ld(3, &ld(EXT_SERVICE_ID, b"com.acme.greeter")));
    let mut file = ld(1, b"x.proto");
    file.extend(ld(2, b"com.acme"));
    file.extend(ld(6, &svc));
    let set = ld(1, &file);
    let err = decode_file_descriptor_set(&set).expect_err("streaming must be refused");
    match err {
        CodegenError::Wire(msg) => assert!(msg.contains("Watch"), "must name the method: {msg}"),
        other => panic!("wrong error shape: {other}"),
    }
}

/// The uninterpreted-option arm: when protoc could not resolve
/// `(modukit.service_id)` (option present at tag 999 but no 50000), the
/// error must point at the missing import, not merely at a missing stamp.
#[test]
fn uninterpreted_service_option_error_points_at_the_import() {
    let mut svc = ld(1, b"Greeter");
    svc.extend(ld(3, &varint_field(999, 1))); // uninterpreted_option
    let mut file = ld(1, b"x.proto");
    file.extend(ld(2, b"com.acme"));
    file.extend(ld(6, &svc));
    let set = ld(1, &file);
    let err = decode_file_descriptor_set(&set).expect_err("must fail loudly");
    match err {
        CodegenError::ServiceIdOption(m) => {
            assert!(
                m.contains("uninterpreted") && m.contains("Greeter"),
                "got: {m}"
            )
        }
        other => panic!("wrong error shape: {other}"),
    }
}

/// The D9 id grammar: every rejection direction of
/// `dotted_reverse_dns` gets its own adversarial stamp value.
#[test]
fn service_id_grammar_rejects_malformed_stamps() {
    let bad_ids = [
        "nodot",        // not dotted
        ".leading.dot", // starts with a dot
        "trailing.",    // ends with a dot
        "double..dot",  // empty label
        "spaced id.x",  // illegal byte (space)
        "carrot/^",     // illegal bytes
    ];
    for raw in bad_ids {
        let mut svc = ld(1, b"Greeter");
        svc.extend(ld(3, &ld(EXT_SERVICE_ID, raw.as_bytes())));
        let mut file = ld(1, b"x.proto");
        file.extend(ld(2, b"com.acme"));
        file.extend(ld(6, &svc));
        let set = ld(1, &file);
        let err = match decode_file_descriptor_set(&set) {
            Ok(_) => panic!("malformed stamp {raw:?} must be refused"),
            Err(e) => e,
        };
        match err {
            CodegenError::ServiceIdOption(m) => assert!(m.contains(raw), "got: {m}"),
            other => panic!("wrong error shape for {raw:?}: {other}"),
        }
    }
}

/// A legal id with the full allowed alphabet (digits, dash, underscore)
/// passes the same grammar that refuses the malformed ones - the positive
/// half of the D9 grammar, asserted through decode.
#[test]
fn service_id_grammar_accepts_the_full_legal_alphabet() {
    let mut svc = ld(1, b"Greeter");
    svc.extend(ld(3, &ld(EXT_SERVICE_ID, b"a-b_0.9z.example_id")));
    let mut file = ld(1, b"x.proto");
    file.extend(ld(2, b"com.acme"));
    file.extend(ld(6, &svc));
    let set = ld(1, &file);
    let files = decode_file_descriptor_set(&set).expect("legal id must pass");
    assert_eq!(files[0].services[0].service_id, "a-b_0.9z.example_id");
}

/// An fdset whose top-level entry is a valid file with NO package or
/// service decodes to an empty-service File; generate() over it fails with
/// the typed RuntimeLedgerMissing (the ledger contract was not among the
/// inputs), proving the generate() door's first error arm.
#[test]
fn generate_without_the_runtime_ledger_is_a_typed_error() {
    let file = ld(1, b"x.proto");
    let set = ld(1, &file);
    let dir = std::env::temp_dir().join(format!("mk-codegen-debt-{}", std::process::id()));
    let fdset = dir.join("noledger.pb");
    std::fs::create_dir_all(&dir).expect("scratch dir");
    std::fs::write(&fdset, &set).expect("write fdset");
    let err = generate(&[fdset], &dir.join("out")).expect_err("ledger missing must fail");
    assert!(
        matches!(err, CodegenError::RuntimeLedgerMissing),
        "got: {err}"
    );
    std::fs::remove_dir_all(&dir).expect("cleanup scratch");
}

/// generate() over an unreadable path surfaces the io error through the
/// From<std::io::Error> arm (typed Io variant).
#[test]
fn generate_over_a_missing_fdset_file_is_a_typed_io_error() {
    let ghost = PathBuf::from("/nonexistent/mk-codegen-debt/nothing.pb");
    let err = generate(&[ghost], &std::env::temp_dir()).expect_err("missing file must fail");
    assert!(matches!(err, CodegenError::Io(_)), "got: {err}");
}

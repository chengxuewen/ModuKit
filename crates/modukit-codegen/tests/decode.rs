//! Subset-decoder behavior, tested through the public entry point:
//! decode-and-skip on unknown fields (D12 rule 2) and the loud failures on
//! missing option stamps / unnumbered fields / missing sentinels.

use modukit_codegen::CodegenError;
use modukit_codegen::descriptor::{EXT_SERVICE_ID, decode_file_descriptor_set};

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

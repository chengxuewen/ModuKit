//! Wire-codec and model contract tests (D84 coverage-debt lane, 2026-10-19).
//! The hand-written protobuf reader is the crate's trust boundary: every
//! arm below asserts a REAL behavior - roundtrip values, typed failures,
//! or the Display contract the protoc-gen door surfaces to users.

use modukit_codegen::error::CodegenError;
use modukit_codegen::wire::{
    FieldCursor, Reader, WT_FIXED32, WT_FIXED64, WT_LEN, WT_VARINT, WireValue,
};

fn varint(n: u64) -> Vec<u8> {
    let mut v = Vec::new();
    let mut n = n;
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
    v.extend(varint(payload.len() as u64));
    v.extend_from_slice(payload);
    v
}

#[test]
fn varint_roundtrips_single_and_multi_byte_values() {
    // Arrange: 1-byte (0x2A), 2-byte (300 = AC 02), and the u64 maximum.
    let max = u64::MAX;
    for (name, bytes, want) in [
        ("one byte", varint(0x2A), 0x2A),
        ("two bytes", varint(300), 300),
        ("u64 max", varint(max), max),
    ] {
        // Act
        let mut r = Reader::new(&bytes);
        let got = r.read_varint();
        // Assert
        assert_eq!(got.expect(name), want, "{name}");
    }
}

#[test]
fn varint_truncated_after_continuation_byte_is_an_error() {
    // Arrange: 0x80 says "one more byte follows" - and none does.
    let bytes = [0x80u8];
    let mut r = Reader::new(&bytes);
    // Act + Assert
    let err = r.read_varint().expect_err("truncated varint must fail");
    match err {
        CodegenError::Wire(m) => assert!(m.contains("truncated"), "got: {m}"),
        other => panic!("wrong shape: {other}"),
    }
}

#[test]
fn varint_longer_than_ten_bytes_is_an_error() {
    // Arrange: eleven continuation bytes - past the u64 encoding bound.
    let bytes = [0xFFu8; 11];
    let mut r = Reader::new(&bytes);
    // Act + Assert
    let err = r.read_varint().expect_err("11-byte varint must fail");
    match err {
        CodegenError::Wire(m) => assert!(m.contains("10 bytes"), "got: {m}"),
        other => panic!("wrong shape: {other}"),
    }
}

#[test]
fn read_fixed_rejects_widths_other_than_four_or_eight() {
    // Arrange
    let mut r = Reader::new(&[0u8; 8]);
    // Act + Assert
    let err = r.read_fixed(5).expect_err("width 5 must fail");
    match err {
        CodegenError::Wire(m) => assert!(m.contains("bad fixed width 5"), "got: {m}"),
        other => panic!("wrong shape: {other}"),
    }
}

#[test]
fn read_fixed_refuses_a_truncated_buffer() {
    // Arrange: 8-byte field promised, 4 delivered.
    let mut r = Reader::new(&[0u8; 4]);
    // Act + Assert
    let err = r.read_fixed(8).expect_err("short fixed64 must fail");
    match err {
        CodegenError::Wire(m) => assert!(m.contains("truncated fixed"), "got: {m}"),
        other => panic!("wrong shape: {other}"),
    }
}

#[test]
fn read_fixed_consumes_exactly_four_then_eight_bytes() {
    // Arrange: 12 bytes, two sentinel values.
    let mut bytes = 0x11223344u32.to_le_bytes().to_vec();
    bytes.extend_from_slice(&0x5566778899AABBCCu64.to_le_bytes());
    let mut r = Reader::new(&bytes);
    // Act
    let four = r.read_fixed(4).expect("fixed32");
    // Assert: the returned slice is the LE bytes, and the cursor advanced.
    assert_eq!(four, &0x11223344u32.to_le_bytes());
    let eight = r.read_fixed(8).expect("fixed64");
    assert_eq!(eight, &0x5566778899AABBCCu64.to_le_bytes());
    assert!(r.is_empty(), "cursor must sit at the end");
}

#[test]
fn read_len_bytes_with_a_length_that_overflows_usize_is_an_error() {
    // Arrange: length varint = u64::MAX; pos + MAX overflows usize.
    let mut bytes = varint(u64::MAX);
    bytes.extend_from_slice(b"payload");
    let mut r = Reader::new(&bytes);
    // Act + Assert
    let err = r
        .read_len_bytes()
        .expect_err("u64::MAX length must overflow");
    match err {
        CodegenError::Wire(m) => assert!(m.contains("length overflow"), "got: {m}"),
        other => panic!("wrong shape: {other}"),
    }
}

#[test]
fn read_len_bytes_short_buffer_is_an_error_naming_the_gap() {
    // Arrange: declares 10 bytes, delivers 3.
    let mut bytes = varint(10);
    bytes.extend_from_slice(b"abc");
    let mut r = Reader::new(&bytes);
    // Act + Assert
    let err = r.read_len_bytes().expect_err("short payload must fail");
    match err {
        CodegenError::Wire(m) => {
            assert!(m.contains("truncated length-delimited"), "got: {m}");
            assert!(m.contains("need 10 bytes"), "must name the gap: {m}");
        }
        other => panic!("wrong shape: {other}"),
    }
}

#[test]
fn next_field_returns_none_on_an_empty_buffer() {
    // Arrange
    let mut r = Reader::new(&[]);
    let mut cur = FieldCursor::new(&mut r);
    // Act + Assert
    assert!(
        matches!(cur.next_field(), Ok(None)),
        "empty input = no fields"
    );
}

#[test]
fn next_field_rejects_field_numbers_beyond_u32() {
    // Arrange: tag = ((2^32 + 1) << 3) | 0 - number 2^32+1 cannot be u32.
    let tag = ((1u64 << 32) + 1) << 3;
    let bytes = varint(tag);
    let mut r = Reader::new(&bytes);
    let mut cur = FieldCursor::new(&mut r);
    // Act + Assert
    let err = cur.next_field().expect_err("2^32+1 must exceed u32");
    match err {
        CodegenError::Wire(m) => assert!(m.contains("out of u32 range"), "got: {m}"),
        other => panic!("wrong shape: {other}"),
    }
}

#[test]
fn next_field_rejects_unknown_wire_types_six_and_seven() {
    for wire in [6u64, 7] {
        // Arrange: field 3 with an unknown wire type.
        let bytes = varint((3 << 3) | wire);
        let mut r = Reader::new(&bytes);
        let mut cur = FieldCursor::new(&mut r);
        // Act + Assert
        let err = cur.next_field().expect_err("unknown wire type must fail");
        match err {
            CodegenError::Wire(m) => {
                assert!(m.contains(&format!("unknown wire type {wire}")), "got: {m}")
            }
            other => panic!("wrong shape for wire {wire}: {other}"),
        }
    }
}

#[test]
fn next_field_decodes_all_four_supported_wire_shapes() {
    // Arrange: one field of each supported wire type, in order.
    let mut bytes = varint(u64::from(1 << 3 | WT_VARINT));
    bytes.extend(varint(7));
    bytes.extend(varint(u64::from(2 << 3 | WT_FIXED64)));
    bytes.extend(0x0807060504030201u64.to_le_bytes());
    bytes.extend(varint(u64::from(3 << 3 | WT_LEN)));
    bytes.extend(varint(3));
    bytes.extend_from_slice(b"abc");
    bytes.extend(varint(u64::from(4 << 3 | WT_FIXED32)));
    bytes.extend(0xDDCCBBAAu32.to_le_bytes());
    let mut r = Reader::new(&bytes);
    let mut cur = FieldCursor::new(&mut r);
    // Act
    let (n1, v1) = cur.next_field().expect("varint").expect("field 1");
    let (n2, v2) = cur.next_field().expect("fixed64").expect("field 2");
    let (n3, v3) = cur.next_field().expect("len").expect("field 3");
    let (n4, v4) = cur.next_field().expect("fixed32").expect("field 4");
    // Assert: numbers in order, values decoded to the right shapes.
    assert_eq!((n1, &v1), (1, &WireValue::Varint(7)));
    assert_eq!(n2, 2);
    assert_eq!(v2, WireValue::Fixed64(0x0807060504030201));
    assert_eq!(n3, 3);
    assert_eq!(v3, WireValue::Len(b"abc".to_vec()));
    assert_eq!(n4, 4);
    assert_eq!(v4, WireValue::Fixed32(0xDDCCBBAA));
    assert!(matches!(cur.next_field(), Ok(None)), "input exhausted");
}

#[test]
fn wire_value_accessors_refuse_the_wrong_shape() {
    // Arrange: one value of each non-matching shape.
    let len = WireValue::Len(b"x".to_vec());
    let varint_v = WireValue::Varint(1);
    // Act + Assert: as_varint on a Len payload refuses.
    let err = len.as_varint().expect_err("Len is not a varint");
    match err {
        CodegenError::Wire(m) => assert!(m.contains("expected varint"), "got: {m}"),
        other => panic!("wrong shape: {other}"),
    }
    // as_len_bytes on a varint refuses.
    let err = varint_v
        .as_len_bytes()
        .expect_err("Varint is not len-delimited");
    match err {
        CodegenError::Wire(m) => assert!(m.contains("expected length-delimited"), "got: {m}"),
        other => panic!("wrong shape: {other}"),
    }
}

#[test]
fn as_len_str_decodes_utf8_and_refuses_invalid_utf8() {
    // Act + Assert: valid UTF-8 round-trips.
    let good = WireValue::Len("héllo".as_bytes().to_vec());
    assert_eq!(good.as_len_str().expect("utf8"), "héllo");
    // Invalid UTF-8 (a bare 0xFF byte) refuses with a wire error.
    let bad = WireValue::Len(vec![0xFF]);
    let err = bad.as_len_str().expect_err("0xFF is not UTF-8");
    match err {
        CodegenError::Wire(m) => assert!(!m.is_empty(), "utf8 error carries the reason"),
        other => panic!("wrong shape: {other}"),
    }
}

/// `FieldType::from_type_number` is pub(crate); the mapping contract is
/// asserted through its only caller, the field decoder: a hand-built
/// FieldDescriptorProto carrying each type number decodes to that variant.
fn set_field_type(n: u64) -> Vec<u8> {
    let mut field = ld(1, b"f");
    field.extend(varint_field_num(3, 1)); // number = 1
    field.extend(varint_field_num(4, 1)); // label = optional
    field.extend(varint_field_num(5, n)); // type = n
    let mut msg = ld(1, b"M");
    msg.extend(ld(2, &field));
    let mut file = ld(2, b"com.acme");
    file.extend(ld(4, &msg));
    ld(1, &file)
}

fn varint_field_num(number: u32, value: u64) -> Vec<u8> {
    let mut v = tag(number, 0);
    v.extend(varint(value));
    v
}

#[test]
fn field_type_maps_every_protobuf_type_number_through_the_field_decoder() {
    // Arrange: the canonical descriptor.proto Type table (values 1..=18),
    // asserted through the decoder door (the mapper itself is pub(crate)).
    for (n, want) in [
        (1u64, "Double"),
        (2, "Float"),
        (3, "Int64"),
        (4, "Uint64"),
        (5, "Int32"),
        (6, "Fixed64"),
        (7, "Fixed32"),
        (8, "Bool"),
        (9, "String"),
        (10, "Group"),
        (11, "Message"),
        (12, "Bytes"),
        (13, "Uint32"),
        (14, "Enum"),
        (15, "Sfixed32"),
        (16, "Sfixed64"),
        (17, "Sint32"),
        (18, "Sint64"),
    ] {
        // Act
        let files = modukit_codegen::descriptor::decode_file_descriptor_set(&set_field_type(n))
            .unwrap_or_else(|e| panic!("type {n} must decode: {e}"));
        // Assert
        let got = format!("{:?}", files[0].messages[0].fields[0].kind);
        assert_eq!(got, want, "type {n}");
    }
}

#[test]
fn field_type_refuses_numbers_outside_the_table() {
    for n in [0u64, 19, 100] {
        // Act + Assert
        let err = modukit_codegen::descriptor::decode_file_descriptor_set(&set_field_type(n))
            .expect_err("outside 1..=18 must fail");
        match err {
            CodegenError::Wire(m) => assert!(
                m.contains(&format!("unknown FieldDescriptorProto.Type: {n}")),
                "got: {m}"
            ),
            other => panic!("wrong shape for {n}: {other}"),
        }
    }
}

#[test]
fn every_error_variant_renders_its_documented_display_contract() {
    // The Display strings are the crate's user-facing contract (they ride
    // protoc-gen failures). Each arm must render non-empty, prefixed text.
    let cases: Vec<(CodegenError, &str)> = vec![
        (
            CodegenError::Io(std::io::Error::other("disk gone")),
            "io: disk gone",
        ),
        (CodegenError::Wire("boom".into()), "wire decode: boom"),
        (
            CodegenError::GroupUnsupported(9),
            "group wire type at field 9",
        ),
        (
            CodegenError::ServiceIdOption("no stamp".into()),
            "(modukit.service_id)",
        ),
        (
            CodegenError::FieldRule("no number".into()),
            "field rule: no number",
        ),
        (
            CodegenError::MissingSentinel("Status".into()),
            "enum sentinel: Status",
        ),
        (
            CodegenError::UnresolvedType(".gone.T".into()),
            "unresolved type reference: .gone.T",
        ),
        (CodegenError::Usage("bad argv".into()), "usage: bad argv"),
    ];
    for (err, fragment) in cases {
        let shown = err.to_string();
        assert!(shown.contains(fragment), "{err:?} rendered {shown:?}");
    }
    // The ledger-missing arm has a fixed, auditable text.
    let shown = CodegenError::RuntimeLedgerMissing.to_string();
    assert!(
        shown.contains("org.modukit.runtime") && shown.contains("missing"),
        "got: {shown}"
    );
}

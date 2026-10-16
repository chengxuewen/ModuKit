//! Hand-written SHA-256 (M7, std-only per the vendor-lock floor). The
//! vectors below are NIST FIPS 180-4 examples plus block-boundary cases
//! cross-checked against Python's hashlib (a NIST-derived implementation):
//!   ""                -> e3b0c442...b855            (FIPS Example 1)
//!   "abc"             -> ba7816bf...15ad            (FIPS Example 1)
//!   1,000,000 x 'a'   -> cdc76e5c...112cd0          (FIPS Example 3)
//!   55/56/57/63/64/65 x 'a' + "abcdefgh"x7         (padding/block boundaries)
//!
//! M7 first-red (sha256-vectors area): written before `sha256.rs` existed;
//! the import below was the observed RED (E0432 unresolved import).

use modukit_core::sha256::{Sha256, hash_bytes, hash_file, hash_hex, to_hex};

fn const_pattern(len: usize, byte: u8) -> Vec<u8> {
    vec![byte; len]
}

#[test]
fn nist_empty_vector() {
    assert_eq!(
        hash_hex(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

#[test]
fn nist_abc_vector() {
    assert_eq!(
        hash_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn block_boundary_vectors() {
    // Padding straddles the 64-byte block at 56..=63 payload bytes; these
    // pins catch off-by-one length/pad errors a single vector cannot.
    let cases: &[(&[u8], &str)] = &[
        (
            b"a",
            "ca978112ca1bbdcafac231b39a23dc4da786eff8147c4e72b9807785afee48bb",
        ),
        (
            // 55 bytes
            b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "9f4390f8d30c2dd92ec9f095b65e2b9ae9b0a925a5258e241c9f1e910f734318",
        ),
        (
            // 56 bytes
            b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "b35439a4ac6f0948b6d6f9e3c6af0f5f590ce20f1bde7090ef7970686ec6738a",
        ),
        (
            // 57 bytes
            b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "f13b2d724659eb3bf47f2dd6af1accc87b81f09f59f2b75e5c0bed6589dfe8c6",
        ),
        (
            // 63 bytes
            b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "7d3e74a05d7db15bce4ad9ec0658ea98e3f06eeecf16b4c6fff2da457ddc2f34",
        ),
        (
            // 64 bytes: exact block
            b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "ffe054fe7ae0cb6dc65c3af9b61d5209f439851db43d0ba5997337df154668eb",
        ),
        (
            // 65 bytes: block + 1
            b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "635361c48bb9eab14198e76ea8ab7f1a41685d6ad62aa9146d301d4f17eb0ae0",
        ),
    ];
    for (input, want) in cases {
        assert_eq!(hash_hex(input), *want, "len {}", input.len());
    }
    let repeated = *b"abcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefgh";
    assert_eq!(
        hash_hex(&repeated[..]),
        "8a261744ce595c50431d19c6d4f68ed24995ed11f7e1d88fe353608f0655ee53"
    );
}

#[test]
fn million_a_streamed_in_chunks() {
    // FIPS Example 3, fed in 4096-byte chunks (the streaming interface, not
    // one whole-message update).
    let mut sha = Sha256::new();
    let stream = const_pattern(1_000_000, b'a');
    for chunk in stream.chunks(4096) {
        sha.update(chunk);
    }
    assert_eq!(
        to_hex(&sha.finalize()),
        "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
    );
}

#[test]
fn split_and_reuse_are_unchanged() {
    // The digest is a function of the byte stream: arbitrary update splits
    // must agree, and one instance can be reused after finalize.
    let one = hash_bytes(b"abc");
    let mut sha = Sha256::new();
    sha.update(b"a");
    sha.update(b"b");
    sha.update(b"c");
    assert_eq!(sha.finalize(), one);
    // finalize resets the instance: re-feeding the same stream must agree.
    sha.update(b"abc");
    assert_eq!(sha.finalize(), one, "instance resets for reuse");
    let mut big = Sha256::new();
    big.update(&const_pattern(2000, b'a'));
    big.update(&const_pattern(2000, b'b'));
    let mut whole = Vec::new();
    whole.extend(const_pattern(2000, b'a'));
    whole.extend(const_pattern(2000, b'b'));
    assert_eq!(to_hex(&big.finalize()), hash_hex(&whole));
}

#[test]
fn file_hash_streams_without_loading_whole_file() {
    // 2 MiB of a repeating pattern, hashed from disk through the chunked
    // reader; equal to the in-memory digest of the same bytes.
    let path = std::env::temp_dir().join(format!("modukit-sha256-{}.bin", std::process::id()));
    let content = {
        let mut v = [0u8; 64];
        for (i, b) in v.iter_mut().enumerate() {
            *b = (i % 251) as u8;
        }
        v.repeat(32 * 1024) // 2 MiB
    };
    std::fs::write(&path, &content).expect("write temp");
    let from_file = hash_file(&path).expect("hash temp file");
    let _ = std::fs::remove_file(&path);
    assert_eq!(to_hex(&from_file), hash_hex(&content));
}

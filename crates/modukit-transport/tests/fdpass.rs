//! Cross-process descriptor passing over SCM_RIGHTS (S2-M3a; modules/07 §3
//! "the pool fd is handed over exactly once via UDS (SCM_RIGHTS) — slot 1)
//! remains the only fd courier (D7)"). The memfd lane's blob fd genuinely
//! crosses a process boundary: the crate side rides the hand-written
//! `fdpass` shim (the documented-unsafe `memfd_create` precedent); the child
//! side is an independent std-only re-implementation, so interop — not one
//! implementation talking to itself — is what the green asserts.
//!
//! Two-stage meta-then-data (D14) is observable here: the CHILD sends the
//! SlotHandle metadata frame with the fd riding it, the PARENT pulls the
//! data through the received fd, then answers with its own memfd blob — the
//! round trip byte-equal in both directions.

// D104 T2.4: test-scaffolding file; the allow-*-in-tests valve does not
// reach plain helper fns in a tests/ target (clippy 0.1.99), so the
// exemption lands at the file head. Production zones stay denied.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::io::{Read, Write};
use std::os::fd::OwnedFd;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use modukit_transport::fdpass::{recv_with_fd, send_with_fd};
use modukit_transport::memfd::MemfdBlob;
use modukit_transport::seam::Transport;

/// Read bound for every cross-process wait in this file (a silent peer is
/// a failure, not an infinite block; same discipline as the core
/// HANDSHAKE_TIMEOUT family).
const CROSS_PROC_TIMEOUT: Duration = Duration::from_secs(5);
/// Bound for reaping the real child after its work is done.
const CHILD_EXIT_BOUND: Duration = Duration::from_secs(10);

const CHILD_BLOB: &[u8] = b"cargo-bytes-from-the-child-0123456789";
const PARENT_BLOB: &[u8] = b"cargo-bytes-from-the-parent-fedcba9876543210";

/// The SlotHandle meta arm of the wire (07 §3 self-sufficient token):
/// `u64 LE slot | u32 LE epoch | u64 LE cell_bytes` — 20 bytes, always
/// sent as the data half riding alongside the descriptor.
fn meta_bytes(slot: u64, epoch: u32, cell: u64) -> Vec<u8> {
    let mut m = slot.to_le_bytes().to_vec();
    m.extend_from_slice(&epoch.to_le_bytes());
    m.extend_from_slice(&cell.to_le_bytes());
    m
}

/// Read a passed blob from its start. The received descriptor shares the
/// SENDER's open file description (that is what SCM_RIGHTS passes — the
/// offset too), so the reader positions itself: the geometry came with the
/// meta frame (D14 two-stage), the fd only locates the bytes.
fn read_blob(fd: OwnedFd) -> Vec<u8> {
    use std::io::{Seek, SeekFrom};
    let mut file = std::fs::File::from(fd);
    file.seek(SeekFrom::Start(0)).expect("position the blob");
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).expect("read the blob");
    buf
}

// spec: TRANSPORT-01
#[test]
fn the_fdpass_shim_round_trips_a_memfd_blob_over_a_socketpair() {
    // In-process leg: proves both crate verbs (send and recv) carry one
    // descriptor + the meta bytes with it, epoch/slot/cell decode intact.
    let (mut a, b) = UnixStream::pair().expect("socketpair");
    let mut blob = match MemfdBlob::create("modukit-fdpass-probe") {
        Ok(x) => x,
        Err(e) => {
            eprintln!("SKIPPING: memfd_create unavailable here: {e}");
            return;
        }
    };
    blob.send(b"one-cargo".to_vec()).expect("write the blob");

    let slot = 7u64;
    let epoch = 3u32;
    let cell = 9u64;
    send_with_fd(
        &a,
        &meta_bytes(slot, epoch, cell),
        blob.raw_fd().expect("open blob"),
    )
    .expect("pass the fd");

    b.set_read_timeout(Some(CROSS_PROC_TIMEOUT))
        .expect("recv bound");
    let mut meta = [0u8; 20];
    let (n, fd) = recv_with_fd(&b, &mut meta).expect("collect the fd");
    assert_eq!(n, 20, "the whole meta frame rode across");
    assert_eq!(u64::from_le_bytes(meta[..8].try_into().unwrap()), slot);
    assert_eq!(u32::from_le_bytes(meta[8..12].try_into().unwrap()), epoch);
    assert_eq!(u64::from_le_bytes(meta[12..].try_into().unwrap()), cell);
    // Two-stage retrieval (D14): the meta alone was enough to know the
    // geometry; the bytes move through the received descriptor.
    assert_eq!(
        read_blob(fd),
        b"one-cargo".to_vec(),
        "blob by the passed fd"
    );

    // No-descriptor discipline: a frame sent as plain bytes yields a typed
    // refusal on collect, never a fabricated fd.
    a.write_all(b"plain").expect("plain write");
    let mut buf = [0u8; 5];
    let err = recv_with_fd(&b, &mut buf).expect_err("no fd in a plain read");
    assert_eq!(err.code(), "CARRIER_UNREACHABLE", "{err}");
}

/// The real-child stand-in: std-only, with its OWN small `memfd_create` /
/// `sendmsg` / `recvmsg` shims (the interop counterparty; the layout here
/// is the x86_64 Linux one the crate shim documents). It passes a memfd
/// blob to the parent, reads the parent's answer blob through a received
/// fd, and exits 0 only if both byte sequences match.
const CHILD_SRC: &str = r#"
use std::ffi::{c_int, c_void};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::fd::FromRawFd;
use std::os::unix::net::UnixStream;

const MFD_CLOEXEC: u32 = 1;
const SOL_SOCKET: c_int = 1;
const SCM_RIGHTS: c_int = 1;

#[repr(C)]
struct CIovec { base: *mut c_void, len: usize }
#[repr(C)]
struct CHdr { name: *mut c_void, namelen: c_int, iov: *mut CIovec, iovlen: usize,
              control: *mut c_void, controllen: usize, flags: c_int }

unsafe extern "C" {
    fn memfd_create(name: *const i8, flags: u32) -> c_int;
    fn sendmsg(fd: c_int, hdr: *const CHdr, flags: c_int) -> isize;
    fn recvmsg(fd: c_int, hdr: *mut CHdr, flags: c_int) -> isize;
}

// SAFETY: raw fd passed by the parent as stdin; the --fd=N convention.
fn pass(sock_fd: c_int, meta: &[u8], fd: c_int) {
    let mut cbuf = [0u8; 24]; // CMSG_SPACE(sizeof(int)) on x86_64
    let len: usize = 20;      // CMSG_LEN(sizeof(int))
    cbuf[0..8].copy_from_slice(&len.to_ne_bytes());          // cmsg_len
    cbuf[8..12].copy_from_slice(&SOL_SOCKET.to_ne_bytes());  // cmsg_level
    cbuf[12..16].copy_from_slice(&SCM_RIGHTS.to_ne_bytes()); // cmsg_type
    cbuf[16..20].copy_from_slice(&fd.to_ne_bytes());         // the fd
    let mut iov = CIovec { base: meta.as_ptr() as *mut c_void, len: meta.len() };
    let hdr = CHdr { name: std::ptr::null_mut(), namelen: 0, iov: &mut iov,
                     iovlen: 1, control: cbuf.as_mut_ptr().cast(), controllen: 24, flags: 0 };
    // SAFETY: connected AF_UNIX socket; the iovec and cbuf outlive the call.
    if unsafe { sendmsg(sock_fd, &hdr, 0) } < 0 {
        std::process::exit(11);
    }
}

fn collect(sock_fd: c_int, meta: &mut [u8]) -> i32 {
    let mut cbuf = [0u8; 24];
    let mut iov = CIovec { base: meta.as_mut_ptr().cast(), len: meta.len() };
    let mut hdr = CHdr { name: std::ptr::null_mut(), namelen: 0, iov: &mut iov,
                         iovlen: 1, control: cbuf.as_mut_ptr().cast(), controllen: 24, flags: 0 };
    // SAFETY: as above; buffers outlive the blocking call (SO_RCVTIMEO set).
    let got = unsafe { recvmsg(sock_fd, &mut hdr, 0) };
    if got < 0 { std::process::exit(12); }
    if hdr.flags & 0x8 != 0 { std::process::exit(13); } // MSG_CTRUNC: fd lost
    if hdr.controllen < 20 { std::process::exit(14); }
    let mut fd = [0u8; 4];
    fd.copy_from_slice(&cbuf[16..20]);
    i32::from_ne_bytes(fd)
}

fn main() {
    let mut fd_num: i32 = 0;
    for a in std::env::args().skip(1) {
        if let Some(rest) = a.strip_prefix("--fd=") { fd_num = rest.parse().unwrap_or(0); }
    }
    // SAFETY: the parent injected the control socket at this descriptor.
    let sock = unsafe { UnixStream::from_raw_fd(fd_num) };
    let _ = sock.set_read_timeout(Some(std::time::Duration::from_secs(5)));
    let sfd: c_int = sock.as_raw_fd_check();

    // 1) Build the child's blob in a fresh memfd and pass it (meta + fd).
    let name = std::ffi::CString::new("modukit-fdpass-child").unwrap();
    // SAFETY: valid NUL-terminated name; memfd_create returns a fresh fd.
    let mfd = unsafe { memfd_create(name.as_ptr(), MFD_CLOEXEC) };
    if mfd < 0 { std::process::exit(21); }
    // SAFETY: freshly created owned fd.
    let mut file = unsafe { File::from_raw_fd(mfd) };
    file.write_all(b"cargo-bytes-from-the-child-0123456789").unwrap();

    let mut meta = (7u64).to_le_bytes().to_vec();
    meta.extend_from_slice(&1u32.to_le_bytes());
    meta.extend_from_slice(&41u64.to_le_bytes());
    pass(sfd, &meta, file.as_raw_fd_check());

    // 2) Collect the parent's answer fd and verify its bytes.
    let mut rmeta = [0u8; 20];
    let got_fd = collect(sfd, &mut rmeta);
    if got_fd < 0 { std::process::exit(22); }
    // SAFETY: fd received via SCM_RIGHTS — now owned by this process.
    let mut answer = unsafe { File::from_raw_fd(got_fd) };
    answer.seek(SeekFrom::Start(0)).unwrap();
    let mut buf = Vec::new();
    answer.read_to_end(&mut buf).unwrap();
    if buf != b"cargo-bytes-from-the-parent-fedcba9876543210".to_vec() {
        std::process::exit(23);
    }
    std::process::exit(0);
}

trait RawFdCheck { fn as_raw_fd_check(&self) -> c_int; }
impl RawFdCheck for File { fn as_raw_fd_check(&self) -> c_int { use std::os::fd::AsRawFd; self.as_raw_fd() as c_int } }
impl RawFdCheck for UnixStream { fn as_raw_fd_check(&self) -> c_int { use std::os::fd::AsRawFd; self.as_raw_fd() as c_int } }
"#;

fn build_child(tag: &str) -> Option<PathBuf> {
    let dir = std::env::temp_dir().join(format!("mk-fdpass-{}-{tag}", std::process::id()));
    let exe = dir.join("fdpass-child");
    if exe.is_file() {
        return Some(exe);
    }
    std::fs::create_dir_all(&dir).expect("temp dir");
    let src = dir.join("child.rs");
    std::fs::write(&src, CHILD_SRC).expect("write child source");
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let out = Command::new(&rustc)
        .args(["--edition", "2024", "-o"])
        .arg(&exe)
        .arg(&src)
        .output();
    match out {
        Ok(o) if o.status.success() => Some(exe),
        Ok(o) => {
            eprintln!(
                "SKIPPING: child build failed: {}",
                String::from_utf8_lossy(&o.stderr)
            );
            None
        }
        Err(e) => {
            eprintln!("SKIPPING: cannot spawn rustc: {e}");
            None
        }
    }
}

#[test]
fn a_memfd_blob_round_trips_across_a_real_process_boundary() {
    let Some(child) = build_child("roundtrip") else {
        return;
    };
    let (parent, child_end) = UnixStream::pair().expect("socketpair");
    parent
        .set_read_timeout(Some(CROSS_PROC_TIMEOUT))
        .expect("recv bound");
    let mut child = {
        let mut cmd = Command::new(&child);
        // The same fd-0 stdio control injection as the kernel executor
        // (01 §1 shape; SCM_RIGHTS carries BUFFER fds riding the channel).
        cmd.stdin(Stdio::from(OwnedFd::from(child_end)))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .arg("--fd=0");
        cmd.spawn().expect("spawn the real child")
    };

    // Stage 1 (meta): the child announces slot/epoch/cell...
    let mut meta = [0u8; 20];
    let (n, child_fd) = recv_with_fd(&parent, &mut meta).expect("collect the child fd");
    assert_eq!(n, 20);
    assert_eq!(u64::from_le_bytes(meta[..8].try_into().unwrap()), 7);
    assert_eq!(u32::from_le_bytes(meta[8..12].try_into().unwrap()), 1);
    assert_eq!(u64::from_le_bytes(meta[12..].try_into().unwrap()), 41);
    // Stage 2 (data): ...and the bytes move through the passed descriptor —
    // across a genuine process boundary, byte-equal.
    assert_eq!(read_blob(child_fd), CHILD_BLOB.to_vec());

    // The parent answers the same way: its own memfd blob, fd on the wire.
    let mut reply = MemfdBlob::create("modukit-fdpass-reply")
        .expect("memfd available (the child needs it too)");
    reply
        .send(PARENT_BLOB.to_vec())
        .expect("write the reply blob");
    send_with_fd(
        &parent,
        &meta_bytes(9, 2, PARENT_BLOB.len() as u64),
        reply.raw_fd().expect("open reply blob"),
    )
    .expect("pass the reply fd");

    // The child verified PARENT_BLOB through the received fd (exit 0 only
    // on byte equality); bounded reap, no unbounded block.
    let deadline = Instant::now() + CHILD_EXIT_BOUND;
    loop {
        match child.try_wait().expect("try_wait") {
            Some(status) => {
                assert!(status.success(), "child cross-check exit: {status}");
                break;
            }
            None => assert!(Instant::now() < deadline, "child never exited"),
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// The send leg on a dead lane: with the peer gone, `sendmsg` is refused by
/// the OS and the shim maps errno through `Backend` — never a silent
/// success that would strand the queued fd. Bounded retries because AF_UNIX
/// may absorb one buffered frame before reporting EPIPE.
#[test]
fn sendmsg_on_a_dead_lane_surfaces_the_typed_backend_error() {
    let (a, b) = UnixStream::pair().expect("socketpair");
    drop(b);
    let mut blob = match MemfdBlob::create("modukit-fdpass-deadlane") {
        Ok(x) => x,
        Err(e) => {
            eprintln!("SKIPPING: memfd_create unavailable here: {e}");
            return;
        }
    };
    blob.send(b"x".to_vec()).expect("write the blob");
    let mut refusal = None;
    for _ in 0..3 {
        if let Err(e) = send_with_fd(&a, b"dead", blob.raw_fd().expect("open blob")) {
            refusal = Some(e);
            break;
        }
    }
    let err = refusal.expect("a dead lane must refuse within three sends");
    assert_eq!(err.code(), "CARRIER_UNREACHABLE", "{err}");
}

/// The recv leg under `set_read_timeout` on an empty queue: the shim is a
/// raw recvmsg, so the OS EAGAIN surfaces as the typed Backend error
/// (documented in the verb's `# Errors` — it is NOT mapped to Closed).
#[test]
fn a_recv_timeout_on_an_empty_lane_is_the_typed_backend_error() {
    let (_a, b) = UnixStream::pair().expect("socketpair");
    b.set_read_timeout(Some(Duration::from_millis(50)))
        .expect("short recv bound");
    let mut buf = [0u8; 20];
    let err = recv_with_fd(&b, &mut buf).expect_err("an empty queue must time out");
    assert_eq!(err.code(), "CARRIER_UNREACHABLE", "{err}");
}

/// Truncation honesty gate, reached the only way the public API allows:
/// a non-conforming peer that attaches more ancillary space than this shim's
/// fixed 24-byte cbuf (seven SCM_RIGHTS ints). The kernel sets MSG_CTRUNC;
/// the refusal must name it — a half-received descriptor is never handed
/// back. (The raw sender mirrors the CHILD_SRC shim layout already documented
/// in this file; the fds the kernel did fit into the cbuf leak inside the
/// test process — bounded, reclaimed at exit — the crate's own sender never
/// produces a message that can truncate.)
#[test]
fn an_oversized_ancillary_block_is_refused_as_ctrunc() {
    #[repr(C)]
    struct CIovec {
        base: *mut std::ffi::c_void,
        len: usize,
    }
    #[repr(C)]
    struct CHdr {
        name: *mut std::ffi::c_void,
        namelen: std::ffi::c_int,
        iov: *mut CIovec,
        iovlen: usize,
        control: *mut std::ffi::c_void,
        controllen: usize,
        flags: std::ffi::c_int,
    }
    unsafe extern "C" {
        fn sendmsg(fd: std::ffi::c_int, hdr: *const CHdr, flags: std::ffi::c_int) -> isize;
    }

    use std::os::fd::AsRawFd;
    let (a, b) = UnixStream::pair().expect("socketpair");
    let nulls: Vec<std::fs::File> = (0..7)
        .map(|_| std::fs::File::open("/dev/null").expect("open /dev/null"))
        .collect();
    // One cmsg header carrying all seven ints: CMSG_LEN(28) = 44 > the 24 the
    // receiver reserved, so the kernel truncates the control channel.
    let mut cbuf = vec![0u8; 16 + 4 * nulls.len()];
    cbuf[0..8].copy_from_slice(&(16 + 4 * nulls.len()).to_ne_bytes());
    cbuf[8..12].copy_from_slice(&1i32.to_ne_bytes()); // SOL_SOCKET
    cbuf[12..16].copy_from_slice(&1i32.to_ne_bytes()); // SCM_RIGHTS
    for (i, f) in nulls.iter().enumerate() {
        let o = 16 + 4 * i;
        cbuf[o..o + 4].copy_from_slice(&f.as_raw_fd().to_ne_bytes());
    }
    let data = meta_bytes(1, 1, 1);
    let mut iov = CIovec {
        base: data.as_ptr().cast_mut().cast(),
        len: data.len(),
    };
    let hdr = CHdr {
        name: std::ptr::null_mut(),
        namelen: 0,
        iov: &mut iov,
        iovlen: 1,
        control: cbuf.as_mut_ptr().cast(),
        controllen: cbuf.len(),
        flags: 0,
    };
    // SAFETY: connected AF_UNIX socket; the iovec and cbuf outlive the call
    // (same layout law the CHILD_SRC shim documents).
    let sent = unsafe { sendmsg(a.as_raw_fd(), &hdr, 0) };
    assert_eq!(sent as usize, data.len(), "the raw peer frame went out");

    let mut buf = [0u8; 20];
    b.set_read_timeout(Some(CROSS_PROC_TIMEOUT))
        .expect("recv bound");
    let err = recv_with_fd(&b, &mut buf).expect_err("44 > 24 must truncate");
    assert_eq!(err.code(), "CARRIER_UNREACHABLE", "{err}");
    assert!(format!("{err}").contains("truncated"), "{err}");
}

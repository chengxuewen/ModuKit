//! SCM_RIGHTS descriptor passing over a connected `UnixStream` — the fd
//! courier of the local-frames slot (D7/D14, modules/07 §3: "the pool fd is
//! handed over exactly once via UDS (SCM_RIGHTS) — slot 1) remains the only
//! fd courier"). A memfd blob's descriptor genuinely crosses a process
//! boundary: one small data frame (the self-sufficient SlotHandle meta,
//! D14) plus the fd riding as ancillary data on the same message.
//!
//! The `sendmsg`/`recvmsg` FFI shims follow the M5 `memfd_create` precedent
//! in `memfd.rs`: a small `extern "C"` declaration against the already-
//! linked libc (no `libc` crate — the vendor-lock floor stays green, an
//! extern block is not a dependency), documented `# Safety` at each call,
//! and errno captured via `io::Error::last_os_error()` — no silent fallback.
//!
//! Layout law (shortcut: Linux-only, x86_64/`LP64`): the `#[repr(C)]`
//! structs below mirror `struct msghdr`/`struct iovec`/`struct cmsghdr` as
//! the kernel and glibc define them on this platform (`msg_control` space
//! is `size_t`, `CMSG_LEN(4) = 20`, `CMSG_SPACE(4) = 24`). A non-Linux
//! build must grow a PAL leg before it gets here; nothing pretends today.

use std::ffi::{c_int, c_void};
use std::io;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::os::unix::net::UnixStream;

use crate::error::TransportError;

/// `SOL_SOCKET` — the cmsg level for SCM_* ancillary data.
const SOL_SOCKET: c_int = 1;
/// `SCM_RIGHTS` — the cmsg type that carries descriptors.
const SCM_RIGHTS: c_int = 1;
/// `MSG_CTRUNC` — ancillary data was truncated: the fd is LOST, never
/// silently half-received (the refusal below is the only honest answer).
const MSG_CTRUNC: c_int = 0x8;

/// `sizeof(struct cmsghdr)` on this LP64 platform (`size_t` + two `int`s,
/// aligned).
const CMSG_HDR_LEN: usize = 16;
/// `CMSG_LEN(sizeof(int))` — header plus the one descriptor we carry.
const CMSG_FD_LEN: usize = CMSG_HDR_LEN + 4;
/// `CMSG_SPACE(sizeof(int))` — the 8-aligned buffer the kernel expects.
const CMSG_FD_SPACE: usize = 24;

/// `struct iovec` (LP64 layout: pointer + `size_t`).
#[repr(C)]
struct CRawIovec {
    base: *mut c_void,
    len: usize,
}

/// `struct msghdr` (LP64 layout; field order and widths are the kernel's
/// `struct mmsghdr`-prefix contract — Rust's `repr(C)` padding matches).
#[repr(C)]
struct CRawMsgHdr {
    msg_name: *mut c_void,
    msg_namelen: c_int,
    msg_iov: *mut CRawIovec,
    msg_iovlen: usize,
    msg_control: *mut c_void,
    msg_controllen: usize,
    msg_flags: c_int,
}

unsafe extern "C" {
    /// `ssize_t sendmsg(int sockfd, const struct msghdr *msg, int flags)`.
    fn sendmsg(sockfd: c_int, msg: *const CRawMsgHdr, flags: c_int) -> isize;
    /// `ssize_t recvmsg(int sockfd, struct msghdr *msg, int flags)`.
    fn recvmsg(sockfd: c_int, msg: *mut CRawMsgHdr, flags: c_int) -> isize;
}

/// Send `data` with one descriptor riding the same message (SCM_RIGHTS).
/// The data frame is the two-stage meta of D14 — keep it small; the bytes
/// it describes move through the fd, not the control channel (04 §2).
///
/// # Errors
///
/// `TransportError::Backend` (`sendmsg`) on an OS refusal, and on a short
/// send — ancillary data attaches to the FIRST byte of a message, so a
/// partial write would strand the fd; it is refused, not retried
/// (shortcut: connected SOCK_STREAM delivers small frames whole; a lane
/// that must carry big data belongs above this verb, not inside it).
pub fn send_with_fd(sock: &UnixStream, data: &[u8], fd: RawFd) -> Result<(), TransportError> {
    // The cmsg buffer is the kernel's wire layout, written by hand per the
    // constants above: cmsg_len | cmsg_level | cmsg_type | the fd.
    let mut cbuf = [0u8; CMSG_FD_SPACE];
    cbuf[0..8].copy_from_slice(&CMSG_FD_LEN.to_ne_bytes());
    cbuf[8..12].copy_from_slice(&SOL_SOCKET.to_ne_bytes());
    cbuf[12..16].copy_from_slice(&SCM_RIGHTS.to_ne_bytes());
    cbuf[16..20].copy_from_slice(&fd.to_ne_bytes());
    let mut iov = CRawIovec {
        base: data.as_ptr().cast_mut().cast::<c_void>(),
        len: data.len(),
    };
    let hdr = CRawMsgHdr {
        msg_name: std::ptr::null_mut(),
        msg_namelen: 0,
        msg_iov: &mut iov,
        msg_iovlen: 1,
        msg_control: cbuf.as_mut_ptr().cast::<c_void>(),
        msg_controllen: CMSG_FD_SPACE,
        msg_flags: 0,
    };
    // SAFETY: `sock` is a connected AF_UNIX stream socket; the iovec points
    // at readable bytes for `len` and the cmsg buffer matches the kernel's
    // `cmsghdr` layout above; both outlive the synchronous call. The fd is
    // borrowed only (the SENDER keeps ownership; SCM_RIGHTS dups it into
    // the receiver at queue time).
    let sent = unsafe { sendmsg(sock.as_raw_fd(), &hdr, 0) };
    if sent < 0 {
        return Err(os_err("sendmsg", &io::Error::last_os_error()));
    }
    if (sent as usize) < data.len() {
        return Err(TransportError::Backend {
            syscall: "sendmsg",
            reason: "short send with ancillary data — the fd was not queued".to_string(),
        });
    }
    Ok(())
}

/// Receive one message into `buf`, collecting the descriptor it carries.
/// Returns `(bytes read, the received fd)`. The `OwnedFd` transfers
/// ownership of a descriptor that names the SENDER'S open file description
/// — the read/write offset is SHARED with the sender (that is what
/// SCM_RIGHTS passes). The reader therefore positions itself from the
/// token it just decoded (D14: meta first, then `seek`/`base + slot*cell`),
/// never assuming a fresh offset.
///
/// Honesty gates (no silent fallback): `MSG_CTRUNC` (ancillary truncated —
/// the fd would be lost) and a message carrying no SCM_RIGHTS cmsg are
/// both refused with `TransportError::Backend`; a timed-out read reports
/// the OS `EAGAIN`/`WouldBlock` from the stream's `set_read_timeout`.
///
/// # Errors
///
/// `TransportError::Backend` (`recvmsg`) as above; `UnexpectedEof`-shaped
/// peer death surfaces as the OS error string (`TransportError::Closed`
/// is not claimed here — a stream EOF is a lost lane, mapped by callers).
pub fn recv_with_fd(sock: &UnixStream, buf: &mut [u8]) -> Result<(usize, OwnedFd), TransportError> {
    let mut cbuf = [0u8; CMSG_FD_SPACE];
    let mut iov = CRawIovec {
        base: buf.as_mut_ptr().cast::<c_void>(),
        len: buf.len(),
    };
    let mut hdr = CRawMsgHdr {
        msg_name: std::ptr::null_mut(),
        msg_namelen: 0,
        msg_iov: &mut iov,
        msg_iovlen: 1,
        msg_control: cbuf.as_mut_ptr().cast::<c_void>(),
        msg_controllen: CMSG_FD_SPACE,
        msg_flags: 0,
    };
    // SAFETY: connected AF_UNIX stream socket; the buffers outlive the
    // synchronous (or SO_RCVTIMEO-bounded) call and match the kernel's
    // `msghdr`/`cmsghdr` layout documented above.
    let got = unsafe { recvmsg(sock.as_raw_fd(), &mut hdr, 0) };
    if got < 0 {
        return Err(os_err("recvmsg", &io::Error::last_os_error()));
    }
    if hdr.msg_flags & MSG_CTRUNC != 0 {
        return Err(TransportError::Backend {
            syscall: "recvmsg",
            reason: "ancillary data truncated (MSG_CTRUNC) — the descriptor was lost".to_string(),
        });
    }
    if hdr.msg_controllen < CMSG_FD_LEN {
        return Err(TransportError::Backend {
            syscall: "recvmsg",
            reason: "message carried no SCM_RIGHTS descriptor".to_string(),
        });
    }
    // Read the cmsg fields back by their documented offsets.
    let cmsg_len = usize::from_ne_bytes(cbuf[0..8].try_into().unwrap_or([0; 8]));
    let cmsg_level = c_int::from_ne_bytes(cbuf[8..12].try_into().unwrap_or([0; 4]));
    let cmsg_type = c_int::from_ne_bytes(cbuf[12..16].try_into().unwrap_or([0; 4]));
    if cmsg_len < CMSG_FD_LEN || cmsg_level != SOL_SOCKET || cmsg_type != SCM_RIGHTS {
        return Err(TransportError::Backend {
            syscall: "recvmsg",
            reason: "ancillary data is not an SCM_RIGHTS descriptor".to_string(),
        });
    }
    let raw = i32::from_ne_bytes(cbuf[16..20].try_into().unwrap_or([0; 4]));
    if raw < 0 {
        return Err(TransportError::Backend {
            syscall: "recvmsg",
            reason: "SCM_RIGHTS carried a negative descriptor".to_string(),
        });
    }
    // SAFETY: the kernel queued a fresh reference to this descriptor into
    // this process's file table on the successful recvmsg above; nothing
    // else owns it, so wrapping it in `OwnedFd` transfers a unique
    // ownership (it closes on drop).
    let owned = unsafe { OwnedFd::from_raw_fd(raw) };
    Ok((got as usize, owned))
}

fn os_err(syscall: &'static str, err: &io::Error) -> TransportError {
    TransportError::Backend {
        syscall,
        reason: err.to_string(),
    }
}

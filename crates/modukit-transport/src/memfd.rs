//! The memfd leg of the local-frames slot (D11 / modules/07 §2): a blob that
//! lives in an anonymous in-memory file (`memfd_create`), not a pipe. It is the
//! third `Transport` shape at M5 scale — `send` writes the blob, `recv` reads
//! the whole blob back (a single-slot lane: this is the buffer-moving leg, not
//! the FIFO message lane the socketpair carries).
//!
//! The `memfd_create` FFI shim is the M3 `dlopen` pattern (a small `extern "C"`
//! declaration against the already-linked libc, no `libc` crate), documented
//! `# Safety` at the call.

use std::ffi::{CString, c_char, c_int, c_uint};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::fd::{AsRawFd, FromRawFd, RawFd};

use crate::error::TransportError;
use crate::seam::Transport;

/// `MFD_CLOEXEC` — set close-on-exec so the descriptor is not leaked into child
/// processes. The C header value, kept as a named constant (no magic number).
const MFD_CLOEXEC: c_uint = 1;

unsafe extern "C" {
    /// `int memfd_create(const char *name, unsigned int flags)` — glibc ≥ 2.27.
    fn memfd_create(name: *const c_char, flags: c_uint) -> c_int;
}

/// A single blob living in a memfd, labelled by `name` (the label shows up in
/// `/proc/self/fd`, which is what "by name" means for an anonymous fd — there
/// is no path to open; cross-process handoff rides the seam's fd-passing leg).
pub struct MemfdBlob {
    name: String,
    file: Option<File>,
}

impl MemfdBlob {
    /// Create a fresh memfd labelled `name`.
    ///
    /// # Errors
    ///
    /// `TransportError::Backend` if `memfd_create` is unsupported here or the
    /// name contains an interior NUL.
    pub fn create(name: &str) -> Result<Self, TransportError> {
        let c_name = CString::new(name).map_err(|_| TransportError::Backend {
            syscall: "memfd_create",
            reason: "name contains an interior NUL".to_string(),
        })?;
        // SAFETY: `c_name` is a valid NUL-terminated string that outlives the
        // call; `memfd_create` reads it and returns a new owned descriptor (or
        // -1). We only wrap a non-negative fd.
        let fd = unsafe { memfd_create(c_name.as_ptr(), MFD_CLOEXEC) };
        if fd < 0 {
            let reason = std::io::Error::last_os_error().to_string();
            return Err(TransportError::Backend {
                syscall: "memfd_create",
                reason,
            });
        }
        // SAFETY: `fd` is a freshly-created, valid descriptor that nothing else
        // owns; wrapping it in `File` transfers ownership (it closes on drop).
        let file = unsafe { File::from_raw_fd(fd) };
        Ok(Self {
            name: name.to_string(),
            file: Some(file),
        })
    }

    /// The `memfd_create` label.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The underlying descriptor while the blob is open — the fd the
    /// `fdpass` lane rides (modules/07 §3: the fd is handed over exactly
    /// once and the SENDER keeps its own copy alive).
    ///
    /// # Errors
    ///
    /// `TransportError::Closed` after `close()` dropped the file.
    pub fn raw_fd(&self) -> Result<RawFd, TransportError> {
        self.file
            .as_ref()
            .map(|f| f.as_raw_fd())
            .ok_or(TransportError::Closed)
    }

    fn file_mut(&mut self) -> Result<&mut File, TransportError> {
        self.file.as_mut().ok_or(TransportError::Closed)
    }

    /// Replace the blob contents (truncate, write from offset 0).
    ///
    /// # Errors
    ///
    /// `TransportError` on a closed blob or an OS write failure.
    pub fn write_blob(&mut self, bytes: &[u8]) -> Result<(), TransportError> {
        let file = self.file_mut()?;
        file.set_len(0)
            .map_err(|e| os_err("ftruncate", &e.to_string()))?;
        file.seek(SeekFrom::Start(0))
            .map_err(|e| os_err("lseek", &e.to_string()))?;
        file.write_all(bytes)
            .map_err(|e| os_err("write", &e.to_string()))?;
        Ok(())
    }

    /// Read the whole blob back (from offset 0).
    ///
    /// # Errors
    ///
    /// `TransportError` on a closed blob or an OS read failure.
    pub fn read_blob(&mut self) -> Result<Vec<u8>, TransportError> {
        let file = self.file_mut()?;
        file.seek(SeekFrom::Start(0))
            .map_err(|e| os_err("lseek", &e.to_string()))?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)
            .map_err(|e| os_err("read", &e.to_string()))?;
        Ok(buf)
    }
}

impl Transport for MemfdBlob {
    fn connect(&mut self) -> Result<(), TransportError> {
        if self.file.is_none() {
            *self = MemfdBlob::create(&self.name)?;
        }
        Ok(())
    }

    fn send(&mut self, msg: Vec<u8>) -> Result<(), TransportError> {
        self.write_blob(&msg)
    }

    fn recv(&mut self) -> Result<Vec<u8>, TransportError> {
        self.read_blob()
    }

    fn close(&mut self) -> Result<(), TransportError> {
        self.file = None;
        Ok(())
    }
}

fn os_err(syscall: &'static str, reason: &str) -> TransportError {
    TransportError::Backend {
        syscall,
        reason: reason.to_string(),
    }
}

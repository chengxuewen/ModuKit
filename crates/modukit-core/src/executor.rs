//! The executor seam (D13 dl side; 01 §1 closed set) with the one Stage-1
//! implementation: `DlopenExecutor`.
//!
//! `Executor::load` validates the handshake without executing plugin service
//! logic; the lifecycle verbs live on the returned boxed handle (modules/03
//! §2: `RESOLVING -> STARTING` is the load+handshake edge, `start()` runs
//! under `STARTING`). `exe`/`wasm` arms of the closed set are Stage-2/5 and
//! intentionally absent here (architecture.md §10 out list).

use std::path::Path;

use libloading::Library;

use crate::abi::{ABI_VERSION, ENTRY_SYMBOL, HostAbiV1, PluginAbiV1, PluginInitFn};
use crate::error::KernelError;
use crate::manifest::{Manifest, PlacementKind};

/// Verbs every executor answers for one artifact (D5 surgery point 1).
pub trait Executor {
    /// Read the handshake and validate the guards; runs no service logic.
    ///
    /// # Errors
    ///
    /// `KernelError` handshake/load family (ABI_MISMATCH, NOT_FOUND).
    fn load(&self, manifest: &Manifest) -> Result<Box<dyn LoadedPlugin + Send>, KernelError>;
}

/// A plugin loaded by an executor: `start` / `stop` drive the plugin-side
/// lifecycle; dropping the handle releases the artifact (for `dl`: the
/// library, i.e. `dlclose`).
pub trait LoadedPlugin {
    /// The descriptor's human name.
    fn plugin_name(&self) -> &str;

    /// Call the plugin `start` with the host table. The host pointer is
    /// valid only for the call (01 §3 rule 2: no ambient authority).
    ///
    /// # Errors
    ///
    /// `KernelError::PluginStartFailed` on a non-zero status (03 §2).
    fn start(&mut self, host: *const HostAbiV1) -> Result<(), KernelError>;

    /// Cooperative stop (01 §3 rule 6: idempotence is the plugin's duty).
    ///
    /// # Errors
    ///
    /// `KernelError::PluginStartFailed`-shaped propagation of a non-zero
    /// status (stop path reuses the same typed channel).
    fn stop(&mut self) -> Result<(), KernelError>;

    /// Stable address hint inside the loaded image for the `ModuleLoaded`
    /// event (10 §2). `dl` reports the descriptor address; executors with
    /// no image answer 0.
    #[must_use]
    fn load_base(&self) -> u64 {
        0
    }
}

/// The `dl` executor: `libloading` + the ABI v1 handshake of `abi.rs`.
#[derive(Debug, Default)]
pub struct DlopenExecutor;

impl DlopenExecutor {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

/// A loaded `plugin.so`. The descriptor pointer is only dereferenced while
/// `_lib` is alive inside this same struct (drop order: `desc` is a raw
/// pointer and has no destructor; methods stop before drop).
struct DlopenPlugin {
    _lib: Library,
    desc: *const PluginAbiV1,
    name: String,
}

// SAFETY: a loaded library handle is process-global; this wrapper is kept
// inside the kernel's `Mutex`-guarded plugin table and only driven from the
// calling thread. The descriptor lives in the library image, which the
// `Library` keeps mapped.
unsafe impl Send for DlopenPlugin {}

impl DlopenPlugin {
    /// 04 §3 guards, run before any call: version integer + struct size.
    fn validate(desc: &PluginAbiV1) -> Result<(), KernelError> {
        if desc.abi_version != ABI_VERSION {
            return Err(KernelError::AbiMismatch {
                expected: ABI_VERSION,
                found: desc.abi_version,
            });
        }
        if desc.size < PluginAbiV1::expected_size() {
            return Err(KernelError::DescriptorTooSmall {
                expected: PluginAbiV1::expected_size(),
                found: desc.size,
            });
        }
        if desc.start.is_none() {
            return Err(KernelError::HandshakeIncomplete { what: "start" });
        }
        if desc.stop.is_none() {
            return Err(KernelError::HandshakeIncomplete { what: "stop" });
        }
        Ok(())
    }

    fn cstr(desc_ptr: *const std::ffi::c_char) -> String {
        if desc_ptr.is_null() {
            return String::from("<null>");
        }
        // SAFETY: caller passed a NUL-terminated C string inside the
        // library image (mapped while `_lib` lives).
        let s = unsafe { std::ffi::CStr::from_ptr(desc_ptr) };
        s.to_string_lossy().into_owned()
    }
}

impl Executor for DlopenExecutor {
    fn load(&self, manifest: &Manifest) -> Result<Box<dyn LoadedPlugin + Send>, KernelError> {
        if manifest.entry.kind != PlacementKind::Dl {
            return Err(KernelError::UnsupportedPlacement {
                kind: manifest.entry.kind.to_string(),
            });
        }
        let path = manifest.entry.artifact.as_str();
        if !Path::new(path).is_file() {
            return Err(KernelError::ArtifactMissing {
                path: path.to_string(),
            });
        }
        // A successful dlopen is NOT a compatibility test (04 §3, rutis'
        // declared anti-goal) - every guard below runs before any call.
        // SAFETY: loading native code runs initializers from a file path
        // supplied by the deploy manifest (trusted internal code at the dl
        // tier, 01 §5); the caller-reviewed guards follow immediately.
        let lib = unsafe { Library::new(path) }.map_err(|e| KernelError::LoadFailed {
            path: path.to_string(),
            reason: e.to_string(),
        })?;
        // SAFETY: the symbol's declared type is the ABI v1 entry signature;
        // the name is a NUL-terminated literal and `lib` outlives `_lib`.
        let init: libloading::Symbol<'_, PluginInitFn> =
            unsafe { lib.get(ENTRY_SYMBOL) }.map_err(|_| KernelError::SymbolMissing {
                path: path.to_string(),
            })?;
        let desc: *const PluginAbiV1 = init();
        if desc.is_null() {
            return Err(KernelError::HandshakeNull);
        }
        // SAFETY: `desc` non-null and points into the mapped image we just
        // obtained from the same library (kept alive by `lib`).
        let validated = unsafe { &*desc };
        DlopenPlugin::validate(validated)?;
        let name = DlopenPlugin::cstr(validated.name);
        // Copy the descriptor struct out of the image: reads stay valid for
        // the wrapper's whole life, and the raw fn pointers are owned by the
        // mapping we hold.
        Ok(Box::new(DlopenPlugin {
            _lib: lib,
            desc,
            name,
        }))
    }
}

impl LoadedPlugin for DlopenPlugin {
    fn plugin_name(&self) -> &str {
        &self.name
    }

    fn load_base(&self) -> u64 {
        // v0 hint: the descriptor lives in the mapped image; its address
        // is stable for the library's lifetime (debug adapters only need a
        // per-module distinct anchor until a real base-address query lands).
        let ptr: *const PluginAbiV1 = self.desc;
        ptr as usize as u64
    }

    fn start(&mut self, host: *const HostAbiV1) -> Result<(), KernelError> {
        // SAFETY: `_lib` keeps the image mapped; validate() at load time
        // guarantees start is Some and the guards passed before any call.
        let status = unsafe {
            let start = (*self.desc)
                .start
                .ok_or(KernelError::HandshakeIncomplete { what: "start" })?;
            start(host)
        };
        if status != 0 {
            return Err(KernelError::PluginStartFailed { status });
        }
        Ok(())
    }

    fn stop(&mut self) -> Result<(), KernelError> {
        // SAFETY: as above for stop; image mapped while `_lib` is alive.
        let status = unsafe {
            let stop = (*self.desc)
                .stop
                .ok_or(KernelError::HandshakeIncomplete { what: "stop" })?;
            stop()
        };
        if status != 0 {
            return Err(KernelError::PluginStartFailed { status });
        }
        Ok(())
    }
}

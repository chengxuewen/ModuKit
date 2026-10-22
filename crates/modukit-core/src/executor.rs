//! The executor seam (D13; 01 §1 closed set): the Stage-1 `dl` side here,
//! the S2-M1 `exe` side next door (re-exported below).
//! implementation: `DlopenExecutor`.
//!
//! `Executor::load` validates the handshake without executing plugin service
//! logic; the lifecycle verbs live on the returned boxed handle (modules/03
//! §2: `RESOLVING -> STARTING` is the load+handshake edge, `start()` runs
//! under `STARTING`). The `exe` arm of the closed set lives next door —
//! `executor_proc`/`proc_supervisor`, re-exported below (S2-M1); the `wasm`
//! arm is Stage-5 (architecture.md §10 out list).

use std::path::Path;

use libloading::Library;

use crate::abi::{ABI_VERSION, ENTRY_SYMBOL, HostAbiV1, PluginAbiV1, PluginInitFn};
use crate::error::KernelError;
use crate::manifest::{Manifest, PlacementKind};

// S2-M1: the `exe` branch of the seam, re-exported so this module stays
// the one front door for executors (lib.rs surface unchanged by module).
pub use crate::executor_proc::{
    CALL_RESULT_TIMEOUT, DIAL_TIMEOUT, HANDSHAKE_TIMEOUT, ProcessExecutor, START_ACK_TIMEOUT,
    STOP_KILL_GRACE,
};
pub use crate::proc_proxy::ProxyService;
pub use crate::proc_supervisor::{Revive, Supervisor};

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

    /// The ABI major the artifact's descriptor declared (D96 N1 v2 door).
    /// The v1 default keeps every non-`dl` executor on the v1 host table.
    #[must_use]
    fn abi_major(&self) -> u32 {
        crate::abi::ABI_VERSION
    }

    /// Stable address hint inside the loaded image for the `ModuleLoaded`
    /// event (10 §2). `dl` reports the descriptor address; executors with
    /// no image answer 0.
    #[must_use]
    fn load_base(&self) -> u64 {
        0
    }

    /// The descriptor's self-report strings for the D7 cross-check
    /// (M7): `(id, version)` as the artifact declares them. Executors
    /// without a self-report answer `(None, None)` and the cross-check
    /// treats the silence as disagreement.
    #[must_use]
    fn self_report(&self) -> (Option<String>, Option<String>) {
        (None, None)
    }
}

/// The `dl` executor: `libloading` + the ABI v1 handshake of `abi.rs`.
#[derive(Debug, Default)]
pub struct DlopenExecutor;

/// The self-report a probe reads from an artifact without running any service
/// logic (D83 additive door): the descriptor's human name plus the D7
/// id/version strings the artifact declares. `None` id/version mirrors the
/// silent-descriptor case the `LoadedPlugin::self_report` default answers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelfReport {
    pub plugin_name: String,
    pub id: Option<String>,
    pub version: Option<String>,
}

impl DlopenExecutor {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// The shared `dl` artifact guard sequence (04 §3) - the single source of
    /// truth for the ABI v1 guards. present-file, dlopen, entry symbol,
    /// non-null descriptor, version/size/function pointers, then read the
    /// name. `load` runs EXACTLY this and boxes the result;
    /// `probe_self_report` runs the same and reads the self-report out before
    /// releasing the handle. A successful dlopen is NOT a compatibility test
    /// (04 §3, rutis' declared anti-goal) - every guard runs before any call.
    fn open_validated(path: &str) -> Result<DlopenPlugin, KernelError> {
        if !Path::new(path).is_file() {
            return Err(KernelError::ArtifactMissing {
                path: path.to_string(),
            });
        }
        // SAFETY: loading native code runs initializers from a file path
        // supplied by the caller (trusted internal code at the dl tier, 01 §5);
        // the descriptor guards below run before any plugin call.
        //
        // RTLD_NODELETE (M2 finding, linux): the probe-then-load doors (the
        // runner's self-report probe, the bundle cross-check) DROP the handle
        // before the real load. Without NODELETE the unmap tears the image
        // down and a later re-load re-runs C++ static initializers - the
        // protobuf global descriptor pool then SIGABRTs on double
        // registration (observed: the cpp fixture under the runner). The flag
        // pins ruling 4's words - unload is capability removal, code
        // residency is the documented fact - at the dlopen level, matching
        // POSIX's own "successful dlclose does not guarantee unload" contract.
        // macOS keeps plain dlopen: dyld has no RTLD_NODELETE; the double-load
        // hazard for C++ gencode there is recorded, unfixed, by ruling.
        #[cfg(target_os = "linux")]
        let lib = {
            // libloading re-exports RTLD_NOW(2)|RTLD_LOCAL(4) only; glibc and
            // musl both define RTLD_NODELETE as 0x1000 (dlfcn.h).
            const RTLD_NODELETE: i32 = 0x1000;
            let unix = unsafe {
                libloading::os::unix::Library::open(
                    Some(path),
                    libloading::os::unix::RTLD_NOW
                        | libloading::os::unix::RTLD_LOCAL
                        | RTLD_NODELETE,
                )
            }
            .map_err(|e| KernelError::LoadFailed {
                path: path.to_string(),
                reason: e.to_string(),
            })?;
            Library::from(unix)
        };
        #[cfg(not(target_os = "linux"))]
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
        Ok(DlopenPlugin {
            _lib: lib,
            desc,
            name,
            abi_major: validated.abi_version,
        })
    }

    /// Read an artifact's self-report through the SAME guard sequence as
    /// `Executor::load` (single source of truth) and release the handle before
    /// returning. It rejects exactly what `load` rejects, with the same typed
    /// `KernelError` - there is no second door with weaker semantics.
    ///
    /// # Errors
    ///
    /// The `KernelError` handshake/load family `load` runs: ArtifactMissing,
    /// LoadFailed, SymbolMissing, HandshakeNull, AbiMismatch,
    /// DescriptorTooSmall, HandshakeIncomplete.
    pub fn probe_self_report(artifact: &Path) -> Result<SelfReport, KernelError> {
        let plugin = Self::open_validated(&artifact.to_string_lossy())?;
        let (id, version) = plugin.self_report();
        let report = SelfReport {
            plugin_name: plugin.plugin_name().to_string(),
            id,
            version,
        };
        drop(plugin); // release the mapping before the report leaves
        Ok(report)
    }
}

/// A loaded `plugin.so`. Unloading is best-effort per the platform
/// (the linux door opens RTLD_NODELETE - see the residency law at the
/// dlopen site below); the descriptor pointer is only dereferenced while
/// `_lib` is alive inside this same struct (drop order: `desc` is a raw
/// pointer and has no destructor; methods stop before drop).
struct DlopenPlugin {
    _lib: Library,
    desc: *const PluginAbiV1,
    name: String,
    /// The ABI major the descriptor declared at the handshake (1 or 2).
    abi_major: u32,
}

// SAFETY: a loaded library handle is process-global; this wrapper is kept
// inside the kernel's `Mutex`-guarded plugin table and only driven from the
// calling thread. The descriptor lives in the library image, which the
// `Library` keeps mapped.
unsafe impl Send for DlopenPlugin {}

impl DlopenPlugin {
    /// 04 §3 guards, run before any call: version integer + struct size.
    fn validate(desc: &PluginAbiV1) -> Result<(), KernelError> {
        // N1 v2 door (D96 ruling 2): a declared major 2 asks for the
        // HostAbiV2 table. Purely additive - the v1 guard below is
        // byte-identical, so any other liar major still lands AbiMismatch.
        if desc.abi_version == crate::abi::ABI_VERSION_V2 {
            return Self::validate_v2(desc);
        }
        // Wave-2 T1 v3 door (D99 R2): a declared major 3 asks for the
        // HostAbiV3 table. Purely additive - mirrors the v2 arm above, so
        // any other liar major still lands AbiMismatch unchanged.
        if desc.abi_version == crate::abi::ABI_VERSION_V3 {
            return Self::validate_v3(desc);
        }
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

    /// v2 door guards: the descriptor shape is unchanged (the SECOND table
    /// is host-side only, D96 ruling 2), so the size/pointer guards are the
    /// v1 ones; only the version-equality guard becomes declared-major 2.
    fn validate_v2(desc: &PluginAbiV1) -> Result<(), KernelError> {
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

    /// v3 door guards (D99 R2): the descriptor shape is unchanged (the
    /// THIRD table is host-side only, recursing the D96 ruling-2 shape),
    /// so the size/pointer guards are the v1/v2 ones; only the version-
    /// equality guard becomes declared-major 3.
    fn validate_v3(desc: &PluginAbiV1) -> Result<(), KernelError> {
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

    /// Read a descriptor C string as `Option` (null -> `None`).
    ///
    /// # Safety
    /// `ptr` either null or a readable NUL-terminated C string inside a
    /// mapped image the caller keeps alive.
    unsafe fn cstr_opt(ptr: *const std::ffi::c_char) -> Option<String> {
        if ptr.is_null() {
            return None;
        }
        // SAFETY: caller guarantees the pointer contract above.

        let s = unsafe { std::ffi::CStr::from_ptr(ptr) };
        s.to_str().ok().map(str::to_string)
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
        // The artifact guards run in ONE place - `open_validated`, shared
        // verbatim with `probe_self_report` - so the two doors can never
        // diverge on what a bad artifact means.
        Ok(Box::new(Self::open_validated(
            manifest.entry.artifact.as_str(),
        )?))
    }
}

impl LoadedPlugin for DlopenPlugin {
    fn plugin_name(&self) -> &str {
        &self.name
    }

    fn abi_major(&self) -> u32 {
        self.abi_major
    }

    fn load_base(&self) -> u64 {
        // v0 hint: the descriptor lives in the mapped image; its address
        // is stable for the library's lifetime (debug adapters only need a
        // per-module distinct anchor until a real base-address query lands).
        let ptr: *const PluginAbiV1 = self.desc;
        ptr as usize as u64
    }

    fn self_report(&self) -> (Option<String>, Option<String>) {
        // SAFETY: `_lib` keeps the image mapped; `desc` points into it
        // and the descriptor was validated at load.
        let d = unsafe { &*self.desc };
        (unsafe { Self::cstr_opt(d.id) }, unsafe {
            Self::cstr_opt(d.version)
        })
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

#[cfg(test)]
mod v3_door_guard_tests {
    //! The liar cells of the wave-2 T1 plan §1, plus the exe interim pin,
    //! over the ONE shared guard sequence (`validate` via
    //! `open_validated` - the same door the child runner walks through
    //! `probe_self_report`, modukit-runtime/src/main.rs). Unit-level here
    //! because the shapes are pure descriptor arithmetic: no artifact is
    //! needed to prove what a false major 3 means at either door.

    use super::*;

    unsafe extern "C" fn stub_start(_host: *const HostAbiV1) -> i32 {
        0
    }

    unsafe extern "C" fn stub_stop() -> i32 {
        0
    }

    fn desc(abi_version: u32, size: u32, start: bool, stop: bool) -> PluginAbiV1 {
        PluginAbiV1 {
            abi_version,
            size,
            name: std::ptr::null(),
            id: std::ptr::null(),
            version: std::ptr::null(),
            start: if start { Some(stub_start) } else { None },
            stop: if stop { Some(stub_stop) } else { None },
        }
    }

    #[test]
    fn declared_major_3_valid_descriptor_passes_the_shared_door() {
        assert!(
            DlopenPlugin::validate(&desc(
                crate::abi::ABI_VERSION_V3,
                PluginAbiV1::expected_size(),
                true,
                true,
            ))
            .is_ok()
        );
    }

    #[test]
    fn false_major_3_keeps_the_descriptor_too_small_shape() {
        let err = DlopenPlugin::validate(&desc(
            crate::abi::ABI_VERSION_V3,
            PluginAbiV1::expected_size() - 1,
            true,
            true,
        ))
        .expect_err("a short v3 descriptor must be refused");
        assert!(
            matches!(err, KernelError::DescriptorTooSmall { .. }),
            "{err}"
        );
    }

    #[test]
    fn false_major_3_keeps_the_handshake_incomplete_shapes() {
        for (start, stop, what) in [(false, true, "start"), (true, false, "stop")] {
            let err = DlopenPlugin::validate(&desc(
                crate::abi::ABI_VERSION_V3,
                PluginAbiV1::expected_size(),
                start,
                stop,
            ))
            .expect_err("a null lifecycle slot must be refused");
            assert!(
                matches!(&err, KernelError::HandshakeIncomplete { what: w } if *w == what),
                "{err}",
            );
        }
    }

    #[test]
    fn unknown_major_4_stays_abi_mismatch_at_the_same_door() {
        // The v3 arm does not swallow the liar guard: an unknown major 4
        // falls through to the equality check exactly as majors 3-pre-T1
        // and 99 did before it (plan §1: the guard already returns it).
        let err = DlopenPlugin::validate(&desc(4, PluginAbiV1::expected_size(), true, true))
            .expect_err("major 4 is not a door this host publishes");
        assert!(
            matches!(
                err,
                KernelError::AbiMismatch {
                    expected: ABI_VERSION,
                    found: 4
                }
            ),
            "{err}"
        );
    }

    /// PINNED behavior, plan §1 exe item (review recast): the child runner
    /// loads through this SAME shared `validate` that T1 opens, so a v3
    /// plugin in an exe child passes the child's door and schedules on the
    /// child's own D74 clock BY CONSTRUCTION (R5-A local semantics). The
    /// honest T1 interim: child-local bells reach child-local subscribers
    /// only - the due-site bridge routing (parent->child and child->parent
    /// crossing) is lane T2, which also flips this pin. There is no
    /// refusal anywhere on the exe arm and the HELLO abi field is the
    /// runner's own self-report (=1), never the plugin's declared major.
    #[test]
    fn exe_interim_pin_shared_door_accepts_v3_and_no_refusal_exists() {
        // The same acceptance asserted above IS the pin: the shared door
        // opens v3 for the child runner too, and nothing on the exe arm
        // refuses or truncates a declared major 3.
        assert!(
            DlopenPlugin::validate(&desc(
                crate::abi::ABI_VERSION_V3,
                PluginAbiV1::expected_size(),
                true,
                true,
            ))
            .is_ok()
        );
    }
}

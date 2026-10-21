//! The exe rung of the embed face (Stage-4 M2, D83 tail-add; M3 appended
//! the service-call door): the eight `modukit_sup_*` verbs declared in
//! `contracts/modukit_embed.h`, served by a process-global [`Supervisor`]
//! - a second ambient kernel, separate from M1's in-process one.
//!
//! Two-rung contract (ruled; do not fake unity):
//! - `embed.rs` serves the dl rung (in-process plugins, `DlopenExecutor`).
//! - this module serves the exe rung (child-process plugins,
//!   `ProcessExecutor` + supervision). The two kernels own SEPARATE
//!   registries, buses, clocks and observatories: a service registered on
//!   one rung is NOT visible on the other, and a wrong-kind manifest is
//!   refused at the install door (MANIFEST_INVALID) on both faces.
//!   Registry-level unification is a future ruling.
//! - child machinery (spawn-door retry, `STOP_KILL_GRACE`, fd ownership,
//!   reaping) stays Rust-side: a C caller never sees a child fd or pid.
//!
//! Error channel, string-table ownership and the panic firewall follow the
//! exact `embed.rs` discipline (`codec::guard`, `codec::neg_kernel`,
//! [`crate::embed::registered_id`]).

use std::ffi::c_char;
use std::sync::{LazyLock, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use crate::embed::registered_id;
use crate::embed_codec::{self as codec, bytes_arg, copy_out, cstr_arg};
use crate::identity::PluginId;
use crate::manifest::Manifest;
use crate::proc_proxy::ProxyService;
use crate::proc_supervisor::{Revive, Supervisor};

struct AmbientSup(Supervisor);

// SAFETY: constructed exactly once here, exclusively from `Supervisor::new()`
// (real boot clock + `ProcessExecutor`, whose own state is Arc/Mutex
// guarded); the outer Mutex serializes all access, so no two threads ever
// hold the supervisor at once - the same justification as `embed::Ambient`.
unsafe impl Send for AmbientSup {}

static SUPERVISOR: LazyLock<Mutex<AmbientSup>> =
    LazyLock::new(|| Mutex::new(AmbientSup(Supervisor::new())));

fn supervisor() -> MutexGuard<'static, AmbientSup> {
    SUPERVISOR.lock().unwrap_or_else(PoisonError::into_inner)
}

#[unsafe(no_mangle)]
/// Rung doc in `contracts/modukit_embed.h`: one-call install -> resolve ->
/// start walk; first failing door's code returns, stage stays readable.
#[allow(clippy::not_unsafe_ptr_arg_deref)] // FFI: contract-pinned signature; derefs null-checked
pub extern "C" fn modukit_sup_install(
    manifest_toml: *const c_char,
    out_plugin_id: *mut *const c_char,
) -> i32 {
    codec::guard(|| {
        let (Some(toml), false) = (cstr_arg(manifest_toml), out_plugin_id.is_null()) else {
            return codec::NEG_SCHEMA_INVALID;
        };
        let manifest = match Manifest::from_toml_str(toml) {
            Ok(m) => m,
            Err(err) => return -codec::d51_number(err.code()),
        };
        let sup = supervisor();
        let kernel = sup.0.kernel();
        let id = match kernel.install(manifest) {
            Ok(id) => id,
            Err(err) => return codec::neg_kernel(&err),
        };
        if let Err(err) = kernel.resolve(&id).and_then(|()| kernel.start(&id)) {
            return codec::neg_kernel(&err);
        }
        drop(sup);
        match registered_id(id.as_str()) {
            Some(c) => {
                // SAFETY: `out_plugin_id` checked non-null above.
                unsafe { *out_plugin_id = c.as_ptr() };
                0
            }
            None => codec::NEG_SCHEMA_INVALID,
        }
    })
}

#[unsafe(no_mangle)]
/// Cooperative stop through the supervised kernel (grace-then-kill is
/// Rust-side machinery; this door just waits for its typed outcome).
pub extern "C" fn modukit_sup_stop(id: *const c_char) -> i32 {
    codec::guard(|| {
        let Some(s) = cstr_arg(id) else {
            return codec::NEG_SCHEMA_INVALID;
        };
        let sup = supervisor();
        match sup.0.kernel().stop(&PluginId::new(s.to_string())) {
            Ok(()) => 0,
            Err(err) => codec::neg_kernel(&err),
        }
    })
}

#[unsafe(no_mangle)]
/// State of a supervised id; same `modukit_plugin_state` codes.
#[allow(clippy::not_unsafe_ptr_arg_deref)] // FFI: contract-pinned signature; derefs null-checked
pub extern "C" fn modukit_sup_state(id: *const c_char, out_state: *mut i32) -> i32 {
    codec::guard(|| {
        if out_state.is_null() {
            return codec::NEG_SCHEMA_INVALID;
        }
        let Some(s) = cstr_arg(id) else {
            return codec::NEG_SCHEMA_INVALID;
        };
        let sup = supervisor();
        match sup.0.kernel().state(&PluginId::new(s.to_string())) {
            Ok(state) => {
                // SAFETY: `out_state` checked non-null above.
                unsafe { *out_state = codec::state_number(state) };
                0
            }
            Err(err) => codec::neg_kernel(&err),
        }
    })
}

#[unsafe(no_mangle)]
/// One supervisor pass. `*out_actions` counts children REAPED + children
/// REVIVED this pass; timer postings and park decisions ride the event
/// ring of this rung (see header comment - deliberate split).
#[allow(clippy::not_unsafe_ptr_arg_deref)] // FFI: contract-pinned signature; derefs null-checked
pub extern "C" fn modukit_sup_step(out_actions: *mut i32) -> i32 {
    codec::guard(|| {
        if out_actions.is_null() {
            return codec::NEG_SCHEMA_INVALID;
        }
        let sup = supervisor();
        let report = sup.0.step();
        // SAFETY: `out_actions` checked non-null above.
        unsafe {
            *out_actions = (report.crashed.len() + report.revived.len()) as i32;
        }
        0
    })
}

#[unsafe(no_mangle)]
/// Dial the child's step clock (tests/replay doorway). Due-job details
/// ride the event ring; essential answer = status code + fired stamp.
#[allow(clippy::not_unsafe_ptr_arg_deref)] // FFI: contract-pinned signature; derefs null-checked
pub extern "C" fn modukit_sup_dial(
    id: *const c_char,
    delta_ns: i64,
    out_status: *mut i32,
    out_fired_at_ns: *mut u64,
) -> i32 {
    codec::guard(|| {
        if delta_ns < 0 || out_status.is_null() || out_fired_at_ns.is_null() {
            return codec::NEG_SCHEMA_INVALID;
        }
        let Some(s) = cstr_arg(id) else {
            return codec::NEG_SCHEMA_INVALID;
        };
        let sup = supervisor();
        match sup.0.dial(
            &PluginId::new(s.to_string()),
            Duration::from_nanos(delta_ns as u64),
        ) {
            Ok(turn) => {
                // SAFETY: both out pointers checked non-null above.
                unsafe {
                    *out_status = turn.status;
                    *out_fired_at_ns = turn.fired_at_ns;
                }
                0
            }
            Err(err) => codec::neg_kernel(&err),
        }
    })
}

#[unsafe(no_mangle)]
/// One restart request per manifest policy (D3 budget + backoff window).
/// `*out_revive`: 0 Revived / 1 Deferred / 2 Parked (header-pinned map).
#[allow(clippy::not_unsafe_ptr_arg_deref)] // FFI: contract-pinned signature; derefs null-checked
pub extern "C" fn modukit_sup_restart(id: *const c_char, out_revive: *mut i32) -> i32 {
    codec::guard(|| {
        if out_revive.is_null() {
            return codec::NEG_SCHEMA_INVALID;
        }
        let Some(s) = cstr_arg(id) else {
            return codec::NEG_SCHEMA_INVALID;
        };
        let sup = supervisor();
        match sup.0.revive(&PluginId::new(s.to_string())) {
            Ok(outcome) => {
                // SAFETY: `out_revive` checked non-null above.
                unsafe {
                    *out_revive = match outcome {
                        Revive::Revived => 0,
                        Revive::Deferred => 1,
                        Revive::Parked => 2,
                    };
                }
                0
            }
            Err(err) => codec::neg_kernel(&err),
        }
    })
}

#[unsafe(no_mangle)]
/// Uninstall a STOPPED supervised record; ACTIVE ids get the typed
/// lifecycle refusal (stop owns child teardown, this door never kills).
pub extern "C" fn modukit_sup_uninstall(id: *const c_char) -> i32 {
    codec::guard(|| {
        let Some(s) = cstr_arg(id) else {
            return codec::NEG_SCHEMA_INVALID;
        };
        let sup = supervisor();
        match sup.0.kernel().uninstall(&PluginId::new(s.to_string())) {
            Ok(()) => 0,
            Err(err) => codec::neg_kernel(&err),
        }
    })
}

#[unsafe(no_mangle)]
/// The service-call door on the exe rung (D94 W2, M3 tail-add): same codec
/// rules as the dl face (`embed.rs` `modukit_service_call`) - opaque bytes
/// both ways, fail-rather-than-truncate cap, negated-D51 channel. It
/// reuses the parent-side machinery that already serves cross-process
/// calls for the exe rung today: a child's REGISTER frame installed a
/// `ProxyService` in THIS rung's kernel registry (`proc_proxy.rs`), so
/// the call rides the same acquire + canonical-id match + `invoke` door
/// the dual-placement proof exercises - no second dispatch path.
#[allow(clippy::not_unsafe_ptr_arg_deref)] // FFI: contract-pinned signature; derefs null-checked
pub extern "C" fn modukit_sup_service_call(
    service_id: *const c_char,
    method: *const c_char,
    input: *const u8,
    input_len: usize,
    response: *mut u8,
    response_cap: usize,
) -> i32 {
    codec::guard(|| {
        let (Some(sid), Some(method), Some(bytes)) = (
            cstr_arg(service_id),
            cstr_arg(method),
            bytes_arg(input, input_len),
        ) else {
            return codec::NEG_SCHEMA_INVALID;
        };
        if response.is_null() {
            return codec::NEG_SCHEMA_INVALID;
        }
        let sup = supervisor();
        let registry = sup.0.kernel().registry();
        // Type-keyed acquire + canonical-id match (the dl face's decision
        // 7 mirrored: a non-top-ranked proxy id answers NOT_FOUND).
        let Some(svc) = registry.acquire::<ProxyService>() else {
            return codec::NEG_NOT_FOUND;
        };
        if svc.service_id().as_str() != sid {
            let _ = registry.release(&svc);
            return codec::NEG_NOT_FOUND;
        }
        let answer = match svc.invoke(method, bytes) {
            // No out_len channel on this door: copy_out with a NULL size
            // slot answers the byte count / QUOTA_EXCEEDED (embed.rs
            // decision 4, same shape as the dl face).
            Ok(reply) => copy_out(&reply, response, response_cap, std::ptr::null_mut()),
            Err(err) => codec::neg_kernel(&err),
        };
        if registry.release(&svc).is_err() {
            return codec::NEG_KERNEL_BUG;
        }
        answer
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex as StdMutex;

    /// The supervisor ambient is process-global (separate from the dl
    /// ambient); tests that touch it serialize here - same discipline as
    /// embed_tests' AMBIENT_SERIAL, own lock because own ambient.
    static SUP_SERIAL: StdMutex<()> = StdMutex::new(());

    fn sup_lock() -> std::sync::MutexGuard<'static, ()> {
        SUP_SERIAL.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn c(s: &str) -> std::ffi::CString {
        std::ffi::CString::new(s).unwrap()
    }

    #[test]
    fn dl_manifest_on_the_sup_face_is_refused() {
        let _serial = sup_lock();
        let mut out: *const c_char = std::ptr::null();
        let rc = modukit_sup_install(
            c("id = \"com.t.sup.wrong\"\nversion = \"0.1.0\"\n[entry]\nkind = \"dl\"\nartifact = \"/nonexistent/x\"\n")
                .as_c_str()
                .as_ptr(),
            &raw mut out,
        );
        assert_eq!(
            rc,
            -codec::d51_number("MANIFEST_INVALID"),
            "install-door refusal"
        );
    }

    #[test]
    fn unknown_ids_answer_with_the_typed_channel() {
        let _serial = sup_lock();
        let ghost = c("com.t.sup.ghost");
        let code = -codec::d51_number("NOT_FOUND");
        let mut st = 0i32;
        assert_eq!(
            modukit_sup_state(ghost.as_c_str().as_ptr(), &raw mut st),
            code
        );
        assert_eq!(modukit_sup_stop(ghost.as_c_str().as_ptr()), code);
        assert_eq!(modukit_sup_uninstall(ghost.as_c_str().as_ptr()), code);
        let mut rv = 9i32;
        assert_eq!(
            modukit_sup_restart(ghost.as_c_str().as_ptr(), &raw mut rv),
            code
        );
    }

    #[test]
    fn schema_null_guards_answer_before_any_kernel_touch() {
        let _serial = sup_lock();
        assert_eq!(
            modukit_sup_stop(std::ptr::null()),
            codec::NEG_SCHEMA_INVALID
        );
        assert_eq!(
            modukit_sup_state(std::ptr::null(), std::ptr::null_mut()),
            codec::NEG_SCHEMA_INVALID
        );
        assert_eq!(
            modukit_sup_step(std::ptr::null_mut()),
            codec::NEG_SCHEMA_INVALID
        );
        let mut st = 0i32;
        let mut fs = 0u64;
        assert_eq!(
            modukit_sup_dial(
                c("com.t.nope").as_c_str().as_ptr(),
                -1,
                &raw mut st,
                &raw mut fs
            ),
            codec::NEG_SCHEMA_INVALID
        );
        assert_eq!(
            modukit_sup_restart(c("com.t.nope").as_c_str().as_ptr(), std::ptr::null_mut()),
            codec::NEG_SCHEMA_INVALID
        );
        assert_eq!(
            modukit_sup_uninstall(std::ptr::null()),
            codec::NEG_SCHEMA_INVALID
        );
        // service-call door: the shape refusals answer before any registry touch.
        let mut sb = [0u8; 8];
        assert_eq!(
            modukit_sup_service_call(
                std::ptr::null(),
                c("echo").as_c_str().as_ptr(),
                sb.as_ptr(),
                0,
                sb.as_mut_ptr(),
                8
            ),
            codec::NEG_SCHEMA_INVALID,
            "null service id is a shape refusal"
        );
        assert_eq!(
            modukit_sup_service_call(
                c("com.t.nope").as_c_str().as_ptr(),
                std::ptr::null(),
                sb.as_ptr(),
                0,
                sb.as_mut_ptr(),
                8
            ),
            codec::NEG_SCHEMA_INVALID,
            "null method is a shape refusal"
        );
        assert_eq!(
            modukit_sup_service_call(
                c("com.t.nope").as_c_str().as_ptr(),
                c("echo").as_c_str().as_ptr(),
                std::ptr::null(),
                4,
                sb.as_mut_ptr(),
                8
            ),
            codec::NEG_SCHEMA_INVALID,
            "len > 0 with a null input is a shape lie"
        );
        assert_eq!(
            modukit_sup_service_call(
                c("com.t.nope").as_c_str().as_ptr(),
                c("echo").as_c_str().as_ptr(),
                sb.as_ptr(),
                2,
                std::ptr::null_mut(),
                8
            ),
            codec::NEG_SCHEMA_INVALID,
            "null response buffer is refused (no size-query on this door)"
        );
    }

    #[test]
    fn sup_service_call_answers_not_found_without_a_matching_provider() {
        let _serial = sup_lock();
        // No supervised child lives on this ambient in unit context, so the
        // proxy door answers the typed NOT_FOUND(-14) for both an empty and
        // a non-empty payload - same channel as the dl face's own door.
        let ghost = c("com.t.sup.ghost.service");
        let method = c("echo");
        let input = *b"hi";
        let mut out = [0u8; 16];
        assert_eq!(
            modukit_sup_service_call(
                ghost.as_c_str().as_ptr(),
                method.as_c_str().as_ptr(),
                input.as_ptr(),
                2,
                out.as_mut_ptr(),
                16
            ),
            codec::NEG_NOT_FOUND,
            "non-empty payload, unknown id -> NOT_FOUND"
        );
        assert_eq!(
            modukit_sup_service_call(
                ghost.as_c_str().as_ptr(),
                method.as_c_str().as_ptr(),
                std::ptr::null(),
                0,
                out.as_mut_ptr(),
                16
            ),
            codec::NEG_NOT_FOUND,
            "empty payload is legal input; the door still answers NOT_FOUND"
        );
    }

    #[test]
    fn exe_manifest_on_the_dl_face_is_refused_at_the_door() {
        // The wall from the other side: the install-door placement gate
        // answers before any record is created - symmetric refusals.
        let _serial = sup_lock();
        let mut out: *const c_char = std::ptr::null();
        let rc = crate::embed::modukit_install(
            c("id = \"com.t.dl.wrong\"\nversion = \"0.1.0\"\n[entry]\nkind = \"exe\"\nartifact = \"/nonexistent/x\"\nrunner = \"/nonexistent/y\"\n")
                .as_c_str()
                .as_ptr(),
            &raw mut out,
        );
        assert_eq!(
            rc,
            -codec::d51_number("MANIFEST_INVALID"),
            "same refusal, this side"
        );
    }

    #[test]
    fn step_on_an_idle_supervisor_reports_zero_actions() {
        let _serial = sup_lock();
        let mut actions = -1i32;
        assert_eq!(modukit_sup_step(&raw mut actions), 0);
        assert_eq!(actions, 0, "fresh rung has nothing to reap or revive");
    }
}

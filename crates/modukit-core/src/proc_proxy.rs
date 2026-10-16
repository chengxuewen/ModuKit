//! The kernel-side proxy of an `exe` plugin's service (S2-M3a, closing the
//! S2-M2 F2 deferral): one `ProxyService` per REGISTER frame rides the SAME
//! registry door the `dl` arm's trampoline uses (`Registry::register` —
//! message bytes in, message bytes out, D69), so a kernel-side consumer
//! dispatches across the process boundary with the same call shape as the
//! in-process `NativeService`.
//!
//! Rule check (01 §3 rule 2): nothing ambient crosses — the proxy carries a
//! plugin id + generation stamp and speaks `proc_wire` CALL/RESULT frames on
//! the control socket the supervisor already owns. No host pointer, no
//! memory, no authority: `invoke` is a synchronous id-correlated
//! request/response (04 §2, the LSP/Wayland shape).
//!
//! Unknown methods are NOT pre-rejected here: the call rides to the child,
//! whose own plugin code answers with its author code — the typed
//! passthrough that makes the `dl` and `exe` rejections byte-equal.

use std::sync::atomic::{AtomicU32, Ordering};

use crate::error::KernelError;
use crate::executor_proc::{
    CALL_RESULT_TIMEOUT, Control, crashed, is_timeout, kill_and_reap, live_entry,
};
use crate::identity::ServiceId;
use crate::proc_wire::{self, MSG_RESULT, WIRE_VERSION};

/// One proxied service: `(service_id, methods)` as the child announced it.
/// The lane serializes calls under the control-table mutex, so exactly one
/// request is ever in flight (v0); the seq still stamps every CALL/RESULT.
#[derive(Debug)]
pub struct ProxyService {
    service_id: ServiceId,
    methods: Vec<String>,
    owner_id: String,
    run_gen: u64,
    seq: AtomicU32,
    live: Control,
}

impl ProxyService {
    pub(crate) const fn new(
        service_id: ServiceId,
        methods: Vec<String>,
        owner_id: String,
        run_gen: u64,
        live: Control,
    ) -> Self {
        Self {
            service_id,
            methods,
            owner_id,
            run_gen,
            seq: AtomicU32::new(0),
            live,
        }
    }

    /// The canonical id the child reported (D9).
    #[must_use]
    pub fn service_id(&self) -> &ServiceId {
        &self.service_id
    }

    /// Advertised method names (informational — see the module doc).
    #[must_use]
    pub fn methods(&self) -> &[String] {
        &self.methods
    }

    /// Send one CALL, await the RESULT: the cross-process twin of
    /// `NativeService::invoke` (bytes in, bytes out, D69).
    ///
    /// # Errors
    ///
    /// `KernelError::ServiceCallFailed` for the child's negative author
    /// code (byte-equal passthrough of the `dl` arm); the crash family for
    /// a dead/hung/mismatched peer (a hung call is killed within
    /// `CALL_RESULT_TIMEOUT`).
    pub fn invoke(&self, method: &str, input: &[u8]) -> Result<Vec<u8>, KernelError> {
        let mut shared = self.live.lock();
        let p = live_entry(&mut shared, &self.owner_id, self.run_gen)?;
        let failed = |what: String| crashed(&self.owner_id, what);
        let sock = p
            .control
            .as_mut()
            .ok_or_else(|| failed("control closed".to_string()))?;
        sock.set_read_timeout(Some(CALL_RESULT_TIMEOUT))
            .map_err(|e| failed(format!("set timeout: {e}")))?;
        let seq = self.seq.fetch_add(1, Ordering::Relaxed);
        if proc_wire::write_frame(
            sock,
            proc_wire::MSG_CALL,
            &proc_wire::call_payload(seq, method, input),
        )
        .is_err()
        {
            kill_and_reap(p);
            return Err(failed("call write: control closed".to_string()));
        }
        // The v0 lane serializes calls under the control-table mutex, so
        // exactly one reply is outstanding — no pump loop until the M3b
        // host loop shares the channel (a stray frame is protocol death).
        match proc_wire::read_frame(sock) {
            Ok((WIRE_VERSION, MSG_RESULT, payload)) => {
                let Some(res) = proc_wire::parse_result(&payload) else {
                    kill_and_reap(p);
                    return Err(failed("result malformed".to_string()));
                };
                if res.seq != seq {
                    kill_and_reap(p);
                    return Err(failed(format!("result seq {} for call {seq}", res.seq)));
                }
                if res.status != 0 {
                    // The child's author code verbatim (D27): the same
                    // typed door the dl NativeService answers on.
                    return Err(KernelError::ServiceCallFailed { code: res.status });
                }
                Ok(res.output)
            }
            Ok((WIRE_VERSION, kind, _)) => {
                kill_and_reap(p);
                Err(failed(format!("call reply: unexpected frame type {kind}")))
            }
            Ok((found, _, _)) => {
                kill_and_reap(p);
                Err(KernelError::WireVersionMismatch {
                    expected: WIRE_VERSION,
                    found,
                })
            }
            Err(err) if is_timeout(&err) => {
                kill_and_reap(p);
                Err(failed("call result timeout, killed".to_string()))
            }
            Err(err) => {
                kill_and_reap(p);
                Err(failed(format!("call read: {err}")))
            }
        }
    }
}

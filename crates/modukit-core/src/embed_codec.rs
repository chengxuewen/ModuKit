//! Embed-face helpers for the HostEmbedV1 verbs (`embed.rs`): the D51
//! numeric mirror, the state/QoS/event-kind mappings, and the two OPAQUE
//! byte codecs the introspection verbs ship. Split from `embed.rs` to keep
//! each file under the project line cap; everything here is `pub(crate)`.
//!
//! The numeric values are transcribed VERBATIM from the frozen dictionary
//! in `contracts/modukit_embed.h` (which mirrors
//! `org.modukit.runtime.proto::KernelCode`); the names come from the one
//! existing mapping function — `KernelError::code()` — never re-derived.

use std::ffi::{CStr, c_char};
use std::panic::{self, AssertUnwindSafe};

use crate::bus::TopicStats;
use crate::counters::CountersView;
use crate::edge_ledger::EdgeKind;
use crate::error::KernelError;
use crate::events::{ObsEvent, ObsRecord};
use crate::introspect::HostView;
use crate::lifecycle::PluginState;
use crate::registry::ServiceEvent;
use crate::topic::QoS;

/// Bad input shape at the FFI boundary (null pointers, len/cap lies,
/// out-of-enum ints, negative dial deltas) — the contract-domain slot.
pub(crate) const NEG_SCHEMA_INVALID: i32 = -11;
/// A kernel-domain panic caught at the boundary (there is no INTERNAL slot
/// in the frozen dictionary; the contract-domain refusal is the honest
/// stand-in) or an unreachable bookkeeping failure.
pub(crate) const NEG_KERNEL_BUG: i32 = -10;
/// The ambient kernel holds no such id / queue / service.
pub(crate) const NEG_NOT_FOUND: i32 = -14;
/// A copy-out where `cap < required` — the fail-rather-than-truncate rule
/// of the header's service-call paragraph, applied to every buffer verb.
pub(crate) const NEG_QUOTA_EXCEEDED: i32 = -19;
/// The header pins it: a deferred-QoS topic (KEEP_ALL/BOUNDED) refuses
/// `bus_subscribe` with DEFERRAL_DECLINED — NOT the `BusError::code()`
/// name (CONTRACT_VIOLATION); where the two disagree the header is law.
pub(crate) const NEG_DEFERRAL_DECLINED: i32 = -26;
/// The header pins it: the dial on a Real-domain clock is a refusal, never
/// a silent no-op.
pub(crate) const NEG_CLOCK_DIAL_REFUSED: i32 = -27;
/// A resumable-read cursor pointing past the ring's end.
pub(crate) const NEG_EPOCH_EXPIRED: i32 = -15;

/// The `MODUKIT_QOS_LOSSY` slot carries no `n` in the flat enum; the embed
/// face publishes it as `Lossy(64)` — the repo's standard queue window
/// (`registry::EVENT_QUEUE`, the NativeService cap floor).
/// shortcut: fixed depth until a D83 tail-add exposes `n` per publish.
pub(crate) const LOSSY_RING: u32 = 64;

/// D51 name -> frozen numeric value (names exactly as `code()` emits them;
/// the reserved slots are unreachable and unmapped, falling back like any
/// unknown name to CONTRACT_VIOLATION — drift becomes visible, never a
/// silent 0).
#[must_use]
pub(crate) fn d51_number(name: &str) -> i32 {
    match name {
        "MANIFEST_INVALID" => 1,
        "DEPENDENCY_MISSING" => 2,
        "DEPENDENCY_CYCLE" => 3,
        "START_TIMEOUT" => 4,
        "START_REJECTED" => 5,
        "STOP_TIMEOUT" => 6,
        "ABI_MISMATCH" => 9,
        "CONTRACT_VIOLATION" => 10,
        "SCHEMA_INVALID" => 11,
        "NOT_FOUND" => 14,
        "EPOCH_EXPIRED" => 15,
        "CARRIER_UNREACHABLE" => 16,
        "QUOTA_EXCEEDED" => 19,
        "CAPABILITY_DENIED" => 20,
        "PROCESS_CRASHED" => 23,
        "IDENTITY_MISMATCH" => 24,
        "SERVICE_CALL_FAILED" => 25,
        "DEFERRAL_DECLINED" => 26,
        "CLOCK_DIAL_REFUSED" => 27,
        _ => 10,
    }
}

/// The negated D51 code a `KernelError` leaves on the verb's channel.
#[must_use]
pub(crate) fn neg_kernel(err: &KernelError) -> i32 {
    -d51_number(err.code())
}

/// `PluginState` -> the header's pinned `modukit_plugin_state` number.
#[must_use]
pub(crate) const fn state_number(state: PluginState) -> i32 {
    match state {
        PluginState::Uninstalled => 0,
        PluginState::Installed => 1,
        PluginState::Resolving => 2,
        PluginState::Starting => 3,
        PluginState::Active => 4,
        PluginState::Stopping => 5,
        PluginState::Resolved => 6,
        PluginState::Failed => 7,
    }
}

/// The header's `modukit_qos` int -> the kernel `QoS`; unknown -> `None`
/// (the caller answers SCHEMA_INVALID).
#[must_use]
pub(crate) const fn qos_from_i32(raw: i32) -> Option<QoS> {
    match raw {
        0 => Some(QoS::LatestValue),
        1 => Some(QoS::KeepAll),
        2 => Some(QoS::Lossy(LOSSY_RING)),
        3 => Some(QoS::Bounded),
        _ => None,
    }
}

/// The `ObsEvent` variant -> the header's `modukit_event_kind` number.
#[must_use]
pub(crate) const fn event_kind(event: &ObsEvent) -> i32 {
    match event {
        ObsEvent::Lifecycle { .. } => 0,
        ObsEvent::Registry(_) => 1,
        ObsEvent::ModuleLoaded { .. } => 2,
        ObsEvent::TimerDone { .. } => 3,
        ObsEvent::EdgeRemoved { .. } => 4,
        ObsEvent::Marker(_) => 5,
        ObsEvent::Log(_) => 6,
    }
}

fn edge_kind_name(kind: EdgeKind) -> &'static str {
    match kind {
        EdgeKind::Registered => "registered",
        EdgeKind::Unregistered => "unregistered",
        EdgeKind::Transition => "transition",
        EdgeKind::ModuleLoaded => "module-loaded",
    }
}

/// The one-line payload text an event record carries (plain data, the same
/// fields the Rust Debug form prints — names are the C-enum intent).
#[must_use]
pub(crate) fn event_payload(event: &ObsEvent) -> String {
    match event {
        ObsEvent::Lifecycle { plugin, from, to } => format!("{plugin} {from}->{to}"),
        ObsEvent::Registry(ServiceEvent::Registered { id, service_id, .. }) => {
            format!("registered {} rid={}", service_id.as_str(), id.value())
        }
        ObsEvent::Registry(ServiceEvent::Unregistered { id, service_id, .. }) => {
            format!("unregistered {} rid={}", service_id.as_str(), id.value())
        }
        ObsEvent::ModuleLoaded {
            plugin,
            artifact,
            load_base,
            symbol_hint,
        } => format!("{plugin} {artifact} base={load_base} sym={symbol_hint}"),
        ObsEvent::TimerDone { token, job } => format!("token={token} job={job}"),
        ObsEvent::EdgeRemoved { subject } => subject.clone(),
        ObsEvent::Marker(text) => text.clone(),
        ObsEvent::Log(log) => format!("{} {} {}", log.level, log.component, log.msg),
    }
}

/// Snapshot codec (one shape per line, `\n`-joined, UTF-8, lines sorted
/// lexicographically inside each section; sections in the documented
/// order plugin/service/topic/edge/counter). Field separator: TAB.
#[must_use]
pub(crate) fn render_snapshot(view: &HostView) -> Vec<u8> {
    let mut out = String::new();
    let mut lines: Vec<String> = view
        .plugins
        .iter()
        .map(|p| {
            format!(
                "plugin\t{}\t{}\t{}\t{}",
                p.id, p.state, p.placement, p.version
            )
        })
        .collect();
    lines.sort();
    for line in lines {
        out.push_str(&line);
        out.push('\n');
    }
    let mut lines: Vec<String> = view
        .services
        .iter()
        .map(|s| format!("service\t{}\t{}", s.service_id.as_str(), s.ranking))
        .collect();
    lines.sort();
    for line in lines {
        out.push_str(&line);
        out.push('\n');
    }
    let mut lines: Vec<String> = view.topics.iter().map(topic_line).collect();
    lines.sort();
    for line in lines {
        out.push_str(&line);
        out.push('\n');
    }
    let mut lines: Vec<String> = view
        .edges
        .iter()
        .map(|e| {
            format!(
                "edge\t{}\t{}\t{}\t{}",
                e.seq,
                e.ts_raw,
                edge_kind_name(e.kind),
                e.subject
            )
        })
        .collect();
    lines.sort();
    for line in lines {
        out.push_str(&line);
        out.push('\n');
    }
    out.push_str(&counter_line(&view.counters));
    out.push('\n');
    out.into_bytes()
}

fn topic_line(t: &TopicStats) -> String {
    format!(
        "topic\t{}\t{}\t{}\t{}",
        t.topic,
        t.published,
        t.delivered,
        t.dropped + t.retention_dropped
    )
}

fn counter_line(c: &CountersView) -> String {
    format!(
        "counters\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        c.registry_acquires,
        c.registry_releases,
        c.lifecycle_transitions,
        c.load_ok,
        c.load_fail,
        c.process_crashes,
        c.process_restarts,
        c.bus_published,
        c.bus_delivered,
        c.bus_dropped
    )
}

/// Append one record: little-endian `i32 kind | u64 seq | u64 ts_ns |
/// u64 payload_len | payload bytes` — the field-for-field wire mirror of
/// the header's `modukit_event` struct (self-delimiting, so a reader can
/// stop at whole-record boundaries).
pub(crate) fn encode_event_record(record: &ObsRecord, out: &mut Vec<u8>) {
    let payload = event_payload(&record.event);
    out.extend_from_slice(&event_kind(&record.event).to_le_bytes());
    out.extend_from_slice(&record.seq.to_le_bytes());
    out.extend_from_slice(&record.ts.raw().to_le_bytes());
    out.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    out.extend_from_slice(payload.as_bytes());
}

/// Encode the resumable window: on a size query (`out_buf` NULL) the full
/// byte answer with the cursor held at `since_seq`; otherwise whole
/// records only, capped at `buf_cap`, cursor advanced past exactly what
/// was copied (self-delimiting records make a truncated tail impossible).
#[must_use]
pub(crate) fn encode_events(
    records: &[ObsRecord],
    buf_cap: usize,
    since_seq: u64,
    size_query: bool,
) -> (Vec<u8>, u64) {
    let mut all = Vec::new();
    for record in records {
        encode_event_record(record, &mut all);
    }
    if size_query {
        return (all, since_seq);
    }
    let mut off = 0usize;
    let mut last = since_seq;
    while off + 28 <= all.len() && off + 28 <= buf_cap {
        let mut word = [0u8; 8];
        word.copy_from_slice(&all[off + 20..off + 28]);
        let len = usize::try_from(u64::from_le_bytes(word)).unwrap_or(usize::MAX);
        let end = off + 28 + len;
        if end > buf_cap {
            break;
        }
        word.copy_from_slice(&all[off + 4..off + 12]);
        last = u64::from_le_bytes(word);
        off = end;
    }
    all.truncate(off);
    (all, last)
}

/// Borrow a caller NUL-string; `None` on null or non-UTF-8. Bad input
/// shape answers SCHEMA_INVALID(11), never a read past the promise.
pub(crate) fn cstr_arg<'a>(ptr: *const c_char) -> Option<&'a str> {
    if ptr.is_null() {
        return None;
    }
    // SAFETY: the header contract is a readable NUL-terminated string.
    unsafe { CStr::from_ptr(ptr) }.to_str().ok()
}

pub(crate) fn bytes_arg<'a>(ptr: *const u8, len: usize) -> Option<&'a [u8]> {
    if len == 0 {
        return Some(&[]);
    }
    if ptr.is_null() {
        return None;
    }
    // SAFETY: `len > 0` with a non-null ptr promises len readable bytes.
    Some(unsafe { std::slice::from_raw_parts(ptr, len) })
}

pub(crate) fn queue_out_len(out: *mut usize, len: usize) {
    // SAFETY: every caller null-checks `out` before entering here.
    unsafe { *out = len };
}

/// Shared copy-out with the header's cap discipline: too-small caps FAIL
/// (negated QUOTA_EXCEEDED) after reporting the need through `*out_len`;
/// NULL `out` is the documented size query. A NULL `out_len` verb (the
/// service-call door) simply skips the store.
pub(crate) fn copy_out(bytes: &[u8], out: *mut u8, cap: usize, out_len: *mut usize) -> i32 {
    let Some(n) = i32::try_from(bytes.len()).ok() else {
        return NEG_QUOTA_EXCEEDED;
    };
    if !out_len.is_null() {
        // SAFETY: non-null `out_len` is a writable caller size slot.
        unsafe { *out_len = bytes.len() };
    }
    if out.is_null() {
        return n;
    }
    if bytes.len() > cap {
        return NEG_QUOTA_EXCEEDED;
    }
    // SAFETY: non-null `out` with cap >= len is a writable caller buffer
    // of at least that many bytes (the verb's own check).
    unsafe { std::slice::from_raw_parts_mut(out, bytes.len()) }.copy_from_slice(bytes);
    n
}

/// The FFI panic wall: a Rust panic NEVER unwinds into C frames (there is
/// no catch_unwind precedent elsewhere in the repo — this boundary is the
/// first one with Rust code behind a C symbol — so the discipline lands
/// The FFI panic wall: a Rust panic NEVER unwinds into C frames (there is
/// no catch_unwind precedent elsewhere in the repo — this boundary is the
/// first one with Rust code behind a C symbol — so the discipline lands
/// here). A caught panic answers CONTRACT_VIOLATION (the honest
/// "kernel refused" family for a kernel-side bug; see NEG_KERNEL_BUG).
pub(crate) fn guard(f: impl FnOnce() -> i32) -> i32 {
    panic::catch_unwind(AssertUnwindSafe(f)).unwrap_or(NEG_KERNEL_BUG)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn d51_numbers_match_the_header_slots() {
        // Anchors copied from contracts/modukit_embed.h (the frozen enum);
        // an unmapped name must fall back to CONTRACT_VIOLATION, never 0.
        assert_eq!(d51_number("MANIFEST_INVALID"), 1);
        assert_eq!(d51_number("ABI_MISMATCH"), 9);
        assert_eq!(d51_number("SCHEMA_INVALID"), 11);
        assert_eq!(d51_number("NOT_FOUND"), 14);
        assert_eq!(d51_number("DEFERRAL_DECLINED"), 26);
        assert_eq!(d51_number("CLOCK_DIAL_REFUSED"), 27);
        assert_eq!(d51_number("NO_SUCH_NAME"), 10);
    }

    #[test]
    fn every_kernel_error_variant_maps_to_a_known_slot() {
        // One sample per KernelError arm: code() emits exactly these names
        // and every one of them resolves to its own slot (not the fallback
        // by accident — the fallback stays reserved for drift visibility).
        let samples = [
            KernelError::UnknownPlugin { id: "x".into() },
            KernelError::InvalidTransition {
                from: "A".into(),
                to: "B".into(),
            },
            KernelError::Manifest(crate::error::ManifestError::Toml("x".into())),
            KernelError::ArtifactMissing { path: "x".into() },
            KernelError::LoadFailed {
                path: "x".into(),
                reason: "y".into(),
            },
            KernelError::SymbolMissing { path: "x".into() },
            KernelError::HandshakeNull,
            KernelError::AbiMismatch {
                expected: 1,
                found: 2,
            },
            KernelError::DescriptorTooSmall {
                expected: 1,
                found: 2,
            },
            KernelError::HandshakeIncomplete { what: "start" },
            KernelError::DependencyMissing {
                service: "x".into(),
            },
            KernelError::PluginStartFailed { status: -1 },
            KernelError::ServiceCallFailed { code: -2 },
            KernelError::UnsupportedPlacement {
                kind: "wasm".into(),
            },
            KernelError::ProcessCrashed {
                id: "x".into(),
                exit: "9".into(),
            },
            KernelError::WireVersionMismatch {
                expected: 1,
                found: 2,
            },
            KernelError::ClockDialRefused { status: 1 },
            KernelError::RoutingDeferred {
                plugin: "p".into(),
                service: "s".into(),
            },
            KernelError::IdentityMismatch {
                what: "id",
                expected: "a".into(),
                found: "b".into(),
            },
            KernelError::Registry(crate::registry::RegistryError::MissingServiceId),
        ];
        for err in &samples {
            assert_ne!(err.code(), "");
            let num = d51_number(err.code());
            assert!(
                num > 0 && num <= 27,
                "{} maps outside the dictionary",
                err.code()
            );
            if num == 10 {
                assert_eq!(err.code(), "CONTRACT_VIOLATION", "fallback collision");
            }
            assert!(neg_kernel(err) < 0);
        }
    }

    #[test]
    fn state_and_qos_and_kind_maps_are_pinned() {
        assert_eq!(state_number(PluginState::Uninstalled), 0);
        assert_eq!(state_number(PluginState::Active), 4);
        assert_eq!(state_number(PluginState::Failed), 7);
        assert_eq!(qos_from_i32(2), Some(QoS::Lossy(LOSSY_RING)));
        assert_eq!(qos_from_i32(99), None);
        assert_eq!(event_kind(&ObsEvent::Marker("m".into())), 5);
    }

    #[test]
    fn snapshot_codec_is_sorted_tab_separated_lines() {
        let view = HostView {
            plugins: vec![
                crate::introspect::PluginView {
                    id: "b.two".into(),
                    state: "ACTIVE".into(),
                    placement: "dl".into(),
                    version: "0.1.0".into(),
                },
                crate::introspect::PluginView {
                    id: "a.one".into(),
                    state: "INSTALLED".into(),
                    placement: "dl".into(),
                    version: "0.2.0".into(),
                },
            ],
            services: Vec::new(),
            topics: Vec::new(),
            edges: Vec::new(),
            counters: CountersView::default(),
        };
        let text = String::from_utf8(render_snapshot(&view)).expect("utf-8 codec");
        let mut lines = text.lines().map(str::to_string).peekable();
        let mut expect_line = |want: &str| {
            assert_eq!(lines.next().as_deref(), Some(want));
        };
        expect_line("plugin\ta.one\tINSTALLED\tdl\t0.2.0");
        expect_line("plugin\tb.two\tACTIVE\tdl\t0.1.0");
        expect_line("counters\t0\t0\t0\t0\t0\t0\t0\t0\t0\t0");
        assert_eq!(lines.next(), None);
    }

    #[test]
    fn event_records_round_trip_field_by_field() {
        let record = ObsRecord {
            seq: 7,
            ts: crate::clock::Timestamp::from(1234),
            event: ObsEvent::Lifecycle {
                plugin: "com.x.y".into(),
                from: "STARTING".into(),
                to: "ACTIVE".into(),
            },
        };
        let mut buf = Vec::new();
        encode_event_record(&record, &mut buf);
        assert_eq!(&buf[0..4], &0i32.to_le_bytes()); // LIFECYCLE kind
        assert_eq!(&buf[4..12], &7u64.to_le_bytes());
        assert_eq!(&buf[12..20], &1234u64.to_le_bytes());
        let len = u64::from_le_bytes(buf[20..28].try_into().unwrap()) as usize;
        assert_eq!(&buf[28..28 + len], b"com.x.y STARTING->ACTIVE");
        assert_eq!(buf.len(), 28 + len, "self-delimiting record");
    }
}

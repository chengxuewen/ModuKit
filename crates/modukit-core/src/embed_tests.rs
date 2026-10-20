//! Unit tests for the embed face — included from `embed.rs` as the
//! `#[cfg(test)] mod tests` so every extern fn is driven here through raw
//! pointers, exactly as a C caller reaches it. The ambient kernel is
//! process-global, so these tests run in parallel against it with unique
//! ids/topics per test and shape-only assertions on shared surfaces
//! (the ring and the table legitimately carry other tests' churn).
//!
//! No C++ fixture is needed: the dlopen success path is owned by the C++
//! demo lane (`examples/cpp/host_demo.cc` + `scripts/demo-stage4-m1.sh`);
//! here the lifecycle doors are covered on their error edges (a TOML file
//! is not an ELF image — resolve refuses NOT_FOUND and parks FAILED).

use super::*;
use crate::events::ObsEvent;
use crate::lifecycle::PluginState;
use std::sync::atomic::AtomicU32;

/// The ambient kernel is process-global by contract - tests that touch it
/// serialize on this lock (the verbs themselves take the kernel mutex per
/// call, so probe-then-copy across two calls is only stable when no other
/// test interleaves; poison must never poison the whole serial group).
static AMBIENT_SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn ambient_lock() -> std::sync::MutexGuard<'static, ()> {
    AMBIENT_SERIAL
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

static NEXT_TEST_ID: AtomicU32 = AtomicU32::new(1);

fn test_id() -> String {
    format!(
        "com.embed.test.t{}",
        NEXT_TEST_ID.fetch_add(1, Ordering::Relaxed)
    )
}

fn c(s: &str) -> CString {
    CString::new(s).unwrap()
}

fn manifest_toml(id: &str, artifact: &str) -> CString {
    c(&format!(
        "id = \"{id}\"\nversion = \"0.1.0\"\n[entry]\nkind = \"dl\"\nartifact = \"{artifact}\"\n"
    ))
}

#[test]
fn abi_probe_reports_one() {
    let _serial = ambient_lock();
    assert_eq!(modukit_embed_abi_version(), 1);
}

#[test]
fn doors_answer_schema_and_not_found_on_bad_shape() {
    let _serial = ambient_lock();
    let mut out: *const c_char = std::ptr::null();
    let mut state = 0i32;
    let mut len = 0usize;
    assert_eq!(
        modukit_install(std::ptr::null(), &raw mut out),
        codec::NEG_SCHEMA_INVALID
    );
    assert_eq!(
        modukit_install(c("id = 3\n").as_ptr(), &raw mut out),
        -1,
        "MANIFEST_INVALID"
    );
    // The artifact gate lives at install: MANIFEST_INVALID sibling.
    let ghost_id = test_id();
    let m = manifest_toml(&ghost_id, "/nonexistent/artifact.so");
    assert_eq!(
        modukit_install(m.as_ptr(), std::ptr::null_mut()),
        codec::NEG_SCHEMA_INVALID
    );
    assert_eq!(modukit_install(m.as_ptr(), &raw mut out), -1);
    // Every door on an untracked id answers NOT_FOUND(14).
    let ghost = c(ghost_id.as_str());
    for (what, got) in [
        ("state", modukit_state(ghost.as_ptr(), &raw mut state)),
        ("resolve", modukit_resolve(ghost.as_ptr())),
        ("start", modukit_start(ghost.as_ptr())),
        ("stop", modukit_stop(ghost.as_ptr())),
        ("uninstall", modukit_uninstall(ghost.as_ptr())),
        (
            "recv",
            modukit_bus_recv(999_999, std::ptr::null_mut(), 0, &raw mut len),
        ),
    ] {
        assert_eq!(got, codec::NEG_NOT_FOUND, "{what}");
    }
    assert_eq!(
        modukit_state(std::ptr::null(), std::ptr::null_mut()),
        codec::NEG_SCHEMA_INVALID
    );
    assert_eq!(modukit_bus_unsubscribe(999_998), codec::NEG_NOT_FOUND);
}

#[test]
fn install_out_id_is_kernel_owned_and_states_walk() {
    let _serial = ambient_lock();
    let id = test_id();
    let artifact = concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml");
    let m = manifest_toml(&id, artifact);
    let mut out: *const c_char = std::ptr::null();
    assert_eq!(modukit_install(m.as_ptr(), &raw mut out), 0);
    assert!(!out.is_null());
    // SAFETY: the verb just wrote a NUL-terminated kernel-owned id.
    assert_eq!(unsafe { CStr::from_ptr(out) }.to_str().unwrap(), id);

    let pid = c(id.as_str());
    let mut state = 0;
    assert_eq!(modukit_state(pid.as_ptr(), &raw mut state), 0);
    assert_eq!(state, codec::state_number(PluginState::Installed));
    // A TOML file is not an ELF image: the dl door refuses NOT_FOUND
    // (LoadFailed) and the guard parks the plugin on FAILED.
    assert_eq!(modukit_resolve(pid.as_ptr()), codec::NEG_NOT_FOUND);
    assert_eq!(modukit_state(pid.as_ptr(), &raw mut state), 0);
    assert_eq!(state, codec::state_number(PluginState::Failed));
    // start/stop outside their table edges are contract refusals.
    assert_eq!(modukit_start(pid.as_ptr()), -10, "CONTRACT_VIOLATION");
    assert_eq!(modukit_stop(pid.as_ptr()), -10);
    assert_eq!(modukit_uninstall(pid.as_ptr()), 0);
    assert_eq!(modukit_state(pid.as_ptr(), &raw mut state), 0);
    assert_eq!(state, codec::state_number(PluginState::Uninstalled));
    // Re-install replaces the terminal record; the id pointer is the
    // one stable table entry (kernel owns the bytes for the process).
    let mut out2: *const c_char = std::ptr::null();
    assert_eq!(modukit_install(m.as_ptr(), &raw mut out2), 0);
    assert_eq!(out, out2, "the same id answers the same stable pointer");
    assert_eq!(modukit_uninstall(pid.as_ptr()), 0);
}

#[test]
fn service_call_answers_not_found_without_a_matching_provider() {
    let _serial = ambient_lock();
    let sid = c("com.embed.test.no-service");
    let method = c("echo");
    let input = *b"hi";
    let mut out = [0u8; 16];
    assert_eq!(
        modukit_service_call(
            std::ptr::null(),
            method.as_ptr(),
            input.as_ptr(),
            2,
            out.as_mut_ptr(),
            16
        ),
        codec::NEG_SCHEMA_INVALID
    );
    assert_eq!(
        modukit_service_call(
            sid.as_ptr(),
            method.as_ptr(),
            std::ptr::null(),
            4,
            out.as_mut_ptr(),
            16
        ),
        codec::NEG_SCHEMA_INVALID,
        "len > 0 with a null input is a shape lie"
    );
    assert_eq!(
        modukit_service_call(
            sid.as_ptr(),
            method.as_ptr(),
            input.as_ptr(),
            2,
            std::ptr::null_mut(),
            16
        ),
        codec::NEG_SCHEMA_INVALID
    );
    assert_eq!(
        modukit_service_call(
            sid.as_ptr(),
            method.as_ptr(),
            input.as_ptr(),
            2,
            out.as_mut_ptr(),
            16
        ),
        codec::NEG_NOT_FOUND
    );
}

#[test]
fn bus_round_trip_probe_cap_discipline_and_release() {
    let _serial = ambient_lock();
    let topic = c(&format!("com.embed.test.{}", test_id().replace('.', "-")));
    let value = *b"0123456789";
    let mut qid = 0i32;
    let mut buf = [0u8; 32];
    let mut len = 0usize;
    // Shape refusals first (nothing published, nothing subscribed).
    assert_eq!(
        modukit_bus_subscribe(topic.as_ptr(), std::ptr::null_mut()),
        codec::NEG_SCHEMA_INVALID
    );
    assert_eq!(
        modukit_bus_publish(c("Bad_Topic").as_ptr(), 0, std::ptr::null(), 0),
        codec::NEG_SCHEMA_INVALID
    );
    assert_eq!(
        modukit_bus_publish(topic.as_ptr(), 99, std::ptr::null(), 0),
        codec::NEG_SCHEMA_INVALID,
        "unknown qos"
    );
    assert_eq!(
        modukit_bus_publish(topic.as_ptr(), 0, std::ptr::null(), 1),
        codec::NEG_SCHEMA_INVALID,
        "len lie"
    );
    // Empty queue, then a live delivery into the subscription.
    assert!(modukit_bus_subscribe(topic.as_ptr(), &raw mut qid) >= 1);
    assert_eq!(modukit_bus_recv(qid, buf.as_mut_ptr(), 32, &raw mut len), 0);
    assert_eq!(
        modukit_bus_publish(topic.as_ptr(), 0, value.as_ptr(), value.len()),
        0
    );
    assert_eq!(
        modukit_bus_recv(qid, buf.as_mut_ptr(), 32, &raw mut len),
        10
    );
    assert_eq!(&buf[..10], &value[..]);
    // The cap is fail-rather-than-truncate and the value survives.
    assert_eq!(
        modukit_bus_publish(topic.as_ptr(), 0, value.as_ptr(), value.len()),
        0
    );
    let mut small = [0u8; 4];
    assert_eq!(
        modukit_bus_recv(qid, small.as_mut_ptr(), 4, &raw mut len),
        codec::NEG_QUOTA_EXCEEDED
    );
    assert_eq!(len, 10, "too-small caps report the need");
    assert_eq!(
        modukit_bus_recv(qid, buf.as_mut_ptr(), 32, &raw mut len),
        10,
        "not consumed"
    );
    // Fresh publish: the probe below must not consume what it measures.
    assert_eq!(
        modukit_bus_publish(topic.as_ptr(), 0, value.as_ptr(), value.len()),
        0
    );
    // NULL out_value is a pure size probe: same value stays queued.
    assert_eq!(
        modukit_bus_recv(qid, std::ptr::null_mut(), 0, &raw mut len),
        10
    );
    assert_eq!(len, 10);
    assert_eq!(
        modukit_bus_recv(qid, buf.as_mut_ptr(), 32, &raw mut len),
        10
    );
    assert_eq!(
        modukit_bus_recv(qid, std::ptr::null_mut(), 0, std::ptr::null_mut()),
        codec::NEG_SCHEMA_INVALID
    );
    // Unsubscribe retires the id for good.
    assert_eq!(modukit_bus_unsubscribe(qid), 0);
    assert_eq!(modukit_bus_unsubscribe(qid), codec::NEG_NOT_FOUND);
    assert_eq!(
        modukit_bus_recv(qid, buf.as_mut_ptr(), 32, &raw mut len),
        codec::NEG_NOT_FOUND
    );
}

#[test]
fn bus_subscribe_deferred_qos_refuses_with_deferral_declined() {
    let _serial = ambient_lock();
    let topic = c(&format!("com.embed.test.d{}", test_id().replace('.', "-")));
    assert_eq!(
        modukit_bus_publish(topic.as_ptr(), 1 /* KEEP_ALL */, std::ptr::null(), 0),
        0
    );
    let mut qid = 0i32;
    assert_eq!(
        modukit_bus_subscribe(topic.as_ptr(), &raw mut qid),
        codec::NEG_DEFERRAL_DECLINED
    );
}

#[test]
fn clock_now_works_and_the_real_domain_refuses_the_dial() {
    let _serial = ambient_lock();
    let mut now: i64 = -1;
    assert_eq!(modukit_clock_now(&raw mut now), 0);
    assert!(now >= 0, "monotonic-since-boot stamp");
    assert_eq!(
        modukit_clock_now(std::ptr::null_mut()),
        codec::NEG_SCHEMA_INVALID
    );
    assert_eq!(modukit_clock_advance(-1), codec::NEG_SCHEMA_INVALID);
    assert_eq!(
        modukit_clock_advance(1_000_000),
        codec::NEG_CLOCK_DIAL_REFUSED
    );
}

#[test]
fn snapshot_probe_then_exact_copy() {
    let _serial = ambient_lock();
    let mut len = 0usize;
    assert_eq!(
        modukit_introspect_snapshot(std::ptr::null_mut(), 0, std::ptr::null_mut()),
        codec::NEG_SCHEMA_INVALID
    );
    let need = modukit_introspect_snapshot(std::ptr::null_mut(), 0, &raw mut len);
    assert!(need > 0, "the cut is never empty (platform services ride)");
    assert_eq!(len, need as usize);
    let mut one = [0u8; 1];
    assert_eq!(
        modukit_introspect_snapshot(one.as_mut_ptr(), 1, &raw mut len),
        codec::NEG_QUOTA_EXCEEDED
    );
    assert_eq!(len, need as usize, "cap-too-small still reports the need");
    let mut buf = vec![0u8; need as usize];
    assert_eq!(
        modukit_introspect_snapshot(buf.as_mut_ptr(), buf.len(), &raw mut len),
        need
    );
    let text = String::from_utf8(buf).expect("the documented codec is UTF-8");
    assert!(
        text.contains("service\torg.modukit.snapshot\t"),
        "trio rides the table"
    );
    assert!(
        text.lines().last().unwrap().starts_with("counters\t"),
        "counters line closes the cut"
    );
}

#[test]
fn events_encode_walk_resume_and_refuse_a_future_cursor() {
    let _serial = ambient_lock();
    let marker = format!(
        "embed-unit-marker-{}",
        NEXT_TEST_ID.fetch_add(1, Ordering::Relaxed)
    );
    let seq = ambient()
        .0
        .observatory()
        .emit(ObsEvent::Marker(marker.clone()));
    let mut len = 0usize;
    let mut cursor = 0u64;
    assert_eq!(
        modukit_introspect_events(
            0,
            std::ptr::null_mut(),
            0,
            std::ptr::null_mut(),
            &raw mut cursor
        ),
        codec::NEG_SCHEMA_INVALID
    );
    let need = modukit_introspect_events(0, std::ptr::null_mut(), 0, &raw mut len, &raw mut cursor);
    assert!(need > 0 && len >= 28, "at least one whole record");
    assert_eq!(cursor, 0, "the size query consumes nothing");
    let mut buf = vec![0u8; len];
    let written = modukit_introspect_events(
        0,
        buf.as_mut_ptr(),
        buf.len(),
        &raw mut len,
        &raw mut cursor,
    ) as usize;
    assert!(written > 0);
    assert!(cursor >= seq, "the cursor rode past the copied records");
    let mut off = 0usize;
    let mut found = false;
    while off + 28 <= written {
        let kind = i32::from_le_bytes(buf[off..off + 4].try_into().unwrap());
        let rseq = u64::from_le_bytes(buf[off + 4..off + 12].try_into().unwrap());
        let plen = u64::from_le_bytes(buf[off + 20..off + 28].try_into().unwrap()) as usize;
        if kind == 5 && buf[off + 28..off + 28 + plen] == *marker.as_bytes() {
            found = true;
            assert_eq!(rseq, seq);
        }
        off += 28 + plen;
    }
    assert!(found, "the marker rode the documented LE record codec");
    assert_eq!(off, written, "records tile the buffer exactly");
    assert_eq!(
        modukit_introspect_events(
            u64::MAX,
            std::ptr::null_mut(),
            0,
            &raw mut len,
            &raw mut cursor
        ),
        codec::NEG_EPOCH_EXPIRED
    );
}

#[test]
fn guard_catches_panics_so_nothing_unwinds_into_c() {
    let _serial = ambient_lock();
    let answer = std::panic::catch_unwind(|| codec::guard(|| panic!("boundary probe panic")));
    assert!(answer.is_ok(), "guard itself must not unwind");
    assert_eq!(answer.unwrap(), codec::NEG_KERNEL_BUG);
}

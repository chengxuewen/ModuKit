//! modukit-debug-reader — Wave-2 reader-only tooling face.
//!
//! This binary is a self-check face over the existing observability data plane.
//! It boots the production `TestHost` kernel and reads through already-shipped
//! introspect surfaces. It does not attach to a running host, does not expose a
//! new ABI verb, and does not define a debug file format.

use std::env;
use std::process::ExitCode;

use modukit_core::introspect::LedgerDump;
use modukit_core::testhost::TestHost;

const PREFIX: &str = "debug-reader";
const SABOTAGE_ENV: &str = "MODUKIT_DEBUG_READER_SABOTAGE";

fn usage() {
    println!("usage: modukit-debug-reader [--help]");
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    if let Some(arg) = args.first() {
        if args.len() == 1 && arg == "--help" {
            usage();
            return ExitCode::SUCCESS;
        }

        eprintln!("{PREFIX}: unknown option {arg}");
        usage();
        return ExitCode::from(2);
    }

    let host = TestHost::new();
    let snapshot = host.snapshot();
    let tail = host.events_since(0);
    let head = host.head();
    let kernel = host.kernel();

    let Some(ledger) = kernel.registry().acquire::<LedgerDump>() else {
        eprintln!("{PREFIX}: ledger dump missing from boot services");
        return ExitCode::FAILURE;
    };
    let ledger_plugins = ledger.dump().len();

    if let Err(err) = kernel.registry().release(&ledger) {
        eprintln!("{PREFIX}: ledger release: {err}");
        return ExitCode::FAILURE;
    }

    println!("{PREFIX}: ledger plugins={ledger_plugins}");
    println!(
        "{PREFIX}: snapshot plugins={} services={} topics={} edges={}",
        snapshot.plugins.len(),
        snapshot.services.len(),
        snapshot.topics.len(),
        snapshot.edges.len()
    );
    println!(
        "{PREFIX}: events head={head} tail={} first_seq={} dropped={}",
        tail.events.len(),
        tail.first_seq,
        tail.dropped
    );
    println!(
        "{PREFIX}: counters registry_acquires={} registry_releases={} lifecycle_transitions={} load_ok={} load_fail={} process_crashes={} process_restarts={} bus_published={} bus_delivered={} bus_dropped={}",
        snapshot.counters.registry_acquires,
        snapshot.counters.registry_releases,
        snapshot.counters.lifecycle_transitions,
        snapshot.counters.load_ok,
        snapshot.counters.load_fail,
        snapshot.counters.process_crashes,
        snapshot.counters.process_restarts,
        snapshot.counters.bus_published,
        snapshot.counters.bus_delivered,
        snapshot.counters.bus_dropped
    );

    if env::var(SABOTAGE_ENV).is_ok_and(|value| value == "1") {
        println!("{PREFIX}: sabotage");
        return ExitCode::FAILURE;
    }

    println!("{PREFIX}: PASS");
    ExitCode::SUCCESS
}

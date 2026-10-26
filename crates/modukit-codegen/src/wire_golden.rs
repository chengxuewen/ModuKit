//! `wire-golden` - emit the 14 canonical plugin-wire v1 golden frames plus
//! an `index.txt` ledger into `--out <dir>` (D106 batch 1,
//! wave-2-plugin-door-plan.md T1.2).
//!
//! Implementation-derived regression lock: the payload bytes come ONLY
//! from the public payload fns of `modukit_core::proc_wire`; the 6-byte
//! TL header mirrors what `write_frame` (proc_wire.rs:123-131) puts on
//! the socket - `write_frame` needs a `UnixStream`, so `frame()` below
//! assembles the identical bytes from the same `WIRE_VERSION` and length
//! constants. All field data are fixed constants: no timestamps, no
//! randomness, two runs are byte-identical.
//!
//! Checked-in consumers: `contracts/golden/wire-v1/` (the goldens),
//! `crates/modukit-codegen/tests/wire_golden.rs` (the instrumented
//! reproduction test - delta-review F5, the coverage path) and
//! `scripts/check-wire-parity.sh` (the fast gate leg).

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::time::Duration;

use modukit_core::abi::ABI_VERSION;
use modukit_core::proc_wire::{
    self, Delivery, DueTurn, Forward, Hello, ServiceRegistration, ack_payload, call_payload,
    deliver_payload, dial_payload, due_payload, forward_payload, hello_payload, hello_payload_caps,
    register_payload, result_payload,
};

/// Canonical handshake identity (fixed; the plan pins id and method).
const ID: &str = "org.modukit.wire.canonical";
const VERSION: &str = "1.0.0";
const NAME: &str = "wire canonical";
const SERVICE_ID: &str = "wire.canonical";
const METHOD: &str = "wire.canonical";
const TOPIC: &str = "org.modukit.wire.canonical.bus";
const SEQ: u32 = 7;
const DIAL: Duration = Duration::from_millis(150);

struct Golden {
    /// File stem: `<name>.frame`.
    name: &'static str,
    /// proc_wire constant name the kind byte comes from (index line).
    kind: &'static str,
    kind_byte: u8,
    /// One-line payload grammar note (index line).
    grammar: &'static str,
    bytes: Vec<u8>,
}

/// The exact bytes `write_frame` (proc_wire.rs:123-131) writes:
/// `[u32 LE payload len][WIRE_VERSION][kind][payload]`. The one framing
/// helper of this bin (write_frame itself is socket-only).
fn frame(kind: u8, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(6 + payload.len());
    out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    out.push(proc_wire::WIRE_VERSION);
    out.push(kind);
    out.extend_from_slice(payload);
    out
}

fn goldens() -> Vec<Golden> {
    let hello = Hello {
        abi: ABI_VERSION,
        id: ID.to_string(),
        version: VERSION.to_string(),
        name: NAME.to_string(),
        caps: 0,
    };
    let caps = Hello {
        caps: proc_wire::HELLO_CAP_BUS_V2 | proc_wire::HELLO_CAP_TIMER_V2,
        ..hello.clone()
    };
    let reg = ServiceRegistration {
        service_id: SERVICE_ID.to_string(),
        methods: vec![METHOD.to_string(), "caps".to_string()],
    };
    let refusal = DueTurn {
        status: proc_wire::DIAL_STATUS_REFUSED_REAL,
        fired_at_ns: 0,
        jobs: Vec::new(),
    };
    let fwd = |op: u8, value: &[u8]| Forward {
        op,
        topic: TOPIC.to_string(),
        qos: 0,
        value: value.to_vec(),
    };
    let deliver = Delivery {
        topic: TOPIC.to_string(),
        qos: 0,
        value: b"hello from bus".to_vec(),
    };
    let plain = hello_payload(&hello);
    let capped = hello_payload_caps(&caps);
    let ack = ack_payload(0);
    let registered = register_payload(&reg);
    let call = call_payload(SEQ, METHOD, b"ping");
    let result = result_payload(SEQ, 0, b"pong");
    let dial = dial_payload(DIAL);
    let due = due_payload(&refusal);
    let pub_frame = forward_payload(&fwd(proc_wire::FWD_PUB, b"hello bus"));
    let sub_frame = forward_payload(&fwd(proc_wire::FWD_SUB, b""));
    let unsub_frame = forward_payload(&fwd(proc_wire::FWD_UNSUB, b""));
    let delivered = deliver_payload(&deliver);
    vec![
        Golden {
            name: "HELLO_plain",
            kind: "MSG_HELLO",
            kind_byte: proc_wire::MSG_HELLO,
            grammar: "u32 LE abi | pstr id | pstr version | pstr name - legacy tail-less HELLO",
            bytes: frame(proc_wire::MSG_HELLO, &plain),
        },
        Golden {
            name: "HELLO_caps",
            kind: "MSG_HELLO",
            kind_byte: proc_wire::MSG_HELLO,
            grammar: "u32 LE abi | pstr x3 | u32 LE caps = BUS_V2|TIMER_V2 (parse_hello_caps tail)",
            bytes: frame(proc_wire::MSG_HELLO, &capped),
        },
        Golden {
            name: "START",
            kind: "MSG_START",
            kind_byte: proc_wire::MSG_START,
            grammar: "EMPTY payload - host -> child lifecycle start",
            bytes: frame(proc_wire::MSG_START, &[]),
        },
        Golden {
            name: "STOP",
            kind: "MSG_STOP",
            kind_byte: proc_wire::MSG_STOP,
            grammar: "EMPTY payload - host -> child cooperative stop",
            bytes: frame(proc_wire::MSG_STOP, &[]),
        },
        Golden {
            name: "ACK",
            kind: "MSG_ACK",
            kind_byte: proc_wire::MSG_ACK,
            grammar: "one i32 LE status = 0 (ack_payload - batch-0 T0.2 twin)",
            bytes: frame(proc_wire::MSG_ACK, &ack),
        },
        Golden {
            name: "REGISTER",
            kind: "MSG_REGISTER",
            kind_byte: proc_wire::MSG_REGISTER,
            grammar: "pstr service_id | u8 count = 2 | pstr method x count",
            bytes: frame(proc_wire::MSG_REGISTER, &registered),
        },
        Golden {
            name: "CALL",
            kind: "MSG_CALL",
            kind_byte: proc_wire::MSG_CALL,
            grammar: "u32 LE seq = 7 | pstr method | u32 LE len = 4 | ascii input",
            bytes: frame(proc_wire::MSG_CALL, &call),
        },
        Golden {
            name: "RESULT",
            kind: "MSG_RESULT",
            kind_byte: proc_wire::MSG_RESULT,
            grammar: "u32 LE seq = 7 | i32 LE status = 0 | u32 LE len = 4 | ascii output",
            bytes: frame(proc_wire::MSG_RESULT, &result),
        },
        Golden {
            name: "DIAL",
            kind: "MSG_DIAL",
            kind_byte: proc_wire::MSG_DIAL,
            grammar: "one u64 LE delta_micros = 150000 (150 ms)",
            bytes: frame(proc_wire::MSG_DIAL, &dial),
        },
        Golden {
            name: "DUE",
            kind: "MSG_DUE",
            kind_byte: proc_wire::MSG_DUE,
            grammar: "i32 LE status = DIAL_STATUS_REFUSED_REAL (-2) | u64 ns = 0 | u16 count = 0",
            bytes: frame(proc_wire::MSG_DUE, &due),
        },
        Golden {
            name: "FORWARD_PUB",
            kind: "MSG_FORWARD",
            kind_byte: proc_wire::MSG_FORWARD,
            grammar: "u8 op = 0 FWD_PUB | pstr topic | i32 qos = 0 | u32 len = 9 | ascii value",
            bytes: frame(proc_wire::MSG_FORWARD, &pub_frame),
        },
        Golden {
            name: "FORWARD_SUB",
            kind: "MSG_FORWARD",
            kind_byte: proc_wire::MSG_FORWARD,
            grammar: "u8 op = 1 FWD_SUB | pstr topic | i32 LE qos = 0 | u32 LE len = 0 (empty)",
            bytes: frame(proc_wire::MSG_FORWARD, &sub_frame),
        },
        Golden {
            name: "FORWARD_UNSUB",
            kind: "MSG_FORWARD",
            kind_byte: proc_wire::MSG_FORWARD,
            grammar: "u8 op = 2 FWD_UNSUB | pstr topic | i32 LE qos = 0 | u32 LE len = 0 (empty)",
            bytes: frame(proc_wire::MSG_FORWARD, &unsub_frame),
        },
        Golden {
            name: "DELIVER",
            kind: "MSG_DELIVER",
            kind_byte: proc_wire::MSG_DELIVER,
            grammar: "pstr topic | i32 LE qos = 0 | u32 LE len = 14 | ascii value",
            bytes: frame(proc_wire::MSG_DELIVER, &delivered),
        },
    ]
}

fn hex_dump(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn index_text(items: &[Golden]) -> String {
    let mut s = String::from(
        "# wire-v1 golden index - 14 canonical frames (D106 batch 1, plan T1.2)\n\
         # Deterministic: fixed constants only, no timestamps, no randomness.\n\
         # Prose ledger: contracts/modukit_wire.h; implementation:\n\
         # crates/modukit-core/src/proc_wire.rs.\n",
    );
    for g in items {
        s.push_str(&format!(
            "\n== {} ==\nkind: {} = {}\nsize: {}\nhex: {}\ngrammar: {}\n\
             provenance: implementation-derived regression lock - regenerate via \
             `cargo run -q -p modukit-codegen --bin wire-golden -- --out \
             contracts/golden/wire-v1`, sanctioned change = hearing + \
             same-commit golden+ledger update\n",
            g.name,
            g.kind,
            g.kind_byte,
            g.bytes.len(),
            hex_dump(&g.bytes),
            g.grammar,
        ));
    }
    s
}

fn out_dir() -> Option<PathBuf> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() == 2 && args[0] == "--out" {
        Some(PathBuf::from(&args[1]))
    } else {
        None
    }
}

fn main() {
    let Some(out) = out_dir() else {
        eprintln!("usage: wire-golden --out <dir>");
        process::exit(2);
    };
    let items = goldens();
    if let Err(err) = fs::create_dir_all(&out) {
        eprintln!("wire-golden: cannot create {}: {err}", out.display());
        process::exit(1);
    }
    for g in &items {
        let path = out.join(format!("{}.frame", g.name));
        if let Err(err) = fs::write(&path, &g.bytes) {
            eprintln!("wire-golden: cannot write {}: {err}", path.display());
            process::exit(1);
        }
    }
    let index = out.join("index.txt");
    if let Err(err) = fs::write(&index, index_text(&items)) {
        eprintln!("wire-golden: cannot write {}: {err}", index.display());
        process::exit(1);
    }
    println!(
        "wire-golden: wrote {} frames + index.txt to {}",
        items.len(),
        out.display()
    );
}

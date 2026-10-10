# 08 — End-to-End Walkthrough: One Video Frame

The worked example that stitches modules 01-07 together. Subject: a 1080p I420 frame from
the cabin camera, consumed by a same-process renderer, a subprocess widget, and a rear-seat
display on another host. Everything here is already adjudicated (D2-D15); this document only
shows the pieces assembled.

## 1. What humans write — exactly two things

```protobuf
// contract: cabin_video.proto (authored original; the ONLY place "frame" is defined)
enum VideoEncoding { RAW_I420 = 0; RAW_NV12 = 1; H264_ANNEXB = 2; }
message PlaneLayout { uint32 stride = 1; uint32 height = 2; uint64 offset = 3; }
message SlotTicket  { uint64 slot = 1; uint32 epoch = 2; }
message PlaneCargo  { SlotTicket ticket = 1; repeated PlaneLayout planes = 2; }

message VideoFrameMeta {
  uint64 seq = 1;  int64 capture_ts = 2;  string source = 3;
  uint32 width = 4; uint32 height = 5;
  VideoEncoding encoding = 6; ColorRange range = 7;
  bool keyframe = 8; uint64 pts = 9;

  oneof cargo {
    PlaneCargo local = 10;    // same-host arm: a TICKET (bytes stay in the pool)
    bytes        tail  = 1000; // off-host arm: the CARGO ITSELF, last field (sidecar)
  }
}

message CabinVideo {
  option (modukit.topic) = { name: "com.yourco.cam.cabin.i420",
                             qos: LOSSY(2), stamp: "frame_meta" };
}
```

```toml
# deployment: modukit.toml (wiring only; zero pixel vocabulary)
[pools.cam]  cell_bytes = 3_300_000; depth = 8      # geometry of where tickets point
route."com.yourco.cam.cabin.h264.to_rearseat" = { via = "webrtc" }
route."telemetry.site_b" = { via = "zenoh", compress = "lz4" }
```

## 2. Three sheets, one frame — "where is the frame data defined?"

| Sheet | Written by | Defines | Never contains |
|---|---|---|---|
| **type face** (`.proto`) | author | what the cargo **is**: `encoding`, `PlaneLayout` template, seq/ts/story fields | any pixel bytes |
| **ticket face** (runtime) | kernel | where the cargo **is right now**: `slot`, `epoch` (or the tail arm exists: cargo rides with) | definitions |
| **deployment face** (`modukit.toml`) | ops | pool geometry (cell size, depth), carriers, compression | cargo vocabulary |

**The pixels themselves live in memory** — a pool cell filled by DMA (camera → plane bytes,
layout per the `PlaneLayout` map), or the tail buffer after a bridge exchange. The `.proto`
is the waybill, never the crate (07 §3-4, D15).

## 3. What codegen emits (one `protoc` run + our plugin; fdset is the only input)

```
cabin_video.proto ─┬─ fdset (ledger, checked in; CI regen-diff gate)
                   ├─ Rust:  VideoFrameMeta + FrameView<'a> + trait stubs + SERVICE_ID consts
                   ├─ C:     plain structs + encode/decode decls + vtable entries
                   ├─ C++:   header-only modukit/video_frame.h (class FrameView, Result<T>)
                   │         — zero protobuf/abseil runtime inside plugins
                   └─ (Stage-5) Python/JS stubs; WIT projection; ROS2 dialect (bridge-time)
```

## 4. The cargo `oneof` — two shapes, one accessor

```
Frame arrives                    f.plane(i) internally:
┌───────────────────────┐        match cargo:
│ local ticket (same    │  ───►   Local(t) → addr = pool_base + t.slot×cell + t.planes[i].offset
│ host: 3.11 MB never   │         Tail(b)  → slice of the frame's OWN byte buffer
│ moves for this leg)   │        both arms return the SAME struct Plane{data,len,stride}
├───────────────────────┤        frame owns the ticket (or the buffer);
│ tail bytes (off-host  │        planes borrow from the frame — never from the pool directly,
│ leg; tag 1000 = last  │        never from the heap directly
│ on the wire, so any   │        dtor: ticket → atomic release | buffer → free
│ reader parses the     │        (Rust enforces by lifetime; C++ by RAII + debug assert;
│ ~90-byte waybill      │         C by release pairing — same ledger, three manners)
│ before a possible     │
│ multi-MB body)        │
└───────────────────────┘
```

Bridge exchange (07 §3): crossing the host boundary, a kernel bridge thread rewrites
`local → tail` — one honest copy; line compression (`lz4`) is a **deployment** axis,
format (`encoding`) is a **type** axis; the two never touch each other. No ticket ever
crosses a network. `EPOCH_EXPIRED` on a slow reader is a *normal* outcome: video QoS is
LOSSY — slow consumers fast-forward, never hold the camera (06 §2 commandments).

## 5. Runtime anatomy (same-host leg, one cell)

```
┌ cell #57 ─────────────────────────────────────────────────────────┐
│ [header 16B: epoch=9 | refcount=0 | flags]                         │
│ [waybill ~90B: seq=1042 ts w h encoding=RAW_I420                   │
│                cargo=local{slot=57, epoch=9, planes=[Y,U,V map]}]  │
│ [Y 2,073,600B][U 518,400B][V 518,400B]   ← written by DMA, never   │
└────────────────────────────────────────────────────────────────────┘
publish: stage(&mut s){ write meta; s.plane_mut(0..) } → commit
delivery: ring broadcast of {seq, slot, epoch} — 16B, zero syscalls
release: refcount-- (atomic); last holder opens the cell for epoch=10
death:   UDS disconnect → kernel reclaims every unreleased hold, keyed by process
```

Consumers, all three, one API (07 §3):

```rust
// same-process renderer           // subprocess widget             // rear seat (remote)
let f = sub.next()?;               let f = sub.next()?;             let f = sub.next()?;
let y = f.plane(0)?;               let y = f.plane(0)?;             let y = f.plane(0)?; // owned
gl_upload(y.data, y.stride);       canvas.draw(y);                  gl_upload(y.data, …); // bytes, not pool
// drop ⇒ release                  // drop ⇒ release               // drop ⇒ free
```

```cpp
// C++ face (generated header; Result, no exceptions, planes borrow the frame)
while (auto f = sub.next()) {
    if (auto y = f->plane(0)) gl_upload(y->data, y->stride);
    auto u = f->plane(1), v = f->plane(2);
}   // f scope end ⇒ ticket released — unwritable to forget
```

## 6. Cargo routing decision tree (automatic; user code never picks)

```
size ≤ ~1KB ─────────────► envelope only (control lane; any carrier carries it free)
>1KB, same-host consumer ─► pool cell + ticket (zero copy — the default)
>1KB, off-host only ──────► [waybill][tail] frame (+ optional compress per route)
media/codec cargo ─────────► producer declares encoding=H264_ANNEXB (type face);
                             webrtc carrier chosen in deployment face
```

## 7. Contrast cargo: ImuBatch (why the same machine serves tables and pixels)

```
proto row:  message ImuSample { float gx=1; float gy=2; float gz=3; int64 ts=4; }
            option (modukit.topic) = { cargo: ARROW, qos: BOUNDED(256) }
compile:    rows→columns mapping (mechanical; 14-type whitelist ∩ protobuf ∩ Arrow)
            schema digest 0x9F2E → fdset entry stores "0x9F2E = 4 columns this order"
runtime:    waybill carries digest+rows; body is Arrow IPC in the SAME cell format
consume:    f.plane()? — no: f.columns() → Arrow arrays by offset; pandas/duckdb
            enter via C Data Interface same-process (pointer handoff, still zero copy)
```
Video and tables share: one ledger, one `oneof` cargo doctrine, one ticket/release account,
one dispatch desk. They differ only in the cargo lane (planes vs columns) — declared, never
hardcoded (04 §2, 06 §2).

## 8. Reading path

Plugin author: 05 → 02 → 04 → 08 (this file). Kernel resident: 01 → 03 → 06 → 07.

## 9. Template status (D23)

The `example-plugin`/`bad-plugin` fixtures hold **template standing**: their layout (crate
skeleton, `.proto` placement, manifest sample, contract-test stub, and the three mouth
rules — logger (D19), clock (D20), no stdout in production) is the official plugin-starting
answer. Changing them is a compatibility-weighted documentation act (CI compiles them
daily — the template cannot silently rot). The `modukit new` generator arrives with the
Stage-2 CLI and must consume the ledger (`fdset`) and the D13 generation faces — it may
never carry a private dialect or template fork. The fourth template piece: contract-test
stubs written against the official `test-support` host (D24) — time-travel tests
(`advance(30s)` beats the real 30 seconds) are the template's own advertisement.

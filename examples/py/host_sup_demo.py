#!/usr/bin/env python3
"""examples/py/host_sup_demo.py - the wave-2 Python sup-rung demo (D100
volume M2; mirror of examples/cpp/host_sup_demo.cc, one-for-one, same label
scheme and success-string shape). Two modes of one script, exactly like the
cpp binary:

  (default) WALK : the supervised host. Drives the sup_* arm of the GENERATED
    ctypes floor (bindings/py/modukit_embed.py, re-emitted from
    contracts/modukit_embed.h by scripts/demo-wave2-py-sup.sh - hand-written
    ctypes prototypes are forbidden, D63): modukit_sup_install (real runner
    spawn + HELLO + start) -> modukit_sup_state(ACTIVE) -> READY line ->
    modukit_sup_step(idle) -> modukit_sup_dial (step-domain child TURNS:
    typed status 0, the corrected word from host_sup_demo.cc:117-123) ->
    the M3-FIDELITY mirror: modukit_sup_service_call carries ONE non-empty
    typed protobuf payload ABI -> TL wire -> child service -> back and
    asserts the response payload bytes equal the pinned expected bytes
    EXACTLY before printing the FIDELITY PASS marker; then sleeps until
    killed.
  --proof  PARENT : fork+exec self in WALK mode (os.fork + direct walk call -
    the parent never loads the kernel, mirroring the cpp parent that only
    forks), wait for the READY handshake line AND the FIDELITY PASS marker
    (a walk child that dies before the marker failed the door), SIGKILL the
    child host, then a /proc cmdline census (stdlib os walk, mirror of
    host_sup_demo.cc:57-84): the runner must have self-reaped on the
    control-pipe EOF. Zero survivors prints `PY-SUP-DEMO PASS`; any
    survivor prints the pid and FAILs (ORPHAN-PROOF under a Python host).

Sabotage levers, each ONE named point (C0, mirror of the cpp two-lever
pattern in scripts/demo-stage4-m2.sh):
  DEMO_PY_M2_SABOTAGE=1 - the parent SIGSTOPs the child instead of killing
    it, so the runner legitimately survives - the census MUST go red on it
    (read at the single kill decision below).
  DEMO_PY_M3_SABOTAGE=1 - corrupts the fidelity stanza's expected bytes
    before the comparison (read at the single `want` point below) - the
    gate MUST go red on a byte-exact child.
Exit 77 = core .so / fixture .so / runner binary absent (skip contract of
the cpp leg and examples/py/host_demo.py).

Payload codec note (inherited from host_demo.py, plan §3 M1 step-0 measured
RED on this box): the demo carries the hand-rolled minimal fixture codec
(precedent: host_demo.cc:184-218) - payload bytes only, never signatures:
every verb call below rides the generated PROTOTYPES. The fidelity EXPECTED
payload is pinned to the cpp stanza's own constant at
examples/cpp/host_sup_demo.cc:139 (`sup-fidelity-payload`).
"""

import ctypes
import os
import signal
import sys
import time
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO / "bindings" / "py"))
import modukit_embed as skin  # noqa: E402  (generated floor; import after path)

# The supervised record id == the fixture's canonical service id
# (mirror of host_sup_demo.cc:55 kServiceId and the manifest at :88).
K_ID = b"com.modukit.demo.DemoService"
# Fidelity expectation, single-source pinned to host_sup_demo.cc:139
# `req.set_payload("sup-fidelity-payload")`.
K_FIDELITY_PAYLOAD = b"sup-fidelity-payload"
# Step-domain dial delta the cpp stanza uses (host_sup_demo.cc:116, 1 s).
K_DIAL_DELTA_NS = 1000000000


def put_varint(value):
    out = bytearray()
    while True:
        byte = value & 0x7F
        value >>= 7
        if value:
            out.append(byte | 0x80)
        else:
            out.append(byte)
            return bytes(out)


def read_varint(buf, off):
    value = 0
    shift = 0
    while True:
        byte = buf[off]
        off += 1
        value |= (byte & 0x7F) << shift
        if not byte & 0x80:
            return value, off
        shift += 7


def encode_echo_request(payload):
    """EchoRequest{ bytes payload = 1 } (com.modukit.demo.proto) - byte-
    identical to protobuf SerializeAsString for this one-field message."""
    return b"\x0a" + put_varint(len(payload)) + payload


def decode_echo_reply(buf):
    """EchoReply{ bytes payload = 1; DemoStatus status = 2 } -> (payload,
    status). Same walk as host_demo.py (proto2 unknown fields skipped)."""
    payload = b""
    status = None
    off = 0
    while off < len(buf):
        key, off = read_varint(buf, off)
        field, wire = key >> 3, key & 7
        if wire == 2:
            length, off = read_varint(buf, off)
            value = buf[off : off + length]
            off += length
            if field == 1:
                payload = value
        elif wire == 0:
            value, off = read_varint(buf, off)
            if field == 2:
                status = value
        else:
            return payload, status
    return payload, status


def census(run_path):
    """Mirror of host_sup_demo.cc:57-84 - first pid (other than self) whose
    /proc cmdline carries the runner path substring, or -1 for none."""
    needle = run_path.encode()
    me = os.getpid()
    for entry in os.listdir("/proc"):
        if not entry.isdigit():
            continue
        pid = int(entry)
        if pid == me:
            continue
        try:
            with open("/proc/%s/cmdline" % entry, "rb") as f:
                cmd = f.read(4096)
        except OSError:
            continue  # vanished between listdir and open (cpp: fopen skip)
        if cmd and needle in cmd:
            return pid
    return -1


def walk_mode(core_so, fixture_so, run_path):
    """Mirror of host_sup_demo.cc walk_mode (:86-173), same print/fail words.
    Every sup call goes through the generated floor - no hand-written
    ctypes prototype lives in this file (D63 / D100 R2 floor)."""
    manifest = (
        'id = "com.modukit.demo.DemoService"\nversion = "0.1.0"\n'
        '[entry]\nkind = "exe"\nartifact = "%s"\nrunner = "%s"\n'
        % (fixture_so, run_path)
    )
    lib = skin.bind(ctypes.CDLL(core_so))
    if lib.modukit_embed_abi_version() != 1:
        sys.stderr.write("sup: embed ABI %d != 1\n" % lib.modukit_embed_abi_version())
        return 1
    id_out = ctypes.c_char_p()
    rc = lib.modukit_sup_install(manifest.encode(), ctypes.byref(id_out))
    if rc != 0 or id_out.value is None:
        sys.stderr.write("sup_install -> %d\n" % rc)
        return 1
    pid = id_out.value
    state = ctypes.c_int32(-1)
    if lib.modukit_sup_state(pid, ctypes.byref(state)) != 0 or (
        state.value != skin.PluginState.STATE_ACTIVE
    ):
        sys.stderr.write("sup state=%d want ACTIVE(4)\n" % state.value)
        return 1
    print("READY %d" % os.getpid())
    sys.stdout.flush()
    actions = ctypes.c_int32(-1)
    if lib.modukit_sup_step(ctypes.byref(actions)) != 0 or actions.value != 0:
        sys.stderr.write("sup step actions=%d want 0\n" % actions.value)
        return 1
    dstatus = ctypes.c_int32(0)
    dfired = ctypes.c_uint64(0)
    rc = lib.modukit_sup_dial(
        pid, K_DIAL_DELTA_NS, ctypes.byref(dstatus), ctypes.byref(dfired)
    )
    if rc != 0 or dstatus.value != 0:
        sys.stderr.write(
            "sup_dial must turn the step-domain child (raw=%d status=%d)\n"
            % (rc, dstatus.value)
        )
        return 1
    print("WALK WORDS OK (install/ACTIVE/step/dial-turn status=%d)" % dstatus.value)
    sys.stdout.flush()

    # FIDELITY stanza (D94 W3 mirrored): the runner's Echo answers with the
    # request payload verbatim, so the response payload bytes must equal the
    # pinned expectation exactly (host_sup_demo.cc:132-169).
    req_bytes = encode_echo_request(K_FIDELITY_PAYLOAD)
    resp = ctypes.create_string_buffer(len(req_bytes) + 256)
    rc = lib.modukit_sup_service_call(
        K_ID, b"echo", req_bytes, len(req_bytes), resp, len(resp)
    )
    want = K_FIDELITY_PAYLOAD
    if os.getenv("DEMO_PY_M3_SABOTAGE") is not None:
        # THE single named sabotage point for the fidelity gate: corrupt the
        # expected bytes before the comparison - the gate MUST go red on a
        # byte-exact child (mirror of host_sup_demo.cc:147-154).
        want = want[:-1] + bytes([want[-1] ^ 0x20])
        print("SABOTAGE: expected payload bytes corrupted")
        sys.stdout.flush()
    fidelity_ok = rc > 0
    if fidelity_ok:
        payload, status = decode_echo_reply(resp.raw[:rc])
        fidelity_ok = payload == want and status == 1  # DEMO_STATUS_OK
    if not fidelity_ok:
        sys.stderr.write("FIDELITY FAIL (sup_service_call raw=%d)\n" % rc)
        return 1  # no marker: the parent loop + gate stay red
    print("FIDELITY PASS (%d typed bytes out, payload byte-exact)" % rc)
    sys.stdout.flush()
    while True:
        time.sleep(1)  # the parent kills us; the runner must survive us by no time


def proof_mode(core_so, fixture_so, run_path):
    """Mirror of host_sup_demo.cc parent side (:211-311)."""
    r_fd, w_fd = os.pipe()
    kid = os.fork()
    if kid == 0:
        os.dup2(w_fd, 1)  # STDOUT_FILENO (cpp dup2(fds[1], STDOUT_FILENO))
        os.close(r_fd)
        os.close(w_fd)
        try:
            rc = walk_mode(core_so, fixture_so, run_path)
        except (OSError, ValueError) as exc:
            sys.stderr.write("walk child aborted: %s\n" % exc)
            rc = 1
        sys.stdout.flush()
        os._exit(rc)
    if kid < 0:
        return 1
    os.close(w_fd)
    rd = os.fdopen(r_fd, "r")

    def read_until(prefix):
        while True:
            line = rd.readline()
            if not line:
                return False  # child died before the marker (EOF)
            sys.stdout.write(line)
            sys.stdout.flush()
            if line.startswith(prefix):
                return True

    if not read_until("READY "):
        sys.stderr.write("SUP-DEMO FAIL (walk child never reached READY)\n")
        os.kill(kid, signal.SIGKILL)
        return 1
    # M3-mirror: the walk child must clear the fidelity stanza before the
    # kill. A child that dies without printing the marker lands here red -
    # the gate is load-bearing parent-side (PIT-18: the runner's own
    # empty-probe guard stays untouched).
    if not read_until("FIDELITY PASS"):
        sys.stderr.write(
            "SUP-DEMO FAIL (no FIDELITY PASS - walk child died at the door)\n"
        )
        os.kill(kid, signal.SIGKILL)
        os.waitpid(kid, 0)
        return 1

    sabotage = os.getenv("DEMO_PY_M2_SABOTAGE") is not None
    if sabotage:
        # THE single named sabotage point for the orphan census: stopping
        # the host instead of killing it leaves the runner's control fd
        # open - the census MUST find a live survivor, proving it is
        # load-bearing (mirror of host_sup_demo.cc:271-281).
        print("SABOTAGE: SIGSTOP instead of SIGKILL (runner expected live)")
        sys.stdout.flush()
        os.kill(kid, signal.SIGSTOP)
    else:
        os.kill(kid, signal.SIGKILL)
        os.waitpid(kid, 0)
    deadline_ms = 1000 if sabotage else 5000
    waited = 0
    while True:
        survivor = census(run_path)
        if survivor < 0:
            if sabotage:
                sys.stderr.write(
                    "SUP-DEMO FAIL (sabotage set but no survivor - "
                    "census is not load-bearing)\n"
                )
                os.kill(kid, signal.SIGKILL)
                return 1
            print("runner self-reaped under a Python host (orphan-proof)")
            print("PY-SUP-DEMO PASS")
            return 0
        if waited >= deadline_ms:
            if sabotage:
                print("SABOTAGE confirmed: runner pid=%d survived host stop" % survivor)
                os.kill(kid, signal.SIGKILL)
                return 1  # gate must be red under sabotage: it prints no PASS line
            sys.stderr.write(
                "SUP-DEMO FAIL: runner pid=%d orphaned after host kill\n" % survivor
            )
            os.kill(kid, signal.SIGKILL)
            return 1
        time.sleep(0.1)
        waited += 100


def main():
    args = [a for a in sys.argv[1:] if a != "--proof"]
    proof = len(args) != len(sys.argv) - 1
    if len(args) < 2:
        sys.stderr.write(
            "usage: host_sup_demo.py [--proof] <libmodukit_core.so> <plugin.so> [runner]\n"
        )
        return 2
    core_so, fixture_so = args[0], args[1]
    run_path = args[2] if len(args) > 2 else "out/cargo-target/debug/modukit-run"
    if not os.path.exists(core_so) or not os.path.exists(fixture_so) or not os.path.exists(run_path):
        sys.stderr.write(
            "sup demo: kernel, fixture or runner absent (%s | %s | %s) - skip\n"
            % (core_so, fixture_so, run_path)
        )
        return 77
    print("=== ModuKit wave-2 M2 Python sup demo (%s) ===" % ("proof" if proof else "walk"))
    sys.stdout.flush()
    return proof_mode(core_so, fixture_so, run_path) if proof else walk_mode(
        core_so, fixture_so, run_path
    )


if __name__ == "__main__":
    sys.exit(main())

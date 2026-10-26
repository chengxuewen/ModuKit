#!/usr/bin/env bash
# Wave-2 plugin-door PYTHON leg (leg twelve; D106 batch 2, plan T2.3/2.4):
# the hand-written skin bindings/py/plugin_runtime.py walks the exe wall as a
# kernel-spawned script child (runner = the absolute python3 path, artifact =
# a staged py_echo.py copy). Stages: [0] RUNNER line; [1] conformance BEFORE
# launching anything (all 14 golden frames re-encoded byte-exact through the
# skin's framing helpers + two hostile-frame plants); [2] embed-driver
# lifecycle - install -> FIDELITY (the cross-volume constant, byte-verbatim)
# -> caps -> BUS ROUNDTRIP (seed crosses out via FORWARD, self-echo crosses
# back via DELIVER, the derivative returns through `seen`) -> DIAL clean
# refusal (-27, child alive) -> cooperative stop -> liar answers the typed
# ServiceCallFailed (-25) twice while staying ACTIVE -> stop -> the next
# echo answers byte-exact again; [3] SIGKILL the driver, /proc census (the
# house law: never pgrep -f) asserts zero strays - the EOF path of the
# kernel's OWN child, not a driver-managed subprocess; [4] the C0 lever
# DEMO_PLUGIN_PY_SABOTAGE=1 skips the skin's REGISTER push at its one named
# point: the lifecycle must go red, no PASS line, exit 1 (the green run
# proves it inline; an externally-set lever reds the whole leg before
# stage [3]).
# PASS-line grammar mirrors scripts/demo-wave2-py-sup.sh; the cargo legs
# route through pixi per D65. Auto-SKIP only when an instrument is absent
# (no python3, no build toolchain at all, no pinned-env protoc - the
# cpp-band instrument contract) - a present toolchain that fails to build is
# RED, never a silent skip (C0). The PLUGIN child needs no .so at all (pure
# stdlib skin); only the driver loads libmodukit_core.so, plain ctypes - no
# LD_PRELOAD anywhere in this leg.
set -uo pipefail
cd "$(dirname "$0")/.."

export PATH="$HOME/.pixi/bin:$PATH"
# T3.1 runner-origin probe (wave-2 benchmark plan §4): pixi default env
# interpreter first, PATH fallback; neither present keeps the loud SKIP
# (marker string byte-identical). One RUNNER line per invocation, version
# read from the interpreter itself.
if [ -x ".pixi/envs/default/bin/python3" ]; then
  PYBIN=".pixi/envs/default/bin/python3"
  PYSRC="pixi-env"
elif command -v python3 >/dev/null 2>&1; then
  PYBIN="python3"
  PYSRC="PATH fallback - pixi env not warmed"
else
  echo "demo-wave2-plugin-py: SKIP (no python3: the skin has no interpreter to run in)"
  exit 0
fi
pyver=$("$PYBIN" --version 2>&1); echo "RUNNER: python ${pyver#Python } ($PYSRC)"
if command -v pixi >/dev/null 2>&1; then
  CARGO=(pixi run cargo)
elif command -v cargo >/dev/null 2>&1; then
  CARGO=(cargo)
else
  echo "demo-wave2-plugin-py: SKIP (no pixi and no cargo: no build toolchain on this box)"
  exit 0
fi
# Protoc pin (the instrument the cpp band auto-skips on) - same contract as
# scripts/demo-wave2-py.sh so the skins rise and fall together.
protoc_found=0
[ -x ".pixi/envs/default/bin/protoc" ] && protoc_found=1
command -v protoc >/dev/null 2>&1 && protoc_found=1
if [ "$protoc_found" -eq 0 ]; then
  echo "demo-wave2-plugin-py: SKIP (no pinned-env protoc: cpp-band instrument contract unmet)"
  exit 0
fi
# The skin + fixtures are tracked repo content now (gitignore carve-out
# batch-2 T2.4): absence = broken tree, RED by the same law as a failed build.
for f in bindings/py/plugin_runtime.py test-plugins/py/py_echo.py test-plugins/py/py_liar.py; do
  [ -f "$f" ] || { echo "demo-wave2-plugin-py: RED (tracked content missing: $f)"; exit 1; }
done

log=$(mktemp)
dlog=$(mktemp)
sablog=$(mktemp)
stage=$(mktemp -d)
trap 'rm -f "$log" "$dlog" "$sablog"; rm -rf "$stage"' EXIT

"${CARGO[@]}" build -p modukit-core >>"$log" 2>&1 || {
  echo "demo-wave2-plugin-py: RED (modukit-core cdylib build failed; last lines:)"
  tail -10 "$log"
  exit 1
}
core_so=""
for c in target/debug/libmodukit_core.so out/cargo-target/debug/libmodukit_core.so; do
  if [ -f "$c" ]; then core_so="$c"; break; fi
done
[ -n "$core_so" ] || { echo "demo-wave2-plugin-py: RED (no libmodukit_core.so after a green build)"; exit 1; }

# The live embed floor is REGENERATED through the emission door (D63/D100):
# the driver's prototypes are generated, never hand-written.
scratch=$(mktemp -d)
"${CARGO[@]}" run -q -p modukit-codegen --bin protoc-gen-modukit -- \
  "@contracts/org.modukit.runtime.proto.fdset" \
  "@contracts/com.modukit.demo.proto.fdset" \
  --embed_header contracts/modukit_embed.h \
  --output_dir "$scratch" --py_out bindings/py >>"$log" 2>&1 || {
  echo "demo-wave2-plugin-py: RED (skin regeneration failed; last lines:)"
  tail -10 "$log"
  rm -rf "$scratch"
  exit 1
}
rm -rf "$scratch"
[ -f bindings/py/modukit_embed.py ] || { echo "demo-wave2-plugin-py: RED (bindings/py/modukit_embed.py not regenerated)"; exit 1; }

# Stage the plugin artifacts (the manifest names staged paths, runner and
# artifact both absolute and existing - the spawn doorway checks is_file).
cp test-plugins/py/py_echo.py test-plugins/py/py_liar.py "$stage/"
# Absolutize the runner path (the kernel is_file-checks the manifest runner
# - executor_proc.rs:225 refuses bare "python3").
case "$PYBIN" in
  */*)
    case "$PYBIN" in
      /*) PYABS="$PYBIN";;
      *) PYABS="$PWD/$PYBIN";;
    esac;;
  *) PYABS=$(command -v "$PYBIN");;
esac
[ -f "$PYABS" ] || { echo "demo-wave2-plugin-py: RED (runner path not an existing file: $PYABS)"; exit 1; }

# --- [1] conformance: BEFORE launching anything. Decode + re-encode every
# golden frame byte-exactly through the skin, then plant the two hostile
# frames (length over FRAME_MAX refused pre-alloc; trailing bytes refused).
cat > "$stage/conformance.py" <<'PYEOF'
# Scratch conformance probe for scripts/demo-wave2-plugin-py.sh stage [1]:
# the goldens (contracts/golden/wire-v1) are the ledger's published truth;
# the skin's own codecs must re-encode each one byte-for-byte. Args:
# <bindings-py-dir> <golden-dir>
import os
import struct
import sys

sys.path.insert(0, sys.argv[1])
import plugin_runtime as rt  # noqa: E402

GOLD = sys.argv[2]

# Per-kind canonicalization: decode the payload, re-encode through the
# mirror encoder - byte identity proves BOTH directions of the codec.
def canonical(kind, payload):
    if kind in (rt.MSG_START, rt.MSG_STOP):
        if payload:
            raise rt.FrameReject("%s must carry the EMPTY payload" % kind)
        return payload
    if kind == rt.MSG_HELLO:
        abi, pid, ver, name, caps = rt.parse_hello(payload)
        return rt.hello_payload(abi, pid, ver, name, caps)
    if kind == rt.MSG_ACK:
        return rt.ack_payload(rt.parse_ack(payload))
    if kind == rt.MSG_REGISTER:
        sid, methods = rt.parse_register(payload)
        return rt.register_payload(sid, methods)
    if kind == rt.MSG_CALL:
        seq, method, inp = rt.parse_call(payload)
        return rt.call_payload(seq, method, inp)
    if kind == rt.MSG_RESULT:
        seq, status, out = rt.parse_result(payload)
        return rt.result_payload(seq, status, out)
    if kind == rt.MSG_DIAL:
        return rt.dial_payload(rt.parse_dial(payload))
    if kind == rt.MSG_DUE:
        status, fired, jobs = rt.parse_due(payload)
        return rt.due_payload(status, fired, jobs)
    if kind == rt.MSG_FORWARD:
        op, topic, qos, value = rt.parse_forward(payload)
        return rt.forward_payload(op, topic, qos, value)
    if kind == rt.MSG_DELIVER:
        topic, qos, value = rt.parse_deliver(payload)
        return rt.deliver_payload(topic, qos, value)
    raise rt.FrameReject("golden carries an unknown kind %d" % kind)

names = [
    "HELLO_plain", "HELLO_caps", "START", "STOP", "ACK", "REGISTER", "CALL",
    "RESULT", "DIAL", "DUE", "FORWARD_PUB", "FORWARD_SUB", "FORWARD_UNSUB",
    "DELIVER",
]
count = 0
for n in names:
    with open(os.path.join(GOLD, n + ".frame"), "rb") as f:
        raw = f.read()
    plen, ver, kind = rt.decode_header(raw[:6])
    if ver != rt.WIRE_VERSION:
        raise rt.FrameReject("%s: foreign version %d" % (n, ver))
    if len(raw) != 6 + plen:
        raise rt.FrameReject("%s: declared %d, file holds %d" % (n, plen, len(raw) - 6))
    payload = raw[6:]
    if rt.encode_frame(kind, canonical(kind, payload)) != raw:
        raise rt.FrameReject("%s: re-encode is not byte-exact" % n)
    count += 1
if count != 14:
    raise AssertionError("expected 14 goldens, walked %d" % count)

class _Shim:
    """Socket-shaped reader: counts the LARGEST recv request so the
    pre-alloc refusal is provable (a >FRAME_MAX length must never reach a
    payload read - the proc_wire.rs:140-149 posture)."""

    def __init__(self, data):
        self.data = data
        self.off = 0
        self.max_request = 0

    def recv(self, n):
        self.max_request = max(self.max_request, n)
        part = self.data[self.off:self.off + n]
        self.off += len(part)
        return part

# Hostile 1: length over FRAME_MAX - refused BEFORE any allocation.
shim = _Shim(struct.pack("<IBB", 0xFFFFFFFF, rt.WIRE_VERSION, rt.MSG_CALL) + b"junk")
try:
    rt.read_frame(shim)
    raise AssertionError("hostile length accepted")
except rt.FrameTooLarge:
    pass
if shim.max_request > 6:
    raise AssertionError("payload read requested despite the over-ceiling length")

# Hostile 2: trailing bytes - refused by the strict parse (the ledger's
# "silence and lies are both disagreement" line).
try:
    with open(os.path.join(GOLD, "REGISTER.frame"), "rb") as f:
        raw = f.read()
    rt.parse_register(raw[6:] + b"\x00")
    raise AssertionError("trailing bytes accepted")
except rt.FrameReject:
    pass

# Write-side pre-check too (the two-way ceiling): an over-ceiling OUTBOUND
# payload must be refused before any send (Bridge.talk precedent).
try:
    rt.encode_frame(rt.MSG_FORWARD, b"x" * (rt.FRAME_MAX + 1))
    raise AssertionError("over-ceiling write accepted")
except rt.FrameTooLarge:
    pass

print("wave2-plugin-py: CONFORMANCE OK (14 frames + 2 hostile)")
PYEOF
if ! "$PYBIN" "$stage/conformance.py" "$PWD/bindings/py" "$PWD/contracts/golden/wire-v1" 2>&1 | tee "$log"; then
  echo "demo-wave2-plugin-py: RED (conformance python exited non-zero)"
  exit 1
fi
if ! grep -q 'wave2-plugin-py: CONFORMANCE OK (14 frames + 2 hostile)' "$log"; then
  echo "demo-wave2-plugin-py: RED (no CONFORMANCE OK line - the codec walk did not complete)"
  exit 1
fi

# --- [2]+[3] driver. Runs the sup-rung lifecycle as its OWN process: the
# leg SIGKILLs it mid-life for the census, so the orphan walk exercises the
# kernel child's EOF path (the .so dies WITH the driver; the plugin child is
# the kernel's own child, not one this shell ever managed).
cat > "$stage/driver.py" <<'PYEOF'
# Scratch embed driver for scripts/demo-wave2-plugin-py.sh stage [2]: drives
# the GENERATED ctypes floor (bindings/py/modukit_embed.py) - sup_install on
# script-shaped exe manifests, the FIDELITY / BUS / liar / stop walk, then
# sleeps with a live child so the leg can SIGKILL it for the census. Args:
# <core-so> <bindings-py-dir> <stage-dir> <runner-abs> <seed>
import ctypes
import os
import sys
import time

core_so, bindings_py, stage, runner = sys.argv[1:5]
seed = sys.argv[5].encode()

sys.path.insert(0, bindings_py)
import modukit_embed as skin  # noqa: E402  (generated floor; import after path)

ID_ECHO = "org.modukit.demo.pyecho"
ID_LIAR = "org.modukit.demo.pyliar"
VERSION = "0.1.0"
ACTIVE = skin.PluginState.STATE_ACTIVE
# D51 codes as the ledger numbers them (modukit_embed.h, embed_codec.rs):
NEG_NOT_FOUND = -14
NEG_SERVICE_CALL_FAILED = -25
NEG_CLOCK_DIAL_REFUSED = -27
K_DIAL_DELTA_NS = 1000000000

# Children inherit these from the driver process (the spawn env is the
# driver's own - same channel the clock lever rides in modukit-run):
# the skin shelf for the STAGED copies, and the roundtrip seed.
os.environ["MODUKIT_PY_SKIN"] = bindings_py
os.environ["MODUKIT_PY_SEED"] = seed.decode()

lib = skin.bind(ctypes.CDLL(core_so))
if lib.modukit_embed_abi_version() != 1:
    sys.stderr.write("driver: embed ABI %d != 1\n" % lib.modukit_embed_abi_version())
    sys.exit(1)


def fail(msg):
    sys.stderr.write("PLUGIN-PY FAIL: %s\n" % msg)
    sys.exit(1)


def manifest(id_str, artifact):
    # The kernel cross-checks HELLO id/version against the manifest (strict,
    # executor_proc.rs:301-314): id = the skin's serve id, version = "0.1.0".
    return (
        'id = "%s"\nversion = "%s"\n[entry]\nkind = "exe"\n'
        'artifact = "%s"\nrunner = "%s"\n' % (id_str, VERSION, artifact, runner)
    )


def install(id_str, artifact):
    id_out = ctypes.c_char_p()
    rc = lib.modukit_sup_install(manifest(id_str, artifact).encode(), ctypes.byref(id_out))
    if rc != 0 or id_out.value is None:
        fail("sup_install(%s) -> %d" % (id_str, rc))
    return id_out.value


def want_active(pid, label):
    state = ctypes.c_int32(-1)
    if lib.modukit_sup_state(pid, ctypes.byref(state)) != 0 or state.value != ACTIVE:
        fail("%s must be ACTIVE(4), state=%d" % (label, state.value))


def call(pid_service, method, inp):
    resp = ctypes.create_string_buffer(max(len(inp), 1) + 4096)
    rc = lib.modukit_sup_service_call(pid_service, method, inp, len(inp), resp, len(resp))
    return rc, resp.raw[:max(rc, 0)]


def stop_uninstall(pid, label):
    rc = lib.modukit_sup_stop(pid)
    if rc != 0:
        fail("%s cooperative stop -> %d" % (label, rc))
    rc = lib.modukit_sup_uninstall(pid)
    if rc != 0:
        fail("%s uninstall -> %d" % (label, rc))


# --- py_echo lifecycle. ONE service id at a time: sup_service_call acquires
# the top-ranked proxy (embed_sup.rs), so the liar only lives once echo is
# unregistered (this also proves the proxy teardown pairs with every stop).
echo_py = os.path.join(stage, "py_echo.py")
pid = install(ID_ECHO, echo_py)
want_active(pid, "py_echo")

rc, out = call(ID_ECHO.encode(), b"echo", seed)
if rc != len(seed) or out != seed:
    fail("FIDELITY echo -> rc=%d bytes=%r (the %d-byte constant must come back verbatim)"
         % (rc, out, len(seed)))
print("FIDELITY PASS (%d bytes verbatim through the script skin)" % rc)

rc, out = call(ID_ECHO.encode(), b"caps", b"")
if rc != 1 or out != b"\x01":
    fail("caps -> rc=%d out=%r want b'\\x01'" % (rc, out))
print("CAPS PASS")

# BUS ROUNDTRIP: the seed crossed out via FORWARD(PUB) in the start window,
# the parent's self-echo (D96 ruling 6) sent it back as a DELIVER, the
# callback's derivative crossed out again - `seen` returns the last payload
# the skin's bus callback received.
rc, out = call(ID_ECHO.encode(), b"seen", b"")
if rc != len(seed) + 4 or out != seed + b"|bus":
    fail("BUS ROUNDTRIP -> rc=%d out=%r want %r (crossing did not complete)"
         % (rc, out, seed + b"|bus"))
print("BUS ROUNDTRIP PASS (seed out, echo back, derivative in `seen`)")

# DIAL clean refusal: the skin answers DUE(DIAL_STATUS_REFUSED_REAL) and the
# parent maps it typed - the child must SURVIVE the refusal (a live,
# cooperative, undialable plugin - executor_proc.rs:850-855).
ds = ctypes.c_int32(0)
df = ctypes.c_uint64(0)
rc = lib.modukit_sup_dial(pid, K_DIAL_DELTA_NS, ctypes.byref(ds), ctypes.byref(df))
if rc != NEG_CLOCK_DIAL_REFUSED:
    fail("sup_dial must surface the clean refusal -27, got %d" % rc)
rc, out = call(ID_ECHO.encode(), b"echo", b"alive?")
if rc != 6 or out != b"alive?":
    fail("child must answer after the refusal (rc=%d)" % rc)
print("DIAL REFUSED PASS (-27, child alive)")

# Cooperative stop (ACK path of the skin) then uninstall - the proxy leaves
# with the run, clearing the registry for the liar's single proxy.
stop_uninstall(pid, "py_echo")
print("STOP PASS (cooperative ack + uninstall)")

# --- py_liar: every call answers the author code -1; the door maps it to
# SERVICE_CALL_FAILED. Two calls both fail TYPED and the plugin stays
# ACTIVE between them - the i32 channel is a contract, not decoration.
liar_py = os.path.join(stage, "py_liar.py")
pid = install(ID_LIAR, liar_py)
want_active(pid, "py_liar")
rc, _ = call(ID_LIAR.encode(), b"lie", b"x")
if rc != NEG_SERVICE_CALL_FAILED:
    fail("liar must answer -25 typed, got %d" % rc)
rc, _ = call(ID_LIAR.encode(), b"lie", b"x")
want_active(pid, "py_liar after the lie")
if rc != NEG_SERVICE_CALL_FAILED:
    fail("second liar call -> %d" % rc)
print("LIAR PASS (typed -25 twice, plugin stayed ACTIVE)")

# The census needs a live child at the kill; the "next echo works" proof is
# py_echo re-installed (single-proxy door again after the liar unregisters).
stop_uninstall(pid, "py_liar")
pid = install(ID_ECHO, echo_py)
want_active(pid, "py_echo (post-liar)")
rc, out = call(ID_ECHO.encode(), b"echo", seed)
if rc != len(seed) or out != seed:
    fail("post-liar echo must answer byte-exact again (rc=%d)" % rc)
print("NEXT ECHO PASS (liar episode cost the family nothing)")

print("READY %d" % os.getpid())
print("LIFECYCLE PASS")
sys.stdout.flush()
while True:
    time.sleep(1)  # the leg SIGKILLs the driver; the plugin rides the EOF
PYEOF
"$PYBIN" "$stage/driver.py" "$PWD/$core_so" "$PWD/bindings/py" "$stage" "$PYABS" \
  "sup-fidelity-payload" >"$dlog" 2>&1 &
dpid=$!
ok=""
for _ in $(seq 1 600); do
  if grep -q '^LIFECYCLE PASS' "$dlog" 2>/dev/null; then ok=1; break; fi
  if ! kill -0 "$dpid" 2>/dev/null; then break; fi
  sleep 0.1
done
if [ -z "$ok" ]; then
  cat "$dlog"
  echo "demo-wave2-plugin-py: RED (lifecycle driver failed or died before the marker)"
  exit 1
fi

# --- [3] orphan census: SIGKILL the driver mid-life (kernel .so gone with
# it), then the /proc walk asserts the staged children self-exited on the
# control EOF. House law: never pgrep -f (it can match this script's own
# command line); os.walk over /proc/<pid>/cmdline, mirror of
# examples/py/host_sup_demo.py:122-140.
# Kill + reap in bash (PIT-15: cleanup legs carry || true), then the scan.
kill -9 "$dpid" 2>/dev/null || true
wait "$dpid" 2>/dev/null || true
"$PYBIN" - "$stage" <<'PYEOF'
import os
import sys
import time

stage = sys.argv[1]
needle = stage.encode() + b"/py_"
deadline = time.monotonic() + 5.0
survivors = []
while time.monotonic() < deadline:
    survivors = []
    for entry in os.listdir("/proc"):
        if not entry.isdigit():
            continue
        try:
            with open("/proc/%s/cmdline" % entry, "rb") as f:
                cmd = f.read(4096)
        except OSError:
            continue  # vanished between listdir and open (py-sup census twin)
        if needle in cmd:
            survivors.append(entry)
    if not survivors:
        break
    time.sleep(0.05)
if survivors:
    sys.stderr.write("CENSUS FAIL: staged plugin pids survived: %s\n" % ",".join(survivors))
    sys.exit(1)
print("CENSUS PASS (zero staged-plugin survivors after the driver SIGKILL)")
PYEOF
census_rc=$?
cat "$dlog"
if [ "$census_rc" -ne 0 ]; then
  echo "demo-wave2-plugin-py: RED (orphan census found strays - EOF path broken)"
  exit 1
fi

# --- [4] sabotage proof, inline every run (C0: a check is only trusted
# after it is OBSERVED red). The lever lands at the ONE named point in the
# skin (bindings/py/plugin_runtime.py, the REGISTER loop): skipping the
# announcement makes the driver's first service_call NOT_FOUND (-14), the
# lifecycle must die before the marker. Exit non-zero AND no marker = green.
DEMO_PLUGIN_PY_SABOTAGE=1 "$PYBIN" "$stage/driver.py" "$PWD/$core_so" "$PWD/bindings/py" \
  "$stage" "$PYABS" "sup-fidelity-payload" >"$sablog" 2>&1
sab_rc=$?
if [ "$sab_rc" -eq 0 ] || grep -q '^LIFECYCLE PASS' "$sablog"; then
  cat "$sablog"
  echo "demo-wave2-plugin-py: RED (sabotage run went GREEN - the lever is not load-bearing)"
  exit 1
fi
if ! grep -q 'FIDELITY echo -> rc=-14' "$sablog"; then
  cat "$sablog"
  echo "demo-wave2-plugin-py: RED (sabotage run died at the wrong door - expect -14 NOT_FOUND)"
  exit 1
fi
echo "SABOTAGE PASS (REGISTER skipped -> lifecycle red, exit $sab_rc)"

echo "demo-wave2-plugin-py: PASS"

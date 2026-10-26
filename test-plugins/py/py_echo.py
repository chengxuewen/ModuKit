#!/usr/bin/env python3
# test-plugins/py/py_echo.py - the wave-2 plugin-door echo fixture
# (D106 batch 2, plan T2.2: docs/wave-2-plugin-door-plan.md section 4).
# The kernel spawns it as `python3 <this-file> --fd=0` (manifest kind=exe,
# runner=<absolute interpreter path>); the control socket IS fd 0 and
# stdout/stderr are nulled - this file must never print.
#
# Three services' worth of behavior behind ONE service id: the embed face's
# sup_service_call door acquires the TOP-RANKED proxy only (embed_sup.rs -
# a second id answers NOT_FOUND), so the leg's addressable word is
# method-scoped: echo = verbatim bytes (the fidelity constant rides it),
# caps = one fixed byte, seen = the last payload the bus callback received
# (the roundtrip's read-back leg).
#
# The bus topic carries a seed: the seed bytes arrive through the spawn
# environment (MODUKIT_PY_SEED) because the embed face has NO bus verb on
# the sup rung - the two rungs own separate kernels (embed_sup.rs header:
# "buses... separate") and the wall is transparent only inside the start
# window (stage-5-plan section 3). The runtime publishes the seed IN that
# window; the parent's self-echo (D96 ruling 6) sends it back as a DELIVER,
# on_bus fires here, and the derivative goes out the same way - two full
# crossings of driver-originated bytes, observable back via `seen`.
#
# Runtime resolution (the robust minimal pick, stated): the MODUKIT_PY_SKIN
# env var wins (the leg stages this file OUTSIDE the repo tree, where the
# __file__-relative default cannot reach bindings/py); the fallback is the
# in-place repo layout (<repo>/test-plugins/py/x -> <repo>/bindings/py), so
# the fixture also runs straight from a checkout without any env.

import os
import sys

ID = "org.modukit.demo.pyecho"
VERSION = "0.1.0"
NAME = "py-echo"
TOPIC = "org.modukit.demo.wire-echo"
DERIV = b"|bus"  # the derivative marker the driver asserts on


def _find_skin():
    cands = []
    env = os.environ.get("MODUKIT_PY_SKIN")
    if env:
        cands.append(env)
    here = os.path.dirname(os.path.abspath(__file__))
    cands.append(os.path.normpath(os.path.join(here, "..", "..", "bindings", "py")))
    for c in cands:
        if os.path.isfile(os.path.join(c, "plugin_runtime.py")):
            return c
    sys.exit(5)  # no console to complain to: the code IS the diagnosis


sys.path.insert(0, _find_skin())
import plugin_runtime as rt  # noqa: E402

state = {"seen": b"", "republished": False}


def on_call(method, inp):
    if method == "echo":
        return (0, inp)
    if method == "caps":
        return (0, b"\x01")
    if method == "seen":
        return (0, state["seen"])
    return (-1, b"")  # the child's own code is the unknown-method authority


def on_bus(topic, payload):
    state["seen"] = payload
    if topic == TOPIC and not state["republished"]:
        # Republish exactly once: the derivative rides the same topic and
        # re-enters through the same self-echo - an unguarded callback would
        # loop the start window against its own drain.
        state["republished"] = True
        rt.publish(TOPIC, payload + DERIV)


seed = os.environ.get("MODUKIT_PY_SEED", "")
# Seeded entry = interest + initial in-window publish (runtime T2.1 shape);
# seedless stays a plain SUB (the wall is then quiet, as in the flag-off
# runner columns of abi_v2_cross.rs).
topics = [(TOPIC, seed.encode("utf-8"))] if seed else [TOPIC]

rt.serve(ID, VERSION, NAME,
         services={ID: ["echo", "caps", "seen"]},
         on_call=on_call,
         on_bus=on_bus,
         topics=topics)

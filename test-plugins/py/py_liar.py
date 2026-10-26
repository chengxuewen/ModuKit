#!/usr/bin/env python3
# test-plugins/py/py_liar.py - the wave-2 plugin-door liar fixture (D106
# batch 2, plan T2.2): declares ONE service and answers every CALL with
# RESULT status=-1 (a D27 author code) and empty output. THE LEG CONSUMES
# IT: the kernel-side proxy must surface the lie as the typed
# SERVICE_CALL_FAILED on the embed channel while the plugin stays ALIVE and
# cooperative - the i32 status channel is a contract, not decoration
# (proc_wire.rs:48-53 doc; proc_proxy.rs returns the author code without
# killing). No bus topics: this skin also proves the tail-less legacy
# HELLO grammar path end to end (caps 0 -> no 4-byte tail).
# Runtime resolution: same two-candidate rule as py_echo.py (env wins,
# in-place relative fallback).

import os
import sys

ID = "org.modukit.demo.pyliar"
VERSION = "0.1.0"
NAME = "py-liar"


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
    sys.exit(5)


sys.path.insert(0, _find_skin())
import plugin_runtime as rt  # noqa: E402


def on_call(method, inp):
    return (-1, b"")  # the lie, verbatim on the D27 channel


rt.serve(ID, VERSION, NAME, services={ID: ["lie"]}, on_call=on_call)

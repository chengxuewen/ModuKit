# Kernel Contract (SDD)

> Test annotation: `// spec: KERNEL-01`; gate: `scripts/spec-trace.sh`.

## KERNEL-01: lifecycle transition table

The kernel exposes exactly these lifecycle states: `UNINSTALLED`, `INSTALLED`,
`RESOLVING`, `STARTING`, `ACTIVE`, `STOPPING`, `RESOLVED`, `FAILED`.

Only the legal edges are accepted: install `UNINSTALLED -> INSTALLED`; resolve
`INSTALLED/RESOLVED -> RESOLVING -> RESOLVED/FAILED`; start
`RESOLVED -> STARTING -> ACTIVE/FAILED`; stop `ACTIVE -> STOPPING -> RESOLVED`;
uninstall `RESOLVED/FAILED -> UNINSTALLED`. Any other request returns the typed
lifecycle error and leaves the state unchanged. The exe-placement walk must follow
the same table across the control socket.

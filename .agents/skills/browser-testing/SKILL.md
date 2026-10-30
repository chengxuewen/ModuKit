---
name: browser-testing
status: inoperative
type: browser
notes: "inoperative: no web surface; UI tier demoted (D89-K2)"
description: "Browser-side verification workflow for a future ModuKit UI surface. Inoperative until the UI tier reopens; kept so the method is ready."
---

# Browser Testing (inoperative)

**Status: inoperative.** ModuKit has no web or wasm demo wall today; the UI tier is
demoted behind a named demand trigger (D89-K2). Do not route a task here until that
trigger fires.

When a browser-facing surface lands:

- Build the page artifact from the same source as the native host; no hand-written
  duplicate.
- Gate the page headlessly (Playwright or DevTools): boot, console errors, network
  failures, and a real frame/heartbeat counter.
- Capture a screenshot and the counter value as evidence; a blank page is not a pass.
- Wire the page gate into `pixi run ci` only when the surface is shipped; until then
  the directory stays inoperative and costs nothing.

The VE profile in `docs/reference/visiaengine.md` is the donor reference for the
future workflow, not an active instruction for this repository.

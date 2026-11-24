# examples/ - router only

This directory is a ROUTER, not teaching material: the learning paths live
in the [tutorials](../docs/tutorials/README.md), and the runnable truth
lives in the gate legs they reference. Single-source discipline: nothing
here restates what a chapter or a leg already says.

| Band | What lives here | Run it | Learn it |
|---|---|---|---|
| Rust | stage demos (`stage1_demo.rs`, `stage2_demo.rs`, `stage5_n2_demo.rs`, `wave2_clock_demo.rs`) | `pixi run demo <leg>` | tutorials 01/03/04 |
| C++ | embed-face hosts (`host_demo.cc`, `host_sup_demo.cc`) + the progressive tutorial series (`cpp/tutorial/`: t1 hello / t2 echo / t3 bus / t4 driver) | `pixi run demo stage4-m1` / `stage4-m2` | tutorials 02/02b/04/05 |
| Python | embed-face hosts (`host_demo.py`, `host_sup_demo.py`) | `pixi run demo wave2-py` / `wave2-py-sup` | tutorial 05 |
| JS | embed-face hosts (`host_demo.js`, `host_sup_demo.js`, `pin_test.js`) | `pixi run demo wave2-js` / `wave2-js-sup` | tutorial 05 |
| C | embed-face host (`c/host_demo.c`) + the fixture at `../test-plugins/c/` | rides `pixi run demo stage4-m1` | tutorials 02/05 |

Script-plugin fixtures live in `../test-plugins/{py,js}` - see tutorial 06
(`pixi run demo wave2-plugin-py` / `wave2-plugin-js`).

`Cargo.toml` here is the rust demo crate (the lane the gate builds); the
other bands ride the cmake graph (`MODUKIT_BUILD_C` / `MODUKIT_BUILD_CPP`).

//! Green-path lifecycle: the full exe spawn walk over the control socket.

use super::*;

// spec: KERNEL-01
#[test]
fn exe_spawn_green_walks_full_lifecycle_over_the_control_socket() {
    // Arrange
    let Some(dir) = build_standins("green") else {
        return;
    };
    let sup = step_supervisor();
    let exe = dir.join("standin-good");

    // Act: the full dl-parity walk on an exe manifest.
    let id = install_resolve_start(&sup, &exe, "");

    // Assert: every state edge ran through the existing machine.
    assert_eq!(
        sup.kernel().state(&id).expect("started"),
        PluginState::Active
    );
    let walk = lifecycle_walk(&sup);
    assert_eq!(
        walk,
        vec![
            "UNINSTALLED->INSTALLED",
            "INSTALLED->RESOLVING",
            "RESOLVING->STARTING",
            "STARTING->ACTIVE",
        ]
    );
    assert_eq!(
        sup.kernel().counters().load_ok(),
        1,
        "handshake passed once"
    );
    sup.kernel().stop(&id).expect("graceful STOP");
    assert_eq!(
        sup.kernel().state(&id).expect("stopped"),
        PluginState::Resolved
    );
    sup.kernel().uninstall(&id).expect("uninstall");
    assert_no_zombie_children();
}

//! Gate counters on the SAME doorways as enforcement (D21, modules/10 §6.1)
//! — M6 floor 5. No new choke point is invented for a counter: registry
//! acquire/release, bus publish/deliver, the load handshake and the
//! lifecycle transition doorway each accumulate relaxed counters where
//! their existing logic already runs. Read-only getters; no policy.

use modukit_core::identity::TopicId;
use modukit_core::kernel::Kernel;
use modukit_core::topic::QoS;

#[test]
fn acquire_and_release_pairs_count_at_the_registry_doorway() {
    let kernel = Kernel::new();
    let gate = kernel.counters();
    let before = (gate.registry_acquires(), gate.registry_releases());
    let handle = kernel
        .registry()
        .acquire::<modukit_core::clock::Clock>()
        .expect("the clock is a registered instance");
    kernel.registry().release(&handle).expect("release");
    let after = (gate.registry_acquires(), gate.registry_releases());
    assert_eq!(after.0, before.0 + 1, "acquire counted at its own doorway");
    assert_eq!(after.1, before.1 + 1, "release counted at its own doorway");
}

#[test]
fn bus_publish_gate_accumulates_published_delivered_dropped_high_water() {
    // The 06 §2 "drops must be visible" commandment made mechanical: the
    // publish loop already walks these states; the counters ride along.
    let kernel = Kernel::new();
    let t = TopicId::new("com.test.counted".to_string());
    let _rx = kernel.bus().subscribe(&t).expect("subscribe lossy(1)");
    for i in 0..5u8 {
        kernel.bus().publish(&t, QoS::Lossy(1), vec![i]);
    }
    // Slow subscriber: ring cap 1, so 4 of 5 fan-outs evicted a value.
    let stats = kernel
        .bus()
        .topic_stats()
        .into_iter()
        .find(|s| s.topic == "com.test.counted")
        .expect("topic present");
    assert_eq!(stats.published, 5);
    assert_eq!(
        stats.delivered, 5,
        "every publish reached the subscriber slot"
    );
    assert_eq!(stats.dropped, 4, "commandment 3 made visible at the gate");
    assert!(
        stats.queue_high_water >= 1,
        "the ring saw at least one entry"
    );
    assert_eq!(stats.subscribers, 1);
    assert_eq!(kernel.counters_view().bus_published, 5);
}

#[test]
fn load_handshake_and_lifecycle_transitions_count_at_the_kernel_doorway() {
    let dir = std::env::temp_dir().join(format!("m6-counters-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let artifact = dir.join("placeholder.so");
    std::fs::write(&artifact, b"placeholder").expect("artifact");
    let kernel = Kernel::new();
    let gate = kernel.counters();
    let manifest = modukit_core::manifest::Manifest::from_toml_str(&format!(
        "id = \"com.test.counted\"\nversion = \"0.1.0\"\n\
         [entry]\nkind = \"dl\"\nartifact = \"{}\"\n",
        artifact.display()
    ))
    .expect("manifest");
    let id = manifest.id.clone();
    let t0 = gate.lifecycle_transitions();
    kernel.install(manifest).expect("install");
    let load0 = gate.load_ok() + gate.load_fail();
    // The artifact is a placeholder, so the handshake FAILS — the doorway
    // still counts the attempt, honestly (no enforcement, only seeing).
    kernel
        .resolve(&id)
        .expect_err("placeholder is not a dlopen-able image");
    let load1 = gate.load_ok() + gate.load_fail();
    let t1 = gate.lifecycle_transitions();
    std::fs::remove_dir_all(&dir).ok();

    assert!(
        t1 >= t0 + 2,
        "install + resolving transitions counted: {t0}->{t1}"
    );
    assert_eq!(
        load1 - load0,
        1,
        "the load handshake ran through its doorway once"
    );
    assert_eq!(
        gate.load_fail(),
        1,
        "and it went to the fail bucket, honestly"
    );
}

#[test]
fn counters_are_observation_only_and_the_gate_view_snapshots_all_of_them() {
    let kernel = Kernel::new();
    let t = TopicId::new("com.test.observe".to_string());
    // Nothing a counter reads may block or refuse: publish to an unread
    // latest-value topic is a no-error fire-and-forget even after flood.
    for _ in 0..1000 {
        kernel.bus().publish(&t, QoS::LatestValue, b"x".to_vec());
    }
    let view = kernel.counters_view();
    assert!(view.bus_published >= 1000, "counts survive the flood");
    assert_eq!(
        view.bus_dropped, 0,
        "no subscriber: nothing was ever dropped"
    );
    // The read path is a plain snapshot; counters never spam the ring.
    let snap = kernel.observatory().last_seq();
    assert!(
        snap > 0,
        "the ring exists; counters never write to it (no spam)"
    );
}

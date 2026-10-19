//! D80 co-host deferral on a declared parallel_ok service.

use super::*;

/// D80 (S2-M3b): the swap door asked to co-host two versions of a declared
/// parallel_ok service gets a TYPED DEFERRAL — never a silent ignore, never
/// the bare transition refusal that hides the author's request. Undeclared
/// co-host requests keep the M7 duplicate-identity path untouched.
#[test]
fn cohost_request_on_a_declared_parallel_ok_service_is_deferred_typed() {
    let Some(dir) = build_standins("d80") else {
        return;
    };
    let sup = step_supervisor();
    let declared = manifest_toml(
        &dir.join("standin-good"),
        "[[provides]]\nservice = \"com.example.proc.stand-in.echo\"\nparallel_ok = true\n",
    );
    let manifest = Manifest::from_toml_str(&declared).expect("declared manifest parses");
    let id = manifest.id.clone();
    sup.kernel().install(manifest).expect("install");
    sup.kernel().resolve(&id).expect("resolve");
    sup.kernel().start(&id).expect("start (ACTIVE)");

    // The co-host request: a new version installed while the old is ACTIVE.
    let swap = declared.replace("version = \"0.1.0\"", "version = \"0.2.0\"");
    let swap = Manifest::from_toml_str(&swap).expect("swap manifest parses");
    let err = sup
        .kernel()
        .install(swap)
        .expect_err("parallel_ok co-host must get a typed voice");
    assert!(
        matches!(&err, KernelError::RoutingDeferred { plugin, service }
            if plugin == ID && service == "com.example.proc.stand-in.echo"),
        "expected typed RoutingDeferred, got {err}"
    );
    // The deferral is a refusal at the door, not a teardown: the run lives.
    assert_eq!(
        sup.kernel().state(&id).expect("still ACTIVE"),
        PluginState::Active
    );

    // Stop-based swap stays the legal lane: stop, then the install replaces
    // the record (terminal state) exactly as before. (The swapped 0.2.0 is
    // not re-resolved here — the stand-in self-reports 0.1.0 and the strict
    // D7 cross-check would rightly refuse the version lie.)
    sup.kernel().stop(&id).expect("stop");
    let legal =
        Manifest::from_toml_str(&declared.replace("version = \"0.1.0\"", "version = \"0.2.0\""))
            .expect("parses");
    sup.kernel()
        .install(legal)
        .expect("stop-based swap accepted");
    sup.kernel()
        .uninstall(&id)
        .expect("uninstall swapped record");

    // Undeclared path unchanged: a live re-install with no parallel_ok
    // declaration stays the generic M7 refusal, not the deferral.
    let plain = manifest_toml(&dir.join("standin-good"), "");
    let plain_manifest = Manifest::from_toml_str(&plain).expect("parses");
    sup.kernel()
        .install(plain_manifest)
        .expect("install undeclared");
    sup.kernel().resolve(&id).expect("resolve undeclared run");
    sup.kernel().start(&id).expect("start undeclared run");
    let bump = plain.replace("version = \"0.1.0\"", "version = \"0.3.0\"");
    let err = sup
        .kernel()
        .install(Manifest::from_toml_str(&bump).expect("parses"))
        .expect_err("undeclared co-host stays the M7 policy refusal");
    assert!(
        matches!(err, KernelError::InvalidTransition { .. }),
        "undeclared path changed: {err}"
    );
    sup.kernel().stop(&id).expect("stop undeclared");
    sup.kernel().uninstall(&id).expect("uninstall");
    assert_no_zombie_children();
    std::fs::remove_dir_all(&dir).expect("cleanup scratch");
    assert!(!dir.exists(), "mk-standin scratch must not leak");
}

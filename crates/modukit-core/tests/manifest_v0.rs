//! Tests: manifest v0 (TOML per D7; field grammar per modules/05 §1-3).
//! Written before the implementation (C0: red observed first).

use std::time::Duration;

use modukit_core::error::ManifestError;
use modukit_core::lifecycle::{PluginState, RestartPolicy};
use modukit_core::manifest::{DepPolicy, Manifest, OnDown, PlacementKind};

const FULL: &str = r#"
id = "com.example.modukit.demo"
version = "0.1.0"
team = "alpha"
start_level = 30
start_timeout = "5s"
stop_timeout = "2s"
min_host = "0.1.0"
capabilities = [ "gpu-share", "com.vendor.note" ]

[entry]
kind = "dl"
artifact = "libexample_plugin.so"

[restart]
policy = "on-failure"
max_retries = 5
backoff = "1s..30s"
cooldown = "5m"

[aliases]
CANBus = "com.acme.hmi.CanBus"

[[requires]]
service = "CANBus"
required = true
policy = "wait(10s)"
on_down = "block"

[[provides]]
service = "com.example.modukit.demo.Nav"
ranking = 100
parallel_ok = false
"#;

#[test]
fn parses_the_documented_field_grammar() {
    let m =
        Manifest::from_toml_str(FULL).unwrap_or_else(|e| panic!("valid manifest rejected: {e}"));
    assert_eq!(m.id.as_str(), "com.example.modukit.demo");
    assert_eq!(m.version, "0.1.0");
    assert_eq!(m.team, "alpha");
    assert_eq!(m.start_level, 30);
    assert_eq!(m.start_timeout, Some(Duration::from_secs(5)));
    assert_eq!(m.stop_timeout, Some(Duration::from_secs(2)));
    assert_eq!(m.entry.kind, PlacementKind::Dl);
    assert_eq!(m.entry.artifact, "libexample_plugin.so");
    assert_eq!(m.capabilities, vec!["gpu-share", "com.vendor.note"]);

    let RestartPolicy::OnFailure {
        max_retries,
        backoff,
        cooldown,
    } = m.restart
    else {
        panic!(
            "restart block must parse into OnFailure, got {:?}",
            m.restart
        );
    };
    assert_eq!(max_retries, 5);
    assert_eq!(backoff.min, Duration::from_secs(1));
    assert_eq!(backoff.max, Duration::from_secs(30));
    assert_eq!(cooldown, Duration::from_secs(300));

    assert_eq!(m.requires.len(), 1);
    let req = &m.requires[0];
    // Aliases resolve at manifest-read time; downstream sees canonical (05 §3).
    assert_eq!(req.service.as_str(), "com.acme.hmi.CanBus");
    assert!(req.required);
    assert_eq!(req.policy, DepPolicy::Wait(Duration::from_secs(10)));
    assert_eq!(req.on_down, Some(OnDown::Block));

    assert_eq!(m.provides.len(), 1);
    assert_eq!(
        m.provides[0].service.as_str(),
        "com.example.modukit.demo.Nav"
    );
    assert_eq!(m.provides[0].ranking, 100);
    assert!(!m.provides[0].parallel_ok);
}

#[test]
fn defaults_apply_to_optional_fields() {
    let toml_src = r#"
id = "com.t.p"
version = "0.0.1"
[entry]
kind = "dl"
artifact = "p.so"
"#;
    let m = Manifest::from_toml_str(toml_src).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(m.team, "default");
    assert_eq!(m.start_level, 0);
    assert_eq!(m.restart, RestartPolicy::None);
    assert!(m.requires.is_empty() && m.provides.is_empty());
}

#[test]
fn rejects_unknown_placement_kind() {
    let toml_src = r#"
id = "com.t.p"
version = "0.0.1"
[entry]
kind = "jar"
artifact = "p.jar"
"#;
    let err = Manifest::from_toml_str(toml_src).expect_err("unknown entry.kind must fail");
    assert!(matches!(err, ManifestError::UnknownPlacement(_)), "{err}");
}

#[test]
fn rejects_malformed_duration_and_policy() {
    let toml_src = r#"
id = "com.t.p"
version = "0.0.1"
start_timeout = "five seconds"
[entry]
kind = "dl"
artifact = "p.so"
"#;
    let err = Manifest::from_toml_str(toml_src).expect_err("bad duration must fail");
    assert!(matches!(err, ManifestError::BadDuration(_)), "{err}");

    let toml_src = r#"
id = "com.t.p"
version = "0.0.1"
[entry]
kind = "dl"
artifact = "p.so"
[[requires]]
service = "com.t.S"
required = true
policy = "poll(1s)"
"#;
    let err =
        Manifest::from_toml_str(toml_src).expect_err("`poll` is not in the policy vocabulary");
    assert!(matches!(err, ManifestError::BadDepPolicy(_)), "{err}");
}

#[test]
fn missing_required_field_is_a_typed_error() {
    let toml_src = r#"
id = "com.t.p"
[entry]
kind = "dl"
artifact = "p.so"
"#;
    let err = Manifest::from_toml_str(toml_src).expect_err("missing version must fail");
    assert!(matches!(err, ManifestError::Toml(_)), "{err}");
}

// The kernel consumes manifest-driven lifecycle below; keep the state names
// referenced here so the test module documents its own dialect only.
#[test]
fn plugin_state_display_uses_doc_names() {
    assert_eq!(PluginState::Resolving.to_string(), "RESOLVING");
    assert_eq!(PluginState::Uninstalled.to_string(), "UNINSTALLED");
}

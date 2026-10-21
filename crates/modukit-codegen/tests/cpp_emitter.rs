//! Stage-4 M1 lane #3: C++ emitter golden tests (ruling C++-3, both faces).
//!
//! Face A (plugin): `generate` also emits a `<module>.hpp` plugin skeleton
//! beside the existing Rust/C outputs, wired to the stock protoc C++
//! generated headers (`--cpp_out` output is consumed AS-IS per C++-1 -
//! this emitter writes includes/dispatch, never redefines messages).
//! Face B (embed): `emit_embed_hpp` turns the checked-in contract original
//! `contracts/modukit_embed.h` into a C++ RAII wrapper header.
//!
//! Goldens live in `testdata/golden/` beside the M4 corpus and follow the
//! same byte-compare + regen discipline; the smoke legs compile the emitted
//! text with the pinned pixi-env compiler and skip cleanly without one.

use std::path::{Path, PathBuf};

use modukit_codegen::{emit_embed_hpp, generate};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crate lives under <repo>/crates/")
        .to_path_buf()
}

fn ledger_fdsets() -> Vec<PathBuf> {
    let contracts = repo_root().join("contracts");
    vec![
        contracts.join("org.modukit.runtime.proto.fdset"),
        contracts.join("com.modukit.demo.proto.fdset"),
    ]
}

#[test]
fn cpp_plugin_face_matches_golden() {
    let root = repo_root();
    let out_dir =
        std::env::temp_dir().join(format!("modukit-codegen-cpp-golden-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out_dir);

    let written = generate(&ledger_fdsets(), &out_dir).expect("generate succeeds");
    let name = "com_modukit_demo.hpp";
    let produced = written
        .iter()
        .find(|p| p.file_name().and_then(|n| n.to_str()) == Some(name))
        .unwrap_or_else(|| panic!("{name} not among generated files: {written:?}"));

    let golden = root
        .join("crates/modukit-codegen/testdata/golden")
        .join(name);
    let produced_bytes = std::fs::read(produced).expect("produced .hpp readable");
    let golden_bytes = std::fs::read(&golden).unwrap_or_else(|e| {
        panic!(
            "golden {} missing (pixi run regen-golden): {e}",
            golden.display()
        )
    });
    assert_eq!(
        produced_bytes, golden_bytes,
        "golden drift in {name} (rerun `pixi run regen-golden` and review the diff)"
    );
    let _ = std::fs::remove_dir_all(&out_dir);
}

#[test]
fn embed_wrapper_matches_golden() {
    let root = repo_root();
    let header = root.join("contracts/modukit_embed.h");
    let text = std::fs::read_to_string(&header).expect("checked-in embed header present");
    let produced = emit_embed_hpp(&text).expect("embed wrapper emits");

    let golden = root
        .join("crates/modukit-codegen/testdata/golden")
        .join("modukit_embed.hpp");
    let golden_text = std::fs::read_to_string(&golden).unwrap_or_else(|e| {
        panic!(
            "golden {} missing (pixi run regen-golden): {e}",
            golden.display()
        )
    });
    assert_eq!(
        produced, golden_text,
        "embed wrapper golden drifted (rerun `pixi run regen-golden` and review the diff)"
    );
}

#[test]
fn embed_wrapper_tracks_the_real_header_verbs() {
    // Structural pin: the wrapper must carry one method per header verb, in
    // declaration order (16 verbs at HostEmbedV1; D83 tail-add keeps order).
    let root = repo_root();
    let header = std::fs::read_to_string(root.join("contracts/modukit_embed.h")).unwrap();
    let produced = emit_embed_hpp(&header).unwrap();
    let header_verbs: Vec<&str> = header
        .lines()
        .filter_map(|l| l.strip_prefix("int32_t modukit_"))
        .map(|l| l.split('(').next().expect("verb name before ("))
        .collect();
    assert_eq!(header_verbs.len(), 24, "HostEmbedV1 verb count moved");
    for verb in header_verbs {
        assert!(
            produced.contains(&format!("modukit_{verb}(")),
            "wrapper lacks a method for verb modukit_{verb}"
        );
    }
    // The version probe rides the wrapper too (handshake discipline).
    assert!(
        produced.contains("modukit_embed_abi_version"),
        "wrapper lacks the version probe"
    );
}

#[test]
fn emitted_cpp_passes_fsyntax_only() {
    // Smoke: the emitted plugin face + the emitted embed wrapper must be
    // compilable C++ under the pinned pixi-env compiler. Skips cleanly when
    // no compiler is on PATH (the pinned env has one; lane #2 proved it).
    let root = repo_root();
    let out_dir =
        std::env::temp_dir().join(format!("modukit-codegen-cpp-smoke-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out_dir);
    std::fs::create_dir_all(&out_dir).unwrap();

    // The plugin face includes the stock protoc C++ header for the same
    // package; for the smoke we pre-generate that header with the pinned
    // protoc into the same temp dir.
    let contracts = root.join("contracts");
    let pb = protoc_cpp_out(&contracts, &out_dir);
    if !pb {
        eprintln!("SKIP: protoc (pinned pixi env) unavailable; compile smoke not run");
        let _ = std::fs::remove_dir_all(&out_dir);
        return;
    }

    let written = generate(&ledger_fdsets(), &out_dir).expect("generate succeeds");
    let face = written
        .iter()
        .find(|p| p.extension().and_then(|e| e.to_str()) == Some("hpp"))
        .expect("plugin-face .hpp emitted");
    let embed_text =
        std::fs::read_to_string(contracts.join("modukit_embed.h")).expect("embed header");
    // The wrapper's quoted include finds the contract header beside it -
    // mirror that layout in the temp dir (a real consumer ships both).
    std::fs::copy(
        contracts.join("modukit_embed.h"),
        out_dir.join("modukit_embed.h"),
    )
    .expect("copy embed header into smoke dir");
    let embed_hpp = out_dir.join("modukit_embed.hpp");
    std::fs::write(&embed_hpp, emit_embed_hpp(&embed_text).unwrap()).unwrap();

    let cxx = find_cxx();
    let Some(cxx) = cxx else {
        eprintln!("SKIP: no C++ compiler on PATH; compile smoke not run");
        let _ = std::fs::remove_dir_all(&out_dir);
        return;
    };
    let proto_include = root.join(".pixi/envs/default/include");
    for src in [face.as_path(), embed_hpp.as_path()] {
        let status = std::process::Command::new(&cxx)
            .arg("-std=c++17")
            .arg("-fsyntax-only")
            .arg("-I")
            .arg(&proto_include)
            .arg("-I")
            .arg(&out_dir)
            .arg(src)
            .status()
            .expect("spawn compiler");
        assert!(
            status.success(),
            "emitted {} failed {} -fsyntax-only",
            src.display(),
            cxx.display()
        );
    }
    let _ = std::fs::remove_dir_all(&out_dir);
}

/// Locate a C++ compiler: prefer the pinned pixi env (lane #2 evidence),
/// fall back to PATH order g++ / c++ / clang++.
fn find_cxx() -> Option<PathBuf> {
    let root = repo_root();
    let pinned = root.join(".pixi/envs/default/bin/x86_64-conda-linux-gnu-g++");
    if pinned.is_file() {
        return Some(pinned);
    }
    ["g++", "c++", "clang++"]
        .into_iter()
        .find_map(which_on_path)
}

fn which_on_path(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join(name))
        .find(|candidate| candidate.is_file())
}

/// Run the pinned-env protoc (or PATH protoc) over every contracts/*.proto
/// with --cpp_out into `out_dir`. Returns false when protoc is unavailable
/// (the smoke leg then SKIPs; golden legs never depend on protoc).
fn protoc_cpp_out(contracts: &Path, out_dir: &Path) -> bool {
    let root = repo_root();
    let pinned = root.join(".pixi/envs/default/bin/protoc");
    let protoc = if pinned.is_file() {
        pinned
    } else {
        match which_on_path("protoc") {
            Some(p) => p,
            None => return false,
        }
    };
    let protos = [
        "modukit.proto",
        "org.modukit.runtime.proto",
        "com.modukit.demo.proto",
    ];
    let mut cmd = std::process::Command::new(protoc);
    cmd.arg(format!("--cpp_out={}", out_dir.display()))
        .arg(format!("--proto_path={}", contracts.display()));
    for p in protos {
        cmd.arg(p);
    }
    cmd.output().map(|o| o.status.success()).unwrap_or(false)
}

//! `protoc-gen-modukit` - the M4 v0 CLI entry.
//!
//! v0 seam (reported to the orchestrator): the binary reads FileDescriptorSet
//! binaries directly (`@path.fdset`) instead of a CodeGeneratorRequest on
//! stdin, and takes `--output_dir` like protoc's plugin convention - so it is
//! NOT yet registered as a real `--plugin` for protoc. The fdset bytes on the
//! wire are the same FileDescriptorSet protoc sends a plugin; switching the
//! seam is a read-path change, not an emitter change.
//!
//! Stage-4 M1 (ruling C++-3 face B): `--embed_header <modukit_embed.h>`
//! additionally writes the C++ embed-face wrapper (`<stem>.hpp`) into the
//! output dir - the checked-in header is the contract original, the wrapper
//! is generated FROM it.

use std::path::PathBuf;
use std::process::ExitCode;

use modukit_codegen::{CodegenError, emit_embed_hpp, emit_embed_py, emit_plugin_hpp, generate};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(files) => {
            for f in files {
                println!("wrote {}", f.display());
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("protoc-gen-modukit: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[String]) -> Result<Vec<PathBuf>, CodegenError> {
    let mut fdsets: Vec<PathBuf> = Vec::new();
    let mut output_dir: Option<PathBuf> = None;
    let mut embed_header: Option<PathBuf> = None;
    let mut plugin_header: Option<PathBuf> = None;
    let mut py_out: Option<PathBuf> = None;
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        if arg == "--output_dir" {
            let value = it
                .next()
                .ok_or_else(|| usage("--output_dir needs a value"))?;
            output_dir = Some(PathBuf::from(value));
        } else if let Some(value) = arg.strip_prefix("--output_dir=") {
            output_dir = Some(PathBuf::from(value));
        } else if arg == "--embed_header" {
            let value = it
                .next()
                .ok_or_else(|| usage("--embed_header needs a path"))?;
            embed_header = Some(PathBuf::from(value));
        } else if let Some(value) = arg.strip_prefix("--embed_header=") {
            embed_header = Some(PathBuf::from(value));
        } else if arg == "--plugin_header" {
            let value = it
                .next()
                .ok_or_else(|| usage("--plugin_header needs a path"))?;
            plugin_header = Some(PathBuf::from(value));
        } else if let Some(value) = arg.strip_prefix("--plugin_header=") {
            plugin_header = Some(PathBuf::from(value));
        } else if arg == "--py_out" {
            let value = it.next().ok_or_else(|| usage("--py_out needs a path"))?;
            py_out = Some(PathBuf::from(value));
        } else if let Some(value) = arg.strip_prefix("--py_out=") {
            py_out = Some(PathBuf::from(value));
        } else if let Some(path) = arg.strip_prefix('@') {
            // protoc-style request file: a FileDescriptorSet binary on disk.
            fdsets.push(PathBuf::from(path));
        } else if arg.starts_with("--") {
            return Err(usage(format!("unknown flag {arg}")));
        } else {
            // Bare paths are accepted too (pixi tasks use @file for clarity).
            fdsets.push(PathBuf::from(arg));
        }
    }
    let output_dir = output_dir.ok_or_else(|| usage("missing --output_dir <dir>"))?;
    if fdsets.is_empty() {
        return Err(usage("no @<file>.fdset inputs given"));
    }
    let mut files = generate(&fdsets, &output_dir)?;
    if let Some(header_path) = &embed_header {
        // The header text parse is pinned to the Rust mirror by the parity
        // gate, so the emitted wrapper trusts the declaration shape it sees.
        let text = std::fs::read_to_string(header_path)?;
        let stem = header_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| usage("--embed_header needs a file path"))?;
        let path = output_dir.join(format!("{stem}.hpp"));
        std::fs::write(&path, emit_embed_hpp(&text)?)?;
        files.push(path);
    }
    if let Some(header_path) = plugin_header {
        // Same trust discipline as the embed wrapper: the parity gate pins
        // modukit_plugin.h to abi::HostAbiV2, so the emitted host view
        // forwards exactly the slot typedefs it parsed (Stage-5 N1).
        let text = std::fs::read_to_string(&header_path)?;
        let stem = header_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| usage("--plugin_header needs a file path"))?;
        let path = output_dir.join(format!("{stem}.hpp"));
        std::fs::write(&path, emit_plugin_hpp(&text)?)?;
        files.push(path);
    }
    if let Some(py_dir) = py_out {
        // Wave-2 Python volume M1 (D100 R2-A): the floor rides the same
        // header text at this door - emission beside --embed_header, never
        // through generate()'s fdset corpus (golden.rs:34-37 seam note).
        let header_path = embed_header
            .as_ref()
            .ok_or_else(|| usage("--py_out needs --embed_header"))?;
        let text = std::fs::read_to_string(header_path)?;
        let stem = header_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| usage("--embed_header needs a file path"))?;
        std::fs::create_dir_all(&py_dir)?;
        let path = py_dir.join(format!("{stem}.py"));
        std::fs::write(&path, emit_embed_py(&text)?)?;
        files.push(path);
    }
    Ok(files)
}

fn usage(msg: impl Into<String>) -> CodegenError {
    let msg = msg.into();
    CodegenError::Usage(format!(
        "{msg}\n\
         usage: protoc-gen-modukit @<descriptor_set.fdset> [more @fdsets...] \
         --output_dir <dir> [--embed_header <modukit_embed.h>] \
         [--plugin_header <modukit_plugin.h>] [--py_out <dir>]\
         (v0 seam: reads FileDescriptorSet binaries directly, not a \
         CodeGeneratorRequest on stdin - see src/main.rs)"
    ))
}

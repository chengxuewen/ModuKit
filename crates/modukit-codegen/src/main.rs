//! `protoc-gen-modukit` - the M4 v0 CLI entry.
//!
//! v0 seam (reported to the orchestrator): the binary reads FileDescriptorSet
//! binaries directly (`@path.fdset`) instead of a CodeGeneratorRequest on
//! stdin, and takes `--output_dir` like protoc's plugin convention - so it is
//! NOT yet registered as a real `--plugin` for protoc. The fdset bytes on the
//! wire are the same FileDescriptorSet protoc sends a plugin; switching the
//! seam is a read-path change, not an emitter change.

use std::path::PathBuf;
use std::process::ExitCode;

use modukit_codegen::{CodegenError, generate};

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
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        if arg == "--output_dir" {
            let value = it
                .next()
                .ok_or_else(|| usage("--output_dir needs a value"))?;
            output_dir = Some(PathBuf::from(value));
        } else if let Some(value) = arg.strip_prefix("--output_dir=") {
            output_dir = Some(PathBuf::from(value));
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
    generate(&fdsets, &output_dir)
}

fn usage(msg: impl Into<String>) -> CodegenError {
    let msg = msg.into();
    CodegenError::Usage(format!(
        "{msg}\n\
         usage: protoc-gen-modukit @<descriptor_set.fdset> [more @fdsets...] \
         --output_dir <dir>\n\
         (v0 seam: reads FileDescriptorSet binaries directly, not a \
         CodeGeneratorRequest on stdin - see src/main.rs)"
    ))
}

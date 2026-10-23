//! JS embed-face bridge + facade emitter (wave-2 JS volume M1, D102 R2-A).
//!
//! Input: the contract ORIGINAL `contracts/modukit_embed.h`, parsed with the
//! same language-neutral text parsers the C++/Python faces use
//! ([`crate::embed::parse_enums`] / [`crate::embed::parse_verbs`]). The header
//! stays the single authority; a second parser or a hand-maintained JS source
//! would be exactly the parallel-surface drift D63 forbids.
//!
//! Output, TWO artifacts (D102 R2-A):
//! - `modukit_embed_bridge.rs`: a self-contained napi module. It dlopens the
//!   kernel C door ONCE from `MODUKIT_CORE_SO` (linking modukit-core as a Rust
//!   dependency is forbidden here), holds one typed symbol per rung-0 verb,
//!   and emits one `#[napi]` forwarder per verb - every body goes through the
//!   table only. Plus the enum mirrors as napi-exposed consts and the
//!   i32 -> named-error mapping table (data, semantics-free).
//! - `modukit_embed.js`: the CommonJS facade - loads the `.node` (path via
//!   env/arg), re-exports the verb functions 1:1 with doc comments citing the
//!   header lines, mirrors the enums as frozen objects, and the error table
//!   with codes as data. No async, no sugar beyond names (D6).
//!
//! M1 scope (D102 R1-A, two milestones): rung-0 verbs only - the filter is
//! [`rung0_verbs`]; the M2 sweep widens that one site to both rungs.
//!
//! Integer crossing note: napi 2.16 implements no `u64`/`usize` conversion
//! (ambiguous against JS numbers), so every 64-bit crossing rides `i64` both
//! ways - lossless for this contract's value domain (sizes, ring seqs; JS
//! numbers round above 2^53, noted in the facade banner).
//!
//! Floor discipline: generated code only, no semantics. Determinism: no
//! timestamps, no iteration-order dependence; the goldens byte-compare and
//! the regen-diff leg own drift. Target: Node 22 (modules ABI 127) via the
//! cached napi stack - versions pinned by the M0 probe (D102 R2-A).

use std::fmt::Write as _;

use crate::embed::{CVerb, enum_class_name, parse_enums, parse_verbs};
use crate::error::CodegenError;

/// M1 rung filter (D102 R1-A): the `modukit_sup_*` exe-rung verbs are
/// excluded here; the M2 sweep widens this one site to both rungs.
fn rung0_verbs(header_text: &str) -> Vec<CVerb> {
    parse_verbs(header_text)
        .into_iter()
        .filter(|v| !v.name.starts_with("modukit_sup_"))
        .collect()
}

/// One normalized parameter declaration: C type + trailing identifier;
/// Owned so the pair outlives the normalization buffer.
struct Param {
    ty: String,
    name: String,
}

fn param_of(decl: &str) -> Param {
    let norm = decl.split_whitespace().collect::<Vec<_>>().join(" ");
    let name_len = norm
        .chars()
        .rev()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .count();
    Param {
        ty: norm[..norm.len() - name_len].trim_end().to_string(),
        name: norm[norm.len() - name_len..].to_string(),
    }
}

/// Classification of one verb parameter for the forwarder plan. Byte-buffer
/// `(ptr, len)` input pairs and `(ptr, cap[, len-out])` output triples are
/// merged into single JS Buffer arguments/properties; the C pair stays
/// positionally intact in the extern type.
enum Role {
    CStrIn {
        name: String,
    },
    CStrOut {
        name: String,
    },
    BufIn {
        name: String,
    },
    BufOut {
        name: String,
        cap: String,
        len_out: Option<String>,
    },
    ScalarIn {
        name: String,
        ty: String,
    },
    ScalarOut {
        name: String,
        ty: String,
    },
}

fn plan_roles(params: &[String]) -> Result<Vec<Role>, CodegenError> {
    let ps: Vec<Param> = params.iter().map(|d| param_of(d)).collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < ps.len() {
        let p = &ps[i];
        match p.ty.as_str() {
            "const char *" => out.push(Role::CStrIn {
                name: p.name.clone(),
            }),
            "const char **" | "char **" => out.push(Role::CStrOut {
                name: p.name.clone(),
            }),
            "const uint8_t *" => {
                out.push(Role::BufIn {
                    name: p.name.clone(),
                });
                // The paired size_t length rides the Buffer's own length;
                // consume it so it never becomes a JS argument.
                if i + 1 < ps.len() && ps[i + 1].ty == "size_t" {
                    i += 1;
                }
            }
            "uint8_t *" => {
                let cap = ps
                    .get(i + 1)
                    .filter(|n| n.ty == "size_t")
                    .map(|n| n.name.to_string())
                    .ok_or_else(|| {
                        CodegenError::EmbedHeader(format!(
                            "buffer-out param `{}` has no size_t cap",
                            p.name
                        ))
                    })?;
                let (jump, len_out) = match ps.get(i + 2) {
                    Some(n) if n.ty == "size_t *" => (3, Some(n.name.to_string())),
                    _ => (2, None),
                };
                out.push(Role::BufOut {
                    name: p.name.clone(),
                    cap,
                    len_out,
                });
                i += jump;
                continue;
            }
            "int32_t *" => out.push(Role::ScalarOut {
                name: p.name.clone(),
                ty: "i32".into(),
            }),
            "int64_t *" => out.push(Role::ScalarOut {
                name: p.name.clone(),
                ty: "i64".into(),
            }),
            "uint64_t *" => out.push(Role::ScalarOut {
                name: p.name.clone(),
                ty: "u64".into(),
            }),
            "size_t *" => out.push(Role::ScalarOut {
                name: p.name.clone(),
                ty: "usize".into(),
            }),
            "int32_t" | "int64_t" | "uint64_t" | "uint32_t" | "size_t" => {
                let ty = match p.ty.as_str() {
                    "int32_t" => "i32",
                    "int64_t" => "i64",
                    "uint64_t" => "u64",
                    "uint32_t" => "u32",
                    _ => "usize",
                };
                out.push(Role::ScalarIn {
                    name: p.name.clone(),
                    ty: ty.into(),
                });
            }
            other => {
                return Err(CodegenError::EmbedHeader(format!(
                    "parameter type `{other}` has no napi-bridge mapping"
                )));
            }
        }
        i += 1;
    }
    Ok(out)
}

/// The extern "C" argument list for the typed symbol: always the full C
/// positional shape (merged pairs included), mirroring the header.
fn extern_args(roles: &[Role]) -> String {
    let mut parts: Vec<String> = Vec::new();
    for r in roles {
        match r {
            Role::CStrIn { .. } => parts.push("*const c_char".into()),
            Role::CStrOut { .. } => parts.push("*mut *const c_char".into()),
            Role::BufIn { .. } => parts.extend(["*const u8".into(), "usize".into()]),
            Role::BufOut { len_out, .. } => {
                parts.extend(["*mut u8".into(), "usize".into()]);
                if len_out.is_some() {
                    parts.push("*mut usize".into());
                }
            }
            Role::ScalarIn { ty, .. } => parts.push(ty.clone()),
            Role::ScalarOut { ty, .. } => parts.push(format!("*mut {ty}")),
        }
    }
    parts.join(", ")
}

fn has_outputs(roles: &[Role]) -> bool {
    roles.iter().any(|r| {
        matches!(
            r,
            Role::CStrOut { .. } | Role::BufOut { .. } | Role::ScalarOut { .. }
        )
    })
}

/// The JS-facing argument list of one forwarder (out-params are not
/// arguments; they ride the returned object's named properties).
fn forwarder_args(roles: &[Role]) -> String {
    let mut parts = Vec::new();
    if has_outputs(roles) {
        parts.push("env: Env".to_string());
    }
    for r in roles {
        match r {
            Role::CStrIn { name } => parts.push(format!("{name}: String")),
            Role::BufIn { name } => parts.push(format!("{name}: Buffer")),
            Role::BufOut { cap, .. } => parts.push(format!("{cap}: i64")),
            // u64/u32/usize arrive as i64: napi 2.16 has no unsigned 64-bit.
            Role::ScalarIn { name, ty } => match ty.as_str() {
                "i32" | "i64" => parts.push(format!("{name}: {ty}")),
                _ => parts.push(format!("{name}: i64")),
            },
            Role::CStrOut { .. } | Role::ScalarOut { .. } => {}
        }
    }
    parts.join(", ")
}

/// `#define MODUKIT_EMBED_ABI_VERSION 1u` -> "1" (leading digits of the value).
fn abi_define_value(header_text: &str) -> Result<String, CodegenError> {
    for line in header_text.lines() {
        if let Some(rest) = line
            .trim()
            .strip_prefix("#define MODUKIT_EMBED_ABI_VERSION")
        {
            let digits: String = rest
                .trim()
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            if !digits.is_empty() {
                return Ok(digits);
            }
        }
    }
    Err(CodegenError::EmbedHeader(
        "no `#define MODUKIT_EMBED_ABI_VERSION <n>` line found".to_string(),
    ))
}

/// First declaration line of a verb (1-based), for facade doc citations.
fn verb_line(header_text: &str, name: &str) -> usize {
    let pat = format!("int32_t {name}(");
    header_text
        .lines()
        .position(|l| l.trim().starts_with(&pat))
        .map(|n| n + 1)
        .unwrap_or(0)
}

fn kernel_code_enum(header_text: &str) -> Result<crate::embed::CEnum, CodegenError> {
    parse_enums(header_text)
        .into_iter()
        .find(|e| e.tag == "modukit_kernel_code")
        .ok_or_else(|| CodegenError::EmbedHeader("no `modukit_kernel_code` enum".to_string()))
}

/// Emit the napi bridge face (`modukit_embed_bridge.rs`) from the embed
/// contract header text.
///
/// # Errors
///
/// [`CodegenError::EmbedHeader`] on contract drift the parsers cannot see
/// through (no enums/verbs, unknown parameter type, missing ABI define).
pub fn emit_embed_js_bridge(header_text: &str) -> Result<String, CodegenError> {
    let enums = parse_enums(header_text);
    if enums.is_empty() {
        return Err(CodegenError::EmbedHeader(
            "no `typedef enum modukit_*` block found".to_string(),
        ));
    }
    let verbs = rung0_verbs(header_text);
    if verbs.is_empty() {
        return Err(CodegenError::EmbedHeader(
            "no rung-0 `int32_t modukit_*(` verb declaration found".to_string(),
        ));
    }
    let abi = abi_define_value(header_text)?;
    let kernel_code = kernel_code_enum(header_text)?;

    let mut out = String::new();
    out.push_str(
        r#"//! Generated by modukit-codegen (wave-2 JS volume M1, D102 R2-A) from
//! modukit_embed.h. DO NOT EDIT - the contract ORIGINAL is the C header this
//! module mirrors; skins are generated FROM it (D63: hand-writing a
//! replacement of this bridge is forbidden).
//!
//! Regen: cargo run -q -p modukit-codegen --bin protoc-gen-modukit --
//!   @contracts/org.modukit.runtime.proto.fdset @contracts/com.modukit.demo.proto.fdset
//!   --embed_header contracts/modukit_embed.h --js_out <dir> --output_dir <dir>
//!
//! JS EMBED BRIDGE ("#,
    );
    let _ = write!(out, "{count}", count = verbs.len(),);
    out.push_str(
        r#" rung-0 verbs, M1 scope - the modukit_sup_* exe
//! rung joins at M2): a napi addon embedding the kernel through the C door
//! ONLY. The kernel .so path is read from MODUKIT_CORE_SO at init and
//! dlopened at exactly ONE site ([`load_kernel`]); every symbol lands once
//! in a typed table; every forwarder body calls through the table. Linking
//! modukit-core as a Rust dependency is forbidden here (D102 R2-A: the C
//! door is the only cross-language exit).
//! Integer crossing: every 64-bit value rides i64 both ways (napi 2.16
//! exports no u64/usize conversion). A zero buffer cap calls the header's
//! documented NULL size-query mode (out_buf NULL, size reported).
//! Node pin: modules ABI 127 / node 22.x (M0 probe; upgrade re-arms R2).
//!
//! Shape discipline: the generated text is emitted verbatim and opts out of
//! rustfmt file-wide - running a formatter over it would move the golden on
//! every formatter release (PIT-88 drift class); the parity of the CONTENTS
//! is what the pins own.
#![cfg_attr(rustfmt, rustfmt::skip)]
use libloading::{Library, Symbol};
use napi::bindgen_prelude::Buffer;
use napi::{Env, Error, JsObject, Result, Status};
use napi_derive::napi;
use std::env;
use std::ffi::{c_char, CStr, CString};
use std::ptr;
use std::sync::OnceLock;

// Enum mirrors as napi-exposed consts - names and values verbatim from
// `contracts/modukit_embed.h` (SCREAMING_SNAKE names ride JS-side as-is).
/// mirror of the `MODUKIT_EMBED_ABI_VERSION` define (the handshake const)
#[napi]
pub const MODUKIT_EMBED_ABI_VERSION: i32 = "#,
    );
    let _ = write!(out, "{abi}");
    out.push_str(";\n");
    for e in &enums {
        let _ = write!(out, "\n// mirror of `{}` (all values below)\n", e.tag);
        for (name, number) in &e.values {
            let _ = writeln!(out, "#[napi]\npub const {name}: i32 = {number};");
        }
    }

    // i32 -> named error mapping (data, semantics-free - the D51 dictionary).
    let _ = write!(
        out,
        r#"
/// i32 -> named-error mapping: the D51 kernel dictionary mirror, raw
/// code number -> name (verbs return -(code) on failure; the facade
/// carries the JS-live table - this is the checked-in bridge mirror.
pub const EMBED_ERRORS: [(i32, &str); "#
    );
    let _ = write!(out, "{}", kernel_code.values.len());
    out.push_str("] = [\n");
    for (name, number) in &kernel_code.values {
        let _ = writeln!(out, "    ({number}, \"{name}\"),");
    }
    out.push_str("];\n");

    // The typed symbol table: exactly one entry per rung-0 verb.
    out.push_str(
        r#"
/// One typed symbol per rung-0 verb, signatures mirroring the C
/// declarations (contracts/modukit_embed.h). Filled once by
/// [`load_kernel`] - the sole dlopen site.
pub struct EmbedSymbols {
"#,
    );
    for v in &verbs {
        let bare = v.name.strip_prefix("modukit_").unwrap_or(&v.name);
        let roles = plan_roles(&v.params)?;
        let _ = writeln!(
            out,
            "    pub {bare}: extern \"C\" fn({}) -> i32,",
            extern_args(&roles)
        );
    }
    out.push_str(
        r#"}

pub struct Kernel {
    pub sym: EmbedSymbols,
}

/// THE one dlopen site: `MODUKIT_CORE_SO` -> libloading::Library, every
/// rung-0 symbol resolved once into the typed table. The Library handle is
/// leaked on purpose: a napi addon may outlive `main`, and dlclosing the
/// kernel out from under the table would be unsound.
///
/// # Safety
/// Every `Symbol::get` below reads a C ABI function pointer exported by the
/// kernel cdylib with exactly the signature `contracts/modukit_embed.h`
/// declares (the parity gate `scripts/check-embed-parity.sh` pins the Rust
/// mirror to that header). A kernel exporting a different shape is contract
/// drift for a D83 hearing, not something this table tolerates.
fn load_kernel() -> Result<Kernel> {
    let path = env::var("MODUKIT_CORE_SO").map_err(|_| Error::new(
        Status::GenericFailure,
        "MODUKIT_CORE_SO is not set: the bridge dlopens the kernel C door from that path",
    ))?;
    let lib: &'static Library = Box::leak(
        Box::new(unsafe { Library::new(&path) }
            .map_err(|e| Error::new(Status::GenericFailure, format!("dlopen {path}: {e}")))?),
    );
    // No macro machinery in the generated bridge (plan R5): one explicit
    // get block per verb, mechanically emitted from the header.
    let sym = EmbedSymbols {
"#,
    );
    for v in &verbs {
        let bare = v.name.strip_prefix("modukit_").unwrap_or(&v.name);
        let roles = plan_roles(&v.params)?;
        let _ = write!(
            out,
            "        {bare}: unsafe {{\n\
            \x20           let s: Symbol<'_, extern \"C\" fn({args}) -> i32> = lib\n\
            \x20               .get(b\"{vn}\\0\")\n\
            \x20               .map_err(|e| Error::new(Status::GenericFailure, format!(\"symbol {vn}: {{e}}\")))?;\n\
            \x20           *s\n\
            \x20       }},\n",
            args = extern_args(&roles),
            vn = v.name,
        );
    }
    out.push_str(
        r#"    };
    Ok(Kernel { sym })
}

/// Kernel handle, initialized on the first verb call (the ambient kernel
/// needs no init verb - the same first-call init the C++/Python faces use).
/// OnceLock keeps one table authoritative.
fn kernel() -> Result<&'static Kernel> {
    static KERNEL: OnceLock<Kernel> = OnceLock::new();
    if KERNEL.get().is_none() {
        let k = load_kernel()?;
        let _ = KERNEL.set(k);
    }
    Ok(KERNEL.get().expect("load_kernel ran above or in the racing winner"))
}
"#,
    );

    // One forwarder per verb.
    for v in &verbs {
        let bare = v.name.strip_prefix("modukit_").unwrap_or(&v.name);
        let roles = plan_roles(&v.params)?;
        let ret = if has_outputs(&roles) {
            "Result<JsObject>"
        } else {
            "Result<i32>"
        };
        let _ = write!(
            out,
            "\n/// forwarder for `{vn}` (contracts/modukit_embed.h:{line}) - calls the\n\
             /// typed table ONLY (the C door). The raw i32 channel rides `raw`; C\n\
             /// out-params ride named properties as data.\n\
             #[napi(js_name = \"{vn}\")]\n\
             pub fn {vn}({fargs}) -> {ret} {{\n\
             \x20   let k = kernel()?;\n",
            vn = v.name,
            line = verb_line(header_text, &v.name),
            fargs = forwarder_args(&roles),
        );

        // Marshalling + positional call args, one pass over the roles.
        let mut call_args: Vec<String> = Vec::new();
        let mut marshal = String::new();
        for r in &roles {
            match r {
                Role::CStrIn { name } => {
                    let _ = writeln!(
                        marshal,
                        "    let c_{name} = CString::new({name}).map_err(|e| Error::new(Status::InvalidArg, format!(\"{}: {name} has an interior NUL ({{e}})\")))?;",
                        v.name,
                    );
                    call_args.push(format!("c_{name}.as_ptr()"));
                }
                Role::BufIn { name } => {
                    call_args.push(format!("{name}.as_ptr()"));
                    call_args.push(format!("{name}.len()"));
                }
                Role::BufOut { name, cap, len_out } => {
                    let _ = writeln!(
                        marshal,
                        "    let {cap}: usize = usize::try_from({cap}).map_err(|e| Error::new(Status::InvalidArg, format!(\"{}: {cap} is not a size_t ({{e}})\")))?;\n\
                         \x20   let mut {name}: Vec<u8> = vec![0u8; {cap}];\n\
                         \x20   // cap 0 = the header's NULL size-query mode (out_buf NULL).\n\
                         \x20   let {name}_ptr: *mut u8 = if {cap} == 0 {{ ptr::null_mut() }} else {{ {name}.as_mut_ptr() }};",
                        v.name,
                    );
                    call_args.push(format!("{name}_ptr"));
                    call_args.push(cap.clone());
                    if let Some(lo) = len_out {
                        let _ = writeln!(marshal, "    let mut {name}_{lo}: usize = 0;");
                        call_args.push(format!("&mut {name}_{lo}"));
                    }
                }
                Role::ScalarIn { name, ty } => match ty.as_str() {
                    "i32" | "i64" => call_args.push(name.clone()),
                    "u64" => call_args.push(format!("{name} as u64")),
                    _ => call_args.push(format!("{name} as usize")),
                },
                Role::CStrOut { name } => {
                    let _ = writeln!(marshal, "    let mut {name}: *const c_char = ptr::null();");
                    call_args.push(format!("&mut {name}"));
                }
                Role::ScalarOut { name, ty } => {
                    let _ = writeln!(marshal, "    let mut {name}: {ty} = 0;");
                    call_args.push(format!("&mut {name}"));
                }
            }
        }
        out.push_str(&marshal);
        if !marshal.is_empty() {
            out.push('\n');
        }
        let _ = writeln!(
            out,
            "    let raw = (k.sym.{bare})({args});",
            args = call_args.join(", "),
        );

        if !has_outputs(&roles) {
            out.push_str("    Ok(raw)\n}\n");
            continue;
        }
        out.push_str("    let mut o = env.create_object()?;\n    o.set(\"raw\", raw)?;\n");
        for r in &roles {
            match r {
                Role::CStrOut { name } => {
                    let _ = write!(
                        out,
                        "    // SAFETY: the kernel returns kernel-owned NUL-terminated static\n\
                         \x20   // bytes (the install note in contracts/modukit_embed.h); copied out\n\
                         \x20   // through napi, never freed here.\n\
                         \x20   let {name}: String = if {name}.is_null() {{\n\
                         \x20       String::new()\n\
                         \x20   }} else {{\n\
                         \x20       unsafe {{ CStr::from_ptr({name}) }}.to_string_lossy().into_owned()\n\
                         \x20   }};\n\
                         \x20   o.set(\"{name}\", {name})?;\n"
                    );
                }
                Role::BufOut { name, len_out, .. } => {
                    let exact = match len_out {
                        Some(lo) => format!("({name}_{lo})"),
                        None => "(raw as usize)".to_string(),
                    };
                    let _ = write!(
                        out,
                        "    let {name}: Vec<u8> = if raw >= 0 {{\n\
                         \x20       {name}.truncate({exact}.min({name}.len()));\n\
                         \x20       {name}\n\
                         \x20   }} else {{\n\
                         \x20       Vec::new()\n\
                         \x20   }};\n\
                         \x20   o.set(\"{name}\", Buffer::from({name}))?;\n"
                    );
                    if let Some(lo) = len_out {
                        let _ = writeln!(out, "    o.set(\"{lo}\", {name}_{lo} as i64)?;");
                    }
                }
                Role::ScalarOut { name, ty } => {
                    // i64 is the JS-crossing type; only wider-unsigned or
                    // pointer-width outs need the cast (no-op casts would be
                    // a clippy::unnecessary_cast on -D warnings).
                    let cast = match ty.as_str() {
                        "i32" | "i64" => "",
                        _ => " as i64",
                    };
                    let _ = writeln!(out, "    o.set(\"{name}\", {name}{cast})?;");
                }
                _ => {}
            }
        }
        out.push_str("    Ok(o)\n}\n");
    }
    Ok(out)
}

/// Emit the JS facade (`modukit_embed.js`) from the embed contract header.
///
/// # Errors
///
/// As [`emit_embed_js_bridge`].
pub fn emit_embed_js_facade(header_text: &str) -> Result<String, CodegenError> {
    let enums = parse_enums(header_text);
    if enums.is_empty() {
        return Err(CodegenError::EmbedHeader(
            "no `typedef enum modukit_*` block found".to_string(),
        ));
    }
    let verbs = rung0_verbs(header_text);
    if verbs.is_empty() {
        return Err(CodegenError::EmbedHeader(
            "no rung-0 `int32_t modukit_*(` verb declaration found".to_string(),
        ));
    }
    let abi = abi_define_value(header_text)?;
    let kernel_code = kernel_code_enum(header_text)?;

    let mut out = String::new();
    out.push_str(
        "/**\n\
         * Generated by modukit-codegen (wave-2 JS volume M1, D102 R2-A) from\n\
         * modukit_embed.h. DO NOT EDIT - the contract ORIGINAL is the C header this\n\
         * facade mirrors; skins are generated FROM it (D63: hand-writing a\n\
         * replacement of this facade is forbidden).\n\
         *\n\
         * Regen: cargo run -q -p modukit-codegen --bin protoc-gen-modukit --\n\
         *   @contracts/org.modukit.runtime.proto.fdset @contracts/com.modukit.demo.proto.fdset\n\
         *   --embed_header contracts/modukit_embed.h --js_out <dir> --output_dir <dir>\n\
         *\n\
         * JS EMBED FLOOR (",
    );
    let _ = write!(out, "{}", verbs.len());
    out.push_str(
        r#" rung-0 verbs, M1 scope - the modukit_sup_* exe
 * rung joins at M2): loads the generated napi bridge (.node) from
 * MODUKIT_JS_BRIDGE or argv[2], re-exports the verb functions 1:1 (each doc
 * line cites its header declaration), mirrors the enums as frozen objects
 * and the D51 error dictionary with codes as data. No async, no sugar
 * beyond names (D6: the face is sync like the C door; the kernel behind
 * the bridge is the ambient process-global in-process one).
 * Integer crossing: 64-bit values arrive as JS numbers through the
 * bridge's i64 channel (exact below 2^53; ns stamps may round - sizes and
 * ring seqs do not). A negative raw is -(D51 code): narrow with
 * EMBED_ERRORS[-raw]. Buffer in, Buffer out (the C byte doors).
 */
'use strict';

const nodePath = require('node:path');
const addonPath = process.env.MODUKIT_JS_BRIDGE || process.argv[2];
if (!addonPath) {
  throw new Error('modukit_embed.js: set MODUKIT_JS_BRIDGE to the built bridge .node path');
}
const addon = require(nodePath.resolve(addonPath));
"#,
    );

    for v in &verbs {
        let c_decl = if v.params.is_empty() {
            format!("int32_t {}(void)", v.name)
        } else {
            format!("int32_t {}({})", v.name, v.params.join(", "))
        };
        let _ = write!(
            out,
            "\n/** {decl} - contracts/modukit_embed.h:{line} */\nexports.{name} = addon.{name};\n",
            decl = c_decl,
            name = v.name,
            line = verb_line(header_text, &v.name),
        );
    }

    for e in &enums {
        let _ = write!(
            out,
            "\n/** mirror of `{tag}` - values verbatim, MODUKIT_ prefix dropped */\nexports.{cls} = Object.freeze({{\n",
            cls = enum_class_name(&e.tag),
            tag = e.tag,
        );
        for (name, number) in &e.values {
            let short = name.strip_prefix("MODUKIT_").unwrap_or(name);
            let _ = writeln!(out, "  {short}: {number},");
        }
        out.push_str("});\n");
    }

    out.push_str(
        "\n/** the D51 kernel dictionary: raw code number -> name (codes as data;\n\
         * verbs return -(code) on failure, so narrow with EMBED_ERRORS[-raw]) */\n\
         exports.EMBED_ERRORS = Object.freeze({\n",
    );
    for (name, number) in &kernel_code.values {
        let short = name.strip_prefix("MODUKIT_").unwrap_or(name);
        let _ = writeln!(out, "  {number}: '{short}',");
    }
    let _ = write!(
        out,
        "}});\n\n/** mirror of the `MODUKIT_EMBED_ABI_VERSION` define (contracts/modukit_embed.h) */\nexports.MODUKIT_EMBED_ABI_VERSION = {abi};\n"
    );
    Ok(out)
}

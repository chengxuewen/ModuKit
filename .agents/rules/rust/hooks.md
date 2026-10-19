---
paths:
  - "**/*.rs"
  - "**/Cargo.toml"
---
# Rust Hooks

> This file extends [common/hooks.md](../common/hooks.md) with Rust-specific content.

## PostToolUse Hooks

No live hook mechanism on this host (the ported `~/.claude/settings.json` pointer was
stripped in the 2026-10-19 dangling-refs sweep); apply these as manual post-edit habits:

- **rustfmt (single file only)**: `rustfmt --edition 2024 <edited-file>` after edit — never workspace `cargo fmt` (edit-safety #12)
- **cargo clippy**: Run lint checks after editing Rust files
- **cargo check**: Verify compilation after changes (faster than `cargo build`)

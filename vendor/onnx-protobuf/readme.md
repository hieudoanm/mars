# onnx protobuf

ONNX protocol buffers messages for Rust.

## Local patches (do not lose on re-vendor)

This copy is vendored via the workspace `[patch.crates-io]` because upstream
`onnx-protobuf 0.2.3` pins a `protobuf-codegen` version check that rejects
`protobuf 3.7.x`. Local changes on top of the published crate:

- `Cargo.toml` — version-check workaround for `protobuf 3.7.x` (see root
  `Cargo.toml` comment).
- `src/proto/onnx.rs` — removed `#![allow(box_pointers)]` (lint deleted in
  current rustc; the allow itself warned under `renamed_and_removed_lints`).
- `src/helpers/mod.rs` — `as_value` return types annotated `<'_>` to satisfy
  `mismatched_lifetime_syntaxes`.

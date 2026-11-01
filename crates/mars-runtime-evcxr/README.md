# mars-runtime-evcxr

Evvxr-based Rust execution backend for the Mars notebook platform.

## Purpose

`mars-runtime-evcxr` provides an optional Evcxr-powered runtime backend (`EvcxrSession`) implementing `mars_runtime::RuntimeBackend`. It is used by `mars-kernel` behind the `experimental-evcxr` feature.

## Modules

- **`session`** — `EvcxrSession`.
- **`backend`** — `RuntimeBackend` impl for Evcxr.
- **`context`** — Execution context handling.
- **`control`** — Execution control/cancellation.
- **`output`** — MIME bundle output normalization (plotters SVG, image bundles).

## Usage

Enabled via `mars-kernel`'s `experimental-evcxr` feature:

```toml
mars-kernel = { path = "../mars-kernel", features = ["experimental-evcxr"] }
```

## Notes

Evvxr is not a mandatory dependency for Mars. This crate exists to provide an alternative runtime backend option.

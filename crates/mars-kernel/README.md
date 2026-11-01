# mars-kernel

Kernel lifecycle and request handling for the Mars notebook platform.

## Purpose

`mars-kernel` orchestrates execution runtimes, handles kernel requests (execute, reset, status), and translates protocol messages into runtime actions.

## Modules

- **`kernel`** — `Kernel`, `RuntimeBackendKind` (Replay, Evcxr).
- **`runtime`** — Runtime creation (`create_runtime`) and hooks.
- **`error`** — `KernelError`.

## Usage

```rust
use mars_kernel::Kernel;
use mars_protocol::{KernelRequest, RequestEnvelope};

let mut kernel = Kernel::new()?;
let response = kernel.handle(RequestEnvelope {
    id: "exec-1".to_owned(),
    request: KernelRequest::Execute {
        source: "let x = 42; x".to_owned(),
        timeout_millis: None,
    },
});
```

## Features

- `experimental-evcxr` — Enable the Evcxr-based runtime backend (optional dependency).

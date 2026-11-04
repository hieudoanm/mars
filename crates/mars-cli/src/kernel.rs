use mars_document::load;
use mars_kernel::{Kernel, RuntimeBackendKind};
use mars_protocol::RequestEnvelope;
use std::io::{self, BufWriter, Write};

pub fn serve_kernel(backend: RuntimeBackendKind, notebook: Option<String>) -> Result<(), String> {
    let stdin = io::stdin();
    let mut stdout = BufWriter::new(io::stdout().lock());
    let mut kernel = if let Some(path) = notebook {
        let notebook = load(&path).map_err(|error| error.to_string())?;
        Kernel::with_configuration(
            notebook.metadata.dependencies,
            notebook.metadata.cargo_lock,
            false,
            false,
            backend,
        )
        .map_err(|error| error.to_string())?
    } else {
        Kernel::with_backend(Default::default(), backend, false)
            .map_err(|error| error.to_string())?
    };
    let mut line = String::new();
    loop {
        line.clear();
        if stdin
            .read_line(&mut line)
            .map_err(|error| error.to_string())?
            == 0
        {
            break;
        }
        if line.trim().is_empty() {
            continue;
        }
        let request: RequestEnvelope =
            serde_json::from_str(&line).map_err(|error| format!("invalid request: {error}"))?;
        let response = kernel.handle(request);
        serde_json::to_writer(&mut stdout, &response).map_err(|error| error.to_string())?;
        stdout.write_all(b"\n").map_err(|error| error.to_string())?;
        stdout.flush().map_err(|error| error.to_string())?;
    }
    Ok(())
}

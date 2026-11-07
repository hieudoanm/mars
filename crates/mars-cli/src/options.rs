use mars_kernel::RuntimeBackendKind;
use std::time::Duration;

pub struct RunOptions {
    pub timeout: Option<Duration>,
    pub offline: bool,
    pub locked: bool,
    pub deterministic: bool,
    pub save: bool,
    pub report: bool,
    pub runtime: RuntimeBackendKind,
}

pub struct ServeOptions {
    pub backend: RuntimeBackendKind,
    pub notebook: Option<String>,
    pub language: Option<String>,
    pub tcp: Option<u16>,
    pub tcp_host: Option<String>,
}

pub fn parse_run_options(args: &[String]) -> Result<RunOptions, String> {
    let mut options = RunOptions {
        timeout: None,
        offline: false,
        locked: false,
        deterministic: false,
        save: true,
        report: false,
        runtime: RuntimeBackendKind::Replay,
    };
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--offline" => options.offline = true,
            "--locked" => options.locked = true,
            "--deterministic" => {
                options.deterministic = true;
                options.locked = true;
            }
            "--no-save" => options.save = false,
            "--report" => options.report = true,
            "--runtime" => {
                let backend = args.get(index + 1).ok_or("missing runtime backend")?;
                options.runtime = parse_backend(backend)?;
                index += 1;
            }
            "--timeout-ms" => {
                let value = args.get(index + 1).ok_or("missing timeout value")?;
                let milliseconds = value
                    .parse::<u64>()
                    .map_err(|_| "timeout must be an integer")?;
                if milliseconds == 0 {
                    return Err("timeout must be greater than zero".to_owned());
                }
                options.timeout = Some(Duration::from_millis(milliseconds));
                index += 1;
            }
            other => return Err(format!("unknown run option: {other}")),
        }
        index += 1;
    }
    Ok(options)
}

pub fn parse_serve_options(args: &[String]) -> Result<ServeOptions, String> {
    let mut options = ServeOptions {
        backend: RuntimeBackendKind::Replay,
        notebook: None,
        language: None,
        tcp: None,
        tcp_host: None,
    };
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--runtime" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "missing value for --runtime".to_owned())?;
                options.backend = parse_backend(value)?;
                index += 2;
            }
            "--notebook" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "missing value for --notebook".to_owned())?;
                options.notebook = Some(value.clone());
                index += 2;
            }
            "--language" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "missing value for --language".to_owned())?;
                options.language = Some(value.clone());
                index += 2;
            }
            "--tcp" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "missing value for --tcp".to_owned())?;
                let port: u16 = value
                    .parse()
                    .map_err(|_| format!("invalid --tcp port: {value}"))?;
                options.tcp = Some(port);
                index += 2;
            }
            "--tcp-host" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "missing value for --tcp-host".to_owned())?;
                options.tcp_host = Some(value.clone());
                index += 2;
            }
            unknown => return Err(format!("unknown kernel serve option: {unknown}")),
        }
    }
    Ok(options)
}

pub struct McpOptions {
    pub backend: RuntimeBackendKind,
    pub offline: bool,
    pub notebook: Option<String>,
    pub language: Option<String>,
}

pub fn parse_mcp_options(args: &[String]) -> Result<McpOptions, String> {
    let mut options = McpOptions {
        backend: RuntimeBackendKind::Replay,
        offline: false,
        notebook: None,
        language: None,
    };
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--runtime" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "missing value for --runtime".to_owned())?;
                options.backend = parse_backend(value)?;
                index += 2;
            }
            "--offline" => {
                options.offline = true;
                index += 1;
            }
            "--notebook" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "missing value for --notebook".to_owned())?;
                options.notebook = Some(value.clone());
                index += 2;
            }
            "--language" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "missing value for --language".to_owned())?;
                options.language = Some(value.clone());
                index += 2;
            }
            unknown => return Err(format!("unknown mcp serve option: {unknown}")),
        }
    }
    Ok(options)
}

fn parse_backend(value: &str) -> Result<RuntimeBackendKind, String> {
    match value {
        "replay" => Ok(RuntimeBackendKind::Replay),
        "evcxr" => Ok(RuntimeBackendKind::Evcxr),
        _ => Err(format!("unsupported runtime backend: {value}")),
    }
}

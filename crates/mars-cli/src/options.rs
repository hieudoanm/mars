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

fn parse_backend(value: &str) -> Result<RuntimeBackendKind, String> {
    match value {
        "replay" => Ok(RuntimeBackendKind::Replay),
        "evcxr" => Ok(RuntimeBackendKind::Evcxr),
        _ => Err(format!("unsupported runtime backend: {value}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn serve_tcp_options_parse() {
        let options =
            parse_serve_options(&args(&["--tcp", "9000", "--tcp-host", "0.0.0.0"])).expect("parse");
        assert_eq!(options.tcp, Some(9000));
        assert_eq!(options.tcp_host.as_deref(), Some("0.0.0.0"));
    }

    #[test]
    fn serve_tcp_requires_port() {
        assert!(parse_serve_options(&args(&["--tcp"])).is_err());
        assert!(parse_serve_options(&args(&["--tcp", "abc"])).is_err());
    }
}

use mars::options::{parse_mcp_options, parse_run_options, parse_serve_options};
use mars_kernel::RuntimeBackendKind;
use std::time::Duration;

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

#[test]
fn run_options_parse_all_flags() {
    let options = parse_run_options(&args(&[
        "--offline",
        "--locked",
        "--deterministic",
        "--no-save",
        "--report",
        "--runtime",
        "evcxr",
        "--timeout-ms",
        "250",
    ]))
    .expect("parse");

    assert!(options.offline);
    assert!(options.locked);
    assert!(options.deterministic);
    assert!(!options.save);
    assert!(options.report);
    assert_eq!(options.runtime, RuntimeBackendKind::Evcxr);
    assert_eq!(options.timeout, Some(Duration::from_millis(250)));
}

#[test]
fn run_options_parse_watch_flags() {
    let options = parse_run_options(&args(&["--watch", "--watch-ms", "500"])).expect("parse");
    assert!(options.watch);
    assert_eq!(options.watch_ms, 500);

    let defaults = parse_run_options(&args(&[])).expect("parse");
    assert!(!defaults.watch);
    assert_eq!(defaults.watch_ms, 1000);

    assert!(parse_run_options(&args(&["--watch-ms"])).is_err());
    assert!(parse_run_options(&args(&["--watch-ms", "0"])).is_err());
    assert!(parse_run_options(&args(&["--watch-ms", "abc"])).is_err());
}

#[test]
fn run_options_use_replay_defaults() {
    let options = parse_run_options(&args(&[])).expect("parse");
    assert_eq!(options.runtime, RuntimeBackendKind::Replay);
    assert!(options.save);
    assert!(!options.offline);
    assert!(!options.locked);
    assert!(!options.deterministic);
    assert!(!options.report);
    assert_eq!(options.timeout, None);
}

#[test]
fn run_options_reject_invalid_values() {
    assert!(parse_run_options(&args(&["--runtime"])).is_err());
    assert!(parse_run_options(&args(&["--runtime", "bogus"])).is_err());
    assert!(parse_run_options(&args(&["--timeout-ms"])).is_err());
    assert!(parse_run_options(&args(&["--timeout-ms", "abc"])).is_err());
    assert!(parse_run_options(&args(&["--timeout-ms", "0"])).is_err());
    assert!(parse_run_options(&args(&["--bogus"])).is_err());
}

#[test]
fn serve_options_parse_all_flags() {
    let options = parse_serve_options(&args(&[
        "--runtime",
        "evcxr",
        "--notebook",
        "demo.mars",
        "--language",
        "python",
        "--tcp",
        "9000",
        "--tcp-host",
        "127.0.0.1",
    ]))
    .expect("parse");

    assert_eq!(options.backend, RuntimeBackendKind::Evcxr);
    assert_eq!(options.notebook.as_deref(), Some("demo.mars"));
    assert_eq!(options.language.as_deref(), Some("python"));
    assert_eq!(options.tcp, Some(9000));
    assert_eq!(options.tcp_host.as_deref(), Some("127.0.0.1"));
}

#[test]
fn serve_options_reject_missing_and_unknown_values() {
    for case in [
        &["--runtime"][..],
        &["--runtime", "bogus"][..],
        &["--notebook"][..],
        &["--language"][..],
        &["--tcp-host"][..],
        &["--bogus"][..],
    ] {
        assert!(parse_serve_options(&args(case)).is_err(), "{case:?}");
    }
}

#[test]
fn mcp_options_parse_all_flags() {
    let options = parse_mcp_options(&args(&[
        "--runtime",
        "evcxr",
        "--offline",
        "--notebook",
        "demo.mars",
        "--language",
        "python",
    ]))
    .expect("parse");

    assert_eq!(options.backend, RuntimeBackendKind::Evcxr);
    assert!(options.offline);
    assert_eq!(options.notebook.as_deref(), Some("demo.mars"));
    assert_eq!(options.language.as_deref(), Some("python"));
}

#[test]
fn mcp_options_reject_invalid_values() {
    for case in [
        &["--runtime"][..],
        &["--runtime", "bogus"][..],
        &["--notebook"][..],
        &["--language"][..],
        &["--bogus"][..],
    ] {
        assert!(parse_mcp_options(&args(case)).is_err(), "{case:?}");
    }
}

#[test]
fn mcp_options_reject_missing_flag_values() {
    for flag in ["--runtime", "--notebook", "--http", "--token", "--language"] {
        assert!(parse_mcp_options(&args(&[flag])).is_err(), "{flag}");
    }
}

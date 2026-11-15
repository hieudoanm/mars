use mars_kernel::runtime_hook;
use std::env;
use std::process::ExitCode;

mod changes;
mod commands;
mod doctor;
mod kernel;
mod lab;
mod mcp;
mod output;
mod templates;
mod util;

fn main() -> ExitCode {
    runtime_hook();
    match commands::run_cli(env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("mars: {message}");
            ExitCode::FAILURE
        }
    }
}

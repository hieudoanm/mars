use mars_kernel::runtime_hook;
use std::env;
use std::process::ExitCode;

mod commands;
mod kernel;
mod options;
mod output;
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

use mars_runtime::RustSession;
use std::{
    env,
    time::{Duration, Instant},
};

fn main() {
    let cell_count = env::args()
        .nth(1)
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(8);
    if cell_count == 0 {
        eprintln!("cell count must be greater than zero");
        std::process::exit(2);
    }

    let mut session = RustSession::new().expect("create runtime session");
    let started = Instant::now();
    let mut durations = Vec::with_capacity(cell_count);
    for index in 0..cell_count {
        let source = if index == 0 {
            "let mut total = 0i64; total += 1;".to_owned()
        } else {
            format!("total += {index}; total")
        };
        let cell_started = Instant::now();
        session.execute(&source).expect("execute benchmark cell");
        durations.push(cell_started.elapsed());
    }

    let total = started.elapsed();
    let average = total / cell_count as u32;
    let last = durations.last().copied().unwrap_or(Duration::ZERO);
    println!("runtime=replay");
    println!("cells={cell_count}");
    println!("total_ms={}", total.as_secs_f64() * 1000.0);
    println!("average_ms={}", average.as_secs_f64() * 1000.0);
    println!("last_cell_ms={}", last.as_secs_f64() * 1000.0);
    println!("execution_count={}", session.execution_count());
}

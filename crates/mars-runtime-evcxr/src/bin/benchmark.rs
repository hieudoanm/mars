use mars_runtime::{RuntimeBackend, RustSession};
use mars_runtime_evcxr::EvcxrSession;
use std::{
    collections::BTreeMap,
    sync::{Arc, atomic::AtomicBool},
    time::{Duration, Instant},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    mars_runtime_evcxr::runtime_hook();
    let mut arguments = std::env::args().skip(1);
    let runs = arguments
        .next()
        .map(|value| value.parse::<usize>())
        .transpose()?
        .unwrap_or(20);
    let backends = match arguments.next().as_deref() {
        None | Some("both") => vec!["replay", "evcxr"],
        Some("replay") => vec!["replay"],
        Some("evcxr") => vec!["evcxr"],
        Some(value) => return Err(format!("unknown backend selector: {value}").into()),
    };
    println!("backend,workload,runs,median_ms,p95_ms,samples_ms");
    for backend in backends {
        benchmark_startup(backend, runs)?;
        benchmark_single_cells(backend, runs)?;
        for count in [8, 16, 32] {
            benchmark_notebook(backend, count, 5)?;
        }
        benchmark_dependency(backend, 5)?;
    }
    Ok(())
}

fn benchmark_startup(backend: &str, runs: usize) -> Result<(), Box<dyn std::error::Error>> {
    let samples = measure(runs, || {
        let started = Instant::now();
        let _session = make_backend(backend)?;
        Ok(started.elapsed())
    })?;
    print_samples(backend, "kernel_startup_cold_session", samples);
    Ok(())
}

fn benchmark_single_cells(backend: &str, runs: usize) -> Result<(), Box<dyn std::error::Error>> {
    for (name, setup, cell) in [
        ("first_cell", Vec::new(), "1 + 1"),
        ("repeated_identical_cell", Vec::new(), "1 + 1"),
        (
            "variable_reuse",
            vec!["let benchmark_values = vec![1, 2, 3, 4, 5];"],
            "benchmark_values.iter().sum::<i32>()",
        ),
        (
            "function_call",
            vec!["fn benchmark_double(value: i32) -> i32 { value * 2 }"],
            "benchmark_double(21)",
        ),
        (
            "struct_instantiation",
            vec!["struct BenchmarkPoint { x: i32, y: i32 }"],
            "BenchmarkPoint { x: 2, y: 5 }.x + BenchmarkPoint { x: 2, y: 5 }.y",
        ),
    ] {
        let samples = measure(runs, || {
            let mut runtime = make_backend(backend)?;
            for source in &setup {
                execute(&mut *runtime, source)?;
            }
            if name == "repeated_identical_cell" {
                execute(&mut *runtime, cell)?;
            }
            let started = Instant::now();
            execute(&mut *runtime, cell)?;
            Ok(started.elapsed())
        })?;
        print_samples(backend, name, samples);
    }
    Ok(())
}

fn benchmark_notebook(
    backend: &str,
    count: usize,
    runs: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let samples = measure(runs, || {
        let mut runtime = make_backend(backend)?;
        let started = Instant::now();
        for index in 0..count {
            execute(
                &mut *runtime,
                &format!("let notebook_cell_{index} = {index};"),
            )?;
        }
        Ok(started.elapsed())
    })?;
    print_samples(backend, &format!("{count}_cells"), samples);
    Ok(())
}

fn benchmark_dependency(backend: &str, runs: usize) -> Result<(), Box<dyn std::error::Error>> {
    let samples = measure(runs, || {
        let started = Instant::now();
        let mut runtime = if backend == "replay" {
            Box::new(RustSession::with_dependencies(BTreeMap::from([(
                "regex".to_owned(),
                "1.12.2".to_owned(),
            )]))?) as Box<dyn RuntimeBackend>
        } else {
            Box::new(EvcxrSession::with_dependencies(BTreeMap::from([(
                "regex".to_owned(),
                "1.12.2".to_owned(),
            )]))?) as Box<dyn RuntimeBackend>
        };
        execute(
            &mut *runtime,
            "regex::Regex::new(\"^mars$\").map(|re| re.is_match(\"mars\"))",
        )?;
        Ok(started.elapsed())
    })?;
    print_samples(backend, "dependency_backed_first_cell", samples);
    Ok(())
}

fn make_backend(backend: &str) -> Result<Box<dyn RuntimeBackend>, Box<dyn std::error::Error>> {
    match backend {
        "replay" => Ok(Box::new(RustSession::new()?)),
        "evcxr" => Ok(Box::new(EvcxrSession::new()?)),
        _ => Err(format!("unknown backend: {backend}").into()),
    }
}

fn execute(
    runtime: &mut dyn RuntimeBackend,
    source: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let result = runtime.execute_with_control(
        source,
        Some(Duration::from_secs(180)),
        Arc::new(AtomicBool::new(false)),
    )?;
    if result.interrupted
        || result
            .outputs
            .iter()
            .any(|output| matches!(output, mars_core::Output::Error { .. }))
    {
        return Err(format!("benchmark cell failed: {source}: {:?}", result.outputs).into());
    }
    Ok(())
}

fn measure<F>(runs: usize, mut operation: F) -> Result<Vec<f64>, Box<dyn std::error::Error>>
where
    F: FnMut() -> Result<Duration, Box<dyn std::error::Error>>,
{
    let mut samples = Vec::with_capacity(runs);
    for _ in 0..runs {
        samples.push(operation()?.as_secs_f64() * 1_000.0);
    }
    Ok(samples)
}

fn print_samples(backend: &str, workload: &str, mut samples: Vec<f64>) {
    samples.sort_by(f64::total_cmp);
    let median = percentile(&samples, 0.50);
    let p95 = percentile(&samples, 0.95);
    let values = samples
        .iter()
        .map(|value| format!("{value:.3}"))
        .collect::<Vec<_>>()
        .join(";");
    println!(
        "{backend},{workload},{},{median:.3},{p95:.3},{values}",
        samples.len()
    );
}

fn percentile(samples: &[f64], quantile: f64) -> f64 {
    samples[((samples.len() as f64 * quantile).ceil() as usize).saturating_sub(1)]
}

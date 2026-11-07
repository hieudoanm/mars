use mars_runtime::execution::{PromptScan, run_child, run_child_streaming};
use mars_runtime::output::{ChunkStream, StreamEvent};
use std::process::Command;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn sh(script: &str) -> std::process::Child {
    Command::new("sh")
        .arg("-c")
        .arg(script)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn sh")
}

#[test]
fn run_child_returns_output_on_success() {
    let child = sh("echo hello; echo oops >&2");
    let output = run_child(
        child,
        Some(Duration::from_secs(10)),
        &AtomicBool::new(false),
    )
    .expect("run");
    assert!(!output.interrupted);
    assert!(output.status.expect("status").success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "hello");
    assert_eq!(String::from_utf8_lossy(&output.stderr).trim(), "oops");
}

#[test]
fn run_child_times_out_and_interrupts() {
    let child = sh("sleep 30");
    let output = run_child(
        child,
        Some(Duration::from_millis(200)),
        &AtomicBool::new(false),
    )
    .expect("run");
    assert!(output.interrupted);
    assert!(output.status.is_none());
}

#[test]
fn run_child_honours_cancellation() {
    let child = sh("sleep 30");
    let cancellation = AtomicBool::new(false);
    let flag = Arc::new(cancellation);
    let setter = Arc::clone(&flag);
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(100));
        setter.store(true, std::sync::atomic::Ordering::Relaxed);
    });
    let output = run_child(child, Some(Duration::from_secs(30)), flag.as_ref()).expect("run");
    assert!(output.interrupted);
}

#[test]
fn streaming_collects_chunks_and_prompts() {
    let events = Arc::new(Mutex::new(Vec::<StreamEvent>::new()));
    let sink_events = Arc::clone(&events);
    let mut sink = move |event: StreamEvent| sink_events.lock().expect("lock").push(event);

    let child = sh(
        "printf 'out1\\n'; printf 'oops\\n' >&2; printf '__MARS_PROMPT__Pick one\\n'; printf 'out2\\n'",
    );
    let output = run_child_streaming(
        child,
        Some(Duration::from_secs(10)),
        &AtomicBool::new(false),
        Some(&mut sink),
    )
    .expect("run");

    assert!(!output.interrupted);
    let collected = events.lock().expect("lock");
    assert!(collected.iter().any(|event| matches!(
        event,
        StreamEvent::Prompt { message } if message == "Pick one"
    )));
    let stdout_text: String = collected
        .iter()
        .filter_map(|event| match event {
            StreamEvent::Chunk {
                stream: ChunkStream::Stdout,
                data,
            } => Some(data.clone()),
            _ => None,
        })
        .collect();
    assert!(stdout_text.contains("out1"));
    assert!(stdout_text.contains("out2"));
    assert!(!stdout_text.contains("__MARS_PROMPT__"));
    assert!(collected.iter().any(|event| matches!(
        event,
        StreamEvent::Chunk {
            stream: ChunkStream::Stderr,
            ..
        }
    )));
}

#[test]
fn streaming_without_sink_still_captures_output() {
    let child = sh("echo done");
    let output = run_child_streaming(
        child,
        None,
        &AtomicBool::new(false),
        None::<&mut dyn FnMut(StreamEvent)>,
    )
    .expect("run");
    assert!(!output.interrupted);
    assert!(String::from_utf8_lossy(&output.stdout).contains("done"));
}

#[test]
fn streaming_timeout_interrupts() {
    let child = sh("sleep 30");
    let output = run_child_streaming(
        child,
        Some(Duration::from_millis(200)),
        &AtomicBool::new(false),
        None::<&mut dyn FnMut(StreamEvent)>,
    )
    .expect("run");
    assert!(output.interrupted);
}

#[test]
fn prompt_scan_handles_split_and_unterminated_markers() {
    let mut scan = PromptScan::default();

    // Marker split across feeds. The scanner holds back a tail that could
    // be the start of a marker, so text is only flushed by finish() or once
    // enough trailing output accumulates.
    let (cleaned, prompts) = scan.feed("before __MARS_PRO");
    assert!(!cleaned.contains("__MARS"));
    assert!(prompts.is_empty());
    let mut all = cleaned;
    let (cleaned, prompts) = scan.feed("MPT__question?\n".repeat(4).as_str());
    assert_eq!(prompts, vec!["question?".to_owned()]);
    all.push_str(&cleaned);
    let (tail, tail_prompts) = scan.finish();
    assert!(tail_prompts.is_empty());
    all.push_str(&tail);
    assert!(all.contains("before "), "{all:?}");
    assert!(all.contains("question?"));

    // Unterminated marker at EOF surfaces via finish().
    let mut scan = PromptScan::default();
    let (cleaned, _) = scan.feed("start __MARS_PROMPT__dangling");
    let (tail, prompts) = scan.finish();
    assert_eq!(prompts, vec!["dangling".to_owned()]);
    assert!(format!("{cleaned}{tail}").contains("start"));
    assert!(!format!("{cleaned}{tail}").contains("__MARS_PROMPT__"));

    // A tail that only partially matches the marker is preserved as text.
    let mut scan = PromptScan::default();
    let mut all = scan.feed("ends with __MARS").0;
    all.push_str(&scan.feed("_PROM").0);
    let (cleaned, prompts) = scan.feed("PT__real?\n");
    all.push_str(&cleaned);
    assert_eq!(prompts, vec!["real?".to_owned()]);
    // Pre-marker text may still be held in the carry; finish() flushes it.
    all.push_str(&scan.finish().0);
    assert!(all.contains("with"), "{all:?}");
    assert!(all.contains("ends"), "{all:?}");
}

use mars_runtime::CancellationToken;
use mars_runtime::execution::{interruption_message, remaining_timeout};
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

#[test]
fn cancellation_token_shares_state_across_clones() {
    let token = CancellationToken::new();
    assert!(!token.is_cancelled());
    let clone = token.clone();
    clone.cancel();
    assert!(token.is_cancelled());
    assert!(clone.is_cancelled());
    // The raw flag is the same allocation.
    assert!(token.flag().load(std::sync::atomic::Ordering::Relaxed));
}

#[test]
fn interruption_message_depends_on_flag_and_phase() {
    let cancelled = AtomicBool::new(true);
    let active = AtomicBool::new(false);
    assert_eq!(
        interruption_message(&cancelled, "compile"),
        "execution cancelled"
    );
    assert_eq!(
        interruption_message(&cancelled, "compilation"),
        "compilation cancelled"
    );
    assert_eq!(
        interruption_message(&active, "compilation"),
        "compilation exceeded the configured time limit"
    );
    assert_eq!(
        interruption_message(&active, "run"),
        "execution exceeded the configured time limit"
    );
}

#[test]
fn remaining_timeout_computes_deadline() {
    let started = Instant::now();
    assert_eq!(remaining_timeout(started, None), None);
    let remaining =
        remaining_timeout(started, Some(Duration::from_secs(60))).expect("some remaining");
    assert!(remaining <= Duration::from_secs(60));
    assert!(remaining > Duration::from_secs(59));
}

use mars_runtime::execution::PromptScan;

#[test]
fn extracts_prompt_and_cleans_output() {
    let mut scan = PromptScan::default();
    let (_, prompts) = scan.feed("Hello\n__MARS_PROMPT__Name?\nWorld\n");
    assert_eq!(prompts, vec!["Name?"]);
    let (tail, _) = scan.finish();
    assert!(tail.contains("Hello"));
    assert!(tail.contains("World"));
    assert!(!tail.contains("__MARS_PROMPT__"));
}

#[test]
fn carries_marker_across_chunks() {
    let mut scan = PromptScan::default();
    let (clean1, prompts1) = scan.feed("start __MARS_PROM");
    assert!(prompts1.is_empty());
    assert!(clean1.starts_with("sta"));
    let (_, prompts2) = scan.feed("PT__Pick one\nrest");
    assert_eq!(prompts2, vec!["Pick one"]);
    let (tail, _) = scan.finish();
    assert!(tail.contains("rest"));
    assert!(!tail.contains("__MARS_PROMPT__"));
}

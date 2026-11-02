use evcxr::{CommandContext, Error as EvcxrError, EvalContextOutputs};
use mars_core::Output;
use mars_runtime::{ExecutionResult, RuntimeBackend, RuntimeError, validate_dependencies};
use serde_json::{Map, Value};
use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

/// Persistent Evcxr-backed session. Evcxr keeps user code in its own subprocess.
pub struct EvcxrSession {
    context: Arc<std::sync::Mutex<CommandContext>>,
    outputs: EvalContextOutputs,
    dependencies: BTreeMap<String, String>,
    offline: bool,
    execution_count: u64,
}

impl EvcxrSession {
    pub fn new() -> Result<Self, RuntimeError> {
        Self::with_configuration(BTreeMap::new(), false)
    }

    pub fn with_dependencies(dependencies: BTreeMap<String, String>) -> Result<Self, RuntimeError> {
        Self::with_configuration(dependencies, false)
    }

    pub fn with_configuration(
        dependencies: BTreeMap<String, String>,
        offline: bool,
    ) -> Result<Self, RuntimeError> {
        validate_dependencies(&dependencies)?;
        let (context, outputs) = create_context(&dependencies, offline)?;
        Ok(Self {
            context,
            outputs,
            dependencies,
            offline,
            execution_count: 0,
        })
    }

    fn recreate(&mut self) -> Result<(), RuntimeError> {
        let (context, outputs) = create_context(&self.dependencies, self.offline)?;
        self.context = context;
        self.outputs = outputs;
        self.execution_count = 0;
        Ok(())
    }

    fn collect_outputs(&self, result: Result<evcxr::EvalOutputs, EvcxrError>) -> Vec<Output> {
        let mut outputs = Vec::new();
        let stdout = drain_channel(&self.outputs.stdout);
        let stderr = drain_channel(&self.outputs.stderr);
        if !stdout.is_empty() {
            outputs.push(Output::Text { text: stdout });
        }
        match result {
            Ok(result) => {
                let mut mime_data = Map::new();
                for (mime_type, content) in result.content_by_mime_type {
                    mime_data.insert(mime_type, Value::String(content));
                }
                append_mime_output(&mut outputs, mime_data);
            }
            Err(error) => {
                let name = match &error {
                    EvcxrError::CompilationErrors(_) => "CompilationError",
                    EvcxrError::SubprocessTerminated(_) => "RuntimeError",
                    EvcxrError::TypeRedefinedVariablesLost(_) | EvcxrError::Message(_) => {
                        "EvcxrError"
                    }
                };
                outputs.push(Output::Error {
                    name: name.to_owned(),
                    message: error.to_string(),
                    traceback: Vec::new(),
                });
            }
        }
        if !stderr.is_empty() {
            outputs.push(Output::Error {
                name: "RuntimeStderr".to_owned(),
                message: stderr,
                traceback: Vec::new(),
            });
        }
        outputs
    }
}

impl RuntimeBackend for EvcxrSession {
    fn execute_with_control(
        &mut self,
        source: &str,
        timeout: Option<Duration>,
        cancellation: Arc<AtomicBool>,
    ) -> Result<ExecutionResult, RuntimeError> {
        if source.trim().is_empty() {
            return Err(RuntimeError::EmptySource);
        }
        if cancellation.load(Ordering::Relaxed) {
            return Ok(interrupted_result(
                Duration::ZERO,
                self.execution_count,
                "execution cancelled before evaluation",
            ));
        }

        let started = Instant::now();
        let context = Arc::clone(&self.context);
        let process_handle = context
            .lock()
            .map_err(|_| RuntimeError::Backend("Evcxr context lock is poisoned".to_owned()))?
            .process_handle();
        let (sender, receiver) = crossbeam_channel::bounded(1);
        let source = source.to_owned();
        thread::Builder::new()
            .name("mars-evcxr-controller".to_owned())
            .spawn(move || {
                let result: Result<Result<evcxr::EvalOutputs, EvcxrError>, String> =
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        let mut context = context
                            .lock()
                            .map_err(|_| "Evcxr context lock is poisoned".to_owned())?;
                        Ok(context.execute(&source))
                    }))
                    .unwrap_or_else(|_| Err("Evcxr evaluation thread panicked".to_owned()));
                let _ = sender.send(result);
            })
            .map_err(|error| {
                RuntimeError::Backend(format!("could not start Evcxr controller thread: {error}"))
            })?;

        let evaluation = loop {
            match receiver.recv_timeout(Duration::from_millis(5)) {
                Ok(Ok(result)) => break result,
                Ok(Err(message)) => break Err(EvcxrError::Message(message)),
                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                    break Err(EvcxrError::Message(
                        "Evcxr evaluation thread stopped".to_owned(),
                    ));
                }
                Err(crossbeam_channel::RecvTimeoutError::Timeout) => {}
            }
            let message = if cancellation.load(Ordering::Relaxed) {
                Some("execution cancelled; Evcxr session was restarted")
            } else if timeout.is_some_and(|limit| started.elapsed() >= limit) {
                Some("execution timed out; Evcxr session was restarted")
            } else {
                None
            };
            if let Some(message) = message {
                terminate_evaluator(&process_handle);
                self.recreate()?;
                return Ok(interrupted_result(
                    started.elapsed(),
                    self.execution_count,
                    message,
                ));
            }
        };

        let failed = evaluation.is_err();
        let outputs = self.collect_outputs(evaluation);
        if failed {
            let lost_process = outputs.iter().any(
                |output| matches!(output, Output::Error { name, .. } if name == "RuntimeError"),
            );
            if lost_process {
                self.recreate()?;
                return Ok(ExecutionResult {
                    outputs,
                    duration: started.elapsed(),
                    execution_count: self.execution_count,
                    interrupted: false,
                });
            }
            return Ok(ExecutionResult {
                outputs,
                duration: started.elapsed(),
                execution_count: self.execution_count,
                interrupted: false,
            });
        }

        self.execution_count += 1;
        Ok(ExecutionResult {
            outputs,
            duration: started.elapsed(),
            execution_count: self.execution_count,
            interrupted: false,
        })
    }

    fn reset(&mut self) -> Result<(), RuntimeError> {
        self.recreate()
    }

    fn execution_count(&self) -> u64 {
        self.execution_count
    }

    fn cargo_lock(&self) -> Result<Option<String>, RuntimeError> {
        Ok(None)
    }
}

fn create_context(
    dependencies: &BTreeMap<String, String>,
    offline: bool,
) -> Result<(Arc<std::sync::Mutex<CommandContext>>, EvalContextOutputs), RuntimeError> {
    let (mut context, outputs) = CommandContext::new()
        .map_err(|error| RuntimeError::Backend(format!("could not start Evcxr: {error}")))?;
    context
        .execute(":cache 500")
        .map_err(|error| RuntimeError::Backend(format!("could not enable Evcxr cache: {error}")))?;
    if offline {
        context.execute(":offline 1").map_err(|error| {
            RuntimeError::Backend(format!("could not enable offline mode: {error}"))
        })?;
    }
    for (name, specification) in dependencies {
        let value = if specification.trim().starts_with('{') {
            specification.clone()
        } else {
            toml_string(specification)
        };
        context
            .execute(&format!(":dep {name} = {value}"))
            .map_err(|error| {
                RuntimeError::Backend(format!("could not add dependency {name}: {error}"))
            })?;
    }
    Ok((Arc::new(std::sync::Mutex::new(context)), outputs))
}

fn toml_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn drain_channel(receiver: &crossbeam_channel::Receiver<String>) -> String {
    let lines = receiver.try_iter().collect::<Vec<_>>();
    if lines.is_empty() {
        String::new()
    } else {
        format!("{}\n", lines.join("\n"))
    }
}

fn append_mime_output(outputs: &mut Vec<Output>, mut data: Map<String, Value>) {
    if data.is_empty() {
        return;
    }
    promote_html_svg(&mut data);
    if data.len() == 1 {
        if let Some(Value::String(text)) = data.remove("text/plain") {
            outputs.push(Output::Text { text });
            return;
        }
        if let Some(Value::String(html)) = data.remove("text/html") {
            outputs.push(Output::Html { html });
            return;
        }
    }
    outputs.push(Output::MimeBundle {
        data,
        metadata: Map::new(),
    });
}

/// Plotters and similar Evcxr display helpers return an SVG wrapped in HTML.
/// Promote that representation to an `image/svg+xml` MIME entry so every Mars
/// frontend can render it as a vector image without sanitising away the SVG.
fn promote_html_svg(data: &mut Map<String, Value>) {
    if data.keys().any(|mime_type| mime_type.starts_with("image/")) {
        return;
    }
    let Some(Value::String(html)) = data.get("text/html") else {
        return;
    };
    let Some(svg) = extract_svg(html) else {
        return;
    };
    let svg = svg.to_owned();
    data.remove("text/html");
    data.insert("image/svg+xml".to_owned(), Value::String(svg));
    data.entry("text/plain".to_owned())
        .or_insert_with(|| Value::String("Figure (SVG)".to_owned()));
}

fn extract_svg(html: &str) -> Option<&str> {
    let lowercase = html.to_ascii_lowercase();
    let start = lowercase.find("<svg")?;
    let open_end = start + lowercase[start..].find('>')? + 1;
    let end = if lowercase[start..open_end].trim_end().ends_with("/>") {
        open_end
    } else {
        let after_open = &lowercase[open_end..];
        open_end + after_open.find("</svg>")? + "</svg>".len()
    };

    // Do not promote HTML that merely contains an SVG (such as an inline
    // icon); plotters figures consist of nothing but the SVG plus wrappers.
    if has_visible_text(&html[..start]) || has_visible_text(&html[end..]) {
        return None;
    }

    Some(&html[start..end])
}

fn has_visible_text(fragment: &str) -> bool {
    let mut text = String::new();
    let mut in_tag = false;
    for character in fragment.chars() {
        match character {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => text.push(character),
            _ => {}
        }
    }
    !text.trim().is_empty()
}

fn terminate_evaluator(process_handle: &Arc<std::sync::Mutex<std::process::Child>>) {
    if let Ok(mut child) = process_handle.lock() {
        let _ = child.kill();
    }
}

fn interrupted_result(duration: Duration, execution_count: u64, message: &str) -> ExecutionResult {
    ExecutionResult {
        outputs: vec![Output::Error {
            name: "Interrupted".to_owned(),
            message: message.to_owned(),
            traceback: Vec::new(),
        }],
        duration,
        execution_count,
        interrupted: true,
    }
}

/// Route Evcxr's re-executed runtime process into its user-code loop.
pub fn runtime_hook() {
    evcxr::runtime_hook();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn promotes_plotters_svg_html_to_a_mime_bundle() {
        let mut outputs = Vec::new();
        append_mime_output(
            &mut outputs,
            Map::from_iter([(
                "text/html".to_owned(),
                Value::String(
                    "<div style=\"\"><svg xmlns=\"http://www.w3.org/2000/svg\"><circle cx=\"4\" cy=\"4\" r=\"4\"/></svg></div>"
                        .to_owned(),
                ),
            )]),
        );

        assert!(matches!(
            outputs.as_slice(),
            [Output::MimeBundle { data, .. }]
                if data.contains_key("image/svg+xml")
                    && data.contains_key("text/plain")
                    && !data.contains_key("text/html")
        ));
        let Output::MimeBundle { data, .. } = &outputs[0] else {
            return;
        };
        assert!(
            data["image/svg+xml"]
                .as_str()
                .is_some_and(|svg| svg.starts_with("<svg"))
        );
    }

    #[test]
    fn keeps_image_mime_bundles_untouched() {
        let mut outputs = Vec::new();
        append_mime_output(
            &mut outputs,
            Map::from_iter([
                (
                    "image/png".to_owned(),
                    Value::String("iVBORw0KGgo=".to_owned()),
                ),
                (
                    "text/html".to_owned(),
                    Value::String("<svg><circle/></svg>".to_owned()),
                ),
            ]),
        );

        assert!(matches!(
            outputs.as_slice(),
            [Output::MimeBundle { data, .. }]
                if data.contains_key("image/png") && data.contains_key("text/html")
        ));
    }

    #[test]
    fn leaves_html_with_surrounding_text_as_html() {
        let mut outputs = Vec::new();
        append_mime_output(
            &mut outputs,
            Map::from_iter([(
                "text/html".to_owned(),
                Value::String("<p>Inline icon</p><svg><circle/></svg>".to_owned()),
            )]),
        );

        assert!(matches!(
            outputs.as_slice(),
            [Output::Html { html }] if html.contains("Inline icon")
        ));
    }

    #[test]
    fn still_collapses_single_text_and_html_outputs() {
        let mut outputs = Vec::new();
        append_mime_output(
            &mut outputs,
            Map::from_iter([("text/plain".to_owned(), Value::String("42".to_owned()))]),
        );
        assert!(matches!(outputs.as_slice(), [Output::Text { text }] if text == "42"));

        let mut outputs = Vec::new();
        append_mime_output(
            &mut outputs,
            Map::from_iter([(
                "text/html".to_owned(),
                Value::String("<b>safe</b>".to_owned()),
            )]),
        );
        assert!(matches!(outputs.as_slice(), [Output::Html { html }] if html == "<b>safe</b>"));
    }
}

use evcxr::{CommandContext, Error as EvcxrError, EvalContextOutputs};
use mars_core::Output;
use mars_runtime::RuntimeError;
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::sync::Arc;

pub struct EvcxrSession {
    pub context: Arc<std::sync::Mutex<CommandContext>>,
    pub outputs: EvalContextOutputs,
    pub dependencies: BTreeMap<String, String>,
    pub offline: bool,
    pub execution_count: u64,
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
        mars_runtime::validate_dependencies(&dependencies)?;
        let (context, outputs) = crate::context::create_context(&dependencies, offline)?;
        Ok(Self {
            context,
            outputs,
            dependencies,
            offline,
            execution_count: 0,
        })
    }

    pub(crate) fn recreate(&mut self) -> Result<(), RuntimeError> {
        let (context, outputs) = crate::context::create_context(&self.dependencies, self.offline)?;
        self.context = context;
        self.outputs = outputs;
        self.execution_count = 0;
        Ok(())
    }

    pub fn collect_outputs(&self, result: Result<evcxr::EvalOutputs, EvcxrError>) -> Vec<Output> {
        let mut outputs = Vec::new();
        let stdout = crate::output::drain_channel(&self.outputs.stdout);
        let stderr = crate::output::drain_channel(&self.outputs.stderr);
        if !stdout.is_empty() {
            outputs.push(Output::Text { text: stdout });
        }
        match result {
            Ok(result) => {
                let mut mime_data = Map::new();
                for (mime_type, content) in result.content_by_mime_type {
                    mime_data.insert(mime_type, Value::String(content));
                }
                crate::output::append_mime_output(&mut outputs, mime_data);
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

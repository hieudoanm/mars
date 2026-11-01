use evcxr::{CommandContext, EvalContextOutputs};
use mars_runtime::RuntimeError;
use std::collections::BTreeMap;

pub fn create_context(
    dependencies: &BTreeMap<String, String>,
    offline: bool,
) -> Result<
    (
        std::sync::Arc<std::sync::Mutex<CommandContext>>,
        EvalContextOutputs,
    ),
    RuntimeError,
> {
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
    Ok((std::sync::Arc::new(std::sync::Mutex::new(context)), outputs))
}

pub fn toml_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

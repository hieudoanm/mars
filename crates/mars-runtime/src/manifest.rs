use crate::error::RuntimeError;
use std::collections::BTreeMap;

pub fn cargo_manifest(dependencies: &BTreeMap<String, String>) -> String {
    let mut manifest = String::from(
        "[package]\nname = \"mars-session\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\n",
    );
    for (name, version) in dependencies {
        manifest.push_str(name);
        manifest.push_str(" = ");
        if let Some(raw) = crate::toml::toml_dependency_value(version) {
            manifest.push_str(raw);
        } else {
            manifest.push_str(&crate::toml::toml_string(version));
        }
        manifest.push('\n');
    }
    manifest
}

pub fn validate_dependencies(dependencies: &BTreeMap<String, String>) -> Result<(), RuntimeError> {
    for (name, specification) in dependencies {
        if name.is_empty()
            || !name.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '-' | '_')
            })
            || specification.trim().is_empty()
        {
            return Err(RuntimeError::InvalidDependency { name: name.clone() });
        }
    }
    Ok(())
}

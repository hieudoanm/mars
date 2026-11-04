use crate::error::DocumentError;
use crate::notebook::Notebook;
use serde_json::Value;
use std::path::{Path, PathBuf};

pub fn to_json_pretty(notebook: &Notebook) -> Result<String, DocumentError> {
    crate::validation::validate(notebook)?;
    serde_json::to_string_pretty(notebook).map_err(DocumentError::Serialization)
}

pub fn from_json(input: &str) -> Result<Notebook, DocumentError> {
    let mut value: Value = serde_json::from_str(input).map_err(DocumentError::Deserialization)?;
    migrate_value(&mut value)?;
    let notebook: Notebook =
        serde_json::from_value(value).map_err(DocumentError::Deserialization)?;
    crate::validation::validate(&notebook)?;
    Ok(notebook)
}

pub fn save_atomic(notebook: &Notebook, path: impl AsRef<Path>) -> Result<(), DocumentError> {
    let path = path.as_ref();
    let content = to_json_pretty(notebook)?;
    let temporary_path = temporary_path(path);
    std::fs::write(&temporary_path, format!("{content}\n")).map_err(|source| {
        DocumentError::Io {
            path: temporary_path.clone(),
            source,
        }
    })?;
    if let Err(source) = std::fs::rename(&temporary_path, path) {
        let _ = std::fs::remove_file(&temporary_path);
        return Err(DocumentError::Io {
            path: path.to_path_buf(),
            source,
        });
    }
    Ok(())
}

pub fn load(path: impl AsRef<Path>) -> Result<Notebook, DocumentError> {
    let path = path.as_ref();
    let content = std::fs::read_to_string(path).map_err(|source| DocumentError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    from_json(&content)
}

fn migrate_value(value: &mut Value) -> Result<(), DocumentError> {
    let version = value
        .get("format_version")
        .and_then(Value::as_u64)
        .ok_or(DocumentError::MissingFormatVersion)? as u32;
    match version {
        1 => {
            let object = value
                .as_object_mut()
                .ok_or(DocumentError::InvalidDocumentShape)?;
            object.insert(
                "format_version".to_owned(),
                Value::from(crate::CURRENT_FORMAT_VERSION),
            );
            Ok(())
        }
        crate::CURRENT_FORMAT_VERSION => Ok(()),
        other => Err(DocumentError::UnsupportedFormatVersion(other)),
    }
}

fn temporary_path(path: &Path) -> PathBuf {
    let mut temporary = path.to_path_buf();
    let suffix = path
        .file_name()
        .and_then(|name| name.to_str())
        .map_or_else(|| "notebook".to_owned(), ToOwned::to_owned);
    temporary.set_file_name(format!(".{suffix}.tmp"));
    temporary
}

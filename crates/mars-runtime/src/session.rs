use crate::error::RuntimeError;
use std::{collections::BTreeMap, fs, path::PathBuf};
use uuid::Uuid;

pub struct RustSession {
    pub workdir: PathBuf,
    pub cells: Vec<String>,
    pub execution_count: u64,
    pub dependencies: BTreeMap<String, String>,
    pub cargo_lock: Option<String>,
    pub offline: bool,
    pub locked: bool,
}

impl RustSession {
    pub fn new() -> Result<Self, RuntimeError> {
        Self::with_dependencies(BTreeMap::new())
    }

    pub fn with_dependencies(dependencies: BTreeMap<String, String>) -> Result<Self, RuntimeError> {
        Self::with_configuration(dependencies, None, false, false)
    }

    pub fn with_configuration(
        dependencies: BTreeMap<String, String>,
        cargo_lock: Option<String>,
        offline: bool,
        locked: bool,
    ) -> Result<Self, RuntimeError> {
        crate::compilation::validate_dependencies(&dependencies)?;
        let workdir = std::env::temp_dir().join(format!("mars-runtime-{}", Uuid::new_v4()));
        fs::create_dir(&workdir).map_err(|source| RuntimeError::Io {
            path: workdir.clone(),
            source,
        })?;
        Ok(Self {
            workdir,
            cells: Vec::new(),
            execution_count: 0,
            dependencies,
            cargo_lock,
            offline,
            locked,
        })
    }

    pub fn reset(&mut self) {
        self.cells.clear();
        self.execution_count = 0;
    }

    pub fn execution_count(&self) -> u64 {
        self.execution_count
    }

    pub fn cargo_lock(&self) -> Result<Option<String>, RuntimeError> {
        let path = self.workdir.join("Cargo.lock");
        match fs::read_to_string(path) {
            Ok(lock) => Ok(Some(lock)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(source) => Err(RuntimeError::Io {
                path: self.workdir.join("Cargo.lock"),
                source,
            }),
        }
    }
}

impl Drop for RustSession {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.workdir);
    }
}

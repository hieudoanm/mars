use mars_lsp::{CellSource, Dependencies, LspError, LspSession};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct LspSessions {
    sessions: Mutex<HashMap<String, LspSession>>,
    base_dir: PathBuf,
}

impl LspSessions {
    pub fn new() -> Self {
        let base_dir = std::env::temp_dir().join("mars-lsp");
        Self {
            sessions: Mutex::new(HashMap::new()),
            base_dir,
        }
    }

    fn with_session<T>(
        &self,
        session: &str,
        f: impl FnOnce(&LspSession) -> Result<T, String>,
    ) -> Result<T, String> {
        let sessions = self
            .sessions
            .lock()
            .map_err(|_| "lsp sessions lock poisoned".to_owned())?;
        let lsp = sessions
            .get(session)
            .ok_or_else(|| "lsp session not synced".to_owned())?;
        f(lsp)
    }

    pub fn sync(
        &self,
        session: &str,
        cells: Vec<CellSource>,
        dependencies: Dependencies,
    ) -> Result<(), String> {
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| "lsp sessions lock poisoned".to_owned())?;
        if !sessions.contains_key(session) {
            let lsp = LspSession::open(&self.base_dir, session).map_err(lsp_error)?;
            sessions.insert(session.to_owned(), lsp);
        }
        let lsp = sessions
            .get_mut(session)
            .ok_or_else(|| "lsp session unavailable".to_owned())?;
        lsp.sync(&cells, &dependencies).map_err(lsp_error)
    }

    pub fn hover(
        &self,
        session: &str,
        cell_id: String,
        line: u32,
        character: u32,
    ) -> Result<Option<serde_json::Value>, String> {
        self.with_session(session, |lsp| {
            lsp.hover(&cell_id, line, character).map_err(lsp_error)
        })
    }

    pub fn completion(
        &self,
        session: &str,
        cell_id: String,
        line: u32,
        character: u32,
    ) -> Result<Option<serde_json::Value>, String> {
        self.with_session(session, |lsp| {
            lsp.completion(&cell_id, line, character).map_err(lsp_error)
        })
    }

    pub fn signature_help(
        &self,
        session: &str,
        cell_id: String,
        line: u32,
        character: u32,
    ) -> Result<Option<serde_json::Value>, String> {
        self.with_session(session, |lsp| {
            lsp.signature_help(&cell_id, line, character)
                .map_err(lsp_error)
        })
    }

    pub fn definition(
        &self,
        session: &str,
        cell_id: String,
        line: u32,
        character: u32,
    ) -> Result<Option<mars_lsp::DefinitionTarget>, String> {
        self.with_session(session, |lsp| {
            lsp.definition(&cell_id, line, character).map_err(lsp_error)
        })
    }

    pub fn code_actions(
        &self,
        session: &str,
        cell_id: String,
        line: u32,
        character: u32,
    ) -> Result<Vec<mars_lsp::LspCodeAction>, String> {
        self.with_session(session, |lsp| {
            lsp.code_actions(&cell_id, line, character)
                .map_err(lsp_error)
        })
    }

    pub fn inlay_hints(&self, session: &str) -> Result<Vec<mars_lsp::CellInlayHint>, String> {
        self.with_session(session, |lsp| {
            lsp.inlay_hints().map_err(|error| error.to_string())
        })
    }

    pub fn semantic_tokens(
        &self,
        session: &str,
    ) -> Result<Vec<mars_lsp::CellSemanticToken>, String> {
        self.with_session(session, |lsp| {
            lsp.semantic_tokens().map_err(|error| error.to_string())
        })
    }

    pub fn diagnostics(&self, session: &str) -> Result<Vec<mars_lsp::CellDiagnostic>, String> {
        self.with_session(session, |lsp| Ok(lsp.poll_diagnostics()))
    }
}

fn lsp_error(error: LspError) -> String {
    error.to_string()
}

#[derive(Deserialize)]
pub struct SyncRequest {
    pub session: String,
    #[serde(default)]
    pub cells: Vec<CellSource>,
    #[serde(default)]
    pub dependencies: Dependencies,
}

#[derive(Deserialize)]
pub struct PositionRequest {
    pub session: String,
    pub cell_id: String,
    pub line: u32,
    pub character: u32,
}

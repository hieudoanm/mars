//! Per-notebook-session LSP state: a scratch Cargo project plus the line
//! mapping between notebook cells and the generated `src/main.rs`.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;

use crate::{LspClient, LspError};

/// A code cell as sent by the frontend for synchronization.
#[derive(Debug, Clone, Deserialize)]
pub struct CellSource {
    pub id: String,
    #[serde(default)]
    pub cell_type: String,
    #[serde(default)]
    pub source: String,
}

/// Notebook-level Cargo dependencies (`name -> version requirement`).
pub type Dependencies = HashMap<String, String>;

/// A diagnostic mapped back to a notebook cell.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CellDiagnostic {
    pub cell_id: String,
    pub line: u32,
    pub column: u32,
    pub severity: String,
    pub message: String,
}

#[doc(hidden)]
pub struct CellSpan {
    pub id: String,
    /// 0-based line in main.rs where this cell's source begins.
    pub start_line: u32,
    /// Number of lines the cell source occupies.
    pub line_count: u32,
}

/// A go-to-definition target mapped back to a notebook cell.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DefinitionTarget {
    pub cell_id: String,
    pub line: u32,
    pub character: u32,
}

/// An inlay hint mapped back to a notebook cell.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CellInlayHint {
    pub cell_id: String,
    pub line: u32,
    pub character: u32,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding_left: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding_right: Option<bool>,
}

/// A semantic token mapped back to a notebook cell. `token_type` is the
/// legend name (e.g. `function`, `keyword`) sent during initialize.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CellSemanticToken {
    pub cell_id: String,
    pub line: u32,
    pub character: u32,
    pub length: u32,
    pub token_type: String,
}

/// A single text edit mapped back to cell-local coordinates.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CellTextEdit {
    pub cell_id: String,
    pub start_line: u32,
    pub start_character: u32,
    pub end_line: u32,
    pub end_character: u32,
    pub new_text: String,
}

/// A code action (quick fix / assist) with edits mapped to notebook cells.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LspCodeAction {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    pub edits: Vec<CellTextEdit>,
}

/// Owns the rust-analyzer client and the generated document for one session.
pub struct LspSession {
    client: LspClient,
    spans: Vec<CellSpan>,
    version: i64,
    doc_line_count: u32,
    root: PathBuf,
}

/// Decode the LSP semantic-token delta encoding into absolute
/// (line, character, length, token-type-index) tuples. Returns an empty
/// iterator result when the data length is not a multiple of 5.
#[doc(hidden)]
pub fn decode_semantic_tokens(data: &[u32]) -> Vec<(u64, u64, u32, usize)> {
    if data.len() % 5 != 0 {
        return Vec::new();
    }
    let mut tokens = Vec::new();
    let mut line: u64 = 0;
    let mut character: u64 = 0;
    for chunk in data.chunks_exact(5) {
        line += chunk[0] as u64;
        if chunk[0] == 0 {
            character += chunk[1] as u64;
        } else {
            character = chunk[1] as u64;
        }
        tokens.push((line, character, chunk[2], chunk[3] as usize));
    }
    tokens
}

impl LspSession {
    /// Create (or reuse) the scratch project under `base_dir` and spawn
    /// rust-analyzer against it.
    pub fn open(base_dir: &Path, session_key: &str) -> Result<Self, LspError> {
        let root = base_dir.join(session_key);
        fs::create_dir_all(root.join("src")).map_err(|e| LspError::Spawn(e.to_string()))?;
        let manifest = root.join("Cargo.toml");
        if !manifest.exists() {
            fs::write(
                &manifest,
                "[package]\nname = \"mars-notebook\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n[dependencies]\n",
            )
            .map_err(|e| LspError::Spawn(e.to_string()))?;
        }
        let main = root.join("src/main.rs");
        if !main.exists() {
            fs::write(&main, "fn main() {}\n").map_err(|e| LspError::Spawn(e.to_string()))?;
        }
        let client = LspClient::spawn(&root)?;
        Ok(Self {
            client,
            spans: Vec::new(),
            version: 0,
            doc_line_count: 0,
            root,
        })
    }

    /// Regenerate Cargo.toml dependencies and the main.rs document, then push
    /// a full-document sync to rust-analyzer.
    pub fn sync(
        &mut self,
        cells: &[CellSource],
        dependencies: &Dependencies,
    ) -> Result<(), LspError> {
        self.version += 1;
        self.write_manifest(dependencies)?;
        let (document, spans) = build_document(cells);
        fs::write(self.root.join("src/main.rs"), &document)
            .map_err(|e| LspError::Spawn(e.to_string()))?;
        self.doc_line_count = document.lines().count() as u32;
        self.spans = spans;
        self.client.sync_document(&document, self.version)
    }

    fn write_manifest(&self, dependencies: &Dependencies) -> Result<(), LspError> {
        let mut manifest = String::from(
            "[package]\nname = \"mars-notebook\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n[dependencies]\n",
        );
        let mut entries: Vec<(&String, &String)> = dependencies.iter().collect();
        entries.sort();
        for (name, spec) in entries {
            manifest.push_str(&format!("{name} = \"{spec}\"\n"));
        }
        fs::write(self.root.join("Cargo.toml"), manifest)
            .map_err(|e| LspError::Spawn(e.to_string()))
    }

    fn map_position(&self, cell_id: &str, line: u32, character: u32) -> Option<(u32, u32)> {
        let span = self.spans.iter().find(|span| span.id == cell_id)?;
        if line >= span.line_count {
            return None;
        }
        Some((span.start_line + line, character))
    }

    fn map_diagnostics(&self) -> Vec<CellDiagnostic> {
        let mut mapped = Vec::new();
        for diagnostic in self.client.take_diagnostics() {
            let Some(range) = diagnostic.get("range") else {
                continue;
            };
            let Some(start) = range.get("start") else {
                continue;
            };
            let (Some(line), Some(character)) = (
                start.get("line").and_then(Value::as_u64),
                start.get("character").and_then(Value::as_u64),
            ) else {
                continue;
            };
            let Some(span) = self.spans.iter().find(|span| {
                line >= span.start_line as u64 && line < (span.start_line + span.line_count) as u64
            }) else {
                continue;
            };
            mapped.push(CellDiagnostic {
                cell_id: span.id.clone(),
                line: (line - span.start_line as u64) as u32,
                column: character as u32,
                severity: diagnostic
                    .get("severity")
                    .and_then(Value::as_i64)
                    .map(|s| match s {
                        1 => "error".to_owned(),
                        2 => "warning".to_owned(),
                        3 => "info".to_owned(),
                        4 => "hint".to_owned(),
                        _ => "diagnostic".to_owned(),
                    })
                    .unwrap_or_else(|| "diagnostic".to_owned()),
                message: diagnostic
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned(),
            });
        }
        mapped
    }

    pub fn hover(
        &self,
        cell_id: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, LspError> {
        let Some((line, character)) = self.map_position(cell_id, line, character) else {
            return Ok(None);
        };
        self.client.hover(line, character)
    }

    pub fn completion(
        &self,
        cell_id: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, LspError> {
        let Some((line, character)) = self.map_position(cell_id, line, character) else {
            return Ok(None);
        };
        self.client.completion(line, character)
    }

    pub fn signature_help(
        &self,
        cell_id: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, LspError> {
        let Some((line, character)) = self.map_position(cell_id, line, character) else {
            return Ok(None);
        };
        self.client.signature_help(line, character)
    }

    /// Resolve a definition request to a cell-local position. Returns `None`
    /// when the target is outside the generated document (e.g. std library).
    pub fn definition(
        &self,
        cell_id: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<DefinitionTarget>, LspError> {
        let Some((line, character)) = self.map_position(cell_id, line, character) else {
            return Ok(None);
        };
        let Some(result) = self.client.definition(line, character)? else {
            return Ok(None);
        };
        let location = first_location(&result);
        let Some(location) = location else {
            return Ok(None);
        };
        if location.0 != self.client.document_uri() {
            return Ok(None);
        }
        let (line, character) = location.1;
        let Some(span) = self.spans.iter().find(|span| {
            line >= span.start_line as u64 && line < (span.start_line + span.line_count) as u64
        }) else {
            return Ok(None);
        };
        Ok(Some(DefinitionTarget {
            cell_id: span.id.clone(),
            line: (line - span.start_line as u64) as u32,
            character: character as u32,
        }))
    }

    /// Map a document position back to cell-local coordinates.
    fn unmap_position(&self, line: u64, character: u64) -> Option<(String, u32, u32)> {
        let span = self.spans.iter().find(|span| {
            line >= span.start_line as u64 && line < (span.start_line + span.line_count) as u64
        })?;
        Some((
            span.id.clone(),
            (line - span.start_line as u64) as u32,
            character as u32,
        ))
    }

    /// Request inlay hints for the whole generated document and map each
    /// hint's position back to cell-local coordinates.
    pub fn inlay_hints(&self) -> Result<Vec<CellInlayHint>, LspError> {
        let Some(result) = self.client.inlay_hints(self.doc_line_count)? else {
            return Ok(Vec::new());
        };
        let Some(items) = result.as_array() else {
            return Ok(Vec::new());
        };
        let mut hints = Vec::new();
        for item in items {
            let position = item.get("position");
            let (Some(line), Some(character)) = (
                position.and_then(|p| p.get("line")).and_then(Value::as_u64),
                position
                    .and_then(|p| p.get("character"))
                    .and_then(Value::as_u64),
            ) else {
                continue;
            };
            let Some((cell_id, line, character)) = self.unmap_position(line, character) else {
                continue;
            };
            let label = match item.get("label") {
                Some(Value::String(text)) => text.clone(),
                Some(Value::Array(parts)) => parts
                    .iter()
                    .filter_map(|part| part.get("value").and_then(Value::as_str))
                    .collect::<String>(),
                _ => continue,
            };
            hints.push(CellInlayHint {
                cell_id,
                line,
                character,
                label,
                kind: item
                    .get("kind")
                    .and_then(Value::as_u64)
                    .map(|kind| match kind {
                        1 => "type".to_owned(),
                        2 => "parameter".to_owned(),
                        _ => "other".to_owned(),
                    }),
                padding_left: item.get("paddingLeft").and_then(Value::as_bool),
                padding_right: item.get("paddingRight").and_then(Value::as_bool),
            });
        }
        Ok(hints)
    }

    /// Request semantic tokens for the whole generated document and decode the
    /// LSP delta encoding into cell-local ranges.
    pub fn semantic_tokens(&self) -> Result<Vec<CellSemanticToken>, LspError> {
        let Some(result) = self.client.semantic_tokens()? else {
            return Ok(Vec::new());
        };
        let Some(data) = result.get("data").and_then(Value::as_array) else {
            return Ok(Vec::new());
        };
        let numbers: Vec<u32> = data
            .iter()
            .filter_map(Value::as_u64)
            .map(|v| v as u32)
            .collect();
        let mut tokens = Vec::new();
        for (line, character, length, type_index) in decode_semantic_tokens(&numbers) {
            let token_type = LspClient::SEMANTIC_TOKEN_TYPES
                .get(type_index)
                .map(|name| (*name).to_owned())
                .unwrap_or_else(|| "unknown".to_owned());
            let Some((cell_id, cell_line, cell_character)) = self.unmap_position(line, character)
            else {
                continue;
            };
            tokens.push(CellSemanticToken {
                cell_id,
                line: cell_line,
                character: cell_character,
                length,
                token_type,
            });
        }
        Ok(tokens)
    }

    /// Request quick fixes / assists at a cell-local position. Edits that
    /// fall outside the generated document or span multiple cells are dropped.
    pub fn code_actions(
        &self,
        cell_id: &str,
        line: u32,
        character: u32,
    ) -> Result<Vec<LspCodeAction>, LspError> {
        let Some((line, character)) = self.map_position(cell_id, line, character) else {
            return Ok(Vec::new());
        };
        let diagnostics = self.client.peek_diagnostics();
        // Quick fixes only apply when the request range intersects a
        // diagnostic, so widen a bare cursor position to the covering
        // diagnostic's range and scope the context to it.
        let covering = diagnostics
            .iter()
            .filter(|diagnostic| diagnostic_covers(diagnostic, line, character))
            .cloned()
            .collect::<Vec<_>>();
        let (start, end) = covering
            .first()
            .and_then(diagnostic_range)
            .unwrap_or(((line, character), (line, character)));
        let context = if covering.is_empty() {
            diagnostics.clone()
        } else {
            covering
        };
        let Some(result) = self
            .client
            .code_action(start.0, start.1, end.0, end.1, &context)?
        else {
            return Ok(Vec::new());
        };
        let Some(actions) = result.as_array() else {
            return Ok(Vec::new());
        };
        let mut mapped = Vec::new();
        for action in actions {
            let Some(edit) = action.get("edit") else {
                continue;
            };
            let mut edits = Vec::new();
            let mut valid = true;
            let change_sets: Vec<&Value> = edit
                .get("changes")
                .and_then(Value::as_object)
                .map(|map| map.values().collect())
                .unwrap_or_default();
            let text_edits: Vec<&Value> = if !change_sets.is_empty() {
                change_sets
                    .iter()
                    .flat_map(|set| set.as_array().into_iter().flatten())
                    .collect()
            } else if let Some(document_changes) =
                edit.get("documentChanges").and_then(Value::as_array)
            {
                let uri = self.client.document_uri().to_owned();
                document_changes
                    .iter()
                    .filter(|change| {
                        change
                            .get("textDocument")
                            .and_then(|doc| doc.get("uri"))
                            .and_then(Value::as_str)
                            == Some(uri.as_str())
                    })
                    .filter_map(|change| change.get("edits").and_then(Value::as_array))
                    .flatten()
                    .collect()
            } else {
                Vec::new()
            };
            for text_edit in text_edits {
                let range = text_edit.get("range");
                let new_text = text_edit.get("newText").and_then(Value::as_str);
                let (Some(range), Some(new_text)) = (range, new_text) else {
                    valid = false;
                    break;
                };
                let (Some(start), Some(end)) = (range.get("start"), range.get("end")) else {
                    valid = false;
                    break;
                };
                let (Some(sl), Some(sc), Some(el), Some(ec)) = (
                    start.get("line").and_then(Value::as_u64),
                    start.get("character").and_then(Value::as_u64),
                    end.get("line").and_then(Value::as_u64),
                    end.get("character").and_then(Value::as_u64),
                ) else {
                    valid = false;
                    break;
                };
                let (Some((sc_id, sline, schar)), Some((ec_id, eline, echar))) =
                    (self.unmap_position(sl, sc), self.unmap_position(el, ec))
                else {
                    valid = false;
                    break;
                };
                if sc_id != ec_id {
                    valid = false;
                    break;
                }
                edits.push(CellTextEdit {
                    cell_id: sc_id,
                    start_line: sline,
                    start_character: schar,
                    end_line: eline,
                    end_character: echar,
                    new_text: new_text.to_owned(),
                });
            }
            if !valid || edits.is_empty() {
                continue;
            }
            mapped.push(LspCodeAction {
                title: action
                    .get("title")
                    .and_then(Value::as_str)
                    .unwrap_or("Quick fix")
                    .to_owned(),
                kind: action
                    .get("kind")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                edits,
            });
        }
        Ok(mapped)
    }

    /// Drain newly published diagnostics mapped to cells.
    pub fn poll_diagnostics(&self) -> Vec<CellDiagnostic> {
        self.map_diagnostics()
    }
}

/// Extract (uri, line, character) from a definition result, which may be a
/// Location, an array of Locations, or a LocationLink/array of LocationLinks.
#[doc(hidden)]
pub fn first_location(result: &Value) -> Option<(String, (u64, u64))> {
    let pick = |value: &Value| -> Option<(String, (u64, u64))> {
        if let Some(uri) = value.get("uri").and_then(Value::as_str) {
            let range = value.get("range")?;
            let start = range.get("start")?;
            let line = start.get("line")?.as_u64()?;
            let character = start.get("character")?.as_u64()?;
            return Some((uri.to_owned(), (line, character)));
        }
        if let Some(uri) = value.get("targetUri").and_then(Value::as_str) {
            let range = value
                .get("targetSelectionRange")
                .or_else(|| value.get("targetRange"))?;
            let start = range.get("start")?;
            let line = start.get("line")?.as_u64()?;
            let character = start.get("character")?.as_u64()?;
            return Some((uri.to_owned(), (line, character)));
        }
        None
    };
    if let Some(items) = result.as_array() {
        return items.iter().find_map(pick);
    }
    pick(result)
}

/// Wrap cell sources in `fn main() { … }` and record each cell's line span.
/// `fn main` keeps top-level `let`/statements valid; nested items are legal.
#[doc(hidden)]
pub fn build_document(cells: &[CellSource]) -> (String, Vec<CellSpan>) {
    let mut document = String::from("fn main() {\n");
    let mut spans = Vec::new();
    for cell in cells.iter().filter(|cell| cell.cell_type == "code") {
        let start_line = document.matches('\n').count() as u32;
        // A trailing bare expression is valid Rust only as the final
        // expression of a block; inside the generated `fn main()` wrapper
        // it must be terminated or it breaks parsing for every following
        // cell. Appending `;` preserves line/column positions.
        let mut body = cell.source.clone();
        if mars_runtime::expression::split_final_expression(&body).is_some() {
            let insert_at = body.trim_end().len();
            body.insert(insert_at, ';');
        }
        document.push_str(&body);
        if !cell.source.ends_with('\n') {
            document.push('\n');
        }
        document.push('\n');
        let line_count = cell.source.matches('\n').count() as u32 + 1;
        spans.push(CellSpan {
            id: cell.id.clone(),
            start_line,
            line_count,
        });
    }
    document.push_str("}\n");
    (document, spans)
}
#[doc(hidden)]
pub fn diagnostic_range(diagnostic: &Value) -> Option<((u32, u32), (u32, u32))> {
    let range = diagnostic.get("range")?;
    let start = range.get("start")?;
    let end = range.get("end")?;
    let position = |value: &Value| {
        Some((
            u32::try_from(value.get("line")?.as_u64()?).ok()?,
            u32::try_from(value.get("character")?.as_u64()?).ok()?,
        ))
    };
    Some((position(start)?, position(end)?))
}

#[doc(hidden)]
pub fn diagnostic_covers(diagnostic: &Value, line: u32, character: u32) -> bool {
    let Some(((sl, sc), (el, ec))) = diagnostic_range(diagnostic) else {
        return false;
    };
    (line, character) >= (sl, sc) && (line, character) <= (el, ec)
}

use crate::error::DocumentError;
use crate::notebook::Notebook;
use mars_core::CellType;

pub fn validate(notebook: &Notebook) -> Result<(), DocumentError> {
    if notebook.format_version != crate::CURRENT_FORMAT_VERSION {
        return Err(DocumentError::UnsupportedFormatVersion(
            notebook.format_version,
        ));
    }
    let mut ids = std::collections::HashSet::with_capacity(notebook.cells.len());
    for cell in &notebook.cells {
        if cell.id.as_str().trim().is_empty() {
            return Err(DocumentError::EmptyCellId);
        }
        if !ids.insert(&cell.id) {
            return Err(DocumentError::DuplicateCellId(cell.id.to_string()));
        }
        if matches!(cell.cell_type, CellType::Markdown | CellType::Raw)
            && (cell.execution_count.is_some() || !cell.outputs.is_empty())
        {
            return Err(DocumentError::NonCodeExecutionData(cell.id.to_string()));
        }
    }
    Ok(())
}

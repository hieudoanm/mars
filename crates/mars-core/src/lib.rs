use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CellId(String);

impl CellId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn from_string(value: impl Into<String>) -> Result<Self, CoreError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(CoreError::InvalidCellId);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for CellId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for CellId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CellType {
    Markdown,
    Code,
    Raw,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Output {
    Text {
        text: String,
    },
    Error {
        name: String,
        message: String,
        traceback: Vec<String>,
    },
    Html {
        html: String,
    },
    Image {
        mime_type: String,
        data: String,
    },
    Table {
        columns: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    Latex {
        latex: String,
    },
    Chart {
        spec: serde_json::Value,
    },
    MimeBundle {
        data: serde_json::Map<String, serde_json::Value>,
        metadata: serde_json::Map<String, serde_json::Value>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    Idle,
    Running,
    Succeeded,
    Failed,
    Interrupted,
}

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("cell ID cannot be empty")]
    InvalidCellId,
}

/// Standard paper sizes defined by aspect ratio (ISO 216 and common US sizes).
///
/// Dimensions are in PostScript points (1 point = 1/72 inch). ISO 216 sizes
/// use the 1:√2 aspect ratio; each successive size halves the previous one by
/// folding along the short side.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PaperSize {
    A0,
    A1,
    A2,
    A3,
    #[default]
    A4,
    A5,
    A6,
    B4,
    B5,
    Letter,
    Legal,
    Tabloid,
}

impl PaperSize {
    /// Returns the width and height in points (portrait orientation).
    pub fn dimensions(self) -> (f64, f64) {
        match self {
            PaperSize::A0 => (2383.93, 3370.39),
            PaperSize::A1 => (1683.78, 2383.93),
            PaperSize::A2 => (1190.55, 1683.78),
            PaperSize::A3 => (841.89, 1190.55),
            PaperSize::A4 => (595.28, 841.89),
            PaperSize::A5 => (419.53, 595.28),
            PaperSize::A6 => (297.64, 419.53),
            PaperSize::B4 => (984.25, 1390.94),
            PaperSize::B5 => (696.93, 984.25),
            PaperSize::Letter => (612.0, 792.0),
            PaperSize::Legal => (612.0, 1008.0),
            PaperSize::Tabloid => (792.0, 1224.0),
        }
    }

    /// Returns the short-side-to-long-side aspect ratio.
    pub fn ratio(self) -> f64 {
        let (w, h) = self.dimensions();
        w / h
    }

    /// Returns `true` if the size follows the ISO 216 1:√2 ratio.
    pub fn is_iso216(self) -> bool {
        matches!(
            self,
            PaperSize::A0
                | PaperSize::A1
                | PaperSize::A2
                | PaperSize::A3
                | PaperSize::A4
                | PaperSize::A5
                | PaperSize::A6
                | PaperSize::B4
                | PaperSize::B5
        )
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "a0" => Ok(PaperSize::A0),
            "a1" => Ok(PaperSize::A1),
            "a2" => Ok(PaperSize::A2),
            "a3" => Ok(PaperSize::A3),
            "a4" => Ok(PaperSize::A4),
            "a5" => Ok(PaperSize::A5),
            "a6" => Ok(PaperSize::A6),
            "b4" => Ok(PaperSize::B4),
            "b5" => Ok(PaperSize::B5),
            "letter" => Ok(PaperSize::Letter),
            "legal" => Ok(PaperSize::Legal),
            "tabloid" => Ok(PaperSize::Tabloid),
            _ => Err(format!("unsupported paper size: {value}")),
        }
    }
}

impl PaperSize {
    /// Returns `true` if this is the default paper size (A4).
    pub fn is_default(&self) -> bool {
        *self == PaperSize::default()
    }
}

impl fmt::Display for PaperSize {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PaperSize::A0 => formatter.write_str("a0"),
            PaperSize::A1 => formatter.write_str("a1"),
            PaperSize::A2 => formatter.write_str("a2"),
            PaperSize::A3 => formatter.write_str("a3"),
            PaperSize::A4 => formatter.write_str("a4"),
            PaperSize::A5 => formatter.write_str("a5"),
            PaperSize::A6 => formatter.write_str("a6"),
            PaperSize::B4 => formatter.write_str("b4"),
            PaperSize::B5 => formatter.write_str("b5"),
            PaperSize::Letter => formatter.write_str("letter"),
            PaperSize::Legal => formatter.write_str("legal"),
            PaperSize::Tabloid => formatter.write_str("tabloid"),
        }
    }
}

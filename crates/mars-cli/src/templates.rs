//! Notebook templates and example discovery for `mars notebook new`.
//!
//! Built-in templates are small starter notebooks generated programmatically.
//! Examples are the `.mars` files shipped under `examples/mars/` in the
//! repository; they are discovered by walking up from the current directory
//! (or via `MARS_EXAMPLES_DIR`) so `--example` works both in the repo and
//! for source checkouts.

use mars_core::{CellId, CellType};
use mars_document::{Cell, Notebook};
use std::path::{Path, PathBuf};

struct TemplateSpec {
    name: &'static str,
    language: &'static str,
    title: &'static str,
    description: &'static str,
    cells: &'static [(&'static str, &'static str)],
}

const TEMPLATES: &[TemplateSpec] = &[
    TemplateSpec {
        name: "rust",
        language: "rust",
        title: "Rust notebook",
        description: "Rust starter: bindings, a function, and formatted output.",
        cells: &[
            (
                "markdown",
                "# Rust notebook\n\nCells share a persistent Rust session. Variables defined in one cell are\navailable in the next, and outputs render inline.\n\nTry editing the function below and re-running the cells.",
            ),
            (
                "code",
                "fn greet(name: &str) -> String {\n    format!(\"Hello, {name} from Mars!\")\n}\n\nlet message = greet(\"explorer\");",
            ),
            ("code", "println!(\"{message}\");\nmessage.len()"),
        ],
    },
    TemplateSpec {
        name: "python",
        language: "python",
        title: "Python notebook",
        description: "Python starter with pandas-style data exploration.",
        cells: &[
            (
                "markdown",
                "# Python notebook\n\nPython cells run in a persistent interpreter with matplotlib,\n`_repr_*` rich output, and interactive `input()` support.",
            ),
            (
                "code",
                "import statistics\n\ndata = [12.4, 15.1, 13.8, 16.2, 14.9, 15.7]\nmean = statistics.fmean(data)",
            ),
            (
                "code",
                "print(f\"mean = {mean:.2f}\")\n{'min': min(data), 'max': max(data), 'stdev': statistics.stdev(data)}",
            ),
        ],
    },
    TemplateSpec {
        name: "ml",
        language: "rust",
        title: "Machine learning notebook",
        description: "Data generation, a tiny model, and evaluation metrics.",
        cells: &[
            (
                "markdown",
                "# Machine learning notebook\n\nA compact workflow: generate data, fit a model, and inspect\nmetrics. Swap in `linfa`, `ndarray`, or `polars` for real datasets.",
            ),
            (
                "code",
                "// Deterministic toy dataset: y = 2x + 1 + noise\nlet xs: Vec<f64> = (0..50).map(|i| i as f64).collect();\nlet ys: Vec<f64> = xs.iter().map(|x| 2.0 * x + 1.0).collect();",
            ),
            (
                "code",
                "let n = xs.len() as f64;\nlet mean_x = xs.iter().sum::<f64>() / n;\nlet mean_y = ys.iter().sum::<f64>() / n;\nlet cov = xs.iter().zip(&ys).map(|(x, y)| (x - mean_x) * (y - mean_y)).sum::<f64>();\nlet var = xs.iter().map(|x| (x - mean_x).powi(2)).sum::<f64>();\nlet slope = cov / var;\nlet intercept = mean_y - slope * mean_x;",
            ),
            (
                "code",
                "println!(\"slope = {slope:.4}, intercept = {intercept:.4}\");\nlet prediction = slope * 10.0 + intercept;\nprintln!(\"prediction at x=10: {prediction:.2}\");",
            ),
        ],
    },
    TemplateSpec {
        name: "dashboard",
        language: "rust",
        title: "Dashboard notebook",
        description: "Metrics summary with chart-ready outputs.",
        cells: &[
            (
                "markdown",
                "# Dashboard notebook\n\nCollect metrics into a summary table and render charts with\n`mars_chart` or your plotting library of choice.",
            ),
            (
                "code",
                "let metrics = [\n    (\"requests\", 1284.0),\n    (\"errors\", 12.0),\n    (\"latency_p95_ms\", 184.0),\n];\nfor (name, value) in metrics {\n    println!(\"{name}: {value}\");\n}",
            ),
            (
                "markdown",
                "## Next steps\n\n- Replace the sample metrics with live data.\n- Use `mars_table` for formatted tables or `plotters` for SVG charts.\n- Export with `mars notebook export --format html` to share.",
            ),
        ],
    },
];

pub fn template_names() -> Vec<&'static str> {
    TEMPLATES.iter().map(|spec| spec.name).collect()
}

fn template_spec(name: &str) -> Option<&'static TemplateSpec> {
    TEMPLATES.iter().find(|spec| spec.name == name)
}

/// Build a notebook from a built-in template.
pub fn notebook_from_template(name: &str) -> Result<Notebook, String> {
    let spec = template_spec(name).ok_or_else(|| {
        format!(
            "unknown template: {name} (available: {})",
            template_names().join(", ")
        )
    })?;
    let mut notebook = Notebook::new();
    notebook.metadata.title = Some(spec.title.to_owned());
    notebook.metadata.language = spec.language.to_owned();
    for (index, (cell_type, source)) in spec.cells.iter().enumerate() {
        let cell_type = match *cell_type {
            "markdown" => CellType::Markdown,
            "raw" => CellType::Raw,
            _ => CellType::Code,
        };
        let is_code = cell_type == CellType::Code;
        let mut cell = Cell::new(cell_type, *source);
        if is_code {
            cell.id = CellId::from_string(format!("{name}-{index}"))
                .map_err(crate::util::error_string)?;
        }
        notebook.add_cell(cell).map_err(crate::util::error_string)?;
    }
    Ok(notebook)
}

/// Directories that may contain `examples/mars/**/*.mars`.
fn example_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(dir) = std::env::var("MARS_EXAMPLES_DIR") {
        roots.push(PathBuf::from(dir));
    }
    if let Ok(cwd) = std::env::current_dir() {
        for ancestor in cwd.ancestors() {
            roots.push(ancestor.join("examples/mars"));
            roots.push(ancestor.join("mars/examples/mars"));
        }
    }
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        roots.push(dir.join("../examples/mars"));
        roots.push(dir.join("../../examples/mars"));
    }
    roots
}

/// Discover example notebooks as `(name, path)` pairs. `name` is the
/// repo-relative path without the `.mars` extension, e.g.
/// `rust/01-getting-started/getting-started`.
pub fn list_examples() -> Vec<(String, PathBuf)> {
    let mut examples = Vec::new();
    for root in example_roots() {
        if !root.is_dir() {
            continue;
        }
        collect_examples(&root, &root, &mut examples);
    }
    examples.sort();
    examples.dedup_by(|a, b| a.0 == b.0);
    examples
}

fn collect_examples(root: &Path, dir: &Path, out: &mut Vec<(String, PathBuf)>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_examples(root, &path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("mars") {
            if let Ok(relative) = path.strip_prefix(root) {
                let name = relative
                    .with_extension("")
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join("/");
                out.push((name, path));
            }
        }
    }
}

/// Load an example notebook by name.
pub fn example_notebook(name: &str) -> Result<Notebook, String> {
    let examples = list_examples();
    let matches: Vec<&(String, PathBuf)> = examples
        .iter()
        .filter(|(example_name, _)| {
            example_name == name
                || example_name.ends_with(&format!("/{name}"))
                || Path::new(example_name)
                    .file_name()
                    .and_then(|f| f.to_str())
                    .is_some_and(|f| f == name)
        })
        .collect();
    match matches.as_slice() {
        [] => Err(format!(
            "unknown example: {name} (run `mars notebook templates` to list)"
        )),
        [(_, path)] => mars_document::load(path).map_err(crate::util::error_string),
        many => Err(format!(
            "ambiguous example: {name} matches {} — use a longer path",
            many.iter()
                .map(|(n, _)| n.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

/// Human-readable listing for `mars notebook templates`.
pub fn list_templates_text() -> String {
    let mut out = String::from("Built-in templates:\n");
    for spec in TEMPLATES {
        out.push_str(&format!(
            "  {:<12} {:<8} {}\n",
            spec.name, spec.language, spec.description
        ));
    }
    let examples = list_examples();
    if examples.is_empty() {
        out.push_str(
            "\nNo example notebooks found (set MARS_EXAMPLES_DIR or run from the repo).\n",
        );
    } else {
        out.push_str(&format!("\nExample notebooks ({}):\n", examples.len()));
        for (name, _) in &examples {
            out.push_str(&format!("  {name}\n"));
        }
    }
    out
}

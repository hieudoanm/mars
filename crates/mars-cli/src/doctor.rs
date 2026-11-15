//! `mars notebook doctor` — runtime/environment diagnostics.
//!
//! Probes the external tools each language runtime needs and reports what is
//! usable on this machine. Pass a `.mars` path to check only the languages
//! that notebook actually uses; without one, all supported runtimes are
//! probed. Missing runtimes for a requested notebook are an error so the
//! command is usable in scripts and CI.

use mars_core::CellType;
use mars_document::{Notebook, load};
use std::process::Command;

struct Probe {
    language: &'static str,
    label: &'static str,
    binaries: &'static [&'static str],
    install_hint: &'static str,
}

const PROBES: &[Probe] = &[
    Probe {
        language: "rust",
        label: "Rust (cargo/rustc)",
        binaries: &["cargo", "rustc"],
        install_hint: "install Rust from https://rustup.rs",
    },
    Probe {
        language: "python",
        label: "Python (python3)",
        binaries: &["python3", "python"],
        install_hint: "install Python 3.10+ and ensure `python3` is on PATH",
    },
    Probe {
        language: "r",
        label: "R (Rscript)",
        binaries: &["Rscript"],
        install_hint: "install R from https://cran.r-project.org",
    },
    Probe {
        language: "julia",
        label: "Julia",
        binaries: &["julia"],
        install_hint: "install Julia from https://julialang.org",
    },
    Probe {
        language: "cpp",
        label: "C++ (clang++/g++)",
        binaries: &["clang++", "g++", "c++"],
        install_hint: "install a C++ toolchain (Xcode CLT, gcc, or clang)",
    },
];

pub fn run_doctor(notebook_path: Option<&str>) -> Result<(), String> {
    let needed = match notebook_path {
        Some(path) => {
            let notebook = load(path).map_err(|error| error.to_string())?;
            notebook_languages(&notebook)
        }
        None => PROBES.iter().map(|probe| probe.language).collect(),
    };

    println!("mars {}", env!("CARGO_PKG_VERSION"));
    if let Some(path) = notebook_path {
        println!("notebook: {path}");
    }
    println!();

    let mut missing: Vec<String> = Vec::new();
    for probe in PROBES
        .iter()
        .filter(|probe| needed.contains(&probe.language))
    {
        match probe_version(probe.binaries) {
            Some(version) => println!("ok      {:<22} {}", probe.label, version),
            None => {
                println!("missing {:<22} {}", probe.label, probe.install_hint);
                missing.push(probe.language.to_owned());
            }
        }
    }

    if !needed.is_empty()
        && needed
            .iter()
            .all(|lang| !PROBES.iter().any(|p| p.language == *lang))
    {
        println!("\nnote: no external runtime required for this notebook's languages");
    }
    if missing.is_empty() {
        println!("\nAll required runtimes are available.");
        Ok(())
    } else if notebook_path.is_some() {
        Err(format!(
            "missing runtimes for notebook languages: {}",
            missing.join(", ")
        ))
    } else {
        println!("\nSome runtimes are missing (install hints above).");
        Ok(())
    }
}

fn notebook_languages(notebook: &Notebook) -> Vec<&'static str> {
    let mut languages: Vec<&'static str> = Vec::new();
    let mut push = |language: &str| {
        if let Some(probe) = PROBES.iter().find(|probe| probe.language == language)
            && !languages.contains(&probe.language)
        {
            languages.push(probe.language);
        }
    };
    push(&notebook.metadata.language);
    for cell in &notebook.cells {
        if cell.cell_type == CellType::Code
            && let Some(language) = cell
                .metadata
                .get("language")
                .and_then(|value| value.as_str())
        {
            push(language);
        }
    }
    languages
}

/// Returns the `--version` output of the first probe binary found on PATH.
fn probe_version(binaries: &[&str]) -> Option<String> {
    for binary in binaries {
        let Ok(output) = Command::new(binary).arg("--version").output() else {
            continue;
        };
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            let first = text.lines().next().unwrap_or_default().trim();
            return Some(if first.is_empty() {
                (*binary).to_owned()
            } else {
                first.to_owned()
            });
        }
    }
    None
}

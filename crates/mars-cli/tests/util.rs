use mars::util::{default_export_path, default_ipynb_path, default_mars_path, usage};
use std::path::Path;

#[test]
fn default_paths_replace_extensions() {
    assert_eq!(default_mars_path("notes.txt"), Path::new("notes.mars"));
    assert_eq!(default_mars_path("dir/demo"), Path::new("dir/demo.mars"));
    assert_eq!(
        default_ipynb_path("dir/demo.mars"),
        Path::new("dir/demo.ipynb")
    );
}

#[test]
fn default_export_paths_cover_formats() {
    assert_eq!(
        default_export_path("book.mars", "html"),
        Path::new("book.html")
    );
    assert_eq!(default_export_path("book.mars", "md"), Path::new("book.md"));
    assert_eq!(
        default_export_path("book.mars", "markdown"),
        Path::new("book.md")
    );
    assert_eq!(
        default_export_path("book.mars", "pdf"),
        Path::new("book.pdf")
    );
    assert_eq!(
        default_export_path("book.mars", "ipynb"),
        Path::new("book.ipynb")
    );
    assert_eq!(
        default_export_path("book.mars", "unknown"),
        Path::new("book.ipynb")
    );
}

#[test]
fn usage_mentions_all_command_groups() {
    let usage = usage();
    for fragment in [
        "mars new",
        "mars open",
        "mars inspect",
        "mars run",
        "mars import",
        "mars export",
        "mars kernel",
        "mars mcp serve",
        "mars tools",
    ] {
        assert!(usage.contains(fragment), "missing {fragment}");
    }
}

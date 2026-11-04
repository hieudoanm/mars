pub mod fence;
#[allow(clippy::module_inception)]
pub mod markdown;
pub mod output;

pub use self::markdown::render_markdown;
pub use self::markdown::render_markdown_notebook;

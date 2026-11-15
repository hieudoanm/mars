//! Slideshow/present-mode HTML export.
//!
//! Slides are split at Markdown cells that open with a level-1 or level-2
//! heading; everything before the first such heading becomes the title
//! slide. Cells keep their rendered outputs, so a deck is a static,
//! self-contained artifact — no kernel required to present.

use crate::html::render_cell_html;
use mars_core::CellType;
use mars_document::{Cell, Notebook};

/// Groups cells into slides. A Markdown cell whose first heading is `#` or
/// `##` starts a new slide; all other cells attach to the current slide.
fn slides(notebook: &Notebook) -> Vec<Vec<&Cell>> {
    let mut groups: Vec<Vec<&Cell>> = vec![Vec::new()];
    for cell in &notebook.cells {
        let starts_slide = cell.cell_type == CellType::Markdown
            && cell
                .source
                .lines()
                .find(|line| !line.trim().is_empty())
                .is_some_and(|line| {
                    let trimmed = line.trim_start();
                    trimmed.starts_with("# ") || trimmed.starts_with("## ")
                });
        if starts_slide && !groups.last().is_some_and(Vec::is_empty) {
            groups.push(Vec::new());
        }
        if let Some(group) = groups.last_mut() {
            group.push(cell);
        }
    }
    groups.retain(|group| !group.is_empty());
    if groups.is_empty() {
        groups.push(Vec::new());
    }
    groups
}

/// Renders the notebook as a self-contained slideshow HTML document.
pub fn render_slides_notebook(notebook: &Notebook) -> String {
    let title = notebook
        .metadata
        .title
        .as_deref()
        .unwrap_or("Project Mars notebook");
    let groups = slides(notebook);
    let mut sections = String::new();
    for (index, group) in groups.iter().enumerate() {
        let mut cells = String::new();
        for cell in group {
            cells.push_str(&render_cell_html(cell));
        }
        sections.push_str(&format!(
            "<section class=\"slide{}\" data-slide=\"{index}\">{cells}</section>",
            if index == 0 { " title-slide" } else { "" }
        ));
    }

    let css = r#"
:root { color-scheme: dark; font-family: system-ui, sans-serif; background: #0c0c0c; color: #e5e5e5; }
* { box-sizing: border-box; }
body { margin: 0; min-height: 100vh; display: flex; align-items: center; justify-content: center; }
.slide { display: none; width: min(1100px, 92vw); max-height: 88vh; overflow-y: auto; padding: 3rem 2rem; }
.slide.active { display: block; }
.title-slide h1, .slide h1, .slide h2 { color: #fb923c; line-height: 1.2; }
.slide h1 { font-size: 2.4rem; } .slide h2 { font-size: 1.9rem; }
.cell { margin: 1.2rem 0; }
.cell-label { color: #f97316; font-size: .85rem; margin-bottom: .35rem; }
pre { overflow-x: auto; border-radius: .5rem; padding: 1rem; }
.code { background: #1c1c1c; border: 1px solid #333; border-left: 4px solid #f97316; }
.raw { background: #1c1c1c; border: 1px solid #333; border-left: 4px solid #a3a3a3; }
.code code, .raw code { font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace; font-size: .9rem; line-height: 1.5; }
.output-text { background: #1f2937; border: 1px solid #374151; border-left: 4px solid #3b82f6; }
.output-error { background: #1c1c1c; border: 1px solid #333; border-left: 4px solid #ef4444; color: #fecaca; }
table { border-collapse: collapse; } th, td { border: 1px solid #525252; padding: .4rem .7rem; }
img { max-width: 100%; } a { color: #fb923c; }
.math { color: #fbbf24; font-family: serif; font-size: 1.15rem; }
.chart { background: #202020; border: 1px solid #404040; padding: .5rem; }
.math-text { display: inline-block; padding: .6rem 1rem; border-radius: .35rem; background: #202020; font-family: Georgia, serif; font-size: 1.2rem; }
.math-frac { display: inline-flex; flex-direction: column; vertical-align: middle; text-align: center; line-height: 1.1; margin: 0 .15rem; }
.math-frac > span:first-child { border-bottom: 1px solid currentColor; padding: 0 .2rem .1rem; }
.math-frac > span:last-child { padding: .1rem .2rem 0; }
.chart-svg { width: 100%; max-width: 720px; height: 260px; }
.hud { position: fixed; right: 1.2rem; bottom: 1rem; display: flex; gap: .6rem; align-items: center; color: #a3a3a3; font-size: .85rem; }
.hud button { background: #262626; color: #e5e5e5; border: 1px solid #404040; border-radius: .4rem; padding: .3rem .8rem; cursor: pointer; font: inherit; }
.hud button:hover { background: #333; }
.hud button:focus-visible { outline: 2px solid #fb923c; outline-offset: 2px; }
@media print {
  body { display: block; background: #fff; color: #111; }
  .slide { display: block; width: auto; max-height: none; overflow: visible; padding: 0; page-break-after: always; }
  .hud { display: none; }
  .code, .raw, .output-text, .output-error, .chart, .math-text { background: #f5f5f5 !important; color: #111 !important; border-color: #ccc !important; }
}
@media (prefers-reduced-motion: reduce) { * { scroll-behavior: auto !important; } }
"#;

    let script = r#"
(function () {
  var slides = Array.prototype.slice.call(document.querySelectorAll('.slide'));
  var index = 0;
  var counter = document.getElementById('counter');
  function show(next) {
    index = Math.max(0, Math.min(slides.length - 1, next));
    slides.forEach(function (slide, i) { slide.classList.toggle('active', i === index); });
    if (counter) counter.textContent = (index + 1) + ' / ' + slides.length;
  }
  document.getElementById('prev').addEventListener('click', function () { show(index - 1); });
  document.getElementById('next').addEventListener('click', function () { show(index + 1); });
  document.addEventListener('keydown', function (event) {
    if (event.key === 'ArrowRight' || event.key === ' ' || event.key === 'PageDown') { event.preventDefault(); show(index + 1); }
    if (event.key === 'ArrowLeft' || event.key === 'PageUp') { event.preventDefault(); show(index - 1); }
    if (event.key === 'Home') show(0);
    if (event.key === 'End') show(slides.length - 1);
  });
  show(0);
})();
"#;

    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{}</title>
<style>{css}</style>
</head>
<body>
{sections}
<div class="hud">
  <button id="prev" type="button" aria-label="Previous slide">← Prev</button>
  <span id="counter"></span>
  <button id="next" type="button" aria-label="Next slide">Next →</button>
</div>
<script>{script}</script>
</body>
</html>
"#,
        crate::util::escape_html(title),
        css = css,
        sections = sections,
        script = script,
    )
}

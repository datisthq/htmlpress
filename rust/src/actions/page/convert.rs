use crate::actions::page::read::create_output_path;

/// Where a page's Markdown sits beside it: `guides/build.html` → `guides/build/index.md`.
pub(crate) fn create_markdown_path(path: &str) -> String {
    let output = create_output_path(path);
    let stem = output.strip_suffix(".html").unwrap_or(&output);
    format!("{stem}.md")
}

#[cfg(test)]
#[path = "convert_unit.rs"]
mod unit;

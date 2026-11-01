use serde_json::json;

use super::create_pipeline;
use crate::actions::page::read::{finish_page, read_source};
use crate::models::config::{PluginEntry, PluginOptions};
use crate::models::pipeline::PipelineOptions;

#[test]
fn runs_the_project_html_plugins() -> Result<(), String> {
    let options = PipelineOptions {
        plugins: vec![PluginEntry::Options(PluginOptions {
            name: "html.externalLinks".to_owned(),
            options: Some(json!({ "target": "_blank" })),
        })],
        ..PipelineOptions::default()
    };
    let html = "<p><a href=\"https://example.com\">Out</a> <a href=\"/in\">In</a></p>";
    let mut page = read_source("a.html", html).map_err(|e| e.to_string())?;
    finish_page(&mut page, &create_pipeline(&options)?).map_err(|e| e.to_string())?;
    assert!(
        page.content.contains("target=\"_blank\""),
        "{}",
        page.content
    );
    assert!(
        page.content.contains("<a href=\"/in\">In</a>"),
        "{}",
        page.content
    );
    Ok(())
}

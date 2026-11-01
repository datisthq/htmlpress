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
    finish_page(&mut page, &mut create_pipeline(&options)?).map_err(|e| e.to_string())?;
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

#[test]
fn merges_a_project_plugin_into_the_one_htmlpress_runs() -> Result<(), String> {
    let options = PipelineOptions {
        plugins: vec![PluginEntry::Options(PluginOptions {
            name: "html.slug".to_owned(),
            options: Some(json!({ "prefix": "x-" })),
        })],
        ..PipelineOptions::default()
    };
    let html = "<h2>Usage</h2><h2 id=\"own\">Own</h2>";
    let mut page = read_source("a.html", html).map_err(|e| e.to_string())?;
    finish_page(&mut page, &mut create_pipeline(&options)?).map_err(|e| e.to_string())?;
    assert!(
        page.content.contains("<h2 id=\"x-usage\">"),
        "{}",
        page.content
    );
    assert!(page.content.contains("<h2 id=\"own\">"), "{}", page.content);
    Ok(())
}

#[test]
fn names_the_hook_of_a_plugin_that_does_not_fit() {
    let options = |name: &str| PipelineOptions {
        plugins: vec![PluginEntry::Name(name.to_owned())],
        ..PipelineOptions::default()
    };
    for (name, reason) in [
        ("html.nope", "plugins.html: no plugin named `html.nope`"),
        ("md.toHtml", "plugins.html: "),
    ] {
        match create_pipeline(&options(name)) {
            Ok(_) => panic!("{name} fit"),
            Err(message) => assert!(message.starts_with(reason), "{name}: {message}"),
        }
    }
}

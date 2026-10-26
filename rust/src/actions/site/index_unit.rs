use std::path::PathBuf;

use super::*;
use crate::actions::page::read::read_page;
use crate::models::config::Config;

#[test]
fn splits_each_page_at_its_headings() -> crate::Result<()> {
    let site = Site {
        root: PathBuf::from("."),
        config: Config {
            base: Some("/repo/".to_owned()),
            ..Config::default()
        },
        title: "Docs".to_owned(),
        pages: vec![
            read_page(
                "guide.html",
                "<title>Guide</title><h1>Guide</h1><p>Start  here &amp; now.</p>\
                 <h2>Install</h2><p>Run it.</p><pre><code>cargo install</code></pre>\
                 <script>skip()</script><h3 id=\"flags\">Flags</h3><p>Use <code>-p</code>.</p>",
            )?,
            read_page("404.html", "<h1>Lost</h1>")?,
        ],
        files: Default::default(),
        assets: Default::default(),
        skills: vec![],
    };
    let index: serde_json::Value = serde_json::from_str(&index_site(&site)?)
        .map_err(|error| Error::Usage(error.to_string()))?;
    let pages = index["pages"].as_array().cloned().unwrap_or_default();
    assert_eq!(pages.len(), 1);
    assert_eq!(pages[0]["url"], "/repo/guide.html");
    let chunks = pages[0]["chunks"].as_array().cloned().unwrap_or_default();
    assert_eq!(chunks[0]["text"], "Start here & now.");
    assert!(chunks[0]["heading"].is_null());
    assert_eq!(chunks[1]["heading"], "Install");
    assert_eq!(chunks[1]["id"], "install");
    assert_eq!(chunks[1]["text"], "Run it. cargo install");
    assert_eq!(chunks[2]["id"], "flags");
    assert_eq!(chunks[2]["text"], "Use -p.");
    Ok(())
}

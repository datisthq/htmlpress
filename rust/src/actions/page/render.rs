use minijinja::value::Serde;
use minijinja::{AutoEscape, Environment, ErrorKind, Output, State, Value, context};
use serde::Serialize;

use crate::models::error::Error;
use crate::models::page::Page;
use crate::models::site::Site;
use crate::settings::{ASSETS_DIR, LAYOUT, RELOAD_PATH};

/// Wrap one page of the site in the layout: the project's escape, else the built-in one.
pub(crate) fn render_page(
    site: &Site,
    page: &Page,
    reload: bool,
) -> crate::Result<String> {
    let mut environment = Environment::new();
    environment.set_formatter(format_value);
    let layout = site.layout.as_deref().unwrap_or(LAYOUT);
    environment
        .add_template("layout.html", layout)
        .map_err(|error| Error::page(&page.path)(error.to_string()))?;
    let template = environment
        .get_template("layout.html")
        .map_err(|error| Error::page(&page.path)(error.to_string()))?;
    let nav: Vec<NavItem> = site
        .pages
        .iter()
        .map(|item| NavItem {
            title: &item.title,
            url: &item.url,
            current: item.path == page.path,
        })
        .collect();
    let description = page
        .description
        .as_ref()
        .or(site.config.description.as_ref());
    template
        .render(context! {
            site => context! { title => &site.title, url => &site.config.site },
            page => Serde(page),
            description => description,
            nav => Serde(&nav),
            assets => format!("/{ASSETS_DIR}"),
            reload => reload.then_some(RELOAD_PATH),
        })
        .map_err(|error| Error::page(&page.path)(error.to_string()))
}

// minijinja's HTML escape also turns `/` into `&#x2f;`, which garbles every URL in the
// output for a reader; only these five characters need escaping in text and attributes.
fn format_value(
    out: &mut Output,
    state: &mut State,
    value: &Value,
) -> Result<(), minijinja::Error> {
    let text = value.as_str().filter(|_| !value.is_safe());
    let Some(text) = text.filter(|_| matches!(state.auto_escape(), AutoEscape::Html))
    else {
        return minijinja::escape_formatter(out, state, value);
    };
    let mut escaped = String::with_capacity(text.len());
    for char in text.chars() {
        match char {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(char),
        }
    }
    out.write_str(&escaped).map_err(|_| {
        minijinja::Error::new(ErrorKind::WriteFailure, "cannot write the page")
    })
}

#[derive(Serialize)]
struct NavItem<'a> {
    title: &'a str,
    url: &'a str,
    current: bool,
}

#[cfg(test)]
#[path = "render_unit.rs"]
mod unit;

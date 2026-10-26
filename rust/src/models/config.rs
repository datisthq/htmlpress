use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// The project's `htmlpress.yaml`, shaped like Livemark's config. Every key is
/// optional.
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct Config {
    /// The public URL the site is served at, such as `https://htmlpress.dev`.
    pub site: Option<String>,
    /// The sub-path the site is served under, such as `/repo/` on GitHub Pages; every
    /// root-relative URL in the build gets it as a prefix.
    pub base: Option<String>,
    /// The site icon's URL, such as `/favicon.svg`; defaults to the logo.
    pub favicon: Option<String>,
    /// An image's URL shown before the site title in the brand.
    pub logo: Option<String>,
    /// The site title in the header and after each page title; defaults to the folder name.
    pub title: Option<String>,
    /// The site description, used for pages without one of their own.
    pub description: Option<String>,
    /// HTML for an announcement bar above the header.
    pub banner: Option<String>,
    /// Links in the header or the sidebar, such as the project's repository.
    pub links: Vec<Link>,
    /// Parts of the site with a navigation of their own, each owning the pages under its
    /// URL prefix; shown as tabs in the header or a list in the sidebar.
    pub sections: Vec<Section>,
    /// Tags added to every page's `<head>`.
    pub head: Head,
    /// How automated systems may use the content, written to `robots.txt`.
    pub content_signals: ContentSignals,
    /// Globs of the `.html` files that become pages, relative to the project; one
    /// pattern or a list. Empty means `**/*.html`. Other `.html` files are copied as-is.
    #[serde(deserialize_with = "read_patterns")]
    pub include: Vec<String>,
    /// Globs of files left out of the build entirely, pages or not.
    #[serde(deserialize_with = "read_patterns")]
    pub exclude: Vec<String>,
    /// Globs of `SKILL.md` files, relative to the project, published for agents under
    /// `.well-known/agent-skills/`; one pattern or a list.
    #[serde(deserialize_with = "read_patterns")]
    pub skills: Vec<String>,
    /// A URL each page's source path is appended to for its "Edit this page" link, such
    /// as `https://github.com/owner/repo/edit/main/docs/`.
    pub edit: Option<String>,
}

fn read_patterns<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<String>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Patterns {
        One(String),
        Many(Vec<String>),
    }
    Ok(match Patterns::deserialize(deserializer)? {
        Patterns::One(pattern) => vec![pattern],
        Patterns::Many(patterns) => patterns,
    })
}

impl Config {
    /// The base without its trailing slash, `/repo`, or empty when the site is at the root.
    pub(crate) fn read_base(&self) -> String {
        let base = self.base.as_deref().unwrap_or_default().trim_matches('/');
        match base.is_empty() {
            true => String::new(),
            false => format!("/{base}"),
        }
    }

    /// `site` and the base, without a trailing slash: what a page's URL follows to be
    /// absolute. None without `site`, since an absolute URL needs a host.
    pub(crate) fn read_origin(&self) -> Option<String> {
        let site = self.site.as_deref()?.trim_end_matches('/');
        Some(format!("{site}{}", self.read_base()))
    }
}

/// One link from the config.
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Link {
    pub url: String,
    pub title: String,
    /// A built-in Lucide icon's name, shown before the title.
    pub icon: Option<String>,
    #[serde(default)]
    pub position: Position,
}

/// One section from the config: `type: article` (the default) owns the pages under
/// `prefix`; `type: custom` is a link to `url`.
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Section {
    pub title: String,
    /// A built-in Lucide icon's name, shown before the title.
    pub icon: Option<String>,
    #[serde(default, rename = "type")]
    pub kind: SectionKind,
    /// The URL prefix its pages share, such as `/guides/`; `article` sections only.
    pub prefix: Option<String>,
    /// Where a `custom` section leads.
    pub url: Option<String>,
    #[serde(default)]
    pub position: Position,
    /// Replaces the site title in the brand while one of its pages is open.
    pub site_title: Option<String>,
    /// Replaces the site description for its pages without one of their own.
    pub site_description: Option<String>,
    /// Where the brand links while one of its pages is open, instead of `/`.
    pub site_link: Option<String>,
}

/// What a [`Section`] is.
#[derive(Debug, Clone, Copy, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SectionKind {
    #[default]
    Article,
    Custom,
}

/// Where a [`Link`] or a [`Section`] is shown.
#[derive(Debug, Clone, Copy, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Position {
    #[default]
    Header,
    Sidebar,
}

/// Site-wide `<head>` tags, each a map of attribute names to values, written after the
/// built-in ones. A meta sharing a built-in one's `name` or `property` replaces it.
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Head {
    pub meta: Vec<BTreeMap<String, String>>,
    pub links: Vec<BTreeMap<String, String>>,
    /// A script's `children` is its inline source rather than an attribute.
    pub scripts: Vec<BTreeMap<String, String>>,
}

/// The `Content-Signal` line of `robots.txt`; every signal defaults to allowed.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct ContentSignals {
    pub search: bool,
    pub ai_input: bool,
    pub ai_train: bool,
}

impl Default for ContentSignals {
    fn default() -> Self {
        Self {
            search: true,
            ai_input: true,
            ai_train: true,
        }
    }
}

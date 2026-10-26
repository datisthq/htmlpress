use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::models::language::{Language, Localized};
use crate::models::page::SourceAction;

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
    pub title: Option<Localized>,
    /// The site description, used for pages without one of their own.
    pub description: Option<Localized>,
    /// HTML for an announcement bar above the header.
    pub banner: Option<Localized>,
    /// Show the trail of folders above each page's content; defaults to true, and a
    /// page's `<meta name="breadcrumbs">` overrides it.
    pub breadcrumbs: Option<bool>,
    /// The languages the site is published in, the default first; one English without.
    pub languages: Vec<Language>,
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
    /// Page metadata set from the config, for files whose `<head>` should not carry it,
    /// such as a `README.md` published as a page.
    pub patches: Vec<Patch>,
    /// The theme code blocks take in light mode, a Shiki theme name; `catppuccin-latte`.
    pub code_theme_light: Option<String>,
    /// The theme code blocks take in dark mode; `catppuccin-mocha`.
    pub code_theme_dark: Option<String>,
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

    /// The light and dark code themes, Livemark's defaults when unset.
    pub(crate) fn read_code_themes(&self) -> (&str, &str) {
        let light = self
            .code_theme_light
            .as_deref()
            .unwrap_or("catppuccin-latte");
        let dark = self
            .code_theme_dark
            .as_deref()
            .unwrap_or("catppuccin-mocha");
        (light, dark)
    }

    /// `site` and the base, without a trailing slash: what a page's URL follows to be
    /// absolute. None without `site`, since an absolute URL needs a host.
    pub(crate) fn read_origin(&self) -> Option<String> {
        let site = self.site.as_deref()?.trim_end_matches('/');
        Some(format!("{site}{}", self.read_base()))
    }

    /// A copy with every [`Localized`] text in one language, else the default one.
    pub(crate) fn localize(&self, code: &str, default: &str) -> Self {
        let localize = |text: &Option<Localized>| {
            text.as_ref().map(|text| text.localize(code, default))
        };
        let mut config = self.clone();
        config.title = localize(&self.title);
        config.description = localize(&self.description);
        config.banner = localize(&self.banner);
        for link in &mut config.links {
            link.title = link.title.localize(code, default);
        }
        for section in &mut config.sections {
            section.title = section.title.localize(code, default);
            section.site_title = localize(&section.site_title);
            section.site_description = localize(&section.site_description);
        }
        config
    }
}

/// One `patches` entry: a page's metadata set from the config, as Livemark's patches set
/// an article's frontmatter.
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Patch {
    /// The `.html` page, or a `.md` file to publish as one, relative to the project.
    pub file: String,
    pub article: PatchArticle,
}

/// The page fields a [`Patch`] sets, each replacing what the page's own head says.
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct PatchArticle {
    pub title: Option<String>,
    pub label: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub order: Option<i64>,
    /// Where the page is served, such as `/introduction/`.
    pub path: Option<String>,
    pub sidebar: Option<bool>,
    pub breadcrumbs: Option<bool>,
    pub toc: Option<bool>,
    pub image: Option<String>,
    /// One author or a list.
    #[serde(deserialize_with = "read_patterns")]
    pub author: Vec<String>,
    pub date: Option<String>,
    pub tags: Option<Vec<String>>,
    pub source_url: Option<String>,
    pub source_action: Option<SourceAction>,
}

/// One link from the config.
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Link {
    pub url: String,
    pub title: Localized,
    /// A built-in Lucide icon's name, shown before the title.
    pub icon: Option<String>,
    #[serde(default)]
    pub position: Position,
    /// Show the link only on pages whose URL starts with this folder path, such as
    /// `/api/`.
    pub prefix: Option<String>,
}

/// One section from the config: `type: article` (the default) owns the pages under
/// `prefix`; `type: custom` is a link to `url`.
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Section {
    pub title: Localized,
    /// A built-in Lucide icon's name, shown before the title.
    pub icon: Option<String>,
    #[serde(default, rename = "type")]
    pub kind: SectionKind,
    /// The URL prefix its pages share, such as `/guides/`; `article` sections only.
    pub prefix: Option<String>,
    /// Where a `custom` section leads.
    pub url: Option<String>,
    /// The Markdown changelog a `changelog` section splits into one page per version,
    /// relative to the project, such as `CHANGELOG.md`.
    pub source: Option<String>,
    /// Show the latest version beside a `changelog` section's title.
    #[serde(default)]
    pub version: bool,
    #[serde(default)]
    pub position: Position,
    /// Replaces the site title in the brand while one of its pages is open.
    pub site_title: Option<Localized>,
    /// Replaces the site description for its pages without one of their own.
    pub site_description: Option<Localized>,
    /// Where the brand links while one of its pages is open, instead of `/`.
    pub site_link: Option<String>,
}

/// What a [`Section`] is.
#[derive(Debug, Clone, Copy, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SectionKind {
    #[default]
    Article,
    /// Dated posts, newest first, with tag pages and an RSS feed.
    Blog,
    /// One page per version of a Markdown changelog.
    Changelog,
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

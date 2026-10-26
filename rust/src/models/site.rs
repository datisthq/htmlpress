use std::collections::BTreeMap;
use std::path::PathBuf;

use std::sync::LazyLock;

use crate::actions::language::resolve::resolve_english;
use crate::models::config::{Config, SectionKind};
use crate::models::language::SiteLanguage;
use crate::models::page::Page;
use crate::models::skill::Skill;

/// A scanned project: its config, its pages in navigation order and its other files.
#[derive(Debug, Clone)]
pub struct Site {
    pub root: PathBuf,
    pub config: Config,
    /// The site title: `title` from the config, else the project folder's name.
    pub title: String,
    pub pages: Vec<Page>,
    /// Every non-page file copied as-is: its path in the output to its source path in
    /// the project, both with `/` separators.
    pub files: BTreeMap<String, String>,
    /// Every asset the build uses, by its path under `.htmlpress/`: the built-in ones with
    /// the project's escapes and own components laid over them.
    pub assets: BTreeMap<String, String>,
    /// The `SKILL.md` files the config's `skills` globs match, published for agents.
    pub skills: Vec<Skill>,
    /// The languages the site is published in, the default first.
    pub languages: Vec<SiteLanguage>,
    /// For each page key, the URL of its version in each language that has one.
    pub translations: BTreeMap<String, BTreeMap<String, String>>,
}

static ENGLISH: LazyLock<SiteLanguage> =
    LazyLock::new(|| resolve_english(&Config::default(), "Htmlpress"));

impl Site {
    /// The language with this code, else the default one.
    pub fn read_language(&self, code: &str) -> &SiteLanguage {
        (self.languages.iter())
            .find(|language| language.code == code)
            .or(self.languages.first())
            .unwrap_or(&ENGLISH)
    }

    /// The URL search engines should index for a page: a changelog's copy in another
    /// language repeats the default one's text, so it points there.
    pub(crate) fn read_canonical_url<'a>(&self, page: &'a Page) -> &'a str {
        let copy = !self.read_language(&page.language).default
            && !page.source.is_empty()
            && (self.config.sections.iter()).any(|section| {
                section.kind == SectionKind::Changelog
                    && section.source.as_deref() == Some(page.source.as_str())
            });
        match copy {
            true => &page.key,
            false => &page.url,
        }
    }
}

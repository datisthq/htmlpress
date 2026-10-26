use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::models::config::Config;

/// One `languages` entry: a language the site is published in. The first is the default,
/// served at the root; every other one under `/<code>/`.
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Language {
    /// The URL segment and the file name suffix, such as `es` for `guide.es.html`.
    pub code: String,
    /// The name shown in the language picker, such as `Español`.
    pub label: String,
    /// The BCP 47 tag for `<html lang>`, such as `pt-BR`; defaults to the code.
    pub locale: Option<String>,
    /// The text direction; `rtl` by default for Arabic, Hebrew, Persian and Urdu locales.
    pub dir: Option<Direction>,
    /// A two-letter country code for the picker's flag; defaults to the locale's region.
    pub flag: Option<String>,
    /// UI strings replacing the built-in ones, by message key.
    #[serde(default)]
    pub messages: BTreeMap<String, String>,
}

/// Which way a [`Language`]'s text runs.
#[derive(Debug, Clone, Copy, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    #[default]
    Ltr,
    Rtl,
}

/// Text from the config: one string for every language, or a map of language codes to
/// text, falling back to the default language's.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Localized {
    Text(String),
    Map(BTreeMap<String, String>),
}

impl Default for Localized {
    fn default() -> Self {
        Self::Text(String::new())
    }
}

// Read as its text, so a localized copy of the config reads like plain strings
impl std::ops::Deref for Localized {
    type Target = str;

    fn deref(&self) -> &str {
        self.text()
    }
}

impl std::fmt::Display for Localized {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.text())
    }
}

impl From<String> for Localized {
    fn from(text: String) -> Self {
        Self::Text(text)
    }
}

impl From<&str> for Localized {
    fn from(text: &str) -> Self {
        Self::Text(text.to_owned())
    }
}

impl Localized {
    /// The text: a localized copy's only one, else the map's first.
    pub fn text(&self) -> &str {
        match self {
            Self::Text(text) => text,
            Self::Map(map) => map.values().next().map_or("", String::as_str),
        }
    }

    /// The text in one language, else in the default one, else any.
    pub(crate) fn localize(&self, code: &str, default: &str) -> Self {
        match self {
            Self::Text(_) => self.clone(),
            Self::Map(map) => Self::Text(
                (map.get(code).or_else(|| map.get(default)))
                    .or_else(|| map.values().next())
                    .cloned()
                    .unwrap_or_default(),
            ),
        }
    }
}

/// A language as the build uses it: resolved defaults, its URL prefix, the config with
/// every [`Localized`] text in this language, and its UI strings.
#[derive(Debug, Clone, PartialEq)]
pub struct SiteLanguage {
    pub code: String,
    pub label: String,
    pub locale: String,
    pub dir: Direction,
    pub flag: Option<String>,
    /// `/` for the default language, `/<code>/` for the others.
    pub prefix: String,
    pub default: bool,
    /// The site title in this language.
    pub title: String,
    pub config: Config,
    /// Every UI string, by message key.
    pub messages: BTreeMap<String, String>,
}

impl SiteLanguage {
    /// The UI string for a key; the key itself when it has none, which a unit test rules out.
    pub(crate) fn read_message<'a>(&'a self, key: &'a str) -> &'a str {
        self.messages.get(key).map_or(key, String::as_str)
    }

    /// The UI string for a key with each `{name}` replaced by its value.
    pub(crate) fn format_message(&self, key: &str, values: &[(&str, &str)]) -> String {
        let mut text = self.read_message(key).to_owned();
        for (name, value) in values {
            text = text.replace(&format!("{{{name}}}"), value);
        }
        text
    }
}

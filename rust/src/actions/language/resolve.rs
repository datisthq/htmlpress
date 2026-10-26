use std::collections::BTreeMap;

use crate::models::config::Config;
use crate::models::language::{Direction, Language, SiteLanguage};
use crate::settings::MESSAGES;

const RTL: [&str; 10] = ["ar", "dv", "fa", "he", "ks", "ps", "sd", "ug", "ur", "yi"];
// The country whose flag stands for a language without a region, as in Livemark
const LANGUAGE_FLAGS: [(&str, &str); 29] = [
    ("ar", "sa"),
    ("cs", "cz"),
    ("da", "dk"),
    ("de", "de"),
    ("el", "gr"),
    ("en", "gb"),
    ("es", "es"),
    ("fa", "ir"),
    ("fi", "fi"),
    ("fr", "fr"),
    ("he", "il"),
    ("hi", "in"),
    ("hu", "hu"),
    ("id", "id"),
    ("it", "it"),
    ("ja", "jp"),
    ("ko", "kr"),
    ("nb", "no"),
    ("nl", "nl"),
    ("pl", "pl"),
    ("pt", "pt"),
    ("ro", "ro"),
    ("ru", "ru"),
    ("sv", "se"),
    ("th", "th"),
    ("tr", "tr"),
    ("uk", "ua"),
    ("vi", "vn"),
    ("zh", "cn"),
];

/// The config's languages with their defaults filled, the default one first; an implicit
/// English one without `languages`. `title` is the site title when the config sets none.
pub(crate) fn resolve_languages(config: &Config, title: &str) -> Vec<SiteLanguage> {
    if config.languages.is_empty() {
        return vec![resolve_english(config, title)];
    }
    let default =
        (config.languages.first()).map_or("en", |language| language.code.as_str());
    (config.languages.iter().enumerate())
        .map(|(index, language)| {
            resolve_language(config, title, language, index, default)
        })
        .collect()
}

/// The language a site without `languages` is written in.
pub(crate) fn resolve_english(config: &Config, title: &str) -> SiteLanguage {
    let english = Language {
        code: "en".to_owned(),
        label: "English".to_owned(),
        ..Language::default()
    };
    resolve_language(config, title, &english, 0, "en")
}

fn resolve_language(
    config: &Config,
    title: &str,
    language: &Language,
    index: usize,
    default: &str,
) -> SiteLanguage {
    let locale = language
        .locale
        .clone()
        .unwrap_or_else(|| language.code.clone());
    let primary = read_primary(&locale);
    let prefix = match index {
        0 => "/".to_owned(),
        _ => format!("/{}/", language.code),
    };
    let mut config = config.localize(&language.code, default);
    // Every routed section and prefixed link is mirrored under the language's prefix
    let localize = |path: &mut Option<String>| {
        if let Some(path) = path {
            *path = format!("{prefix}{}", path.trim_start_matches('/'));
        }
    };
    if index > 0 {
        for section in &mut config.sections {
            localize(&mut section.prefix);
        }
        for link in &mut config.links {
            localize(&mut link.prefix);
        }
    }
    SiteLanguage {
        dir: (language.dir).unwrap_or(match RTL.contains(&primary.as_str()) {
            true => Direction::Rtl,
            false => Direction::Ltr,
        }),
        flag: (language.flag.clone()).or_else(|| read_flag(&locale, &primary)),
        prefix,
        default: index == 0,
        title: (config.title.as_ref())
            .map_or(title.to_owned(), |title| title.text().to_owned()),
        messages: read_messages(&primary, &language.messages),
        code: language.code.clone(),
        label: language.label.clone(),
        locale,
        config,
    }
}

/// The built-in English strings, overlaid by the bundled ones for the locale's language,
/// then by the config's own.
pub(crate) fn read_messages(
    primary: &str,
    overrides: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    let mut messages = parse_messages("en");
    messages.extend(parse_messages(primary));
    messages.extend(overrides.clone());
    messages
}

/// One bundled message table; empty for a language htmlpress does not bundle.
pub(crate) fn parse_messages(code: &str) -> BTreeMap<String, String> {
    (MESSAGES.iter())
        .find(|(name, _)| *name == code)
        .and_then(|(_, yaml)| serde_saphyr::from_str(yaml).ok())
        .unwrap_or_default()
}

// The locale's region (`pt-BR` → `br`), else the language's usual country
fn read_flag(locale: &str, primary: &str) -> Option<String> {
    let region = (locale.split(['-', '_']).nth(1)).filter(|region| {
        region.len() == 2 && region.chars().all(|char| char.is_ascii_alphabetic())
    });
    match region {
        Some(region) => Some(region.to_lowercase()),
        None => (LANGUAGE_FLAGS.iter())
            .find(|(language, _)| *language == primary)
            .map(|(_, flag)| (*flag).to_owned()),
    }
}

fn read_primary(locale: &str) -> String {
    let primary = locale.split(['-', '_']).next().unwrap_or(locale);
    primary.to_lowercase()
}

#[cfg(test)]
#[path = "resolve_unit.rs"]
mod unit;

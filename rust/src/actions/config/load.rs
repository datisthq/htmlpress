use std::path::Path;

use crate::actions::code::highlight::has_theme;
use crate::actions::icon::render::{list_icons, render_icon};
use crate::actions::language::resolve::parse_messages;
use crate::models::config::{Config, SectionKind};
use crate::models::error::Error;
use crate::settings::CONFIG_FILE;

/// Read `htmlpress.yaml` from the project root; a project without one gets the defaults.
pub fn load_config(root: &Path) -> crate::Result<Config> {
    let path = root.join(CONFIG_FILE);
    if !path.is_file() {
        return Ok(Config::default());
    }
    let text = std::fs::read_to_string(&path).map_err(Error::io(&path))?;
    // An empty file is a project with every default, not a parse error
    if text.trim().is_empty() {
        return Ok(Config::default());
    }
    let config: Config = serde_saphyr::from_str(&text).map_err(|error| {
        let message = error.to_string();
        let message = message
            .strip_prefix("error: ")
            .unwrap_or(&message)
            .to_owned();
        Error::Config {
            path: path.clone(),
            message,
        }
    })?;
    // A base without its leading slash would be read as relative to every page
    if let Some(base) = &config.base
        && !base.starts_with('/')
    {
        return Err(Error::Config {
            path,
            message: format!("base must start with \"/\", got \"{base}\""),
        });
    }
    let (light, dark) = config.read_code_themes();
    if let Some(theme) = [light, dark].into_iter().find(|theme| !has_theme(theme)) {
        return Err(Error::Config {
            path,
            message: format!(
                "no built-in code theme \"{theme}\"; see https://textmate-grammars-themes.netlify.app"
            ),
        });
    }
    if let Some(message) = check_icons(&config) {
        return Err(Error::Config { path, message });
    }
    if let Some(message) = check_sections(&config) {
        return Err(Error::Config { path, message });
    }
    if let Some(message) = check_languages(&config) {
        return Err(Error::Config { path, message });
    }
    Ok(config)
}

fn check_icons(config: &Config) -> Option<String> {
    let links = config
        .links
        .iter()
        .map(|link| (link.title.text(), &link.icon));
    let sections =
        (config.sections.iter()).map(|section| (section.title.text(), &section.icon));
    let patches =
        (config.patches.iter()).map(|patch| (patch.file.as_str(), &patch.article.icon));
    for (title, icon) in links.chain(sections).chain(patches) {
        if let Some(icon) = icon
            && render_icon(icon).is_none()
        {
            return Some(format!(
                "\"{title}\": no built-in icon \"{icon}\"; the icons are: {}",
                list_icons()
            ));
        }
    }
    None
}

// Each kind needs its own key, and a prefix must be a folder path, so matching a page's
// URL against it never splits a name (`/guide` would take `/guides.html`)
fn check_sections(config: &Config) -> Option<String> {
    for link in &config.links {
        if let Some(prefix) = &link.prefix
            && (!prefix.starts_with('/') || !prefix.ends_with('/'))
        {
            return Some(format!(
                "link \"{}\": prefix must start and end with \"/\", got \"{prefix}\"",
                link.title
            ));
        }
    }
    for section in &config.sections {
        let title = &section.title;
        match (section.kind, &section.prefix, &section.url) {
            (
                SectionKind::Article | SectionKind::Blog | SectionKind::Changelog,
                Some(prefix),
                _,
            ) => {
                if !prefix.starts_with('/') || !prefix.ends_with('/') {
                    return Some(format!(
                        "section \"{title}\": prefix must start and end with \"/\", \
                         got \"{prefix}\""
                    ));
                }
            }
            (
                SectionKind::Article | SectionKind::Blog | SectionKind::Changelog,
                None,
                _,
            ) => {
                return Some(format!("section \"{title}\" needs a prefix"));
            }
            (SectionKind::Custom, _, None) => {
                return Some(format!("section \"{title}\" of type custom needs a url"));
            }
            (SectionKind::Custom, _, Some(_)) => {}
        }
        if section.kind == SectionKind::Changelog && section.source.is_none() {
            return Some(format!(
                "section \"{title}\" of type changelog needs a source"
            ));
        }
    }
    None
}

// A code is a URL segment and a file name suffix, so it must be plain and unique; a
// section under another language's prefix would be served as that language's page
fn check_languages(config: &Config) -> Option<String> {
    let english = parse_messages("en");
    for (index, language) in config.languages.iter().enumerate() {
        let code = &language.code;
        let plain = code.starts_with(|char: char| char.is_ascii_lowercase())
            && (code.chars()).all(|char| {
                char.is_ascii_lowercase() || char.is_ascii_digit() || char == '-'
            });
        if !plain {
            return Some(format!(
                "language \"{code}\": a code is lowercase letters, digits and \"-\", starting with a letter"
            ));
        }
        if config.languages[..index]
            .iter()
            .any(|other| other.code == *code)
        {
            return Some(format!("language \"{code}\" is listed twice"));
        }
        if language.label.trim().is_empty() {
            return Some(format!("language \"{code}\" needs a label"));
        }
        if let Some(flag) = &language.flag
            && !(flag.len() == 2 && flag.chars().all(|char| char.is_ascii_lowercase()))
        {
            return Some(format!(
                "language \"{code}\": flag must be a two-letter country code, got \"{flag}\""
            ));
        }
        if let Some(key) = language
            .messages
            .keys()
            .find(|key| !english.contains_key(*key))
        {
            return Some(format!("language \"{code}\": no message \"{key}\""));
        }
        let prefix = format!("/{code}/");
        let routed = (config.sections.iter())
            .filter(|section| section.kind != SectionKind::Custom)
            .find(|section| {
                (section.prefix.as_deref())
                    .is_some_and(|other| other.starts_with(&prefix))
            });
        if index > 0
            && let Some(section) = routed
        {
            return Some(format!(
                "section \"{}\": prefix collides with language \"{code}\"",
                section.title
            ));
        }
    }
    None
}

#[cfg(test)]
#[path = "load_unit.rs"]
mod unit;

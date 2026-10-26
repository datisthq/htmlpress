use crate::actions::page::read::create_url;
use crate::models::language::SiteLanguage;
use crate::models::page::Page;

/// Set a page's language from `<meta name="language">`, else from a `.<code>.html` suffix
/// naming a configured language, else the default one, and move it under that language's
/// prefix: `guide.es.html` is served at `/es/guide.html`, its key `/guide.html`. Suffixes
/// are read only when `suffixes` is set, so a site without `languages` keeps its paths.
pub(crate) fn detect_language(
    page: &mut Page,
    languages: &[SiteLanguage],
    suffixes: bool,
) -> Result<(), String> {
    let Some(default) = languages.first() else {
        return Ok(());
    };
    let suffix = (suffixes)
        .then(|| read_suffix(&page.path, languages))
        .flatten();
    let code = match page.language.trim() {
        "" => suffix.unwrap_or(&default.code).to_owned(),
        code => code.to_owned(),
    };
    let Some(language) = languages.iter().find(|language| language.code == code) else {
        return Err(format!(
            "<meta name=\"language\">: language \"{code}\" is not configured"
        ));
    };
    if let Some(suffix) = suffix
        && let Some(stem) = page.path.strip_suffix(&format!(".{suffix}.html"))
    {
        page.path = format!("{stem}.html");
    }
    // A translation kept in a folder named after its language is already under its prefix
    let folder = language.prefix.trim_start_matches('/');
    if let Some(path) = page.path.strip_prefix(folder).filter(|_| !language.default) {
        page.path = path.to_owned();
    }
    page.key = create_url(&page.path);
    page.path = format!("{folder}{}", page.path);
    page.url = create_url(&page.path);
    page.language = code;
    Ok(())
}

/// The URL a page under a language's prefix shares with its translations.
pub(crate) fn create_key(url: &str, language: &SiteLanguage) -> String {
    match url.strip_prefix(language.prefix.trim_end_matches('/')) {
        Some(key) if !language.default && key.starts_with('/') => key.to_owned(),
        _ => url.to_owned(),
    }
}

fn read_suffix<'a>(path: &str, languages: &'a [SiteLanguage]) -> Option<&'a str> {
    let stem = path.strip_suffix(".html")?;
    let (_, suffix) = stem.rsplit_once('.')?;
    (languages.iter())
        .map(|language| language.code.as_str())
        .find(|code| *code == suffix)
}

#[cfg(test)]
#[path = "detect_unit.rs"]
mod unit;

use crate::actions::page::read::create_url;
use crate::models::language::SiteLanguage;
use crate::models::page::Page;

/// Set a page's language as Livemark does, and move it under that language's prefix: from
/// `<meta name="language">`, else a `.<code>.html` or `.<code>.md` suffix of its source
/// naming a configured language, else the default one; `guide.es.html` is served at
/// `/es/guide/`, its key `/guide/`. A literal `path` is served as written and must start
/// with the prefix; a route's language is the prefix its URL is under. Suffixes are read
/// only when `suffixes` is set, so a site without `languages` keeps its paths.
pub(crate) fn detect_language(
    page: &mut Page,
    languages: &[SiteLanguage],
    suffixes: bool,
) -> Result<(), String> {
    let Some(default) = languages.first() else {
        return Ok(());
    };
    if page.is_route() {
        let language = (languages.iter().skip(1))
            .find(|language| page.url.starts_with(&language.prefix))
            .unwrap_or(default);
        page.key = create_key(&page.url, language);
        page.language = language.code.clone();
        return Ok(());
    }
    let suffix = (suffixes)
        .then(|| read_suffix(&page.source, languages))
        .flatten();
    let code = match page.language.trim() {
        "" => suffix
            .map_or(default.code.as_str(), |(_, code)| code)
            .to_owned(),
        code => code.to_owned(),
    };
    let Some(language) = languages.iter().find(|language| language.code == code) else {
        return Err(format!(
            "<meta name=\"language\">: language \"{code}\" is not configured"
        ));
    };
    if page.literal {
        if !page.url.starts_with(&language.prefix) {
            return Err(format!(
                "path \"{}\" must start with \"{}\", its language's prefix",
                page.url, language.prefix
            ));
        }
    } else {
        if let Some((_, suffix)) = suffix
            && let Some(stem) = (page.path.to_lowercase())
                .strip_suffix(&format!(".{}.html", suffix.to_lowercase()))
                .and_then(|stem| page.path.get(..stem.len()))
        {
            page.path = format!("{stem}.html");
        }
        page.path = format!("{}{}", language.prefix.trim_start_matches('/'), page.path);
        page.url = create_url(&page.path);
    }
    page.key = create_key(&page.url, language);
    page.language = code;
    Ok(())
}

/// The sources a translation at `source` may translate, as Livemark finds its original:
/// the name without its language suffix beside it, `guide.es.md` → `guide.html` or
/// `guide.md`. None for a source with no suffix naming a configured language.
pub(crate) fn list_originals(
    source: &str,
    languages: &[SiteLanguage],
) -> Option<[String; 2]> {
    let (stem, _) = read_suffix(source, languages)?;
    Some([format!("{stem}.html"), format!("{stem}.md")])
}

/// The URL a page under a language's prefix shares with its translations.
pub(crate) fn create_key(url: &str, language: &SiteLanguage) -> String {
    match url.strip_prefix(language.prefix.trim_end_matches('/')) {
        Some(key) if !language.default && key.starts_with('/') => key.to_owned(),
        _ => url.to_owned(),
    }
}

// The source without its extension and language suffix, and the suffix's code
fn read_suffix<'a>(
    source: &'a str,
    languages: &'a [SiteLanguage],
) -> Option<(&'a str, &'a str)> {
    let stem = (source.strip_suffix(".html")).or_else(|| source.strip_suffix(".md"))?;
    let (name, suffix) = stem.rsplit_once('.')?;
    let file = name.rsplit('/').next().unwrap_or(name);
    (languages.iter())
        .map(|language| language.code.as_str())
        .find(|code| *code == suffix && !file.is_empty())
        .map(|code| (name, code))
}

#[cfg(test)]
#[path = "detect_unit.rs"]
mod unit;

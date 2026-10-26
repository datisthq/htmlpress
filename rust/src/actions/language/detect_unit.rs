use super::*;
use crate::actions::language::resolve::resolve_languages;
use crate::actions::page::read::read_page;
use crate::models::config::Config;
use crate::models::language::Language;

fn create_languages() -> Vec<SiteLanguage> {
    let language = |code: &str| Language {
        code: code.to_owned(),
        label: code.to_owned(),
        ..Language::default()
    };
    let config = Config {
        languages: vec![language("en"), language("es")],
        ..Config::default()
    };
    resolve_languages(&config, "Docs")
}

fn detect(
    path: &str,
    html: &str,
    suffixes: bool,
) -> Result<(String, String, String), String> {
    let mut page = read_page(path, html).map_err(|error| error.to_string())?;
    detect_language(&mut page, &create_languages(), suffixes)?;
    Ok((page.language, page.url, page.key))
}

fn own(parts: (&str, &str, &str)) -> (String, String, String) {
    (parts.0.to_owned(), parts.1.to_owned(), parts.2.to_owned())
}

#[test]
fn moves_a_translation_under_its_languages_prefix() -> Result<(), String> {
    assert_eq!(
        detect("guide/install.es.html", "", true)?,
        own(("es", "/es/guide/install/", "/guide/install/"))
    );
    assert_eq!(detect("index.es.html", "", true)?, own(("es", "/es/", "/")));
    assert_eq!(
        detect("guide.en.html", "", true)?,
        own(("en", "/guide/", "/guide/"))
    );
    assert_eq!(
        detect("vite.config.html", "", true)?,
        own(("en", "/vite.config/", "/vite.config/"))
    );
    Ok(())
}

#[test]
fn reads_the_language_meta_over_the_suffix() -> Result<(), String> {
    let meta =
        r#"<meta name="language" content="es"><meta name="path" content="/start/">"#;
    assert_eq!(
        detect("inicio.html", meta, true)?,
        own(("es", "/es/start/", "/start/"))
    );
    let folder = r#"<meta name="language" content="es">"#;
    assert_eq!(
        detect("es/guia.html", folder, true)?,
        own(("es", "/es/guia/", "/guia/"))
    );
    assert_eq!(
        detect("guide.es.html", "", false)?,
        own(("en", "/guide.es/", "/guide.es/"))
    );
    let unknown = detect("a.html", r#"<meta name="language" content="fr">"#, true);
    assert!(unknown.is_err_and(|error| error.contains("\"fr\" is not configured")));
    Ok(())
}

#[test]
fn takes_the_language_prefix_off_a_url() {
    let languages = create_languages();
    assert_eq!(create_key("/es/blog/", &languages[1]), "/blog/");
    assert_eq!(create_key("/es/", &languages[1]), "/");
    assert_eq!(create_key("/es/", &languages[0]), "/es/");
}

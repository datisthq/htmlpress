use super::*;
use crate::models::language::Localized;

fn create_config(yaml: &str) -> crate::Result<Config> {
    serde_saphyr::from_str(yaml).map_err(|error| crate::Error::Config {
        path: "htmlpress.yaml".into(),
        message: error.to_string(),
    })
}

#[test]
fn makes_one_english_language_without_languages() {
    let languages = resolve_languages(&Config::default(), "Docs");
    assert_eq!(languages.len(), 1);
    let language = &languages[0];
    assert_eq!(
        (language.code.as_str(), language.prefix.as_str()),
        ("en", "/")
    );
    assert_eq!(
        (language.locale.as_str(), language.title.as_str()),
        ("en", "Docs")
    );
    assert_eq!(language.read_message("toc.title"), "On this page");
}

#[test]
fn prefixes_every_language_but_the_first_and_localizes_the_config() -> crate::Result<()> {
    let config = create_config(concat!(
        "title: { en: Docs, es: Documentación }\n",
        "description: Shared\n",
        "languages:\n",
        "  - { code: en, label: English }\n",
        "  - { code: es, label: Español, messages: { toc.title: Índice } }\n",
        "  - { code: ar, label: العربية }\n",
        "  - { code: pt, label: Português, locale: pt-BR }\n",
        "sections:\n",
        "  - { title: { en: Guide }, prefix: / }\n",
        "links:\n",
        "  - { title: API, url: /api/, prefix: /api/ }\n",
    ))?;
    let languages = resolve_languages(&config, "folder");
    let prefixes: Vec<&str> = languages.iter().map(|item| item.prefix.as_str()).collect();
    assert_eq!(prefixes, ["/", "/es/", "/ar/", "/pt/"]);
    let es = &languages[1];
    assert_eq!(es.title, "Documentación");
    assert_eq!(es.config.description, Some(Localized::from("Shared")));
    assert_eq!(es.config.sections[0].title.text(), "Guide");
    assert_eq!(es.config.sections[0].prefix.as_deref(), Some("/es/"));
    assert_eq!(es.config.links[0].prefix.as_deref(), Some("/es/api/"));
    assert_eq!(languages[0].config.sections[0].prefix.as_deref(), Some("/"));
    let flags: Vec<Option<&str>> =
        languages.iter().map(|item| item.flag.as_deref()).collect();
    assert_eq!(flags, [Some("gb"), Some("es"), Some("sa"), Some("br")]);
    assert_eq!(es.read_message("toc.title"), "Índice");
    assert_eq!(es.read_message("nav.next"), "Siguiente");
    assert_eq!(languages[2].dir, Direction::Rtl);
    assert_eq!(languages[3].locale, "pt-BR");
    assert_eq!(languages[3].read_message("nav.next"), "Next");
    Ok(())
}

#[test]
fn bundles_every_english_message_in_every_language() {
    let english = parse_messages("en");
    assert!(english.len() > 30);
    for (code, _) in MESSAGES {
        let messages = parse_messages(code);
        let keys: Vec<&String> = messages.keys().collect();
        assert_eq!(keys, english.keys().collect::<Vec<_>>(), "{code}");
    }
}

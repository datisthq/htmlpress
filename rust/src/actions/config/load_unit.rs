use super::*;
use crate::models::config::Position;

#[test]
fn defaults_without_a_config_file() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    assert_eq!(load_config(root.path())?, Config::default());
    Ok(())
}

#[test]
fn reads_the_config_file() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let text = "title: Docs\nsite: https://example.com\n";
    std::fs::write(root.path().join(CONFIG_FILE), text).map_err(Error::io("write"))?;
    let config = load_config(root.path())?;
    assert_eq!(config.title.as_deref(), Some("Docs"));
    assert_eq!(config.site.as_deref(), Some("https://example.com"));
    Ok(())
}

#[test]
fn reads_the_links_and_the_edit_url() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let text = "edit: https://example.com/edit/\nlinks:\n  - title: GitHub\n    url: https://github.com\n  - title: Chat\n    url: https://chat.example.com\n    position: sidebar\n";
    std::fs::write(root.path().join(CONFIG_FILE), text).map_err(Error::io("write"))?;
    let config = load_config(root.path())?;
    assert_eq!(config.edit.as_deref(), Some("https://example.com/edit/"));
    assert_eq!(config.links.len(), 2);
    assert_eq!(config.links[0].title, "GitHub".into());
    assert_eq!(config.links[0].url, "https://github.com");
    assert_eq!(config.links[0].position, Position::Header);
    assert_eq!(config.links[1].position, Position::Sidebar);
    Ok(())
}

#[test]
fn rejects_an_unknown_key() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    std::fs::write(root.path().join(CONFIG_FILE), "titel: Docs\n")
        .map_err(Error::io("write"))?;
    let error = load_config(root.path())
        .err()
        .map(|error| error.to_string());
    assert!(error.is_some_and(|message| message.contains("titel")));
    Ok(())
}

#[test]
fn reads_head_tags_and_an_empty_file() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let text = "head:\n  meta:\n    - name: author\n      content: Ann\n  scripts:\n    - src: /stats.js\n      defer: \"\"\n";
    std::fs::write(root.path().join(CONFIG_FILE), text).map_err(Error::io("write"))?;
    let config = load_config(root.path())?;
    assert_eq!(
        config.head.meta[0].get("content").map(String::as_str),
        Some("Ann")
    );
    assert_eq!(
        config.head.scripts[0].get("src").map(String::as_str),
        Some("/stats.js")
    );
    std::fs::write(root.path().join(CONFIG_FILE), "\n").map_err(Error::io("write"))?;
    assert_eq!(load_config(root.path())?, Config::default());
    Ok(())
}

#[test]
fn reads_the_base_and_rejects_one_without_a_leading_slash() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let write = |text: &str| {
        std::fs::write(root.path().join(CONFIG_FILE), text).map_err(Error::io("write"))
    };
    write("site: https://x.dev/\nbase: /repo/\n")?;
    let config = load_config(root.path())?;
    assert_eq!(config.read_base(), "/repo");
    assert_eq!(config.read_origin().as_deref(), Some("https://x.dev/repo"));
    write("base: repo/\n")?;
    assert!(matches!(
        load_config(root.path()),
        Err(Error::Config { .. })
    ));
    assert_eq!(Config::default().read_base(), "");
    Ok(())
}

#[test]
fn reads_sections_and_rejects_incomplete_ones() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let write = |text: &str| {
        std::fs::write(root.path().join(CONFIG_FILE), text).map_err(Error::io("write"))
    };
    write(
        "sections:\n  - title: Docs\n    prefix: /\n  - title: Blog\n    type: custom\n    url: https://x.dev/blog\n    position: sidebar\n",
    )?;
    let config = load_config(root.path())?;
    assert_eq!(config.sections.len(), 2);
    assert_eq!(
        config.sections[1].kind,
        crate::models::config::SectionKind::Custom
    );
    for text in [
        "sections:\n  - title: Docs\n",
        "sections:\n  - title: Docs\n    prefix: /docs\n",
        "sections:\n  - title: Blog\n    type: custom\n",
        "sections:\n  - title: Blog\n    type: wiki\n    prefix: /blog/\n",
    ] {
        write(text)?;
        assert!(
            matches!(load_config(root.path()), Err(Error::Config { .. })),
            "{text}"
        );
    }
    Ok(())
}

#[test]
fn rejects_an_unknown_icon() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let path = root.path().join(CONFIG_FILE);
    std::fs::write(
        &path,
        "links:\n  - title: A\n    url: /\n    icon: github\n",
    )
    .map_err(Error::io("write"))?;
    assert!(load_config(root.path()).is_ok());
    std::fs::write(&path, "links:\n  - title: A\n    url: /\n    icon: githb\n")
        .map_err(Error::io("write"))?;
    let error = load_config(root.path())
        .err()
        .map(|error| error.to_string());
    assert!(error.is_some_and(|message| message.contains("no built-in icon \"githb\"")));
    Ok(())
}

#[test]
fn requires_a_source_for_a_changelog() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let path = root.path().join(CONFIG_FILE);
    std::fs::write(
        &path,
        "sections:\n  - title: Log\n    type: changelog\n    prefix: /log/\n",
    )
    .map_err(Error::io("write"))?;
    assert!(matches!(
        load_config(root.path()),
        Err(Error::Config { .. })
    ));
    std::fs::write(
        &path,
        "sections:\n  - title: News\n    type: blog\n    prefix: /blog/\n",
    )
    .map_err(Error::io("write"))?;
    assert!(load_config(root.path()).is_ok());
    Ok(())
}

#[test]
fn rejects_an_unknown_code_theme() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let path = root.path().join(CONFIG_FILE);
    std::fs::write(&path, "codeThemeDark: github-dark\n").map_err(Error::io("write"))?;
    assert_eq!(
        load_config(root.path())?.read_code_themes(),
        ("catppuccin-latte", "github-dark")
    );
    std::fs::write(&path, "codeThemeLight: nope\n").map_err(Error::io("write"))?;
    assert!(matches!(
        load_config(root.path()),
        Err(Error::Config { .. })
    ));
    Ok(())
}

#[test]
fn reads_languages_and_rejects_bad_ones() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let write = |text: &str| {
        std::fs::write(root.path().join(CONFIG_FILE), text).map_err(Error::io("write"))
    };
    write(
        "title: { en: Docs, es: Documentos }\nlanguages:\n  - { code: en, label: English }\n  - { code: es, label: Español, flag: es }\n",
    )?;
    let config = load_config(root.path())?;
    assert_eq!(config.languages.len(), 2);
    let bad = [
        "  - { code: EN, label: English }\n",
        "  - { code: en, label: English }\n  - { code: en, label: Again }\n",
        "  - { code: en, label: \" \" }\n",
        "  - { code: en, label: English, flag: gbr }\n",
        "  - { code: en, label: English, messages: { nope: x } }\n",
        "  - { code: en, label: English }\n  - { code: es, label: Español }\nsections:\n  - { title: S, prefix: /es/x/ }\n",
    ];
    for text in bad {
        write(&format!("languages:\n{text}"))?;
        assert!(
            matches!(load_config(root.path()), Err(Error::Config { .. })),
            "{text}"
        );
    }
    Ok(())
}

use super::*;
use crate::actions::site::scan::scan_site;
use crate::models::error::Error;

#[test]
fn keeps_one_languages_pages_and_lists_the_untranslated_ones() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let write = |path: &str, text: &str| {
        let file = root.path().join(path);
        std::fs::write(&file, text).map_err(Error::io(&file))
    };
    write(
        "htmlpress.yaml",
        "title: { en: Docs, es: Documentos }\nlanguages:\n  - { code: en, label: English }\n  - { code: es, label: Español }\n",
    )?;
    write("index.html", "<h1>Home</h1>")?;
    write("index.es.html", "<h1>Inicio</h1>")?;
    write("guide.html", "<h1>Guide</h1>")?;
    let site = scan_site(root.path())?;
    let english = localize_site(&site, "en");
    let urls = |site: &Site| -> Vec<(String, bool)> {
        (site.pages.iter())
            .map(|page| (page.url.clone(), page.untranslated))
            .collect()
    };
    assert_eq!(english.title, "Docs");
    assert_eq!(
        urls(&english),
        [("/".to_owned(), false), ("/guide.html".to_owned(), false)]
    );
    let spanish = localize_site(&site, "es");
    assert_eq!(spanish.title, "Documentos");
    assert_eq!(
        urls(&spanish),
        [
            ("/es/".to_owned(), false),
            ("/es/guide.html".to_owned(), true)
        ]
    );
    assert_eq!(spanish.pages[1].key, "/guide.html");
    Ok(())
}

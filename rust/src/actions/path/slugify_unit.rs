use super::*;

#[test]
fn slugifies_every_folder_and_the_name() {
    let path = slugify_path("User Guide/Getting Started.html");
    assert_eq!(path, "user-guide/getting-started.html");
}

#[test]
fn keeps_a_slug_as_it_is() {
    assert_eq!(slugify_path("guides/setup.html"), "guides/setup.html");
    assert_eq!(slugify_path("index.html"), "index.html");
    assert_eq!(slugify_path("404.html"), "404.html");
}

#[test]
fn keeps_a_language_suffix_a_suffix() {
    assert_eq!(slugify_path("FAQ Page.es.html"), "faq-page.es.html");
}

#[test]
fn splits_camel_case_as_livemark_does() {
    assert_eq!(slugify_path("gettingStarted.html"), "getting-started.html");
    assert_eq!(slugify_path("HTMLPages.html"), "html-pages.html");
    assert_eq!(slugify_path("README.html"), "readme.html");
}

#[test]
fn collapses_symbols_and_keeps_unicode_letters() {
    assert_eq!(slugify_path("C++ & Rust!.html"), "c-rust.html");
    assert_eq!(slugify_path("Über uns.html"), "über-uns.html");
    assert_eq!(slugify_path("指南/安装.html"), "指南/安装.html");
}

#[test]
fn keeps_a_segment_of_symbols_alone() {
    assert_eq!(slugify_path("_/x.html"), "_/x.html");
}

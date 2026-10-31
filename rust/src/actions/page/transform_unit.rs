use super::transform_page;

#[test]
fn keeps_prose_that_only_looks_like_syntax() {
    let html = "<p>At 10:30 it costs $5 and $10, see index.html or note:important</p>";
    assert_eq!(transform_page(html).as_deref(), Ok(html));
}

#[test]
fn links_urls_and_replaces_shortcodes_in_prose() -> Result<(), String> {
    let html = transform_page("<p>See https://x.dev or a@b.dev :rocket:</p>")?;
    assert!(
        html.contains("<a href=\"https://x.dev\">https://x.dev</a>"),
        "{html}"
    );
    assert!(
        html.contains("<a href=\"mailto:a@b.dev\">a@b.dev</a>"),
        "{html}"
    );
    assert!(html.contains('\u{1F680}'), "{html}");
    Ok(())
}

#[test]
fn leaves_literal_regions_alone() {
    let html = "<code>https://x.dev :rocket: $a$</code>\
                <p data-literal=\"\">https://x.dev :rocket: $a$</p>\
                <htmlpress-terminal>https://x.dev :rocket: $a$</htmlpress-terminal>";
    assert_eq!(transform_page(html).as_deref(), Ok(html));
}

#[test]
fn never_takes_an_id_the_page_uses() -> Result<(), String> {
    let html =
        transform_page("<p id=\"intro\"></p><h2>Intro</h2><h2>Intro</h2><h1>T</h1>")?;
    assert!(html.contains("<h2 id=\"intro-1\">Intro</h2><h2 id=\"intro-2\">Intro</h2>"));
    assert!(html.contains("<h1>T</h1>"), "{html}");
    Ok(())
}

#[test]
fn renders_math_as_mathml_in_its_element() -> Result<(), String> {
    let html = transform_page(
        "<p>a $x_1$ b</p><htmlpress-math display>\\frac{a}{b}</htmlpress-math>",
    )?;
    assert!(
        html.contains("<htmlpress-math data-latex=\"x_1\">"),
        "{html}"
    );
    assert!(
        html.contains("<htmlpress-math display=\"\" data-latex=\"\\frac{a}{b}\">"),
        "{html}"
    );
    assert!(html.contains("<annotation encoding=\"application/x-tex\">x_1</annotation>"));
    assert_eq!(html.matches("<math").count(), 2, "{html}");
    Ok(())
}

#[test]
fn fails_on_invalid_math() {
    let error = transform_page("<htmlpress-math>\\frac{a</htmlpress-math>");
    assert!(error.is_err_and(|error| error.starts_with("invalid math")));
}

#[test]
fn keeps_a_fragments_leading_head_tags_in_place() {
    let html = "<title>T</title><meta name=\"order\" content=\"1\"><p>x</p>";
    assert_eq!(transform_page(html).as_deref(), Ok(html));
}

#[test]
fn reads_a_document_only_by_its_own_tags() {
    let html = "<header>a</header><htmlpress-card>b</htmlpress-card>";
    assert_eq!(transform_page(html).as_deref(), Ok(html));
}

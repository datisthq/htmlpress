use super::*;
use crate::actions::text::enrich::enrich_text;

#[test]
fn renders_written_and_dollar_math() -> Result<(), String> {
    let html = enrich_text(
        "<p>Energy $E = mc^2$ and</p><p>$$\\int_0^1 x\\,dx$$</p><htmlpress-math>a &lt; b</htmlpress-math>",
    )?;
    assert!(
        html.contains("<htmlpress-math>E = mc^2</htmlpress-math>"),
        "{html}"
    );
    assert!(
        html.contains("<htmlpress-math display>\\int_0^1 x\\,dx</htmlpress-math>"),
        "{html}"
    );
    let rendered = render_math(&html)?;
    assert!(
        rendered.contains("<htmlpress-math data-latex=\"E = mc^2\"><math><mi>E</mi>"),
        "{rendered}"
    );
    assert!(rendered.contains("<math display=\"block\">"));
    assert!(rendered.contains("data-latex=\"a < b\""), "{rendered}");
    assert!(render_math("<htmlpress-math>\\frac{1}{</htmlpress-math>").is_err());
    Ok(())
}

#[test]
fn keeps_prices_escapes_and_code_as_text() -> Result<(), String> {
    for html in [
        "<p>It costs $5 and $10 later.</p>",
        "<p>Pay $ 5 now.</p>",
        "<p><code>$x$</code></p>",
        "<p data-literal>$x$</p>",
        "<p>Only one $ sign</p>",
    ] {
        assert_eq!(enrich_text(html)?, html);
    }
    assert_eq!(
        enrich_text("<p>A \\$5 fee, $x$</p>")?,
        "<p>A $5 fee, <htmlpress-math>x</htmlpress-math></p>"
    );
    Ok(())
}

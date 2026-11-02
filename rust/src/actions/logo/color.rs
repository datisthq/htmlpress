const PRIMARY: &str = "--htmlpress-primary:";
const DARK: &str = "prefers-color-scheme: dark";

/// Colors a `currentColor` logo with the tokens' primary, light and dark, for where it
/// is drawn alone, as the favicon, outside any page's CSS.
pub(crate) fn color_logo(svg: &str, tokens: &str) -> String {
    let Some(light) = read_primary(tokens) else {
        return svg.to_owned();
    };
    let dark = tokens
        .find(DARK)
        .and_then(|start| read_primary(&tokens[start..]));
    let Some(end) = svg
        .find("<svg")
        .and_then(|start| svg[start..].find('>').map(|offset| start + offset + 1))
    else {
        return svg.to_owned();
    };
    let mut style = format!("<style>svg{{color:{light}}}");
    if let Some(dark) = dark {
        style.push_str(&format!("@media ({DARK}){{svg{{color:{dark}}}}}"));
    }
    style.push_str("</style>");
    format!("{}{style}{}", &svg[..end], &svg[end..])
}

// A value naming another token cannot resolve in a file drawn on its own
fn read_primary(tokens: &str) -> Option<&str> {
    let start = tokens.find(PRIMARY)? + PRIMARY.len();
    let value = tokens[start..].split([';', '}']).next()?.trim();
    (!value.is_empty() && !value.contains("var(")).then_some(value)
}

#[cfg(test)]
#[path = "color_unit.rs"]
mod unit;

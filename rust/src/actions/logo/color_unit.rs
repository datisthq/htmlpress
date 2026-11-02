use super::color_logo;

const SVG: &str = r#"<svg viewBox="0 0 1 1" fill="currentColor"><rect/></svg>"#;

#[test]
fn colors_the_logo_with_the_light_and_dark_primary() {
    let tokens =
        ":root { --htmlpress-primary-foreground: white; --htmlpress-primary: red; }
        @media (prefers-color-scheme: dark) { :root { --htmlpress-primary: blue; } }";
    assert_eq!(
        color_logo(SVG, tokens),
        r#"<svg viewBox="0 0 1 1" fill="currentColor"><style>svg{color:red}@media (prefers-color-scheme: dark){svg{color:blue}}</style><rect/></svg>"#
    );
}

#[test]
fn keeps_only_the_light_primary_without_a_dark_set() {
    let colored = color_logo(SVG, ":root { --htmlpress-primary: red }");
    assert!(colored.contains("<style>svg{color:red}</style><rect/>"));
}

#[test]
fn leaves_the_logo_as_is_without_a_usable_primary() {
    assert_eq!(color_logo(SVG, ":root {}"), SVG);
    assert_eq!(
        color_logo(SVG, ":root { --htmlpress-primary: var(--x); }"),
        SVG
    );
}

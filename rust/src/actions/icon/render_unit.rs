use super::*;

#[test]
fn renders_a_known_icon_and_refuses_an_unknown_one() {
    let svg = render_icon("book-open").unwrap_or_default();
    assert!(svg.starts_with("<svg viewBox=\"0 0 24 24\""));
    assert!(svg.contains("<path d='M12 7v14'/>"));
    assert!(render_icon("no-such-icon").is_none());
    assert!(list_icons().starts_with("activity, archive"));
}

#[test]
fn keeps_the_icon_set_sorted_for_lookup() {
    assert!(ICONS.windows(2).all(|pair| pair[0].0 < pair[1].0));
}

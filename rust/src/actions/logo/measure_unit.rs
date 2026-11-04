use super::measure_logo;

#[test]
fn measure_logo_scales_an_svg_view_box() {
    let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 400 200"></svg>"#;
    assert_eq!(measure_logo(svg, 26), Some((52, 26)));
}

#[test]
fn measure_logo_prefers_an_svg_width_and_height() {
    let svg = br#"<?xml version="1.0"?><svg width="30px" height="30" viewBox="0 0 1 2">"#;
    assert_eq!(measure_logo(svg, 26), Some((26, 26)));
}

#[test]
fn measure_logo_reads_a_png_header() {
    let mut png = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
    png.extend(100u32.to_be_bytes());
    png.extend(50u32.to_be_bytes());
    assert_eq!(measure_logo(&png, 26), Some((52, 26)));
}

#[test]
fn measure_logo_is_none_without_a_size() {
    assert_eq!(
        measure_logo(br#"<svg width="100%" height="100%">"#, 26),
        None
    );
    assert_eq!(measure_logo(b"GIF89a", 26), None);
}

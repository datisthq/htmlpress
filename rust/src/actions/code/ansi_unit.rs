use super::*;

#[test]
fn styles_colors_and_resets() {
    assert_eq!(
        render_ansi("\u{1b}[1;32mok\u{1b}[0m plain <x>"),
        "<span style=\"color:#a3be8c;font-weight:bold\">ok</span> plain &lt;x&gt;"
    );
}

#[test]
fn reads_256_and_true_colors() {
    assert_eq!(
        render_ansi("\u{1b}[38;5;196ma\u{1b}[48;2;1;2;3mb"),
        concat!(
            "<span style=\"color:#ff0000\">a</span>",
            "<span style=\"color:#ff0000;background-color:#010203\">b</span>"
        )
    );
}

#[test]
fn drops_other_escapes() {
    assert_eq!(render_ansi("a\u{1b}[2Kb\u{1b}c"), "abc");
}

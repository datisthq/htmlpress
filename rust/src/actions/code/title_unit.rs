use super::*;
use crate::actions::node::write::write_nodes;

fn frame(html: &str) -> Result<String, String> {
    let alloc = Allocator::new();
    let mut nodes = parse_fragment(html, &alloc);
    add_titles(&mut nodes, &alloc)?;
    write_nodes(nodes)
}

#[test]
fn frames_a_titled_block_with_its_language_logo() -> Result<(), String> {
    let html = "<pre data-title=\"a &lt;b&gt;.ts\"><code class=\"language-ts\">x</code></pre><pre><code>y</code></pre>";
    let framed = frame(html)?;
    assert!(
        framed.starts_with(
            "<figure class=\"htmlpress-code-figure\"><figcaption><svg viewBox"
        ),
        "{framed}"
    );
    assert!(
        framed.contains("</svg>a &lt;b>.ts</figcaption><pre data-title"),
        "{framed}"
    );
    assert!(framed.ends_with("</pre></figure><pre><code>y</code></pre>"));
    let typescript = create_icon("typescript");
    assert!(typescript.starts_with("<svg") && create_icon("ts") == typescript);
    assert!(
        create_icon("haskell") == create_icon("default")
            && !create_icon("haskell").is_empty()
    );
    Ok(())
}

#[test]
fn sets_the_height_of_a_collapsed_block() -> Result<(), String> {
    let html = "<pre data-collapse=\"3\"><code>a</code></pre>";
    assert!(frame(html)?.contains("style=\"--htmlpress-code-lines: 3\""));
    assert!(frame("<pre data-collapse=\"many\"><code>a</code></pre>").is_err());
    Ok(())
}

#[test]
fn keeps_the_icon_set_sorted_for_lookup() {
    assert!(LANGUAGE_ICONS.windows(2).all(|pair| pair[0].0 < pair[1].0));
}

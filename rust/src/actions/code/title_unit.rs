use super::*;

#[test]
fn frames_a_titled_block() -> Result<(), String> {
    let html = "<pre data-title=\"a &lt;b&gt;.yaml\"><code>x</code></pre><pre><code>y</code></pre>";
    assert_eq!(
        add_titles(html)?,
        "<figure class=\"htmlpress-code-figure\"><figcaption>a &lt;b&gt;.yaml</figcaption>\
         <pre data-title=\"a &lt;b&gt;.yaml\"><code>x</code></pre></figure><pre><code>y</code></pre>"
    );
    Ok(())
}

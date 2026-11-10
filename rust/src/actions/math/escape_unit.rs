use super::*;

#[test]
fn escapes_only_the_dollars_the_source_escaped() {
    assert_eq!(
        escape_text("a $x$ b $", r"a \$x\$ b $").as_deref(),
        Some(r"a \$x\$ b $")
    );
    assert_eq!(escape_text(r"a \$", r"a \\$"), None);
    assert_eq!(escape_text("costs $5", "costs $5"), None);
    assert_eq!(escape_text("$ and $", r"&#36; and \$"), None);
}

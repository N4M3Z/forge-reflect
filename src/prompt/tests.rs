use super::*;

#[test]
fn test_strip_frontmatter_and_h1() {
    let input = "---\ntitle: Test\n---\n# My Title\n\nBody text here.\n";
    let result = strip_frontmatter_and_h1(input);
    assert_eq!(result.trim(), "Body text here.");
}

#[test]
fn test_strip_h1_only() {
    let input = "# My Title\n\nBody text here.\n";
    let result = strip_frontmatter_and_h1(input);
    assert_eq!(result.trim(), "Body text here.");
}

#[test]
fn test_no_frontmatter_no_h1() {
    let input = "Just body text.\n";
    let result = strip_frontmatter_and_h1(input);
    assert_eq!(result.trim(), "Just body text.");
}

// @specre 01JMBJK7QRVX3N4P5G6H8W9Y0Z
pub fn render(id: &str, name: &str) -> String {
    format!(
        r#"---
id: "{id}"
name: "{name}"
status: "draft"
---

## Related Files

-

## Functional Overview



## Scenarios

###

1.
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_contains_frontmatter_fields() {
        let output = render("01ABC123DEF456GHI789JKL0MN", "my_specre");
        assert!(output.starts_with("---\n"));
        assert!(output.contains(r#"id: "01ABC123DEF456GHI789JKL0MN""#));
        assert!(output.contains(r#"name: "my_specre""#));
        assert!(output.contains(r#"status: "draft""#));
    }

    #[test]
    fn template_contains_required_sections() {
        let output = render("01ABC123DEF456GHI789JKL0MN", "test");
        assert!(output.contains("## Related Files"));
        assert!(output.contains("## Functional Overview"));
        assert!(output.contains("## Scenarios"));
    }

    #[test]
    fn template_ends_with_newline() {
        let output = render("01ABC123DEF456GHI789JKL0MN", "test");
        assert!(output.ends_with('\n'));
    }
}

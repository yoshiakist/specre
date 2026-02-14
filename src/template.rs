// @specre 01JMBJK7QRVX3N4P5G6H8W9Y0Z
// @specre 01KHDF9WHR5HFM4RQCF6HS3KCC
pub fn render(id: &str, name: &str, language: &str) -> String {
    let (related, overview, scenarios) = section_headings(language);
    format!(
        r#"---
id: "{id}"
name: "{name}"
status: "draft"
---

## {related}

-

## {overview}



## {scenarios}

###

1.
"#
    )
}

fn section_headings(language: &str) -> (&'static str, &'static str, &'static str) {
    match language {
        "ja" => ("関連ファイル", "機能概要", "シナリオ"),
        _ => ("Related Files", "Functional Overview", "Scenarios"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_contains_frontmatter_fields() {
        let output = render("01ABC123DEF456GHI789JKL0MN", "my_specre", "en");
        assert!(output.starts_with("---\n"));
        assert!(output.contains(r#"id: "01ABC123DEF456GHI789JKL0MN""#));
        assert!(output.contains(r#"name: "my_specre""#));
        assert!(output.contains(r#"status: "draft""#));
    }

    #[test]
    fn template_contains_required_sections() {
        let output = render("01ABC123DEF456GHI789JKL0MN", "test", "en");
        assert!(output.contains("## Related Files"));
        assert!(output.contains("## Functional Overview"));
        assert!(output.contains("## Scenarios"));
    }

    #[test]
    fn template_contains_japanese_sections() {
        let output = render("01ABC123DEF456GHI789JKL0MN", "test", "ja");
        assert!(output.contains("## 関連ファイル"));
        assert!(output.contains("## 機能概要"));
        assert!(output.contains("## シナリオ"));
    }

    #[test]
    fn template_ends_with_newline() {
        let output = render("01ABC123DEF456GHI789JKL0MN", "test", "en");
        assert!(output.ends_with('\n'));
    }

    #[test]
    fn unknown_language_falls_back_to_english() {
        let output = render("01ABC123DEF456GHI789JKL0MN", "test", "fr");
        assert!(output.contains("## Related Files"));
    }
}

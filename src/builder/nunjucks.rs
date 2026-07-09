//! Nunjucks-compatible template processing using Tera
//!
//! This module provides full Nunjucks/Jinja2 template syntax support for Markdown content,
//! including conditionals, loops, and filters.
//!
//! ## Supported Features
//!
//! ### Conditionals
//! ```text
//! {% if condition %}
//!   content
//! {% elif other_condition %}
//!   other content
//! {% else %}
//!   fallback
//! {% endif %}
//! ```
//!
//! ### Loops
//! ```text
//! {% for item in list %}
//!   {{ item }}
//! {% endfor %}
//! ```
//!
//! ### Filters
//! ```text
//! {{ value | upper }}
//! {{ value | lower }}
//! {{ value | default("fallback") }}
//! ```

use crate::parser::BookConfig;
use anyhow::{Context, Result};
use tera::{Context as TeraContext, Tera};

/// Process Nunjucks templates in Markdown content
///
/// This function replaces the simple `expand_variables()` approach with full Tera template
/// processing, supporting conditionals, loops, and filters while maintaining backward
/// compatibility with `{{ book.xxx }}` syntax.
///
/// # Arguments
/// * `content` - The Markdown content containing Nunjucks templates
/// * `config` - Book configuration containing variables
///
/// # Returns
/// * `Ok(String)` - Processed content with templates rendered
/// * `Err` - Template parsing or rendering error with location info
pub fn process_nunjucks_templates(content: &str, config: &BookConfig) -> Result<String> {
    // Fast path: if no template syntax detected, return as-is
    if !has_template_syntax(content) {
        return Ok(content.to_string());
    }

    // Find protected regions (code blocks) to exclude from template processing
    let protected_regions = find_protected_regions(content);

    // If content has protected regions, we need to handle them specially
    if !protected_regions.is_empty() {
        return process_with_protected_regions(content, config, &protected_regions);
    }

    // No protected regions, process the entire content
    render_template(content, config)
}

/// Check if content contains any Nunjucks template syntax
fn has_template_syntax(content: &str) -> bool {
    // Quick check for common template markers
    content.contains("{{") || content.contains("{%")
}

/// Find all protected regions in the content (fenced code blocks)
/// These regions should not have template processing applied
///
/// Handles ``` and ~~~ fences, longer fence runs (````), and fences indented
/// inside list items. Limitation: pure 4-space indented code blocks are not
/// detected (that requires full markdown block parsing).
fn find_protected_regions(content: &str) -> Vec<(usize, usize)> {
    let mut regions = Vec::new();
    // (region_start_byte, fence_char, fence_len) while inside a fence
    let mut open: Option<(usize, char, usize)> = None;
    let mut pos = 0usize;

    for line in content.split_inclusive('\n') {
        let trimmed = line.trim_start();

        match open {
            Some((start, fence_char, fence_len)) => {
                // Closing fence: same char, at least as long, alone on the line
                let run = trimmed.chars().take_while(|&c| c == fence_char).count();
                if run >= fence_len && trimmed.trim_end().chars().all(|c| c == fence_char) {
                    regions.push((start, pos + line.len()));
                    open = None;
                }
            }
            None => {
                let first = trimmed.chars().next();
                if matches!(first, Some('`') | Some('~')) {
                    let fence_char = first.unwrap();
                    let run = trimmed.chars().take_while(|&c| c == fence_char).count();
                    // Info strings after ``` must not contain backticks
                    let info_ok = fence_char == '~' || !trimmed[run..].contains('`');
                    if run >= 3 && info_ok {
                        open = Some((pos, fence_char, run));
                    }
                }
            }
        }

        pos += line.len();
    }

    // An unclosed fence protects everything to the end of the document
    // (matches how markdown renders it)
    if let Some((start, _, _)) = open {
        regions.push((start, content.len()));
    }

    regions
}

/// Process content with protected regions
///
/// Each protected region (code block) is swapped for a placeholder token,
/// the WHOLE document is rendered once, then the original blocks are
/// restored. Rendering the document in one pass keeps template blocks that
/// span a code block (e.g. `{% if %}` before / `{% endif %}` after) working;
/// the previous approach of rendering the segments independently failed on
/// them and the whole page fell back to unprocessed content.
fn process_with_protected_regions(
    content: &str,
    config: &BookConfig,
    protected_regions: &[(usize, usize)],
) -> Result<String> {
    let mut template = String::new();
    let mut blocks: Vec<&str> = Vec::new();
    let mut last_end = 0;

    for (start, end) in protected_regions {
        template.push_str(&content[last_end..*start]);
        template.push_str(&protected_placeholder(blocks.len()));
        blocks.push(&content[*start..*end]);
        last_end = *end;
    }
    template.push_str(&content[last_end..]);

    let mut rendered = render_template(&template, config)?;

    // Restore code blocks. replace() (not replacen) so blocks duplicated by
    // {% for %} loops are restored at every occurrence; blocks dropped by a
    // false {% if %} branch simply have no occurrence to restore.
    for (idx, block) in blocks.iter().enumerate() {
        rendered = rendered.replace(&protected_placeholder(idx), block);
    }

    Ok(rendered)
}

/// Placeholder token for a protected region.
/// U+F8FF is a private-use character that does not occur in normal content
/// and contains no template syntax, so Tera passes it through untouched.
fn protected_placeholder(idx: usize) -> String {
    format!("\u{F8FF}GBPROTECTED{}\u{F8FF}", idx)
}

/// Render a template string using Tera
fn render_template(content: &str, config: &BookConfig) -> Result<String> {
    let mut tera = Tera::default();

    // Add custom template with a unique name
    tera.add_raw_template("__content__", content)
        .with_context(|| format_template_error(content, "Failed to parse template"))?;

    // Build context from book config
    let mut context = TeraContext::new();

    // Add all variables from book.json to context
    // They're accessible both as top-level and under "book" object
    for (key, value) in &config.variables {
        // Add as top-level variable
        context.insert(key, &json_to_tera_value(value));
    }

    // Add a "book" object for {{ book.xxx }} compatibility
    // This maintains backward compatibility with the existing syntax
    let book_map: std::collections::HashMap<String, tera::Value> = config
        .variables
        .iter()
        .map(|(k, v)| (k.clone(), json_to_tera_value(v)))
        .collect();
    context.insert("book", &book_map);

    // Render the template
    tera.render("__content__", &context)
        .with_context(|| format_template_error(content, "Failed to render template"))
}

/// Convert serde_json::Value to tera::Value
fn json_to_tera_value(json: &serde_json::Value) -> tera::Value {
    match json {
        serde_json::Value::Null => tera::Value::Null,
        serde_json::Value::Bool(b) => tera::Value::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                tera::Value::Number(i.into())
            } else if let Some(f) = n.as_f64() {
                tera::Value::Number(serde_json::Number::from_f64(f).unwrap_or_else(|| 0.into()))
            } else {
                tera::Value::String(n.to_string())
            }
        }
        serde_json::Value::String(s) => tera::Value::String(s.clone()),
        serde_json::Value::Array(arr) => {
            let values: Vec<tera::Value> = arr.iter().map(json_to_tera_value).collect();
            tera::Value::Array(values)
        }
        serde_json::Value::Object(obj) => {
            let map: tera::Map<String, tera::Value> = obj
                .iter()
                .map(|(k, v)| (k.clone(), json_to_tera_value(v)))
                .collect();
            tera::Value::Object(map)
        }
    }
}

/// Format template error with helpful context
fn format_template_error(content: &str, message: &str) -> String {
    // Try to find the problematic line
    let lines: Vec<&str> = content.lines().collect();
    let preview_lines = lines.iter().take(5).cloned().collect::<Vec<_>>().join("\n");

    format!(
        "{}\n\nContent preview:\n{}{}",
        message,
        preview_lines,
        if lines.len() > 5 { "\n..." } else { "" }
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_config(variables: HashMap<String, serde_json::Value>) -> BookConfig {
        BookConfig {
            variables,
            ..Default::default()
        }
    }

    // === Basic Variable Tests ===

    #[test]
    fn test_basic_variable_expansion() {
        let mut vars = HashMap::new();
        vars.insert("version".to_string(), serde_json::json!("1.0.0"));
        vars.insert("author".to_string(), serde_json::json!("Guide Inc"));

        let config = create_test_config(vars);
        let content = "Version: {{ book.version }}\nAuthor: {{ book.author }}";
        let result = process_nunjucks_templates(content, &config).unwrap();

        assert_eq!(result, "Version: 1.0.0\nAuthor: Guide Inc");
    }

    #[test]
    fn test_variable_without_spaces() {
        let mut vars = HashMap::new();
        vars.insert("version".to_string(), serde_json::json!("2.0.0"));

        let config = create_test_config(vars);
        let content = "Version: {{book.version}}";
        let result = process_nunjucks_templates(content, &config).unwrap();

        assert_eq!(result, "Version: 2.0.0");
    }

    #[test]
    fn test_number_variable() {
        let mut vars = HashMap::new();
        vars.insert("year".to_string(), serde_json::json!(2024));

        let config = create_test_config(vars);
        let content = "Year: {{ book.year }}";
        let result = process_nunjucks_templates(content, &config).unwrap();

        assert_eq!(result, "Year: 2024");
    }

    #[test]
    fn test_boolean_variable() {
        let mut vars = HashMap::new();
        vars.insert("published".to_string(), serde_json::json!(true));

        let config = create_test_config(vars);
        let content = "Published: {{ book.published }}";
        let result = process_nunjucks_templates(content, &config).unwrap();

        assert_eq!(result, "Published: true");
    }

    // === Conditional Tests ===

    #[test]
    fn test_if_condition_true() {
        let mut vars = HashMap::new();
        vars.insert("show_feature".to_string(), serde_json::json!(true));

        let config = create_test_config(vars);
        let content = "{% if book.show_feature %}Feature is enabled{% endif %}";
        let result = process_nunjucks_templates(content, &config).unwrap();

        assert_eq!(result, "Feature is enabled");
    }

    #[test]
    fn test_if_condition_false() {
        let mut vars = HashMap::new();
        vars.insert("show_feature".to_string(), serde_json::json!(false));

        let config = create_test_config(vars);
        let content = "{% if book.show_feature %}Feature is enabled{% endif %}";
        let result = process_nunjucks_templates(content, &config).unwrap();

        assert_eq!(result, "");
    }

    #[test]
    fn test_if_else_condition() {
        let mut vars = HashMap::new();
        vars.insert("premium".to_string(), serde_json::json!(false));

        let config = create_test_config(vars);
        let content = "{% if book.premium %}Premium content{% else %}Free content{% endif %}";
        let result = process_nunjucks_templates(content, &config).unwrap();

        assert_eq!(result, "Free content");
    }

    #[test]
    fn test_if_elif_else_condition() {
        let mut vars = HashMap::new();
        vars.insert("tier".to_string(), serde_json::json!("pro"));

        let config = create_test_config(vars);
        let content = r#"{% if book.tier == "basic" %}Basic{% elif book.tier == "pro" %}Professional{% else %}Enterprise{% endif %}"#;
        let result = process_nunjucks_templates(content, &config).unwrap();

        assert_eq!(result, "Professional");
    }

    // === Loop Tests ===

    #[test]
    fn test_for_loop_array() {
        let mut vars = HashMap::new();
        vars.insert(
            "features".to_string(),
            serde_json::json!(["Search", "Export", "Share"]),
        );

        let config = create_test_config(vars);
        let content = "Features:\n{% for feature in book.features %}- {{ feature }}\n{% endfor %}";
        let result = process_nunjucks_templates(content, &config).unwrap();

        assert_eq!(result, "Features:\n- Search\n- Export\n- Share\n");
    }

    #[test]
    fn test_for_loop_with_index() {
        let mut vars = HashMap::new();
        vars.insert("items".to_string(), serde_json::json!(["A", "B", "C"]));

        let config = create_test_config(vars);
        let content = "{% for item in book.items %}{{ loop.index }}. {{ item }}\n{% endfor %}";
        let result = process_nunjucks_templates(content, &config).unwrap();

        assert_eq!(result, "1. A\n2. B\n3. C\n");
    }

    #[test]
    fn test_for_loop_empty_array() {
        let mut vars = HashMap::new();
        vars.insert("items".to_string(), serde_json::json!([]));

        let config = create_test_config(vars);
        let content = "{% for item in book.items %}{{ item }}{% endfor %}";
        let result = process_nunjucks_templates(content, &config).unwrap();

        assert_eq!(result, "");
    }

    // === Filter Tests ===

    #[test]
    fn test_upper_filter() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), serde_json::json!("guide"));

        let config = create_test_config(vars);
        let content = "{{ book.name | upper }}";
        let result = process_nunjucks_templates(content, &config).unwrap();

        assert_eq!(result, "GUIDE");
    }

    #[test]
    fn test_lower_filter() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), serde_json::json!("GUIDE"));

        let config = create_test_config(vars);
        let content = "{{ book.name | lower }}";
        let result = process_nunjucks_templates(content, &config).unwrap();

        assert_eq!(result, "guide");
    }

    #[test]
    fn test_default_filter_with_value() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), serde_json::json!("Guide"));

        let config = create_test_config(vars);
        let content = r#"{{ book.name | default(value="Unknown") }}"#;
        let result = process_nunjucks_templates(content, &config).unwrap();

        assert_eq!(result, "Guide");
    }

    #[test]
    fn test_default_filter_without_value() {
        let vars = HashMap::new();

        let config = create_test_config(vars);
        let content = r#"{{ book.name | default(value="Unknown") }}"#;
        let result = process_nunjucks_templates(content, &config).unwrap();

        assert_eq!(result, "Unknown");
    }

    #[test]
    fn test_capitalize_filter() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), serde_json::json!("guide inc"));

        let config = create_test_config(vars);
        let content = "{{ book.name | capitalize }}";
        let result = process_nunjucks_templates(content, &config).unwrap();

        assert_eq!(result, "Guide inc");
    }

    #[test]
    fn test_length_filter() {
        let mut vars = HashMap::new();
        vars.insert("items".to_string(), serde_json::json!(["a", "b", "c"]));

        let config = create_test_config(vars);
        let content = "{{ book.items | length }}";
        let result = process_nunjucks_templates(content, &config).unwrap();

        assert_eq!(result, "3");
    }

    // === Code Block Protection Tests ===

    #[test]
    fn test_preserve_code_block() {
        let mut vars = HashMap::new();
        vars.insert("version".to_string(), serde_json::json!("1.0.0"));

        let config = create_test_config(vars);
        let content = r#"Version: {{ book.version }}

```javascript
const template = "{{ book.version }}";
```

End"#;
        let result = process_nunjucks_templates(content, &config).unwrap();

        assert!(result.contains("Version: 1.0.0"));
        assert!(result.contains(r#""{{ book.version }}""#));
        assert!(result.contains("End"));
    }

    #[test]
    fn test_multiple_code_blocks() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), serde_json::json!("Test"));

        let config = create_test_config(vars);
        let content = r#"Name: {{ book.name }}

```
{{ book.name }} in code
```

Middle: {{ book.name }}

```rust
let x = "{{ book.name }}";
```

End"#;
        let result = process_nunjucks_templates(content, &config).unwrap();

        // Outside code blocks should be expanded
        assert!(result.contains("Name: Test"));
        assert!(result.contains("Middle: Test"));
        // Inside code blocks should be preserved
        assert!(result.contains("{{ book.name }} in code"));
        assert!(result.contains(r#""{{ book.name }}""#));
    }

    #[test]
    fn test_indented_fence_in_list_is_protected() {
        // Regression: the fence regex only matched column-0 fences, so code
        // blocks indented inside list items were template-processed
        let mut vars = HashMap::new();
        vars.insert("token".to_string(), serde_json::json!("SECRET"));

        let config = create_test_config(vars);
        let content =
            "- Step 1\n  ```bash\n  {{ book.token }}\n  ```\n\nOutside: {{ book.token }}\n";
        let result = process_nunjucks_templates(content, &config).unwrap();

        assert!(
            result.contains("{{ book.token }}"),
            "indented code block must stay literal: {}",
            result
        );
        assert!(result.contains("Outside: SECRET"), "{}", result);
    }

    #[test]
    fn test_tilde_fence_is_protected() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), serde_json::json!("X"));

        let config = create_test_config(vars);
        let content = "~~~js\n{{ book.name }}\n~~~\n\n{{ book.name }}\n";
        let result = process_nunjucks_templates(content, &config).unwrap();

        assert!(result.contains("{{ book.name }}"), "{}", result);
        assert!(result.contains("\nX"), "{}", result);
    }

    #[test]
    fn test_longer_backtick_fence_protected() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), serde_json::json!("X"));

        let config = create_test_config(vars);
        // ```` fence containing a ``` line — must close only at ````
        let content = "````\n```\n{{ book.name }}\n```\n````\n";
        let result = process_nunjucks_templates(content, &config).unwrap();

        assert!(result.contains("{{ book.name }}"), "{}", result);
    }

    #[test]
    fn test_template_block_spanning_code_block() {
        // Regression: segments were rendered independently, so an {% if %}
        // opened before a code block and closed after it was a parse error
        // and the whole page fell through unprocessed
        let mut vars = HashMap::new();
        vars.insert("show".to_string(), serde_json::json!(true));

        let config = create_test_config(vars);
        let content = r#"{% if book.show %}
```
code sample with {{ book.show }}
```
{% endif %}

After"#;
        let result = process_nunjucks_templates(content, &config).unwrap();

        // Code block content preserved verbatim (template syntax untouched)
        assert!(result.contains("code sample with {{ book.show }}"));
        assert!(result.contains("After"));
        assert!(!result.contains("{% if"));
    }

    #[test]
    fn test_template_block_spanning_code_block_false_branch() {
        let mut vars = HashMap::new();
        vars.insert("show".to_string(), serde_json::json!(false));

        let config = create_test_config(vars);
        let content = r#"Before

{% if book.show %}
```
hidden code
```
{% endif %}

After"#;
        let result = process_nunjucks_templates(content, &config).unwrap();

        assert!(result.contains("Before"));
        assert!(result.contains("After"));
        // The code block inside the false branch must disappear entirely,
        // including its placeholder
        assert!(!result.contains("hidden code"));
        assert!(!result.contains('\u{F8FF}'));
    }

    #[test]
    fn test_code_block_duplicated_by_for_loop() {
        let mut vars = HashMap::new();
        vars.insert("items".to_string(), serde_json::json!(["a", "b"]));

        let config = create_test_config(vars);
        let content = r#"{% for item in book.items %}
```
sample
```
{% endfor %}"#;
        let result = process_nunjucks_templates(content, &config).unwrap();

        assert_eq!(result.matches("sample").count(), 2);
        assert!(!result.contains('\u{F8FF}'));
    }

    // === Edge Cases ===

    #[test]
    fn test_no_template_syntax() {
        let config = create_test_config(HashMap::new());
        let content = "This is plain markdown without any template syntax.";
        let result = process_nunjucks_templates(content, &config).unwrap();

        assert_eq!(result, content);
    }

    #[test]
    fn test_empty_content() {
        let config = create_test_config(HashMap::new());
        let content = "";
        let result = process_nunjucks_templates(content, &config).unwrap();

        assert_eq!(result, "");
    }

    #[test]
    fn test_nested_objects() {
        let mut vars = HashMap::new();
        vars.insert(
            "author".to_string(),
            serde_json::json!({
                "name": "John Doe",
                "email": "john@example.com"
            }),
        );

        let config = create_test_config(vars);
        let content = "Author: {{ book.author.name }} <{{ book.author.email }}>";
        let result = process_nunjucks_templates(content, &config).unwrap();

        assert_eq!(result, "Author: John Doe <john@example.com>");
    }

    // === Compatibility Tests ===

    #[test]
    fn test_top_level_variable_access() {
        // Variables should be accessible both as book.xxx and just xxx
        let mut vars = HashMap::new();
        vars.insert("version".to_string(), serde_json::json!("1.0.0"));

        let config = create_test_config(vars);

        // Both syntaxes should work
        let content1 = "{{ book.version }}";
        let content2 = "{{ version }}";

        let result1 = process_nunjucks_templates(content1, &config).unwrap();
        let result2 = process_nunjucks_templates(content2, &config).unwrap();

        assert_eq!(result1, "1.0.0");
        assert_eq!(result2, "1.0.0");
    }

    // === Complex Markdown Tests ===

    #[test]
    fn test_template_in_markdown_heading() {
        let mut vars = HashMap::new();
        vars.insert("version".to_string(), serde_json::json!("2.0"));

        let config = create_test_config(vars);
        let content = "# Guide v{{ book.version }}\n\nWelcome to version {{ book.version }}.";
        let result = process_nunjucks_templates(content, &config).unwrap();

        assert_eq!(result, "# Guide v2.0\n\nWelcome to version 2.0.");
    }

    #[test]
    fn test_conditional_markdown_sections() {
        let mut vars = HashMap::new();
        vars.insert("show_advanced".to_string(), serde_json::json!(true));

        let config = create_test_config(vars);
        let content = r#"## Basic Usage

This is basic content.

{% if book.show_advanced %}
## Advanced Usage

This is advanced content.
{% endif %}"#;
        let result = process_nunjucks_templates(content, &config).unwrap();

        assert!(result.contains("## Basic Usage"));
        assert!(result.contains("## Advanced Usage"));
        assert!(result.contains("This is advanced content."));
    }
}

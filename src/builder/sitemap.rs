use crate::parser::{Summary, SummaryItem};

/// Options for sitemap generation
#[derive(Debug, Clone)]
pub struct SitemapOptions {
    pub columns: u8,
    pub depth: Option<u8>,
}

impl Default for SitemapOptions {
    fn default() -> Self {
        Self {
            columns: 3,
            depth: None,
        }
    }
}

impl SitemapOptions {
    pub fn parse(directive: &str) -> Self {
        let mut options = Self::default();

        let content = directive
            .trim_start_matches("{{!SITEMAP")
            .trim_end_matches("}}")
            .trim();

        for part in content.split_whitespace() {
            if let Some(value) = part.strip_prefix("columns=") {
                if let Ok(cols) = value.parse::<u8>() {
                    options.columns = cols.clamp(1, 6);
                }
            } else if let Some(value) = part.strip_prefix("depth=") {
                if let Ok(d) = value.parse::<u8>() {
                    options.depth = Some(d);
                }
            }
        }

        options
    }
}

/// Generate sitemap HTML - mirrors SUMMARY.md structure
pub fn generate_sitemap_html(summary: &Summary, options: &SitemapOptions) -> String {
    let mut html = String::new();

    // Only add inline style if columns is explicitly set to non-default value
    // Default (3) uses CSS media queries for responsive behavior
    if options.columns == 3 {
        html.push_str(r#"<div class="sitemap">"#);
    } else {
        html.push_str(&format!(
            r#"<div class="sitemap" style="column-count: {};">"#,
            options.columns
        ));
    }
    html.push('\n');

    for item in &summary.items {
        render_item(&mut html, item, 1, options.depth);
    }

    html.push_str("</div>\n");
    html
}

/// Render an item and its children
fn render_item(html: &mut String, item: &SummaryItem, depth: u8, max_depth: Option<u8>) {
    if let Some(max) = max_depth {
        if depth > max {
            return;
        }
    }

    match item {
        SummaryItem::Link {
            title,
            path,
            children,
        } => {
            // Top-level items become sections
            if depth == 1 {
                html.push_str(r#"<div class="sitemap-section">"#);
                html.push('\n');
            }

            // Render the item title
            let class = match depth {
                1 => "sitemap-h1",
                2 => "sitemap-h2",
                3 => "sitemap-h3",
                _ => "sitemap-h4",
            };

            if let Some(p) = path {
                let html_path = to_html_path(p);
                html.push_str(&format!(
                    r#"<div class="{}"><a href="{}">{}</a></div>"#,
                    class,
                    html_path,
                    html_escape(title)
                ));
            } else {
                html.push_str(&format!(
                    r#"<div class="{}">{}</div>"#,
                    class,
                    html_escape(title)
                ));
            }
            html.push('\n');

            // Render children
            if !children.is_empty() {
                render_children(html, children, depth + 1, max_depth);
            }

            if depth == 1 {
                html.push_str("</div>\n");
            }
        }
        SummaryItem::PartTitle(title) => {
            html.push_str(&format!(
                r#"<div class="sitemap-part">{}</div>"#,
                html_escape(title)
            ));
            html.push('\n');
        }
        SummaryItem::Separator => {}
    }
}

/// Render children - leaf items inline with |, branch items as tree
fn render_children(html: &mut String, children: &[SummaryItem], depth: u8, max_depth: Option<u8>) {
    if let Some(max) = max_depth {
        if depth > max {
            return;
        }
    }

    // Separate leaf items (no children) from branch items (have children)
    let mut leaf_items: Vec<(&str, Option<&str>)> = Vec::new();
    let mut branch_items: Vec<&SummaryItem> = Vec::new();

    for child in children {
        if let SummaryItem::Link {
            title,
            path,
            children: grandchildren,
        } = child
        {
            if grandchildren.is_empty() {
                leaf_items.push((title, path.as_deref()));
            } else {
                branch_items.push(child);
            }
        }
    }

    html.push_str(r#"<div class="sitemap-children">"#);
    html.push('\n');

    // Render leaf items inline with |
    if !leaf_items.is_empty() {
        html.push_str(r#"<div class="sitemap-inline">"#);
        for (i, (title, path)) in leaf_items.iter().enumerate() {
            if let Some(p) = path {
                let html_path = to_html_path(p);
                html.push_str(&format!(
                    r#"<a href="{}">{}</a>"#,
                    html_path,
                    html_escape(title)
                ));
            } else {
                html.push_str(&html_escape(title));
            }
            if i < leaf_items.len() - 1 {
                html.push_str(r#" <span class="sitemap-sep">|</span> "#);
            }
        }
        html.push_str("</div>\n");
    }

    // Render branch items as tree
    for branch in branch_items {
        render_item(html, branch, depth, max_depth);
    }

    html.push_str("</div>\n");
}

fn to_html_path(path: &str) -> String {
    path.replace(".md", ".html")
        .replace(".adoc", ".html")
        .replace(".asciidoc", ".html")
        .trim_start_matches('/')
        .to_string()
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Process sitemap directives in HTML content
pub fn process_sitemap_directives(content: &str, summary: &Summary) -> String {
    let mut result = content.to_string();

    let mut search_start = 0;
    while let Some(start) = result[search_start..].find("{{!SITEMAP") {
        let abs_start = search_start + start;

        if let Some(end_offset) = result[abs_start..].find("}}") {
            let abs_end = abs_start + end_offset + 2;
            let directive = &result[abs_start..abs_end];

            let options = SitemapOptions::parse(directive);
            let sitemap_html = generate_sitemap_html(summary, &options);

            result.replace_range(abs_start..abs_end, &sitemap_html);
            search_start = abs_start + sitemap_html.len();
        } else {
            search_start = abs_start + 10;
        }
    }

    result
}

/// Get the default CSS for sitemap styling
pub fn get_sitemap_css() -> &'static str {
    r#"
/* Sitemap Styles */
.sitemap {
  column-count: 3;
  column-gap: 24px;
}

@media (max-width: 1200px) {
  .sitemap { column-count: 2; }
}

@media (max-width: 768px) {
  .sitemap { column-count: 1; }
}

.sitemap-part {
  column-span: all;
  font-size: 16px;
  font-weight: bold;
  color: #333;
  border-bottom: 2px solid #333;
  padding: 8px 0 4px 0;
  margin: 16px 0 8px 0;
}

.sitemap-section {
  background: var(--sitemap-section-bg, #f8f9fa);
  border-radius: 8px;
  padding: 12px;
  margin-bottom: 12px;
  box-shadow: 0 1px 3px rgba(0,0,0,0.08);
}

.sitemap-h1 {
  font-size: 14px;
  font-weight: 600;
  color: var(--sitemap-h1-color, #333);
  margin-bottom: 6px;
  padding-bottom: 4px;
  border-bottom: 1px solid #1a73e8;
}

.sitemap-h1 a { color: var(--sitemap-link-color, #1a73e8); text-decoration: none; }
.sitemap-h1 a:hover { text-decoration: underline; }

.sitemap-h2 {
  font-size: 13px;
  font-weight: 600;
  color: var(--sitemap-h2-color, #333);
  margin: 4px 0 2px 0;
}

.sitemap-h2 a { color: inherit; text-decoration: none; }
.sitemap-h2 a:hover { text-decoration: underline; }

.sitemap-h3 {
  font-size: 12px;
  font-weight: 500;
  color: var(--sitemap-h3-color, #555);
  margin: 2px 0;
}

.sitemap-h3 a { color: var(--sitemap-link-color, #1a73e8); text-decoration: none; }
.sitemap-h3 a:hover { text-decoration: underline; }

.sitemap-h4 {
  font-size: 11px;
  color: var(--sitemap-h4-color, #666);
  margin: 1px 0;
}

.sitemap-h4 a { color: var(--sitemap-link-color, #1a73e8); text-decoration: none; }
.sitemap-h4 a:hover { text-decoration: underline; }

.sitemap-children {
  margin-left: 12px;
  border-left: 1px solid #ddd;
  padding-left: 8px;
}

.sitemap-inline {
  font-size: 12px;
  line-height: 1.6;
  color: #333;
  word-wrap: break-word;
  overflow-wrap: break-word;
}

.sitemap-inline a {
  color: var(--sitemap-link-color, #1a73e8);
  text-decoration: none;
}

.sitemap-inline a:hover {
  text-decoration: underline;
}

.sitemap-sep {
  color: #666;
}
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_options_default() {
        let options = SitemapOptions::parse("{{!SITEMAP}}");
        assert_eq!(options.columns, 3);
        assert_eq!(options.depth, None);
    }
}

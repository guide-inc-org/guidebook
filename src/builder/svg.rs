//! SVG processing utilities for HTML optimization
//!
//! This module provides two main functions:
//! - `externalize_inline_svg`: Extracts inline SVGs to separate files for better caching
//! - `inline_svg_files`: Inlines SVG files into HTML for fewer HTTP requests
//!
//! Icon SVGs (with `fill="currentColor"`) are skipped to preserve their dynamic behavior.

use anyhow::Result;
use regex::Regex;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

static WIDTH_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"width\s*=\s*["']([^"']+)["']"#).unwrap());
static HEIGHT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"height\s*=\s*["']([^"']+)["']"#).unwrap());
// Attribute presence checks anchored to a preceding delimiter so that
// stroke-width= / data-width= do not count as a width attribute
static HAS_WIDTH_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"[\s"']width\s*=\s*["']"#).unwrap());
static HAS_HEIGHT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"[\s"']height\s*=\s*["']"#).unwrap());

/// Check if an SVG is an icon (has fill="currentColor")
/// Icon SVGs should be kept inline to preserve their dynamic color behavior
fn is_icon_svg(svg_content: &str) -> bool {
    svg_content.contains(r#"fill="currentColor""#) || svg_content.contains(r#"fill='currentColor'"#)
}

/// Generate a unique filename for an externalized SVG
fn generate_svg_filename(index: usize, output_dir: &Path) -> String {
    let svg_dir = output_dir.join("assets").join("svg");
    let filename = format!("inline-{}.svg", index);

    // Ensure the directory exists
    let _ = fs::create_dir_all(&svg_dir);

    format!("assets/svg/{}", filename)
}

/// Find the next inline `<svg>...</svg>` element, handling NESTED `<svg>`
/// tags (a naive non-greedy regex stops at the first `</svg>` and cuts a
/// nested element in half, emitting invalid unclosed XML).
///
/// Returns byte offsets `(start, open_tag_end, element_end)`.
fn find_svg_element(html: &str) -> Option<(usize, usize, usize)> {
    let mut search = 0usize;

    loop {
        let rel = html[search..].find("<svg")?;
        let start = search + rel;

        // Must be a real svg tag: "<svg" followed by whitespace, '>' or '/'
        let after = html[start + 4..].chars().next();
        if !matches!(after, Some(c) if c.is_whitespace() || c == '>' || c == '/') {
            search = start + 4;
            continue;
        }

        // Locate the end of the opening tag
        let open_tag_end = match html[start..].find('>') {
            Some(p) => start + p + 1,
            None => return None,
        };

        // Self-closing <svg ... />
        if html[start..open_tag_end]
            .trim_end_matches('>')
            .trim_end()
            .ends_with('/')
        {
            return Some((start, open_tag_end, open_tag_end));
        }

        // Scan forward for the matching close, tracking nesting depth
        let mut depth = 1usize;
        let mut pos = open_tag_end;
        while depth > 0 {
            let next_open = html[pos..].find("<svg").and_then(|o| {
                let abs = pos + o;
                let ch = html[abs + 4..].chars().next();
                matches!(ch, Some(c) if c.is_whitespace() || c == '>' || c == '/').then_some(abs)
            });
            let next_close = html[pos..].find("</svg>").map(|c| pos + c);

            match (next_open, next_close) {
                (Some(o), Some(c)) if o < c => {
                    // Nested opener (self-closing nested tags stay depth-neutral)
                    let inner_end = html[o..].find('>').map(|p| o + p + 1).unwrap_or(html.len());
                    if !html[o..inner_end]
                        .trim_end_matches('>')
                        .trim_end()
                        .ends_with('/')
                    {
                        depth += 1;
                    }
                    pos = inner_end;
                }
                (_, Some(c)) => {
                    depth -= 1;
                    pos = c + "</svg>".len();
                }
                // Unclosed element — treat as not-an-element, keep HTML as-is
                _ => return None,
            }
        }

        return Some((start, open_tag_end, pos));
    }
}

/// Externalize inline SVGs to separate files
///
/// Finds all inline `<svg>...</svg>` elements in the HTML, writes them to separate files,
/// and replaces them with `<img src="...">` tags.
///
/// SVGs with `fill="currentColor"` (icon SVGs) are skipped to preserve their dynamic behavior.
///
/// # Arguments
/// * `html` - The HTML content to process
/// * `output_dir` - The directory where SVG files will be written
/// * `root_prefix` - Relative prefix from the page to the output root
///   ("./" at root, "../../" at depth 2) so nested pages resolve the file
///
/// # Returns
/// The modified HTML with inline SVGs replaced by img tags
pub fn externalize_inline_svg(html: &str, output_dir: &Path, root_prefix: &str) -> Result<String> {
    let mut result = String::new();
    let mut rest = html;
    let mut svg_index = 0;

    while let Some((start, open_tag_end, end)) = find_svg_element(rest) {
        result.push_str(&rest[..start]);
        let svg_content = &rest[start..end];
        // Attributes sit between "<svg" and the closing '>' of the opening tag
        let svg_attrs = rest[start + 4..open_tag_end - 1].trim_end_matches('/');

        if is_icon_svg(svg_content) {
            result.push_str(svg_content);
        } else {
            // Generate filename and path
            let relative_path = generate_svg_filename(svg_index, output_dir);
            let svg_file_path = output_dir.join(&relative_path);

            // Ensure parent directory exists
            if let Some(parent) = svg_file_path.parent() {
                fs::create_dir_all(parent)?;
            }

            // Write SVG content to file
            fs::write(&svg_file_path, svg_content)?;

            // Extract width and height from SVG attributes if present
            let width = WIDTH_REGEX.captures(svg_attrs).map(|c| c[1].to_string());
            let height = HEIGHT_REGEX.captures(svg_attrs).map(|c| c[1].to_string());

            // Build replacement img tag
            let mut img_tag = format!(r#"<img src="{}{}""#, root_prefix, relative_path);
            if let Some(w) = width {
                img_tag.push_str(&format!(r#" width="{}""#, w));
            }
            if let Some(h) = height {
                img_tag.push_str(&format!(r#" height="{}""#, h));
            }
            img_tag.push_str(r#" alt="SVG image">"#);

            result.push_str(&img_tag);
            svg_index += 1;
        }

        rest = &rest[end..];
    }

    result.push_str(rest);
    Ok(result)
}

/// Inline SVG files into HTML
///
/// Finds all `<img src="...svg">` tags and replaces them with the inline SVG content.
/// This reduces HTTP requests by embedding SVGs directly in the HTML.
///
/// # Arguments
/// * `html` - The HTML content to process
/// * `base_dir` - The base directory for resolving relative SVG paths
///
/// # Returns
/// The modified HTML with img tags replaced by inline SVGs
pub fn inline_svg_files(html: &str, base_dir: &Path) -> Result<String> {
    // Regex to match img tags with SVG sources
    let img_regex = Regex::new(r#"<img([^>]+)src\s*=\s*["']([^"']+\.svg)["']([^>]*)>"#)?;

    let mut result = html.to_string();
    let mut offset: i64 = 0;

    for caps in img_regex.captures_iter(html) {
        let full_match = caps.get(0).unwrap();
        let before_src = &caps[1];
        let svg_path = &caps[2];
        let after_src = &caps[3];

        // Resolve the SVG file path
        let svg_file_path = base_dir.join(svg_path);

        // Read SVG content if file exists
        let svg_content = match fs::read_to_string(&svg_file_path) {
            Ok(content) => content,
            Err(_) => {
                // File not found, skip this replacement
                continue;
            }
        };

        // Skip icon SVGs (keep as img tags)
        if is_icon_svg(&svg_content) {
            continue;
        }

        // Extract width and height from img tag attributes
        let attrs = format!("{}{}", before_src, after_src);
        let width = WIDTH_REGEX.captures(&attrs).map(|c| c[1].to_string());
        let height = HEIGHT_REGEX.captures(&attrs).map(|c| c[1].to_string());

        // Modify SVG to include width/height if specified in img tag.
        // Only the ROOT opening tag counts — a naive whole-file contains()
        // check false-positives on child elements (<rect width=...>) and the
        // img tag's size is then silently dropped
        let mut modified_svg = svg_content.clone();
        let root_tag_end = modified_svg.find('>').map(|p| p + 1).unwrap_or(0);
        let root_has_width = HAS_WIDTH_REGEX.is_match(&modified_svg[..root_tag_end]);
        let root_has_height = HAS_HEIGHT_REGEX.is_match(&modified_svg[..root_tag_end]);
        if let Some(w) = width {
            if !root_has_width {
                modified_svg = modified_svg.replacen("<svg", &format!(r#"<svg width="{}""#, w), 1);
            }
        }
        if let Some(h) = height {
            if !root_has_height {
                modified_svg = modified_svg.replacen("<svg", &format!(r#"<svg height="{}""#, h), 1);
            }
        }

        // Calculate adjusted positions
        let start = (full_match.start() as i64 + offset) as usize;
        let end = (full_match.end() as i64 + offset) as usize;

        // Replace in result
        result.replace_range(start..end, &modified_svg);

        // Update offset
        offset += modified_svg.len() as i64 - (full_match.end() - full_match.start()) as i64;
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_is_icon_svg() {
        assert!(is_icon_svg(r#"<svg fill="currentColor"><path/></svg>"#));
        assert!(is_icon_svg(r#"<svg fill='currentColor'><path/></svg>"#));
        assert!(!is_icon_svg(r#"<svg fill="blue"><path/></svg>"#));
        assert!(!is_icon_svg(r#"<svg><path/></svg>"#));
    }

    #[test]
    fn test_externalize_inline_svg() {
        let temp_dir = tempdir().unwrap();
        let output_dir = temp_dir.path();

        let html = r#"<html><body>
<svg width="100" height="100"><circle cx="50" cy="50" r="40"/></svg>
<p>Some text</p>
</body></html>"#;

        let result = externalize_inline_svg(html, output_dir, "./").unwrap();

        // Should replace SVG with img tag
        assert!(result.contains(r#"<img src="./assets/svg/inline-0.svg""#));
        assert!(result.contains(r#"width="100""#));
        assert!(result.contains(r#"height="100""#));
        assert!(!result.contains("<circle"));

        // SVG file should be created
        let svg_file = output_dir.join("assets/svg/inline-0.svg");
        assert!(svg_file.exists());
        let svg_content = fs::read_to_string(svg_file).unwrap();
        assert!(svg_content.contains("<circle"));
    }

    #[test]
    fn test_externalize_skips_icon_svg() {
        let temp_dir = tempdir().unwrap();
        let output_dir = temp_dir.path();

        let html = r#"<html><body>
<svg fill="currentColor"><path d="M10 10"/></svg>
</body></html>"#;

        let result = externalize_inline_svg(html, output_dir, "./").unwrap();

        // Icon SVG should remain inline
        assert!(result.contains(r#"fill="currentColor""#));
        assert!(result.contains("<svg"));
        assert!(!result.contains("<img"));
    }

    #[test]
    fn test_inline_svg_files() {
        let temp_dir = tempdir().unwrap();
        let base_dir = temp_dir.path();

        // Create a test SVG file
        let svg_content = r#"<svg viewBox="0 0 100 100"><rect width="100" height="100"/></svg>"#;
        fs::write(base_dir.join("test.svg"), svg_content).unwrap();

        let html = r#"<html><body>
<img src="test.svg" alt="Test">
</body></html>"#;

        let result = inline_svg_files(html, base_dir).unwrap();

        // Should inline the SVG
        assert!(result.contains("<svg viewBox"));
        assert!(result.contains("<rect"));
        assert!(!result.contains("<img"));
    }

    #[test]
    fn test_inline_svg_skips_icon() {
        let temp_dir = tempdir().unwrap();
        let base_dir = temp_dir.path();

        // Create an icon SVG file
        let svg_content = r#"<svg fill="currentColor"><path d="M10 10"/></svg>"#;
        fs::write(base_dir.join("icon.svg"), svg_content).unwrap();

        let html = r#"<html><body>
<img src="icon.svg" alt="Icon">
</body></html>"#;

        let result = inline_svg_files(html, base_dir).unwrap();

        // Icon SVG should remain as img tag
        assert!(result.contains("<img"));
        assert!(result.contains(r#"src="icon.svg""#));
        assert!(!result.contains(r#"fill="currentColor""#));
    }

    #[test]
    fn test_inline_svg_missing_file() {
        let temp_dir = tempdir().unwrap();
        let base_dir = temp_dir.path();

        let html = r#"<html><body>
<img src="nonexistent.svg" alt="Missing">
</body></html>"#;

        let result = inline_svg_files(html, base_dir).unwrap();

        // Should keep img tag unchanged when file doesn't exist
        assert!(result.contains("<img"));
        assert!(result.contains("nonexistent.svg"));
    }

    #[test]
    fn test_externalize_multiple_svgs() {
        let temp_dir = tempdir().unwrap();
        let output_dir = temp_dir.path();

        let html = r#"<html><body>
<svg id="svg1"><circle r="10"/></svg>
<p>Text between</p>
<svg id="svg2"><rect width="20"/></svg>
</body></html>"#;

        let result = externalize_inline_svg(html, output_dir, "./").unwrap();

        // Both SVGs should be externalized
        assert!(result.contains("inline-0.svg"));
        assert!(result.contains("inline-1.svg"));
        assert!(!result.contains("<circle"));
        assert!(!result.contains("<rect"));
    }

    #[test]
    fn test_externalize_nested_svg_kept_whole() {
        // Regression: a non-greedy regex cut nested <svg> elements at the
        // FIRST </svg>, writing unclosed XML and leaving debris in the HTML
        let temp_dir = tempdir().unwrap();
        let output_dir = temp_dir.path();

        let html = r#"<svg width="200" height="200" viewBox="0 0 200 200">
  <svg x="10" y="10" width="50" height="50"><rect width="50" height="50"/></svg>
  <circle cx="100" cy="100" r="80"/>
</svg>"#;

        let result = externalize_inline_svg(html, output_dir, "./").unwrap();

        // Exactly one img tag; no leftover svg debris in the HTML
        assert_eq!(result.matches("<img").count(), 1, "{}", result);
        assert!(!result.contains("</svg>"), "no debris: {}", result);
        assert!(!result.contains("<circle"), "{}", result);

        // The written file must contain the WHOLE element (balanced tags)
        let svg_content = fs::read_to_string(output_dir.join("assets/svg/inline-0.svg")).unwrap();
        assert_eq!(svg_content.matches("<svg").count(), 2);
        assert_eq!(svg_content.matches("</svg>").count(), 2);
        assert!(svg_content.contains("<circle"));
    }

    #[test]
    fn test_externalize_nested_page_prefix() {
        let temp_dir = tempdir().unwrap();
        let html = r#"<svg width="10"><rect width="10"/></svg>"#;
        let result = externalize_inline_svg(html, temp_dir.path(), "../../").unwrap();
        assert!(
            result.contains(r#"<img src="../../assets/svg/inline-0.svg""#),
            "{}",
            result
        );
    }

    #[test]
    fn test_inline_svg_child_width_does_not_block_root_injection() {
        // Regression: a whole-file contains("width=") check false-positived
        // on child elements, so the img tag's size never got applied
        let temp_dir = tempdir().unwrap();
        let base_dir = temp_dir.path();

        let svg_content = r#"<svg viewBox="0 0 400 300"><rect width="400" height="300"/></svg>"#;
        fs::write(base_dir.join("chart.svg"), svg_content).unwrap();

        let html = r#"<img src="chart.svg" width="100" height="75">"#;
        let result = inline_svg_files(html, base_dir).unwrap();

        assert!(
            result.contains(r#"<svg width="100""#)
                || result.contains(r#"<svg height="75" width="100""#),
            "img size must be injected into the root svg tag: {}",
            result
        );
    }
}

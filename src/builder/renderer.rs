use percent_encoding::percent_decode_str;
use pulldown_cmark::{html, CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use std::cell::RefCell;
use std::collections::{BTreeSet, HashSet};
use std::path::Path;

/// Table of Contents item
#[derive(Debug, Clone)]
pub struct TocItem {
    pub level: u8,
    pub text: String,
    pub id: String,
}

/// Extract headings from markdown content for TOC generation
pub fn extract_headings(content: &str) -> Vec<TocItem> {
    let content = apply_honkit_source_rules(content);

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    let parser = parse_with_honkit_inline_rules(&content, options);

    let mut headings = Vec::new();
    let mut in_heading: Option<HeadingLevel> = None;
    let mut heading_text = String::new();
    let mut custom_heading_id: Option<String> = None;
    // Must mirror the slug assignment in render_markdown_internal exactly
    // (dedup counters consume slugs for ALL heading levels, not just h2-h4)
    let mut used_slugs: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for event in parser {
        match &event {
            Event::Start(Tag::Heading { level, id, .. }) => {
                in_heading = Some(*level);
                heading_text.clear();
                custom_heading_id = id.as_ref().map(|s| s.to_string());
            }
            Event::Text(text) if in_heading.is_some() => {
                heading_text.push_str(text);
            }
            Event::End(TagEnd::Heading(level)) if in_heading.is_some() => {
                let level_num = heading_level_to_num(*level);
                let id = custom_heading_id
                    .take()
                    .unwrap_or_else(|| dedupe_slug(slugify(&heading_text), &mut used_slugs));
                // Only include h2, h3, h4 in TOC (skip h1 which is page title)
                if (2..=4).contains(&level_num) {
                    headings.push(TocItem {
                        level: level_num,
                        text: heading_text.clone(),
                        id,
                    });
                }
                in_heading = None;
            }
            _ => {}
        }
    }

    headings
}

/// The pages of a site, for reading the `.md` links on them the way HonKit does: a link to a
/// page goes to its `.html`. A link to any other file stays as it is, and the file is
/// copied. All paths are relative to the root of the site, and a language's prefix is a part
/// of them.
#[derive(Default)]
pub struct BookLinks {
    /// The source paths of the pages
    pages: HashSet<String>,
    /// The source paths of the READMEs that only have an index.html: not in the summary
    homes_only: HashSet<String>,
    /// The `.md` files that the pages link to and that are not pages
    linked_files: RefCell<BTreeSet<String>>,
}

impl BookLinks {
    /// Add a page, by its path from the root of the site
    pub fn add_page(&mut self, path: &str) {
        if let Some(path) = normalize_site_path(path) {
            self.homes_only.remove(&path);
            self.pages.insert(path);
        }
    }

    /// Add the README of a book, which HonKit always builds as its index.html
    pub fn add_home(&mut self, path: &str) {
        if let Some(path) = normalize_site_path(path) {
            if !self.pages.contains(&path) {
                self.homes_only.insert(path);
            }
        }
    }

    /// The files that the pages rendered so far link to and that are not pages
    pub fn linked_files(&self) -> Vec<String> {
        self.linked_files.borrow().iter().cloned().collect()
    }

    /// What a `.md` link on the page at current_path becomes
    fn resolve(&self, current_path: &str, url: &str) -> String {
        let Some(file) = md_link_file(url) else {
            return url.to_string();
        };
        let current_dir = current_path.rsplit_once('/').map_or("", |(dir, _)| dir);
        let target = match file.strip_prefix('/') {
            Some(from_root) => from_root.to_string(),
            None => format!("{}/{}", current_dir, file),
        };
        let Some(target) = normalize_site_path(&target) else {
            return url.to_string();
        };

        if self.pages.contains(&target) {
            return fix_md_extension(url);
        }
        if self.homes_only.contains(&target) {
            return url.replacen("README.md", "index.html", 1);
        }
        self.linked_files.borrow_mut().insert(target);
        url.to_string()
    }
}

/// The file that a link with a `.md` file goes to, decoded, or None for any other link
fn md_link_file(url: &str) -> Option<String> {
    if has_url_scheme(url) || url.starts_with("//") {
        return None;
    }
    let file = url.split(['#', '?']).next().unwrap_or("");
    if !file.ends_with(".md") {
        return None;
    }
    let file = file
        .replace("&amp;", "&")
        .replace("&#39;", "'")
        .replace("&#x27;", "'");
    let file = percent_decode_str(&file).decode_utf8().ok()?;
    Some(file.into_owned())
}

/// Whether a URL starts with a scheme ("https:", "mailto:", ...)
pub(crate) fn has_url_scheme(url: &str) -> bool {
    let Some((scheme, _)) = url.split_once(':') else {
        return false;
    };
    scheme.starts_with(|c: char| c.is_ascii_alphabetic())
        && scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
}

/// Resolve the `.` and `..` of a path from the root of the site. None if it goes out of it.
pub(crate) fn normalize_site_path(path: &str) -> Option<String> {
    let mut parts = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            _ => parts.push(part),
        }
    }
    Some(parts.join("/"))
}

/// Render a page of a site: like render_markdown_with_path, with the links read by links
pub fn render_book_page(
    content: &str,
    current_path: &str,
    hardbreaks: bool,
    links: &BookLinks,
) -> String {
    let content = content.replace("\r\n", "\n").replace("\r", "\n");
    let html = render_markdown_internal(&content, hardbreaks, Some((links, current_path)));
    convert_relative_links_to_absolute(&html, current_path)
}

/// Render the README of a book for its index.html, with the links read by links
pub fn render_book_home(
    content: &str,
    current_path: &str,
    hardbreaks: bool,
    links: &BookLinks,
) -> String {
    let content = content.replace("\r\n", "\n").replace("\r", "\n");
    render_markdown_internal(&content, hardbreaks, Some((links, current_path)))
}

/// Render markdown content to HTML with Mermaid support
/// current_path: the path of the current markdown file (e.g., "Customer/AssetStatus/PortfolioTop.md")
/// hardbreaks: when true, treat single newlines as hard breaks (<br>)
#[cfg(test)]
pub fn render_markdown_with_path(
    content: &str,
    current_path: Option<&str>,
    hardbreaks: bool,
) -> String {
    // Normalize CRLF/CR to LF for consistent line handling
    let content = content.replace("\r\n", "\n").replace("\r", "\n");
    let html = render_markdown_internal(&content, hardbreaks, None);

    // If we have a current path, convert relative links to absolute
    if let Some(path) = current_path {
        convert_relative_links_to_absolute(&html, path)
    } else {
        html
    }
}

/// Render markdown content to HTML (backward compatible)
pub fn render_markdown(content: &str) -> String {
    // Normalize CRLF/CR to LF for consistent line handling
    let content = content.replace("\r\n", "\n").replace("\r", "\n");
    render_markdown_internal(&content, false, None)
}

fn render_markdown_internal(
    content: &str,
    hardbreaks: bool,
    book_links: Option<(&BookLinks, &str)>,
) -> String {
    // Strip all UTF-8 BOM characters (fixes reference link parsing issues)
    // BOM can appear at start of file or in concatenated content from @import
    let content = content.replace('\u{FEFF}', "");

    // Convert footnote definitions to placeholders (preserve original position). First,
    // because the fixes below ask the parser how it reads each line, and the lines after a
    // definition's first line only read as they will once they are dedented.
    let content = convert_footnote_definitions_inline(&content);

    // Preprocess: fix image paths with spaces
    let content = fix_image_paths_with_spaces(&content);
    // Preprocess: read the source the way HonKit does
    let content = apply_honkit_source_rules(&content);
    // Preprocess: let the last definition of a reference label win
    let content = keep_last_reference_definitions(&content);
    // Preprocess: fix malformed table rows
    let content = fix_table_separator_columns(&content);
    // Preprocess: end tables where HonKit does
    let content = end_tables_at_lines_without_pipes(&content);

    // Convert footnote references [^n] to placeholders BEFORE markdown parsing
    // This prevents [A][^1] from being interpreted as a markdown link reference
    let content = convert_footnote_references_to_placeholder(&content);

    let parser = parse_with_honkit_inline_rules(&content, render_options());
    let parser = apply_honkit_list_looseness(parser, &honkit_loose_items(&content));

    // Process events to handle mermaid code blocks and heading IDs
    let mut in_mermaid = false;
    let mut mermaid_content = String::new();
    let mut in_heading: Option<HeadingLevel> = None;
    let mut heading_text = String::new();
    let mut custom_heading_id: Option<String> = None; // Store custom ID from {#id} syntax
                                                      // Track used slugs so duplicate headings get unique ids (-1, -2, ...)
    let mut used_slugs: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut events: Vec<Event> = Vec::new();

    for event in parser {
        match &event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
                let lang_str = lang.as_ref();
                if lang_str == "mermaid" || lang_str.starts_with("mermaid") {
                    in_mermaid = true;
                    mermaid_content.clear();
                    continue;
                }
            }
            Event::End(TagEnd::CodeBlock) if in_mermaid => {
                // Output mermaid div instead of code block
                let mermaid_html = format!(
                    r#"<div class="mermaid">{}</div>"#,
                    html_escape(&mermaid_content)
                );
                events.push(Event::Html(mermaid_html.into()));
                in_mermaid = false;
                continue;
            }
            Event::Text(text) if in_mermaid => {
                mermaid_content.push_str(text);
                continue;
            }
            // Track heading start and capture custom ID from {#id} syntax
            Event::Start(Tag::Heading { level, id, .. }) => {
                in_heading = Some(*level);
                heading_text.clear();
                // Capture custom ID if provided via {#custom-id} syntax
                custom_heading_id = id.as_ref().map(|s| s.to_string());
                events.push(event.clone());
                continue;
            }
            // Capture heading text
            Event::Text(text) if in_heading.is_some() => {
                heading_text.push_str(text);
                events.push(event.clone());
                continue;
            }
            // End of heading: inject ID
            Event::End(TagEnd::Heading(level)) if in_heading.is_some() => {
                // Use custom ID if provided, otherwise generate from heading text
                // (deduplicated: same text yields id, id-1, id-2, ...)
                let id = custom_heading_id
                    .take()
                    .unwrap_or_else(|| dedupe_slug(slugify(&heading_text), &mut used_slugs));
                let level_num = heading_level_to_num(*level);
                // Pop the heading content and rebuild with ID
                let mut heading_events = Vec::new();
                while let Some(ev) = events.pop() {
                    if matches!(ev, Event::Start(Tag::Heading { .. })) {
                        break;
                    }
                    heading_events.push(ev);
                }
                heading_events.reverse();

                // Push heading with ID as raw HTML
                let open_tag = format!(r#"<h{} id="{}">"#, level_num, id);
                events.push(Event::Html(open_tag.into()));
                events.extend(heading_events);
                events.push(Event::Html(format!("</h{}>", level_num).into()));

                in_heading = None;
                continue;
            }
            // Convert soft breaks to hard breaks when hardbreaks option is enabled
            Event::SoftBreak if hardbreaks => {
                events.push(Event::HardBreak);
                continue;
            }
            _ => {}
        }
        events.push(event);
    }

    let mut html_output = String::new();
    html::push_html(&mut html_output, events.into_iter());

    // Convert footnote placeholders to HTML. Before autolinking, which would take a
    // placeholder right after a URL for the rest of the URL.
    html_output = convert_footnote_placeholders_to_html(&html_output);

    // Fix relative links: convert .md to .html
    html_output = match book_links {
        Some((links, current_path)) => {
            rewrite_hrefs(&html_output, |url| links.resolve(current_path, url))
        }
        None => fix_relative_links(&html_output),
    };

    // Note: Root-relative links (starting with /) are handled in convert_relative_links_to_absolute()
    // which has access to the current file path and can calculate the correct relative path

    // Auto-link URLs that are not already linked
    html_output = autolink_urls(&html_output);

    // Add target="_blank" to external links (Markdown-style links like [text](https://...))
    html_output = add_target_blank_to_external_links(&html_output);

    // Convert any remaining markdown images inside HTML blocks to <img> tags
    html_output = convert_remaining_markdown_images(&html_output);

    html_output
}

fn render_options() -> Options {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    // Don't use pulldown-cmark's footnote processing - we handle it ourselves
    // options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    options
}

/// Rewrite the source where HonKit reads it differently from CommonMark, in the ways that
/// can change headings. Shared by rendering and TOC extraction so that they agree.
fn apply_honkit_source_rules(content: &str) -> String {
    // First: a comment that goes changes how the lines after it are read
    let content = remove_comments_between_list_items(content);
    let content = keep_lines_in_open_list_items(&content);
    let content = keep_inline_tags_in_paragraphs(&content);
    let content = fix_heading_marker_spaces(&content);
    let content = keep_braces_at_heading_ends(&content);
    fix_link_destinations_with_spaces(&content)
}

/// Parse markdown into events, following HonKit where its inline rules differ from
/// CommonMark:
/// - Only `~~` strikes through: a single `~` stays literal ("0~100 and 0~100")
/// - `__bold__` also works next to a letter (see find_intraword_strong)
fn parse_with_honkit_inline_rules(content: &str, options: Options) -> Vec<Event<'_>> {
    let mut events = Vec::new();
    // For each open strikethrough, whether it was written with a single `~`
    let mut strikethroughs = Vec::new();
    // Adjacent text, merged: pulldown-cmark splits it at every unresolved delimiter
    let mut text_run = TextRun::default();
    // Inside a code block or an image description, text is not inline markdown
    let mut verbatim_depth = 0;

    for (event, range) in Parser::new_ext(content, options).into_offset_iter() {
        match event {
            Event::Start(Tag::Strikethrough) => {
                let single = !content[range.clone()].starts_with("~~");
                strikethroughs.push(single);
                if single {
                    text_run.push("~", range.start..range.start + 1);
                } else {
                    text_run.flush_into(&mut events, content);
                    events.push(Event::Start(Tag::Strikethrough));
                }
            }
            Event::End(TagEnd::Strikethrough) => {
                if strikethroughs.pop() == Some(true) {
                    text_run.push("~", range.end - 1..range.end);
                } else {
                    text_run.flush_into(&mut events, content);
                    events.push(Event::End(TagEnd::Strikethrough));
                }
            }
            Event::Text(text) if verbatim_depth == 0 => text_run.push(&text, range),
            event => {
                text_run.flush_into(&mut events, content);
                match &event {
                    Event::Start(Tag::CodeBlock(_) | Tag::Image { .. }) => verbatim_depth += 1,
                    Event::End(TagEnd::CodeBlock | TagEnd::Image) => verbatim_depth -= 1,
                    _ => {}
                }
                events.push(event);
            }
        }
    }
    text_run.flush_into(&mut events, content);

    events
}

/// For every list item of the document, in order, whether HonKit puts its text in a
/// paragraph. HonKit decides item by item, CommonMark for the whole list: an item is loose
/// when it has a blank line inside (one in a nested list counts), ends with one, or comes
/// after an item that ended with one. The last item of a list is not loose for its end.
///
/// None leaves it to the parser: in a block quote, the blank lines have markers on them.
fn honkit_loose_items(content: &str) -> Vec<Option<bool>> {
    let mut loose_items = Vec::new();
    // For each open list: its items so far, as an index into loose_items and a source range
    let mut lists: Vec<Vec<(usize, std::ops::Range<usize>)>> = Vec::new();
    let mut quote_depth = 0;

    for (event, range) in Parser::new_ext(content, render_options()).into_offset_iter() {
        match event {
            Event::Start(Tag::BlockQuote) => quote_depth += 1,
            Event::End(TagEnd::BlockQuote) => quote_depth -= 1,
            Event::Start(Tag::List(_)) => lists.push(Vec::new()),
            Event::Start(Tag::Item) => {
                if let Some(items) = lists.last_mut() {
                    items.push((loose_items.len(), range));
                }
                loose_items.push(None);
            }
            Event::End(TagEnd::List(_)) => {
                let items = lists.pop().unwrap_or_default();
                if quote_depth > 0 {
                    continue;
                }
                let mut after_blank_line = false;
                for (i, (index, range)) in items.iter().enumerate() {
                    let next_item = items.get(i + 1);
                    let end = next_item.map_or(range.end, |(_, next)| next.start);
                    let item = blank_out_whitespace_lines(&content[range.start..end]);

                    let mut loose = after_blank_line
                        || item
                            .match_indices("\n\n")
                            .any(|(at, _)| !item[at + 2..].trim().is_empty());
                    if next_item.is_some() {
                        after_blank_line = item.ends_with("\n\n");
                        loose |= after_blank_line;
                    }
                    loose_items[*index] = Some(loose);
                }
            }
            _ => {}
        }
    }

    loose_items
}

/// Empty the lines that hold nothing but spaces, tabs and no-break spaces: HonKit reads
/// those as blank lines before it looks at anything else
fn blank_out_whitespace_lines(text: &str) -> String {
    text.split('\n')
        .map(|line| {
            if line.chars().all(|c| matches!(c, ' ' | '\t' | '\u{a0}')) {
                ""
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Put the text of the list items in paragraphs, or take it out of them, the way HonKit
/// does. See honkit_loose_items.
fn apply_honkit_list_looseness<'a>(
    events: Vec<Event<'a>>,
    loose_items: &[Option<bool>],
) -> Vec<Event<'a>> {
    #[derive(PartialEq)]
    enum Open {
        Item(Option<bool>),
        RemovedParagraph,
        Other,
    }

    let mut result = Vec::with_capacity(events.len());
    let mut open: Vec<Open> = Vec::new();
    let mut loose_items = loose_items.iter().copied();
    let mut in_added_paragraph = false;

    for event in events {
        if open.last() == Some(&Open::Item(Some(true))) {
            let inline = is_inline_event(&event);
            if inline != in_added_paragraph {
                result.push(if inline {
                    Event::Start(Tag::Paragraph)
                } else {
                    Event::End(TagEnd::Paragraph)
                });
                in_added_paragraph = inline;
            }
        }
        match &event {
            Event::Start(Tag::Item) => open.push(Open::Item(loose_items.next().flatten())),
            Event::Start(Tag::Paragraph) if open.last() == Some(&Open::Item(Some(false))) => {
                open.push(Open::RemovedParagraph);
                continue;
            }
            Event::Start(_) => open.push(Open::Other),
            Event::End(_) => {
                let closed = open.pop();
                if closed == Some(Open::RemovedParagraph) {
                    continue;
                }
            }
            _ => {}
        }
        result.push(event);
    }

    result
}

/// Whether the event can be a part of a paragraph
fn is_inline_event(event: &Event) -> bool {
    match event {
        Event::Start(
            Tag::Emphasis | Tag::Strong | Tag::Strikethrough | Tag::Link { .. } | Tag::Image { .. },
        ) => true,
        Event::Start(_) | Event::End(_) | Event::Rule => false,
        _ => true,
    }
}

/// Adjacent text events on their way to becoming one
#[derive(Default)]
struct TextRun {
    text: String,
    source_start: Option<usize>,
    source_end: usize,
}

impl TextRun {
    fn push(&mut self, text: &str, source: std::ops::Range<usize>) {
        self.text.push_str(text);
        self.source_start.get_or_insert(source.start);
        self.source_end = source.end;
    }

    fn flush_into<'a>(&mut self, events: &mut Vec<Event<'a>>, content: &str) {
        let Some(source_start) = self.source_start.take() else {
            return;
        };
        let text = std::mem::take(&mut self.text);
        let mut rest = text.as_str();

        // A backslash-escaped underscore looks the same as a delimiter once parsed
        if !content[source_start..self.source_end].contains("\\_") {
            while let Some((open, close)) = find_intraword_strong(rest) {
                if open > 0 {
                    events.push(Event::Text(rest[..open].to_string().into()));
                }
                events.push(Event::Start(Tag::Strong));
                events.push(Event::Text(rest[open + 2..close].to_string().into()));
                events.push(Event::End(TagEnd::Strong));
                rest = &rest[close + 2..];
            }
        }

        if !rest.is_empty() {
            events.push(Event::Text(rest.to_string().into()));
        }
    }
}

/// Find the next `__bold__` in text that CommonMark left literal because a delimiter
/// touches a letter ("これは__太字__です"). HonKit renders it bold. Only when non-ASCII
/// text is involved: identifiers like SOME__NAME__HERE and URLs stay literal.
/// Returns the byte offsets of the opening and the closing delimiter.
fn find_intraword_strong(text: &str) -> Option<(usize, usize)> {
    let mut from = 0;

    while let Some(found) = text[from..].find("__") {
        let open = from + found;
        let inner = open + 2;
        let before = text[..open].chars().last();
        let first = text[inner..].chars().next();

        // An opening delimiter is not part of a longer run and is followed by text
        if before == Some('_') || first.is_none_or(|c| c == '_' || c.is_whitespace()) {
            from = inner;
            continue;
        }

        let close = find_closing_strong(text, inner)?;
        // The words the delimiters touch
        let word_before = text[..open]
            .rsplit(char::is_whitespace)
            .next()
            .unwrap_or("");
        let word_after = text[close + 2..]
            .split(char::is_whitespace)
            .next()
            .unwrap_or("");
        let is_ascii =
            word_before.is_ascii() && text[inner..close].is_ascii() && word_after.is_ascii();
        if !is_ascii && !word_before.contains("://") {
            return Some((open, close));
        }
        from = close + 2;
    }

    None
}

/// Find the `__` closing a bold that starts at inner: it follows text and is not part
/// of a longer run
fn find_closing_strong(text: &str, inner: usize) -> Option<usize> {
    let mut from = inner;

    while let Some(found) = text[from..].find("__") {
        let close = from + found;
        let follows_text = close > inner && !text[..close].ends_with(char::is_whitespace);
        if follows_text && !text[close + 2..].starts_with('_') {
            return Some(close);
        }
        from = close + 1;
    }

    None
}

/// How the parser reads a line of the source
#[derive(Clone, Copy, PartialEq, Eq)]
enum LineKind {
    Markdown,
    Code,
    /// The first line of a raw HTML block
    RawHtmlStart,
    /// A later line of a raw HTML block
    RawHtml,
}

/// How the parser reads each line of a document. The fixes that work on the text before it
/// is parsed ask this so that they only change what will be read as Markdown: nothing in a
/// code block or a raw HTML block (<style>, <script>, <pre>, a comment, ...) is.
fn line_kinds(content: &str) -> Vec<LineKind> {
    let mut line_starts = Vec::new();
    let mut offset = 0;
    for line in content.split_inclusive('\n') {
        line_starts.push(offset);
        offset += line.len();
    }

    let mut kinds = vec![LineKind::Markdown; line_starts.len()];
    for (event, range) in Parser::new_ext(content, render_options()).into_offset_iter() {
        let kind = match event {
            Event::Start(Tag::CodeBlock(_)) => LineKind::Code,
            Event::Start(Tag::HtmlBlock) => LineKind::RawHtml,
            _ => continue,
        };
        let first = line_starts.partition_point(|&start| start <= range.start) - 1;
        let end = line_starts.partition_point(|&start| start < range.end);
        kinds[first..end].fill(kind);
        if kind == LineKind::RawHtml {
            kinds[first] = LineKind::RawHtmlStart;
        }
    }

    kinds
}

/// Tells which lines of a footnote's continuation belong to a fenced code block. The parser
/// cannot tell yet: these lines are still indented, and the fence only counts once they
/// are dedented.
#[derive(Default)]
struct FenceTracker {
    /// (fence_char, fence_length) while inside a fenced code block
    open: Option<(char, usize)>,
}

impl FenceTracker {
    fn is_open(&self) -> bool {
        self.open.is_some()
    }

    /// Feed the next line. Returns true if it belongs to a fenced code block, the
    /// opening and closing fences included.
    fn is_code_line(&mut self, line: &str) -> bool {
        let trimmed = line.trim_start();

        if let Some((fence_char, fence_len)) = self.open {
            // Closing fence: a run of the same char, at least as long, alone on the line
            let run = trimmed.chars().take_while(|&c| c == fence_char).count();
            if run >= fence_len && trimmed.trim_end().chars().all(|c| c == fence_char) {
                self.open = None;
            }
            return true;
        }

        let fence_char = match trimmed.chars().next() {
            Some(c @ ('`' | '~')) => c,
            _ => return false,
        };
        let fence_len = trimmed.chars().take_while(|&c| c == fence_char).count();
        // The info string of a backtick fence has no backticks: "```x```" is inline code
        if fence_len < 3 || (fence_char == '`' && trimmed[fence_len..].contains('`')) {
            return false;
        }

        self.open = Some((fence_char, fence_len));
        true
    }
}

fn heading_level_to_num(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

/// Generate a URL-safe slug from text (matching github-slugger / HonKit behavior)
fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .filter_map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                Some(c)
            } else if c.is_whitespace() {
                Some('-')
            } else if c > '\x7F' {
                // Keep non-ASCII characters (Japanese, etc.)
                Some(c)
            } else {
                // Remove other special characters (/, ., etc.) to match github-slugger
                None
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Deduplicate a slug against previously used ones (github-slugger behavior:
/// "foo", "foo-1", "foo-2", ...). Skips suffixes that collide with slugs
/// that literally occurred earlier.
fn dedupe_slug(slug: String, used: &mut std::collections::HashMap<String, usize>) -> String {
    if !used.contains_key(&slug) {
        used.insert(slug.clone(), 0);
        return slug;
    }

    let mut n = used[&slug] + 1;
    let mut candidate = format!("{}-{}", slug, n);
    while used.contains_key(&candidate) {
        n += 1;
        candidate = format!("{}-{}", slug, n);
    }
    used.insert(slug, n);
    used.insert(candidate.clone(), 0);
    candidate
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Convert footnote definitions in place (HonKit style): the first line becomes a
/// blockquote holding the number and the back link; the lines after it stay in the
/// document, so they see its reference definitions and are rendered like any other block.
///
/// The first line is wrapped in placeholders rather than HTML because an HTML block
/// would not be parsed as markdown. See convert_footnote_placeholders_to_html.
fn convert_footnote_definitions_inline(content: &str) -> String {
    let mut result_lines = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let kinds = line_kinds(content);
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let definition = match kinds[i] {
            LineKind::Markdown => parse_footnote_def_start(line),
            _ => None,
        };
        i += 1;

        // Check if this line starts a footnote definition [^n]:
        let Some((number, first_line_content)) = definition else {
            result_lines.push(line.to_string());
            continue;
        };

        // Collect continuation lines (indented or list items until next footnote/heading/blank)
        let mut continuation_lines: Vec<&str> = Vec::new();
        let mut fence = FenceTracker::default();
        while i < lines.len() {
            let next_line = lines[i];
            let trimmed = next_line.trim_start();
            let is_raw_html = matches!(kinds[i], LineKind::RawHtmlStart | LineKind::RawHtml);

            // Stop if: empty line, new footnote, heading (a fenced code block runs to its
            // fence, and a raw HTML block to its end)
            if !fence.is_open()
                && !is_raw_html
                && (trimmed.is_empty()
                    || (trimmed.starts_with("[^") && trimmed.contains("]:"))
                    || trimmed.starts_with('#'))
            {
                break;
            }

            if !is_raw_html {
                fence.is_code_line(next_line);
            }
            continuation_lines.push(next_line);
            i += 1;
        }

        // Blank lines keep the placeholders in a paragraph of their own
        result_lines.push(String::new());
        result_lines.push(format!(
            "%%FNDEF_OPEN_{}%%{}%%FNDEF_CLOSE_{}%%",
            number,
            first_line_content.trim_end(),
            number
        ));
        result_lines.push(String::new());
        if !continuation_lines.is_empty() {
            result_lines.extend(dedent_lines(&continuation_lines));
            result_lines.push(String::new());
        }
    }

    result_lines.join("\n")
}

/// Remove the indentation the lines have in common, preserving relative structure
fn dedent_lines(lines: &[&str]) -> Vec<String> {
    let min_indent = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start().len())
        .min()
        .unwrap_or(0);

    lines
        .iter()
        .map(|line| {
            line.get(min_indent..)
                .unwrap_or(line.trim_start())
                .to_string()
        })
        .collect()
}

/// Convert footnote references [^n] to placeholder (before parsing)
/// Placeholder format: %%FNREF_n%% - will be converted to HTML after markdown parsing
///
/// Skips code and raw HTML so that literal text like the regex character class `[^abc]`
/// is not turned into a footnote reference.
fn convert_footnote_references_to_placeholder(content: &str) -> String {
    let defined = footnote_definition_labels(content);
    map_markdown_text(content, |text| {
        convert_footnote_refs_in_text(text, &defined)
    })
}

/// The labels of the footnote definitions, which are placeholders by now
fn footnote_definition_labels(content: &str) -> HashSet<&str> {
    let prefix = "%%FNDEF_OPEN_";
    content
        .match_indices(prefix)
        .filter_map(|(start, _)| {
            let rest = &content[start + prefix.len()..];
            rest.find("%%").map(|end| &rest[..end])
        })
        .collect()
}

/// Whether [^label] is a footnote reference. A label of letters and digits always is. One
/// with symbols, like `3-1`, is only when the document defines it: `[^a-z]` in a sentence
/// is more likely a character class.
fn is_footnote_reference(label: &str, defined: &HashSet<&str>) -> bool {
    if label.is_empty() {
        return false;
    }
    label.chars().all(char::is_alphanumeric)
        || (label
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | '.'))
            && defined.contains(label))
}

/// Apply a fix to the text of a document that the parser reads as Markdown: outside code
/// blocks, raw HTML blocks and inline code spans
fn map_markdown_text(content: &str, fix: impl Fn(&str) -> String) -> String {
    let mut result = String::new();

    for (line, kind) in content.lines().zip(line_kinds(content)) {
        if kind != LineKind::Markdown {
            result.push_str(line);
        } else {
            let mut rest = line;
            while let Some((start, end)) = find_inline_code_span(rest) {
                result.push_str(&fix(&rest[..start]));
                result.push_str(&rest[start..end]); // code span verbatim
                rest = &rest[end..];
            }
            result.push_str(&fix(rest));
        }
        result.push('\n');
    }

    // lines() drops the trailing newline info; restore original ending
    if !content.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }

    result
}

/// Find the next inline code span (a run of N backticks closed by an
/// equal-length run, per CommonMark). Returns the byte range including
/// the backticks, or None if no closed span exists.
fn find_inline_code_span(s: &str) -> Option<(usize, usize)> {
    let bytes = s.as_bytes();
    let mut open = s.find('`')?;

    loop {
        let open_len = bytes[open..].iter().take_while(|&&b| b == b'`').count();
        // Search for a closing run of exactly open_len backticks
        let mut idx = open + open_len;
        while idx < bytes.len() {
            if bytes[idx] == b'`' {
                let run_start = idx;
                while idx < bytes.len() && bytes[idx] == b'`' {
                    idx += 1;
                }
                if idx - run_start == open_len {
                    return Some((open, idx));
                }
            } else {
                idx += 1;
            }
        }
        // This opener never closes; try the next backtick run after it
        let next = s[open + open_len..].find('`')?;
        open = open + open_len + next;
    }
}

/// Convert footnote references [^n] to placeholders in plain (non-code) text
fn convert_footnote_refs_in_text(content: &str, defined: &HashSet<&str>) -> String {
    let mut result = String::new();
    let mut chars = content.char_indices().peekable();

    while let Some((i, c)) = chars.next() {
        if c == '[' && content[i..].starts_with("[^") {
            // Find the closing ]
            let rest = &content[i + 2..];
            if let Some(end) = rest.find(']') {
                let number = &rest[..end];
                // Make sure it's a reference (not a definition - no : after ])
                let after = &rest[end + 1..];
                if !after.starts_with(':') && is_footnote_reference(number, defined) {
                    // This is a reference, convert to placeholder
                    result.push_str(&format!("%%FNREF_{}%%", number));
                    // Skip past the reference: ^number]
                    // We already consumed '[', so skip: ^ + number + ]
                    for _ in 0..(1 + end + 1) {
                        chars.next();
                    }
                    continue;
                }
            }
        }
        result.push(c);
    }

    result
}

/// Convert footnote placeholders to HTML (after markdown parsing)
fn convert_footnote_placeholders_to_html(html: &str) -> String {
    // A definition's first line is a paragraph of its own: the blockquote replaces the <p>.
    // Use blockquote to match HonKit styling (left border)
    let html = replace_delimited(html, "<p>%%FNDEF_OPEN_", "%%", |number| {
        format!("<blockquote id=\"fn_{}\"><sup>{}</sup>. ", number, number)
    });
    let html = replace_delimited(&html, "%%FNDEF_CLOSE_", "%%</p>", |number| {
        format!(
            "<a href=\"#reffn_{}\" title=\"Jump back to footnote [{}] in the text.\"> ↩</a></blockquote>",
            number, number
        )
    });
    // A definition that did not end up in a paragraph of its own: give the source back
    let html = replace_delimited(&html, "%%FNDEF_OPEN_", "%%", |number| {
        format!("[^{}]: ", number)
    });
    let html = replace_delimited(&html, "%%FNDEF_CLOSE_", "%%", |_| String::new());
    replace_delimited(&html, "%%FNREF_", "%%", |number| {
        format!(
            "<sup><a href=\"#fn_{}\" id=\"reffn_{}\">{}</a></sup>",
            number, number, number
        )
    })
}

/// Replace every prefix + name + suffix in the text with render(name)
fn replace_delimited(
    text: &str,
    prefix: &str,
    suffix: &str,
    render: impl Fn(&str) -> String,
) -> String {
    let mut result = String::with_capacity(text.len());
    let mut rest = text;

    while let Some(start) = rest.find(prefix) {
        let after_prefix = &rest[start + prefix.len()..];
        let Some(end) = after_prefix.find(suffix) else {
            break;
        };
        // The suffix found belongs to a later placeholder: this one has another suffix
        if after_prefix[..end].contains("%%") {
            result.push_str(&rest[..start + prefix.len()]);
            rest = after_prefix;
            continue;
        }
        result.push_str(&rest[..start]);
        result.push_str(&render(&after_prefix[..end]));
        rest = &after_prefix[end + suffix.len()..];
    }

    result.push_str(rest);
    result
}

/// Parse a footnote definition start line, returns (number, rest_of_line)
fn parse_footnote_def_start(line: &str) -> Option<(&str, &str)> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with("[^") {
        return None;
    }

    // Find the closing ]
    let after_bracket = &trimmed[2..];
    let end_bracket = after_bracket.find("]:")?;
    let number = &after_bracket[..end_bracket];

    // Get the content after ]:
    let rest = &after_bracket[end_bracket + 2..].trim_start();
    Some((number, rest))
}

/// Fix malformed table rows:
/// - Add missing trailing | to header rows
/// - Fix separator rows where column count doesn't match header
/// - Widen the header to the widest row (a table drops the cells beyond its
///   header; HonKit keeps them)
fn fix_table_separator_columns(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let kinds = line_kinds(content);
    let mut result = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        // A table starts at a row (starts with |) followed by a separator row
        if kinds[i] == LineKind::Markdown
            && line.trim().starts_with('|')
            && lines.get(i + 1).is_some_and(|l| is_table_separator_row(l))
        {
            let separator = lines[i + 1];
            let fixed_header = fix_table_row_trailing_pipe(line);
            let header_cols = count_table_columns(&fixed_header);
            let separator_cols = count_table_columns(separator);

            let body_rows = lines[i + 2..]
                .iter()
                .take_while(|row| row.trim().starts_with('|'))
                .count();
            let widest_row = lines[i + 2..i + 2 + body_rows]
                .iter()
                .map(|row| count_filled_table_columns(row))
                .max()
                .unwrap_or(0);
            let cols = header_cols.max(widest_row);

            // An empty cell for each column the header is short of
            result.push(format!(
                "{}{}",
                fixed_header,
                "|".repeat(cols - header_cols)
            ));

            // If column counts don't match, fix the separator row
            if cols > 0 && separator_cols != cols {
                let indent = &separator[..separator.len() - separator.trim_start().len()];
                result.push(format!(
                    "{}{}",
                    indent,
                    generate_separator_row(cols, separator)
                ));
            } else {
                result.push(separator.to_string());
            }

            i += 2;
            continue;
        }

        result.push(line.to_string());
        i += 1;
    }

    result.join("\n")
}

/// End a table at the first line HonKit does not read as a row, so that a note right under
/// a table stays a note. CommonMark's extension runs a table up to the blank line and makes
/// a row of any line on the way. HonKit wants a pipe: at the start of the line in a table
/// with outer pipes, anywhere in one without.
///
/// A line with rows after it stays in the table: HonKit would show those rows as text.
///
/// Only at the top level: a blank line would make a list item loose, and HonKit has no
/// tables in lists to follow.
fn end_tables_at_lines_without_pipes(content: &str) -> String {
    let mut table_ends = Vec::new();
    let mut depth = 0;

    for (event, range) in Parser::new_ext(content, render_options()).into_offset_iter() {
        match event {
            Event::Start(tag) => {
                if depth == 0 && matches!(tag, Tag::Table(_)) {
                    table_ends.extend(first_line_after_table_rows(content, range));
                }
                depth += 1;
            }
            Event::End(_) => depth -= 1,
            _ => {}
        }
    }

    let mut result = String::with_capacity(content.len() + table_ends.len());
    let mut copied = 0;
    for end in table_ends {
        result.push_str(&content[copied..end]);
        result.push('\n');
        copied = end;
    }
    result.push_str(&content[copied..]);
    result
}

/// Where the line of a table that ends it for HonKit starts, if there is one
fn first_line_after_table_rows(content: &str, table: std::ops::Range<usize>) -> Option<usize> {
    let mut lines = content[table.clone()].split_inclusive('\n').peekable();
    let header = lines.next()?;
    let separator = lines.next()?;
    let outer_pipes =
        header.trim_start().starts_with('|') && separator.trim_start().starts_with('|');
    let is_row = |line: &str| {
        if outer_pipes {
            line.trim_start().starts_with('|')
        } else {
            line.contains('|')
        }
    };

    let mut start = table.start + header.len() + separator.len();
    while let Some(line) = lines.next() {
        if !is_row(line) && !lines.peek().is_some_and(|next| is_row(next)) {
            return Some(start);
        }
        start += line.len();
    }
    None
}

/// Add trailing pipe to table row if missing
fn fix_table_row_trailing_pipe(line: &str) -> String {
    let trimmed = line.trim();
    if trimmed.starts_with('|') && !trimmed.ends_with('|') {
        format!("{}|", line)
    } else {
        line.to_string()
    }
}

/// Split a table row into its cells. An escaped pipe does not separate cells, and the
/// trailing pipe is optional.
fn table_cells(line: &str) -> Vec<&str> {
    let Some(row) = line.trim().strip_prefix('|') else {
        return Vec::new();
    };

    let mut cells = Vec::new();
    let mut start = 0;
    let mut escaped = false;
    for (i, c) in row.char_indices() {
        if c == '|' && !escaped {
            cells.push(&row[start..i]);
            start = i + 1;
        }
        escaped = c == '\\' && !escaped;
    }
    if start < row.len() {
        cells.push(&row[start..]);
    }

    cells
}

/// Count the number of columns in a table row
fn count_table_columns(line: &str) -> usize {
    table_cells(line).len()
}

/// Count the columns of a table row up to its last cell with content
fn count_filled_table_columns(line: &str) -> usize {
    table_cells(line)
        .iter()
        .rposition(|cell| !cell.trim().is_empty())
        .map_or(0, |last| last + 1)
}

/// Check if a line is a table separator row (contains only |, -, :, and whitespace)
fn is_table_separator_row(line: &str) -> bool {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
        return false;
    }

    // Must contain at least one dash
    if !trimmed.contains('-') {
        return false;
    }

    // All characters must be |, -, :, or whitespace
    trimmed
        .chars()
        .all(|c| c == '|' || c == '-' || c == ':' || c.is_whitespace())
}

/// Generate a separator row with the specified number of columns
/// Preserves per-column alignment from the original separator
fn generate_separator_row(col_count: usize, original: &str) -> String {
    // Parse alignments from original separator row
    let trimmed = original.trim();
    let original_alignments: Vec<&str> = trimmed
        .trim_start_matches('|')
        .trim_end_matches('|')
        .split('|')
        .map(|cell| {
            let cell = cell.trim();
            if cell.starts_with(':') && cell.ends_with(':') {
                ":--:" // center
            } else if cell.starts_with(':') {
                ":--" // left (explicit)
            } else if cell.ends_with(':') {
                "--:" // right
            } else {
                "--" // left (default)
            }
        })
        .collect();

    // Build new separator with correct number of columns
    // Use original alignments where available, default to "--" for extra columns
    let cols: Vec<&str> = (0..col_count)
        .map(|i| {
            if i < original_alignments.len() {
                original_alignments[i]
            } else {
                "--" // default alignment for extra columns
            }
        })
        .collect();

    format!("|{}|", cols.join("|"))
}

/// Fix the space after heading markers (common mistakes in Japanese documents):
/// - Full-width: converts "##　見出し" to "## 見出し"
/// - Missing: converts "##見出し" to "## 見出し" (HonKit does not need the space)
///
/// A missing space is only fixed when text can follow, so that the mask
/// "####-####-####" stays literal, and not on lines indented as far as a code block:
/// the space would not make them a heading.
fn fix_heading_marker_spaces(content: &str) -> String {
    content
        .lines()
        .zip(line_kinds(content))
        .map(|(line, kind)| {
            if kind != LineKind::Markdown {
                return line.to_string();
            }

            // Only spaces and tabs indent: a line after a full-width space is text
            let trimmed = line.trim_start_matches([' ', '\t']);
            let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
            if !(1..=6).contains(&hash_count) {
                return line.to_string();
            }

            let leading_whitespace = &line[..line.len() - trimmed.len()];
            let after_hashes = &trimmed[hash_count..];
            let marker = "#".repeat(hash_count);

            if let Some(rest) = after_hashes.strip_prefix('\u{3000}') {
                return format!("{}{} {}", leading_whitespace, marker, rest);
            }

            let starts_text = after_hashes.chars().next().is_some_and(|c| {
                !c.is_whitespace()
                    && (c.is_alphanumeric()
                        || !c.is_ascii()
                        || "[(*_`<".contains(c)
                        || (c == '-' && !after_hashes.chars().all(|c| matches!(c, '-' | '#'))))
            });
            if starts_text && indent_width(leading_whitespace) <= 3 {
                return format!("{}{} {}", leading_whitespace, marker, after_hashes);
            }

            line.to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The width of a line's indentation: a tab goes to the next multiple of 4
fn indent_width(indentation: &str) -> usize {
    indentation.chars().fold(0, |width, c| match c {
        '\t' => width + 4 - width % 4,
        _ => width + 1,
    })
}

/// `{id}` at the end of a heading is a part of its text in HonKit, which only reads
/// `{#id}`. pulldown-cmark reads it as an attribute and takes it out of the text, as in
/// "GET /users/{id}". Headings with an id, a class or a `key=value` are left to it.
fn keep_braces_at_heading_ends(content: &str) -> String {
    let mut closing_braces = Vec::new();
    for (event, range) in Parser::new_ext(content, render_options()).into_offset_iter() {
        let Event::Start(Tag::Heading {
            id, classes, attrs, ..
        }) = &event
        else {
            continue;
        };
        if id.is_some() || !classes.is_empty() || attrs.iter().any(|(_, value)| value.is_some()) {
            continue;
        }
        if let Some(at) = heading_closing_brace(&content[range.clone()]) {
            closing_braces.push(range.start + at);
        }
    }

    let mut result = content.to_string();
    for at in closing_braces.into_iter().rev() {
        result.replace_range(at..at + 1, "&#125;");
    }
    result
}

/// The offset of the `}` that ends the text of a heading's source, if one does
fn heading_closing_brace(heading: &str) -> Option<usize> {
    let mut text = heading.trim_end();
    // A setext heading: the text is above the underline
    if let Some((above, underline)) = text.rsplit_once('\n') {
        let underline = underline.trim();
        if !underline.is_empty() && underline.chars().all(|c| matches!(c, '=' | '-')) {
            text = above.trim_end();
        }
    }
    text.ends_with('}').then(|| text.len() - 1)
}

/// The tags that HonKit reads as a part of a text. See keep_inline_tags_in_paragraphs. Not
/// "code", which HonKit reads the same way: what a code element holds stays as it is written.
const HONKIT_INLINE_TAGS: &[&str] = &[
    "a", "em", "strong", "small", "s", "cite", "q", "dfn", "abbr", "data", "time", "var", "samp",
    "kbd", "sub", "sup", "i", "b", "u", "mark", "ruby", "rt", "rp", "bdi", "bdo", "span", "br",
    "wbr", "ins", "del", "img",
];

/// Whether a line holds one of HONKIT_INLINE_TAGS and nothing else
fn is_lone_inline_tag(line: &str) -> bool {
    let line = line.trim();
    let Some(tag) = line.strip_prefix('<').and_then(|l| l.strip_suffix('>')) else {
        return false;
    };
    if tag.contains(['<', '>']) {
        return false;
    }
    let name = tag.strip_prefix('/').unwrap_or(tag);
    let name_end = name
        .find(|c: char| !c.is_ascii_alphanumeric())
        .unwrap_or(name.len());
    HONKIT_INLINE_TAGS.contains(&name[..name_end].to_ascii_lowercase().as_str())
}

/// A line with nothing but an inline tag, like `<span class="x">`, starts a paragraph in
/// HonKit. In CommonMark it starts an HTML block, and the lines up to the next blank line
/// are not read as Markdown. Anything after the tag on its line keeps the line from starting
/// a block; an empty comment does not show.
fn keep_inline_tags_in_paragraphs(content: &str) -> String {
    let mut line_ends = Vec::new();
    for (event, range) in Parser::new_ext(content, render_options()).into_offset_iter() {
        if !matches!(event, Event::Start(Tag::HtmlBlock)) {
            continue;
        }
        let block = &content[range.clone()];
        let first_line = block.lines().next().unwrap_or("");
        if is_lone_inline_tag(first_line) {
            line_ends.push(range.start + first_line.len());
        }
    }

    let mut result = content.to_string();
    for at in line_ends.into_iter().rev() {
        result.insert_str(at, "<!-- -->");
    }
    result
}

/// Whether an HTML block that starts with this line can go on after a blank line: a comment,
/// `<pre>`, `<script>`, `<style>`, `<textarea>`, `<!...>` or `<?...>`
fn html_block_crosses_blank_lines(trimmed: &str) -> bool {
    let lower = trimmed.to_ascii_lowercase();
    lower.starts_with("<!")
        || lower.starts_with("<?")
        || ["<pre", "<script", "<style", "<textarea"]
            .iter()
            .any(|tag| lower.starts_with(tag))
}

/// The column where the text of a list item starts, given where its marker starts
fn list_item_text_column(indent: usize, trimmed: &str) -> usize {
    let marker = trimmed.find([' ', '\t']).unwrap_or(trimmed.len());
    let after_marker = &trimmed[marker..];
    let rest = after_marker.trim_start_matches([' ', '\t']);
    let spaces = indent_width(&after_marker[..after_marker.len() - rest.len()]);
    indent + marker + if (1..=4).contains(&spaces) { spaces } else { 1 }
}

/// A list item that is open while keep_lines_in_open_list_items reads the source
struct OpenListItem {
    /// The columns of the marker and the text, as written
    marker: usize,
    text: usize,
    /// How far the item is moved
    shift: usize,
}

/// Where a list item goes among the open items, the way HonKit reads the indentation. Each
/// list strips the indentation of its items from their lines and reads what is left, so:
/// - A marker at the column of an open item is the next item of that item's list
/// - After a blank line, a marker with no indentation left ends the list at that level
/// - Any other marker is inside the deepest open item, however little it is indented
enum ListItemPlace {
    SiblingOf(usize),
    EndsListAt(usize),
    InsideDeepest,
}

fn list_item_place(open: &[OpenListItem], indent: usize, after_blank_line: bool) -> ListItemPlace {
    let mut stripped = 0;
    for (level, item) in open.iter().enumerate() {
        let left = indent.saturating_sub(stripped);
        if left == item.marker - stripped.min(item.marker) {
            return ListItemPlace::SiblingOf(level);
        }
        if after_blank_line && left == 0 {
            return ListItemPlace::EndsListAt(level);
        }
        stripped = item.text;
    }
    ListItemPlace::InsideDeepest
}

/// Open the list items that start on a line: "- 1. text" is an item in an item
fn push_list_items(open: &mut Vec<OpenListItem>, indent: usize, trimmed: &str, shift: usize) {
    let mut marker = indent;
    let mut rest = trimmed;
    loop {
        let text = list_item_text_column(marker, rest);
        open.push(OpenListItem {
            marker,
            text,
            shift,
        });
        let after_marker = rest.find([' ', '\t']).map_or("", |at| &rest[at..]);
        rest = after_marker.trim_start_matches([' ', '\t']);
        if !is_list_item(rest) {
            return;
        }
        marker = text;
    }
}

/// Close the items that a line after a blank line is not in: in each list, HonKit strips the
/// indentation of the items from their lines, and a line with no indentation left ends it
fn close_list_items_before(open: &mut Vec<OpenListItem>, indent: usize) {
    while !open.is_empty() {
        let around = open.len().checked_sub(2).map_or(0, |at| open[at].text);
        if indent > around {
            return;
        }
        open.pop();
    }
}

/// Whether a line, without its indentation, is a heading to HonKit: it needs no space after
/// the hashes
fn is_atx_heading_in_honkit(trimmed: &str) -> bool {
    let hashes = trimmed.chars().take_while(|&c| c == '#').count();
    (1..=6).contains(&hashes) && !trimmed[hashes..].trim().is_empty()
}

/// Indent the lines of the lists so that CommonMark reads the structure that HonKit does. The
/// two read indentation differently:
/// - After a blank line, HonKit keeps a line in the open item for as long as it is indented
///   more than the text of the item around that one. CommonMark also wants it indented as
///   far as the item's own text: the line ends the item, and the deeper items after it
///   become a code block.
/// - See ListItemPlace for the markers. CommonMark reads a marker that is indented a little
///   differently from the items before it as the next item of their list.
///
/// - HonKit keeps a heading or a code fence that follows the text of an item in the item.
///   In CommonMark, it ends the list if it is not indented as far as the item's text.
///
/// Lines that are indented as far as a code block stay what they are in CommonMark: a code
/// block after a blank line and the text going on otherwise, where HonKit makes a code block
/// of a line that was just indented too far.
fn keep_lines_in_open_list_items(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let kinds = line_kinds(content);
    let mut result = Vec::with_capacity(lines.len());
    let mut fence = FenceTracker::default();
    let mut open: Vec<OpenListItem> = Vec::new();
    let mut after_blank_line = false;
    // How far the lines since the last blank line are moved, on top of the item's shift
    let mut moved_by = 0;
    // How far the lines of the open code fence are moved, on top of the item's shift
    let mut fence_moved_by = 0;

    let reindent = |line: &str, by: usize| {
        if by == 0 {
            return line.to_string();
        }
        let trimmed = line.trim_start_matches([' ', '\t']);
        let indent = indent_width(&line[..line.len() - trimmed.len()]);
        format!("{}{}", " ".repeat(indent + by), trimmed)
    };

    for (i, &line) in lines.iter().enumerate() {
        let trimmed = line.trim_start_matches([' ', '\t']);
        let indent = indent_width(&line[..line.len() - trimmed.len()]);

        let fence_was_open = fence.is_open();
        if fence.is_code_line(line) {
            if !fence_was_open {
                if after_blank_line {
                    close_list_items_before(&mut open, indent);
                }
                fence_moved_by = open
                    .last()
                    .map_or(0, |item| item.text.saturating_sub(indent));
            }
            let shift = open.last().map_or(0, |item| item.shift);
            result.push(reindent(line, shift + fence_moved_by));
            after_blank_line = false;
            continue;
        }
        fence_moved_by = 0;

        if kinds[i] == LineKind::RawHtml && moved_by == 0 {
            let shift = open.last().map_or(0, |item| item.shift);
            result.push(reindent(line, shift));
            after_blank_line = false;
            continue;
        }

        if trimmed.is_empty() {
            result.push(line.to_string());
            after_blank_line = true;
            moved_by = 0;
            continue;
        }

        if is_list_item(trimmed) {
            moved_by = 0;
            let to = match list_item_place(&open, indent, after_blank_line) {
                ListItemPlace::SiblingOf(level) => {
                    let to = open[level].marker + open[level].shift;
                    open.truncate(level);
                    Some(to)
                }
                ListItemPlace::EndsListAt(level) => {
                    open.truncate(level);
                    Some(open.last().map_or(indent, |item| item.text + item.shift))
                }
                ListItemPlace::InsideDeepest => {
                    let (around, shift) =
                        open.last().map_or((0, 0), |item| (item.text, item.shift));
                    let is_code = indent >= around + 4;
                    (!is_code).then_some(around.max(indent) + shift)
                }
            };
            if let Some(to) = to {
                push_list_items(&mut open, indent, trimmed, to.saturating_sub(indent));
            }
        } else if moved_by == 0 && after_blank_line {
            close_list_items_before(&mut open, indent);
            let crosses_blank_lines =
                kinds[i] == LineKind::RawHtmlStart && html_block_crosses_blank_lines(trimmed);
            if let (Some(item), false) = (open.last(), crosses_blank_lines) {
                moved_by = item.text.saturating_sub(indent);
            }
        } else if moved_by == 0 && is_atx_heading_in_honkit(trimmed) {
            moved_by = open
                .last()
                .map_or(0, |item| item.text.saturating_sub(indent));
        }

        after_blank_line = false;
        let shift = open.last().map_or(0, |item| item.shift);
        result.push(reindent(line, shift + moved_by));
    }

    result.join("\n")
}

/// Whether a line, without its indentation, starts a list item
fn is_list_item(trimmed: &str) -> bool {
    let digits = trimmed.bytes().take_while(u8::is_ascii_digit).count();
    let after_marker = if digits > 0 {
        trimmed[digits..].strip_prefix(['.', ')'])
    } else {
        trimmed.strip_prefix(['-', '*', '+'])
    };
    after_marker.is_some_and(|rest| rest.starts_with([' ', '\t']))
}

/// Remove HTML comments between the lines of a list. HonKit keeps the list going. In
/// CommonMark the comment is an HTML block that ends the list item, and a deeper item
/// after it becomes a code block.
fn remove_comments_between_list_items(content: &str) -> String {
    let mut content = content.to_string();
    // A deeper item after a comment reads as code until the comment is gone, so a comment
    // after that item only shows on the next pass
    while let Some(fewer_lines) = remove_list_comments_in_one_pass(&content) {
        content = fewer_lines;
    }
    content
}

/// Returns the document without the comments found, or None if there are none
fn remove_list_comments_in_one_pass(content: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    let kinds = line_kinds(content);
    let mut result = Vec::with_capacity(lines.len());
    // Whether the lines since the last blank line include a list item
    let mut in_list = false;
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        if matches!(kinds[i], LineKind::Code | LineKind::RawHtml) {
            result.push(line);
            i += 1;
            continue;
        }

        let trimmed = line.trim();
        if in_list && trimmed.starts_with("<!--") {
            let close = (i..lines.len()).find(|&j| lines[j].contains("-->"));
            // A comment that ends its line, followed by another item. Anything else may
            // need the comment: a reference definition cannot follow the item's text.
            let removable = close.filter(|&j| {
                lines[j].trim_end().ends_with("-->")
                    && lines
                        .get(j + 1)
                        .is_some_and(|next| is_list_item(next.trim()))
            });
            if let Some(close) = removable {
                i = close + 1;
                continue;
            }
        }

        in_list = !trimmed.is_empty() && (in_list || is_list_item(trimmed));
        result.push(line);
        i += 1;
    }

    (result.len() < lines.len()).then(|| result.join("\n"))
}

/// Percent-encode the spaces and angle brackets of a link destination, the way HonKit
/// reads "[text](a b.md)". CommonMark would leave it as text.
fn fix_link_destinations_with_spaces(content: &str) -> String {
    let labels: HashSet<String> = content
        .lines()
        .filter_map(reference_definition_label)
        .collect();

    map_markdown_text(content, |text| {
        let mut result = String::with_capacity(text.len());
        let mut rest = text;

        while let Some(start) = rest.find("](") {
            let after_open = &rest[start + 2..];
            let Some(end) = after_open.find(')') else {
                break;
            };
            let destination = &after_open[..end];
            let (path, title) = split_link_title(destination);
            let path = path.trim();
            // "<a b>" is already fine, and a parenthesis may start a title
            let is_bare = !path.starts_with('<') && !path.contains('(');

            result.push_str(&rest[..start + 2]);
            // In "[text][label](a b)" with a defined label, HonKit and CommonMark read a
            // reference link, and the parentheses are text after it
            if is_bare && path.contains(' ') && !closes_reference_label(&rest[..start], &labels) {
                let encoded = path
                    .replace(' ', "%20")
                    .replace('<', "%3C")
                    .replace('>', "%3E");
                result.push_str(&encoded);
                result.push_str(title);
            } else {
                result.push_str(destination);
            }
            result.push(')');
            rest = &after_open[end + 1..];
        }

        result.push_str(rest);
        result
    })
}

/// Whether `before`, the text up to a `]`, ends in the `[label` of a `[text][label]` with a
/// defined label
fn closes_reference_label(before: &str, labels: &HashSet<String>) -> bool {
    let Some(open) = before.rfind('[') else {
        return false;
    };
    before[..open].ends_with(']') && labels.contains(&normalize_label(&before[open + 1..]))
}

/// Split `path "title"` into the path and the title (with the space before it). HonKit
/// reads either kind of quote as a title.
fn split_link_title(destination: &str) -> (&str, &str) {
    let end = destination.trim_end();
    let Some(quote) = end.chars().last().filter(|c| matches!(c, '"' | '\'')) else {
        return (destination, "");
    };
    let body = &end[..end.len() - 1];
    match body.find(&format!(" {quote}")) {
        Some(i) => destination.split_at(i),
        None => (destination, ""),
    }
}

/// Keep only the last definition of each reference label: HonKit lets the last one
/// win, CommonMark the first.
fn keep_last_reference_definitions(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let labels: Vec<Option<String>> = lines
        .iter()
        .zip(line_kinds(content))
        .map(|(line, kind)| match kind {
            LineKind::Markdown => reference_definition_label(line),
            _ => None,
        })
        .collect();
    let mut last_definition = std::collections::HashMap::new();
    for (i, label) in labels.iter().enumerate() {
        if let Some(label) = label {
            last_definition.insert(label.clone(), i);
        }
    }

    lines
        .iter()
        .zip(&labels)
        .enumerate()
        .filter(|(i, (_, label))| label.as_ref().is_none_or(|l| last_definition[l] == *i))
        .map(|(_, (line, _))| *line)
        .collect::<Vec<_>>()
        .join("\n")
}

/// The normalized label if the line is a reference definition ("[label]: url")
fn reference_definition_label(line: &str) -> Option<String> {
    let indent = line.len() - line.trim_start_matches(' ').len();
    if indent > 3 {
        return None;
    }

    let rest = line[indent..].strip_prefix('[')?;
    // Footnote definitions look the same
    if rest.starts_with('^') {
        return None;
    }

    let (label, destination) = rest.split_once("]:")?;
    if label.trim().is_empty()
        || label.contains(['[', ']'])
        || !is_reference_destination(destination)
    {
        return None;
    }

    Some(normalize_label(label))
}

/// A reference label in the form to compare it in
fn normalize_label(label: &str) -> String {
    label
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Whether the text after "[label]:" is a destination with an optional title. Any other
/// text makes the line an ordinary sentence ("[Note]: see the next page"), and two
/// sentences that start alike must both stay.
fn is_reference_destination(text: &str) -> bool {
    let text = text.trim();
    if text.is_empty() {
        return false;
    }

    let after_path = match text.strip_prefix('<') {
        Some(rest) => match rest.split_once('>') {
            Some((_, after_path)) => after_path,
            None => return false,
        },
        None => &text[text.find(char::is_whitespace).unwrap_or(text.len())..],
    };

    let title = after_path.trim_start();
    if title.is_empty() {
        return true;
    }
    let closing_quote = match title.chars().next() {
        Some('"') => '"',
        Some('\'') => '\'',
        Some('(') => ')',
        _ => return false,
    };
    after_path.starts_with(char::is_whitespace)
        && title.len() >= 2
        && title.ends_with(closing_quote)
}

/// Fix image paths that contain spaces by wrapping them in angle brackets
/// Converts ![alt](path with space.png) to ![alt](<path with space.png>)
/// In a raw HTML block, convert_remaining_markdown_images reads the path as it is.
fn fix_image_paths_with_spaces(content: &str) -> String {
    map_markdown_text(content, fix_image_paths_in_text)
}

fn fix_image_paths_in_text(content: &str) -> String {
    let mut result = String::new();
    let mut chars = content.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '!' {
            // Check for image syntax: ![...](...)
            if chars.peek() == Some(&'[') {
                // Collect the entire potential image syntax
                let mut img_str = String::from("!");
                img_str.push(chars.next().unwrap()); // '['

                // Read alt text until ']'
                let mut bracket_depth = 1;
                while let Some(&ch) = chars.peek() {
                    img_str.push(chars.next().unwrap());
                    if ch == '[' {
                        bracket_depth += 1;
                    } else if ch == ']' {
                        bracket_depth -= 1;
                        if bracket_depth == 0 {
                            break;
                        }
                    }
                }

                // Check for '(' after ']'
                if chars.peek() == Some(&'(') {
                    img_str.push(chars.next().unwrap()); // '('

                    // Read URL until ')'
                    let mut url = String::new();
                    let mut paren_depth = 1;
                    let mut closed = false;
                    while let Some(&ch) = chars.peek() {
                        if ch == '(' {
                            paren_depth += 1;
                            url.push(chars.next().unwrap());
                        } else if ch == ')' {
                            paren_depth -= 1;
                            if paren_depth == 0 {
                                chars.next(); // consume ')'
                                closed = true;
                                break;
                            }
                            url.push(chars.next().unwrap());
                        } else {
                            url.push(chars.next().unwrap());
                        }
                    }

                    // Check if the path contains spaces and doesn't already use angle brackets
                    let (path, title) = split_link_title(&url);
                    let path = path.trim();
                    if closed && path.contains(' ') && !path.starts_with('<') {
                        img_str.push('<');
                        img_str.push_str(path);
                        img_str.push('>');
                        img_str.push_str(title);
                    } else {
                        img_str.push_str(&url);
                    }
                    if closed {
                        img_str.push(')');
                    }
                }

                result.push_str(&img_str);
            } else {
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }

    result
}

fn fix_relative_links(html: &str) -> String {
    rewrite_hrefs(html, fix_md_extension)
}

fn rewrite_hrefs(html: &str, rewrite: impl Fn(&str) -> String) -> String {
    // Replace .md links with .html, but ONLY inside href attribute values.
    // A blind string replace would also rewrite occurrences in visible text,
    // e.g. `chapter1.md#section` inside <code>.
    let mut result = String::new();
    let mut chars = html.char_indices().peekable();
    let mut in_tag = false;

    while let Some((_, c)) = chars.next() {
        result.push(c);

        if c == '<' {
            in_tag = true;
            continue;
        }
        if c == '>' {
            in_tag = false;
            continue;
        }

        // Attribute values only exist inside tags; this keeps literal text
        // like href="x.md" inside <code> untouched
        if in_tag && (c == '"' || c == '\'') {
            let quote_char = c;
            // Check if this quote opens an href attribute value
            // (look at the 5 chars just before the quote we pushed)
            let before_quote = &result[..result.len() - quote_char.len_utf8()];
            let suffix: String = before_quote
                .chars()
                .rev()
                .take(5)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            if suffix.eq_ignore_ascii_case("href=") {
                // Collect the URL up to the closing quote
                let mut url = String::new();
                let mut closed = false;
                for (_, ch) in chars.by_ref() {
                    if ch == quote_char {
                        result.push_str(&rewrite(&url));
                        result.push(quote_char);
                        closed = true;
                        break;
                    }
                    url.push(ch);
                }
                if !closed {
                    // Unterminated attribute: emit what we collected as-is
                    result.push_str(&url);
                }
            }
        }
    }

    // Normalize backslashes to forward slashes in href attributes
    normalize_path_separators(&result)
}

/// Convert a `.md` extension in a URL to `.html` (also handles `.md#anchor`). A link out of
/// the book stays as it is: another site's `.md` has no `.html`.
fn fix_md_extension(url: &str) -> String {
    if has_url_scheme(url) || url.starts_with("//") {
        return url.to_string();
    }
    if let Some(stripped) = url.strip_suffix(".md") {
        format!("{}.html", stripped)
    } else if let Some(pos) = url.find(".md#") {
        format!("{}.html{}", &url[..pos], &url[pos + 3..])
    } else {
        url.to_string()
    }
}

/// True when the quote just pushed onto `result` opens an href/src attribute
/// value. The quote itself must be excluded before checking — including it
/// makes the ends_with() check structurally impossible to match.
fn quote_opens_link_attr(result: &str, quote_char: char) -> bool {
    let before = &result[..result.len() - quote_char.len_utf8()];
    let tail: String = before
        .chars()
        .rev()
        .take(5)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    let tail = tail.to_ascii_lowercase();
    tail.ends_with("href=") || tail.ends_with("src=")
}

/// Rewrite the value of every href/src attribute with `f`.
/// Unterminated attribute values are emitted unchanged.
fn map_link_attr_values(html: &str, f: impl Fn(String) -> String) -> String {
    let mut result = String::new();
    let mut chars = html.char_indices().peekable();

    while let Some((_, c)) = chars.next() {
        result.push(c);

        if (c == '"' || c == '\'') && quote_opens_link_attr(&result, c) {
            let quote_char = c;
            let mut url = String::new();
            let mut closed = false;
            for (_, ch) in chars.by_ref() {
                if ch == quote_char {
                    result.push_str(&f(url.clone()));
                    result.push(quote_char);
                    closed = true;
                    break;
                }
                url.push(ch);
            }
            if !closed {
                result.push_str(&url);
            }
        }
    }

    result
}

/// Remove leading slashes from internal links
/// Converts href="/path/to/file" → href="path/to/file"
/// Skips protocol-relative URLs (//example.com) and external links
fn remove_leading_slash_from_links(html: &str) -> String {
    map_link_attr_values(html, |url| {
        if url.starts_with('/') && !url.starts_with("//") {
            let lower = url.to_lowercase();
            if !lower.starts_with("/http://") && !lower.starts_with("/https://") {
                return url.chars().skip(1).collect();
            }
        }
        url
    })
}

/// Convert backslashes to forward slashes in href and src attributes
/// Handles Windows-style paths like href="path\to\file" → href="path/to/file"
fn normalize_path_separators(html: &str) -> String {
    map_link_attr_values(html, |url| url.replace('\\', "/"))
}

/// Add target="_blank" rel="noopener noreferrer" to external links that don't have target attribute
/// This handles Markdown-style links [text](https://...) that were converted to <a href="...">
fn add_target_blank_to_external_links(html: &str) -> String {
    let mut result = String::new();
    let mut chars = html.char_indices().peekable();

    while let Some((i, c)) = chars.next() {
        if c == '<' && html[i..].starts_with("<a ") {
            // Found an anchor tag start
            let mut tag_content = String::from("<a ");
            // Skip past "<a "
            chars.next(); // 'a'
            chars.next(); // ' '

            // Collect the entire tag until '>'
            for (_, ch) in chars.by_ref() {
                tag_content.push(ch);
                if ch == '>' {
                    break;
                }
            }

            // Check if this is an external link without target attribute
            let tag_lower = tag_content.to_lowercase();
            let has_target = tag_lower.contains("target=");
            let is_external = tag_lower.contains("href=\"http://")
                || tag_lower.contains("href=\"https://")
                || tag_lower.contains("href='http://")
                || tag_lower.contains("href='https://");

            if is_external && !has_target {
                // Insert target="_blank" rel="noopener noreferrer" before the closing >
                let without_close = tag_content.trim_end_matches('>');
                result.push_str(without_close);
                result.push_str(" target=\"_blank\" rel=\"noopener noreferrer\">");
            } else {
                result.push_str(&tag_content);
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// Elements whose text is not prose: nothing in them is a link or an image
const VERBATIM_ELEMENTS: [&str; 5] = ["code", "pre", "script", "style", "textarea"];

/// Keep track of the verbatim element that the HTML being scanned is inside, given the text
/// of its next tag
fn track_verbatim_element(open: &mut Option<String>, tag: &str) {
    let tag = tag.to_lowercase();
    if let Some(element) = open {
        // The `<` of "a<b" in a script looks like a tag that runs up to the real one
        let last_tag = tag.rsplit('<').next().unwrap_or(&tag);
        if last_tag.strip_prefix('/').map(str::trim_end) == Some(element.as_str()) {
            *open = None;
        }
    } else {
        let name = tag
            .split(|c: char| c.is_whitespace() || c == '/')
            .next()
            .unwrap_or("");
        if VERBATIM_ELEMENTS.contains(&name) {
            *open = Some(name.to_string());
        }
    }
}

/// Auto-link URLs that are not already inside anchor tags or code blocks
/// Converts bare URLs like https://example.com to <a href="..." target="_blank">...</a>
fn autolink_urls(html: &str) -> String {
    let mut result = String::new();
    let mut chars = html.char_indices().peekable();
    let mut verbatim_element = None;

    while let Some((i, c)) = chars.next() {
        // Check if we're inside an HTML tag
        if c == '<' {
            result.push(c);

            // Collect the tag
            let mut tag_content = String::new();
            for (_, ch) in chars.by_ref() {
                result.push(ch);
                if ch == '>' {
                    break;
                }
                tag_content.push(ch);
            }

            track_verbatim_element(&mut verbatim_element, &tag_content);
            continue;
        }

        // Skip auto-linking if inside code block
        if verbatim_element.is_some() {
            result.push(c);
            continue;
        }

        // Check for http:// or https://
        if c == 'h' && html[i..].starts_with("http://") || html[i..].starts_with("https://") {
            // Check if this URL is already inside an href=""
            if result.ends_with("href=\"") || result.ends_with("src=\"") {
                // Already in an href, just copy normally
                result.push(c);
                continue;
            }

            // Extract the URL
            let url_start = i;
            let mut url_end = i + 1;

            // Continue consuming URL characters
            while let Some(&(next_i, next_c)) = chars.peek() {
                // URL ends at whitespace, <, >, ", '
                if next_c.is_whitespace()
                    || next_c == '<'
                    || next_c == '>'
                    || next_c == '"'
                    || next_c == '\''
                {
                    break;
                }
                url_end = next_i + next_c.len_utf8();
                chars.next();
            }

            let mut url = &html[url_start..url_end];

            // Remove trailing punctuation that's likely not part of URL
            while url.ends_with('.')
                || url.ends_with(',')
                || url.ends_with(';')
                || url.ends_with(':')
                || url.ends_with(')')
                || url.ends_with('!')
                || url.ends_with('?')
            {
                url = &url[..url.len() - 1];
            }

            // Create the link with target="_blank"
            result.push_str(&format!(r#"<a href="{}" target="_blank">{}</a>"#, url, url));

            // If we trimmed trailing punctuation, add it back
            let trimmed_len = url_end - url_start - url.len();
            if trimmed_len > 0 {
                result.push_str(&html[url_start + url.len()..url_end]);
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// Convert remaining markdown image syntax ![alt](url) to <img> tags
/// This handles images inside raw HTML blocks that pulldown-cmark doesn't parse
/// Skips content inside verbatim elements
fn convert_remaining_markdown_images(html: &str) -> String {
    let mut result = String::new();
    let mut chars = html.char_indices().peekable();
    let mut verbatim_element = None;

    while let Some((i, c)) = chars.next() {
        // Check if we're inside an HTML tag
        if c == '<' {
            result.push(c);

            // Collect the tag
            let mut tag_content = String::new();
            for (_, ch) in chars.by_ref() {
                result.push(ch);
                if ch == '>' {
                    break;
                }
                tag_content.push(ch);
            }

            track_verbatim_element(&mut verbatim_element, &tag_content);
            continue;
        }

        // Skip image conversion if inside code block
        if verbatim_element.is_some() {
            result.push(c);
            continue;
        }

        if c == '!' {
            if let Some((alt, path, title, len)) = parse_remaining_image(&html[i..]) {
                // Output as <img> tag (escape quotes so alt/url text cannot
                // break out of the attribute value)
                let title = if title.is_empty() {
                    String::new()
                } else {
                    format!(r#" title="{}""#, html_escape(title))
                };
                result.push_str(&format!(
                    r#"<img src="{}" alt="{}"{}>"#,
                    html_escape(path).replace(' ', "%20"),
                    html_escape(alt),
                    title
                ));
                while chars.peek().is_some_and(|(next, _)| *next < i + len) {
                    chars.next();
                }
                continue;
            }
        }

        result.push(c);
    }

    result
}

/// Read an image the Markdown parser left as text, at the start of `rest`. It has to close
/// on its line and before any tag, otherwise it is just text that looks like one. Returns
/// the alt text, the path, the title, and the length of the source.
fn parse_remaining_image(rest: &str) -> Option<(&str, &str, &str, usize)> {
    let inner = rest.strip_prefix("![")?;
    let alt_len = find_closing_bracket(inner, '[', ']')?;
    let (alt, after_alt) = inner.split_at(alt_len);
    let destination = after_alt.strip_prefix("](")?;
    let destination_len = find_closing_bracket(destination, '(', ')')?;

    let (path, title) = split_link_title(&destination[..destination_len]);
    let path = path.trim();
    let path = path
        .strip_prefix('<')
        .and_then(|path| path.strip_suffix('>'))
        .unwrap_or(path);
    if alt.contains('<') || path.contains('<') {
        return None;
    }

    // The title still has its quotes, which are one byte each
    let title = title.trim();
    let title = title.get(1..title.len().saturating_sub(1)).unwrap_or("");

    let len = "![".len() + alt_len + "](".len() + destination_len + ")".len();
    Some((alt, path, title, len))
}

/// The position of the bracket closing a run of text, on the same line.
fn find_closing_bracket(text: &str, open: char, close: char) -> Option<usize> {
    let mut depth = 1;
    for (i, ch) in text.char_indices() {
        if ch == '\n' {
            return None;
        } else if ch == open {
            depth += 1;
        } else if ch == close {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
    }
    None
}

/// Convert internal links to proper relative paths from current file
///
/// Handles two cases:
/// 1. Root-relative links (starting with /): e.g., "/api-docs/" → "../../api-docs/" (at depth 2)
/// 2. Root-relative links without slash: e.g., "Customer/AssetStatus/File.html" → "../../Customer/AssetStatus/File.html"
///
/// current_path: e.g., "Customer/AssetStatus/PortfolioTop.md"
fn convert_relative_links_to_absolute(html: &str, current_path: &str) -> String {
    let result = html.to_string();

    // Calculate the depth (number of directories from root)
    // e.g., "Customer/AssetStatus/PortfolioTop.md" -> depth 2
    let depth = Path::new(current_path)
        .parent()
        .map(|p| {
            let dir = p.to_string_lossy();
            if dir.is_empty() {
                0
            } else {
                dir.matches('/').count() + 1
            }
        })
        .unwrap_or(0);

    // Create the prefix to go back to root (e.g., "../../" for depth 2)
    let root_prefix: String = "../".repeat(depth);

    // Bare directory paths (e.g., "CommonUI/Dialog.md") are file-relative.
    // Normalize them to a canonical root-based path (root_prefix + current_dir + url)
    // so SPA navigation cannot accumulate relative segments, while preserving
    // the file-relative meaning of the link as written.
    let current_dir = Path::new(current_path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let bare_dir_prefix = if current_dir.is_empty() {
        String::new()
    } else {
        format!("{}{}/", root_prefix, current_dir)
    };

    // href: root-relative paths are book-root-relative; bare directory paths are file-relative
    let result = adjust_attribute_urls(
        &result,
        r#"href=""#,
        &root_prefix,
        depth,
        Some(&bare_dir_prefix),
    );
    // src: only root-relative paths (/assets/x.png). Page-relative image
    // paths (images/foo.png) are correct as written and must not be rewritten
    adjust_attribute_urls(&result, r#"src=""#, &root_prefix, depth, None)
}

/// Adjust URLs in one attribute type (href="..." / src="...") for page depth
fn adjust_attribute_urls(
    html: &str,
    attr_pattern: &str,
    root_prefix: &str,
    depth: usize,
    bare_dir_prefix: Option<&str>,
) -> String {
    let result = html;
    let mut new_result = String::new();
    let mut last_end = 0;
    let mut search_start = 0;

    while let Some(attr_pos) = result[search_start..].find(attr_pattern) {
        let abs_attr_pos = search_start + attr_pos;
        let url_start = abs_attr_pos + attr_pattern.len();

        // Find the closing quote
        if let Some(url_end_offset) = result[url_start..].find('"') {
            let url_end = url_start + url_end_offset;
            let url = &result[url_start..url_end];

            // Check if this is a root-relative link (starts with /)
            // Convert "/api-docs/" to "../../api-docs/" based on depth
            let is_root_relative = url.starts_with('/')
                && !url.starts_with("//")  // Skip protocol-relative URLs
                && !url.to_lowercase().starts_with("/http://")
                && !url.to_lowercase().starts_with("/https://");

            if is_root_relative {
                // Copy everything up to the URL
                new_result.push_str(&result[last_end..url_start]);
                // Add the root prefix + URL without leading slash
                new_result.push_str(root_prefix);
                new_result.push_str(&url[1..]); // Skip the leading /
                last_end = url_end;
                search_start = url_end + 1;
                continue;
            }

            // Check if this is an internal link that needs conversion (no leading /)
            // Skip: external links (http/https), anchor-only (#), already relative (../ or ./), data URIs
            // Skip: same-directory links (no "/" in path) - these are already correct relative links
            let needs_conversion = bare_dir_prefix.is_some()
                && !url.is_empty()
                && url.contains('/')  // Only convert links with directory paths
                && !url.starts_with("http://")
                && !url.starts_with("https://")
                && !url.starts_with('#')
                && !url.starts_with("../")
                && !url.starts_with("./")
                && !url.starts_with('/')
                && !url.starts_with("mailto:")
                && !url.starts_with("javascript:")
                && !url.starts_with("data:")
                && depth > 0;

            if needs_conversion {
                // Copy everything up to the URL
                new_result.push_str(&result[last_end..url_start]);
                // Normalize the file-relative bare path to a canonical
                // root-based relative path (root_prefix + current_dir + url)
                new_result.push_str(bare_dir_prefix.unwrap_or_default());
                new_result.push_str(url);
                last_end = url_end;
            }

            search_start = url_end + 1;
        } else {
            search_start = url_start + 1;
        }
    }

    // Copy the remaining part
    new_result.push_str(&result[last_end..]);

    new_result
}

// =============================================================================
// AsciiDoc Rendering
// =============================================================================

/// Render AsciiDoc content to HTML
/// Applies the same post-processing as markdown (target="_blank", link normalization, etc.)
pub fn render_asciidoc(content: &str) -> String {
    render_asciidoc_internal(content)
}

/// Render AsciiDoc content to HTML with path for relative link conversion
pub fn render_asciidoc_with_path(content: &str, current_path: Option<&str>) -> String {
    let html = render_asciidoc_internal(content);

    // If we have a current path, convert relative links to absolute
    if let Some(path) = current_path {
        convert_relative_links_to_absolute(&html, path)
    } else {
        html
    }
}

/// Extract headings from AsciiDoc content for TOC generation
pub fn extract_headings_from_asciidoc(content: &str) -> Vec<TocItem> {
    let mut headings = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // AsciiDoc headings: == Level 1, === Level 2, ==== Level 3, etc.
        if trimmed.starts_with("==") && !trimmed.starts_with("====") {
            // Count the equals signs
            let eq_count = trimmed.chars().take_while(|&c| c == '=').count();

            // Level mapping: == is h2, === is h3, ==== is h4
            // (= is h1 which is typically the document title)
            if (2..=5).contains(&eq_count) {
                let level = eq_count as u8; // 2 = h2, 3 = h3, etc.
                let text = trimmed[eq_count..].trim().to_string();

                // Only include h2, h3, h4 in TOC (skip h1 which is page title)
                if (2..=4).contains(&level) && !text.is_empty() {
                    let id = slugify(&text);
                    headings.push(TocItem { level, text, id });
                }
            }
        }
    }

    headings
}

fn render_asciidoc_internal(content: &str) -> String {
    // Normalize CRLF/CR to LF for consistent line handling
    let content = content.replace("\r\n", "\n").replace("\r", "\n");

    // Strip all UTF-8 BOM characters
    let content = content.replace('\u{FEFF}', "");

    // Use asciidocr to convert to HTML
    // 1. Create a Scanner to tokenize the content
    let scanner = asciidocr::scanner::Scanner::new(&content);

    // 2. Create a Parser and parse the tokens (parser expects iterator of Result<Token, ScannerError>)
    let mut parser = asciidocr::parser::Parser::new(std::path::PathBuf::from("."));

    match parser.parse(scanner) {
        Ok(asg) => {
            // 3. Render the ASG to HTMLBook
            match asciidocr::backends::htmls::render_htmlbook(&asg) {
                Ok(html) => {
                    // Extract just the body content (asciidocr outputs full HTML document)
                    let html = extract_body_content(&html);

                    // Apply the same post-processing as markdown
                    let html = fix_asciidoc_relative_links(&html);
                    let html = remove_leading_slash_from_links(&html);
                    let html = autolink_urls(&html);

                    add_target_blank_to_external_links(&html)
                }
                Err(e) => {
                    eprintln!("  Warning: AsciiDoc conversion error: {:?}", e);
                    format!("<p>{}</p>", html_escape(&content))
                }
            }
        }
        Err(e) => {
            eprintln!("  Warning: AsciiDoc parsing error: {:?}", e);
            // Return the content wrapped in a simple paragraph as fallback
            format!("<p>{}</p>", html_escape(&content))
        }
    }
}

/// Extract body content from full HTML document
/// asciidocr outputs full HTML with <!DOCTYPE>, <html>, <head>, <body>
/// We only need the content inside <body>
fn extract_body_content(html: &str) -> String {
    // Try to find <body> and </body> tags
    if let Some(body_start) = html.find("<body>") {
        let content_start = body_start + 6; // length of "<body>"
        if let Some(body_end) = html.find("</body>") {
            return html[content_start..body_end].trim().to_string();
        }
    }
    // If no body tags found, return as-is
    html.to_string()
}

/// Fix relative links in AsciiDoc output
/// Converts .adoc and .asciidoc links to .html
fn fix_asciidoc_relative_links(html: &str) -> String {
    let mut result = html.to_string();

    // Replace .adoc and .asciidoc links with .html
    let patterns = [
        (r#".adoc""#, r#".html""#),
        (r#".adoc#"#, r#".html#"#),
        (r#".adoc'"#, r#".html'"#),
        (r#".asciidoc""#, r#".html""#),
        (r#".asciidoc#"#, r#".html#"#),
        (r#".asciidoc'"#, r#".html'"#),
        // Also handle .md links for mixed content
        (r#".md""#, r#".html""#),
        (r#".md#"#, r#".html#"#),
        (r#".md'"#, r#".html'"#),
    ];

    for (from, to) in patterns {
        result = result.replace(from, to);
    }

    // Normalize backslashes to forward slashes in href attributes
    result = normalize_path_separators(&result);

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_basic_markdown() {
        let md = "# Hello\n\nThis is a **test**.";
        let html = render_markdown(md);
        // Heading now includes ID attribute
        assert!(
            html.contains("<h1 id=\"hello\">Hello</h1>"),
            "HTML: {}",
            html
        );
        assert!(html.contains("<strong>test</strong>"));
    }

    #[test]
    fn test_render_table() {
        let md = r#"
| Header 1 | Header 2 |
|----------|----------|
| Cell 1   | Cell 2   |
"#;
        let html = render_markdown(md);
        assert!(html.contains("<table>"));
        assert!(html.contains("<th>Header 1</th>"));
    }

    #[test]
    fn test_render_mermaid() {
        let md = r#"
```mermaid
sequenceDiagram
    A->>B: Hello
```
"#;
        let html = render_markdown(md);
        assert!(html.contains(r#"<div class="mermaid">"#));
        assert!(html.contains("sequenceDiagram"));
    }

    #[test]
    fn test_fix_relative_links() {
        let html = r#"<a href="chapter1.md">Link</a>"#;
        let fixed = fix_relative_links(html);
        assert!(fixed.contains(r#"href="chapter1.html""#));
    }

    #[test]
    fn test_image_in_table() {
        let md = r#"
| Col1 | Col2 |
|:--:|:--:|
|![](test.png)|text|
"#;
        let html = render_markdown(md);
        println!("Generated HTML: {}", html);
        assert!(
            html.contains("<img"),
            "Image tag should be generated: {}",
            html
        );
    }

    #[test]
    fn test_image_in_table_japanese() {
        let md = r#"## デザイン
|該当するタイムラインがある場合|該当するタイムラインがない場合|
|:--:|:--:|
|![](../../../assets/Customer/TimeLine/B-0-8-Timeline Information Page.png)|![](../../../assets/Customer/TimeLine/B-0-8-Timeline Information Page0件.png)|
## 項目一覧"#;
        let html = render_markdown(md);
        println!("Generated HTML: {}", html);
        assert!(
            html.contains("<img"),
            "Image tag should be generated: {}",
            html
        );
    }

    #[test]
    fn test_image_with_space_in_filename() {
        // Test: space in filename
        let md = r#"|![](test file.png)|"#;
        let html = render_markdown(md);
        println!("With space: {}", html);

        // Test: no space in filename
        let md2 = r#"|![](testfile.png)|"#;
        let html2 = render_markdown(md2);
        println!("No space: {}", html2);
    }

    #[test]
    fn test_autolink_urls() {
        // Test: bare URL should become a link
        let md = "Guide Git:https://github.com/guide-inc-org/kcmsr-member-site-spec";
        let html = render_markdown(md);
        println!("Autolink result: {}", html);
        assert!(html.contains(r#"<a href="https://github.com/guide-inc-org/kcmsr-member-site-spec" target="_blank">"#),
            "URL should be auto-linked: {}", html);
    }

    #[test]
    fn test_autolink_does_not_double_link() {
        // Test: already linked URL should not be double-linked
        let md = "[Link](https://example.com)";
        let html = render_markdown(md);
        println!("Already linked result: {}", html);
        // Should have exactly one href for the URL
        let count = html.matches("https://example.com").count();
        assert_eq!(count, 1, "URL should appear only once: {}", html);
    }

    #[test]
    fn test_multiline_footnotes() {
        let md = r#"Text with footnote[^1].

[^1]: First line
- Second line
- Third line
[^2]: Another footnote"#;
        let html = render_markdown(md);
        println!("Footnote HTML: {}", html);
        // The footnote should be properly rendered with list items inside
        assert!(
            html.contains("<li>"),
            "Footnote should contain list items: {}",
            html
        );
    }

    #[test]
    fn test_slugify_matches_github_slugger() {
        // Test that slugify matches github-slugger / HonKit behavior
        // Special characters like / should be removed, not converted to hyphens
        assert_eq!(
            slugify("/auth/verification-email/resend"),
            "authverification-emailresend"
        );
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("A.B.C"), "abc"); // Periods removed
        assert_eq!(slugify("日本語テスト"), "日本語テスト"); // Japanese preserved
        assert_eq!(slugify("test_underscore"), "test_underscore"); // Underscores preserved
        assert_eq!(slugify("a--b"), "a-b"); // Multiple hyphens collapsed
    }

    #[test]
    fn test_duplicate_heading_ids_are_deduplicated() {
        // Regression: identical headings produced identical ids
        let md = "## 概要\n\n本文A\n\n## 概要\n\n本文B\n\n## 概要\n\n本文C\n";
        let html = render_markdown(md);
        assert!(html.contains(r#"<h2 id="概要">"#), "html: {}", html);
        assert!(html.contains(r#"<h2 id="概要-1">"#), "html: {}", html);
        assert!(html.contains(r#"<h2 id="概要-2">"#), "html: {}", html);
    }

    #[test]
    fn test_toc_ids_match_rendered_heading_ids_for_duplicates() {
        let md = "## 概要\n\n本文A\n\n### 詳細\n\n## 概要\n\n本文B\n";
        let html = render_markdown(md);
        let toc = extract_headings(md);
        for item in &toc {
            assert!(
                html.contains(&format!(r#"id="{}""#, item.id)),
                "TOC id {} not found in rendered html: {}",
                item.id,
                html
            );
        }
        // The two 概要 entries must have distinct ids
        let ids: Vec<&str> = toc.iter().map(|t| t.id.as_str()).collect();
        assert_eq!(ids.iter().filter(|i| **i == "概要").count(), 1);
        assert!(ids.contains(&"概要-1"));
    }

    #[test]
    fn test_md_link_in_inline_code_not_rewritten() {
        // Regression: blind .md# replacement rewrote text inside <code>
        let md = "詳細は `chapter1.md#section` を参照。\n\n[link](other.md#anchor)\n";
        let html = render_markdown(md);
        assert!(
            html.contains("<code>chapter1.md#section</code>"),
            "inline code must be untouched: {}",
            html
        );
        assert!(
            html.contains(r##"href="other.html#anchor""##),
            "real links must still be converted: {}",
            html
        );
    }

    #[test]
    fn test_md_link_in_code_block_not_rewritten() {
        let md = "```\nsee chapter1.md#section and \"file.md\"\n```\n";
        let html = render_markdown(md);
        assert!(
            html.contains("chapter1.md#section"),
            "code block must be untouched: {}",
            html
        );
        assert!(!html.contains("chapter1.html"), "html: {}", html);
    }

    #[test]
    fn test_normalize_path_separators_fires() {
        // Regression: the href=/src= detection included the just-pushed quote
        // in the suffix check, so it never matched and backslashes survived
        let html =
            r#"<a href="docs\sub\file.html">x</a> <img src="img\pic.png"> and text\with\backslash"#;
        let fixed = normalize_path_separators(html);
        assert!(fixed.contains(r#"href="docs/sub/file.html""#), "{}", fixed);
        assert!(fixed.contains(r#"src="img/pic.png""#), "{}", fixed);
        // Backslashes outside attributes stay untouched
        assert!(fixed.contains(r"text\with\backslash"), "{}", fixed);
    }

    #[test]
    fn test_remove_leading_slash_fires() {
        // Same regression shape as normalize_path_separators
        let html = r#"<a href="/guide/page.html">x</a><img src="/assets/pic.png"><a href="//cdn.example.com/x">y</a>"#;
        let fixed = remove_leading_slash_from_links(html);
        assert!(fixed.contains(r#"href="guide/page.html""#), "{}", fixed);
        assert!(fixed.contains(r#"src="assets/pic.png""#), "{}", fixed);
        // Protocol-relative URL untouched
        assert!(fixed.contains(r#"href="//cdn.example.com/x""#), "{}", fixed);
    }

    #[test]
    fn test_root_relative_img_src_depth_adjusted() {
        // Regression: only href was depth-adjusted; root-relative <img src>
        // stayed absolute and 404'd on nested pages / subpath hosting
        let md = "![img](/assets/pic.png)\n\n[link](/api-docs/)\n";
        let html = render_markdown_with_path(md, Some("Guide/Sub/Page.md"), false);
        assert!(
            html.contains(r#"src="../../assets/pic.png""#),
            "src must be depth-adjusted: {}",
            html
        );
        assert!(
            html.contains(r#"href="../../api-docs/""#),
            "href behavior unchanged: {}",
            html
        );
    }

    #[test]
    fn test_page_relative_img_src_untouched() {
        // Page-relative image paths are correct as written — the bare-dir
        // conversion applied to href must NOT be applied to src
        let md = "![img](images/pic.png)\n";
        let html = render_markdown_with_path(md, Some("Guide/Sub/Page.md"), false);
        assert!(
            html.contains(r#"src="images/pic.png""#),
            "page-relative src must stay as written: {}",
            html
        );
    }

    #[test]
    fn test_remaining_markdown_image_escapes_quotes() {
        let html = r#"<div>![weather "sunny"](icon.png)</div>"#;
        let fixed = convert_remaining_markdown_images(html);
        assert!(
            fixed.contains(r#"alt="weather &quot;sunny&quot;""#),
            "quotes in alt must be escaped: {}",
            fixed
        );
    }

    #[test]
    fn test_href_like_text_outside_tag_not_rewritten() {
        // Raw HTML passes through markdown unescaped; href= appearing as
        // visible text (not inside a tag) must not be treated as a link
        let html = r#"<p>example: href="a.md" is the syntax. <a href="b.md">real</a></p>"#;
        let fixed = fix_relative_links(html);
        assert!(fixed.contains(r#"href="a.md" is the syntax"#), "{}", fixed);
        assert!(fixed.contains(r#"<a href="b.html">"#), "{}", fixed);
    }

    #[test]
    fn test_footnote_ref_in_inline_code_not_converted() {
        // Regression: the regex character class [^abc] inside inline code
        // was converted to a footnote reference
        let md = "正規表現 `[^abc]` は abc 以外にマッチする。[^1]\n\n[^1]: 実際の脚注\n";
        let html = render_markdown(md);
        assert!(
            html.contains("<code>[^abc]</code>"),
            "inline code must be untouched: {}",
            html
        );
        assert!(
            !html.contains("reffn_abc"),
            "no footnote must be generated from code: {}",
            html
        );
        // The real footnote reference still works
        assert!(html.contains("reffn_1"), "html: {}", html);
    }

    #[test]
    fn test_footnote_ref_in_fenced_code_not_converted() {
        let md = "```\nmatch = re.compile(r\"[^abc]\")\n```\n\n本文[^2]です。\n\n[^2]: 脚注\n";
        let html = render_markdown(md);
        assert!(
            !html.contains("reffn_abc"),
            "no footnote from fenced code: {}",
            html
        );
        assert!(
            html.contains("reffn_2"),
            "real footnote still works: {}",
            html
        );
    }
}

#[test]
fn test_footnote_in_table() {
    let md = r#"| Col1 | Col2 | Col3 |
|------|------|------|
| [A][^1] | data | end |

[A]: #link
[^1]: Footnote one
"#;
    let html = render_markdown(md);
    println!("HTML: {}", html);
    // Check that table cells are separate
    assert!(
        html.contains("<td>data</td>") || html.contains(">data<"),
        "data should be in its own cell: {}",
        html
    );
}

#[test]
fn test_reference_link_basic() {
    // Test basic reference link functionality
    let md = r#"[改定履歴][AL_RH]

[AL_RH]: #改訂履歴"#;
    let html = render_markdown(md);
    println!("Test 1 (basic with space): {}", html);
    assert!(
        html.contains("<a "),
        "Reference link should create anchor: {}",
        html
    );
}

#[test]
fn test_reference_link_no_space() {
    // Test reference link without space after colon (HonKit format)
    let md = r#"[改定履歴][AL_RH]

[AL_RH]:#改訂履歴"#;
    let html = render_markdown(md);
    println!("Test 2 (no space): {}", html);
    // This might fail - checking pulldown-cmark behavior
    assert!(
        html.contains("<a "),
        "Reference link without space should work: {}",
        html
    );
}

#[test]
fn test_reference_link_after_html_comment() {
    // Test reference link definition after HTML comment
    let md = r#"[改定履歴][AL_RH]

<!-- 目次 -->
[AL_RH]: #改訂履歴"#;
    let html = render_markdown(md);
    println!("Test 3 (after HTML comment): {}", html);
    assert!(
        html.contains("<a "),
        "Reference link after HTML comment should work: {}",
        html
    );
}

#[test]
fn test_reference_link_after_html_comment_with_blank_line() {
    // Test reference link definition after HTML comment with blank line
    let md = r#"[改定履歴][AL_RH]

<!-- 目次 -->

[AL_RH]: #改訂履歴"#;
    let html = render_markdown(md);
    println!("Test 4 (after HTML comment with blank line): {}", html);
    assert!(
        html.contains("<a "),
        "Reference link after HTML comment with blank line should work: {}",
        html
    );
}

#[test]
fn test_reference_link_with_bom() {
    // Test reference link with UTF-8 BOM at start of definitions
    // BOM is \xEF\xBB\xBF (357 273 277 in octal)
    let bom = "\u{FEFF}";
    let md = format!(
        r#"[改定履歴][AL_RH]

{}<!-- 目次 -->
[AL_RH]:#改訂履歴"#,
        bom
    );
    let html = render_markdown(&md);
    println!("Test 5 (with BOM): {}", html);
    assert!(
        html.contains("<a "),
        "Reference link with BOM should work: {}",
        html
    );
}

#[test]
fn test_footnote_with_list() {
    let html = render_markdown("本文[^1]\n\n[^1]: 見出し\n- 項目1\n- 項目2\n\n後ろの段落");
    assert!(
        html.contains("<ul>") && html.contains("<li>項目1</li>") && html.contains("<li>項目2</li>"),
        "Should contain list: {}",
        html
    );
    let blockquote_end = html.find("</blockquote>").expect("no blockquote");
    let list = html.find("<ul>").expect("no list");
    assert!(
        blockquote_end < list,
        "The list should follow the first line: {}",
        html
    );
    assert!(html.contains("<p>後ろの段落</p>"), "{}", html);
}

#[test]
fn test_full_reference_link_in_footnote() {
    // Test [text][ref] pattern in footnotes - the bug that was reported
    let md = r#"Text[^1].

[^1]: .paymentAvailableStatus=[未申込][決済方法申込状態]の場合: "銀行引落(登録)"

[決済方法申込状態]:#決済方法申込状態"#;
    let html = render_markdown(md);
    println!("Full reference link in footnote: {}", html);
    // The [未申込] should become a link to #決済方法申込状態. The href is percent-encoded
    // like any other link in the document, which a browser resolves the same way.
    assert!(
        html.contains("<a href=\"#%E6%B1%BA%E6%B8%88") && html.contains(">未申込</a>"),
        "Full reference link [text][ref] should be resolved: {}",
        html
    );
    // The text after should be preserved
    assert!(
        html.contains("の場合:"),
        "Text after reference link should be preserved: {}",
        html
    );
}

#[test]
fn test_multilang_relative_links() {
    let md = "[Link](repositories/docs-path.md)";
    let current_path = "getting-started.md";
    let html = render_markdown_with_path(md, Some(current_path), false);
    println!("Result HTML: {}", html);

    // With the fix, it should NOT include "../" even if it contains a slash, because depth is 0
    assert!(
        !html.contains("../repositories/docs-path.html"),
        "Should not prepend ../ to relative links when depth is 0: {}",
        html
    );
    assert!(
        html.contains("href=\"repositories/docs-path.html\""),
        "Should preserve relative link: {}",
        html
    );
}

#[test]
fn test_footnote_reference_style_images() {
    let md = "本文[^1]\n\n[^1]: ![][icon] の説明\n- ![][icon]\n- ![代替][icon]\n- ![icon]\n\n[icon]: images/icon.png\n";
    let html = render_markdown(md);
    assert_eq!(
        html.matches("<img src=\"images/icon.png\"").count(),
        4,
        "Every reference-style image should be an <img>: {}",
        html
    );
    assert!(html.contains("alt=\"代替\""), "{}", html);
    assert!(
        !html.contains("!<a"),
        "An image should not become a link: {}",
        html
    );
    assert!(!html.contains("%%FN"), "{}", html);
}

#[test]
fn test_footnote_first_line_inline_formatting() {
    let html = render_markdown("本文[^1]\n\n[^1]: `code` と **太字** と [リンク](other.md)\n");
    assert!(
        html.contains("<blockquote id=\"fn_1\"><sup>1</sup>. <code>code</code>"),
        "{}",
        html
    );
    assert!(html.contains("<strong>太字</strong>"), "{}", html);
    assert!(html.contains("href=\"other.html\""), "{}", html);
    assert!(
        html.contains(
            "<a href=\"#reffn_1\" title=\"Jump back to footnote [1] in the text.\"> ↩</a></blockquote>"
        ),
        "The back link should close the blockquote: {}",
        html
    );
    assert!(
        !html.contains("<p><blockquote") && !html.contains("</blockquote></p>"),
        "The blockquote should replace the paragraph: {}",
        html
    );
    assert!(!html.contains("%%FN"), "{}", html);
}

#[test]
fn test_footnote_first_line_ending_with_url() {
    let html = render_markdown("本文[^1]\n\n[^1]: 参照先 https://example.com/page\n");
    assert!(
        html.contains(
            "<a href=\"https://example.com/page\" target=\"_blank\">https://example.com/page</a><a href=\"#reffn_1\""
        ),
        "The URL should end before the back link: {}",
        html
    );
    assert!(!html.contains("%%FN"), "{}", html);
}

#[test]
fn test_footnote_reference_right_after_url() {
    let html = render_markdown("https://example.com/page[^1] の続き\n\n[^1]: 脚注\n");
    assert!(
        html.contains(
            "<a href=\"https://example.com/page\" target=\"_blank\">https://example.com/page</a><sup><a href=\"#fn_1\""
        ),
        "The URL should end before the reference: {}",
        html
    );
    assert!(!html.contains("%%FN"), "{}", html);
}

#[test]
fn test_footnote_definition_in_raw_html_block() {
    // A comment or <pre> is not parsed: what looks like a definition stays as written
    // and does not reach for the end of the next real definition
    let md = "<!--\n[^8]: コメントの中\n-->\n\n本文A\n\n<pre>\n[^9]: pre の中\n</pre>\n\n本文B\n\n[^1]: 本物\n\n本文C\n";
    let html = render_markdown(md);
    assert!(html.contains("[^8]: コメントの中"), "{}", html);
    assert!(html.contains("[^9]: pre の中"), "{}", html);
    for text in ["本文A", "本文B", "本文C"] {
        assert_eq!(
            html.matches(&format!("<p>{}</p>", text)).count(),
            1,
            "{} should appear once: {}",
            text,
            html
        );
    }
    assert!(
        html.contains("<blockquote id=\"fn_1\"><sup>1</sup>. 本物<a href=\"#reffn_1\""),
        "{}",
        html
    );
    assert_eq!(html.matches("<blockquote").count(), 1, "{}", html);
    assert!(!html.contains("%%FN"), "{}", html);
}

#[test]
fn test_footnote_definition_in_fenced_code_not_converted() {
    let html = render_markdown("```\n[^1]: コードの中\n```\n");
    assert!(html.contains("[^1]: コードの中"), "{}", html);
    assert!(!html.contains("<blockquote"), "{}", html);
    assert!(!html.contains("%%FN"), "{}", html);
}

#[test]
fn test_footnote_continuation_with_fenced_code() {
    // A blank line inside the fence does not end the footnote, and the fence is code
    let html = render_markdown("[^1]: 見出し\n```\n[^2]: コードの中\n\n続き\n```\n");
    assert!(
        html.contains("<pre><code>[^2]: コードの中\n\n続き\n</code></pre>"),
        "{}",
        html
    );
    assert_eq!(html.matches("<blockquote").count(), 1, "{}", html);
}

#[test]
fn test_single_tilde_stays_literal() {
    let html = render_markdown("0 ~100 と 0~ 100");
    assert!(html.contains("<p>0 ~100 と 0~ 100</p>"), "{}", html);
    assert!(!html.contains("<del>"), "{}", html);

    let html = render_markdown("~a **b** c~");
    assert!(html.contains("<p>~a <strong>b</strong> c~</p>"), "{}", html);
}

#[test]
fn test_double_tilde_strikes_through() {
    let html = render_markdown("~~取り消し~~ と 0~ 100");
    assert!(html.contains("<del>取り消し</del> と 0~ 100"), "{}", html);
}

#[test]
fn test_single_tilde_in_code_is_untouched() {
    let html = render_markdown("`~a~`\n\n```\n~b~\n```\n");
    assert!(html.contains("<code>~a~</code>"), "{}", html);
    assert!(html.contains("~b~\n</code></pre>"), "{}", html);
    assert!(!html.contains("<del>"), "{}", html);
}

#[test]
fn test_single_tilde_in_toc() {
    let md = "## 0 ~100 と 0~ 100\n";
    let toc = extract_headings(md);
    assert_eq!(toc[0].text, "0 ~100 と 0~ 100");
    let html = render_markdown(md);
    assert!(
        html.contains(&format!("<h2 id=\"{}\">", toc[0].id)),
        "TOC id {} should match the heading: {}",
        toc[0].id,
        html
    );
}

#[test]
fn test_heading_without_space() {
    let md = "##見出し\n\n###[リンク](other.md)\n\n##Title\n\n```\n##コードの中\n```\n";
    let html = render_markdown(md);
    assert!(html.contains("<h2 id=\"見出し\">見出し</h2>"), "{}", html);
    assert!(html.contains("<h3 id=\"リンク\">"), "{}", html);
    assert!(html.contains("<h2 id=\"title\">Title</h2>"), "{}", html);
    assert!(
        html.contains("<pre><code>##コードの中\n</code></pre>"),
        "A fence should be left alone: {}",
        html
    );

    let toc = extract_headings(md);
    let texts: Vec<&str> = toc.iter().map(|item| item.text.as_str()).collect();
    assert_eq!(texts, ["見出し", "リンク", "Title"]);
}

#[test]
fn test_hashes_that_are_not_headings_stay_literal() {
    let html = render_markdown("####-####-####\n\n#######七個\n\n    ##字下げ\n");
    assert!(html.contains("<p>####-####-####</p>"), "{}", html);
    assert!(html.contains("<p>#######七個</p>"), "{}", html);
    assert!(html.contains("<pre><code>##字下げ"), "{}", html);
    assert!(!html.contains("<h"), "{}", html);
    assert!(extract_headings("####-####-####\n\n#######七個\n").is_empty());
}

#[test]
fn test_last_reference_definition_wins() {
    let html = render_markdown("[リンク]\n\n[リンク]: first.html\n\n[リンク]: last.html\n");
    assert!(
        html.contains("<a href=\"last.html\">リンク</a>"),
        "{}",
        html
    );
    assert!(!html.contains("first.html"), "{}", html);

    // Labels match regardless of case and spacing, and images follow the same rule
    let html =
        render_markdown("![代替][Foo  Bar]\n\n[foo bar]: first.png\n\n[FOO BAR]: last.png\n");
    assert!(html.contains("<img src=\"last.png\""), "{}", html);
}

#[test]
fn test_reference_definition_in_comment_or_fence_does_not_win() {
    let md = "[リンク]\n\n[リンク]: real.html\n\n<!--\n[リンク]: comment.html\n-->\n\n```\n[リンク]: code.html\n```\n";
    let html = render_markdown(md);
    assert!(
        html.contains("<a href=\"real.html\">リンク</a>"),
        "{}",
        html
    );
    assert!(
        html.contains("[リンク]: code.html"),
        "A fence should be left alone: {}",
        html
    );
}

#[test]
fn test_bold_next_to_non_ascii_letter() {
    let html = render_markdown("これは__太字__です");
    assert!(
        html.contains("<p>これは<strong>太字</strong>です</p>"),
        "{}",
        html
    );

    // Only the word after the closing delimiter is non-ASCII
    let html = render_markdown("Set __color__của item");
    assert!(
        html.contains("<p>Set <strong>color</strong>của item</p>"),
        "{}",
        html
    );

    // Twice in a line, and in a heading
    let html = render_markdown("一つ__目__と二つ__目__です");
    assert_eq!(html.matches("<strong>目</strong>").count(), 2, "{}", html);
    let toc = extract_headings("## これは__太字__です\n");
    assert_eq!(toc[0].text, "これは太字です");
}

#[test]
fn test_double_underscore_in_ascii_word_stays_literal() {
    for text in ["SOME__NAME__HERE", "__init__py", "name__suffix__.txt"] {
        let html = render_markdown(text);
        assert!(html.contains(&format!("<p>{}</p>", text)), "{}", html);
    }
    // Bold in CommonMark too
    let html = render_markdown("__init__");
    assert!(html.contains("<strong>init</strong>"), "{}", html);
}

#[test]
fn test_double_underscore_in_url_stays_literal() {
    let html = render_markdown("https://example.com/資料__a__b");
    assert!(html.contains("__a__b"), "{}", html);
    assert!(!html.contains("<strong>"), "{}", html);
}

#[test]
fn test_double_underscore_in_code_or_escaped_stays_literal() {
    let html = render_markdown("`これは__太字__です`");
    assert!(html.contains("<code>これは__太字__です</code>"), "{}", html);

    let html = render_markdown("```\nこれは__太字__です\n```\n");
    assert!(
        html.contains("これは__太字__です\n</code></pre>"),
        "{}",
        html
    );

    let html = render_markdown("これは\\_\\_太字\\_\\_です");
    assert!(html.contains("<p>これは__太字__です</p>"), "{}", html);

    let html = render_markdown("![これは__太字__です](image.png)");
    assert!(html.contains("alt=\"これは__太字__です\""), "{}", html);
}

#[test]
fn test_table_row_wider_than_header_keeps_its_cells() {
    let html = render_markdown("| A | B |\n|:--|--:|\n| 1 | 2 | 余り |\n| 3 | 4 |\n");
    assert!(html.contains("<td>余り</td>"), "{}", html);
    assert_eq!(html.matches("</th>").count(), 3, "{}", html);
    assert!(
        html.contains("<th style=\"text-align: left\">A</th>")
            && html.contains("<th style=\"text-align: right\">B</th>"),
        "The alignment should be kept: {}",
        html
    );
}

#[test]
fn test_table_row_with_empty_extra_cell_does_not_widen() {
    let html = render_markdown("| A | B |\n|---|---|\n| 1 | 2 | |\n");
    assert_eq!(html.matches("</th>").count(), 2, "{}", html);
}

#[test]
fn test_table_escaped_pipe_is_not_a_column() {
    let html = render_markdown("| A | B |\n|---|---|\n| 1 \\| 2 | 3 |\n");
    assert_eq!(html.matches("</th>").count(), 2, "{}", html);
    assert!(html.contains("<td>1 | 2</td>"), "{}", html);
}

#[test]
fn test_table_in_fenced_code_is_not_widened() {
    let md = "```\n| A |\n|---|\n| 1 | 2 |\n```\n";
    let html = render_markdown(md);
    assert!(
        html.contains("| A |\n|---|\n| 1 | 2 |\n</code></pre>"),
        "{}",
        html
    );
}

#[test]
fn test_comment_between_list_lines_keeps_the_list() {
    let html = render_markdown("- 親\n    - 子\n    <!-- メモ -->\n        - 孫\n");
    assert!(html.contains("<li>孫</li>"), "{}", html);
    assert!(!html.contains("<pre>"), "{}", html);
    assert!(!html.contains("メモ"), "{}", html);

    // Over several lines
    let html = render_markdown(
        "- 親\n    - 子\n    <!-- メモ\n        - 消した項目\n    -->\n        - 孫\n",
    );
    assert!(html.contains("<li>孫</li>"), "{}", html);
    assert!(!html.contains("<pre>"), "{}", html);
    assert!(!html.contains("消した項目"), "{}", html);
}

#[test]
fn test_comment_outside_a_list_is_kept() {
    // After a blank line it still separates two lists
    let html = render_markdown("- 一\n\n<!-- メモ -->\n\n- 二\n");
    assert!(html.contains("<!-- メモ -->"), "{}", html);
    assert_eq!(html.matches("<ul>").count(), 2, "{}", html);

    let html = render_markdown("段落\n<!-- メモ -->\n");
    assert!(html.contains("<!-- メモ -->"), "{}", html);

    // Text after the comment is not lost
    let html = render_markdown("- 一\n  <!-- メモ --> 続き\n- 二\n");
    assert!(html.contains("続き"), "{}", html);

    // A reference definition after a list needs it: it cannot follow the item's text
    let html = render_markdown("[リンク]\n\n- 一\n<!-- メモ -->\n[リンク]:#target\n");
    assert!(html.contains("<a href=\"#target\">リンク</a>"), "{}", html);
    assert!(!html.contains("[リンク]:"), "{}", html);
}

#[test]
fn test_comment_in_code_is_kept() {
    // In a list, followed by what looks like an item
    let html = render_markdown("- 一\n  ```\n  <!-- メモ -->\n  - コード\n  ```\n");
    assert!(html.contains("&lt;!-- メモ --&gt;"), "{}", html);

    let html = render_markdown("- 一\n\n      コード\n      <!-- メモ -->\n      - コード\n");
    assert!(html.contains("&lt;!-- メモ --&gt;"), "{}", html);

    // Outside a list
    let html = render_markdown("    <!-- メモ -->\n    - コード\n");
    assert!(html.contains("&lt;!-- メモ --&gt;"), "{}", html);
}

#[test]
fn test_link_destination_with_spaces() {
    let html = render_markdown("[リンク](other.md#a b)");
    assert!(
        html.contains("<a href=\"other.html#a%20b\">リンク</a>"),
        "{}",
        html
    );

    // Angle brackets would not survive in a href, and the cell of a table is no different
    let html = render_markdown("| A |\n|---|\n| [リンク](other.md#a <br>b) |\n");
    assert!(
        html.contains("<a href=\"other.html#a%20%3Cbr%3Eb\">リンク</a>"),
        "{}",
        html
    );
}

#[test]
fn test_link_destination_with_spaces_in_toc() {
    let md = "## [リンク](a b)\n";
    let toc = extract_headings(md);
    assert_eq!(toc[0].text, "リンク");
    let html = render_markdown(md);
    assert!(
        html.contains(&format!("<h2 id=\"{}\">", toc[0].id)),
        "{}",
        html
    );
}

#[test]
fn test_link_destination_forms_left_alone() {
    let html = render_markdown("[リンク](other.md \"題\")");
    assert!(
        html.contains("<a href=\"other.html\" title=\"題\">リンク</a>"),
        "{}",
        html
    );

    let html = render_markdown("[リンク](other.md (題))");
    assert!(
        html.contains("<a href=\"other.html\" title=\"題\">リンク</a>"),
        "{}",
        html
    );

    let html = render_markdown("[リンク](<a b.md>)");
    assert!(
        html.contains("<a href=\"a%20b.html\">リンク</a>"),
        "{}",
        html
    );

    let html = render_markdown("![代替](a b.png)");
    assert!(html.contains("<img src=\"a%20b.png\""), "{}", html);
}

#[test]
fn test_link_destination_with_spaces_in_code_is_untouched() {
    let html = render_markdown("`[リンク](a b)`");
    assert!(html.contains("<code>[リンク](a b)</code>"), "{}", html);

    let html = render_markdown("```\n[リンク](a b)\n```\n");
    assert!(html.contains("[リンク](a b)\n</code></pre>"), "{}", html);
}

#[test]
fn test_link_destination_with_spaces_and_title() {
    let html = render_markdown("[リンク](a b.md \"題 名\")");
    assert!(
        html.contains("<a href=\"a%20b.html\" title=\"題 名\">リンク</a>"),
        "{}",
        html
    );
}

#[test]
fn test_image_with_title() {
    let html = render_markdown("![代替](a.png \"題 名\")");
    assert!(
        html.contains("<img src=\"a.png\" alt=\"代替\" title=\"題 名\""),
        "{}",
        html
    );

    let html = render_markdown("![代替](a b.png '題 名')");
    assert!(
        html.contains("<img src=\"a%20b.png\" alt=\"代替\" title=\"題 名\""),
        "{}",
        html
    );
}

#[test]
fn test_image_path_with_spaces_in_code_is_untouched() {
    let html = render_markdown("`![代替](a b.png)`");
    assert!(html.contains("<code>![代替](a b.png)</code>"), "{}", html);

    let html = render_markdown("```\n![代替](a b.png)\n```\n");
    assert!(html.contains("![代替](a b.png)\n</code></pre>"), "{}", html);
}

#[test]
fn test_image_in_raw_html_with_spaces_or_title() {
    let html = render_markdown("<div>\n![代替](a b.png)\n</div>");
    assert!(
        html.contains("<img src=\"a%20b.png\" alt=\"代替\">"),
        "{}",
        html
    );

    let html = render_markdown("<div>\n![代替](a.png \"題 名\")\n</div>");
    assert!(
        html.contains("<img src=\"a.png\" alt=\"代替\" title=\"題 名\">"),
        "{}",
        html
    );
}

#[test]
fn test_image_syntax_in_raw_html_that_does_not_close_stays_text() {
    let html = render_markdown("<div>\n![代替\n</div>");
    assert_eq!(html.trim(), "<div>\n![代替\n</div>");

    let html = render_markdown("<div>![代替](a</div>\n<div>(b)</div>");
    assert!(!html.contains("<img"), "{}", html);
    assert!(html.contains("<div>(b)</div>"), "{}", html);

    let html = render_markdown("<div>![代替](a</div><div>b)</div>");
    assert!(!html.contains("<img"), "{}", html);

    let html = render_markdown("<div>![代替</div><div>](a.png)</div>");
    assert!(!html.contains("<img"), "{}", html);

    let html = render_markdown("<div>\n![代替](a\nb)\n</div>");
    assert!(!html.contains("<img"), "{}", html);
}

#[test]
fn test_unclosed_image_stays_text() {
    let html = render_markdown("![代替](a.png\n\n次の段落 (かっこ)");
    assert!(!html.contains("<img"), "{}", html);
    assert!(html.contains("<p>![代替](a.png</p>"), "{}", html);
    assert!(html.contains("<p>次の段落 (かっこ)</p>"), "{}", html);

    let html = render_markdown("![代替](a b.png");
    assert!(html.contains("<p>![代替](a b.png</p>"), "{}", html);
}

#[test]
fn test_source_fixes_leave_raw_html_blocks_alone() {
    // Nothing in these is read as Markdown: each line has to come out as it was written
    let lines = [
        "#tbl td { color: red; }",
        "##heading",
        "see [x](a b)",
        "[dup]: one.html",
        "[dup]: two.html",
        "| A |",
        "|---|",
        "| 1 | 2 |",
        "[^1]: a footnote",
        "a reference [^1]",
        "![alt](a b.png)",
    ];
    for tag in ["style", "script", "pre", "textarea"] {
        let md = format!("<{tag}>\n{}\n</{tag}>\n", lines.join("\n"));
        let html = render_markdown(&md);
        for line in lines {
            assert!(html.contains(line), "<{tag}>: {line}: {html}");
        }
        assert!(extract_headings(&md).is_empty(), "<{tag}>");
    }

    // A <div> runs to the blank line, a comment to its end
    let md = format!("<div>\n{}\n</div>\n", lines[..8].join("\n"));
    let html = render_markdown(&md);
    for line in &lines[..8] {
        assert!(html.contains(line), "<div>: {line}: {html}");
    }
    let md = format!("<!--\n{}\n-->\n", lines.join("\n"));
    assert_eq!(render_markdown(&md).trim(), md.trim());

    // After the block, the source is Markdown again
    let html = render_markdown("<style>\n#x { }\n</style>\n\n##見出し\n\n[リンク](a b)\n");
    assert!(html.contains("<h2 id=\"見出し\">"), "{}", html);
    assert!(html.contains("<a href=\"a%20b\">"), "{}", html);
}

#[test]
fn test_fences_in_raw_html_do_not_open() {
    // A fence inside a <pre> opens nothing: the heading after it is still a heading
    let html = render_markdown("<pre>\n```\n</pre>\n\n##見出し\n\n[^1]: 脚注\n");
    assert!(html.contains("<h2 id=\"見出し\">"), "{}", html);
    assert!(html.contains("<blockquote id=\"fn_1\">"), "{}", html);
}

#[test]
fn test_source_fixes_leave_indented_code_alone() {
    let lines = [
        "##code",
        "see [x](a b)",
        "[dup]: one.html",
        "[dup]: two.html",
        "| A |",
        "|---|",
        "| 1 | 2 |",
        "[^1]: a footnote",
        "a reference [^1]",
        "![alt](a b.png)",
    ];
    for indent in ["    ", "\t", "  \t"] {
        let md = lines
            .iter()
            .map(|line| format!("{indent}{line}\n"))
            .collect::<String>();
        let html = render_markdown(&md);
        assert!(html.starts_with("<pre><code>"), "{:?}: {}", indent, html);
        for line in lines {
            assert!(html.contains(line), "{:?}: {}: {}", indent, line, html);
        }
    }
}

#[test]
fn test_hash_after_tab_in_a_paragraph_stays_literal() {
    let html = render_markdown("段落\n\t##続き\n");
    assert!(html.contains("##続き"), "{}", html);
    assert!(!html.contains("<h2"), "{}", html);
}

#[test]
fn test_hash_after_full_width_space_stays_literal() {
    // A full-width space does not indent: the line is text, with nothing added to it
    let html = render_markdown("　##見出し\n");
    assert!(html.contains("<p>　##見出し</p>"), "{}", html);
}

#[test]
fn test_sentence_after_bracket_is_not_a_reference_definition() {
    let md = "[注意]: 次の ページ を参照\n\n本文\n\n[注意]: 別の 文章 です\n";
    let html = render_markdown(md);
    assert!(html.contains("[注意]: 次の ページ を参照"), "{}", html);
    assert!(html.contains("[注意]: 別の 文章 です"), "{}", html);

    // A quote that does not close, and text after a closing bracket
    for line in ["[注意]: a \"b", "[注意]: a \"b\" c", "[注意]: <a> b"] {
        let html = render_markdown(&format!("{line}\n\n本文\n\n{line}\n"));
        assert_eq!(html.matches("[注意]:").count(), 2, "{}", html);
    }
}

#[test]
fn test_last_reference_definition_wins_in_every_form() {
    for first in [
        "[リンク]: first.html",
        "[リンク]:first.html",
        "[リンク]: <first file.html>",
        "[リンク]: first.html \"題 名\"",
        "[リンク]: first.html '題 名'",
        "[リンク]: first.html (題 名)",
    ] {
        let html = render_markdown(&format!("[リンク]\n\n{first}\n\n[リンク]: last.html\n"));
        assert!(html.contains("<a href=\"last.html\""), "{}", html);
        assert!(!html.contains("first"), "{}", html);
    }
}

#[test]
fn test_comment_in_code_or_raw_html_after_a_list_item_is_kept() {
    // Code that looks like a list, then a comment, then an item
    let html = render_markdown("    - コード\n    <!-- メモ -->\n    - コード\n");
    assert!(html.contains("&lt;!-- メモ --&gt;"), "{}", html);

    let html = render_markdown("- 一\n<pre>\n<!-- メモ -->\n- 二\n</pre>\n");
    assert!(html.contains("<!-- メモ -->"), "{}", html);
    assert!(html.contains("- 二"), "{}", html);
}

#[test]
fn test_several_comments_between_list_lines() {
    // The second comment reads as code until the first is gone
    let html = render_markdown("- 親\n<!-- 一 -->\n    - 子\n    <!-- 二 -->\n    - 子の次\n");
    assert!(!html.contains("<!--"), "{}", html);
    assert!(!html.contains("<pre>"), "{}", html);
    assert!(html.contains("<li>子</li>"), "{}", html);
    assert!(html.contains("<li>子の次</li>"), "{}", html);
}

#[test]
fn test_footnote_continuation_with_raw_html() {
    // A blank line inside the <pre> does not end the footnote
    let html = render_markdown("[^1]: 一行目\n<pre>\n一\n\n二\n</pre>\n\n本文\n");
    assert!(html.contains("<pre>\n一\n\n二\n</pre>"), "{}", html);
    assert_eq!(html.matches("<blockquote").count(), 1, "{}", html);
}

#[test]
fn test_footnote_continuation_with_indented_fence() {
    // Once dedented it is a fence, and a blank line inside it does not end the footnote
    let html = render_markdown("[^1]: 一行目\n    ```\n    一\n\n    二\n    ```\n\n本文\n");
    assert!(
        html.contains("<pre><code>一\n\n二\n</code></pre>"),
        "{}",
        html
    );
    assert!(html.contains("<p>本文</p>"), "{}", html);
}

#[test]
fn test_urls_and_images_in_verbatim_elements_are_left_alone() {
    for tag in ["script", "style", "textarea", "pre", "code"] {
        let md = format!(
            "<{tag}>\nhttps://example.com/in\nif (a<b) x = ![alt](in.png);\n</{tag}>\n\nhttps://example.com/out\n\n<div>![alt](out.png)</div>\n"
        );
        let html = render_markdown(&md);
        assert!(
            html.contains("\nhttps://example.com/in\n"),
            "<{tag}>: {html}"
        );
        assert!(html.contains("![alt](in.png)"), "<{tag}>: {html}");
        // The element ends where it ends, even after a "<" that opens nothing
        assert!(
            html.contains("<a href=\"https://example.com/out\""),
            "<{tag}>: {html}"
        );
        assert!(html.contains("<img src=\"out.png\""), "<{tag}>: {html}");
    }

    // A tag that only starts like one of them
    let html = render_markdown("<preview>https://example.com/a</preview>\n");
    assert!(
        html.contains("<a href=\"https://example.com/a\""),
        "{}",
        html
    );

    // Code inside <pre> does not end the <pre>
    let html = render_markdown("<pre><code>一</code>\nhttps://example.com/a\n</pre>\n");
    assert!(!html.contains("<a href"), "{}", html);
}

#[test]
fn test_footnote_label_with_symbols_is_a_reference_when_defined() {
    let html = render_markdown("| a |\n|---|\n| x[^3-1] |\n\n[^3-1]: note\n");
    assert!(
        html.contains("<td>x<sup><a href=\"#fn_3-1\" id=\"reffn_3-1\">3-1</a></sup></td>"),
        "{}",
        html
    );
    assert!(html.contains("<blockquote id=\"fn_3-1\">"), "{}", html);
    assert!(!html.contains("%%FN"), "{}", html);

    for label in ["a_b", "1.2"] {
        let html = render_markdown(&format!("x[^{0}]\n\n[^{0}]: note\n", label));
        assert!(
            html.contains(&format!("id=\"reffn_{}\"", label)),
            "{}",
            html
        );
    }
}

#[test]
fn test_footnote_label_with_symbols_stays_text_without_a_definition() {
    let html = render_markdown("Use [^a-z] here.\n\n[^1]: note\n");
    assert!(html.contains("Use [^a-z] here."), "{}", html);
    assert!(!html.contains("reffn_a-z"), "{}", html);
}

#[test]
fn test_footnote_label_of_letters_and_digits_needs_no_definition() {
    let html = render_markdown("x[^7]\n");
    assert!(html.contains("id=\"reffn_7\""), "{}", html);
}

#[test]
fn test_list_items_are_loose_one_by_one() {
    let html = render_markdown("- a\n- b\n\n- c\n- d\n");
    assert!(html.contains("<li>a</li>"), "{}", html);
    assert!(html.contains("<li>\n<p>b</p>\n</li>"), "{}", html);
    assert!(html.contains("<li>\n<p>c</p>\n</li>"), "{}", html);
    assert!(html.contains("<li>d</li>"), "{}", html);
}

#[test]
fn test_blank_line_in_a_nested_list_makes_the_item_loose() {
    let html = render_markdown("- a\n  - b\n\n  - c\n- d\n");
    assert!(html.contains("<p>a</p>"), "{}", html);
    assert!(html.contains("<p>b</p>"), "{}", html);
    assert!(html.contains("<p>c</p>"), "{}", html);
    assert!(html.contains("<li>d</li>"), "{}", html);
}

#[test]
fn test_loose_list_item_keeps_text_and_nested_list_apart() {
    let html = render_markdown("- a\n\n  - c\n- d\n");
    assert!(html.contains("<p>a</p>"), "{}", html);
    assert!(html.contains("<li>c</li>"), "{}", html);
    assert!(html.contains("<li>d</li>"), "{}", html);
}

#[test]
fn test_paragraph_added_to_a_list_item_holds_its_inline_markup() {
    for (markdown, expected) in [
        ("*a*", "<em>a</em>"),
        ("**a**", "<strong>a</strong>"),
        ("~~a~~", "<del>a</del>"),
        ("[a](b.html)", "<a href=\"b.html\">a</a>"),
        ("![a](b.png)", "<img src=\"b.png\" alt=\"a\" />"),
    ] {
        // Tight as CommonMark reads it, so the parser does not add the paragraph itself
        let html = render_markdown(&format!("- {} x\n  - c\n\n  - d\n- e\n", markdown));
        assert!(html.contains(&format!("<p>{} x</p>", expected)), "{}", html);
    }
}

#[test]
fn test_blank_line_after_the_last_item_does_not_make_it_loose() {
    let html = render_markdown("- a\n- b\n\nafter\n");
    assert!(html.contains("<li>a</li>"), "{}", html);
    assert!(html.contains("<li>b</li>"), "{}", html);
}

#[test]
fn test_line_of_spaces_is_a_blank_line_in_a_list() {
    let html = render_markdown("- a\n   \n- b\n- c\n");
    assert!(html.contains("<p>a</p>"), "{}", html);
    assert!(html.contains("<p>b</p>"), "{}", html);
    assert!(html.contains("<li>c</li>"), "{}", html);
}

#[test]
fn test_list_in_a_quote_is_left_to_the_parser() {
    let html = render_markdown("> - a\n>\n> - b\n");
    assert!(html.contains("<p>a</p>"), "{}", html);
    assert!(html.contains("<p>b</p>"), "{}", html);
}

#[test]
fn test_line_without_a_pipe_ends_a_table() {
    let html = render_markdown("| a |\n|---|\n| 1 |\nnote\n\nafter\n");
    assert!(html.contains("<p>note</p>"), "{}", html);
    assert_eq!(html.matches("<tr>").count(), 2, "{}", html);
}

#[test]
fn test_row_of_a_table_with_outer_pipes_starts_with_a_pipe() {
    let html = render_markdown("| a | b |\n|---|---|\n| 1 | 2 |\nc | d\n");
    assert!(html.contains("<p>c | d</p>"), "{}", html);
    assert_eq!(html.matches("<tr>").count(), 2, "{}", html);
}

#[test]
fn test_row_of_a_table_without_outer_pipes_has_a_pipe_anywhere() {
    let html = render_markdown("a | b\n---|---\n1 | 2\nc | d\nnote\n");
    assert!(html.contains("<td>c</td>"), "{}", html);
    assert!(html.contains("<p>note</p>"), "{}", html);
}

#[test]
fn test_line_without_a_pipe_after_a_table_in_a_list_is_left_to_the_parser() {
    let html = render_markdown("- x\n\n  | a |\n  |---|\n  | 1 |\n  note\n- y\n");
    assert!(html.contains("<td>note</td>"), "{}", html);
    assert!(html.contains("<li>y</li>"), "{}", html);
}

#[test]
fn test_line_without_a_pipe_between_rows_stays_in_the_table() {
    let html = render_markdown("| a |\n|---|\n| 1 |\nnote\n| 2 |\nend\n");
    assert!(html.contains("<td>note</td>"), "{}", html);
    assert!(html.contains("<td>2</td>"), "{}", html);
    assert!(html.contains("<p>end</p>"), "{}", html);
}

#[test]
fn test_footnote_continuation_is_read_as_markdown() {
    // The first line alone reads as a reference definition, and the indented lines as code
    let html = render_markdown("[^1]: 注\n    [リンク](a b.md)\n");
    assert!(
        html.contains("<a href=\"a%20b.html\">リンク</a>"),
        "{}",
        html
    );

    let html = render_markdown("[^1]: 注\n    ![代替](a b.png)\n");
    assert!(html.contains("<img src=\"a%20b.png\""), "{}", html);

    let html = render_markdown("[^1]: 注\n    続き[^2]\n\n[^2]: 次\n");
    assert!(
        html.contains("<a href=\"#fn_2\" id=\"reffn_2\">"),
        "{}",
        html
    );

    let html = render_markdown("[^1]: 注\n    - 一\n    <!-- メモ -->\n    - 二\n");
    assert!(!html.contains("メモ"), "{}", html);

    let html = render_markdown("[^1]: 注\n    | A |\n    |---|\n    | 1 | 2 |\n");
    assert!(html.contains("<td>2</td>"), "{}", html);

    let html = render_markdown("![代替][a]\n\n[^1]: 注\n    [a]: first.png\n    [a]: last.png\n");
    assert!(html.contains("<img src=\"last.png\""), "{}", html);
}

/// The HTML without its whitespace, to compare the structure of a list
#[cfg(test)]
fn without_whitespace(html: &str) -> String {
    html.split_whitespace().collect()
}

#[test]
fn test_list_items_nest_by_the_indentation_of_their_own_list() {
    // HonKit: an item only ends at a marker with the same indentation
    assert_eq!(
        without_whitespace(&render_markdown("- one\n   - two\n  - three\n- four\n")),
        "<ul><li>one<ul><li>two<ul><li>three</li></ul></li></ul></li><li>four</li></ul>"
    );
    assert_eq!(
        without_whitespace(&render_markdown("- one\n     - two\n    - three\n")),
        "<ul><li>one<ul><li>two<ul><li>three</li></ul></li></ul></li></ul>"
    );
    // Siblings with the same indentation stay siblings
    assert_eq!(
        without_whitespace(&render_markdown("- one\n    - two\n    - three\n- four\n")),
        "<ul><li>one<ul><li>two</li><li>three</li></ul></li><li>four</li></ul>"
    );
}

#[test]
fn test_list_item_strips_the_indentation_of_the_items_around_it() {
    assert_eq!(
        without_whitespace(&render_markdown("- a\n - b\n  - c\n")),
        "<ul><li>a<ul><li>b</li><li>c</li></ul></li></ul>"
    );
}

#[test]
fn test_marker_in_a_code_block_of_a_list_item_opens_no_item() {
    let html = render_markdown("- one\n\n        - code\n         - more\n");
    assert!(html.contains("<pre><code>  - code\n   - more"), "{}", html);
}

#[test]
fn test_list_item_with_two_markers_on_a_line() {
    assert_eq!(
        without_whitespace(&render_markdown("- 1. one\n    - two\n")),
        "<ul><li><ol><li>one<ul><li>two</li></ul></li></ol></li></ul>"
    );
    assert_eq!(
        without_whitespace(&render_markdown(
            "- 1. one\n     - two\n     - three\n- four\n"
        )),
        "<ul><li><ol><li>one<ul><li>two</li><li>three</li></ul></li></ol></li><li>four</li></ul>"
    );
}

#[test]
fn test_lines_after_a_blank_line_stay_in_the_deepest_list_item_they_reach() {
    assert_eq!(
        without_whitespace(&render_markdown("- one\n    - two\n\n    text\n")),
        "<ul><li><p>one</p><ul><li><p>two</p><p>text</p></li></ul></li></ul>"
    );
    // With no indentation, they end the list
    assert_eq!(
        without_whitespace(&render_markdown("- one\n    - two\n\ntext\n")),
        "<ul><li>one<ul><li>two</li></ul></li></ul><p>text</p>"
    );
}

#[test]
fn test_fence_and_heading_right_after_a_list_item_stay_in_it() {
    assert_eq!(
        without_whitespace(&render_markdown("- one\n```\ncode\nmore\n```\n- two\n")),
        "<ul><li>one<pre><code>codemore</code></pre></li><li>two</li></ul>"
    );
    assert_eq!(
        without_whitespace(&render_markdown("- one\n### Title\n- two\n")),
        "<ul><li>one<h3id=\"title\">Title</h3></li><li>two</li></ul>"
    );
    assert_eq!(
        without_whitespace(&render_markdown("- one\n##Title\n")),
        "<ul><li>one<h2id=\"title\">Title</h2></li></ul>"
    );
    // After a blank line, they end the list
    assert_eq!(
        without_whitespace(&render_markdown("- one\n\n```\ncode\n```\n")),
        "<ul><li>one</li></ul><pre><code>code</code></pre>"
    );
    assert_eq!(
        without_whitespace(&render_markdown("- one\n\n### Title\n")),
        "<ul><li>one</li></ul><h3id=\"title\">Title</h3>"
    );
}

#[test]
fn test_heading_lines_that_stay_in_a_list_item() {
    assert!(is_atx_heading_in_honkit("##Title"));
    assert!(is_atx_heading_in_honkit("### Title"));
    assert!(!is_atx_heading_in_honkit("#"));
    assert!(!is_atx_heading_in_honkit("####### seven"));
    assert!(!is_atx_heading_in_honkit("text"));
}

#[test]
fn test_list_item_number_other_than_one_stays_in_the_text() {
    // A case where HonKit restarts the numbers at 1: the number as written stays
    let html = render_markdown("- one\n  2. two\n");
    assert!(!html.contains("<ol"), "{}", html);
    assert!(html.contains("2. two"), "{}", html);
}

#[test]
fn test_code_block_in_a_list_item_stays_code() {
    let html = render_markdown("- one\n\n        code\n- two\n");
    assert!(html.contains("<pre><code>  code"), "{}", html);
    assert_eq!(html.matches("<li>").count(), 2, "{}", html);
}

#[test]
fn test_lone_inline_tag_is_a_part_of_a_paragraph() {
    let html = render_markdown("<span class=\"x\">\n**text**\n</span>\n");
    assert!(html.starts_with("<p><span class=\"x\">"), "{}", html);
    assert!(html.contains("<strong>text</strong>"), "{}", html);

    let html = render_markdown("<br>\n");
    assert!(html.starts_with("<p><br>"), "{}", html);

    let html = render_markdown("<MARK>\ntext\n</MARK>\n");
    assert!(html.starts_with("<p><MARK>"), "{}", html);

    // A block element stays a block, and so does a tag that only starts like an inline one
    let html = render_markdown("<div>\n**text**\n</div>\n");
    assert!(html.contains("**text**"), "{}", html);
    let html = render_markdown("<spanish>\n**text**\n</spanish>\n");
    assert!(html.contains("**text**"), "{}", html);
    // More than the tag on its line
    assert!(!is_lone_inline_tag("<span>text</span>"));
    assert!(!is_lone_inline_tag("<span><b>"));
    assert!(is_lone_inline_tag("  </span>  "));
    assert!(is_lone_inline_tag("<br/>"));
}

#[test]
fn test_braces_at_the_end_of_a_heading_stay() {
    let html = render_markdown("### Title {name}\n");
    assert!(html.contains(">Title {name}</h3>"), "{}", html);
    let html = render_markdown("Title {name}\n=====\n");
    assert!(html.contains(">Title {name}</h1>"), "{}", html);

    // An id stays an id
    let html = render_markdown("### Title {#my-id}\n");
    assert!(html.contains("id=\"my-id\""), "{}", html);
    assert!(!html.contains('{'), "{}", html);
}

#[test]
fn test_reference_link_followed_by_parentheses() {
    // A defined label ends the link, and the parentheses are text
    let html = render_markdown("[a][b](c d)\n\n[b]: e.png\n");
    assert!(html.contains("<a href=\"e.png\">a</a>(c d)"), "{}", html);

    // No such label: the second brackets and the parentheses are a link
    let html = render_markdown("[a][x](c d)\n");
    assert!(html.contains("[a]<a href=\"c%20d\">x</a>"), "{}", html);
}

#[test]
fn test_heading_marker_followed_by_a_hyphen_and_text() {
    let html = render_markdown("###-text\n");
    assert!(html.contains(">-text</h3>"), "{}", html);

    // A mask, not a heading
    let html = render_markdown("###-###\n");
    assert!(!html.contains("<h3"), "{}", html);
}

#[test]
fn test_md_links_to_other_sites_stay() {
    let html = render_markdown("[a](https://example.com/a.md) [b](//example.com/b.md)\n");
    assert!(
        html.contains("href=\"https://example.com/a.md\""),
        "{}",
        html
    );
    assert!(html.contains("href=\"//example.com/b.md\""), "{}", html);
}

#[test]
fn test_book_links_read_md_links_by_what_is_a_page() {
    let mut links = BookLinks::default();
    links.add_page("en/a/page.md");
    links.add_page("ja/b/other.md");
    links.add_page("ja/../en/moved.md");
    links.add_home("en/README.md");
    links.add_home("en/a/README.md");
    links.add_page("en/a/README.md");

    let page = "en/a/page.md";
    assert_eq!(links.resolve(page, "page.md"), "page.html");
    assert_eq!(links.resolve(page, "page.md#part"), "page.html#part");
    assert_eq!(
        links.resolve(page, "../../ja/b/other.md"),
        "../../ja/b/other.html"
    );
    assert_eq!(links.resolve(page, "/en/moved.md"), "/en/moved.html");
    assert_eq!(links.resolve(page, "README.md"), "README.html");
    // A README that only has an index.html
    assert_eq!(links.resolve(page, "../README.md"), "../index.html");
    assert!(links.linked_files().is_empty());

    // Anything else stays, and the file gets copied
    assert_eq!(
        links.resolve(page, "notes/extra.md#part"),
        "notes/extra.md#part"
    );
    assert_eq!(links.resolve(page, "with%20space.md"), "with%20space.md");
    assert_eq!(
        links.linked_files(),
        ["en/a/notes/extra.md", "en/a/with space.md"]
    );

    // Out of the site, other sites and other files are none of its business
    assert_eq!(links.resolve(page, "../../../up.md"), "../../../up.md");
    assert_eq!(
        links.resolve(page, "https://example.com/a.md"),
        "https://example.com/a.md"
    );
    assert_eq!(links.resolve(page, "image.png"), "image.png");
    assert_eq!(links.linked_files().len(), 2);
}

#[test]
fn test_normalize_site_path() {
    assert_eq!(
        normalize_site_path("a/./b/../c.md").as_deref(),
        Some("a/c.md")
    );
    assert_eq!(normalize_site_path("a/../../b.md"), None);
    assert_eq!(normalize_site_path("../b.md"), None);
}

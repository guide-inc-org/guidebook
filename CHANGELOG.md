# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Fixed
- HonKit compatibility, footnotes: the lines after a footnote's first line are rendered with the rest of the document, so reference-style images and links work there (`![][label]` no longer turned into `!<a href=""></a>`, a definition right after a footnote is no longer swallowed, a fenced block may contain blank lines). The first line's inline formatting (`` `code` ``, `**bold**`, links) is rendered instead of shown as is
- Footnotes: a URL right before a footnote reference no longer swallows it (broken `<a>`). A line that looks like a definition inside an HTML comment or `<pre>` is left as written (it used to close the comment early and expose the rest of the page)
- HonKit compatibility, strikethrough: a single `~` stays literal (`0 ~100 and 0~ 100`), only `~~text~~` strikes through, in the TOC too
- HonKit compatibility, headings: `##Heading` without a space is a heading, in the TOC too. Masks like `####-####` and lines indented as code stay literal
- HonKit compatibility, reference definitions: when a label is defined twice, the last definition wins (definitions in comments and code do not count)
- HonKit compatibility, bold: `__bold__` touching a letter is bold when non-ASCII text is involved (`これは__太字__です`). ASCII identifiers (`SOME__NAME__HERE`), URLs, code and `\_\_` stay literal
- HonKit compatibility, tables: cells beyond the header are kept by widening the header and the separator to the widest row with content (they used to be dropped)
- HonKit compatibility, lists: an HTML comment between two list items no longer ends the list (the deeper item after it became a code block)
- HonKit compatibility, links: a link destination with spaces is a link (`[text](a b.md)`), with the spaces and angle brackets percent-encoded (it stayed as text)
- HonKit compatibility, images: a title after the path is a title (`![alt](a.png "title")` was a broken image), for links with spaces in the path too. An image with a space in its path is no longer rewritten inside code, and an image without its closing parenthesis no longer swallows the text up to the next `)`. In a raw HTML block, an image with spaces in its path or with a title works, and text that only starts like an image stays text
- HonKit compatibility, lists: whether the text of a list item is in a paragraph is decided item by item, not for the whole list (a blank line between two items added paragraphs to every item, and a blank line in a nested list added none to its parent)
- HonKit compatibility, tables: a line without a pipe right under a table is a paragraph, not a row with one cell. A line with more rows after it stays in the table (HonKit shows those rows as text)
- HonKit compatibility, footnotes: a label with `-`, `_` or `.` (`[^3-1]`) is a reference when the document defines it. Without a definition (`[^a-z]`) it stays literal
- Source fixes: footnotes, link destinations with spaces, image paths, table rows, headings, reference definitions and comments between list items are only fixed on lines the parser reads as Markdown, so raw HTML blocks and indented code are left alone (`#x` in a `<style>`, `[a]: b` and `](a b)` in a `<pre>`). A `#` after a tab or a full-width space stays literal, and `[note]: a whole sentence` is not a reference definition
- Autolink and images: URLs and images inside `<script>`, `<style>` and `<textarea>` are left alone
- Favicon: the build now actually emits `gitbook/images/favicon.ico` / apple-touch-icon (previously only linked, always 404) and content pages link them with the depth-aware root path
- Front matter: a `----` line no longer closes the front matter early (closing delimiter must be exactly `---`); UTF-8 BOM-prefixed files keep their front matter
- Glossary: a `>` inside a quoted attribute value no longer breaks the tag tracker (spans were injected into attributes); `class="no-glossary"` on void/self-closing elements (`<img/>` etc.) no longer disables the glossary for the rest of the page
- SUMMARY.md: a second link in the same list item no longer silently replaces the first entry
- Nunjucks: fences indented in list items, `~~~` fences, and longer ` ```` ` runs are now protected from template processing (previously only column-0 backtick fences were)
- Remote images: URLs are entity-decoded before download (`&amp;` broke multi-parameter/signed URLs); non-image responses (e.g. HTML error pages served with 200) are refused instead of being saved as broken `.png`; downloaded images now resolve from nested pages; self-closing `<img/>` no longer gains a double slash
- Inline SVG: nested `<svg>` elements are externalized whole (the previous regex cut them at the first `</svg>`, emitting invalid XML); `width`/`height` presence is checked on the root tag only (child `<rect width=...>` no longer blocks size injection); externalized files resolve from nested pages
- Links: root-relative `<img src="/...">` is now depth-adjusted like `href` (was 404 on nested pages); Windows backslash normalization and leading-slash removal in attributes actually fire (the detection was structurally dead code); sidebar and sitemap `href` values are HTML-escaped; `.md`/`.adoc` → `.html` conversion only touches the trailing extension
- Assets: output is now self-contained real copies instead of absolute-path symlinks (deployed `_book` no longer breaks); changed assets are refreshed on rebuild (previously never); a broken symlink in assets warns instead of aborting the build; building with `-o` inside the source no longer re-copies the output into itself
- Build: stale files from deleted/renamed pages are cleaned from the output (dot entries like `.git` preserved); a non-UTF-8 page warns and builds instead of aborting everything; repeated `@import` of the same snippet works (only true cycles are rejected)
- Search index: pages referenced with anchors (`file.md#sec`) are indexed; the index is built from the processed content (front matter no longer leaks in, imported/template content is searchable)
- Serve: editing `.adoc` pages and images now triggers hot reload; extensionless URLs get the livereload script
- Self-update: if the final rename fails the previous binary is restored instead of leaving no executable
- book.json: `openapi` paths are validated against path traversal (absolute / `..` rejected)
- Glossary: multi-byte terms (Japanese) no longer eat the text that follows them (byte-length vs char-count skip bug)
- Glossary: word-boundary detection now uses script classes (kanji/hiragana/katakana/alphanumeric), so Japanese terms inside Japanese sentences actually match; partial matches inside same-script compounds (e.g. 専門用語集) are still rejected
- LANGS.md: malformed lines (missing bracket/paren) are skipped instead of panicking and aborting the whole build
- Nunjucks: template blocks spanning fenced code blocks (`{% if %}` before / `{% endif %}` after) now work — the document is rendered in a single pass with code blocks protected by placeholders
- Links: `.md` → `.html` conversion is now scoped to `href` attributes; literal text like `chapter1.md#section` inside inline code is no longer rewritten
- Footnotes: `[^abc]` inside fenced code blocks and inline code spans is no longer converted to a footnote reference (regex character classes in code samples stay intact)
- Headings: duplicate headings get deduplicated ids (`概要`, `概要-1`, `概要-2` — github-slugger behavior); TOC extraction mirrors the same logic (including custom `{#id}`) so sidebar anchors always match

### Security
- Update vulnerable transitive dependencies (14 RustSec advisories): aws-lc-sys 0.42.0, rustls-webpki 0.103.13, quinn-proto 0.11.16, tar 0.4.46, crossbeam-epoch 0.9.20, rand, anyhow (Cargo.lock only)
- Add path traversal protection in dev server (canonicalize check against temp_dir)
- Add image download size limit (50 MB max)
- Add SHA256 checksum verification for self-update binary downloads
- Remove `.unwrap()` from HTTP header creation (use safe fallbacks)
- Replace `unwrap_or_default()` with proper error responses (404/500) for file reads

### Changed
- Cache compiled regex patterns using `LazyLock` for better performance
- Remove unused `output_dir` field from `ImageDownloader`

### Added
- Integration tests for full build pipeline
- Malformed input tests for error resilience
- CI workflow with `cargo test`, `cargo clippy`, `cargo fmt --check`, and `cargo audit`
- CONTRIBUTING.md with development guide

## [0.1.65] - 2026-02-14

### Changed
- Version bump

## [0.1.13] - 2025-12-25

### Fixed
- Enable collapsible.js by default (no book.json required)

## [0.1.12] - 2025-12-25

### Fixed
- Fix SPA navigation URL accumulation bug

## [0.1.10] - 2025-12-25

### Fixed
- Fix image paths (remove `<base>` tag)

### Added
- Release workflow for multi-platform builds

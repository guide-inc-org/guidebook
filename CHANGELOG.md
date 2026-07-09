# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Fixed
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

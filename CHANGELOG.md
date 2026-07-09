# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Fixed
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

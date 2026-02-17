# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Security
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

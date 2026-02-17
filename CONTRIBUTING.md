# Contributing to Guidebook

## Prerequisites

- [Rust](https://rustup.rs/) (latest stable)

## Building

```bash
# Debug build
cargo build

# Release build
cargo build --release
```

## Testing

```bash
# Run all tests (unit + integration)
cargo test

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test integration_test
cargo test --test malformed_input_test
```

## Code Quality

```bash
# Check formatting
cargo fmt --check

# Auto-format
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Security audit
cargo install cargo-audit
cargo audit
```

## Project Structure

```
src/
├── main.rs              # CLI entry point, dev server, self-update
├── parser/
│   ├── mod.rs           # Parser module exports
│   ├── book_config.rs   # book.json configuration
│   ├── frontmatter.rs   # YAML front matter extraction
│   ├── glossary.rs      # Glossary term definitions
│   ├── langs.rs         # LANGS.md for multi-language support
│   └── summary.rs       # SUMMARY.md table of contents
├── builder/
│   ├── mod.rs           # Build orchestration
│   ├── renderer.rs      # Markdown/AsciiDoc to HTML
│   ├── template.rs      # Tera HTML templates
│   ├── images.rs        # Remote image downloading
│   ├── svg.rs           # SVG optimization
│   ├── nunjucks.rs      # Nunjucks template processing
│   ├── sitemap.rs       # Sitemap directives
│   └── openapi.rs       # Swagger UI generation
templates/               # Embedded static assets (CSS/JS)
tests/                   # Integration tests
```

## Making Changes

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Run the full check suite:
   ```bash
   cargo fmt --check && cargo clippy -- -D warnings && cargo test
   ```
5. Commit your changes
6. Push and open a Pull Request

## Security Model

### Dev Server (`guidebook serve`)

The dev server enforces strict path containment:

- **Rejected inputs**: Any URL path containing `..` after URL decoding is rejected with `403 Forbidden`
- **Path validation**: Both direct file access and `.html` fallback paths are validated via `canonicalize()` + `starts_with()` to ensure they resolve within the temporary build directory
- **Symlink protection**: Symlinks that resolve outside the build directory are blocked
- **Defense in depth**: Three layers — URL-level `..` rejection, component-level `ParentDir` check, and filesystem-level `canonicalize()` verification

### Remote Image Downloading (`fetchRemoteImages`)

- **Size limit**: 50 MB per image (`MAX_IMAGE_SIZE`)
- **Content-Length pre-check**: If the server advertises `Content-Length` exceeding the limit, the download is rejected before any data is read
- **Streaming enforcement**: The response body is read in 8 KB chunks; if the cumulative size exceeds the limit mid-stream, the download is aborted immediately
- **No full buffering**: The entire response is never loaded into memory at once before size validation

### Self-Update (`guidebook update`)

Trust model:

- **Source**: Downloads from `https://github.com/guide-inc-org/guidebook/releases/` only
- **Checksum required**: SHA256 checksum verification is mandatory. If the release notes do not contain a checksum for the downloaded artifact, the update is refused
- **Checksum format**: Release notes must include lines in the format: `<sha256hash>  <filename>` (64 hex chars, two spaces, filename)
- **No bypass**: There is no `--skip-verify` flag or equivalent. Unverified binaries are never installed

### Rejected Input Summary

| Component | Input | Response |
|-----------|-------|----------|
| `serve` | URL with `..` (raw or encoded) | `403 Forbidden` |
| `serve` | Path resolving outside build dir | `403 Forbidden` |
| `serve` | Symlink escaping build dir | `403 Forbidden` |
| `images` | Image > 50 MB (Content-Length) | Error, skip image |
| `images` | Image > 50 MB (streaming) | Abort mid-download |
| `update` | Release without checksum | Error, refuse update |
| `update` | Checksum mismatch | Error, refuse update |

## Release Process

1. Update version in `Cargo.toml`
2. Run `cargo check` to update `Cargo.lock`
3. Update `CHANGELOG.md`
4. Commit and push to `main`
5. Create and push a tag: `git tag vX.Y.Z && git push origin vX.Y.Z`
6. GitHub Actions builds multi-platform binaries automatically
7. Optionally publish to crates.io: `cargo publish`
8. **Important**: Include SHA256 checksums in the release notes for all binary artifacts

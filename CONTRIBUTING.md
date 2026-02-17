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

## Release Process

1. Update version in `Cargo.toml`
2. Run `cargo check` to update `Cargo.lock`
3. Update `CHANGELOG.md`
4. Commit and push to `main`
5. Create and push a tag: `git tag vX.Y.Z && git push origin vX.Y.Z`
6. GitHub Actions builds multi-platform binaries automatically
7. Optionally publish to crates.io: `cargo publish`

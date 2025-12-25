# CLAUDE.md - guidebook

A static site generator compatible with HonKit/GitBook.

## Project Overview

- **Language:** Rust
- **Published to:** crates.io (`cargo install guidebook`)
- **Repository:** https://github.com/guide-inc-org/guidebook

## Build & Test

```bash
# Build
cargo build --release

# Test
cargo test

# Build documentation locally
./target/release/guidebook build

# Start dev server
./target/release/guidebook serve
```

## Release Procedure

1. Update version in `Cargo.toml`
2. Run `cargo check` to update `Cargo.lock`
3. Commit & push (includes both Cargo.toml and Cargo.lock)
4. Create and push tag (binary is auto-generated to GitHub Releases)
5. Publish to crates.io

```bash
# 1. Update version in Cargo.toml, then:
cargo check

# 2. Commit everything (Cargo.toml + Cargo.lock)
git add -A && git commit -m "Bump version to vX.Y.Z"
git push origin main

# 3. Create & push tag (triggers release workflow)
git tag vX.Y.Z
git push origin vX.Y.Z

# 4. Publish to crates.io
cargo publish
```

**Important:** Always run `cargo check` after updating version to ensure `Cargo.lock` is updated before committing. This prevents having to make a separate commit for `Cargo.lock` after publishing.

## Directory Structure

```
src/
├── main.rs          # CLI entry point
├── builder/
│   ├── mod.rs       # Build process
│   ├── renderer.rs  # Markdown to HTML conversion
│   └── template.rs  # HTML template
├── parser/
│   ├── mod.rs
│   ├── book_config.rs  # book.json parser
│   ├── langs.rs        # LANGS.md parser (multi-language support)
│   └── summary.rs      # SUMMARY.md parser
templates/
├── gitbook.css      # Stylesheet
├── gitbook.js       # Client-side JS
├── collapsible.js   # Collapsible sections
└── search.js        # Search functionality
```

## Important Design Decisions

### Do NOT use `<base>` tag

**Reason:** Using `<base href>` causes relative image paths in markdown (e.g., `../../../assets/...`) to resolve from base, breaking when deployed to subdirectories.

**Solution:** Embed `root_path` directly into CSS/JS/links (same approach as HonKit)

## CI/CD

### Release Workflow

`.github/workflows/release.yml` - Publishes Linux binary to GitHub Releases on tag push.

Consumers download pre-built binary:
```yaml
- name: Install guidebook
  run: |
    curl -sL https://github.com/guide-inc-org/guidebook/releases/latest/download/guidebook-linux-x86_64.tar.gz | tar xz
    ./guidebook build
```

## Changelog

- **2025-12-25 v0.1.13:** Enable collapsible.js by default (no book.json required)
- **2025-12-25 v0.1.12:** Fix SPA navigation URL accumulation bug
- **2025-12-25 v0.1.10:** Fix image paths (remove `<base>` tag), add release workflow

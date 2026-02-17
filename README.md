# guidebook

A fast, HonKit/GitBook-compatible static site generator written in Rust.

[![Crates.io](https://img.shields.io/crates/v/guidebook.svg)](https://crates.io/crates/guidebook)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Feedback](https://img.shields.io/badge/Feedback-Issues%20%26%20Requests-blue)](https://github.com/guide-inc-org/guidebook-feedback)

## Quick Start

### Install

**macOS / Linux:**
```bash
curl -fsSL https://raw.githubusercontent.com/guide-inc-org/guidebook/main/install.sh | sh
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/guide-inc-org/guidebook/main/install.ps1 | iex
```

**Via Cargo (alternative):**
```bash
cargo install guidebook
```

### Create and Preview a Book

```bash
# Initialize a new book
guidebook init my-book
cd my-book

# Start preview server with hot reload
guidebook serve

# Open http://localhost:4000
```

### Build for Production

```bash
guidebook build -o _book
```

### Update

```bash
guidebook update
```

## Features

- **Fast** - Built with Rust for maximum performance
- **HonKit/GitBook Compatible** - Drop-in replacement
- **Hot Reload** - Live preview with auto-refresh
- **Multi-language Support** - Build books in multiple languages
- **Mermaid Diagrams** - Native support for diagrams
- **Collapsible Chapters** - Expandable sidebar navigation
- **Full-text Search** - Built-in search functionality
- **AsciiDoc Support** - Use `.adoc` files alongside Markdown
- **KaTeX Math** - Render mathematical formulas
- **Glossary** - Auto-linked term definitions with tooltips
- **Templates** - Nunjucks/Jinja2 conditionals, loops, and filters
- **OpenAPI / Swagger UI** - Generate API documentation
- **Self-update** - Update with a single command

## Project Structure

```
your-book/
├── book.json       # Configuration (optional)
├── README.md       # Introduction
├── SUMMARY.md      # Table of contents
└── chapter1.md
```

### SUMMARY.md

```markdown
# Summary

* [Introduction](README.md)
* [Chapter 1](chapter1.md)
  * [Section 1.1](chapter1/section1.md)
```

## Migration from HonKit

guidebook is a drop-in replacement for HonKit. Just install and run:

```bash
# Replace: npx honkit build
guidebook build

# Replace: npx honkit serve
guidebook serve
```

No configuration changes required.

## Security

- **Dev server**: Path traversal protection blocks `..` access and symlink escapes
- **Image downloads**: 50 MB size limit with streaming enforcement (aborts mid-download)
- **Self-update**: SHA256 checksum verification is mandatory; unverified binaries are refused

See [CONTRIBUTING.md](CONTRIBUTING.md#security-model) for details.

## Feedback

Found a bug? Have a feature request?

👉 [guidebook-feedback](https://github.com/guide-inc-org/guidebook-feedback)

You can write in English, Japanese, or Vietnamese.

## License

MIT

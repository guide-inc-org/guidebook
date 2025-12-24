# rustbook

A fast, HonKit/GitBook-compatible static site generator written in Rust.

## Overview

rustbook is a drop-in replacement for GitBook/HonKit that generates static documentation sites from Markdown files. It's designed to be fast, reliable, and fully compatible with existing GitBook/HonKit projects.

## Quick Start

```bash
# Clone the repository
git clone https://github.com/guide-inc-org/rustbook.git
cd rustbook/rustbook

# Build
cargo build --release

# Generate a book
./target/release/rustbook build /path/to/your/book -o /path/to/output
```

## Features

- **GitBook/HonKit Compatible** - Works with existing `book.json`, `SUMMARY.md`, and `LANGS.md`
- **Fast** - Built with Rust for maximum performance
- **Multi-language Support** - Build books in multiple languages
- **Mermaid Diagrams** - Native support for Mermaid diagram rendering
- **Collapsible Chapters** - Sidebar with expandable/collapsible navigation
- **SPA Navigation** - Smooth page transitions without full page reloads
- **Japanese Support** - Proper handling of CJK characters in heading IDs and anchors
- **Tolerant Parsing** - Handles common Markdown mistakes (e.g., full-width spaces in headings)

## Project Structure

```
rustbook/
├── README.md           # This file
├── rustbook/           # Main Rust application
│   ├── Cargo.toml
│   ├── src/            # Source code
│   └── templates/      # HTML/CSS/JS templates
└── .gitignore
```

## Documentation

See [rustbook/README.md](rustbook/README.md) for detailed documentation.

## License

MIT

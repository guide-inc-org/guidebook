# guidebook

A fast, HonKit/GitBook-compatible static site generator written in Rust.

Your existing GitBook/HonKit books build unchanged — same `SUMMARY.md`, `book.json`, `GLOSSARY.md` and `LANGS.md` — with a single binary instead of a Node.js toolchain.

[![CI](https://github.com/guide-inc-org/guidebook/actions/workflows/ci.yml/badge.svg)](https://github.com/guide-inc-org/guidebook/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/guide-inc-org/guidebook)](https://github.com/guide-inc-org/guidebook/releases/latest)
[![Crates.io](https://img.shields.io/crates/v/guidebook.svg)](https://crates.io/crates/guidebook)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Feedback](https://img.shields.io/badge/Feedback-Issues%20%26%20Requests-blue)](https://github.com/guide-inc-org/guidebook-feedback)

📖 **[Documentation](https://guide-inc-org.github.io/guidebook/)** — built with guidebook itself (English / 日本語 / Tiếng Việt)

## Why guidebook?

|  | guidebook | HonKit | mdBook |
|---|---|---|---|
| Runtime | single binary | Node.js + npm | single binary |
| Existing GitBook books build unchanged | ✅ | ✅ | ❌ (different format) |
| `book.json` variables + Nunjucks templates | ✅ | ✅ | ❌ |
| `GLOSSARY.md` term tooltips | ✅ | ✅ | ❌ |
| Multilingual books (`LANGS.md`) | ✅ | ✅ | ❌ |
| Mermaid diagrams | built-in | plugin | preprocessor |
| AsciiDoc pages | ✅ | ❌ | ❌ |
| Math | KaTeX built-in | plugin | MathJax |

**Speed** (this repo's 22-page Japanese docs, Apple Silicon, wall-clock): guidebook v0.1.70 builds in **0.02 s**, HonKit takes **2.1 s** on the same book (`npx honkit build`, warm cache). The full trilingual 66-page site builds in 0.04 s.

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

## 日本語

guidebook は HonKit/GitBook 互換の静的サイトジェネレーターです。既存の GitBook 形式のブック（`SUMMARY.md` / `book.json` / `GLOSSARY.md` / `LANGS.md`）を書式変換なしでそのままビルドできます。Node.js 環境は不要で、単一バイナリをインストールするだけです。多言語ブック・Mermaid 図・用語集ツールチップ・全文検索・数式（KaTeX）に対応し、日本語の見出し・アンカー・用語集も正しく扱えます。社内の大量の設計書ブックを日々ビルドする実運用の中で開発されています。

## License

MIT

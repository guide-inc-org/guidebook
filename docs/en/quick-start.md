# Quick Start

## Initialize a New Book

The fastest way to start:

```bash
guidebook init my-book
```

This creates:

```
my-book/
├── book.json       # Configuration
├── README.md       # Introduction (becomes index.html)
└── SUMMARY.md      # Table of contents
```

Edit `SUMMARY.md` to add chapters, then start writing.

## Manual Setup

Alternatively, create the structure yourself:

```
my-book/
├── README.md       # Introduction (becomes index.html)
├── SUMMARY.md      # Table of contents
└── chapter1.md     # Your content
```

### README.md

```markdown
# My Book

Welcome to my book!
```

### SUMMARY.md

```markdown
# Summary

* [Introduction](README.md)
* [Chapter 1](chapter1.md)
```

### chapter1.md

```markdown
# Chapter 1

This is the first chapter.
```

## Preview Your Book

```bash
cd my-book
guidebook serve
```

Open http://localhost:4000 in your browser.

The server watches for file changes and automatically reloads.

## Build for Production

```bash
guidebook build -o _book
```

This creates a `_book` folder with static HTML files ready to deploy.

## Deploy

Upload the `_book` folder to any static hosting:

- GitHub Pages
- Netlify
- Vercel
- Cloudflare Pages
- Any web server

# Project Structure

## Basic Structure

```
my-book/
├── book.json       # Configuration (optional)
├── README.md       # Introduction
├── SUMMARY.md      # Table of contents
├── chapter1.md
├── chapter2/
│   ├── README.md   # Chapter 2 intro
│   ├── section1.md
│   └── section2.md
├── assets/
│   └── images/
└── styles/
    └── website.css # Custom styles
```

## Required Files

### SUMMARY.md

Defines the table of contents and navigation structure:

```markdown
# Summary

* [Introduction](README.md)
* [Getting Started](getting-started.md)
* [Advanced Topics](advanced/README.md)
  * [Topic 1](advanced/topic1.md)
  * [Topic 2](advanced/topic2.md)
```

### README.md

The introduction page, becomes `index.html`.

## Optional Files

### book.json

Configuration file. See [Configuration](config.md).

### GLOSSARY.md

Define terms that are automatically linked throughout your book:

```markdown
## API
Application Programming Interface

## HTML
HyperText Markup Language
```

Terms appear as tooltips when hovered. See [Glossary](features/glossary.md).

### LANGS.md

For multi-language books:

```markdown
# Languages

* [English](en/)
* [日本語](ja/)
```

## Assets

Place images and other assets in an `assets/` folder:

```
![My Image](assets/images/screenshot.png)
```

Relative paths are also supported:

```
![My Image](../assets/images/screenshot.png)
```

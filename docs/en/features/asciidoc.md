# AsciiDoc Support

guidebook supports AsciiDoc (`.adoc` and `.asciidoc`) files alongside Markdown.

## Usage

Reference `.adoc` files in your `SUMMARY.md` just like Markdown files:

```markdown
# Summary

* [Introduction](README.md)
* [AsciiDoc Chapter](chapter.adoc)
```

## Supported Syntax

guidebook renders AsciiDoc with its built-in renderer, supporting:

- Headings (`= Title`, `== Section`, etc.)
- Paragraphs and text formatting (`*bold*`, `_italic_`, `` `code` ``)
- Lists (ordered and unordered)
- Code blocks
- Links and cross-references
- Tables
- Admonitions (NOTE, TIP, WARNING, IMPORTANT, CAUTION)

## Mixing Formats

You can freely mix Markdown and AsciiDoc files in the same book. Each file is rendered with the appropriate parser based on its extension.

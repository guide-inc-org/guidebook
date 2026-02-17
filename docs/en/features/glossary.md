# Glossary

Create a `GLOSSARY.md` file in your book's root to define terms. These terms are automatically detected and displayed as tooltips throughout your book.

## Format

```markdown
## API
Application Programming Interface

## HTML
HyperText Markup Language

## SSR
Server-Side Rendering
```

Each term starts with `## Term` followed by its definition on the next line.

## How It Works

When guidebook builds your book, it scans all pages for glossary terms and wraps them in tooltip spans:

```html
<span class="glossary-term" data-definition="Application Programming Interface">API</span>
```

Hovering over a term shows its definition.

## Exclusions

Terms are **not** replaced inside:

- Code blocks (`` ` `` and ` ``` `)
- Links (`<a>` tags)
- Headings (`<h1>` through `<h6>`)
- Script tags
- Elements with `class="no-glossary"`

To prevent replacement in a specific element, add the `no-glossary` class:

```html
<span class="no-glossary">API</span>
```

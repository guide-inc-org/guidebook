# SVG Processing

guidebook provides two SVG processing modes controlled via `book.json`.

## Externalize Inline SVGs

Extract inline `<svg>` elements from HTML into separate `.svg` files:

```json
{
    "externalize_svg": true
}
```

Inline SVGs are replaced with `<img>` tags pointing to the extracted files. Icon SVGs (those with `fill="currentColor"`) are kept inline to preserve dynamic color behavior.

## Inline SVG Files

Embed external SVG files directly into the HTML:

```json
{
    "inline_svg": true
}
```

External `<img src="*.svg">` references are replaced with the SVG content inlined in the HTML. Icon SVGs remain as `<img>` tags.

## When to Use

| Option | Use Case |
|--------|----------|
| `externalize_svg` | Reduce HTML size, enable browser caching of SVGs |
| `inline_svg` | Eliminate extra HTTP requests, enable CSS styling of SVGs |

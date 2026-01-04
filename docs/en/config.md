# Configuration

Configuration is optional. guidebook works out of the box without any configuration.

## book.json

Create a `book.json` file in your book's root directory:

```json
{
    "title": "My Book",
    "description": "A description of my book",
    "author": "Your Name",
    "plugins": [
        "collapsible-chapters",
        "back-to-top-button",
        "mermaid-md-adoc"
    ],
    "styles": {
        "website": "styles/website.css"
    }
}
```

## Options

| Option | Description | Default |
|--------|-------------|---------|
| `title` | Book title | `"My Book"` |
| `description` | Book description | `""` |
| `author` | Author name | `""` |
| `plugins` | Enabled plugins | See below |
| `styles.website` | Custom CSS file | `null` |

## Default Plugins

These plugins are enabled by default (no configuration needed):

- `collapsible-chapters` - Collapsible sidebar navigation
- `back-to-top-button` - Back to top button
- `mermaid-md-adoc` - Mermaid diagram support

To disable a default plugin, prefix it with `-`:

```json
{
    "plugins": ["-mermaid-md-adoc"]
}
```

## Custom Styles

Create a CSS file and reference it in `book.json`:

```css
/* styles/website.css */
.book {
    font-family: "Noto Sans", sans-serif;
}

.markdown-section h2 {
    border-left: 4px solid #007acc;
    padding-left: 10px;
}
```

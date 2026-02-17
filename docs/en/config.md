# Configuration

Configuration is optional. guidebook works out of the box without any configuration.

## book.json

Create a `book.json` file in your book's root directory:

```json
{
    "title": "My Book",
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
| `plugins` | Enabled plugins | See below |
| `styles.website` | Custom CSS file | `null` |
| `variables` | User-defined variables for `{{ book.xxx }}` | `{}` |
| `hardbreaks` | Treat single newlines as `<br>` | `false` |
| `math` | Enable KaTeX math rendering | `false` |
| `externalize_svg` | Externalize inline SVGs to separate files | `false` |
| `inline_svg` | Inline SVG files into HTML | `false` |
| `fetchRemoteImages` | Download remote images at build time | `false` |
| `openapi` | OpenAPI/Swagger UI specification file | `null` |

## Default Plugins

These plugins are enabled by default (no configuration needed):

- `collapsible-chapters` - Collapsible sidebar navigation
- `back-to-top-button` - Back to top button
- `mermaid-md-adoc` - Mermaid diagram support
- `fontsettings` - Font size and theme controls (white/sepia/night)

To disable a default plugin, prefix it with `-`:

```json
{
    "plugins": ["-mermaid-md-adoc"]
}
```

## Variables

Define custom variables and use them in your Markdown files with Nunjucks/Jinja2 syntax:

```json
{
    "variables": {
        "version": "1.0.0",
        "appName": "My App"
    }
}
```

In your Markdown:

```markdown
Current version: {{ book.version }}

Welcome to {{ book.appName }}!
```

Variables inside code blocks (`` ` `` and ` ``` `) are not expanded.

## Math (KaTeX)

Enable math rendering:

```json
{
    "math": true
}
```

Use `$...$` for inline math and `$$...$$` for display math.

## Remote Image Downloading

Download remote HTTPS images at build time for offline access:

```json
{
    "fetchRemoteImages": true
}
```

Images are cached in `_remote_images/` with CRC32-based filenames. Maximum download size is 50 MB per image.

## OpenAPI / Swagger UI

Generate Swagger UI from an OpenAPI spec:

```json
{
    "openapi": "swagger.json"
}
```

For multiple APIs:

```json
{
    "openapi": {
        "api-docs": "swagger/v1.json",
        "admin-api": "swagger/admin.json"
    }
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

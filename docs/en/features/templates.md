# Templates

guidebook supports Nunjucks/Jinja2 template syntax in Markdown files, powered by the Tera template engine.

## Variables

Define variables in `book.json` and use them in Markdown:

```json
{
    "variables": {
        "version": "2.0.0",
        "appName": "My App"
    }
}
```

```markdown
Current version: {{ book.version }}

Welcome to {{ book.appName }}!
```

## Conditionals

```markdown
{% if book.version %}
Version: {{ book.version }}
{% else %}
Version not set
{% endif %}
```

`{% elif %}` is also supported.

## Loops

```markdown
{% for item in book.features %}
- {{ item }}
{% endfor %}
```

The `loop.index` variable is available inside loops (1-based).

## Filters

| Filter | Description |
|--------|-------------|
| `upper` | Convert to uppercase |
| `lower` | Convert to lowercase |
| `capitalize` | Capitalize first letter |
| `length` | Get the length |
| `default(value="fallback")` | Provide a fallback value |

Example:

```markdown
{{ book.appName | upper }}
{{ book.subtitle | default(value="No subtitle") }}
```

## Code Block Protection

Template syntax inside code blocks (`` ` `` and ` ``` `) is **not** processed. This lets you document template syntax without it being evaluated.

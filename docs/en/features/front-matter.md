# Front Matter

Add YAML front matter to your Markdown files to set page-level metadata.

## Syntax

```markdown
---
title: Custom Page Title
description: A brief description of this page
---

# Page Content

Your content here.
```

## Supported Fields

| Field | Description |
|-------|-------------|
| `title` | Overrides the page title (used in `<title>` and navigation) |
| `description` | Page description (used in `<meta>` tag) |

## Behavior

- Front matter is stripped from the rendered output
- If `title` is set, it overrides the first `# Heading` in the file
- Empty front matter (`---\n---`) is valid and simply ignored
- Invalid YAML is silently ignored (page renders normally)

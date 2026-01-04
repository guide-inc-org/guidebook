# Collapsible Chapters

The sidebar navigation supports collapsible chapters for nested content.

## How It Works

Chapters with sub-items can be expanded or collapsed by clicking the arrow icon.

```markdown
# Summary

* [Introduction](README.md)
* [Chapter 1](chapter1/README.md)
  * [Section 1.1](chapter1/section1.md)
  * [Section 1.2](chapter1/section2.md)
* [Chapter 2](chapter2.md)
```

In this example, "Chapter 1" will have an expand/collapse arrow.

## State Persistence

The sidebar state is saved in `localStorage`, so expanded/collapsed chapters persist across page reloads.

## Disable Collapsible Chapters

```json
{
    "plugins": ["-collapsible-chapters"]
}
```

# Math (KaTeX)

Render mathematical formulas using KaTeX.

## Enable

Set `math: true` in `book.json`:

```json
{
    "math": true
}
```

## Inline Math

Use `$...$` for inline formulas:

```markdown
The equation $E = mc^2$ is famous.
```

## Display Math

Use `$$...$$` for display (block) formulas:

```markdown
$$
\int_0^\infty e^{-x^2} dx = \frac{\sqrt{\pi}}{2}
$$
```

## KaTeX Reference

For supported functions and syntax, see the [KaTeX documentation](https://katex.org/docs/supported).

# Import Files

Include content from other files using the `@import` directive.

## Syntax

```html
<!-- @import("path/to/file.md") -->
```

The directive is replaced with the contents of the referenced file during build.

## Examples

Include a shared header:

```html
<!-- @import("shared/header.md") -->
```

Include a code file:

```html
<!-- @import("examples/config.json") -->
```

## Recursive Imports

Imported files can contain their own `@import` directives. guidebook handles these recursively and detects circular imports to prevent infinite loops.

## Path Resolution

Paths are relative to the file containing the `@import` directive.

# OpenAPI / Swagger UI

Generate interactive Swagger UI pages from OpenAPI specifications.

## Single API

```json
{
    "openapi": "swagger.json"
}
```

This generates a Swagger UI page at `api-docs.html`.

## Multiple APIs

```json
{
    "openapi": {
        "api-docs": "swagger/v1.json",
        "admin-api": "swagger/admin.json"
    }
}
```

Each key becomes a separate HTML page (e.g., `api-docs.html`, `admin-api.html`).

## Specification Format

Both JSON and YAML OpenAPI specifications are supported. The spec file path is relative to the book's root directory.

# OpenAPI / Swagger UI

OpenAPI 仕様からインタラクティブな Swagger UI ページを生成します。

## 単一 API

```json
{
    "openapi": "swagger.json"
}
```

`api-docs.html` に Swagger UI ページが生成されます。

## 複数 API

```json
{
    "openapi": {
        "api-docs": "swagger/v1.json",
        "admin-api": "swagger/admin.json"
    }
}
```

各キーが個別の HTML ページになります（例：`api-docs.html`、`admin-api.html`）。

## 仕様ファイル形式

JSON と YAML の OpenAPI 仕様をサポートしています。仕様ファイルのパスはブックのルートディレクトリからの相対パスです。

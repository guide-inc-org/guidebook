# OpenAPI / Swagger UI

Tạo trang Swagger UI tương tác từ đặc tả OpenAPI.

## API đơn

```json
{
    "openapi": "swagger.json"
}
```

Tạo trang Swagger UI tại `api-docs.html`.

## Nhiều API

```json
{
    "openapi": {
        "api-docs": "swagger/v1.json",
        "admin-api": "swagger/admin.json"
    }
}
```

Mỗi key trở thành một trang HTML riêng (ví dụ: `api-docs.html`, `admin-api.html`).

## Định dạng đặc tả

Hỗ trợ cả đặc tả OpenAPI dạng JSON và YAML. Đường dẫn file đặc tả tương đối so với thư mục gốc của sách.

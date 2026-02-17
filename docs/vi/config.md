# Cấu hình

Cấu hình là tùy chọn. guidebook hoạt động mà không cần cấu hình.

## book.json

Tạo file `book.json` trong thư mục gốc của sách:

```json
{
    "title": "Sách của tôi",
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

## Tùy chọn

| Tùy chọn | Mô tả | Mặc định |
|----------|-------|----------|
| `title` | Tiêu đề sách | `"My Book"` |
| `plugins` | Plugin được bật | Xem bên dưới |
| `styles.website` | File CSS tùy chỉnh | `null` |
| `variables` | Biến do người dùng định nghĩa (`{{ book.xxx }}`) | `{}` |
| `hardbreaks` | Coi xuống dòng đơn là `<br>` | `false` |
| `math` | Bật rendering công thức KaTeX | `false` |
| `externalize_svg` | Tách SVG inline thành file riêng | `false` |
| `inline_svg` | Nhúng file SVG vào HTML | `false` |
| `fetchRemoteImages` | Tải hình ảnh remote khi build | `false` |
| `openapi` | File đặc tả OpenAPI/Swagger UI | `null` |

## Plugin mặc định

Các plugin sau được bật mặc định (không cần cấu hình):

- `collapsible-chapters` - Sidebar có thể thu gọn
- `back-to-top-button` - Nút quay lại đầu trang
- `mermaid-md-adoc` - Hỗ trợ biểu đồ Mermaid
- `fontsettings` - Điều chỉnh font và theme (trắng/sepia/tối)

Để tắt plugin mặc định, thêm tiền tố `-`:

```json
{
    "plugins": ["-mermaid-md-adoc"]
}
```

## Biến

Định nghĩa biến tùy chỉnh và sử dụng trong Markdown với cú pháp Nunjucks/Jinja2:

```json
{
    "variables": {
        "version": "1.0.0",
        "appName": "My App"
    }
}
```

Trong Markdown:

```markdown
Phiên bản hiện tại: {{ book.version }}

Chào mừng đến {{ book.appName }}!
```

Biến trong code block (`` ` `` và ` ``` `) không được mở rộng.

## Công thức (KaTeX)

Bật rendering công thức:

```json
{
    "math": true
}
```

Dùng `$...$` cho công thức inline và `$$...$$` cho công thức display.

## Tải hình ảnh remote

Tải hình ảnh HTTPS remote khi build để xem offline:

```json
{
    "fetchRemoteImages": true
}
```

Hình ảnh được cache trong `_remote_images/` với tên file dựa trên CRC32. Kích thước tối đa 50 MB mỗi hình.

## OpenAPI / Swagger UI

Tạo Swagger UI từ đặc tả OpenAPI:

```json
{
    "openapi": "swagger.json"
}
```

Cho nhiều API:

```json
{
    "openapi": {
        "api-docs": "swagger/v1.json",
        "admin-api": "swagger/admin.json"
    }
}
```

## CSS tùy chỉnh

Tạo file CSS và tham chiếu trong `book.json`:

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

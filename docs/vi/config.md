# Cấu hình

Cấu hình là tùy chọn. guidebook hoạt động mà không cần cấu hình.

## book.json

Tạo file `book.json` trong thư mục gốc của sách:

```json
{
    "title": "Sách của tôi",
    "description": "Mô tả sách",
    "author": "Tên tác giả",
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
| `description` | Mô tả sách | `""` |
| `author` | Tên tác giả | `""` |
| `plugins` | Plugin được bật | Xem bên dưới |
| `styles.website` | File CSS tùy chỉnh | `null` |

## Plugin mặc định

Các plugin sau được bật mặc định (không cần cấu hình):

- `collapsible-chapters` - Sidebar có thể thu gọn
- `back-to-top-button` - Nút quay lại đầu trang
- `mermaid-md-adoc` - Hỗ trợ biểu đồ Mermaid

Để tắt plugin mặc định, thêm tiền tố `-`:

```json
{
    "plugins": ["-mermaid-md-adoc"]
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

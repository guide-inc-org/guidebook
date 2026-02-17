# Template

guidebook hỗ trợ cú pháp template Nunjucks/Jinja2 trong file Markdown, sử dụng engine Tera.

## Biến

Định nghĩa biến trong `book.json` và sử dụng trong Markdown:

```json
{
    "variables": {
        "version": "2.0.0",
        "appName": "My App"
    }
}
```

```markdown
Phiên bản hiện tại: {{ book.version }}

Chào mừng đến {{ book.appName }}!
```

## Điều kiện

```markdown
{% if book.version %}
Phiên bản: {{ book.version }}
{% else %}
Chưa đặt phiên bản
{% endif %}
```

`{% elif %}` cũng được hỗ trợ.

## Vòng lặp

```markdown
{% for item in book.features %}
- {{ item }}
{% endfor %}
```

Biến `loop.index` có sẵn trong vòng lặp (bắt đầu từ 1).

## Filter

| Filter | Mô tả |
|--------|-------|
| `upper` | Chuyển thành chữ hoa |
| `lower` | Chuyển thành chữ thường |
| `capitalize` | Viết hoa chữ cái đầu |
| `length` | Lấy độ dài |
| `default(value="fallback")` | Cung cấp giá trị mặc định |

Ví dụ:

```markdown
{{ book.appName | upper }}
{{ book.subtitle | default(value="Không có phụ đề") }}
```

## Bảo vệ code block

Cú pháp template trong code block (`` ` `` và ` ``` `) **không** được xử lý. Điều này cho phép bạn viết tài liệu về cú pháp template mà không bị đánh giá.

# Bảng thuật ngữ

Tạo file `GLOSSARY.md` trong thư mục gốc của sách để định nghĩa thuật ngữ. Các thuật ngữ được tự động phát hiện và hiển thị dưới dạng tooltip trong toàn bộ sách.

## Định dạng

```markdown
## API
Application Programming Interface

## HTML
HyperText Markup Language

## SSR
Server-Side Rendering
```

Mỗi thuật ngữ bắt đầu bằng `## Thuật ngữ` theo sau là định nghĩa ở dòng tiếp theo.

## Cách hoạt động

Khi build, guidebook quét tất cả các trang để tìm thuật ngữ và bọc chúng trong span có tooltip:

```html
<span class="glossary-term" data-definition="Application Programming Interface">API</span>
```

Hover vào thuật ngữ sẽ hiển thị định nghĩa.

## Ngoại lệ

Thuật ngữ **không** được thay thế trong:

- Code block (`` ` `` và ` ``` `)
- Link (`<a>` tag)
- Heading (`<h1>` đến `<h6>`)
- Script tag
- Phần tử có `class="no-glossary"`

Để ngăn thay thế trong một phần tử cụ thể, thêm class `no-glossary`:

```html
<span class="no-glossary">API</span>
```

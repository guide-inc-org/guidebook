# Front Matter

Thêm YAML front matter vào file Markdown để thiết lập metadata cho trang.

## Cú pháp

```markdown
---
title: Tiêu đề trang tùy chỉnh
description: Mô tả ngắn về trang này
---

# Nội dung trang

Nội dung ở đây.
```

## Các trường được hỗ trợ

| Trường | Mô tả |
|--------|-------|
| `title` | Ghi đè tiêu đề trang (dùng trong `<title>` và navigation) |
| `description` | Mô tả trang (dùng trong `<meta>` tag) |

## Hành vi

- Front matter được loại bỏ khỏi output đã render
- Nếu `title` được đặt, nó ghi đè `# Heading` đầu tiên trong file
- Front matter rỗng (`---\n---`) hợp lệ và đơn giản bị bỏ qua
- YAML không hợp lệ được bỏ qua âm thầm (trang render bình thường)

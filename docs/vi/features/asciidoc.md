# Hỗ trợ AsciiDoc

guidebook hỗ trợ file AsciiDoc (`.adoc` và `.asciidoc`) cùng với Markdown.

## Sử dụng

Tham chiếu file `.adoc` trong `SUMMARY.md` giống như file Markdown:

```markdown
# Mục lục

* [Giới thiệu](README.md)
* [Chương AsciiDoc](chapter.adoc)
```

## Cú pháp được hỗ trợ

guidebook render AsciiDoc với renderer tích hợp, hỗ trợ:

- Heading (`= Tiêu đề`, `== Section`, v.v.)
- Đoạn văn và định dạng text (`*bold*`, `_italic_`, `` `code` ``)
- Danh sách (có thứ tự và không thứ tự)
- Code block
- Link và tham chiếu chéo
- Bảng
- Chú thích (NOTE, TIP, WARNING, IMPORTANT, CAUTION)

## Kết hợp định dạng

Bạn có thể tự do kết hợp file Markdown và AsciiDoc trong cùng một sách. Mỗi file được render với parser phù hợp dựa trên phần mở rộng.

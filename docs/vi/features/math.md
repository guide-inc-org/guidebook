# Công thức (KaTeX)

Render công thức toán học bằng KaTeX.

## Bật

Đặt `math: true` trong `book.json`:

```json
{
    "math": true
}
```

## Công thức inline

Dùng `$...$` cho công thức inline:

```markdown
Phương trình $E = mc^2$ rất nổi tiếng.
```

## Công thức display

Dùng `$$...$$` cho công thức display (block):

```markdown
$$
\int_0^\infty e^{-x^2} dx = \frac{\sqrt{\pi}}{2}
$$
```

## Tham khảo KaTeX

Xem [tài liệu KaTeX](https://katex.org/docs/supported) cho các hàm và cú pháp được hỗ trợ.

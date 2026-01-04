# Hỗ trợ đa ngôn ngữ

guidebook hỗ trợ build sách bằng nhiều ngôn ngữ.

## Thiết lập

Tạo file `LANGS.md` ở thư mục gốc của sách:

```markdown
# Languages

* [English](en/)
* [日本語](ja/)
* [Tiếng Việt](vi/)
```

Tổ chức nội dung:

```
my-book/
├── LANGS.md
├── book.json
├── en/
│   ├── README.md
│   ├── SUMMARY.md
│   └── ...
├── ja/
│   ├── README.md
│   ├── SUMMARY.md
│   └── ...
└── vi/
    ├── README.md
    ├── SUMMARY.md
    └── ...
```

## Bộ chọn ngôn ngữ

Bộ chọn ngôn ngữ xuất hiện ở góc trên bên trái, cho phép người đọc chuyển đổi ngôn ngữ.

## Mẹo

- Mỗi thư mục ngôn ngữ có `SUMMARY.md` riêng
- Bạn có thể có nội dung khác nhau cho mỗi ngôn ngữ
- Hình ảnh có thể được chia sẻ giữa các ngôn ngữ bằng đường dẫn tương đối

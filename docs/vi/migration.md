# Di chuyển từ HonKit

guidebook được thiết kế như một sự thay thế trực tiếp cho HonKit.

## Di chuyển nhanh

1. Cài đặt guidebook:
   ```bash
   curl -fsSL https://raw.githubusercontent.com/guide-inc-org/guidebook/main/install.sh | sh
   ```

2. Thay thế lệnh:
   ```bash
   # Trước (HonKit)
   npx honkit build
   npx honkit serve

   # Sau (guidebook)
   guidebook build
   guidebook serve
   ```

Vậy thôi! Không cần thay đổi cấu hình.

## Tương thích

### Được hỗ trợ

- Cấu hình `book.json`
- Định dạng `SUMMARY.md`
- Đa ngôn ngữ `LANGS.md`
- Render Markdown
- CSS tùy chỉnh
- Hầu hết các plugin (collapsible-chapters, back-to-top-button)

### Không được hỗ trợ

- Export PDF/EPUB (chỉ web)
- Plugin JavaScript (guidebook sử dụng implementation Rust tích hợp)
- Định dạng GitBook cũ

## Lợi ích khi chuyển đổi

| Tính năng | HonKit | guidebook |
|-----------|--------|-----------|
| Cài đặt | Node.js + npm | Binary duy nhất |
| Tốc độ build | Giây | Mili giây |
| Dependencies | Nhiều | Không có |
| Tự cập nhật | Thủ công | `guidebook update` |

## Di chuyển CI/CD

### Trước (HonKit)

```yaml
- uses: actions/setup-node@v4
- run: npm install -g honkit
- run: honkit build
```

### Sau (guidebook)

```yaml
- run: |
    curl -sL https://github.com/guide-inc-org/guidebook/releases/latest/download/guidebook-linux-x86_64.tar.gz | tar xz
    ./guidebook build
```

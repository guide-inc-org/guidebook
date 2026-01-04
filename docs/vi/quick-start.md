# Bắt đầu nhanh

## Tạo sách mới

Tạo thư mục với cấu trúc sau:

```
my-book/
├── README.md       # Giới thiệu (trở thành index.html)
├── SUMMARY.md      # Mục lục
└── chapter1.md     # Nội dung
```

### README.md

```markdown
# Sách của tôi

Chào mừng bạn!
```

### SUMMARY.md

```markdown
# Mục lục

* [Giới thiệu](README.md)
* [Chương 1](chapter1.md)
```

### chapter1.md

```markdown
# Chương 1

Đây là chương đầu tiên.
```

## Xem trước

```bash
cd my-book
guidebook serve
```

Mở http://localhost:4000 trong trình duyệt.

Server theo dõi thay đổi file và tự động reload.

## Build cho production

```bash
guidebook build -o _book
```

Tạo thư mục `_book` với các file HTML tĩnh sẵn sàng để deploy.

## Deploy

Upload thư mục `_book` lên bất kỳ hosting tĩnh nào:

- GitHub Pages
- Netlify
- Vercel
- Cloudflare Pages
- Bất kỳ web server nào

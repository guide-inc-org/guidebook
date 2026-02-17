# Tự cập nhật

Cập nhật guidebook lên phiên bản mới nhất trực tiếp từ CLI.

## Sử dụng

```bash
guidebook update
```

Tải phiên bản mới nhất từ GitHub và thay thế binary hiện tại.

## Cách hoạt động

1. Kiểm tra phiên bản mới nhất trên GitHub
2. So sánh với phiên bản hiện tại
3. Tải binary phù hợp cho platform (macOS, Linux, hoặc Windows)
4. Xác minh checksum SHA256 (nếu có trong release)
5. Thay thế binary hiện tại

## Hỗ trợ platform

| Platform | Định dạng archive |
|----------|------------------|
| macOS (Intel/ARM) | `.tar.gz` |
| Linux | `.tar.gz` |
| Windows | `.zip` |

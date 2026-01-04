# Chương có thể thu gọn

Sidebar navigation hỗ trợ thu gọn các chương có nội dung lồng nhau.

## Cách hoạt động

Các chương có sub-items có thể mở rộng hoặc thu gọn bằng cách click vào icon mũi tên.

```markdown
# Mục lục

* [Giới thiệu](README.md)
* [Chương 1](chapter1/README.md)
  * [Phần 1.1](chapter1/section1.md)
  * [Phần 1.2](chapter1/section2.md)
* [Chương 2](chapter2.md)
```

Trong ví dụ này, "Chương 1" sẽ có mũi tên mở rộng/thu gọn.

## Lưu trạng thái

Trạng thái sidebar được lưu trong `localStorage`, nên trạng thái mở rộng/thu gọn được giữ sau khi reload trang.

## Tắt chức năng thu gọn

```json
{
    "plugins": ["-collapsible-chapters"]
}
```

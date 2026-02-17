# Xử lý SVG

guidebook cung cấp hai chế độ xử lý SVG được điều khiển qua `book.json`.

## Tách SVG inline ra file riêng

Trích xuất phần tử `<svg>` inline từ HTML thành file `.svg` riêng:

```json
{
    "externalize_svg": true
}
```

SVG inline được thay thế bằng thẻ `<img>` trỏ đến file đã trích xuất. SVG icon (có `fill="currentColor"`) được giữ nguyên inline để duy trì hành vi màu động.

## Nhúng file SVG vào HTML

Nhúng file SVG bên ngoài trực tiếp vào HTML:

```json
{
    "inline_svg": true
}
```

Tham chiếu `<img src="*.svg">` bên ngoài được thay thế bằng nội dung SVG inline trong HTML. SVG icon vẫn giữ là thẻ `<img>`.

## Khi nào sử dụng

| Tùy chọn | Trường hợp sử dụng |
|----------|-------------------|
| `externalize_svg` | Giảm kích thước HTML, cho phép browser cache SVG |
| `inline_svg` | Loại bỏ request HTTP bổ sung, cho phép CSS styling SVG |

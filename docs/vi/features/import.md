# Import file

Nhúng nội dung từ file khác bằng directive `@import`.

## Cú pháp

```html
<!-- @import("path/to/file.md") -->
```

Directive được thay thế bằng nội dung của file được tham chiếu khi build.

## Ví dụ

Nhúng header chung:

```html
<!-- @import("shared/header.md") -->
```

Nhúng file code:

```html
<!-- @import("examples/config.json") -->
```

## Import đệ quy

File được import có thể chứa directive `@import` của riêng chúng. guidebook xử lý đệ quy và phát hiện import vòng tròn để ngăn vòng lặp vô hạn.

## Giải quyết đường dẫn

Đường dẫn tương đối so với file chứa directive `@import`.

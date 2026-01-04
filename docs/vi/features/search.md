# Tìm kiếm

guidebook có tính năng tìm kiếm toàn văn tích hợp sẵn.

## Cách hoạt động

Index tìm kiếm được tạo khi build. Người dùng có thể tìm kiếm qua ô tìm kiếm trong sidebar.

## Tính năng

- Tìm kiếm toàn văn trên tất cả các trang
- Kết quả hiển thị ngay khi gõ
- Điều hướng bằng bàn phím (phím mũi tên, Enter)
- Highlight nội dung khớp

## Search Index

Index tìm kiếm được lưu trong `search_index.json` ở thư mục output.

Với sách lớn, build lần đầu có thể mất vài giây để tạo index.

## Lưu ý khi Hot Reload

Trong quá trình phát triển (`guidebook serve`), search index không được tạo lại mỗi lần thay đổi để cải thiện tốc độ rebuild. Khởi động lại server để cập nhật search index.

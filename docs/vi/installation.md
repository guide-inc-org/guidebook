# Cài đặt

## Cài đặt nhanh (Khuyến nghị)

### macOS / Linux

```bash
curl -fsSL https://raw.githubusercontent.com/guide-inc-org/guidebook/main/install.sh | sh
```

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/guide-inc-org/guidebook/main/install.ps1 | iex
```

## Qua Cargo

Nếu bạn đã cài đặt Rust:

```bash
cargo install guidebook
```

## Xác nhận cài đặt

```bash
guidebook --version
```

## Cập nhật

Để cập nhật lên phiên bản mới nhất:

```bash
guidebook update
```

## Gỡ cài đặt

### Nếu cài qua script

```bash
rm ~/.local/bin/guidebook
# hoặc
rm /usr/local/bin/guidebook
```

### Nếu cài qua Cargo

```bash
cargo uninstall guidebook
```

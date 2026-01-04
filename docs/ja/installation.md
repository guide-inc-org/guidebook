# インストール

## クイックインストール（推奨）

### macOS / Linux

```bash
curl -fsSL https://raw.githubusercontent.com/guide-inc-org/guidebook/main/install.sh | sh
```

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/guide-inc-org/guidebook/main/install.ps1 | iex
```

## Cargo 経由

Rust がインストールされている場合：

```bash
cargo install guidebook
```

## インストール確認

```bash
guidebook --version
```

## アップデート

最新バージョンに更新：

```bash
guidebook update
```

## アンインストール

### インストールスクリプトでインストールした場合

```bash
rm ~/.local/bin/guidebook
# または
rm /usr/local/bin/guidebook
```

### Cargo でインストールした場合

```bash
cargo uninstall guidebook
```
